#!/usr/bin/env python3
# Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
# SPDX-License-Identifier: AGPL-3.0-or-later

from __future__ import annotations

import importlib.util
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "verify_gcs_artifact_handoff.py"
spec = importlib.util.spec_from_file_location("artifact_handoff", SCRIPT)
assert spec and spec.loader
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)

STEPS = (
    "Upload WIF trust evidence",
    "Download observed WIF trust evidence",
    "Upload qualification evidence",
    "Download qualification evidence",
)

def step(name: str, body: str) -> str:
    return "      - name: " + name + "\n" + body + "\n"

def workflow() -> str:
    return "\n".join((
        step(STEPS[0],
             "        id: upload_wif_evidence\n"
             "        if-no-files-found: error\n"
             + "".join("        - " + value + "\n" for value in module.OBSERVER_FILES)),
        step(STEPS[1],
             "        artifact-ids: needs.observe-wif-trust.outputs.artifact_id\n"
             "        digest-mismatch: error"),
        step(STEPS[2],
             "        id: upload_evidence\n"
             "        if-no-files-found: error\n"
             + "".join("        - " + value + "\n" for value in module.FINAL_FILES)),
        step(STEPS[3],
             "        artifact-ids: needs.qualify.outputs.artifact_id\n"
             "        digest-mismatch: error"),
    ))

def expect_failure(data: str, label: str) -> None:
    try:
        module.verify(data)
    except AssertionError:
        return
    raise AssertionError("accepted invalid artifact handoff: " + label)

def main() -> None:
    good = module.verify(workflow())
    assert good["observer_file_count"] == 7
    assert good["final_file_count"] == 6

    expect_failure(
        workflow().replace(OBSERVER_FILES[0], "artifacts/missing.json"),
        "missing observer evidence",
    )
    extra_observer = workflow().replace(
        OBSERVER_FILES[0],
        OBSERVER_FILES[0] + "\n            artifacts/unexpected.json",
    )
    expect_failure(
        extra_observer,
        "unexpected observer evidence",
    )
    expect_failure(
        workflow().replace(FINAL_FILES[-1], "artifacts/missing.json"),
        "missing final evidence",
    )
    extra_final = workflow().replace(
        FINAL_FILES[0],
        FINAL_FILES[0] + "\n            artifacts/unexpected.json",
    )
    expect_failure(
        extra_final,
        "unexpected final evidence",
    )
    expect_failure(
        workflow().replace("needs.qualify.outputs.artifact_id", "wrong-id"),
        "publish artifact ID drift",
    )
    expect_failure(
        workflow().replace("digest-mismatch: error", "digest-mismatch: warn", 1),
        "observer digest policy drift",
    )
    print("artifact handoff semantic checks: PASS")

if __name__ == "__main__":
    main()
