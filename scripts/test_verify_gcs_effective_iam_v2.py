#!/usr/bin/env python3
# Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
# SPDX-License-Identifier: AGPL-3.0-or-later

"""Regression tests for effective IAM audit v2."""

from __future__ import annotations

import importlib.util
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "verify_gcs_effective_iam.py"

spec = importlib.util.spec_from_file_location("effective_iam_v2", SCRIPT)
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


def result(
    role: str = module.EXPECTED_ROLE,
    attached: str = (
        "//cloudresourcemanager.googleapis.com/projects/sol-atlas"
    ),
    members: list[str] | None = None,
    identities: list[str] | None = None,
    permissions: list[str] | None = None,
    condition: str | None = None,
) -> dict[str, object]:
    acl: dict[str, object] = {
        "resources": [{"fullResourceName": RESOURCE}],
        "accesses": [
            {"permission": p}
            for p in permissions or module.REQUIRED_PERMISSIONS
        ],
    }
    if condition is not None:
        acl["conditionEvaluation"] = {"evaluationValue": condition}
    return {
        "attachedResourceFullName": attached,
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
        "accessControlLists": [acl],
        "fullyExplored": True,
    }


def payload(results: list[dict[str, object]], errors=None, complete=True):
    return {
        "fullyExplored": complete,
        "nonCriticalErrors": list(errors or []),
        "analysisResults": results,
    }


def must_fail(data, label: str) -> None:
    try:
        module.extract_findings(data, PRINCIPAL, RESOURCE)
    except AssertionError:
        return
    raise AssertionError("accepted invalid evidence: " + label)


def main() -> None:
    good = module.extract_findings(
        payload([result()]),
        PRINCIPAL,
        RESOURCE,
    )
    assert len(good) == 1
    assert good[0]["role"] == module.EXPECTED_ROLE
    assert module.required_permissions_are_present(
        good[0]["permissions"]
    )

    assert module.scope_flag("projects/sol-atlas") == (
        "--project",
        "sol-atlas",
    )
    assert module.scope_flag("folders/123") == (
        "--folder",
        "123",
    )
    assert module.scope_flag("organizations/456") == (
        "--organization",
        "456",
    )

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

    must_fail(
        payload([result(role="roles/iam.serviceAccountTokenCreator")]),
        "alternate role",
    )
    must_fail(
        payload([result(role="customRoles/alternate")]),
        "custom role",
    )
    must_fail(
        payload([
            result(
                members=[PRINCIPAL, "user:unexpected@example.com"]
            )
        ]),
        "additional member",
    )
    must_fail(
        payload([
            result(
                identities=[PRINCIPAL, "user:unexpected@example.com"]
            )
        ]),
        "additional identity",
    )
    must_fail(
        payload([
            result(
                attached=(
                    "//cloudresourcemanager.googleapis.com/projects/other"
                )
            )
        ]),
        "different project attachment",
    )
    must_fail(
        payload([result(condition="CONDITIONAL")]),
        "conditional access",
    )
    must_fail(
        payload([result(condition="FALSE")]),
        "false access",
    )
    must_fail(
        payload([result(permissions=[
            "iam.serviceAccounts.getAccessToken"
        ])]),
        "missing second WIF permission",
    )
    must_fail(
        payload([result()], complete=False),
        "incomplete top-level result",
    )
    must_fail(
        payload([result()], errors=[{"code": "PERMISSION_DENIED"}]),
        "non-critical error",
    )
    must_fail(
        payload([result(), result()]),
        "multiple binding paths",
    )
    must_fail(
        payload([{
            **result(),
            "accessControlLists": [
                {
                    "resources": [],
                    "accesses": [
                        {
                            "permission": (
                                "iam.serviceAccounts.getAccessToken"
                            )
                        }
                    ],
                }
            ],
        }]),
        "missing target resource",
    )

    print("effective IAM v2 semantic checks: PASS")


if __name__ == "__main__":
    main()
