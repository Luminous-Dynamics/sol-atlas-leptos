# Subterra Atlas v0.1 — Implementation and Validation Status

**Repository:** Luminous-Dynamics/sol-atlas-leptos (public)  
**Branch:** `research/subterra-atlas-v0.1`  
**Date:** 2026-11-16  
**Production code changes:** none

## Repository assessment

The inspected production code has a functioning Earth-first globe and Solar System visual layout, but planetary site data is not yet multi-body cartography:
- Existing `Site` and related records use unqualified latitude/longitude fields.
- Layer labels and region records are Earth-centric.
- The geodetic distance helper contains a fixed Earth radius (6,371 km).
- The Solar System body list provides orbital visualization parameters, not planetary body-fixed reference frames or surface shape models.
- Existing `DataProvenance` distinguishes observed, curated and scenario data but does not yet capture full per-observation coordinate frame, calibration, uncertainty and processing lineage.

## Artifacts

- `ATLAS_INTEGRATION_v0.1.md`: architecture and coordinate/evidence requirements.
- `PLANETARY_SITE_SCHEMA_v0.1.json`: explicit body/frame/coordinate/observation/provenance record.
- `fixtures/synthetic-moon-site.json`: synthetic site record with a body-fixed frame and unresolved cave connectivity.

## Validation

| Check | Result |
|---|---|
| JSON Schema draft-2020-12 meta-validation | PASS |
| Synthetic Moon fixture validation | PASS |
| Rust compile/test | Not applicable; no Rust source changed |
| Coordinate transforms | Not implemented in this increment |
| Renderer layers | Not implemented in this increment |
| Real planetary site dataset | Not included; fixture is synthetic |

## Next increments

1. Add body-aware geodetic/Cartesian transforms using controlled IAU/PDS/NAIF frame and shape-model references.
2. Add explicit body and frame metadata alongside existing Earth types; do not silently reinterpret old datasets.
3. Add a multi-body Subterra feature layer with evidence-state and preservation filters.
4. Add surface-to-local-cave frame registration and residual/uncertainty display.
5. Add read-only ingestion of versioned Subterra site evidence from Symtropy.
6. Add regression tests for Earth behavior and planetary frame transformations, including longitude direction, frame epoch, poles and irregular small-body shapes.

No existing Earth dataset or production coordinate transform is modified by this research increment.
