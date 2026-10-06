#!/usr/bin/env python3
# Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
# SPDX-License-Identifier: AGPL-3.0-or-later

"""Offline tests for GCS qualification report resource identity binding."""

from __future__ import annotations

import importlib.util
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts" / "qualify_gcs_external_effect.py"
spec = importlib.util.spec_from_file_location("qualify_gcs_external_effect", SCRIPT)
assert spec and spec.loader
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)

PREFIX = "sol-atlas/qualification/run-1-attempt-1-abc"


def expect_rejection(report: dict[str, object], message: str) -> None:
    try:
        module.verify_resource_identity(report)
    except AssertionError:
        return
    raise AssertionError(message)


def main() -> None:
    good = {
        "service": "Google Cloud Storage",
        "bucket": "sol-atlas-qualification",
        "object_prefix": PREFIX,
        "object_names": [
            PREFIX + "/main.bin",
            PREFIX + "/point-in-time.bin",
            PREFIX + "/metadata-race.bin",
        ],
    }
    module.verify_resource_identity(good)

    expect_rejection(
        dict(good, service="Other service"),
        "wrong external service was accepted",
    )
    expect_rejection(
        dict(good, bucket=""),
        "missing bucket identity was accepted",
    )
    expect_rejection(
        dict(good, object_prefix=""),
        "missing object prefix was accepted",
    )
    expect_rejection(
        dict(good, object_names=good["object_names"][:2]),
        "incomplete object identity list was accepted",
    )
    expect_rejection(
        dict(good, object_names=[
            PREFIX + "/main.bin",
            PREFIX + "/point-in-time.bin",
            "other-prefix/metadata-race.bin",
        ]),
        "object prefix drift was accepted",
    )
    expect_rejection(
        dict(good, object_names=[
            PREFIX + "/main.bin",
            PREFIX + "/main.bin",
            PREFIX + "/metadata-race.bin",
        ]),
        "duplicate object identity was accepted",
    )

    print("offline GCS report resource-identity checks: PASS")


if __name__ == "__main__":
    main()
