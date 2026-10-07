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
REQUIRED_CHECK_INTEGRATION_ID = 15368
EXPECTED_RULESET_NAME = "Sol Atlas main governance root"
EXPECTED_REPOSITORY_ID = 1195997641
REQUIRED_RULESET_SOURCE_TYPE = "Organization"
REQUIRED_RULESET_SOURCE = "Luminous-Dynamics"
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


def targets_exact_repository(ruleset: dict[str, object]) -> bool:
    conditions = ruleset.get("conditions")
    if not isinstance(conditions, dict):
        return False
    repository_id = conditions.get("repository_id")
    ref_name = conditions.get("ref_name")
    if not isinstance(repository_id, dict) or not isinstance(ref_name, dict):
        return False
    return (
        repository_id.get("include") == [EXPECTED_REPOSITORY_ID]
        and repository_id.get("exclude") == []
        and ref_name.get("include") == ["refs/heads/" + DEFAULT_BRANCH]
        and ref_name.get("exclude") == []
    )


def matches_expected_name(ruleset: dict[str, object]) -> bool:
    return ruleset.get("name") == EXPECTED_RULESET_NAME

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
    if ruleset.get("enforcement") != "active":
        raise AssertionError("governance ruleset is not active")
    if ruleset.get("source_type") != REQUIRED_RULESET_SOURCE_TYPE:
        raise AssertionError("governance root is not organization-owned")
    if ruleset.get("source") != REQUIRED_RULESET_SOURCE:
        raise AssertionError("governance root organization mismatch")
    if not targets_exact_repository(ruleset):
        raise AssertionError("governance ruleset does not target exact repository and main")
    if not matches_expected_name(ruleset):
        raise AssertionError("governance ruleset name does not match frozen root")

    raw_rules = ruleset.get("rules")
    if not isinstance(raw_rules, list):
        raise AssertionError("governance rules are not a list")
    if any(not isinstance(rule, dict) for rule in raw_rules):
        raise AssertionError("governance rules contain malformed entries")
    rules = list(raw_rules)

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

    pr_rule = has_rule(rules, "pull_request")
    assert pr_rule is not None
    pr_parameters = pr_rule.get("parameters")
    if not isinstance(pr_parameters, dict):
        raise AssertionError("pull request parameters missing")
    approvals = pr_parameters.get("required_approving_review_count")
    if not isinstance(approvals, int) or approvals < 1:
        raise AssertionError("governance root requires at least one approval")
    if pr_parameters.get("dismiss_stale_reviews_on_push") is not True:
        raise AssertionError("stale approvals must be dismissed on push")
    if pr_parameters.get("require_last_push_approval") is not True:
        raise AssertionError("latest push requires independent approval")
    if pr_parameters.get("required_review_thread_resolution") is not True:
        raise AssertionError("review threads must be resolved before merge")
    if pr_parameters.get("allowed_merge_methods") != ["squash"]:
        raise AssertionError("governance root must permit squash merges only")

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
    if integration_id != REQUIRED_CHECK_INTEGRATION_ID:
        raise AssertionError(
            "required Check is not bound to the GitHub Actions App"
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
        "required_ruleset_source_type": REQUIRED_RULESET_SOURCE_TYPE,
        "required_ruleset_source": REQUIRED_RULESET_SOURCE,
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
    if not candidates:
        raise AssertionError(
            "at least one active governance ruleset must target main"
        )

    verified_candidates = []
    verified_details = []
    for candidate in candidates:
        try:
            verified_candidates.append(verify_ruleset(candidate))
            verified_details.append(candidate)
        except AssertionError:
            continue
    if not verified_candidates:
        raise AssertionError(
            "no active main ruleset satisfies the governance root"
        )
    verified = verified_candidates[0]
    result: dict[str, object] = {
        "schema": SCHEMA,
        "repository": REPO,
        "default_branch": DEFAULT_BRANCH,
        "ruleset_count": len(summaries),
        "main_ruleset_count": len(candidates),
        "qualifying_main_ruleset_count": len(verified_candidates),
        "verified": True,
        "rulesets": verified_candidates,
        "ruleset": verified,
        "ruleset_response_digest": digest(verified_details[0]),
        "qualifying_ruleset_response_digests": [
            digest(detail) for detail in verified_details
        ],
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
