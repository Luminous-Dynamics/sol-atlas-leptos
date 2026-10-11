#!/usr/bin/env python3
"""Fail-closed contract validator for the public lunar site-screening dataset.

The file intentionally contains no site scores or point coordinates. This gate
protects that boundary until comparable, provenance-bound measurements exist.
Only the Python standard library is required.
"""
from __future__ import annotations

import json
import sys
from datetime import date
from pathlib import Path
from urllib.parse import urlparse
from typing import Any

ROOT = Path(__file__).resolve().parents[1]
DEFAULT_DATA = ROOT / "docs/data/lunar_south_pole_site_screening.v1.json"
SCHEMA_ID = "org.luminousdynamics.sol-atlas.lunar-site-screening/v1"

ROOT_KEYS = {
    "schema",
    "created_utc_date",
    "status",
    "decision_rule",
    "coordinate_policy",
    "current_program_context",
    "scoring",
    "candidates",
    "required_comparison_layers",
    "comparison_policy",
}
CANDIDATE_REQUIRED_KEYS = {
    "id", "name", "status", "proposed_roles", "evidence_state",
    "supported_observations", "unresolved", "known_metrics", "sources",
}
CANDIDATE_OPTIONAL_KEYS = {"interpretation_boundary"}
EXPECTED_NAMES = {
    "peak-near-cabeus-b": "Peak near Cabeus B",
    "haworth": "Haworth",
    "malapert-massif": "Malapert Massif",
    "mons-mouton-plateau": "Mons Mouton Plateau",
    "mons-mouton": "Mons Mouton",
    "nobile-rim-1": "Nobile Rim 1",
    "nobile-rim-2": "Nobile Rim 2",
    "de-gerlache-rim-2": "de Gerlache Rim 2",
    "slater-plain": "Slater Plain",
    "peak-near-shackleton": "Peak near Shackleton",
}
EVIDENCE_STATES = {
    "candidate_list_only",
    "regional_illumination_comms_evidence_available",
    "robotic_touchdown_outcome_available",
    "regional_science_rationale_available",
    "historical_regional_model_evidence",
}
REQUIRED_LAYERS = {
    "topography_slope_and_uncertainty",
    "boulder_small_crater_and_roughness_hazards",
    "landing_ellipse_and_dispersion",
    "date_specific_illumination",
    "temperature_and_cold_soak",
    "direct_to_earth_and_relay_visibility",
    "psr_boundaries_and_traverse_routes",
    "resource_measurement_grade_and_uncertainty",
    "radiation_and_surface_environment",
    "power_storage_and_survival_loads",
    "operations_egress_and_plume_dust_separation",
}
FORBIDDEN_LOCATION_OR_RANKING_KEYS = {
    "lat", "latitude", "lon", "long", "longitude", "coordinates", "points",
    "point", "landing_site", "landing_sites", "score", "scores", "rank",
    "ranking", "winner", "weighted_score", "composite_score", "site_score",
    "suitability_score",
}


def _is_nonempty_string(value: Any) -> bool:
    return isinstance(value, str) and bool(value.strip())


def _https_url(value: Any) -> bool:
    if not isinstance(value, str):
        return False
    parsed = urlparse(value)
    return parsed.scheme == "https" and bool(parsed.netloc)


def _check_forbidden_keys(value: Any, path: str, errors: list[str]) -> None:
    if isinstance(value, dict):
        for key, child in value.items():
            normalized = str(key).strip().lower().replace("-", "_")
            if normalized in FORBIDDEN_LOCATION_OR_RANKING_KEYS:
                errors.append(f"{path}.{key}: point-coordinate or ranking fields are prohibited")
            _check_forbidden_keys(child, f"{path}.{key}", errors)
    elif isinstance(value, list):
        for index, child in enumerate(value):
            _check_forbidden_keys(child, f"{path}[{index}]", errors)


def validate_document(document: Any) -> list[str]:
    """Return contract violations; an empty list means the document is valid."""
    errors: list[str] = []
    if not isinstance(document, dict):
        return ["root must be a JSON object"]

    missing_root = ROOT_KEYS - set(document)
    extra_root = set(document) - ROOT_KEYS
    if missing_root:
        errors.append(f"root is missing required keys: {', '.join(sorted(missing_root))}")
    if extra_root:
        errors.append(f"root contains unrecognized keys: {', '.join(sorted(extra_root))}")
    _check_forbidden_keys(document, "$", errors)

    if document.get("schema") != SCHEMA_ID:
        errors.append(f"schema must equal {SCHEMA_ID!r}")
    if not _is_nonempty_string(document.get("created_utc_date")):
        errors.append("created_utc_date must be a non-empty ISO date")
    else:
        try:
            date.fromisoformat(document["created_utc_date"])
        except ValueError:
            errors.append("created_utc_date must be a non-empty ISO date")

    if document.get("status") != "research_screening_not_site_selection":
        errors.append("status must preserve the research-only, non-selection boundary")
    if not _is_nonempty_string(document.get("decision_rule")):
        errors.append("decision_rule must be documented")
    if not _is_nonempty_string(document.get("coordinate_policy")):
        errors.append("coordinate_policy must explain that point coordinates are not asserted")
    elif "no point coordinates" not in document["coordinate_policy"].lower():
        errors.append("coordinate_policy must explicitly state that no point coordinates are asserted")

    # A blank or fabricated metric is worse than an explicit unknown. Numerical
    # metrics remain disallowed until their provenance and uncertainty are represented.
    if document.get("scoring", object()) is not None:
        errors.append("scoring must remain null; do not introduce a scalar winner")
    policy_for_weights = document.get("comparison_policy")
    if not isinstance(policy_for_weights, dict) or policy_for_weights.get("weights", object()) is not None:
        errors.append("comparison_policy.weights must remain null; use constraint/Pareto tracks")

    context = document.get("current_program_context")
    if not isinstance(context, dict):
        errors.append("current_program_context must be an object")
    else:
        expected_context_keys = {"artemis_iii", "artemis_iv", "evidence_url"}
        if set(context) != expected_context_keys:
            errors.append("current_program_context must contain only Artemis III, Artemis IV, and evidence_url")
        for key in ("artemis_iii", "artemis_iv"):
            if not _is_nonempty_string(context.get(key)):
                errors.append(f"current_program_context.{key} must be a sourced-context statement")
        if not _https_url(context.get("evidence_url")):
            errors.append("current_program_context.evidence_url must be an HTTPS URL")

    layers = document.get("required_comparison_layers")
    if not isinstance(layers, list) or not all(_is_nonempty_string(item) for item in layers):
        errors.append("required_comparison_layers must be a list of non-empty strings")
    else:
        if len(layers) != len(set(layers)):
            errors.append("required_comparison_layers must not contain duplicates")
        missing_layers = REQUIRED_LAYERS - set(layers)
        if missing_layers:
            errors.append(f"required_comparison_layers is missing: {', '.join(sorted(missing_layers))}")

    policy = document.get("comparison_policy")
    if not isinstance(policy, dict):
        errors.append("comparison_policy must be an object")
    else:
        expected_policy_keys = {"method", "weights", "output", "missing_data", "required_metadata"}
        if set(policy) != expected_policy_keys:
            errors.append("comparison_policy has missing or unrecognized keys")
        if policy.get("method") != "constraint_first_pareto":
            errors.append("comparison_policy.method must be constraint_first_pareto")
        if policy.get("output") != "separate suitability tracks; no hidden scalar winner":
            errors.append("comparison_policy.output must disclose separate suitability tracks")
        if policy.get("missing_data") != "unknown_not_zero":
            errors.append("missing data must remain unknown_not_zero")
        metadata = policy.get("required_metadata")
        expected_metadata = {
            "source_url", "source_version_or_acquisition_date", "sha256_when_available",
            "coordinate_frame", "units", "spatial_resolution", "uncertainty", "evidence_state",
        }
        if not isinstance(metadata, list) or not expected_metadata.issubset(set(metadata)):
            errors.append("comparison_policy.required_metadata must require source, coordinates frame, units, resolution, uncertainty, and evidence state")

    candidates = document.get("candidates")
    if not isinstance(candidates, list):
        errors.append("candidates must be a list")
        return errors

    seen_ids: set[str] = set()
    for index, candidate in enumerate(candidates):
        path = f"candidates[{index}]"
        if not isinstance(candidate, dict):
            errors.append(f"{path} must be an object")
            continue
        keys = set(candidate)
        missing = CANDIDATE_REQUIRED_KEYS - keys
        extra = keys - CANDIDATE_REQUIRED_KEYS - CANDIDATE_OPTIONAL_KEYS
        if missing:
            errors.append(f"{path} is missing keys: {', '.join(sorted(missing))}")
        if extra:
            errors.append(f"{path} contains unrecognized keys: {', '.join(sorted(extra))}")

        candidate_id = candidate.get("id")
        if not _is_nonempty_string(candidate_id):
            errors.append(f"{path}.id must be a non-empty string")
        else:
            if candidate_id in seen_ids:
                errors.append(f"{path}.id is duplicated: {candidate_id}")
            seen_ids.add(candidate_id)
            if candidate_id not in EXPECTED_NAMES:
                errors.append(f"{path}.id is not in the reviewed candidate set: {candidate_id}")
            elif candidate.get("name") != EXPECTED_NAMES[candidate_id]:
                errors.append(f"{path}.name does not match the reviewed ID {candidate_id!r}")

        if candidate.get("status") not in {"candidate_region", "historical_candidate_region_comparator"}:
            errors.append(f"{path}.status is not a recognized region/comparator state")
        if candidate.get("evidence_state") not in EVIDENCE_STATES:
            errors.append(f"{path}.evidence_state is not a recognized evidence state")

        for key in ("name",):
            if not _is_nonempty_string(candidate.get(key)):
                errors.append(f"{path}.{key} must be a non-empty string")
        for key in ("proposed_roles", "supported_observations", "unresolved", "sources"):
            values = candidate.get(key)
            if not isinstance(values, list) or not values:
                errors.append(f"{path}.{key} must be a non-empty list")
            elif key == "sources":
                for source_index, url in enumerate(values):
                    if not _https_url(url):
                        errors.append(f"{path}.sources[{source_index}] must be an HTTPS URL")
            elif not all(_is_nonempty_string(item) for item in values):
                errors.append(f"{path}.{key} must contain only non-empty strings")

        # Metrics are deliberately empty until sourced, epoch-specific values
        # (with units, uncertainty, and spatial resolution) can be bound in.
        if candidate.get("known_metrics") != {}:
            errors.append(f"{path}.known_metrics must remain empty until comparable evidence is provenance-bound")
        if candidate.get("status") == "historical_candidate_region_comparator" and candidate_id != "peak-near-shackleton":
            errors.append(f"{path}: only peak-near-shackleton may be the historical comparator")

    if seen_ids != set(EXPECTED_NAMES):
        missing_ids = set(EXPECTED_NAMES) - seen_ids
        unexpected_ids = seen_ids - set(EXPECTED_NAMES)
        if missing_ids:
            errors.append(f"reviewed candidate set is incomplete: {', '.join(sorted(missing_ids))}")
        if unexpected_ids:
            errors.append(f"reviewed candidate set has unexpected IDs: {', '.join(sorted(unexpected_ids))}")

    return errors


def main(argv: list[str] | None = None) -> int:
    args = sys.argv[1:] if argv is None else argv
    path = Path(args[0]) if args else DEFAULT_DATA
    try:
        document = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        print(f"ERROR: cannot read lunar site-screening JSON at {path}: {exc}", file=sys.stderr)
        return 2

    errors = validate_document(document)
    if errors:
        print(f"Lunar site-screening contract failed with {len(errors)} violation(s):", file=sys.stderr)
        for error in errors:
            print(f" - {error}", file=sys.stderr)
        return 1

    print(
        "Lunar site-screening contract passed: 10 reviewed region/comparator records; "
        "no point coordinates, scalar rankings, or unsourced numeric metrics."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
