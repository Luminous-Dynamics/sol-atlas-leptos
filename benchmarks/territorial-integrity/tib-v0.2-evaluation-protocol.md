# TIB v0.2 — Adversarial Evaluation Protocol

**Suite:** `TIB-SYNTH-STRUCTURAL-001`  
**Version:** 0.2.0  
**Status:** synthetic contract benchmark; not a field benchmark or legal certification  
**Manifest:** [tib-v0.1-manifest.json](./tib-v0.1-manifest.json) (suite metadata is versioned in-place)  
**Frozen inputs:** 29 JSON fixtures pinned by Git blob identity

## 1. What this benchmark measures

TIB is intended to measure whether a land/territorial information system preserves the evidence and limits attached to a claim while moving across data models and workflows. It is not a competition to infer legal title from incomplete data.

The v0.2 corpus contains **29 synthetic fixtures: 8 expected structural accepts and 21 expected structural rejects**. It adds tested cases for:
- vertical/subsurface extent with an explicit vertical datum;
- terminal challenge withdrawal and consistent resolved state;
- historical jurisdictional date intervals;
- stewardship completion tied to upstream governance, quantitative units, independent verification and evidence;
- negative or incomplete spatial extents;
- missing evidence, duplicate evidence IDs and incomplete jurisdictional vocabulary mappings;
- contractual-basis evidence gaps;
- inconsistent challenge lifecycle and stewardship self-verification.

The corpus contains no real community records, personal data, sacred-site locations, real parcel geometry or actual title determinations. The expected disposition is only whether a record satisfies the current structural contract.

## 2. Paradigm: preserve the claim, qualify the evidence, refuse unsupported closure

Treat territorial records as a versioned, temporal, multi-party evidence graph rather than a single parcel row or owner field.

The architecture separates:
1. the party or group reference;
2. the asserted right, responsibility or restriction;
3. its legal, customary, contractual or administrative basis;
4. the source evidence and source version;
5. the authority and its scoped reference;
6. the spatial and temporal extent, including uncertainty and vertical datum;
7. active challenges, decisions and evidence frontiers;
8. governance-owned commitments;
9. independent observations and completion verification.

No accumulation of signatures, reputation scores, DHT consensus, model confidence or votes may promote an assertion to legal registration. An absence claim remains bounded by its declared sources and evidence frontier. An incomplete or stale record is not converted into a positive conclusion merely to keep a workflow moving.

## 3. Four outcome classes for future cross-system evaluation

The current Rust v0.1 validator emits structural accept/reject only. A future cross-system benchmark adapter should preserve four distinct outcomes:

| Outcome | Meaning | Permitted consequence |
|---|---|---|
| `STRUCTURAL_ACCEPT` | The payload is internally well-formed and references are consistent | It may enter the next qualification stage; no legal or factual promotion |
| `STRUCTURAL_REJECT` | A deterministic contract invariant is violated | Reject the payload and identify the violated invariant |
| `REVIEW_REQUIRED` | The format is valid but a competent human or external authority must resolve a material question | Hold any dependent decision; preserve the unresolved issue |
| `NOT_EVALUABLE` | Required source, permission, geometry, temporal or ground-truth data is unavailable | Report the coverage gap; do not impute a pass |

Do not implement these higher-order outcomes by pretending the current boolean validator already supports them. They are the proposed benchmark adapter contract. The machine-readable envelope is [tib-evaluation-result-v0.schema.json](./tib-evaluation-result-v0.schema.json); [this synthetic expected-case annotation](./examples/tib-review-required-expected-case-v0.json) demonstrates a critical authority-currentness gap without pretending a system ran or issued a legal conclusion.

## 4. Evaluation tracks and metrics

| Track | Metric | Initial target (proposed, not measured) |
|---|---|---|
| Structural integrity | False accepts / false rejects on frozen fixtures | 0 false accepts and 0 false rejects on the labelled v0.2 corpus |
| Claim-status integrity | Unauthorized status promotions | 0 across all round trips and replay |
| Provenance | Required source-link resolution and exact source-version retention | 100% for in-scope records |
| Lossless interoperability | Source-code/profile, relationship, basis, party, and temporal-field round-trip retention | 100% for fields declared in the exchange profile |
| Challenge lifecycle | Retention of submitted, under-review, rejected, withdrawn and resolved states | 100% state preservation; no active challenge silently dropped |
| Spatial integrity | CRS-aware overlap and 3D volume conflict detection vs independent reference geometry | Report per geometry/CRS class; target >=99% recall on synthetic in-scope cases after independent ground truth is authored |
| Spatial uncertainty | Calibration and error by source class, scale, CRS and survey method | No universal metre threshold; require source accuracy and uncertainty to be displayed |
| Temporal integrity | Expiry, effective time, capture time, supersession and revocation correctness | 100% of deterministic edge cases |
| Community governance | Unauthorized disclosure, consent purpose mismatch, revocation and representation review | 0 critical disclosure or authorization violations in adversarial tests |
| Stewardship | Invalid closure rate; observation-to-obligation traceability | 0 invalid verified-complete closures |
| Operational inclusion | Task completion, correction rate, time, offline recovery and language accessibility | Establish with participating users; no synthetic pass threshold |
| Operations | P50/P95 latency, throughput, storage, bandwidth, recovery point and recovery time | Measure against identical hardware, data and service boundaries |

Report precision, recall, false-accept rate, false-reject rate, severity-weighted omission, calibration error, geometric error, temporal consistency, latency and cost separately. A single trust, sustainability or rights-security score is prohibited: it hides incomparable uncertainties and trade-offs.

## 5. Mandatory adversarial expansion

The next suites must exercise conditions not covered by this structural validator:
- expired, suspended, scoped or revoked authority credentials;
- source documents whose digest matches but whose issuer, legal competence or jurisdiction does not;
- stale registry snapshots, late-arriving decisions, supersession and retroactive corrections;
- same-geometry claims with different vertical extents, CRS transformations, survey tolerances and boundary uncertainty;
- omitted households, overlapping customary claims and representative-mandate disputes;
- evidence replay, clock skew, event reordering and split-brain reconciliation;
- a valid but non-public source referenced from a public export;
- environmental measurement bias, sensor drift, missing baseline and unsupported causal inference;
- satellite change detection incorrectly described as proof of illegal occupation;
- self-reported restoration incorrectly described as independently verified ecological recovery.

A data-format validator cannot settle these questions. The expected result must be `REVIEW_REQUIRED` or `NOT_EVALUABLE` until the required source or competent reviewer is available.

## 6. Fair industry comparison

Compare against existing products and workflows by capability, not by branding or one headline score.

- **FAO SOLA Suite** already combines formal land-administration tools with participatory community tenure recording and LADM/STDM alignment. It is a baseline and potential integration partner, not a feature to reimplement.
- **FAO Open Tenure** supports participatory field collection, mobile mapping and community-led recording. Benchmark the complementary assurance and exchange layer; do not claim superior community participation without user studies.
- **STDM and Cadasta** are relevant comparators for social-tenure and community-data workflows. Use equivalent data, permissions and user tasks.
- **Project Indwe** is a live South African mining-cadastre programme. The BLSA tracker recorded the Western Cape pilot and 37 registered right-holders, while reporting no progress update for July–September 2026 and a phased rollout target through July 2027 (tracker last updated 1 October 2026). This is too early to claim comparative operational performance; use only public, permissioned interfaces and disclosed capabilities.
- **ISO 19152-1:2024 / ISO 19152-2:2025** provide the LADM conceptual/land-registration model. Interoperability must be tested against the applicable parts and national profiles; a mapping document is not conformance.
- **OGC API Features, OGC API Records and STAC** are interoperability surfaces for features, resource metadata and spatiotemporal assets. They do not, by themselves, establish legal authority, data permission or evidentiary sufficiency.

### Matched protocol

1. Publish one frozen, synthetic input bundle and versioned mapping instructions.
2. Give every comparator the same rights, authorities, evidence, challenge, geometry, uncertainty and temporal facts.
3. Record native capability, adapter loss, manual work, unavailable features and every field dropped or transformed.
4. Have independent land-administration, legal, surveying/geospatial and community-governance reviewers label the expected result before running systems.
5. Blind-score outputs for omitted rights-holders, false authority promotion, dispute loss, privacy leakage and provenance correctness.
6. Publish all failures, configuration, software version, source snapshot and cost boundary. Do not infer competitor weakness from a feature not exposed by a trial or public documentation.

## 7. Known limitations and release gates

The current Sol Atlas validator checks record shape and selected internal references. It does not authenticate documents, verify signatures or issuer competence, resolve legal effect, validate CRS transformations, survey geometry, detect real-world overlap, prove consent, establish ecological recovery or guarantee completeness of a source frontier.

The 29 cases are a deterministic synthetic structural corpus, not 29 benchmark wins. The Rust tests must run on the exact head; then an independent implementation must reproduce the fixture dispositions. Field studies require willing communities, agreed data governance, South African legal and surveying expertise, environmental-science review, and independently reviewable challenge/appeal processes.

## 8. Research and standards references

- [ISO 19152-1:2024 — LADM generic conceptual model](https://www.iso.org/standard/81263.html)
- [ISO 19152-2:2025 — LADM land registration](https://www.iso.org/standard/81264.html)
- [FAO SOLA Suite](https://www.fao.org/tenure/resources/toolbox/sola/en)
- [FAO Community Tenure Recording / Open Tenure](https://www.fao.org/tenure/resources/toolbox/sola/community-tenure-recording/en)
- [FAO Voluntary Guidelines on the Responsible Governance of Tenure](https://www.fao.org/land-water/land/land-governance-and-planning/land-resources-planning-toolbox/detail/voluntary-guidelines-on-responsible-governance-of-tenure-of-land-fisheries-and-forests-in-the-context-of-national-food-security-%28vggt%29/en)
- [OGC API Features](https://www.ogc.org/standards/ogcapi-features/)
- [OGC API Records](https://ogcapi.ogc.org/records/)
- [OGC STAC](https://www.ogc.org/standards/stac/)
- [BLSA Mining Cadastre Reform Tracker — Project Indwe](https://tracker.blsa.org.za/economic/?reform=Mining+cadastral+system)
