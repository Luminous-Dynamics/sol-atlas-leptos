#!/usr/bin/env python3
# Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
# SPDX-License-Identifier: AGPL-3.0-or-later

"""Verify the externally governed main-branch trust root."""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
from pathlib import Path

SCHEMA = "sol-atlas:github-governance-root-audit:v1"
REPO = "Luminous-Dynamics/sol-atlas-leptos"
DEFAULT_BRANCH = "main"
REQUIRED_CHECK = "Check"
API_VERSION = "2026-03-10"


def canonical(value: object) -> bytes:
    return json.dumps(
        value,
        sort_keys=True,
        separators=(",", ":"),
        ensure_ascii=True,
    ).encode("utf-8")


def digest(value: object) -> str:
    return "sha256:" + hashlib.sha256(canonical(value)).hexdigest()


def gh_api(path: str) -> object:
    result = subprocess.run(
        [
            "gh",
            "--hostname",
            "github.com",
            "api",
            path,
            "--header",
            "Accept: application/vnd.github+json",
            "--header",
            "X-GitHub-Api-Version: " + API_VERSION,
        ],
        check=True,
        capture_output=True,
        text=True,
        timeout=30,
    )
    try:
        return json.loads(result.stdout)
    except json.JSONDecodeError as exc:
        raise AssertionError("GitHub API returned invalid JSON") from exc


def list_rulesets() -> list[dict[str, object]]:
    payload = gh_api(
        "/repos/"
        + REPO
        + "/rulesets?includes_parents=true&targets=branch&per_page=100"
    )
    if not isinstance(payload, list):
        raise AssertionError("repository rulesets response is not a list")
    if len(payload) >= 100:
        raise AssertionError("ruleset response may be truncated")
    for index, ruleset in enumerate(payload):
        if not isinstance(ruleset, dict):
            raise AssertionError(
                f"ruleset summary {index} is not an object"
            )
    return payload


def fetch_ruleset(ruleset_id: int) -> dict[str, object]:
    payload = gh_api(
        "/repos/"
        + REPO
        + "/rulesets/"
        + str(ruleset_id)
    )
    if not isinstance(payload, dict):
        raise AssertionError("ruleset detail is not an object")
    return payload


def targets_default_branch(ruleset: dict[str, object]) -> bool:
    conditions = ruleset.get("conditions")
    if not isinstance(conditions, dict):
        return False
    refs = conditions.get("ref_name")
    if not isinstance(refs, dict):
        return False
    include = refs.get("include")
    return isinstance(include, list) and (
        "refs/heads/" + DEFAULT_BRANCH in include
        or "~DEFAULT_BRANCH" in include
    )


def has_rule(
    rules: list[dict[str, object]],
    rule_type: str,
) -> dict[str, object] | None:
    matches = [
        rule
        for rule in rules
        if isinstance(rule, dict) and rule.get("type") == rule_type
    ]
    if len(matches) != 1:
        return None
    return matches[0]


def verify_ruleset(
    ruleset: dict[str, object],
) -> dict[str, object]:
    if ruleset.get("target") != "branch":
        raise AssertionError("governance ruleset is not a branch ruleset")
    if ruleset.get("enforcement") not in ("active", "enabled"):
        raise AssertionError("governance ruleset is not active")
    if not targets_default_branch(ruleset):
        raise AssertionError("governance ruleset does not target main")

    raw_rules = ruleset.get("rules")
    if not isinstance(raw_rules, list):
        raise AssertionError("governance rules are not a list")
    rules = [rule for rule in raw_rules if isinstance(rule, dict)]

    for required in (
        "pull_request",
        "required_signatures",
        "deletion",
        "non_fast_forward",
        "required_status_checks",
    ):
        if has_rule(rules, required) is None:
            raise AssertionError(
                "required governance rule missing: " + required
            )

    status_rule = has_rule(rules, "required_status_checks")
    assert status_rule is not None
    parameters = status_rule.get("parameters")
    if not isinstance(parameters, dict):
        raise AssertionError("required status check parameters missing")
    strict = parameters.get("strict_required_status_checks_policy")
    if strict is not True:
        raise AssertionError("required status checks are not strict")

    checks = parameters.get("required_status_checks")
    if not isinstance(checks, list):
        raise AssertionError("required status check list is missing")
    matching_checks = [
        check
        for check in checks
        if isinstance(check, dict)
        and check.get("context") == REQUIRED_CHECK
    ]
    if len(matching_checks) != 1:
        raise AssertionError(
            "exactly one required Check context is required"
        )
    integration_id = matching_checks[0].get("integration_id")
    if not isinstance(integration_id, int) or integration_id <= 0:
        raise AssertionError(
            "required Check must be bound to a specific GitHub App"
        )

    bypass = ruleset.get("bypass_actors")
    if not isinstance(bypass, list):
        raise AssertionError(
            "bypass actors are not observable; governance audit is incomplete"
        )
    for actor in bypass:
        if not isinstance(actor, dict):
            raise AssertionError("invalid bypass actor")
        if actor.get("bypass_mode") != "pull_request":
            raise AssertionError(
                "governance bypass must require a pull request"
            )

    return {
        "id": ruleset.get("id"),
        "name": ruleset.get("name"),
        "source_type": ruleset.get("source_type"),
        "source": ruleset.get("source"),
        "enforcement": ruleset.get("enforcement"),
        "target": ruleset.get("target"),
        "required_check": REQUIRED_CHECK,
        "required_check_integration_id": integration_id,
        "bypass_actor_count": len(bypass),
        "rules": sorted(
            str(rule.get("type"))
            for rule in rules
            if rule.get("type") is not None
        ),
    }


def verify(
    output: str | None,
) -> dict[str, object]:
    summaries = list_rulesets()
    details = []
    for summary in summaries:
        ruleset_id = summary.get("id")
        if isinstance(ruleset_id, int):
            details.append(fetch_ruleset(ruleset_id))

    candidates = [
        detail
        for detail in details
        if detail.get("target") == "branch"
        and detail.get("enforcement") in ("active", "enabled")
        and targets_default_branch(detail)
    ]
    if len(candidates) != 1:
        raise AssertionError(
            "exactly one active governance ruleset must target main"
        )

    verified = verify_ruleset(candidates[0])
    result: dict[str, object] = {
        "schema": SCHEMA,
        "repository": REPO,
        "default_branch": DEFAULT_BRANCH,
        "ruleset_count": len(summaries),
        "main_ruleset_count": len(candidates),
        "verified": True,
        "ruleset": verified,
        "ruleset_response_digest": digest(candidates[0]),
        "all_rulesets_response_digest": digest(
            {"summaries": summaries, "details": details}
        ),
        "claim_ceiling": (
            "The observed GitHub ruleset currently enforces the selected "
            "main-branch protections and a specific GitHub App source for "
            "the required Check context. This does not prove administrator "
            "or organization-owner intent outside the ruleset, legacy branch "
            "protection that is inaccessible to the observer, or governance "
            "outside GitHub."
        ),
    }
    if output:
        destination = Path(output)
        destination.parent.mkdir(parents=True, exist_ok=True)
        destination.write_text(
            json.dumps(result, indent=2, sort_keys=True) + "\n",
            encoding="utf-8",
        )
    return result


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output")
    args = parser.parse_args()
    result = verify(args.output)
    print(
        "verified GitHub governance root: "
        + str(result["ruleset"]["id"])
        + " "
        + str(result["ruleset"]["required_check_integration_id"])
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
