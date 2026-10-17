# Coverage-Aware Spatial Constraint Overlays

Status: implementation note v0.1. This module interprets overlay receipts produced by a GIS adapter; it does not itself perform polygon intersections, raster sampling, coordinate transformation or legal/environmental analysis.

## Paradigm shift: absence needs proof

Traditional map overlays often reduce a constraint to “intersects” or “does not intersect.” That binary view is unsafe when source coverage is incomplete, geometries are invalid, coordinate systems do not match, observations are stale, or resolution is insufficient.

This implementation uses four findings:

- **Present:** the adapter reports a confirmed intersection under valid geometry, matching CRS, accepted source policy/state, acceptable overlay age and adequate accuracy. Complete coverage is not required to establish that a constraint was found.
- **Absent:** the adapter reports disjoint geometry *and* the source demonstrably covers the entire candidate area at adequate accuracy under the accepted evidence policy.
- **NeedsEvidence:** the relation is indeterminate, coverage is partial/unknown, accuracy is unknown/insufficient, evidence policy is unresolved/rejected, a required layer is missing, or a source state is not accepted.
- **Invalid:** geometry is invalid or the candidate and constraint CRSs mismatch. No silent coordinate transformation is performed.

The candidate gate is also conservative:

- Any confirmed **hard** constraint makes the candidate Excluded, even if another required hard layer is unresolved.
- Without a confirmed hard failure, any hard constraint not confirmed Absent leaves the candidate at NeedsEvidence.
- A candidate is Eligible only when all required hard constraints are confirmed Absent.
- Soft constraints remain visible in the report but do not override hard-gate eligibility; unresolved soft metrics must still be surfaced to downstream scoring.

This distinguishes “we proved no intersection within the required coverage” from “we could not find one.”

## Rust contract

The new \`sol-atlas-core::spatial_overlay\` module accepts a typed receipt from a GIS adapter and checks:
- candidate geometry identity via SHA-256 syntax and consistency across overlay receipts;
- source payload and constraint-layer geometry digest syntax;
- candidate CRS versus layer CRS, without implicit transforms;
- geometry validity, source-policy disposition and evidence state;
- overlay computation timestamp and an optional maximum age;
- coverage, spatial relation and declared positional accuracy;
- complete receipt presence for every candidate/required-constraint pair.

It returns stable dispositions and reasons for each candidate/constraint pair, plus aggregate counts for eligible, excluded and needs-evidence candidates. Each assessment retains source, layer geometry, candidate geometry and overlay-engine identifiers so a later evidence graph can bind the exact operation.

A digest check here checks syntax and compares supplied IDs; a separate ingestion layer must hash the actual bytes and protect artifact immutability. The GIS adapter must be responsible for the correctness of the reported geometry operation and evidence metadata.

## Synthetic benchmark

The fixture at \`docs/research/fixtures/spatial-overlay-benchmark-v0.1.jsonl\` contains 21 independent cases, including:
1. confirmed hard-constraint intersection with partial coverage;
2. complete coverage plus a disjoint result;
3. partial or unknown coverage with disjoint result;
4. missing required layers;
5. CRS mismatch, invalid/unknown geometry and indeterminate relations;
6. stale or rejected source evidence, insufficient/unknown accuracy and stale overlay receipts;
7. soft-constraint presence/missing data without changing hard-gate semantics;
8. confirmed hard-constraint presence alongside a separate missing hard layer;
9. an estimated source state that is not accepted under the test policy;
10. mixed candidate outcomes.

An independent JavaScript mirror reproduced all 21 declared candidate gates and per-constraint findings from the fetched-back fixture. That is an independent semantics check, not a Rust build/CI pass or validation of geometry arithmetic.

## Interoperability and research anchors

- **OGC API - Features** offers interoperable feature discovery/query building blocks and publishes coordinate-reference information for collections: https://www.ogc.org/standards/ogcapi-features/ and https://ogcapi.ogc.org/features/overview.html
- **OGC API - Tiles** specifies interoperable retrieval of map, vector-feature and other geospatial tiles. Tiles are useful for map delivery, but a rendered tile should not replace authoritative analytical geometry or coverage metadata: https://www.ogc.org/standards/ogcapi-tiles/
- **STAC 1.1.0** standardizes metadata for spatiotemporal assets such as satellite/drone imagery, point clouds, elevation models and related data: https://www.ogc.org/standards/stac/

These standards support discovery and data exchange; they do not establish that an individual layer is current, authoritative, complete or suitable for a specific constraint decision.

## Promotion gates

Before real parcel screening depends on this layer:

1. Add at least 30 pinned positive/negative controls across polygon, line, point and raster-derived constraints.
2. Test geometry operations against an independently implemented GIS library/reference, including boundary touches, holes, slivers, multipolygons, invalid rings and empty geometry.
3. Exercise CRS/axis-order transformations against known coordinates, with dateline/projection boundary cases and unit tests. Record transformation library/version and grid files.
4. Distinguish vector coverage masks, raster nodata masks, outside-source-extent, zero-valued cells and missing tiles.
5. Test resolution/positional uncertainty by perturbing geometry near candidate boundaries; report the size of indeterminate boundary bands.
6. Validate source payload hashes against exact bytes and pin source/overlay engine versions and configuration.
7. Measure throughput and memory across increasing candidate × layer counts; preserve deterministic results when work is parallelized or tiled.
8. Ensure protected/ecological/land-rights/hazard hard gates cannot be neutralized by weighted preference scores or unknown data.

No synthetic result or successful metadata check is a planning approval, proof of hazard absence, ecological clearance or engineering certification.
