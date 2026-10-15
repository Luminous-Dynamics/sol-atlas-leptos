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

The pilot validator uses only Python's standard library. It ensures that planned artifacts are not assigned invented hashes, that acquired artifacts require SHA-256 and verified product metadata, source rights remain explicit, CHIRPS station overlap is disclosed, the provider's gauge-undercatch-correction recommendation is retained for reference comparisons, Legates–Willmott correction grids are not treated as independent observations or applied automatically, and a baseline/independent-validation/operational state cannot be claimed without its prerequisites.

## Inventory the official WaPOR v3 catalogue before choosing raster layers

First list candidate mapsets and preserve the raw JSON response bytes:
```sh
python3 scripts/earth-systems/inspect_wapor_catalog.py \
  --search AETI \
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

The inventory reads the official public FAO WaPOR v3 `/mapsets` and raster API metadata only. It follows pagination only within the official HTTPS API path, rejects literal, encoded, and double-encoded dot-segment traversal (including encoded separators), caps each JSON response at 16 MiB and aggregate response bytes per paginated listing at 64 MiB, limits pages/record counts, rejects existing output paths before discovery, records retrieval times and response SHA-256 digests, and can preserve each raw JSON response in a new directory. Important: the `/mapsets` endpoint is not an exhaustive Level III area inventory; do not infer that a Level III product is absent because its code is missing from this result. Use the area-specific MOSAICSET index below for Level III discovery. Before writing, the tool verifies that each response hash matches its bytes and that metadata/page counts and snapshot labels are consistent; it rejects mismatches before creating the output directory. If writing a page fails, the tool removes the incomplete snapshot; if writing the inventory manifest fails after capture, it removes that newly created snapshot too. Cleanup failures are reported rather than silently ignored. If the JSON inventory file itself is only partially written, the tool removes that incomplete file before returning an error; when JSON is sent to stdout and the pipe fails, it removes the associated new snapshot instead of leaving unpaired raw responses. Existing output files are never deleted on a collision. The [dated WaPOR Level III area review](../../docs/pilots/WAPOR_LEVEL_III_REVIEW_2026-10-13.md) records what the public FAO overview establishes and, importantly, what still requires a live catalogue snapshot. Output/snapshot paths must be new, so earlier snapshots cannot be silently overwritten. **It never downloads raster data.** Confirm mapset units, temporal codes, quality/coverage information, final versus near-real-time status, 2024 Level III re-release and March 2025 Baixo Limpopo NPP/AGBP correction history, plus exact product reuse terms before selecting files.

The first command is a catalogue search, not a guarantee that the search result is the correct mapset. The second command must use a reviewed exact code; the profile must remain `design_only` until product selection, access/rights review, boundary and source artifacts have been frozen.

## Conflict-to-supply-chain first-case readiness

The first source-readiness candidate is the [Black Sea Grain Initiative (2022–2023)](../../docs/pilots/BLACK_SEA_GRAIN_INITIATIVE_READINESS.md). Its machine-readable record is [black-sea-grain-initiative-readiness.json](../../docs/pilots/black-sea-grain-initiative-readiness.json). The current published-data transcription is explicitly discovery-only: the official UN page bytes and monthly FAO CSV bytes have **not** been acquired or hashed, the UCDP ZIP has not been acquired, and the Comtrade query has not run. The JSON holds the publisher-reported monthly Initiative values for an arithmetic transcription check; it is not a frozen evidence artifact and does not satisfy source qualification.

The profile keeps July 2023 partial through 14 July and prevents pre-August values in the Initiative-specific series being imputed as zero. The FAO index is downstream context, not proof of causality. Validate the source bytes, reparse the table/CSV, freeze query parameters, establish controls and then conduct independent review before promoting the case beyond C0.

## Interpretation limit

All validators here check metadata and state-machine invariants only. They do not fetch external sources, prove a license, validate a scientific model, establish source independence, or show that any domain is operational. The pilot profile is intentionally `design_only`: no WaPOR or CHIRPS artifacts have been acquired or processed. The current product IDs, temporal coverage, boundary, processing vintages, exact rights snapshots and independent reference records must be checked before the profile may advance.

The catalog now includes a candidate global conflict / supply-chain resilience domain in addition to Earth-observation domains. Its source entries are metadata-only; no conflict, trade, port or humanitarian feed is connected by this catalog change. UCDP's listed CC BY 4.0 status is scoped to the documented dataset terms, while ACLED, GDELT, PortWatch, Comtrade, HAPI, Open Supply Hub and EPCIS remain product/access review-gated as applicable.

The wider architecture, cross-domain boundaries, priorities and D0–D5 gates are described in [the Earth Systems Intelligence RFC](../../docs/EARTH_SYSTEMS_INTELLIGENCE_RFC.md). The mineral and water–food work remains active; the conflict/supply-chain track is research-only until frozen source snapshots, outcome records and independent backtests exist. The highest-priority work remains the mineral benchmark in parallel with this independently scoped water–food pilot.

## Inventory WaPOR Level III data by area code

FAO lists Baixo Limpopo's Level III area code as `MBL`. Level III files are published through a separate `MOSAICSET` directory index; the `/mapsets` API inventory above is not sufficient to discover area-specific Level III files. Capture the area-specific index before selecting products or dates:

```sh
python3 scripts/earth-systems/inspect_wapor_level3_index.py \
  --mapset-code L3-AETI-M \
  --area-code MBL \
  --snapshot-index /tmp/wapor-l3-mbl-aeti-index.html \
  --output /tmp/wapor-l3-mbl-aeti-index.json
```

The tool snapshots and hashes the exact HTML index, lists files for the requested area, separates raster links from JSON sidecar links, and reports all area codes visible under that mapset. It does **not** download GeoTIFFs or sidecar JSON bytes, and a listing is not proof of finality, scientific validity, or reuse rights. Run its offline regression suite with `python3 -m unittest discover -s tests -p "test_wapor_level3_index.py" -v`. If no `MBL` entries appear, treat that as a dated directory observation and investigate the portal/catalogue rather than claiming the product is unavailable globally. Verify exact sidecar metadata, period semantics, scale/units, final/NRT status, correction vintage, spatial support and terms before acquisition. FAO's published area code is grounded in its [WaPOR data-access correction notice](https://www.fao.org/in-action/remote-sensing-for-water-productivity/wapor-data-access/en), and the [pinned WaPOR v3 API tutorial](https://github.com/un-fao/FAO-Water-Applications/blob/10a70cda0e0023258bc9152e43a73b6e8e20b669/WaPOR/WaPORv3_API.ipynb) documents the separate mapset-family endpoint.

## Capture the CHIRPS source index before selecting a study period

The period-free snapshot mode captures the current official HTML index and hashes its exact bytes without choosing any analysis months:
```sh
python3 scripts/earth-systems/plan_chirps_monthly.py \
  --snapshot-only \
  --snapshot-index /tmp/chirps-index.html \
  --output /tmp/chirps-index-manifest.json
```

The manifest marks `requested_period: null` and states that no analysis period was selected; no GeoTIFF is downloaded. Use this first when studying source availability before the WaPOR layer and temporal overlap are known. The snapshot and manifest paths must be new.

The existing **Earth systems catalog validation** workflow attempts separate CHIRPS-index and WaPOR `/mapsets` source captures, a bounded metadata-only probe of national `L2-AETI-M`, and an area-specific Level III index capture for `L3-AETI-M` / `MBL`. Each successful capture is uploaded as a 14-day artifact; the source jobs are explicitly non-gating, so a source outage does not invalidate deterministic unit tests. Read the job logs and inspect the artifact presence before claiming a source snapshot exists. A failed or missing WaPOR capture is a blocked inventory—not evidence that the catalog is empty or the product unavailable. The WaPOR capture contains API metadata and raw JSON responses only, not raster files. After this manual workflow is available on the repository's default branch, **Actions → Earth systems source snapshot (manual) → Run workflow** can be used for an intentional refresh between code changes.

## Inventory CHIRPS monthly Africa files before downloading

From the repository root, run:
```sh
python3 scripts/earth-systems/plan_chirps_monthly.py \
  --start 2018-01 --end 2024-12 \
  --snapshot-index /tmp/chirps-index.html \
  --output /tmp/chirps-monthly-plan.json
```

The planner reads the official CHIRPS v3 monthly Africa-subset GeoTIFF directory index only. It can save the exact HTML index bytes at a new `--snapshot-index` path, hashes those bytes, records the retrieval timestamp, verifies that every requested month is listed, and emits candidate source URLs with null raster digests. Both `--snapshot-index` and `--output` are create-only: existing paths are rejected rather than overwritten, including a race detected at file creation. If either write fails after creating a partial file, the planner removes that partial artifact; if the plan cannot be saved after this invocation captured the index, the newly created index snapshot is also removed. Cleanup failures are reported. It does not download, extract, crop or process any raster. Choose the actual range only after the matching WaPOR product's final coverage has been confirmed; the example range is not a scientific recommendation. The browser view showed the monthly Africa index through 2026-08 on both 2026-10-13 and the follow-up check on 2026-10-14; see the [2026-10-13 observation](../../docs/pilots/CHIRPS_MONTHLY_INDEX_REVIEW_2026-10-13.md) and [2026-10-14 recheck](../../docs/pilots/CHIRPS_MONTHLY_INDEX_REVIEW_2026-10-14.md). These are observations, not raw-byte snapshots; always capture the actual index before selecting a study period.

The planner has an offline mode for testing a saved HTML listing with `--index-file path/to/index.html`. This yields `index_retrieved_at_utc: null` so an offline fixture cannot masquerade as a live source query. Local inputs must be regular files; the planner rejects sources larger than 16 MiB from file metadata before opening them and uses a capped read to guard against a file growing after the size check.
