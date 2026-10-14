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

    crop_cohorts = doc.get("candidate_crop_cohorts")
    if not isinstance(crop_cohorts, list) or not crop_cohorts:
        errors.append("candidate_crop_cohorts must define at least one region-specific candidate cohort")
        crop_cohorts = []
    cohort_ids: set[str] = set()
    selected_cohorts = []
    for index, raw_cohort in enumerate(crop_cohorts):
        cohort = _as_dict(raw_cohort)
        cohort_id = cohort.get("cohort_id")
        if not isinstance(cohort_id, str) or not cohort_id.strip():
            errors.append(f"candidate_crop_cohorts[{index}].cohort_id must be non-empty")
        elif cohort_id in cohort_ids:
            errors.append(f"duplicate candidate crop cohort id: {cohort_id}")
        else:
            cohort_ids.add(cohort_id)
        if cohort.get("selection_state") == "selected_for_baseline":
            selected_cohorts.append(cohort_id)

    geography = _as_dict(doc.get("geography"))
    if geography.get("boundary_state") == "verified_and_frozen":
        boundary_source = str(geography.get("boundary_source", "")).lower()
        if any(term in boundary_source for term in ("not yet acquired", "not frozen", "unverified")):
            errors.append("verified boundary state conflicts with unresolved boundary_source text")

    context_refs = doc.get("context_references")
    if not isinstance(context_refs, list) or not context_refs:
        errors.append("context_references must contain at least one reviewed context lead")
        context_refs = []
    context_ids: set[str] = set()
    for index, raw_ref in enumerate(context_refs):
        context_ref = _as_dict(raw_ref)
        reference_id = context_ref.get("reference_id")
        if not isinstance(reference_id, str) or not reference_id.strip():
            errors.append(f"context_references[{index}].reference_id must be a non-empty identifier")
        elif reference_id in context_ids:
            errors.append(f"duplicate context reference_id: {reference_id}")
        else:
            context_ids.add(reference_id)
        limitations = context_ref.get("limitations")
        if not isinstance(limitations, list) or not limitations:
            errors.append(f"context_references[{index}] must retain explicit limitations")

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
            if source_id != "LOCAL-OBSERVATIONS" and not isinstance(source.get("redistribution_allowed"), bool):
                errors.append(f"{where}: acquired external artifact requires an explicit redistribution decision")

        if source_id == "LOCAL-OBSERVATIONS":
            if acquisition in {"not_acquired", "rejected"} and source.get("rights_state") != "permission_required":
                errors.append("local observations must retain permission_required until scope and authorization are recorded")
            if acquisition == "acquired":
                if source.get("rights_state") != "permission_granted":
                    errors.append("acquired local observations require explicit permission_granted state")
                if source.get("redistribution_allowed") is not None:
                    errors.append("private local observations keep redistribution_allowed=null unless a separate sharing licence is modeled")
                if not isinstance(selected, str) or not selected.strip():
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

    site_constraints = _as_dict(doc.get("site_constraints"))
    required_site_constraints = (
        "irrigation_water_quantity_and_quality",
        "soil_condition_and_contamination",
        "land_access_and_permission",
        "postharvest_storage_energy_and_transport",
        "market_nutrition_and_affordability",
        "regional_supply_and_contingency",
    )
    site_states: dict[str, str] = {}
    for constraint_id in required_site_constraints:
        constraint = _as_dict(site_constraints.get(constraint_id))
        state = constraint.get("state")
        if state not in {"not_assessed", "assessment_required", "assessed", "blocked"}:
            errors.append(f"site_constraints.{constraint_id}.state is missing or invalid")
        elif isinstance(state, str):
            site_states[constraint_id] = state
        if not isinstance(constraint.get("required_evidence"), list) or len(constraint["required_evidence"]) < 2:
            errors.append(f"site_constraints.{constraint_id}.required_evidence must contain at least two evidence items")
        if not isinstance(constraint.get("claim_limit"), str) or len(constraint["claim_limit"].strip()) < 30:
            errors.append(f"site_constraints.{constraint_id}.claim_limit must state a specific decision limit")

        evidence_refs = constraint.get("evidence_refs")
        reviewer = constraint.get("reviewed_by")
        if not isinstance(evidence_refs, list) or any(
            not isinstance(ref, str) or not ref.startswith(("https://", "urn:", "sha256:"))
            for ref in evidence_refs
        ):
            errors.append(f"site_constraints.{constraint_id}.evidence_refs must contain valid auditable references")
        if state == "assessed":
            if not isinstance(evidence_refs, list) or not evidence_refs:
                errors.append(f"site_constraints.{constraint_id}=assessed requires at least one evidence reference")
            if not isinstance(reviewer, str) or len(reviewer.strip()) < 3:
                errors.append(f"site_constraints.{constraint_id}=assessed requires a reviewer identity or role")
        if state == "not_assessed" and (evidence_refs or reviewer is not None):
            errors.append(f"site_constraints.{constraint_id}=not_assessed must not carry assessment evidence or reviewer")

    if any(gates.get(gate) not in {"not_started", "blocked", "passed"} for gate in GATES):
        errors.append("validation.gates must contain valid D0-D5 states")

    # A passed gate must be backed by the corresponding artifact/state,
    # not merely by a manually edited status label.
    if gates.get("D0") == "passed" and geography.get("boundary_state") != "verified_and_frozen":
        errors.append("D0=passed requires an authoritative, verified-and-frozen study boundary")

    if gates.get("D1") == "passed":
        if len(sources) != len(EXPECTED_SOURCES) or any(
            _as_dict(source).get("acquisition_state") != "acquired" for source in sources
        ):
            errors.append("D1=passed requires acquisition and review of every admitted source artifact")
        for index, raw in enumerate(sources):
            source = _as_dict(raw)
            if source.get("acquisition_state") != "acquired":
                continue
            digest = source.get("artifact_sha256")
            if not isinstance(digest, str) or not SHA256_RE.fullmatch(digest):
                errors.append(f"D1=passed requires a frozen SHA-256 for sources[{index}]")
            if not isinstance(source.get("selected_product"), str) or not source.get("selected_product", "").strip():
                errors.append(f"D1=passed requires an exact product identity for sources[{index}]")
            if source.get("source_catalog_verified") is not True:
                errors.append(f"D1=passed requires product-identity verification for sources[{index}]")
            if source.get("source_id") == "LOCAL-OBSERVATIONS":
                if source.get("rights_state") != "permission_granted" or source.get("redistribution_allowed") is not None:
                    errors.append(f"D1=passed requires recorded processing permission and private-by-default rights for sources[{index}]")
            else:
                if not isinstance(source.get("redistribution_allowed"), bool):
                    errors.append(f"D1=passed requires an explicit external-source rights decision for sources[{index}]")
                if source.get("rights_state") in {"license_not_reviewed", "permission_required"}:
                    errors.append(f"D1=passed cannot include unresolved rights for sources[{index}]")

    if gates.get("D1") == "passed" and gates.get("D0") != "passed":
        errors.append("D1=passed requires D0=passed")
    if gates.get("D2") == "passed":
        if any(gates.get(gate) != "passed" for gate in ("D0", "D1")):
            errors.append("D2=passed requires D0-D1=passed")
        if evidence.get("baseline_computed") is not True:
            errors.append("D2=passed requires baseline_computed=true")
        if not selected_cohorts:
            errors.append("D2=passed requires at least one crop cohort selected for baseline evaluation")
    if gates.get("D3") == "passed" and evidence.get("independent_validation_complete") is not True:
        errors.append("D3=passed requires independent_validation_complete=true")
    if gates.get("D4") == "passed":
        if any(gates.get(gate) != "passed" for gate in ("D0", "D1", "D2", "D3")):
            errors.append("D4=passed requires D0-D3=passed")
        if evidence.get("expert_review_complete") is not True:
            errors.append("D4=passed requires expert_review_complete=true")
    if gates.get("D5") == "passed":
        if any(site_states.get(constraint_id) != "assessed" for constraint_id in required_site_constraints):
            errors.append("D5=passed requires assessment of every site constraint")
        if any(gates.get(gate) != "passed" for gate in ("D0", "D1", "D2", "D3", "D4")):
            errors.append("D5=passed requires D0-D4=passed")
        if evidence.get("independent_validation_complete") is not True or evidence.get("expert_review_complete") is not True:
            errors.append("D5=passed requires independent validation and expert review evidence")
        if evidence.get("decision_support_authorized") is not True:
            errors.append("D5=passed requires decision_support_authorized=true")

    if status == "design_only":
        if any(site_states.get(constraint_id) != "not_assessed" for constraint_id in required_site_constraints):
            errors.append("design_only status requires every site constraint to remain not_assessed")
        if any(_as_dict(cohort).get("selection_state") != "candidate_not_selected" for cohort in crop_cohorts):
            errors.append("design_only status requires all crop cohorts to remain candidate_not_selected")
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

    if status == "decision_support_candidate" and any(
        site_states.get(constraint_id) != "assessed" for constraint_id in required_site_constraints
    ):
        errors.append("decision_support_candidate status requires all site constraints assessed")

    if status == "decision_support_candidate" and not selected_cohorts:
        errors.append("decision_support_candidate status requires a crop cohort selected for baseline evaluation")

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
