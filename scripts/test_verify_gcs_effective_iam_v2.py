#!/usr/bin/env python3
# Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
# SPDX-License-Identifier: AGPL-3.0-or-later

"""Adversarial regression tests for effective IAM parsing."""

from __future__ import annotations

import importlib.util
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "verify_gcs_effective_iam.py"

spec = importlib.util.spec_from_file_location("effective_iam_adv", SCRIPT)
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
    "repo:Luminous-Dynamics@216969177/"
    "sol-atlas-leptos@1195997641:ref:refs/heads/main"
)


def result(
    *,
    role: str = module.EXPECTED_ROLE,
    members: list[str] | None = None,
    identities: list[str] | None = None,
    permissions: list[str] | None = None,
    attached: str = (
        "//cloudresourcemanager.googleapis.com/projects/sol-atlas"
    ),
    complete: bool = True,
) -> dict[str, object]:
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
        "fullyExplored": complete,
    }


def payload(
    results: list[dict[str, object]],
    *,
    complete: bool = True,
    errors: list[object] | None = None,
) -> dict[str, object]:
    return {
        "fullyExplored": complete,
        "nonCriticalErrors": errors or [],
        "analysisResults": results,
    }


def expect_reject(data: dict[str, object], label: str) -> None:
    try:
        module.extract_findings(
            data,
            PRINCIPAL,
            RESOURCE,
            OIDC_SUBJECT,
        )
    except AssertionError:
        return
    raise AssertionError("accepted invalid evidence: " + label)


def main() -> None:
    broad = next(
        value
        for value in module.expected_workload_principal_sets(PRINCIPAL)
        if value.endswith("/*")
    )
    subject_principal = module.subject_principal(
        PRINCIPAL,
        OIDC_SUBJECT,
    )

    expect_reject(
        payload([
            result(
                members=[broad],
                identities=[broad],
            )
        ]),
        "pool-wide principal set",
    )
    expect_reject(
        payload([
            result(
                members=[subject_principal],
                identities=[subject_principal],
            )
        ]),
        "immutable subject principal",
    )
    expect_reject(
        payload([
            result(
                members=[PRINCIPAL, broad],
            )
        ]),
        "mixed exact and broad members",
    )
    expect_reject(
        payload([
            result(
                members=[PRINCIPAL, "group:unexpected@example.com"],
            )
        ]),
        "mixed exact and unrelated members",
    )
    expect_reject(
        payload([result(role="roles/iam.serviceAccountTokenCreator")]),
        "alternate predefined role",
    )
    expect_reject(
        payload([result(role="customRoles/alternate")]),
        "custom role",
    )
    expect_reject(
        payload([result(complete=False)]),
        "incomplete result",
    )
    expect_reject(
        payload(
            [result()],
            complete=False,
        ),
        "incomplete response",
    )
    expect_reject(
        payload(
            [result()],
            errors=[{"code": "PERMISSION_DENIED"}],
        ),
        "analysis error",
    )
    expect_reject(
        payload([result(), result()]),
        "multiple effective bindings",
    )
    expect_reject(
        payload([
            result(
                attached=(
                    "//cloudresourcemanager.googleapis.com/projects/other"
                )
            )
        ]),
        "wrong project",
    )

    multiple_acl = result()
    multiple_acl["accessControlLists"] = [
        multiple_acl["accessControlLists"][0],
        {
            "resources": [{"fullResourceName": RESOURCE}],
            "accesses": [
                {"permission": "iam.serviceAccounts.signBlob"}
            ],
        },
    ]
    accepted = module.extract_findings(
        payload([multiple_acl]),
        PRINCIPAL,
        RESOURCE,
        OIDC_SUBJECT,
    )
    assert "iam.serviceAccounts.signBlob" in accepted[0]["permissions"]

    print("adversarial effective-IAM checks: PASS")


if __name__ == "__main__":
    main()
