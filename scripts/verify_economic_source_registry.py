#!/usr/bin/env python3
"""Verify the World Bank informal-economy source qualification record v3.

This checks source metadata and project approval gates only. It is not a workbook
parser, source-authenticity check, empirical replication, or legal opinion.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
SOURCE_PATH = ROOT / "data" / "economic_sources" / "world_bank_informal_economy_database_v4.json"

TOP_KEYS = {
    "schema_version", "registry_id", "registry_version", "supersedes_registry_version",
    "record_kind", "status", "source_id", "publisher", "title", "canonical_url",
    "data_url", "source_page_date", "source_period_start", "source_period_end",
    "max_economies_claimed", "measure_count_claimed_by_source_page",
    "source_page_update_frequency", "source_page_claims", "primary_methodology_citation",
    "official_asset", "reproducibility_package_evidence", "license_review",
    "provenance", "qualification_gates", "interpretation_ceiling", "cross_catalog_evidence", "derivative_candidate",
}
CLAIMS_KEYS = {"output_methods", "employment_indicators", "perception_sources",
               "count_reconciliation", "concept_inventory"}
CONCEPT_KEYS = {
    "source_page_claimed_measure_count", "output_series_listed", "employment_indicators_listed",
    "perception_indicators_listed", "series_count_if_all_listed_as_separate",
    "conceptual_count_if_output_estimators_grouped", "count_reconciliation_status", "note",
}
CITATION_KEYS = {"authors", "year", "title", "publication", "url"}
ASSET_KEYS = {"label", "format", "url", "discovered_on_official_page",
              "downloaded_in_this_review", "source_page_update_frequency", "note"}
PACKAGE_KEYS = {"catalog_url", "catalog_created_date", "dataset_access_date",
                "filename_in_package", "access_policy_claim", "license_label",
                "license_url", "interpretation"}
LICENSE_KEYS = {
    "status", "rights_decision", "official_page_download_present",
    "publicly_available_in_reproducibility_package", "dataset_license_label",
    "redistribution_approved", "commercial_use_approved", "approval_status",
    "general_world_bank_dataset_terms_url", "general_world_bank_terms_summary",
    "specific_evidence_url", "specific_evidence_label", "specific_evidence_url_points_to",
    "interpretation_limit", "required_next_action",
}
DERIVATIVE_KEYS = {
    "candidate_id", "status", "catalog_url", "catalog_created_date", "catalog_last_modified_date",
    "source_access_date", "dataset_names_in_package", "source_sheets_reported",
    "processed_variables_reported", "other_reported_variables", "license_label_reported",
    "license_url", "rights_scope", "raw_or_derivative", "downloaded_in_this_review",
    "source_snapshot_digest", "digest_status", "project_status", "next_actions",
}
DERIVATIVE_VARIABLE_KEYS = {"id", "meaning"}
PROVENANCE_KEYS = {
    "acquisition_method", "workbook_downloaded", "source_snapshot_id",
    "source_snapshot_digest", "digest_status", "raw_observations_embedded",
    "observed_column_inventory", "last_reviewed_at",
}
CROSS_CATALOG_KEYS = {"catalog_url", "reviewed_on", "catalog_name", "variables", "interpretation_limit"}
VARIABLE_KEYS = {"id", "label"}
REPRO_CROSS_CATALOG_KEYS = {"catalog_url", "reviewed_on", "catalog_name", "reported_sheets",
                            "reported_variables", "source_context"}
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
    "obtain and hash the specific reproducibility-package derivative before considering its values for import",
    "do not transfer the derivative's CC BY 4.0 metadata to the current official Excel workbook or to the separately author-provided companion file",
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


def read_source() -> dict[str, Any]:
    try:
        value = json.loads(SOURCE_PATH.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise ContractError(f"cannot load source record: {error}") from error
    return exact_keys(value, TOP_KEYS, "source")


def validate(source: dict[str, Any]) -> int:
    require(source["schema_version"] == 1, "unsupported record schema")
    require(source["registry_id"] == "sol-atlas-economic-source-registry", "registry ID drift")
    require(source["registry_version"] == "world-bank-informal-economy-database/v4", "version drift")
    require(source["supersedes_registry_version"] == "world-bank-informal-economy-database/v3",
            "superseded record reference drift")
    require(source["record_kind"] == "source_qualification_candidate", "record kind drift")
    require(source["status"] == "metadata_only_not_approved_for_data_ingestion_or_redistribution",
            "candidate moved beyond metadata-only status without qualification")
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
    }, "both output methods must remain explicit")
    require(len(claims["employment_indicators"]) == 4, "employment inventory drift")
    require(len(claims["perception_sources"]) == 3, "perception source inventory drift")
    inventory = exact_keys(claims["concept_inventory"], CONCEPT_KEYS, "concept_inventory")
    require(inventory["source_page_claimed_measure_count"] == 11, "claimed measure count drift")
    require(inventory["output_series_listed"] == len(claims["output_methods"]) == 2,
            "two output series/methods must stay explicit")
    require(inventory["employment_indicators_listed"] == len(claims["employment_indicators"]) == 4,
            "employment series count drift")
    require(inventory["perception_indicators_listed"] == 6,
            "one WEF, four enterprise-survey and one WVS indicators must remain documented")
    require(inventory["series_count_if_all_listed_as_separate"] ==
            inventory["output_series_listed"] + inventory["employment_indicators_listed"] +
            inventory["perception_indicators_listed"] == 12,
            "literal listed-series count must stay 12")
    require(inventory["conceptual_count_if_output_estimators_grouped"] == 11,
            "optional conceptual grouping arithmetic drift")
    require(inventory["count_reconciliation_status"] == "unresolved_until_exact_workbook_inventory",
            "do not prematurely resolve source page's 11-versus-12 inventory question")
    require("exact official workbook edition" in inventory["note"] and
            "do not select an interpretation" in inventory["note"],
            "uncertainty about source's intended count must remain explicit")
    require("actual official workbook edition" in claims["count_reconciliation"],
            "workbook-level qualification requirement missing")

    citation = exact_keys(source["primary_methodology_citation"], CITATION_KEYS, "primary_methodology_citation")
    require(citation["year"] == 2021, "methodology citation year drift")
    require(citation["url"] == "https://cepr.org/publications/dp16497", "methodology URL drift")

    asset = exact_keys(source["official_asset"], ASSET_KEYS, "official_asset")
    require(asset["label"] == "Informal economy data (Excel)" and asset["format"] == "xlsx",
            "official asset identity drift")
    require(asset["url"] == source["data_url"] and asset["discovered_on_official_page"] is True,
            "official Excel link evidence missing")
    require(asset["downloaded_in_this_review"] is False, "do not claim workbook bytes were inspected")
    require(asset["source_page_update_frequency"] == "every_two_years", "asset update cadence drift")

    package = exact_keys(source["reproducibility_package_evidence"], PACKAGE_KEYS,
                         "reproducibility_package_evidence")
    require(package["catalog_url"] == "https://reproducibility.worldbank.org/catalog/583",
            "reproducibility catalog URL drift")
    require(package["catalog_created_date"] == "2026-06-17", "catalog date drift")
    require(package["dataset_access_date"] == "2026-05-05", "catalog source access date drift")
    require(package["filename_in_package"] == "DGE_Raw.xlsx", "package filename evidence drift")
    require(package["access_policy_claim"] == "publicly_available_and_included_in_reproducibility_package",
            "public availability evidence missing")
    require(package["license_label"] == "Custom license", "custom-license label not recorded")
    require(package["license_url"] == (
        "https://www.worldbank.org/en/about/legal/terms-of-use-for-datasets"
    ), "reproducibility package terms URL drift")
    require("not a complete rights analysis" in package["interpretation"],
            "availability must remain distinct from permission")

    cross = source["cross_catalog_evidence"]
    require(isinstance(cross, list) and len(cross) == 2, "expected two secondary catalog records")
    qog = exact_keys(cross[0], CROSS_CATALOG_KEYS, "cross_catalog_evidence[0]")
    require(qog["catalog_url"] == "https://datafinder.qog.gu.se/dataset/ied", "QoG catalog URL drift")
    require([x["id"] for x in qog["variables"]] == ["ied_dge", "ied_mimic"],
            "separate DGE/MIMIC downstream variables not recorded")
    for index, variable in enumerate(qog["variables"]):
        exact_keys(variable, VARIABLE_KEYS, f"QoG variable[{index}]")
    require("does not establish the exact sheet/column inventory" in qog["interpretation_limit"],
            "secondary catalog overclaimed as official workbook schema")
    repro = exact_keys(cross[1], REPRO_CROSS_CATALOG_KEYS, "cross_catalog_evidence[1]")
    require(repro["catalog_url"] == "https://reproducibility.worldbank.org/catalog/330",
            "secondary reproduction catalog URL drift")
    require(set(repro["reported_sheets"]) == {"DGE_p", "MIMIC_p"},
            "processed derivative sheet names drift")
    require(set(repro["reported_variables"]) == {"dge_paverage_1993_2019", "mimic_paverage_1993_2019"},
            "processed derivative variable names drift")
    require("must not be assumed to match" in repro["source_context"],
            "derivative must remain distinct from the current official workbook")

    license_review = exact_keys(source["license_review"], LICENSE_KEYS, "license_review")
    require(license_review["status"] == "source_specific_review_required", "terms gate weakened")
    require(license_review["rights_decision"] == "not_determined", "rights conclusion overstated")
    require(license_review["official_page_download_present"] is True, "official download evidence missing")
    require(license_review["publicly_available_in_reproducibility_package"] is True,
            "public availability evidence missing")
    require(license_review["dataset_license_label"] == "Custom license", "license label drift")
    require(license_review["redistribution_approved"] is False, "project has not approved redistribution")
    require(license_review["commercial_use_approved"] is False, "project has not approved commercial use")
    require(license_review["approval_status"] == "not_approved_pending_terms_check",
            "project approval state drift")
    require(license_review["specific_evidence_url"] == package["catalog_url"], "license evidence URL mismatch")
    require(license_review["specific_evidence_label"] == "Custom license", "specific evidence label drift")
    require("not a legal finding" in license_review["interpretation_limit"],
            "do not misstate project approval status as legal prohibition")

    derivative = exact_keys(source["derivative_candidate"], DERIVATIVE_KEYS, "derivative_candidate")
    require(derivative["candidate_id"] == "world-bank-informal-economy-database-with-fixes",
            "derivative candidate ID drift")
    require(derivative["status"] == "public_reproducibility_package_candidate_not_yet_materialized_or_hash_verified",
            "derivative candidate must not be treated as captured/approved")
    require(derivative["catalog_url"] == "https://reproducibility.worldbank.org/catalog/330",
            "derivative catalog URL drift")
    require(derivative["catalog_created_date"] == "2025-09-15" and
            derivative["catalog_last_modified_date"] == "2026-01-06" and
            derivative["source_access_date"] == "2024-09", "derivative dates drift")
    require(set(derivative["dataset_names_in_package"]) == {
        "informal-economy-database-with-fixes.dta", "INFORMALITY DATA SET.dta"
    }, "derivative filenames drift")
    require(set(derivative["source_sheets_reported"]) == {"DGE_p", "MIMIC_p"},
            "reported DGE/MIMIC source sheets drift")
    vars_reported = derivative["processed_variables_reported"]
    require(len(vars_reported) == 2, "expected DGE and MIMIC period-average variables")
    by_var = {}
    for index, raw in enumerate(vars_reported):
        variable = exact_keys(raw, DERIVATIVE_VARIABLE_KEYS, f"derivative variable[{index}]")
        by_var[variable["id"]] = variable
    require(set(by_var) == {"dge_paverage_1993_2019", "mimic_paverage_1993_2019"},
            "processed derivative variable IDs drift")
    require(set(derivative["other_reported_variables"]) == {"economy", "ccode", "var4", "var5"},
            "processed derivative identifier columns drift")
    require(derivative["license_label_reported"] ==
            "Creative Commons Attribution 4.0 International License (CC BY 4.0)",
            "derivative-specific reported license drift")
    require(derivative["license_url"] == license_review["specific_evidence_url_points_to"],
            "derivative license URL drift")
    require(derivative["raw_or_derivative"] == "processed derivative; not the current official Excel workbook",
            "derivative/source distinction weakened")
    require(derivative["downloaded_in_this_review"] is False and
            derivative["source_snapshot_digest"] is None and
            derivative["digest_status"] == "not_captured",
            "do not claim derivative bytes/digest were captured")
    require(derivative["project_status"] ==
            "not_approved_for_import_until_package_asset_is_obtained_hashed_and_its_metadata_reconciled",
            "derivative import gate weakened")
    require(len(derivative["next_actions"]) == 5 and
            any("do not infer the raw workbook has the same rights" in item
                for item in derivative["next_actions"]),
            "derivative rights scope guard missing")

    provenance = exact_keys(source["provenance"], PROVENANCE_KEYS, "provenance")
    require(provenance["workbook_downloaded"] is False, "workbook acquisition must remain unclaimed")
    require(provenance["source_snapshot_id"] is None, "snapshot ID not captured")
    require(provenance["source_snapshot_digest"] is None, "do not fabricate a digest")
    require(provenance["digest_status"] == "not_captured", "digest status drift")
    require(provenance["raw_observations_embedded"] is False, "do not embed unqualified raw observations")
    require(provenance["observed_column_inventory"] is None, "do not claim an observed column inventory")

    gates = source["qualification_gates"]
    require(isinstance(gates, list) and len(gates) == len(EXPECTED_GATES), "qualification gate count drift")
    require(set(gates) == EXPECTED_GATES, "qualification gate removed, duplicated, or changed")
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
    print("PASS: World Bank source v3 preserves official metadata and cross-catalog evidence.")
    print("PASS: the 11-versus-12 measure count remains unresolved pending exact workbook inspection.")
    print("PASS: the CC BY 4.0 derivative candidate remains separate from the official Excel workbook rights gate.")
    print("PASS: availability evidence stays separate from project approval of redistribution/commercial use.")
    print("SCOPE: metadata contract only; no workbook values/digest or exact official sheet inventory.")
    print(f"OPEN: {count} source qualification gates; ingestion and redistribution remain unapproved.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
