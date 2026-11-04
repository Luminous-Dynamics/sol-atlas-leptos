# HDB-17: Stone-to-Settlement Architectural Metrology v0.1

## Purpose

HDB-17 evaluates whether architectural observations can be carried from individual stones through wall segments and larger assemblies without losing physical identity, measurement semantics, uncertainty, provenance or unresolved relationships.

The initial Rust slice validates a source-bound graph contract. It is not a detector, GIS engine, metrology engine, or archaeological interpretation engine.

## Implemented contract

- Physical element identity is separate from geometry observation identity. One physical element may carry multiple observation records from different captures.
- Hierarchy is explicit: stone → wall segment → wall → building → compound → insula → neighbourhood → settlement.
- Parentage distinguishes assigned, unassigned, ambiguous and not assessed. Ambiguity requires multiple distinct, valid next-level candidates.
- Source captures are scoped to the same site and snapshot; capture bytes have SHA-256 identifiers; verified rights require a receipt reference.
- Pixel-only measurements require pixel area and prohibit metric area, perimeter, CRS and calibration claims.
- Planar and 3D surface area semantics are distinct. Metric geometry requires a declared CRS and a calibration record referenced by a source capture in that CRS.
- Model and simulation geometry require a method-build digest and are counted separately.
- Direct measurement requires a field-metrology-capable capture type.
- Metric uncertainty must be finite and nonnegative; a declared 90% interval must contain the measured area.
- The report is deterministic and fixes its claim ceiling at `structural_integrity_only`.

## Deliberate non-claims

A valid report does not prove:
- source owner identity, signature authenticity, rights authenticity or complete source inventories;
- that the supplied geometry digest corresponds to the true source geometry;
- image annotation correctness, detector precision/recall or real-world measurement accuracy;
- completeness of the feature inventory or correctness of assembly interpretation;
- archaeological identity, chronology, labour, wealth, status, ethnicity, political boundaries or settlement history.

A metric observation may still be wrong even when its metadata and graph structure are internally consistent. Independent field measurements and blind adjudication remain required.

## Benchmark plan

The outer benchmark should separately score:
1. stone instance detection at 15×15, 20×20 and 25×25 cm² thresholds;
2. wall segmentation at 0.7, 0.6 and 0.5 wall-coverage cutoffs;
3. metric geometry against independent RTK/total-station/validated photogrammetric references;
4. hierarchical edge accuracy and aggregate measurement stability;
5. annotation plus correction effort under matched quality floors.

Report macro-by-site results, confidence intervals, OOD region/period results and subgroup strata. Do not use random overlapping image tiles as independent train/test units. Keep image annotation, field metric truth, chronology and historical interpretation as distinct ground-truth tiers.

## Reproduction references

- Uziel et al. (2026), *Semi-automatic detection of building stones and wall segments of archaeological ruins*, Journal of Archaeological Science 185, 106430. DOI: 10.1016/j.jas.2025.106430.
- Uziel et al. (2026), *Quantifying ancient architecture: Stone-level machine learning reveals architectural variation and settlement growth at Shivta*, PNAS. DOI: 10.1073/pnas.2628851123.
