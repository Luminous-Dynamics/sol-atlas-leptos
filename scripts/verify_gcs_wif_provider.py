#!/usr/bin/env python3
# Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
# SPDX-License-Identifier: AGPL-3.0-or-later

"""Fail-closed verifier for the trusted GCS GitHub OIDC provider configuration."""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
from pathlib import Path


PROFILE_PATH = (
    "sol-atlas-policy-store-contract/conformance/"
    "gcs_wif_trust_profile_v1.json"
)


def canonical(value: object) -> bytes:
    return json.dumps(
        value,
        sort_keys=True,
        separators=(",", ":"),
        ensure_ascii=True,
    ).encode("utf-8")


def digest(value: object) -> str:
    return "sha256:" + hashlib.sha256(canonical(value)).hexdigest()


def load_profile() -> dict[str, object]:
    profile = json.loads(Path(PROFILE_PATH).read_text(encoding="utf-8"))
    if profile.get("schema") != "sol-atlas:gcs-wif-trust-profile:v1":
        raise AssertionError("wrong WIF trust profile schema")
    if not profile.get("required_condition_terms"):
        raise AssertionError("WIF trust profile has no required conditions")
    return profile


def provider_parts(resource: str) -> tuple[str, str, str]:
    prefix = "projects/"
    parts = resource.split("/")
    if len(parts) != 8 or parts[0] != "projects" or parts[2] != "locations":
        raise AssertionError("invalid workload identity provider resource")
    if parts[4] != "workloadIdentityPools" or parts[6] != "providers":
        raise AssertionError("invalid workload identity provider resource")
    return parts[1], parts[5], parts[7]


def describe_provider(resource: str) -> dict[str, object]:
    _, pool_id, provider_id = provider_parts(resource)
    result = subprocess.run(
        [
            "gcloud",
            "iam",
            "workload-identity-pools",
            "providers",
            "describe",
            provider_id,
            "--location=global",
            f"--workload-identity-pool={pool_id}",
            "--format=json",
        ],
        check=True,
        capture_output=True,
        text=True,
        timeout=30,
    )
    return json.loads(result.stdout)


def verify(provider_resource: str) -> dict[str, object]:
    profile = load_profile()
    provider = describe_provider(provider_resource)
    mappings = provider.get("attributeMapping") or {}
    condition = str(provider.get("attributeCondition") or "")
    required_mappings = profile["required_attribute_mappings"]

    for name, expected in required_mappings.items():
        if mappings.get(name) != expected:
            raise AssertionError(
                f"attribute mapping drift: {name!r} != {expected!r}"
            )

    for term in profile["required_condition_terms"]:
        if term not in condition:
            raise AssertionError(
                f"attribute condition is missing required term: {term}"
            )

    provider_digest = digest(
        {
            "name": provider.get("name"),
            "attributeMapping": mappings,
            "attributeCondition": condition,
            "issuerUri": provider.get("issuerUri"),
            "attributeConditionProfile": profile,
        }
    )
    return {
        "schema": "sol-atlas:gcs-wif-trust-verification:v1",
        "provider_resource": provider_resource,
        "provider_digest": provider_digest,
        "profile_path": PROFILE_PATH,
        "profile_digest": digest(profile),
        "attribute_mapping_verified": True,
        "attribute_condition_verified": True,
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--provider-resource", required=True)
    parser.add_argument("--output")
    args = parser.parse_args()

    result = verify(args.provider_resource)
    payload = json.dumps(result, indent=2, sort_keys=True) + "\n"
    if args.output:
        Path(args.output).parent.mkdir(parents=True, exist_ok=True)
        Path(args.output).write_text(payload, encoding="utf-8")
    print(payload, end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
