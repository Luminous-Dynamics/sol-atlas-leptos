#!/usr/bin/env python3
# Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
# SPDX-License-Identifier: AGPL-3.0-or-later

"""Verify the repository's GitHub OIDC subject customization state."""

from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
from pathlib import Path

REPOSITORY = "Luminous-Dynamics/sol-atlas-leptos"
EXPECTED_IMMUTABLE = True
SCHEMA = "sol-atlas:github-oidc-sub-configuration:v1"


def canonical(value: object) -> bytes:
    return json.dumps(
        value,
        sort_keys=True,
        separators=(",", ":"),
        ensure_ascii=True,
    ).encode("utf-8")


def digest(value: object) -> str:
    return "sha256:" + hashlib.sha256(canonical(value)).hexdigest()


def load_configuration() -> dict[str, object]:
    result = subprocess.run(
        [
            "gh",
            "api",
            f"/repos/{REPOSITORY}/actions/oidc/customization/sub",
            "--header",
            "Accept: application/vnd.github+json",
            "--header",
            "X-GitHub-Api-Version: 2026-03-10",
        ],
        check=True,
        capture_output=True,
        text=True,
        timeout=30,
    )
    payload = json.loads(result.stdout)
    if not isinstance(payload, dict):
        raise AssertionError("GitHub OIDC customization response is not an object")
    return payload


def verify(payload: dict[str, object], output: str | None) -> dict[str, object]:
    if payload.get("use_immutable_subject") is not EXPECTED_IMMUTABLE:
        raise AssertionError(
            "OIDC immutable subject mode is not explicitly enabled"
        )
    if payload.get("use_default") is not True:
        raise AssertionError(
            "OIDC subject configuration must use the default template"
        )
    if payload.get("include_claim_keys") != []:
        raise AssertionError(
            "OIDC subject configuration must not customize claim keys"
        )

    result: dict[str, object] = {
        "schema": SCHEMA,
        "repository": REPOSITORY,
        "use_default": payload.get("use_default"),
        "include_claim_keys": payload.get("include_claim_keys"),
        "use_immutable_subject": payload.get("use_immutable_subject"),
        "default_template_required": True,
        "custom_claim_keys_forbidden": True,
        "configuration": payload,
        "configuration_digest": digest(payload),
        "claim_ceiling": (
            "GitHub repository OIDC subject customization is explicitly configured "
            "for immutable repository-ID-based subjects with no custom claim-key "
            "template. This does not prove the token's signature, contents, or cloud "
            "provider trust configuration."
        ),
    }
    if output:
        path = Path(output)
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(
            json.dumps(result, indent=2, sort_keys=True) + "\n",
            encoding="utf-8",
        )
    return result


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--output")
    args = parser.parse_args()
    result = verify(load_configuration(), args.output)
    print(
        "verified GitHub immutable OIDC subject configuration: "
        + str(result["repository"])
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
