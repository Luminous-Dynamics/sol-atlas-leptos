#!/usr/bin/env python3
# Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
# SPDX-License-Identifier: AGPL-3.0-or-later

"""Verify protection on the credentialed GCS qualification environment."""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
from pathlib import Path

SCHEMA = "sol-atlas:github-qualification-environment-audit:v1"
REPO = "Luminous-Dynamics/sol-atlas-leptos"
ENVIRONMENT = "sol-atlas-gcs-qualification"
DEFAULT_BRANCH = "main"
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


def get_environment() -> dict[str, object]:
    payload = gh_api(
        "/repos/" + REPO + "/environments/" + ENVIRONMENT
    )
    if not isinstance(payload, dict):
        raise AssertionError("environment response is not an object")
    if payload.get("name") != ENVIRONMENT:
        raise AssertionError("qualification environment name mismatch")
    return payload


def get_branch_policies() -> dict[str, object]:
    payload = gh_api(
        "/repos/"
        + REPO
        + "/environments/"
        + ENVIRONMENT
        + "/deployment-branch-policies?per_page=100"
    )
    if not isinstance(payload, dict):
        raise AssertionError("branch-policy response is not an object")
    policies = payload.get("branch_policies")
    total = payload.get("total_count")
    if not isinstance(policies, list) or not isinstance(total, int):
        raise AssertionError("branch-policy response is malformed")
    if total != len(policies):
        raise AssertionError("branch-policy response may be truncated")
    if any(not isinstance(policy, dict) for policy in policies):
        raise AssertionError("branch-policy response has malformed entries")
    return payload


def verify_environment(environment: dict[str, object]) -> dict[str, object]:
    protection_rules = environment.get("protection_rules")
    if not isinstance(protection_rules, list):
        raise AssertionError("environment protection rules are not observable")
    reviewer_rules = [
        rule
        for rule in protection_rules
        if isinstance(rule, dict)
        and rule.get("type") == "required_reviewers"
    ]
    if len(reviewer_rules) != 1:
        raise AssertionError(
            "exactly one required-reviewers protection rule is required"
        )
    reviewer_rule = reviewer_rules[0]
    if reviewer_rule.get("prevent_self_review") is not True:
        raise AssertionError("environment self-review must be prevented")
    reviewers = reviewer_rule.get("reviewers")
    if not isinstance(reviewers, list) or not reviewers:
        raise AssertionError("environment requires at least one reviewer")
    for reviewer in reviewers:
        if not isinstance(reviewer, dict):
            raise AssertionError("malformed environment reviewer")
        reviewer_type = reviewer.get("type")
        identity = reviewer.get("reviewer")
        if reviewer_type not in ("User", "Team"):
            raise AssertionError("invalid environment reviewer type")
        if not isinstance(identity, dict):
            raise AssertionError("environment reviewer identity is missing")
        if not isinstance(identity.get("id"), int) or identity["id"] <= 0:
            raise AssertionError("environment reviewer ID is invalid")

    deployment_policy = environment.get("deployment_branch_policy")
    if not isinstance(deployment_policy, dict):
        raise AssertionError("environment deployment branch policy is missing")
    if deployment_policy.get("protected_branches") is not False:
        raise AssertionError(
            "environment must use an explicit custom branch policy"
        )
    if deployment_policy.get("custom_branch_policies") is not True:
        raise AssertionError("environment custom branch policy is disabled")

    return {
        "name": ENVIRONMENT,
        "reviewer_rule_count": len(reviewer_rules),
        "reviewer_count": len(reviewers),
        "prevent_self_review": True,
        "protected_branches": False,
        "custom_branch_policies": True,
    }


def verify_branch_policies(payload: dict[str, object]) -> dict[str, object]:
    policies = payload.get("branch_policies")
    if not isinstance(policies, list):
        raise AssertionError("branch-policy list is missing")
    names = []
    for policy in policies:
        name = policy.get("name")
        if not isinstance(name, str) or not name:
            raise AssertionError("branch policy has invalid name")
        names.append(name)
    if names != [DEFAULT_BRANCH]:
        raise AssertionError(
            "qualification environment must permit exactly main"
        )
    return {
        "policy_count": len(policies),
        "allowed_branch_patterns": names,
    }


def verify(output: str | None) -> dict[str, object]:
    environment = get_environment()
    branch_policies = get_branch_policies()
    environment_result = verify_environment(environment)
    branch_result = verify_branch_policies(branch_policies)
    result: dict[str, object] = {
        "schema": SCHEMA,
        "repository": REPO,
        "environment": ENVIRONMENT,
        "verified": True,
        "environment_evidence": environment_result,
        "branch_policy_evidence": branch_result,
        "environment_response_digest": digest(environment),
        "branch_policy_response_digest": digest(branch_policies),
        "claim_ceiling": (
            "Observed GitHub protection configuration requires an independent "
            "reviewer, prevents self-review, and permits only the main branch "
            "to deploy to this environment. This does not prove reviewer intent, "
            "organization governance outside the environment, Cloud IAM state, "
            "or external-effect correctness."
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
        "verified qualification environment: "
        + result["environment"]
        + " "
        + str(result["branch_policy_evidence"]["allowed_branch_patterns"]),
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
