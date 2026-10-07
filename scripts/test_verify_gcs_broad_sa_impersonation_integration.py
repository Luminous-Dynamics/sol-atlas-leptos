#!/usr/bin/env python3
# Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
# SPDX-License-Identifier: AGPL-3.0-or-later

"""Stubbed integration tests for broad service-account impersonation."""

from __future__ import annotations

import importlib.util
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "verify_gcs_broad_sa_impersonation.py"

spec = importlib.util.spec_from_file_location(
    "broad_sa_integration",
    SCRIPT,
)
assert spec and spec.loader
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)

PRINCIPAL = (
    "principalSet://iam.googleapis.com/projects/123456789/"
    "locations/global/workloadIdentityPools/github/"
    "attribute.repository_id/1195997641"
)
QUALIFICATION = (
    "//iam.googleapis.com/projects/sol-atlas/serviceAccounts/"
    "qualification@sol-atlas.iam.gserviceaccount.com"
)
OIDC_SUBJECT = (
    "repo:Luminous-Dynamics@216969177/sol-atlas-leptos@1195997641:"
    "environment:sol-atlas-gcs-qualification"
)

OTHER = (
    "//iam.googleapis.com/projects/sol-atlas/serviceAccounts/"
    "privileged@sol-atlas.iam.gserviceaccount.com"
)


def envelope(result):
    return {
        "fullyExplored": True,
        "nonCriticalErrors": [],
        "analysisResults": [result],
    }


def intended():
    return {
        "fullyExplored": True,
        "attachedResourceFullName": QUALIFICATION,
        "iamBinding": {
            "role": "roles/iam.workloadIdentityUser",
            "members": [PRINCIPAL],
        },
        "identityList": {"identities": [{"name": PRINCIPAL}]},
        "accessControlLists": [
            {
                "resources": [{"fullResourceName": QUALIFICATION}],
                "accesses": [
                    {"permission": "iam.serviceAccounts.getAccessToken"},
                    {"permission": "iam.serviceAccounts.getOpenIdToken"},
                ],
            }
        ],
    }


def forbidden():
    result = intended()
    result["attachedResourceFullName"] = (
        "//cloudresourcemanager.googleapis.com/projects/sol-atlas"
    )
    result["iamBinding"] = {
        "role": "roles/iam.serviceAccountUser",
        "members": [PRINCIPAL],
    }
    result["accessControlLists"] = [
        {
            "resources": [{"fullResourceName": OTHER}],
            "accesses": [
                {"permission": "iam.serviceAccounts.actAs"}
            ],
        }
    ]
    return result


def main() -> None:
    original = module.run_analysis
    original_subject = module.load_immutable_oidc_subject
    module.load_immutable_oidc_subject = lambda path: OIDC_SUBJECT
    try:
        module.run_analysis = lambda scope: envelope(intended())
        result = module.verify(
            "projects/sol-atlas",
            "sol-atlas",
            "qualification@sol-atlas.iam.gserviceaccount.com",
            PRINCIPAL,
            "unused-oidc.json",
            None,
        )
        assert result["broad_impersonation_absent"] is True
        assert result["wif_profile_digest"].startswith("sha256:")

        module.run_analysis = lambda scope: envelope(forbidden())
        try:
            module.verify(
                "projects/sol-atlas",
                "sol-atlas",
                "qualification@sol-atlas.iam.gserviceaccount.com",
                PRINCIPAL,
                "unused-oidc.json",
                None,
            )
        except AssertionError:
            pass
        else:
            raise AssertionError(
                "broad service-account pivot was accepted"
            )
    finally:
        module.run_analysis = original
        module.load_immutable_oidc_subject = original_subject

    print("broad service-account integration checks: PASS")


if __name__ == "__main__":
    main()