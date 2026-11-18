# LADM Profile Crosswalk — TERR-001

**Status:** Research mapping; not a conformance claim  
**Date:** 2026-11-18  
**Implementation context:** `sol-atlas-core::territorial` v0.1  
**Standards baseline:** ISO 19152-1:2024 and ISO 19152-2:2025

## Purpose

This document describes how the Sol Atlas territorial evidence contract relates to the ISO Land Administration Domain Model (LADM). The goal is interoperability with established land-administration information models, not to recreate LADM or claim full conformance.

LADM is deliberately a jurisdiction-adaptable conceptual model. The model must not encode one country's legal hierarchy as a universal rule. This crosswalk identifies direct mappings, external references and gaps that require a later profile.

## Core rule

```text
LADM-aligned interchange
!= LADM conformance
!= national legal validity
!= cadastral authority
!= proof of ownership
```

The v0.1 Rust type set is an evidence envelope around references to authoritative land-administration, community, survey, environmental and legal records. It is not a complete implementation of the LADM conceptual schema or any South African national profile.

## Crosswalk

| LADM concept | LADM role | TERR-001 v0.1 representation | Status / gap |
|---|---|---|---|
| LA_Party | Person or organisation with a role in land administration | `TerritorialAssertion.holder_ref`; `AuthorityReference.issuer_ref`; `StewardshipObligation.responsible_party_ref` | Opaque references only. Party, person, group-party, identity and role records remain in their owning registries. |
| LA_GroupParty | Group of parties | `holder_ref` may identify a community, collective or group record | Reference only. Group membership, representation, mandate and succession must be resolved by the relevant community/governance system; no group-member inference is made. |
| LA_BAUnit | Basic administrative unit to which rights/restrictions may be attached | No direct field in v0.1. `territory_ref` identifies the evidence bundle, not an LA_BAUnit | **Gap.** Add an explicit external BAUnit reference and jurisdiction-profile identifier before claiming full RRR/BAUnit mapping. Do not overload `territory_ref`. |
| LA_RRR | Rights, restrictions and responsibilities | `interest_kind`, `relationship`, `source_interest_type_code` and `source_interest_type_profile_ref` | Partial semantic mapping. Original jurisdictional codes and code-system profile are retained when generic mapping is incomplete. Legal effect, priority, share, beneficiaries, and conditionality still need versioned jurisdiction profiles. |
| LA_SpatialUnit | Spatial unit, including parcel and other legally relevant spaces | `SpatialExtent.feature_ref`, horizontal CRS, optional vertical range/datum and horizontal uncertainty | External geometry reference. No geometry is stored here; topology, survey method, accuracy class, geometry validity, 3D volumes and representation history are not yet implemented. |
| LA_Source | Source from which an administrative object is obtained | `EvidenceReference` | Partial. Evidence references preserve source URI, kind, capture time, digest and disclosure class, but do not authenticate the issuer, establish legal authenticity, or guarantee source completeness. |
| Administrative source / authority | Instrument, decision or record issued by a competent organisation | `AuthorityReference` linked to source evidence | Partial. Authority kind, jurisdiction, scope and validity are represented; credential verification, delegation, revocation, recognition and competence remain external. |
| LA_VersionedObject | Versioning of administrative objects | Schema version and content digest, plus `TemporalExtent` valid-time strings | Partial. The current record does not implement LADM's full versioning model, transaction-time history, bitemporal reconciliation or event-sourced supersession. Coordinate with Mycelix EPI-008. |
| Surveying / spatial representation | Survey and geometry/topology evidence | Survey `EvidenceReference` and spatial metadata | Partial. A survey file can be referenced; surveyor competence, cadastral adjustment, topology, datum transformation, accuracy statements and legal survey acceptance are not qualified. |
| Dispute / challenge | Contestation over assertions or records | `ChallengeReference`, `ChallengeAssessment` and independent `DisputeState` | Domain extension, not a direct LADM substitute. Challenge history is explicit; a no-challenge observation requires a declared scope/frontier, searched sources and challenge-search report. It is not a global absence claim. Court/tribunal and customary processes still need jurisdictional profiles and external outcomes. |
| Stewardship / environmental duty | Long-running obligation with physical outcome evidence | `StewardshipObligation` | Domain extension. Links an originating evidence item, responsible party, spatial feature, verification protocol and independent verifier; does not prove the outcome scientifically. |

## Why rights are modelled as relationships, not an owner field

A land-administration object can have multiple parties and multiple RRR relationships over overlapping or nested spatial units. The relevant relationships can include:

- a registered surface interest;
- a customary or informal use/occupation claim;
- a mineral or subsurface authorisation;
- a groundwater or surface-water permission;
- an easement or utility corridor;
- a conservation covenant or restriction;
- a rehabilitation, access or benefit-sharing responsibility.

Those are not interchangeable rights and may be governed by different legislation, instruments, institutions and procedures. In South Africa, mineral rights and surface interests must not be collapsed into one ownership flag; section 3 of the MPRDA establishes the State's custodianship role for mineral and petroleum resources. The data model therefore retains separate interest kinds, legal/administrative bases, authorities, evidence and dispute states.

The current `InterestKind` enum is intentionally a small starter vocabulary. Every assertion can retain the original source interest-type code and its defining profile/code-system reference. Mapping an unmapped local right to generic `Other` requires both references. This preserves the identifier for loss-aware round trips; it does not claim that the system has interpreted the local legal meaning. A production jurisdiction profile must document any semantic transformation and keep unmapped terms unresolved rather than coercing them into a generic right.

## Absence, challenge searches and evidence frontiers

A zero-row search result is meaningful only relative to an exact, declared search frontier. The Sol Atlas contract distinguishes:
- `Unassessed`: no defensible assessment of challenge presence has been made;
- `NoChallengeObservedWithinFrontier`: no challenge was observed within an explicitly bounded scope, source set and evidence frontier with complete declared coverage;
- `Challenged`, `UnderReview` and `Resolved`: states tied to linked challenge records and their lifecycle.

A challenge-search assessment is not a proof that no challenge exists in the world. It is a bounded result for specified source systems, geographic/legal scope, time frontier and retrieval evidence. An unavailable, partial, stale or incomplete source cannot support `NoChallengeObservedWithinFrontier`. This composes with Mycelix EPI-008's EvidenceFrontier and CurrentnessAssessment semantics rather than introducing a second global completeness claim.

## Authority and evidence constraints

For v0.1 the validator enforces a narrow source/authority pairing:

| Declared basis | Required evidence | Required linked authority |
|---|---|---|
| `asserted` | At least one referenced evidence item | None implied |
| `community_attested` | Community decision evidence | Customary institution or community-governance body |
| `administrative_record` | Registry extract or permit | Registry or statutory regulator |
| `court_decision` | Court order | Court or tribunal |
| `contractual_instrument` | Contract | Contracting party |
| `statutory_instrument` | Statutory instrument | Statutory regulator |

These checks validate only the shape and reference pairing. They do not establish the authority's legal competence, signature authenticity, validity at the relevant time, territorial jurisdiction, procedural compliance or legitimacy of a community mandate. A reference to an instrument is not a legal conclusion about its effect.

## Spatial and temporal interoperability

### Geometry

Use OGC API Features-compatible feature identities and explicit CRS declarations. Use STAC for source-native satellite, aerial, LiDAR, point-cloud, DEM and derived raster asset metadata, and OGC API Records for discovery of evidence and dataset metadata.

Do not assume:
- a surface parcel polygon describes underground mineral space;
- coordinates in two different CRSs can be compared directly;
- a display map's pixels establish survey precision;
- a coarse land-cover product establishes a parcel boundary;
- two geometries overlap merely because their bounding boxes overlap.

The next spatial tranche should qualify coordinate transformations and topology against known-answer fixtures, including axis-order errors, datum shifts, precision loss, antimeridian handling and vertical datum mismatches.

### Time

The current `TemporalExtent` is a deliberately simple valid-time projection. It is not enough for source capture, publication, assertion, verification and transaction time to be conflated.

Reuse Mycelix EPI-008's typed temporal roles and evidence frontiers:
- source publication time;
- evidence capture/receipt time;
- observation time;
- assertion time;
- authority/instrument valid-from and valid-until;
- dispute filing and decision time;
- supersession/revocation time;
- analysis and verification time.

A future adapter should be able to answer "what was asserted, from which sources, under which authority and according to which evidence frontier at decision time T?" without rewriting history when later evidence arrives.

## South African profile boundary

A South African jurisdiction profile should separately map, at minimum:
- Deeds Registry and Surveyor-General data;
- communal/customary and informal land-right claims;
- MPRDA permissions and rights, including Project Indwe data where publicly and lawfully accessible;
- water-use authorisations and catchment/resource constraints;
- environmental authorisations and mine financial-provision/closure obligations;
- protected areas, biodiversity priorities and heritage restrictions;
- court and tribunal decisions, administrative appeals and unresolved challenges.

The profile must preserve the applicable law/instrument version and effective interval. It must be reviewed by appropriately qualified South African land, customary-law, mining, water, environmental and surveying practitioners. It must not hard-code a single national hierarchy into the generic LADM-aligned exchange model.

## Community data governance

Apply FAIR interoperability where it serves legitimate users, alongside CARE requirements for community-governed data: Collective Benefit, Authority to Control, Responsibility and Ethics.

In practical terms:
- no exact sacred-site or culturally sensitive coordinates in public exports by default;
- a community decision is not inferred from an individual testimony;
- representation and mandate validity remain externally governed;
- publication consent, measurement consent, site access and legal authority are distinct permissions;
- withdrawal/restriction changes create an auditable later state but do not claim that prior external copies have been erased;
- communities can retain records outside the platform and provide selective evidence references.

## Conformance work required before a claim of LADM support

1. Freeze a selected LADM edition, package scope and South African profile/version.
2. Map parties/groups, BAUnits, spatial units, RRRs, sources, administrative sources and versioned objects in a machine-readable schema.
3. Preserve multiplicity: multiple parties per RRR, multiple RRRs per spatial unit, and multiple spatial units/volumes per BAUnit.
4. Add formal mappings for external identifiers, legal instrument types and national codes without lossy coercion.
5. Add round-trip tests against an independently generated LADM/JSON or GML test set.
6. Demonstrate that disputed, unregistered, informal and customary claims remain representable without being promoted to registered land rights.
7. Demonstrate that sensitive community data cannot leak through API responses, logs, maps, search indexes or model prompts.
8. Obtain independent review by LADM/geospatial specialists and South African legal/surveying practitioners.

## Reference standards

- [ISO 19152-1:2024 — LADM generic conceptual model](https://www.iso.org/standard/81263.html)
- [ISO 19152-2:2025 — LADM land registration](https://www.iso.org/standard/81264.html)
- [OGC API Features](https://www.ogc.org/standards/ogcapi-features/)
- [OGC API Records](https://www.ogc.org/standards/ogcapi-records/)
- [STAC](https://www.ogc.org/standards/stac/)
- [FAO Voluntary Guidelines on the Responsible Governance of Tenure](https://www.fao.org/tenure/voluntary-guidelines/en/)
- [CARE Principles for Indigenous Data Governance](https://www.gida-global.org/careprinciples)
- [MPRDA, Act 28 of 2002](https://www.gov.za/ve/documents/mineral-and-petroleum-resources-development-act)

