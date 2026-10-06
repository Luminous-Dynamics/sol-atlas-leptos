#!/usr/bin/env python3
# Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
# SPDX-License-Identifier: AGPL-3.0-or-later

"""Fail-closed check that external GitHub Actions use immutable commit refs."""

from __future__ import annotations

import argparse
import re
from pathlib import Path

SHA_REF = re.compile(
    r"^[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+@(?P<sha>[0-9a-f]{40})$"
)
USES_LINE = re.compile(r"^\s*(?:-\s*)?uses:\s*(?P<ref>[^\s#]+)")


def workflow_paths(root: Path) -> list[Path]:
    workflow_dir = root / ".github" / "workflows"
    return sorted(
        path
        for path in [*workflow_dir.glob("*.yml"), *workflow_dir.glob("*.yaml")]
        if path.is_file()
    )


def validate(root: Path) -> list[str]:
    paths = workflow_paths(root)
    if not paths:
        raise AssertionError("no GitHub workflow files found")

    errors: list[str] = []
    for path in paths:
        for line_number, raw_line in enumerate(
            path.read_text(encoding="utf-8").splitlines(), 1
        ):
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
    return errors


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", default=".")
    args = parser.parse_args()

    errors = validate(Path(args.root))
    if errors:
        print("\n".join(errors))
        return 1

    print("verified: all external GitHub Actions use 40-hex immutable refs")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
