#!/usr/bin/env python3
"""Deterministic self-test for the trusted qualification witness."""

from __future__ import annotations

import importlib.util
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
TARGET = ROOT / "tools/verify_qualification_run.py"


def load_module():
    spec = importlib.util.spec_from_file_location("verify_qualification_run", TARGET)
    if spec is None or spec.loader is None:
        raise SystemExit("could not load trusted verifier")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def expect_failure(fn, label: str) -> None:
    try:
        fn()
    except SystemExit:
        return
    raise SystemExit(f"expected trusted verifier rejection: {label}")


def packet(v) -> dict[str, object]:
    return {
        "schema": v.EXPECTED_SCHEMA,
        "qualification_kind": v.EXPECTED_KIND,
        "status": "passed",
        "repository": "Luminous-Dynamics/sol-atlas-leptos",
        "event_name": "pull_request",
        "ref": "refs/pull/13/merge",
        "pull_request_number": 13,
        "source_revision": "0" * 40,
        "base_revision": "1" * 40,
        "workflow_name": v.EXPECTED_WORKFLOW,
        "workflow_ref": "Luminous-Dynamics/sol-atlas-leptos/.github/workflows/check.yml@refs/pull/13/merge",
        "workflow_file_path": v.EXPECTED_WORKFLOW_FILE,
        "workflow_sha": "2" * 40,
        "checked_out_workflow_file_sha256": "3" * 64,
        "verifier_sha256": "4" * 64,
        "run_id": 99,
        "run_attempt": 1,
        "job_name": v.EXPECTED_JOB,
        "check_run_id": 88,
        "workflow_run_url": "https://github.com/Luminous-Dynamics/sol-atlas-leptos/actions/runs/99",
        "runner_os": "Linux",
        "runner_arch": "X64",
        "rust_toolchain": v.EXPECTED_TOOLCHAIN,
        "rustc_version_verbose": "rustc 1.99.0 (fixture)",
        "cargo_version_verbose": "cargo 1.99.0 (fixture)",
        "cargo_lock_sha256": "5" * 64,
        "test_inventory_sha256": "6" * 64,
        "qualification_commands": v.EXPECTED_COMMANDS,
    }


def run_record(v, p) -> dict[str, object]:
    return {
        "id": p["run_id"],
        "name": v.EXPECTED_WORKFLOW,
        "event": "pull_request",
        "status": "completed",
        "conclusion": "success",
        "head_sha": p["source_revision"],
        "path": v.EXPECTED_WORKFLOW_FILE,
        "pull_requests": [
            {
                "number": p["pull_request_number"],
                "base": {"sha": p["base_revision"]},
                "head": {
                    "sha": p["source_revision"],
                    "repo": {"full_name": "example/fork"},
                },
            }
        ],
    }


def main() -> int:
    v = load_module()
    p = packet(v)
    r = run_record(v, p)

    v.verify_packet(
        p,
        r,
        p["repository"],
        "https://github.com",
        p["run_id"],
    )
    assert v.verify_pr_metadata(r, p) == "example/fork"

    steps = [
        {"name": name, "status": "completed", "conclusion": "success"}
        for name in v.REQUIRED_STEPS
    ]
    v.verify_jobs(
        {
            "jobs": [
                {
                    "name": v.EXPECTED_JOB,
                    "workflow_name": v.EXPECTED_WORKFLOW,
                    "head_sha": p["source_revision"],
                    "status": "completed",
                    "conclusion": "success",
                    "steps": steps,
                }
            ]
        },
        p,
    )

    expect_failure(
        lambda: v.verify_packet(
            {**p, "status": "failure"},
            r,
            p["repository"],
            "https://github.com",
            p["run_id"],
        ),
        "failed workflow status",
    )
    expect_failure(
        lambda: v.verify_packet(
            {**p, "ref": "refs/pull/13/head"},
            r,
            p["repository"],
            "https://github.com",
            p["run_id"],
        ),
        "non-canonical pull request ref",
    )

    original_fetch_file = v.fetch_file
    calls: list[tuple[str, str, str]] = []
    source_workflow = b"trusted-source-workflow"
    source_verifier = b"pr-verifier"
    source_lock = b"cargo-lock"
    executed_workflow = source_workflow

    def fake_fetch(api_url, repository, path, ref, token):
        calls.append((repository, path, ref))
        if path == v.EXPECTED_WORKFLOW_FILE and ref == p["source_revision"]:
            return source_workflow
        if path == v.EXPECTED_WORKFLOW_FILE and ref == p["workflow_sha"]:
            return executed_workflow
        if path == "tools/verify_qualification_execution_evidence.py":
            return source_verifier
        if path == "Cargo.lock":
            return source_lock
        raise AssertionError((repository, path, ref))

    policy_source = """permissions:
  contents: read
provenance-construction:
    if: github.event_name == 'pull_request'
ref: ${{ github.event.pull_request.head.sha }}
test "$(git rev-parse HEAD)" = "$EXPECTED_HEAD"
cargo test -p sol-atlas-core --locked -- --list
cargo test -p sol-atlas-core --locked
python3 tools/test_verify_qualification_execution_evidence.py
python3 tools/verify_qualification_execution_evidence.py
name: sol-atlas-qualification-execution-v2
"""

    original_snippets = v.REQUIRED_WORKFLOW_SNIPPETS
    original_privileges = v.FORBIDDEN_PRIVILEGES
    v.REQUIRED_WORKFLOW_SNIPPETS = [policy_source]
    v.FORBIDDEN_PRIVILEGES = []
    v.fetch_file = fake_fetch
    try:
        v.verify_source_bindings(
            "https://api.github.com",
            "example/fork",
            "Luminous-Dynamics/sol-atlas-leptos",
            p["source_revision"],
            p,
            "",
        )
        assert calls[0] == ("example/fork", v.EXPECTED_WORKFLOW_FILE, p["source_revision"])
        assert calls[1] == ("Luminous-Dynamics/sol-atlas-leptos", v.EXPECTED_WORKFLOW_FILE, p["workflow_sha"])

        executed_workflow = b"tampered-executed-workflow"
        expect_failure(
            lambda: v.verify_source_bindings(
                "https://api.github.com",
                "example/fork",
                "Luminous-Dynamics/sol-atlas-leptos",
                p["source_revision"],
                p,
                "",
            ),
            "workflow definition mismatch",
        )
    finally:
        v.fetch_file = original_fetch_file
        v.REQUIRED_WORKFLOW_SNIPPETS = original_snippets
        v.FORBIDDEN_PRIVILEGES = original_privileges

    print("trusted qualification witness self-test passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
