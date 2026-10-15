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
        cls.case_path = ROOT / "docs" / "pilots" / "black-sea-grain-initiative-readiness.json"
        cls.case_profile = json.loads(cls.case_path.read_text(encoding="utf-8"))
        cls.trade_plan_path = ROOT / "docs" / "pilots" / "black-sea-grain-trade-query-plan.json"
        cls.trade_plan = json.loads(cls.trade_plan_path.read_text(encoding="utf-8"))

    def test_catalog_passes_structural_validation(self):
        self.assertEqual(validate_catalog(self.catalog), [])

    def test_has_ten_distinct_domains(self):
        ids = [domain["id"] for domain in self.catalog["domains"]]
        self.assertEqual(len(ids), 10)
        self.assertEqual(len(ids), len(set(ids)))

    def test_conflict_supply_chain_domain_is_registered_as_candidate_research(self):
        domain = next(
            d for d in self.catalog["domains"]
            if d["id"] == "global-conflict-and-supply-chain-resilience"
        )
        self.assertEqual(domain["readiness"], "candidate")
        self.assertEqual(domain["priority"], "P1")
        source_ids = {source["source_id"] for source in domain["data_sources"]}
        self.assertTrue({
            "UCDP-GED", "ACLED-API", "GDELT-2", "IMF-PORTWATCH",
            "UN-COMTRADE", "HDX-HAPI", "OPEN-SUPPLY-HUB", "GS1-EPCIS-2",
        }.issubset(source_ids))
        self.assertIs(domain["validation"]["baseline_required"], True)
        self.assertIs(domain["validation"]["independent_review_required"], True)

    def test_conflict_supply_chain_sources_do_not_overclaim_rights_or_causality(self):
        domain = next(
            d for d in self.catalog["domains"]
            if d["id"] == "global-conflict-and-supply-chain-resilience"
        )
        sources = {source["source_id"]: source for source in domain["data_sources"]}
        self.assertEqual(sources["UCDP-GED"]["rights_state"], "cc_by_4_0_with_attribution")
        self.assertIs(sources["UCDP-GED"]["redistribution_allowed"], True)
        for source_id in ("ACLED-API", "IMF-PORTWATCH", "UN-COMTRADE", "HDX-HAPI", "OPEN-SUPPLY-HUB", "GS1-EPCIS-2"):
            with self.subTest(source=source_id):
                self.assertIsNone(sources[source_id]["redistribution_allowed"])
        limitations = " ".join(
            limit for source in domain["data_sources"] for limit in source["key_limitations"]
        ).lower()
        self.assertIn("not an independently verified fact", limitations)
        self.assertIn("not bills of lading", limitations)
        self.assertIn("does not establish", limitations)
        self.assertIn("not a public global shipment database", limitations)

    def test_black_sea_candidate_is_source_readiness_only(self):
        profile = self.case_profile
        self.assertEqual(profile["schema_version"], "luminous.sol-atlas.conflict-case-readiness.v1")
        self.assertEqual(profile["case_id"], "black-sea-grain-initiative-2022-2023")
        self.assertEqual(profile["status"], "design_only")
        self.assertEqual(profile["readiness_stage"], "C0_source_readiness")
        self.assertEqual(profile["case_selection"]["disposition"], "selected_for_source_readiness_review_only")
        self.assertFalse(profile["case_selection"]["analysis_window_frozen"])
        self.assertFalse(profile["published_source_findings"]["exact_source_bytes_acquired"])
        self.assertIsNone(profile["published_source_findings"]["exact_source_sha256"])
        self.assertTrue(all(value is False for value in profile["validation_gates"].values()))

    def test_transcribed_black_sea_months_reconcile_without_claiming_source_capture(self):
        findings = self.case_profile["published_source_findings"]
        monthly = findings["monthly_exports"]
        self.assertEqual(len(monthly), 12)
        self.assertEqual(sum(row["metric_tonnes"] for row in monthly), findings["reported_total_metric_tonnes_through_2023_07_14"])
        self.assertEqual(findings["reported_total_metric_tonnes_through_2023_07_14"], 32817650)
        self.assertEqual(findings["reported_voyages_through_2023_07_14"], 1003)
        self.assertTrue(monthly[-1]["partial_period"])
        self.assertEqual(monthly[-1]["period_end_date"], "2023-07-14")
        self.assertEqual(monthly[-1]["month"], "2023-07")
        self.assertTrue(all(not row["partial_period"] for row in monthly[:-1]))
        self.assertFalse(findings["exact_source_bytes_acquired"])
        self.assertIsNone(findings["exact_source_sha256"])

    def test_black_sea_case_sources_are_registered_with_unresolved_rights(self):
        domain = next(
            d for d in self.catalog["domains"]
            if d["id"] == "global-conflict-and-supply-chain-resilience"
        )
        sources = {source["source_id"]: source for source in domain["data_sources"]}
        profile_sources = {source["source_id"]: source for source in self.case_profile["candidate_sources"]}
        for source_id in ("UN-BSGI-JCC", "FAO-FFPI"):
            with self.subTest(source=source_id):
                self.assertIn(source_id, sources)
                self.assertIn(source_id, profile_sources)
                self.assertIsNone(sources[source_id]["redistribution_allowed"])
                self.assertFalse(profile_sources[source_id]["artifact_state"].endswith("captured"))

    def test_black_sea_case_forbids_zero_imputation_and_causal_overclaim(self):
        forbidden = " ".join(self.case_profile["prohibited_claims"]).lower()
        self.assertIn("zero ukrainian grain exports", forbidden)
        self.assertIn("causally", forbidden)
        self.assertIn("complete monthly period", forbidden)
        self.assertIn("exact source bytes", forbidden)


    def test_comtrade_query_plan_is_not_misrepresented_as_executed(self):
        plan = self.trade_plan
        self.assertEqual(plan["schema_version"], "luminous.sol-atlas.un-comtrade-query-plan.v1")
        self.assertEqual(plan["status"], "query_plan_only")
        self.assertEqual(plan["execution_state"], "not_run")
        self.assertFalse(plan["analysis_window_frozen"])
        self.assertFalse(plan["source"]["raw_response_bytes_acquired"])
        self.assertIsNone(plan["source"]["source_sha256"])
        self.assertTrue(all(value is False for value in plan["validation_gates"].values()))

    def test_comtrade_chunks_cover_each_month_once_without_oversized_requests(self):
        plan = self.trade_plan
        chunks = plan["request_chunks"]
        pairs = [(chunk["periods"][0], chunk["commodity_code"]) for chunk in chunks]
        expected = [
            (f"{year}{month:02d}", code)
            for year in range(2017, 2024)
            for month in range(1, 13)
            for code in self.trade_plan["query_dimensions"]["commodity_codes"]
        ]
        self.assertEqual(len(chunks), 168)
        self.assertTrue(all(len(chunk["periods"]) == 1 for chunk in chunks))
        self.assertEqual(pairs, expected)
        self.assertEqual(len(pairs), len(set(pairs)))
        self.assertTrue(all(chunk["commodity_code"] in {"1001", "1005"} for chunk in chunks))
        self.assertEqual(self.trade_plan["request_policy"]["minimum_seconds_between_comtrade_requests"], 1.0)
        self.assertIs(self.trade_plan["request_policy"]["no_parallel_comtrade_requests"], True)

    def test_comtrade_query_dimensions_are_explicit_and_fail_closed_on_classification(self):
        query = self.trade_plan["query_dimensions"]
        self.assertEqual(query["type_code"], "C")
        self.assertEqual(query["frequency_code"], "M")
        self.assertEqual(query["reporter_code"], 804)
        self.assertEqual(query["partner_code"], 0)
        self.assertEqual(query["flow_code"], "X")
        self.assertEqual(query["commodity_codes"], ["1001", "1005"])
        self.assertIn("do not stitch", query["classification_policy"].lower())
        self.assertTrue(self.trade_plan["fields_policy"]["preserve_all_returned_fields"])
        self.assertTrue(self.trade_plan["fields_policy"]["no_implicit_unit_conversion"])

    def test_comtrade_preview_truncation_and_source_bytes_are_explicit_gates(self):
        policy = self.trade_plan["request_policy"]
        self.assertEqual(policy["public_preview_record_limit"], 500)
        self.assertTrue(policy["fail_closed_if_count_exceeds_returned_rows"])
        self.assertTrue(policy["fail_closed_if_returned_rows_reach_preview_limit"])
        self.assertTrue(policy["preserve_raw_response_bytes_before_parsing"])
        self.assertTrue(policy["store_raw_sha256_and_retrieval_timestamp"])
        ceiling = " ".join(self.trade_plan["interpretation_ceiling"]["not_established"]).lower()
        self.assertIn("individual shipments", ceiling)
        self.assertIn("physical routes", ceiling)
        self.assertEqual(self.trade_plan["control_design"]["state"], "not_preregistered")
        self.assertEqual(self.trade_plan["control_design"]["selected_controls"], [])

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

    def test_redistribution_flag_must_be_boolean_or_null(self):
        doc = copy.deepcopy(self.catalog)
        doc["domains"][0]["data_sources"][0]["redistribution_allowed"] = 1
        self.assertTrue(any("must be true, false or null" in error for error in validate_catalog(doc)))

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

    def test_chirps_v3_has_version_and_station_lineage_caveats_in_both_domains(self):
        for domain_id in ("water-security", "food-land-water-productivity"):
            with self.subTest(domain=domain_id):
                domain = next(d for d in self.catalog["domains"] if d["id"] == domain_id)
                source = next(s for s in domain["data_sources"] if s["source_id"] == "CHIRPS3")
                self.assertEqual(source["rights_state"], "cc_by_4_0_with_attribution")
                self.assertIs(source["redistribution_allowed"], True)
                limitations = " ".join(source["key_limitations"]).lower()
                self.assertIn("preliminary and final", limitations)
                self.assertIn("station", limitations)
                self.assertIn("gauge-undercatch-corrected", limitations)
                self.assertIn("legates-willmott", limitations)
                self.assertIn("not independent station observations", limitations)

    def test_wapor_level_three_correction_history_is_a_catalogued_limitation(self):
        for domain_id in ("water-security", "food-land-water-productivity"):
            with self.subTest(domain=domain_id):
                domain = next(d for d in self.catalog["domains"] if d["id"] == domain_id)
                source = next(s for s in domain["data_sources"] if s["source_id"] == "FAO-WAPOR")
                limitations = " ".join(source["key_limitations"]).lower()
                self.assertIn("march 2025", limitations)
                self.assertIn("correction-vintage review", limitations)
                self.assertIn("not an exhaustive level iii area inventory", limitations)
                self.assertIn("mosaicset indexes", limitations)
                self.assertIn("baixo limpopo: mbl", limitations)

    def test_malformed_enum_and_url_values_fail_closed_without_crashing(self):
        doc = copy.deepcopy(self.catalog)
        domain = doc["domains"][0]
        domain["priority"] = []
        domain["readiness"] = {}
        source = domain["data_sources"][0]
        source["access_status"] = []
        source["rights_state"] = {}
        source["url"] = "https://[malformed"
        errors = validate_catalog(doc)
        self.assertTrue(any(".priority is invalid" in e for e in errors))
        self.assertTrue(any(".readiness is invalid" in e for e in errors))
        self.assertTrue(any(".access_status is invalid" in e for e in errors))
        self.assertTrue(any("not a recognized review state" in e for e in errors))
        self.assertTrue(any(".url must be an HTTPS URL" in e for e in errors))

    def test_chirps_catalogue_discloses_monthly_primary_and_daily_downscaling(self):
        for domain_id in ("water-security", "food-land-water-productivity"):
            with self.subTest(domain=domain_id):
                domain = next(d for d in self.catalog["domains"] if d["id"] == domain_id)
                source = next(s for s in domain["data_sources"] if s["source_id"] == "CHIRPS3")
                limitations = " ".join(source["key_limitations"]).lower()
                self.assertIn("fundamental chirps v3 timescales", limitations)
                self.assertIn("daily rnl/sat", limitations)
                self.assertIn("0.05-degree", limitations)
                self.assertIn("not be resampled to a finer grid", limitations)

    def test_missing_domain_limitations_are_rejected(self):
        doc = copy.deepcopy(self.catalog)
        del doc["domains"][0]["data_sources"][0]["key_limitations"]
        self.assertTrue(any("key_limitations" in error for error in validate_catalog(doc)))


if __name__ == "__main__":
    unittest.main()
