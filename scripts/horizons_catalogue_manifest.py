#!/usr/bin/env python3
"""Strict shared loader for the Horizons capture catalogue contract.

Capture preflight and offline packet verification must apply the same structural
rules before trusting stable body IDs or provider bindings.
"""
from __future__ import annotations

import json
from pathlib import Path
from typing import Any


class CatalogueManifestError(ValueError):
    """The shared catalogue manifest is malformed or ambiguous."""


def _unique_object_pairs(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        if key in result:
            raise CatalogueManifestError(f"duplicate JSON object key: {key!r}")
        result[key] = value
    return result


def _reject_nonstandard_constant(token: str) -> Any:
    raise CatalogueManifestError(f"non-standard JSON numeric token: {token}")


def _required_nonempty_string(
    value: Any, field: str, context: str
) -> str:
    if not isinstance(value, str) or not value:
        raise CatalogueManifestError(f"{context} has invalid {field}")
    return value


def load_catalogue_manifest(path: str | Path) -> dict[str, Any]:
    """Load and validate the shared manifest, rejecting ambiguous identity data."""
    try:
        manifest = json.loads(
            Path(path).read_text(encoding="utf-8"),
            object_pairs_hook=_unique_object_pairs,
            parse_constant=_reject_nonstandard_constant,
        )
    except CatalogueManifestError:
        raise
    except (OSError, UnicodeError, ValueError) as error:
        raise CatalogueManifestError(
            f"cannot read strict catalogue manifest: {error}"
        ) from error

    if not isinstance(manifest, dict):
        raise CatalogueManifestError("catalogue manifest root must be an object")
    schema_version = manifest.get("schema_version")
    if type(schema_version) is not int or schema_version != 1:
        raise CatalogueManifestError("unsupported catalogue manifest schema_version")
    if "description" in manifest and not isinstance(manifest["description"], str):
        raise CatalogueManifestError("catalogue manifest description must be a string")

    objects = manifest.get("objects")
    if not isinstance(objects, list):
        raise CatalogueManifestError("catalogue manifest objects must be a list")

    seen_ids: set[str] = set()
    for index, obj in enumerate(objects):
        context = f"catalogue manifest object {index}"
        if not isinstance(obj, dict):
            raise CatalogueManifestError(f"{context} must be an object")
        object_id = _required_nonempty_string(obj.get("id"), "id", context)
        _required_nonempty_string(obj.get("name"), "name", f"catalogue object {object_id}")
        _required_nonempty_string(obj.get("kind"), "kind", f"catalogue object {object_id}")
        if object_id in seen_ids:
            raise CatalogueManifestError(f"duplicate catalogue binding ID: {object_id}")
        seen_ids.add(object_id)
        for binding_key in ("command", "center"):
            binding = obj.get(binding_key)
            if binding_key not in obj or (
                binding is not None and (not isinstance(binding, str) or not binding)
            ):
                raise CatalogueManifestError(
                    f"catalogue object {object_id} has invalid {binding_key}"
                )

    special_centers = manifest.get("special_centers", [])
    if not isinstance(special_centers, list):
        raise CatalogueManifestError("catalogue manifest special_centers must be a list")
    for index, special in enumerate(special_centers):
        context = f"special centre {index} in catalogue manifest"
        if not isinstance(special, dict):
            raise CatalogueManifestError(f"{context} must be an object")
        special_id = _required_nonempty_string(special.get("id"), "id", context)
        _required_nonempty_string(special.get("name"), "name", context)
        _required_nonempty_string(special.get("provider_center"), "provider_center", context)
        if special_id in seen_ids:
            raise CatalogueManifestError(f"duplicate catalogue binding ID: {special_id}")
        seen_ids.add(special_id)

    return manifest
