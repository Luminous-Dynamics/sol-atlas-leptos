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

SCHEMA = "sol-atlas:gcs-observer-isolation-audit:v2"
OBSERVER_PROFILE_PATH = (
    "sol-atlas-policy-store-contract/conformance/"
    "gcs_iam_observer_profile_v1.json"
)
OBSERVER_PROFILE_SCHEMA = "sol-atlas:gcs-iam-observer-profile:v1"


def load_observer_profile() -> dict[str, object]:
    path = Path(__file__).resolve().parents[1] / OBSERVER_PROFILE_PATH
    profile = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(profile, dict):
        raise AssertionError("observer profile is not an object")
    if profile.get("schema") != OBSERVER_PROFILE_SCHEMA:
        raise AssertionError("observer profile schema drift")
    list_fields = (
        "forbidden_service_account_permissions",
        "forbidden_project_permissions",
        "project_pivot_permissions",
    )
    for field in list_fields:
        values = profile.get(field)
        if (
            not isinstance(values, list)
            or not values
            or any(not isinstance(value, str) or not value for value in values)
        ):
            raise AssertionError(
                "observer profile field is malformed: " + field
            )
    required = profile.get("required_observer_permission")
    if not isinstance(required, str) or not required:
        raise AssertionError("observer profile required permission is malformed")
    trust_path = profile.get("trust_profile_path")
    trust_schema = profile.get("trust_profile_schema")
    if trust_path != (
        "sol-atlas-policy-store-contract/conformance/"
        "gcs_wif_trust_profile_v9.json"
    ):
        raise AssertionError("observer trust profile path drift")
    if trust_schema != "sol-atlas:gcs-wif-trust-profile:v9":
        raise AssertionError("observer trust profile schema drift")
    return profile


OBSERVER_PROFILE = load_observer_profile()
FORBIDDEN_SERVICE_ACCOUNT_PERMISSIONS = tuple(
    OBSERVER_PROFILE["forbidden_service_account_permissions"]
)
REQUIRED_OBSERVER_PERMISSION = OBSERVER_PROFILE[
    "required_observer_permission"
]
OBSERVER_FORBIDDEN_PROJECT_PERMISSIONS = tuple(
    OBSERVER_PROFILE["forbidden_project_permissions"]
)
PROJECT_PIVOT_PERMISSIONS = tuple(
    OBSERVER_PROFILE["project_pivot_permissions"]
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
    forbidden_permissions: tuple[str, ...],
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

        binding = result.get("iamBinding")
        if not isinstance(binding, dict):
            raise AssertionError(
                f"observer project-pivot result {index} has no IAM binding"
            )
        members = binding.get("members")
        if not isinstance(members, list):
            raise AssertionError(
                f"observer project-pivot result {index} has invalid members"
            )
        module = load_effective_iam_module()
        if any(
            member in module.FORBIDDEN_UNIVERSAL_PRINCIPALS
            for member in members
            if isinstance(member, str)
        ):
            raise AssertionError(
                "universal principal grants observer project pivot access"
            )

        access_lists = result.get("accessControlLists")
        if not isinstance(access_lists, list) or not access_lists:
            raise AssertionError(
                f"observer result {index} has no access-control list"
            )
        preliminary_permissions = set()
        for access_list in access_lists:
            if not isinstance(access_list, dict):
                raise AssertionError(
                    f"observer result {index} has invalid access list"
                )
            accesses = access_list.get("accesses")
            if not isinstance(accesses, list) or not accesses:
                raise AssertionError(
                    f"observer result {index} has invalid accesses"
                )
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
                preliminary_permissions.add(permission)
        if preliminary_permissions.intersection(forbidden_permissions):
            raise AssertionError(
                "observer binding carries a forbidden effect permission"
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
            raise AssertionError(
                "observer identity query returned an unrelated IAM binding"
            )
        if members != [observer_principal]:
            raise AssertionError(
                "observer access is granted through a broader IAM member"
            )
        observer_seen = True

        resources_seen = set()
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


def run_observer_authority_analysis(
    scope: str,
    identity: str,
    project_resource: str,
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
            "--full-resource-name=" + project_resource,
            "--permissions=" + ",".join(
                OBSERVER_FORBIDDEN_PROJECT_PERMISSIONS
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
        raise AssertionError(
            "observer authority response is not an object"
        )
    if payload.get("fullyExplored") is not True:
        raise AssertionError(
            "observer authority response is not fully explored"
        )
    errors = payload.get("nonCriticalErrors")
    if errors is not None and (
        not isinstance(errors, list) or errors
    ):
        raise AssertionError(
            "observer authority response reported errors"
        )
    results = payload.get("analysisResults")
    if not isinstance(results, list):
        raise AssertionError(
            "observer authority analysisResults is not a list"
        )
    return payload


def extract_observer_authority(
    payload: dict[str, object],
    observer_principal: str,
    project_resource: str,
) -> set[str]:
    if payload.get("fullyExplored") is not True:
        raise AssertionError(
            "observer authority response is not fully explored"
        )
    errors = payload.get("nonCriticalErrors")
    if errors is not None and (
        not isinstance(errors, list) or errors
    ):
        raise AssertionError(
            "observer authority response reported errors"
        )
    results = payload.get("analysisResults")
    if not isinstance(results, list):
        raise AssertionError(
            "observer authority analysisResults is not a list"
        )

    observed: set[str] = set()
    module = load_effective_iam_module()
    for index, result in enumerate(results):
        if not isinstance(result, dict):
            raise AssertionError(
                f"observer authority result {index} is not an object"
            )
        if result.get("fullyExplored") is not True:
            raise AssertionError(
                f"observer authority result {index} is not fully explored"
            )

        identities = result.get("identityList")
        if not isinstance(identities, dict):
            raise AssertionError(
                f"observer authority result {index} has invalid identities"
            )
        identity_entries = identities.get("identities")
        if not isinstance(identity_entries, list):
            raise AssertionError(
                f"observer authority result {index} has invalid identity list"
            )
        identity_names: list[str] = []
        for identity in identity_entries:
            if not isinstance(identity, dict):
                raise AssertionError(
                    f"observer authority result {index} has invalid identity"
                )
            name = identity.get("name")
            if not isinstance(name, str) or not name:
                raise AssertionError(
                    f"observer authority result {index} has invalid identity name"
                )
            identity_names.append(name)
        if observer_principal not in identity_names:
            continue

        binding = result.get("iamBinding")
        if not isinstance(binding, dict):
            raise AssertionError(
                f"observer authority result {index} has no binding"
            )
        members = binding.get("members")
        if not isinstance(members, list):
            raise AssertionError(
                f"observer authority result {index} has invalid members"
            )
        if members != [observer_principal]:
            raise AssertionError(
                "observer authority is granted through a broader IAM member"
            )
        if any(
            member in module.FORBIDDEN_UNIVERSAL_PRINCIPALS
            for member in members
            if isinstance(member, str)
        ):
            raise AssertionError(
                "observer authority is granted through a universal principal"
            )

        access_lists = result.get("accessControlLists")
        if not isinstance(access_lists, list) or not access_lists:
            raise AssertionError(
                f"observer authority result {index} has no ACL"
            )
        for access_list in access_lists:
            if not isinstance(access_list, dict):
                raise AssertionError(
                    f"observer authority result {index} has invalid ACL"
                )
            resources = access_list.get("resources")
            accesses = access_list.get("accesses")
            if not isinstance(resources, list) or not resources:
                raise AssertionError(
                    f"observer authority result {index} has invalid resources"
                )
            if not isinstance(accesses, list) or not accesses:
                raise AssertionError(
                    f"observer authority result {index} has invalid accesses"
                )
            condition = access_list.get("conditionEvaluation")
            if condition is not None:
                if not isinstance(condition, dict):
                    raise AssertionError(
                        f"observer authority result {index} has invalid condition"
                    )
                value = condition.get("evaluationValue")
                if value == "FALSE":
                    continue
                if value != "TRUE":
                    raise AssertionError(
                        "observer project authority is conditional or unresolved"
                    )
            for resource in resources:
                if not isinstance(resource, dict):
                    raise AssertionError(
                        f"observer authority result {index} has invalid resource"
                    )
                if resource.get("fullResourceName") != project_resource:
                    raise AssertionError(
                        "observer authority targeted a different project"
                    )
            for access in accesses:
                if not isinstance(access, dict):
                    raise AssertionError(
                        f"observer authority result {index} has invalid access"
                    )
                permission = access.get("permission")
                if not isinstance(permission, str):
                    raise AssertionError(
                        f"observer authority result {index} has invalid permission"
                    )
                if permission in OBSERVER_FORBIDDEN_PROJECT_PERMISSIONS:
                    observed.add(permission)

    return observed


def run_project_pivot_analysis(
    scope: str,
    identity: str,
    project_resource: str,
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
            "--full-resource-name=" + project_resource,
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
        raise AssertionError("observer project-pivot response is not an object")
    if payload.get("fullyExplored") is not True:
        raise AssertionError("observer project-pivot response is not fully explored")
    errors = payload.get("nonCriticalErrors")
    if errors is not None and (
        not isinstance(errors, list) or errors
    ):
        raise AssertionError("observer project-pivot response reported errors")
    if not isinstance(payload.get("analysisResults"), list):
        raise AssertionError(
            "observer project-pivot analysisResults is not a list"
        )
    return payload


def extract_project_pivots(
    payload: dict[str, object],
    project_resource: str,
) -> set[str]:
    results = payload.get("analysisResults")
    if not isinstance(results, list):
        raise AssertionError("observer project-pivot results are not a list")
    observed: set[str] = set()
    for index, result in enumerate(results):
        if not isinstance(result, dict):
            raise AssertionError(
                f"observer project-pivot result {index} is not an object"
            )
        if result.get("fullyExplored") is not True:
            raise AssertionError(
                f"observer project-pivot result {index} is not fully explored"
            )
        access_lists = result.get("accessControlLists")
        if not isinstance(access_lists, list) or not access_lists:
            raise AssertionError(
                f"observer project-pivot result {index} has no ACL"
            )
        for access_list in access_lists:
            if not isinstance(access_list, dict):
                raise AssertionError(
                    f"observer project-pivot result {index} has invalid ACL"
                )
            resources = access_list.get("resources")
            accesses = access_list.get("accesses")
            if not isinstance(resources, list) or not resources:
                raise AssertionError(
                    f"observer project-pivot result {index} has invalid resources"
                )
            if not isinstance(accesses, list) or not accesses:
                raise AssertionError(
                    f"observer project-pivot result {index} has invalid accesses"
                )
            condition = access_list.get("conditionEvaluation")
            if condition is not None:
                if not isinstance(condition, dict):
                    raise AssertionError(
                        f"observer project-pivot result {index} has invalid condition"
                    )
                if condition.get("evaluationValue") != "TRUE":
                    raise AssertionError(
                        "observer project-pivot access is conditional or unresolved"
                    )
            for resource in resources:
                if not isinstance(resource, dict):
                    raise AssertionError(
                        f"observer project-pivot result {index} has invalid resource"
                    )
                if resource.get("fullResourceName") != project_resource:
                    raise AssertionError(
                        "observer project-pivot targeted a different project"
                    )
            for access in accesses:
                if not isinstance(access, dict):
                    raise AssertionError(
                        f"observer project-pivot result {index} has invalid access"
                    )
                permission = access.get("permission")
                if not isinstance(permission, str):
                    raise AssertionError(
                        f"observer project-pivot result {index} has invalid permission"
                    )
                observed.add(permission)
    return observed


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
        FORBIDDEN_SERVICE_ACCOUNT_PERMISSIONS,
    )
    project_resource = module.project_resource(project_id)
    project_payload = run_project_pivot_analysis(
        scope,
        observer_principal,
        project_resource,
    )
    project_permissions = extract_project_pivots(
        project_payload,
        project_resource,
    )
    authority_payload = run_observer_authority_analysis(
        scope,
        observer_principal,
        project_resource,
    )
    observer_authority = extract_observer_authority(
        authority_payload,
        observer_principal,
        project_resource,
    )
    if observer_authority:
        raise AssertionError(
            "observer has a forbidden project-level trust authority: "
            + ", ".join(sorted(observer_authority))
        )
    if project_permissions.intersection(PROJECT_PIVOT_PERMISSIONS):
        raise AssertionError(
            "observer has a project-level execution or policy pivot: "
            + ", ".join(sorted(
                project_permissions.intersection(PROJECT_PIVOT_PERMISSIONS)
            ))
        )
    result.update(
        {
            "schema": SCHEMA,
            "scope": scope,
            "project_id": project_id,
            "observer_service_account": observer_service_account,
            "effect_service_account": effect_service_account,
            "observer_profile_path": OBSERVER_PROFILE_PATH,
            "observer_profile_digest": digest(OBSERVER_PROFILE),
            "policy_analyzer_response_digest": digest(payload),
            "project_pivot_permissions": list(PROJECT_PIVOT_PERMISSIONS),
            "project_pivot_permissions_absent": True,
            "project_pivot_observed_permissions": sorted(project_permissions),
            "project_pivot_response_digest": digest(project_payload),
            "observer_authority_permissions": list(
                OBSERVER_FORBIDDEN_PROJECT_PERMISSIONS
            ),
            "observer_authority_permissions_absent": True,
            "observer_authority_observed_permissions": sorted(
                observer_authority
            ),
            "observer_authority_response_digest": digest(
                authority_payload
            ),
            "project_pivot_permissions": list(PROJECT_PIVOT_PERMISSIONS),
            "project_pivot_permissions_absent": True,
            "claim_ceiling": (
                "The configured IAM observer was positively observed with "
                "service-account IAM policy-read access and no listed effect "
                "service-account execution/mutation permission in the selected "
                "Policy Analyzer scope. This does not prove deny/PAB effects, "
                "other project-level privilege pivots, transitive impersonation, "
                "or immutable IAM state. Observer authority is bounded "
                "only to the listed project-level trust/policy mutation "
                "permissions; other unrelated project permissions remain outside "
                "this audit."
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
