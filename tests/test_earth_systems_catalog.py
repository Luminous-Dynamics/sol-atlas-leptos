import copy
import json
import sys
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts" / "earth-systems"))
from validate_catalog import validate_catalog


class EarthSystemsCatalogTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.catalog_path = ROOT / "docs" / "earth-systems-domain-catalog.json"
        cls.schema_path = ROOT / "docs" / "schemas" / "earth-systems-domain-catalog.schema.json"
        cls.catalog = json.loads(cls.catalog_path.read_text(encoding="utf-8"))
        cls.schema = json.loads(cls.schema_path.read_text(encoding="utf-8"))

    def test_catalog_passes_structural_validation(self):
        self.assertEqual(validate_catalog(self.catalog), [])

    def test_has_nine_distinct_domains(self):
        ids = [domain["id"] for domain in self.catalog["domains"]]
        self.assertEqual(len(ids), 9)
        self.assertEqual(len(ids), len(set(ids)))

    def test_every_domain_has_independent_validation(self):
        for domain in self.catalog["domains"]:
            with self.subTest(domain=domain["id"]):
                self.assertIs(domain["validation"]["baseline_required"], True)
                self.assertIs(domain["validation"]["independent_review_required"], True)
                self.assertGreaterEqual(len(domain["validation"]["metrics"]), 3)

    def test_every_source_has_limits_and_explicit_rights_state(self):
        for domain in self.catalog["domains"]:
            for source in domain["data_sources"]:
                with self.subTest(domain=domain["id"], source=source["source_id"]):
                    self.assertTrue(source["key_limitations"])
                    self.assertTrue(source["rights_state"])
                    self.assertIn("url", source)

    def test_duplicate_domain_id_is_rejected(self):
        doc = copy.deepcopy(self.catalog)
        doc["domains"][1]["id"] = doc["domains"][0]["id"]
        self.assertTrue(any("duplicate domain id" in error for error in validate_catalog(doc)))

    def test_unknown_or_unreviewed_rights_cannot_claim_redistribution(self):
        doc = copy.deepcopy(self.catalog)
        source = doc["domains"][0]["data_sources"][0]
        source["redistribution_allowed"] = True
        source["rights_state"] = "review_per_artifact"
        self.assertTrue(any("fail-closed rule" in error for error in validate_catalog(doc)))

    def test_unrecognized_rights_state_is_rejected(self):
        doc = copy.deepcopy(self.catalog)
        doc["domains"][0]["data_sources"][0]["rights_state"] = "free-use"
        self.assertTrue(any("not a recognized review state" in error for error in validate_catalog(doc)))

    def test_source_id_cannot_conflict_across_domains(self):
        doc = copy.deepcopy(self.catalog)
        repeated = doc["domains"][1]["data_sources"][0]
        repeated["source_id"] = "SA-MPM-2024"
        repeated["url"] = "https://example.org/wrong-source"
        repeated["provider"] = "Unknown"
        self.assertTrue(any("conflicts with earlier provider/URL" in error for error in validate_catalog(doc)))

    def test_schema_is_draft_2020_12_and_fail_closed(self):
        self.assertIn("2020-12", self.schema["$schema"])
        source_schema = self.schema["properties"]["domains"]["items"]["properties"]["data_sources"]["items"]
        self.assertEqual(source_schema["properties"]["redistribution_allowed"]["type"], ["boolean", "null"])
        self.assertTrue(source_schema["allOf"])

    def test_missing_domain_limitations_are_rejected(self):
        doc = copy.deepcopy(self.catalog)
        del doc["domains"][0]["data_sources"][0]["key_limitations"]
        self.assertTrue(any("key_limitations" in error for error in validate_catalog(doc)))


if __name__ == "__main__":
    unittest.main()
