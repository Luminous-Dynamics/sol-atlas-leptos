#!/usr/bin/env python3
# Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
# SPDX-License-Identifier: AGPL-3.0-or-later

"""Regression tests for execution/observer IAM separation."""

from __future__ import annotations

import importlib.util
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "verify_gcs_effective_iam.py"

spec = importlib.util.spec_from_file_location("effective_iam_sep", SCRIPT)
assert spec and spec.loader
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)

PRINCIPAL = (
    "principalSet://iam.googleapis.com/projects/123456789/"
    "locations/global/workloadIdentityPools/github/"
    "attribute.repository_id/1195997641"
)
RESOURCE = (
    "//iam.googleapis.com/projects/sol-atlas/serviceAccounts/"
    "qualification@sol-atlas.iam.gserviceaccount.com"
)

OIDC_SUBJECT = (
    "repo:Luminous-Dynamics@216969177/sol-atlas-leptos@1195997641:"
    "environment:sol-atlas-gcs-qualification"
)

def finding(permissions: list[str]) -> dict[str, object]:
    return {
        "attachedResourceFullName": (
            "//cloudresourcemanager.googleapis.com/projects/sol-atlas"
        ),
        "iamBinding": {
            "role": module.EXPECTED_ROLE,
            "members": [PRINCIPAL],
        },
        "identityList": {"identities": [{"name": PRINCIPAL}]},
        "accessControlLists": [
            {
                "resources": [{"fullResourceName": RESOURCE}],
                "accesses": [
                    {"permission": value}
                    for value in permissions
                ],
            }
        ],
        "fullyExplored": True,
    }


def envelope(result: dict[str, object]) -> dict[str, object]:
    return {
        "fullyExplored": True,
        "nonCriticalErrors": [],
        "analysisResults": [result],
    }


def main() -> None:
    expected_sets = module.expected_workload_principal_sets(PRINCIPAL)
    sorted_sets = sorted(expected_sets)
    assert PRINCIPAL in expected_sets
    assert any(member.endswith("/*") for member in sorted_sets)
    assert any(
        "/attribute.repository/" in member
        for member in sorted_sets
    )
    profile = json.loads(
        (ROOT / module.WIF_PROFILE_PATH).read_text(encoding="utf-8")
    )
    assert profile["schema"] == module.WIF_PROFILE_SCHEMA
    assert module.digest(profile) == module.digest(
        json.loads(
            (ROOT / module.WIF_PROFILE_PATH).read_text(encoding="utf-8")
        )
    )
    normal = module.extract_findings(
        envelope(finding(list(module.REQUIRED_PERMISSIONS))),
        PRINCIPAL,
        RESOURCE,
        OIDC_SUBJECT,
    )[0]
    assert module.execution_permissions_are_clean(
        normal["permissions"]
    )
    assert not module.execution_permissions_are_clean(
        ["iam.serviceAccounts.getIamPolicy"]
    )
    try:
        forbidden = module.extract_findings(
            envelope(
                finding(
                    [
                        "iam.serviceAccounts.getAccessToken",
                        "iam.serviceAccounts.getIamPolicy",
                    ]
                )
            ),
            PRINCIPAL,
            RESOURCE,
            OIDC_SUBJECT,
        )
        module.validate_permission_ceiling(forbidden)
    except AssertionError:
        pass
    else:
        raise AssertionError("observer policy permission was accepted")

    try:
        incomplete = module.extract_findings(
            envelope(
                finding(["iam.serviceAccounts.getAccessToken"])
            ),
            PRINCIPAL,
            RESOURCE,
            OIDC_SUBJECT,
        )
        module.validate_permission_ceiling(incomplete)
    except AssertionError:
        pass
    else:
        raise AssertionError("missing WIF permission was accepted")

    print("execution/observer separation checks: PASS")


if __name__ == "__main__":
    main()
