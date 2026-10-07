#!/usr/bin/env python3
# Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
# SPDX-License-Identifier: AGPL-3.0-or-later

"""Stubbed integration tests for the GitHub governance root audit."""

from __future__ import annotations

import json
import importlib.util
from pathlib import Path
from tempfile import TemporaryDirectory

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "verify_github_governance_root.py"

spec = importlib.util.spec_from_file_location(
    "governance_root_integration",
    SCRIPT,
)
assert spec and spec.loader
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


def good_ruleset(ruleset_id: int = 77):
    return {
        "id": ruleset_id,
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
            {
                "type": "pull_request",
                "parameters": {
                    "required_approving_review_count": 1,
                    "dismiss_stale_reviews_on_push": True,
                    "require_last_push_approval": True,
                },
            },
            {"type": "required_signatures"},
            {"type": "deletion"},
            {"type": "non_fast_forward"},
            {
                "type": "required_status_checks",
                "parameters": {
                    "strict_required_status_checks_policy": True,
                    "required_status_checks": [
                        {
                            "context": "Check",
                            "integration_id": 15368,
                        }
                    ],
                },
            },
        ],
        "bypass_actors": [],
    }


def main() -> None:
    original_list = module.list_rulesets
    original_fetch = module.fetch_ruleset
    try:
        module.list_rulesets = lambda: []
        module.fetch_ruleset = lambda ruleset_id: good_ruleset(ruleset_id)
        try:
            module.verify(None)
        except AssertionError:
            pass
        else:
            raise AssertionError("empty governance root was accepted")

        module.list_rulesets = lambda: [
            {"id": 77},
            {"id": 78},
        ]
        module.fetch_ruleset = lambda ruleset_id: (
            good_ruleset(ruleset_id)
            if ruleset_id == 77
            else {
                **good_ruleset(ruleset_id),
                "rules": [],
            }
        )
        with TemporaryDirectory() as tmp:
            output = str(Path(tmp) / "governance.json")
            result = module.verify(output)
            assert result["verified"] is True
            assert result["qualifying_main_ruleset_count"] == 1
            assert result["ruleset"]["id"] == 77
            written = json.loads(
                Path(output).read_text(encoding="utf-8")
            )
            assert written["ruleset"]["required_check"] == "Check"
    finally:
        module.list_rulesets = original_list
        module.fetch_ruleset = original_fetch

    print("GitHub governance root integration checks: PASS")


if __name__ == "__main__":
    main()
