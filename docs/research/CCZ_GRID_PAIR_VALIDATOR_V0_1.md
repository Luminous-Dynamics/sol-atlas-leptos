# CCZ Grid-Pair Validator v0.1

**Status:** synthetic-only executable metadata contract. No GEBCO artifact has been ingested by this work.

## Problem

The GEBCO grid and its Type Identifier (TID) grid are separate artifacts. Matching file names or URLs does not establish that they share the same edition, raster dimensions, bounds, coordinate reference, axis order, spacing or registration. A visually plausible overlay can be wrong if those assumptions differ.

GEBCO describes its 2026 terrain grid as a 15-arc-second, pixel-centre-registered global grid with 43,200 rows by 86,400 columns. The accompanying TID grid describes the source-data type for corresponding cells. The grid includes heterogeneous measured and inferred source data, and the resolution of the output grid must not be confused with direct measurement resolution. GEBCO explicitly warns against using the grid for navigation or other safety-at-sea purposes.

References:
- [GEBCO_2026 Grid and TID documentation](https://www.gebco.net/data-products-gridded-bathymetry-data/gebco2026-grid)
- [GEBCO TID code definitions](https://www.gebco.net/gebco-tid-grid)
- [GEBCO grid development and interpolation explanation](https://www.gebco.net/data-products/gridded-bathymetry-data/grid-development)

## Implemented contract

The Rust module `sol-atlas-core::ccz_grid_pair` validates a pair of strict metadata records before any downstream workflow treats them as aligned. It requires each artifact to point to an existing source snapshot and contain a well-formed SHA-256 reference. The existing exact-byte receipt remains responsible for computing and comparing hashes; the pair validator does not claim to verify bytes.

Pair checks compare product family, edition, grid identity, dimensions, outer-edge bounds, cell spacing, CRS, horizontal units, row/column axes and registration. Each raster must also declare its own NoData representation. Dimension/spacing/bounds must be internally consistent, not just consistent across the pair.

The bathymetry metadata must include vertical datum and unit semantics before it can pass this research-context metadata gate. TID metadata is an independent artifact with its own snapshot, digest reference and NoData rules; it must not inherit the bathymetry artifact's provenance.

Cell decoders keep distinct states for numeric depth values and NoData. TID decoding preserves:
- direct-measurement categories separately from predicted/interpolated categories;
- code 70 (pre-generated grid with mixed source types) separately from code 71 (unknown source);
- unknown future/unrecognized codes as their original numeric value;
- a declared NoData sentinel separately from any valid TID code.

No scalar "data quality" score is produced, and no species-presence, ecological-safety, route-clearance or extraction decision is inferred from the TID value.

## Benchmark corpus and acceptance targets

The in-module synthetic regression tests provide a positive control and fault-injection controls for:

| Fault or distinction | Required behavior |
|---|---|
| Aligned pair with matching geometry and edition | Accept metadata pair |
| Product edition mismatch | Reject |
| Row/column dimension mismatch | Reject |
| Spatial bounds mismatch | Reject |
| CRS mismatch | Reject |
| Axis-order mismatch | Reject |
| Spacing inconsistent with extent | Reject |
| Missing bathymetry vertical datum or units | Reject |
| Unknown NoData encoding on either grid | Reject |
| Malformed artifact digest reference | Reject |
| Declared depth NoData sentinel | Preserve as `NoData`, never as depth zero |
| Actual depth value zero with a different sentinel | Preserve as numeric zero, without adding physical meaning |
| TID code 70 vs 71 | Preserve mixed-source vs unknown-source distinction |
| Unrecognized TID code | Preserve original code, not silently relabel as NoData |
| Declared TID NoData sentinel | Preserve as NoData, not unknown-source code |

Acceptance goals for the executable suite:
- every injected pairing fault is rejected (zero false accepts across the frozen fixture suite);
- zero silent NoData-to-zero conversions;
- zero coercions of unknown or mixed TID classes into directly measured data;
- the same input record produces the same ordered issue list;
- rights review and exact-byte verification remain separate, mandatory gates;
- no navigation or ecological-safety claims are emitted.

These are deterministic software-contract tests, not a hydrodynamic model, a biological model, an independent scientific validation, or evidence that a real-world artifact is ingestible. The current fixture is synthetic. `not_ingested` remains the truthful status until one endpoint's terms are reviewed, an authorized artifact is acquired and pinned, and its metadata/payload pairing is independently checked.

## Next research milestone

After the contract tests are passing on the exact commit, build a tiny parser-backed fixture from synthetic NetCDF metadata and cell arrays. Only after selected-endpoint terms are resolved should the pipeline attempt a bounded real-data subset. At that stage, compare decoded metadata against the source documentation and an independent reader; preserve original bytes, parsed values, tool/configuration versions and output digests. Keep environmental evidence (species/habitat observations and sampling effort) in its own model rather than treating bathymetry/TID as a biodiversity proxy.
