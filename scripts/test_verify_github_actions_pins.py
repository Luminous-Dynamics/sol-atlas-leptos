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

spec = importlib.util.spec_from_file_location(
    "verify_github_actions_pins",
    SCRIPT,
)
assert spec and spec.loader
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


def main() -> None:
    with TemporaryDirectory() as tmp:
        root = Path(tmp)
        workflow = root / ".github" / "workflows" / "qualify-gcs.yml"
        workflow.parent.mkdir(parents=True)
        workflow.write_text(
            """
jobs:
  qualify:
    steps:
      - uses: actions/checkout@aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
      - run: python3 verify.py --token-output "$RUNNER_TEMP/token.jwt"
      - name: cleanup
        run: rm -f "$RUNNER_TEMP/token.jwt"
      - uses: actions/upload-artifact@bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
""".strip()
            + "\n",
            encoding="utf-8",
        )
        errors = module.validate(root)
        assert errors == [], errors

        workflow.write_text(
            """
jobs:
  qualify:
    steps:
      - uses: actions/checkout@aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
      - run: python3 verify.py --token-output "$RUNNER_TEMP/token.jwt"
      - uses: actions/cache@cccccccccccccccccccccccccccccccccccccccc
      - name: cleanup
        run: rm -f "$RUNNER_TEMP/token.jwt"
""".strip()
            + "\n",
            encoding="utf-8",
        )
        errors = module.validate(root)
        assert len(errors) == 1
        assert "OIDC token is materialized" in errors[0]

        workflow.write_text(
            """
jobs:
  qualify:
    steps:
      - uses: actions/checkout@aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
      - run: python3 verify.py --token-output "$RUNNER_TEMP/token.jwt"
      - name: upload
        uses: actions/upload-artifact@bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
      - name: cleanup
        run: rm -f "$RUNNER_TEMP/token.jwt"
""".strip()
            + "\n",
            encoding="utf-8",
        )
        errors = module.validate(root)
        assert len(errors) == 2
        assert any("OIDC token is materialized" in error for error in errors)
        assert any(
            "artifact upload occurs before OIDC credential cleanup" in error
            for error in errors
        )

        workflow.write_text(
            """
jobs:
  qualify:
    permissions:
      id-token: write
    steps:
      - run: python3 verify.py --token-output "$RUNNER_TEMP/token.jwt"
      - name: cleanup
        run: rm -f "$RUNNER_TEMP/token.jwt"
  publish:
    needs: qualify
    permissions:
      attestations: write
    steps:
      - uses: actions/upload-artifact@bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
""".strip()
            + "\n",
            encoding="utf-8",
        )
        errors = module.validate(root)
        assert errors == [], errors

        workflow.write_text(
            """
jobs:
  qualify:
    permissions:
      id-token: write
    steps:
      - run: python3 verify.py --token-output "$RUNNER_TEMP/token.jwt"
      - name: cleanup
        run: rm -f "$RUNNER_TEMP/token.jwt"
  publish:
    needs: qualify
    permissions:
      id-token: write
    steps: []
""".strip()
            + "\n",
            encoding="utf-8",
        )
        errors = module.validate(root)
        assert any(
            "publication job must not receive id-token: write" in error
            for error in errors
        )

        workflow.write_text(
            """
jobs:
  qualify:
    permissions:
      id-token: write
    steps:
      - run: python3 verify.py --token-output "$RUNNER_TEMP/token.jwt"
      - name: cleanup
        run: rm -f "$RUNNER_TEMP/token.jwt"
      - uses: actions/upload-artifact@bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
  publish:
    needs: qualify
    permissions:
      attestations: write
    steps:
      - uses: actions/checkout@aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
      - uses: actions/download-artifact@cccccccccccccccccccccccccccccccccccccccc
      - uses: actions/attest@dddddddddddddddddddddddddddddddddddddddd
""".strip()
            + "\n",
            encoding="utf-8",
        )
        errors = module.validate(root)
        assert errors == [], errors

        workflow.write_text(
            """
jobs:
  qualify:
    permissions:
      id-token: write
    outputs:
      evidence_bundle: unsafe
    steps:
      - run: python3 verify.py --token-output "$RUNNER_TEMP/token.jwt"
      - name: cleanup
        run: rm -f "$RUNNER_TEMP/token.jwt"
      - uses: actions/upload-artifact@bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb
  publish:
    needs: qualify
    permissions:
      attestations: write
    steps: []
""".strip()
            + "\n",
            encoding="utf-8",
        )
        errors = module.validate(root)
        assert any(
            "native artifact handoff" in error
            for error in errors
        )

    print("offline GitHub Actions hardening checks: PASS")


if __name__ == "__main__":
    main()
