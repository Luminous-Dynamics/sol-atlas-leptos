#!/usr/bin/env python3
"""Verify the World Bank informal-economy source qualification record.

This is a source-metadata contract check, not a workbook parser, license opinion,
or validation of any observed data values. The candidate remains metadata-only.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
SOURCE_PATH = ROOT / "data" / "economic_sources" / "world_bank_informal_economy_database_v1.json"

TOP_KEYS = {
    "schema_version", "registry_id", "registry_version", "record_kind", "status",
    "source_id", "publisher", "title", "canonical_url", "data_url", "source_page_date",
    "source_period_start", "source_period_end", "max_economies_claimed",
    "measure_count_claimed_by_source_page", "source_page_update_frequency",
    "source_page_claims", "primary_methodology_citation", "license_review",
    "provenance", "qualification_gates", "interpretation_ceiling",
}
CLAIMS_KEYS = {"output_methods", "employment_indicators", "perception_sources", "count_reconciliation"}
CITATION_KEYS = {"authors", "year", "title", "publication", "url"}
LICENSE_KEYS = {
    "status", "redistributable", "commercial_use_approved",
    "general_world_bank_dataset_terms_url", "general_world_bank_terms_summary",
    "conflicting_or_ambiguous_evidence", "required_next_action",
}
AMBIGUITY_KEYS = {"source_url", "catalog_label", "catalog_license_url", "interpretation_limit"}
PROVENANCE_KEYS = {
    "acquisition_method", "workbook_downloaded", "source_snapshot_id",
    "source_snapshot_digest", "digest_status", "raw_observations_embedded",
    "observed_column_inventory", "last_reviewed_at",
}
EXPECTED_GATES = {
    "confirm exact workbook edition and digest",
    "capture workbook sheet and variable inventory without altering the original",
    "reconcile measure count against the 2021 methodology paper and workbook variables",
    "record definition, unit, denominator, geography, period, method, missingness and source for each variable",
    "separate DGE and MIMIC estimates as alternative model-based measures; do not average or sum by default",
    "separate model estimates from survey-derived employment and perception indicators",
    "check country-year coverage and missingness before comparing economies",
    "resolve source-specific terms and third-party restrictions before any data redistribution or commercial use",
    "do not infer illegality or adjudicated illicit output from informality estimates",
}


class ContractError(Exception):
    """The source qualification record violates a fail-closed invariant."""


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ContractError(message)


def exact_keys(value: Any, expected: set[str], where: str) -> dict[str, Any]:
    require(isinstance(value, dict), f"{where} must be an object")
    actual = set(value)
    require(actual == expected, (
        f"{where} keys differ: missing={sorted(expected - actual)}, "
        f"unexpected={sorted(actual - expected)}"
    ))
    return value


def nonempty_text(value: Any, where: str) -> str:
    require(isinstance(value, str) and bool(value.strip()), f"{where} must be non-empty text")
    require(value == value.strip(), f"{where} must not have surrounding whitespace")
    return value


def read_source() -> dict[str, Any]:
    try:
        value = json.loads(SOURCE_PATH.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise ContractError(f"cannot load source record: {error}") from error
    return exact_keys(value, TOP_KEYS, "source")


def validate(source: dict[str, Any]) -> int:
    require(source["schema_version"] == 1, "unsupported schema version")
    require(source["registry_id"] == "sol-atlas-economic-source-registry", "registry ID drift")
    require(source["registry_version"] == "world-bank-informal-economy-database/v1", "version drift")
    require(source["record_kind"] == "source_qualification_candidate", "record kind must stay candidate")
    require(source["status"] == "metadata_only_not_approved_for_data_ingestion_or_redistribution",
            "source moved beyond metadata-only status without qualification")
    require(source["source_id"] == "world_bank_informal_economy_database", "source ID drift")
    require(source["publisher"] == "World Bank Prospects Group", "publisher drift")
    require(source["title"] == "Informal Economy Database", "title drift")
    require(source["canonical_url"] == "https://www.worldbank.org/en/brief/2024/01/09/informal-economy-database",
            "official source URL drift")
    require(source["data_url"] == (
        "https://thedocs.worldbank.org/en/doc/37511318c092e6fd4ca3c60f0af0bea3-0350012021/"
        "related/informal-economy-database.xlsx"
    ), "source workbook URL drift")
    require(source["source_period_start"] == 1990 and source["source_period_end"] == 2020,
            "historical period boundary drift")
    require(source["max_economies_claimed"] == 196, "economy-count claim drift")
    require(source["measure_count_claimed_by_source_page"] == 11, "source-page measure-count claim drift")
    require(source["source_page_update_frequency"] == "every_two_years", "update-frequency claim drift")

    claims = exact_keys(source["source_page_claims"], CLAIMS_KEYS, "source_page_claims")
    require(set(claims["output_methods"]) == {
        "dynamic_general_equilibrium_model", "multiple_indicators_multiple_causes_model"
    }, "both model-based output methods must remain separately identified")
    require(len(claims["employment_indicators"]) == 4, "employment indicator inventory drift")
    require(len(claims["perception_sources"]) == 3, "survey-source inventory drift")
    require("reconcile" in claims["count_reconciliation"].lower() and
            "workbook" in claims["count_reconciliation"].lower(),
            "page-versus-variable-count reconciliation warning missing")

    citation = exact_keys(source["primary_methodology_citation"], CITATION_KEYS, "primary_methodology_citation")
    require(citation["year"] == 2021, "methodology citation year drift")
    require(citation["url"] == "https://cepr.org/publications/dp16497", "methodology citation URL drift")

    license_review = exact_keys(source["license_review"], LICENSE_KEYS, "license_review")
    require(license_review["status"] == "source_specific_review_required",
            "source-specific terms gate weakened")
    require(license_review["redistributable"] is False, "redistribution not yet qualified")
    require(license_review["commercial_use_approved"] is False, "commercial use not yet qualified")
    require(license_review["general_world_bank_dataset_terms_url"] ==
            "https://www.worldbank.org/ext/en/legal/terms-conditions/datasets", "terms URL drift")
    ambiguous = exact_keys(license_review["conflicting_or_ambiguous_evidence"], AMBIGUITY_KEYS,
                           "license_review.conflicting_or_ambiguous_evidence")
    require(ambiguous["catalog_label"] == "Custom license", "specific-license warning dropped")
    require(ambiguous["source_url"] == "https://reproducibility.worldbank.org/catalog/583",
            "secondary terms evidence URL drift")
    require("does not by itself prove" in ambiguous["interpretation_limit"],
            "secondary catalog must not be treated as a conclusive license ruling")
    require("before ingestion" in license_review["required_next_action"],
            "source-specific review action missing")

    provenance = exact_keys(source["provenance"], PROVENANCE_KEYS, "provenance")
    require(provenance["workbook_downloaded"] is False, "do not claim the workbook was downloaded")
    require(provenance["source_snapshot_id"] is None, "source snapshot ID is not yet captured")
    require(provenance["source_snapshot_digest"] is None, "do not fabricate a source digest")
    require(provenance["digest_status"] == "not_captured", "digest status drift")
    require(provenance["raw_observations_embedded"] is False, "candidate record must not embed observations")
    require(provenance["observed_column_inventory"] is None, "column inventory not yet observed")

    gates = source["qualification_gates"]
    require(isinstance(gates, list) and len(gates) == len(EXPECTED_GATES), "qualification gate inventory drift")
    require(set(gates) == EXPECTED_GATES, "a qualification gate was removed, duplicated, or changed")
    require("not a current-year estimate" in source["interpretation_ceiling"],
            "interpretation ceiling must disallow a current-year inference")
    require("not evidence of illegal activity" in source["interpretation_ceiling"],
            "interpretation ceiling must prevent criminality inference")
    return len(gates)


def main() -> int:
    try:
        count = validate(read_source())
    except ContractError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        return 1
    print("PASS: World Bank source candidate preserves its metadata, period and method distinctions.")
    print("PASS: custom/source-specific reuse gate, missing digest, and measure-count reconciliation are explicit.")
    print(f"SCOPE: metadata contract only; {count} qualification gates remain open; no workbook values were validated.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
