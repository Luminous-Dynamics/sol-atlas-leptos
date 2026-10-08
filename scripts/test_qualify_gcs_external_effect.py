#!/usr/bin/env python3
# Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
# SPDX-License-Identifier: AGPL-3.0-or-later

"""Offline tests for GCS qualification report resource identity binding."""

from __future__ import annotations

import importlib.util
import json
from pathlib import Path
from tempfile import TemporaryDirectory

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "qualify_gcs_external_effect.py"
spec = importlib.util.spec_from_file_location("qualify_gcs_external_effect", SCRIPT)
assert spec and spec.loader
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)

OBJECT_PREFIX = "sol-atlas/qualification"
OBJECT_ROOT = OBJECT_PREFIX + "/run-1-attempt-1-abc"


def expect_rejection(report: dict[str, object], message: str) -> None:
    try:
        module.verify_resource_identity(report)
    except AssertionError:
        return
    raise AssertionError(message)


def main() -> None:
    assert module.SCHEMA == (
        "sol-atlas:recovery-execution-effect-external-report:v8"
    )
    assert module.EFFECTIVE_IAM_AUDIT_SCHEMA == (
        "sol-atlas:gcs-wif-effective-iam-audit:v8"
    )
    assert module.EFFECTIVE_IAM_PROJECT_PIVOT_PERMISSIONS == [
        "cloudbuild.builds.create",
        "deploymentmanager.deployments.create",
        "compute.instances.create",
        "run.services.create",
        "run.jobs.create",
        "cloudfunctions.functions.create",
        "resourcemanager.projects.setIamPolicy",
    ]
    good = {
        "service": "Google Cloud Storage",
        "bucket": "sol-atlas-qualification",
        "object_prefix": OBJECT_PREFIX,
        "object_root": OBJECT_ROOT,
        "object_names": [
            OBJECT_ROOT + "/main.bin",
            OBJECT_ROOT + "/point-in-time.bin",
            OBJECT_ROOT + "/metadata-race.bin",
        ],
    }
    module.verify_resource_identity(good)

    server_good = {
        "schema": module.GITHUB_RUN_VERIFICATION_SCHEMA,
        "repository": "Luminous-Dynamics/sol-atlas-leptos",
        "repository_id": "1195997641",
        "repository_owner_id": "216969177",
        "workflow_name": "Qualify GCS external effect",
        "workflow_path": ".github/workflows/qualify-gcs.yml",
        "workflow_id": 311325850,
        "workflow_id_frozen": 311325850,
        "run_path_verified": ".github/workflows/qualify-gcs.yml",
        "head_branch": "main",
        "event": "workflow_dispatch",
        "github_server": "github.com",
        "context_sha_matches_server": True,
        "context_run_attempt_matches_server": True,
        "referenced_workflows": [],
        "run_id": "12345",
        "run_attempt": "1",
        "head_sha": "a" * 40,
    }
    with TemporaryDirectory() as tmp:
        verification_path = Path(tmp) / "github-run.json"
        verification_path.write_text(json.dumps(server_good), encoding="utf-8")
        module.load_github_workflow_run_verification(
            str(verification_path)
        )
        bad_server = dict(
            server_good,
            workflow_id_frozen=999999,
        )
        verification_path.write_text(
            json.dumps(bad_server),
            encoding="utf-8",
        )
        try:
            module.load_github_workflow_run_verification(
                str(verification_path)
            )
        except AssertionError:
            pass
        else:
            raise AssertionError(
                "report loader accepted tampered GitHub workflow identity"
            )

    assert module.point_in_time_semantics(
        "ObservedNotApplied",
        "Applied",
        "ObservedAppliedSameRequest",
    )
    assert not module.point_in_time_semantics(
        "ObservedDifferentRequest",
        "Applied",
        "ObservedAppliedSameRequest",
    )
    assert not module.point_in_time_semantics(
        "ObservedNotApplied",
        "RejectedPrecondition",
        "ObservedNotApplied",
    )

    expect_rejection(
        dict(good, service="Other service"),
        "wrong external service was accepted",
    )
    expect_rejection(
        dict(good, bucket=""),
        "missing bucket identity was accepted",
    )
    expect_rejection(
        dict(good, object_prefix=""),
        "missing object prefix was accepted",
    )
    expect_rejection(
        dict(good, object_names=good["object_names"][:2]),
        "incomplete object identity list was accepted",
    )
    expect_rejection(
        dict(good, object_names=[
            OBJECT_ROOT + "/main.bin",
            OBJECT_ROOT + "/point-in-time.bin",
            "other-prefix/metadata-race.bin",
        ]),
        "object prefix drift was accepted",
    )
    expect_rejection(
        dict(good, object_names=[
            OBJECT_ROOT + "/main.bin",
            OBJECT_ROOT + "/main.bin",
            OBJECT_ROOT + "/metadata-race.bin",
        ]),
        "duplicate object identity was accepted",
    )


    profile_path = (
        "sol-atlas-policy-store-contract/conformance/"
        "gcs_wif_trust_profile_v9.json"
    )
    valid_wif = dict(
        schema="sol-atlas:gcs-wif-trust-verification:v9",
        profile_path=profile_path,
        profile_digest=module.digest(
            json.loads(Path(profile_path).read_text(encoding="utf-8"))
        ),
        attribute_mapping_verified=True,
        attribute_condition_verified=True,
        service_account_binding_verified=True,
        service_account_direct_policy_exact_verified=True,
        forbidden_direct_service_account_roles_absent=True,
        provider_pool_exclusive=True,
        service_account_project_verified=True,
        oidc_audience_verified=True,
        observer_service_account="observer@sol-atlas.iam.gserviceaccount.com",
        observer_identity_verified=True,
        project_id="sol-atlas",
        service_account="qualification@sol-atlas.iam.gserviceaccount.com",
    )
    policy_targets = []
    project = (
        "//cloudresourcemanager.googleapis.com/projects/"
        "sol-atlas"
    )
    service_account = (
        "//iam.googleapis.com/projects/sol-atlas/serviceAccounts/"
        "qualification@sol-atlas.iam.gserviceaccount.com"
    )
    negative_targets = [
        (project, "cloudbuild.builds.create"),
        (project, "deploymentmanager.deployments.create"),
        (service_account, "iam.serviceAccountKeys.create"),
        (service_account, "iam.serviceAccounts.getIamPolicy"),
        (service_account, "iam.serviceAccounts.setIamPolicy"),
    ]
    for resource, permission in negative_targets:
        policy_targets.append(
            {
                "resource": resource,
                "permission": permission,
                "expected_overall_access_state": "CANNOT_ACCESS",
                "overall_access_state": "CANNOT_ACCESS",
                "allow_access_state": "ALLOW_ACCESS_STATE_NOT_GRANTED",
                "deny_access_state": "DENY_ACCESS_STATE_NOT_DENIED",
                "pab_access_state": "PAB_ACCESS_STATE_NOT_ENFORCED",
                "response_digest": "sha256:" + "0" * 64,
            }
        )
    for object_name in good["object_names"]:
        resource = (
            "//storage.googleapis.com/projects/_/buckets/"
            + good["bucket"]
            + "/objects/"
            + object_name
        )
        for permission in (
            "storage.objects.create",
            "storage.objects.get",
            "storage.objects.delete",
        ):
            policy_targets.append(
                {
                    "resource": resource,
                    "permission": permission,
                    "expected_overall_access_state": "CAN_ACCESS",
                    "overall_access_state": "CAN_ACCESS",
                    "allow_access_state": "ALLOW_ACCESS_STATE_GRANTED",
                    "deny_access_state": "DENY_ACCESS_STATE_NOT_DENIED",
                    "pab_access_state": "PAB_ACCESS_STATE_NOT_ENFORCED",
                    "response_digest": "sha256:" + "0" * 64,
                }
            )
    policy_audit = {
        "schema": module.POLICY_EFFECT_AUDIT_SCHEMA,
        "api_version": "v3beta",
        "principal": valid_wif["service_account"],
        "target_count": len(policy_targets),
        "targets": policy_targets,
        "all_targets_verified": True,
    }
    module.validate_policy_effect_audit(
        policy_audit,
        valid_wif,
        good["bucket"],
        good["object_names"],
        "sol-atlas",
    )
    assert len(policy_targets) == 14
    tampered_policy = json.loads(json.dumps(policy_audit))
    tampered_policy["targets"][0]["overall_access_state"] = "CAN_ACCESS"
    try:
        module.validate_policy_effect_audit(
            tampered_policy,
            valid_wif,
            good["bucket"],
            good["object_names"],
            "sol-atlas",
        )
    except AssertionError:
        pass
    else:
        raise AssertionError(
            "Policy Troubleshooter validator accepted tampered state"
        )

    module.validate_wif_verification(valid_wif)
    same_identity = dict(
        valid_wif,
        observer_service_account="qualification@sol-atlas.iam.gserviceaccount.com",
    )
    try:
        module.validate_wif_verification(same_identity)
    except AssertionError:
        pass
    else:
        raise AssertionError("report accepted effect identity as observer")
    weakened_wif = dict(
        valid_wif,
        service_account_direct_policy_exact_verified=False,
    )
    try:
        module.validate_wif_verification(weakened_wif)
    except AssertionError:
        pass
    else:
        raise AssertionError("report WIF validator accepted weakened exact-policy evidence")

    forbidden = {
        "schema": "sol-atlas:gcs-wif-trust-verification:v9",
        "service_account_binding_verified": True,
        "service_account_direct_policy_exact_verified": False,
        "forbidden_direct_service_account_roles_absent": False,
        "attribute_mapping_verified": True,
        "attribute_condition_verified": True,
        "provider_pool_exclusive": True,
        "service_account_project_verified": True,
        "oidc_audience_verified": True,
    }
    with TemporaryDirectory() as tmp:
        verification_path = Path(tmp) / "wif.json"
        verification_path.write_text(json.dumps(forbidden), encoding="utf-8")
        try:
            module.load_wif_verification(str(verification_path))
        except AssertionError:
            pass
        else:
            raise AssertionError("report loader accepted missing alternate-authority exclusion")

    print("offline GCS report resource-identity checks: PASS")


if __name__ == "__main__":
    main()
