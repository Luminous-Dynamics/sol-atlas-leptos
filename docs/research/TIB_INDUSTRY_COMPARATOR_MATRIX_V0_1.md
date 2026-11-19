# TIB Industry Comparator Matrix v0.1

**Snapshot date:** 19 November 2026  
**Purpose:** evidence-led baseline selection for the Territorial Integrity Benchmark (TIB)  
**Status:** desk-research capability matrix; no hands-on comparative benchmark has been completed

## 1. How to read this matrix

This is not a product ranking. It distinguishes documented capabilities from measured outcomes and from features we have not independently tested.

- **Documented capability** means the provider or standards body publicly describes the capability.
- **Operational evidence** means a current deployment/pilot status is reported by a source; it is not an effectiveness evaluation.
- **Not measured** means TIB has not run a matched task against the comparator.
- **Not disclosed / not established** means the cited source does not establish a capability; it does not prove the product lacks it.

No system should lose points because a trial account, restricted interface or undocumented feature could not be accessed. Such cases are recorded as unavailable or not evaluable.

## 2. Comparator matrix

| Comparator | Documented strengths to respect | TIB test focus | Current evidence boundary |
|---|---|---|---|
| **FAO Open Tenure / Community Tenure Recording** | Participatory community-led recording; mobile field capture; offline use; multilingual forms; evidence attachments; community validation/moderation; disputes and challenges; offline/online synchronization; encrypted local database; Community Server aligned to LADM | Offline capture and recovery; field completion by user group; community moderation and challenge lifecycle; import/export fidelity; data sovereignty; low-bandwidth synchronization; accessibility and support cost | These are provider-described capabilities. TIB has not run matched field tasks, tested current releases or measured user outcomes. [FAO source](https://www.fao.org/tenure/resources/toolbox/sola/community-tenure-recording/en) |
| **STDM (UN-Habitat / GLTN)** | Pro-poor, participatory model for people-to-land relations independent of formality/legality/technical accuracy; formal, informal, customary, occupancy, common and overlapping relationships; party–social tenure relationship–spatial unit–supporting evidence structure; LADM specialization. The 2026 GLTN/FIG/UN-Habitat publication reports STDM as a normative annex in ISO 19152-2:2025 | Rights/relationship expressivity; customary and collective interests; overlapping claims; evidence linkage; local vocabulary preservation; migration to LADM profiles; field usability and governance | This is a core semantic/inclusion baseline, not merely a competitor database. TIB has not executed comparative conformance against STDM 1.8 or a deployed national profile. [GLTN 2026 publication](https://gltn.net/publications/fifteen-years-of-stdm-new-publication-captures-journey-and-road-ahead); [STDM overview](https://stdm.gltn.net/) |
| **Cadasta Foundation** | Publicly describes partner/community control of land and resource data; rights-based and community-led decisions; UNDRIP, FPIC and CARE principles; privacy, self-determination and data serving communities | Consent purpose and withdrawal; data access, export and transition; role/partner governance; sensitive-data disclosure; community review and correction; continuity after project funding ends | Treat community data governance as a strength to meet, not a feature to copy superficially. TIB has not audited Cadasta deployments or measured outcomes. [Cadasta data sovereignty and governance](https://cadasta.org/data-sovereignty-governance/) |
| **FAO SOLA / land-administration toolchain** | Open-source land-administration components, including community-tenure recording and registry workflows; implementation guidance and fit-for-purpose orientation | Mapping fidelity to national workflows; administrative transaction support; interoperability between community records and formal systems; cost and deployment capacity | TIB has not completed a live SOLA implementation or migration trial. [FAO SOLA](https://www.fao.org/tenure/resources/toolbox/sola/en) |
| **South Africa Project Indwe** | Digital mining-cadastre pilot operating in Western Cape; BLSA tracker reports 37 right-holders registered; phased rollout intended to address application backlog; geospatial grid and environmental restrictions described in public reporting | External register source status; currentness and history migration; application/appeal workflow; authority and scope verification; public API/official receipt availability; environmental and overlapping-interest context | Operational evidence, not an impact evaluation. BLSA's tracker is dated 1 October 2026 and notes no progress update for July–September 2026. Do not infer API availability or national coverage. [BLSA tracker](https://tracker.blsa.org.za/economic/?reform=Mining+cadastral+system); [June 2026 rollout report](https://iol.co.za/business-report/2026-06-16-new-mining-cadastre-system-nears-national-rollout-as-government-prioritises-data-accuracy/) |
| **ISO LADM 19152-1:2024 / 19152-2:2025** | Standardized concepts for parties, basic administrative units, rights, responsibilities, restrictions, spatial units, sources and land registration, including spaces above and below the surface; STDM normative annex in Part 2 | Profile conformance; semantic preservation; unmapped-field reporting; versioned-source compatibility; vertical and temporal extent handling | Standards are models, not deployed adjudicators or proof of legal effect. LADM conformance does not establish whether a particular issuer is competent or whether an assertion is true. [Part 1](https://www.iso.org/standard/81263.html); [Part 2](https://www.iso.org/standard/81264.html) |
| **OGC API Features / JSON-FG / STAC** | Standardized feature discovery/encoding and spatiotemporal asset metadata. JSON-FG adds CRS, temporal feature characteristics, conformance declarations and optional solids/prisms/curves/schema typing | CRS/axis-order/vertical-datum round trips; feature identity; geometry loss reporting; time semantics; source asset provenance; schema negotiation | Interchange standards do not establish legal boundaries, survey precision, land rights or evidence authenticity. [JSON-FG](https://www.ogc.org/standards/json-fg/); [OGC API Features](https://www.ogc.org/standards/ogcapi-features/); [STAC](https://www.ogc.org/standards/stac/) |
| **Sol Atlas + Mycelix (current research branch)** | Synthetic structural corpus; reference-only LADM RRR adapter; bounded challenge/frontier semantics; source-linked authority references; decision packet; fail-closed operation lifecycle model and transition vectors | Exact-head tests; cross-system loss audit; authority lifecycle; replay/timeout safety; late challenges/revocations; offline sync; privacy and community review; independent legal/geospatial/user studies | Implementation and synthetic test work only. Not a national registry, live Property host gate, community pilot, independent legal assessment or demonstrated industry-beating system. |

## 3. What the industry is telling us

### A. Inclusion is already a mature design concern

STDM explicitly models diverse people-to-land relationships and supporting evidence. Open Tenure already supports offline field capture, multilingual workflows and community moderation. Cadasta centers community data control and rights-based governance. A new architecture that only adds a cleaner database schema or AI-generated map would not be a meaningful advance.

### B. National systems are heterogeneous and institution-constrained

A 2026 *World Development* study benchmarked property institutions in 85 jurisdictions, including digital coverage, interoperability and public land-management indicators. Its 15-observation Eastern and Southern Sub-Saharan Africa sample includes South Africa but is **not a South Africa-specific estimate**. In that regional sample, reported API access was 6.7% for registries and 13.3% for cadastres. The study also records gaps in digital seller-right verification, court-case visibility and mortgage-data access. This supports a benchmark for API-unavailable workflows, assisted processes and evidence frontiers—not an API-only architecture.

Source: [Deininger, Hilhorst, Nkurunziza and Zevenbergen, *Benchmarking property institutions*, World Development 202 (June 2026), 107354](https://doi.org/10.1016/j.worlddev.2026.107354).

### C. FELA is broader than technology

The UN Framework for Effective Land Administration has nine pathways across governance, policy/legal, finance, data, innovation, standards, partnerships, capacity/education and advocacy/awareness. TIB must include inclusion, institutional ownership, cost, user capability, recourse and communication. A technically correct exchange that excludes affected people is a failed result.

Source: [UN-GGIM 2026 FELA materials](https://www.un.org/globalgeospatial/sites/default/files/2026-06/fela.pdf).

## 4. Matched evaluation protocol

For each comparator, use the same de-identified synthetic or independently authorized input bundle, task definitions, user roles and scoring rubric.

1. Freeze the source bundle, schema versions, software releases and field vocabulary.
2. Ask independent land-administration, legal, surveying/geospatial, privacy and community-governance reviewers to label expected results before running systems.
3. Run matched user tasks: record a customary/collective interest offline; import a formal registry assertion; add a challenge; process a late authority revocation; represent a subsurface volume; withdraw publication permission; reconcile a timed-out external operation; and project a stewardship obligation only after independent evidence.
4. Record native capability, configuration effort, adapter loss, manual intervention, unavailable source, training and support requirements.
5. Publish per-task results, failures, dissent and uncertainty. Do not collapse them into one “trust” or “accuracy” score.
6. Separate synthetic conformance, hands-on product evaluation, independent expert review and field-user research. They are four different evidence classes.

## 5. Proposed TIB scorecard

| Dimension | Measures | Initial target |
|---|---|---:|
| Rights inclusion | Recall of independently labelled affected parties and interests; omissions by rights type | ≥99% recall on a closed expert-labelled corpus; report every omission |
| Semantic fidelity | Source-code/profile retention; RRR and temporal/vertical field round trips | 100% retention of required fields or explicit, reviewer-visible loss |
| Authority safety | False acceptance of wrong-scope, revoked, stale or unverified authority | 0 critical false accepts |
| Dispute continuity | Challenge/appeal preservation through import, export, replay and projection | 100% preservation; no silent resolution |
| Transaction safety | Mutation after unknown outcome; duplicate retry before reconciliation | 0 |
| Spatial qualification | CRS, axis-order, vertical datum and uncertainty error by geometry class | 0 silent coordinate reinterpretations; publish class-specific error |
| Community control | Consent purpose, withdrawal, correction, export and restricted disclosure | 0 unauthorized critical disclosures; test with participating communities |
| Offline resilience | Sync conflict and recovery under outage, clock skew and concurrent updates | 0 silent overwrite; every conflict resolved or held |
| Stewardship evidence | False completion without independent, baseline-linked evidence | 0 |
| Operations and inclusion | Completion time, support effort, bandwidth, field-device cost, training, accessibility | Establish baselines; report by user group, not only mean |

These are proposed thresholds, not achieved results or universal legal standards.

## 6. The defensible differentiation hypothesis

Sol Atlas/Mycelix should not claim to replace STDM's social-tenure model, Open Tenure's community workflows, Cadasta's data-governance practice, ISO LADM, OGC geometry standards or a national cadastre.

The testable differentiation is a **federated evidence and transaction-assurance layer** that can preserve the meaning and provenance of rights assertions across those systems:

- plural rights, responsibilities and restrictions rather than one owner field;
- exact evidence/source versions and jurisdiction-specific vocabularies;
- bitemporal authority, dispute, correction and revocation histories;
- bounded “not observed” claims rather than universal absence;
- 2D/3D/4D spatial and temporal uncertainty;
- read-only, scope-limited status projection from qualified external receipts;
- retry/reconciliation safety when external outcomes are unknown;
- purpose-specific consent, restricted knowledge and community-controlled review;
- stewardship commitments connected to independent outcome observations.

This advantage is not demonstrated until matched experiments show lower information loss, fewer unsafe false promotions, equal or better inclusion, and lower recovery burden without weakening community control.

