#!/usr/bin/env python3
# Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
# SPDX-License-Identifier: AGPL-3.0-or-later

"""Offline semantic tests for GitHub Actions workflow hardening."""

from __future__ import annotations

import importlib.util
from pathlib import Path
from tempfile import TemporaryDirectory

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "verify_github_actions_pins.py"
spec = importlib.util.spec_from_file_location("verify_pins", SCRIPT)
assert spec and spec.loader
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)

PIN = "a" * 40
UPLOAD = "b" * 40
DOWNLOAD = "c" * 40
ATTEST = "d" * 40
GH = "${{"

GOOD = """
jobs:
  qualify:
    permissions:
      contents: read
      id-token: write
      attestations: write
    outputs:
      artifact_id: ${{ steps.upload_evidence.outputs.artifact-id }}
    steps:
      - uses: actions/checkout@PIN
      - run: python3 verify.py --token-output "$RUNNER_TEMP/token.jwt"
      - name: cleanup
        run: rm -f "$RUNNER_TEMP/token.jwt"
      - name: upload
        id: upload_evidence
        uses: actions/upload-artifact@UPLOAD
      - uses: actions/attest@ATTEST
  publish:
    needs: qualify
    permissions:
      contents: read
    steps:
      - uses: actions/download-artifact@DOWNLOAD
        with:
          artifact-ids: ${{ needs.qualify.outputs.artifact_id }}
          digest-mismatch: error
""".strip()

GOOD = GOOD.replace("PIN", PIN)
GOOD = GOOD.replace("UPLOAD", UPLOAD)
GOOD = GOOD.replace("DOWNLOAD", DOWNLOAD)
GOOD = GOOD.replace("ATTEST", ATTEST)

def expect(path: Path, root: Path, text: str, needle: str) -> None:
    path.write_text(text + "\n", encoding="utf-8")
    errors = module.validate(root)
    assert any(needle in error for error in errors), errors

def main() -> None:
    with TemporaryDirectory() as tmp:
        root = Path(tmp)
        path = root / ".github" / "workflows" / "qualify-gcs.yml"
        path.parent.mkdir(parents=True)
        path.write_text(GOOD + "\n", encoding="utf-8")
        assert module.validate(root) == []

        bad = GOOD.replace(
            "  publish:\n    needs: qualify\n",
            "  publish:\n    needs: qualify\n    permissions:\n      id-token: write\n",
        )
        expect(path, root, bad, "publish must not grant id-token: write")

        bad = GOOD.replace("digest-mismatch: error", "digest-mismatch: warn")
        expect(path, root, bad, "artifact digest mismatch")

        bad = GOOD.replace(
            "artifact-ids: ${{ needs.qualify.outputs.artifact_id }}",
            "name: sol-atlas-gcs-external-effect-qualification",
        )
        expect(path, root, bad, "exact qualification artifact ID")

        bad = GOOD.replace(
            "actions/download-artifact@" + DOWNLOAD,
            "actions/download-artifact@main",
        )
        expect(path, root, bad, "not immutable")

        bad = GOOD.replace("  publish:", "    evidence_bundle: unsafe\n  publish:")
        expect(path, root, bad, "oversized job-output handoff")

    print("offline GitHub Actions hardening checks: PASS")

if __name__ == "__main__":
    main()