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

SCHEMA = "sol-atlas:gcs-policy-troubleshooter-audit:v4"
API_VERSION = "v3beta"
TARGET_MANIFEST_SCHEMA = "sol-atlas:gcs-policy-troubleshooter-targets:v3"
TARGET_MANIFEST_PATH = (
    "sol-atlas-policy-store-contract/conformance/"
    "gcs_policy_troubleshooter_targets_v3.json"
)
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


def validate_bucket(bucket: str) -> str:
    if (
        not isinstance(bucket, str)
        or not bucket
        or "/" in bucket
        or any(character.isspace() for character in bucket)
    ):
        raise AssertionError("bucket name is invalid")
    return bucket


def validate_object_root(object_root: str) -> str:
    if (
        not isinstance(object_root, str)
        or not object_root
        or object_root.startswith("/")
        or object_root.endswith("/")
        or "//" in object_root
        or any(character in object_root for character in "\r\n")
    ):
        raise AssertionError("object root is invalid")
    return object_root


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
        "response": payload,
        "response_digest": digest(payload),
    }


def validate_target_manifest_binding(
    manifest_path: str,
) -> tuple[str, str]:
    if manifest_path != TARGET_MANIFEST_PATH:
        raise AssertionError(
            "target manifest path is not the frozen manifest"
        )
    path = Path(manifest_path)
    if not path.is_file():
        raise AssertionError(
            "frozen Policy Troubleshooter target manifest is missing"
        )
    document = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(document, dict):
        raise AssertionError("frozen target manifest is not an object")
    if document.get("schema") != TARGET_MANIFEST_SCHEMA:
        raise AssertionError("frozen target manifest schema drift")
    targets = document.get("targets")
    if not isinstance(targets, list) or not targets:
        raise AssertionError("frozen target manifest has no targets")
    return manifest_path, digest(document)


def target_vector_shape(
    targets: list[dict[str, object]],
) -> list[tuple[object, object, object, object]]:
    return [
        (
            target.get("resource_template"),
            target.get("permission"),
            target.get("expected_overall_access_state"),
            target.get("resource"),
        )
        for target in targets
        if isinstance(target, dict)
    ]


def validate_target_vector_binding(
    supplied: list[dict[str, object]],
    frozen: list[dict[str, object]],
) -> None:
    if target_vector_shape(supplied) != target_vector_shape(frozen):
        raise AssertionError(
            "supplied target vector is not the expanded frozen manifest"
        )


def verify_targets(
    principal: str,
    targets: list[dict[str, object]],
    output: str | None,
    manifest_path: str = TARGET_MANIFEST_PATH,
    project_id: str | None = None,
    bucket: str | None = None,
    object_root: str | None = None,
) -> dict[str, object]:
    principal = validate_service_account(principal)
    if not targets:
        raise AssertionError("no Policy Troubleshooter targets supplied")
    frozen_manifest_path, frozen_manifest_digest = (
        validate_target_manifest_binding(manifest_path)
    )
    if project_id is None:
        raise AssertionError(
            "project ID is required to bind frozen target vector"
        )
    frozen_targets = load_targets(
        TARGET_MANIFEST_PATH,
        project_id,
        principal,
        bucket,
        object_root,
    )
    validate_target_vector_binding(targets, frozen_targets)
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

    input_manifest = json.loads(
        Path(manifest_path).read_text(encoding="utf-8")
    )
    if input_manifest.get("schema") != TARGET_MANIFEST_SCHEMA:
        raise AssertionError("target manifest schema drift")
    frozen_manifest = json.loads(
        Path(TARGET_MANIFEST_PATH).read_text(encoding="utf-8")
    )
    if frozen_manifest.get("schema") != TARGET_MANIFEST_SCHEMA:
        raise AssertionError("checked-in target manifest schema drift")
    input_digest = digest(input_manifest)
    frozen_digest = digest(frozen_manifest)
    if (
        manifest_path != TARGET_MANIFEST_PATH
        or input_digest != frozen_digest
    ):
        raise AssertionError("target manifest input is not the frozen manifest")
    result: dict[str, object] = {
        "schema": SCHEMA,
        "api_version": API_VERSION,
        "target_manifest_path": frozen_manifest_path,
        "target_manifest_digest": frozen_manifest_digest,
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
    bucket: str | None = None,
    object_root: str | None = None,
) -> list[dict[str, object]]:
    document = json.loads(Path(path).read_text(encoding="utf-8"))
    if not isinstance(document, dict):
        raise AssertionError("target manifest is not an object")
    manifest_schema = document.get("schema")
    if manifest_schema != TARGET_MANIFEST_SCHEMA:
        raise AssertionError("wrong target manifest schema")
    raw_targets = document.get("targets")
    if not isinstance(raw_targets, list):
        raise AssertionError("target manifest has no target list")
    project_id = validate_project_id(project_id)
    service_account = validate_service_account(service_account)
    if bucket is not None:
        bucket = validate_bucket(bucket)
    if object_root is not None:
        object_root = validate_object_root(object_root)
    targets: list[dict[str, object]] = []
    for raw_target in raw_targets:
        if not isinstance(raw_target, dict):
            raise AssertionError("target manifest contains malformed target")
        template = raw_target.get("resource_template")
        if not isinstance(template, str) or not template.startswith("//"):
            raise AssertionError("target manifest has invalid resource template")
        fields = {
            "project_id": project_id,
            "service_account": service_account,
        }
        if "{bucket}" in template:
            if bucket is None:
                raise AssertionError(
                    "target requires bucket but none was supplied"
                )
            fields["bucket"] = bucket
        if "{object_root}" in template:
            if object_root is None:
                raise AssertionError(
                    "target requires object root but none was supplied"
                )
            fields["object_root"] = object_root
        try:
            resource = template.format(**fields)
        except (KeyError, ValueError) as exc:
            raise AssertionError("target resource template is invalid") from exc
        target = dict(raw_target)
        target["resource"] = resource
        targets.append(target)
        validate_target(target)
    return targets


def object_resource(bucket: str, object_name: str) -> str:
    validate_bucket(bucket)
    if not object_name or object_name.startswith("/") or object_name.endswith("/"):
        raise AssertionError("object name is invalid")
    return (
        "//storage.googleapis.com/projects/_/buckets/"
        + bucket
        + "/objects/"
        + object_name
    )


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--principal-email", required=True)
    parser.add_argument("--targets", required=True)
    parser.add_argument("--project-id", required=True)
    parser.add_argument("--bucket")
    parser.add_argument("--object-root")
    parser.add_argument("--output")
    args = parser.parse_args()
    result = verify_targets(
        args.principal_email,
        load_targets(
            args.targets,
            args.project_id,
            args.principal_email,
            args.bucket,
            args.object_root,
        ),
        args.output,
        args.targets,
        args.project_id,
        args.bucket,
        args.object_root,
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
