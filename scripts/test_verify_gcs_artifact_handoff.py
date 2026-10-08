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
    "Promote observed WIF trust evidence",
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
             + "".join("        " + value + "\n" for value in module.OBSERVER_FILES)),
        step(STEPS[1],
             "        artifact-ids: needs.observe-wif-trust.outputs.artifact_id\n"
             "        path: artifacts/wif-observer/\n"
             "        digest-mismatch: error"),
        step(
            STEPS[2],
            "".join(
                "        test -s artifacts/wif-observer/" + source + "\n"
                + "        cp artifacts/wif-observer/"
                + source
                + " artifacts/"
                + destination
                + "\n"
                for source, destination in module.PROMOTION_MAP
            ),
        ),
        step(STEPS[3],
             "        id: upload_evidence\n"
             "        if-no-files-found: error\n"
             + "".join("        " + value + "\n" for value in module.FINAL_FILES)),
        step(STEPS[4],
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

    duplicate_step = workflow() + step(
        "Upload WIF trust evidence",
        "        id: decoy\n",
    )
    expect_failure(
        duplicate_step,
        "duplicate observer upload step",
    )
    reordered = workflow()
    marker_a = step(
        "Download observed WIF trust evidence",
        "        decoy: true\n",
    )
    marker_b = step(
        "Promote observed WIF trust evidence",
        "        decoy: true\n",
    )
    reordered = reordered.replace(marker_a, "")
    reordered = reordered.replace(marker_b, marker_a, 1)
    expect_failure(
        reordered,
        "observer download/promotion order drift",
    )

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
    promotion_drift = workflow().replace(
        "artifacts/wif-observer/" + module.PROMOTION_MAP[0][0],
        "artifacts/wif-observer/unexpected.json",
        1,
    )
    expect_failure(
        promotion_drift,
        "observer promotion source drift",
    )
    promotion_target_drift = workflow().replace(
        "artifacts/" + module.PROMOTION_MAP[0][1],
        "artifacts/unexpected-target.json",
        1,
    )
    expect_failure(
        promotion_target_drift,
        "observer promotion destination drift",
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
        workflow().replace(
            "path: artifacts/wif-observer/",
            "path: artifacts/wrong-observer/",
        ),
        "observer download namespace drift",
    )
    expect_failure(
        workflow().replace(
            "merge-multiple: true",
            "merge-multiple: false",
        ),
        "observer merge semantics drift",
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
