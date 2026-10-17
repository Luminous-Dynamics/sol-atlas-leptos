#!/usr/bin/env python3
# Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
# SPDX-License-Identifier: AGPL-3.0-or-later

"""Offline tests for GitHub immutable subject configuration."""

from __future__ import annotations

import importlib.util
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "verify_github_oidc_subject_configuration.py"

spec = importlib.util.spec_from_file_location("oidc_subject_config", SCRIPT)
assert spec and spec.loader
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


def main() -> None:
    good = {
        "use_default": True,
        "include_claim_keys": [],
        "use_immutable_subject": True,
    }
    result = module.verify(good, None)
    assert result["schema"] == module.SCHEMA
    assert result["use_immutable_subject"] is True
    assert result["default_template_required"] is True
    assert result["custom_claim_keys_forbidden"] is True
    assert result["configuration"] == good
    assert result["configuration_digest"] == module.digest(good)
    assert result["configuration_digest"] != module.digest(
        dict(good, use_immutable_subject=False)
    )

    for bad in (
        {"use_default": True, "include_claim_keys": []},
        {
            "use_default": False,
            "include_claim_keys": ["repo", "context"],
            "use_immutable_subject": False,
        },
        {
            "use_default": True,
            "include_claim_keys": [],
            "use_immutable_subject": False,
        },
        {
            "use_default": False,
            "include_claim_keys": [],
            "use_immutable_subject": True,
        },
        {
            "use_default": True,
            "include_claim_keys": ["repository_id"],
            "use_immutable_subject": True,
        },
    ):
        try:
            module.verify(bad, None)
        except AssertionError:
            pass
        else:
            raise AssertionError("non-immutable OIDC configuration was accepted")

    tampered = dict(result, configuration=dict(good, use_default=False))
    if tampered["configuration_digest"] == module.digest(
        tampered["configuration"]
    ):
        raise AssertionError(
            "tampered raw configuration unexpectedly retained original digest"
        )

    print("offline immutable OIDC subject configuration checks: PASS")


if __name__ == "__main__":
    main()
