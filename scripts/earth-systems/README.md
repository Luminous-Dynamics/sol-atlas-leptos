# Earth systems domain catalog and pilot protocols

This directory contains the machine-readable registry of research domains beyond mineral exploration, plus fail-closed checks for a focused water–food pilot.

## Validate the domain registry

From the repository root:

    python3 scripts/earth-systems/validate_catalog.py
    python3 -m unittest discover -s tests -p "test_earth_systems_catalog.py" -v

The catalog validator checks required domain/source fields, unique domain IDs, repeated source-ID consistency, HTTPS source links, required data limitations, domain-specific validation metrics, and fail-closed rights states.

## Validate the Baixo Limpopo pilot profile

From the repository root:

    python3 scripts/earth-systems/validate_pilot.py
    python3 -m unittest discover -s tests -p "test_water_food_pilot.py" -v

The pilot profile and JSON Schema are located at [docs/pilots/baixo-limpopo-water-productivity.json](../../docs/pilots/baixo-limpopo-water-productivity.json) and [docs/schemas/water-food-pilot.schema.json](../../docs/schemas/water-food-pilot.schema.json). The scientific protocol is [docs/pilots/BAIXO_LIMPOPO_WATER_PRODUCTIVITY.md](../../docs/pilots/BAIXO_LIMPOPO_WATER_PRODUCTIVITY.md).

The pilot validator uses only Python's standard library. It ensures that planned artifacts are not assigned invented hashes, that acquired artifacts require SHA-256 and verified product metadata, source rights remain explicit, CHIRPS station overlap is disclosed, and a baseline/independent-validation/operational state cannot be claimed without its prerequisites.

## Inventory the official WaPOR v3 catalogue before choosing raster layers

First list candidate mapsets and preserve the raw JSON response bytes:
```sh
python3 scripts/earth-systems/inspect_wapor_catalog.py \
  --search MBL \
  --snapshot-dir /tmp/wapor-mbl-mapsets-snapshot \
  --output /tmp/wapor-mbl-mapsets.json
```

Review the returned records, then use an exact `code` from that output to list its raster metadata:
```sh
python3 scripts/earth-systems/inspect_wapor_catalog.py \
  --mapset-code EXACT_MAPSET_CODE \
  --snapshot-dir /tmp/wapor-exact-mapset-snapshot \
  --output /tmp/wapor-exact-mapset.json
```

The inventory reads the official public FAO WaPOR v3 mapset and raster API metadata only. It follows pagination only within the official HTTPS API path, rejects literal, encoded, and double-encoded dot-segment traversal (including encoded separators), limits response sizes/pages/record counts, rejects existing output paths before discovery, records retrieval times and response SHA-256 digests, and can preserve each raw JSON response in a new directory. The [dated WaPOR Level III area review](../../docs/pilots/WAPOR_LEVEL_III_REVIEW_2026-10-13.md) records what the public FAO overview establishes and, importantly, what still requires a live catalogue snapshot. Output/snapshot paths must be new, so earlier snapshots cannot be silently overwritten. **It never downloads raster data.** Confirm mapset units, temporal codes, quality/coverage information, final versus near-real-time status, 2024 Level III re-release and March 2025 Baixo Limpopo NPP/AGBP correction history, plus exact product reuse terms before selecting files.

The first command is a catalogue search, not a guarantee that the search result is the correct mapset. The second command must use a reviewed exact code; the profile must remain `design_only` until product selection, access/rights review, boundary and source artifacts have been frozen.

## Interpretation limit

All validators here check metadata and state-machine invariants only. They do not fetch external sources, prove a license, validate a scientific model, establish source independence, or show that any domain is operational. The pilot profile is intentionally `design_only`: no WaPOR or CHIRPS artifacts have been acquired or processed. The current product IDs, temporal coverage, boundary, processing vintages, exact rights snapshots and independent reference records must be checked before the profile may advance.

The wider architecture, cross-domain boundaries, priorities and D0–D5 gates are described in [the Earth Systems Intelligence RFC](../../docs/EARTH_SYSTEMS_INTELLIGENCE_RFC.md). The highest-priority work remains the mineral benchmark in parallel with this independently scoped water–food pilot.

## Inventory CHIRPS monthly Africa files before downloading

From the repository root, run:
```sh
python3 scripts/earth-systems/plan_chirps_monthly.py \
  --start 2018-01 --end 2024-12 \
  --snapshot-index /tmp/chirps-index.html \
  --output /tmp/chirps-monthly-plan.json
```

The planner reads the official CHIRPS v3 monthly Africa-subset GeoTIFF directory index only. It can save the exact HTML index bytes at a new `--snapshot-index` path, hashes those bytes, records the retrieval timestamp, verifies that every requested month is listed, and emits candidate source URLs with null raster digests. Both `--snapshot-index` and `--output` are create-only: existing paths are rejected rather than overwritten, including a race detected at file creation. It does not download, extract, crop or process any raster. Choose the actual range only after the matching WaPOR product's final coverage has been confirmed; the example range is not a scientific recommendation. A dated browser review found the live monthly Africa index through 2026-08 on 2026-10-13; see [the dated index review note](../../docs/pilots/CHIRPS_MONTHLY_INDEX_REVIEW_2026-10-13.md). Browser rendering is not a byte snapshot, so always capture the actual index before selecting a study period.

The planner has an offline mode for testing a saved HTML listing with `--index-file path/to/index.html`. This yields `index_retrieved_at_utc: null` so an offline fixture cannot masquerade as a live source query.
