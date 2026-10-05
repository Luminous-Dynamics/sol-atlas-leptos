#!/usr/bin/env python3
"""Independent verifier for QualificationExecutionEvidenceV2 packets."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
from pathlib import Path


SHA1 = re.compile(r"^[0-9a-f]{40}$")
SHA256 = re.compile(r"^[0-9a-f]{64}$")

REQUIRED_KEYS = {
    "schema",
    "qualification_kind",
    "status",
    "repository",
    "event_name",
    "ref",
    "pull_request_number",
    "source_revision",
    "base_revision",
    "workflow_name",
    "workflow_ref",
    "workflow_file_path",
    "workflow_sha",
    "checked_out_workflow_file_sha256",
    "verifier_sha256",
    "run_id",
    "run_attempt",
    "job_name",
    "check_run_id",
    "workflow_run_url",
    "runner_os",
    "runner_arch",
    "rust_toolchain",
    "rustc_version_verbose",
    "cargo_version_verbose",
    "cargo_lock_sha256",
    "test_inventory_sha256",
    "qualification_commands",
}

EXPECTED_RUST_TOOLCHAIN = "1.99.0"
EXPECTED_RUSTC_PREFIX = "rustc 1.99.0 "
EXPECTED_WORKFLOW_NAME = "Check"
EXPECTED_WORKFLOW_FILE_PATH = ".github/workflows/check.yml"
EXPECTED_JOB_NAME = "provenance-construction"

EXPECTED_COMMANDS = [
    "cargo test -p sol-atlas-core --locked -- --list",
    "cargo test -p sol-atlas-core --locked",
]


def fail(message: str) -> "NoReturn":
    raise SystemExit(f"qualification evidence verification failed: {message}")


def reject_duplicate_object_keys(pairs: list[tuple[str, object]]) -> dict[str, object]:
    result: dict[str, object] = {}
    for key, value in pairs:
        if key in result:
            fail(f"packet contains duplicate JSON object key: {key}")
        result[key] = value
    return result


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("packet", type=Path)
    parser.add_argument("sidecar", type=Path)
    parser.add_argument("--workflow-file", type=Path, required=True)
    parser.add_argument("--verifier-file", type=Path, required=True)
    parser.add_argument("--cargo-lock", type=Path, required=True)
    parser.add_argument("--test-inventory", type=Path, required=True)
    parser.add_argument("--repository", required=True)
    parser.add_argument("--event-name", required=True)
    parser.add_argument("--ref", required=True)
    parser.add_argument("--pull-request-number", required=True, type=int)
    parser.add_argument("--source-revision", required=True)
    parser.add_argument("--base-revision", required=True)
    parser.add_argument("--workflow-name", required=True)
    parser.add_argument("--workflow-ref", required=True)
    parser.add_argument("--workflow-file-path", required=True)
    parser.add_argument("--workflow-sha", required=True)
    parser.add_argument("--run-id", required=True, type=int)
    parser.add_argument("--run-attempt", required=True, type=int)
    parser.add_argument("--job-name", required=True)
    parser.add_argument("--check-run-id", required=True, type=int)
    parser.add_argument("--server-url", required=True)
    parser.add_argument("--runner-os", required=True)
    parser.add_argument("--runner-arch", required=True)
    parser.add_argument("--rust-toolchain", required=True)
    parser.add_argument("--rustc-version-verbose", required=True)
    parser.add_argument("--cargo-version-verbose", required=True)
    return parser.parse_args()


def main() -> int:
    args = parse_args()

    if not args.packet.is_file():
        fail(f"packet does not exist: {args.packet}")
    for label, path in (
        ("sidecar", args.sidecar),
        ("workflow file", args.workflow_file),
        ("verifier file", args.verifier_file),
        ("Cargo.lock", args.cargo_lock),
        ("test inventory", args.test_inventory),
    ):
        if not path.is_file():
            fail(f"{label} does not exist: {path}")

    try:
        evidence = json.loads(
            args.packet.read_text(encoding="utf-8"),
            object_pairs_hook=reject_duplicate_object_keys,
        )
        if not isinstance(evidence, dict):
            fail("packet JSON root is not an object")
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as exc:
        fail(f"packet is not valid UTF-8 JSON: {exc}")

    if set(evidence) != REQUIRED_KEYS:
        fail("packet key set does not exactly match QualificationExecutionEvidenceV2")

    if evidence["schema"] != "sol-atlas:qualification-execution-evidence:v2":
        fail("unexpected schema")
    if evidence["qualification_kind"] != "sol_atlas_core_provenance_construction":
        fail("unexpected qualification kind")
    if evidence["status"] != "passed":
        fail("status is not passed")
    if args.rust_toolchain != EXPECTED_RUST_TOOLCHAIN:
        fail(f"qualification requires Rust toolchain {EXPECTED_RUST_TOOLCHAIN}")
    if args.event_name != "pull_request":
        fail("qualification requires a pull_request workflow event")
    expected_ref = f"refs/pull/{args.pull_request_number}/merge"
    if args.ref != expected_ref:
        fail("qualification ref does not match the current pull_request merge ref")
    if args.workflow_name != EXPECTED_WORKFLOW_NAME:
        fail(f"qualification requires workflow {EXPECTED_WORKFLOW_NAME!r}")
    if args.workflow_file_path != EXPECTED_WORKFLOW_FILE_PATH:
        fail(f"qualification requires workflow file {EXPECTED_WORKFLOW_FILE_PATH!r}")
    expected_workflow_ref = (
        f"{args.repository}/{EXPECTED_WORKFLOW_FILE_PATH}@refs/pull/"
        f"{args.pull_request_number}/merge"
    )
    if args.workflow_ref != expected_workflow_ref:
        fail("qualification workflow_ref does not match the canonical pull_request merge ref")
    if args.job_name != EXPECTED_JOB_NAME:
        fail(f"qualification requires job {EXPECTED_JOB_NAME!r}")

    expected_scalars = {
        "repository": args.repository,
        "event_name": args.event_name,
        "ref": args.ref,
        "pull_request_number": args.pull_request_number,
        "source_revision": args.source_revision,
        "base_revision": args.base_revision,
        "workflow_name": args.workflow_name,
        "workflow_ref": args.workflow_ref,
        "workflow_file_path": args.workflow_file_path,
        "workflow_sha": args.workflow_sha,
        "run_id": args.run_id,
        "run_attempt": args.run_attempt,
        "job_name": args.job_name,
        "check_run_id": args.check_run_id,
        "runner_os": args.runner_os,
        "runner_arch": args.runner_arch,
        "rust_toolchain": args.rust_toolchain,
        "rustc_version_verbose": args.rustc_version_verbose,
        "cargo_version_verbose": args.cargo_version_verbose,
    }
    for key, expected in expected_scalars.items():
        if evidence[key] != expected:
            fail(f"{key} does not match the current GitHub execution context")

    for key in ("source_revision", "base_revision", "workflow_sha"):
        if not SHA1.fullmatch(evidence[key]):
            fail(f"{key} is not a lowercase 40-character SHA-1 hex value")

    for key in (
        "checked_out_workflow_file_sha256",
        "verifier_sha256",
        "cargo_lock_sha256",
        "test_inventory_sha256",
    ):
        if not SHA256.fullmatch(evidence[key]):
            fail(f"{key} is not a lowercase 64-character SHA-256 hex value")

    file_bindings = {
        "checked_out_workflow_file_sha256": args.workflow_file,
        "verifier_sha256": args.verifier_file,
        "cargo_lock_sha256": args.cargo_lock,
        "test_inventory_sha256": args.test_inventory,
    }
    for key, path in file_bindings.items():
        actual = hashlib.sha256(path.read_bytes()).hexdigest()
        if evidence[key] != actual:
            fail(f"{key} does not match {path}")

    for key in ("pull_request_number", "run_id", "run_attempt", "check_run_id"):
        if not isinstance(evidence[key], int) or isinstance(evidence[key], bool) or evidence[key] <= 0:
            fail(f"{key} is not a positive integer")

    expected_url = (
        f"{args.server_url.rstrip('/')}/{args.repository}/actions/runs/{args.run_id}"
    )
    if evidence["workflow_run_url"] != expected_url:
        fail("workflow_run_url does not bind to the current run")

    for key in ("runner_os", "runner_arch", "rust_toolchain", "rustc_version_verbose", "cargo_version_verbose"):
        if not isinstance(evidence[key], str) or not evidence[key]:
            fail(f"{key} is not a non-empty string")
    if not evidence["rustc_version_verbose"].startswith(EXPECTED_RUSTC_PREFIX):
        fail("rustc_version_verbose does not identify the required Rust toolchain")

    if evidence["qualification_commands"] != EXPECTED_COMMANDS:
        fail("qualification command inventory does not match the V2 contract")

    packet_digest = hashlib.sha256(args.packet.read_bytes()).hexdigest()
    sidecar = args.sidecar.read_text(encoding="utf-8")
    if sidecar != packet_digest + "\n":
        fail("SHA-256 sidecar does not match packet bytes")

    print(
        "qualification evidence verified:",
        evidence["repository"],
        f"PR #{evidence['pull_request_number']}",
        evidence["source_revision"],
        f"run {evidence['run_id']}",
        f"check {evidence['check_run_id']}",
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
