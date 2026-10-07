#!/usr/bin/env python3
# Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
# SPDX-License-Identifier: AGPL-3.0-or-later

"""Offline semantic tests for the GitHub OIDC claim verifier."""

from __future__ import annotations

import importlib.util
import json
from pathlib import Path
from tempfile import TemporaryDirectory

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "verify_github_oidc_claims.py"

spec = importlib.util.spec_from_file_location("verify_github_oidc_claims", SCRIPT)
assert spec and spec.loader
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)

QUALIFIER = ROOT / "scripts" / "qualify_gcs_external_effect.py"
qualifier_spec = importlib.util.spec_from_file_location(
    "qualify_gcs_external_effect",
    QUALIFIER,
)
assert qualifier_spec and qualifier_spec.loader
qualifier = importlib.util.module_from_spec(qualifier_spec)
qualifier_spec.loader.exec_module(qualifier)

AUDIENCE = (
    "https://iam.googleapis.com/projects/123/locations/global/"
    "workloadIdentityPools/sol-atlas/providers/github"
)
NOW = 1_750_000_000.0
CONTEXT = {
    "GITHUB_WORKFLOW_SHA": "1" * 40,
    "GITHUB_SHA": "2" * 40,
    "GITHUB_RUN_ID": "12345",
    "GITHUB_RUN_ATTEMPT": "1",
    "RUNNER_ENVIRONMENT": "github-hosted",
}


def expect_rejection(payload: dict[str, object], message: str) -> None:
    try:
        module.verify_claims(
            payload,
            AUDIENCE,
            CONTEXT,
            now=NOW,
        )
    except AssertionError:
        pass
    else:
        raise AssertionError(message)


def main() -> None:
    with TemporaryDirectory() as tmp:
        token_path = Path(tmp) / "verified.jwt"
        token = "header.payload.signature"
        module.write_token(token, str(token_path))
        assert token_path.read_bytes() == token.encode("ascii")
        assert oct(token_path.stat().st_mode & 0o777) == "0o600"

    expected = module.expected_claims(AUDIENCE, CONTEXT)
    expected["sub"] = (
        "repo:Luminous-Dynamics@216969177/"
        "sol-atlas-leptos@1195997641:"
        "ref:refs/heads/main"
    )
    expected.update(
        {
            "iat": NOW - 10,
            "exp": NOW + 100,
            "nbf": NOW - 10,
        }
    )
    assert module.verify_claims(
        expected,
        AUDIENCE,
        CONTEXT,
        now=NOW,
    ) == expected

    expect_rejection(
        dict(expected, sub="repo:Luminous-Dynamics/sol-atlas:environment:tampered"),
        "mutable OIDC subject was accepted",
    )

for name in (
        "aud",
        "workflow_sha",
        "sha",
        "workflow_ref",
        "runner_environment",
    ):
        mutated = dict(expected)
        mutated[name] = "tampered"
        expect_rejection(
            mutated,
            f"tampered {name} claim was accepted",
        )

    expired = dict(expected, exp=NOW - 61)
    expect_rejection(expired, "expired OIDC token was accepted")

    future_issued = dict(expected, iat=NOW + 61)
    expect_rejection(future_issued, "future-issued OIDC token was accepted")

    not_yet_valid = dict(expected, nbf=NOW + 61)
    expect_rejection(not_yet_valid, "not-yet-valid OIDC token was accepted")

    inconsistent_lifetime = dict(
        expected,
        iat=NOW + 10,
        exp=NOW + 5,
    )
    expect_rejection(
        inconsistent_lifetime,
        "OIDC token with exp <= iat was accepted",
    )

    inverted_validity = dict(
        expected,
        nbf=NOW + 50,
        exp=NOW + 40,
    )
    expect_rejection(
        inverted_validity,
        "OIDC token with nbf > exp was accepted",
    )

    skew_boundary = dict(
        expected,
        exp=NOW - module.OIDC_CLOCK_SKEW_SECONDS,
    )
    expect_rejection(
        skew_boundary,
        "OIDC token at the expiration skew boundary was accepted",
    )

    non_numeric = dict(expected, exp="not-a-number")
    expect_rejection(
        non_numeric,
        "OIDC token with non-numeric exp was accepted",
    )

    oidc_artifact = {
        "schema": module.SCHEMA,
        "audience": AUDIENCE,
        "claims": expected,
        "temporal_claims_valid": True,
        "verified_at_unix": NOW,
        "verified_at": "2025-06-15T15:06:40+00:00",
        "clock_skew_seconds": module.OIDC_CLOCK_SKEW_SECONDS,
        "cryptographic_verification": "not_performed_locally",
        "wif_exchange_uses_this_exact_verified_token": True,
        "claims_digest": qualifier.digest(expected),
    }
    with TemporaryDirectory() as tmp:
        artifact_path = Path(tmp) / "oidc.json"
        artifact_path.write_text(
            json.dumps(oidc_artifact),
            encoding="utf-8",
        )
        assert qualifier.load_oidc_claims(str(artifact_path)) == oidc_artifact

        for field, value in (
            (
                "cryptographic_verification",
                "delegated_to_gcp_wif_exchange",
            ),
            (
                "wif_exchange_uses_this_exact_verified_token",
                False,
            ),
            ("audience", "https://iam.googleapis.com/projects/tampered"),
        ):
            tampered = dict(oidc_artifact, **{field: value})
            artifact_path.write_text(
                json.dumps(tampered),
                encoding="utf-8",
            )
            try:
                qualifier.load_oidc_claims(str(artifact_path))
            except AssertionError:
                pass
            else:
                raise AssertionError(
                    f"report loader accepted tampered OIDC {field} evidence"
                )

    qualifier.verify_oidc_temporal_evidence(oidc_artifact)
    qualifier.verify_oidc_claim_identity(
        expected,
        {
            "workflow_sha": CONTEXT["GITHUB_WORKFLOW_SHA"],
            "sha": CONTEXT["GITHUB_SHA"],
            "run_id": CONTEXT["GITHUB_RUN_ID"],
            "run_attempt": CONTEXT["GITHUB_RUN_ATTEMPT"],
            "runner_environment": CONTEXT["RUNNER_ENVIRONMENT"],
        },
        AUDIENCE,
    )

    for name in (
        "repository",
        "repository_id",
        "repository_owner_id",
        "environment",
        "event_name",
        "workflow",
        "ref",
        "ref_type",
        "workflow_ref",
        "run_id",
        "run_attempt",
        "runner_environment",
    ):
        mutated = dict(expected, **{name: "tampered"})
        try:
            qualifier.verify_oidc_claim_identity(
                mutated,
                {
                    "workflow_sha": CONTEXT["GITHUB_WORKFLOW_SHA"],
                    "sha": CONTEXT["GITHUB_SHA"],
                    "run_id": CONTEXT["GITHUB_RUN_ID"],
                    "run_attempt": CONTEXT["GITHUB_RUN_ATTEMPT"],
                    "runner_environment": CONTEXT["RUNNER_ENVIRONMENT"],
                },
                AUDIENCE,
            )
        except AssertionError:
            pass
        else:
            raise AssertionError(
                f"report verifier accepted tampered OIDC {name} claim"
            )

    expired_artifact = dict(
        oidc_artifact,
        claims=dict(expected, exp=NOW - 61),
        claims_digest=qualifier.digest(
            dict(expected, exp=NOW - 61),
        ),
    )
    try:
        qualifier.verify_oidc_temporal_evidence(expired_artifact)
    except AssertionError:
        pass
    else:
        raise AssertionError("report verifier accepted expired OIDC evidence")

    wrong_clock_skew = dict(
        oidc_artifact,
        clock_skew_seconds=61,
    )
    try:
        qualifier.verify_oidc_temporal_evidence(wrong_clock_skew)
    except AssertionError:
        pass
    else:
        raise AssertionError("report verifier accepted a widened clock-skew policy")

    wrong_timestamp = dict(
        oidc_artifact,
        verified_at_unix=NOW + 1,
    )
    try:
        qualifier.verify_oidc_temporal_evidence(wrong_timestamp)
    except AssertionError:
        pass
    else:
        raise AssertionError("report verifier accepted inconsistent timestamps")

    naive_timestamp = dict(
        oidc_artifact,
        verified_at="2025-06-15T15:06:40",
    )
    try:
        qualifier.verify_oidc_temporal_evidence(naive_timestamp)
    except AssertionError:
        pass
    else:
        raise AssertionError("report verifier accepted a naive timestamp")

    malformed = "not-a-jwt"
    try:
        module.decode_payload(malformed)
    except AssertionError:
        pass
    else:
        raise AssertionError("malformed JWT was accepted")

    valid_request_url = (
        "https://token.actions.githubusercontent.com/"
        "?token=runner-token"
    )
    module.validate_oidc_request_url(valid_request_url)

    rewritten = module.with_audience(
        "https://token.actions.githubusercontent.com/?audience=wrong&token=runner",
        AUDIENCE,
    )
    assert rewritten.count("audience=") == 1
    assert f"audience={AUDIENCE}" in rewritten

    for url in (
        "http://token.actions.githubusercontent.com/?token=runner-token",
        "https://evil.example/?token=runner-token",
        "https://user:pass@token.actions.githubusercontent.com/?token=runner-token",
        "https://token.actions.githubusercontent.com:444/?token=runner-token",
        "https://token.actions.githubusercontent.com:invalid/?token=runner-token",
        "https://token.actions.githubusercontent.com/#fragment",
    ):
        try:
            module.validate_oidc_request_url(url)
        except AssertionError:
            pass
        else:
            raise AssertionError(f"unapproved OIDC request URL was accepted: {url}")

    print("offline GitHub OIDC verifier semantic checks: PASS")


if __name__ == "__main__":
    main()
