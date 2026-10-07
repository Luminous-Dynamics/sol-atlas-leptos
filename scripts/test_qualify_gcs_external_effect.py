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

PREFIX = "sol-atlas/qualification/run-1-attempt-1-abc"


def expect_rejection(report: dict[str, object], message: str) -> None:
    try:
        module.verify_resource_identity(report)
    except AssertionError:
        return
    raise AssertionError(message)


def main() -> None:
    assert module.SCHEMA == (
        "sol-atlas:recovery-execution-effect-external-report:v6"
    )
    good = {
        "service": "Google Cloud Storage",
        "bucket": "sol-atlas-qualification",
        "object_prefix": PREFIX,
        "object_names": [
            PREFIX + "/main.bin",
            PREFIX + "/point-in-time.bin",
            PREFIX + "/metadata-race.bin",
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
        "workflow_id_frozen": 311325850,
        "run_path_verified": ".github/workflows/qualify-gcs.yml@refs/heads/main",
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
            PREFIX + "/main.bin",
            PREFIX + "/point-in-time.bin",
            "other-prefix/metadata-race.bin",
        ]),
        "object prefix drift was accepted",
    )
    expect_rejection(
        dict(good, object_names=[
            PREFIX + "/main.bin",
            PREFIX + "/main.bin",
            PREFIX + "/metadata-race.bin",
        ]),
        "duplicate object identity was accepted",
    )


    profile_path = (
        "sol-atlas-policy-store-contract/conformance/"
        "gcs_wif_trust_profile_v8.json"
    )
    valid_wif = dict(
        schema="sol-atlas:gcs-wif-trust-verification:v8",
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
    )
    module.validate_wif_verification(valid_wif)
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
        "schema": "sol-atlas:gcs-wif-trust-verification:v8",
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
