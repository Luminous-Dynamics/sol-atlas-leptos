#!/usr/bin/env python3
# Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
# SPDX-License-Identifier: AGPL-3.0-or-later

"""Audit downstream service-account access with Policy Troubleshooter."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import subprocess
from pathlib import Path

SCHEMA = "sol-atlas:gcs-policy-troubleshooter-audit:v1"
API_VERSION = "v3beta"
DIGEST_RE = re.compile(r"^sha256:[0-9a-f]{64}$")
PROJECT_ID_RE = re.compile(r"^[a-z][a-z0-9-]{4,28}[a-z0-9]$")

UNKNOWN_STATES = {
    "ALLOW_ACCESS_STATE_UNSPECIFIED",
    "ALLOW_ACCESS_STATE_UNKNOWN_CONDITIONAL",
    "ALLOW_ACCESS_STATE_UNKNOWN_INFO",
    "DENY_ACCESS_STATE_UNSPECIFIED",
    "DENY_ACCESS_STATE_UNKNOWN_CONDITIONAL",
    "DENY_ACCESS_STATE_UNKNOWN_INFO",
    "PAB_ACCESS_STATE_UNSPECIFIED",
    "PAB_ACCESS_STATE_UNKNOWN_INFO",
    "OVERALL_ACCESS_STATE_UNSPECIFIED",
}


def canonical(value: object) -> bytes:
    return json.dumps(
        value,
        sort_keys=True,
        separators=(",", ":"),
        ensure_ascii=True,
    ).encode("utf-8")


def digest(value: object) -> str:
    return "sha256:" + hashlib.sha256(canonical(value)).hexdigest()


def validate_project_id(project_id: str) -> str:
    if not isinstance(project_id, str) or not PROJECT_ID_RE.fullmatch(project_id):
        raise AssertionError("project ID is invalid")
    return project_id


def validate_service_account(principal: str) -> str:
    if (
        not isinstance(principal, str)
        or principal.count("@") != 1
        or principal.startswith("@")
        or principal.endswith("@")
        or not principal.endswith(".iam.gserviceaccount.com")
    ):
        raise AssertionError("principal is not a service-account email")
    return principal


def validate_target(target: dict[str, object]) -> tuple[str, str, str]:
    if not isinstance(target, dict):
        raise AssertionError("target is not an object")
    resource = target.get("resource")
    permission = target.get("permission")
    expected = target.get("expected_overall_access_state")
    if (
        not isinstance(resource, str)
        or not resource.startswith("//")
        or not resource
        or not isinstance(permission, str)
        or not permission
        or not isinstance(expected, str)
        or expected not in {
            "CAN_ACCESS",
            "CANNOT_ACCESS",
        }
    ):
        raise AssertionError("malformed troubleshooter target")
    return resource, permission, expected


def run_troubleshooter(
    principal: str,
    resource: str,
    permission: str,
) -> dict[str, object]:
    validate_service_account(principal)
    result = subprocess.run(
        [
            "gcloud",
            "beta",
            "policy-intelligence",
            "troubleshoot-policy",
            "iam",
            resource,
            "--principal-email=" + principal,
            "--permission=" + permission,
            "--format=json",
        ],
        check=True,
        capture_output=True,
        text=True,
        timeout=60,
    )
    payload = json.loads(result.stdout)
    if not isinstance(payload, dict):
        raise AssertionError("Policy Troubleshooter response is not an object")
    return payload


def access_state(payload: dict[str, object], field: str) -> str:
    value = payload.get(field)
    if not isinstance(value, str):
        raise AssertionError("missing " + field)
    return value


def validate_response(
    payload: dict[str, object],
    principal: str,
    resource: str,
    permission: str,
    expected: str,
) -> dict[str, object]:
    access_tuple = payload.get("accessTuple")
    if not isinstance(access_tuple, dict):
        raise AssertionError("Policy Troubleshooter access tuple is missing")
    if access_tuple.get("principal") != principal:
        raise AssertionError("Policy Troubleshooter principal drift")
    if access_tuple.get("fullResourceName") != resource:
        raise AssertionError("Policy Troubleshooter resource drift")
    if access_tuple.get("permission") != permission:
        raise AssertionError("Policy Troubleshooter permission drift")

    overall = access_state(payload, "overallAccessState")
    allow = payload.get("allowPolicyExplanation")
    deny = payload.get("denyPolicyExplanation")
    pab = payload.get("pabPolicyExplanation")
    if not isinstance(allow, dict):
        raise AssertionError("allow-policy explanation is missing")
    if not isinstance(deny, dict):
        raise AssertionError("deny-policy explanation is missing")
    if not isinstance(pab, dict):
        raise AssertionError("PAB explanation is missing")

    allow_state = access_state(allow, "allowAccessState")
    deny_state = access_state(deny, "denyAccessState")
    pab_state = access_state(
        pab,
        "principalAccessBoundaryAccessState",
    )
    states = (overall, allow_state, deny_state, pab_state)
    if any(state in UNKNOWN_STATES for state in states):
        raise AssertionError(
            "Policy Troubleshooter returned unknown or unspecified access state"
        )
    if overall != expected:
        raise AssertionError(
            "unexpected Policy Troubleshooter overall state: "
            + overall
            + " != "
            + expected
        )

    return {
        "principal": principal,
        "resource": resource,
        "permission": permission,
        "expected_overall_access_state": expected,
        "overall_access_state": overall,
        "allow_access_state": allow_state,
        "deny_access_state": deny_state,
        "pab_access_state": pab_state,
        "allow_policy_explanation": allow,
        "deny_policy_explanation": deny,
        "pab_policy_explanation": pab,
        "response_digest": digest(payload),
    }


def verify_targets(
    principal: str,
    targets: list[dict[str, object]],
    output: str | None,
) -> dict[str, object]:
    principal = validate_service_account(principal)
    if not targets:
        raise AssertionError("no Policy Troubleshooter targets supplied")
    observations: list[dict[str, object]] = []
    seen: set[tuple[str, str]] = set()
    for target in targets:
        resource, permission, expected = validate_target(target)
        key = (resource, permission)
        if key in seen:
            raise AssertionError("duplicate troubleshooter target")
        seen.add(key)
        payload = run_troubleshooter(
            principal,
            resource,
            permission,
        )
        observations.append(
            validate_response(
                payload,
                principal,
                resource,
                permission,
                expected,
            )
        )

    result: dict[str, object] = {
        "schema": SCHEMA,
        "api_version": API_VERSION,
        "principal": principal,
        "target_count": len(observations),
        "targets": observations,
        "all_targets_verified": True,
        "claim_ceiling": (
            "Observed Policy Troubleshooter allow, deny, and Principal Access "
            "Boundary states for the selected service account, resources, and "
            "permissions. Unknown or unspecified policy visibility fails closed. "
            "This does not prove workload-identity federation, VPC Service Controls, "
            "Cloud Storage ACL access, transitive impersonation, or immutable IAM state."
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


def load_targets(
    path: str,
    project_id: str,
    service_account: str,
) -> list[dict[str, object]]:
    document = json.loads(Path(path).read_text(encoding="utf-8"))
    if not isinstance(document, dict):
        raise AssertionError("target manifest is not an object")
    if document.get("schema") != "sol-atlas:gcs-policy-troubleshooter-targets:v1":
        raise AssertionError("wrong target manifest schema")
    raw_targets = document.get("targets")
    if not isinstance(raw_targets, list):
        raise AssertionError("target manifest has no target list")
    project_id = validate_project_id(project_id)
    service_account = validate_service_account(service_account)
    targets: list[dict[str, object]] = []
    for raw_target in raw_targets:
        if not isinstance(raw_target, dict):
            raise AssertionError("target manifest contains malformed target")
        template = raw_target.get("resource_template")
        if not isinstance(template, str) or not template.startswith("//"):
            raise AssertionError("target manifest has invalid resource template")
        try:
            resource = template.format(
                project_id=project_id,
                service_account=service_account,
            )
        except (KeyError, ValueError) as exc:
            raise AssertionError("target resource template is invalid") from exc
        target = dict(raw_target)
        target["resource"] = resource
        targets.append(target)
        validate_target(target)
    return targets


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--principal-email", required=True)
    parser.add_argument("--targets", required=True)
    parser.add_argument("--project-id", required=True)
    parser.add_argument("--output")
    args = parser.parse_args()
    result = verify_targets(
        args.principal_email,
        load_targets(
            args.targets,
            args.project_id,
            args.principal_email,
        ),
        args.output,
    )
    print(
        "verified IAM policy effects: "
        + result["principal"]
        + " "
        + str(result["target_count"])
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
