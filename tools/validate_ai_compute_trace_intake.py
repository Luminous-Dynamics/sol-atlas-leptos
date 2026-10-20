#!/usr/bin/env python3
"""Conformance and fail-closed mutation tests for the trace-intake JSON Schema.

This is development/CI tooling only; production Sol-Atlas code remains Rust.
The constructed positive control is fictional schema input, not a dataset,
meter record, license review, or empirical benchmark result.
"""
from __future__ import annotations

import copy
import json
from pathlib import Path
import sys
from typing import Any

from jsonschema import Draft202012Validator, FormatChecker

ROOT = Path(__file__).resolve().parents[1]
SCHEMA_PATH = ROOT / "schemas/ai-compute-energy-trace-intake-v1.schema.json"
PLANNED_PATHS = (
    ROOT / "schemas/examples/ai-compute-energy-trace-intake-v1.planned.json",
    ROOT / "schemas/examples/google-clusterdata2019-power-trace-intake.planned.json",
)


def load_json(path: Path) -> Any:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        raise AssertionError(f"Cannot load {path.relative_to(ROOT)}: {exc}") from exc


def error_summary(validator: Draft202012Validator, instance: Any) -> str:
    errors = sorted(
        validator.iter_errors(instance),
        key=lambda error: (
            tuple(str(part) for part in error.absolute_path),
            tuple(str(part) for part in error.absolute_schema_path),
        ),
    )
    return "\n".join(
        f"- instance /{'/'.join(map(str, e.absolute_path))}: {e.message}"
        for e in errors[:12]
    )


def expect_valid(
    name: str, validator: Draft202012Validator, instance: Any
) -> None:
    summary = error_summary(validator, instance)
    if summary:
        raise AssertionError(f"{name} must validate but did not:\n{summary}")
    print(f"PASS {name}")


def expect_invalid(
    name: str, validator: Draft202012Validator, instance: Any
) -> None:
    summary = error_summary(validator, instance)
    if not summary:
        raise AssertionError(f"{name} unexpectedly validated")
    print(f"PASS {name} (rejected)")


def positive_control(planned: dict[str, Any]) -> dict[str, Any]:
    """Build a synthetic, CI-only fixture for the highest-evidence schema path."""
    item = copy.deepcopy(planned)
    item["intake_id"] = "ci-only-positive-control-not-data"
    item["dataset"].update(
        {
            "name": "CI-only structural positive control (not a dataset)",
            "publisher": "Sol-Atlas schema validation test",
            "release": "fictional-positive-control",
            "source_kind": "metered_experiment",
            "source_uri": "https://example.invalid/ci-only-positive-control",
            "description": (
                "All fields in this in-memory object are fictional test values. "
                "It is not a dataset, metering record, license determination, "
                "or empirical benchmark result."
            ),
        }
    )
    item["license_gate"] = {
        "status": "approved",
        "decision": "approved_for_benchmark",
        "license_identifier": "CI-ONLY-NOT-A-REAL-LICENSE-REVIEW",
        "terms_uri": "https://example.invalid/ci-only-license-terms",
        "reviewed_at": "2026-10-20T00:00:00Z",
        "review_note": (
            "Fictional schema-control value; no license review was conducted."
        ),
    }
    item["artifact"] = {
        "state": "normalized",
        "raw_sha256": "a" * 64,
        "raw_bytes": 100,
        "retrieved_at": "2026-10-20T00:00:00Z",
        "normalized_sha256": "b" * 64,
        "normalized_bytes": 80,
    }
    item["time_scope"] = {
        "timestamp_semantics": "event_time",
        "start_utc": "2026-10-01T00:00:00Z",
        "end_utc": "2026-10-02T00:00:00Z",
        "source_timezone": "UTC",
        "source_time_unit": "seconds",
        "aggregation_interval_seconds": 60,
        "notes": "Fictional CI positive-control timestamps; not trace data.",
    }
    for index, mapping in enumerate(item["field_mappings"]):
        mapping.update(
            {
                "mapping_status": "mapped",
                "source_field": f"ci_only_source_field_{index}",
                "source_unit": "fictional-unit",
                "conversion_rule": "identity",
                "records_checked": 10,
                "records_missing": 0,
                "notes": "Fictional field mapping; not evidence from a dataset.",
            }
        )
    item["workload_quality"] = {
        "status": "measured",
        "quality_metric": "CI-only placeholder; no measurement",
        "acceptance_rule": "CI-only structural branch coverage rule",
        "model_or_job_identity": "ci-only-positive-control-v1",
        "notes": "Fictional test values only; no workload was run.",
    }
    item["energy_attribution"] = {
        "class": "metered",
        "boundary": "facility",
        "metering_method": "CI-only placeholder; no metering performed",
        "instrument_or_model": "CI-only placeholder; not a real instrument",
        "uncertainty_percent": 1.0,
        "notes": "Fictional test values only; no energy was measured.",
    }
    item["data_quality"] = {
        "stage": "normalized",
        "records_read": 10,
        "records_kept": 9,
        "records_excluded": 1,
        "duplicate_records": 0,
        "missing_timestamps": 0,
        "out_of_scope_timestamps": 0,
        "measured_missingness_fraction": 0.0,
        "checks_run": ["ci-only-structural-check"],
        "notes": "Fictional counts solely for JSON Schema branch coverage.",
    }
    item["transformation"] = {
        "pipeline_id": "ci-only-not-a-transformer",
        "pipeline_version": "0.0.0-test-only",
        "source_commit": "c" * 40,
        "config_sha256": "d" * 64,
        "normalization_notes": "No transform ran; this is a schema test object.",
    }
    item["claim_policy"] = {
        "claim_ceiling": "quality_accepted_work_per_kwh",
        "allowed_claims": [
            "Schema positive-control only; not a quantitative benchmark result."
        ],
        "prohibited_inferences": list(
            planned["claim_policy"]["prohibited_inferences"]
        ),
    }
    return item


def main() -> int:
    schema = load_json(SCHEMA_PATH)
    planned_instances = [(path, load_json(path)) for path in PLANNED_PATHS]

    # Validate the schema document against its declared Draft 2020-12 meta-schema.
    Draft202012Validator.check_schema(schema)
    validator = Draft202012Validator(
        schema,
        format_checker=FormatChecker(),
    )
    print("PASS schema is valid Draft 2020-12")

    for path, planned in planned_instances:
        expect_valid(path.relative_to(ROOT).as_posix(), validator, planned)

    # Use Alibaba's compact fixture as a template for a wholly fictional positive
    # control. This intentionally does not certify the Google trace's power or
    # license metadata and is never used as empirical evidence.
    planned = planned_instances[0][1]
    expect_valid(
        "fictional positive-control schema path (not real evidence)",
        validator,
        positive_control(planned),
    )

    mutation = copy.deepcopy(planned)
    mutation["claim_policy"]["claim_ceiling"] = "quality_accepted_work_per_kwh"
    expect_invalid(
        "planned artifact cannot claim measured quality-accepted efficiency",
        validator,
        mutation,
    )

    mutation = positive_control(planned)
    mutation["dataset"]["source_kind"] = "synthetic"
    expect_invalid(
        "synthetic source cannot support metered/high-evidence claim ceiling",
        validator,
        mutation,
    )

    mutation = positive_control(planned)
    mutation["license_gate"].update(
        {"status": "not_reviewed", "decision": "metadata_only"}
    )
    expect_invalid(
        "high-evidence claims require approved license review and terms",
        validator,
        mutation,
    )

    mutation = positive_control(planned)
    mutation["field_mappings"][0]["mapping_status"] = "planned"
    expect_invalid(
        "quality-accepted claim requires mapped canonical fields",
        validator,
        mutation,
    )

    mutation = positive_control(planned)
    mutation["artifact"]["state"] = "retrieved"
    expect_invalid(
        "normalized evidence cannot be mislabeled as merely retrieved",
        validator,
        mutation,
    )

    mutation = copy.deepcopy(planned)
    mutation["unexpected_field"] = True
    expect_invalid(
        "unknown top-level properties are rejected",
        validator,
        mutation,
    )

    mutation = copy.deepcopy(planned)
    mutation["created_at"] = "not-a-timestamp"
    expect_invalid(
        "date-time formats are actively checked",
        validator,
        mutation,
    )

    mutation = copy.deepcopy(planned)
    mutation["dataset"]["source_uri"] = "not-a-uri"
    expect_invalid(
        "URI formats are actively checked",
        validator,
        mutation,
    )

    print("PASS 9 schema/fixture assertions completed; no external data used")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except AssertionError as exc:
        print(str(exc), file=sys.stderr)
        raise SystemExit(1)
