#!/usr/bin/env python3
# Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
# SPDX-License-Identifier: AGPL-3.0-or-later

"""Fail-closed checks for GitHub Actions workflow hardening."""

from __future__ import annotations

import argparse
import re
from pathlib import Path

SHA_REF = re.compile(
    r"^[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+@[0-9a-f]{40}$"
)
USES_LINE = re.compile(r"^\s*(?:-\s*)?uses:\s*(?P<ref>[^\s#]+)")


def workflow_paths(root: Path) -> list[Path]:
    workflow_dir = root / ".github" / "workflows"
    return sorted(
        path
        for path in (
            *workflow_dir.glob("*.yml"),
            *workflow_dir.glob("*.yaml"),
        )
        if path.is_file()
    )


def job_bounds(lines: list[str], job: str) -> tuple[int, int] | None:
    start = next(
        (i for i, line in enumerate(lines) if line == f"  {job}:"),
        None,
    )
    if start is None:
        return None
    end = next(
        (
            i
            for i in range(start + 1, len(lines))
            if lines[i].startswith("  ")
            and not lines[i].startswith("    ")
        ),
        len(lines),
    )
    return start, end


def section(lines: list[str], job: str) -> list[str]:
    bounds = job_bounds(lines, job)
    return [] if bounds is None else lines[bounds[0] : bounds[1]]


def validate_qualification_policy(
    path: Path,
    lines: list[str],
) -> list[str]:
    if path.name != "qualify-gcs.yml":
        return []

    errors: list[str] = []
    qualify = section(lines, "qualify")
    publish = section(lines, "publish")
    if not qualify:
        errors.append(f"{path}: missing qualify job")
        return errors
    if not publish:
        errors.append(f"{path}: missing publish job")
        return errors

    if "      actions: read" not in qualify:
        errors.append(f"{path}: qualify must grant actions: read")
    server_output = (
        "            --output artifacts/github-workflow-run-verification.json"
    )
    if "      id-token: write" not in qualify:
        errors.append(f"{path}: qualify must grant id-token: write")
    if "      attestations: write" not in qualify:
        errors.append(f"{path}: qualify must grant attestations: write")
    if "      id-token: write" in publish:
        errors.append(f"{path}: publish must not grant id-token: write")
    if "      attestations: write" in publish:
        errors.append(f"{path}: publish must not grant attestations: write")
    if "      attestations: read" not in publish:
        errors.append(f"{path}: publish must grant attestations: read")
    if "    needs: qualify" not in publish:
        errors.append(f"{path}: publish must depend on qualify")

    workflow_text = "\n".join(lines)
    if "evidence_bundle:" in workflow_text:
        errors.append(f"{path}: oversized job-output handoff is forbidden")
    if "base64 -w0" in workflow_text:
        errors.append(f"{path}: base64 job-output handoff is forbidden")

    if (
        "artifact_id: ${{ steps.upload_evidence.outputs.artifact-id }}"
        not in workflow_text
    ):
        errors.append(
            f"{path}: qualify must expose the exact uploaded artifact ID"
        )
    if (
        "artifact-ids: ${{ needs.qualify.outputs.artifact_id }}"
        not in "\n".join(publish)
    ):
        errors.append(
            f"{path}: publish must download by exact qualification artifact ID"
        )
    if "actions/download-artifact@3e5f45b2cfb9172054b4087a40e8e0b5a5461e7c" not in (
        "\n".join(publish)
    ):
        errors.append(f"{path}: publish must pin download-artifact v8.0.1")
    if "digest-mismatch: error" not in "\n".join(publish):
        errors.append(
            f"{path}: publish must fail on artifact digest mismatch"
        )

    qualify_text = "\n".join(qualify)
    publish_text = "\n".join(publish)
    if "actions/attest@" not in qualify_text:
        errors.append(f"{path}: qualify must attest the report")
    if "actions/attest@" in publish_text:
        errors.append(f"{path}: publish must not attest the report")
    if "gh attestation verify" not in publish_text:
        errors.append(f"{path}: publish must cryptographically verify the report attestation")
    if "artifacts/gcs-external-effect-report.json" not in publish_text:
        errors.append(f"{path}: attestation verification must target the report")
    if "--signer-workflow" not in publish_text:
        errors.append(f"{path}: attestation verification must pin the signer workflow")
    if (
        (
            '--signer-workflow \\\n'
            '              "$GITHUB_REPOSITORY/.github/workflows/qualify-gcs.yml"'
        )
        not in publish_text
    ):
        errors.append(
            f"{path}: attestation verification must use the exact qualification workflow signer"
        )
    if "--signer-digest \"$GITHUB_WORKFLOW_SHA\"" not in publish_text:
        errors.append(
            f"{path}: attestation verification must bind signer digest "
            "to GITHUB_WORKFLOW_SHA"
        )
    if "--source-digest \"$GITHUB_SHA\"" not in publish_text:
        errors.append(f"{path}: attestation verification must bind source digest to GITHUB_SHA")
    if "--source-ref \"$GITHUB_REF\"" not in publish_text:
        errors.append(f"{path}: attestation verification must bind source ref to GITHUB_REF")
    if "--predicate-type \"https://slsa.dev/provenance/v1\"" not in publish_text:
        errors.append(f"{path}: attestation verification must pin the SLSA provenance predicate")
    if "--deny-self-hosted-runners" not in publish_text:
        errors.append(f"{path}: attestation verification must reject self-hosted runners")
    if "--cert-oidc-issuer" not in publish_text:
        errors.append(f"{path}: attestation verification must pin the GitHub OIDC issuer")
    if "actions/upload-artifact@" not in qualify_text:
        errors.append(f"{path}: qualify must upload evidence")
    expected_artifact_path = (
        "          path: |\n"
        "            artifacts/github-oidc-claims.json\n"
        "            artifacts/gcs-wif-credential-config-verification.json\n"
        "            artifacts/gcs-wif-trust-verification.json\n"
        "            artifacts/github-workflow-run-verification.json\n"
        "            artifacts/gcs-external-effect-report.json"
    )
    upload_actions = qualify_text.count("actions/upload-artifact@")
    if upload_actions != 1:
        errors.append(
            f"{path}: qualification must contain exactly one evidence uploader"
        )
    if expected_artifact_path not in qualify_text:
        errors.append(
            f"{path}: qualification must explicitly allowlist the five evidence files"
        )
    if "            artifacts/\n" in qualify_text:
        errors.append(
            f"{path}: qualification must not upload the broad artifacts directory"
        )
    if "actions/upload-artifact@" in publish_text:
        errors.append(f"{path}: publish must not upload a second evidence artifact")

    server_verification_line = next(
        (
            i
            for i, line in enumerate(lines)
            if line.strip() == "python3 scripts/verify_github_workflow_run.py \\"
        ),
        None,
    )
    token_line = next(
        (
            i
            for i, line in enumerate(lines)
            if "--token-output" in line
            and "sol-atlas-github-oidc-token.jwt" in line
        ),
        None,
    )
    cleanup_line = next(
        (
            i
            for i, line in enumerate(lines)
            if line.strip() == "- name: Remove transient OIDC credentials"
        ),
        None,
    )
    upload_line = next(
        (
            i
            for i, line in enumerate(lines)
            if "uses: actions/upload-artifact@" in line
        ),
        None,
    )
    attest_line = next(
        (
            i
            for i, line in enumerate(lines)
            if "uses: actions/attest@" in line
        ),
        None,
    )
    publish_line = next(
        (i for i, line in enumerate(lines) if line == "  publish:"),
        None,
    )
    report_verify_line = next(
        (
            i
            for i, line in enumerate(lines)
            if (
                publish_line is not None
                and i > publish_line
                and "python3 scripts/qualify_gcs_external_effect.py"
                in line
            )
        ),
        None,
    )
    verify_attestation_line = next(
        (
            i
            for i, line in enumerate(lines)
            if line.strip() == "gh attestation verify \\"
        ),
        None,
    )
    if server_verification_line is None:
        errors.append(f"{path}: GitHub server workflow-run verification is required")
    if (
        server_verification_line is not None
        and server_output not in qualify_text
    ):
        errors.append(
            f"{path}: GitHub server verification must write the exact evidence file"
        )
    if (
        server_verification_line is not None
        and token_line is not None
        and server_verification_line >= token_line
    ):
        errors.append(
            f"{path}: GitHub server workflow-run verification must precede "
            "OIDC token materialization"
        )
    if token_line is None:
        errors.append(f"{path}: OIDC token materialization is required")
    if cleanup_line is None:
        errors.append(f"{path}: transient credential cleanup is required")
    if (
        token_line is not None
        and cleanup_line is not None
        and cleanup_line <= token_line
    ):
        errors.append(f"{path}: cleanup must follow OIDC token materialization")
    if (
        token_line is not None
        and cleanup_line is not None
        and any(
            (match := USES_LINE.match(line))
            and not match.group("ref").startswith("./")
            for line in lines[token_line + 1 : cleanup_line]
        )
    ):
        errors.append(
            f"{path}: no external action may run in the OIDC secret window"
        )
    if (
        upload_line is None
        or cleanup_line is None
        or attest_line is None
        or publish_line is None
        or upload_line <= cleanup_line
        or upload_line >= attest_line
        or attest_line >= publish_line
    ):
        errors.append(
            f"{path}: evidence upload and attestation must occur after cleanup in qualify"
        )
    publish_lines = "\n".join(publish)
    if "python3 scripts/qualify_gcs_external_effect.py" not in publish_lines:
        errors.append(
            f"{path}: publication must re-verify the qualification report"
        )
    if verify_attestation_line is None:
        errors.append(f"{path}: attestation verification command is required")
    elif verify_attestation_line <= publish_line:
        errors.append(f"{path}: attestation verification must occur inside publish")
    if (
        report_verify_line is not None
        and verify_attestation_line is not None
        and verify_attestation_line <= report_verify_line
    ):
        errors.append(
            f"{path}: attestation verification must follow report re-verification"
        )

    return errors


def validate(root: Path) -> list[str]:
    paths = workflow_paths(root)
    if not paths:
        raise AssertionError("no GitHub workflow files found")

    errors: list[str] = []
    for path in paths:
        lines = path.read_text(encoding="utf-8").splitlines()
        for line_number, raw_line in enumerate(lines, 1):
            stripped = raw_line.strip()
            if not stripped or stripped.startswith("#"):
                continue
            match = USES_LINE.match(raw_line)
            if not match:
                continue
            ref = match.group("ref")
            if ref.startswith("./"):
                continue
            if not SHA_REF.fullmatch(ref):
                errors.append(
                    f"{path}:{line_number}: external action ref is not immutable: {ref}"
                )
        errors.extend(validate_qualification_policy(path, lines))
    return errors


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", default=".")
    args = parser.parse_args()

    errors = validate(Path(args.root))
    if errors:
        print("\n".join(errors))
        return 1

    print("verified: GitHub Actions workflow hardening policy")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
