# Source Evidence Contract for Site Selection

Status: research and implementation note v0.1. The module evaluates metadata against an explicit policy; it does not determine whether the underlying dataset is true or complete.

## Core rule

A map feature is not a finding. Each site fact needs a traceable source, edition, rights status, authority appropriate to that fact type, data date, payload identity, coordinate reference information and coverage. A source that was downloaded today can still contain stale data. A missing or non-covering layer is not evidence that a hazard or constraint is absent.

This follows two industry-wide interoperability needs:
- OGC API - Features provides standard building blocks for discovering, querying and accessing geospatial features. Feature schemas and spatial metadata still need to be validated by the client: https://www.ogc.org/standards/ogcapi-features/
- IFC 4.3 is an open international model for sharing building and infrastructure information through the lifecycle, while buildingSMART IDS lets teams specify machine-checkable information requirements for model exchange: https://standards.buildingsmart.org/IFC/RELEASE/IFC4_3/HTML/content/introduction.htm and https://technical.buildingsmart.org/projects/information-delivery-specification-ids/
- NIST identifies semantic interoperability and machine-readable building data as a central challenge for lifecycle digitalization: https://www.nist.gov/programs-projects/building-digitization-and-semantic-interoperability

These standards support interoperable data; they do not validate a suitability decision.

## What the Rust slice checks

The source evidence module currently records:
- source ID, publisher, dataset, revision, payload SHA-256 syntax, rights/license class and authority class;
- retrieval time versus source data-as-of time, explicit expiry, and a project-specific maximum age;
- CRS identifier, axis order, horizontal unit, declared spatial bounds and optional ground resolution;
- a policy's accepted authority/rights classes, required CRS and axis order, required extent, resolution cap, timestamp and payload-digest requirements.

The assessment preserves a primary disposition plus reasons. Statuses include Ready, Stale, Expired, FutureDated, UnknownFreshness, MissingDigest, RightsUnknown, RightsNotAllowed, AuthorityNotAccepted, SpatialMetadataMissing, SpatialMetadataInvalid, CrsMismatch, AxisOrderUnknown, AxisOrderMismatch, CoverageUnknown, CoverageGap, ResolutionUnknown, ResolutionInsufficient and InvalidMetadata.

A Ready result means only that supplied metadata satisfies the supplied policy. It does not independently download or hash the real payload, interpret legal title, perform a coordinate transform, run a spatial intersection against a protected-area polygon, or confirm that the mapped field is accurate. The ingestion and GIS layers must do that work and supply the correct status/metadata.

## Important design choices

1. **Policy is fact-specific.** Accepted evidence for planning permission need not be the accepted evidence for lived-experience impact or biodiversity surveys. The module has no global hierarchy claiming government, commercial, professional, community or research inputs are always superior.
2. **Rights are explicit.** Public accessibility is not equivalent to redistribution permission. Restricted and unknown rights should be quarantined unless the specific approved workflow allows them.
3. **Retrieval is not freshness.** Downloading an old dataset today does not refresh its source-data timestamp.
4. **CRS mismatch fails closed.** This module intentionally does not silently convert coordinates. A geospatial adapter must apply a named, versioned transformation and independently test axis order, units and boundary behavior.
5. **Coverage gaps remain gaps.** A source bounding box is only a coarse coverage check. Real evaluation must also verify geometry validity, layer-specific masks/coverage, spatial resolution, topology and edge behavior.
6. **Metadata validity is not fact validity.** A valid hash proves identity only when the exact bytes are hashed and compared. It does not prove that the dataset's contents are correct or authoritative.

## Synthetic benchmark

The fixture at docs/research/fixtures/site-evidence-benchmark-v0.1.jsonl contains 16 synthetic cases:
- accepted fresh source;
- stale source despite recent retrieval;
- missing source timestamp;
- unknown/restricted rights;
- CRS mismatch and axis-order unknown;
- declared coverage gap and unknown coverage;
- expired and future-dated source data;
- malformed/missing digest;
- insufficient spatial resolution;
- unaccepted authority class;
- missing spatial metadata.

An independent JavaScript mirror matched all 16 declared dispositions after fetching back the JSONL fixture. This is not execution of the Rust implementation.

## Proposed promotion gates

Before real parcel data is used, expand to at least 30 cases and include:
- digest byte-level verification in the ingestion layer, with source payload and manifest pinned;
- valid and invalid CRS/axis transformations against independently known coordinates, including axis order, dateline, projected units and boundary tests;
- geometry validity, topology, spatial coverage masks, raster nodata and resolution mismatch;
- rights and attribution/redistribution controls;
- per-source freshness and expiry rules, future timestamps, conflict resolution and correction history;
- source-specific, jurisdiction-specific authority policies;
- determinism and source-update replay tests;
- explicit uncertainty maps and no-data overlays in the UI.

No synthetic fixture or passing metadata check establishes land rights, zoning, hazard absence, ecological clearance or engineering safety.

## Research references

- OGC API - Features and feature schemas: https://www.ogc.org/standards/ogcapi-features/ and https://docs.ogc.org/is/23-058r2/23-058r2.html
- buildingSMART IFC 4.3.2.0 (IFC4.3 ADD2, ISO 16739-1:2024): https://technical.buildingsmart.org/standards/ifc/ifc-schema-specifications/
- buildingSMART Information Delivery Specification: https://technical.buildingsmart.org/projects/information-delivery-specification-ids/
- NIST, Building Digitization and Semantic Interoperability: https://www.nist.gov/programs-projects/building-digitization-and-semantic-interoperability

The standards anchor data exchange. The evidence policy, source quality and real-site findings need independent validation.
