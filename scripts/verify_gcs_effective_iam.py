#!/usr/bin/env python3
# Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
# SPDX-License-Identifier: AGPL-3.0-or-later

"""Audit effective service-account impersonation access with Policy Analyzer."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import subprocess
from pathlib import Path

SCHEMA = "sol-atlas:gcs-wif-effective-iam-audit:v7"
EXPECTED_ROLE = "roles/iam.workloadIdentityUser"
REQUIRED_PERMISSIONS = (
    "iam.serviceAccounts.getAccessToken",
    "iam.serviceAccounts.getOpenIdToken",
)
FORBIDDEN_EXECUTION_PERMISSIONS = (
    "iam.serviceAccounts.getIamPolicy",
    "iam.serviceAccounts.setIamPolicy",
)
PROJECT_PIVOT_PERMISSIONS = (
    "cloudbuild.builds.create",
    "deploymentmanager.deployments.create",
)
CRITICAL_PERMISSIONS = (
    "iam.serviceAccounts.getAccessToken",
    "iam.serviceAccounts.getOpenIdToken",
    "iam.serviceAccounts.signBlob",
    "iam.serviceAccounts.signJwt",
    "iam.serviceAccounts.implicitDelegation",
    "iam.serviceAccounts.actAs",
    "iam.serviceAccountKeys.create",
    *FORBIDDEN_EXECUTION_PERMISSIONS,
)
SCOPE_PATTERN = re.compile(r"^(projects|folders|organizations)/[A-Za-z0-9._-]+$")
ANCESTOR_PATTERN = re.compile(
    r"^//cloudresourcemanager\.googleapis\.com/"
    r"(projects|folders|organizations)/[A-Za-z0-9._-]+$"
)
CLAIM_CEILING = (
    "Policy Analyzer-observed effective IAM allow-policy access for the selected "
    "service account and listed credential-capability permissions. The analysis "
    "must be fully explored and is scoped by the configured project, folder, or "
    "organization. Data is best-effort and may lag recent policy changes. This "
    "does not prove deny-policy or Principal Access Boundary effects, arbitrary "
    "service-triggered privilege paths, transitive impersonation chains, or a "
    "globally immutable IAM state."
)


def canonical(value: object) -> bytes:
    return json.dumps(
        value,
        sort_keys=True,
        separators=(",", ":"),
        ensure_ascii=True,
    ).encode("utf-8")


def digest(value: object) -> str:
    return "sha256:" + hashlib.sha256(canonical(value)).hexdigest()


def project_resource(project_id: str) -> str:
    if (
        not project_id
        or project_id.startswith("-")
        or "/" in project_id
    ):
        raise AssertionError("invalid project identity")
    return (
        "//cloudresourcemanager.googleapis.com/projects/"
        + project_id
    )


def service_account_resource(
    project_id: str,
    service_account: str,
) -> str:
    if (
        not project_id
        or project_id.startswith("-")
        or "/" in project_id
        or not service_account
        or service_account.count("@") != 1
        or service_account.startswith("@")
        or service_account.endswith("@")
    ):
        raise AssertionError("invalid project or service-account identity")
    return (
        "//iam.googleapis.com/projects/"
        + project_id
        + "/serviceAccounts/"
        + service_account
    )


def validate_scope(scope: str) -> str:
    if not SCOPE_PATTERN.fullmatch(scope):
        raise AssertionError("invalid Cloud Asset scope")
    return scope


def scope_flag(scope: str) -> tuple[str, str]:
    scope = validate_scope(scope)
    kind, identifier = scope.split("/", 1)
    return "--" + kind[:-1], identifier


def run_analysis(
    scope: str,
    resource: str,
) -> dict[str, object]:
    flag, identifier = scope_flag(scope)
    result = subprocess.run(
        [
            "gcloud",
            "asset",
            "analyze-iam-policy",
            flag + "=" + identifier,
            "--full-resource-name=" + resource,
            "--permissions=" + ",".join(CRITICAL_PERMISSIONS),
            "--format=json",
        ],
        check=True,
        capture_output=True,
        text=True,
        timeout=60,
    )
    payload = json.loads(result.stdout)
    if not isinstance(payload, dict):
        raise AssertionError("Policy Analyzer response is not an object")
    if payload.get("fullyExplored") is not True:
        raise AssertionError("Policy Analyzer response is not fully explored")
    errors = payload.get("nonCriticalErrors")
    if errors is not None and (
        not isinstance(errors, list) or errors
    ):
        raise AssertionError(
            "Policy Analyzer reported invalid or non-critical errors"
        )
    results = payload.get("analysisResults")
    if not isinstance(results, list) or not results:
        raise AssertionError("Policy Analyzer returned no analysis results")
    return payload


WIF_PRINCIPAL_SET_PREFIX = "principalSet://iam.googleapis.com/projects/"
WIF_PROFILE_PATH = (
    "sol-atlas-policy-store-contract/conformance/"
    "gcs_wif_trust_profile_v8.json"
)
WIF_PROFILE_SCHEMA = "sol-atlas:gcs-wif-trust-profile:v8"
OIDC_CLAIMS_SCHEMA = "sol-atlas:github-oidc-claims:v5"
OIDC_IMMUTABLE_SUBJECT_PREFIX = (
    "repo:Luminous-Dynamics@216969177/sol-atlas-leptos@1195997641:"
)



def principal_set_prefix(expected_principal: str) -> str:
    marker = "/attribute.repository_id/"
    if marker not in expected_principal:
        raise AssertionError(
            "expected principal is not the frozen repository-id principal set"
        )
    prefix = expected_principal.split(marker, 1)[0]
    if not prefix.startswith(WIF_PRINCIPAL_SET_PREFIX):
        raise AssertionError("expected principal has invalid WIF principal-set prefix")
    return prefix


def expected_workload_principal_sets(expected_principal: str) -> set[str]:
    profile_path = Path(__file__).resolve().parents[1] / WIF_PROFILE_PATH
    profile = json.loads(profile_path.read_text(encoding="utf-8"))
    if profile.get("schema") != WIF_PROFILE_SCHEMA:
        raise AssertionError("WIF trust profile schema drift")
    condition = profile.get("exact_attribute_condition")
    if not isinstance(condition, str):
        raise AssertionError("WIF trust profile condition is missing")
    condition_values = {}
    for clause in condition.split(" && "):
        if not clause.startswith("assertion.") or "==" not in clause:
            continue
        name, value = clause.split("==", 1)
        value = value.strip().strip("'")
        condition_values[name.removeprefix("assertion.")] = value
    mapped = profile.get("required_attribute_mappings")
    if not isinstance(mapped, dict):
        raise AssertionError("WIF trust profile mappings are missing")
    missing = set(condition_values) - {
        key.removeprefix("attribute.")
        for key in mapped
        if key.startswith("attribute.")
    }
    if missing:
        raise AssertionError("WIF trust profile mappings are incomplete")
    prefix = principal_set_prefix(expected_principal)
    suffix = expected_principal.split(prefix + "/", 1)[1]
    repository_id = suffix.split("/", 1)[1]
    if condition_values["repository_id"] != repository_id:
        raise AssertionError("expected principal repository ID disagrees with WIF profile")
    members = {prefix + "/*", expected_principal}
    for attribute, value in condition_values.items():
        members.add(prefix + "/attribute." + attribute + "/" + value)
    return members


def load_immutable_oidc_subject(path: str) -> str:
    document = json.loads(Path(path).read_text(encoding="utf-8"))
    if not isinstance(document, dict):
        raise AssertionError("OIDC evidence is not an object")
    if document.get("schema") != OIDC_CLAIMS_SCHEMA:
        raise AssertionError("OIDC evidence schema drift")
    claims = document.get("claims")
    if not isinstance(claims, dict):
        raise AssertionError("OIDC evidence has no claims object")
    if document.get("claims_digest") != digest(claims):
        raise AssertionError("OIDC evidence claims digest drift")
    expected = {
        "repository": "Luminous-Dynamics/sol-atlas-leptos",
        "repository_id": "1195997641",
        "repository_owner_id": "216969177",
        "environment": "sol-atlas-gcs-qualification",
        "workflow": "Qualify GCS external effect",
        "event_name": "workflow_dispatch",
        "ref": "refs/heads/main",
    }
    for name, value in expected.items():
        if claims.get(name) != value:
            raise AssertionError("OIDC evidence identity drift for " + name)
    subject = claims.get("sub")
    if not isinstance(subject, str) or not subject.startswith(
        OIDC_IMMUTABLE_SUBJECT_PREFIX
    ):
        raise AssertionError("OIDC evidence is not an immutable repository subject")
    return subject


def subject_principal(expected_principal: str, subject: str) -> str:
    return (
        expected_principal.split("/attribute.repository_id/", 1)[0]
        .replace("principalSet://", "principal://", 1)
        + "/subject/"
        + subject
    )

def principal_matches_expected(
    identity: str,
    expected_principal: str,
    oidc_subject: str,
) -> str | None:
    if identity == expected_principal:
        return "exact"
    if identity == subject_principal(expected_principal, oidc_subject):
        return "immutable-subject"
    if identity in expected_workload_principal_sets(expected_principal):
        return "containing-principal-set"
    return None


def valid_attached_resource(
    resource: object,
    expected_resource: str,
) -> bool:
    if not isinstance(resource, str):
        return False
    expected_project_resource = expected_resource.replace(
        "//iam.googleapis.com/projects/",
        "//cloudresourcemanager.googleapis.com/projects/",
        1,
    ).split("/serviceAccounts/", 1)[0]
    return (
        resource == expected_resource
        or resource == expected_project_resource
        or bool(
            re.fullmatch(
                r"//cloudresourcemanager\.googleapis\.com/"
                r"(folders|organizations)/[A-Za-z0-9._-]+",
                resource,
            )
        )
    )


def run_project_pivot_analysis(
    scope: str,
    project_target: str,
) -> dict[str, object]:
    flag, identifier = scope_flag(scope)
    result = subprocess.run(
        [
            "gcloud",
            "asset",
            "analyze-iam-policy",
            flag + "=" + identifier,
            "--full-resource-name=" + project_target,
            "--permissions=" + ",".join(PROJECT_PIVOT_PERMISSIONS),
            "--format=json",
        ],
        check=True,
        capture_output=True,
        text=True,
        timeout=60,
    )
    payload = json.loads(result.stdout)
    if not isinstance(payload, dict):
        raise AssertionError("project pivot response is not an object")
    if payload.get("fullyExplored") is not True:
        raise AssertionError("project pivot response is not fully explored")
    if payload.get("nonCriticalErrors") or []:
        raise AssertionError("project pivot response has non-critical errors")
    results = payload.get("analysisResults")
    if not isinstance(results, list):
        raise AssertionError("project pivot analysisResults is not a list")
    return payload


def valid_project_attachment(
    resource: object,
    expected_project_resource: str,
) -> bool:
    return (
        resource == expected_project_resource
        or (
            isinstance(resource, str)
            and bool(
                re.fullmatch(
                    r"//cloudresourcemanager\.googleapis\.com/"
                    r"(folders|organizations)/[A-Za-z0-9._-]+",
                    resource,
                )
            )
        )
    )


def extract_findings(
    payload: dict[str, object],
    expected_principal: str,
    expected_resource: str,
) -> list[dict[str, object]]:
    if payload.get("fullyExplored") is not True:
        raise AssertionError("Policy Analyzer response is not fully explored")
    errors = payload.get("nonCriticalErrors")
    if errors is not None and (
        not isinstance(errors, list) or errors
    ):
        raise AssertionError(
            "Policy Analyzer reported invalid or non-critical errors"
        )
    results = payload.get("analysisResults")
    if not isinstance(results, list):
        raise AssertionError("Policy Analyzer analysisResults is not a list")

    findings: list[dict[str, object]] = []
    for index, result in enumerate(results):
        if not isinstance(result, dict):
            raise AssertionError(f"analysis result {index} is not an object")
        if result.get("fullyExplored") is not True:
            raise AssertionError(
                f"analysis result {index} is not fully explored"
            )

        identity_list = result.get("identityList")
        if not isinstance(identity_list, dict):
            raise AssertionError(
                f"analysis result {index} has invalid identity list"
            )
        identities = identity_list.get("identities")
        if not isinstance(identities, list):
            raise AssertionError(
                f"analysis result {index} has invalid identities"
            )
        identity_names = []
        for identity in identities:
            if not isinstance(identity, dict):
                raise AssertionError(
                    f"analysis result {index} has invalid identity"
                )
            name = identity.get("name")
            if not isinstance(name, str) or not name:
                raise AssertionError(
                    f"analysis result {index} has invalid identity name"
                )
            identity_names.append(name)

        matches = [
            principal_matches_expected(
                name,
                expected_principal,
                oidc_subject,
            )
            for name in identity_names
        ]
        match_kinds = [match for match in matches if match is not None]
        if match_kinds != ["exact"]:
            raise AssertionError(
                "service-account binding matched a broader workload principal set"
            )

        binding = result.get("iamBinding")
        if not isinstance(binding, dict):
            raise AssertionError(
                f"analysis result {index} has no IAM binding"
            )
        role = binding.get("role")
        members = binding.get("members")
        attached = result.get("attachedResourceFullName")
        if role != EXPECTED_ROLE:
            raise AssertionError(
                f"unexpected effective role for expected principal: {role!r}"
            )
        member_matches = [
            principal_matches_expected(
                member,
                expected_principal,
                oidc_subject,
            )
            for member in members
            if isinstance(member, str)
        ]
        if members != [expected_principal]:
            if any(match is not None for match in member_matches):
                raise AssertionError(
                    "broader or mixed workload principal binding detected"
                )
            continue
        if not valid_attached_resource(attached, expected_resource):
            raise AssertionError(
                f"invalid effective IAM policy attachment: {attached!r}"
            )

        accesses: set[str] = set()
        resource_seen = False
        access_lists = result.get("accessControlLists")
        if not isinstance(access_lists, list) or not access_lists:
            raise AssertionError(
                f"analysis result {index} has no access-control list"
            )
        for access_list in access_lists:
            if not isinstance(access_list, dict):
                raise AssertionError(
                    f"analysis result {index} has invalid access list"
                )
            resources = access_list.get("resources")
            if not isinstance(resources, list) or not resources:
                raise AssertionError(
                    f"analysis result {index} has invalid resources"
                )
            for resource in resources:
                if not isinstance(resource, dict):
                    raise AssertionError(
                        f"analysis result {index} has invalid resource entry"
                    )
                if resource.get("fullResourceName") != expected_resource:
                    raise AssertionError(
                        "Policy Analyzer result targeted a different resource"
                    )
                resource_seen = True

            condition = access_list.get("conditionEvaluation")
            if condition is not None:
                if not isinstance(condition, dict):
                    raise AssertionError(
                        f"analysis result {index} has invalid condition evaluation"
                    )
                if condition.get("evaluationValue") != "TRUE":
                    raise AssertionError(
                        "conditional effective IAM access is not admitted"
                    )

            access_entries = access_list.get("accesses")
            if not isinstance(access_entries, list) or not access_entries:
                raise AssertionError(
                    f"analysis result {index} has no accesses"
                )
            for access in access_entries:
                if not isinstance(access, dict):
                    raise AssertionError(
                        f"analysis result {index} has invalid access"
                    )
                permission = access.get("permission")
                if isinstance(permission, str):
                    accesses.add(permission)

        if not resource_seen:
            raise AssertionError(
                f"analysis result {index} has no target resource"
            )

        findings.append(
            {
                "attached_resource": attached,
                "role": role,
                "members": list(members),
                "identities": identity_names,
                "expected_principal_resolved": True,
                "principal_match_kinds": sorted(set(match_kinds)),
                "permissions": sorted(accesses),
                "fully_explored": True,
            }
        )

    if len(findings) != 1:
        raise AssertionError(
            "Policy Analyzer did not resolve exactly one binding for expected principal"
        )
    return findings


def extract_project_pivot_findings(
    payload: dict[str, object],
    expected_principal: str,
    expected_project_resource: str,
) -> list[dict[str, object]]:
    if payload.get("fullyExplored") is not True:
        raise AssertionError("project pivot analysis is not fully explored")
    errors = payload.get("nonCriticalErrors")
    if errors is not None and (
        not isinstance(errors, list) or errors
    ):
        raise AssertionError(
            "project pivot analysis reported invalid or non-critical errors"
        )
    results = payload.get("analysisResults")
    if not isinstance(results, list):
        raise AssertionError("project pivot analysisResults is not a list")

    findings: list[dict[str, object]] = []
    for index, result in enumerate(results):
        if not isinstance(result, dict):
            raise AssertionError(f"project pivot result {index} is not an object")
        if result.get("fullyExplored") is not True:
            raise AssertionError(
                f"project pivot result {index} is not fully explored"
            )

        identities = result.get("identityList")
        if not isinstance(identities, dict):
            raise AssertionError(
                f"project pivot result {index} has invalid identity list"
            )
        identity_entries = identities.get("identities")
        if not isinstance(identity_entries, list):
            raise AssertionError(
                f"project pivot result {index} has invalid identities"
            )
        names = []
        for identity in identity_entries:
            if not isinstance(identity, dict):
                raise AssertionError(
                    f"project pivot result {index} has invalid identity"
                )
            name = identity.get("name")
            if not isinstance(name, str) or not name:
                raise AssertionError(
                    f"project pivot result {index} has invalid identity name"
                )
            names.append(name)

        pivot_matches = [
            principal_matches_expected(name, expected_principal)
            for name in names
        ]
        pivot_match_kinds = [
            match for match in pivot_matches if match is not None
        ]
        if not pivot_match_kinds:
            continue

        binding = result.get("iamBinding")
        if not isinstance(binding, dict):
            raise AssertionError(
                f"project pivot result {index} has no IAM binding"
            )
        role = binding.get("role")
        members = binding.get("members")
        if not isinstance(role, str) or not role:
            raise AssertionError(
                f"project pivot result {index} has invalid role"
            )
        if not isinstance(members, list) or not any(
            principal_matches_expected(member, expected_principal)
            for member in members
            if isinstance(member, str)
        ):
            raise AssertionError(
                "project pivot binding does not contain expected principal set"
            )

        attachments = result.get("attachedResourceFullName")
        if (
            not isinstance(attachments, str)
            or not valid_project_attachment(
                attachments,
                expected_project_resource,
            )
        ):
            raise AssertionError(
                f"invalid project pivot attachment: {attachments!r}"
            )

        observed_permissions: set[str] = set()
        uncertain = False
        active_acl_seen = False
        access_lists = result.get("accessControlLists")
        if not isinstance(access_lists, list) or not access_lists:
            raise AssertionError(
                f"project pivot result {index} has no access-control list"
            )
        for access_list in access_lists:
            if not isinstance(access_list, dict):
                raise AssertionError(
                    f"project pivot result {index} has invalid access list"
                )
            resources = access_list.get("resources")
            if not isinstance(resources, list) or not resources:
                raise AssertionError(
                    f"project pivot result {index} has invalid resources"
                )
            for resource in resources:
                if not isinstance(resource, dict):
                    raise AssertionError(
                        f"project pivot result {index} has invalid resource"
                    )
                if resource.get("fullResourceName") != expected_project_resource:
                    raise AssertionError(
                        "project pivot result targeted a different project"
                    )
            condition = access_list.get("conditionEvaluation")
            if condition is not None:
                if not isinstance(condition, dict):
                    raise AssertionError(
                        f"project pivot result {index} has invalid condition"
                    )
                value = condition.get("evaluationValue")
                if value == "CONDITIONAL":
                    uncertain = True
                elif value == "FALSE":
                    continue
                elif value == "TRUE":
                    active_acl_seen = True
                else:
                    raise AssertionError(
                        "project pivot result has unknown condition state"
                    )
            if condition is None:
                active_acl_seen = True
            accesses = access_list.get("accesses")
            if not isinstance(accesses, list) or not accesses:
                raise AssertionError(
                    f"project pivot result {index} has no accesses"
                )
            for access in accesses:
                if not isinstance(access, dict):
                    raise AssertionError(
                        f"project pivot result {index} has invalid access"
                    )
                permission = access.get("permission")
                if isinstance(permission, str):
                    observed_permissions.add(permission)

        if uncertain:
            raise AssertionError(
                "project pivot access could not be determined"
            )
        if not active_acl_seen:
            continue
        if not observed_permissions.intersection(PROJECT_PIVOT_PERMISSIONS):
            raise AssertionError(
                "project pivot result does not contain a queried pivot permission"
            )
        findings.append(
            {
                "attached_resource": attachments,
                "role": role,
                "members": list(members),
                "identities": names,
                "principal_match_kinds": sorted(set(pivot_match_kinds)),
                "permissions": sorted(observed_permissions),
                "fully_explored": True,
            }
        )

    return findings


def required_permissions_are_present(observed_permissions: list[str]) -> bool:
    return all(
        permission in observed_permissions
        for permission in REQUIRED_PERMISSIONS
    )


def execution_permissions_are_clean(
    observed_permissions: list[str],
) -> bool:
    return not set(observed_permissions).intersection(
        FORBIDDEN_EXECUTION_PERMISSIONS
    )


def validate_permission_ceiling(
    findings: list[dict[str, object]],
) -> list[str]:
    observed_permissions = sorted(
        {
            permission
            for finding in findings
            for permission in finding["permissions"]
        }
    )
    if not execution_permissions_are_clean(observed_permissions):
        forbidden_observed = sorted(
            set(observed_permissions).intersection(
                FORBIDDEN_EXECUTION_PERMISSIONS
            )
        )
        raise AssertionError(
            "execution principal has observer policy privileges: "
            + ", ".join(forbidden_observed)
        )
    if not required_permissions_are_present(observed_permissions):
        missing = [
            permission
            for permission in REQUIRED_PERMISSIONS
            if permission not in observed_permissions
        ]
        raise AssertionError(
            "Policy Analyzer did not observe required WIF permissions: "
            + ", ".join(missing)
        )
    return observed_permissions


def verify(
    scope: str,
    project_id: str,
    service_account: str,
    expected_principal: str,
    oidc_claims: str,
    output: str | None,
) -> dict[str, object]:
    scope = validate_scope(scope)
    resource = service_account_resource(project_id, service_account)
    if not expected_principal:
        raise AssertionError("expected principal is required")

    oidc_subject = load_immutable_oidc_subject(oidc_claims)
    payload = run_analysis(scope, resource)
    findings = extract_findings(
        payload,
        expected_principal,
        resource,
        oidc_subject,
    )
    observed_permissions = validate_permission_ceiling(findings)

    project_target = project_resource(project_id)
    project_pivot_payload = run_project_pivot_analysis(
        scope,
        project_target,
    )
    project_pivots = extract_project_pivot_findings(
        project_pivot_payload,
        expected_principal,
        project_target,
        oidc_subject,
    )
    if project_pivots:
        raise AssertionError(
            "execution principal has a project-level service-account execution pivot"
        )

    observed_roles = sorted(
        {str(finding["role"]) for finding in findings}
    )
    attached_resources = sorted(
        {
            str(finding["attached_resource"])
            for finding in findings
            if finding["attached_resource"] is not None
        }
    )
    result: dict[str, object] = {
        "schema": SCHEMA,
        "scope": scope,
        "project_id": project_id,
        "service_account": service_account,
        "service_account_resource": resource,
        "expected_principal": expected_principal,
        "expected_role": EXPECTED_ROLE,
        "queried_permissions": list(CRITICAL_PERMISSIONS),
        "forbidden_execution_permissions": list(
            FORBIDDEN_EXECUTION_PERMISSIONS
        ),
        "forbidden_execution_permissions_absent": True,
        "required_permissions_verified": True,
        "principal_selection_mode": (
            "permission_query_with_frozen_workload_principal_set_filter"
        ),
        "wif_profile_path": WIF_PROFILE_PATH,
        "wif_profile_digest": digest(
            json.loads(
                Path(__file__).resolve().parents[1].joinpath(
                    WIF_PROFILE_PATH
                ).read_text(encoding="utf-8")
            )
        ),
        "matched_workload_principal_sets": sorted(
            expected_workload_principal_sets(expected_principal)
        ),
        "observed_permissions": observed_permissions,
        "observed_roles": observed_roles,
        "observed_attached_resources": attached_resources,
        "finding_count": len(findings),
        "fully_explored": True,
        "non_critical_errors": [],
        "findings": findings,
        "project_pivot_permissions": list(PROJECT_PIVOT_PERMISSIONS),
        "project_pivot_findings": project_pivots,
        "project_pivot_permissions_absent": True,
        "project_pivot_response_digest": digest(project_pivot_payload),
        "policy_analyzer_response_digest": digest(payload),
        "claim_ceiling": CLAIM_CEILING,
    }
    if output:
        destination = Path(output)
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_text(
            json.dumps(result, indent=2, sort_keys=True) + "\n",
            encoding="utf-8",
        )
    return result


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--scope", required=True)
    parser.add_argument("--project-id", required=True)
    parser.add_argument("--service-account", required=True)
    parser.add_argument("--expected-principal", required=True)
    parser.add_argument("--oidc-claims", required=True)
    parser.add_argument("--output")
    args = parser.parse_args()
    result = verify(
        args.scope,
        args.project_id,
        args.service_account,
        args.expected_principal,
        args.output,
    )
    print(
        "verified effective IAM allow-policy audit: "
        + result["scope"]
        + " "
        + result["service_account_resource"]
        + " "
        + str(result["finding_count"])
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
