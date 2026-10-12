# Earth systems domain catalog

This directory contains a machine-readable registry of research domains beyond mineral exploration.

## Validate the registry

From the repository root:

    python3 scripts/earth-systems/validate_catalog.py
    python3 -m unittest discover -s tests -p "test_earth_systems_catalog.py" -v

The validator uses only the Python standard library. It checks required domain/source fields, unique domain IDs, repeated source-ID consistency, HTTPS source links, required data limitations, domain-specific validation metrics, and fail-closed rights states.

Validation is structural only: it does not fetch external sources, prove a license, validate a scientific model, or establish that a domain is implemented.

## Current domain inventory

The canonical machine-readable file is [docs/earth-systems-domain-catalog.json](../../docs/earth-systems-domain-catalog.json). The architecture, prioritized pilots, source limits and validation gates are described in [the Earth Systems Intelligence RFC](../../docs/EARTH_SYSTEMS_INTELLIGENCE_RFC.md).

The highest-priority follow-on to the mineral benchmark is a Southern Africa water–food-security pilot, with hazard observation as another good validation domain. This choice tests different scientific assumptions and decision contexts using established public source systems, while keeping local ground-truth needs explicit.
