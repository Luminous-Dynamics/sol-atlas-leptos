#!/usr/bin/env python3
"""Fail-closed validation for the Johannesburg/Highveld agroclimate profile.

This validates schema, profile consistency, and evidence-maturity declarations.
It does not ingest weather/soil rasters or validate agronomic predictions.
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from urllib.parse import urlsplit
from typing import Any

from jsonschema import Draft202012Validator, FormatChecker

ROOT = Path(__file__).resolve().parents[2]
DEFAULT_PROFILE = ROOT / "docs/pilots/johannesburg-highveld-food-resilience.json"
DEFAULT_SCHEMA = ROOT / "docs/schemas/agroclimate-model-profile.schema.json"
SHA256_RE = re.compile(r"^[a-f0-9]{64}$")
EXPECTED_SOURCES = {
    "FAO-GAEZ-V5",
    "ERA5-LAND",
    "CHIRPS-V3",
    "SOILGRIDS-V2",
    "SAWS-SEASONAL",
    "LOCAL-OBSERVATIONS",
}
GATES = ("D0", "D1", "D2", "D3", "D4", "D5")
EVIDENCE_FLAGS = (
    "source_artifacts_acquired",
    "baseline_computed",
    "independent_validation_complete",
    "expert_review_complete",
    "decision_support_authorized",
)


def _as_dict(value: Any) -> dict:
    return value if isinstance(value, dict) else {}


def validate_profile(doc: Any, schema: dict) -> list[str]:
    """Return all structural and cross-field errors without mutating doc."""
    if not isinstance(doc, dict):
        return ["profile root must be an object"]

    validator = Draft202012Validator(schema, format_checker=FormatChecker())
    errors = [
        f"{list(error.path)}: {error.message}"
        for error in sorted(
            validator.iter_errors(doc),
            key=lambda item: (list(map(str, item.path)), item.message),
        )
    ]

    profile_id = doc.get("profile_id")
    if profile_id != "agroclimate-johannesburg-highveld-v1":
        errors.append("profile_id must identify the Johannesburg/Highveld V1 profile")

    geography = _as_dict(doc.get("geography"))
    if geography.get("boundary_state") == "verified_and_frozen":
        boundary_source = str(geography.get("boundary_source", "")).lower()
        if any(term in boundary_source for term in ("not yet acquired", "not frozen", "unverified")):
            errors.append("verified boundary state conflicts with unresolved boundary_source text")

    sources = doc.get("sources")
    if not isinstance(sources, list):
        sources = []
    seen: set[str] = set()
    source_by_id: dict[str, dict] = {}
    for index, raw in enumerate(sources):
        source = _as_dict(raw)
        source_id = source.get("source_id")
        where = f"sources[{index}]"
        if not isinstance(source_id, str) or source_id not in EXPECTED_SOURCES:
            errors.append(f"{where}.source_id is not an admitted source")
            continue
        if source_id in seen:
            errors.append(f"duplicate source_id: {source_id}")
        seen.add(source_id)
        source_by_id[source_id] = source

        url = source.get("url")
        if source_id == "LOCAL-OBSERVATIONS":
            if url is not None:
                errors.append("LOCAL-OBSERVATIONS must not invent a public URL before a source is selected")
        elif not isinstance(url, str):
            errors.append(f"{where}.url must be an HTTPS URL for an external source")
        else:
            try:
                parsed = urlsplit(url)
            except ValueError:
                parsed = None
            if not parsed or parsed.scheme != "https" or not parsed.netloc:
                errors.append(f"{where}.url must be an HTTPS URL")

        acquisition = source.get("acquisition_state")
        digest = source.get("artifact_sha256")
        selected = source.get("selected_product")
        if acquisition in {"not_acquired", "rejected"} and digest is not None:
            errors.append(f"{where}: unacquired/rejected source cannot claim an artifact SHA-256")
        if acquisition == "acquired":
            if not isinstance(digest, str) or not SHA256_RE.fullmatch(digest):
                errors.append(f"{where}: acquired source requires a lowercase 64-character SHA-256")
            if not isinstance(selected, str) or not selected.strip():
                errors.append(f"{where}: acquired source requires an exact selected_product identifier")
            if source.get("source_catalog_verified") is not True:
                errors.append(f"{where}: acquisition requires source catalog/product identity verification")
            if source.get("rights_state") in {"license_not_reviewed", "permission_required"}:
                errors.append(f"{where}: acquisition cannot be promoted before required rights/permission review")
            if source.get("redistribution_allowed") is None:
                errors.append(f"{where}: acquired artifact requires an explicit redistribution decision")

        if source_id == "LOCAL-OBSERVATIONS":
            if source.get("rights_state") != "permission_required":
                errors.append("local observations must retain permission_required until scope and authorization are recorded")
            if acquisition == "acquired" and source.get("selected_product") is None:
                errors.append("acquired local observations require a versioned authorized source/measurement protocol")

    if seen != EXPECTED_SOURCES:
        errors.append(f"source set mismatch: expected {sorted(EXPECTED_SOURCES)}, got {sorted(seen)}")

    # Source-specific epistemic boundaries prevent a schema-valid profile from
    # silently changing the resolution, independence, or access caveats.
    era = source_by_id.get("ERA5-LAND", {})
    era_spatial = str(era.get("native_spatial_support", "")).lower()
    if "9 km" not in era_spatial or "0.1-degree" not in era_spatial:
        errors.append("ERA5-LAND must disclose native ~9 km support and the distributed 0.1-degree grid")

    chirps = source_by_id.get("CHIRPS-V3", {})
    if "station" not in str(chirps.get("upstream_overlap", "")).lower():
        errors.append("CHIRPS V3 source overlap must disclose station observations")
    if not any("index" in str(item).lower() and "hash" in str(item).lower()
               for item in chirps.get("required_checks", []) if isinstance(item, str)):
        errors.append("CHIRPS V3 requires a fresh official index snapshot and hash before freezing months")

    soil = source_by_id.get("SOILGRIDS-V2", {})
    soil_limits = str(soil.get("native_spatial_support", "")).lower()
    if "not suitable for farm-level" not in soil_limits:
        errors.append("SoilGrids must retain its explicit farm-level limitation")
    soil_checks = " ".join(str(item) for item in soil.get("required_checks", []) if isinstance(item, str)).lower()
    if "rest api" not in soil_checks or "paused" not in soil_checks:
        errors.append("SoilGrids access check must disclose the currently paused REST API and alternate access review")

    saws = source_by_id.get("SAWS-SEASONAL", {})
    saws_scope = str(saws.get("native_spatial_support", "")).lower()
    if "not assume" not in saws_scope or "farm-scale" not in saws_scope:
        errors.append("SAWS seasonal notice must not be represented as a farm-scale forecast")
    if saws.get("acquisition_state") == "not_acquired" and not any(
        "original" in str(item).lower() and "bulletin" in str(item).lower()
        for item in saws.get("required_checks", [])
    ):
        errors.append("SAWS summary must require acquisition of the original bulletin before use")

    gaez = source_by_id.get("FAO-GAEZ-V5", {})
    if gaez.get("acquisition_state") == "not_acquired" and gaez.get("selected_product") is not None:
        if "exact" not in str(gaez.get("selected_product")).lower():
            errors.append("GAEZ selected_product must be an exact layer ID or null, not an unqualified product claim")

    periods = doc.get("candidate_reference_periods", [])
    period_ids: set[str] = set()
    for index, raw in enumerate(periods if isinstance(periods, list) else []):
        period = _as_dict(raw)
        period_id = period.get("period_id")
        if isinstance(period_id, str) and period_id in period_ids:
            errors.append(f"duplicate candidate period_id: {period_id}")
        if isinstance(period_id, str):
            period_ids.add(period_id)
        start, end = period.get("start_year"), period.get("end_year")
        if isinstance(start, int) and isinstance(end, int) and end < start:
            errors.append(f"candidate_reference_periods[{index}] ends before it starts")

    validation = _as_dict(doc.get("validation"))
    gates = _as_dict(validation.get("gates"))
    status = doc.get("status")
    evidence = _as_dict(doc.get("evidence_state"))
    model_arch = _as_dict(doc.get("model_architecture"))
    evaluation = _as_dict(doc.get("evaluation"))

    if any(gates.get(gate) not in {"not_started", "blocked", "passed"} for gate in GATES):
        errors.append("validation.gates must contain valid D0-D5 states")

    if status == "design_only":
        if any(evidence.get(flag) is not False for flag in EVIDENCE_FLAGS):
            errors.append("design_only status requires every evidence_state flag to be false")
        if any(gates.get(gate) != "not_started" for gate in GATES):
            errors.append("design_only status requires all D0-D5 gates to remain not_started")
        if evaluation.get("state") != "not_started":
            errors.append("design_only status requires evaluation.state=not_started")
        if model_arch.get("model_selection_state") != "not_started":
            errors.append("design_only status requires model_selection_state=not_started")
        if any(_as_dict(source).get("acquisition_state") != "not_acquired" for source in sources):
            errors.append("design_only status requires all profile source artifacts to remain not_acquired")

    if evidence.get("baseline_computed") is True:
        if any(gates.get(gate) != "passed" for gate in ("D0", "D1", "D2")):
            errors.append("baseline_computed=true requires D0-D2=passed")
        if evidence.get("source_artifacts_acquired") is not True:
            errors.append("baseline_computed=true requires source_artifacts_acquired=true")
    if evidence.get("independent_validation_complete") is True:
        if evidence.get("baseline_computed") is not True or gates.get("D3") != "passed":
            errors.append("independent_validation_complete=true requires baseline and D3=passed")
    if evidence.get("expert_review_complete") is True and gates.get("D4") != "passed":
        errors.append("expert_review_complete=true requires D4=passed")
    if evidence.get("decision_support_authorized") is True:
        if any(gates.get(gate) != "passed" for gate in GATES):
            errors.append("decision_support_authorized=true requires all D0-D5 gates passed")
        if evidence.get("independent_validation_complete") is not True or evidence.get("expert_review_complete") is not True:
            errors.append("decision_support_authorized=true requires independent validation and expert review")
        if status != "decision_support_candidate":
            errors.append("decision_support_authorized=true requires decision_support_candidate status")

    if status == "descriptive_baseline" and (
        evidence.get("baseline_computed") is not True or
        any(gates.get(gate) != "passed" for gate in ("D0", "D1", "D2"))
    ):
        errors.append("descriptive_baseline status requires a computed baseline and D0-D2=passed")
    if status == "independently_validated" and (
        evidence.get("independent_validation_complete") is not True or
        any(gates.get(gate) != "passed" for gate in ("D0", "D1", "D2", "D3"))
    ):
        errors.append("independently_validated status requires independent validation and D0-D3=passed")
    if status == "decision_support_candidate" and (
        evidence.get("decision_support_authorized") is not True or
        any(gates.get(gate) != "passed" for gate in GATES)
    ):
        errors.append("decision_support_candidate status requires all evidence and D0-D5=passed")

    return errors


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--profile", type=Path, default=DEFAULT_PROFILE)
    parser.add_argument("--schema", type=Path, default=DEFAULT_SCHEMA)
    args = parser.parse_args(argv)
    try:
        schema = json.loads(args.schema.read_text(encoding="utf-8"))
        doc = json.loads(args.profile.read_text(encoding="utf-8"))
        Draft202012Validator.check_schema(schema)
        errors = validate_profile(doc, schema)
    except (OSError, json.JSONDecodeError, ValueError) as exc:
        print(f"FAIL: {exc}", file=sys.stderr)
        return 2
    if errors:
        print("FAIL: agroclimate profile validation")
        for error in errors:
            print(f"  - {error}")
        return 1
    source_count = len(doc.get("sources", []))
    print(
        f"PASS: agroclimate profile schema and evidence gates; "
        f"status={doc['status']}; sources={source_count}; "
        "scientific_validity=not_assessed"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
