#!/usr/bin/env python3
"""Trusted verifier for a completed Sol Atlas qualification Check run.

Runs only from a trusted default-branch workflow. PR-produced artifacts and
source files are treated as untrusted data; this verifier never executes PR
code or imports the PR verifier.
"""

from __future__ import annotations

import argparse
import base64
import hashlib
import json
import os
import re
import sys
from pathlib import Path
from typing import NoReturn
from urllib.error import HTTPError
from urllib.request import Request, urlopen


SHA1 = re.compile(r"^[0-9a-f]{40}$")
SHA256 = re.compile(r"^[0-9a-f]{64}$")

EXPECTED_SCHEMA = "sol-atlas:qualification-execution-evidence:v2"
EXPECTED_KIND = "sol_atlas_core_provenance_construction"
EXPECTED_WORKFLOW = "Check"
EXPECTED_WORKFLOW_FILE = ".github/workflows/check.yml"
EXPECTED_JOB = "provenance-construction"
EXPECTED_TOOLCHAIN = "1.99.0"
EXPECTED_CHECK_WORKFLOW_BLOB_SHA1 = "d7755372feaa9545e23b3e2dac73525caaedc25d"
EXPECTED_PR_VERIFIER_BLOB_SHA1 = "94553d235db403fbcc0a3a0a9377ec1108a91510"

EXPECTED_COMMANDS = [
    "cargo test -p sol-atlas-core --locked -- --list",
    "cargo test -p sol-atlas-core --locked",
]

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

REQUIRED_STEPS = [
    "Assert exact pull-request head",
    "Install Rust",
    "Capture dependency identity",
    "Inspect compiled core test inventory",
    "Execute core package",
    "Adversarially self-test qualification verifier",
    "Assert dependency and checkout integrity",
    "Emit qualification execution evidence",
    "Validate qualification execution evidence independently",
    "Publish qualification execution evidence",
]

REQUIRED_WORKFLOW_SNIPPETS = [
    "permissions:\n  contents: read",
    "provenance-construction:\n    if: github.event_name == 'pull_request'",
    "ref: ${{ github.event.pull_request.head.sha }}",
    'test "$(git rev-parse HEAD)" = "$EXPECTED_HEAD"',
    "cargo test -p sol-atlas-core --locked -- --list",
    "cargo test -p sol-atlas-core --locked",
    "python3 tools/test_verify_qualification_execution_evidence.py",
    "python3 tools/verify_qualification_execution_evidence.py",
    "name: sol-atlas-qualification-execution-v2",
]

FORBIDDEN_PRIVILEGES = [
    "id-token: write",
    "attestations: write",
    "artifact-metadata: write",
]


def fail(message: str) -> NoReturn:
    raise SystemExit(f"trusted qualification verification failed: {message}")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--artifact-dir", type=Path, required=True)
    parser.add_argument("--run-id", type=int, required=True)
    parser.add_argument("--repository", required=True)
    parser.add_argument("--server-url", required=True)
    parser.add_argument("--api-url", required=True)
    parser.add_argument("--token", default=os.environ.get("GITHUB_TOKEN", ""))
    return parser.parse_args()


def api_get(url: str, token: str) -> object:
    headers = {
        "Accept": "application/vnd.github+json",
        "X-GitHub-Api-Version": "2026-03-10",
    }
    if token:
        headers["Authorization"] = f"Bearer {token}"
    try:
        request = Request(url, headers=headers, method="GET")
        with urlopen(request, timeout=30) as response:
            return json.load(response)
    except HTTPError as exc:
        fail(f"GitHub API request failed ({exc.code}): {url}")
    except (OSError, json.JSONDecodeError) as exc:
        fail(f"GitHub API request could not be decoded: {url}: {exc}")


def digest(value: bytes) -> str:
    return hashlib.sha256(value).hexdigest()



def reject_duplicate_keys(
    pairs: list[tuple[str, object]],
) -> dict[str, object]:
    result: dict[str, object] = {}
    for key, value in pairs:
        if key in result:
            fail(f"packet contains duplicate JSON object key: {key}")
        result[key] = value
    return result


def load_packet(path: Path) -> tuple[dict[str, object], bytes]:
    if not path.is_file():
        fail(f"packet does not exist: {path}")
    try:
        packet_bytes = path.read_bytes()
        packet = json.loads(
            packet_bytes.decode("utf-8"),
            object_pairs_hook=reject_duplicate_keys,
        )
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as exc:
        fail(f"packet is not valid UTF-8 JSON: {exc}")
    if not isinstance(packet, dict):
        fail("packet JSON root is not an object")
    if set(packet) != REQUIRED_KEYS:
        fail("packet key set does not exactly match V2")
    return packet, packet_bytes


def verify_sidecar(path: Path, packet_bytes: bytes) -> None:
    if not path.is_file():
        fail(f"sidecar does not exist: {path}")
    try:
        sidecar = path.read_text(encoding="utf-8")
    except (OSError, UnicodeDecodeError) as exc:
        fail(f"sidecar is not valid UTF-8: {exc}")
    if sidecar != digest(packet_bytes) + "\n":
        fail("packet SHA-256 sidecar is not exact")


def fetch_file(
    api_url: str,
    repository: str,
    path: str,
    ref: str,
    token: str,
) -> bytes:
    url = (
        f"{api_url.rstrip('/')}/repos/{repository}/contents/"
        f"{path}?ref={ref}"
    )
    payload = api_get(url, token)
    if not isinstance(payload, dict):
        fail(f"contents response is not an object: {path}")
    if payload.get("type") != "file":
        fail(f"contents target is not a file: {path}")
    if payload.get("encoding") != "base64":
        fail(f"contents response is not base64: {path}")
    content = payload.get("content")
    if not isinstance(content, str):
        fail(f"contents response has no file content: {path}")
    try:
        return base64.b64decode(content, validate=True)
    except (ValueError, base64.binascii.Error) as exc:
        fail(f"contents response has invalid base64: {path}: {exc}")


def verify_packet(
    packet: dict[str, object],
    run: dict[str, object],
    repository: str,
    server_url: str,
    run_id: int,
) -> None:
    if packet["schema"] != EXPECTED_SCHEMA:
        fail("unexpected schema")
    if packet["qualification_kind"] != EXPECTED_KIND:
        fail("unexpected qualification kind")
    if packet["status"] != "passed":
        fail("packet status is not passed")
    if packet["repository"] != repository:
        fail("packet repository mismatch")
    if packet["event_name"] != "pull_request":
        fail("packet event_name is not pull_request")

    pr_number = packet["pull_request_number"]
    if (
        not isinstance(pr_number, int)
        or isinstance(pr_number, bool)
        or pr_number <= 0
    ):
        fail("pull_request_number is not a positive integer")

    expected_ref = f"refs/pull/{pr_number}/merge"
    if packet["ref"] != expected_ref:
        fail("packet ref is not the canonical pull_request merge ref")
    if packet["workflow_name"] != EXPECTED_WORKFLOW:
        fail("packet workflow_name mismatch")
    if packet["workflow_file_path"] != EXPECTED_WORKFLOW_FILE:
        fail("packet workflow_file_path mismatch")
    if packet["job_name"] != EXPECTED_JOB:
        fail("packet job_name mismatch")
    if packet["rust_toolchain"] != EXPECTED_TOOLCHAIN:
        fail("packet Rust toolchain is not the pinned 1.99.0")

    for key in ("source_revision", "base_revision", "workflow_sha"):
        value = packet[key]
        if not isinstance(value, str) or not SHA1.fullmatch(value):
            fail(f"{key} is not a lowercase SHA-1")
    for key in (
        "checked_out_workflow_file_sha256",
        "verifier_sha256",
        "cargo_lock_sha256",
        "test_inventory_sha256",
    ):
        value = packet[key]
        if not isinstance(value, str) or not SHA256.fullmatch(value):
            fail(f"{key} is not a lowercase SHA-256")

    for key in ("run_id", "run_attempt", "check_run_id"):
        value = packet[key]
        if not isinstance(value, int) or isinstance(value, bool) or value <= 0:
            fail(f"{key} is not a positive integer")
    if packet["run_id"] != run_id:
        fail("packet run_id does not match witnessed run")

    expected_url = f"{server_url.rstrip('/')}/{repository}/actions/runs/{run_id}"
    if packet["workflow_run_url"] != expected_url:
        fail("packet workflow_run_url mismatch")
    if packet["qualification_commands"] != EXPECTED_COMMANDS:
        fail("qualification command inventory mismatch")

    expected_run_fields = {
        "id": run_id,
        "name": EXPECTED_WORKFLOW,
        "event": "pull_request",
        "status": "completed",
        "conclusion": "success",
        "head_sha": packet["source_revision"],
        "path": EXPECTED_WORKFLOW_FILE,
    }
    for key, expected in expected_run_fields.items():
        if run.get(key) != expected:
            fail(f"workflow run {key} mismatch")


def verify_pr_metadata(
    run: dict[str, object],
    packet: dict[str, object],
) -> str:
    pull_requests = run.get("pull_requests")
    if not isinstance(pull_requests, list):
        fail("workflow run has no pull-request association")
    for pr in pull_requests:
        if not isinstance(pr, dict):
            continue
        if pr.get("number") != packet["pull_request_number"]:
            continue
        base = pr.get("base")
        head = pr.get("head")
        if not isinstance(base, dict) or base.get("sha") != packet["base_revision"]:
            fail("workflow run base SHA does not match packet")
        if not isinstance(head, dict):
            fail("workflow run head metadata is malformed")
        repo = head.get("repo")
        if not isinstance(repo, dict):
            fail("workflow run head repository metadata is malformed")
        full_name = repo.get("full_name")
        if not isinstance(full_name, str) or not full_name:
            fail("workflow run head repository name is missing")
        return full_name
    fail("workflow run is not associated with packet PR")


def verify_jobs(jobs_payload: object, packet: dict[str, object]) -> None:
    if not isinstance(jobs_payload, dict):
        fail("workflow jobs response is not an object")
    jobs = jobs_payload.get("jobs")
    if not isinstance(jobs, list):
        fail("workflow jobs response has no job list")

    matches = [
        job
        for job in jobs
        if isinstance(job, dict) and job.get("name") == EXPECTED_JOB
    ]
    if len(matches) != 1:
        fail(f"expected exactly one {EXPECTED_JOB} job")
    job = matches[0]

    if job.get("status") != "completed" or job.get("conclusion") != "success":
        fail("provenance-construction job did not complete successfully")
    if job.get("head_sha") != packet["source_revision"]:
        fail("provenance-construction head SHA mismatch")
    if job.get("workflow_name") != EXPECTED_WORKFLOW:
        fail("provenance-construction workflow identity mismatch")

    steps = job.get("steps")
    if not isinstance(steps, list):
        fail("provenance-construction job has no step records")

    named: dict[str, list[dict[str, object]]] = {}
    for step in steps:
        if isinstance(step, dict) and isinstance(step.get("name"), str):
            named.setdefault(step["name"], []).append(step)

    for name in REQUIRED_STEPS:
        matches = named.get(name, [])
        if len(matches) != 1:
            fail(f"expected exactly one required step: {name!r}")
        if (
            matches[0].get("status") != "completed"
            or matches[0].get("conclusion") != "success"
        ):
            fail(f"required step did not succeed: {name!r}")


def verify_artifact(
    payload: object,
    artifact_dir: Path,
) -> None:
    if not isinstance(payload, dict):
        fail("artifacts response is not an object")
    artifacts = payload.get("artifacts")
    if not isinstance(artifacts, list):
        fail("artifacts response has no artifact list")
    matches = [
        artifact
        for artifact in artifacts
        if isinstance(artifact, dict)
        and artifact.get("name") == "sol-atlas-qualification-execution-v2"
    ]
    if len(matches) != 1:
        fail("expected exactly one qualification evidence artifact")
    if matches[0].get("expired") is True:
        fail("qualification evidence artifact is expired")

    packet, packet_bytes = load_packet(
        artifact_dir / "qualification-execution-v2.json"
    )
    verify_sidecar(
        artifact_dir / "qualification-execution-v2.json.sha256",
        packet_bytes,
    )
    if packet["run_id"] <= 0:
        fail("artifact packet has invalid run_id")


def verify_source_bindings(
    api_url: str,
    source_repository: str,
    base_repository: str,
    source_revision: str,
    packet: dict[str, object],
    token: str,
) -> None:
    workflow = fetch_file(
        api_url, source_repository, EXPECTED_WORKFLOW_FILE, source_revision, token
    )
    executed_workflow = fetch_file(
        api_url,
        base_repository,
        EXPECTED_WORKFLOW_FILE,
        packet["workflow_sha"],
        token,
    )
    verifier = fetch_file(
        api_url,
        source_repository,
        "tools/verify_qualification_execution_evidence.py",
        source_revision,
        token,
    )
    cargo_lock = fetch_file(
        api_url, source_repository, "Cargo.lock", source_revision, token
    )

    if digest(workflow) != packet["checked_out_workflow_file_sha256"]:
        fail("PR workflow bytes do not match packet hash")
    if workflow != executed_workflow:
        fail("workflow bytes at PR head differ from the GitHub workflow definition commit")
    if digest(verifier) != packet["verifier_sha256"]:
        fail("PR verifier bytes do not match packet hash")
    if digest(cargo_lock) != packet["cargo_lock_sha256"]:
        fail("PR Cargo.lock bytes do not match packet hash")

    workflow_text = executed_workflow.decode("utf-8", errors="strict")
    for snippet in REQUIRED_WORKFLOW_SNIPPETS:
        if snippet not in workflow_text:
            fail(f"required workflow policy is missing: {snippet!r}")
    for snippet in FORBIDDEN_PRIVILEGES:
        if snippet in workflow_text:
            fail(f"workflow contains forbidden privilege: {snippet!r}")

    action_refs = re.findall(
        r"^[ \t-]*uses:[ \t]*[^@\s]+@([^\s#]+)",
        workflow_text,
        re.MULTILINE,
    )
    if any(not SHA1.fullmatch(ref) for ref in action_refs):
        fail("workflow contains an unpinned action reference")


def main() -> int:
    args = parse_args()
    packet, _ = load_packet(args.artifact_dir / "qualification-execution-v2.json")
    verify_sidecar(
        args.artifact_dir / "qualification-execution-v2.json.sha256",
        (args.artifact_dir / "qualification-execution-v2.json").read_bytes(),
    )

    run = api_get(
        f"{args.api_url.rstrip('/')}/repos/{args.repository}/actions/runs/{args.run_id}",
        args.token,
    )
    if not isinstance(run, dict):
        fail("workflow run API response is not an object")

    verify_packet(
        packet,
        run,
        args.repository,
        args.server_url,
        args.run_id,
    )
    head_repository = verify_pr_metadata(run, packet)

    jobs = api_get(
        f"{args.api_url.rstrip('/')}/repos/{args.repository}/actions/runs/"
        f"{args.run_id}/jobs?per_page=100",
        args.token,
    )
    verify_jobs(jobs, packet)

    artifacts = api_get(
        f"{args.api_url.rstrip('/')}/repos/{args.repository}/actions/runs/"
        f"{args.run_id}/artifacts?per_page=100",
        args.token,
    )
    verify_artifact(artifacts, args.artifact_dir)

    verify_source_bindings(
        args.api_url,
        head_repository,
        args.repository,
        packet["source_revision"],
        packet,
        args.token,
    )

    print(
        "trusted qualification run verified:",
        args.repository,
        f"PR #{packet['pull_request_number']}",
        packet["source_revision"],
        f"run {args.run_id}",
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
