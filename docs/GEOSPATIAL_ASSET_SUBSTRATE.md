# Sol Atlas Geospatial Asset Substrate

**Status:** Proposed architecture / implementation plan  
**Date researched:** 2026-10-10  
**Scope:** imagery, elevation, vector geography, 3D city content, temporal observations, asset provenance, renderer interoperability  
**Decision owner:** Sol Atlas maintainers

## Summary

Sol Atlas should grow from a globe with domain-specific layers into an evidence-aware planetary exploration system. The next step is not to replace the existing globe or immediately commit to a new renderer. It is to establish a renderer-neutral geospatial asset contract and test the real data path end-to-end.

The target architecture separates five concerns:

1. **Discovery/catalog:** find what data exists, where it covers, when it was captured, what it costs, and how it may be used.
2. **Acquisition and validation:** retrieve allowed assets, capture immutable source snapshots when appropriate, validate metadata and format, and record limitations.
3. **Processing and delivery:** serve or package raster, vector, terrain, and 3D content in formats suited to each asset and deployment mode.
4. **Domain semantics:** preserve stable geographic/capability identities, temporal semantics, provenance, and explicit uncertainty in `sol-atlas-core`.
5. **Presentation:** let the current WebGL2 renderer or a future renderer consume the same validated projection without becoming an authority on evidence or qualification.

This document is a proposal, not evidence that the ingestion pipeline or these integrations already exist.

## Current baseline

The current application initializes a static data baseline via `static_data::load_all()`, with an optional Holochain-backed overlay behind a feature flag. The shared `sol-atlas-core` crate already contains geography, level-of-detail, data, H3 and capability-related logic. Recent capability/provenance work establishes valuable constraints: capability and instance identity are distinct; H3 aggregation must preserve reversibility; a source locator is not source verification; and structural validity does not establish a claim's truth.

Treat static fixtures as a useful bootstrapping/fallback path, not as the eventual global imagery delivery system. Extend the existing design rather than replacing its domain semantics with generic map properties.

## Architectural decision

### 1. Make the asset catalog renderer-neutral

Introduce a versioned internal asset model in the core or a narrowly scoped data crate. An asset is a specific dataset or captured artifact, not simply a URL or a label.

The exact Rust type layout is a follow-on implementation decision; the model should carry, where applicable:

- stable asset ID and schema version;
- human-readable title and asset kind (imagery, elevation, vector features, 3D tiles, point cloud, scientific raster, or packaged tiles);
- provider/publisher identity and canonical source locator;
- explicit licence, attribution text/link, access terms, redistribution restrictions, and review status;
- spatial footprint and coordinate reference information, including safe handling of antimeridian-crossing coverage;
- observation/acquisition time as an interval when necessary, publication time, catalog discovery time, local retrieval time, and processing time as distinct concepts;
- stated ground sampling distance or spatial resolution with units, native dimensions where available, format/media type, and known limitations;
- references to captured artifact snapshots, content digests, processing activities, and inputs where known;
- current availability and validation dimensions, each with a precise meaning;
- explicit missing, redacted, unknown, unavailable, and unresolved states rather than fabricated defaults.

Asset identity must not be derived from its mutable URL alone. Two snapshots from the same source URL may represent different content. Conversely, a digest proves equality to the bytes hashed only when those bytes were actually captured and the digest was independently recomputed; it does not prove who published them or whether their assertions are true.

Keep external STAC/OGC identities and source locators as interoperability fields. They must not silently become Mycelix authority identifiers.

### 2. Adopt existing open geospatial formats where they fit

Use standards as adapters, not as a single mandated storage engine:

| Data / role | Preferred interoperability path | Why |
| --- | --- | --- |
| Data discovery and spatiotemporal metadata | STAC-compatible catalog and item/asset adapter | Consistent search by place, time and asset; supports source catalogs without requiring one provider |
| Large raster source products and analysis | Cloud Optimized GeoTIFF (COG) where the source supports it | Range requests and internal tiling/overviews allow clients and processors to read relevant regions without downloading a whole large raster |
| Network-delivered maps and raster/vector tiles | OGC API - Tiles / compatible tile interfaces | Renderer can consume multiple services behind stable adapters |
| Massive 3D buildings, photogrammetry and point clouds | OGC 3D Tiles 1.1-compatible adapter where suitable | Hierarchical streaming for large 3D scenes |
| Static/offline tile packages | Evaluate PMTiles and other appropriate read-only packages | A single range-readable archive can simplify low-cost hosting and offline-region delivery |

Do not confuse the source format with the runtime representation. A scientific COG may be an input to processing, while the renderer receives a tiled, color-mapped product. Record the processing activity and retain lineage to source artifacts wherever available.

References:
- STAC: https://stacspec.org/
- OGC Cloud Optimized GeoTIFF: https://www.ogc.org/standards/ogc-cloud-optimized-geotiff/
- OGC API - Tiles: https://www.ogc.org/standards/ogcapi-tiles/
- OGC 3D Tiles: https://www.ogc.org/standards/3DTiles/
- PMTiles concepts: https://docs.protomaps.com/pmtiles/

### 3. Keep the renderer behind a thin adapter

Retain the existing Rust/WebGL2 globe while defining explicit source interfaces for:

- imagery/raster layers;
- elevation/terrain;
- vector features;
- 3D tiles and point clouds;
- temporal asset selection;
- source attribution and data-coverage/quality overlays.

The renderer should receive a stable view/projection contract. It must not infer empirical truth, source authenticity, operational availability, or qualification from what is visible on screen.

Run a bounded CesiumJS proof-of-concept alongside the current renderer before deciding whether to reuse it for some or all globe rendering. CesiumJS is Apache-2.0-licensed and supports imagery, terrain and 3D Tiles from a mix of providers and open services; Cesium ion is an optional hosted ecosystem, not a requirement to use the open-source runtime. The comparison should include integration cost with Leptos/WASM, mobile support, bundle size, loading behavior, accessibility of styling/interaction, offline behavior, and source portability. Do not make an irreversible renderer switch based on a feature checklist alone.

Reference: https://github.com/CesiumGS/cesium

#### Current renderer constraint (inspected 2026-10-10)

The current WebGL2 path renders a single Earth sphere using a full-globe Blue Marble texture, with separate global topology, cloud and night-light textures. The texture loader swaps one image into a WebGL texture; its current image callbacks are retained for the application lifetime. This is appropriate for a small, bounded set of planet textures, but it is not a tile scheduler and should not be used unchanged for thousands of dynamic tile requests.

The next custom-renderer experiment is tracked in [issue #70](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/70): preserve Blue Marble as the low-detail fallback and overlay globe-conforming imagery patches selected from camera visibility and zoom. Web Mercator XYZ tiles must not be stretched as though they were equirectangular textures. The tile path needs bounded concurrency/cache/texture memory, source/attribution records, correct parent-tile fallback, stale-request cancellation and context-loss recovery. Keep this as an independently testable overlay; do not replace the base globe before the pilot proves stable.

### 4. Treat freshness and uncertainty as first-class

Google Earth's own documentation describes imagery from multiple providers and acquisition periods; imagery is not necessarily current or real-time, and mosaics can have date ranges. Sol Atlas should make those limitations legible instead of disguising them.

Keep at least these times separate:

- observation/acquisition time or interval;
- source publication time;
- catalog discovery time;
- download/snapshot time;
- transformation/analysis time;
- scenario or simulation time;
- assessment/qualification time, if supplied by an external authority.

Do not label an image "live" solely because it loaded successfully. Do not treat the newest observation as the most accurate observation for every analytical task. If dates are unavailable, show that they are unavailable. If a rendered tile combines observations from multiple dates, preserve the provider's range or describe the mosaic interval rather than inventing a single timestamp.

Reference: https://support.google.com/earth/answer/6327779?hl=en

### 5. Model validation as separate evidence dimensions

Avoid a single boolean such as `verified`. An asset can have valid JSON metadata while its source is unreachable; it can be reachable while its bytes are not captured; a file can pass a format parser while its georeferencing is wrong.

Proposed non-equivalent checks include:

- metadata schema validation;
- licence/access policy review;
- source endpoint reachability at a recorded time;
- byte capture and content-digest verification;
- media/container-format validation;
- CRS/georeferencing validation;
- geographic coverage and resolution checks;
- rendering of a frozen fixture;
- independent scientific review, when a domain requires one.

These checks report only their defined scope. A successfully rendered asset is not therefore factually accurate; a digest is not a publisher signature; a syntactically valid STAC Item is not proof of data quality.

Use the repository's source-attributed provenance graph and versioned validation-report patterns. Preserve conflicts and unknown states. Do not invent missing artifact references or promote a `Scenario` to `Observed` or `Qualified` during aggregation.

### 6. Build a source and licence policy into ingestion

Before a provider is allowed into a production catalog, record its access and redistribution terms, required attribution, rate limits, cache policy, and any restrictions on derived data. Apply those rules at the adapter and packaging boundary, not only in UI copy.

OpenStreetMap data is licensed under ODbL and requires attribution; produced databases and data distributions can trigger share-alike obligations. The public OSM tile servers also have a separate usage policy. Do not treat an open dataset as permission to bulk-download, rehost every tile, or ignore a provider's rate limits.

Satellite and aerial imagery must follow each provider's own terms. A visually accessible image is not automatically reusable imagery. Prefer public/open data for the initial baseline and introduce commercial imagery adapters only with a recorded licence/contract that permits the intended use.

References:
- https://www.openstreetmap.org/copyright
- https://operations.osmfoundation.org/policies/tiles/

### 7. Harden acquisition as an untrusted-input boundary

Remote catalogs and their assets are external inputs. Ingestion should use allowlisted protocols and provider policies; bound redirects, response sizes, processing time and archive expansion; validate media types and actual formats; reject malformed coordinates/CRS metadata; and isolate parsers for complex formats. Do not turn arbitrary user-provided URLs into a privileged server-side fetch capability. Record failures as failures, not empty data that looks like a valid zero.

Never publish credentials, signed URLs, private source paths, or sensitive observation metadata in public logs or attribution fields.

## First vertical slice

Build one reproducible pilot through the entire data path before attempting global completeness:

1. Choose one mixed urban and rural/environmental pilot area based on source availability and permissions.
2. Register at least one lawful satellite/raster source, one terrain/elevation source, and one vector geography source.
3. Preserve provider attribution, acquisition-time semantics, licence, footprint, resolution metadata, and source artifact lineage.
4. Render a zoom path from regional view to the finest licensed detail actually available.
5. Show an asset/coverage inspector that explains provider, observation interval, resolution, last catalog check, licence, validation dimensions and known gaps.
6. Add an explicit no-data/unknown visualization so missing imagery cannot masquerade as low-resolution coverage or verified absence of a real-world feature.
7. Freeze inputs and expected catalog/projection outputs for deterministic tests.

## Initial source shortlist (researched 2026-10-10)

The sources below are candidates, not a declaration that ingestion or redistribution has been approved. Review the precise product, collection, source attribution and intended use before enabling each one. A provider’s portal terms may differ from the licence attached to the underlying dataset.

| Need | Candidate | Useful coverage/detail | Main limitation / gate |
| --- | --- | --- | --- |
| World-scale geography | Natural Earth | Global cultural, physical and raster layers at 1:10m, 1:50m and 1:110m scales; public-domain use | Regional reference geography, not streets, buildings or aerial imagery |
| Optical Earth-observation baseline | Copernicus Data Space STAC + Sentinel-2 | The current CDSE catalogue lists L1C for world coverage from 2015 onward and L3 quarterly mosaics worldwide; the original ESA L2A listing is Europe-only, while another L2A processing listing has restricted date/coverage | Do not assume a uniform global L2A archive. Choose the exact collection/product by region, date and processing level; preserve capture interval, cloud/quality metadata, band-specific resolution and legal notice. Sentinel-2 supports land-change analysis, not building-level imagery |
| Public tile-pipeline test layer | NASA GIBS WMTS/TMS (e.g., MODIS corrected reflectance or Blue Marble Next Generation) | Public standards-based WMTS/WMS and generic TMS/XYZ access with explicit projection, tile matrix and optional time dimension | Good for proving tile scheduling, attribution and date selection globally, but many products are coarse browse visualizations rather than close-up aerial imagery. Inspect each layer's native resolution, date availability and projection. NASA requests a GIBS acknowledgement; retain source identity and required credit, and do not infer permission for offline redistribution from service accessibility |
| Global terrain baseline | Copernicus DEM GLO-30, subject to access review | Candidate 30 m global elevation product | As of the CDSE announcement dated 17 July 2026, the COP-DEM-GLO-30 View Service became restricted to authorized user categories from 28 July 2026; requests without the required access may default to the 90 m DEM. Verify the exact distribution path and user eligibility before choosing it as the production baseline. Preserve licence, vertical datum and processing notes |
| Global building footprints | Overture Maps buildings theme | Global-scale building-footprint candidate, with cloud-hosted GeoParquet releases and stable GERS IDs | Theme is ODbL because it incorporates OpenStreetMap. Overture documents lower footprint precision for ML-derived sources, most pronounced in the Global South. Treat it as a coverage layer, not universally accurate building geometry; prioritize validation/corroboration with local and authoritative sources |
| Places and points of interest | Overture Maps places theme | Tens of millions of global points of interest; September 2026 source table lists roughly 81 million records | Separate licence boundary from buildings: places contains no OSM data and uses CDLA Permissive 2.0 or Apache 2.0 depending on source. Preserve release ID and theme-specific source attribution; joining it with OSM-derived databases requires a separate licence analysis |
| Opportunistic high-detail aerial imagery | OpenAerialMap | Contributor-uploaded imagery that may provide better local detail where available | Coverage is uneven. The Open Imagery Network terms describe CC BY 4.0 licensing for imagery contributed under those terms; check each asset’s metadata, attribution and service-use conditions |
| High-resolution U.S. terrain | USGS 3DEP / The National Map | U.S. elevation products from approximately 10 m seamless DEMs to 1 m products where available; free products and services, public-domain federal data | U.S.-specific. One-meter seamless coverage is being built out, so coverage needs a product-level check rather than a global guarantee |
| Interactive open map reference | OpenStreetMap data, through a suitable provider or a self-hosted pipeline | Rich community-maintained roads, paths, places and other features | Do not treat the public OSM raster/vector tile servers as bulk data endpoints. Their policies prohibit bulk prefetching and offline packages; data licensing and tile-service policies are separate issues |

### Recommended first stack

1. Use Natural Earth for low-zoom world context, with no implication that small-scale features are navigable at street level.
2. Evaluate Copernicus DEM GLO-30 only after confirming that the intended distribution path and Sol Atlas users are eligible under current CDSE access conditions; keep a second terrain source in the fallback plan.
3. Use CDSE STAC to choose the right Sentinel-2 collection by region and date: L1C has worldwide coverage in the current catalogue; the available L2A coverage is not uniformly global. Add quarterly mosaics only where their temporal aggregation suits the use case.
4. Ingest Overture buildings and places as separate themes with separate licence/source policies. Measure missingness and geometry quality by region; add local/cadastral or national mapping sources where their terms permit, especially in underrepresented regions.
5. Treat OpenAerialMap and national/local aerial datasets as optional higher-detail overlays where coverage and per-asset usage rights are explicitly documented.
6. Keep offline packages to data sources whose licences and service terms explicitly allow packaging. Never bulk-download OSM standard tiles to build our own archives.

This stack is intentionally heterogeneous: it can produce a consistent globe, but it will not produce Google Earth-quality close-up imagery in every location. Track actual coverage, capture date, ground sampling distance, licence state and known gaps as first-class measurements. Treat coverage, completeness, positional accuracy and source eligibility as separate dimensions. Global coverage is not the same as uniform quality; Overture itself notes lower precision for some ML-derived building data in the Global South.

### Official source references

- Natural Earth downloads and public-domain terms: https://www.naturalearthdata.com/downloads/ and https://www.naturalearthdata.com/about/terms-of-use/
- NASA GIBS public WMTS/TMS access patterns, projections and time dimension: https://nasa-gibs.github.io/gibs-api-docs/access-basics/
- NASA GIBS data-use acknowledgement guidance: https://nasa-gibs.github.io/gibs-api-docs/
- Copernicus Data Space STAC API and Sentinel-2 L2A collection: https://documentation.dataspace.copernicus.eu/APIs/STAC.html
- Copernicus Sentinel data terms: https://dataspace.copernicus.eu/terms-and-conditions
- ESA Sentinel-2 spatial resolutions and revisit summary: https://www.esa.int/Applications/Observing_the_Earth/Copernicus/Sentinel-2/Facts_and_figures
- Copernicus DEM GLO-30 product licence: https://documentation.dataspace.copernicus.eu/APIs/SentinelHub/Data/DEM/resources/license/License-COPDEM-30.pdf
- CDSE Copernicus DEM 30m View Service restriction announcement (17 July 2026): https://dataspace.copernicus.eu/news/2026-7-17-copernicus-dem-30m-view-service-license-acceptance
- Overture buildings guide, sources, licensing and documented Global South quality caveat: https://docs.overturemaps.org/guides/buildings/
- Overture places guide and source/theme licensing: https://docs.overturemaps.org/guides/places/
- Overture attribution/licensing by theme: https://docs.overturemaps.org/attribution/
- STAC Item core specification (bbox, datetime and assets): https://github.com/radiantearth/stac-spec/blob/master/item-spec/item-spec.md
- STAC Projection Extension (modern proj:code, deprecated proj:epsg, WKT2/PROJJSON): https://github.com/stac-extensions/projection
- STAC Raster Extension (per-band raster:spatial_resolution): https://github.com/stac-extensions/raster
- OpenAerialMap terms: https://openaerialmap.org/legal/
- USGS 3DEP products and resolution summary: https://www.usgs.gov/3d-elevation-program/about-3dep-products-services
- OSM tile usage policy, including offline/bulk restrictions: https://operations.osmfoundation.org/policies/tiles/

### Acceptance criteria for that slice

- Identical frozen inputs produce identical normalized catalog and semantic projection outputs.
- Asset ID and snapshot identity are distinct from a locator URL.
- Unknown acquisition dates, licences, CRS or resolution never receive fabricated defaults.
- Conflicting provider assertions and temporal intervals are preserved, not silently collapsed.
- Licence and attribution data survives through to the UI and offline exports.
- Missing, unreachable, invalid, and merely not-yet-evaluated assets have distinct states.
- H3 aggregation remains reversible to the exact represented asset/feature IDs and does not upgrade evidence status.
- No tile/source integration introduces a mandatory vendor-specific backend.
- Tests cover antimeridian footprints, overlapping sources, date ranges, duplicate locators with distinct snapshots, malformed metadata and deterministic ordering.
- Browser/WASM rendering, memory, first-useful-imagery latency, network bytes and level-of-detail transitions are measured on a fixed fixture. Thresholds are set from measured baseline and target devices rather than invented in advance.
- All CI conclusions are reported for the exact tested commit; queued, cancelled, skipped, or missing checks do not count as passes.

## Delivery sequence

1. **Contract and catalog model:** the versioned asset/catalog model and an offline STAC Item subset importer are now authored in [PR #67](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/67); exact-head CI has not yet produced a completed validation result.
2. **Catalog discovery and frozen sample:** add a real, source-licensed STAC Item sample, compare the imported metadata against the source record, and produce a reproducible catalog snapshot. The current importer does not query catalogs over the network.
3. **Renderer experiment:** compare the existing renderer and CesiumJS on the same frozen pilot data, with a written measurement protocol.
4. **Tile overlay:** implement bounded XYZ/TMS selection, cache/texture lifecycle and camera-driven image tiles over the current fallback, as tracked by [issue #70](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/70).
5. **Terrain + imagery vertical slice:** connect the chosen data adapters and show real provider metadata, licences, temporal coverage and gaps in the UI.
6. **3D and temporal expansion:** add 3D Tiles, historical imagery and scientific layers only after their ingestion/provenance path is tested.
7. **Global coverage:** measure actual geographic coverage, freshness, resolution, source-policy eligibility and data-quality gaps, then prioritize the highest-value open sources and partnerships.

## Open decisions

- Which pilot datasets offer the best combination of geographic coverage, detail, clear licence, and reliable access?
- Does CesiumJS materially reduce time-to-quality without compromising Sol Atlas's UX and offline goals?
- Which validation dimensions belong in the core model versus provider/analysis adapters?
- How will offline packages preserve attribution, licence metadata, time coverage and source lineage?
- What coverage/freshness metrics should appear globally, and which require domain-specific interpretation?

## Research references

- STAC specification: https://stacspec.org/
- OGC COG: https://www.ogc.org/standards/ogc-cloud-optimized-geotiff/
- OGC API - Tiles: https://www.ogc.org/standards/ogcapi-tiles/
- OGC 3D Tiles: https://www.ogc.org/standards/3DTiles/
- CesiumJS repository and licence: https://github.com/CesiumGS/cesium
- PMTiles: https://docs.protomaps.com/pmtiles/
- OpenStreetMap licensing: https://www.openstreetmap.org/copyright
- OpenStreetMap tile usage policy: https://operations.osmfoundation.org/policies/tiles/
- Google Earth imagery dates and collection notes: https://support.google.com/earth/answer/6327779?hl=en

## Non-claims

This plan does not establish global imagery parity with Google Earth, complete geospatial coverage, current conditions, accuracy of third-party source data, legal permission for any source not individually reviewed, or success of the renderer experiment. These must be demonstrated by the implemented pipeline and its evidence.
