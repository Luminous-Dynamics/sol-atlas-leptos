# Cultural and Anthropological Knowledge Graph Architecture

Sol Atlas can evolve from a civilizational/state-formation atlas into a broader **Cultural Systems Knowledge Graph (CSKG)**. The civilizational layer should remain a domain projection, not the ontology's ceiling.

## Design thesis

Culture is not a static collection of things belonging to a people. It is a temporally evolving system of practices, knowledge, institutions, places, languages, artifacts, technologies, relationships, memories, interpretations, and communities.

The graph should model:
- entities: people, communities, institutions, polities, places, artifacts, manuscripts, languages, traditions, techniques, concepts, ecological features;
- practices: rituals, ceremonies, crafts, foodways, governance practices, healing, agriculture, navigation, astronomy, education, music, storytelling;
- transmission: teaching, apprenticeship, migration, translation, copying, trade, observation, adaptation;
- transformation: reinterpretation, hybridization, innovation, loss, revival, suppression, standardization, localization;
- contexts: ecological, economic, political, technological, linguistic, religious, demographic and geographic contexts;
- claims and interpretations: source assertions, scholarly proposals, competing interpretations, unresolved questions;
- provenance: source snapshots, evidence, publication/capture/availability time, assessment and qualification;
- community recognition: when and by whom a living practice is recognized, maintained or transmitted.

UNESCO treats intangible cultural heritage as living, community-based, transmitted and continually recreated, including oral traditions, performing arts, social practices, knowledge concerning nature and the universe, and traditional craftsmanship. It also explicitly notes that these domains overlap rather than forming rigid external categories.

## Better abstraction than cultural DKG

The long-term architecture should be:

Civilizational Knowledge Graph -> Cultural Systems Graph -> state formation, institutions, law, language, religion, knowledge transmission, material culture, ecology, economy, migration and technology as interoperable projections.

This avoids making culture an essentialist container.

A community can participate in many traditions. A tradition can cross communities. A practice can change without becoming a different practice. A language can cross political borders. An institution can survive a polity. A polity can disappear while practices continue. A technology can be transmitted without political succession. Similarity does not by itself prove transmission.

## Core relation families

### Continuity
- institutional_continuity
- practice_continuity
- linguistic_continuity
- knowledge_continuity
- material_continuity

### Transmission
- taught
- apprenticed
- translated
- copied
- migrated_with
- traded
- observed
- institutionalized

### Transformation
- adapted_from
- reinterpreted_from
- hybridized_with
- localized_as
- standardized_as
- revived_from
- replaced_by
- suppressed_by

### Context
- practiced_at
- practiced_during
- associated_with
- dependent_on
- enabled_by
- constrained_by

### Social participation
- maintained_by
- transmitted_by
- practiced_by
- recognized_by
- contested_by

None of these relations should imply identity, ancestry, ownership, legitimacy or causal influence unless that stronger claim has its own evidence.

## Temporal model

The current StateSnapshotV1 / HistoricalTransitionV1 architecture should become one specialization of a more general temporal model:

EntityStateSnapshot + TransformationEvent + EvidenceFrontier + Claim/Interpretation + Provenance

This can represent a polity changing constitutional form, a ritual changing performance context, a language splitting or converging, a craft technique migrating, a manuscript being copied and translated, an institution being absorbed while its practices persist, or a knowledge tradition being revived after reduced transmission.

W3C PROV is a useful interoperability vocabulary for entities, activities, agents, derivations, responsibility and temporal provenance. Sol Atlas should use those concepts as an interoperability boundary without making PROV itself the domain ontology.

## Epistemic boundary

Source assertion != canonical claim != interpretation != established fact.

Provenance != truth.

Graph topology must never upgrade qualification. Similarity must never automatically become ancestry. Co-occurrence must never automatically become causation. Geographic proximity must never automatically become transmission. Community recognition must not be replaced by external classification.

## Community-sensitive design

Living cultural knowledge requires an additional dimension absent from a purely historical atlas: access and stewardship.

A future record may carry public, community-restricted, sacred/restricted, sensitive, embargoed, or unknown-access status.

The graph should support community-provided descriptions and recognition without turning external scholarship into the final authority over living heritage.

## DKG federation

The canonical pipeline remains:

External DKG assertion -> canonical claim -> evidence/source closure -> temporal availability -> qualification/assessment -> evidence frontier -> Sol Atlas projection -> reversible visualization.

Sol Atlas is therefore a projection/federation layer, not a replacement for Mycelix EPI's canonical evidence authority.

## First implemented vertical slice

The first executable cultural-systems contract is now the transmission slice in
sol-atlas-core/src/cultural_systems.rs. It adds:

- typed practice, tradition, community and transmission-event identifiers;
- explicit transmission modes rather than an inferred generic edge;
- CulturalEvidenceClosureV1 as the reusable claim/evidence/source/assessment/qualification/frontier boundary;
- CulturalTransmissionV1 with bounded event time and complete evidence/source closure;
- CulturalTransformationV1 with explicit transformation classes sharing the same closure boundary;
- CanonicalClaimAdmissionV1 as a projection-side admission context for an externally owned canonical claim;
- CommunityRecognitionV1 as a separate, evidence-backed participation/recognition dimension;
- AccessPolicyV1 as stewardship metadata independent of epistemic qualification;
- CulturalProjectionV1 as a renderer-neutral union of transmission and transformation projections;
- CulturalProjectionAdmissionV1 as an explicit renderer-facing admission object;
- CulturalProjectionAuditV1 as the reversible “why is this visible?” record, including community-recognition evidence and stewardship metadata.

The important boundary is now executable:

External canonical claim resolution -> reusable evidence closure -> temporal frontier -> cultural transmission/transformation projection.

A transmission cannot be admitted merely because two entities are similar,
nearby, co-occurring, or connected in a DKG. Community recognition and access
policy are carried forward without changing qualification. The projection audit
also makes community-recognition evidence inspectable without conflating it with
the evidence supporting the transmission claim itself.

## Architectural consequence

The current civilizational work should not be discarded. Instead, State Formation becomes one projection over Cultural Systems, which becomes one projection over Civilizational Knowledge, with shared temporal, provenance, qualification, identity and evidence contracts. The cultural layer now has a reusable transformation boundary rather than a transmission-only schema, so later language, knowledge, institutional and material-culture slices can compose the same admission machinery.

The result is a temporally versioned, evidence-reversible map of how human knowledge, practices, institutions and communities change and interact.

## Initial implementation boundary

First extract reusable contracts from the current civilizational layer:
1. EntityStateSnapshotV1
2. TransformationEventV1
3. ClaimRef
4. EvidenceClosure
5. SourceSnapshot
6. EvidenceFrontier
7. Qualification
8. CommunityRecognition
9. AccessPolicy
10. ProjectionAudit

Then implement small vertical slices: knowledge transmission; language evolution; institutional continuity; ritual/practice transmission; material/technological transmission.

Each slice must remain reversible to evidence and temporally bounded.

## Non-goals

The graph must not produce cultural purity scores, civilization rankings, ancestry rankings, true-culture determinations, automatic ownership claims, automatic territorial entitlement, deterministic cultural identity assignment, or autonomous adjudication of contested heritage.

The objective is to make cultural history more inspectable, plural, temporal and evidence-reversible, not to replace human or community interpretation.


## Interoperability research boundary

Recent standards review reinforces a useful separation of concerns.

- **CIDOC CRM** is event-centric and explicitly designed to integrate heterogeneous cultural-heritage information across museums, archives and libraries. Its current published site lists CRM 7.4 (August 2026), while CRMgeo 2.0 provides a spatiotemporal bridge toward GeoSPARQL.
- **CRMinf** is specifically concerned with argumentation and inference: premises, conclusions and reasoning activities. That makes it a useful conceptual analogue for Sol Atlas's distinction between canonical claims, assessments, interpretations and evidence paths.
- **W3C PROV** supplies a general provenance vocabulary around entities, activities, agents, derivations and responsibility. It should remain an interoperability layer rather than become the domain ontology for cultural systems.
- CIDOC CRM's own scope guidance warns against unconstrained ontology growth. This supports keeping Sol Atlas's Rust contracts deliberately small and projection-oriented, with richer domain semantics supplied by canonical DKG/claim authorities and external ontology mappings.

### Architectural consequence

We should **not** attempt to turn the cultural Rust module into a complete CIDOC CRM implementation.

Instead:

    Canonical DKG / Claim Authority
              ↓
    Sol Atlas cultural projection contracts
              ↓
    CIDOC CRM / CRMinf / PROV mappings
              ↓
    External archives, museums, research datasets

The Rust layer owns deterministic projection safety, temporal frontier replay, admission, access/stewardship metadata and reversibility. External semantic standards provide interoperability mappings and richer domain semantics.

This also suggests a future `OntologyMappingV1` boundary rather than embedding ontology-specific identifiers into every projection record.


## Interoperability contract: mappings are evidence-bearing assertions

The interoperability layer is intentionally separate from the cultural domain model.

`OntologyMappingV1` records a projection-side correspondence between a Sol Atlas identifier and an external semantic concept. It does not implement CIDOC CRM, CRMinf, CRMgeo, PROV, or any other external ontology.

Each mapping carries its own local type and identifier, external ontology namespace and concept identifier, mapping semantics (`Exact`, `Broad`, `Narrow`, `Related`, or `Contextual`), canonical claim reference, evidence and source-snapshot closure, qualification, and temporal evidence frontier.

> An ontology mapping is an assertion about semantic correspondence; it cannot upgrade the epistemic qualification of the mapped cultural claim.

An `Exact` mapping may mean that a local concept is asserted to correspond to an external ontology concept without implying that the underlying historical claim is established.

### Current interoperability targets

- **CIDOC CRM 7.4** is treated as the current research target, but its August 2026 release is currently listed as a draft. The implementation therefore references ontology identifiers without hard-coding the draft release into the projection schema.
- **CRMinf 1.2.1** is the natural future mapping target for explicit argumentation, premises, conclusions, and inference activities.
- **CRMgeo** is the future spatial/temporal semantic mapping target for historical geometry and temporal entities. Its current 2.0/2.0.1 work remains a draft/release-candidate track rather than something Sol Atlas should embed.
- **W3C PROV-O** is the provenance interoperability target. Sol Atlas retains its deterministic frontier and source-closure invariants locally and can project them into PROV rather than making PROV the admission mechanism.

The separation keeps the core deterministic and version-stable while allowing external ontology mappings to evolve independently.

## Argumentation boundary

The next semantic boundary after evidence closure is:

`Claim -> Assessment -> Interpretation/Hypothesis -> Evidence -> Source Snapshot`

rather than treating `qualification` as if it were the entire reasoning model.

Sol Atlas should continue to consume canonical claim/assessment/interpretation references owned by the external DKG authority. It should not create a second canonical historical adjudication system.

This gives us a future CRMinf-compatible projection without coupling the Rust projection contracts to one ontology's class hierarchy.

### Temporal argumentation invariant

Argumentation is now temporalized at the projection boundary as well. `CulturalArgumentationRefV1` distinguishes:

- `event_time`: when the historical event occurred (owned by the projection itself);
- `assessment_time`: when an assessment was performed;
- `interpretation_time`: when an interpretation was formulated;
- `available_by`: the earliest epistemic frontier at which that argumentation record may be used.

A 2026 interpretation therefore cannot silently appear in a 1920 replay merely because its underlying evidence is older. The projection requires the argumentation record to be available by the selected frontier, while the canonical assessment and interpretation remain externally owned.

This is deliberately aligned with CRMinf's treatment of argumentation and belief as temporally situated, while retaining Sol Atlas's deterministic frontier rule: later argumentation is excluded rather than used to retroactively upgrade an earlier historical view.


## Projection identity hardening

The cultural projection boundary now keeps transmission and transformation identities distinct at the type level. `CulturalProjectionIdV1` and `CulturalProjectionAdmissionV2` preserve the semantic variant instead of coercing a transformation identifier into a transmission-shaped field. `CulturalProjectionAuditV2` provides the same distinction for evidence-reversible “why shown?” records.

Ontology interoperability is represented separately by `OntologyMappingV1`. A mapping to CIDOC CRM, CRMinf, CRMgeo, or PROV-O is treated as an evidence-bearing projection assertion: it carries the canonical claim/evidence/source closure and frontier, but cannot upgrade qualification or become a second canonical claim authority.

This matches the current external standards boundary: CIDOC CRM 7.4 was published in August 2026 as a draft; CRMinf 1.2.1 provides an argumentation extension; CRMgeo 2.0.1 is a 2026 release candidate for spatiotemporal/GeoSPARQL integration. These standards should remain interoperability targets rather than being embedded wholesale into the Rust projection model.

Recent temporal-KG research also reinforces the architecture's separation of temporal context and uncertainty rather than treating a graph snapshot as timeless truth. This supports retaining explicit event/validity/evidence-frontier dimensions and keeping reasoning downstream of evidence closure.
