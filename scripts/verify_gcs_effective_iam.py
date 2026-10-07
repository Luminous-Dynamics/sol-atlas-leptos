#!/usr/bin/env python3
# Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
# SPDX-License-Identifier: AGPL-3.0-or-later

"""Audit effective service-account impersonation access with Policy Analyzer."""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
from pathlib import Path

SCHEMA = "sol-atlas:gcs-wif-effective-iam-audit:v1"
EXPECTED_ROLE = "roles/iam.workloadIdentityUser"
REQUIRED_PERMISSIONS = (
    "iam.serviceAccounts.getAccessToken",
    "iam.serviceAccounts.getOpenIdToken",
)
CRITICAL_PERMISSIONS = (
    "iam.serviceAccounts.getAccessToken",
    "iam.serviceAccounts.getOpenIdToken",
    "iam.serviceAccounts.signBlob",
    "iam.serviceAccounts.signJwt",
    "iam.serviceAccounts.implicitDelegation",
    "iam.serviceAccounts.actAs",
    "iam.serviceAccountKeys.create",
)
CLAIM_CEILING = (
    "Policy Analyzer-observed effective IAM allow-policy access for the selected "
    "service account and listed credential-capability permissions; the result is "
    "best-effort and may lag recent policy changes. It does not prove deny-policy "
    "or Principal Access Boundary effects, transitive impersonation chains, or "
    "a globally immutable IAM state."
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
    ):
        raise AssertionError("invalid project or service-account identity")
    return (
        "//iam.googleapis.com/projects/"
        + project_id
        + "/serviceAccounts/"
        + service_account
    )


def run_analysis(
    project_id: str,
    resource: str,
) -> dict[str, object]:
    result = subprocess.run(
        [
            "gcloud",
            "asset",
            "analyze-iam-policy",
            "--projects=" + project_id,
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
    results = payload.get("analysisResults")
    if not isinstance(results, list) or not results:
        raise AssertionError("Policy Analyzer returned no analysis results")
    return payload


def extract_findings(
    payload: dict[str, object],
    expected_principal: str,
    expected_resource: str,
) -> list[dict[str, object]]:
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

        binding = result.get("iamBinding")
        if not isinstance(binding, dict):
            raise AssertionError(
                f"analysis result {index} has no IAM binding"
            )
        role = binding.get("role")
        members = binding.get("members")
        if role != EXPECTED_ROLE:
            raise AssertionError(
                f"unexpected effective role: {role!r}"
            )
        if members != [expected_principal]:
            raise AssertionError(
                f"unexpected effective IAM members: {members!r}"
            )

        identity_list = result.get("identityList") or {}
        if not isinstance(identity_list, dict):
            raise AssertionError(
                f"analysis result {index} has invalid identity list"
            )
        identities = identity_list.get("identities") or []
        if not isinstance(identities, list):
            raise AssertionError(
                f"analysis result {index} has invalid identities"
            )
        identity_names = [
            identity.get("name")
            for identity in identities
            if isinstance(identity, dict)
        ]
        if identity_names != [expected_principal]:
            raise AssertionError(
                f"unexpected analyzed identities: {identity_names!r}"
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
            resources = access_list.get("resources") or []
            if not isinstance(resources, list):
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
            for access in access_list.get("accesses") or []:
                if not isinstance(access, dict):
                    continue
                permission = access.get("permission")
                if isinstance(permission, str):
                    accesses.add(permission)

        if not resource_seen:
            raise AssertionError(
                f"analysis result {index} has no target resource"
            )

        findings.append(
            {
                "attached_resource": result.get("attachedResourceFullName"),
                "role": role,
                "members": list(members),
                "identities": identity_names,
                "permissions": sorted(accesses),
                "fully_explored": True,
            }
        )
    if len(findings) != 1:
        raise AssertionError(
            "Policy Analyzer observed more than one effective binding path"
        )
    return findings


def verify(
    project_id: str,
    service_account: str,
    expected_principal: str,
    output: str | None,
) -> dict[str, object]:
    resource = service_account_resource(project_id, service_account)
    if not expected_principal:
        raise AssertionError("expected principal is required")
    payload = run_analysis(project_id, resource)
    findings = extract_findings(payload, expected_principal, resource)
    observed_permissions = sorted(
        {
            permission
            for finding in findings
            for permission in finding["permissions"]
        }
    )
    for permission in REQUIRED_PERMISSIONS:
        if permission not in observed_permissions:
            raise AssertionError(
                "Policy Analyzer did not observe the required WIF permission: "
                + permission
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
        "project_id": project_id,
        "service_account": service_account,
        "service_account_resource": resource,
        "expected_principal": expected_principal,
        "expected_role": EXPECTED_ROLE,
        "queried_permissions": list(CRITICAL_PERMISSIONS),
        "required_permissions_verified": True,
        "observed_permissions": observed_permissions,
        "observed_roles": observed_roles,
        "observed_attached_resources": attached_resources,
        "finding_count": len(findings),
        "fully_explored": True,
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
    parser.add_argument("--project-id", required=True)
    parser.add_argument("--service-account", required=True)
    parser.add_argument("--expected-principal", required=True)
    parser.add_argument("--output")
    args = parser.parse_args()
    result = verify(
        args.project_id,
        args.service_account,
        args.expected_principal,
        args.output,
    )
    print(
        "verified effective IAM allow-policy audit: "
        + result["service_account_resource"]
        + " "
        + str(result["finding_count"])
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
