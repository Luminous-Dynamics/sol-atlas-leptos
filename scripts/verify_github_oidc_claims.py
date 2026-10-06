#!/usr/bin/env python3
# Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
# SPDX-License-Identifier: AGPL-3.0-or-later

"""Fail-closed verifier for the GitHub OIDC claims used by GCS qualification."""

from __future__ import annotations

import argparse
import base64
import hashlib
import json
import math
import os
import re
import time
import urllib.parse
import urllib.request
from datetime import datetime, timezone
from pathlib import Path

SCHEMA = "sol-atlas:github-oidc-claims:v4"
ISSUER = "https://token.actions.githubusercontent.com"
OIDC_REQUEST_HOST = "token.actions.githubusercontent.com"
REPO = "Luminous-Dynamics/sol-atlas-leptos"
REPOSITORY_ID = "1195997641"
REPOSITORY_OWNER_ID = "216969177"
ENVIRONMENT = "sol-atlas-gcs-qualification"
WORKFLOW = "Qualify GCS external effect"
EVENT = "workflow_dispatch"
REF = "refs/heads/main"
WORKFLOW_REF = (
    "Luminous-Dynamics/sol-atlas-leptos/.github/workflows/"
    "qualify-gcs.yml@refs/heads/main"
)
HEX40 = re.compile(r"^[0-9a-f]{40}$")
DECIMAL = re.compile(r"^[0-9]+$")
OIDC_CLOCK_SKEW_SECONDS = 60


def canonical(value: object) -> bytes:
    return json.dumps(
        value,
        sort_keys=True,
        separators=(",", ":"),
        ensure_ascii=True,
    ).encode("utf-8")


def digest(value: object) -> str:
    return "sha256:" + hashlib.sha256(canonical(value)).hexdigest()


def decode_payload(token: str) -> dict[str, object]:
    parts = token.split(".")
    if len(parts) != 3 or any(not part for part in parts):
        raise AssertionError("OIDC token is not a compact JWT")
    payload_part = parts[1]
    payload_part += "=" * (-len(payload_part) % 4)
    try:
        payload = json.loads(
            base64.urlsafe_b64decode(payload_part.encode("ascii")).decode("utf-8")
        )
    except (ValueError, UnicodeDecodeError) as exc:
        raise AssertionError("OIDC JWT payload is not valid JSON") from exc
    if not isinstance(payload, dict):
        raise AssertionError("OIDC JWT payload is not an object")
    return payload


def runtime_context() -> dict[str, str]:
    names = [
        "GITHUB_WORKFLOW_SHA",
        "GITHUB_SHA",
        "GITHUB_RUN_ID",
        "GITHUB_RUN_ATTEMPT",
    ]
    values = {name: os.environ.get(name, "") for name in names}
    missing = [name for name, value in values.items() if not value]
    if missing:
        raise AssertionError(
            "missing runtime context: " + ", ".join(missing)
        )
    if not HEX40.fullmatch(values["GITHUB_WORKFLOW_SHA"]):
        raise AssertionError("GITHUB_WORKFLOW_SHA is not a lowercase 40-hex commit")
    if not HEX40.fullmatch(values["GITHUB_SHA"]):
        raise AssertionError("GITHUB_SHA is not a lowercase 40-hex commit")
    if not DECIMAL.fullmatch(values["GITHUB_RUN_ID"]):
        raise AssertionError("GITHUB_RUN_ID is not a decimal run identifier")
    if not DECIMAL.fullmatch(values["GITHUB_RUN_ATTEMPT"]):
        raise AssertionError("GITHUB_RUN_ATTEMPT is not a decimal attempt identifier")
    return values


def expected_claims(audience: str, context: dict[str, str]) -> dict[str, str]:
    return {
        "iss": ISSUER,
        "aud": audience,
        "repository": REPO,
        "repository_id": REPOSITORY_ID,
        "repository_owner_id": REPOSITORY_OWNER_ID,
        "environment": ENVIRONMENT,
        "event_name": EVENT,
        "workflow": WORKFLOW,
        "ref": REF,
        "ref_type": "branch",
        "workflow_ref": WORKFLOW_REF,
        "workflow_sha": context["GITHUB_WORKFLOW_SHA"],
        "sha": context["GITHUB_SHA"],
        "run_id": context["GITHUB_RUN_ID"],
        "run_attempt": context["GITHUB_RUN_ATTEMPT"],
    }


def numeric_date(payload: dict[str, object], name: str) -> float:
    value = payload.get(name)
    if (
        isinstance(value, bool)
        or not isinstance(value, (int, float))
        or not math.isfinite(float(value))
    ):
        raise AssertionError(f"OIDC claim {name!r} is missing or not a finite number")
    return float(value)


def verify_temporal_claims(
    payload: dict[str, object],
    *,
    now: float,
    clock_skew: float = OIDC_CLOCK_SKEW_SECONDS,
) -> dict[str, object]:
    if not math.isfinite(now) or not math.isfinite(clock_skew) or clock_skew < 0:
        raise AssertionError("OIDC verifier clock parameters are invalid")
    issued_at = numeric_date(payload, "iat")
    expires_at = numeric_date(payload, "exp")
    not_before = numeric_date(payload, "nbf")
    if expires_at <= issued_at or not_before > expires_at:
        raise AssertionError("OIDC temporal claims have no valid interval")
    if issued_at > now + clock_skew:
        raise AssertionError("OIDC token is issued in the future")
    if not_before > now + clock_skew:
        raise AssertionError("OIDC token is not yet valid")
    if expires_at <= now - clock_skew:
        raise AssertionError("OIDC token is expired")
    return {
        "iat": payload["iat"],
        "exp": payload["exp"],
        "nbf": payload["nbf"],
    }


def verify_claims(
    payload: dict[str, object],
    audience: str,
    context: dict[str, str],
    *,
    now: float,
) -> dict[str, object]:
    expected = expected_claims(audience, context)
    observed: dict[str, object] = {}
    for name, expected_value in expected.items():
        value = payload.get(name)
        if not isinstance(value, str):
            raise AssertionError(f"OIDC claim {name!r} is missing or not a string")
        if value != expected_value:
            raise AssertionError(
                f"OIDC claim drift for {name}: {value!r} != {expected_value!r}"
            )
        observed[name] = value
    observed.update(verify_temporal_claims(payload, now=now))
    return observed


def validate_oidc_request_url(url: str) -> None:
    parsed = urllib.parse.urlsplit(url)
    try:
        port = parsed.port
    except ValueError as exc:
        raise AssertionError("GitHub OIDC request URL has an invalid port") from exc
    if (
        parsed.scheme != "https"
        or parsed.hostname != OIDC_REQUEST_HOST
        or port not in (None, 443)
        or parsed.username is not None
        or parsed.password is not None
        or parsed.fragment
    ):
        raise AssertionError("GitHub OIDC request URL is not an approved HTTPS origin")


def with_audience(url: str, audience: str) -> str:
    parsed = urllib.parse.urlsplit(url)
    query = [
        (name, value)
        for name, value in urllib.parse.parse_qsl(
            parsed.query,
            keep_blank_values=True,
        )
        if name != "audience"
    ]
    query.append(("audience", audience))
    return urllib.parse.urlunsplit(
        (
            parsed.scheme,
            parsed.netloc,
            parsed.path,
            urllib.parse.urlencode(query),
            parsed.fragment,
        )
    )


def request_token(audience: str) -> str:
    request_url = os.environ.get("ACTIONS_ID_TOKEN_REQUEST_URL", "")
    request_token_value = os.environ.get("ACTIONS_ID_TOKEN_REQUEST_TOKEN", "")
    if not request_url or not request_token_value:
        raise AssertionError("GitHub OIDC request environment is unavailable")
    validate_oidc_request_url(request_url)

    request = urllib.request.Request(
        with_audience(request_url, audience),
        headers={
            "Accept": "application/json",
            "Authorization": "Bearer " + request_token_value,
        },
    )
    try:
        with urllib.request.urlopen(request, timeout=30) as response:
            document = json.load(response)
    except Exception as exc:
        raise AssertionError("GitHub OIDC token request failed") from exc

    token = document.get("value") if isinstance(document, dict) else None
    if not isinstance(token, str) or not token:
        raise AssertionError("GitHub OIDC response contained no JWT")
    return token


def write_token(token: str, output: str) -> None:
    destination = Path(output)
    destination.parent.mkdir(parents=True, exist_ok=True)
    destination.write_text(token, encoding="ascii")
    destination.chmod(0o600)


def verify(
    audience: str,
    output: str | None,
    token_output: str | None,
) -> dict[str, object]:
    if not audience.startswith("https://iam.googleapis.com/projects/"):
        raise AssertionError("OIDC audience is not a Google provider resource")
    context = runtime_context()
    token = request_token(audience)
    verification_time = time.time()
    claims = verify_claims(
        decode_payload(token),
        audience,
        context,
        now=verification_time,
    )
    result: dict[str, object] = {
        "schema": SCHEMA,
        "audience": audience,
        "claims": claims,
        "claims_digest": digest(claims),
        "token_digest": "sha256:" + hashlib.sha256(
            token.encode("ascii"),
        ).hexdigest(),
        "temporal_claims_valid": True,
        "verified_at_unix": verification_time,
        "verified_at": datetime.fromtimestamp(
            verification_time, timezone.utc
        ).isoformat(),
        "clock_skew_seconds": OIDC_CLOCK_SKEW_SECONDS,
        "workflow_sha_matches_runner": (
            claims["workflow_sha"] == context["GITHUB_WORKFLOW_SHA"]
        ),
        "source_sha_matches_runner": claims["sha"] == context["GITHUB_SHA"],
        "cryptographic_verification": "not_performed_locally",
        "wif_exchange_uses_this_exact_verified_token": True,
    }
    if output:
        destination = Path(output)
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_text(
            json.dumps(result, indent=2, sort_keys=True) + "\n",
            encoding="utf-8",
        )
    if token_output:
        write_token(token, token_output)
    return result


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--audience", required=True)
    parser.add_argument("--output")
    parser.add_argument("--token-output")
    args = parser.parse_args()
    result = verify(args.audience, args.output, args.token_output)
    print(
        "verified GitHub OIDC claims: "
        + result["claims_digest"]
        + " "
        + args.audience
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
