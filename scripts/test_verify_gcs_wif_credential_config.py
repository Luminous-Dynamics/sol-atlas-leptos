#!/usr/bin/env python3
# Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
# SPDX-License-Identifier: AGPL-3.0-or-later

"""Offline semantic tests for the file-sourced WIF config verifier."""

from __future__ import annotations

import importlib.util
import json
from pathlib import Path
from tempfile import TemporaryDirectory

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "verify_gcs_wif_credential_config.py"

spec = importlib.util.spec_from_file_location(
    "verify_gcs_wif_credential_config",
    SCRIPT,
)
assert spec and spec.loader
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)

PROVIDER = (
    "projects/123456789/locations/global/"
    "workloadIdentityPools/sol-atlas/providers/github"
)
SERVICE_ACCOUNT = "sol-atlas@project.iam.gserviceaccount.com"


def expect_rejection(config: dict[str, object], path: Path, token: Path, oidc: Path) -> None:
    path.write_text(json.dumps(config), encoding="utf-8")
    try:
        module.verify(
            str(path),
            str(token),
            str(oidc),
            PROVIDER,
            SERVICE_ACCOUNT,
            None,
        )
    except AssertionError:
        pass
    else:
        raise AssertionError("tampered WIF credential configuration was accepted")


def main() -> None:
    with TemporaryDirectory() as tmp:
        root = Path(tmp)
        token = root / "oidc.jwt"
        config = root / "credentials.json"
        oidc = root / "oidc.json"
        token.write_text("header.payload.signature\n", encoding="ascii")
        token_digest = module.bytes_digest(token)
        oidc_document = {
            "schema": "sol-atlas:github-oidc-claims:v4",
            "claims": {"aud": "https://example.invalid"},
            "token_digest": token_digest,
        }
        oidc.write_text(json.dumps(oidc_document), encoding="utf-8")

        expected = {
            "type": "external_account",
            "audience": module.expected_audience(PROVIDER),
            "subject_token_type": module.TOKEN_TYPE,
            "token_url": module.STS_URL,
            "credential_source": {"file": str(token)},
            "service_account_impersonation_url": (
                module.expected_impersonation_url(SERVICE_ACCOUNT)
            ),
        }
        config.write_text(json.dumps(expected), encoding="utf-8")
        result = module.verify(
            str(config),
            str(token),
            str(oidc),
            PROVIDER,
            SERVICE_ACCOUNT,
            None,
        )
        assert result["exact_verified_token_bound"] is True
        assert result["token_digest"] == token_digest

        for field, value in (
            ("audience", "https://iam.googleapis.com/projects/tampered"),
            ("subject_token_type", "wrong-token-type"),
            ("token_url", "https://evil.example/token"),
            ("service_account_impersonation_url", "https://evil.example/sa"),
        ):
            mutated = dict(expected, **{field: value})
            expect_rejection(mutated, config, token, oidc)

        mutated_source = dict(
            expected,
            credential_source={"file": str(root / "other.jwt")},
        )
        expect_rejection(mutated_source, config, token, oidc)

        token.write_text("tampered-token\n", encoding="ascii")
        config.write_text(json.dumps(expected), encoding="utf-8")
        try:
            module.verify(
                str(config),
                str(token),
                str(oidc),
                PROVIDER,
                SERVICE_ACCOUNT,
                None,
            )
        except AssertionError:
            pass
        else:
            raise AssertionError("WIF config accepted a drifted token file")

    print("offline WIF credential-config verifier semantic checks: PASS")


if __name__ == "__main__":
    main()
