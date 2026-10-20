#!/usr/bin/env python3
"""Validate the demand-response trace and report contracts in CI only.

The positive report fixture uses synthetic arithmetic values. It is a contract
test, not a measured facility or grid-service result.
"""
from __future__ import annotations

import copy
import json
from pathlib import Path
import sys
from typing import Any

from jsonschema import Draft202012Validator, FormatChecker

ROOT = Path(__file__).resolve().parents[1]
INPUT_SCHEMA_PATH = ROOT / "schemas/demand-response-trace-v1.schema.json"
REPORT_SCHEMA_PATH = ROOT / "schemas/demand-response-assessment-report-v1.schema.json"
FIXTURE_PATH = ROOT / "schemas/examples/demand-response-trace-v1.synthetic.json"


def load_json(path: Path) -> Any:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        raise AssertionError(f"Cannot load {path.relative_to(ROOT)}: {exc}") from exc


def validator_for(path: Path) -> Draft202012Validator:
    schema = load_json(path)
    Draft202012Validator.check_schema(schema)
    print(f"PASS valid Draft 2020-12 schema: {path.relative_to(ROOT)}")
    return Draft202012Validator(schema, format_checker=FormatChecker())


def error_summary(validator: Draft202012Validator, instance: Any) -> str:
    errors = sorted(
        validator.iter_errors(instance),
        key=lambda error: (
            tuple(str(part) for part in error.absolute_path),
            tuple(str(part) for part in error.absolute_schema_path),
        ),
    )
    return "\n".join(
        f"- /{'/'.join(map(str, error.absolute_path))}: {error.message}"
        for error in errors[:12]
    )


def expect_valid(
    name: str, validator: Draft202012Validator, instance: Any
) -> None:
    errors = error_summary(validator, instance)
    if errors:
        raise AssertionError(f"{name} should be valid:\n{errors}")
    print(f"PASS {name}")


def expect_invalid(
    name: str, validator: Draft202012Validator, instance: Any
) -> None:
    errors = error_summary(validator, instance)
    if not errors:
        raise AssertionError(f"{name} unexpectedly validated")
    print(f"PASS {name} (rejected)")


def known_answer_report() -> dict[str, Any]:
    """Synthetic output-shaped report for report-schema validation only."""
    errors = [0.0] * 9 + [0.8]
    metrics = {
        "schema_version": "1.0",
        "fixture_id": "synthetic-demand-response-rsr-edr-known-answer-v1",
        "synthetic": True,
        "claim_ceiling": "metric_accounting_only",
        "rsr": {
            "sample_count": 10,
            "reserve_capacity_kw": 10.0,
            "normalized_absolute_errors": errors,
            "mean_normalized_absolute_error": 0.08,
            "p90_normalized_absolute_error": 0.08,
            "max_normalized_absolute_error": 0.8,
            "error_threshold": 0.3,
            "max_allowed_exceedance_fraction": 0.1,
            "threshold_exceedance_count": 1,
            "threshold_exceedance_fraction": 0.1,
            "tracking_requirement_satisfied": False,
        },
        "rsr_reference": {
            "sample_count": 10,
            "normalization_basis": "source_normalized_error_series",
            "normalized_absolute_errors": errors,
            "mean_normalized_absolute_error": 0.08,
            "p90_normalized_absolute_error": 0.08,
            "max_normalized_absolute_error": 0.8,
            "error_threshold": 0.3,
            "max_allowed_exceedance_fraction": 0.1,
            "threshold_exceedance_count": 1,
            "threshold_exceedance_fraction": 0.1,
            "tracking_requirement_satisfied": False,
        },
        "edr": {
            "sample_count": 3,
            "duration_hours": 3.0,
            "baseline_energy_kwh": 300.0,
            "event_energy_kwh": 260.0,
            "net_energy_reduction_kwh": 40.0,
            "net_energy_reduction_fraction": 40.0 / 300.0,
            "gross_curtailment_kwh": 50.0,
            "rebound_energy_kwh": 10.0,
            "rebound_interval_count": 1,
            "peak_baseline_power_kw": 100.0,
            "peak_event_power_kw": 110.0,
            "peak_event_power_excess_over_baseline_kw": 10.0,
        },
    }
    return {
        "receipt": {
            "receipt_schema_version": "1.0",
            "digest_algorithm": "SHA-256",
            "digest_scope": "exact_input_bytes_and_compact_metrics_json",
            "input_sha256": "a" * 64,
            "metrics_sha256": "b" * 64,
            "fixture_id": metrics["fixture_id"],
            "synthetic": True,
            "claim_ceiling": "metric_accounting_only",
            "verification_boundary": (
                "synthetic schema example; digests are placeholders"
            ),
        },
        "metrics": metrics,
    }


def main() -> int:
    input_validator = validator_for(INPUT_SCHEMA_PATH)
    report_validator = validator_for(REPORT_SCHEMA_PATH)
    fixture = load_json(FIXTURE_PATH)

    expect_valid("committed synthetic trace fixture", input_validator, fixture)
    expect_valid(
        "synthetic known-answer report shape",
        report_validator,
        known_answer_report(),
    )

    mutation = copy.deepcopy(fixture)
    mutation["claim_ceiling"] = "quality_accepted_work_per_kwh"
    expect_invalid("input cannot overstate claim ceiling", input_validator, mutation)

    mutation = copy.deepcopy(fixture)
    mutation["unmodeled_grid_attribute"] = True
    expect_invalid("unknown root fields are rejected", input_validator, mutation)

    mutation = copy.deepcopy(fixture)
    mutation["rsr"]["interval_ids"][1] = mutation["rsr"]["interval_ids"][0]
    expect_invalid("duplicate sample identifiers are rejected", input_validator, mutation)

    mutation = copy.deepcopy(fixture)
    mutation["rsr"]["reserve_capacity_kw"] = 0
    expect_invalid("zero reserve is rejected by schema", input_validator, mutation)

    mutation = copy.deepcopy(fixture)
    mutation["rsr_reference"]["interval_ids"][1] = mutation["rsr_reference"]["interval_ids"][0]
    expect_invalid(
        "duplicate source-error sample identifiers are rejected",
        input_validator,
        mutation,
    )

    mutation = copy.deepcopy(fixture)
    del mutation["rsr"]
    del mutation["rsr_reference"]
    del mutation["edr"]
    expect_invalid("at least one metric section is required", input_validator, mutation)

    report = known_answer_report()
    mutation = copy.deepcopy(report)
    mutation["receipt"]["input_sha256"] = "not-a-digest"
    expect_invalid("report rejects malformed input digest", report_validator, mutation)

    mutation = copy.deepcopy(report)
    mutation["metrics"]["edr"]["rebound_energy_kwh"] = -1.0
    expect_invalid("report rejects negative rebound energy", report_validator, mutation)

    mutation = copy.deepcopy(report)
    mutation["metrics"]["additional_score"] = 1000000
    expect_invalid("report rejects unknown metric fields", report_validator, mutation)

    print("PASS 11 demand-response schema assertions completed; no external data used")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except AssertionError as exc:
        print(str(exc), file=sys.stderr)
        raise SystemExit(1)
