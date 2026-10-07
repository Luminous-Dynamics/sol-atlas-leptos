#!/usr/bin/env python3
# Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
# SPDX-License-Identifier: AGPL-3.0-or-later

"""Regression tests for broad service-account impersonation audit."""

from __future__ import annotations

import importlib.util
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "verify_gcs_broad_sa_impersonation.py"

spec = importlib.util.spec_from_file_location("broad_sa_audit", SCRIPT)
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
OTHER_SERVICE_ACCOUNT = (
    "//iam.googleapis.com/projects/sol-atlas/serviceAccounts/"
    "privileged@sol-atlas.iam.gserviceaccount.com"
)
PROJECT = "//cloudresourcemanager.googleapis.com/projects/sol-atlas"


def finding(
    *,
    role: str,
    permission: str,
    principal: str = PRINCIPAL,
    resource: str = QUALIFICATION,
    attached: str | None = None,
    condition: str | None = None,
) -> dict[str, object]:
    acl: dict[str, object] = {
        "resources": [{"fullResourceName": resource}],
        "accesses": [{"permission": permission}],
    }
    if condition is not None:
        acl["conditionEvaluation"] = {"evaluationValue": condition}
    return {
        "attachedResourceFullName": attached or resource,
        "iamBinding": {
            "role": role,
            "members": [principal],
        },
        "identityList": {"identities": [{"name": principal}]},
        "accessControlLists": [acl],
        "fullyExplored": True,
    }


def envelope(
    results: list[dict[str, object]],
    complete: bool = True,
    errors=None,
) -> dict[str, object]:
    return {
        "fullyExplored": complete,
        "nonCriticalErrors": list(errors or []),
        "analysisResults": results,
    }


def expect_failure(data: dict[str, object], label: str) -> None:
    try:
        module.extract_findings(
            data,
            PRINCIPAL,
            QUALIFICATION,
            OIDC_SUBJECT,
        )
    except AssertionError:
        return
    raise AssertionError("accepted invalid impersonation evidence: " + label)


def main() -> None:
    clean = finding(
        role=module.EXPECTED_ROLE,
        permission="iam.serviceAccounts.getAccessToken",
    )
    clean["accessControlLists"][0]["accesses"].append({
        "permission": "iam.serviceAccounts.getOpenIdToken"
    })
    assert module.extract_findings(
        envelope([clean]),
        PRINCIPAL,
        QUALIFICATION,
        OIDC_SUBJECT,
    ) == []

    expect_failure(
        envelope([
            finding(
                role="roles/iam.serviceAccountUser",
                permission="iam.serviceAccounts.actAs",
                resource=OTHER_SERVICE_ACCOUNT,
            )
        ]),
        "actAs on another service account",
    )
    expect_failure(
        envelope([
            finding(
                role="roles/iam.serviceAccountTokenCreator",
                permission="iam.serviceAccounts.getAccessToken",
                resource=OTHER_SERVICE_ACCOUNT,
            )
        ]),
        "token creator on another service account",
    )

    for permission in (
        "iam.serviceAccounts.getOpenIdToken",
        "iam.serviceAccounts.signBlob",
        "iam.serviceAccounts.signJwt",
        "iam.serviceAccounts.implicitDelegation",
        "iam.serviceAccountKeys.create",
    ):
        expect_failure(
            envelope([
                finding(
                    role="roles/iam.serviceAccountTokenCreator",
                    permission=permission,
                    resource=OTHER_SERVICE_ACCOUNT,
                )
            ]),
            "credential capability on another service account: " + permission,
        )


    expected_sets = module.load_effective_iam_module().expected_workload_principal_sets(
        PRINCIPAL
    )
    for broad_member in (
        next(member for member in expected_sets if member.endswith("/*")),
        next(
            member
            for member in expected_sets
            if "/attribute.repository/" in member
        ),
    ):
        expect_failure(
            envelope([
                finding(
                    role="roles/iam.serviceAccountTokenCreator",
                    permission="iam.serviceAccounts.getAccessToken",
                    principal=broad_member,
                    resource=OTHER_SERVICE_ACCOUNT,
                )
            ]),
            "broad workload principal set",
        )

    expect_failure(
        envelope([
            finding(
                role="roles/iam.serviceAccountUser",
                permission="iam.serviceAccounts.actAs",
                resource=QUALIFICATION,
            )
        ]),
        "actAs on qualification account via alternate role",
    )

    for universal in module.load_effective_iam_module().FORBIDDEN_UNIVERSAL_PRINCIPALS:
        expect_failure(
            envelope([
                finding(
                    role="roles/iam.serviceAccountTokenCreator",
                    permission="iam.serviceAccounts.getAccessToken",
                    principal=universal,
                    resource=OTHER_SERVICE_ACCOUNT,
                )
            ]),
            "universal principal on another service account",
        )

    assert module.extract_findings(
        envelope([
            finding(
                role="roles/iam.serviceAccountTokenCreator",
                permission="iam.serviceAccounts.getAccessToken",
                condition="FALSE",
                resource=OTHER_SERVICE_ACCOUNT,
            )
        ]),
        PRINCIPAL,
        QUALIFICATION,
        OIDC_SUBJECT,
    ) == []

    expect_failure(
        envelope([
            finding(
                role="roles/iam.serviceAccountUser",
                permission="iam.serviceAccounts.actAs",
                condition="CONDITIONAL",
                resource=OTHER_SERVICE_ACCOUNT,
            )
        ]),
        "conditional access",
    )

    assert module.extract_findings(
        envelope([
            finding(
                role="roles/iam.serviceAccountUser",
                permission="iam.serviceAccounts.actAs",
                principal="principalSet://iam.googleapis.com/unrelated",
                resource=OTHER_SERVICE_ACCOUNT,
            )
        ]),
        PRINCIPAL,
        QUALIFICATION,
        OIDC_SUBJECT,
    ) == []

    mixed = finding(
        role="roles/iam.serviceAccountTokenCreator",
        permission="iam.serviceAccounts.getAccessToken",
        resource=QUALIFICATION,
    )
    mixed["accessControlLists"].append({
        "resources": [{"fullResourceName": OTHER_SERVICE_ACCOUNT}],
        "accesses": [{"permission": "iam.serviceAccounts.getAccessToken"}],
    })
    expect_failure(
        envelope([mixed]),
        "binding covering qualification and another service account",
    )

    expect_failure(
        {
            "fullyExplored": True,
            "nonCriticalErrors": "malformed",
            "analysisResults": [],
        },
        "malformed error envelope",
    )
    expect_failure(
        envelope([], complete=False),
        "incomplete analyzer response",
    )
    expect_failure(
        envelope(
            [clean],
            errors=[{"code": "PERMISSION_DENIED"}],
        ),
        "non-critical analyzer error",
    )

    print("broad service-account impersonation checks: PASS")


if __name__ == "__main__":
    main()
