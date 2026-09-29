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

## Architectural consequence

The current civilizational work should not be discarded. Instead, State Formation becomes one projection over Cultural Systems, which becomes one projection over Civilizational Knowledge, with shared temporal, provenance, qualification, identity and evidence contracts.

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
