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
BUCKET = "sol-atlas-qualification"
OBJECT_ROOT = "sol-atlas/qualification/run-123-attempt-1"


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

    assert good["response"] == response()
    assert good["response_digest"] == module.digest(
        good["response"]
    )
    tampered_raw = dict(
        good["response"],
        overallAccessState="CAN_ACCESS",
    )
    assert good["response_digest"] != module.digest(tampered_raw)

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

    object_resource = module.object_resource(
        BUCKET,
        OBJECT_ROOT + "/main.bin",
    )
    assert object_resource == (
        "//storage.googleapis.com/projects/_/buckets/"
        "sol-atlas-qualification/objects/"
        "sol-atlas/qualification/run-123-attempt-1/main.bin"
    )

    v3_manifest = {
        "schema": module.TARGET_MANIFEST_SCHEMA,
        "targets": [
            {
                "resource_template": (
                    "//storage.googleapis.com/projects/_/buckets/{bucket}/"
                    "objects/{object_root}/main.bin"
                ),
                "permission": "storage.objects.create",
                "expected_overall_access_state": "CAN_ACCESS",
            }
        ],
    }
    with TemporaryDirectory() as tmp:
        manifest_path = Path(tmp) / "targets-v3.json"
        manifest_path.write_text(
            json.dumps(v2_manifest),
            encoding="utf-8",
        )
        expanded = module.load_targets(
            str(manifest_path),
            "sol-atlas",
            PRINCIPAL,
            BUCKET,
            OBJECT_ROOT,
        )
        assert expanded[0]["resource"] == object_resource

        for kwargs in (
            {
                "project_id": "sol-atlas",
                "service_account": PRINCIPAL,
                "object_root": OBJECT_ROOT,
            },
            {
                "project_id": "sol-atlas",
                "service_account": PRINCIPAL,
                "bucket": BUCKET,
            },
        ):
            try:
                module.load_targets(
                    str(manifest_path),
                    kwargs["project_id"],
                    kwargs["service_account"],
                    kwargs.get("bucket"),
                    kwargs.get("object_root"),
                )
            except AssertionError:
                pass
            else:
                raise AssertionError(
                    "accepted v3 target manifest without required bindings"
                )

    with TemporaryDirectory() as tmp:
        legacy_path = Path(tmp) / "legacy-v1.json"
        legacy_path.write_text(
            json.dumps(
                {
                    "schema": "sol-atlas:gcs-policy-troubleshooter-targets:v1",
                    "targets": [],
                }
            ),
            encoding="utf-8",
        )
        try:
            module.load_targets(
                str(legacy_path),
                "sol-atlas",
                PRINCIPAL,
            )
        except AssertionError:
            pass
        else:
            raise AssertionError(
                "accepted superseded Policy Troubleshooter target manifest"
            )

    targets = [
        {
            "resource": RESOURCE,
            "permission": "cloudbuild.builds.create",
            "expected_overall_access_state": "CANNOT_ACCESS",
        }
    ]
    manifest = {
        "schema": module.TARGET_MANIFEST_SCHEMA,
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
        assert expanded[0]["resource"] == (
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

    canonical_path = (
        ROOT
        / "sol-atlas-policy-store-contract"
        / "conformance"
        / "gcs_policy_troubleshooter_targets_v3.json"
    )
    bound_path, bound_digest = module.validate_target_manifest_binding(
        str(canonical_path.relative_to(ROOT))
    )
    assert bound_path == module.TARGET_MANIFEST_PATH
    assert bound_digest.startswith("sha256:")

    with TemporaryDirectory() as tmp:
        alternate = Path(tmp) / "alternate-v3.json"
        alternate.write_text(
            json.dumps(
                {
                    "schema": module.TARGET_MANIFEST_SCHEMA,
                    "targets": list(manifest["targets"])
                    + [
                        {
                            "resource_template": RESOURCE,
                            "permission": "storage.objects.create",
                            "expected_overall_access_state": "CAN_ACCESS",
                        }
                    ],
                }
            ),
            encoding="utf-8",
        )
        try:
            module.validate_target_manifest_binding(str(alternate))
        except AssertionError:
            pass
        else:
            raise AssertionError(
                "accepted alternate v3 manifest before external calls"
            )

    frozen_shape = [
        {
            "resource_template": "//cloudresourcemanager.googleapis.com/projects/{project_id}",
            "permission": "cloudbuild.builds.create",
            "expected_overall_access_state": "CANNOT_ACCESS",
            "resource": "//cloudresourcemanager.googleapis.com/projects/sol-atlas",
        }
    ]
    supplied_shape = list(frozen_shape)
    module.validate_target_vector_binding(
        supplied_shape,
        frozen_shape,
    )
    tampered_shape = list(frozen_shape)
    tampered_shape[0] = dict(
        tampered_shape[0],
        permission="cloudbuild.builds.get",
    )
    try:
        module.validate_target_vector_binding(
            tampered_shape,
            frozen_shape,
        )
    except AssertionError:
        pass
    else:
        raise AssertionError(
            "accepted tampered frozen target vector"
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
