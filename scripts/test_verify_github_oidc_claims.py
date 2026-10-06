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
CONTEXT = {
    "GITHUB_WORKFLOW_SHA": "1" * 40,
    "GITHUB_SHA": "2" * 40,
    "GITHUB_RUN_ID": "12345",
    "GITHUB_RUN_ATTEMPT": "1",
}


def main() -> None:
    expected = module.expected_claims(AUDIENCE, CONTEXT)
    assert module.verify_claims(expected, AUDIENCE, CONTEXT) == expected

    for name in ("aud", "workflow_sha", "sha", "workflow_ref"):
        mutated = dict(expected)
        mutated[name] = "tampered"
        try:
            module.verify_claims(mutated, AUDIENCE, CONTEXT)
        except AssertionError:
            pass
        else:
            raise AssertionError(f"tampered {name} claim was accepted")

    try:
        module.decode_payload("not-a-jwt")
    except AssertionError:
        pass
    else:
        raise AssertionError("malformed JWT was accepted")

    print("offline GitHub OIDC verifier semantic checks: PASS")


if __name__ == "__main__":
    main()
