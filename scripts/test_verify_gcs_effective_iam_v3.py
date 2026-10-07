#!/usr/bin/env python3
# Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
# SPDX-License-Identifier: AGPL-3.0-or-later

"""Regression tests for execution/observer IAM separation."""

from __future__ import annotations

import importlib.util
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "verify_gcs_effective_iam.py"

spec = importlib.util.spec_from_file_location("effective_iam_v3", SCRIPT)
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


def binding(role: str, permissions: list[str]) -> dict[str, object]:
    return {
        "attachedResourceFullName": (
            "//cloudresourcemanager.googleapis.com/projects/sol-atlas"
        ),
        "iamBinding": {
            "role": role,
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
                    for permission in permissions
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


def expect_failure(result: dict[str, object], label: str) -> None:
    try:
        module.extract_findings(envelope(result), PRINCIPAL, RESOURCE)
    except AssertionError:
        return
    raise AssertionError("accepted forbidden execution path: " + label)


def main() -> None:
    normal = binding(
        module.EXPECTED_ROLE,
        list(module.REQUIRED_PERMISSIONS),
    )
    finding = module.extract_findings(
        envelope(normal),
        PRINCIPAL,
        RESOURCE,
    )[0]
    assert module.required_permissions_are_present(
        finding["permissions"]
    )
    assert module.execution_permissions_are_clean(
        finding["permissions"]
    )
    assert not module.execution_permissions_are_clean(
        ["iam.serviceAccounts.getIamPolicy"]
    )

    expect_failure(
        binding(
            "roles/iam.serviceAccountViewer",
            ["iam.serviceAccounts.getIamPolicy"],
        ),
        "service-account viewer",
    )
    expect_failure(
        binding(
            "roles/iam.serviceAccountAdmin",
            ["iam.serviceAccounts.setIamPolicy"],
        ),
        "service-account admin",
    )

    print("execution/observer IAM separation checks: PASS")


if __name__ == "__main__":
    main()
