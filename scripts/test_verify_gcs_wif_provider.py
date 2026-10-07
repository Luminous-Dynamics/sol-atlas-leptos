#!/usr/bin/env python3
# Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
# SPDX-License-Identifier: AGPL-3.0-or-later

"""Offline semantic tests for the GCS WIF verifier."""

from __future__ import annotations

import importlib.util
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "verify_gcs_wif_provider.py"

spec = importlib.util.spec_from_file_location("verify_gcs_wif_provider", SCRIPT)
assert spec and spec.loader
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)

EXPECTED = (
    "assertion.environment=='sol-atlas-gcs-qualification' && "
    "assertion.event_name=='workflow_dispatch' && "
    "assertion.repository=='Luminous-Dynamics/sol-atlas-leptos' && "
    "assertion.repository_id=='1195997641' && "
    "assertion.repository_owner_id=='216969177' && "
    "assertion.workflow=='Qualify GCS external effect' && "
    "assertion.ref=='refs/heads/main' && "
    "assertion.workflow_ref=='Luminous-Dynamics/sol-atlas-leptos/"
    ".github/workflows/qualify-gcs.yml@refs/heads/main' && "
    "assertion.runner_environment=='github-hosted'"
)


def binding_policy(member: str) -> dict[str, object]:
    return {
        "bindings": [
            {
                "role": "roles/iam.workloadIdentityUser",
                "members": [member],
            }
        ]
    }


def main() -> None:
    profile = module.load_profile()
    assert profile["schema"] == "sol-atlas:gcs-wif-trust-profile:v9"
    assert profile["supersedes"] == "sol-atlas:gcs-wif-trust-profile:v8"
    assert profile["exact_attribute_condition"] == EXPECTED
    assert (
        profile["required_attribute_mappings"]["attribute.runner_environment"]
        == "assertion.runner_environment"
    )
    assert profile["pool_is_exclusive"] is True
    assert profile["service_account_binding_is_exclusive"] is True
    assert profile["service_account_direct_policy_is_exact"] is True
    assert profile["attribute_mapping_is_exact"] is True
    assert profile["service_account_must_reside_in_project"] is True
    assert profile["oidc_audience_mode"] == "provider_resource_default"
    mappings = profile["required_attribute_mappings"]
    assert module.attribute_mappings_are_exact(mappings, profile)
    widened_mappings = dict(mappings)
    widened_mappings["attribute.unexpected"] = "assertion.repository_visibility"
    assert not module.attribute_mappings_are_exact(widened_mappings, profile)

    assert module.condition_is_exact(
        {"exact_attribute_condition": EXPECTED},
        EXPECTED,
    )
    assert not module.condition_is_exact(
        {"exact_attribute_condition": EXPECTED},
        "(" + EXPECTED + ") || true",
    )

    assert module.service_account_is_in_project("project-a", "project-a")
    assert not module.service_account_is_in_project("project-b", "project-a")

    original_text = module.run_text
    module.run_text = lambda command: (
        "observer@sol-atlas.iam.gserviceaccount.com\n"
    )
    try:
        assert module.active_identity() == (
            "observer@sol-atlas.iam.gserviceaccount.com"
        )
        module.run_text = lambda command: (
            "observer@sol-atlas.iam.gserviceaccount.com\n"
            "second@sol-atlas.iam.gserviceaccount.com\n"
        )
        try:
            module.active_identity()
        except AssertionError:
            pass
        else:
            raise AssertionError("multiple active identities were accepted")
    finally:
        module.run_text = original_text

    original_identity = module.active_identity
    module.active_identity = lambda: "observer@sol-atlas.iam.gserviceaccount.com"
    try:
        assert module.observer_identity_is_exact(
            "observer@sol-atlas.iam.gserviceaccount.com",
            "effect@sol-atlas.iam.gserviceaccount.com",
        )
        assert not module.observer_identity_is_exact(
            "effect@sol-atlas.iam.gserviceaccount.com",
            "effect@sol-atlas.iam.gserviceaccount.com",
        )
        assert not module.observer_identity_is_exact(
            "other@sol-atlas.iam.gserviceaccount.com",
            "effect@sol-atlas.iam.gserviceaccount.com",
        )
    finally:
        module.active_identity = original_identity

    assert module.binding_is_exclusive(
        binding_policy("expected"),
        "roles/iam.workloadIdentityUser",
        "expected",
    )
    assert module.direct_policy_is_exact(
        binding_policy("expected"),
        "roles/iam.workloadIdentityUser",
        "expected",
    )
    assert not module.direct_policy_is_exact(
        {
            "bindings": [
                {
                    "role": "roles/iam.workloadIdentityUser",
                    "members": ["expected"],
                },
                {
                    "role": "roles/viewer",
                    "members": ["expected"],
                },
            ]
        },
        "roles/iam.workloadIdentityUser",
        "expected",
    )
    assert not module.direct_policy_is_exact(
        {
            "bindings": [
                {
                    "role": "customRoles/alternate",
                    "members": ["expected"],
                }
            ]
        },
        "roles/iam.workloadIdentityUser",
        "expected",
    )
    assert not module.direct_policy_is_exact(
        {
            "bindings": [
                {
                    "role": "roles/iam.workloadIdentityUser",
                    "members": ["expected"],
                    "condition": {"title": "unexpected"},
                }
            ]
        },
        "roles/iam.workloadIdentityUser",
        "expected",
    )
    assert not module.binding_is_exclusive(
        {
            "bindings": [
                {
                    "role": "roles/iam.workloadIdentityUser",
                    "members": ["expected", "other"],
                }
            ]
        },
        "roles/iam.workloadIdentityUser",
        "expected",
    )

    assert module.binding_is_present(
        binding_policy("expected"),
        "roles/iam.workloadIdentityUser",
        "expected",
    )
    assert not module.binding_is_present(
        binding_policy("other"),
        "roles/iam.workloadIdentityUser",
        "expected",
    )

    assert module.provider_parts(
        "projects/123/locations/global/"
        "workloadIdentityPools/pool/providers/github"
    ) == ("123", "pool", "github")

    try:
        module.provider_parts(
            "projects/123/locations/us-central1/"
            "workloadIdentityPools/pool/providers/github"
        )
    except AssertionError:
        pass
    else:
        raise AssertionError("non-global provider was accepted")

    forbidden = module.forbidden_direct_roles(profile)
    assert "roles/iam.serviceAccountTokenCreator" in forbidden
    assert module.direct_policy_has_forbidden_roles(
        {"bindings": [{"role": "roles/iam.serviceAccountTokenCreator"}]},
        forbidden,
    )
    assert module.direct_policy_has_forbidden_roles(
        {"bindings": [{"role": "roles/iam.serviceAccountKeyAdmin"}]},
        forbidden,
    )
    assert not module.direct_policy_has_forbidden_roles(
        binding_policy("expected"),
        forbidden,
    )

    assert module.providers_are_exclusive(
        [{"name": "expected"}],
        "expected",
    )
    assert not module.providers_are_exclusive(
        [{"name": "expected"}, {"name": "other"}],
        "expected",
    )

    print("offline WIF verifier semantic checks: PASS")


if __name__ == "__main__":
    main()
