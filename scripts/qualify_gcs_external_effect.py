#!/usr/bin/env python3
# Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
# SPDX-License-Identifier: AGPL-3.0-or-later

"""Real-service qualification harness for the GCS generation-fenced adapter."""

from __future__ import annotations

import argparse
import hashlib
import json
import math
import os
import subprocess
import uuid
from concurrent.futures import ThreadPoolExecutor
from datetime import datetime, timezone
from pathlib import Path

if __package__:
    from .gcs_generation_fenced_adapter import (
        GcsGenerationFencedObject,
        MutationRequest,
        access_token,
        sha256_prefixed,
    )
    from .verify_github_oidc_claims import expected_claims, verify_immutable_subject
else:
    from gcs_generation_fenced_adapter import (
        GcsGenerationFencedObject,
        MutationRequest,
        access_token,
        sha256_prefixed,
    )
    from verify_github_oidc_claims import expected_claims, verify_immutable_subject


SCHEMA = "sol-atlas:recovery-execution-effect-external-report:v8"
CASE_SET_PATH = (
    "sol-atlas-policy-store-contract/conformance/"
    "gcs_external_effect_cases_v2.json"
)
EXPECTED_CASE_SET = (
    "sol-atlas:recovery-execution-effect-external-conformance-cases:"
    "gcs-generation-v2"
)
EXPECTED_PREVIOUS_CASE_SET = (
    "sol-atlas:recovery-execution-effect-external-conformance-cases:"
    "gcs-generation-v1"
)
CASE_SET_SCHEMA = "sol-atlas:recovery-execution-effect-case-set:v1"
ADAPTER_PATH = "scripts/gcs_generation_fenced_adapter.py"
HARNESS_PATH = "scripts/qualify_gcs_external_effect.py"
WORKFLOW_PATH = ".github/workflows/qualify-gcs.yml"
ADAPTER_ID = "gcs-generation-fenced-object"
HARNESS_ID = "sol-atlas-gcs-external-conformance"
OIDC_CLAIMS_SCHEMA = "sol-atlas:github-oidc-claims:v5"
OIDC_SUBJECT_CONFIGURATION_SCHEMA = "sol-atlas:github-oidc-sub-configuration:v1"
WIF_CREDENTIAL_CONFIG_SCHEMA = (
    "sol-atlas:gcp-wif-credential-config-verification:v1"
)
GITHUB_RUN_VERIFICATION_SCHEMA = "sol-atlas:github-workflow-run-verification:v1"
EFFECTIVE_IAM_AUDIT_SCHEMA = "sol-atlas:gcs-wif-effective-iam-audit:v8"
EFFECTIVE_IAM_PROJECT_PIVOT_PERMISSIONS = [
    "cloudbuild.builds.create",
    "deploymentmanager.deployments.create",
    "compute.instances.create",
    "run.services.create",
    "run.jobs.create",
    "cloudfunctions.functions.create",
    "resourcemanager.projects.setIamPolicy",
]
BROAD_SA_AUDIT_SCHEMA = (
    "sol-atlas:gcs-broad-service-account-impersonation-audit:v2"
)
OBSERVER_ISOLATION_AUDIT_SCHEMA = "sol-atlas:gcs-observer-isolation-audit:v2"
POLICY_EFFECT_AUDIT_SCHEMA = "sol-atlas:gcs-policy-troubleshooter-audit:v2"
DIGEST_RE = re.compile(r"^sha256:[0-9a-f]{64}$")
OBSERVER_PROFILE_PATH = (
    "sol-atlas-policy-store-contract/conformance/"
    "gcs_iam_observer_profile_v1.json"
)
OBSERVER_PROFILE_SCHEMA = "sol-atlas:gcs-iam-observer-profile:v1"
OBSERVER_AUTHORITY_FORBIDDEN_PROJECT_PERMISSIONS = [
    "resourcemanager.projects.setIamPolicy",
    "resourcemanager.projects.update",
    "resourcemanager.projects.delete",
    "iam.roles.create",
    "iam.roles.update",
    "iam.roles.delete",
    "iam.denypolicies.create",
    "iam.denypolicies.update",
    "iam.denypolicies.delete",
    "iam.workloadIdentityPools.create",
    "iam.workloadIdentityPools.update",
    "iam.workloadIdentityPools.delete",
    "iam.workloadIdentityPools.setIamPolicy",
    "iam.workloadIdentityPoolProviders.create",
    "iam.workloadIdentityPoolProviders.update",
    "iam.workloadIdentityPoolProviders.delete",
    "iam.serviceAccounts.create",
    "iam.serviceAccounts.disable",
    "iam.serviceAccounts.delete",
    "iam.serviceAccounts.enable",
    "iam.serviceAccounts.setIamPolicy",
]
WIF_TRUST_PROFILE_PATH = (
    "sol-atlas-policy-store-contract/conformance/"
    "gcs_wif_trust_profile_v9.json"
)
WIF_TRUST_PROFILE_SCHEMA = "sol-atlas:gcs-wif-trust-profile:v9"
CLAIM_CEILING = (
    "GCS generation-precondition evidence only; replay safety applies while "
    "the qualified live object state remains retained; no universal "
    "exactly-once claim."
)
def load_case_set() -> dict[str, object]:
    case_set = json.loads(
        Path(CASE_SET_PATH).read_text(encoding="utf-8")
    )
    if (
        case_set.get("schema") != CASE_SET_SCHEMA
        or case_set.get("case_set") != EXPECTED_CASE_SET
        or case_set.get("supersedes") != EXPECTED_PREVIOUS_CASE_SET
    ):
        raise AssertionError("invalid GCS case-set identity or lineage")
    cases = case_set.get("cases")
    if not isinstance(cases, list) or not cases:
        raise AssertionError("GCS case-set has no cases")
    case_ids = [case.get("id") for case in cases if isinstance(case, dict)]
    if (
        len(case_ids) != len(cases)
        or any(not isinstance(case_id, str) or not case_id for case_id in case_ids)
        or len(set(case_ids)) != len(case_ids)
    ):
        raise AssertionError("GCS case-set has invalid or duplicate case IDs")
    return case_set


def case_ids(case_set: dict[str, object]) -> list[str]:
    return [case["id"] for case in case_set["cases"]]


def github_execution_context() -> dict[str, str]:
    required = {
        "repository": os.environ.get("GITHUB_REPOSITORY", ""),
        "repository_id": os.environ.get("GITHUB_REPOSITORY_ID", ""),
        "environment": os.environ.get("GITHUB_ENVIRONMENT", ""),
        "workflow": os.environ.get("GITHUB_WORKFLOW", ""),
        "workflow_ref": os.environ.get("GITHUB_WORKFLOW_REF", ""),
        "workflow_sha": os.environ.get("GITHUB_WORKFLOW_SHA", ""),
        "event": os.environ.get("GITHUB_EVENT_NAME", ""),
        "ref": os.environ.get("GITHUB_REF", ""),
        "run_id": os.environ.get("GITHUB_RUN_ID", ""),
        "sha": os.environ.get("GITHUB_SHA", ""),
        "run_attempt": os.environ.get("GITHUB_RUN_ATTEMPT", ""),
        "runner_environment": os.environ.get("RUNNER_ENVIRONMENT", ""),
    }
    missing = [name for name, value in required.items() if not value]
    if missing:
        raise AssertionError(
            "missing GitHub execution context: " + ", ".join(missing)
        )
    expected = {
        "repository": "Luminous-Dynamics/sol-atlas-leptos",
        "repository_id": "1195997641",
        "environment": "sol-atlas-gcs-qualification",
        "workflow": "Qualify GCS external effect",
        "event": "workflow_dispatch",
        "ref": "refs/heads/main",
        "runner_environment": "github-hosted",
        "workflow_ref": (
            "Luminous-Dynamics/sol-atlas-leptos/.github/workflows/"
            "qualify-gcs.yml@refs/heads/main"
        ),
    }
    for name, expected_value in expected.items():
        if required[name] != expected_value:
            raise AssertionError(
                f"GitHub execution context drift for {name}: "
                f"{required[name]!r} != {expected_value!r}"
            )
    return required


def verify_checked_out_source_commit() -> str:
    expected = os.environ.get("GITHUB_SHA", "")
    if not expected:
        raise AssertionError("GITHUB_SHA is missing")
    actual = git_head()
    if actual != expected:
        raise AssertionError(
            f"checked-out source drift: {actual!r} != GITHUB_SHA {expected!r}"
        )
    return actual


def load_github_workflow_run_verification(path: str) -> dict[str, object]:
    verification = json.loads(Path(path).read_text(encoding="utf-8"))
    if verification.get("schema") != GITHUB_RUN_VERIFICATION_SCHEMA:
        raise AssertionError("wrong GitHub workflow-run verification schema")
    required = {
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
    }
    for name, expected in required.items():
        if verification.get(name) != expected:
            raise AssertionError(
                f"GitHub workflow-run verification mismatch for {name}"
            )
    if verification.get("context_sha_matches_server") is not True:
        raise AssertionError("GitHub workflow-run SHA was not server-verified")
    if verification.get("context_run_attempt_matches_server") is not True:
        raise AssertionError(
            "GitHub workflow-run attempt was not server-verified"
        )
    if verification.get("referenced_workflows") not in ([], None):
        raise AssertionError(
            "unexpected reusable workflow evidence in direct qualification run"
        )
    return verification


def validate_wif_verification(verification: dict[str, object]) -> None:
    if verification.get("schema") != "sol-atlas:gcs-wif-trust-verification:v9":
        raise AssertionError("wrong WIF trust verification schema")
    if verification.get("profile_path") != WIF_TRUST_PROFILE_PATH:
        raise AssertionError("WIF verification profile path drift")
    profile = json.loads(
        Path(WIF_TRUST_PROFILE_PATH).read_text(encoding="utf-8")
    )
    if profile.get("schema") != WIF_TRUST_PROFILE_SCHEMA:
        raise AssertionError("checked-in WIF trust profile schema drift")
    if verification.get("profile_digest") != digest(profile):
        raise AssertionError("WIF verification profile digest drift")
    if not verification.get("attribute_mapping_verified"):
        raise AssertionError("WIF attribute mapping was not verified")
    if not verification.get("attribute_condition_verified"):
        raise AssertionError("WIF attribute condition was not verified")
    if not verification.get("service_account_binding_verified"):
        raise AssertionError("WIF service-account binding was not verified")
    binding_member = verification.get("service_account_binding_member")
    if not isinstance(binding_member, str) or not binding_member:
        raise AssertionError(
            "WIF trust evidence lacks exact service-account binding member"
        )
    provider_resource = verification.get("provider_resource")
    if not isinstance(provider_resource, str):
        raise AssertionError("WIF provider resource is missing")
    provider_parts = provider_resource.split("/")
    if (
        len(provider_parts) != 8
        or provider_parts[0] != "projects"
        or provider_parts[2] != "locations"
        or provider_parts[3] != "global"
        or provider_parts[4] != "workloadIdentityPools"
        or provider_parts[6] != "providers"
    ):
        raise AssertionError("WIF provider resource shape drift")
    binding = profile.get("required_service_account_binding")
    if not isinstance(binding, dict):
        raise AssertionError("WIF profile binding is missing")
    template = binding.get("member_template")
    if not isinstance(template, str):
        raise AssertionError("WIF profile member template is missing")
    expected_member = template.format(
        project_number=provider_parts[1],
        pool_id=provider_parts[5],
    )
    if binding_member != expected_member:
        raise AssertionError("WIF service-account binding member drift")
    if not verification.get("service_account_direct_policy_exact_verified"):
        raise AssertionError(
            "WIF direct service-account policy was not verified exact"
        )
    if not verification.get("forbidden_direct_service_account_roles_absent"):
        raise AssertionError(
            "WIF direct service-account alternate authority roles were not excluded"
        )
    if not verification.get("provider_pool_exclusive"):
        raise AssertionError("WIF provider pool was not verified exclusive")
    if not verification.get("service_account_project_verified"):
        raise AssertionError("WIF service account project was not verified")
    observer = verification.get("observer_service_account")
    target = verification.get("service_account")
    if observer == target:
        raise AssertionError("WIF observer and effect identities must differ")
    if (
        verification.get("observer_identity_verified") is not True
        or not isinstance(observer, str)
        or not observer
        or observer == target
    ):
        raise AssertionError(
            "WIF trust evidence lacks a distinct verified IAM observer"
        )
    if not verification.get("observer_service_account_project_verified"):
        raise AssertionError(
            "WIF observer service-account project was not verified"
        )
    if not verification.get(
        "observer_service_account_direct_policy_exact_verified"
    ):
        raise AssertionError(
            "WIF observer service-account direct policy was not verified exact"
        )
    if not verification.get("observer_service_account_project_verified"):
        raise AssertionError(
            "WIF observer service-account project was not verified"
        )
    if not verification.get(
        "observer_service_account_direct_policy_exact_verified"
    ):
        raise AssertionError(
            "WIF observer service-account direct policy was not verified exact"
        )
    if not verification.get("oidc_audience_verified"):
        raise AssertionError("WIF OIDC audience was not verified")


def load_wif_verification(path: str) -> dict[str, object]:
    verification = json.loads(Path(path).read_text(encoding="utf-8"))
    if not isinstance(verification, dict):
        raise AssertionError("WIF trust verification is not an object")
    validate_wif_verification(verification)
    return verification


def expected_effective_workload_principal_sets(
    expected_principal: str,
    profile: dict[str, object],
) -> list[str]:
    marker = "/attribute.repository_id/"
    if marker not in expected_principal:
        raise AssertionError("effective-IAM principal is not repository-ID based")
    prefix = expected_principal.split(marker, 1)[0]
    suffix = expected_principal.split(prefix + "/", 1)[1]
    repository_id = suffix.split("/", 1)[1]

    condition = profile.get("exact_attribute_condition")
    mappings = profile.get("required_attribute_mappings")
    if not isinstance(condition, str) or not isinstance(mappings, dict):
        raise AssertionError("WIF profile principal-set inputs are incomplete")

    mapped_names = {
        key.removeprefix("attribute.")
        for key in mappings
        if isinstance(key, str) and key.startswith("attribute.")
    }
    non_attribute_mappings = {
        key
        for key in mappings
        if key != "google.subject" and not key.startswith("attribute.")
    }
    if non_attribute_mappings:
        raise AssertionError(
            "WIF profile maps an unsupported Google claim"
        )
    values: dict[str, str] = {}
    for clause in condition.split(" && "):
        if not clause.startswith("assertion.") or "==" not in clause:
            raise AssertionError("WIF profile condition contains an unexpected clause")
        name, value = clause.split("==", 1)
        key = name.removeprefix("assertion.")
        values[key] = value.strip().strip("'")

    required_names = {
        "environment",
        "event_name",
        "repository",
        "repository_id",
        "repository_owner_id",
        "workflow",
        "ref",
        "workflow_ref",
        "runner_environment",
    }
    if required_names - set(values):
        raise AssertionError("WIF profile principal-set condition is incomplete")
    if required_names != mapped_names:
        raise AssertionError(
            "WIF profile mapped attributes must exactly match its condition"
        )
    if values["repository_id"] != repository_id:
        raise AssertionError("effective-IAM principal disagrees with WIF profile")

    members = {expected_principal, prefix + "/*"}
    for name in sorted(required_names):
        members.add(
            prefix
            + "/attribute."
            + name
            + "/"
            + values[name]
        )
    return sorted(members)


def is_sha256_digest(value: object) -> bool:
    return (
        isinstance(value, str)
        and len(value) == 71
        and value.startswith("sha256:")
        and all(character in "0123456789abcdef" for character in value[7:])
    )


def validate_effective_iam_audit(
    audit: dict[str, object],
    wif_verification: dict[str, object],
    observer_oidc_claims: dict[str, object],
) -> None:
    if audit.get("schema") != EFFECTIVE_IAM_AUDIT_SCHEMA:
        raise AssertionError("wrong effective-IAM audit schema")
    if audit.get("wif_verification_digest") != digest(wif_verification):
        raise AssertionError("effective-IAM WIF evidence digest drift")
    if audit.get("wif_profile_path") != WIF_TRUST_PROFILE_PATH:
        raise AssertionError("effective-IAM profile path drift")
    profile_path = Path(WIF_TRUST_PROFILE_PATH)
    profile = json.loads(profile_path.read_text(encoding="utf-8"))
    if profile.get("schema") != WIF_TRUST_PROFILE_SCHEMA:
        raise AssertionError("effective-IAM WIF profile schema drift")
    expected_profile_digest = digest(profile)
    if audit.get("wif_profile_digest") != expected_profile_digest:
        raise AssertionError("effective-IAM profile digest drift")
    if audit.get("wif_profile_digest") != wif_verification.get(
        "profile_digest"
    ):
        raise AssertionError("effective-IAM WIF profile digest disagreement")
    if audit.get("principal_selection_mode") != (
        "permission_query_with_frozen_workload_principal_set_filter"
    ):
        raise AssertionError("effective-IAM principal selection mode drift")
    expected_sets = expected_effective_workload_principal_sets(
        wif_verification.get("service_account_binding_member", ""),
        profile,
    )
    if audit.get("matched_workload_principal_sets") != expected_sets:
        raise AssertionError("effective-IAM principal-set universe drift")
    if audit.get("service_account") != wif_verification.get(
        "service_account"
    ):
        raise AssertionError("effective-IAM service-account drift")
    if audit.get("observer_service_account") != wif_verification.get(
        "observer_service_account"
    ):
        raise AssertionError("effective-IAM observer identity drift")
    if audit.get("observer_identity_verified") is not True:
        raise AssertionError("effective-IAM observer identity was not verified")
    if audit.get("expected_principal") != wif_verification.get(
        "service_account_binding_member"
    ):
        raise AssertionError("effective-IAM principal root drift")
    if audit.get("oidc_claims_digest") != digest(observer_oidc_claims):
        raise AssertionError("effective-IAM OIDC evidence digest drift")
    observed_oidc_claims = observer_oidc_claims.get("claims")
    if not isinstance(observed_oidc_claims, dict):
        raise AssertionError("effective-IAM observer OIDC claims are missing")
    if audit.get("oidc_subject") != observed_oidc_claims.get("sub"):
        raise AssertionError("effective-IAM immutable subject drift")
    if audit.get("fully_explored") is not True:
        raise AssertionError("effective-IAM audit was not fully explored")
    if audit.get("non_critical_errors") not in ([], None):
        raise AssertionError("effective-IAM audit contains errors")
    if audit.get("forbidden_execution_permissions_absent") is not True:
        raise AssertionError("effective-IAM observer permissions were not excluded")
    if audit.get("required_permissions_verified") is not True:
        raise AssertionError("effective-IAM required permissions were not verified")
    if audit.get("project_pivot_permissions_absent") is not True:
        raise AssertionError("effective-IAM project pivots were not excluded")
    if audit.get("project_pivot_findings") != []:
        raise AssertionError("effective-IAM contains project pivot findings")
    findings = audit.get("findings")
    if not isinstance(findings, list) or len(findings) != 1:
        raise AssertionError("effective-IAM does not have exactly one intended binding")
    finding = findings[0]
    if not isinstance(finding, dict):
        raise AssertionError("effective-IAM finding is not an object")
    if finding.get("role") != "roles/iam.workloadIdentityUser":
        raise AssertionError("effective-IAM intended binding role drift")
    if finding.get("members") != [wif_verification.get(
        "service_account_binding_member"
    )]:
        raise AssertionError("effective-IAM intended binding member drift")
    if finding.get("principal_match_kinds") != ["exact"]:
        raise AssertionError("effective-IAM intended binding principal match drift")
    if finding.get("identities") != [
        wif_verification.get("service_account_binding_member")
    ]:
        raise AssertionError("effective-IAM intended identity drift")

    if audit.get("project_pivot_permissions") != (
        EFFECTIVE_IAM_PROJECT_PIVOT_PERMISSIONS
    ):
        raise AssertionError("effective-IAM pivot permission set drift")
    if not is_sha256_digest(audit.get("project_pivot_response_digest")):
        raise AssertionError("effective-IAM pivot response digest is malformed")
    if not is_sha256_digest(audit.get("policy_analyzer_response_digest")):
        raise AssertionError("effective-IAM analyzer response digest is malformed")


def validate_observer_isolation_audit(
    audit: dict[str, object],
    wif_verification: dict[str, object],
) -> None:
    if audit.get("observer_profile_path") != OBSERVER_PROFILE_PATH:
        raise AssertionError("observer isolation profile path drift")
    observer_profile_path = Path(OBSERVER_PROFILE_PATH)
    observer_profile = json.loads(
        observer_profile_path.read_text(encoding="utf-8")
    )
    if observer_profile.get("schema") != OBSERVER_PROFILE_SCHEMA:
        raise AssertionError("observer isolation profile schema drift")
    if audit.get("observer_profile_digest") != digest(
        observer_profile
    ):
        raise AssertionError("observer isolation profile digest drift")
    if audit.get("schema") != OBSERVER_ISOLATION_AUDIT_SCHEMA:
        raise AssertionError("wrong observer isolation audit schema")
    observer = wif_verification.get("observer_service_account")
    effect = wif_verification.get("service_account")
    if audit.get("observer_service_account") != observer:
        raise AssertionError("observer isolation observer identity drift")
    if audit.get("effect_service_account") != effect:
        raise AssertionError("observer isolation effect identity drift")
    if not isinstance(observer, str) or not observer:
        raise AssertionError("WIF observer identity is missing")
    expected_principal = "serviceAccount:" + observer
    if audit.get("observer_principal") != expected_principal:
        raise AssertionError("observer isolation principal drift")
    if audit.get("required_observer_permission_verified") is not True:
        raise AssertionError("observer IAM read privilege was not verified")
    if audit.get("forbidden_effect_permissions_absent") is not True:
        raise AssertionError("observer effect privileges were not excluded")
    observed = audit.get("observed_permissions")
    if not isinstance(observed, list):
        raise AssertionError("observer isolation permissions are malformed")
    if "iam.serviceAccounts.getIamPolicy" not in observed:
        raise AssertionError("observer IAM policy-read permission is missing")
    forbidden = audit.get("forbidden_permissions")
    if not isinstance(forbidden, list):
        raise AssertionError("observer forbidden permission list is malformed")
    if set(observed).intersection(
        str(permission) for permission in forbidden
    ):
        raise AssertionError("observer isolation contains forbidden permissions")
    digest_value = audit.get("policy_analyzer_response_digest")
    if not is_sha256_digest(digest_value):
        raise AssertionError("observer isolation response digest is malformed")
    if audit.get("project_pivot_permissions") != [
        "cloudbuild.builds.create",
        "deploymentmanager.deployments.create",
        "compute.instances.create",
        "run.services.create",
        "run.jobs.create",
        "cloudfunctions.functions.create",
        "resourcemanager.projects.setIamPolicy",
    ]:
        raise AssertionError("observer isolation pivot permission set drift")
    if audit.get("project_pivot_permissions_absent") is not True:
        raise AssertionError(
            "observer isolation project pivots were not excluded"
        )
    if audit.get("project_pivot_observed_permissions") != []:
        raise AssertionError(
            "observer isolation contains project pivot permissions"
        )
    if not is_sha256_digest(audit.get("project_pivot_response_digest")):
        raise AssertionError(
            "observer isolation project pivot digest is malformed"
        )
    expected_observer_profile_permissions = observer_profile.get(
        "forbidden_project_permissions"
    )
    if audit.get("observer_authority_permissions") != (
        expected_observer_profile_permissions
    ):
        raise AssertionError(
            "observer authority permission set drift"
        )
    if audit.get("observer_authority_permissions_absent") is not True:
        raise AssertionError(
            "observer authority permissions were not excluded"
        )
    if audit.get("observer_authority_observed_permissions") != []:
        raise AssertionError(
            "observer authority findings were present"
        )
    if not is_sha256_digest(
        audit.get("observer_authority_response_digest")
    ):
        raise AssertionError(
            "observer authority response digest is malformed"
        )
    claim = audit.get("claim_ceiling")
    if not isinstance(claim, str) or not claim:
        raise AssertionError("observer isolation claim ceiling is missing")


def validate_broad_sa_audit(
    audit: dict[str, object],
    wif_verification: dict[str, object],
    observer_oidc_claims: dict[str, object],
) -> None:
    if audit.get("schema") != BROAD_SA_AUDIT_SCHEMA:
        raise AssertionError("wrong broad service-account audit schema")
    if audit.get("wif_verification_digest") != digest(wif_verification):
        raise AssertionError("broad-SA WIF evidence digest drift")
    if audit.get("wif_profile_path") != WIF_TRUST_PROFILE_PATH:
        raise AssertionError("broad-SA profile path drift")
    if audit.get("wif_profile_digest") != wif_verification.get(
        "profile_digest"
    ):
        raise AssertionError("broad-SA profile digest drift")
    if audit.get("service_account") != wif_verification.get(
        "service_account"
    ):
        raise AssertionError("broad-SA service-account drift")
    if audit.get("observer_service_account") != wif_verification.get(
        "observer_service_account"
    ):
        raise AssertionError("broad-SA observer identity drift")
    if audit.get("observer_identity_verified") is not True:
        raise AssertionError("broad-SA observer identity was not verified")
    if audit.get("expected_principal") != wif_verification.get(
        "service_account_binding_member"
    ):
        raise AssertionError("broad-SA principal root drift")
    if audit.get("oidc_claims_digest") != digest(observer_oidc_claims):
        raise AssertionError("broad-SA OIDC evidence digest drift")
    if audit.get("broad_impersonation_absent") is not True:
        raise AssertionError("broad-SA impersonation was not excluded")
    if audit.get("broad_impersonation_findings") != []:
        raise AssertionError("broad-SA contains impersonation findings")
    queried = audit.get("queried_permissions")
    required = {
        "iam.serviceAccounts.actAs",
        "iam.serviceAccounts.getAccessToken",
        "iam.serviceAccounts.getOpenIdToken",
        "iam.serviceAccounts.signBlob",
        "iam.serviceAccounts.signJwt",
        "iam.serviceAccounts.implicitDelegation",
        "iam.serviceAccountKeys.create",
        "iam.serviceAccounts.setIamPolicy",
    }
    if not isinstance(queried, list) or not required.issubset(set(queried)):
        raise AssertionError("broad-SA audit permission set is incomplete")


def validate_observer_oidc_claims(
    claims: dict[str, object],
    github_context: dict[str, str],
    wif_verification: dict[str, object],
) -> None:
    oidc_audience = wif_verification.get("oidc_expected_audience")
    if not isinstance(oidc_audience, str) or not oidc_audience:
        raise AssertionError("observer WIF audience is missing")
    observed = claims.get("claims")
    if not isinstance(observed, dict):
        raise AssertionError("missing observer OIDC claims")
    if claims.get("schema") != OIDC_CLAIMS_SCHEMA:
        raise AssertionError("wrong observer OIDC claims schema")
    if claims.get("claims_digest") != digest(observed):
        raise AssertionError("observer OIDC claims digest mismatch")
    if claims.get("audience") != observed.get("aud"):
        raise AssertionError("observer OIDC audience mismatch")
    verify_oidc_claim_identity(observed, github_context, oidc_audience)
    verify_oidc_temporal_evidence(claims)


def load_observer_oidc_claims(path: str) -> dict[str, object]:
    claims = load_oidc_claims(path)
    return claims


def verify_oidc_temporal_evidence(claims: dict[str, object]) -> None:
    observed = claims.get("claims")
    if not isinstance(observed, dict):
        raise AssertionError("missing observed temporal OIDC claims")
    values = {}
    for name in ("iat", "exp", "nbf"):
        value = observed.get(name)
        if isinstance(value, bool) or not isinstance(value, (int, float)):
            raise AssertionError(f"OIDC {name} claim is not numeric")
        values[name] = float(value)
    verified_at = claims.get("verified_at_unix")
    clock_skew = claims.get("clock_skew_seconds")
    if (
        isinstance(verified_at, bool)
        or not isinstance(verified_at, (int, float))
        or isinstance(clock_skew, bool)
        or not isinstance(clock_skew, (int, float))
        or not math.isfinite(float(verified_at))
        or not math.isfinite(float(clock_skew))
        or float(clock_skew) != 60.0
    ):
        raise AssertionError("invalid OIDC temporal verification metadata")
    verified_at = float(verified_at)
    clock_skew = float(clock_skew)
    recorded_at = claims.get("verified_at")
    if not isinstance(recorded_at, str) or not recorded_at:
        raise AssertionError("missing OIDC verification timestamp")
    try:
        parsed_recorded_at = datetime.fromisoformat(recorded_at)
    except ValueError as exc:
        raise AssertionError("OIDC verification timestamp is not ISO-8601") from exc
    if parsed_recorded_at.tzinfo is None:
        raise AssertionError("OIDC verification timestamp has no timezone")
    if not math.isclose(
        parsed_recorded_at.timestamp(),
        verified_at,
        rel_tol=0.0,
        abs_tol=1e-6,
    ):
        raise AssertionError("OIDC verification timestamps disagree")
    if values["exp"] <= values["iat"] or values["nbf"] > values["exp"]:
        raise AssertionError("OIDC temporal claims have no valid interval")
    if values["iat"] > verified_at + clock_skew:
        raise AssertionError("OIDC iat is later than the recorded verification time")
    if values["nbf"] > verified_at + clock_skew:
        raise AssertionError("OIDC nbf is later than the recorded verification time")
    if values["exp"] <= verified_at - clock_skew:
        raise AssertionError("OIDC exp predates the recorded verification time")


def load_wif_credential_config_verification(
    path: str,
    wif_verification: dict[str, object],
    oidc_claims: dict[str, object],
) -> dict[str, object]:
    verification = json.loads(Path(path).read_text(encoding="utf-8"))
    if verification.get("schema") != WIF_CREDENTIAL_CONFIG_SCHEMA:
        raise AssertionError("wrong WIF credential-config verification schema")
    if verification.get("exact_verified_token_bound") is not True:
        raise AssertionError("WIF credential config is not bound to the verified token")
    if verification.get("provider_resource") != wif_verification.get(
        "provider_resource"
    ):
        raise AssertionError("WIF credential config provider resource drift")
    if verification.get("service_account") != wif_verification.get(
        "service_account"
    ):
        raise AssertionError("WIF credential config service-account drift")
    if verification.get("credential_audience") != wif_verification.get(
        "oidc_expected_audience"
    ):
        raise AssertionError("WIF credential config audience drift")
    if verification.get("token_digest") != oidc_claims.get("token_digest"):
        raise AssertionError("WIF credential config token digest drift")
    return verification


def verify_oidc_claim_identity(
    observed_claims: dict[str, object],
    github_context: dict[str, str],
    audience: str,
) -> None:
    context = {
        "GITHUB_WORKFLOW_SHA": github_context["workflow_sha"],
        "GITHUB_SHA": github_context["sha"],
        "GITHUB_RUN_ID": github_context["run_id"],
        "GITHUB_RUN_ATTEMPT": github_context["run_attempt"],
        "RUNNER_ENVIRONMENT": github_context["runner_environment"],
    }
    expected = expected_claims(audience, context)
    for name, expected_value in expected.items():
        if observed_claims.get(name) != expected_value:
            raise AssertionError(
                f"OIDC claim mismatch in report for {name}: "
                f"{observed_claims.get(name)!r} != {expected_value!r}"
            )
    if not isinstance(observed_claims.get("sub"), str):
        raise AssertionError("OIDC immutable subject is missing from report")
    verify_immutable_subject(observed_claims)


def load_oidc_claims(path: str) -> dict[str, object]:
    claims = json.loads(Path(path).read_text(encoding="utf-8"))
    if claims.get("schema") != OIDC_CLAIMS_SCHEMA:
        raise AssertionError("wrong GitHub OIDC claims schema")
    observed = claims.get("claims")
    if not isinstance(observed, dict) or not observed:
        raise AssertionError("missing GitHub OIDC claims")
    if claims.get("audience") != observed.get("aud"):
        raise AssertionError("OIDC artifact audience does not match its claims")
    verify_immutable_subject(observed)
    if claims.get("claims_digest") != digest(observed):
        raise AssertionError("GitHub OIDC claims digest mismatch")
    if claims.get("cryptographic_verification") != "not_performed_locally":
        raise AssertionError("unexpected local OIDC cryptographic-verification mode")
    if claims.get("wif_exchange_uses_this_exact_verified_token") is not True:
        raise AssertionError("OIDC evidence is not bound to the WIF token source")
    if claims.get("workflow_sha_matches_runner") is not True:
        raise AssertionError("OIDC workflow_sha does not match runner context")
    if claims.get("source_sha_matches_runner") is not True:
        raise AssertionError("OIDC sha does not match runner context")
    if claims.get("temporal_claims_valid") is not True:
        raise AssertionError("OIDC temporal claims were not validated")
    verify_oidc_temporal_evidence(claims)
    return claims


def validate_oidc_subject_configuration(
    configuration: dict[str, object],
) -> None:
    if configuration.get("schema") != OIDC_SUBJECT_CONFIGURATION_SCHEMA:
        raise AssertionError("wrong OIDC subject configuration schema")
    if configuration.get("repository") != "Luminous-Dynamics/sol-atlas-leptos":
        raise AssertionError("OIDC subject configuration repository drift")
    raw = configuration.get("configuration")
    if not isinstance(raw, dict):
        raise AssertionError("OIDC subject raw configuration is missing")
    if configuration.get("configuration_digest") != digest(raw):
        raise AssertionError("OIDC subject raw configuration digest mismatch")
    if raw.get("use_default") is not True:
        raise AssertionError("OIDC raw subject default template was not verified")
    if raw.get("use_immutable_subject") is not True:
        raise AssertionError(
            "OIDC raw immutable subject mode was not verified"
        )
    if raw.get("include_claim_keys") != []:
        raise AssertionError(
            "OIDC raw subject claim customization was not excluded"
        )
    if configuration.get("use_default") != raw.get("use_default"):
        raise AssertionError("OIDC subject summary/default drift")
    if configuration.get("use_immutable_subject") != raw.get(
        "use_immutable_subject"
    ):
        raise AssertionError("OIDC subject summary/immutable drift")
    if configuration.get("include_claim_keys") != raw.get(
        "include_claim_keys"
    ):
        raise AssertionError("OIDC subject summary/claims drift")
    if configuration.get("default_template_required") is not True:
        raise AssertionError("OIDC default-template assertion is missing")
    if configuration.get("custom_claim_keys_forbidden") is not True:
        raise AssertionError("OIDC custom-claim assertion is missing")
    if not is_sha256_digest(configuration.get("configuration_digest")):
        raise AssertionError("OIDC subject configuration digest is malformed")


def load_oidc_subject_configuration(path: str) -> dict[str, object]:
    configuration = json.loads(Path(path).read_text(encoding="utf-8"))
    if not isinstance(configuration, dict):
        raise AssertionError("OIDC subject configuration is not an object")
    validate_oidc_subject_configuration(configuration)
    return configuration


def validate_case_set() -> str:
    case_set = load_case_set()
    for case in case_set["cases"]:
        if (
            not isinstance(case, dict)
            or not isinstance(case.get("id"), str)
            or not isinstance(case.get("semantic"), str)
            or not case["semantic"].strip()
        ):
            raise AssertionError("GCS case-set contains an incomplete case")
    case_set_digest = digest(case_set)
    print(
        "verified case-set: "
        + EXPECTED_CASE_SET
        + " "
        + case_set_digest
    )
    return case_set_digest


def git_sha(path: str) -> str:
    result = subprocess.run(
        ["git", "rev-parse", "HEAD:" + path],
        check=True,
        capture_output=True,
        text=True,
    )
    return result.stdout.strip()


def git_sha_at_commit(commit: str, path: str) -> str:
    result = subprocess.run(
        ["git", "rev-parse", f"{commit}:{path}"],
        check=True,
        capture_output=True,
        text=True,
    )
    return result.stdout.strip()


def git_head() -> str:
    result = subprocess.run(
        ["git", "rev-parse", "HEAD"],
        check=True,
        capture_output=True,
        text=True,
    )
    return result.stdout.strip()


def profile() -> dict[str, str]:
    return {
        "schema": "sol-atlas:recovery-execution-effect-safety-profile:v1",
        "fencing": "EnforcedAtMutationBoundary",
        "idempotency": "StableKey",
        "reconciliation": "StrongReadBack",
        "claim_ceiling": CLAIM_CEILING,
    }


def profile_digest(value: dict[str, str]) -> str:
    material = (
        f"schema={len(value['schema'])}:{value['schema']}|"
        "fencing=enforced|"
        "idempotency=stable-key|"
        "reconciliation=strong-read-back|"
        f"claim={len(value['claim_ceiling'])}:{value['claim_ceiling']}"
    )
    return "sha256:" + hashlib.sha256(material.encode()).hexdigest()


def canonical(value: object) -> bytes:
    return json.dumps(
        value,
        sort_keys=True,
        separators=(",", ":"),
        ensure_ascii=True,
    ).encode("utf-8")


def digest(value: object) -> str:
    return "sha256:" + hashlib.sha256(canonical(value)).hexdigest()


def request(
    name: str,
    generation: int,
    *,
    execution: str | None = None,
    key: str | None = None,
    body: bytes | None = None,
) -> MutationRequest:
    payload = body or f"sol-atlas:{name}:v1".encode()
    execution_id = execution or f"gcs-{name}-{uuid.uuid4().hex[:12]}"
    idempotency_key = key or f"gcs-key-{name}-{uuid.uuid4().hex[:12]}"
    return MutationRequest(
        execution_id=execution_id,
        input_fingerprint=sha256_prefixed(payload),
        attempt_id=f"attempt-{name}",
        fence_generation=generation,
        idempotency_key=idempotency_key,
        body=payload,
    )


def point_in_time_semantics(
    before: str,
    mutation: str,
    after: str,
) -> bool:
    return (
        before == "ObservedNotApplied"
        and mutation == "Applied"
        and after == "ObservedAppliedSameRequest"
    )


def create_setup(resource: GcsGenerationFencedObject, name: str) -> int:
    setup = request(
        name,
        0,
        execution=f"setup-{name}-{uuid.uuid4().hex[:12]}",
        key=f"setup-key-{name}-{uuid.uuid4().hex[:12]}",
    )
    result = resource.raw_put(setup)
    if result.status not in {200, 201}:
        raise RuntimeError(
            f"setup object creation failed with HTTP {result.status}"
        )
    state = resource.state()
    if state is None:
        raise RuntimeError("setup object was not observable after creation")
    return state.generation


def record(case_id: str, passed: bool, observed: dict[str, object]) -> dict[str, object]:
    return {
        "id": case_id,
        "passed": passed,
        "observed": observed,
    }


def run_qualification(
    bucket: str,
    object_prefix: str,
    wif_verification_path: str,
    oidc_claims_path: str,
    oidc_subject_configuration_path: str,
    wif_credential_config_verification_path: str,
    github_run_verification_path: str,
    observer_oidc_claims_path: str,
    effective_iam_audit_path: str,
    broad_sa_audit_path: str,
    observer_isolation_audit_path: str,
    policy_effect_audit_path: str,
    object_root: str | None,
) -> dict[str, object]:
    case_set = load_case_set()
    wif_verification = load_wif_verification(wif_verification_path)
    github_run_verification = load_github_workflow_run_verification(
        github_run_verification_path
    )
    github_context = github_execution_context()
    oidc_claims = load_oidc_claims(oidc_claims_path)
    oidc_subject_configuration = load_oidc_subject_configuration(oidc_subject_configuration_path)
    observer_oidc_claims = load_observer_oidc_claims(
        observer_oidc_claims_path
    )
    validate_observer_oidc_claims(
        observer_oidc_claims,
        github_context,
        wif_verification,
    )
    effective_iam_audit = json.loads(
        Path(effective_iam_audit_path).read_text(encoding="utf-8")
    )
    broad_sa_audit = json.loads(
        Path(broad_sa_audit_path).read_text(encoding="utf-8")
    )
    observer_isolation_audit = json.loads(
        Path(observer_isolation_audit_path).read_text(encoding="utf-8")
    )
    policy_effect_audit = json.loads(
        Path(policy_effect_audit_path).read_text(encoding="utf-8")
    )
    validate_effective_iam_audit(
        effective_iam_audit,
        wif_verification,
        observer_oidc_claims,
    )
    validate_broad_sa_audit(
        broad_sa_audit,
        wif_verification,
        observer_oidc_claims,
    )
    validate_observer_isolation_audit(
        observer_isolation_audit,
        wif_verification,
    )
    run_id = os.environ.get("GITHUB_RUN_ID", "local")
    run_attempt = os.environ.get("GITHUB_RUN_ATTEMPT", "1")
    expected_root = expected_object_root(
        object_prefix,
        run_id,
        run_attempt,
    )
    if object_root != expected_root:
        raise AssertionError("qualification object root is not bound to this run")
    if object_root is None:
        raise AssertionError("qualification object root is required")
    root = object_root
    main_name = root + "/main.bin"
    point_name = root + "/point-in-time.bin"
    race_name = root + "/metadata-race.bin"
    validate_policy_effect_audit(
        policy_effect_audit,
        wif_verification,
        bucket,
        [main_name, point_name, race_name],
        str(wif_verification.get("project_id", "")),
    )
    oidc_audience = wif_verification.get("oidc_expected_audience")
    if not isinstance(oidc_audience, str) or not oidc_audience:
        raise AssertionError("WIF verification has no OIDC expected audience")
    observed_claims = oidc_claims.get("claims")
    if not isinstance(observed_claims, dict):
        raise AssertionError("missing observed GitHub OIDC claims")
    if oidc_claims.get("audience") != observed_claims.get("aud"):
        raise AssertionError("OIDC artifact audience does not match its claims")
    verify_oidc_claim_identity(observed_claims, github_context, oidc_audience)
    wif_credential_config_verification = (
        load_wif_credential_config_verification(
            wif_credential_config_verification_path,
            wif_verification,
            oidc_claims,
        )
    )
    source_commit = verify_checked_out_source_commit()
    token = access_token()
    resource = GcsGenerationFencedObject(bucket, main_name, token)
    point_resource = GcsGenerationFencedObject(bucket, point_name, token)
    race_resource = GcsGenerationFencedObject(bucket, race_name, token)
    cases: list[dict[str, object]] = []

    try:
        generation = create_setup(resource, "main")

        current = request(
            "advance",
            generation,
            execution="execution-gcs-advance",
            key="idempotency-gcs-advance",
            body=b"sol-atlas:gcs:advance:v1",
        )
        applied = resource.apply(current)
        after_current = resource.state()
        if applied != "Applied" or after_current is None:
            raise AssertionError("current-fence mutation did not apply")
        cases.append(
            record(
                "current_fence_accepted",
                True,
                {
                    "result": applied,
                    "precondition_generation": generation,
                    "result_generation": after_current.generation,
                },
            )
        )
        generation = after_current.generation

        before_replay = generation
        replay = resource.apply(current)
        after_replay = resource.state()
        if replay != "AlreadyAppliedSameRequest" or after_replay is None:
            raise AssertionError("stable-key replay was not recognized")
        cases.append(
            record(
                "stable_key_replay_safe",
                after_replay.generation == before_replay,
                {
                    "result": replay,
                    "generation_before": before_replay,
                    "generation_after": after_replay.generation,
                },
            )
        )

        stale = request(
            "stale",
            generation - 1,
            execution="execution-gcs-stale",
            key="idempotency-gcs-stale",
        )
        stale_result = resource.raw_put(stale)
        cases.append(
            record(
                "stale_fence_rejected",
                stale_result.status == 412,
                {
                    "http_status": stale_result.status,
                    "precondition_generation": stale.fence_generation,
                    "current_generation": generation,
                },
            )
        )
        if stale_result.status != 412:
            raise AssertionError("GCS accepted a stale generation precondition")

        future = request(
            "future",
            generation + 1,
            execution="execution-gcs-future",
            key="idempotency-gcs-future",
        )
        future_result = resource.raw_put(future)
        cases.append(
            record(
                "future_fence_rejected",
                future_result.status == 412,
                {
                    "http_status": future_result.status,
                    "precondition_generation": future.fence_generation,
                    "current_generation": generation,
                },
            )
        )
        if future_result.status != 412:
            raise AssertionError("GCS accepted an unestablished future generation")

        before_identity = resource.state()
        same_key_other_execution = request(
            "same-key-other-execution",
            generation,
            execution="execution-gcs-other",
            key=current.idempotency_key,
            body=current.body,
        )
        identity_result = resource.apply(same_key_other_execution)
        after_identity = resource.state()
        if (
            identity_result != "RejectedIdentityMismatch"
            or before_identity is None
            or after_identity is None
            or after_identity.generation != before_identity.generation
        ):
            raise AssertionError("cross-execution idempotency collision was not blocked")
        cases.append(
            record(
                "different_request_same_key_rejected",
                True,
                {
                    "result": identity_result,
                    "generation_unchanged": True,
                },
            )
        )

        changed_same_key = request(
            "same-execution-different-request",
            generation,
            execution=current.execution_id,
            key=current.idempotency_key,
            body=b"sol-atlas:gcs:changed-request:v1",
        )
        changed_result = resource.apply(changed_same_key)
        cases.append(
            record(
                "changed_request_same_key_rejected",
                changed_result == "RejectedIdentityMismatch",
                {"result": changed_result},
            )
        )
        if changed_result != "RejectedIdentityMismatch":
            raise AssertionError("same-key request mutation was not rejected")

        changed_key = request(
            "changed-key",
            generation,
            execution=current.execution_id,
            key="different-idempotency-key",
            body=current.body,
        )
        changed_key_result = resource.apply(changed_key)
        cases.append(
            record(
                "changed_idempotency_key_rejected",
                changed_key_result == "RejectedIdentityMismatch",
                {"result": changed_key_result},
            )
        )
        if changed_key_result != "RejectedIdentityMismatch":
            raise AssertionError("idempotency-key drift was not rejected")

        concurrent_a = request(
            "concurrent-a",
            generation,
            execution="execution-gcs-concurrent-a",
            key="idempotency-gcs-concurrent-a",
            body=b"sol-atlas:gcs:concurrent:a:v1",
        )
        concurrent_b = request(
            "concurrent-b",
            generation,
            execution="execution-gcs-concurrent-b",
            key="idempotency-gcs-concurrent-b",
            body=b"sol-atlas:gcs:concurrent:b:v1",
        )
        with ThreadPoolExecutor(max_workers=2) as pool:
            futures = [
                pool.submit(resource.raw_put, concurrent_a),
                pool.submit(resource.raw_put, concurrent_b),
            ]
            concurrent_results = [future.result() for future in futures]
        statuses = sorted(result.status for result in concurrent_results)
        concurrency_ok = statuses == [200, 412]
        winning_request = (
            concurrent_a if concurrent_results[0].status == 200 else concurrent_b
        )
        cases.append(
            record(
                "concurrent_fencing_preserved",
                concurrency_ok,
                {
                    "http_statuses": statuses,
                    "winner_execution": winning_request.execution_id,
                },
            )
        )
        if not concurrency_ok:
            raise AssertionError(
                f"expected exactly one GCS 200 and one 412, got {statuses}"
            )
        winning_state = resource.state()
        winner_coherent = (
            winning_state is not None
            and winning_state.metadata.get("execution-id")
            == winning_request.execution_id
            and winning_state.metadata.get("input-fingerprint")
            == winning_request.input_fingerprint
            and winning_state.metadata.get("attempt-id")
            == winning_request.attempt_id
            and winning_state.metadata.get("fence-generation")
            == str(winning_request.fence_generation)
            and winning_state.metadata.get("idempotency-key")
            == winning_request.idempotency_key
            and winning_state.data_sha256 == sha256_prefixed(winning_request.body)
        )
        if not winner_coherent:
            raise AssertionError(
                "concurrent winner state does not match the successful request"
            )
        cases[-1]["passed"] = concurrency_ok and winner_coherent
        cases[-1]["observed"]["winner_state_coherent"] = winner_coherent
        generation = winning_state.generation

        race_generation = create_setup(race_resource, "metadata-race")
        baseline = race_resource.state()
        if baseline is None:
            raise AssertionError("metadata-race setup was not observable")

        def mutate_metadata(observed):
            result = race_resource.update_metadata(
                observed.generation,
                observed.metageneration,
                {"reconciliation-marker": "race-v1"},
            )
            if result.status != 200:
                raise AssertionError(
                    f"metadata race update failed: HTTP {result.status}"
                )

        coherent_state = race_resource.state(
            between_metadata_and_data=mutate_metadata
        )
        metadata_coherence_ok = (
            coherent_state is not None
            and coherent_state.generation == race_generation
            and coherent_state.metageneration > baseline.metageneration
            and coherent_state.metadata.get("reconciliation-marker") == "race-v1"
            and coherent_state.data_sha256 == baseline.data_sha256
        )
        cases.append(
            record(
                "metadata_readback_coherence",
                metadata_coherence_ok,
                {
                    "generation_unchanged": (
                        coherent_state.generation == baseline.generation
                        if coherent_state
                        else False
                    ),
                    "metageneration_before": baseline.metageneration,
                    "metageneration_after": (
                        coherent_state.metageneration
                        if coherent_state
                        else None
                    ),
                    "marker_observed": (
                        coherent_state.metadata.get("reconciliation-marker")
                        if coherent_state
                        else None
                    ),
                    "body_digest_unchanged": (
                        coherent_state.data_sha256 == baseline.data_sha256
                        if coherent_state
                        else False
                    ),
                    "retry_on_precondition_mismatch": True
                },
            )
        )
        if not metadata_coherence_ok:
            raise AssertionError(
                "metadata/data read-back did not converge to one coherent snapshot"
            )

        lost_ack = request(
            "lost-ack",
            generation,
            execution="execution-gcs-lost-ack",
            key="idempotency-gcs-lost-ack",
            body=b"sol-atlas:gcs:lost-ack:v1",
        )
        before_lost_ack = resource.state()
        if before_lost_ack is None:
            raise AssertionError("lost-ack object disappeared before the request")
        resource.raw_put(lost_ack, discard_response=True)
        reconciled = resource.reconcile(lost_ack)
        after_lost_ack = resource.state()
        if after_lost_ack is None:
            raise AssertionError("lost-ack object disappeared during reconciliation")
        if reconciled == "ObservedAppliedSameRequest":
            no_second_mutation = after_lost_ack.generation != before_lost_ack.generation
        else:
            no_second_mutation = after_lost_ack.generation == before_lost_ack.generation
        lost_ack_observation_ok = reconciled in {
            "ObservedAppliedSameRequest",
            "ObservedNotApplied",
            "ObservedDifferentRequest",
        }
        cases.append(
            record(
                "indeterminate_ack_reconciled",
                no_second_mutation and lost_ack_observation_ok,
                {
                    "acknowledgement": "response_discarded_before_read",
                    "reconciliation": reconciled,
                    "commit_status": (
                        "observed_applied"
                        if reconciled == "ObservedAppliedSameRequest"
                        else "indeterminate_at_reconciliation"
                    ),
                    "generation_before": before_lost_ack.generation,
                    "generation_after": after_lost_ack.generation,
                    "absence_is_not_non_commit": True,
                    "blind_retry_performed": False,
                },
            )
        )
        if not (no_second_mutation and lost_ack_observation_ok):
            raise AssertionError(
                "lost-ack reconciliation did not produce a bounded observation"
            )
        point_generation = 0
        point_request = request(
            "point-in-time-target",
            point_generation,
            execution="execution-gcs-point-in-time",
            key="idempotency-gcs-point-in-time",
            body=b"sol-atlas:gcs:point-in-time:v1",
        )
        before = point_resource.reconcile(point_request)
        applied_point = point_resource.apply(point_request)
        after = point_resource.reconcile(point_request)
        point_ok = point_in_time_semantics(
            before,
            applied_point,
            after,
        )
        cases.append(
            record(
                "point_in_time_semantics_explicit",
                point_ok,
                {
                    "before_apply": before,
                    "mutation": applied_point,
                    "after_apply": after,
                },
            )
        )
        if not point_ok:
            raise AssertionError("point-in-time reconciliation semantics changed")

        if [case["id"] for case in cases] != case_ids(case_set):
            raise AssertionError("case-set execution order drifted")

        evidence = {
            "case_set": EXPECTED_CASE_SET,
            "case_set_digest": digest(case_set),
            "cases": cases,
            "all_passed": all(bool(case["passed"]) for case in cases),
            "cleanup_succeeded": False,
        }
        return {
            "schema": SCHEMA,
            "wif_verification": wif_verification,
            "wif_verification_digest": digest(wif_verification),
            "github_execution_context": github_context,
            "github_execution_context_digest": digest(github_context),
            "github_workflow_run_verification": github_run_verification,
            "github_workflow_run_verification_digest": digest(
                github_run_verification
            ),
            "observer_github_oidc_claims": observer_oidc_claims,
            "github_oidc_subject_configuration": oidc_subject_configuration,
            "github_oidc_subject_configuration_digest": digest(
                oidc_subject_configuration
            ),
            "observer_github_oidc_claims_digest": digest(
                observer_oidc_claims
            ),
            "effective_iam_audit": effective_iam_audit,
            "effective_iam_audit_digest": digest(effective_iam_audit),
            "broad_sa_impersonation_audit": broad_sa_audit,
            "broad_sa_impersonation_audit_digest": digest(broad_sa_audit),
            "observer_isolation_audit": observer_isolation_audit,
            "observer_isolation_audit_digest": digest(observer_isolation_audit),
            "policy_effect_audit": policy_effect_audit,
            "policy_effect_audit_digest": digest(policy_effect_audit),
            "github_oidc_claims": oidc_claims,
            "github_oidc_claims_digest": digest(oidc_claims),
            "wif_credential_config_verification": (
                wif_credential_config_verification
            ),
            "wif_credential_config_verification_digest": digest(
                wif_credential_config_verification
            ),
            "checked_out_source_commit": source_commit,
            "status": "qualified",
            "service": "Google Cloud Storage",
            "gcp_project_id": wif_verification.get("project_id"),
            "bucket": bucket,
            "object_prefix": object_prefix,
            "object_root": root,
            "adapter_id": ADAPTER_ID,
            "adapter_revision": git_sha(ADAPTER_PATH),
            "harness_id": HARNESS_ID,
            "harness_revision": git_sha(HARNESS_PATH),
            "workflow_path": WORKFLOW_PATH,
            "workflow_revision": git_sha(WORKFLOW_PATH),
            "source_commit": git_head(),
            "profile": profile(),
            "profile_digest": profile_digest(profile()),
            "evidence_digest": digest(evidence),
            "case_set": EXPECTED_CASE_SET,
            "case_set_digest": digest(case_set),
            "case_set_path": CASE_SET_PATH,
            "object_names": [main_name, point_name, race_name],
            "generated_at": datetime.now(timezone.utc).isoformat(),
            "evidence": evidence,
        }
    finally:
        cleanup_errors = []
        for target in (resource, point_resource, race_resource):
            try:
                state = target.state()
                if state is not None:
                    result = target.delete(state.generation)
                    if not 200 <= result.status < 300:
                        cleanup_errors.append(
                            f"{target.object_name}: HTTP {result.status}"
                        )
            except Exception as exc:
                cleanup_errors.append(f"{target.object_name}: {exc}")
        if cleanup_errors:
            raise RuntimeError(
                "qualification cleanup failed: " + "; ".join(cleanup_errors)
            )


def finalize_report(report: dict[str, object]) -> dict[str, object]:
    evidence = dict(report["evidence"])
    evidence["cleanup_succeeded"] = True
    evidence["all_passed"] = all(bool(case["passed"]) for case in evidence["cases"])
    report["evidence"] = evidence
    report["evidence_digest"] = digest(evidence)
    report["status"] = "qualified" if evidence["all_passed"] else "unqualified"
    report["report_digest"] = digest(report)
    return report


def write_report(path: str, report: dict[str, object]) -> None:
    destination = Path(path)
    destination.parent.mkdir(parents=True, exist_ok=True)
    destination.write_text(
        json.dumps(report, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )


def object_resource_name(bucket: str, object_name: str) -> str:
    return (
        "//storage.googleapis.com/projects/_/buckets/"
        + bucket
        + "/objects/"
        + object_name
    )


def expected_object_root(
    object_prefix: str,
    run_id: str,
    run_attempt: str,
) -> str:
    if not object_prefix or not run_id or not run_attempt:
        raise AssertionError("object-root binding inputs are incomplete")
    return (
        object_prefix.rstrip("/")
        + "/run-"
        + run_id
        + "-attempt-"
        + run_attempt
    )


def validate_policy_effect_audit(
    audit: dict[str, object],
    wif_verification: dict[str, object],
    bucket: str,
    object_names: list[str],
    project_id: str,
) -> None:
    if audit.get("schema") != POLICY_EFFECT_AUDIT_SCHEMA:
        raise AssertionError("wrong Policy Troubleshooter audit schema")
    if audit.get("api_version") != "v3beta":
        raise AssertionError("wrong Policy Troubleshooter API version")
    manifest_path = (
        "sol-atlas-policy-store-contract/conformance/"
        "gcs_policy_troubleshooter_targets_v2.json"
    )
    manifest = json.loads(
        Path(manifest_path).read_text(encoding="utf-8")
    )
    if manifest.get("schema") != "sol-atlas:gcs-policy-troubleshooter-targets:v2":
        raise AssertionError("Policy Troubleshooter target manifest schema drift")
    if audit.get("target_manifest_path") != manifest_path:
        raise AssertionError("Policy Troubleshooter target manifest path drift")
    if audit.get("target_manifest_digest") != digest(manifest):
        raise AssertionError("Policy Troubleshooter target manifest digest drift")
    if audit.get("principal") != wif_verification.get("service_account"):
        raise AssertionError("Policy Troubleshooter principal drift")
    if audit.get("all_targets_verified") is not True:
        raise AssertionError("Policy Troubleshooter targets were not all verified")
    targets = audit.get("targets")
    if not isinstance(targets, list) or len(targets) != 14:
        raise AssertionError("Policy Troubleshooter target vector drift")
    project_resource = "//cloudresourcemanager.googleapis.com/projects/" + project_id
    service_account = wif_verification.get("service_account")
    if not isinstance(service_account, str) or not service_account:
        raise AssertionError("WIF service account is missing")
    sa_resource = (
        "//iam.googleapis.com/projects/"
        + project_id
        + "/serviceAccounts/"
        + service_account
    )
    expected: dict[tuple[str, str], str] = {
        (project_resource, "cloudbuild.builds.create"): "CANNOT_ACCESS",
        (
            project_resource,
            "deploymentmanager.deployments.create",
        ): "CANNOT_ACCESS",
        (
            sa_resource,
            "iam.serviceAccountKeys.create",
        ): "CANNOT_ACCESS",
        (
            sa_resource,
            "iam.serviceAccounts.getIamPolicy",
        ): "CANNOT_ACCESS",
        (
            sa_resource,
            "iam.serviceAccounts.setIamPolicy",
        ): "CANNOT_ACCESS",
    }
    object_permissions = (
        "storage.objects.create",
        "storage.objects.get",
        "storage.objects.delete",
    )
    for object_name in object_names:
        resource = object_resource_name(bucket, object_name)
        for permission in object_permissions:
            expected[(resource, permission)] = "CAN_ACCESS"
    if len(expected) != 14:
        raise AssertionError("Policy Troubleshooter expected target vector is malformed")
    seen: set[tuple[str, str]] = set()
    for target in targets:
        if not isinstance(target, dict):
            raise AssertionError("Policy Troubleshooter target is malformed")
        target_principal = target.get("principal")
        resource = target.get("resource")
        permission = target.get("permission")
        state = target.get("overall_access_state")
        expected_state = target.get("expected_overall_access_state")
        key = (resource, permission)
        if (
            target_principal != audit.get("principal")
            or not isinstance(resource, str)
            or not isinstance(permission, str)
            or key in seen
            or expected.get(key) != expected_state
            or state != expected_state
        ):
            raise AssertionError("Policy Troubleshooter target evidence drift")
        for field in (
            "allow_access_state",
            "deny_access_state",
            "pab_access_state",
        ):
            if not isinstance(target.get(field), str):
                raise AssertionError(
                    "Policy Troubleshooter policy-plane state is missing"
                )
            if target[field].endswith("UNKNOWN_INFO") or (
                "UNKNOWN_CONDITIONAL" in target[field]
            ) or target[field].endswith("UNSPECIFIED"):
                raise AssertionError(
                    "Policy Troubleshooter policy-plane state is unresolved"
                )
        response_digest = target.get("response_digest")
        if not isinstance(response_digest, str) or not DIGEST_RE.fullmatch(
            response_digest
        ):
            raise AssertionError("Policy Troubleshooter response digest is invalid")
        seen.add(key)
    if seen != set(expected):
        raise AssertionError("Policy Troubleshooter target set is incomplete")


def verify_resource_identity(report: dict[str, object]) -> None:
    if report.get("service") != "Google Cloud Storage":
        raise AssertionError("wrong external service")
    bucket = report.get("bucket")
    object_prefix = report.get("object_prefix")
    object_names = report.get("object_names")
    object_root = report.get("object_root")
    if not isinstance(bucket, str) or not bucket:
        raise AssertionError("missing GCS bucket identity")
    if not isinstance(object_prefix, str) or not object_prefix:
        raise AssertionError("missing GCS object prefix")
    if (
        not isinstance(object_root, str)
        or not object_root
        or not object_root.startswith(object_prefix.rstrip("/") + "/run-")
    ):
        raise AssertionError("missing or invalid qualification object root")
    expected_object_names = [
        object_root + "/main.bin",
        object_root + "/point-in-time.bin",
        object_root + "/metadata-race.bin",
    ]
    if object_names != expected_object_names:
        raise AssertionError("GCS object identities are incomplete or reordered")


def verify_report(path: str) -> None:
    report = json.loads(Path(path).read_text(encoding="utf-8"))
    expected_case_set = load_case_set()
    if report.get("schema") != SCHEMA:
        raise AssertionError("wrong report schema")
    if report.get("status") != "qualified":
        raise AssertionError("report is not qualified")
    if report.get("case_set") != EXPECTED_CASE_SET:
        raise AssertionError("wrong case-set identity")
    if report.get("case_set_path") != CASE_SET_PATH:
        raise AssertionError("wrong case-set path")
    verify_resource_identity(report)
    expected_case_set_digest = digest(expected_case_set)
    github_run_verification = report.get("github_workflow_run_verification")
    if not isinstance(github_run_verification, dict):
        raise AssertionError("missing GitHub workflow-run verification")
    if report.get("github_workflow_run_verification_digest") != digest(
        github_run_verification
    ):
        raise AssertionError("GitHub workflow-run verification digest mismatch")
    load_github_workflow_run_verification_from_report = (
        github_run_verification
    )
    if (
        load_github_workflow_run_verification_from_report.get("run_id")
        != os.environ.get("GITHUB_RUN_ID")
    ):
        raise AssertionError("GitHub workflow-run ID does not match current run")
    if (
        load_github_workflow_run_verification_from_report.get("run_attempt")
        != os.environ.get("GITHUB_RUN_ATTEMPT")
    ):
        raise AssertionError(
            "GitHub workflow-run attempt does not match current run"
        )
    if (
        load_github_workflow_run_verification_from_report.get("head_sha")
        != report.get("checked_out_source_commit")
    ):
        raise AssertionError(
            "GitHub server workflow-run SHA does not match checked-out source"
        )
    if report.get("case_set_digest") != expected_case_set_digest:
        raise AssertionError("case-set digest mismatch")
    wif_verification = report.get("wif_verification")
    if not isinstance(wif_verification, dict):
        raise AssertionError("missing WIF trust verification")
    validate_wif_verification(wif_verification)
    github_run_verification = report.get("github_workflow_run_verification")
    if not isinstance(github_run_verification, dict):
        raise AssertionError("missing GitHub workflow-run verification")
    if report.get("github_workflow_run_verification_digest") != digest(
        github_run_verification
    ):
        raise AssertionError("GitHub workflow-run verification digest mismatch")
    load_github_workflow_run_verification_from_report = github_run_verification
    if (
        load_github_workflow_run_verification_from_report.get("run_id")
        != os.environ.get("GITHUB_RUN_ID")
    ):
        raise AssertionError("GitHub workflow-run ID does not match current run")
    if (
        load_github_workflow_run_verification_from_report.get("run_attempt")
        != os.environ.get("GITHUB_RUN_ATTEMPT")
    ):
        raise AssertionError("GitHub workflow-run attempt does not match current run")
    expected_report_object_root = expected_object_root(
        str(report.get("object_prefix", "")),
        str(load_github_workflow_run_verification_from_report.get("run_id")),
        str(
            load_github_workflow_run_verification_from_report.get(
                "run_attempt"
            )
        ),
    )
    if report.get("object_root") != expected_report_object_root:
        raise AssertionError("GCS object root is not bound to the server run")
    github_context = report.get("github_execution_context")
    if not isinstance(github_context, dict):
        raise AssertionError("missing GitHub execution context")
    expected_context = {
        "repository": "Luminous-Dynamics/sol-atlas-leptos",
        "repository_id": "1195997641",
        "environment": "sol-atlas-gcs-qualification",
        "workflow": "Qualify GCS external effect",
        "event": "workflow_dispatch",
        "ref": "refs/heads/main",
        "runner_environment": "github-hosted",
        "workflow_ref": (
            "Luminous-Dynamics/sol-atlas-leptos/.github/workflows/"
            "qualify-gcs.yml@refs/heads/main"
        ),
    }
    for name, expected_value in expected_context.items():
        if github_context.get(name) != expected_value:
            raise AssertionError(
                f"GitHub execution context mismatch for {name}"
            )
    if report.get("github_execution_context_digest") != digest(github_context):
        raise AssertionError("GitHub execution context digest mismatch")
    observer_oidc_claims = report.get("observer_github_oidc_claims")
    if not isinstance(observer_oidc_claims, dict):
        raise AssertionError("missing observer GitHub OIDC claims")
    validate_observer_oidc_claims(
        observer_oidc_claims,
        github_context,
        wif_verification,
    )
    if report.get("observer_github_oidc_claims_digest") != digest(
        observer_oidc_claims
    ):
        raise AssertionError("observer OIDC artifact digest mismatch")
    oidc_subject_configuration = report.get(
        "github_oidc_subject_configuration"
    )
    if not isinstance(oidc_subject_configuration, dict):
        raise AssertionError(
            "missing GitHub OIDC subject configuration evidence"
        )
    validate_oidc_subject_configuration(oidc_subject_configuration)
    if report.get("github_oidc_subject_configuration_digest") != digest(
        oidc_subject_configuration
    ):
        raise AssertionError(
            "GitHub OIDC subject configuration evidence digest mismatch"
        )

    effective_iam_audit = report.get("effective_iam_audit")
    if not isinstance(effective_iam_audit, dict):
        raise AssertionError("missing effective-IAM audit")
    validate_effective_iam_audit(
        effective_iam_audit,
        wif_verification,
        observer_oidc_claims,
    )
    if report.get("effective_iam_audit_digest") != digest(effective_iam_audit):
        raise AssertionError("effective-IAM audit digest mismatch")
    broad_sa_audit = report.get("broad_sa_impersonation_audit")
    if not isinstance(broad_sa_audit, dict):
        raise AssertionError("missing broad service-account audit")
    validate_broad_sa_audit(
        broad_sa_audit,
        wif_verification,
        observer_oidc_claims,
    )
    if report.get("broad_sa_impersonation_audit_digest") != digest(
        broad_sa_audit
    ):
        raise AssertionError("broad service-account audit digest mismatch")
    policy_effect_audit = report.get("policy_effect_audit")
    if not isinstance(policy_effect_audit, dict):
        raise AssertionError("missing Policy Troubleshooter audit")
    report_bucket = report.get("bucket")
    report_objects = report.get("object_names")
    report_project = github_context.get("repository")
    if (
        not isinstance(report_bucket, str)
        or not isinstance(report_objects, list)
        or len(report_objects) != 3
    ):
        raise AssertionError("missing GCS resource identity for policy audit")
    validate_policy_effect_audit(
        policy_effect_audit,
        wif_verification,
        report_bucket,
        report_objects,
        report.get("gcp_project_id", ""),
    )
    if report.get("policy_effect_audit_digest") != digest(policy_effect_audit):
        raise AssertionError("Policy Troubleshooter audit digest mismatch")

    observer_isolation_audit = report.get("observer_isolation_audit")
    if not isinstance(observer_isolation_audit, dict):
        raise AssertionError("missing observer isolation audit")
    validate_observer_isolation_audit(
        observer_isolation_audit,
        wif_verification,
    )
    if report.get("observer_isolation_audit_digest") != digest(
        observer_isolation_audit
    ):
        raise AssertionError("observer isolation audit digest mismatch")
    oidc_claims = report.get("github_oidc_claims")
    if not isinstance(oidc_claims, dict):
        raise AssertionError("missing GitHub OIDC claims")
    if oidc_claims.get("schema") != OIDC_CLAIMS_SCHEMA:
        raise AssertionError("wrong GitHub OIDC claims schema in report")
    observed_claims = oidc_claims.get("claims")
    if not isinstance(observed_claims, dict):
        raise AssertionError("missing observed GitHub OIDC claims")
    if oidc_claims.get("claims_digest") != digest(observed_claims):
        raise AssertionError("GitHub OIDC claims digest mismatch in report")
    if oidc_claims.get("cryptographic_verification") != "not_performed_locally":
        raise AssertionError("unexpected local OIDC cryptographic-verification mode in report")
    if oidc_claims.get("wif_exchange_uses_this_exact_verified_token") is not True:
        raise AssertionError("report is not bound to the verified WIF token")
    if oidc_claims.get("workflow_sha_matches_runner") is not True:
        raise AssertionError("report OIDC workflow_sha mismatch")
    if oidc_claims.get("source_sha_matches_runner") is not True:
        raise AssertionError("report OIDC source sha mismatch")
    if oidc_claims.get("temporal_claims_valid") is not True:
        raise AssertionError("report OIDC temporal claims were not validated")
    verify_oidc_temporal_evidence(oidc_claims)
    if report.get("github_oidc_claims_digest") != digest(oidc_claims):
        raise AssertionError("GitHub OIDC artifact digest mismatch")
    wif_credential_config_verification = report.get(
        "wif_credential_config_verification"
    )
    if not isinstance(wif_credential_config_verification, dict):
        raise AssertionError("missing WIF credential-config verification")
    if report.get("wif_credential_config_verification_digest") != digest(
        wif_credential_config_verification
    ):
        raise AssertionError("WIF credential-config verification digest mismatch")
    if wif_credential_config_verification.get("schema") != (
        WIF_CREDENTIAL_CONFIG_SCHEMA
    ):
        raise AssertionError("wrong WIF credential-config schema in report")
    if wif_credential_config_verification.get("exact_verified_token_bound") is not True:
        raise AssertionError("report WIF token binding is not exact")
    if wif_credential_config_verification.get("token_digest") != oidc_claims.get(
        "token_digest"
    ):
        raise AssertionError("report WIF token digest mismatch")
    if wif_credential_config_verification.get("provider_resource") != wif_verification.get(
        "provider_resource"
    ):
        raise AssertionError("report WIF provider resource mismatch")
    if wif_credential_config_verification.get("service_account") != wif_verification.get(
        "service_account"
    ):
        raise AssertionError("report WIF service account mismatch")
    if wif_credential_config_verification.get("credential_audience") != wif_verification.get(
        "oidc_expected_audience"
    ):
        raise AssertionError("report WIF audience mismatch")
    oidc_audience = wif_verification.get("oidc_expected_audience")
    if not isinstance(oidc_audience, str) or not oidc_audience:
        raise AssertionError("WIF verification has no OIDC expected audience")
    if oidc_claims.get("audience") != observed_claims.get("aud"):
        raise AssertionError("OIDC artifact audience does not match its claims")
    verify_oidc_claim_identity(observed_claims, github_context, oidc_audience)
    if observed_claims.get("workflow_sha") != github_context.get("workflow_sha"):
        raise AssertionError("OIDC workflow_sha does not match GITHUB_WORKFLOW_SHA")
    if observed_claims.get("sha") != github_context.get("sha"):
        raise AssertionError("OIDC sha does not match GITHUB_SHA")
    if observed_claims.get("aud") != wif_verification.get("oidc_expected_audience"):
        raise AssertionError("OIDC audience does not match WIF expected audience")
    checked_out_source = report.get("checked_out_source_commit")
    if checked_out_source != github_context.get("sha"):
        raise AssertionError("checked-out source does not match GITHUB_SHA")
    if checked_out_source != git_head():
        raise AssertionError("checked-out source drift detected")
    workflow_sha = github_context.get("workflow_sha")
    if (
        not isinstance(workflow_sha, str)
        or len(workflow_sha) != 40
        or any(char not in "0123456789abcdef" for char in workflow_sha)
    ):
        raise AssertionError("invalid GITHUB_WORKFLOW_SHA in report")
    workflow_at_claim = git_sha_at_commit(workflow_sha, WORKFLOW_PATH)
    workflow_at_checkout = git_sha(WORKFLOW_PATH)
    if workflow_at_claim != workflow_at_checkout:
        raise AssertionError("workflow source drift detected")
    if report.get("wif_verification_digest") != digest(wif_verification):
        raise AssertionError("WIF verification digest mismatch")
    if report.get("adapter_revision") != git_sha(ADAPTER_PATH):
        raise AssertionError("adapter revision drift detected")
    if report.get("harness_revision") != git_sha(HARNESS_PATH):
        raise AssertionError("harness revision drift detected")
    if report.get("workflow_revision") != git_sha(WORKFLOW_PATH):
        raise AssertionError("workflow revision drift detected")
    if report.get("source_commit") != git_head():
        raise AssertionError("source commit drift detected")

    expected_profile = profile()
    if report.get("profile") != expected_profile:
        raise AssertionError("profile drift detected")
    if report.get("profile_digest") != profile_digest(expected_profile):
        raise AssertionError("profile digest mismatch")

    evidence = report.get("evidence")
    if not isinstance(evidence, dict):
        raise AssertionError("missing evidence")
    expected_ids = case_ids(expected_case_set)
    if [case.get("id") for case in evidence.get("cases", [])] != expected_ids:
        raise AssertionError("case-set vector mismatch")
    if evidence.get("case_set") != EXPECTED_CASE_SET:
        raise AssertionError("evidence case-set identity mismatch")
    if evidence.get("case_set_digest") != expected_case_set_digest:
        raise AssertionError("evidence case-set digest mismatch")
    if not evidence.get("cleanup_succeeded"):
        raise AssertionError("cleanup was not successful")
    if not evidence.get("all_passed"):
        raise AssertionError("one or more qualification cases failed")
    if report.get("evidence_digest") != digest(evidence):
        raise AssertionError("evidence digest mismatch")

    report_digest = report.pop("report_digest", None)
    if report_digest != digest(report):
        raise AssertionError("report digest mismatch")

    print("verified: exact source, profile, case set, evidence, and report")


def main() -> int:
    parser = argparse.ArgumentParser()
    sub = parser.add_subparsers(dest="command", required=True)
    sub.add_parser("validate-case-set")
    qualify = sub.add_parser("qualify")
    qualify.add_argument("--bucket", required=True)
    qualify.add_argument("--object-prefix", default="sol-atlas/qualification")
    qualify.add_argument("--wif-verification", required=True)
    qualify.add_argument("--oidc-claims", required=True)
    qualify.add_argument("--oidc-subject-configuration", required=True)
    qualify.add_argument("--github-run-verification", required=True)
    qualify.add_argument(
        "--wif-credential-config-verification",
        required=True,
    )
    qualify.add_argument("--observer-oidc-claims", required=True)
    qualify.add_argument("--effective-iam-audit", required=True)
    qualify.add_argument("--broad-sa-audit", required=True)
    qualify.add_argument("--observer-isolation-audit", required=True)
    qualify.add_argument("--policy-effect-audit", required=True)
    qualify.add_argument("--object-root", required=True)
    qualify.add_argument("--output", required=True)
    verify = sub.add_parser("verify")
    verify.add_argument("--report", required=True)
    args = parser.parse_args()

    if args.command == "validate-case-set":
        validate_case_set()
        return 0

    if args.command == "verify":
        verify_report(args.report)
        return 0

    report = run_qualification(
        args.bucket,
        args.object_prefix,
        args.wif_verification,
        args.oidc_claims,
        args.oidc_subject_configuration,
        args.wif_credential_config_verification,
        args.github_run_verification,
        args.observer_oidc_claims,
        args.effective_iam_audit,
        args.broad_sa_audit,
        args.observer_isolation_audit,
        args.policy_effect_audit,
        args.object_root,
    )
    report = finalize_report(report)
    write_report(args.output, report)
    verify_report(args.output)
    print(
        "QUALIFIED "
        + report["adapter_id"]
        + " "
        + report["source_commit"]
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
