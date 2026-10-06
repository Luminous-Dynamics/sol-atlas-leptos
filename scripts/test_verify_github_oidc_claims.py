#!/usr/bin/env python3
# Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
# SPDX-License-Identifier: AGPL-3.0-or-later

"""Offline semantic tests for the GitHub OIDC claim verifier."""

from __future__ import annotations

import importlib.util
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "verify_github_oidc_claims.py"

spec = importlib.util.spec_from_file_location("verify_github_oidc_claims", SCRIPT)
assert spec and spec.loader
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)

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
    expected = module.expected_claims(AUDIENCE, CONTEXT)
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

    for name in ("aud", "workflow_sha", "sha", "workflow_ref"):
        mutated = dict(expected)
        mutated[name] = "tampered"
        expect_rejection(mutated, f"tampered {name} claim was accepted")

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
