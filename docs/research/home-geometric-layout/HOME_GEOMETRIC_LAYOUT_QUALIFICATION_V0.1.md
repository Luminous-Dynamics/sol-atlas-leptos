# Home Geometric Layout Qualification v0.1

**Status:** synthetic geometry fixture and deterministic Rust evaluator authored; hosted validation pending.  
**Maturity:** 2D orthogonal rectangles only.  
**Purpose:** bridge the resident's symbolic room/activity graph to dimensioned candidate geometry without turning a generated drawing into a safety or permit claim.

## Why this is the next step

The pipeline now has resident-authored intent, symbolic activity-to-space assignments, hard constraints, material records, assembly demand, DfMA hypotheses and whole-cost-of-place boundaries. The missing executable bridge is a geometric object that can be checked independently of its visual rendering.

A floor plan should be treated as a typed constraint object with traceability:
- every geometric room maps to exactly one resident-intent space;
- every source space appears exactly once;
- every room's area is reconciled exactly in integer square millimetres;
- every room lies inside the declared envelope and overlaps no other room;
- every opening lies on a shared room boundary;
- every declared route is connected through qualified openings;
- each source-graph DIRECT_ACCESS requirement has an actual connecting opening.

This is a geometry consistency checker, not a floor-plan generator or building-code engine.

## Industry position and interoperability

Generative layout is becoming an incumbent feature, not a defensible moat by itself. Autodesk announced Building Layout Explorer in Forma as an experimental AI workflow in June 2026 and discussed further layout exploration at Autodesk University in October 2026. Finch markets residential generation and large-scale option exploration. Luminous should not claim novelty for "AI makes floor plans" or compete on render count alone.

The differentiation is resident intent preservation, exact geometric failure detection, explicit evidence maturity, non-compensatory constraints, traceable revision invalidation, and clean specialist hand-off. The system should interoperate with existing BIM rather than create a proprietary replacement ontology.

- IFC 4.3.2.0 is the current official IFC release (ISO 16739-1:2024); it is the vendor-neutral open exchange representation for built assets.
- IDS 1.0 is an official buildingSMART information-requirement standard. Its documentation explicitly lists advanced geometry checks (such as clashes and boundary-distance requirements) as outside IDS v1 scope. That creates a clear separation: use IFC/IDS for exchange and information requirements; run geometry-specific qualification in a dedicated, auditable engine.
- RIBA's Plan of Work treats briefing, design, construction and operation as staged outcomes with separate strategies such as fire, inclusive design, cost, procurement and plan-for-use. A geometric pass cannot collapse these roles.

Sources:
- Autodesk Forma Building Layout Explorer (2 June 2026): https://adsknews.autodesk.com/en/news/building-layout-explorer-in-autodesk-forma/
- Autodesk Forma AU 2026 announcement (14 October 2026): https://blogs.autodesk.com/forma/2026/10/14/what-autodesk-forma-announced-au-2026/
- Finch: https://www.finch3d.com/
- IFC: https://www.buildingsmart.org/standards/bsi-standards/industry-foundation-classes/
- IDS: https://www.buildingsmart.org/standards/bsi-standards/information-delivery-specification-ids/
- IDS geometry limitations: https://github.com/buildingSMART/IDS/blob/development/Documentation/UserManual/specifications.md
- RIBA Plan of Work: https://www.riba.org/our-work/insights-and-resources/riba-plan-of-work/

## v0.1 contract

Coordinate system:
- unit: integer millimetres;
- origin: south-west of the synthetic rectangular envelope;
- positive X: east; positive Y: north;
- no fractional millimetre coordinates;
- floor index 0 only.

Geometry:
- orthogonal axis-aligned rectangles;
- no polygons, curved walls, wall thickness, structural grid, shafts, stair geometry, roof geometry, external setbacks, easements or terrain;
- room and outdoor-space area must match the source graph exactly;
- room rectangles may touch at boundaries but may not overlap by positive area.

Openings:
- represented as a segment on a shared horizontal or vertical room boundary;
- opening spans must lie wholly inside the shared boundary segment;
- a valid opening creates a traversable edge between the two rooms;
- the v0.1 opening is only a connectivity portal. It does not describe a door leaf, swing, clear opening, threshold, fire rating, acoustic rating, hardware or construction detail.

Routes:
- routes are graph connectivity assertions over the valid opening graph;
- v0.1 checks reachability only;
- it does not calculate travel distance, turning circles, gradients, clear width, travel time, protected escape paths or accessible-route compliance.

## Synthetic baseline

Source graph: GRAPH-HV-B from the resident spatial-intent graph fixture. This graph contains 78 m² of internal programme area and 8 m² outdoor service-yard area.

The geometric fixture contains ten one-to-one space mappings. Exact room areas sum to:
- internal: 78,000,000 mm²;
- outdoor: 8,000,000 mm².

The declared 13,750 mm × 8,500 mm envelope is synthetic and is not a survey, plot boundary, lawful buildable envelope, setback result or planning approval. Room placement is a synthetic test of the checker, not an endorsed layout.

## Qualifier tests

The Rust binary authors 16 unit tests:
1. valid synthetic geometry candidate;
2. positive-area room overlap;
3. a room outside the envelope;
4. zero room width;
5. source-area mismatch;
6. missing space;
7. duplicate space;
8. an opening not on the shared boundary;
9. an opening span outside the shared boundary;
10. missing required direct access;
11. disconnected access route;
12. physical maturity promotion;
13. scalar score introduction;
14. nonzero floor-index promotion;
15. coordinate-origin drift;
16. unknown source graph candidate.

The evaluator uses integer arithmetic and checked addition/multiplication for rectangle coordinates and area. It does not use floating-point area tolerances.

## Benchmark ladder and target metrics

**G0 — Structural integrity (this PR):** exact schema/fixture identity, graph-space traceability, dimensions, envelope, overlap, opening geometry and graph reachability.

**G1 — Independent reference parity:** differential testing against an independently implemented computational-geometry oracle; report exact disagreements and malformed-input rejection.

**G2 — Professional adjudication:** blind architect/technologist review of dimensioned candidates; measure false-pass rate, false-block rate, defect recall and adjudicator agreement by defect class.

**G3 — Interoperability:** IFC export/import round-trip tests; compare space IDs, placements, areas, openings and units; preserve source identity and change receipts.

**G4 — Detailed design integration:** walls, wall thickness, doors/windows, structural grids, plumbing/electrical zones, daylight/solar analysis, ventilation, fire/egress and accessibility using their qualified specialist owners and jurisdiction-specific rule sets.

**G5 — Physical delivery:** site/fabrication/as-built reconciliation, tolerance accumulation, first-time-right, rework, cost and post-occupancy outcomes. A digital geometry pass is never a substitute for physical inspection.

Initial synthetic acceptance targets:
- zero false geometric passes in the frozen adversarial set;
- 100% exact known-answer agreement;
- 100% one-to-one room traceability;
- zero silent unit conversions;
- zero promotion of acoustic, visual-privacy, accessibility, fire, daylight, thermal, structural, water or legal claims;
- deterministic replay from frozen inputs.

Real-world targets must be set after an independently adjudicated corpus exists. Do not publish numeric superiority over Autodesk, Finch, Revit, Archicad, or professional practice without matched tasks, the same source brief/site constraints, independent review, and comparable outputs.

## Explicitly not evaluated

No floor-plan aesthetic, daylight, solar gain, overheating, ventilation, noise transmission, sightlines, accessibility, fire compartmentation, means of escape, structural feasibility, foundations, moisture, services, material performance, cost, manufacturing, construction sequence, legal rights, zoning, authority rules, site suitability or resident acceptance is evaluated.

No permit, compliance, safety, buildability, professional design or occupancy claim is supported. Candidate remains synthetic and unranked.