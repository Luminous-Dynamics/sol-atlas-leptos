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
            if source["url"] is None:
                source["url"] = f"https://example.invalid/local-evidence-{index}"
        doc["candidate_crop_cohorts"] = [{
            "cohort_id": "mediterranean-baseline-crops",
            "crop_names": ["wheat", "tomato"],
            "selection_state": "candidate_not_selected",
            "rationale": "Illustrative region-specific crop shortlist for a schema portability test; no agronomic decision is implied.",
            "review_requirements": [
                "Review region-specific thermal and seasonal requirements.",
                "Obtain local validation data before selection."
            ],
            "claim_limit": "This is a schema fixture and is not an agronomic recommendation."
        }]
        doc["context_references"] = [{
            "reference_id": "GLOBAL-REGION-CONTEXT-1",
            "title": "Example regional agricultural context source",
            "url": "https://example.invalid/regional-agriculture",
            "reference_type": "research_reference",
            "role": "Illustrative region-specific source role for reusable schema validation.",
            "limitations": [
                "This is a synthetic schema fixture, not evidence of real data availability."
            ]
        }]
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

    def test_five_site_constraints_are_explicit_and_not_assessed_at_design_stage(self):
        required = {
            "irrigation_water_quantity_and_quality",
            "soil_condition_and_contamination",
            "land_access_and_permission",
            "postharvest_storage_energy_and_transport",
            "market_nutrition_and_affordability",
            "regional_supply_and_contingency",
        }
        self.assertEqual(set(self.profile["site_constraints"]), required)
        self.assertTrue(all(
            item["state"] == "not_assessed"
            for item in self.profile["site_constraints"].values()
        ))
        self.assertTrue(all(
            len(item["required_evidence"]) >= 2 and len(item["claim_limit"]) >= 30
            for item in self.profile["site_constraints"].values()
        ))

    def test_assessed_site_constraint_requires_evidence_and_reviewer(self):
        doc = copy.deepcopy(self.profile)
        constraint = doc["site_constraints"]["soil_condition_and_contamination"]
        constraint["state"] = "assessed"
        schema_errors = list(Draft202012Validator(self.schema).iter_errors(doc))
        self.assertTrue(any("evidence_refs" in error.message or "reviewed_by" in error.message for error in schema_errors))

    def test_assessed_site_constraint_accepts_auditable_reference_shape(self):
        doc = copy.deepcopy(self.profile)
        constraint = doc["site_constraints"]["soil_condition_and_contamination"]
        constraint["state"] = "assessed"
        constraint["evidence_refs"] = ["urn:fixture:evidence:soil-screening-1"]
        constraint["reviewed_by"] = "qualified-reviewer-role"
        schema_errors = list(Draft202012Validator(self.schema).iter_errors(doc))
        self.assertFalse(any(
            list(error.path)[:2] == ["site_constraints", "soil_condition_and_contamination"]
            and ("evidence_refs" in error.message or "reviewed_by" in error.message)
            for error in schema_errors
        ))

    def test_site_constraint_references_are_empty_when_unassessed(self):
        doc = copy.deepcopy(self.profile)
        constraint = doc["site_constraints"]["irrigation_water_quantity_and_quality"]
        constraint["evidence_refs"] = ["urn:fixture:should-not-be-here"]
        schema_errors = list(Draft202012Validator(self.schema).iter_errors(doc))
        self.assertTrue(schema_errors)

    def test_food_resilience_does_not_assume_local_self_sufficiency(self):
        constraint = self.profile["site_constraints"]["regional_supply_and_contingency"]
        self.assertIn("does not require all food to be produced locally", constraint["claim_limit"])
        self.assertTrue(any(
            "Regional supply" in item for item in self.profile["model_architecture"]["required_inputs"]
        ))
        self.assertTrue(any(
            "regional supply" in item.lower() for item in self.profile["evaluation"]["minimum_local_evidence"]
        ))

    def test_policy_context_references_are_explicit_and_limited(self):
        references = {item["reference_id"]: item for item in self.profile["context_references"]}
        self.assertIn("COJ-FOOD-RESILIENCE-POLICY", references)
        self.assertIn("COJ-UPPER-JUKSKEI-WATER-QUALITY-2026", references)
        self.assertTrue(all(item["limitations"] for item in references.values()))
        self.assertTrue(any("not a substitute" in limit.lower()
                            for item in references.values() for limit in item["limitations"]))

    def test_irrigation_quality_guidance_carries_legacy_date_caveat(self):
        reference = next(
            item for item in self.profile["context_references"]
            if item["reference_id"] == "DWS-IRRIGATION-WATER-QUALITY-GUIDELINES"
        )
        self.assertIn("1996", reference["title"])
        self.assertTrue(any(
            "verify the currently authoritative edition" in limit.lower()
            for limit in reference["limitations"]
        ))

    def test_local_crop_cohort_is_evidence_led_but_not_selected(self):
        cohorts = self.profile["candidate_crop_cohorts"]
        self.assertEqual(len(cohorts), 1)
        cohort = cohorts[0]
        self.assertEqual(
            cohort["crop_names"],
            ["tomato", "spinach", "green pepper", "cucumber"],
        )
        self.assertEqual(cohort["selection_state"], "candidate_not_selected")
        self.assertGreaterEqual(len(cohort["review_requirements"]), 3)
        self.assertIn("not a recommendation", cohort["claim_limit"].lower())

    def test_design_only_cannot_claim_crop_cohort_selected_for_baseline(self):
        doc = copy.deepcopy(self.profile)
        doc["candidate_crop_cohorts"][0]["selection_state"] = "selected_for_baseline"
        self.assertTrue(any("design_only status requires all crop cohorts" in error for error in self.errors(doc)))

    def test_duplicate_crop_cohort_ids_are_rejected(self):
        doc = copy.deepcopy(self.profile)
        duplicate = copy.deepcopy(doc["candidate_crop_cohorts"][0])
        doc["candidate_crop_cohorts"].append(duplicate)
        self.assertTrue(any("duplicate candidate crop cohort id" in error for error in self.errors(doc)))

    def test_duplicate_context_reference_ids_are_rejected(self):
        doc = copy.deepcopy(self.profile)
        duplicate = copy.deepcopy(doc["context_references"][0])
        doc["context_references"].append(duplicate)
        self.assertTrue(any("duplicate context reference_id" in error for error in self.errors(doc)))

    def test_design_only_cannot_claim_a_site_constraint_was_assessed(self):
        doc = copy.deepcopy(self.profile)
        doc["site_constraints"]["soil_condition_and_contamination"]["state"] = "assessed"
        self.assertTrue(any("requires every site constraint to remain not_assessed" in error for error in self.errors(doc)))

    def test_decision_support_requires_every_site_constraint_assessed(self):
        doc = copy.deepcopy(self.profile)
        doc["status"] = "decision_support_candidate"
        doc["evidence_state"] = {
            "source_artifacts_acquired": True,
            "baseline_computed": True,
            "independent_validation_complete": True,
            "expert_review_complete": True,
            "decision_support_authorized": True,
        }
        doc["geography"]["boundary_state"] = "verified_and_frozen"
        for gate in ("D0", "D1", "D2", "D3", "D4", "D5"):
            doc["validation"]["gates"][gate] = "passed"
        self.assertTrue(any("requires assessment of every site constraint" in error for error in self.errors(doc)))

    def test_shared_schema_rejects_unrecognized_site_constraint_state(self):
        doc = copy.deepcopy(self.profile)
        doc["site_constraints"]["irrigation_water_quantity_and_quality"]["state"] = "probably_safe"
        schema_errors = list(Draft202012Validator(self.schema).iter_errors(doc))
        self.assertTrue(schema_errors)

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

    def test_shared_schema_requires_external_source_url(self):
        doc = copy.deepcopy(self.profile)
        doc["sources"][0]["url"] = None
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
        source["rights_state"] = "license_reviewed"
        self.assertTrue(any("local observations must retain permission_required" in error for error in self.errors(doc)))

    def test_authorized_local_observations_can_remain_private(self):
        doc = copy.deepcopy(self.profile)
        doc["status"] = "data_readiness"
        source = next(item for item in doc["sources"] if item["source_id"] == "LOCAL-OBSERVATIONS")
        source["acquisition_state"] = "acquired"
        source["selected_product"] = "urn:mycelix:measurement-protocol:highveld-site-v1"
        source["artifact_sha256"] = "a" * 64
        source["source_identity_verified"] = True
        source["rights_state"] = "permission_granted"
        source["redistribution_allowed"] = None
        self.assertEqual(self.errors(doc), [])

    def test_acquired_local_observations_cannot_claim_public_redistribution(self):
        doc = copy.deepcopy(self.profile)
        doc["status"] = "data_readiness"
        source = next(item for item in doc["sources"] if item["source_id"] == "LOCAL-OBSERVATIONS")
        source["acquisition_state"] = "acquired"
        source["selected_product"] = "urn:mycelix:measurement-protocol:highveld-site-v1"
        source["artifact_sha256"] = "a" * 64
        source["source_identity_verified"] = True
        source["rights_state"] = "permission_granted"
        source["redistribution_allowed"] = False
        self.assertTrue(any("redistribution_allowed=null" in error for error in self.errors(doc)))

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

    def test_D0_can_pass_when_boundary_and_gate_evidence_are_frozen(self):
        doc = copy.deepcopy(self.profile)
        doc["status"] = "data_readiness"
        doc["geography"]["boundary_state"] = "verified_and_frozen"
        doc["geography"]["boundary_source"] = "Authoritative municipal boundary artifact reviewed and hash recorded."
        doc["geography"]["boundary_evidence_ref"] = "urn:fixture:boundary:joburg-v1"
        doc["geography"]["boundary_sha256"] = "a" * 64
        doc["geography"]["boundary_crs"] = "EPSG:4326"
        doc["validation"]["gates"]["D0"] = "passed"
        doc["validation"]["gate_evidence"]["D0"] = {
            "evidence_refs": ["sha256:" + "a" * 64],
            "reviewed_by": "independent-boundary-reviewer",
            "reason": None,
        }
        self.assertFalse(self.errors(doc))

    def test_D0_cannot_pass_without_boundary_digest(self):
        doc = copy.deepcopy(self.profile)
        doc["status"] = "data_readiness"
        doc["geography"]["boundary_state"] = "verified_and_frozen"
        doc["geography"]["boundary_source"] = "Authoritative municipal boundary artifact reviewed and hash recorded."
        doc["geography"]["boundary_evidence_ref"] = "urn:fixture:boundary:joburg-v1"
        doc["geography"]["boundary_crs"] = "EPSG:4326"
        doc["validation"]["gates"]["D0"] = "passed"
        doc["validation"]["gate_evidence"]["D0"] = {
            "evidence_refs": ["sha256:" + "a" * 64],
            "reviewed_by": "independent-boundary-reviewer",
            "reason": None,
        }
        self.assertTrue(any("boundary_sha256" in error for error in self.errors(doc)))

    def test_unverified_boundary_cannot_claim_hash_crs_or_reference(self):
        doc = copy.deepcopy(self.profile)
        doc["geography"]["boundary_sha256"] = "a" * 64
        doc["geography"]["boundary_crs"] = "EPSG:4326"
        doc["geography"]["boundary_evidence_ref"] = "urn:fixture:premature-boundary"
        self.assertTrue(any("unverified boundary must not claim" in error for error in self.errors(doc)))

    def test_passed_gate_cannot_skip_evidence_reference_or_reviewer(self):
        doc = copy.deepcopy(self.profile)
        doc["status"] = "data_readiness"
        doc["geography"]["boundary_state"] = "verified_and_frozen"
        doc["geography"]["boundary_source"] = "Authoritative municipal boundary artifact reviewed and hash recorded."
        doc["geography"]["boundary_evidence_ref"] = "urn:fixture:boundary:joburg-v1"
        doc["geography"]["boundary_sha256"] = "a" * 64
        doc["geography"]["boundary_crs"] = "EPSG:4326"
        doc["validation"]["gates"]["D0"] = "passed"
        self.assertTrue(any("gate D0=passed requires at least one evidence reference" in error for error in self.errors(doc)))

    def test_unstarted_gate_cannot_carry_evidence(self):
        doc = copy.deepcopy(self.profile)
        doc["validation"]["gate_evidence"]["D0"]["evidence_refs"] = ["urn:fixture:premature"]
        self.assertTrue(any("gate D0=not_started must not carry evidence" in error for error in self.errors(doc)))

    def test_D1_cannot_pass_with_unacquired_sources(self):
        doc = copy.deepcopy(self.profile)
        doc["status"] = "data_readiness"
        doc["validation"]["gates"]["D1"] = "passed"
        self.assertTrue(any("D1=passed requires acquisition" in error for error in self.errors(doc)))

    def test_D1_cannot_skip_the_frozen_study_boundary(self):
        doc = copy.deepcopy(self.profile)
        doc["status"] = "data_readiness"
        doc["validation"]["gates"]["D1"] = "passed"
        errors = self.errors(doc)
        self.assertTrue(any("D1=passed requires D0=passed" in error for error in errors))

    def test_D2_cannot_pass_before_a_baseline_exists(self):
        doc = copy.deepcopy(self.profile)
        doc["status"] = "data_readiness"
        doc["validation"]["gates"]["D2"] = "passed"
        self.assertTrue(any("D2=passed requires baseline_computed=true" in error for error in self.errors(doc)))

    def test_D2_requires_a_selected_crop_cohort(self):
        doc = copy.deepcopy(self.profile)
        doc["status"] = "data_readiness"
        doc["validation"]["gates"]["D2"] = "passed"
        doc["validation"]["gates"]["D0"] = "passed"
        doc["validation"]["gates"]["D1"] = "passed"
        doc["evidence_state"]["source_artifacts_acquired"] = True
        doc["evidence_state"]["baseline_computed"] = True
        doc["geography"]["boundary_state"] = "verified_and_frozen"
        self.assertTrue(any("D2=passed requires at least one crop cohort selected" in error for error in self.errors(doc)))

    def test_D2_requires_a_frozen_model_choice(self):
        doc = copy.deepcopy(self.profile)
        doc["status"] = "data_readiness"
        doc["validation"]["gates"]["D0"] = "passed"
        doc["validation"]["gates"]["D1"] = "passed"
        doc["validation"]["gates"]["D2"] = "passed"
        doc["evidence_state"]["source_artifacts_acquired"] = True
        doc["evidence_state"]["baseline_computed"] = True
        doc["candidate_crop_cohorts"][0]["selection_state"] = "selected_for_baseline"
        self.assertTrue(any("D2=passed requires model_selection_state to freeze" in error for error in self.errors(doc)))

    def test_D3_cannot_pass_before_independent_validation(self):
        doc = copy.deepcopy(self.profile)
        doc["status"] = "data_readiness"
        doc["validation"]["gates"]["D3"] = "passed"
        self.assertTrue(any("D3=passed requires independent_validation_complete=true" in error for error in self.errors(doc)))

    def test_D3_requires_calibrated_model_state(self):
        doc = copy.deepcopy(self.profile)
        doc["status"] = "data_readiness"
        doc["validation"]["gates"]["D0"] = "passed"
        doc["validation"]["gates"]["D1"] = "passed"
        doc["validation"]["gates"]["D2"] = "passed"
        doc["validation"]["gates"]["D3"] = "passed"
        doc["evidence_state"]["source_artifacts_acquired"] = True
        doc["evidence_state"]["baseline_computed"] = True
        doc["evidence_state"]["independent_validation_complete"] = True
        doc["candidate_crop_cohorts"][0]["selection_state"] = "selected_for_baseline"
        doc["model_architecture"]["model_selection_state"] = "selected_for_baseline"
        errors = self.errors(doc)
        self.assertTrue(any("D3=passed requires model_selection_state=selected_and_calibrated" in error for error in errors))

    def test_D4_cannot_pass_before_expert_review(self):
        doc = copy.deepcopy(self.profile)
        doc["status"] = "data_readiness"
        doc["validation"]["gates"]["D4"] = "passed"
        self.assertTrue(any("D4=passed requires expert_review_complete=true" in error for error in self.errors(doc)))

    def test_D4_cannot_skip_prior_validation_gates_even_after_review(self):
        doc = copy.deepcopy(self.profile)
        doc["status"] = "data_readiness"
        doc["validation"]["gates"]["D4"] = "passed"
        doc["evidence_state"]["expert_review_complete"] = True
        errors = self.errors(doc)
        self.assertTrue(any("D4=passed requires D0-D3=passed" in error for error in errors))

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
