#!/usr/bin/env python3
# Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
# SPDX-License-Identifier: AGPL-3.0-or-later

"""Guard repository-controlled governance audit workflows against privilege."""

from __future__ import annotations

import argparse
import re
from pathlib import Path

SCHEMA = "sol-atlas:github-audit-workflow-safety:v1"
ROOT = Path(__file__).resolve().parents[1]
WORKFLOW_SPECS = {
    ".github/workflows/audit-governance-root.yml": {
        "permissions": ("contents: read",),
    },
    ".github/workflows/audit-qualification-environment.yml": {
        "permissions": ("contents: read", "actions: read"),
    },
}
FORBIDDEN = (
    "secrets.",
    "pull_request_target:",
    "workflow_run:",
    "workflow_call:",
    "repository_dispatch:",
    "id-token: write",
    "attestations: write",
    "contents: write",
    "actions: write",
    "deployments: write",
    "packages: write",
    "security-events: write",
    "permissions: write",
    "environment:",
)
ACTION_RE = re.compile(r"^\\s*-?\\s*uses:\s+[^@\\s]+@([0-9a-f]{40})(?:\\s+#.*)?$")


def permission_block(text: str) -> list[str]:
    lines = text.splitlines()
    try:
        start = lines.index("permissions:")
        end = next(
            index for index in range(start + 1, len(lines))
            if lines[index] == "jobs:"
        )
    except (ValueError, StopIteration) as exc:
        raise AssertionError("workflow permissions block is missing") from exc
    values = []
    for line in lines[start + 1:end]:
        stripped = line.strip()
        if stripped:
            values.append(stripped)
    return values


def audit_workflow(path: Path, expected_permissions: tuple[str, ...]) -> dict[str, object]:
    text = path.read_text(encoding="utf-8")
    if not text.startswith("name:"):
        raise AssertionError("audit workflow name is missing")
    try:
        trigger_start = text.index("on:\n") + len("on:\n")
        trigger_end = text.index("permissions:\n")
    except ValueError as exc:
        raise AssertionError("workflow trigger/permissions sections are missing") from exc
    trigger_lines = [
        line.strip()
        for line in text[trigger_start:trigger_end].splitlines()
        if line.strip()
    ]
    if trigger_lines != ["workflow_dispatch:"]:
        raise AssertionError("audit workflow must use workflow_dispatch only")
    for forbidden in FORBIDDEN:
        if forbidden in text:
            raise AssertionError("forbidden audit-workflow privilege: " + forbidden)
    if "GH_TOKEN: ${{ github.token }}" not in text:
        raise AssertionError("audit workflow must use github.token")
    if text.count("GH_TOKEN: ${{ github.token }}") != 1:
        raise AssertionError("audit workflow must have exactly one GH_TOKEN binding")
    if re.search(r"^\\s{4,}permissions:", text, flags=re.MULTILINE):
        raise AssertionError("job-level permissions are not allowed")
    permissions = permission_block(text)
    if tuple(permissions) != expected_permissions:
        raise AssertionError(
            "unexpected audit workflow permissions: "
            + repr(permissions)
        )
    action_refs = []
    for line in text.splitlines():
        if "uses:" not in line:
            continue
        match = ACTION_RE.match(line)
        if not match:
            raise AssertionError("audit workflow action is not immutably pinned")
        action_refs.append(match.group(1))
    if not action_refs:
        raise AssertionError("audit workflow has no pinned actions")
    return {
        "path": str(path.relative_to(ROOT)),
        "permissions": permissions,
        "action_count": len(action_refs),
        "action_shas": action_refs,
    }


def verify() -> dict[str, object]:
    findings = []
    for relative, spec in WORKFLOW_SPECS.items():
        path = ROOT / relative
        if not path.is_file():
            raise AssertionError("required audit workflow is missing: " + relative)
        findings.append(audit_workflow(path, spec["permissions"]))
    return {
        "schema": SCHEMA,
        "workflow_count": len(findings),
        "verified": True,
        "workflows": findings,
        "claim_ceiling": (
            "The checked-in governance audit workflows use only the expected "
            "read permissions, github.token, workflow_dispatch, and immutable "
            "action references. This does not prove GitHub organization policy "
            "or runtime token behavior."
        ),
    }


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output")
    args = parser.parse_args()
    result = verify()
    if args.output:
        destination = ROOT / args.output if not Path(args.output).is_absolute() else Path(args.output)
        destination.parent.mkdir(parents=True, exist_ok=True)
        import json

        destination.write_text(
            json.dumps(result, indent=2, sort_keys=True) + "\n",
            encoding="utf-8",
        )
    print("audit workflow safety checks: PASS")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
