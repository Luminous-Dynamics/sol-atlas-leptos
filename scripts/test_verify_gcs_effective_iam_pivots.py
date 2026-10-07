#!/usr/bin/env python3
# Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
# SPDX-License-Identifier: AGPL-3.0-or-later

"""Regression tests for project-level execution pivots."""

from __future__ import annotations

import importlib.util
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "verify_gcs_effective_iam.py"

spec = importlib.util.spec_from_file_location("effective_iam_pivots", SCRIPT)
assert spec and spec.loader
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)

PRINCIPAL = (
    "principalSet://iam.googleapis.com/projects/123456789/"
    "locations/global/workloadIdentityPools/github/"
    "attribute.repository_id/1195997641"
)
PROJECT = "//cloudresourcemanager.googleapis.com/projects/sol-atlas"


def pivot(
    permission: str,
    principal: str = PRINCIPAL,
    condition: str | None = None,
    attached: str = PROJECT,
    members: list[str] | None = None,
) -> dict[str, object]:
    acl: dict[str, object] = {
        "resources": [{"fullResourceName": PROJECT}],
        "accesses": [{"permission": permission}],
    }
    if condition is not None:
        acl["conditionEvaluation"] = {"evaluationValue": condition}
    return {
        "attachedResourceFullName": attached,
        "iamBinding": {
            "role": "roles/cloudbuild.builds.editor",
            "members": members or [principal],
        },
        "identityList": {"identities": [{"name": principal}]},
        "accessControlLists": [acl],
        "fullyExplored": True,
    }


def envelope(results: list[dict[str, object]]) -> dict[str, object]:
    return {
        "fullyExplored": True,
        "nonCriticalErrors": [],
        "analysisResults": results,
    }


def expect_failure(data: dict[str, object], label: str) -> None:
    try:
        module.extract_project_pivot_findings(
            data,
            PRINCIPAL,
            PROJECT,
        )
    except AssertionError:
        return
    raise AssertionError("accepted invalid pivot evidence: " + label)


def main() -> None:
    clean = module.extract_project_pivot_findings(
        envelope([]),
        PRINCIPAL,
        PROJECT,
    )
    assert clean == []

    unrelated = module.extract_project_pivot_findings(
        envelope([
            pivot(
                "cloudbuild.builds.create",
                principal="principalSet://iam.googleapis.com/unrelated",
            )
        ]),
        PRINCIPAL,
        PROJECT,
    )
    assert unrelated == []

    containing_member = next(iter(
        module.expected_workload_principal_sets(PRINCIPAL) - {PRINCIPAL}
    ))
    broadened = module.extract_project_pivot_findings(
        envelope([
            pivot("cloudbuild.builds.create", principal=containing_member)
        ]),
        PRINCIPAL,
        PROJECT,
    )
    assert broadened[0]["principal_match_kinds"] == [
        "containing-principal-set"
    ]
    detected = module.extract_project_pivot_findings(
        envelope([pivot("cloudbuild.builds.create")]),
        PRINCIPAL,
        PROJECT,
    )
    assert detected[0]["permissions"] == ["cloudbuild.builds.create"]

    detected_deploy = module.extract_project_pivot_findings(
        envelope([pivot("deploymentmanager.deployments.create")]),
        PRINCIPAL,
        PROJECT,
    )
    assert detected_deploy[0]["permissions"] == [
        "deploymentmanager.deployments.create"
    ]

    false_condition = module.extract_project_pivot_findings(
        envelope([
            pivot(
                "cloudbuild.builds.create",
                condition="FALSE",
            )
        ]),
        PRINCIPAL,
        PROJECT,
    )
    assert false_condition == []

    expect_failure(
        envelope([
            pivot(
                "cloudbuild.builds.create",
                condition="CONDITIONAL",
            )
        ]),
        "conditional grant",
    )
    expect_failure(
        envelope([
            pivot(
                "cloudbuild.builds.create",
                attached="//cloudresourcemanager.googleapis.com/projects/other",
            )
        ]),
        "wrong attachment",
    )
    expect_failure(
        envelope([
            pivot(
                "cloudbuild.builds.create",
                members=[PRINCIPAL, "group:unexpected@example.com"],
            )
        ]),
        "unexpected binding members",
    )
    expect_failure(
        {
            "fullyExplored": False,
            "nonCriticalErrors": [],
            "analysisResults": [],
        },
        "incomplete response",
    )

    assert module.valid_project_attachment(PROJECT, PROJECT)
    assert module.valid_project_attachment(
        "//cloudresourcemanager.googleapis.com/folders/123",
        PROJECT,
    )
    assert not module.valid_project_attachment(
        "//iam.googleapis.com/projects/sol-atlas/serviceAccounts/a@b",
        PROJECT,
    )

    print("project execution pivot checks: PASS")


if __name__ == "__main__":
    main()
