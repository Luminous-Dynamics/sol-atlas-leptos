#!/usr/bin/env python3
# Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
# SPDX-License-Identifier: AGPL-3.0-or-later

"""Offline semantic tests for the GitHub governance root auditor."""

from __future__ import annotations

import importlib.util
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "verify_github_governance_root.py"

spec = importlib.util.spec_from_file_location("governance_root", SCRIPT)
assert spec and spec.loader
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


def rule(rule_type: str, parameters=None):
    result = {"type": rule_type}
    if parameters is not None:
        result["parameters"] = parameters
    return result


def good_ruleset():
    return {
        "id": 77,
        "name": "Protected main governance root",
        "source_type": "Repository",
        "source": module.REPO,
        "enforcement": "active",
        "target": "branch",
        "conditions": {
            "ref_name": {
                "include": ["refs/heads/main"],
                "exclude": [],
            }
        },
        "rules": [
            rule(
                "pull_request",
                {
                    "required_approving_review_count": 1,
                    "dismiss_stale_reviews_on_push": True,
                    "require_last_push_approval": True,
                },
            ),
            rule("required_signatures"),
            rule("deletion"),
            rule("non_fast_forward"),
            rule(
                "required_status_checks",
                {
                    "strict_required_status_checks_policy": True,
                    "required_status_checks": [
                        {
                            "context": "Check",
                            "integration_id": 15368,
                        }
                    ],
                },
            ),
        ],
        "bypass_actors": [
            {
                "actor_id": 42,
                "actor_type": "Team",
                "bypass_mode": "pull_request",
            }
        ],
    }


def expect_failure(ruleset, label: str):
    try:
        module.verify_ruleset(ruleset)
    except AssertionError:
        return
    raise AssertionError("accepted invalid governance root: " + label)


def main():
    verified = module.verify_ruleset(good_ruleset())
    assert verified["required_check"] == "Check"
    assert verified["required_check_integration_id"] == 15368

    for rule_type in (
        "pull_request",
        "required_signatures",
        "deletion",
        "non_fast_forward",
        "required_status_checks",
    ):
        broken = good_ruleset()
        broken["rules"] = [
            r for r in broken["rules"]
            if r["type"] != rule_type
        ]
        expect_failure(broken, "missing " + rule_type)

    broken = good_ruleset()
    broken["rules"][0]["parameters"]["required_approving_review_count"] = "1"
    expect_failure(broken, "non-integer approval count")

    broken = good_ruleset()
    broken["rules"].append(None)
    expect_failure(broken, "malformed rule entry")

    layered = good_ruleset()
    layered["id"] = 78
    verified_layered = module.verify_ruleset(layered)
    assert verified_layered["id"] == 78

    broken = good_ruleset()
    broken["conditions"]["ref_name"]["include"] = ["refs/heads/dev"]
    expect_failure(broken, "wrong target")

    broken = good_ruleset()
    broken["enforcement"] = "evaluate"
    expect_failure(broken, "non-active")

    broken = good_ruleset()
    broken["rules"][-1]["parameters"]["strict_required_status_checks_policy"] = False
    expect_failure(broken, "non-strict status checks")

    broken = good_ruleset()
    broken["rules"][-1]["parameters"]["required_status_checks"][0][
        "integration_id"
    ] = None
    expect_failure(broken, "unbound status-check source")

    broken = good_ruleset()
    broken["bypass_actors"][0]["bypass_mode"] = "always"
    expect_failure(broken, "unreviewed bypass")

    broken = good_ruleset()
    broken["bypass_actors"] = None
    expect_failure(broken, "unobservable bypass policy")

    print("GitHub governance root semantic checks: PASS")


if __name__ == "__main__":
    main()
