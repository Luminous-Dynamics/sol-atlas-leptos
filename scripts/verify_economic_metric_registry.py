#!/usr/bin/env python3
"""Validate the source-attributed South Africa QLFS economic metric registry.

This is a deterministic contract verifier, not a QLFS estimator and not a
license or source-authenticity validator. It intentionally checks that
published headline percentages remain separate and that missing source counts
are not reverse-engineered from rounded percentages.
"""

from __future__ import annotations

import json
import re
import sys
from datetime import date
from pathlib import Path
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
REGISTRY_PATH = ROOT / "data" / "economic_metrics" / "south_africa_qlfs_q4_2025_v1.json"

TOP_KEYS = {
    "schema_version",
    "registry_id",
    "registry_version",
    "registry_kind",
    "status",
    "note",
    "sources",
    "comparability_events",
    "metrics",
}
SOURCE_KEYS = {
    "source_id",
    "publisher",
    "title",
    "source_url",
    "publication_date",
    "source_period",
    "related_data_portal_url",
    "source_snapshot_id",
    "source_snapshot_digest",
    "digest_status",
    "acquisition_method",
    "reuse_status",
    "review_note",
}
EVENT_KEYS = {
    "event_id",
    "event_kind",
    "effective_period",
    "source_id",
    "source_url",
    "description",
    "impacted_metric_ids",
    "comparability_policy",
    "display_warning",
}
METRIC_KEYS = {
    "metric_id",
    "definition_version",
    "label",
    "value",
    "unit",
    "value_kind",
    "unit_of_analysis",
    "account_boundary",
    "recording_state",
    "formality_dimension",
    "legal_assessment_state",
    "numerator_definition",
    "numerator_value",
    "denominator_definition",
    "denominator_value",
    "geography",
    "reference_period",
    "source_publication_date",
    "known_at",
    "source_id",
    "source_definition_id",
    "source_snapshot_id",
    "source_snapshot_digest",
    "source_url",
    "source_method",
    "uncertainty",
    "reuse_status",
    "comparability_event_ids",
    "aggregation_policy",
    "interpretation_ceiling",
}
GEO_KEYS = {"name", "code_system", "code"}
UNCERTAINTY_KEYS = {"published_interval", "note"}
EXPECTED_VALUES = {
    "informal_sector_share_of_jobs": "21.4",
    "informal_employment_share_of_employed_people": "33.5",
}
DECIMAL = re.compile(r"-?(?:0|[1-9][0-9]*)(?:\.[0-9]+)?\Z")


class ContractError(Exception):
    """A registry invariant was violated."""


def require(condition: bool, message: str) -> None:
    if not condition:
        raise ContractError(message)


def exact_keys(value: Any, expected: set[str], where: str) -> dict[str, Any]:
    require(isinstance(value, dict), f"{where} must be a JSON object")
    actual = set(value)
    require(actual == expected, (
        f"{where} keys differ: missing={sorted(expected - actual)}, "
        f"unexpected={sorted(actual - expected)}"
    ))
    return value


def nonempty_text(value: Any, where: str) -> str:
    require(isinstance(value, str) and bool(value.strip()), f"{where} must be non-empty text")
    require(value == value.strip(), f"{where} must not contain surrounding whitespace")
    return value


def iso_date(value: Any, where: str) -> date:
    text = nonempty_text(value, where)
    try:
        parsed = date.fromisoformat(text)
    except ValueError as error:
        raise ContractError(f"{where} must be YYYY-MM-DD") from error
    require(parsed.isoformat() == text, f"{where} must use canonical YYYY-MM-DD form")
    return parsed


def read_registry() -> dict[str, Any]:
    try:
        registry = json.loads(REGISTRY_PATH.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as error:
        raise ContractError(f"cannot load registry: {error}") from error
    return exact_keys(registry, TOP_KEYS, "registry")


def validate_registry(registry: dict[str, Any]) -> int:
    require(registry["schema_version"] == 1, "unsupported metric-registry schema_version")
    require(registry["registry_id"] == "sol-atlas-economic-metric-registry", "registry identity drift")
    require(registry["registry_version"] == "south-africa-qlfs-q4-2025/v1", "registry version drift")
    require(registry["registry_kind"] == "source_attributed_aggregate_observations", "registry kind drift")
    require(registry["status"] == "research_seed_not_a_complete_qlfs_extract", "do not overstate registry completeness")
    note = nonempty_text(registry["note"], "registry.note")
    require("deliberately null" in note, "registry must explain why source counts are null")

    sources = registry["sources"]
    require(isinstance(sources, list) and len(sources) == 1, "expected exactly one reviewed headline-summary source")
    source = exact_keys(sources[0], SOURCE_KEYS, "sources[0]")
    require(source["source_id"] == "stats_sa_qlfs_q4_2025_summary", "source ID drift")
    require(source["publisher"] == "Statistics South Africa (Stats SA)", "source publisher drift")
    require(source["source_url"] == "https://www.statssa.gov.za/?p=19240", "unexpected Stats SA source URL")
    require(source["related_data_portal_url"] == "https://isibaloweb.statssa.gov.za/pages/surveys/pss/qlfs/2025/qlfs2025.php", "unexpected QLFS data portal")
    require(iso_date(source["publication_date"], "source.publication_date") == date(2026, 3, 4), "source publication date drift")
    require(source["source_period"] == "2025-Q4", "source period drift")
    require(source["source_snapshot_digest"] is None, "no source-body digest was captured; do not fabricate one")
    require(source["digest_status"] == "not_captured", "source digest state drift")
    require(source["reuse_status"] == "terms_review_required", "unreviewed reuse terms must remain explicit")
    require("public access alone" in source["review_note"], "source-use caveat missing")

    metrics = registry["metrics"]
    require(isinstance(metrics, list) and len(metrics) == 2, "expected exactly two pilot metrics")
    by_id: dict[str, dict[str, Any]] = {}
    for index, raw in enumerate(metrics):
        metric = exact_keys(raw, METRIC_KEYS, f"metrics[{index}]")
        metric_id = nonempty_text(metric["metric_id"], f"metrics[{index}].metric_id")
        require(metric_id not in by_id, f"duplicate metric_id: {metric_id}")
        by_id[metric_id] = metric
    require(set(by_id) == set(EXPECTED_VALUES), f"metric ID inventory drifted: {sorted(by_id)}")

    for metric_id, expected_value in EXPECTED_VALUES.items():
        metric = by_id[metric_id]
        require(metric["value"] == expected_value, f"{metric_id} published value drift")
        require(metric["unit"] == "percent", f"{metric_id} unit must remain percent")
        require(isinstance(metric["value"], str) and DECIMAL.fullmatch(metric["value"]) is not None,
                f"{metric_id} value must be a plain-decimal source token")
        require(metric["source_id"] == source["source_id"], f"{metric_id} source mismatch")
        require(metric["source_url"] == source["source_url"], f"{metric_id} source URL mismatch")
        require(metric["source_snapshot_id"] == source["source_snapshot_id"], f"{metric_id} source snapshot mismatch")
        require(metric["source_snapshot_digest"] is None, f"{metric_id} invented a source-body digest")
        require(metric["reuse_status"] == "terms_review_required", f"{metric_id} reuse gate was weakened")
        require(metric["reference_period"] == "2025-Q4", f"{metric_id} period mismatch")
        require(iso_date(metric["source_publication_date"], f"{metric_id}.source_publication_date") == date(2026, 3, 4),
                f"{metric_id} publication date mismatch")
        iso_date(metric["known_at"], f"{metric_id}.known_at")
        require(metric["known_at"] >= metric["source_publication_date"], f"{metric_id} known_at precedes publication")
        require(metric["geography"] == {
            "name": "South Africa",
            "code_system": "ISO 3166-1 alpha-2",
            "code": "ZA",
        }, f"{metric_id} geography drift")
        exact_keys(metric["geography"], GEO_KEYS, f"{metric_id}.geography")
        exact_keys(metric["uncertainty"], UNCERTAINTY_KEYS, f"{metric_id}.uncertainty")
        require(metric["uncertainty"]["published_interval"] is None,
                f"{metric_id} must not invent an uncertainty interval absent from the headline source")
        require(metric["numerator_value"] is None and metric["denominator_value"] is None,
                f"{metric_id} must not reverse-engineer counts from rounded percentages")
        require(metric["legal_assessment_state"] == "not_assessed",
                f"{metric_id} must not infer a legal state from an employment statistic")
        require(metric["aggregation_policy"] == "non_aggregable_with_other_metric_ids",
                f"{metric_id} aggregation guard drift")
        require(metric["interpretation_ceiling"], f"{metric_id} needs an explicit interpretation ceiling")

    sector = by_id["informal_sector_share_of_jobs"]
    employment = by_id["informal_employment_share_of_employed_people"]
    require(sector["denominator_definition"] != employment["denominator_definition"],
            "the two metrics must retain distinct denominator definitions")
    require(sector["value_kind"] != employment["value_kind"],
            "the two metrics must retain distinct economic concepts")
    require(sector["account_boundary"] != employment["account_boundary"],
            "the two metrics must retain distinct account boundaries")
    require(sector["unit_of_analysis"] != employment["unit_of_analysis"],
            "the two metrics must retain distinct units of analysis")
    require(sector["numerator_value"] is None and employment["numerator_value"] is None,
            "headline rates do not justify synthesizing integer counts")

    events = registry["comparability_events"]
    require(isinstance(events, list) and len(events) == 1, "expected one explicit informality definition event")
    event = exact_keys(events[0], EVENT_KEYS, "comparability_events[0]")
    require(event["event_id"] == "stats_sa_informality_definition_revision_2025_q3", "definition event ID drift")
    require(event["event_kind"] == "definition_change", "definition event kind drift")
    require(event["effective_period"] == "2025-Q3", "Q3 2025 definition break was lost")
    require(event["source_id"] == source["source_id"] and event["source_url"] == source["source_url"],
            "definition event source mismatch")
    require(event["impacted_metric_ids"] == ["informal_sector_share_of_jobs"],
            "do not silently generalize the enterprise-definition change to every labour metric")
    require(event["comparability_policy"] == "break_requires_bridge", "definition break policy weakened")
    require("without a documented comparability bridge" in event["display_warning"],
            "definition-break warning missing")
    require(sector["comparability_event_ids"] == [event["event_id"]], "sector metric must reference its definition break")
    require(employment["comparability_event_ids"] == [], "do not attach unsupported definition-event impacts to informal-employment metric")

    for metric_id, metric in by_id.items():
        require(all(event_id in {event["event_id"] for event in events}
                    for event_id in metric["comparability_event_ids"]),
                f"{metric_id} references an unknown comparability event")

    return len(metrics)


def main() -> int:
    try:
        count = validate_registry(read_registry())
    except ContractError as error:
        print(f"FAIL: {error}", file=sys.stderr)
        return 1
    print(f"PASS: {count} source-attributed QLFS aggregate metrics satisfy the versioned registry contract.")
    print("PASS: distinct denominators/concepts, explicit Q3 2025 definition break, and no fabricated counts/digest.")
    print("SCOPE: contract validation only; no QLFS microdata recomputation, sampling-error calculation, or licence determination.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
