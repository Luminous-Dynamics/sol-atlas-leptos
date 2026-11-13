# EARTH-OBSERVABILITY-BGS-SOBI-ADAPTER-001
## BGS Single Onshore Borehole Index (SOBI) OGC API Features source profile v0.1

**Parent:** EARTH-OBSERVABILITY-QUAL-001 (#761)  
**Status:** schema adapter and synthetic tests added; no production network client or live-data ingestion claimed  
**Source profile reviewed:** 2026-11-13

## 1. Why this is the first external adapter

The British Geological Survey (BGS) publishes the Single Onshore Borehole Index (SOBI) as an open, GIS point dataset covering more than one million borehole, shaft and well index records across Great Britain. The BGS dataset page lists it as open and free for commercial, research and public use under the Open Government Licence (OGL), with an acknowledgement requirement. The index references digital and analogue records; individual scans are separately accessible.

The BGS OpenGeoscience API exposes a collection named **Borehole Records** at:

- Collection: https://ogcapi.bgs.ac.uk/collections/onshoreboreholeindex?f=html
- Items: https://ogcapi.bgs.ac.uk/collections/onshoreboreholeindex/items?f=json
- Queryables / schema: linked from the collection metadata.
- Supported representations include GeoJSON, CSV, HTML and JSON-LD.

This gives Sol-Atlas a real, standards-based source shape to qualify, without prematurely claiming that an index record is a calibrated measurement or geological ground truth.

## 2. Rights, service and acquisition boundary

Keep these facts separate:

1. **Dataset licence:** BGS lists the SOBI dataset as OGL and provides an acknowledgement requirement: “Contains British Geological Survey materials © UKRI [YEAR].” Check the current collection/source terms and any linked third-party material before each use.
2. **API service policy:** BGS describes its API services as subject to fair-use/rate-limit controls and explicitly says they are not intended for bulk downloading of an entire dataset. Do not crawl or page through all SOBI records. Use a bounded bbox, explicit feature ID or narrowly scoped request, and obey current service policy.
3. **API maturity:** the OpenGeoscience OGC API is identified by BGS as a beta service. Endpoints, attributes, availability and performance may change. Pin the API profile/collection metadata version or its digest in every ingest receipt.
4. **Related artifacts:** a scan URL or AGS log URL in an index feature is only a discovered link. It does not prove separate access, processing, derivative, retention or redistribution rights for the linked artifact. Do not automatically fetch it.
5. **Ground-truth ceiling:** SOBI is an index of record locations and metadata. It is not itself a full lithology dataset, borehole trajectory survey, groundwater time series or an independent geological validation set.

BGS reports more than one million SOBI records and says thousands of new records are added each year. A 13 November 2026 sample API page returned 10 features with numberMatched 1,358,984 and a next-page link. This count is a time-specific query response, not a guarantee of completeness or a license to bulk-extract the collection.

## 3. Observed field profile and semantics

The following fields were present in the current collection schema/sample. This is a source-profile observation, not a promise that every record populates every field.

| Field | Observed purpose / type | Adapter handling |
|---|---|---|
| GeoJSON type / features | FeatureCollection containing Feature records | Validate collection and feature types |
| feature id | Numeric feature identifier in sampled records | Preserve source ID; do not assume it is interchangeable with other IDs |
| geometry | Point with two CRS84 coordinate values | Interpret as longitude, latitude; reject invalid ranges/dimensions; no coordinate transformation in this adapter |
| properties.id / bgs_id | Numeric SOBI/BGS record identifiers in sample | Prefer bgs_id for stable key; preserve both |
| reference | Source reference text such as a map/grid identifier | Preserve as a display/search attribute, not assumed globally unique |
| name | Provider record name | Preserve; not a canonical ID |
| grid_ref | Grid-reference text | Preserve alongside coordinates |
| easting / northing | Projected coordinate values | Keep raw; do not use for reprojection until CRS and reference system are pinned |
| precision | Provider text label such as “± METRE” or “± 10 METRES” | Preserve verbatim; do not reinterpret as sigma, RMSE or independently verified accuracy |
| length | Numeric index value | Preserve raw; positive means only positive provider-reported index value; zero/negative values are unqualified codes until a source value dictionary is confirmed |
| year_known | String or null | Preserve the source value and null; do not turn null into year 0 or current year |
| date_updated | Timestamp-like string | Treat as source-record update time, not borehole drilling or observation time |
| held_at | Archive/custodian code | Preserve |
| length_scan_cat | Source category string | Preserve as an index attribute; do not infer depth semantics |
| scan_url / ags_log_url | Related artifact URIs or null | Emit separate related-artifact records with rights status UNQUALIFIED; do not dereference automatically |
| sitereport / water_well_ref / scan_quality | Optional source attributes | Preserve without converting missing values into negative findings |

### Important negative-control observation

The live API sample includes a record whose name contains “NOTIFICATION ONLY”, with length = -2, length_scan_cat = “-2_Y”, year_known = null and a precision label. That is direct evidence that non-positive length codes and missing year values occur in the source. It does **not** establish the complete meaning of the -2 code.

Therefore the adapter preserves -2 and classifies it as NonPositiveCodeUninterpreted. It never converts it to 0 metres, a measured borehole depth, or a “no borehole” finding. The source's complete field dictionary must be obtained before interpreting any coded values.

## 4. Implemented Rust adapter profile

The branch adds:

- sol-atlas-core/src/bgs_sobi.rs
- public module export sol_atlas_core::bgs_sobi
- synthetic fixture research/earth-observability/fixtures/bgs-sobi-ogc-api-synthetic-v0.1.json

The parser currently:
- deserializes a BGS-style GeoJSON FeatureCollection and retains unknown future fields;
- verifies collection/page counts and feature types;
- requires a stable feature identity from bgs_id, properties.id or feature id;
- validates Point geometry and finite longitude/latitude ranges in CRS84 order;
- preserves precision, length, years and timestamps exactly as source fields;
- distinguishes missing, positive reported index value and non-positive uninterpreted length code;
- preserves matched count and next-page URL as metadata, without assuming full inventory coverage;
- emits scan/AGS links as separate related artifacts with Unqualified rights status;
- never opens network connections, downloads scans, pages through the collection, converts coordinates, or claims a field observation.

The synthetic fixture has two non-geographic test features, one with a positive length value and one with a non-positive code and missing year. It does not contain live BGS records or real coordinates.

## 5. Qualification and adversarial tests

The Rust tests exercise:

- collection parsing, numberReturned consistency and matched-count snapshots;
- explicit GeoJSON longitude/latitude coordinate order;
- verbatim retention of the provider precision label;
- negative length code and null year preservation;
- scan link separation and unqualified rights;
- retention of unknown future API fields;
- rejection of invalid longitude and missing stable identity;
- rejection when numberReturned disagrees with the feature count.

Pending tests before production adapter admission:

- compare the pinned collection/schema/queryables digest at runtime and record drift;
- validate API response Content-Type, pagination URL host and query scope;
- enforce a configurable request budget, backoff and rate limit;
- validate each link's host and owner-approved retrieval policy;
- verify exact OGL notice/attribution and any third-party rights per artifact;
- add a bounded source request fixture from an approved test area without committing restricted or unnecessary real coordinates;
- test schema drift and version fallback against a recorded source contract;
- authenticate source responses and preserve HTTP headers, request URL, retrieval time and raw response digest.

## 6. Next integration steps

**BGS-1 — Source contract:** capture the collection metadata, queryables, JSON schema and service-policy revision as versioned source-profile evidence.  
**BGS-2 — Bounded retrieval:** add an opt-in, low-rate, bbox/ID-limited HTTP adapter with host allow-list, response-size ceiling, exact request receipt and no automatic next-page crawl.  
**BGS-3 — Related artifacts:** create separately rights-qualified artifact references for scan and AGS log resources. The SOBI index grant must not be inherited by the linked documents.  
**BGS-4 — Domain integration:** route the location index record into Sol-Atlas evidence graph as a borehole-index claim, not a lithology model input by default. A separate source document/measurement adapter must establish the actual log intervals, survey geometry, method, units and rights.  
**BGS-5 — Benchmark use:** use a permitted bounded subset for source coverage/metadata/duplicate-detection and identifier tests. For geological modelling, establish a different target dataset with compatible reference truth and held-out boreholes.

## 7. References

- BGS SOBI dataset page: https://www.bgs.ac.uk/datasets/boreholes-index/
- BGS OpenGeoscience OGC API collection: https://ogcapi.bgs.ac.uk/collections/onshoreboreholeindex?f=html
- BGS OGC API items: https://ogcapi.bgs.ac.uk/collections/onshoreboreholeindex/items?f=json
- BGS APIs and fair-use policy: https://www.bgs.ac.uk/technologies/web-services/api/
- BGS service acknowledgement and terms: https://www.bgs.ac.uk/technologies/web-services/api/
- BGS data.gov.uk source metadata: https://www.data.gov.uk/dataset/58f48757-fbaf-4e61-af12-c80408aac894/single-onshore-borehole-index
