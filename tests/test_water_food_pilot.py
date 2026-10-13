import copy
import json
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts" / "earth-systems"))
from validate_pilot import validate_pilot


class WaterFoodPilotTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.pilot_path = ROOT / "docs" / "pilots" / "baixo-limpopo-water-productivity.json"
        cls.schema_path = ROOT / "docs" / "schemas" / "water-food-pilot.schema.json"
        cls.pilot = json.loads(cls.pilot_path.read_text(encoding="utf-8"))
        cls.schema = json.loads(cls.schema_path.read_text(encoding="utf-8"))

    def test_design_profile_passes_structural_validation(self):
        self.assertEqual(validate_pilot(self.pilot), [])

    def test_source_set_is_exact_and_distinct(self):
        ids = [source["source_id"] for source in self.pilot["sources"]]
        self.assertEqual(set(ids), {"FAO-WAPOR", "CHIRPS3"})
        self.assertEqual(len(ids), len(set(ids)))

    def test_no_artifact_digest_is_claimed_before_acquisition(self):
        doc = copy.deepcopy(self.pilot)
        doc["sources"][0]["artifact_sha256"] = "a" * 64
        self.assertTrue(any("not_acquired source must not claim" in e for e in validate_pilot(doc)))

    def test_acquired_artifact_requires_exact_product_identifier(self):
        doc = copy.deepcopy(self.pilot)
        doc["sources"][0]["acquisition_state"] = "acquired"
        doc["sources"][0]["artifact_sha256"] = "a" * 64
        doc["sources"][0]["source_catalog_verified"] = True
        doc["sources"][0]["selected_product"] = None
        self.assertTrue(any("requires an exact selected_product identifier" in e for e in validate_pilot(doc)))

    def test_acquired_artifact_requires_sha256(self):
        doc = copy.deepcopy(self.pilot)
        doc["sources"][0]["acquisition_state"] = "acquired"
        doc["sources"][0]["source_catalog_verified"] = True
        self.assertTrue(any("requires a 64-character SHA-256" in e for e in validate_pilot(doc)))

    def test_wapor_level_three_correction_history_must_be_checked(self):
        source = next(s for s in self.pilot["sources"] if s["source_id"] == "FAO-WAPOR")
        checks = " ".join(source["required_checks"]).lower()
        self.assertIn("march 2025", checks)
        self.assertIn("reacquire affected layers", checks)

    def test_chirps_station_overlap_must_be_disclosed(self):
        doc = copy.deepcopy(self.pilot)
        source = next(s for s in doc["sources"] if s["source_id"] == "CHIRPS3")
        source["upstream_overlap"] = "Satellite-only source with no overlap."
        self.assertTrue(any("station overlap must be disclosed" in e for e in validate_pilot(doc)))

    def test_validator_rejects_removed_gauge_correction_guidance(self):
        doc = copy.deepcopy(self.pilot)
        source = next(s for s in doc["sources"] if s["source_id"] == "CHIRPS3")
        source["upstream_overlap"] = (
            "CHIRPS blends satellite estimates with station observations. "
            "Check station lineage and exclude contributing gauges from independent tests."
        )
        self.assertTrue(any(
            "gauge-undercatch-correction recommendation" in error
            for error in validate_pilot(doc)
        ))

    def test_chirps_reference_comparison_preserves_gauge_correction_provenance(self):
        source = next(s for s in self.pilot["sources"] if s["source_id"] == "CHIRPS3")
        overlap = source["upstream_overlap"].lower()
        checks = " ".join(source["required_checks"]).lower()
        candidates = " ".join(self.pilot["validation"]["independent_reference_candidates"]).lower()
        self.assertIn("gauge-undercatch-corrected", overlap)
        self.assertIn("correction method/version", checks)
        self.assertIn("gauge-undercatch-corrected observations", candidates)

    def test_unknown_source_is_rejected(self):
        doc = copy.deepcopy(self.pilot)
        doc["sources"][0]["source_id"] = "GENERIC-RAINFALL"
        self.assertTrue(any("not an admitted source" in e for e in validate_pilot(doc)))

    def test_malformed_unhashable_values_are_rejected_without_crashing(self):
        doc = copy.deepcopy(self.pilot)
        doc["sources"][0]["source_id"] = ["FAO-WAPOR"]
        doc["status"] = []
        doc["validation"]["gates"]["D3"] = []
        errors = validate_pilot(doc)
        self.assertTrue(any("source_id is not an admitted source" in e for e in errors))
        self.assertTrue(any("status is not a recognized pilot status" in e for e in errors))
        self.assertTrue(any("validation.gates.D3 is missing or invalid" in e for e in errors))

    def test_latest_chirps_month_observation_is_dated_and_requires_fresh_snapshot(self):
        source = next(s for s in self.pilot["sources"] if s["source_id"] == "CHIRPS3")
        checks = " ".join(source["required_checks"]).lower()
        self.assertIn("2026-10-13", checks)
        self.assertIn("2026-08", checks)
        self.assertIn("2026-09 not listed", checks)
        self.assertIn("not a byte snapshot", checks)
        self.assertIn("fresh raw index", checks)

    def test_chirps_primary_selection_is_monthly_africa_geotiff(self):
        source = next(s for s in self.pilot["sources"] if s["source_id"] == "CHIRPS3")
        self.assertIn("monthly Africa-subset GeoTIFF", source["selected_product"])
        self.assertIn("monthly as the primary alignment", self.pilot["analysis"]["temporal_alignment"])
        checks = " ".join(source["required_checks"]).lower()
        self.assertIn("official monthly africa geotiff directory", checks)
        self.assertIn("daily rnl/sat", checks)

    def test_local_reference_leads_keep_scale_and_status_caveats(self):
        candidates = " ".join(self.pilot["validation"]["independent_reference_candidates"]).lower()
        self.assertIn("inir.gov.mz", candidates)
        self.assertIn("ine.gov.mz", candidates)
        self.assertIn("province-scale context only", candidates)
        self.assertIn("not evidence of data availability", candidates)

    def test_wapor_redistribution_is_fail_closed(self):
        doc = copy.deepcopy(self.pilot)
        doc["sources"][0]["redistribution_allowed"] = True
        self.assertTrue(any("WaPOR rights must remain" in e for e in validate_pilot(doc)))

    def test_baseline_cannot_be_claimed_without_processed_data_and_d2(self):
        doc = copy.deepcopy(self.pilot)
        doc["evidence_state"]["baseline_computed"] = True
        self.assertTrue(any("baseline_computed=true requires" in e for e in validate_pilot(doc)))

    def test_independently_validated_status_requires_d3_and_evidence(self):
        doc = copy.deepcopy(self.pilot)
        doc["status"] = "independently_validated"
        errors = validate_pilot(doc)
        self.assertTrue(any("independently_validated status requires D0-D3=passed" in e for e in errors))
        self.assertTrue(any("independent-validation evidence" in e for e in errors))

    def test_baseline_status_cannot_skip_d1_or_d2(self):
        doc = copy.deepcopy(self.pilot)
        doc["status"] = "descriptive_baseline"
        errors = validate_pilot(doc)
        self.assertTrue(any("descriptive_baseline status requires D0-D2=passed" in e for e in errors))

    def test_decision_support_candidate_requires_all_gates_and_review(self):
        doc = copy.deepcopy(self.pilot)
        doc["status"] = "decision_support_candidate"
        errors = validate_pilot(doc)
        self.assertTrue(any("decision_support_candidate status requires every D0-D5 gate" in e for e in errors))
        self.assertTrue(any("requires source, baseline, validation, review and D5 evidence" in e for e in errors))

    def test_passed_d0_requires_frozen_study_boundary(self):
        doc = copy.deepcopy(self.pilot)
        doc["validation"]["gates"]["D0"] = "passed"
        self.assertTrue(any("D0=passed requires a verified and frozen study boundary" in e for e in validate_pilot(doc)))

    def test_passed_d1_requires_all_source_hashes(self):
        doc = copy.deepcopy(self.pilot)
        doc["validation"]["gates"]["D1"] = "passed"
        self.assertTrue(any("D1=passed requires every admitted source" in e for e in validate_pilot(doc)))

    def test_operational_status_requires_all_evidence_gates(self):
        doc = copy.deepcopy(self.pilot)
        doc["status"] = "operational"
        doc["evidence_state"]["operational_claim_authorized"] = True
        self.assertTrue(any("operational status requires all source digests" in e for e in validate_pilot(doc)))
        self.assertTrue(any("every D0-D5 gate" in e for e in validate_pilot(doc)))

    def test_evidence_flags_require_real_booleans(self):
        doc = copy.deepcopy(self.pilot)
        doc["evidence_state"]["baseline_computed"] = 1
        self.assertTrue(any("baseline_computed must be a boolean" in e for e in validate_pilot(doc)))

    def test_schema_contract_is_draft_2020_12_and_fail_closed(self):
        self.assertIn("2020-12", self.schema["$schema"])
        self.assertTrue(self.schema["additionalProperties"] is False)
        self.assertEqual(self.schema["properties"]["sources"]["maxItems"], 2)

    def test_schema_requires_maturity_prerequisites(self):
        rules = {}
        for rule in self.schema["allOf"]:
            status = rule.get("if", {}).get("properties", {}).get("status", {}).get("const")
            if status:
                rules[status] = rule
        for status in ("data_readiness", "descriptive_baseline", "independently_validated", "decision_support_candidate"):
            self.assertIn(status, rules)
        independent = rules["independently_validated"]["then"]["properties"]["validation"]["properties"]["gates"]["properties"]
        self.assertEqual(independent["D3"]["const"], "passed")
        candidate = rules["decision_support_candidate"]["then"]["properties"]["validation"]["properties"]["gates"]["properties"]
        self.assertTrue(all(candidate[g]["const"] == "passed" for g in ("D0", "D1", "D2", "D3", "D4", "D5")))

    def test_schema_evidence_flags_require_matching_passed_gates(self):
        state_rules = {}
        for rule in self.schema["allOf"]:
            properties = rule.get("if", {}).get("properties", {}).get("evidence_state", {}).get("properties", {})
            for field, definition in properties.items():
                if definition == {"const": True}:
                    state_rules[field] = rule
        for field in ("source_artifacts_downloaded", "observations_processed", "baseline_computed",
                      "independent_validation_complete", "independent_review_complete", "d5_accepted"):
            self.assertIn(field, state_rules)
            self.assertIn("validation", state_rules[field]["then"]["properties"])
        downloaded = state_rules["source_artifacts_downloaded"]["then"]["properties"]["validation"]["properties"]["gates"]["properties"]
        self.assertEqual(downloaded["D1"]["const"], "passed")
        baseline = state_rules["baseline_computed"]["then"]["properties"]["validation"]["properties"]["gates"]["properties"]
        self.assertEqual(baseline["D2"]["const"], "passed")

    def test_schema_makes_d0_depend_on_frozen_boundary(self):
        self.assertTrue(any(
            "D0" in rule.get("if", {}).get("properties", {}).get("validation", {}).get("properties", {}).get("gates", {}).get("properties", {})
            and rule.get("then", {}).get("properties", {}).get("geography", {}).get("properties", {}).get("boundary_state", {}).get("const") == "verified_and_frozen"
            for rule in self.schema["allOf"]
        ))

    def test_invalid_gate_state_is_rejected(self):
        doc = copy.deepcopy(self.pilot)
        doc["validation"]["gates"]["D3"] = "queued"
        self.assertTrue(any("validation.gates.D3 is missing or invalid" in e for e in validate_pilot(doc)))


if __name__ == "__main__":
    unittest.main()

