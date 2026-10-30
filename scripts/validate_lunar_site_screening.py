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
DEFAULT_CATALOG = ROOT / "docs/data/lunar_geospatial_source_catalog.v1.json"
CATALOG_SCHEMA_ID = "org.luminousdynamics.sol-atlas.lunar-source-catalog/v1"
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
    "connecting-ridge": "Connecting Ridge",
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
        if not isinstance(metadata, list) or not all(_is_nonempty_string(item) for item in metadata):
            errors.append("comparison_policy.required_metadata must be a list of non-empty strings")
        elif not expected_metadata.issubset(set(metadata)):
            errors.append("comparison_policy.required_metadata must require source, coordinate frame, units, resolution, uncertainty, and evidence state")

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

        candidate_status = candidate.get("status")
        if not isinstance(candidate_status, str) or candidate_status not in {
            "candidate_region", "historical_candidate_region_comparator"
        }:
            errors.append(f"{path}.status is not a recognized region/comparator state")
        candidate_evidence_state = candidate.get("evidence_state")
        if not isinstance(candidate_evidence_state, str) or candidate_evidence_state not in EVIDENCE_STATES:
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
        if (
            candidate_status == "historical_candidate_region_comparator"
            and candidate_id not in {"peak-near-shackleton", "connecting-ridge"}
        ):
            errors.append(f"{path}: only reviewed historical comparator IDs may use this status")

    if seen_ids != set(EXPECTED_NAMES):
        missing_ids = set(EXPECTED_NAMES) - seen_ids
        unexpected_ids = seen_ids - set(EXPECTED_NAMES)
        if missing_ids:
            errors.append(f"reviewed candidate set is incomplete: {', '.join(sorted(missing_ids))}")
        if unexpected_ids:
            errors.append(f"reviewed candidate set has unexpected IDs: {', '.join(sorted(unexpected_ids))}")

    return errors



SOURCE_REQUIRED_KEYS = {
    "id", "title", "publisher", "url", "product_families",
    "spatial_resolution", "coordinate_reference", "temporal_scope",
    "uncertainty_and_limitations", "recommended_use",
    "required_acceptance_checks", "candidate_crosswalk",
}
SOURCE_OPTIONAL_KEYS = {"sample_direct_assets", "asset_access_notes"}
SOURCE_IDS = {
    "pgda-lola-high-resolution-site-products",
    "pgda-lola-polar-illumination",
    "pgda-lola-south-pole-dem-mosaic",
    "nasa-svs-south-pole-illumination-2025-2028",
    "lroc-nac-shackleton-de-gerlache-connecting-ridge-dtm",
    "lroc-global-nac-dtm-footprint-shapefiles",
    "usgs-artemis-iii-candidate-navigation-grids",
}
LEAD_IDS = {
    "connecting-ridge-illumination-lead",
    "five-metre-pixels-are-not-five-metre-ground-truth",
    "long-period-illumination-is-not-an-operational-forecast",
}
ASSET_REQUIRED_KEYS = {"title", "url", "asset_type", "verification_state"}
ASSET_OPTIONAL_KEYS = {"note", "sha256"}


def validate_source_catalog(catalog: Any, screening_document: Any) -> list[str]:
    """Validate source metadata and make accidental provenance overclaims fail closed."""
    errors: list[str] = []
    if not isinstance(catalog, dict):
        return ["catalog root must be a JSON object"]

    root_keys = {
        "schema", "created_utc_date", "status", "decision_rule",
        "metric_policy", "sources", "ingestion_sequence", "historical_leads",
    }
    missing = root_keys - set(catalog)
    extra = set(catalog) - root_keys
    if missing:
        errors.append(f"catalog root is missing keys: {', '.join(sorted(missing))}")
    if extra:
        errors.append(f"catalog root contains unrecognized keys: {', '.join(sorted(extra))}")
    if catalog.get("schema") != CATALOG_SCHEMA_ID:
        errors.append(f"catalog schema must equal {CATALOG_SCHEMA_ID!r}")
    created = catalog.get("created_utc_date")
    if not _is_nonempty_string(created):
        errors.append("catalog created_utc_date must be a non-empty ISO date")
    else:
        try:
            date.fromisoformat(created)
        except ValueError:
            errors.append("catalog created_utc_date must be a non-empty ISO date")
    if catalog.get("status") != "source_catalogue_ingestion_plan_not_ingested":
        errors.append("catalog status must not imply source products have been ingested")
    if not _is_nonempty_string(catalog.get("decision_rule")):
        errors.append("catalog decision_rule must be documented")
    metric_policy = catalog.get("metric_policy")
    if not _is_nonempty_string(metric_policy) or "No derived site metrics" not in metric_policy:
        errors.append("catalog metric_policy must explicitly state that no derived site metrics are asserted")

    sequence = catalog.get("ingestion_sequence")
    if not isinstance(sequence, list) or not sequence or not all(_is_nonempty_string(s) for s in sequence):
        errors.append("catalog ingestion_sequence must be a non-empty list of non-empty strings")

    sources = catalog.get("sources")
    if not isinstance(sources, list):
        errors.append("catalog sources must be a list")
        sources = []
    seen_source_ids: set[str] = set()
    for index, source in enumerate(sources):
        path = f"sources[{index}]"
        if not isinstance(source, dict):
            errors.append(f"{path} must be an object")
            continue
        keys = set(source)
        missing_source_keys = SOURCE_REQUIRED_KEYS - keys
        extra_source_keys = keys - SOURCE_REQUIRED_KEYS - SOURCE_OPTIONAL_KEYS
        if missing_source_keys:
            errors.append(f"{path} is missing keys: {', '.join(sorted(missing_source_keys))}")
        if extra_source_keys:
            errors.append(f"{path} contains unrecognized keys: {', '.join(sorted(extra_source_keys))}")

        source_id = source.get("id")
        if not _is_nonempty_string(source_id):
            errors.append(f"{path}.id must be a non-empty string")
        else:
            if source_id in seen_source_ids:
                errors.append(f"{path}.id is duplicated: {source_id}")
            seen_source_ids.add(source_id)
            if source_id not in SOURCE_IDS:
                errors.append(f"{path}.id is not in the reviewed source set: {source_id}")

        for key in (
            "title", "publisher", "spatial_resolution", "coordinate_reference",
            "temporal_scope", "recommended_use",
        ):
            if not _is_nonempty_string(source.get(key)):
                errors.append(f"{path}.{key} must be a non-empty string")
        if not _https_url(source.get("url")):
            errors.append(f"{path}.url must be an HTTPS URL")
        for key in ("product_families", "uncertainty_and_limitations", "required_acceptance_checks"):
            values = source.get(key)
            if not isinstance(values, list) or not values or not all(_is_nonempty_string(v) for v in values):
                errors.append(f"{path}.{key} must be a non-empty list of non-empty strings")

        crosswalk = source.get("candidate_crosswalk")
        if not isinstance(crosswalk, list) or not crosswalk:
            errors.append(f"{path}.candidate_crosswalk must be a non-empty list")
        else:
            seen_crosswalk_ids: set[str] = set()
            for entry_index, entry in enumerate(crosswalk):
                entry_path = f"{path}.candidate_crosswalk[{entry_index}]"
                if not isinstance(entry, dict):
                    errors.append(f"{entry_path} must be an object")
                    continue
                allowed_crosswalk_keys = {"candidate_id", "product_site", "mapping_state", "note"}
                if set(entry) - allowed_crosswalk_keys:
                    errors.append(f"{entry_path} contains unrecognized keys")
                candidate_id = entry.get("candidate_id")
                if not _is_nonempty_string(candidate_id):
                    errors.append(f"{entry_path}.candidate_id must be a non-empty string")
                elif candidate_id in seen_crosswalk_ids:
                    errors.append(f"{entry_path}.candidate_id is duplicated: {candidate_id}")
                else:
                    seen_crosswalk_ids.add(candidate_id)
                product_site = entry.get("product_site")
                if product_site is not None and not _is_nonempty_string(product_site):
                    errors.append(f"{entry_path}.product_site must be null or a non-empty string")
                if not _is_nonempty_string(entry.get("mapping_state")):
                    errors.append(f"{entry_path}.mapping_state must be a non-empty string")
                if "note" in entry and not _is_nonempty_string(entry.get("note")):
                    errors.append(f"{entry_path}.note must be a non-empty string when present")

            if source_id == "pgda-lola-high-resolution-site-products":
                if not isinstance(screening_document, dict):
                    errors.append("screening document must be an object to validate candidate crosswalk coverage")
                else:
                    candidates = screening_document.get("candidates")
                    expected_candidate_ids = {
                        candidate.get("id") for candidate in candidates
                        if isinstance(candidates, list) and isinstance(candidate, dict)
                        and _is_nonempty_string(candidate.get("id"))
                    } if isinstance(candidates, list) else set()
                    if seen_crosswalk_ids != expected_candidate_ids:
                        errors.append(
                            f"{entry_path}: primary LOLA crosswalk must match the full reviewed candidate ID set"
                        )

        assets = source.get("sample_direct_assets", [])
        if not isinstance(assets, list):
            errors.append(f"{path}.sample_direct_assets must be a list when present")
            assets = []
        for asset_index, asset in enumerate(assets):
            asset_path = f"{path}.sample_direct_assets[{asset_index}]"
            if not isinstance(asset, dict):
                errors.append(f"{asset_path} must be an object")
                continue
            asset_keys = set(asset)
            missing_asset_keys = ASSET_REQUIRED_KEYS - asset_keys
            extra_asset_keys = asset_keys - ASSET_REQUIRED_KEYS - ASSET_OPTIONAL_KEYS
            if missing_asset_keys:
                errors.append(f"{asset_path} is missing keys: {', '.join(sorted(missing_asset_keys))}")
            if extra_asset_keys:
                errors.append(f"{asset_path} contains unrecognized keys: {', '.join(sorted(extra_asset_keys))}")
            for key in ("title", "asset_type", "verification_state"):
                if not _is_nonempty_string(asset.get(key)):
                    errors.append(f"{asset_path}.{key} must be a non-empty string")
            if not _https_url(asset.get("url")):
                errors.append(f"{asset_path}.url must be an HTTPS URL")
            state = asset.get("verification_state", "")
            if isinstance(state, str) and "downloaded_and_hashed" in state:
                digest = asset.get("sha256")
                if not isinstance(digest, str) or len(digest) != 64 or any(ch not in "0123456789abcdefABCDEF" for ch in digest):
                    errors.append(f"{asset_path}: downloaded_and_hashed state requires a 64-character SHA-256")
            if "note" in asset and not _is_nonempty_string(asset.get("note")):
                errors.append(f"{asset_path}.note must be a non-empty string when present")

        access_notes = source.get("asset_access_notes", [])
        if not isinstance(access_notes, list) or not all(_is_nonempty_string(n) for n in access_notes):
            errors.append(f"{path}.asset_access_notes must be a list of non-empty strings when present")

    if seen_source_ids != SOURCE_IDS:
        if SOURCE_IDS - seen_source_ids:
            errors.append(f"catalog source set is incomplete: {', '.join(sorted(SOURCE_IDS - seen_source_ids))}")
        if seen_source_ids - SOURCE_IDS:
            errors.append(f"catalog source set has unexpected IDs: {', '.join(sorted(seen_source_ids - SOURCE_IDS))}")

    leads = catalog.get("historical_leads")
    if not isinstance(leads, list):
        errors.append("catalog historical_leads must be a list")
        leads = []
    seen_lead_ids: set[str] = set()
    for index, lead in enumerate(leads):
        path = f"historical_leads[{index}]"
        if not isinstance(lead, dict):
            errors.append(f"{path} must be an object")
            continue
        required = {
            "id", "title", "related_region", "finding", "evidence_state",
            "source_url", "time_window", "next_action", "interpretation_limits",
        }
        optional = {"heights_above_surface_m"}
        missing = required - set(lead)
        extra = set(lead) - required - optional
        if missing:
            errors.append(f"{path} is missing keys: {', '.join(sorted(missing))}")
        if extra:
            errors.append(f"{path} contains unrecognized keys: {', '.join(sorted(extra))}")
        lead_id = lead.get("id")
        if not _is_nonempty_string(lead_id):
            errors.append(f"{path}.id must be a non-empty string")
        else:
            if lead_id in seen_lead_ids:
                errors.append(f"{path}.id is duplicated: {lead_id}")
            seen_lead_ids.add(lead_id)
            if lead_id not in LEAD_IDS:
                errors.append(f"{path}.id is not in the reviewed lead set: {lead_id}")
        for key in ("title", "related_region", "finding", "evidence_state", "time_window", "next_action"):
            if not _is_nonempty_string(lead.get(key)):
                errors.append(f"{path}.{key} must be a non-empty string")
        if not _https_url(lead.get("source_url")):
            errors.append(f"{path}.source_url must be an HTTPS URL")
        limitations = lead.get("interpretation_limits")
        if not isinstance(limitations, list) or not limitations or not all(_is_nonempty_string(v) for v in limitations):
            errors.append(f"{path}.interpretation_limits must be a non-empty list of non-empty strings")
        heights = lead.get("heights_above_surface_m", [])
        if not isinstance(heights, list) or any(not isinstance(h, (int, float)) or isinstance(h, bool) or h < 0 for h in heights):
            errors.append(f"{path}.heights_above_surface_m must contain only non-negative numbers")
        if lead_id == "connecting-ridge-illumination-lead":
            boundary_text = " ".join(limitations) if isinstance(limitations, list) else ""
            if "not a new measurement" not in boundary_text:
                errors.append(f"{path} must state that the source-reported result is not a new measurement")

    if seen_lead_ids != LEAD_IDS:
        errors.append("catalog historical lead set must contain the three reviewed IDs")
    return errors


def main(argv: list[str] | None = None) -> int:
    args = sys.argv[1:] if argv is None else argv
    path = Path(args[0]) if args else DEFAULT_DATA
    try:
        document = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        print(f"ERROR: cannot read lunar site-screening JSON at {path}: {exc}", file=sys.stderr)
        return 2

    try:
        catalog = json.loads(DEFAULT_CATALOG.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        print(f"ERROR: cannot read lunar geospatial source catalogue at {DEFAULT_CATALOG}: {exc}", file=sys.stderr)
        return 2

    errors = validate_document(document)
    catalog_errors = validate_source_catalog(catalog, document)
    if errors or catalog_errors:
        total = len(errors) + len(catalog_errors)
        print(f"Lunar site-screening/source-catalog contract failed with {total} violation(s):", file=sys.stderr)
        for error in errors:
            print(f" - screening: {error}", file=sys.stderr)
        for error in catalog_errors:
            print(f" - source catalog: {error}", file=sys.stderr)
        return 1

    print(
        "Lunar evidence contracts passed: 11 reviewed region/comparator records, "
        "7 reviewed geospatial source families, and 3 bounded historical leads; "
        "no asserted site rankings or unverified asset hashes."
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
