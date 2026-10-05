#!/usr/bin/env python3
"""Adversarial self-test for the qualification evidence V2 verifier."""

from __future__ import annotations

import hashlib
import json
import subprocess
import sys
import tempfile
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
VERIFIER = ROOT / "tools/verify_qualification_execution_evidence.py"
WORKFLOW = ROOT / ".github/workflows/check.yml"
CARGO_LOCK = ROOT / "Cargo.lock"

EXPECTED_COMMANDS = [
    "cargo test -p sol-atlas-core --locked -- --list",
    "cargo test -p sol-atlas-core --locked",
]


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def run_verifier(
    packet: Path,
    sidecar: Path,
    inventory: Path,
    verifier: Path,
    workflow: Path = WORKFLOW,
    cargo_lock: Path = CARGO_LOCK,
) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        [
            sys.executable,
            str(verifier),
            str(packet),
            str(sidecar),
            "--workflow-file",
            str(workflow),
            "--verifier-file",
            str(verifier),
            "--cargo-lock",
            str(cargo_lock),
            "--test-inventory",
            str(inventory),
            "--repository",
            "Luminous-Dynamics/sol-atlas-leptos",
            "--pull-request-number",
            "13",
            "--source-revision",
            "0123456789abcdef0123456789abcdef01234567",
            "--base-revision",
            "89abcdef0123456789abcdef0123456789abcdef",
            "--workflow-name",
            "Check",
            "--workflow-ref",
            "Luminous-Dynamics/sol-atlas-leptos/.github/workflows/check.yml@refs/pull/13/head",
            "--workflow-file-path",
            ".github/workflows/check.yml",
            "--workflow-sha",
            "fedcba9876543210fedcba9876543210fedcba98",
            "--run-id",
            "999999999",
            "--run-attempt",
            "1",
            "--job-name",
            "provenance-construction",
            "--check-run-id",
            "888888888",
            "--server-url",
            "https://github.com",
        ],
        text=True,
        capture_output=True,
        check=False,
    )


def main() -> int:
    if not VERIFIER.is_file() or not WORKFLOW.is_file() or not CARGO_LOCK.is_file():
        raise SystemExit("repository verification fixtures are missing")

    with tempfile.TemporaryDirectory(prefix="sol-atlas-qualification-verifier-") as temp:
        root = Path(temp)
        inventory = root / "test-inventory.txt"
        inventory.write_text("fixture-test-one\nfixture-test-two\n", encoding="utf-8")

        evidence = {
            "schema": "sol-atlas:qualification-execution-evidence:v2",
            "qualification_kind": "sol_atlas_core_provenance_construction",
            "status": "passed",
            "repository": "Luminous-Dynamics/sol-atlas-leptos",
            "pull_request_number": 13,
            "source_revision": "0123456789abcdef0123456789abcdef01234567",
            "base_revision": "89abcdef0123456789abcdef0123456789abcdef",
            "workflow_name": "Check",
            "workflow_ref": "Luminous-Dynamics/sol-atlas-leptos/.github/workflows/check.yml@refs/pull/13/head",
            "workflow_file_path": ".github/workflows/check.yml",
            "workflow_sha": "fedcba9876543210fedcba9876543210fedcba98",
            "checked_out_workflow_file_sha256": digest(WORKFLOW),
            "verifier_sha256": digest(VERIFIER),
            "run_id": 999999999,
            "run_attempt": 1,
            "job_name": "provenance-construction",
            "check_run_id": 888888888,
            "workflow_run_url": "https://github.com/Luminous-Dynamics/sol-atlas-leptos/actions/runs/999999999",
            "runner_os": "Linux",
            "runner_arch": "X64",
            "rust_toolchain": "1.99.0",
            "rustc_version_verbose": "fixture",
            "cargo_version_verbose": "fixture",
            "cargo_lock_sha256": digest(CARGO_LOCK),
            "test_inventory_sha256": digest(inventory),
            "qualification_commands": EXPECTED_COMMANDS,
        }

        packet = root / "qualification.json"
        sidecar = root / "qualification.json.sha256"

        def write_packet(value: dict) -> None:
            packet.write_text(
                json.dumps(value, sort_keys=True, separators=(",", ":"), ensure_ascii=False) + "\n",
                encoding="utf-8",
            )
            sidecar.write_text(digest(packet) + "\n", encoding="utf-8")

        write_packet(evidence)
        result = run_verifier(packet, sidecar, inventory, VERIFIER)
        if result.returncode != 0:
            raise SystemExit(f"valid packet rejected: {result.stderr}")

        tampered = dict(evidence)
        tampered["run_id"] += 1
        write_packet(tampered)
        stale_sidecar = sidecar.read_text(encoding="utf-8")
        tampered["run_id"] += 1
        packet.write_text(
            json.dumps(tampered, sort_keys=True, separators=(",", ":"), ensure_ascii=False) + "\n",
            encoding="utf-8",
        )
        sidecar.write_text(stale_sidecar, encoding="utf-8")
        result = run_verifier(packet, sidecar, inventory, VERIFIER)
        if result.returncode == 0:
            raise SystemExit("verifier accepted a packet with a stale sidecar")

        tampered = dict(evidence)
        tampered["verifier_sha256"] = "0" * 64
        write_packet(tampered)
        result = run_verifier(packet, sidecar, inventory, VERIFIER)
        if result.returncode == 0:
            raise SystemExit("verifier accepted a verifier-identity mismatch")

        tampered_workflow = root / "tampered-check.yml"
        tampered_workflow.write_bytes(WORKFLOW.read_bytes() + b"\\n# adversarial fixture mutation\\n")
        write_packet(evidence)
        result = run_verifier(
            packet,
            sidecar,
            inventory,
            VERIFIER,
            workflow=tampered_workflow,
        )
        if result.returncode == 0:
            raise SystemExit("verifier accepted a workflow-file hash mismatch")

        tampered_cargo_lock = root / "tampered-Cargo.lock"
        tampered_cargo_lock.write_bytes(CARGO_LOCK.read_bytes() + b"\\n# adversarial fixture mutation\\n")
        result = run_verifier(
            packet,
            sidecar,
            inventory,
            VERIFIER,
            cargo_lock=tampered_cargo_lock,
        )
        if result.returncode == 0:
            raise SystemExit("verifier accepted a Cargo.lock hash mismatch")

        tampered_inventory = root / "tampered-test-inventory.txt"
        tampered_inventory.write_bytes(inventory.read_bytes() + b"fixture-test-three\\n")
        result = run_verifier(
            packet,
            sidecar,
            tampered_inventory,
            VERIFIER,
        )
        if result.returncode == 0:
            raise SystemExit("verifier accepted a test-inventory hash mismatch")

        tampered_verifier = root / "tampered-verifier.py"
        tampered_verifier.write_bytes(VERIFIER.read_bytes() + b"\\n# adversarial fixture mutation\\n")
        result = run_verifier(
            packet,
            sidecar,
            inventory,
            tampered_verifier,
        )
        if result.returncode == 0:
            raise SystemExit("verifier accepted a verifier-file hash mismatch")

        print("qualification evidence verifier adversarial self-test passed")
        return 0


if __name__ == "__main__":
    sys.exit(main())
