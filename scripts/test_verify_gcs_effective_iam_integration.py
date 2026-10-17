#!/usr/bin/env python3
# Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
# SPDX-License-Identifier: AGPL-3.0-or-later

"""Stubbed integration tests for the effective-IAM audit control flow."""

from __future__ import annotations

import json
import importlib.util
from pathlib import Path
from tempfile import TemporaryDirectory

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "verify_gcs_effective_iam.py"

spec = importlib.util.spec_from_file_location("effective_iam_integration", SCRIPT)
assert spec and spec.loader
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)

PRINCIPAL = (
    "principalSet://iam.googleapis.com/projects/123456789/"
    "locations/global/workloadIdentityPools/github/"
    "attribute.repository_id/1195997641"
)
PROJECT = "sol-atlas"
SERVICE_ACCOUNT = "qualification@sol-atlas.iam.gserviceaccount.com"
RESOURCE = (
    "//iam.googleapis.com/projects/sol-atlas/serviceAccounts/"
    + SERVICE_ACCOUNT
)
OIDC_SUBJECT = (
    "repo:Luminous-Dynamics@216969177/sol-atlas-leptos@1195997641:"
    "environment:sol-atlas-gcs-qualification"
)



def service_account_payload(permissions=None):
    return {
        "fullyExplored": True,
        "nonCriticalErrors": [],
        "analysisResults": [
            {
                "fullyExplored": True,
                "attachedResourceFullName": RESOURCE,
                "iamBinding": {
                    "role": module.EXPECTED_ROLE,
                    "members": [PRINCIPAL],
                },
                "identityList": {
                    "identities": [{"name": PRINCIPAL}]
                },
                "accessControlLists": [
                    {
                        "resources": [{"fullResourceName": RESOURCE}],
                        "accesses": [
                            {"permission": permission}
                            for permission in (
                                permissions
                                or list(module.REQUIRED_PERMISSIONS)
                            )
                        ],
                    }
                ],
            }
        ],
    }


def empty_pivot_payload():
    return {
        "fullyExplored": True,
        "nonCriticalErrors": [],
        "analysisResults": [],
    }


def pivot_payload():
    return {
        "fullyExplored": True,
        "nonCriticalErrors": [],
        "analysisResults": [
            {
                "fullyExplored": True,
                "attachedResourceFullName": (
                    "//cloudresourcemanager.googleapis.com/projects/"
                    + PROJECT
                ),
                "iamBinding": {
                    "role": "roles/iam.serviceAccountUser",
                    "members": [PRINCIPAL],
                },
                "identityList": {
                    "identities": [{"name": PRINCIPAL}]
                },
                "accessControlLists": [
                    {
                        "resources": [
                            {
                                "fullResourceName": (
                                    "//cloudresourcemanager.googleapis.com/"
                                    "projects/" + PROJECT
                                )
                            }
                        ],
                        "accesses": [
                            {"permission": "iam.serviceAccounts.actAs"}
                        ],
                    }
                ],
            }
        ],
    }


def main() -> None:
    original_run = module.run_analysis
    original_pivot = module.run_project_pivot_analysis
    original_subject = module.load_immutable_oidc_subject
    try:
        module.run_analysis = lambda scope, resource: (
            service_account_payload()
        )
        module.run_project_pivot_analysis = lambda scope, resource: (
            empty_pivot_payload()
        )
        module.load_immutable_oidc_subject = lambda path: OIDC_SUBJECT
        with TemporaryDirectory() as tmp:
            output = str(Path(tmp) / "effective.json")
            result = module.verify(
                "projects/" + PROJECT,
                PROJECT,
                SERVICE_ACCOUNT,
                PRINCIPAL,
                "unused-oidc.json",
                output,
            )
            assert result["schema"] == module.SCHEMA
            assert result["project_pivot_permissions_absent"] is True
            assert result["project_pivot_response_digest"].startswith(
                "sha256:"
            )
            assert result["wif_profile_digest"].startswith("sha256:")
            written = json.loads(Path(output).read_text(encoding="utf-8"))
            assert written["schema"] == module.SCHEMA
            assert written["wif_profile_path"] == module.WIF_PROFILE_PATH

        module.run_project_pivot_analysis = lambda scope, resource: (
            pivot_payload()
        )
        try:
            module.verify(
                "projects/" + PROJECT,
                PROJECT,
                SERVICE_ACCOUNT,
                PRINCIPAL,
                "unused-oidc.json",
                None,
            )
        except AssertionError:
            pass
        else:
            raise AssertionError("active project pivot was accepted")
    finally:
        module.run_analysis = original_run
        module.run_project_pivot_analysis = original_pivot
        module.load_immutable_oidc_subject = original_subject

    print("effective IAM integration checks: PASS")


if __name__ == "__main__":
    main()
