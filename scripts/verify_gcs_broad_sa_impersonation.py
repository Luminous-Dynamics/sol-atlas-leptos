#!/usr/bin/env python3
# Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
# SPDX-License-Identifier: AGPL-3.0-or-later

"""Audit broad service-account impersonation access with Policy Analyzer."""

from __future__ import annotations

import argparse
import importlib.util
import json
import subprocess
from pathlib import Path

SCHEMA = "sol-atlas:gcs-broad-service-account-impersonation-audit:v1"
EXPECTED_ROLE = "roles/iam.workloadIdentityUser"
PERMISSIONS = (
    "iam.serviceAccounts.actAs",
    "iam.serviceAccounts.getAccessToken",
    "iam.serviceAccounts.getOpenIdToken",
    "iam.serviceAccounts.signBlob",
    "iam.serviceAccounts.signJwt",
    "iam.serviceAccounts.implicitDelegation",
    "iam.serviceAccountKeys.create",
)


def load_effective_iam_module():
    path = Path(__file__).with_name("verify_gcs_effective_iam.py")
    spec = importlib.util.spec_from_file_location(
        "verify_gcs_effective_iam_shared",
        path,
    )
    if not spec or not spec.loader:
        raise AssertionError("cannot load effective-IAM verifier")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def qualification_resource(
    project_id: str,
    service_account: str,
) -> str:
    return load_effective_iam_module().service_account_resource(
        project_id,
        service_account,
    )


def run_analysis(scope: str) -> dict[str, object]:
    module = load_effective_iam_module()
    flag, identifier = module.scope_flag(scope)
    result = subprocess.run(
        [
            "gcloud",
            "asset",
            "analyze-iam-policy",
            flag + "=" + identifier,
            "--permissions=" + ",".join(PERMISSIONS),
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
        raise AssertionError("impersonation analysis is not fully explored")
    errors = payload.get("nonCriticalErrors") or []
    if errors:
        raise AssertionError(
            "impersonation analysis reported non-critical errors"
        )
    results = payload.get("analysisResults")
    if not isinstance(results, list):
        raise AssertionError(
            "impersonation analysisResults is not a list"
        )
    return payload


def extract_findings(
    payload: dict[str, object],
    expected_principal: str,
    qualification_resource_name: str,
) -> list[dict[str, object]]:
    module = load_effective_iam_module()
    expected_sets = module.expected_workload_principal_sets(
        expected_principal
    )
    results = payload.get("analysisResults")
    if not isinstance(results, list):
        raise AssertionError("analysisResults is not a list")

    findings: list[dict[str, object]] = []
    for index, result in enumerate(results):
        if not isinstance(result, dict):
            raise AssertionError(
                f"impersonation result {index} is not an object"
            )
        if result.get("fullyExplored") is not True:
            raise AssertionError(
                f"impersonation result {index} is not fully explored"
            )

        identity_list = result.get("identityList")
        if not isinstance(identity_list, dict):
            raise AssertionError(
                f"impersonation result {index} has no identity list"
            )
        identities = identity_list.get("identities")
        if not isinstance(identities, list):
            raise AssertionError(
                f"impersonation result {index} has invalid identities"
            )
        identity_names = []
        for identity in identities:
            if not isinstance(identity, dict):
                raise AssertionError(
                    f"impersonation result {index} has invalid identity"
                )
            name = identity.get("name")
            if not isinstance(name, str) or not name:
                raise AssertionError(
                    f"impersonation result {index} has invalid identity name"
                )
            identity_names.append(name)

        matches = [
            name for name in identity_names if name in expected_sets
        ]
        if not matches:
            continue

        binding = result.get("iamBinding")
        if not isinstance(binding, dict):
            raise AssertionError(
                f"impersonation result {index} has no IAM binding"
            )
        role = binding.get("role")
        members = binding.get("members")
        if not isinstance(members, list):
            raise AssertionError(
                f"impersonation result {index} has invalid binding members"
            )

        access_lists = result.get("accessControlLists")
        if not isinstance(access_lists, list) or not access_lists:
            raise AssertionError(
                f"impersonation result {index} has no access-control list"
            )

        permissions: set[str] = set()
        resources: set[str] = set()
        conditional = False
        for access_list in access_lists:
            if not isinstance(access_list, dict):
                raise AssertionError(
                    f"impersonation result {index} has invalid access list"
                )
            raw_resources = access_list.get("resources")
            if (
                not isinstance(raw_resources, list)
                or not raw_resources
            ):
                raise AssertionError(
                    f"impersonation result {index} has invalid resources"
                )
            for resource in raw_resources:
                if not isinstance(resource, dict):
                    raise AssertionError(
                        f"impersonation result {index} has invalid resource"
                    )
                name = resource.get("fullResourceName")
                if not isinstance(name, str) or not name:
                    raise AssertionError(
                        f"impersonation result {index} has invalid resource name"
                    )
                resources.add(name)

            condition = access_list.get("conditionEvaluation")
            if condition is not None:
                if not isinstance(condition, dict):
                    raise AssertionError(
                        f"impersonation result {index} has invalid condition"
                    )
                value = condition.get("evaluationValue")
                if value == "CONDITIONAL":
                    conditional = True
                elif value == "FALSE":
                    continue
                elif value != "TRUE":
                    raise AssertionError(
                        "impersonation result has unknown condition state"
                    )

            accesses = access_list.get("accesses")
            if not isinstance(accesses, list) or not accesses:
                raise AssertionError(
                    f"impersonation result {index} has no accesses"
                )
            for access in accesses:
                if not isinstance(access, dict):
                    raise AssertionError(
                        f"impersonation result {index} has invalid access"
                    )
                permission = access.get("permission")
                if isinstance(permission, str):
                    permissions.add(permission)

        if conditional:
            raise AssertionError(
                "impersonation access is conditionally unresolved"
            )

        if not permissions.intersection(PERMISSIONS):
            continue

        intended = (
            role == EXPECTED_ROLE
            and members == [expected_principal]
            and result.get("attachedResourceFullName")
            == qualification_resource_name
            and resources == {qualification_resource_name}
            and permissions == {
                "iam.serviceAccounts.getAccessToken",
                "iam.serviceAccounts.getOpenIdToken",
            }
        )
        if intended:
            continue

        findings.append(
            {
                "attached_resource": result.get("attachedResourceFullName"),
                "role": role,
                "members": list(members),
                "identities": identity_names,
                "principal_match_kinds": [
                    "containing-principal-set"
                    if name != expected_principal
                    else "exact"
                    for name in matches
                ],
                "permissions": sorted(permissions),
                "resources": sorted(resources),
                "fully_explored": True,
            }
        )

    return findings


def verify(
    scope: str,
    project_id: str,
    service_account: str,
    expected_principal: str,
    output: str | None,
) -> dict[str, object]:
    module = load_effective_iam_module()
    scope = module.validate_scope(scope)
    qualification = qualification_resource(
        project_id,
        service_account,
    )
    if not expected_principal:
        raise AssertionError("expected principal is required")

    payload = run_analysis(scope)
    findings = extract_findings(
        payload,
        expected_principal,
        qualification,
    )
    if findings:
        raise AssertionError(
            "broad service account credential permissions detected"
        )

    result: dict[str, object] = {
        "schema": SCHEMA,
        "scope": scope,
        "project_id": project_id,
        "service_account": service_account,
        "qualification_resource": qualification,
        "expected_principal": expected_principal,
        "queried_permissions": list(PERMISSIONS),
        "intended_qualification_binding_allowed": True,
        "broad_impersonation_findings": [],
        "broad_impersonation_absent": True,
        "policy_analyzer_response_digest": module.digest(payload),
        "claim_ceiling": (
            "No broad service-account impersonation allow-policy path was "
            "observed for the expected workload identity or frozen principal "
            "sets within the selected Policy Analyzer scope. The intended "
            "exact WIF token-creation binding on the qualification service "
            "account is explicitly allowed. This does not prove deny/PAB "
            "effects, arbitrary service-triggered privilege, transitive "
            "impersonation, or immutable IAM state."
        ),
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
        "verified broad service-account impersonation audit: "
        + result["scope"]
        + " "
        + result["qualification_resource"]
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
