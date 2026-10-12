"""Negative tests for the lunar site-screening evidence boundary."""
from __future__ import annotations

import copy
import json
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))
CATALOG_DATA = ROOT / "docs/data/lunar_geospatial_source_catalog.v1.json"

from scripts.validate_lunar_site_screening import DEFAULT_DATA, validate_document  # noqa: E402


class LunarSiteScreeningContractTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.canonical = json.loads(DEFAULT_DATA.read_text(encoding="utf-8"))
        cls.catalog = json.loads(CATALOG_DATA.read_text(encoding="utf-8"))

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


    def test_source_catalog_has_complete_provenance_metadata(self) -> None:
        self.assertEqual(
            self.catalog["schema"],
            "org.luminousdynamics.sol-atlas.lunar-source-catalog/v1",
        )
        self.assertEqual(self.catalog["status"], "source_catalogue_ingestion_plan_not_ingested")
        self.assertEqual(len(self.catalog["sources"]), 4)
        self.assertEqual(len(self.catalog["ingestion_sequence"]), 6)
        self.assertIn("No derived site metrics", self.catalog["metric_policy"])
        for source in self.catalog["sources"]:
            self.assertTrue(source["url"].startswith("https://"))
            self.assertTrue(source["publisher"])
            self.assertTrue(source["spatial_resolution"])
            self.assertTrue(source["coordinate_reference"])
            self.assertTrue(source["temporal_scope"])
            self.assertTrue(source["uncertainty_and_limitations"])
            self.assertTrue(source["required_acceptance_checks"])

    def test_source_catalog_crosswalk_covers_every_reviewed_candidate(self) -> None:
        expected_ids = {candidate["id"] for candidate in self.canonical["candidates"]}
        crosswalk = self.catalog["sources"][0]["candidate_crosswalk"]
        self.assertEqual({item["candidate_id"] for item in crosswalk}, expected_ids)
        self.assertTrue(all(item["mapping_state"] for item in crosswalk))
        self.assertTrue(
            any(item["mapping_state"] == "no_exact_name_match_validated" for item in crosswalk)
        )

    def test_every_source_uses_structured_crosswalk_states(self) -> None:
        for source in self.catalog["sources"]:
            crosswalk = source["candidate_crosswalk"]
            self.assertIsInstance(crosswalk, list)
            self.assertTrue(crosswalk)
            for entry in crosswalk:
                self.assertTrue(entry["candidate_id"])
                self.assertTrue(entry["mapping_state"])
                self.assertIn("product_site", entry)
                if "note" in entry:
                    self.assertTrue(entry["note"])

    def test_direct_assets_keep_retrieval_state_and_label_links(self) -> None:
        topography = next(
            source for source in self.catalog["sources"]
            if source["id"] == "pgda-lola-high-resolution-site-products"
        )
        illumination = next(
            source for source in self.catalog["sources"]
            if source["id"] == "pgda-lola-polar-illumination"
        )
        self.assertEqual(len(topography["sample_direct_assets"]), 5)
        self.assertEqual(len(illumination["sample_direct_assets"]), 5)
        for source in (topography, illumination):
            self.assertTrue(source["asset_access_notes"])
            for asset in source["sample_direct_assets"]:
                self.assertTrue(asset["url"].startswith("https://"))
                self.assertTrue(asset["asset_type"])
                self.assertTrue(asset["verification_state"])
                self.assertNotIn("downloaded_and_hashed", asset["verification_state"])
        self.assertTrue(
            any("not_downloaded_or_hashed" in a["verification_state"]
                for a in topography["sample_direct_assets"])
        )
        self.assertTrue(
            any(a["url"].endswith("_JP2.LBL") for a in illumination["sample_direct_assets"])
        )

    def test_failed_cog_probe_is_scoped_and_alternative_is_not_overclaimed(self) -> None:
        illumination = next(
            source for source in self.catalog["sources"]
            if source["id"] == "pgda-lola-polar-illumination"
        )
        notes = " ".join(illumination["asset_access_notes"])
        self.assertIn("one sampled URL failure only", notes)
        self.assertIn("could not render the binary or label endpoints", notes)
        self.assertTrue(
            any("listed_in_pds_directory_binary_not_downloaded_or_hashed"
                == asset["verification_state"]
                for asset in illumination["sample_direct_assets"])
        )

    def test_uncertain_region_name_overlaps_are_not_claimed_as_exact(self) -> None:
        crosswalk = self.catalog["sources"][0]["candidate_crosswalk"]
        de_gerlache = next(item for item in crosswalk if item["candidate_id"] == "de-gerlache-rim-2")
        self.assertIn("not_equivalent", de_gerlache["mapping_state"])
        self.assertIsNotNone(de_gerlache["product_site"])

    def test_illumination_products_preserve_time_averaging_limit(self) -> None:
        illumination = next(
            source for source in self.catalog["sources"]
            if source["id"] == "pgda-lola-polar-illumination"
        )
        self.assertIn("18.6-year", illumination["temporal_scope"])
        self.assertTrue(
            any("date-specific" in item for item in illumination["uncertainty_and_limitations"])
        )

    def test_historical_leads_preserve_sources_windows_and_limits(self) -> None:
        leads = self.catalog["historical_leads"]
        self.assertEqual(len(leads), 3)
        connecting_ridge = next(
            lead for lead in leads if lead["id"] == "connecting-ridge-illumination-lead"
        )
        self.assertIn("2024-01-01", connecting_ridge["time_window"])
        self.assertIn("2026-01-01", connecting_ridge["time_window"])
        self.assertEqual(connecting_ridge["source_url"], "https://pgda.gsfc.nasa.gov/products/78")
        self.assertTrue(connecting_ridge["interpretation_limits"])
        self.assertTrue(
            any("not a new measurement" in item for item in connecting_ridge["interpretation_limits"])
        )
        self.assertIn("future crewed mission window", " ".join(connecting_ridge["interpretation_limits"]))


if __name__ == "__main__":
    unittest.main()
