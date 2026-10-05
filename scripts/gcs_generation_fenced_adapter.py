#!/usr/bin/env python3
# Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
# SPDX-License-Identifier: AGPL-3.0-or-later

"""Minimal Cloud Storage adapter for real mutation-boundary qualification.

The adapter deliberately uses the Cloud Storage XML API for mutations because
its x-goog-if-generation-match header is evaluated by the service at mutation
time. Metadata and media are read through the JSON API.

This is qualification tooling, not a general-purpose storage client.
"""

from __future__ import annotations

import hashlib
import http.client
import json
import subprocess
from dataclasses import dataclass
from typing import Callable, Mapping
from urllib.parse import quote


HOST = "storage.googleapis.com"
META_PREFIX = "x-goog-meta-sol-atlas-"


@dataclass(frozen=True)
class ObjectState:
    generation: int
    metageneration: int
    data_sha256: str
    metadata: Mapping[str, str]
    size: int


@dataclass(frozen=True)
class HttpResult:
    status: int
    headers: Mapping[str, str]
    body: bytes


@dataclass(frozen=True)
class MutationRequest:
    execution_id: str
    input_fingerprint: str
    attempt_id: str
    fence_generation: int
    idempotency_key: str
    body: bytes

    def metadata(self) -> dict[str, str]:
        return {
            "execution-id": self.execution_id,
            "input-fingerprint": self.input_fingerprint,
            "attempt-id": self.attempt_id,
            "fence-generation": str(self.fence_generation),
            "idempotency-key": self.idempotency_key,
        }


def sha256_prefixed(data: bytes) -> str:
    return "sha256:" + hashlib.sha256(data).hexdigest()


def access_token() -> str:
    token = subprocess.run(
        ["gcloud", "auth", "print-access-token"],
        check=True,
        capture_output=True,
        text=True,
        timeout=30,
    ).stdout.strip()
    if not token:
        raise RuntimeError("gcloud returned an empty access token")
    return token


class GcsGenerationFencedObject:
    def __init__(self, bucket: str, object_name: str, token: str) -> None:
        if not bucket or not object_name or not token:
            raise ValueError("bucket, object_name, and token are required")
        self.bucket = bucket
        self.object_name = object_name
        self.token = token

    def _connection(self) -> http.client.HTTPSConnection:
        return http.client.HTTPSConnection(HOST, timeout=30)

    def _json_path(self) -> str:
        return (
            "/storage/v1/b/"
            + quote(self.bucket, safe="")
            + "/o/"
            + quote(self.object_name, safe="")
        )

    def _xml_path(self) -> str:
        return "/" + quote(self.bucket, safe="") + "/" + quote(
            self.object_name, safe="/"
        )

    def _request(
        self,
        method: str,
        path: str,
        *,
        body: bytes = b"",
        headers: Mapping[str, str] | None = None,
    ) -> HttpResult:
        request_headers = {
            "Authorization": "Bearer " + self.token,
            "Connection": "close",
        }
        request_headers.update(headers or {})
        connection = self._connection()
        try:
            connection.request(method, path, body=body, headers=request_headers)
            response = connection.getresponse()
            payload = response.read()
            return HttpResult(response.status, dict(response.headers), payload)
        finally:
            connection.close()

    def metadata(self) -> ObjectState | None:
        result = self._request("GET", self._json_path())
        if result.status == 404:
            return None
        if result.status != 200:
            raise RuntimeError(
                f"GCS metadata GET failed: HTTP {result.status}: "
                f"{result.body[:500]!r}"
            )
        document = json.loads(result.body)
        metadata = {
            str(key).lower(): str(value)
            for key, value in document.get("metadata", {}).items()
        }
        return ObjectState(
            generation=int(document["generation"]),
            metageneration=int(document["metageneration"]),
            data_sha256="",
            metadata=metadata,
            size=int(document.get("size", "0")),
        )

    def data(
        self,
        generation: int | None = None,
        metageneration: int | None = None,
    ) -> bytes:
        path = self._json_path() + "?alt=media"
        query = []
        if generation is not None:
            query.append("ifGenerationMatch=" + str(generation))
        if metageneration is not None:
            query.append("ifMetagenerationMatch=" + str(metageneration))
        if query:
            path += "&" + "&".join(query)
        result = self._request("GET", path)
        if result.status != 200:
            raise RuntimeError(
                f"GCS media GET failed: HTTP {result.status}: "
                f"{result.body[:500]!r}"
            )
        return result.body

    def update_metadata(
        self,
        generation: int,
        metageneration: int,
        updates: Mapping[str, str],
    ) -> HttpResult:
        if generation <= 0 or metageneration <= 0 or not updates:
            raise ValueError("generation, metageneration, and updates are required")
        metadata = {
            key.removeprefix(META_PREFIX): value
            for key, value in updates.items()
        }
        path = (
            self._json_path()
            + "?ifGenerationMatch="
            + str(generation)
            + "&ifMetagenerationMatch="
            + str(metageneration)
        )
        return self._request(
            "PATCH",
            path,
            body=json.dumps({"metadata": metadata}).encode("utf-8"),
            headers={"Content-Type": "application/json"},
        )

    def state(
        self,
        between_metadata_and_data: Callable[[ObjectState], None] | None = None,
    ) -> ObjectState | None:
        hook = between_metadata_and_data
        for _attempt in range(3):
            metadata = self.metadata()
            if metadata is None:
                return None
            observed = ObjectState(
                generation=metadata.generation,
                metageneration=metadata.metageneration,
                data_sha256="",
                metadata=metadata.metadata,
                size=metadata.size,
            )
            if hook is not None:
                hook(observed)
                hook = None
            try:
                body = self.data(
                    metadata.generation,
                    metadata.metageneration,
                )
            except RuntimeError as error:
                if "HTTP 412" not in str(error):
                    raise
                continue
            return ObjectState(
                generation=metadata.generation,
                metageneration=metadata.metageneration,
                data_sha256=sha256_prefixed(body),
                metadata=metadata.metadata,
                size=len(body),
            )
        raise RuntimeError("GCS object changed during bounded read-back")

    def raw_put(
        self,
        request: MutationRequest,
        *,
        discard_response: bool = False,
    ) -> HttpResult:
        headers = {
            "Content-Type": "application/octet-stream",
            "x-goog-if-generation-match": str(request.fence_generation),
            "Content-Length": str(len(request.body)),
        }
        headers.update(
            {
                META_PREFIX + key: value
                for key, value in request.metadata().items()
            }
        )
        if not discard_response:
            return self._request(
                "PUT",
                self._xml_path(),
                body=request.body,
                headers=headers,
            )

        connection = self._connection()
        try:
            connection.request(
                "PUT",
                self._xml_path(),
                body=request.body,
                headers={
                    "Authorization": "Bearer " + self.token,
                    "Connection": "close",
                    **headers,
                },
            )
            # The request bytes have been sent; deliberately do not read the
            # response. Closing now models a lost completion acknowledgement.
        finally:
            connection.close()
        return HttpResult(0, {}, b"")

    def delete(self, generation: int) -> HttpResult:
        return self._request(
            "DELETE",
            self._xml_path(),
            headers={"x-goog-if-generation-match": str(generation)},
        )

    def reconcile(self, request: MutationRequest) -> str:
        state = self.state()
        if state is None:
            return "ObservedNotApplied"

        metadata = state.metadata
        exact = (
            metadata.get("execution-id") == request.execution_id
            and metadata.get("input-fingerprint") == request.input_fingerprint
            and metadata.get("attempt-id") == request.attempt_id
            and metadata.get("fence-generation") == str(request.fence_generation)
            and metadata.get("idempotency-key") == request.idempotency_key
            and state.data_sha256 == sha256_prefixed(request.body)
        )
        if exact:
            return "ObservedAppliedSameRequest"

        if metadata.get("idempotency-key") == request.idempotency_key:
            return "ObservedDifferentRequest"

        if metadata.get("execution-id") == request.execution_id:
            return "ObservedDifferentRequest"

        return "ObservedDifferentRequest"

    def apply(self, request: MutationRequest) -> str:
        state = self.state()
        if state is not None:
            metadata = state.metadata
            same_request = (
                metadata.get("execution-id") == request.execution_id
                and metadata.get("input-fingerprint") == request.input_fingerprint
                and metadata.get("attempt-id") == request.attempt_id
                and metadata.get("fence-generation") == str(request.fence_generation)
                and metadata.get("idempotency-key") == request.idempotency_key
                and state.data_sha256 == sha256_prefixed(request.body)
            )
            if same_request:
                return "AlreadyAppliedSameRequest"

            same_key = metadata.get("idempotency-key") == request.idempotency_key
            same_execution = metadata.get("execution-id") == request.execution_id
            if same_key or same_execution:
                return "RejectedIdentityMismatch"

            if request.fence_generation < state.generation:
                return "RejectedStaleFence"
            if request.fence_generation > state.generation:
                return "RejectedFutureFence"

        result = self.raw_put(request)
        if result.status == 412:
            state = self.state()
            if state is None:
                return "RejectedPrecondition"
            if request.fence_generation < state.generation:
                return "RejectedStaleFence"
            if request.fence_generation > state.generation:
                return "RejectedFutureFence"
            return "RejectedPrecondition"

        if result.status not in {200, 201}:
            raise RuntimeError(
                f"GCS PUT failed: HTTP {result.status}: "
                f"{result.body[:500]!r}"
            )

        observed = self.reconcile(request)
        if observed != "ObservedAppliedSameRequest":
            raise RuntimeError(
                "GCS mutation succeeded but exact read-back did not match"
            )
        return "Applied"
