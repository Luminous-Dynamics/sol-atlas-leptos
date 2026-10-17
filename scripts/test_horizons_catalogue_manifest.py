#!/usr/bin/env python3
"""Unit tests for the shared strict Horizons catalogue-manifest loader."""
from __future__ import annotations

import json
import unittest
from pathlib import Path
from tempfile import TemporaryDirectory

from horizons_catalogue_manifest import CatalogueManifestError, load_catalogue_manifest


ROOT = Path(__file__).resolve().parents[1]
MANIFEST_PATH = ROOT / "sol-atlas-core/tests/fixtures/horizons/catalogue-bindings.json"


class CatalogueManifestTests(unittest.TestCase):
    def test_repository_manifest_has_unique_valid_identities(self) -> None:
        manifest = load_catalogue_manifest(MANIFEST_PATH)
        self.assertEqual(manifest["schema_version"], 1)
        self.assertEqual(len(manifest["objects"]), 43)
        self.assertEqual(
            {center["id"] for center in manifest["special_centers"]},
            {"ssb"},
        )

    def test_duplicate_json_keys_are_rejected(self) -> None:
        with TemporaryDirectory() as temporary:
            path = Path(temporary) / "manifest.json"
            path.write_text(
                '{"schema_version":1,"schema_version":1,"objects":[]}',
                encoding="utf-8",
            )
            with self.assertRaisesRegex(
                CatalogueManifestError, "duplicate JSON object key: 'schema_version'"
            ):
                load_catalogue_manifest(path)

    def test_nonstandard_numeric_constants_are_rejected(self) -> None:
        with TemporaryDirectory() as temporary:
            path = Path(temporary) / "manifest.json"
            path.write_text(
                '{"schema_version":1,"objects":[],"unused":NaN}',
                encoding="utf-8",
            )
            with self.assertRaisesRegex(
                CatalogueManifestError, "non-standard JSON numeric token: NaN"
            ):
                load_catalogue_manifest(path)

    def test_schema_version_requires_integer_one_not_boolean(self) -> None:
        with TemporaryDirectory() as temporary:
            path = Path(temporary) / "manifest.json"
            path.write_text(
                json.dumps({"schema_version": True, "objects": []}),
                encoding="utf-8",
            )
            with self.assertRaisesRegex(
                CatalogueManifestError, "unsupported catalogue manifest schema_version"
            ):
                load_catalogue_manifest(path)

    def test_duplicate_ids_across_objects_and_special_centres_are_rejected(self) -> None:
        manifest = {
            "schema_version": 1,
            "objects": [
                {"id": "mars", "name": "Mars", "kind": "planet", "command": "499", "center": "@499"}
            ],
            "special_centers": [
                {"id": "mars", "name": "Mars special", "provider_center": "@0"}
            ],
        }
        with TemporaryDirectory() as temporary:
            path = Path(temporary) / "manifest.json"
            path.write_text(json.dumps(manifest), encoding="utf-8")
            with self.assertRaisesRegex(
                CatalogueManifestError, "duplicate catalogue binding ID: mars"
            ):
                load_catalogue_manifest(path)

    def test_malformed_manifest_shapes_fail_closed(self) -> None:
        cases = [
            ([], "catalogue manifest root must be an object"),
            (
                {"schema_version": 1, "objects": ["mars"]},
                "catalogue manifest object 0 must be an object",
            ),
            (
                {
                    "schema_version": 1,
                    "objects": [
                        {"id": "mars", "name": "Mars", "kind": "planet", "command": "499"}
                    ],
                },
                "catalogue object mars has invalid center",
            ),
            (
                {"schema_version": 1, "objects": [], "special_centers": None},
                "catalogue manifest special_centers must be a list",
            ),
            (
                {
                    "schema_version": 1,
                    "objects": [],
                    "special_centers": [{"id": "ssb", "name": "SSB"}],
                },
                "special centre 0 in catalogue manifest has invalid provider_center",
            ),
        ]
        with TemporaryDirectory() as temporary:
            path = Path(temporary) / "manifest.json"
            for value, expected_error in cases:
                with self.subTest(expected_error=expected_error):
                    path.write_text(json.dumps(value), encoding="utf-8")
                    with self.assertRaisesRegex(CatalogueManifestError, expected_error):
                        load_catalogue_manifest(path)


if __name__ == "__main__":
    unittest.main()
