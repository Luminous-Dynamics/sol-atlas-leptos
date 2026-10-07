#!/usr/bin/env python3
# Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
# SPDX-License-Identifier: AGPL-3.0-or-later

"""Stubbed integration tests for the qualification environment audit."""

from __future__ import annotations

import json
import importlib.util
from pathlib import Path
from tempfile import TemporaryDirectory

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "verify_github_qualification_environment.py"

spec = importlib.util.spec_from_file_location("qualification_environment_integration", SCRIPT)
assert spec and spec.loader
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


ENVIRONMENT = {
    "name": module.ENVIRONMENT,
    "protection_rules": [
        {
            "type": "required_reviewers",
            "prevent_self_review": True,
            "reviewers": [{"type": "Team", "reviewer": {"id": 123}}],
        }
    ],
    "deployment_branch_policy": {
        "protected_branches": False,
        "custom_branch_policies": True,
    },
}

POLICIES = {
    "total_count": 1,
    "branch_policies": [{"id": 99, "name": "main"}],
}


def main():
    original_env = module.get_environment
    original_policies = module.get_branch_policies
    try:
        module.get_environment = lambda: ENVIRONMENT
        module.get_branch_policies = lambda: POLICIES
        with TemporaryDirectory() as tmp:
            output = str(Path(tmp) / "environment.json")
            result = module.verify(output)
            assert result["verified"] is True
            assert result["environment"] == module.ENVIRONMENT
            written = json.loads(Path(output).read_text(encoding="utf-8"))
            assert written["branch_policy_evidence"]["allowed_branch_patterns"] == [
                "main"
            ]

        module.get_environment = lambda: {
            **ENVIRONMENT,
            "deployment_branch_policy": {
                "protected_branches": False,
                "custom_branch_policies": False,
            },
        }
        try:
            module.verify(None)
        except AssertionError:
            pass
        else:
            raise AssertionError("invalid environment policy was accepted")
    finally:
        module.get_environment = original_env
        module.get_branch_policies = original_policies

    print("qualification environment integration checks: PASS")


if __name__ == "__main__":
    main()
