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
    ".github/workflows/qualify-gcs.yml@refs/heads/main'"
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
    assert module.condition_is_exact(
        {"exact_attribute_condition": EXPECTED},
        EXPECTED,
    )
    assert not module.condition_is_exact(
        {"exact_attribute_condition": EXPECTED},
        "(" + EXPECTED + ") || true",
    )

    assert module.binding_is_exclusive(
        binding_policy("expected"),
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
