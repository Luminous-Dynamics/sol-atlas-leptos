#!/usr/bin/env python3
# Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
# SPDX-License-Identifier: AGPL-3.0-or-later

from __future__ import annotations

from pathlib import Path

WORKFLOW = ".github/workflows/qualify-gcs.yml"
OBSERVER_FILES = (
    "artifacts/gcs-wif-trust-verification.json",
    "observer-artifacts/github-oidc-claims.json",
    "observer-artifacts/github-oidc-subject-configuration.json",
    "artifacts/gcs-effective-iam-verification.json",
    "artifacts/gcs-broad-sa-impersonation-verification.json",
    "artifacts/gcs-observer-isolation-verification.json",
    "artifacts/gcs-policy-effect-verification.json",
)
PROMOTION_MAP = (
    ("gcs-wif-trust-verification.json", "gcs-wif-trust-verification.json"),
    ("github-oidc-claims.json", "observer-github-oidc-claims.json"),
    ("github-oidc-subject-configuration.json", "github-oidc-subject-configuration.json"),
    ("gcs-effective-iam-verification.json", "gcs-effective-iam-verification.json"),
    ("gcs-broad-sa-impersonation-verification.json", "gcs-broad-sa-impersonation-verification.json"),
    ("gcs-observer-isolation-verification.json", "gcs-observer-isolation-verification.json"),
    ("gcs-policy-effect-verification.json", "gcs-policy-effect-verification.json"),
)
FINAL_FILES = (
    "artifacts/github-oidc-claims.json",
    "artifacts/github-oidc-subject-configuration.json",
    "artifacts/gcs-wif-credential-config-verification.json",
    "artifacts/gcs-wif-trust-verification.json",
    "artifacts/github-workflow-run-verification.json",
    "artifacts/gcs-external-effect-report.json",
)

def step(workflow: str, name: str) -> str:
    marker = "      - name: " + name + "\n"
    start = workflow.find(marker)
    if start < 0:
        raise AssertionError("missing workflow step: " + name)
    start += len(marker)
    end = workflow.find("      - name: ", start)
    return workflow[start:] if end < 0 else workflow[start:end]

def require(block: str, needles: tuple[str, ...], label: str) -> None:
    missing = [needle for needle in needles if needle not in block]
    if missing:
        raise AssertionError(label + " missing: " + ", ".join(missing))


def path_entries(block: str) -> tuple[str, ...]:
    marker = "          path: |\n"
    start = block.find(marker)
    if start < 0:
        raise AssertionError("artifact upload path block is missing")
    start += len(marker)
    end = block.find("\n          if-no-files-found:", start)
    if end < 0:
        raise AssertionError("artifact upload path termination is missing")
    entries = tuple(
        line.strip()
        for line in block[start:end].splitlines()
        if line.strip()
    )
    if not entries:
        raise AssertionError("artifact upload path is empty")
    return entries


def verify(workflow: str) -> dict[str, object]:
    observer = step(workflow, "Upload WIF trust evidence")
    if path_entries(observer) != OBSERVER_FILES:
        raise AssertionError("observer artifact allowlist drift")
    require(
        observer,
        ("if-no-files-found: error", "id: upload_wif_evidence"),
        "observer upload controls",
    )
    promotion = step(
        workflow,
        "Promote observed WIF trust evidence",
    )
    promotion_tests = tuple(
        "test -s artifacts/wif-observer/" + source
        for source, _ in PROMOTION_MAP
    )
    require(
        promotion,
        promotion_tests,
        "observer evidence promotion",
    )
    promotion_copies = tuple(
        "cp artifacts/wif-observer/"
        + source
        + " artifacts/"
        + destination
        for source, destination in PROMOTION_MAP
    )
    require(
        promotion,
        promotion_copies,
        "observer evidence promotion copy",
    )

    download_observer = step(workflow, "Download observed WIF trust evidence")
    require(
        download_observer,
        (
            "needs.observe-wif-trust.outputs.artifact_id",
            "path: artifacts/wif-observer/",
            "merge-multiple: true",
            "digest-mismatch: error",
        ),
        "observer artifact handoff",
    )
    final = step(workflow, "Upload qualification evidence")
    if path_entries(final) != FINAL_FILES:
        raise AssertionError("final artifact allowlist drift")
    require(
        final,
        ("if-no-files-found: error", "id: upload_evidence"),
        "final upload controls",
    )
    publish = step(workflow, "Download qualification evidence")
    require(
        publish,
        (
            "needs.qualify.outputs.artifact_id",
            "path: artifacts/",
            "digest-mismatch: error",
        ),
        "publish artifact handoff",
    )
    return {
        "schema": "sol-atlas:qualification-artifact-handoff:v1",
        "observer_file_count": len(OBSERVER_FILES),
        "final_file_count": len(FINAL_FILES),
        "observer_handoff_verified": True,
        "publish_handoff_verified": True,
    }

def main() -> int:
    workflow = Path(WORKFLOW).read_text(encoding="utf-8")
    result = verify(workflow)
    print("qualification artifact handoff checks: PASS " + str(result["final_file_count"]))
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
