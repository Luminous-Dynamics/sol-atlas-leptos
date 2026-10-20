# Landscape Design Workbench v0

**Status:** initial interactive prototype; schematic design intent only.  
**Primary surface:** Sol-Atlas (/design/landscape).  
**Design principle:** from drawing objects to composing a living, evidence-aware landscape.

## Why this product

Current landscape-design tools already cover powerful terrain modeling, geolocation, 2D/3D authoring, planting schedules, CAD/BIM exchange, and presentation. This initiative should not claim to replace those capabilities in its first iteration. The opportunity is to connect visual design to the conditions a place depends on: water, catchments, heat and shade, soil/ecology, movement, services, biodiversity, maintenance, community ownership, and observed outcomes—while preserving the difference between a beautiful proposal and evidence that it will work.

Competitive references to study against published capabilities:
- SketchUp landscape architecture: terrain from contours, draping, geolocation, shadow study, interoperability — https://www.sketchup.com/en/industries/landscape-architecture
- Vectorworks Landmark: integrated landscape plans, terrain, planting schedules, take-offs and 2D/3D workflows — https://www.vectorworks.net/en-US/start/landscape-design
- QGIS: reference GIS capability baseline — https://qgis.org/
- OGC API - Tiles: standardized tiled geospatial retrieval — https://ogcapi.ogc.org/tiles/
- RFC 7946 GeoJSON: geographic feature interchange in WGS84 longitude/latitude — https://www.rfc-editor.org/rfc/rfc7946
- OGC JSON-FG, released 2026-05-21, extends the GeoJSON model for richer feature/geometries use cases while maintaining GeoJSON compatibility — https://www.ogc.org/standards/json-fg/

These are capability references, not a benchmark result or a claim of parity.

## v0 interaction

Route: **/design/landscape**.

- Canvas: responsive 1000 × 650 local drawing-unit plane, illustrative boundary and contours.
- Tools: canopy tree, planting bed, water feature, path, built element.
- Interaction: select tool; click canvas to place; pointer or keyboard-select an existing symbol; inspect its anchor; delete the selected element; undo/redo the last add/delete command; reset to deterministic starter plan.
- Scope context: site/parcel, district, watershed, bioregion. In v0 this labels the intended planning context; it does not rescale or georeference the geometry.
- Layer controls: measurement grid and illustrative contours.
- Project metadata: editable project name in the UI.
- Export: visible, downloadable, parseable native JSON draft snapshot with schema version, scope, stable feature IDs, local drawing-unit coordinates, source/status tags, and an explicit georeferenced=false declaration.
- Local-first: import reads a locally selected native JSON file; export downloads a native snapshot. Neither operation uploads project data.
- Import is limited to the authored schematic v0 contract, validates the entire document before mutating editor state, rejects synthetic benchmark scenes and preserves the current project on any error. Files over 64 MiB are rejected.

### Symbol geometry contract

The rendered marks are symbols. A feature snapshot currently stores a symbol anchor, not full plant polygons, path centerlines, hydraulic geometry, or surveyed footprints; the snapshot contains only the feature anchor and does not preserve symbol footprint or stroke extents. Do not use icon extents for quantities, simulation, or construction documents.

### Coordinate contract

LOCAL_CARTESIAN_METRES is a project-local drawing frame, with canvas top-left as origin, X rightward, and Y downward to match the screen. These are arbitrary drawing units, not metres, a map CRS, surveyed boundary, or geographic claim. Never emit these coordinates as RFC 7946 longitude/latitude. A later GIS import/export path must preserve source CRS metadata, apply a real coordinate transform, and fail closed when transformation or datum-grid requirements are unsatisfied.

### Native snapshot example

    {
      "schema": "luminous.landscape-design.v0",
      "schema_uri": "https://luminousdynamics.io/schemas/landscape-design-v0.schema.json",
      "project_name": "Untitled landscape",
      "scope": "site-parcel",
      "coordinate_frame": {
        "kind": "LOCAL_CARTESIAN_DRAWING_UNITS",
        "origin": "canvas top-left",
        "x_axis": "right",
        "y_axis": "down",
        "georeferenced": false
      },
      "geometry_status": "schematic-design-intent",
      "features": [
        {
          "id": "feature-0001",
          "kind": "tree",
          "x_du": 180,
          "y_du": 160,
          "status": "design-intent",
          "source": "user-authored-schematic",
          "geometry_representation": "schematic-symbol-anchor-v0"
        }
      ]
    }

### Strict Rust validation and edit history

The validator distinguishes `schematic-design-intent` from `synthetic-benchmark` documents and requires feature `status`/`source` tags to match that document-level declaration. This prevents benchmark-generated scenes from masquerading as user-authored plans.

- `src/landscape.rs` is a reusable, pure-Rust validation boundary for schema identity, coordinate-frame claims, scope, project name, allowed fields, unique IDs, feature kind/label coherence, anchor bounds, and design-intent tags.
- The editor runs the validator before exposing a downloadable snapshot; failure clears the export payload and displays the validation errors.
- Native tests cover malformed roots, unsupported scopes, duplicate IDs, out-of-bounds anchors, mismatched type/label, unknown fields, georeferencing claims and parity between Rust constants and the committed JSON Schema.
- Editor history is command-aware: undoing an add removes only that addition; undoing a delete restores the removed feature at its original position; redo reapplies the exact command and refuses conflicting feature identities. New edits clear the redo branch. Symbol selection is keyboard accessible; Enter/Space select a focused symbol rather than creating an accidental new feature.

### Machine-readable contract and benchmark manifest

- JSON Schema: [schemas/landscape-design-v0.schema.json](../schemas/landscape-design-v0.schema.json). Every snapshot declares the schema URI as well as its schema version.
- Canonical starter fixture: [fixtures/landscape/seed-landscape-v0.json](../fixtures/landscape/seed-landscape-v0.json), checked against the exporter by a native unit test.
- The Rust generator `synthetic_benchmark_snapshot(feature_count, seed)` is implemented in `src/landscape.rs`, with determinism and size-bound tests. Its output is validated by the same snapshot validator as authored plans.
- Frozen deterministic workload protocol: [benchmarks/landscape/workloads-v0.json](../benchmarks/landscape/workloads-v0.json). This defines feature counts, a seeded generator, action traces, raw measurements and negative controls. It contains no measured performance claims.
- Interoperability profile: [docs/LANDSCAPE_INTEROPERABILITY_PROFILE_V0.md](LANDSCAPE_INTEROPERABILITY_PROFILE_V0.md), which maps data families to OGC/STAC standards and sets source-ingestion gates.
- Initial performance metric registry: [metrics/landscape/registry-v0.json](../metrics/landscape/registry-v0.json), with schema [landscape-performance-registry-v0.schema.json](../schemas/landscape-performance-registry-v0.schema.json). This is a metric-definition contract only, not populated field data.
- Benchmark implementation work: [LAND-BENCH-001](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/195).
- Outcome/performance metric registry work: [LAND-METRICS-001](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/197).

## South African pilot source catalog

The first source catalog is [south-africa-johannesburg-v0.json](../data-sources/landscape/south-africa-johannesburg-v0.json), with a structural schema at [landscape-source-catalog-v0.schema.json](../schemas/landscape-source-catalog-v0.schema.json). A native test checks its schema link, unique source IDs, explicit license states, validation gates, and the invariant that the registry is metadata-only.

Candidates include the Gauteng C-Plan v4 and the 2023 national biodiversity-priority service from SANBI BGIS; CHIRPS v3 daily rainfall; Copernicus DEM GLO-30; OpenStreetMap; and Sentinel-2 L2A imagery. These are not interchangeable sources and they have different spatial scales, update patterns, and legal/access requirements.

- **SANBI biodiversity layers:** useful for screening conservation priorities, but not a substitute for a site ecological survey or an automatic legal decision. Verify redistribution terms for each service/layer.
- **CHIRPS v3 rainfall:** regional/catchment time-series context. The Earth Engine catalog showed coverage through 31 July 2026 at review time; pin exact collection and observation window on ingestion.
- **Copernicus DEM:** GLO-30 is a digital surface model, not automatically bare-earth terrain. The service access changed in August 2026; explicitly request the intended product and verify actual resolution, CRS, vertical datum, quality masks and access rights.
- **OpenStreetMap:** preserve extract timestamp and ODbL attribution/share-alike obligations. Missing mapped data is not evidence that a feature does not exist.
- **Sentinel-2:** use dated scene IDs, cloud/shadow masks and native 10/20/60 m band-resolution metadata. Imagery classification remains an observation/inference, not a field species inventory.

The catalog records unresolved licensing and ingestion state deliberately. No layer should enter a simulation merely because the editor can render it. The dedicated ingestion contract is [LAND-DATA-001 #206](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/206).

### Evidence readiness is a first-class editor state

The sidebar reads the versioned Johannesburg/Highveld source catalog and counts catalogued records separately from records marked ingested. In v0 the catalog is metadata-only: a candidate URL is not a downloaded or license-cleared dataset. When no records are ingested, the UI must state that analysis is blocked rather than showing favorable defaults.

Before any source can support a model, qualify license/attribution, source version and observation date, CRS/axis order/units/vertical datum, spatial/temporal resolution, no-data and quality masks, and an independent reference. Visualizing a candidate layer is not the same as qualifying it for simulation.

### Browser-local persistence in v0

The editor now offers explicit **Save in browser** and **Load browser draft** actions. These use one versioned same-origin browser-local key; they do not sync to a server, and no autosave runs in the background. A loaded snapshot is fully parsed and validated before it replaces the current project.

The v0 browser-storage path is deliberately capped at 2 MiB because localStorage quotas vary and it is not a suitable storage engine for very large spatial projects. Larger projects should use portable JSON export/import in this version. Browser-local storage is not encrypted, can be cleared by the browser, and should not hold sensitive land or participant details on shared devices.

Follow-up: move durable project persistence to IndexedDB with transactional writes, versioned project IDs, chunking/large-file support, crash recovery, migration receipts, and quota/eviction handling. Do not simply raise the localStorage size limit and claim scale-out.

## Rust-first architecture and repository boundaries

### Sol-Atlas — map and editor system of record

Own the Leptos UI, drawing tools, selection and history, layer visibility, project schema, coordinate-reference frames, spatial search, external dataset catalogues, tiling, import/export adapters, attribution, data currency, and map/render benchmarks. Use the existing application and route rather than starting a competing frontend or GIS abstraction.

Possible later core module boundaries:
- landscape::project: versioned native document and command history.
- landscape::geometry: project-local geometry, units, validity and extents.
- landscape::exchange: GeoJSON/JSON-FG/GeoPackage/CSV adapters where justified.
- landscape::layers: source attribution, scale/resolution, confidence and freshness.
- landscape::render: visual-only styling and decluttering, never a source of domain truth.

Adopt Georust geo-types/operations when appropriate; use PROJ through the Rust proj bindings for actual CRS transformations where platform support permits. Make native/browser build implications explicit. Avoid adding a second general-purpose geometry kernel without a demonstrated gap.

### Symtropy — physical and ecological scenario engine

Own deterministic, seeded simulation and scenario contracts for surface-water pathways, infiltration and drainage, terrain modification/cut-fill, shade/microclimate, habitat connectivity, route/accessibility, disturbance, maintenance, and time-evolving growth. Each model reports assumptions, boundary conditions, validation status, ranges/uncertainty and out-of-domain conditions. Never derive flow, safety, yield or ecological feasibility from canvas appearance.

### Symthaea — read-only reasoning and design critique

Consume versioned project snapshots and scenario/evidence outputs. Generate alternatives and explain trade-offs (heat reduction versus water demand, habitat connectivity versus access, embodied materials versus lifetime maintenance, public access versus sensitive habitat). Require cited inputs, uncertainty and falsification questions. Recommendations must be replayable and never silently mutate the authoritative design or actuate physical systems.

### Mycelix — provenance, participation and stewardship

Own project identity and version history, who contributed or approved which change, evidence attestations, landholder/community consent, governance rules, maintenance commitments, monitoring observations, benefit distribution, and change disputes. Separate contributor identity, legal ownership, consent, and technical authorship. Default private projects to private; publish only explicit user-authorized artifacts.

### Shared contracts, not duplicated subsystems

- A design snapshot is a versioned artifact with source links/digests and project-frame metadata.
- Simulation runs consume a pinned snapshot digest and return a typed result artifact; they do not write back over the authored plan.
- Symthaea's recommendation references exact snapshot and result digests.
- Mycelix attests decisions/observations without becoming a second geometry database.
- Sol-Atlas renders facts, scenarios and proposals with distinct visual encodings; no unsupported single sustainability score.

## Scale strategy: one document model, multiple spatial resolutions

| Scale | Main design objects | Required context before decision-quality claims |
|---|---|---|
| Site / parcel | Buildings, canopy, beds, paths, water, soils, access, utilities | Survey/CRS, utilities, slope, water availability/quality, shade, soil, legal boundaries |
| District / neighbourhood | Streets, greenways, public space, tree canopy, heat refuge, urban drainage | Network continuity, heat exposure, service access, ownership/maintenance, stormwater capacity |
| Watershed / catchment | Subcatchments, streams/wetlands, infiltration, floodplain, retention, upstream interventions | DEM quality, drainage networks, rainfall IDF/time series, soils, calibrated hydrology and downstream impacts |
| Bioregion / landscape | Habitat corridors, restoration mosaics, agricultural/urban/forest systems, fire and drought regimes | Ecological classification, connectivity and fragmentation, seasonal climate, land tenure/rights, disturbance and monitoring history |

Do not simply zoom the parcel tool out and call it regional planning. Use scale-specific primitives, aggregation and source-resolution disclosures. A coarse dataset cannot support a precise parcel-level conclusion.

## Executable native benchmark harness

The frozen workload manifest now has a portable Rust-only native harness. Run:

    cargo test --features landscape-bench --bin landscape-bench
    cargo run --release --features landscape-bench --bin landscape-bench -- --out reports/landscape/native-kernel-v0.json

The harness derives default counts and seed from the manifest, creates deterministic synthetic scenes, validates them with the production v0 document validator, serializes normalized JSON, calculates a SHA-256 snapshot digest, and records raw generation/validation/serialization/digest timings plus p50/p95 summaries. The report includes available commit, toolchain, OS/architecture, CPU and memory metadata.

This is a **native document-kernel benchmark only**. It does not measure browser frame rate, pointer interaction, user task success, geospatial operations or real landscape outcomes. Those require separate browser/reference-machine, interoperability and field-evaluation studies. Report structure is versioned by [landscape-benchmark-report-v0.schema.json](../schemas/landscape-benchmark-report-v0.schema.json); there are no performance results until the executable has been run on a declared reference machine and raw report committed. CI runs only a small smoke workload, not the 10k/100k qualification workload.

## Benchmark methodology

### Reproducible protocol

For every benchmark run, persist: commit SHA, build profile, Rust/toolchain versions, browser/GPU/OS, hardware, scene manifest and digest, fixed seed, input data version/digest, viewport/device scale factor, cache state, command sequence, run count, and raw per-run observations. Separate cold-start and warmed steady-state; report p50/p95 and variation, not only best time. Freeze scene manifests and never compare unmatched styling/data.

### Editor workload suite

- 100 / 1,000 / 10,000 / 100,000 vector features.
- Place, select, move, undo, toggle layers, pan, zoom, export and deterministic reload.
- Mixed geometry: points, paths, polygons, overlapping labels and clustered tree symbols.
- Valid and adversarial fixtures: empty project, duplicate IDs, invalid ring, missing CRS, stale source, non-finite input, huge extents, dense overlap, missing tile, offline reload.
- Data integrity: round-trip coordinates, feature IDs/types/properties, CRS, units, attribution and provenance.
- Accessibility: keyboard-only controls, visible focus, contrast, reduced motion, zoom/text scaling and assistive-technology labels.

### Metrics

- Interaction latency p50/p95; frame pacing and long frames; initial render and reload time; export time/size; memory high-water mark; spatial query latency; time to first useful view; errors and dropped/missing geometry; import/export round-trip correctness; accessibility task completion; user time to complete matched design tasks; number of unsupported claims exposed by adversarial review.

### Initial targets (hypotheses to measure, not current results)

- At 10,000 simple vector features on a pinned reference machine, local edit action p95 < 50 ms and p95 frame time <= 33 ms during steady-state pan/zoom.
- Feature IDs, types, coordinates and CRS metadata survive native round-trip exactly.
- 100% of unsupported/missing CRS scenarios fail closed with a visible error; no silent coordinate reinterpretation.
- No user action requires a network connection for the local editing/export path.
- No loss of authored features under undo/redo/reset/property edit tests.
- 100,000 features is a stress case until measurement supports a product claim.

Targets become product gates only after the reference hardware and benchmark harness are committed. Against QGIS/SketchUp/Vectorworks, first compare task workflows, interoperability and integrity—not claims of general speed or feature superiority.

## Roadmap

1. **v0 — schematic editor:** implemented in the first branch; interaction, scope controls, visible layers and native JSON snapshot.
2. **v0.2 — project document:** explicit properties, stable IDs/UUIDs, command history, undo/redo, native save/load, validation and golden fixtures.
3. **v0.3 — geospatial context:** real CRS-aware basemap/terrain import and source provenance, beginning with one documented South African pilot area; no fake coordinates.
4. **v0.4 — decision layers:** water, shade, canopy, accessibility, land rights and habitat constraints using existing Sol-Atlas sources.
5. **v0.5 — simulation handoff:** immutable project digest and Symtropy scenario manifest; compare modelled output with independent reference cases.
6. **v0.6 — design critique:** Symthaea generates evidence-linked alternatives with stated trade-offs, sensitivity and uncertainty.
7. **v0.7 — collaborative stewardship:** Mycelix decision history, consent, observation and maintenance commitments.
8. **v1 — delivery package:** dimensioned drawings, planting/surface schedules, quantity takeoffs, accessible/public-review exports, reproducible scenarios and verification receipts.

## First-slice acceptance criteria

- / route and /labs/reactor-twin still resolve.
- /design/landscape renders on the existing Rust/Leptos stack.
- All five tools and four scale contexts can be selected with accessible, visible state.
- Selecting a symbol must not trigger placement; deleting one selected symbol removes only its ID and preserves unselected features.
- Evidence readiness must count metadata records separately from ingested data, and must not present un-ingested candidate sources as analysis-ready.
- Importing the canonical authored fixture restores its scope, project name, IDs, feature kinds and anchors; malformed, oversized, unsupported, synthetic, or out-of-range-ID snapshots leave the current project unchanged.
- Undo/redo must round-trip both add and delete commands, and reject an identity conflict without mutating the feature list.
- One valid canvas click adds one feature; undo removes the latest; reset restores the exact starter fixtures.
- Grid/context visibility toggles work independently of editing.
- Export is valid JSON, deterministic for the same project and explicitly not georeferenced.
 - Browser-local save/load is explicit, bounded to 2 MiB in v0, schema-validated, and preserves the current project when stored data is missing or invalid.
- Tests cover schema, round-trip, feature IDs and machine keys; the exact PR head passes formatting, native tests and target wasm build.
- No simulation, field-validated ecological result, construction compliance, terrain survey or QGIS/SketchUp parity is claimed by this prototype.


The starter fixture is canonical only for the schematic v0 symbol-anchor representation; it does not carry surveyed geometry, physical dimensions, or measured impact values.


Durable native save/load, real geometry types and CRS-safe interchange are tracked in [LAND-DOC-001](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/198). The v0 validator only qualifies the current schematic symbol-anchor format; it does not validate geospatial truth or ecological/engineering claims.
