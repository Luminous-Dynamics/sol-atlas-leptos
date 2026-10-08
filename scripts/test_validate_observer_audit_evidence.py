#!/usr/bin/env python3
# Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
# SPDX-License-Identifier: AGPL-3.0-or-later

"""Adversarial tests for observer audit evidence bound into the report."""

from __future__ import annotations

import importlib.util
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "qualify_gcs_external_effect.py"

spec = importlib.util.spec_from_file_location("qualify_report", SCRIPT)
assert spec and spec.loader
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)

PROFILE_PATH = ROOT / "sol-atlas-policy-store-contract/conformance/gcs_wif_trust_profile_v9.json"
PROFILE = json.loads(PROFILE_PATH.read_text(encoding="utf-8"))
PROFILE_DIGEST = module.digest(PROFILE)
PRINCIPAL = (
    "principalSet://iam.googleapis.com/projects/123456789/"
    "locations/global/workloadIdentityPools/github/"
    "attribute.repository_id/1195997641"
)
OBSERVER = "observer@sol-atlas.iam.gserviceaccount.com"
EFFECT = "qualification@sol-atlas.iam.gserviceaccount.com"

WIF = {
    "schema": "sol-atlas:gcs-wif-trust-verification:v9",
    "profile_path": module.WIF_TRUST_PROFILE_PATH,
    "profile_digest": PROFILE_DIGEST,
    "service_account": EFFECT,
    "service_account_binding_member": PRINCIPAL,
    "observer_service_account": OBSERVER,
    "observer_identity_verified": True,
    "observer_service_account_project_verified": True,
    "observer_service_account_direct_policy_exact_verified": True,
    "service_account_direct_policy_exact_verified": True,
    "attribute_mapping_verified": True,
    "attribute_condition_verified": True,
    "provider_pool_exclusive": True,
    "service_account_project_verified": True,
    "oidc_audience_verified": True,
    "oidc_expected_audience": "aud",
}
OBSERVER_OIDC = {
    "schema": module.OIDC_CLAIMS_SCHEMA,
    "claims": {"sub": "frozen-observer-subject"},
    "claims_digest": module.digest({"sub": "frozen-observer-subject"}),
    "audience": "aud",
}

EXPECTED_PRINCIPAL_SET_MEMBERS = (
    module.expected_effective_workload_principal_sets(
        PRINCIPAL,
        PROFILE,
    )
)


EFFECTIVE = {
    "schema": module.EFFECTIVE_IAM_AUDIT_SCHEMA,
    "wif_verification_digest": module.digest(WIF),
    "wif_profile_path": module.WIF_TRUST_PROFILE_PATH,
    "wif_profile_digest": PROFILE_DIGEST,
    "service_account": EFFECT,
    "observer_service_account": OBSERVER,
    "observer_identity_verified": True,
    "expected_principal": PRINCIPAL,
    "oidc_claims_digest": module.digest(OBSERVER_OIDC),
    "principal_selection_mode": (
        "permission_query_with_frozen_workload_principal_set_filter"
    ),
    "matched_workload_principal_sets": EXPECTED_PRINCIPAL_SET_MEMBERS,
    "fully_explored": True,
    "non_critical_errors": [],
    "forbidden_execution_permissions_absent": True,
    "required_permissions_verified": True,
    "project_pivot_permissions": list(
        module.EFFECTIVE_IAM_PROJECT_PIVOT_PERMISSIONS
    ),
    "project_pivot_response_digest": "sha256:" + "a" * 64,
    "policy_analyzer_response_digest": "sha256:" + "b" * 64,
    "project_pivot_permissions_absent": True,
    "project_pivot_findings": [],
    "observer_authority_permissions": module.OBSERVER_AUTHORITY_FORBIDDEN_PROJECT_PERMISSIONS,
    "observer_authority_permissions_absent": True,
    "observer_authority_observed_permissions": [],
    "observer_authority_response_digest": "sha256:" + "c" * 64,
    "findings": [{
        "role": "roles/iam.workloadIdentityUser",
        "members": [PRINCIPAL],
        "identities": [PRINCIPAL],
        "principal_match_kinds": ["exact"],
        "permissions": [
            "iam.serviceAccounts.getAccessToken",
            "iam.serviceAccounts.getOpenIdToken",
        ],
    }],
}
OBSERVER_ISOLATION = {
    "schema": module.OBSERVER_ISOLATION_AUDIT_SCHEMA,
    "observer_principal": "serviceAccount:" + OBSERVER,
    "observer_service_account": OBSERVER,
    "effect_service_account": EFFECT,
    "observed_permissions": [
        "iam.serviceAccounts.getIamPolicy"
    ],
    "required_observer_permission_verified": True,
    "forbidden_effect_permissions_absent": True,
    "forbidden_permissions": [
        "iam.serviceAccounts.actAs",
        "iam.serviceAccounts.getAccessToken",
        "iam.serviceAccounts.getOpenIdToken",
        "iam.serviceAccounts.implicitDelegation",
        "iam.serviceAccounts.signBlob",
        "iam.serviceAccounts.signJwt",
        "iam.serviceAccountKeys.create",
        "iam.serviceAccounts.setIamPolicy",
    ],
    "policy_analyzer_response_digest": "sha256:" + "d" * 64,
    "project_pivot_permissions": list(
        module.OBSERVER_ISOLATION_AUDIT_PROJECT_PIVOT_PERMISSIONS
        if hasattr(
            module,
            "OBSERVER_ISOLATION_AUDIT_PROJECT_PIVOT_PERMISSIONS",
        )
        else module.PROJECT_PIVOT_PERMISSIONS
    ),
    "project_pivot_permissions_absent": True,
    "project_pivot_observed_permissions": [],
    "project_pivot_response_digest": "sha256:" + "e" * 64,
    "observer_authority_permissions": (
        module.OBSERVER_AUTHORITY_FORBIDDEN_PROJECT_PERMISSIONS
    ),
    "observer_authority_permissions_absent": True,
    "observer_authority_observed_permissions": [],
    "observer_authority_response_digest": "sha256:" + "f" * 64,
    "claim_ceiling": "observer audit bounded",
}


BROAD = {
    "schema": module.BROAD_SA_AUDIT_SCHEMA,
    "wif_verification_digest": module.digest(WIF),
    "wif_profile_path": module.WIF_TRUST_PROFILE_PATH,
    "wif_profile_digest": PROFILE_DIGEST,
    "service_account": EFFECT,
    "observer_service_account": OBSERVER,
    "observer_identity_verified": True,
    "expected_principal": PRINCIPAL,
    "oidc_claims_digest": module.digest(OBSERVER_OIDC),
    "broad_impersonation_absent": True,
    "broad_impersonation_findings": [],
    "queried_permissions": [
        "iam.serviceAccounts.actAs",
        "iam.serviceAccounts.getAccessToken",
        "iam.serviceAccounts.getOpenIdToken",
        "iam.serviceAccounts.signBlob",
        "iam.serviceAccounts.signJwt",
        "iam.serviceAccounts.implicitDelegation",
        "iam.serviceAccountKeys.create",
        "iam.serviceAccounts.setIamPolicy",
    ],
}


def rejects(fn, value, label: str) -> None:
    try:
        fn(value, WIF, OBSERVER_OIDC)
    except AssertionError:
        return
    raise AssertionError("accepted tampered observer evidence: " + label)


def reject_observer_isolation(value, label: str) -> None:
    try:
        module.validate_observer_isolation_audit(
            value,
            WIF,
        )
    except AssertionError:
        return
    raise AssertionError(
        "accepted tampered observer isolation evidence: " + label
    )


def main() -> None:
    module.validate_effective_iam_audit(
        EFFECTIVE,
        WIF,
        OBSERVER_OIDC,
    )
    module.validate_broad_sa_audit(
        BROAD,
        WIF,
        OBSERVER_OIDC,
    )

    module.validate_observer_isolation_audit(
        OBSERVER_ISOLATION,
        WIF,
    )

    for field, value in (
        ("observer_authority_permissions", []),
        ("observer_authority_response_digest", "sha256:tampered"),
        ("project_pivot_permissions", []),
    ):
        tampered = dict(OBSERVER_ISOLATION)
        tampered[field] = value
        reject_observer_isolation(value=tampered, label=field)

    for field, value in (
        ("wif_verification_digest", "sha256:tampered"),
        ("wif_profile_digest", "sha256:tampered"),
        ("expected_principal", "principalSet://tampered"),
        ("oidc_claims_digest", "sha256:tampered"),
        ("principal_selection_mode", "weak-filter"),
        ("matched_workload_principal_sets", [PRINCIPAL]),
        ("project_pivot_permissions", []),
        ("observer_authority_permissions", []),
        ("observer_authority_response_digest", "sha256:tampered"),
        ("project_pivot_response_digest", "sha256:tampered"),
        ("policy_analyzer_response_digest", "sha256:tampered"),
    ):
        tampered = dict(EFFECTIVE)
        tampered[field] = value
        rejects(
            module.validate_effective_iam_audit,
            tampered,
            "effective " + field,
        )

    tampered_wif = dict(WIF, service_account=OBSERVER)
    rejects(
        module.validate_effective_iam_audit,
        EFFECTIVE,
        "effective audit baseline with invalid WIF is expected to remain bound",
    ) if False else None
    try:
        module.validate_effective_iam_audit(
            EFFECTIVE,
            tampered_wif,
            OBSERVER_OIDC,
        )
    except AssertionError:
        pass
    else:
        raise AssertionError("effective audit accepted WIF identity drift")

    for field, value in (
        ("wif_verification_digest", "sha256:tampered"),
        ("wif_profile_digest", "sha256:tampered"),
        ("expected_principal", "principalSet://tampered"),
        ("oidc_claims_digest", "sha256:tampered"),
    ):
        tampered = dict(BROAD)
        tampered[field] = value
        try:
            module.validate_broad_sa_audit(
                tampered,
                WIF,
                OBSERVER_OIDC,
            )
        except AssertionError:
            pass
        else:
            raise AssertionError("broad audit accepted " + field + " drift")

    tampered = dict(BROAD)
    tampered["queried_permissions"] = []
    try:
        module.validate_broad_sa_audit(
            tampered,
            WIF,
            OBSERVER_OIDC,
        )
    except AssertionError:
        pass
    else:
        raise AssertionError("broad audit accepted an incomplete permission set")

    print("observer audit evidence tamper checks: PASS")


if __name__ == "__main__":
    main()
