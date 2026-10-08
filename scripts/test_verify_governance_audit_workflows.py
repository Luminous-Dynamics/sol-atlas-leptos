#!/usr/bin/env python3
# Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
# SPDX-License-Identifier: AGPL-3.0-or-later

"""Regression tests for governance audit workflow safety."""

from __future__ import annotations

import importlib.util
from pathlib import Path
from tempfile import TemporaryDirectory

ROOT = Path(__file__).resolve().parents[1]
SCRIPT = ROOT / "scripts/verify_governance_audit_workflows.py"

spec = importlib.util.spec_from_file_location("audit_workflow_safety", SCRIPT)
assert spec and spec.loader
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)

GOOD = "\n".join([
    "name: Audit",
    "",
    "on:",
    "  workflow_dispatch:",
    "",
    "permissions:",
    "  contents: read",
    "",
    "jobs:",
    "  audit:",
    "    runs-on: ubuntu-latest",
    "    steps:",
    "      - uses: actions/checkout@" + "a" * 40,
    "        env:",
    "          GH_TOKEN: ${{ github.token }}",
    "        run: python3 scripts/verify_github_governance_root.py",
])


def expect_failure(text: str, permissions=("contents: read",)) -> None:
    with TemporaryDirectory() as tmp:
        path = Path(tmp) / "audit.yml"
        path.write_text(text, encoding="utf-8")
        try:
            module.audit_workflow(path, permissions)
        except AssertionError:
            return
    raise AssertionError("accepted unsafe audit workflow")


def main() -> None:
    with TemporaryDirectory() as tmp:
        path = Path(tmp) / "audit.yml"
        path.write_text(GOOD, encoding="utf-8")
        result = module.audit_workflow(path, ("contents: read",))
        assert result["action_count"] == 1

    expect_failure(GOOD.replace("workflow_dispatch:", "workflow_dispatch:\n  push:"))
    expect_failure(GOOD.replace("contents: read", "contents: write"))
    expect_failure(GOOD.replace("GH_TOKEN: ${{ github.token }}", "GH_TOKEN: ${{ secrets.ADMIN_TOKEN }}"))
    expect_failure(GOOD.replace("GH_TOKEN: ${{ github.token }}", "GH_TOKEN: ${{ github.token }}\n          OTHER: ${{ secrets.X }}"))
    expect_failure(GOOD.replace("workflow_dispatch:", "workflow_run:"))
    expect_failure(GOOD.replace("workflow_dispatch:", "workflow_call:"))
    expect_failure(GOOD.replace("workflow_dispatch:", "pull_request_target:"))
    expect_failure(GOOD.replace("uses: actions/checkout@" + "a" * 40, "uses: actions/checkout@main"))
    expect_failure(GOOD.replace("permissions:\n  contents: read", "permissions:\n  contents: read\n  actions: write"))
    expect_failure(GOOD.replace("jobs:", "    permissions:\n      contents: read\n\njobs:"))
    expect_failure(GOOD.replace("jobs:", "    environment: production\n\njobs:"))

    print("governance audit workflow safety checks: PASS")


if __name__ == "__main__":
    main()
