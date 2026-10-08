#!/usr/bin/env python3
# Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
# SPDX-License-Identifier: AGPL-3.0-or-later

"""Regression tests for observer/effect-account isolation."""

from __future__ import annotations

import importlib.util
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "verify_gcs_observer_isolation.py"

spec = importlib.util.spec_from_file_location("observer_isolation", SCRIPT)
assert spec and spec.loader
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)

OBSERVER = "iam-observer@sol-atlas.iam.gserviceaccount.com"
OBSERVER_PRINCIPAL = "serviceAccount:" + OBSERVER
EFFECT = (
    "//iam.googleapis.com/projects/sol-atlas/serviceAccounts/"
    "qualification@sol-atlas.iam.gserviceaccount.com"
)


def finding(
    permissions: list[str],
    *,
    member: str = OBSERVER_PRINCIPAL,
    condition: str | None = None,
    attached: str = (
        "//cloudresourcemanager.googleapis.com/projects/sol-atlas"
    ),
) -> dict[str, object]:
    acl: dict[str, object] = {
        "resources": [{"fullResourceName": EFFECT}],
        "accesses": [{"permission": value} for value in permissions],
    }
    if condition is not None:
        acl["conditionEvaluation"] = {"evaluationValue": condition}
    return {
        "attachedResourceFullName": attached,
        "iamBinding": {
            "role": "roles/iam.serviceAccountViewer",
            "members": [member],
        },
        "identityList": {
            "identities": [{"name": OBSERVER_PRINCIPAL}]
        },
        "accessControlLists": [acl],
        "fullyExplored": True,
    }


def envelope(results, complete=True, errors=None):
    return {
        "fullyExplored": complete,
        "nonCriticalErrors": list(errors or []),
        "analysisResults": list(results),
    }


def expect_failure(data, label: str) -> None:
    try:
        module.extract(
            data,
            OBSERVER_PRINCIPAL,
            EFFECT,
            module.FORBIDDEN_SERVICE_ACCOUNT_PERMISSIONS,
        )
    except AssertionError:
        return
    raise AssertionError("accepted invalid observer isolation: " + label)


def authority_finding(
    permission: str,
    *,
    identity: str = OBSERVER_PRINCIPAL,
    members=None,
    condition: str | None = None,
) -> dict[str, object]:
    acl: dict[str, object] = {
        "resources": [{
            "fullResourceName":
                "//cloudresourcemanager.googleapis.com/projects/sol-atlas"
        }],
        "accesses": [{"permission": permission}],
    }
    if condition is not None:
        acl["conditionEvaluation"] = {
            "evaluationValue": condition
        }
    return {
        "iamBinding": {
            "members": list(members or [OBSERVER_PRINCIPAL])
        },
        "identityList": {
            "identities": [{"name": identity}]
        },
        "accessControlLists": [acl],
        "fullyExplored": True,
    }


def main() -> None:
    clean = module.extract(
        envelope([
            finding([module.REQUIRED_OBSERVER_PERMISSION])
        ]),
        OBSERVER_PRINCIPAL,
        EFFECT,
        module.FORBIDDEN_SERVICE_ACCOUNT_PERMISSIONS,
    )
    assert clean["required_observer_permission_verified"] is True
    assert clean["forbidden_effect_permissions_absent"] is True

    for permission in module.FORBIDDEN_SERVICE_ACCOUNT_PERMISSIONS:
        expect_failure(
            envelope([
                finding([
                    module.REQUIRED_OBSERVER_PERMISSION,
                    permission,
                ])
            ]),
            "forbidden permission " + permission,
        )

    authority_clean = module.extract_observer_authority(
        envelope([]),
        OBSERVER_PRINCIPAL,
        "//cloudresourcemanager.googleapis.com/projects/sol-atlas",
    )
    assert authority_clean == set()

    authority_detected = module.extract_observer_authority(
        envelope([
            authority_finding(
                "resourcemanager.projects.setIamPolicy"
            )
        ]),
        OBSERVER_PRINCIPAL,
        "//cloudresourcemanager.googleapis.com/projects/sol-atlas",
    )
    assert authority_detected == {"resourcemanager.projects.setIamPolicy"}

    authority_false = module.extract_observer_authority(
        envelope([
            authority_finding(
                "iam.roles.create",
                condition="FALSE",
            )
        ]),
        OBSERVER_PRINCIPAL,
        "//cloudresourcemanager.googleapis.com/projects/sol-atlas",
    )
    assert authority_false == set()

    try:
        module.extract_observer_authority(
            envelope([
                authority_finding(
                    "iam.workloadIdentityPools.update",
                    condition="CONDITIONAL",
                )
            ]),
            OBSERVER_PRINCIPAL,
            "//cloudresourcemanager.googleapis.com/projects/sol-atlas",
        )
    except AssertionError:
        pass
    else:
        raise AssertionError(
            "conditional observer authority was accepted"
        )

    try:
        module.extract_observer_authority(
            envelope([
                authority_finding(
                    "iam.workloadIdentityPools.update",
                    members=["allAuthenticatedUsers"],
                )
            ]),
            OBSERVER_PRINCIPAL,
            "//cloudresourcemanager.googleapis.com/projects/sol-atlas",
        )
    except AssertionError:
        pass
    else:
        raise AssertionError(
            "universal observer authority was accepted"
        )

    pivot_clean = {
        "fullyExplored": True,
        "nonCriticalErrors": [],
        "analysisResults": [],
    }
    assert module.extract_project_pivots(
        pivot_clean,
        "//cloudresourcemanager.googleapis.com/projects/sol-atlas",
    ) == set()

    pivot_result = {
        "fullyExplored": True,
        "identityList": {
            "identities": [{"name": OBSERVER_PRINCIPAL}]
        },
        "accessControlLists": [{
            "resources": [{
                "fullResourceName":
                    "//cloudresourcemanager.googleapis.com/projects/sol-atlas"
            }],
            "accesses": [{
                "permission": "cloudbuild.builds.create"
            }],
        }],
    }
    universal_pivot = dict(pivot_result)
    universal_pivot["iamBinding"] = {
        "members": ["allAuthenticatedUsers"]
    }
    try:
        module.extract_project_pivots(
            {
                "fullyExplored": True,
                "nonCriticalErrors": [],
                "analysisResults": [universal_pivot],
            },
            "//cloudresourcemanager.googleapis.com/projects/sol-atlas",
        )
    except AssertionError:
        pass
    else:
        raise AssertionError(
            "project pivot parser accepted universal principal"
        )

    observed_pivot = module.extract_project_pivots(
        {
            "fullyExplored": True,
            "nonCriticalErrors": [],
            "analysisResults": [pivot_result],
        },
        "//cloudresourcemanager.googleapis.com/projects/sol-atlas",
    )
    assert observed_pivot == {"cloudbuild.builds.create"}

    for permission in (
        "deploymentmanager.deployments.create",
        "compute.instances.create",
        "run.services.create",
        "run.jobs.create",
        "cloudfunctions.functions.create",
        "resourcemanager.projects.setIamPolicy",
    ):
        result = dict(pivot_result)
        result["accessControlLists"] = [{
            "resources": [{
                "fullResourceName":
                    "//cloudresourcemanager.googleapis.com/projects/sol-atlas"
            }],
            "accesses": [{"permission": permission}],
        }]
        assert module.extract_project_pivots(
            {
                "fullyExplored": True,
                "nonCriticalErrors": [],
                "analysisResults": [result],
            },
            "//cloudresourcemanager.googleapis.com/projects/sol-atlas",
        ) == {permission}


    for universal in ("allUsers", "allAuthenticatedUsers"):
        expect_failure(
            envelope([
                finding(
                    [module.REQUIRED_OBSERVER_PERMISSION],
                    member=universal,
                )
            ]),
            "universal principal " + universal,
        )

    expect_failure(
        envelope([
            finding(
                [module.REQUIRED_OBSERVER_PERMISSION],
                condition="CONDITIONAL",
            )
        ]),
        "conditional access",
    )

    expect_failure(
        envelope([
            finding(
                [module.REQUIRED_OBSERVER_PERMISSION],
                attached="//cloudresourcemanager.googleapis.com/projects/other",
            )
        ]),
        "wrong attachment",
    )

    expect_failure(
        envelope([], complete=False),
        "incomplete response",
    )

    expect_failure(
        envelope(
            [],
            errors=[{"code": "PERMISSION_DENIED"}],
        ),
        "analysis error",
    )

    try:
        module.service_account_principal("malformed")
    except AssertionError:
        pass
    else:
        raise AssertionError("malformed observer identity was accepted")

    assert module.service_account_principal(OBSERVER) == OBSERVER_PRINCIPAL
    try:
        module.extract_project_pivots(
            {
                "fullyExplored": True,
                "nonCriticalErrors": [{"code": "DENIED"}],
                "analysisResults": [],
            },
            "//cloudresourcemanager.googleapis.com/projects/sol-atlas",
        )
    except AssertionError:
        pass
    else:
        raise AssertionError("project pivot analyzer accepted an error response")
    print("observer isolation checks: PASS")


if __name__ == "__main__":
    main()
