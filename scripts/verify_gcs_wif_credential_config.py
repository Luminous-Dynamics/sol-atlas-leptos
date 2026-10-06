#!/usr/bin/env python3
# Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
# SPDX-License-Identifier: AGPL-3.0-or-later

"""Verify a file-sourced Google WIF credential configuration."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path

SCHEMA = "sol-atlas:gcp-wif-credential-config-verification:v1"
TOKEN_TYPE = "urn:ietf:params:oauth:token-type:jwt"
STS_URL = "https://sts.googleapis.com/v1/token"


def canonical(value: object) -> bytes:
    return json.dumps(
        value,
        sort_keys=True,
        separators=(",", ":"),
        ensure_ascii=True,
    ).encode("utf-8")


def digest(value: object) -> str:
    return "sha256:" + hashlib.sha256(canonical(value)).hexdigest()


def bytes_digest(path: Path) -> str:
    return "sha256:" + hashlib.sha256(path.read_bytes()).hexdigest()


def expected_audience(provider_resource: str) -> str:
    if not provider_resource.startswith("projects/"):
        raise AssertionError("WIF provider resource is not a full project resource")
    return "//iam.googleapis.com/" + provider_resource


def expected_impersonation_url(service_account: str) -> str:
    if (
        not service_account
        or service_account.count("@") != 1
        or service_account.endswith("@")
    ):
        raise AssertionError("service account identity is malformed")
    return (
        "https://iamcredentials.googleapis.com/v1/projects/-/serviceAccounts/"
        + service_account
        + ":generateAccessToken"
    )


def verify(
    config_path: str,
    token_path: str,
    oidc_claims_path: str,
    provider_resource: str,
    service_account: str,
    output: str | None,
) -> dict[str, object]:
    config_file = Path(config_path)
    token_file = Path(token_path)
    oidc_file = Path(oidc_claims_path)
    if not config_file.is_file():
        raise AssertionError("WIF credential configuration file is missing")
    if not token_file.is_file():
        raise AssertionError("verified OIDC token file is missing")
    if not oidc_file.is_file():
        raise AssertionError("OIDC evidence file is missing")

    config = json.loads(config_file.read_text(encoding="utf-8"))
    oidc = json.loads(oidc_file.read_text(encoding="utf-8"))
    if config.get("type") != "external_account":
        raise AssertionError("WIF credential config is not an external account")
    if oidc.get("schema") != "sol-atlas:github-oidc-claims:v4":
        raise AssertionError("wrong OIDC evidence schema")
    observed_claims = oidc.get("claims")
    if not isinstance(observed_claims, dict):
        raise AssertionError("OIDC evidence contains no claims")
    token_digest = oidc.get("token_digest")
    if not isinstance(token_digest, str):
        raise AssertionError("OIDC evidence contains no token digest")
    actual_token_digest = bytes_digest(token_file)
    if actual_token_digest != token_digest:
        raise AssertionError("WIF token file drifted from verified OIDC token")

    expected = {
        "audience": expected_audience(provider_resource),
        "subject_token_type": TOKEN_TYPE,
        "token_url": STS_URL,
        "credential_source": {"file": str(token_file)},
        "service_account_impersonation_url": (
            expected_impersonation_url(service_account)
        ),
    }
    for name, expected_value in expected.items():
        if config.get(name) != expected_value:
            raise AssertionError(
                f"WIF credential config drift for {name}: "
                f"{config.get(name)!r} != {expected_value!r}"
            )

    result: dict[str, object] = {
        "schema": SCHEMA,
        "credential_config_digest": bytes_digest(config_file),
        "token_digest": actual_token_digest,
        "provider_resource": provider_resource,
        "credential_audience": config["audience"],
        "subject_token_type": config["subject_token_type"],
        "token_url": config["token_url"],
        "credential_source_file": config["credential_source"]["file"],
        "service_account": service_account,
        "service_account_impersonation_url": (
            config["service_account_impersonation_url"]
        ),
        "exact_verified_token_bound": True,
    }
    if output:
        destination = Path(output)
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_text(
            json.dumps(result, indent=2, sort_keys=True) + "\n",
            encoding="utf-8",
        )
    return result


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--config", required=True)
    parser.add_argument("--token-file", required=True)
    parser.add_argument("--oidc-claims", required=True)
    parser.add_argument("--provider-resource", required=True)
    parser.add_argument("--service-account", required=True)
    parser.add_argument("--output")
    args = parser.parse_args()
    result = verify(
        args.config,
        args.token_file,
        args.oidc_claims,
        args.provider_resource,
        args.service_account,
        args.output,
    )
    print(
        "verified exact WIF credential binding: "
        + result["credential_config_digest"]
        + " "
        + result["token_digest"]
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
