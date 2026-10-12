#!/usr/bin/env python3
"""Validate the Earth systems research-domain catalog using Python's standard library."""
from __future__ import annotations

import argparse
import json
import sys
from datetime import date
from pathlib import Path
from urllib.parse import urlsplit
from typing import Any

REQUIRED_TOP = {"catalog_version", "last_reviewed", "title", "policy", "domains"}
REQUIRED_DOMAIN = {
    "id", "title", "priority", "readiness", "question", "scope", "target_geography",
    "output_types", "data_sources", "required_checks", "prohibited_inferences", "validation",
}
REQUIRED_SOURCE = {
    "source_id", "name", "provider", "url", "role", "access_status", "rights_state",
    "redistribution_allowed", "key_limitations",
}
ALLOWED_ACCESS = {
    "published_reference", "public_catalog", "public_with_terms",
    "account_or_api_key", "api_and_product_terms",
}
ALLOWED_PRIORITIES = {"P0", "P1", "P2", "P3"}
ALLOWED_READINESS = {"candidate", "benchmark_in_progress", "future_research"}
REDISTRIBUTION_ALLOWLIST = {"cc_by_4_0_with_attribution"}


def validate_catalog(doc: Any) -> list[str]:
    errors: list[str] = []
    if not isinstance(doc, dict):
        return ["catalog root must be an object"]
    missing_top = sorted(REQUIRED_TOP - set(doc))
    if missing_top:
        errors.append(f"catalog missing top-level fields: {missing_top}")
    if not isinstance(doc.get("catalog_version"), str) or not doc.get("catalog_version"):
        errors.append("catalog_version must be a non-empty string")
    try:
        date.fromisoformat(doc.get("last_reviewed", ""))
    except (TypeError, ValueError):
        errors.append("last_reviewed must be an ISO date (YYYY-MM-DD)")
    policy = doc.get("policy")
    if not isinstance(policy, dict):
        errors.append("policy must be an object")
    else:
        if not isinstance(policy.get("common_rules"), list) or len(policy["common_rules"]) < 5:
            errors.append("policy.common_rules must contain at least five rules")
        architecture = policy.get("architecture")
        if not isinstance(architecture, dict) or not all(
            isinstance(architecture.get(k), str) and architecture[k].strip()
            for k in ("sol_atlas", "symthaea", "mycelix", "symtropy")
        ):
            errors.append("policy.architecture must define Sol-Atlas, Symthaea, Mycelix and Symtropy roles")

    domains = doc.get("domains")
    if not isinstance(domains, list) or not domains:
        return errors + ["domains must be a non-empty array"]

    ids: set[str] = set()
    source_definitions: dict[str, tuple[str, str]] = {}
    for index, domain in enumerate(domains):
        where = f"domains[{index}]"
        if not isinstance(domain, dict):
            errors.append(f"{where} must be an object")
            continue
        missing = sorted(REQUIRED_DOMAIN - set(domain))
        if missing:
            errors.append(f"{where} missing fields: {missing}")
        domain_id = domain.get("id")
        if not isinstance(domain_id, str) or not domain_id:
            errors.append(f"{where}.id must be a non-empty string")
        elif domain_id in ids:
            errors.append(f"duplicate domain id: {domain_id}")
        else:
            ids.add(domain_id)
        if domain.get("priority") not in ALLOWED_PRIORITIES:
            errors.append(f"{where}.priority is invalid")
        if domain.get("readiness") not in ALLOWED_READINESS:
            errors.append(f"{where}.readiness is invalid")
        for field in ("title", "question", "scope"):
            if not isinstance(domain.get(field), str) or not domain[field].strip():
                errors.append(f"{where}.{field} must be a non-empty string")
        for field in ("target_geography", "output_types", "required_checks", "prohibited_inferences"):
            value = domain.get(field)
            minimum = 2 if field == "prohibited_inferences" else 1
            if not isinstance(value, list) or len(value) < minimum or any(
                not isinstance(item, str) or not item.strip() for item in value
            ):
                errors.append(f"{where}.{field} must contain at least {minimum} non-empty strings")

        validation = domain.get("validation")
        if not isinstance(validation, dict):
            errors.append(f"{where}.validation must be an object")
        else:
            if not isinstance(validation.get("split_strategy"), str) or len(validation["split_strategy"].strip()) < 10:
                errors.append(f"{where}.validation.split_strategy must be specific")
            if not isinstance(validation.get("metrics"), list) or len(validation["metrics"]) < 3:
                errors.append(f"{where}.validation.metrics must contain at least three metrics")
            if validation.get("baseline_required") is not True:
                errors.append(f"{where}.validation.baseline_required must be true")
            if validation.get("independent_review_required") is not True:
                errors.append(f"{where}.validation.independent_review_required must be true")

        sources = domain.get("data_sources")
        if not isinstance(sources, list) or not sources:
            errors.append(f"{where}.data_sources must be a non-empty array")
            continue
        domain_source_ids: set[str] = set()
        for source_index, source in enumerate(sources):
            sw = f"{where}.data_sources[{source_index}]"
            if not isinstance(source, dict):
                errors.append(f"{sw} must be an object")
                continue
            missing_source = sorted(REQUIRED_SOURCE - set(source))
            if missing_source:
                errors.append(f"{sw} missing fields: {missing_source}")
            source_id = source.get("source_id")
            if not isinstance(source_id, str) or not source_id:
                errors.append(f"{sw}.source_id must be non-empty")
            elif source_id in domain_source_ids:
                errors.append(f"{where} repeats source_id {source_id}")
            else:
                domain_source_ids.add(source_id)
            for field in ("name", "provider", "role", "rights_state"):
                if not isinstance(source.get(field), str) or not source[field].strip():
                    errors.append(f"{sw}.{field} must be a non-empty string")
            url = source.get("url")
            parts = urlsplit(url) if isinstance(url, str) else None
            if not parts or parts.scheme != "https" or not parts.netloc:
                errors.append(f"{sw}.url must be an HTTPS URL")
            access = source.get("access_status")
            if access not in ALLOWED_ACCESS:
                errors.append(f"{sw}.access_status is invalid")
            allowed = source.get("redistribution_allowed")
            if allowed not in (True, False, None):
                errors.append(f"{sw}.redistribution_allowed must be true, false or null")
            if allowed is True and source.get("rights_state") not in REDISTRIBUTION_ALLOWLIST:
                errors.append(f"{sw}: fail-closed rule: rights state does not permit redistribution=true")
            limitations = source.get("key_limitations")
            if not isinstance(limitations, list) or not limitations or any(
                not isinstance(limit, str) or not limit.strip() for limit in limitations
            ):
                errors.append(f"{sw}.key_limitations must list at least one concrete limitation")

            # Reused source IDs must resolve to the same canonical resource/provider.
            if isinstance(source_id, str) and source_id and isinstance(url, str) and isinstance(source.get("provider"), str):
                prior = source_definitions.get(source_id)
                definition = (source.get("provider", ""), url)
                if prior is not None and prior != definition:
                    errors.append(f"{sw}: source_id {source_id} conflicts with earlier provider/URL {prior}")
                else:
                    source_definitions[source_id] = definition

    return errors


def main(argv: list[str] | None = None) -> int:
    repo_root = Path(__file__).resolve().parents[2]
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("catalog", nargs="?", type=Path, default=repo_root / "docs" / "earth-systems-domain-catalog.json")
    args = parser.parse_args(argv)
    try:
        doc = json.loads(args.catalog.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        print(f"catalog read error: {exc}", file=sys.stderr)
        return 2
    errors = validate_catalog(doc)
    if errors:
        print("Earth systems catalog validation failed:", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 1
    print(f"Catalog valid: {len(doc['domains'])} domains, {sum(len(d['data_sources']) for d in doc['domains'])} source records.")
    print("This is structural validation only; no source was fetched and no scientific claim was evaluated.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
