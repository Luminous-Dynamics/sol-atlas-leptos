# CTER Benchmark and Industry Comparison Programme

**Tracking:** [CTER-BENCHMARK-001 #955](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/955)  
**Research date:** 2026-11-20  
**Status:** comparative research and benchmark targets; no vendor performance result or field outcome established.

## 1. Industry position: start by not claiming what already exists

### FAO Open Tenure

Open Tenure already supports open-source offline mobile collection, encrypted local storage, multilingual forms, participatory validation, dispute/challenge workflows, evidence attachments, secure import/export and synchronization with its Community Server. FAO describes a community flow from information sharing and collection through public review, moderation and community validation. Its Community Server is LADM-aligned. Offline capture, participatory mapping, multilingual data entry, and dispute recording are therefore not CTER's novelty claim.

Reference: [FAO Community Tenure Recording / Open Tenure](https://www.fao.org/tenure/resources/toolbox/sola/community-tenure-recording/en).

### FAO SOLA

The SOLA suite provides free/open-source fit-for-purpose land-administration tools for land registration, cadastre and community tenure recording. FAO describes SOLA as LADM-aligned and locally adaptable. CTER should interoperate with existing SOLA installations and their national/community profiles, not create a competing land registry.

References: [FAO SOLA suite](https://www.fao.org/tenure/resources/toolbox/sola/en), [SOLA Land Administration](https://www.fao.org/tenure/resources/toolbox/sola/sola-land-administration/en).

### Cadasta

Cadasta's technology suite combines offline-ready geospatial capture, secure storage, role-based sharing, dashboards and integrations with government and other third-party systems. Its 2025 reflection stresses making community-generated data institutionally actionable without losing integrity, ownership or context. This overlaps strongly with the deployment, accessibility and data-governance goals CTER must respect.

References: [Cadasta technology](https://cadasta.org/technology/), [Cadasta 2025 reflection](https://cadasta.org/2025-annual-report/page-3/), [Cadasta field lessons, 2026-02-19](https://cadasta.org/what-we-learned-from-supporting-the-mapping-of-millions-of-hectares-lessons-from-indonesia-myanmar-and-india/).

### Standards and boundaries

ISO 19152-1:2024 supplies a generic conceptual model for parties, administrative units, rights/responsibilities/restrictions, spatial units and sources; ISO 19152-2:2025 extends the model for land registration, including above- and below-surface legal spaces. Neither standard decides national law or substitutes for the systems and authorities that maintain source records.

References: [ISO 19152-1:2024](https://www.iso.org/standard/81263.html), [ISO 19152-2:2025](https://committee.iso.org/cms/live/live/en/sites/isoorg/contents/data/standard/08/12/81264.html).

IETF RFC 9943 defines SCITT's transparency architecture and requires an append-only, non-equivocating and replayable verifiable data structure; SCITT Transparency Services issue COSE receipts under explicit registration policies. Mycelix owns the receipt/TS boundary in this programme; CTER consumes qualified receipts/checkpoints and must not fork a second transparency ledger. A registration receipt is not proof that a real-world claim is true or legally effective.

Reference: [RFC 9943](https://www.rfc-editor.org/rfc/rfc9943.html).

## 2. The CTER differentiation hypothesis

The research hypothesis is that a multi-owner territorial evidence package can preserve the **relationships between evidence, authority, decisions, objections, restrictions and custody** through offline partitions, organization changes, provider exit and long-term cryptographic renewal.

The target is not another map, cadastre, title database, signature service or global confidence score. It is a portable and independently verifiable evidence relationship containing:
- the exact source and version commitments;
- distinct claim bases and issuing/authority sources;
- event, correction, supersession, objection and challenge lineage;
- consent, purpose, recipient, access class and disclosure mandate;
- typed receipt references and currentness/revocation context;
- archive frontier, withheld/unavailable state, key custody and restore proof.

This is a hypothesis, not a demonstrated product advantage.

## 3. Comparative benchmark design

| Family | What to compare | Primary metrics |
|---|---|---|
| LADM interoperability | Open Tenure/SOLA/Cadasta adapter profiles vs CTER mappings using the same fixtures | semantic round-trip loss, unmapped terms, preserved RRR/source/party/time references |
| Offline evidence convergence | same event corpus, partition schedules and node topology | exact convergence, silent loss, conflict recall, false conflict rate, time-to-reconcile |
| Authority and consent | same operation/purpose/recipient/data-class adversarial cases | false ALLOW count, scope-mismatch recall, type-laundering detection, UNKNOWN fidelity |
| Archive/custody exit | restore same declared archive under provider outage and independent implementation | complete-frontier restore rate, missing-object detection, semantic restore, key independence |
| Privacy and data sovereignty | attack same outputs and channels using synthetic sensitive-location fixtures | unauthorized disclosures, inference/re-identification risk, withdrawal propagation |
| Field usability | compare the same authorized task with Open Tenure/Cadasta workflows where feasible | completion rate, task time, user errors, training time, language coverage, battery/network/storage |
| Long-term verification | same historical evidence under key rotation, algorithm deprecation and trust-anchor change | historical verification coverage, renewal completeness, false-currentness rate, migration loss |

Do not collapse these into one weighted score. Report the safety-critical failure rates separately from usability and performance.

## 4. Benchmark protocol

1. Freeze an open synthetic corpus with schema versions, deterministic seeds and expected reason codes.
2. Add a hidden held-out suite for independent red teams; never include real claimant identities, sacred-site coordinates or protected community knowledge in public fixtures.
3. Run matched implementations on identical data distributions, device classes, network traces, fault schedules and machine/toolchain versions.
4. Publish executable scripts, raw logs, corpus digest, tool versions, hardware details, and all non-passing or excluded runs.
5. Separate structural evaluator results, cryptographic verification, authority resolution, protocol integration, physical/network experiments, field studies and legal outcomes.
6. Re-run independent restores with no source-provider credentials and require byte-level plus semantic verification.
7. Obtain community-specific data agreements, consent and review before any real community dataset is used.

## 5. Initial acceptance targets

Targets only; not achieved results:

| Property | Target |
|---|---:|
| Negative controls resulting in eligible | 0 |
| Receipt-type and exact scope/purpose/recipient mismatch detection | 100% |
| Stale/unknown/unavailable authority promoted to positive | 0 |
| Exact duplicate replay idempotence | 100% |
| Same receipt ID with divergent contents accepted | 0 |
| Eventual convergence after all valid events are delivered | 100% |
| Silent loss of declared or withheld archive objects | 0 |
| Independent restore of a declared complete frontier | 100% |
| Unauthorized disclosure across tested channels | 0 |
| Cross-language frozen-fixture agreement | 100% |

### Scale and performance

Use 1k / 10k / 100k / 1M event corpora and publish p50/p95/p99 latency, throughput, memory high-water mark, CPU, storage growth, recovery time and energy where physically measured. Pin event fan-out, conflict density, chain depth, payload sizes, node count, partition schedule, hardware and toolchain.

Do not set an absolute performance target until a reproducible baseline has been run on the same reference hardware. Compare against current Open Tenure/SOLA/Cadasta workflow baselines only where the same task and deployment envelope can be reproduced; otherwise report the comparison as qualitative.

## 6. Paradigm shifts to pursue

### Evidence relationships, not document piles

A document hash is not enough. Build a typed graph that makes the path from source to assertion, decision, objection, correction and current projection traversable and independently checkable.

### Authorization as a scoped, expiring capability

Permission is a tuple over principal, operation, subject/resource, purpose, recipient, jurisdiction, data classes and time. No generic `verified=true`, identity credential, transport receipt or consent for another purpose can satisfy it. For offline operation, a bounded lease makes staleness explicit rather than pretending disconnected nodes know revocation state.

### Contestability as a first-class state

Dissent is neither silently discarded nor universally converted into a veto. The governing community's documented procedure, representation assessment, gaps, appeal route and challenge status remain visible. Unresolved material challenges pause affected operations; technical systems do not invent one global majority rule.

### Custody exit as a tested operation

Independence is not a policy statement. Test cold restoration under provider loss, missing keys, partial archives, changed schemas and cryptographic migration. The restoring party must verify the same declared frontier without the original provider's credentials.

### Fail-closed without erasing rights

`UNKNOWN` and `INCOMPLETE` block new disclosure/mutation while preserving the claim that evidence may exist. Missing receipts or resolver outages are not proof that no right or consent exists.

## 7. Work ownership and claim limits

- Sol-Atlas CTER owns portable profile, consumer adapter, corpus, relay convergence and archive-exit qualification.
- Mycelix owns identity, issuer and authority resolution, consent, typed AuthorizationReceipts, currentness/revocation and SCITT transparency services.
- Symtropy may generate controlled network/physical traces; model outputs do not confer rights or establish field outcomes.
- Symthaea may perform read-only evidence explanation, anomaly discovery and counterexample generation; it cannot issue permissions or adjudicate rights.
- Independent partners and affected communities decide whether real deployments, governance procedures and data-use arrangements are appropriate.

A benchmark pass is not statutory registration, legal effect, community legitimacy, scientific truth or consent. No real-world pilot, partner endorsement or production authorization is implied.
