#!/usr/bin/env python3
# Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
# SPDX-License-Identifier: AGPL-3.0-or-later

"""Regression tests for project-level service-account execution pivots."""

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
OIDC_SUBJECT = (
    "repo:Luminous-Dynamics@216969177/sol-atlas-leptos@1195997641:"
    "ref:refs/heads/main"
)


def pivot(
    permission: str,
    principal: str = PRINCIPAL,
    *,
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


def envelope(
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


def extract(data: dict[str, object]) -> list[dict[str, object]]:
    return module.extract_project_pivot_findings(
        data,
        PRINCIPAL,
        PROJECT,
        OIDC_SUBJECT,
    )


def expect_failure(data: dict[str, object], label: str) -> None:
    try:
        extract(data)
    except AssertionError:
        return
    raise AssertionError("accepted invalid pivot evidence: " + label)


def main() -> None:
    assert extract(envelope([])) == []

    unrelated = extract(
        envelope([
            pivot(
                "cloudbuild.builds.create",
                "principalSet://iam.googleapis.com/unrelated",
            )
        ])
    )
    assert unrelated == []

    expected_sets = module.expected_workload_principal_sets(PRINCIPAL)
    containment_candidates = [
        value for value in expected_sets
        if value.endswith("/*") or "/attribute.repository/" in value
    ]
    assert len(containment_candidates) == 2

    for candidate in containment_candidates:
        detected = extract(
            envelope([
                pivot(
                    "cloudbuild.builds.create",
                    candidate,
                )
            ])
        )
        assert detected[0]["principal_match_kinds"] == [
            "containing-principal-set"
        ]

    subject = module.subject_principal(PRINCIPAL, OIDC_SUBJECT)
    expect_failure(
        envelope([
            pivot(
                "cloudbuild.builds.create",
                subject,
            )
        ]),
        "immutable subject principal",
    )

    for universal in module.FORBIDDEN_UNIVERSAL_PRINCIPALS:
        expect_failure(
            envelope([
                pivot(
                    "cloudbuild.builds.create",
                    universal,
                )
            ]),
            "universal principal",
        )

    detected = extract(
        envelope([pivot("cloudbuild.builds.create")])
    )
    assert detected[0]["permissions"] == ["cloudbuild.builds.create"]

    detected = extract(
        envelope([pivot("deploymentmanager.deployments.create")])
    )
    assert detected[0]["permissions"] == [
        "deploymentmanager.deployments.create"
    ]

    assert extract(
        envelope([
            pivot(
                "cloudbuild.builds.create",
                condition="FALSE",
            )
        ])
    ) == []

    expect_failure(
        envelope([
            pivot(
                "cloudbuild.builds.create",
                condition="CONDITIONAL",
            )
        ]),
        "conditional pivot",
    )
    expect_failure(
        envelope([
            pivot(
                "cloudbuild.builds.create",
                attached=(
                    "//cloudresourcemanager.googleapis.com/projects/other"
                ),
            )
        ]),
        "wrong project attachment",
    )
    expect_failure(
        envelope([
            pivot(
                "cloudbuild.builds.create",
                members=[
                    PRINCIPAL,
                    "group:unexpected@example.com",
                ],
            )
        ]),
        "mixed binding",
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
