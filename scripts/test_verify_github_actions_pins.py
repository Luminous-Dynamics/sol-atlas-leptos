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
  observe-wif-trust:
    permissions:
      contents: read
      actions: read
      id-token: write
    steps:
      - uses: actions/checkout@PIN
      - name: Verify GitHub server workflow-run provenance
        run: |
          python3 scripts/verify_github_workflow_run.py \
            --output observer-artifacts/github-workflow-run-verification.json
      - run: python3 observer.py --token-output "$RUNNER_TEMP/observer-token.jwt"
      - name: Remove transient observer credentials
        run: rm -f "$RUNNER_TEMP/observer-token.jwt"
      - uses: actions/upload-artifact@UPLOAD
        with:
          name: observer
          path: observer-artifacts/observer.json

  qualify:
    permissions:
      contents: read
      actions: read
      id-token: write
      attestations: write
    outputs:
      artifact_id: ${{ steps.upload_evidence.outputs.artifact-id }}
    steps:
      - uses: actions/checkout@PIN
      - name: Verify GitHub server workflow-run provenance
        run: |
          python3 scripts/verify_github_workflow_run.py \
            --output artifacts/github-workflow-run-verification.json
      - run: python3 verify.py --token-output "$RUNNER_TEMP/token.jwt"
      - name: cleanup
        run: rm -f "$RUNNER_TEMP/token.jwt"
      - name: upload
        id: upload_evidence
        uses: actions/upload-artifact@UPLOAD
        with:
          path: |
            artifacts/github-oidc-claims.json
            artifacts/gcs-wif-credential-config-verification.json
            artifacts/gcs-wif-trust-verification.json
            artifacts/github-workflow-run-verification.json
            artifacts/gcs-external-effect-report.json
      - uses: actions/attest@ATTEST
  publish:
    needs: qualify
    permissions:
      contents: read
      attestations: read
    steps:
      - uses: actions/download-artifact@DOWNLOAD
        with:
          artifact-ids: ${{ needs.qualify.outputs.artifact_id }}
          digest-mismatch: error
      - name: Re-verify qualification artifact
        run: |
          python3 scripts/qualify_gcs_external_effect.py \
            verify \
            --report artifacts/gcs-external-effect-report.json
      - name: Verify GitHub artifact attestation
        run: |
          gh attestation verify \
            artifacts/gcs-external-effect-report.json \
            --repo "$GITHUB_REPOSITORY" \
            --signer-workflow \
              "$GITHUB_REPOSITORY/.github/workflows/qualify-gcs.yml" \
            --signer-digest "$GITHUB_WORKFLOW_SHA" \
            --source-digest "$GITHUB_SHA" \
            --source-ref "$GITHUB_REF" \
            --predicate-type "https://slsa.dev/provenance/v1" \
            --deny-self-hosted-runners \
            --cert-oidc-issuer \
              "https://token.actions.githubusercontent.com"
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

        bad = GOOD.replace("      actions: read\n", "")
        expect(path, root, bad, "qualify must grant actions: read")

        bad = GOOD.replace(
            "            --output artifacts/github-workflow-run-verification.json\n",
            "            --output artifacts/other.json\n",
        )
        expect(
            path,
            root,
            bad,
            "GitHub server verification must write the exact evidence file",
        )

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

        bad = GOOD.replace(
            "attestations: read",
            "attestations: none",
        )
        expect(path, root, bad, "publish must grant attestations: read")

        bad = GOOD.replace(
            "      - name: Verify GitHub artifact attestation\n"
            "        run: |\n"
            "          gh attestation verify \\\n"
            "            artifacts/gcs-external-effect-report.json \\\n"
            "            --repo \\\"$GITHUB_REPOSITORY\\\" \\\n"
            "            --signer-workflow \\\n"
            "              \\\"$GITHUB_REPOSITORY/.github/workflows/qualify-gcs.yml\\\" \\\n"
            "            --signer-digest \\\"$GITHUB_WORKFLOW_SHA\\\" \\\n"
            "            --source-digest \\\"$GITHUB_SHA\\\" \\\n"
            "            --source-ref \\\"$GITHUB_REF\\\" \\\n"
            "            --cert-oidc-issuer \\\n"
            "              \\\"https://token.actions.githubusercontent.com\\\"",
            "      - name: Verify GitHub artifact attestation\n"
            "        run: |\n"
            "          gh attestation verify \\\n"
            "            artifacts/gcs-external-effect-report.json \\\n"
            "            --repo \\\"$GITHUB_REPOSITORY\\\" \\\n"
            "            --signer-workflow \\\n"
            "              \\\"$GITHUB_REPOSITORY/.github/workflows/qualify-gcs.yml\\\"",
        )
        expect(path, root, bad, "attestation verification must bind source digest")

        bad = GOOD.replace(
            '            --signer-workflow \\\n'
            '              "$GITHUB_REPOSITORY/.github/workflows/qualify-gcs.yml" \\\n',
            '            --signer-workflow \\\n'
            '              "$GITHUB_REPOSITORY/.github/workflows/other.yml" \\\n',
        )
        expect(
            path,
            root,
            bad,
            "attestation verification must use the exact qualification workflow signer",
        )

        bad = GOOD.replace(
            '            --signer-digest "$GITHUB_WORKFLOW_SHA" \\\n',
            "",
        )
        expect(path, root, bad, "attestation verification must bind signer digest")

        bad = GOOD.replace(
            '            --predicate-type "https://slsa.dev/provenance/v1" \\\n',
            "",
        )
        expect(
            path,
            root,
            bad,
            "attestation verification must pin the SLSA provenance predicate",
        )

        bad = GOOD.replace(
            '            --deny-self-hosted-runners \\\n',
            "",
        )
        expect(
            path,
            root,
            bad,
            "attestation verification must reject self-hosted runners",
        )

        verify_marker = (
            "      - name: Re-verify qualification artifact\n"
            "        run: |\n"
            "          python3 scripts/qualify_gcs_external_effect.py \\\n"
            "            verify \\\n"
            "            --report artifacts/gcs-external-effect-report.json\n"
        )
        attest_marker = (
            "      - name: Verify GitHub artifact attestation\n"
            "        run: |\n"
            "          gh attestation verify \\\n"
        )
        bad = GOOD.replace(
            verify_marker + attest_marker,
            attest_marker + verify_marker,
        )
        expect(
            path,
            root,
            bad,
            "attestation verification must follow report re-verification",
        )
        bad = GOOD.replace(
            "            artifacts/gcs-external-effect-report.json",
            "            artifacts/gcs-external-effect-report.json\n"
            "            artifacts/",
        )
        expect(
            path,
            root,
            bad,
            "qualification must not upload the broad artifacts directory",
        )
        expect(
            path,
            root,
            bad,
            "qualification must explicitly allowlist the four evidence files",
        )

        bad = GOOD.replace("  publish:", "    evidence_bundle: unsafe\n  publish:")
        expect(path, root, bad, "oversized job-output handoff")

    print("offline GitHub Actions hardening checks: PASS")

if __name__ == "__main__":
    main()