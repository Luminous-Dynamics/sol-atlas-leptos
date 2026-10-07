#!/usr/bin/env python3
# Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
# SPDX-License-Identifier: AGPL-3.0-or-later

"""Offline semantic tests for the effective-IAM audit parser."""

from __future__ import annotations

import importlib.util
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "verify_gcs_effective_iam.py"

spec = importlib.util.spec_from_file_location("verify_gcs_effective_iam", SCRIPT)
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


def analysis(
    *,
    role: str = "roles/iam.workloadIdentityUser",
    members: list[str] | None = None,
    identities: list[str] | None = None,
    permissions: list[str] | None = None,
    attached: str = "//cloudresourcemanager.googleapis.com/projects/sol-atlas",
    fully_explored: bool = True,
) -> dict[str, object]:
    return {
        "attachedResourceFullName": attached,
        "iamBinding": {
            "role": role,
            "members": list(members or [PRINCIPAL]),
        },
        "accessControlLists": [
            {
                "resources": [{"fullResourceName": RESOURCE}],
                "accesses": [
                    {"permission": permission}
                    for permission in permissions
                    or list(module.REQUIRED_PERMISSIONS)
                ],
            }
        ],
        "identityList": {
            "identities": [
                {"name": identity}
                for identity in identities or [PRINCIPAL]
            ]
        },
        "fullyExplored": fully_explored,
    }


def reject(result: dict[str, object], name: str) -> None:
    try:
        module.extract_findings({"analysisResults": [result]}, PRINCIPAL, RESOURCE)
    except AssertionError:
        return
    raise AssertionError(f"tampered Policy Analyzer result was accepted: {name}")


def main() -> None:
    payload = {"analysisResults": [analysis()]}
    findings = module.extract_findings(payload, PRINCIPAL, RESOURCE)
    assert findings[0]["role"] == module.EXPECTED_ROLE
    assert module.REQUIRED_PERMISSIONS[0] in findings[0]["permissions"]
    result = module.verify  # keep module import and public API covered

    reject(
        analysis(role="roles/iam.serviceAccountTokenCreator"),
        "token creator role",
    )
    reject(
        analysis(role="customRoles/alternate"),
        "custom role",
    )
    reject(
        analysis(members=[PRINCIPAL, "group:unexpected@example.com"]),
        "additional member",
    )
    reject(
        analysis(identities=[PRINCIPAL, "user:unexpected@example.com"]),
        "additional identity",
    )
    reject(
        analysis(fully_explored=False),
        "not fully explored",
    )
    reject(
        analysis(attached="//cloudresourcemanager.googleapis.com/projects/other"),
        "unexpected attached resource is allowed by analyzer semantics",
    )
    reject(
        analysis(
            permissions=["iam.serviceAccounts.getAccessToken"],
        ),
        "missing required permission",
    )

    try:
        module.service_account_resource(
            "sol-atlas",
            "malformed",
        )
    except AssertionError:
        pass
    else:
        raise AssertionError("malformed service account was accepted")

    print("offline effective-IAM audit semantic checks: PASS")


if __name__ == "__main__":
    main()
