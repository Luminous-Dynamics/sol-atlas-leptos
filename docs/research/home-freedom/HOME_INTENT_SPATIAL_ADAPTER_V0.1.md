# Home intent to spatial-design owner crosswalk v0.1

**Study:** HOME-INTENT-SPATIAL-BRIDGE-001  
**Parent:** [issue #806](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/806)  
**Source:** Home Design Brief Profile v0.1  
**Maturity:** PROPOSED_OWNER_REVIEW_REQUIRED  
**Execution:** NO_RUNTIME_ADAPTER

## 1. Purpose

This crosswalk records how the resident home brief should be translated into the existing Luminous design and accessibility owners without creating a second geometry, spatial topology, accessible-home planner or blueprint framework.

The crosswalk names target semantics and owner issues. It intentionally does not invent target schema paths or claim owner acceptance. Until a target owner provides an exact versioned schema and accepts the mapping, every row remains OWNER_CONFIRMATION_REQUIRED.

## 2. Owners and responsibilities

| Owner | Responsibility reused |
|---|---|
| Sol-Atlas #800 / PR #801 | Resident-authored brief, design candidate set and home-to-assembly requirements |
| Symthaea #7790 | Bounded intent problem, alternatives, Pareto exploration and value-of-information |
| Symtropy #1687 | Resident-led accessible-home tasks, room-by-room requirements and review states |
| Symtropy #1766 | Context-aware interior/spatial design and design workflow composition |
| Symtropy #455 | Faceted spatial topology (access/occupancy, air, acoustic, visibility, thermal etc.) |
| Symtropy #683 | Exact realized-geometry provider prerequisite; no production geometry claim until it qualifies |
| Symtropy #493 | Portable inert blueprints and local adaptation lineage |
| Mycelix #5149 | Consent, preference provenance, decision authority, approvals and revocation |
| Sol-Atlas #593 / #604 | Climate/site and South African authority/product context |
| Sol-Atlas #622; Symtropy #3773 | Fabrication bridge and panel-cell handoff after exact geometry exists |

## 3. Proposed semantic mappings

| Home brief source | Target semantic | Owner | Transformation/loss rule |
|---|---|---|---|
| household_frame | Occupancy and household-use context | Symthaea #7790; Symtropy #1687 | Preserve stated count/pattern; do not infer diagnosis, family roles or culture |
| intent.activities | Resident-stated functional task set | Symtropy #1687; Symthaea #7790 | Preserve source wording, frequency, dedicated-space and privacy intent; task decomposition must keep source IDs |
| intent.space_relationships | Spatial relationship constraints | Symtropy #1766 / #455 | Preserve relation type; unresolved/ambiguous mappings remain explicit |
| constraints | Non-negotiable design constraints | Symthaea #7790; Symtropy #1687 | Never convert to a preference, weight or reward term; contradictions remain blockers |
| preferences | Resident-editable soft preferences | Symthaea #7790; Mycelix #5149 | Keep priority/tradeability and source attribution; no automatic preference replacement |
| site_reference | Site, climate, rights and authority context references | Sol-Atlas #593/#604; Symtropy #1766 | Preserve evidence state and exact artifact references; do not infer parcel facts |
| delivery | Phased, adaptable design intent / blueprint parameter context | Symtropy #493; Sol-Atlas #622 | A blueprint remains inert data; adaptation creates a new lineage and does not authorize fabrication |
| privacy | Purpose-scoped participation and sharing conditions | Mycelix #5149 | Denial/revocation blocks export; local-only is the default |

These are semantic proposals, not exact field-to-field mappings. An owner may reject or refine any proposal.

## 4. Geometry promotion boundary

Symtropy #683 currently states that an eligible exact, complete, current realized-geometry provider has not yet been qualified. Therefore this adapter must not:
- generate or certify exact geometry;
- imply a floor plan fits a parcel;
- compute a production quantity takeoff;
- assert accessibility from a symbolic room programme;
- promote spatial topology or physical simulation results.

The only eligible current output is a resident intent / requirements package that carries unresolved geometry and review needs. When #683 later provides a qualified geometry receipt, a separately versioned adapter may bind geometry IDs and revision references.

## 5. Value-of-information rather than false certainty

The compiler should help residents and delivery teams identify which missing input could most change the decision. Candidate questions may include:
- Does the site boundary/right/servitude evidence change the feasible form?
- Is a current geotechnical investigation required before comparing foundations?
- Which municipal authority profile/code questions need confirmation?
- Would a site orientation/shading measurement change the courtyard decision?
- Which resident preference conflict needs another conversation?
- Which supplier quotation or availability check could change the assembly choice?

VOI outputs are recommendations about evidence collection. They do not commission surveys, spend money, access private data, book professionals or authorize construction.

## 6. P0 conformance gates

- Every source brief ID and non-negotiable constraint survives or is explicitly unresolved.
- 0 hard/soft severity inversions.
- 0 unsupported semantic mapped as exact equivalence.
- 0 OWNER_CONFIRMATION_REQUIRED row represented as a runtime-connected API.
- 0 accessibility requirement marked verified by the adapter.
- 0 geometry/quantity/buildability claim without exact qualified owner receipts.
- 0 disclosure after consent denial/revocation.
- 100% transformations and decompositions retain source IDs and loss notes.
- Positive controls: an owner-confirmed exact mapping can be admitted in a later version without changing source semantics.

## 7. Claim ceiling

This artifact is an owner-review crosswalk proposal. It contains no runtime integration, confirmed target field paths, accessibility verdict, candidate geometry, solver result, building approval or professional sign-off.
