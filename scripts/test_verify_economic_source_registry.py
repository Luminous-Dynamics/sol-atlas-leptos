"""Regression tests for fail-closed source-registry validation."""

from __future__ import annotations

import copy
import unittest

import verify_economic_source_registry as verifier


class SourceRegistryVerifierTests(unittest.TestCase):
    def setUp(self) -> None:
        self.source = copy.deepcopy(verifier.read_source())

    def test_checked_in_record_passes_contract(self) -> None:
        self.assertEqual(verifier.validate(self.source), 17)

    def test_boolean_cannot_masquerade_as_integer_schema_version(self) -> None:
        self.source["schema_version"] = True
        with self.assertRaisesRegex(verifier.ContractError, "schema_version must be integer 1"):
            verifier.validate(self.source)

    def test_unknown_license_evidence_id_fails_with_contract_error(self) -> None:
        self.source["license_evidence_matrix"][0]["evidence_id"] = "unreviewed_record"
        with self.assertRaisesRegex(verifier.ContractError, "license evidence IDs differ"):
            verifier.validate(self.source)

    def test_duplicate_rights_reconciliation_action_is_rejected(self) -> None:
        actions = self.source["rights_adjudication"]["required_actions"]
        actions[1] = actions[0]
        with self.assertRaisesRegex(verifier.ContractError, "must not be duplicated"):
            verifier.validate(self.source)

    def test_duplicate_derivative_action_is_rejected(self) -> None:
        actions = self.source["derivative_candidate"]["next_actions"]
        actions[1] = actions[0]
        with self.assertRaisesRegex(verifier.ContractError, "missing or duplicated"):
            verifier.validate(self.source)

    def test_employment_indicator_substitution_is_rejected(self) -> None:
        indicators = self.source["source_page_claims"]["employment_indicators"]
        indicators[0] = "invented_indicator"
        with self.assertRaisesRegex(verifier.ContractError, "employment inventory content/type drift"):
            verifier.validate(self.source)

    def test_perception_source_count_change_is_rejected(self) -> None:
        perception = self.source["source_page_claims"]["perception_sources"]
        perception[1]["count"] = 2
        with self.assertRaisesRegex(verifier.ContractError, "perception source inventory drift"):
            verifier.validate(self.source)

    def test_methodology_author_substitution_is_rejected(self) -> None:
        self.source["primary_methodology_citation"]["authors"][0] = "Unreviewed Author"
        with self.assertRaisesRegex(verifier.ContractError, "methodology citation authors drift"):
            verifier.validate(self.source)

    def test_period_year_must_be_an_integer_not_a_float(self) -> None:
        self.source["source_period_start"] = 1990.0
        with self.assertRaisesRegex(verifier.ContractError, "source_period_start must remain integer 1990"):
            verifier.validate(self.source)

    def test_non_text_license_evidence_id_fails_as_contract_error(self) -> None:
        self.source["license_evidence_matrix"][0]["evidence_id"] = ["bad", "shape"]
        with self.assertRaisesRegex(verifier.ContractError, "evidence_id must be non-empty text"):
            verifier.validate(self.source)

    def test_data360_period_is_distinct_from_current_official_brief(self) -> None:
        self.source["edition_history_evidence"]["source_period_end"] = 2020
        with self.assertRaisesRegex(verifier.ContractError, "Data360 edition source_period_end must remain integer 2018"):
            verifier.validate(self.source)

    def test_data360_measure_count_must_remain_twelve(self) -> None:
        self.source["edition_history_evidence"]["measure_count_claimed"] = 11
        with self.assertRaisesRegex(verifier.ContractError, "Data360 edition measure_count_claimed must remain integer 12"):
            verifier.validate(self.source)

    def test_data360_metadata_does_not_claim_data_bytes_or_observations(self) -> None:
        self.source["edition_history_evidence"]["data_observations_captured"] = True
        with self.assertRaisesRegex(verifier.ContractError, "do not overstate Data360 metadata or observation acquisition"):
            verifier.validate(self.source)

    def test_archived_v7_snapshot_remains_valid_and_separate(self) -> None:
        archived = verifier.read_archived_v7()
        self.assertEqual(verifier.validate_archived_v7(archived), 16)
        self.assertEqual(archived["registry_version"], "world-bank-informal-economy-database/v7")
        self.assertNotIn("data360_api_review", archived)

    def test_archived_snapshot_cannot_be_silently_promoted_to_v8(self) -> None:
        archived = verifier.read_archived_v7()
        archived["registry_version"] = "world-bank-informal-economy-database/v8"
        with self.assertRaisesRegex(verifier.ContractError, "archived snapshot must remain v7"):
            verifier.validate_archived_v7(archived)

    def test_data360_page_limit_cannot_drift_from_documented_thousand(self) -> None:
        self.source["data360_api_review"]["documented_routes"]["retrieve_observations"]["documented_max_records_per_response"] = 500
        with self.assertRaisesRegex(verifier.ContractError, "documented 1000-row limit"):
            verifier.validate(self.source)

    def test_blocked_api_probe_does_not_claim_an_http_status(self) -> None:
        self.source["data360_api_review"]["probe"]["http_status"] = 200
        with self.assertRaisesRegex(verifier.ContractError, "do not claim Data360 HTTP status"):
            verifier.validate(self.source)

    def test_data360_default_terms_do_not_approve_redistribution(self) -> None:
        self.source["data360_api_review"]["terms_review"]["derived_data_redistribution_approved"] = True
        with self.assertRaisesRegex(verifier.ContractError, "Data360 rights must not be treated as project approval"):
            verifier.validate(self.source)

    def test_api_acquisition_protocol_must_not_have_duplicate_actions(self) -> None:
        actions = self.source["data360_api_review"]["acquisition_protocol"]
        actions[1] = actions[0]
        with self.assertRaisesRegex(verifier.ContractError, "eight distinct"):
            verifier.validate(self.source)


if __name__ == "__main__":
    unittest.main()
