#!/usr/bin/env python3
# Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
# SPDX-License-Identifier: AGPL-3.0-or-later

"""Fail-closed verifier for the trusted GCS GitHub OIDC path."""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
from pathlib import Path

PROFILE_PATH = (
    "sol-atlas-policy-store-contract/conformance/"
    "gcs_wif_trust_profile_v6.json"
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


def load_profile() -> dict[str, object]:
    profile = json.loads(Path(PROFILE_PATH).read_text(encoding="utf-8"))
    if profile.get("schema") != "sol-atlas:gcs-wif-trust-profile:v6":
        raise AssertionError("wrong WIF trust profile schema")
    if not profile.get("exact_attribute_condition"):
        raise AssertionError("WIF trust profile has no exact condition")
    if not profile.get("required_service_account_binding"):
        raise AssertionError("WIF profile has no service-account binding")
    if profile.get("pool_is_exclusive") is not True:
        raise AssertionError("WIF profile does not require an exclusive provider pool")
    if profile.get("service_account_binding_is_exclusive") is not True:
        raise AssertionError("WIF profile does not require an exclusive service-account binding")
    if profile.get("service_account_must_reside_in_project") is not True:
        raise AssertionError("WIF profile does not require same-project service account")
    if profile.get("oidc_audience_mode") != "provider_resource_default":
        raise AssertionError("WIF profile does not require the provider default audience")
    return profile


def provider_parts(resource: str) -> tuple[str, str, str]:
    parts = resource.split("/")
    if len(parts) != 8 or parts[0] != "projects" or parts[2] != "locations":
        raise AssertionError("invalid workload identity provider resource")
    if parts[4] != "workloadIdentityPools" or parts[6] != "providers":
        raise AssertionError("invalid workload identity provider resource")
    if not parts[1] or not parts[5] or not parts[7]:
        raise AssertionError("workload identity provider resource has empty identifiers")
    if parts[3] != "global":
        raise AssertionError("only global GitHub OIDC providers are supported")
    return parts[1], parts[5], parts[7]


def run_json(command: list[str]) -> dict[str, object]:
    result = subprocess.run(
        command,
        check=True,
        capture_output=True,
        text=True,
        timeout=30,
    )
    return json.loads(result.stdout)


def describe_provider(
    project_id: str,
    pool_id: str,
    provider_id: str,
) -> dict[str, object]:
    return run_json(
        [
            "gcloud",
            "iam",
            "workload-identity-pools",
            "providers",
            "describe",
            provider_id,
            "--project=" + project_id,
            "--location=global",
            f"--workload-identity-pool={pool_id}",
            "--format=json",
        ]
    )


def providers_in_pool(
    project_id: str,
    pool_id: str,
) -> list[dict[str, object]]:
    result = run_json(
        [
            "gcloud",
            "iam",
            "workload-identity-pools",
            "providers",
            "list",
            "--project=" + project_id,
            "--location=global",
            f"--workload-identity-pool={pool_id}",
            "--format=json",
        ]
    )
    if not isinstance(result, list):
        raise AssertionError("provider list response is not a list")
    return result


def providers_are_exclusive(
    providers: list[dict[str, object]],
    expected_name: str,
) -> bool:
    return len(providers) == 1 and providers[0].get("name") == expected_name


def binding_is_exclusive(
    policy: dict[str, object],
    role: str,
    member: str,
) -> bool:
    bindings = [
        binding
        for binding in policy.get("bindings", [])
        if binding.get("role") == role
    ]
    return (
        len(bindings) == 1
        and bindings[0].get("members") == [member]
        and "condition" not in bindings[0]
    )


def project_number(project_id: str) -> str:
    result = run_json(
        [
            "gcloud",
            "projects",
            "describe",
            project_id,
            "--format=json",
        ]
    )
    number = result.get("projectNumber")
    if not number:
        raise AssertionError("project has no numeric projectNumber")
    return str(number)


def service_account_project(service_account: str) -> str:
    result = run_json(
        [
            "gcloud",
            "iam",
            "service-accounts",
            "describe",
            service_account,
            "--format=json",
        ]
    )
    if not isinstance(result, dict):
        raise AssertionError("service account response is not an object")
    project = result.get("projectId")
    if not project:
        raise AssertionError("service account has no projectId")
    return str(project)


def service_account_is_in_project(
    service_account_project_id: str,
    expected_project_id: str,
) -> bool:
    return service_account_project_id == expected_project_id


def service_account_policy(service_account: str) -> dict[str, object]:
    return run_json(
        [
            "gcloud",
            "iam",
            "service-accounts",
            "get-iam-policy",
            service_account,
            "--format=json",
        ]
    )


def binding_is_present(
    policy: dict[str, object],
    role: str,
    member: str,
) -> bool:
    for binding in policy.get("bindings", []):
        if binding.get("role") != role:
            continue
        if member in binding.get("members", []):
            return True
    return False


def condition_is_exact(
    profile: dict[str, object],
    condition: str,
) -> bool:
    return condition == str(profile["exact_attribute_condition"])


def verify(
    provider_resource: str,
    configured_project_id: str,
    service_account: str,
) -> dict[str, object]:
    profile = load_profile()
    provider_project, pool_id, provider_id = provider_parts(provider_resource)
    number = project_number(configured_project_id)
    if provider_project != number:
        raise AssertionError(
            "WIF provider project number does not match configured GCP project"
        )

    provider = describe_provider(
        configured_project_id,
        pool_id,
        provider_id,
    )
    providers = providers_in_pool(configured_project_id, pool_id)
    provider_name = str(provider.get("name") or "")
    if provider_name != provider_resource:
        raise AssertionError("provider resource does not match canonical provider name")
    if not providers_are_exclusive(providers, provider_name):
        raise AssertionError("WIF provider pool contains another provider")

    mappings = provider.get("attributeMapping") or {}
    condition = str(provider.get("attributeCondition") or "")
    allowed_audiences = provider.get("allowedAudiences") or []
    if provider.get("issuerUri") != "https://token.actions.githubusercontent.com":
        raise AssertionError("unexpected GitHub OIDC issuer")
    if allowed_audiences != []:
        raise AssertionError("WIF provider permits non-default OIDC audiences")
    expected_audience = "https://iam.googleapis.com/" + provider_resource

    for name, expected in profile["required_attribute_mappings"].items():
        if mappings.get(name) != expected:
            raise AssertionError(
                f"attribute mapping drift: {name!r} != {expected!r}"
            )

    if not condition_is_exact(profile, condition):
        raise AssertionError("attribute condition differs from frozen profile")

    binding = profile["required_service_account_binding"]
    expected_member = str(binding["member_template"]).format(
        project_number=number,
        pool_id=pool_id,
    )
    sa_project = service_account_project(service_account)
    if not service_account_is_in_project(sa_project, configured_project_id):
        raise AssertionError("service account is outside configured GCP project")

    policy = service_account_policy(service_account)
    role = str(binding["role"])
    if not binding_is_exclusive(policy, role, expected_member):
        raise AssertionError(
            "service-account WIF binding is missing or non-exclusive"
        )

    provider_digest = digest(
        {
            "name": provider.get("name"),
            "attributeMapping": mappings,
            "attributeCondition": condition,
            "issuerUri": provider.get("issuerUri"),
            "allowedAudiences": allowed_audiences,
            "expectedAudience": expected_audience,
            "serviceAccount": service_account,
            "serviceAccountRole": role,
            "serviceAccountMember": expected_member,
            "attributeConditionProfile": profile,
        }
    )
    return {
        "schema": "sol-atlas:gcs-wif-trust-verification:v6",
        "provider_resource": provider_resource,
        "provider_project_number": number,
        "provider_digest": provider_digest,
        "profile_path": PROFILE_PATH,
        "profile_digest": digest(profile),
        "service_account": service_account,
        "service_account_project": sa_project,
        "service_account_project_verified": True,
        "service_account_binding_verified": True,
        "provider_pool_exclusive": True,
        "attribute_mapping_verified": True,
        "attribute_condition_verified": True,
        "oidc_audience_verified": True,
        "oidc_audience_mode": str(profile["oidc_audience_mode"]),
        "oidc_expected_audience": expected_audience,
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--provider-resource", required=True)
    parser.add_argument("--project-id", required=True)
    parser.add_argument("--service-account", required=True)
    parser.add_argument("--output")
    args = parser.parse_args()

    result = verify(
        args.provider_resource,
        args.project_id,
        args.service_account,
    )
    payload = json.dumps(result, indent=2, sort_keys=True) + "\n"
    if args.output:
        Path(args.output).parent.mkdir(parents=True, exist_ok=True)
        Path(args.output).write_text(payload, encoding="utf-8")
    print(payload, end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
