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

    def test_acquired_artifact_requires_sha256(self):
        doc = copy.deepcopy(self.pilot)
        doc["sources"][0]["acquisition_state"] = "acquired"
        doc["sources"][0]["source_catalog_verified"] = True
        self.assertTrue(any("requires a 64-character SHA-256" in e for e in validate_pilot(doc)))

    def test_chirps_station_overlap_must_be_disclosed(self):
        doc = copy.deepcopy(self.pilot)
        source = next(s for s in doc["sources"] if s["source_id"] == "CHIRPS3")
        source["upstream_overlap"] = "Satellite-only source with no overlap."
        self.assertTrue(any("station overlap must be disclosed" in e for e in validate_pilot(doc)))

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

    def test_wapor_redistribution_is_fail_closed(self):
        doc = copy.deepcopy(self.pilot)
        doc["sources"][0]["redistribution_allowed"] = True
        self.assertTrue(any("WaPOR rights must remain" in e for e in validate_pilot(doc)))

    def test_baseline_cannot_be_claimed_without_processed_data_and_d2(self):
        doc = copy.deepcopy(self.pilot)
        doc["evidence_state"]["baseline_computed"] = True
        self.assertTrue(any("baseline_computed=true requires" in e for e in validate_pilot(doc)))

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

    def test_invalid_gate_state_is_rejected(self):
        doc = copy.deepcopy(self.pilot)
        doc["validation"]["gates"]["D3"] = "queued"
        self.assertTrue(any("validation.gates.D3 is missing or invalid" in e for e in validate_pilot(doc)))


if __name__ == "__main__":
    unittest.main()

