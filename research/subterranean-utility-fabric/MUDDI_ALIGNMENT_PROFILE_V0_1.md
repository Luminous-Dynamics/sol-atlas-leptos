# Subterranean Utility Evidence Profile — MUDDI Alignment v0.1

**Status:** Draft logical-model crosswalk. Not a MUDDI conformance claim, utility registry or engineering approval.  
**Owner:** Sol-Atlas #755  
**Programme:** Symtropy SUF #4910  
**Canonical spatial evidence owner:** Sol-Atlas #450  
**Canonical evidence/approval graph:** Sol-Atlas PR #138  
**Robotics and qualification owners:** Symtropy #4913 and #4925

## 1. Purpose

Define a thin, security-conscious interchange profile that lets Sol-Atlas represent underground utility evidence while preserving the responsible utility's authority over canonical asset records and operational state.

The profile is aligned conceptually to **OGC Model for Underground Data Definition and Integration (MUDDI) Part 1 v1.0**. MUDDI is an approved conceptual model, not a ready-made JSON or Rust schema. A conformant implementation must derive a use-case-specific logical model, choose concrete geometry/value types, implement the core concepts and associations, and supply the applicable conformance evidence.

This v0.1 is a crosswalk and research contract only. It does not implement or claim full MUDDI conformance.

## 2. Architectural boundary

| Concern | Owner | Sol-Atlas role |
|---|---|---|
| Real-world asset identity and current operational state | Utility / infrastructure owner | Refer to it; do not mint a competing canonical asset ID |
| Utility-network topology and service operations | Utility owner / domain system | Consume authorized, versioned snapshots |
| Survey, GPR, exposed verification and robotic observations | Surveyor / instrument / inspection provider | Preserve as attributed observations with lineage |
| Spatial reference, geometry representation, evidence view and uncertainty | Sol-Atlas | Own renderer-independent spatial evidence and transformations |
| Engineering scenario execution and physical qualification | Symtropy and named domain solvers | Consume qualified inputs and emit scoped results |
| Robot/task proposals | Symthaea | Bounded proposals only; no physical authority |
| Actor, work authorization, material and accepted-work provenance | Mycelix / responsible institution | Reference exact receipts; no ledger-issued permit |

The evidence model must not turn a map projection into the canonical asset database. An observation may challenge an asset record; it does not overwrite that record without the owner-controlled reconciliation process.

## 3. MUDDI core crosswalk

The JSON crosswalk at `muddi-core-crosswalk-v0.1.json` lists the MUDDI v1.0 core concepts and maps each to current Luminous owners. The key distinctions are:

- **MUDDIObject:** the common spatially identified underground/surface object envelope.
- **MUDDIAsset:** the asset identity and asset-specific attributes; canonical ownership remains external.
- **MUDDIEvent:** time-bounded events associated with the underground environment, including observations and lifecycle events.
- **MUDDISpace:** spatial regions such as sites, service areas and planning areas.
- **Network:** a logical collection/network context.
- **NetworkAsset:** network member assets.
- **NetworkConveyance:** physical or logical conveyance of a commodity/service.
- **NetworkNode / NetworkLink:** network topology.
- **NetworkAccessory:** supporting infrastructure such as ducts, chambers, valves or sensors.

MUDDI's extended **Observation** concept is used for observations of a feature of interest. A survey, GPR trace, exposed verification, robot inspection or test result should be represented as an observation/event with an explicit link to the feature it concerns and the evidence that supports it.

### Identifiers are not interchangeable

MUDDI distinguishes:

1. **ObjectID** — persistent identifier for the real-world object.
2. **RecordID** — persistent identifier for a particular representation/record in a dataset or data product.
3. **SystemID** — identifier assigned by a particular implementation system.

Do not substitute one for another. A single real-world asset may have several records and system-specific aliases. Every SystemID must carry its system/provider namespace. An ObjectID is owner-issued or owner-mapped; Sol-Atlas does not infer ownership from a geometry match.

## 4. Claim-bearing spatial representation

A geometry is not just a coordinate array. Every geometry-bearing record must carry:

- geometry type and dimensionality (2D / 2.5D / 3D);
- horizontal CRS, vertical datum and coordinate epoch where applicable;
- units and axis order;
- position/extent uncertainty and its representation;
- source observation or owner-record reference;
- valid time, observation time and publication/record time;
- transformation lineage and exact source snapshot/revision;
- epistemic class and currentness/conflict status;
- disclosure classification and permitted purpose.

A route may have several valid representations: a surveyed 3D alignment, a simplified 2D planning geometry and a topology-only network link. Those representations must share identity/lineage but not be collapsed into one allegedly exact geometry.

Coordinate frame and datum mismatch is a hard incompatibility, not a warning to be hidden by the renderer.

## 5. Evidence traceability contract

An observation record must identify:

- feature of interest or explicit unresolved target;
- observing organization/actor and role;
- method, instrument, firmware/software and calibration identities;
- observation time and valid-time scope;
- spatial frame and position uncertainty;
- original data artifact reference, digest and custody/rights state;
- processing/transformation lineage and algorithm revision;
- quality/adjudication state and reviewer;
- relationship to other observations (corroborates, conflicts, supersedes, duplicates, derived-from);
- authorized purpose and disclosure class.

A SHA-256 digest is an integrity reference, not proof of source authenticity, instrument calibration, scientific truth or legal authority. Signature verification, custody and access policy are separate controls.

An observation that conflicts with the current asset record remains a conflict until resolved by the responsible owner. Never choose the newest timestamp automatically: valid time, observed time and recorded time are distinct.

## 6. State vocabulary

At minimum, preserve these distinctions:

- `observed` — directly measured or witnessed within a declared method envelope;
- `compiled_secondary` — compiled from other sources;
- `model_inference` — generated by a declared model;
- `scenario_assumption` — synthetic or hypothetical input;
- `speculative_hypothesis` — unconfirmed proposition;
- `unknown` — required information has not been established;
- `unobserved` — a spatial/temporal interval was not inspected;
- `unavailable` — required source or service could not be obtained;
- `withheld` — information exists but is not available to this caller;
- `stale` — evidence no longer meets a declared currentness policy;
- `conflicting` — admissible evidence sources disagree;
- `unsupported` — the requested claim is outside model, sensor or qualification scope.

None of unknown, unobserved, unavailable, withheld, stale, conflicting or unsupported may be silently converted to absence, empty geometry, zero or clear.

## 7. Security and disclosure

Underground infrastructure data can expose critical routes, service dependencies, access chambers, isolation points, sensor placements and failure locations. MUDDI explicitly leaves implementation security to the platform and warns about malicious use of underground infrastructure information.

Use purpose-bound, least-privilege access with distinct disclosure classes:

| Class | Intended consumer | Geometry policy |
|---|---|---|
| Public aggregate | General public | Aggregated service area/status only; no exact topology or access points |
| Partner generalized | Approved planning partners | Generalized geometry and minimum necessary attributes for a named purpose |
| Operational | Authorized utility/contractor roles | Exact geometry only for approved work scope and time window |
| Restricted critical | Designated security/owner roles | Highest-sensitivity geometry, dependencies and security observations; explicit approval and audit |

The class labels are policy inputs, not an authorization engine. Enforcement belongs to the host platform and utility owner. The synthetic fixture uses abstract local coordinates only and must never contain a real route or live infrastructure record.

## 8. Interoperability and conformance

The profile should map to MUDDI v1.0 core concepts and preserve the model's associations. It should not invent proprietary replacements for MUDDI class names without an explicit mapping table.

Before making a conformance claim, the implementation must:
1. define a complete logical model and concrete geometry/value types;
2. map all mandatory core concepts, associations and attributes;
3. implement the applicable MUDDI abstract tests (including the core class, association, attribute and concrete-value-type tests);
4. publish fixtures and reproducible conformance evidence;
5. document optional extensions and jurisdiction-specific restrictions.

PAS 128:2022 may inform survey-quality workflows and confidence classification, but it is not South African law and does not replace South African statutory, municipal, utility-owner or project requirements. MUDDI alignment does not certify survey quality.

## 9. Benchmark and negative controls

The v0.1 benchmark contract requires:
- 100% of MUDDI core concepts mapped or visibly unresolved/externally owned;
- ObjectID / RecordID / SystemID collision and namespace-alias rejection;
- no stale or superseded geometry promoted to current;
- CRS, vertical datum, epoch, axis order and unit incompatibilities rejected;
- every geometry claim to carry explicit dimensionality, frame and uncertainty state;
- every observation linked to a feature of interest and evidence reference, or explicitly unresolved;
- contradictory observations preserved as conflicts;
- withheld data never rendered as absent;
- restricted synthetic geometry not disclosed to unauthorized roles;
- byte-stable canonical serialization for frozen synthetic fixtures;
- export/import round-trip preserving identifiers, provenance, unknowns, conflicts and security labels.

No single score should hide a failed negative control. Report each case, disposition and exact profile revision.

## 10. Claim ceiling

This profile is a draft interchange crosswalk. It contains no production adapter, live utility data, MUDDI conformance result, survey validation, owner authorization, geospatial accuracy result, permit, construction approval or physical safety case. A mapped concept is not proof that a real asset exists at the mapped location.

## References

- OGC MUDDI Part 1 v1.0 (approved 7 June 2024; published 5 July 2024): https://docs.ogc.org/is/23-024/23-024.html
- OGC MUDDI standard and schema resources: https://www.ogc.org/standards/muddi/
- BSI PAS 128:2022 overview and revision history: https://accord-checkout.bsigroup.com/products/pas-128-underground-utility-detection-verification-and-location-specification
- Sol-Atlas provenance owner #450: https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/450
- Sol-Atlas evidence graph PR #138: https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/138
- Symtropy SUF programme #4910: https://github.com/Luminous-Dynamics/symtropy/issues/4910
