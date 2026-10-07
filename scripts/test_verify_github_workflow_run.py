#!/usr/bin/env python3
# Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
# SPDX-License-Identifier: AGPL-3.0-or-later

"""Offline semantic tests for GitHub server workflow-run provenance."""

from __future__ import annotations

import importlib.util

SCRIPT = __import__("pathlib").Path(__file__).resolve().parents[1] / (
    "scripts/verify_github_workflow_run.py"
)
spec = importlib.util.spec_from_file_location("verify_github_workflow_run", SCRIPT)
assert spec and spec.loader
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)

CONTEXT = {
    "GITHUB_REPOSITORY": "Luminous-Dynamics/sol-atlas-leptos",
    "GITHUB_REPOSITORY_ID": "1195997641",
    "GITHUB_RUN_ID": "12345",
    "GITHUB_RUN_ATTEMPT": "2",
    "GITHUB_SHA": "a" * 40,
    "GITHUB_WORKFLOW": "Qualify GCS external effect",
    "GITHUB_WORKFLOW_REF": (
        "Luminous-Dynamics/sol-atlas-leptos/.github/workflows/"
        "qualify-gcs.yml@refs/heads/main"
    ),
    "GITHUB_WORKFLOW_SHA": "b" * 40,
    "GITHUB_EVENT_NAME": "workflow_dispatch",
    "GITHUB_REF": "refs/heads/main",
    "GITHUB_SERVER_URL": "https://github.com",
}

RUN = {
    "id": 12345,
    "run_attempt": 2,
    "head_sha": "a" * 40,
    "head_branch": "main",
    "event": "workflow_dispatch",
    "name": "Qualify GCS external effect",
    "workflow_id": 9876,
    "status": "in_progress",
    "conclusion": None,
    "path": ".github/workflows/qualify-gcs.yml@refs/heads/main",
    "referenced_workflows": [],
    "repository": {
        "full_name": "Luminous-Dynamics/sol-atlas-leptos",
        "id": 1195997641,
        "owner": {"id": 216969177},
    },
}

WORKFLOW = {
    "name": "Qualify GCS external effect",
    "path": ".github/workflows/qualify-gcs.yml",
}


def reject(run: dict[str, object], workflow: dict[str, object], name: str) -> None:
    try:
        module.verify_run_record(CONTEXT, run, workflow)
    except AssertionError:
        return
    raise AssertionError(f"tampered workflow-run record was accepted: {name}")


def main() -> None:
    result = module.verify_run_record(CONTEXT, RUN, WORKFLOW)
    assert result["context_sha_matches_server"] is True
    assert result["context_run_attempt_matches_server"] is True
    assert result["referenced_workflows"] == []

    reject(dict(RUN, workflow_id=1234), WORKFLOW, "workflow_id")
    reject(dict(RUN, path=".github/workflows/other.yml@refs/heads/main"), WORKFLOW, "run_path")
    reject(dict(RUN, head_sha="c" * 40), WORKFLOW, "head_sha")
    reject(dict(RUN, run_attempt=1), WORKFLOW, "run_attempt")
    reject(dict(RUN, event="push"), WORKFLOW, "event")
    reject(dict(RUN, head_branch="feature"), WORKFLOW, "head_branch")
    reject(
        dict(RUN, name="other workflow"),
        WORKFLOW,
        "workflow name",
    )
    reject(
        dict(
            RUN,
            repository={
                "full_name": "other/repository",
                "id": 1195997641,
            },
        ),
        WORKFLOW,
        "repository",
    )
    reject(
        dict(RUN, referenced_workflows=[{"path": "other/reusable.yml"}]),
        WORKFLOW,
        "unexpected reusable workflow",
    )
    reject(
        RUN,
        {"name": "other workflow", "path": WORKFLOW["path"]},
        "workflow API name",
    )
    reject(
        RUN,
        {"name": WORKFLOW["name"], "path": ".github/workflows/other.yml"},
        "workflow API path",
    )

    print("offline GitHub server workflow-run provenance checks: PASS")


if __name__ == "__main__":
    main()
