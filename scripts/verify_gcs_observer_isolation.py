#!/usr/bin/env python3
# Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
# SPDX-License-Identifier: AGPL-3.0-or-later

"""Verify the IAM observer cannot become an effect executor."""

from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
import subprocess
from pathlib import Path

SCHEMA = "sol-atlas:gcs-observer-isolation-audit:v1"
FORBIDDEN_SERVICE_ACCOUNT_PERMISSIONS = (
    "iam.serviceAccounts.actAs",
    "iam.serviceAccounts.getAccessToken",
    "iam.serviceAccounts.getOpenIdToken",
    "iam.serviceAccounts.implicitDelegation",
    "iam.serviceAccounts.signBlob",
    "iam.serviceAccounts.signJwt",
    "iam.serviceAccountKeys.create",
    "iam.serviceAccounts.setIamPolicy",
)
REQUIRED_OBSERVER_PERMISSION = "iam.serviceAccounts.getIamPolicy"


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


def canonical(value: object) -> bytes:
    return json.dumps(
        value,
        sort_keys=True,
        separators=(",", ":"),
        ensure_ascii=True,
    ).encode("utf-8")


def digest(value: object) -> str:
    return "sha256:" + hashlib.sha256(canonical(value)).hexdigest()


def service_account_principal(email: str) -> str:
    if not email or email.count("@") != 1:
        raise AssertionError("invalid observer service account")
    return "serviceAccount:" + email


def run_analysis(
    scope: str,
    identity: str,
    resource: str,
) -> dict[str, object]:
    module = load_effective_iam_module()
    flag, identifier = module.scope_flag(scope)
    result = subprocess.run(
        [
            "gcloud",
            "asset",
            "analyze-iam-policy",
            flag + "=" + identifier,
            "--identity=" + identity,
            "--full-resource-name=" + resource,
            "--permissions=" + ",".join(
                (REQUIRED_OBSERVER_PERMISSION,)
                + FORBIDDEN_SERVICE_ACCOUNT_PERMISSIONS
            ),
            "--format=json",
        ],
        check=True,
        capture_output=True,
        text=True,
        timeout=60,
    )
    payload = json.loads(result.stdout)
    if not isinstance(payload, dict):
        raise AssertionError("observer analysis response is not an object")
    if payload.get("fullyExplored") is not True:
        raise AssertionError("observer analysis is not fully explored")
    errors = payload.get("nonCriticalErrors")
    if errors is not None and (
        not isinstance(errors, list) or errors
    ):
        raise AssertionError("observer analysis reported errors")
    results = payload.get("analysisResults")
    if not isinstance(results, list):
        raise AssertionError("observer analysisResults is not a list")
    return payload


def extract(
    payload: dict[str, object],
    observer_principal: str,
    effect_resource: str,
) -> dict[str, object]:
    if payload.get("fullyExplored") is not True:
        raise AssertionError("observer analysis is not fully explored")
    errors = payload.get("nonCriticalErrors")
    if errors:
        raise AssertionError("observer analysis reported errors")
    results = payload.get("analysisResults")
    if not isinstance(results, list):
        raise AssertionError("observer analysisResults is not a list")

    observed: set[str] = set()
    forbidden: set[str] = set()
    observer_seen = False
    for index, result in enumerate(results):
        if not isinstance(result, dict):
            raise AssertionError(f"observer result {index} is not an object")
        if result.get("fullyExplored") is not True:
            raise AssertionError(f"observer result {index} is not fully explored")

        binding = result.get("iamBinding")
        if not isinstance(binding, dict):
            raise AssertionError(f"observer result {index} has no IAM binding")
        members = binding.get("members")
        if not isinstance(members, list):
            raise AssertionError(
                f"observer result {index} has invalid binding members"
            )
        module = load_effective_iam_module()
        if any(
            member in module.FORBIDDEN_UNIVERSAL_PRINCIPALS
            for member in members
            if isinstance(member, str)
        ):
            raise AssertionError(
                "universal principal grants effect-account access"
            )

        identities = result.get("identityList")
        if not isinstance(identities, dict):
            raise AssertionError(
                f"observer result {index} has invalid identities"
            )
        identity_entries = identities.get("identities")
        if not isinstance(identity_entries, list):
            raise AssertionError(
                f"observer result {index} has invalid identity list"
            )
        names = []
        for identity in identity_entries:
            if not isinstance(identity, dict):
                raise AssertionError(
                    f"observer result {index} has invalid identity"
                )
            name = identity.get("name")
            if not isinstance(name, str) or not name:
                raise AssertionError(
                    f"observer result {index} has invalid identity name"
                )
            names.append(name)
        if observer_principal not in names:
            continue
        observer_seen = True

        resources_seen = set()
        access_lists = result.get("accessControlLists")
        if not isinstance(access_lists, list) or not access_lists:
            raise AssertionError(
                f"observer result {index} has no access-control list"
            )
        for access_list in access_lists:
            if not isinstance(access_list, dict):
                raise AssertionError(
                    f"observer result {index} has invalid access list"
                )
            resources = access_list.get("resources")
            accesses = access_list.get("accesses")
            if not isinstance(resources, list) or not resources:
                raise AssertionError(
                    f"observer result {index} has invalid resources"
                )
            if not isinstance(accesses, list) or not accesses:
                raise AssertionError(
                    f"observer result {index} has invalid accesses"
                )
            condition = access_list.get("conditionEvaluation")
            if condition is not None:
                if not isinstance(condition, dict):
                    raise AssertionError(
                        f"observer result {index} has invalid condition"
                    )
                value = condition.get("evaluationValue")
                if value != "TRUE":
                    raise AssertionError(
                        "observer access is conditional or unresolved"
                    )
            for resource in resources:
                if not isinstance(resource, dict):
                    raise AssertionError(
                        f"observer result {index} has invalid resource"
                    )
                name = resource.get("fullResourceName")
                if name != effect_resource:
                    raise AssertionError(
                        "observer analysis targeted a different resource"
                    )
                resources_seen.add(name)
            for access in accesses:
                if not isinstance(access, dict):
                    raise AssertionError(
                        f"observer result {index} has invalid access"
                    )
                permission = access.get("permission")
                if not isinstance(permission, str):
                    raise AssertionError(
                        f"observer result {index} has invalid permission"
                    )
                observed.add(permission)
                if permission in FORBIDDEN_SERVICE_ACCOUNT_PERMISSIONS:
                    forbidden.add(permission)

    if not observer_seen:
        raise AssertionError("observer principal was not resolved")
    if forbidden:
        raise AssertionError(
            "observer can execute or mutate effect service account: "
            + ", ".join(sorted(forbidden))
        )
    if REQUIRED_OBSERVER_PERMISSION not in observed:
        raise AssertionError(
            "observer policy-read permission was not positively observed"
        )
    return {
        "observer_principal": observer_principal,
        "effect_resource": effect_resource,
        "observed_permissions": sorted(observed),
        "required_observer_permission_verified": True,
        "forbidden_effect_permissions_absent": True,
        "forbidden_permissions": list(FORBIDDEN_SERVICE_ACCOUNT_PERMISSIONS),
    }


def active_identity() -> str:
    module = load_effective_iam_module()
    return module.active_identity()


def verify(
    scope: str,
    project_id: str,
    effect_service_account: str,
    observer_service_account: str,
    output: str | None,
) -> dict[str, object]:
    module = load_effective_iam_module()
    scope = module.validate_scope(scope)
    if observer_service_account == effect_service_account:
        raise AssertionError("observer and effect service accounts must differ")
    observer_principal = service_account_principal(observer_service_account)
    if active_identity() != observer_service_account:
        raise AssertionError("active identity is not the configured observer")
    effect_resource = module.service_account_resource(
        project_id,
        effect_service_account,
    )
    payload = run_analysis(
        scope,
        observer_principal,
        effect_resource,
    )
    result = extract(
        payload,
        observer_principal,
        effect_resource,
    )
    result.update(
        {
            "schema": SCHEMA,
            "scope": scope,
            "project_id": project_id,
            "observer_service_account": observer_service_account,
            "effect_service_account": effect_service_account,
            "policy_analyzer_response_digest": digest(payload),
            "claim_ceiling": (
                "The configured IAM observer was positively observed with "
                "service-account IAM policy-read access and no listed effect "
                "service-account execution/mutation permission in the selected "
                "Policy Analyzer scope. This does not prove deny/PAB effects, "
                "other project-level privilege pivots, transitive impersonation, "
                "or immutable IAM state."
            ),
        }
    )
    if output:
        path = Path(output)
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(
            json.dumps(result, indent=2, sort_keys=True) + "\n",
            encoding="utf-8",
        )
    return result


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--scope", required=True)
    parser.add_argument("--project-id", required=True)
    parser.add_argument("--effect-service-account", required=True)
    parser.add_argument("--observer-service-account", required=True)
    parser.add_argument("--output")
    args = parser.parse_args()
    result = verify(
        args.scope,
        args.project_id,
        args.effect_service_account,
        args.observer_service_account,
        args.output,
    )
    print(
        "verified observer isolation: "
        + result["observer_service_account"]
        + " -> "
        + result["effect_service_account"]
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
