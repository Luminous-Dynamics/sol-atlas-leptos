#!/usr/bin/env python3
# Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
# SPDX-License-Identifier: AGPL-3.0-or-later

"""Offline semantic tests for qualification environment protection."""

from __future__ import annotations

import json
import importlib.util
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "verify_github_qualification_environment.py"

spec = importlib.util.spec_from_file_location("qualification_environment", SCRIPT)
assert spec and spec.loader
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


def good_environment():
    return {
        "name": module.ENVIRONMENT,
        "protection_rules": [
            {
                "type": "required_reviewers",
                "prevent_self_review": True,
                "reviewers": [
                    {
                        "type": "Team",
                        "reviewer": {"id": 123},
                    }
                ],
            }
        ],
        "deployment_branch_policy": {
            "protected_branches": False,
            "custom_branch_policies": True,
        },
    }


def good_policies():
    return {
        "total_count": 1,
        "branch_policies": [{"id": 1, "name": "main"}],
    }


def expect_failure(fn, label: str):
    try:
        fn()
    except AssertionError:
        return
    raise AssertionError("accepted invalid environment state: " + label)


def main():
    manifest = json.loads(
        (ROOT / ".github/governance/protected-qualification-environment.json").read_text(
            encoding="utf-8"
        )
    )
    assert manifest["name"] == module.ENVIRONMENT
    assert manifest["required_reviewer_count_min"] == 1
    assert manifest["prevent_self_review"] is True
    assert manifest["deployment_branch_policy"]["custom_branch_policies"] is True
    assert manifest["deployment_branch_policy"]["allowed_branch_patterns"] == [
        "main"
    ]
    verified = module.verify_environment(good_environment())
    assert verified["prevent_self_review"] is True
    assert verified["reviewer_count"] == 1
    assert module.verify_branch_policies(good_policies())["allowed_branch_patterns"] == [
        "main"
    ]

    broken = good_environment()
    broken["protection_rules"][0]["prevent_self_review"] = False
    expect_failure(lambda: module.verify_environment(broken), "self review")

    broken = good_environment()
    broken["protection_rules"][0]["reviewers"] = []
    expect_failure(lambda: module.verify_environment(broken), "missing reviewer")

    broken = good_environment()
    broken["deployment_branch_policy"]["custom_branch_policies"] = False
    expect_failure(lambda: module.verify_environment(broken), "branch policy disabled")

    expect_failure(
        lambda: module.verify_branch_policies({
            "total_count": 2,
            "branch_policies": [
                {"id": 1, "name": "main"},
                {"id": 2, "name": "release/*"},
            ],
        }),
        "additional branch",
    )

    expect_failure(
        lambda: module.verify_branch_policies({
            "total_count": 1,
            "branch_policies": [{"id": 1, "name": "dev"}],
        }),
        "wrong branch",
    )

    print("qualification environment semantic checks: PASS")


if __name__ == "__main__":
    main()
