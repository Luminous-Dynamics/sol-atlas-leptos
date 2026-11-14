# Pluralistic Cultural Knowledge Provenance Protocol — HMP-001 v0.1

**Status:** Research and cataloguing protocol; no new cultural corpus or community authorization is claimed.  
**Date:** 2026-11-14  
**Parent:** [Symthaea HMP-001](https://github.com/Luminous-Dynamics/symthaea/blob/research/human-meaning-pluralism-v0-1/docs/research/HUMAN_MEANING_PLURALISM_V0_1.md)

## 1. Purpose

Sol-Atlas can contribute a historically, geographically and linguistically situated knowledge atlas: not a universal encyclopedia that claims to contain every religion or culture, but a transparent map of sources, claims, practices, interpretations, gaps and permissions.

Religions, spiritual traditions, philosophies and cultural practices are living, internally diverse and continuously changing. There is no finite definitive list. A catalog is always partial and must display its coverage limits.

## 2. Unit of cataloguing: the situated claim

Use the shared [Pluralistic Knowledge Record v0.1 schema](https://github.com/Luminous-Dynamics/symthaea/blob/research/human-meaning-pluralism-v0-1/schemas/pluralistic-knowledge-record-v0.1.0.schema.json).

Do not make “religion”, “culture” or “community” the smallest unit of truth. A record should represent a particular statement, source or interpretation, attributed to a defined voice and scope. Preserve:

- original wording and language where authorized;
- translation, transliteration and unresolved semantic distinctions;
- geographic and temporal context;
- whether the source is a community self-description, participant testimony, practitioner interpretation, scholarship, archival material or external observation;
- whether a statement describes doctrine, lived practice, ritual, historical event, metaphysical belief, ethical norm or personal experience;
- internal disagreements and corrections;
- source rights, permitted operations and community review status.

A scholar's account is not a community's self-description. One person's testimony is not a universal statement about a religion. An archive's silence is not evidence that a practice never existed.

## 3. Coverage without false completeness

Build an open-ended index that can represent:

- religious and spiritual traditions, denominations, schools and movements;
- nonreligious worldviews and secular ethical traditions;
- indigenous, local, diasporic and syncretic practices;
- philosophical traditions and oral knowledge;
- historical, contemporary and emergent practices;
- people who identify with a tradition, people who interpret it differently, and people who have left or reject it.

Do not set a target number of religions or countries as a proxy for representativeness. Track coverage by language, locality, time period, source type, author/voice, tradition/subtradition, access class and review status. Show unknowns and gaps visibly.

## 4. Rights-aware ingestion

Follow CARE-oriented governance and the Mycelix contract. Public availability does not automatically authorize training, translation, adaptation or redistribution.

- Do not scrape or reproduce sacred, initiatory, restricted or community-controlled material without explicit authority.
- Allow metadata-only placeholders where the existence of a source can be named but its contents must not be copied.
- Record whether consent covers display, indexing, retrieval, translation, model inference, training and redistribution independently.
- Store precise locations and personal identifiers only when necessary and explicitly authorized; use generalized geography for sensitive sites.
- Preserve oral knowledge under the contributor/community's chosen attribution, anonymity, access and correction terms.
- Offer correction, contextualization and takedown routes; show the review state rather than silently overwriting contested history.

A source digest provides integrity evidence, not permission, authenticity or truth. A public map is not automatically safe for precise coordinates.

## 5. Search and presentation behavior

When a user asks about a tradition or practice, the system should:

1. ask or infer only the minimum needed context; never assume the user's identity or beliefs;
2. distinguish established source content from interpretation, contested claims and unknowns;
3. present internal diversity proportionately rather than selecting a single “official” voice without justification;
4. identify the time and place to which a practice description applies;
5. avoid presenting an external scholarly taxonomy as the community's own categories;
6. say when a source is restricted, missing, inaccessible or not authorized for this use;
7. offer a correction channel and preserve the original attribution.

Do not transform this atlas into a conversion, persuasion, profiling or belief-inference system.

## 6. Proposed quality measures

| Measure | What to audit |
|---|---|
| Attribution coverage | Share of claims with a named source/voice type and bounded scope |
| Temporal/geographic precision | Claims with explicit time/place ranges and no false present-day generalization |
| Language integrity | Original-language availability, translator identity/qualification where appropriate, ambiguity notes |
| Perspective diversity | Distinct, legitimately sourced interpretations without tokenizing one voice as representative |
| Permission completeness | Operation-specific permissions and authority references |
| Correction responsiveness | Time to display corrections and propagate retractions |
| Coverage honesty | Missing regions, languages, time periods and communities surfaced rather than concealed |
| Sensitive-location safety | No unauthorized precise coordinates or exposed protected sites |

Do not score traditions by popularity, number of documents, or degree of conformity to an external ontology.

## 7. Pilot plan

1. Start with public, clearly licensed academic and archival sources; do not begin with restricted oral or ceremonial knowledge.
2. Select a small, deliberately diverse pilot with source and language coverage recorded. Include African philosophy and religious history as research domains without assuming a single African worldview.
3. Have qualified subject experts review historical and scholarly claims; invite community review where self-description or community-held knowledge is involved.
4. Use synthetic examples for schema tests until rights and review are complete.
5. Publish a data card with scope, exclusions, rights, provenance, review, known bias and correction procedures before any corpus is described as representative.

## 8. Claim boundary

This protocol defines requirements and quality measures. It does not claim that Sol-Atlas currently contains comprehensive religious/cultural knowledge, that a community has authorized collection, or that its present interface enforces these requirements.


## 9. Interaction and plurality-integrity handoff

Sol-Atlas provenance is an input to the system-level [Plurality Integrity under Interaction (PIB)](https://github.com/Luminous-Dynamics/symthaea/blob/research/human-meaning-pluralism-v0-1/docs/research/PLURALITY_INTEGRITY_BENCHMARK_V0_1.md) benchmark. The benchmark asks whether a multi-agent system preserves situated perspectives through interaction, and distinguishes evidence-supported convergence from false consensus, status pressure and dissent erasure.

For any future Sol-Atlas record supplied to a PIB episode, the evidence handoff must retain:
- source/claim ID and exact revision or digest;
- speaker/voice type and the limited scope that attribution supports;
- language, translation status and unresolved semantic mappings;
- temporal/geographic scope and the time of the source;
- rights/permission decision for each requested operation;
- corrections, supersession and unresolved disagreement;
- whether a perspective is a direct source account, an interpretation, or an analyst-created summary.

The atlas must not infer that two records describe the same cultural category just because they share a country, language, broad tradition label or translated word. "Not comparable" and "scope unknown" are valid values. A majority, model-generated synthesis or simulated agent vote cannot overwrite a source record or turn its attributed statement into a community-wide position.

**Status:** interface and provenance requirements only. No PIB corpus, human interaction data, Sol-Atlas-to-Symtropy export adapter or empirical validation is implemented.
