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

SCHEMA = "sol-atlas:gcs-wif-effective-iam-audit:v3"
EXPECTED_ROLE = "roles/iam.workloadIdentityUser"
REQUIRED_PERMISSIONS = (
    "iam.serviceAccounts.getAccessToken",
    "iam.serviceAccounts.getOpenIdToken",
)
FORBIDDEN_EXECUTION_PERMISSIONS = (
    "iam.serviceAccounts.getIamPolicy",
    "iam.serviceAccounts.setIamPolicy",
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
    "does not prove deny-policy or Principal Access Boundary effects, transitive "
    "impersonation chains, or a globally immutable IAM state."
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
    expected_principal: str,
) -> dict[str, object]:
    flag, identifier = scope_flag(scope)
    result = subprocess.run(
        [
            "gcloud",
            "asset",
            "analyze-iam-policy",
            flag + "=" + identifier,
            "--full-resource-name=" + resource,
            "--identity=" + expected_principal,
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
    errors = payload.get("nonCriticalErrors") or []
    if errors:
        raise AssertionError(
            "Policy Analyzer reported non-critical errors; audit is fail-closed"
        )
    results = payload.get("analysisResults")
    if not isinstance(results, list) or not results:
        raise AssertionError("Policy Analyzer returned no analysis results")
    return payload


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
        resource == expected_project_resource
        or bool(
            re.fullmatch(
                r"//cloudresourcemanager\.googleapis\.com/"
                r"(folders|organizations)/[A-Za-z0-9._-]+",
                resource,
            )
        )
    )


def extract_findings(
    payload: dict[str, object],
    expected_principal: str,
    expected_resource: str,
) -> list[dict[str, object]]:
    results = payload.get("analysisResults")
    if not isinstance(results, list):
        raise AssertionError("Policy Analyzer analysisResults is not a list")
    findings: list[dict[str, object]] = []
    seen_bindings: set[tuple[str, str, tuple[str, ...]]] = set()
    for index, result in enumerate(results):
        if not isinstance(result, dict):
            raise AssertionError(f"analysis result {index} is not an object")
        if result.get("fullyExplored") is not True:
            raise AssertionError(
                f"analysis result {index} is not fully explored"
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
                f"unexpected effective role: {role!r}"
            )
        if members != [expected_principal]:
            raise AssertionError(
                f"unexpected effective IAM members: {members!r}"
            )
        if not valid_attached_resource(attached, expected_resource):
            raise AssertionError(
                f"invalid effective IAM policy attachment: {attached!r}"
            )

        binding_key = (
            str(attached),
            str(role),
            tuple(str(member) for member in members),
        )
        if binding_key in seen_bindings:
            raise AssertionError("duplicate effective IAM binding result")
        seen_bindings.add(binding_key)

        identity_list = result.get("identityList")
        if not isinstance(identity_list, dict):
            raise AssertionError(
                f"analysis result {index} has invalid identity list"
            )
        identities = identity_list.get("identities")
        if not isinstance(identities, list) or len(identities) != 1:
            raise AssertionError(
                f"analysis result {index} has unexpected identity count"
            )
        identity = identities[0]
        if not isinstance(identity, dict) or identity.get("name") != expected_principal:
            raise AssertionError(
                f"analysis result {index} identity mismatch"
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
                "identities": [expected_principal],
                "permissions": sorted(accesses),
                "fully_explored": True,
            }
        )

    if len(findings) != 1:
        raise AssertionError(
            "Policy Analyzer observed multiple effective binding paths"
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


def verify(
    scope: str,
    project_id: str,
    service_account: str,
    expected_principal: str,
    output: str | None,
) -> dict[str, object]:
    scope = validate_scope(scope)
    resource = service_account_resource(project_id, service_account)
    if not expected_principal:
        raise AssertionError("expected principal is required")
    payload = run_analysis(scope, resource, expected_principal)
    findings = extract_findings(payload, expected_principal, resource)
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
        "observed_permissions": observed_permissions,
        "observed_roles": observed_roles,
        "observed_attached_resources": attached_resources,
        "finding_count": len(findings),
        "fully_explored": True,
        "non_critical_errors": [],
        "findings": findings,
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
