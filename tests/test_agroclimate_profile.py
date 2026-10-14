import copy
import json
import sys
import unittest
from pathlib import Path

from jsonschema import Draft202012Validator

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT / "scripts" / "earth-systems"))
from validate_agroclimate_profile import validate_profile


class AgroclimateProfileTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.profile_path = ROOT / "docs" / "pilots" / "johannesburg-highveld-food-resilience.json"
        cls.schema_path = ROOT / "docs" / "schemas" / "agroclimate-model-profile.schema.json"
        cls.profile = json.loads(cls.profile_path.read_text(encoding="utf-8"))
        cls.schema = json.loads(cls.schema_path.read_text(encoding="utf-8"))

    def errors(self, doc):
        return validate_profile(doc, self.schema)

    def test_design_profile_passes_structural_and_semantic_validation(self):
        self.assertEqual(self.errors(self.profile), [])

    def test_schema_is_draft_2020_12_and_fail_closed(self):
        self.assertEqual(self.schema["$schema"], "https://json-schema.org/draft/2020-12/schema")
        self.assertIs(self.schema["additionalProperties"], False)
        Draft202012Validator.check_schema(self.schema)

    def test_exact_six_source_families_are_distinct(self):
        ids = [source["source_id"] for source in self.profile["sources"]]
        self.assertEqual(len(ids), 6)
        self.assertEqual(len(set(ids)), 6)

    def test_shared_schema_accepts_other_regions_and_source_families(self):
        doc = copy.deepcopy(self.profile)
        doc["profile_id"] = "agroclimate-mediterranean-v1"
        doc["geography"]["name"] = "Mediterranean candidate study region"
        doc["geography"]["country"] = "Spain"
        for index, source in enumerate(doc["sources"], start=1):
            source["source_id"] = f"GLOBAL-CLIMATE-SOURCE-{index}"
            source["rights_state"] = "license_not_reviewed"
        schema_errors = list(Draft202012Validator(self.schema).iter_errors(doc))
        self.assertEqual(
            schema_errors,
            [],
            "the reusable schema should permit another regional profile; the Johannesburg validator pins the pilot's exact source set",
        )

    def test_profile_does_not_claim_source_artifacts_are_acquired(self):
        self.assertFalse(self.profile["evidence_state"]["source_artifacts_acquired"])
        self.assertTrue(all(source["acquisition_state"] == "not_acquired" for source in self.profile["sources"]))
        self.assertTrue(all(source["artifact_sha256"] is None for source in self.profile["sources"]))

    def test_unacquired_source_cannot_claim_digest(self):
        doc = copy.deepcopy(self.profile)
        doc["sources"][0]["artifact_sha256"] = "a" * 64
        self.assertTrue(any("cannot claim an artifact SHA-256" in error for error in self.errors(doc)))

    def test_acquired_source_requires_exact_product_digest_and_catalog_verification(self):
        doc = copy.deepcopy(self.profile)
        source = doc["sources"][0]
        source["acquisition_state"] = "acquired"
        source["selected_product"] = None
        source["artifact_sha256"] = None
        self.assertTrue(any("acquired source requires" in error for error in self.errors(doc)))

    def test_external_source_urls_must_be_https(self):
        doc = copy.deepcopy(self.profile)
        doc["sources"][0]["url"] = "http://example.invalid/data"
        self.assertTrue(any("must be an HTTPS URL" in error for error in self.errors(doc)))

    def test_shared_schema_rejects_non_https_source_urls(self):
        doc = copy.deepcopy(self.profile)
        doc["sources"][0]["url"] = "http://example.invalid/data"
        schema_errors = list(Draft202012Validator(self.schema).iter_errors(doc))
        self.assertTrue(any(list(error.path)[-1:] == ["url"] for error in schema_errors))

    def test_local_observation_lead_does_not_invent_a_url(self):
        doc = copy.deepcopy(self.profile)
        source = next(item for item in doc["sources"] if item["source_id"] == "LOCAL-OBSERVATIONS")
        source["url"] = "https://example.invalid/farm-data"
        self.assertTrue(any("must not invent a public URL" in error for error in self.errors(doc)))

    def test_local_observations_require_permission_review(self):
        doc = copy.deepcopy(self.profile)
        source = next(item for item in doc["sources"] if item["source_id"] == "LOCAL-OBSERVATIONS")
        source["rights_state"] = "cc_by_4_0_with_attribution"
        self.assertTrue(any("local observations must retain permission_required" in error for error in self.errors(doc)))

    def test_soilgrids_farm_scale_limit_is_preserved(self):
        doc = copy.deepcopy(self.profile)
        source = next(item for item in doc["sources"] if item["source_id"] == "SOILGRIDS-V2")
        source["native_spatial_support"] = "250m accurate map that is ground truth for each farm."
        self.assertTrue(any("farm-level limitation" in error for error in self.errors(doc)))

    def test_soilgrids_paused_rest_api_and_alternate_path_must_be_checked(self):
        doc = copy.deepcopy(self.profile)
        source = next(item for item in doc["sources"] if item["source_id"] == "SOILGRIDS-V2")
        source["required_checks"] = [item for item in source["required_checks"] if "REST API" not in item]
        self.assertTrue(any("paused REST API" in error for error in self.errors(doc)))

    def test_era5land_native_and_distributed_grid_support_must_be_disclosed(self):
        doc = copy.deepcopy(self.profile)
        source = next(item for item in doc["sources"] if item["source_id"] == "ERA5-LAND")
        source["native_spatial_support"] = "Very high-resolution local weather observations."
        self.assertTrue(any("native ~9 km support" in error for error in self.errors(doc)))

    def test_chirps_station_overlap_must_be_disclosed(self):
        doc = copy.deepcopy(self.profile)
        source = next(item for item in doc["sources"] if item["source_id"] == "CHIRPS-V3")
        source["upstream_overlap"] = "Satellite-only rainfall measurements."
        self.assertTrue(any("station observations" in error for error in self.errors(doc)))

    def test_chirps_fresh_index_snapshot_and_hash_are_required(self):
        doc = copy.deepcopy(self.profile)
        source = next(item for item in doc["sources"] if item["source_id"] == "CHIRPS-V3")
        source["required_checks"] = [item for item in source["required_checks"] if "index" not in item.lower()]
        self.assertTrue(any("fresh official index snapshot" in error for error in self.errors(doc)))

    def test_saws_national_summary_cannot_be_promoted_to_farm_forecast(self):
        doc = copy.deepcopy(self.profile)
        source = next(item for item in doc["sources"] if item["source_id"] == "SAWS-SEASONAL")
        source["native_spatial_support"] = "Exact deterministic forecast for every Johannesburg farm."
        self.assertTrue(any("farm-scale forecast" in error for error in self.errors(doc)))

    def test_saws_original_bulletin_is_a_required_evidence_item(self):
        doc = copy.deepcopy(self.profile)
        source = next(item for item in doc["sources"] if item["source_id"] == "SAWS-SEASONAL")
        source["required_checks"] = [item for item in source["required_checks"] if "bulletin" not in item.lower()]
        self.assertTrue(any("original bulletin" in error for error in self.errors(doc)))

    def test_global_climate_archetypes_must_not_be_independent_black_box_models(self):
        doc = copy.deepcopy(self.profile)
        doc["model_architecture"]["climate_archetypes_are_labels_only"] = False
        self.assertTrue(self.errors(doc))

    def test_cross_region_transfer_penalty_and_calibration_benefit_are_evaluated(self):
        requirements = " ".join(self.profile["evaluation"]["validation_design"]).lower()
        self.assertIn("unadapted shared model", requirements)
        self.assertIn("local calibration", requirements)
        self.assertIn("transfer penalty", requirements)
        self.assertIn("calibration benefit", requirements)

    def test_current_season_signal_is_not_a_verified_johannesburg_forecast(self):
        saws = next(item for item in self.profile["sources"] if item["source_id"] == "SAWS-SEASONAL")
        self.assertEqual(saws["acquisition_state"], "not_acquired")
        self.assertIsNone(saws["artifact_sha256"])
        self.assertIn("not assume", saws["native_spatial_support"].lower())
        self.assertTrue(any("not a deterministic Johannesburg prediction" in claim for claim in self.profile["claim_ceiling"]))

    def test_design_only_requires_all_gates_not_started(self):
        doc = copy.deepcopy(self.profile)
        doc["validation"]["gates"]["D0"] = "passed"
        self.assertTrue(any("all D0-D5 gates to remain not_started" in error for error in self.errors(doc)))

    def test_D0_requires_a_verified_frozen_boundary(self):
        doc = copy.deepcopy(self.profile)
        doc["status"] = "data_readiness"
        doc["validation"]["gates"]["D0"] = "passed"
        self.assertTrue(any("D0=passed requires an authoritative" in error for error in self.errors(doc)))

    def test_D0_can_pass_when_boundary_is_verified_and_frozen(self):
        doc = copy.deepcopy(self.profile)
        doc["status"] = "data_readiness"
        doc["geography"]["boundary_state"] = "verified_and_frozen"
        doc["geography"]["boundary_source"] = "Authoritative municipal boundary artifact reviewed and hash recorded."
        doc["validation"]["gates"]["D0"] = "passed"
        self.assertFalse(self.errors(doc))

    def test_D1_cannot_pass_with_unacquired_sources(self):
        doc = copy.deepcopy(self.profile)
        doc["status"] = "data_readiness"
        doc["validation"]["gates"]["D1"] = "passed"
        self.assertTrue(any("D1=passed requires acquisition" in error for error in self.errors(doc)))

    def test_D2_cannot_pass_before_a_baseline_exists(self):
        doc = copy.deepcopy(self.profile)
        doc["status"] = "data_readiness"
        doc["validation"]["gates"]["D2"] = "passed"
        self.assertTrue(any("D2=passed requires baseline_computed=true" in error for error in self.errors(doc)))

    def test_D3_cannot_pass_before_independent_validation(self):
        doc = copy.deepcopy(self.profile)
        doc["status"] = "data_readiness"
        doc["validation"]["gates"]["D3"] = "passed"
        self.assertTrue(any("D3=passed requires independent_validation_complete=true" in error for error in self.errors(doc)))

    def test_D4_cannot_pass_before_expert_review(self):
        doc = copy.deepcopy(self.profile)
        doc["status"] = "data_readiness"
        doc["validation"]["gates"]["D4"] = "passed"
        self.assertTrue(any("D4=passed requires expert_review_complete=true" in error for error in self.errors(doc)))

    def test_D5_cannot_pass_without_prior_gates_and_authority(self):
        doc = copy.deepcopy(self.profile)
        doc["status"] = "data_readiness"
        doc["validation"]["gates"]["D5"] = "passed"
        errors = self.errors(doc)
        self.assertTrue(any("D5=passed requires D0-D4=passed" in error for error in errors))
        self.assertTrue(any("D5=passed requires decision_support_authorized=true" in error for error in errors))

    def test_baseline_requires_D0_through_D2(self):
        doc = copy.deepcopy(self.profile)
        doc["status"] = "descriptive_baseline"
        doc["evidence_state"]["baseline_computed"] = True
        doc["evidence_state"]["source_artifacts_acquired"] = True
        self.assertTrue(any("baseline_computed=true requires D0-D2=passed" in error for error in self.errors(doc)))

    def test_independent_validation_cannot_skip_baseline_or_D3(self):
        doc = copy.deepcopy(self.profile)
        doc["evidence_state"]["independent_validation_complete"] = True
        self.assertTrue(any("requires baseline and D3=passed" in error for error in self.errors(doc)))

    def test_decision_support_requires_every_gate_and_expert_review(self):
        doc = copy.deepcopy(self.profile)
        doc["evidence_state"]["decision_support_authorized"] = True
        self.assertTrue(any("requires all D0-D5 gates passed" in error for error in self.errors(doc)))
        self.assertTrue(any("requires independent validation and expert review" in error for error in self.errors(doc)))

    def test_invalid_candidate_period_order_is_rejected(self):
        doc = copy.deepcopy(self.profile)
        doc["candidate_reference_periods"][0]["start_year"] = 2000
        doc["candidate_reference_periods"][0]["end_year"] = 1981
        self.assertTrue(any("ends before it starts" in error for error in self.errors(doc)))

    def test_candidate_period_ids_must_be_unique(self):
        doc = copy.deepcopy(self.profile)
        doc["candidate_reference_periods"][1]["period_id"] = doc["candidate_reference_periods"][0]["period_id"]
        self.assertTrue(any("duplicate candidate period_id" in error for error in self.errors(doc)))

    def test_missing_local_evidence_cannot_be_replaced_by_soilgrids_alone(self):
        local = next(item for item in self.profile["sources"] if item["source_id"] == "LOCAL-OBSERVATIONS")
        self.assertEqual(local["acquisition_state"], "not_acquired")
        self.assertTrue(any("field-representative soil measurements" in item.lower() for item in self.profile["evaluation"]["minimum_local_evidence"]))
        self.assertTrue(any("field-representative soil measurements" in item.lower() for item in self.profile["evaluation"]["minimum_local_evidence"]))

    def test_exact_crops_and_model_layers_remain_unselected(self):
        gaez = next(item for item in self.profile["sources"] if item["source_id"] == "FAO-GAEZ-V5")
        self.assertIsNone(gaez["selected_product"])
        self.assertEqual(self.profile["model_architecture"]["model_selection_state"], "not_started")
        self.assertTrue(any("no crop yield" in claim.lower() for claim in self.profile["claim_ceiling"]))


if __name__ == "__main__":
    unittest.main()
