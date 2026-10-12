"""Negative tests for the lunar site-screening evidence boundary."""
from __future__ import annotations

import copy
import hashlib
import json
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))
CATALOG_DATA = ROOT / "docs/data/lunar_geospatial_source_catalog.v1.json"

from scripts.validate_lunar_site_screening import (  # noqa: E402
    DEFAULT_DATA,
    validate_document,
    validate_source_catalog,
)
from scripts.lunar_asset_receipt import ReceiptError, build_receipt  # noqa: E402


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
        self.assertEqual(len(self.catalog["sources"]), 7)
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

    def test_lroc_connecting_ridge_dtm_is_catalogued_without_verification_overclaim(self) -> None:
        source = next(
            item for item in self.catalog["sources"]
            if item["id"] == "lroc-nac-shackleton-de-gerlache-connecting-ridge-dtm"
        )
        self.assertEqual(
            source["url"],
            "https://data.lroc.im-ldi.com/lroc/view_rdr/NAC_DTM_SHACKRDGE02",
        )
        self.assertIn("3.50 m/pixel", source["spatial_resolution"])
        self.assertTrue(any("confidence" in family.lower() for family in source["product_families"]))
        self.assertEqual(
            source["candidate_crosswalk"][0]["candidate_id"],
            "connecting-ridge",
        )
        self.assertIn(
            "roi_validation_required",
            source["candidate_crosswalk"][0]["mapping_state"],
        )
        self.assertEqual(len(source["sample_direct_assets"]), 5)
        self.assertTrue(all(
            "downloaded_and_hashed" not in asset["verification_state"]
            for asset in source["sample_direct_assets"]
        ))
        self.assertTrue(any(asset["url"].endswith("NAC_DTM_SHACKRDGE02.LBL")
                            for asset in source["sample_direct_assets"]))
        self.assertTrue(any(asset["url"].endswith("NAC_DTM_SHACKRDGE02_CONF.IMG")
                            for asset in source["sample_direct_assets"]))

    def test_global_lroc_dtm_footprint_inventory_is_catalogued_as_inventory_not_candidate_geometry(self) -> None:
        source = next(
            item for item in self.catalog["sources"]
            if item["id"] == "lroc-global-nac-dtm-footprint-shapefiles"
        )
        self.assertEqual(
            source["url"],
            "https://data.lroc.im-ldi.com/lroc/view_rdr/SHAPEFILE_NAC_DTMS",
        )
        self.assertIn("Moon_2000", source["coordinate_reference"])
        self.assertIn("2026-09-15", source["temporal_scope"])
        self.assertTrue(any("candidate-region boundaries" in item
                            for item in source["uncertainty_and_limitations"]))
        self.assertEqual(len(source["sample_direct_assets"]), 3)
        self.assertTrue(any(asset["url"].endswith("NAC_DTMS_180.ZIP")
                            for asset in source["sample_direct_assets"]))
        self.assertTrue(any(asset["url"].endswith("NAC_DTMS_360.ZIP")
                            for asset in source["sample_direct_assets"]))
        self.assertTrue(all(
            "downloaded_and_hashed" not in asset["verification_state"]
            for asset in source["sample_direct_assets"]
        ))

    def test_usgs_candidate_navigation_grids_do_not_overclaim_candidate_boundaries(self) -> None:
        source = next(
            item for item in self.catalog["sources"]
            if item["id"] == "usgs-artemis-iii-candidate-navigation-grids"
        )
        self.assertEqual(
            source["url"],
            "https://www.usgs.gov/data/down-selected-artemis-iii-candidate-landing-site-navigational-grids",
        )
        self.assertIn("1 km", source["spatial_resolution"])
        self.assertIn("JPL DE421", source["coordinate_reference"])
        self.assertEqual(len(source["candidate_crosswalk"]), 9)
        self.assertEqual(
            {entry["candidate_id"] for entry in source["candidate_crosswalk"]},
            {
                "peak-near-cabeus-b", "haworth", "malapert-massif",
                "mons-mouton-plateau", "mons-mouton", "nobile-rim-1",
                "nobile-rim-2", "de-gerlache-rim-2", "slater-plain",
            },
        )
        self.assertTrue(all(
            "boundary_not_validated" in entry["mapping_state"]
            for entry in source["candidate_crosswalk"]
        ))
        self.assertEqual(source["sample_direct_assets"][0]["url"], "https://doi.org/10.5066/P1MEQ6UK")
        self.assertIn("not a direct archive URL", source["sample_direct_assets"][0]["asset_type"])
        self.assertNotIn("downloaded_and_hashed", source["sample_direct_assets"][0]["verification_state"])

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

    def test_canonical_source_catalog_passes_fail_closed_validator(self) -> None:
        self.assertEqual([], validate_source_catalog(self.catalog, self.canonical))

    def test_source_catalog_rejects_unverified_asset_claim_without_hash(self) -> None:
        catalog = copy.deepcopy(self.catalog)
        illumination = next(
            source for source in catalog["sources"]
            if source["id"] == "pgda-lola-polar-illumination"
        )
        illumination["sample_direct_assets"][0]["verification_state"] = "downloaded_and_hashed"
        errors = validate_source_catalog(catalog, self.canonical)
        self.assertTrue(any("requires a 64-character SHA-256" in error for error in errors))

    def test_source_catalog_rejects_incomplete_primary_crosswalk(self) -> None:
        catalog = copy.deepcopy(self.catalog)
        catalog["sources"][0]["candidate_crosswalk"].pop()
        errors = validate_source_catalog(catalog, self.canonical)
        self.assertTrue(any("primary LOLA crosswalk must match" in error for error in errors))

    def test_source_catalog_rejects_non_https_asset(self) -> None:
        catalog = copy.deepcopy(self.catalog)
        topography = catalog["sources"][0]
        topography["sample_direct_assets"][0]["url"] = "http://example.invalid/file.tif"
        errors = validate_source_catalog(catalog, self.canonical)
        self.assertTrue(any("sample_direct_assets[0].url must be an HTTPS URL" in error for error in errors))

    def test_asset_receipt_hashes_local_bytes_without_claiming_origin(self) -> None:
        asset_url = next(
            asset["url"]
            for source in self.catalog["sources"]
            if source["id"] == "pgda-lola-polar-illumination"
            for asset in source["sample_direct_assets"]
            if asset["url"].endswith("_JP2.LBL")
        )
        payload = b"PDS_LABEL_TEST_BYTES\\n"
        with tempfile.TemporaryDirectory() as directory:
            local_asset = Path(directory) / "sample.lbl"
            local_asset.write_bytes(payload)
            receipt = build_receipt(
                self.catalog,
                CATALOG_DATA,
                local_asset,
                asset_url,
                captured_utc="2026-10-12T08:00:00Z",
            )
        self.assertEqual(receipt["asset"]["sha256"], hashlib.sha256(payload).hexdigest())
        self.assertEqual(receipt["asset"]["byte_count"], len(payload))
        self.assertTrue(receipt["verification_boundary"]["local_bytes_hashed"])
        self.assertFalse(receipt["verification_boundary"]["url_origin_authenticated"])
        self.assertFalse(receipt["verification_boundary"]["pds_label_parsed"])
        self.assertFalse(receipt["verification_boundary"]["site_metrics_computed"])

    def test_asset_receipt_rejects_symlink_input(self) -> None:
        asset_url = next(
            asset["url"]
            for source in self.catalog["sources"]
            for asset in source.get("sample_direct_assets", [])
            if asset["url"].endswith("_JP2.LBL")
        )
        with tempfile.TemporaryDirectory() as directory:
            local_asset = Path(directory) / "source.lbl"
            linked_asset = Path(directory) / "symlink.lbl"
            local_asset.write_bytes(b"local test bytes")
            linked_asset.symlink_to(local_asset)
            with self.assertRaisesRegex(ReceiptError, "symbolic-link inputs are not accepted"):
                build_receipt(
                    self.catalog,
                    CATALOG_DATA,
                    linked_asset,
                    asset_url,
                    captured_utc="2026-10-12T08:00:00Z",
                )

    def test_asset_receipt_rejects_catalog_changed_after_parse(self) -> None:
        altered_catalog = copy.deepcopy(self.catalog)
        altered_catalog["decision_rule"] = "modified in-memory document"
        asset_url = next(
            asset["url"]
            for source in self.catalog["sources"]
            for asset in source.get("sample_direct_assets", [])
            if asset["url"].endswith("_JP2.LBL")
        )
        with tempfile.TemporaryDirectory() as directory:
            local_asset = Path(directory) / "sample.lbl"
            local_asset.write_bytes(b"some bytes")
            with self.assertRaisesRegex(ReceiptError, "changed after parsing"):
                build_receipt(
                    altered_catalog,
                    CATALOG_DATA,
                    local_asset,
                    asset_url,
                    captured_utc="2026-10-12T08:00:00Z",
                )

    def test_asset_receipt_rejects_unlisted_url(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            local_asset = Path(directory) / "sample.tif"
            local_asset.write_bytes(b"some bytes")
            with self.assertRaises(ReceiptError):
                build_receipt(
                    self.catalog,
                    CATALOG_DATA,
                    local_asset,
                    "https://example.invalid/unreviewed.tif",
                    captured_utc="2026-10-12T08:00:00Z",
                )

    def test_asset_receipt_refuses_known_broken_cog_url(self) -> None:
        broken_url = next(
            asset["url"]
            for source in self.catalog["sources"]
            for asset in source.get("sample_direct_assets", [])
            if "404" in asset["verification_state"]
        )
        with tempfile.TemporaryDirectory() as directory:
            local_asset = Path(directory) / "wrong-source.tif"
            local_asset.write_bytes(b"some bytes")
            with self.assertRaisesRegex(ReceiptError, "returned 404"):
                build_receipt(
                    self.catalog,
                    CATALOG_DATA,
                    local_asset,
                    broken_url,
                    captured_utc="2026-10-12T08:00:00Z",
                )

    def test_direct_assets_keep_retrieval_state_and_label_links(self) -> None:
        topography = next(
            source for source in self.catalog["sources"]
            if source["id"] == "pgda-lola-high-resolution-site-products"
        )
        illumination = next(
            source for source in self.catalog["sources"]
            if source["id"] == "pgda-lola-polar-illumination"
        )
        self.assertEqual(len(topography["sample_direct_assets"]), 6)
        self.assertEqual(len(illumination["sample_direct_assets"]), 7)
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
