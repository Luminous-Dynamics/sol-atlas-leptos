"""Regression tests for fail-closed source-registry validation."""

from __future__ import annotations

import copy
import unittest

import verify_economic_source_registry as verifier


class SourceRegistryVerifierTests(unittest.TestCase):
    def setUp(self) -> None:
        self.source = copy.deepcopy(verifier.read_source())

    def test_checked_in_record_passes_contract(self) -> None:
        self.assertEqual(verifier.validate(self.source), 15)

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


if __name__ == "__main__":
    unittest.main()
