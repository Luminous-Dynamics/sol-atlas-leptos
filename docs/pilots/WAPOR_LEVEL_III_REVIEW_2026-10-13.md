# WaPOR Level III Area and Access Review — 2026-10-13

## Observation record

- **Reviewed on:** 2026-10-13 (UTC date).
- **Source:** FAO's public [WaPOR data overview](https://www.fao.org/in-action/remote-sensing-for-water-productivity/wapor-data/) and [WaPOR data access guide](https://www.fao.org/in-action/remote-sensing-for-water-productivity/wapor-data-access/en).
- **Capture mode:** browser-rendered public pages. This is a dated source review, **not** a saved raw HTTP response, live API inventory, product metadata snapshot, or raster acquisition.
- **Area-list date stated by FAO:** April 2026.

## Findings

1. FAO describes three WaPOR product levels: Level I global, Level II continental/national or river-basin, and Level III irrigation-scheme/sub-basin products. The overview describes nominal ground resolutions of 300 m, 100 m and 20 m respectively. A layer's advertised grid spacing must not be confused with its effective information resolution, accuracy, or valid-data coverage.
2. The Level III area list includes **Mozambique — Baixo Limpopo** without the asterisk that FAO uses to mark areas where “no more data is currently being produced.” **Lamego** is listed with that asterisk. This supports keeping Baixo Limpopo as a candidate geography, but it does not establish that every layer, every date, or the exact mapset needed by this pilot is currently available.
3. FAO identifies layers including evapotranspiration, land cover, net primary productivity, precipitation, phenology, quality layers, reference evapotranspiration, relative soil moisture, total biomass production and water productivity. The overview gives AETI as an example available annually, monthly and dekadally; temporal cadence must still be checked in each selected layer's metadata.
4. FAO describes a two-stage release pattern for dekadal layers: near-real-time (NRT) data first, followed after additional processing by a final product incorporating further observations and quality improvements. Monthly, seasonal and annual aggregations can inherit those changes. Do not combine NRT and final records silently or pick a study interval from the calendar alone.
5. FAO's access guide says the WaPOR v3 API does not require an API token or credentials. This supports automated public metadata discovery. It does **not** establish that this repository's client contract matches the API's current responses: the first CLI inventory still needs to capture and validate actual paginated responses.

## Consequence for the Baixo Limpopo pilot

- Keep the pilot at `design_only`. No mapset, layer, study period, boundary, raster or reference record has been admitted by this page review.
- Run `inspect_wapor_catalog.py --search MBL` or another reviewed search against the live API, preserve raw response bytes, and inspect candidate mapset codes before selecting an exact code.
- For the selected exact mapset, preserve raster metadata and verify temporal coverage, units, NRT/final state, quality layers, no-data semantics, spatial footprint, version and release/correction history.
- Resolve FAO's Level III methodology re-release and the March 2025 correction notices for Baixo Limpopo NPP/AGBP where applicable. Do not assume those corrections affect all other layers equally.
- Confirm the product-specific terms and attribution obligations before acquiring or redistributing raster artifacts. “Publicly accessible” and “API needs no token” are not blanket permission to mirror or redistribute every derived artifact.
- Select the common CHIRPS/WaPOR period only after the exact final WaPOR layer coverage and a fresh, SHA-256-recorded CHIRPS index snapshot are available.

## Explicit limitations

- No WaPOR API endpoint was queried in this review; no exact API response, mapset identifier, raster count or period is asserted.
- No raster bytes were downloaded, hashed or processed.
- A browser-rendered page is a useful dated observation, not a reproducible source-response artifact. Capture exact bytes in the acquisition environment before advancing the source-readiness gate.

## References

- FAO WaPOR data overview (level descriptions, area list, layer frequency and NRT/final semantics): https://www.fao.org/in-action/remote-sensing-for-water-productivity/wapor-data/
- FAO WaPOR data access guide (API access channels and credentials note): https://www.fao.org/in-action/remote-sensing-for-water-productivity/wapor-data-access/en
- FAO WaPOR database history: https://www.fao.org/in-action/remote-sensing-for-water-productivity/wapor-database/database-history/en
- Repository API inventory tool: ../../scripts/earth-systems/inspect_wapor_catalog.py
- Related dated CHIRPS availability review: CHIRPS_MONTHLY_INDEX_REVIEW_2026-10-13.md
