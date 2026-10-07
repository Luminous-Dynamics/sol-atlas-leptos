#!/usr/bin/env python3
# Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
# SPDX-License-Identifier: AGPL-3.0-or-later

"""Cross-check GitHub runner context against GitHub's server-side run record."""

from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
from pathlib import Path

REPO = "Luminous-Dynamics/sol-atlas-leptos"
REPOSITORY_ID = "1195997641"
WORKFLOW_NAME = "Qualify GCS external effect"
WORKFLOW_PATH = ".github/workflows/qualify-gcs.yml"
WORKFLOW_ID = 311325850
REPOSITORY_OWNER_ID = "216969177"
EVENT = "workflow_dispatch"
REF = "refs/heads/main"
RUNNER_HOST = "github.com"
HEX40 = re.compile(r"^[0-9a-f]{40}$")
DECIMAL = re.compile(r"^[0-9]+$")


def runtime_context() -> dict[str, str]:
    names = (
        "GITHUB_REPOSITORY",
        "GITHUB_REPOSITORY_ID",
        "GITHUB_RUN_ID",
        "GITHUB_RUN_ATTEMPT",
        "GITHUB_SHA",
        "GITHUB_WORKFLOW",
        "GITHUB_WORKFLOW_REF",
        "GITHUB_WORKFLOW_SHA",
        "GITHUB_EVENT_NAME",
        "GITHUB_REF",
        "GITHUB_SERVER_URL",
    )
    values = {name: os.environ.get(name, "") for name in names}
    missing = [name for name, value in values.items() if not value]
    if missing:
        raise AssertionError(
            "missing GitHub runtime context: " + ", ".join(missing)
        )
    if values["GITHUB_REPOSITORY"] != REPO:
        raise AssertionError("unexpected GitHub repository")
    if values["GITHUB_REPOSITORY_ID"] != REPOSITORY_ID:
        raise AssertionError("unexpected GitHub repository ID")
    if not DECIMAL.fullmatch(values["GITHUB_RUN_ID"]):
        raise AssertionError("GITHUB_RUN_ID is not decimal")
    if not DECIMAL.fullmatch(values["GITHUB_RUN_ATTEMPT"]):
        raise AssertionError("GITHUB_RUN_ATTEMPT is not decimal")
    for name in ("GITHUB_SHA", "GITHUB_WORKFLOW_SHA"):
        if not HEX40.fullmatch(values[name]):
            raise AssertionError(f"{name} is not a lowercase 40-hex commit")
    if values["GITHUB_WORKFLOW"] != WORKFLOW_NAME:
        raise AssertionError("unexpected GitHub workflow name")
    if values["GITHUB_WORKFLOW_REF"] != (
        f"{REPO}/{WORKFLOW_PATH}@refs/heads/main"
    ):
        raise AssertionError("unexpected GitHub workflow ref")
    if values["GITHUB_EVENT_NAME"] != EVENT:
        raise AssertionError("unexpected GitHub event")
    if values["GITHUB_REF"] != REF:
        raise AssertionError("unexpected GitHub ref")
    if values["GITHUB_SERVER_URL"] != "https://" + RUNNER_HOST:
        raise AssertionError("unexpected GitHub server")
    return values


def gh_api(path: str) -> object:
    result = subprocess.run(
        [
            "gh",
            "--hostname",
            RUNNER_HOST,
            "api",
            path,
            "--header",
            "Accept: application/vnd.github+json",
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


def verify_run_record(
    context: dict[str, str],
    run: dict[str, object],
    workflow: dict[str, object],
) -> dict[str, object]:
    workflow_id = run.get("workflow_id")
    if workflow_id != WORKFLOW_ID:
        raise AssertionError("workflow-run workflow ID mismatch")
    repository = run.get("repository")
    if not isinstance(repository, dict):
        raise AssertionError("workflow-run record has no repository object")
    if repository.get("full_name") != REPO:
        raise AssertionError("workflow-run repository mismatch")
    if str(repository.get("id")) != REPOSITORY_ID:
        raise AssertionError("workflow-run repository ID mismatch")
    owner = repository.get("owner")
    if not isinstance(owner, dict):
        raise AssertionError("workflow-run repository owner is missing")
    if str(owner.get("id")) != REPOSITORY_OWNER_ID:
        raise AssertionError("workflow-run repository owner ID mismatch")
    expected_run = {
        "id": int(context["GITHUB_RUN_ID"]),
        "run_attempt": int(context["GITHUB_RUN_ATTEMPT"]),
        "head_sha": context["GITHUB_SHA"],
        "head_branch": "main",
        "event": EVENT,
        "name": WORKFLOW_NAME,
        "path": WORKFLOW_PATH + "@" + REF,
    }
    for name, expected in expected_run.items():
        if run.get(name) != expected:
            raise AssertionError(
                f"workflow-run server mismatch for {name}: "
                f"{run.get(name)!r} != {expected!r}"
            )
    referenced = run.get("referenced_workflows")
    if referenced not in ([], None):
        raise AssertionError(
            "unexpected reusable workflows in direct qualification run"
        )
    if workflow.get("name") != WORKFLOW_NAME:
        raise AssertionError("workflow server name mismatch")
    if workflow.get("path") != WORKFLOW_PATH:
        raise AssertionError("workflow server path mismatch")
    return {
        "schema": "sol-atlas:github-workflow-run-verification:v1",
        "repository": REPO,
        "repository_id": REPOSITORY_ID,
        "repository_owner_id": REPOSITORY_OWNER_ID,
        "run_id": context["GITHUB_RUN_ID"],
        "run_attempt": context["GITHUB_RUN_ATTEMPT"],
        "workflow_id": workflow_id,
        "workflow_name": workflow.get("name"),
        "workflow_path": workflow.get("path"),
        "head_sha": run.get("head_sha"),
        "head_branch": run.get("head_branch"),
        "event": run.get("event"),
        "run_status": run.get("status"),
        "run_conclusion": run.get("conclusion"),
        "run_path": run.get("path"),
        "referenced_workflows": referenced or [],
        "github_server": RUNNER_HOST,
        "context_sha_matches_server": (
            run.get("head_sha") == context["GITHUB_SHA"]
        ),
        "context_run_attempt_matches_server": (
            run.get("run_attempt") == int(context["GITHUB_RUN_ATTEMPT"])
        ),
    }


def verify(output: str | None) -> dict[str, object]:
    context = runtime_context()
    run = gh_api(
        f"/repos/{REPO}/actions/runs/{context['GITHUB_RUN_ID']}"
    )
    if not isinstance(run, dict):
        raise AssertionError("workflow-run API response is not an object")
    workflow_id = run.get("workflow_id")
    if not isinstance(workflow_id, int) or workflow_id <= 0:
        raise AssertionError("workflow-run record has no valid workflow ID")
    workflow = gh_api(
        f"/repos/{REPO}/actions/workflows/{workflow_id}"
    )
    if not isinstance(workflow, dict):
        raise AssertionError("workflow API response is not an object")
    result = verify_run_record(context, run, workflow)
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
        "verified GitHub server workflow-run provenance: "
        + str(result["workflow_id"])
        + " "
        + str(result["run_id"])
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
