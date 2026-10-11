"""Negative tests for the lunar site-screening evidence boundary."""
from __future__ import annotations

import copy
import json
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))

from scripts.validate_lunar_site_screening import DEFAULT_DATA, validate_document  # noqa: E402


class LunarSiteScreeningContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.canonical = json.loads(DEFAULT_DATA.read_text(encoding="utf-8"))

    def setUp(self) -> None:
        self.document = copy.deepcopy(self.canonical)

    def assert_invalid(self, expected_fragment: str) -> None:
        errors = validate_document(self.document)
        self.assertTrue(errors, "mutated evidence document unexpectedly passed")
        self.assertTrue(
            any(expected_fragment in error for error in errors),
            f"expected {expected_fragment!r} in errors, got: {errors}",
        )

    def test_canonical_dataset_passes(self) -> None:
        self.assertEqual([], validate_document(self.document))

    def test_scalar_scoring_is_rejected(self) -> None:
        self.document["scoring"] = {"method": "weighted_sum", "winner": "mons-mouton-plateau"}
        self.assert_invalid("scoring must remain null")

    def test_nonempty_metrics_are_rejected(self) -> None:
        self.document["candidates"][0]["known_metrics"] = {"slope_degrees": 2.1}
        self.assert_invalid("known_metrics must remain empty")

    def test_point_coordinates_are_rejected(self) -> None:
        self.document["candidates"][0]["latitude"] = -89.0
        self.assert_invalid("point-coordinate or ranking fields are prohibited")

    def test_candidate_score_field_is_rejected(self) -> None:
        self.document["candidates"][0]["score"] = 0.95
        self.assert_invalid("point-coordinate or ranking fields are prohibited")

    def test_duplicate_candidate_ids_are_rejected(self) -> None:
        self.document["candidates"][1]["id"] = self.document["candidates"][0]["id"]
        self.assert_invalid("is duplicated")

    def test_missing_source_url_is_rejected(self) -> None:
        self.document["candidates"][0]["sources"] = []
        self.assert_invalid("sources must be a non-empty list")

    def test_weighted_policy_is_rejected(self) -> None:
        self.document["comparison_policy"]["weights"] = {"illumination": 0.5}
        self.assert_invalid("comparison_policy.weights must remain null")

    def test_missing_comparison_layer_is_rejected(self) -> None:
        self.document["required_comparison_layers"].remove("landing_ellipse_and_dispersion")
        self.assert_invalid("landing_ellipse_and_dispersion")

    def test_unknown_fields_are_rejected(self) -> None:
        self.document["candidates"][0]["confidence"] = 0.8
        self.assert_invalid("contains unrecognized keys")

    def test_malformed_candidate_status_is_rejected_without_crashing(self) -> None:
        self.document["candidates"][0]["status"] = []
        self.assert_invalid("status is not a recognized region/comparator state")

    def test_malformed_metadata_is_rejected_without_crashing(self) -> None:
        self.document["comparison_policy"]["required_metadata"] = [{}]
        self.assert_invalid("required_metadata must be a list of non-empty strings")


if __name__ == "__main__":
    unittest.main()
