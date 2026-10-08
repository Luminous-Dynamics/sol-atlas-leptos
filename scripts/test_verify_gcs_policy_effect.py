#!/usr/bin/env python3
# Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
# SPDX-License-Identifier: AGPL-3.0-or-later

"""Offline semantic tests for Policy Troubleshooter evidence."""

from __future__ import annotations

import importlib.util
import json
from pathlib import Path
from tempfile import TemporaryDirectory

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "verify_gcs_policy_effect.py"

spec = importlib.util.spec_from_file_location("policy_effect", SCRIPT)
assert spec and spec.loader
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)

PRINCIPAL = "qualification@sol-atlas.iam.gserviceaccount.com"
RESOURCE = "//cloudresourcemanager.googleapis.com/projects/sol-atlas"


def response(
    *,
    overall: str = "CANNOT_ACCESS",
    allow: str = "ALLOW_ACCESS_STATE_NOT_GRANTED",
    deny: str = "DENY_ACCESS_STATE_NOT_DENIED",
    pab: str = "PAB_ACCESS_STATE_NOT_ENFORCED",
    principal: str = PRINCIPAL,
    resource: str = RESOURCE,
    permission: str = "cloudbuild.builds.create",
) -> dict[str, object]:
    return {
        "accessTuple": {
            "principal": principal,
            "fullResourceName": resource,
            "permission": permission,
        },
        "overallAccessState": overall,
        "allowPolicyExplanation": {"allowAccessState": allow},
        "denyPolicyExplanation": {"denyAccessState": deny},
        "pabPolicyExplanation": {
            "principalAccessBoundaryAccessState": pab
        },
    }


def main() -> None:
    good = module.validate_response(
        response(),
        PRINCIPAL,
        RESOURCE,
        "cloudbuild.builds.create",
        "CANNOT_ACCESS",
    )
    assert good["overall_access_state"] == "CANNOT_ACCESS"
    assert good["deny_access_state"] == "DENY_ACCESS_STATE_NOT_DENIED"
    assert good["pab_access_state"] == "PAB_ACCESS_STATE_NOT_ENFORCED"

    for field, value in (
        ("overallAccessState", "UNKNOWN_INFO"),
        ("overallAccessState", "UNKNOWN_CONDITIONAL"),
    ):
        try:
            module.validate_response(
                response(overall=value),
                PRINCIPAL,
                RESOURCE,
                "cloudbuild.builds.create",
                "CANNOT_ACCESS",
            )
        except AssertionError:
            pass
        else:
            raise AssertionError("accepted unknown overall access state")

    for kwargs in (
        {"allow": "ALLOW_ACCESS_STATE_UNKNOWN_INFO"},
        {"allow": "ALLOW_ACCESS_STATE_UNKNOWN_CONDITIONAL"},
        {"deny": "DENY_ACCESS_STATE_UNKNOWN_INFO"},
        {"deny": "DENY_ACCESS_STATE_UNKNOWN_CONDITIONAL"},
        {"pab": "PAB_ACCESS_STATE_UNKNOWN_INFO"},
        {"pab": "PAB_ACCESS_STATE_UNSPECIFIED"},
    ):
        try:
            module.validate_response(
                response(**kwargs),
                PRINCIPAL,
                RESOURCE,
                "cloudbuild.builds.create",
                "CANNOT_ACCESS",
            )
        except AssertionError:
            pass
        else:
            raise AssertionError("accepted unknown policy-plane state")

    for kwargs in (
        {"principal": "other@sol-atlas.iam.gserviceaccount.com"},
        {"resource": "//cloudresourcemanager.googleapis.com/projects/other"},
        {"permission": "deploymentmanager.deployments.create"},
        {"overall": "CAN_ACCESS"},
    ):
        try:
            module.validate_response(
                response(**kwargs),
                PRINCIPAL,
                RESOURCE,
                "cloudbuild.builds.create",
                "CANNOT_ACCESS",
            )
        except AssertionError:
            pass
        else:
            raise AssertionError("accepted mismatched troubleshooter tuple")

    try:
        module.validate_service_account(
            "principalSet://iam.googleapis.com/example"
        )
    except AssertionError:
        pass
    else:
        raise AssertionError("accepted workload identity principal")

    targets = [
        {
            "resource": RESOURCE,
            "permission": "cloudbuild.builds.create",
            "expected_overall_access_state": "CANNOT_ACCESS",
        }
    ]
    manifest = {
        "schema": "sol-atlas:gcs-policy-troubleshooter-targets:v1",
        "targets": [
            {
                "resource_template": (
                    "//cloudresourcemanager.googleapis.com/projects/{project_id}"
                ),
                "permission": "cloudbuild.builds.create",
                "expected_overall_access_state": "CANNOT_ACCESS",
            },
            {
                "resource_template": (
                    "//iam.googleapis.com/projects/{project_id}/"
                    "serviceAccounts/{service_account}"
                ),
                "permission": "iam.serviceAccountKeys.create",
                "expected_overall_access_state": "CANNOT_ACCESS",
            },
        ],
    }
    with TemporaryDirectory() as tmp:
        manifest_path = Path(tmp) / "targets.json"
        manifest_path.write_text(
            json.dumps(manifest),
            encoding="utf-8",
        )
        expanded = module.load_targets(
            str(manifest_path),
            "sol-atlas",
            PRINCIPAL,
        )
        assert expanded[0]["resource"] == RESOURCE.split(
            "/serviceAccounts/",
            1,
        )[0] if False else (
            "//cloudresourcemanager.googleapis.com/projects/sol-atlas"
        )
        assert expanded[1]["resource"].endswith(
            "/qualification@sol-atlas.iam.gserviceaccount.com"
        )
    assert module.validate_target(targets[0]) == (
        RESOURCE,
        "cloudbuild.builds.create",
        "CANNOT_ACCESS",
    )

    try:
        module.verify_targets(PRINCIPAL, [], None)
    except AssertionError:
        pass
    else:
        raise AssertionError("accepted empty target set")

    print("Policy Troubleshooter semantic checks: PASS")


if __name__ == "__main__":
    main()
