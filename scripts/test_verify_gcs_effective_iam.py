#!/usr/bin/env python3
# Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
# SPDX-License-Identifier: AGPL-3.0-or-later

"""Baseline regression tests for effective IAM evidence."""

from __future__ import annotations

import importlib.util
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "verify_gcs_effective_iam.py"

spec = importlib.util.spec_from_file_location("effective_iam", SCRIPT)
assert spec and spec.loader
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)

PRINCIPAL = "principalSet://iam.googleapis.com/projects/123456789/locations/global/workloadIdentityPools/github/attribute.repository_id/1195997641"
RESOURCE = "//iam.googleapis.com/projects/sol-atlas/serviceAccounts/qualification@sol-atlas.iam.gserviceaccount.com"
OIDC_SUBJECT = "repo:Luminous-Dynamics@216969177/sol-atlas-leptos@1195997641:ref:refs/heads/main"


def finding(
    role: str = module.EXPECTED_ROLE,
    members: list[str] | None = None,
    identities: list[str] | None = None,
    permissions: list[str] | None = None,
) -> dict[str, object]:
    return {
        "attachedResourceFullName": (
            "//cloudresourcemanager.googleapis.com/projects/sol-atlas"
        ),
        "iamBinding": {
            "role": role,
            "members": members or [PRINCIPAL],
        },
        "identityList": {
            "identities": [
                {"name": value}
                for value in identities or [PRINCIPAL]
            ]
        },
        "accessControlLists": [
            {
                "resources": [{"fullResourceName": RESOURCE}],
                "accesses": [
                    {"permission": value}
                    for value in permissions
                    or list(module.REQUIRED_PERMISSIONS)
                ],
            }
        ],
        "fullyExplored": True,
    }


def envelope(results: list[dict[str, object]]) -> dict[str, object]:
    return {
        "fullyExplored": True,
        "nonCriticalErrors": [],
        "analysisResults": results,
    }


def main() -> None:
    result = module.extract_findings(
        envelope([finding()]),
        PRINCIPAL,
        RESOURCE,
        OIDC_SUBJECT,
    )
    assert len(result) == 1
    assert result[0]["role"] == module.EXPECTED_ROLE
    assert module.required_permissions_are_present(
        result[0]["permissions"]
    )
    assert module.execution_permissions_are_clean(
        result[0]["permissions"]
    )

    assert module.valid_attached_resource(RESOURCE, RESOURCE)
    assert module.valid_attached_resource(
        "//cloudresourcemanager.googleapis.com/projects/sol-atlas",
        RESOURCE,
    )
    assert module.valid_attached_resource(
        "//cloudresourcemanager.googleapis.com/folders/123",
        RESOURCE,
    )
    assert module.valid_attached_resource(
        "//cloudresourcemanager.googleapis.com/organizations/456",
        RESOURCE,
    )

    assert module.project_resource("sol-atlas").endswith(
        "/projects/sol-atlas"
    )
    print("baseline effective-IAM checks: PASS")


if __name__ == "__main__":
    main()
