#!/usr/bin/env python3
"""Verify the World Bank informal-economy source qualification record v8.

This checks source metadata and project approval gates only. It is not a workbook
parser, source-authenticity check, empirical replication, or legal opinion.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
SOURCE_PATH = ROOT / "data" / "economic_sources" / "world_bank_informal_economy_database.json"
ARCHIVED_V7_PATH = ROOT / "data" / "economic_sources" / "world_bank_informal_economy_database_v7.json"

TOP_KEYS = {
    "schema_version", "registry_id", "registry_version", "supersedes_registry_version", "edition_history_evidence", "data360_api_review",
    "record_kind", "status", "source_id", "publisher", "title", "canonical_url",
    "data_url", "source_page_date", "source_period_start", "source_period_end",
    "max_economies_claimed", "measure_count_claimed_by_source_page",
    "source_page_update_frequency", "source_page_claims", "primary_methodology_citation",
    "official_asset", "reproducibility_package_evidence", "license_review",
    "provenance", "qualification_gates", "interpretation_ceiling", "cross_catalog_evidence", "derivative_candidate", "license_evidence_matrix", "rights_adjudication", "package_access_scope", "asset_acquisition",
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
LICENSE_EVIDENCE_KEYS = {
    "evidence_id", "source_url", "source_title", "source_record_date", "named_dataset",
    "named_asset", "data_url", "access_policy_label", "license_label", "license_url",
    "evidence_strength", "caveat",
}
RIGHTS_ADJUDICATION_KEYS = {"status", "project_status", "not_a_legal_finding",
                            "reconciliation_note", "required_actions"}
PACKAGE_ACCESS_KEYS = {
    "package_id", "package_catalog_url", "package_zip", "package_zip_status",
    "package_data_and_code_status", "source_dataset_row_status", "source_dataset_filename",
    "source_dataset_url", "resolution",
}
ASSET_ACQUISITION_KEYS = {
    "official_page_url", "official_excel_url", "browser_attempt", "direct_container_attempt",
    "snapshot_id", "sha256", "byte_length", "workbook_inspected", "column_inventory",
    "data_ingestion_authorized", "note",
}
BROWSER_ATTEMPT_KEYS = {"attempted_on", "result", "detail"}
CONTAINER_ATTEMPT_KEYS = {
    "attempted_on", "result", "detail", "destination_path", "bytes_captured", "http_status",
}
EDITION_HISTORY_KEYS = {
    "evidence_id", "dataset_id", "dataset_title", "metadata_url", "api_documentation_url",
    "catalogue_search_url", "reviewed_on", "metadata_version_date", "source_period_start",
    "source_period_end", "measure_count_claimed", "catalogue_indicator_count",
    "measure_breakdown", "competing_official_brief", "exact_workbook_inventory_captured",
    "metadata_pdf_bytes_captured", "data_observations_captured", "interpretation", "access_note",
}
MEASURE_BREAKDOWN_KEYS = {
    "output_methods", "employment_indicators", "perception_indicators",
    "total_if_each_output_estimator_is_separate",
}
COMPETING_BRIEF_KEYS = {
    "url", "source_period_start", "source_period_end", "measure_count_claimed",
}
DATA360_API_REVIEW_KEYS = {
    "evidence_id", "api_root", "api_spec_url", "official_api_documentation_url",
    "official_terms_url", "reviewed_on", "documented_routes", "probe", "terms_review",
    "acquisition_protocol", "qualification_status",
}
API_ROUTE_KEYS = {"method", "path", "dataset_query_parameter", "purpose"}
API_DATA_ROUTE_KEYS = {
    "method", "path", "dataset_query_parameter", "indicator_query_parameter",
    "period_query_parameters", "pagination_parameter", "documented_max_records_per_response",
}
API_METADATA_ROUTE_KEYS = {"method", "path", "request_body_field", "query_language"}
API_PROBE_KEYS = {
    "attempted_on", "route_probe_result", "attempted_indicator_url", "http_status",
    "response_bytes_captured", "indicator_ids_captured", "data_observations_captured",
    "digest", "interpretation",
}
DATA360_TERMS_KEYS = {
    "default_label", "third_party_exception_disclosed", "specific_indicator_terms_must_be_checked",
    "official_terms_summary", "data360_api_or_metadata_ingestion_approved",
    "derived_data_redistribution_approved",
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
    "reconcile CC BY 4.0 evidence for informal-economy-database.xlsx with the Custom license catalogue entry",
    "distinguish parent-package embargo from the access policy for the separately linked official Excel dataset",
    "preserve the distinction between a catalogue's embargoed reproduction ZIP and its separately linked original dataset asset",
    "capture an HTTP response, workbook byte length and SHA-256 before claiming exact-asset verification",
    "reconcile the Data360 WB_INFECDB metadata claim of 12 measures over 1990–2018 against the official brief claim of 11 measures over 1990–2020 and the exact workbook edition",
    "qualify the World Bank Data360 API response and indicator-level terms: capture immutable paginated responses, counts, indicator IDs, digests, and any third-party reuse restrictions before importing observations",
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


def read_archived_v7() -> dict[str, Any]:
    try:
        value = json.loads(ARCHIVED_V7_PATH.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise ContractError(f"cannot load archived v7 source record: {error}") from error
    archived_keys = TOP_KEYS - {"data360_api_review"}
    return exact_keys(value, archived_keys, "archived_v7_source")


def validate_archived_v7(source: dict[str, Any]) -> int:
    require(type(source["schema_version"]) is int and source["schema_version"] == 1,
            "archived v7 schema version drift")
    require(source["registry_version"] == "world-bank-informal-economy-database/v7",
            "archived snapshot must remain v7")
    require(source["supersedes_registry_version"] == "world-bank-informal-economy-database/v6",
            "archived v7 history link drift")
    require(source["status"] == "metadata_only_not_approved_for_data_ingestion_or_redistribution",
            "archived v7 must remain metadata-only")
    require(source["source_period_start"] == 1990 and source["source_period_end"] == 2020,
            "archived official-brief period drift")
    require(source["measure_count_claimed_by_source_page"] == 11,
            "archived official-brief measure-count claim drift")
    require(source["source_page_claims"]["concept_inventory"]["count_reconciliation_status"] ==
            "unresolved_until_exact_workbook_inventory_and_edition_reconciliation",
            "archived v7 edition uncertainty must remain")
    gates = source["qualification_gates"]
    expected_v7_gates = EXPECTED_GATES - {
        "qualify the World Bank Data360 API response and indicator-level terms: capture immutable paginated responses, counts, indicator IDs, digests, and any third-party reuse restrictions before importing observations"
    }
    require(isinstance(gates, list) and len(gates) == 16 and set(gates) == expected_v7_gates,
            "archived v7 qualification gates were unexpectedly changed")
    require(source["edition_history_evidence"]["source_period_end"] == 2018
            and source["edition_history_evidence"]["measure_count_claimed"] == 12,
            "archived v7 must preserve Data360 edition evidence")
    require(source["asset_acquisition"]["sha256"] is None
            and source["asset_acquisition"]["data_ingestion_authorized"] is False,
            "archived v7 must not claim workbook acquisition or authorization")
    return len(gates)


def validate(source: dict[str, Any]) -> int:
    require(type(source["schema_version"]) is int and source["schema_version"] == 1,
            "unsupported record schema (schema_version must be integer 1; booleans are not integers here)")
    require(source["registry_id"] == "sol-atlas-economic-source-registry", "registry ID drift")
    require(source["registry_version"] == "world-bank-informal-economy-database/v8", "version drift")
    require(source["supersedes_registry_version"] == "world-bank-informal-economy-database/v7",
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
    require(source["source_page_date"] == "2024-01-09", "official source-page date drift")
    for field, expected in (
        ("source_period_start", 1990),
        ("source_period_end", 2020),
        ("max_economies_claimed", 196),
        ("measure_count_claimed_by_source_page", 11),
    ):
        require(type(source[field]) is int and source[field] == expected,
                f"{field} must remain integer {expected}")
    require(source["source_page_update_frequency"] == "every_two_years", "update-frequency claim drift")

    claims = exact_keys(source["source_page_claims"], CLAIMS_KEYS, "source_page_claims")
    expected_output_methods = {
        "dynamic_general_equilibrium_model", "multiple_indicators_multiple_causes_model"
    }
    require(isinstance(claims["output_methods"], list)
            and len(claims["output_methods"]) == 2
            and all(isinstance(item, str) for item in claims["output_methods"])
            and set(claims["output_methods"]) == expected_output_methods,
            "output method inventory content/type drift")

    expected_employment = {
        "self_employment", "informal_employment",
        "employment_outside_formal_sector", "labor_force_without_pension_insurance",
    }
    employment = claims["employment_indicators"]
    require(isinstance(employment, list)
            and len(employment) == 4
            and all(isinstance(item, str) for item in employment)
            and set(employment) == expected_employment,
            "employment inventory content/type drift")

    expected_perception = {
        ("World Economic Forum Executive Opinion Surveys", 1),
        ("World Bank Enterprise Surveys", 4),
        ("World Values Survey", 1),
    }
    perception = claims["perception_sources"]
    require(isinstance(perception, list) and len(perception) == 3,
            "perception source inventory shape drift")
    perception_pairs = set()
    for index, raw in enumerate(perception):
        item = exact_keys(raw, {"source", "count"}, f"perception_sources[{index}]")
        require(isinstance(item["source"], str) and item["source"].strip(),
                f"perception_sources[{index}].source must be non-empty text")
        require(type(item["count"]) is int and item["count"] > 0,
                f"perception_sources[{index}].count must be a positive integer")
        perception_pairs.add((item["source"], item["count"]))
    require(perception_pairs == expected_perception,
            "perception source inventory drift")
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
    require(inventory["count_reconciliation_status"] ==
            "unresolved_until_exact_workbook_inventory_and_edition_reconciliation",
            "do not prematurely resolve the 11-versus-12 measure and edition question")
    require("exact official workbook edition" in inventory["note"] and
            "do not select an interpretation" in inventory["note"],
            "uncertainty about source's intended count must remain explicit")
    require("actual official workbook edition" in claims["count_reconciliation"],
            "workbook-level qualification requirement missing")
    require("1990–2018" in claims["count_reconciliation"]
            and "1990–2020" in claims["count_reconciliation"]
            and "edition" in claims["count_reconciliation"].lower(),
            "alternate World Bank Data360 metadata edition must remain part of count reconciliation")

    edition = exact_keys(source["edition_history_evidence"], EDITION_HISTORY_KEYS,
                         "edition_history_evidence")
    require(edition["evidence_id"] == "world_bank_data360_wb_infecdb_metadata_alternate_edition",
            "alternate Data360 edition evidence ID drift")
    require(edition["dataset_id"] == "WB_INFECDB"
            and edition["dataset_title"] == "Informal Economy Database",
            "Data360 dataset identity drift")
    require(edition["metadata_url"] ==
            "https://data360files.worldbank.org/data360-data/datasetmetadata/WB_INFECDB.pdf",
            "Data360 metadata URL drift")
    require(edition["api_documentation_url"] == "https://data360.worldbank.org/en/api",
            "Data360 API documentation URL drift")
    require(edition["catalogue_search_url"] == "https://data360.worldbank.org/en/search",
            "Data360 catalogue URL drift")
    require(edition["reviewed_on"] == "2026-10-13" and edition["metadata_version_date"] is None,
            "unknown metadata edition date must remain explicit")
    for field, expected in (
        ("source_period_start", 1990),
        ("source_period_end", 2018),
        ("measure_count_claimed", 12),
        ("catalogue_indicator_count", 12),
    ):
        require(type(edition[field]) is int and edition[field] == expected,
                f"Data360 edition {field} must remain integer {expected}")
    breakdown = exact_keys(edition["measure_breakdown"], MEASURE_BREAKDOWN_KEYS,
                           "edition_history_evidence.measure_breakdown")
    for field, expected in (
        ("output_methods", 2),
        ("employment_indicators", 4),
        ("perception_indicators", 6),
        ("total_if_each_output_estimator_is_separate", 12),
    ):
        require(type(breakdown[field]) is int and breakdown[field] == expected,
                f"Data360 measure breakdown {field} must remain integer {expected}")
    competing = exact_keys(edition["competing_official_brief"], COMPETING_BRIEF_KEYS,
                           "edition_history_evidence.competing_official_brief")
    require(competing["url"] == source["canonical_url"],
            "competing source brief must match canonical official page")
    for field, expected in (
        ("source_period_start", 1990),
        ("source_period_end", 2020),
        ("measure_count_claimed", 11),
    ):
        require(type(competing[field]) is int and competing[field] == expected,
                f"competing official brief {field} must remain integer {expected}")
    require(edition["exact_workbook_inventory_captured"] is False
            and edition["metadata_pdf_bytes_captured"] is False
            and edition["data_observations_captured"] is False,
            "do not overstate Data360 metadata or observation acquisition")
    require("not proof of the exact current workbook inventory" in edition["interpretation"],
            "Data360 metadata must not be conflated with the current workbook")
    require("not successfully queried" in edition["access_note"],
            "Data360 API retrieval limitation must remain explicit")

    api = exact_keys(source["data360_api_review"], DATA360_API_REVIEW_KEYS, "data360_api_review")
    require(api["evidence_id"] == "world_bank_data360_open_api_v1",
            "Data360 API evidence ID drift")
    require(api["api_root"] == "https://data360api.worldbank.org",
            "Data360 API root drift")
    require(api["api_spec_url"] ==
            "https://raw.githubusercontent.com/worldbank/open-api-specs/refs/heads/main/Data360%20Open_API.json",
            "Data360 OpenAPI specification URL drift")
    require(api["official_api_documentation_url"] == "https://data360.worldbank.org/en/api",
            "Data360 API documentation URL drift")
    require(api["official_terms_url"] == "https://data360.worldbank.org/en/about",
            "Data360 terms URL drift")
    require(api["reviewed_on"] == "2026-10-13"
            and api["qualification_status"] == "documented_acquisition_route_not_execution_verified",
            "Data360 API execution status must stay unqualified")

    routes = exact_keys(api["documented_routes"], {
        "list_indicator_ids", "retrieve_observations", "retrieve_metadata"
    }, "data360_api_review.documented_routes")
    indicator_route = exact_keys(routes["list_indicator_ids"], API_ROUTE_KEYS,
                                 "Data360 API indicator route")
    require(indicator_route["method"] == "GET"
            and indicator_route["path"] == "/data360/indicators"
            and indicator_route["dataset_query_parameter"] == "datasetId",
            "Data360 indicator discovery route drift")
    require("discover published series" in indicator_route["purpose"],
            "indicator discovery purpose missing")
    data_route = exact_keys(routes["retrieve_observations"], API_DATA_ROUTE_KEYS,
                            "Data360 API data route")
    require(data_route["method"] == "GET"
            and data_route["path"] == "/data360/data"
            and data_route["dataset_query_parameter"] == "DATABASE_ID"
            and data_route["indicator_query_parameter"] == "INDICATOR",
            "Data360 observation route identity drift")
    require(data_route["period_query_parameters"] == ["timePeriodFrom", "timePeriodTo", "TIME_PERIOD"],
            "Data360 bounded period query parameters drift")
    require(data_route["pagination_parameter"] == "skip"
            and type(data_route["documented_max_records_per_response"]) is int
            and data_route["documented_max_records_per_response"] == 1000,
            "Data360 pagination contract must preserve the documented 1000-row limit")
    metadata_route = exact_keys(routes["retrieve_metadata"], API_METADATA_ROUTE_KEYS,
                                "Data360 API metadata route")
    require(metadata_route["method"] == "POST"
            and metadata_route["path"] == "/data360/metadata"
            and metadata_route["request_body_field"] == "query",
            "Data360 metadata route identity drift")
    require("OData-like" in metadata_route["query_language"],
            "Data360 metadata query semantics missing")

    probe = exact_keys(api["probe"], API_PROBE_KEYS, "data360_api_review.probe")
    require(probe["attempted_on"] == "2026-10-13"
            and probe["route_probe_result"] == "web_reader_refused_direct_api_url",
            "do not misstate the direct API probe result")
    require(probe["attempted_indicator_url"] ==
            "https://data360api.worldbank.org/data360/indicators?datasetId=WB_INFECDB",
            "Data360 indicator probe URL drift")
    require(probe["http_status"] is None and probe["digest"] is None
            and probe["response_bytes_captured"] is False
            and probe["indicator_ids_captured"] is False
            and probe["data_observations_captured"] is False,
            "do not claim Data360 HTTP status, response bytes, indicator IDs, data, or digest")
    require("does not establish that the World Bank endpoint is unavailable" in probe["interpretation"],
            "browser retrieval restriction must not be described as an API outage")

    terms = exact_keys(api["terms_review"], DATA360_TERMS_KEYS, "data360_api_review.terms_review")
    require(terms["default_label"] ==
            "Creative Commons Attribution 4.0 International License (CC BY 4.0), unless specifically labelled otherwise",
            "Data360 default licensing language drift")
    require(terms["third_party_exception_disclosed"] is True
            and terms["specific_indicator_terms_must_be_checked"] is True,
            "Data360 third-party licensing exception must remain explicit")
    require(terms["data360_api_or_metadata_ingestion_approved"] is False
            and terms["derived_data_redistribution_approved"] is False,
            "Data360 rights must not be treated as project approval")
    require("do not by themselves resolve rights" in terms["official_terms_summary"],
            "Data360 terms scope must remain separate from workbook rights")
    protocol = api["acquisition_protocol"]
    require(isinstance(protocol, list) and len(protocol) == 8
            and len(set(protocol)) == 8
            and all(isinstance(item, str) and item.strip() for item in protocol),
            "Data360 acquisition protocol must contain eight distinct, non-empty actions")
    require(any("immutable page response" in action for action in protocol),
            "immutable per-page response retention requirement missing")
    require(any("1000 records per response" in action for action in protocol),
            "1000-record pagination requirement missing")
    require(any("third-party exceptions" in action for action in protocol),
            "indicator-level reuse terms gate missing")

    citation = exact_keys(source["primary_methodology_citation"], CITATION_KEYS, "primary_methodology_citation")
    require(type(citation["year"]) is int and citation["year"] == 2021,
            "methodology citation year must be integer 2021")
    require(citation["authors"] == [
        "Ceyhun Elgin", "M. Ayhan Kose", "Franziska Ohnsorge", "Shu Yu"
    ], "methodology citation authors drift")
    require(citation["title"] == "Understanding Informality",
            "methodology citation title drift")
    require(citation["publication"] == "CERP Discussion Paper 16497",
            "methodology publication identifier drift")
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
    require(license_review["rights_decision"] ==
            "conflicting_secondary_catalog_labels_exact_workbook_rights_not_yet_adjudicated",
            "catalog conflict must not be resolved by assumption")
    require(license_review["official_page_download_present"] is True, "official download evidence missing")
    require(license_review["publicly_available_in_reproducibility_package"] is True,
            "public availability evidence missing")
    require(license_review["dataset_license_label"] ==
            "conflicting_metadata_cc_by_4_0_and_custom_license", "catalog-label conflict was lost")
    require(license_review["redistribution_approved"] is False, "project has not approved redistribution")
    require(license_review["commercial_use_approved"] is False, "project has not approved commercial use")
    require(license_review["approval_status"] ==
            "not_approved_pending_exact_asset_and_terms_reconciliation", "project approval state drift")
    require(license_review["specific_evidence_url"] == "https://reproducibility.worldbank.org/catalog/265",
            "explicit CC BY 4.0 evidence URL missing")
    require("CC BY 4.0" in license_review["specific_evidence_label"] and
            "Custom license" in license_review["specific_evidence_label"],
            "both conflicting catalogue labels must be recorded")
    require(license_review["specific_evidence_url_points_to"] ==
            "https://datacatalog.worldbank.org/public-licenses#cc-by", "CC BY license URL drift")
    require("not a legal finding" in license_review["interpretation_limit"],
            "do not misstate project approval status as legal prohibition")

    evidence = source["license_evidence_matrix"]
    require(isinstance(evidence, list) and len(evidence) == 3, "expected three separate license evidence records")
    by_evidence = {}
    for index, raw in enumerate(evidence):
        item = exact_keys(raw, LICENSE_EVIDENCE_KEYS, f"license_evidence_matrix[{index}]")
        require(isinstance(item["evidence_id"], str) and item["evidence_id"].strip(),
                f"license_evidence_matrix[{index}].evidence_id must be non-empty text")
        require(item["evidence_id"] not in by_evidence, "duplicate license evidence ID")
        by_evidence[item["evidence_id"]] = item
        require(item["data_url"] == "https://www.worldbank.org/en/research/brief/informal-economy-database",
                "catalogue entry should resolve to official source page")
    require(set(by_evidence) == {
        "world_bank_catalog_jobless_development_265",
        "world_bank_catalog_informalizing_jobs_330",
        "world_bank_catalog_sme_training_583",
    }, "license evidence IDs differ from the three reviewed source records")
    ccby = by_evidence["world_bank_catalog_jobless_development_265"]
    require(ccby["named_asset"] == "informal-economy-database.xlsx", "specific Excel asset filename drift")
    require(ccby["license_label"] == "Creative Commons Attribution 4.0 International License",
            "explicit CC BY evidence was weakened")
    require(ccby["license_url"] == "https://datacatalog.worldbank.org/public-licenses#cc-by",
            "explicit CC BY terms link drift")
    require("same catalogue page states in its package-level overview" in ccby["caveat"],
            "parent package embargo caveat missing")
    custom = by_evidence["world_bank_catalog_sme_training_583"]
    require(custom["named_asset"] == "DGE_Raw.xlsx" and custom["license_label"] == "Custom license",
            "Custom license evidence was lost")
    require(custom["license_url"] == "https://www.worldbank.org/en/about/legal/terms-of-use-for-datasets",
            "Custom license evidence URL drift")
    derivative_evidence = by_evidence["world_bank_catalog_informalizing_jobs_330"]
    require(derivative_evidence["named_asset"] == "informal-economy-database-with-fixes.dta",
            "processed derivative evidence was confused with raw workbook")
    require(derivative_evidence["evidence_strength"] == "processed_derivative_license_record",
            "derivative licence evidence scope drift")

    adjudication = exact_keys(source["rights_adjudication"], RIGHTS_ADJUDICATION_KEYS, "rights_adjudication")
    require(adjudication["status"] == license_review["rights_decision"],
            "rights adjudication status and source review status differ")
    require("does not assert that the dataset is legally prohibited" in adjudication["not_a_legal_finding"],
            "project approval gate must not be described as legal prohibition")
    require(len(adjudication["required_actions"]) == 7,
            "rights reconciliation actions must remain explicit")
    require(len(set(adjudication["required_actions"])) == len(adjudication["required_actions"]),
            "rights reconciliation actions must not be duplicated")
    require(any("Custom license entry" in action and "reproduction catalogue" in action
                for action in adjudication["required_actions"]),
            "reconciliation of catalogue-label conflict missing")
    require(any("embargoed parent ZIP" in action for action in adjudication["required_actions"]),
            "parent-package embargo and source asset scope must remain separate")

    package_scope = exact_keys(source["package_access_scope"], PACKAGE_ACCESS_KEYS, "package_access_scope")
    require(package_scope["package_id"] == "RR_WLD_2025_286", "catalog parent package ID drift")
    require(package_scope["package_zip_status"] == "temporarily_embargoed_per_package_overview",
            "parent package embargo must remain explicit")
    require(package_scope["source_dataset_row_status"] == "publicly_available_and_included_per_dataset_row",
            "source row's separate availability claim was lost")
    require(package_scope["source_dataset_filename"] == "informal-economy-database.xlsx",
            "official dataset filename drift")
    require("scope ambiguity" in package_scope["resolution"],
            "catalog's conflicting scope language must be recorded")

    acquisition = exact_keys(source["asset_acquisition"], ASSET_ACQUISITION_KEYS, "asset_acquisition")
    require(acquisition["official_page_url"] == "https://www.worldbank.org/en/brief/2024/01/09/informal-economy-database",
            "official source page URL drift")
    require(acquisition["official_excel_url"] == source["data_url"], "official Excel URL mismatch")
    browser = exact_keys(acquisition["browser_attempt"], BROWSER_ATTEMPT_KEYS, "asset_acquisition.browser_attempt")
    require(browser["result"] == "reader_rejected_binary_workbook_content_type",
            "do not hide the browser reader limitation")
    require("not a source HTTP failure" in browser["detail"],
            "browser parser limitation must not be attributed to the World Bank host")
    direct = exact_keys(acquisition["direct_container_attempt"], CONTAINER_ATTEMPT_KEYS,
                        "asset_acquisition.direct_container_attempt")
    require(direct["result"] == "failed_dns_resolution_in_execution_environment",
            "direct fetch attempt status drift")
    require("Could not resolve host" in direct["detail"] and direct["http_status"] is None,
            "DNS failure must not be represented as a remote HTTP response")
    require(direct["bytes_captured"] is False, "do not claim workbook bytes were captured")
    require(acquisition["snapshot_id"] is None and acquisition["sha256"] is None and
            acquisition["byte_length"] is None and acquisition["workbook_inspected"] is False and
            acquisition["column_inventory"] is None,
            "do not fabricate source snapshot, digest, length, or workbook inventory")
    require(acquisition["data_ingestion_authorized"] is False,
            "source ingestion must remain blocked until qualification is completed")

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
    require(derivative["license_url"] == "https://www.worldbank.org/en/about/legal/terms-of-use-for-datasets",
            "derivative-specific license URL drift")
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
            len(set(derivative["next_actions"])) == len(derivative["next_actions"]) and
            any("do not infer the raw workbook has the same rights" in item
                for item in derivative["next_actions"]),
            "derivative rights scope guard missing or duplicated")

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
        archived_count = validate_archived_v7(read_archived_v7())
    except ContractError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        return 1
    print("PASS: World Bank source v8 preserves official metadata, edition evidence, and Data360 acquisition gates.")
    print(f"PASS: archived v7 remains unchanged in scope with {archived_count} qualification gates.")
    print("PASS: the 11-versus-12 and 1990–2018-versus-1990–2020 metadata differences remain unresolved pending exact workbook/edition inspection.")
    print("PASS: Data360 routes, 1000-record pagination, direct-probe limitation, and third-party terms remain explicit.")
    print("PASS: CC BY 4.0 and Custom license catalogue evidence remain distinct and unresolved for the exact workbook.")
    print("PASS: the processed derivative's CC BY 4.0 metadata remains separate from the official workbook rights gate.")
    print("PASS: parent-package embargo, source-row availability, and environment-level fetch failure are distinct.")
    print("SCOPE: workbook bytes/digest/sheets unavailable in this environment; data ingestion remains blocked.")
    print("PASS: availability evidence stays separate from project approval of redistribution/commercial use.")
    print("SCOPE: metadata contract only; no workbook values/digest or exact official sheet inventory.")
    print(f"OPEN: {count} source qualification gates; ingestion and redistribution remain unapproved.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
