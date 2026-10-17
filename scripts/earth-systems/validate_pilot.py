#!/usr/bin/env python3
"""Fail-closed structural checks for the Baixo Limpopo pilot profile.

This validates readiness/evidence metadata only. It does not fetch datasets or
assess scientific validity.
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from datetime import date
from pathlib import Path
from typing import Any
from urllib.parse import urlsplit

EXPECTED_SOURCES = {"FAO-WAPOR", "CHIRPS3"}
EXPECTED_GATES = ("D0", "D1", "D2", "D3", "D4", "D5")
GATE_STATES = {"not_started", "blocked", "passed"}
PILOT_STATUSES = {
    "design_only", "data_readiness", "descriptive_baseline",
    "independently_validated", "decision_support_candidate", "operational",
}
SHA256_RE = re.compile(r"^[a-fA-F0-9]{64}$")
REQUIRED_TOP = {
    "pilot_id", "status", "review_date", "title", "geography", "question",
    "intended_outputs", "sources", "analysis", "validation", "claim_ceiling",
    "evidence_state",
}
REQUIRED_SOURCE = {
    "source_id", "provider", "url", "product_family", "product_version",
    "selected_product", "product_selection_state", "acquisition_state",
    "artifact_sha256", "source_catalog_verified", "rights_state",
    "redistribution_allowed", "native_spatial_support",
    "native_temporal_support", "upstream_overlap", "required_checks",
}
STATE_FIELDS = (
    "source_artifacts_downloaded", "observations_processed", "baseline_computed",
    "independent_validation_complete", "independent_review_complete",
    "d5_accepted", "operational_claim_authorized",
)


def validate_pilot(doc: Any) -> list[str]:
    errors: list[str] = []
    if not isinstance(doc, dict):
        return ["pilot profile root must be an object"]

    missing = sorted(REQUIRED_TOP - set(doc))
    if missing:
        errors.append(f"pilot missing top-level fields: {missing}")
    if not isinstance(doc.get("pilot_id"), str) or not doc.get("pilot_id", "").startswith("water-food-"):
        errors.append("pilot_id must start with 'water-food-'")
    if not isinstance(doc.get("status"), str) or doc["status"] not in PILOT_STATUSES:
        errors.append("status is not a recognized pilot status")
    try:
        date.fromisoformat(doc.get("review_date", ""))
    except (TypeError, ValueError):
        errors.append("review_date must be an ISO date (YYYY-MM-DD)")
    for field in ("title", "question"):
        if not isinstance(doc.get(field), str) or len(doc[field].strip()) < 10:
            errors.append(f"{field} must be a non-empty, specific string")

    geography = doc.get("geography")
    geography_fields = (
        "name", "country", "boundary_state", "boundary_source",
        "spatial_scope_rule", "cross_border_note",
    )
    if not isinstance(geography, dict):
        errors.append("geography must be an object")
    else:
        for field in geography_fields:
            if not isinstance(geography.get(field), str) or not geography[field].strip():
                errors.append(f"geography.{field} must be a non-empty string")
        if not isinstance(geography.get("boundary_state"), str) or geography["boundary_state"] not in {"unverified", "verified_and_frozen"}:
            errors.append("geography.boundary_state must be explicit")

    outputs = doc.get("intended_outputs")
    if not isinstance(outputs, list) or len(outputs) < 3 or any(not isinstance(x, str) or not x.strip() for x in outputs):
        errors.append("intended_outputs must contain at least three non-empty strings")
    claims = doc.get("claim_ceiling")
    if not isinstance(claims, list) or len(claims) < 5 or any(not isinstance(x, str) or not x.strip() for x in claims):
        errors.append("claim_ceiling must contain at least five explicit limits")

    sources = doc.get("sources")
    seen: set[str] = set()
    if not isinstance(sources, list) or len(sources) != 2:
        errors.append("pilot must define exactly the two admitted source families")
        sources = sources if isinstance(sources, list) else []
    for i, source in enumerate(sources):
        where = f"sources[{i}]"
        if not isinstance(source, dict):
            errors.append(f"{where} must be an object")
            continue
        missing_source = sorted(REQUIRED_SOURCE - set(source))
        if missing_source:
            errors.append(f"{where} missing fields: {missing_source}")
        source_id = source.get("source_id")
        if not isinstance(source_id, str) or source_id not in EXPECTED_SOURCES:
            errors.append(f"{where}.source_id is not an admitted source")
        elif source_id in seen:
            errors.append(f"duplicate source_id: {source_id}")
        else:
            seen.add(source_id)
        url = source.get("url")
        try:
            parts = urlsplit(url) if isinstance(url, str) else None
        except ValueError:
            parts = None
        if not parts or parts.scheme != "https" or not parts.netloc:
            errors.append(f"{where}.url must be an HTTPS URL")
        if not isinstance(source.get("acquisition_state"), str) or source["acquisition_state"] not in {"not_acquired", "acquired", "rejected"}:
            errors.append(f"{where}.acquisition_state is invalid")
        digest = source.get("artifact_sha256")
        if source.get("acquisition_state") == "not_acquired" and digest is not None:
            errors.append(f"{where}: not_acquired source must not claim an artifact digest")
        if source.get("acquisition_state") == "acquired":
            selected = source.get("selected_product")
            if not isinstance(selected, str) or not selected.strip():
                errors.append(f"{where}: acquired source requires an exact selected_product identifier")
            if not isinstance(digest, str) or not SHA256_RE.fullmatch(digest):
                errors.append(f"{where}: acquired source requires a 64-character SHA-256")
            if source.get("source_catalog_verified") is not True:
                errors.append(f"{where}: acquired source requires verified product metadata")
        if type(source.get("source_catalog_verified")) is not bool:
            errors.append(f"{where}.source_catalog_verified must be a boolean")
        if source_id == "FAO-WAPOR":
            if source.get("rights_state") != "review_product_metadata" or source.get("redistribution_allowed") is not None:
                errors.append(f"{where}: WaPOR rights must remain product-review-required and redistribution unknown")
        if source_id == "CHIRPS3":
            if source.get("rights_state") != "cc_by_4_0_with_attribution" or source.get("redistribution_allowed") is not True:
                errors.append(f"{where}: CHIRPS v3 must retain its reviewed CC BY 4.0 attribution status")
            overlap = source.get("upstream_overlap")
            if not isinstance(overlap, str) or "station" not in overlap.lower():
                errors.append(f"{where}: CHIRPS upstream station overlap must be disclosed")
            elif "gauge-undercatch-corrected" not in overlap.lower():
                errors.append(
                    f"{where}: CHIRPS reference-comparison guidance must retain "
                    "the provider's gauge-undercatch-correction recommendation"
                )
        for field in ("provider", "product_family", "product_version", "product_selection_state",
                      "native_spatial_support", "native_temporal_support", "upstream_overlap"):
            if not isinstance(source.get(field), str) or not source[field].strip():
                errors.append(f"{where}.{field} must be a non-empty string")
        checks = source.get("required_checks")
        if not isinstance(checks, list) or len(checks) < 3 or any(not isinstance(x, str) or not x.strip() for x in checks):
            errors.append(f"{where}.required_checks must list at least three checks")
        elif source_id == "CHIRPS3":
            check_text = " ".join(checks).lower()
            if (
                "legates-willmott" not in check_text
                or "not independent station observations" not in check_text
                or "must not be applied automatically" not in check_text
            ):
                errors.append(
                    f"{where}: CHIRPS checks must bound Legates-Willmott grids as "
                    "auxiliary factors, not independent observations or automatic corrections"
                )
        elif source_id == "FAO-WAPOR":
            check_text = " ".join(checks).lower()
            if (
                "mosaicset" not in check_text
                or "l3-aeti-m" not in check_text
                or "mbl" not in check_text
                or "not an exhaustive level iii area inventory" not in check_text
            ):
                errors.append(
                    f"{where}: WaPOR pilot must inventory the area-specific Level III "
                    "MOSAICSET index for L3-AETI-M / MBL and must not infer Level III "
                    "absence from the /mapsets endpoint"
                )

    if seen != EXPECTED_SOURCES:
        errors.append(f"source set must be exactly {sorted(EXPECTED_SOURCES)}")

    analysis = doc.get("analysis")
    analysis_fields = (
        "estimand", "baseline", "spatial_alignment", "temporal_alignment",
        "missingness_policy", "independence_policy",
    )
    if not isinstance(analysis, dict):
        errors.append("analysis must be an object")
    else:
        for field in analysis_fields:
            if not isinstance(analysis.get(field), str) or len(analysis[field].strip()) < 20:
                errors.append(f"analysis.{field} must be a specific statement")

    validation = doc.get("validation")
    gates = {}
    if not isinstance(validation, dict):
        errors.append("validation must be an object")
    else:
        if validation.get("independence_review_required") is not True:
            errors.append("validation.independence_review_required must be true")
        candidates = validation.get("independent_reference_candidates")
        if not isinstance(candidates, list) or not candidates or any(not isinstance(x, str) or len(x.strip()) < 10 for x in candidates):
            errors.append("validation.independent_reference_candidates must be specific")
        metrics = validation.get("metrics")
        if not isinstance(metrics, list) or len(metrics) < 3 or any(not isinstance(x, str) or len(x.strip()) < 10 for x in metrics):
            errors.append("validation.metrics must contain at least three specific metrics")
        gates = validation.get("gates")
        if not isinstance(gates, dict):
            errors.append("validation.gates must be an object")
            gates = {}
        for gate in EXPECTED_GATES:
            if not isinstance(gates.get(gate), str) or gates[gate] not in GATE_STATES:
                errors.append(f"validation.gates.{gate} is missing or invalid")

    state = doc.get("evidence_state")
    if not isinstance(state, dict):
        errors.append("evidence_state must be an object")
        state = {}
    for field in STATE_FIELDS:
        if type(state.get(field)) is not bool:
            errors.append(f"evidence_state.{field} must be a boolean")

    all_sources_acquired = len(sources) == 2 and all(
        isinstance(s, dict) and s.get("acquisition_state") == "acquired"
        and isinstance(s.get("artifact_sha256"), str) and SHA256_RE.fullmatch(s["artifact_sha256"])
        for s in sources
    )
    if state.get("source_artifacts_downloaded") is True and not all_sources_acquired:
        errors.append("source_artifacts_downloaded=true requires every admitted source to be acquired and hashed")
    if state.get("observations_processed") is True and state.get("source_artifacts_downloaded") is not True:
        errors.append("observations_processed=true requires source_artifacts_downloaded=true")
    if state.get("baseline_computed") is True:
        if state.get("observations_processed") is not True or gates.get("D2") != "passed":
            errors.append("baseline_computed=true requires processed observations and D2=passed")
    if state.get("independent_validation_complete") is True:
        if state.get("baseline_computed") is not True or gates.get("D3") != "passed":
            errors.append("independent_validation_complete=true requires baseline and D3=passed")
    if state.get("independent_review_complete") is True and gates.get("D4") != "passed":
        errors.append("independent_review_complete=true requires D4=passed")
    if state.get("d5_accepted") is True:
        required_preconditions = (
            state.get("independent_validation_complete") is True,
            state.get("independent_review_complete") is True,
            gates.get("D5") == "passed",
        )
        if not all(required_preconditions):
            errors.append("d5_accepted=true requires independent validation, review, and D5=passed")
    if state.get("operational_claim_authorized") is True:
        if doc.get("status") != "operational" or state.get("d5_accepted") is not True:
            errors.append("operational_claim_authorized=true requires status=operational and d5_accepted=true")
    # A passed gate must be backed by its minimum evidence, and evidence cannot
    # be marked complete while its corresponding gate remains unpassed.
    if gates.get("D0") == "passed":
        if not isinstance(geography, dict) or geography.get("boundary_state") != "verified_and_frozen":
            errors.append("D0=passed requires a verified and frozen study boundary")
    if gates.get("D1") == "passed":
        if state.get("source_artifacts_downloaded") is not True or not all_sources_acquired:
            errors.append("D1=passed requires every admitted source to be acquired, verified and hashed")
    if gates.get("D2") == "passed":
        if state.get("observations_processed") is not True or state.get("baseline_computed") is not True:
            errors.append("D2=passed requires processed observations and a computed baseline")
    if gates.get("D3") == "passed" and state.get("independent_validation_complete") is not True:
        errors.append("D3=passed requires independent_validation_complete=true")
    if gates.get("D4") == "passed" and state.get("independent_review_complete") is not True:
        errors.append("D4=passed requires independent_review_complete=true")
    if gates.get("D5") == "passed" and state.get("d5_accepted") is not True:
        errors.append("D5=passed requires d5_accepted=true")
    if state.get("source_artifacts_downloaded") is True and gates.get("D1") != "passed":
        errors.append("source_artifacts_downloaded=true requires D1=passed")
    if state.get("operational_claim_authorized") is True:
        if doc.get("status") != "operational" or state.get("d5_accepted") is not True:
            errors.append("operational_claim_authorized=true requires status=operational and d5_accepted=true")

    status = doc.get("status")
    state_true = lambda key: state.get(key) is True
    no_downstream_states = ("observations_processed", "baseline_computed",
                            "independent_validation_complete", "independent_review_complete",
                            "d5_accepted", "operational_claim_authorized")
    if status == "design_only":
        if any(state_true(k) for k in STATE_FIELDS) or any(gates.get(g) == "passed" for g in EXPECTED_GATES):
            errors.append("design_only cannot claim evidence or passed gates")
    elif status == "data_readiness":
        if gates.get("D0") != "passed":
            errors.append("data_readiness status requires D0=passed")
        if any(state_true(k) for k in no_downstream_states):
            errors.append("data_readiness cannot claim processed, baseline, validated or later-stage evidence")
        if any(gates.get(g) == "passed" for g in ("D2", "D3", "D4", "D5")):
            errors.append("data_readiness status cannot have D2-D5 passed")
    elif status == "descriptive_baseline":
        if not all(gates.get(g) == "passed" for g in ("D0", "D1", "D2")):
            errors.append("descriptive_baseline status requires D0-D2=passed")
        if not all(state_true(k) for k in ("source_artifacts_downloaded", "observations_processed", "baseline_computed")):
            errors.append("descriptive_baseline status requires acquired sources, processed observations and baseline evidence")
        if any(state_true(k) for k in ("independent_validation_complete", "independent_review_complete", "d5_accepted", "operational_claim_authorized")):
            errors.append("descriptive_baseline status cannot claim validation/review/D5 or operational authorization")
        if any(gates.get(g) == "passed" for g in ("D3", "D4", "D5")):
            errors.append("descriptive_baseline status cannot have D3-D5 passed")
    elif status == "independently_validated":
        if not all(gates.get(g) == "passed" for g in ("D0", "D1", "D2", "D3")):
            errors.append("independently_validated status requires D0-D3=passed")
        if not all(state_true(k) for k in ("source_artifacts_downloaded", "observations_processed", "baseline_computed", "independent_validation_complete")):
            errors.append("independently_validated status requires source, baseline and independent-validation evidence")
        if any(state_true(k) for k in ("d5_accepted", "operational_claim_authorized")):
            errors.append("independently_validated status cannot claim D5 acceptance or operational authorization")
        if any(gates.get(g) == "passed" for g in ("D4", "D5")):
            errors.append("independently_validated status cannot have D4-D5 passed")
    elif status == "decision_support_candidate":
        required_state = ("source_artifacts_downloaded", "observations_processed", "baseline_computed",
                          "independent_validation_complete", "independent_review_complete", "d5_accepted")
        if not all(gates.get(g) == "passed" for g in EXPECTED_GATES):
            errors.append("decision_support_candidate status requires every D0-D5 gate to be passed")
        if not all(state_true(k) for k in required_state):
            errors.append("decision_support_candidate status requires source, baseline, validation, review and D5 evidence")
        if state_true("operational_claim_authorized"):
            errors.append("decision_support_candidate cannot claim operational authorization")
    elif status == "operational":
        if not all_sources_acquired or any(state.get(k) is not True for k in STATE_FIELDS):
            errors.append("operational status requires all source digests, completed evidence gates, and explicit authorization")
        if any(gates.get(g) != "passed" for g in EXPECTED_GATES):
            errors.append("operational status requires every D0-D5 gate to be passed")

    return errors


def main(argv: list[str] | None = None) -> int:
    repo_root = Path(__file__).resolve().parents[2]
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "pilot", nargs="?", type=Path,
        default=repo_root / "docs" / "pilots" / "baixo-limpopo-water-productivity.json",
    )
    args = parser.parse_args(argv)
    try:
        doc = json.loads(args.pilot.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        print(f"pilot profile read error: {exc}", file=sys.stderr)
        return 2
    errors = validate_pilot(doc)
    if errors:
        print("Water-food pilot validation failed:", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1
    print("Pilot profile valid: 2 source families; design_only; no scientific output is certified.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

