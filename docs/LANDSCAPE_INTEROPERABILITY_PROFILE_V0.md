# Landscape Interoperability Profile v0

**Status:** architectural profile, not an implemented compliance claim.  
**Pilot:** Johannesburg / Highveld, Gauteng, South Africa.  
**Rule:** preserve a source's native meaning and provenance; do not make every data family look like the same layer.

## Paradigm shift

The landscape application should not become a bespoke GIS monolith. It should be a standards-aware **composition and evidence layer**:

1. Discover datasets without presuming permission or suitability.
2. Pin source identity/version and verify licensing, CRS, units, time, resolution, quality masks and attribution.
3. Transform only through an explicit, tested coordinate operation.
4. Keep authored design, observations, curated datasets, synthetic data and model results distinguishable.
5. Give Symtropy only model-qualified inputs and give Symthaea only versioned outputs with provenance and uncertainty.
6. Record human approval, access policy and stewardship in Mycelix without transferring scientific truth or geometry ownership to governance records.

## Standards by purpose

| Purpose | Preferred standard / format | Boundary |
|---|---|---|
| Simple geographic feature interchange | [RFC 7946 GeoJSON](https://www.rfc-editor.org/rfc/rfc7946) | RFC 7946 coordinates are WGS 84 longitude/latitude. Never encode LOCAL_CARTESIAN_DRAWING_UNITS as GeoJSON. Test axis order, ring validity and real coordinate transformation. |
| Richer feature/geometry exchange | [OGC JSON-FG](https://www.ogc.org/announcement/ogc-releases-the-features-and-geometries-json-json-fg-standard/) | Released 2026-05-21. Evaluate typed features, richer geometry and coordinate reference metadata for project/interchange use; pin the conformance profile and test actual partner support. Do not label an arbitrary native JSON snapshot as JSON-FG. |
| Feature discovery/query | [OGC API - Features](https://www.ogc.org/standards/ogcapi-features/) | Part 1 Core uses WGS 84 longitude/latitude; Part 2 addresses other CRSs by reference. Inspect collection metadata and CRS capabilities before query and conversion. |
| Environmental spatiotemporal sampling | [OGC API - Environmental Data Retrieval (EDR)](https://www.ogc.org/standards/ogcapi-edr/) | EDR 1.2.0 was approved 2026-04-30 and published 2026-09-08. Position/area/radius/cube/trajectory/corridor query patterns help retrieve subsets; they do not guarantee scientific fitness or access rights. |
| Satellite imagery, DEM, lidar and derived spatial assets | [STAC](https://www.ogc.org/standards/stac/) | STAC 1.1.0 standardizes spatiotemporal asset metadata and search. Pin item/collection, asset href, acquisition time, processing baseline, checksum, license and relevant extensions; asset discovery is not asset validation. |
| Environmental observations and sensor metadata | [OGC SensorThings API](https://www.ogc.org/standards/sensorthings/) | Use for observations/Things/Datastreams and their metadata when supported. It does not replace Mycelix's participant consent, project approval, or observation provenance requirements. |
| Offline portable geospatial package | [OGC GeoPackage 1.4](https://www.ogc.org/standards/geopackage/) | Useful for SQLite-backed vector, raster/tile and related spatial tables. Validate its contents and metadata; do not claim full project/history round-trip unless that profile is implemented and tested. |
| Large view/render workloads | [OGC API - Tiles](https://www.ogc.org/standards/ogcapi-tiles/) | Tiles are delivery/rendering units, not the authoritative editable geometry or source of analytical truth. Publish extent, resolution, CRS/tile matrix and provenance. |
| Cross-system provenance concepts | [W3C PROV-O](https://www.w3.org/TR/prov-o/) | Reuse entity/activity/agent relationships where useful for lineage. Map into existing evidence and stewardship models; do not create a disconnected parallel provenance graph. |

## Shared object contracts

### A source candidate

The existing data-sources/landscape/south-africa-johannesburg-v0.json catalog is metadata-only. Each record must declare:
- stable source/layer ID, publisher/custodian, service and metadata URIs;
- explicit license status, attribution text, and restrictions on caching/redistribution;
- source version or observation date, retrieval time, content digest after retrieval;
- geometry/data type, CRS and axis order, measurement units, vertical datum where relevant;
- nominal resolution, spatial/temporal coverage and no-data/quality-mask semantics;
- intended uses, non-uses, confidence limitations and scale restrictions;
- validation gates, including at least one independent reference check for any claim that needs it.

A URL resolving successfully is not sufficient to ingest, display as authoritative, or feed into a model.

### An ingested dataset

A source becomes ingested only after all dataset-specific gates are satisfied. Persist an immutable manifest referencing source ID, exact URI/version, retrieved bytes/content digest, parser/transform versions, native and target CRSs, transformation/accuracy, filtering/masking choices, license review, attribution, and raw validation results. Corrections create a new manifest revision; never silently retcon the input behind an old simulation.

### A model-ready input

Each Symtropy model declares required inputs, compatible geometry/CRS, units, minimum temporal/spatial resolution, boundary conditions, acceptable quality/currentness, and independent verification requirements. The handoff pins the source manifest digests as well as the design snapshot digest. Missing/unsupported inputs produce not-computable or out-of-domain, never zeros or a green status.

### An observation

An observation must distinguish the observed phenomenon from the sensor/system and method that produced it; include event/observation time, units, location precision, uncertainty/quality flags, source/reference, and consent/access constraints. Mycelix stores approvals and community stewardship context; a community report or approval is not automatically a verified measurement.

## Explicit source-specific rules for the Johannesburg/Highveld pilot

- **SANBI/Gauteng biodiversity plans:** screening and conservation-context layers only until the exact layer license, category meaning, current plan version and authoritative-custodian conditions are verified. No automatic development approval or species-level conclusion.
- **CHIRPS v3 rainfall:** preserve collection ID, retrieval/observation interval, units, masks, and version-specific terms. Grid rainfall is regional/catchment context, not a parcel rain-gauge measurement or an unqualified local runoff calibration.
- **Copernicus DEM GLO-30:** request the intended instance explicitly; store the actual tile CRS/vertical datum and product/quality/infill metadata. A DSM is not automatically bare-earth terrain, survey or engineering grading data.
- **OpenStreetMap:** preserve the extract timestamp and ODbL attribution/share-alike obligations. Missing map objects are not evidence of physical absence; mapped lines do not prove legal access or accessibility.
- **Sentinel-2 L2A:** pin scene IDs, acquisition date, processing baseline, CRS/transform, cloud/shadow masks and provider terms. Multi-date land-cover interpretation needs independent reference data and should not claim species inventories from 10–20 m imagery.
- **All sources:** cataloged, public-facing, or accessible does not imply fit for a particular scale, permissible for republication, current, or validated.

## Acceptance tests for the first ingestion adapter

1. A successful response with unknown/unapproved license cannot be cached in a redistributable offline pack or presented as fully cleared.
2. Missing CRS, axis order, vertical datum (when relevant), units, acquisition date or required resolution blocks model handoff with a typed error.
3. A known-location sample detects axis swaps and transform mistakes; fixtures include both expected-good and expected-bad coordinates.
4. Repeated ingestion of the same pinned bytes and transform version emits a deterministic manifest digest.
5. Changed source bytes, parsing version, transform parameters or masks change the derived-input digest.
6. Unknown/masked/no-data cells stay distinct from measured zero.
7. A tiled rendering result never replaces the source dataset/geometry or model-ready input.
8. The current catalog still reports metadata-only-not-ingested until real ingestion and validation receipts exist.

## Competitive benchmark

Do not compete with QGIS on the claim "we have more GIS tools." Compare:
- **Interchange integrity:** independent parse/load/export across Sol-Atlas and QGIS using the same pinned fixtures; exact CRS, geometry, IDs/attributes where the format supports them, source lineage and license notices.
- **Time to validated context:** time from dataset discovery to a reviewer-approved, model-ready manifest, including time spent on license/CRS/resolution and negative-control rejection.
- **Failure detection:** percentage of intentionally invalid source cases caught; false acceptance is more important than raw throughput.
- **Reproducibility:** byte-identical normalized manifests for pinned inputs and transforms.
- **Scale honesty:** fraction of displayed/modelled quantities with valid declared scale, resolution and uncertainty; count unsupported precision claims as defects.
- **Performance:** cold/warm query, viewport load and tile latency with pinned browser, data bytes, hardware, cache conditions and p50/p95 raw traces.

Targets must be preregistered before measurements. A fast map render or a successful API query is not evidence of correct science.

## Implementation order

1. Finish source-catalog schema and readiness display in the Sol-Atlas v0 editor.
2. Build one read-only adapter for source metadata/discovery before downloading data.
3. Implement one low-risk ingestion fixture with explicit CRS/license gates and an independently checkable known-location case.
4. Emit immutable dataset manifests and hand them to Symtropy only when the model's requirements are satisfied.
5. Have Symthaea explain source gaps and alternative consequences against those exact manifests.
6. Have Mycelix record who consented/approved a project decision and stewarded monitoring; never infer ownership, truth or consent from location alone.
