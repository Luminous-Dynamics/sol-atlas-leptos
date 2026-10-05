#!/usr/bin/env python3
"""Independent verifier for QualificationExecutionEvidenceV1 packets."""

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
    "pull_request_number",
    "source_revision",
    "base_revision",
    "workflow_name",
    "workflow_ref",
    "workflow_sha",
    "checked_out_workflow_file_sha256",
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

EXPECTED_COMMANDS = [
    "cargo test -p sol-atlas-core --locked -- --list",
    "cargo test -p sol-atlas-core --locked",
]


def fail(message: str) -> "NoReturn":
    raise SystemExit(f"qualification evidence verification failed: {message}")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("packet", type=Path)
    parser.add_argument("sidecar", type=Path)
    parser.add_argument("--repository", required=True)
    parser.add_argument("--pull-request-number", required=True, type=int)
    parser.add_argument("--source-revision", required=True)
    parser.add_argument("--base-revision", required=True)
    parser.add_argument("--workflow-name", required=True)
    parser.add_argument("--workflow-ref", required=True)
    parser.add_argument("--workflow-sha", required=True)
    parser.add_argument("--run-id", required=True, type=int)
    parser.add_argument("--run-attempt", required=True, type=int)
    parser.add_argument("--job-name", required=True)
    parser.add_argument("--check-run-id", required=True, type=int)
    parser.add_argument("--server-url", required=True)
    return parser.parse_args()


def main() -> int:
    args = parse_args()

    if not args.packet.is_file():
        fail(f"packet does not exist: {args.packet}")
    if not args.sidecar.is_file():
        fail(f"sidecar does not exist: {args.sidecar}")

    try:
        evidence = json.loads(args.packet.read_text(encoding="utf-8"))
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as exc:
        fail(f"packet is not valid UTF-8 JSON: {exc}")

    if set(evidence) != REQUIRED_KEYS:
        fail("packet key set does not exactly match QualificationExecutionEvidenceV1")

    if evidence["schema"] != "sol-atlas:qualification-execution-evidence:v1":
        fail("unexpected schema")
    if evidence["qualification_kind"] != "sol_atlas_core_provenance_construction":
        fail("unexpected qualification kind")
    if evidence["status"] != "passed":
        fail("status is not passed")

    expected_scalars = {
        "repository": args.repository,
        "pull_request_number": args.pull_request_number,
        "source_revision": args.source_revision,
        "base_revision": args.base_revision,
        "workflow_name": args.workflow_name,
        "workflow_ref": args.workflow_ref,
        "workflow_sha": args.workflow_sha,
        "run_id": args.run_id,
        "run_attempt": args.run_attempt,
        "job_name": args.job_name,
        "check_run_id": args.check_run_id,
    }
    for key, expected in expected_scalars.items():
        if evidence[key] != expected:
            fail(f"{key} does not match the current GitHub execution context")

    for key in ("source_revision", "base_revision", "workflow_sha"):
        if not SHA1.fullmatch(evidence[key]):
            fail(f"{key} is not a lowercase 40-character SHA-1 hex value")

    for key in (
        "checked_out_workflow_file_sha256",
        "cargo_lock_sha256",
        "test_inventory_sha256",
    ):
        if not SHA256.fullmatch(evidence[key]):
            fail(f"{key} is not a lowercase 64-character SHA-256 hex value")

    for key in ("pull_request_number", "run_id", "run_attempt", "check_run_id"):
        if not isinstance(evidence[key], int) or isinstance(evidence[key], bool) or evidence[key] <= 0:
            fail(f"{key} is not a positive integer")

    expected_url = (
        f"{args.server_url.rstrip('/')}/{args.repository}/actions/runs/{args.run_id}"
    )
    if evidence["workflow_run_url"] != expected_url:
        fail("workflow_run_url does not bind to the current run")

    if evidence["qualification_commands"] != EXPECTED_COMMANDS:
        fail("qualification command inventory does not match the V1 contract")

    packet_digest = hashlib.sha256(args.packet.read_bytes()).hexdigest()
    sidecar = args.sidecar.read_text(encoding="utf-8").strip()
    if sidecar != packet_digest:
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
