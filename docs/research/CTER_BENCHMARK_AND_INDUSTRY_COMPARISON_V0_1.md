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


## 8. Follow-on benchmark: authorization-to-egress binding

A new workstream, [CTER-EGRESS-BINDING-001 #960](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/960), extends the benchmark past request-side authorization. Its research profile is [CTER_EGRESS_BINDING_PROFILE_V0_1.md](CTER_EGRESS_BINDING_PROFILE_V0_1.md); the machine-readable contract and synthetic suite are:

- \`schemas/cter-disclosure-manifest-v0.1.schema.json\`
- \`benchmarks/territorial-integrity/egress/cases-v0.1.json\`

The new evaluation dimension is whether a prepared package remains identical to the exact request and upstream decision at release time: object bytes, source/transform lineage, project/evidence frontier, recipient and destination-key epoch, transport channel, currentness/revocation recheck, and release-attempt reconciliation.

This is not a claim that policy engines or data-packaging standards lack these capabilities. ODRL, PROV-O, BagIt, SCITT and policy-decision logging already cover important parts. CTER's testable hypothesis is their composition across a portable evidence package and an independently verifiable custody boundary. The initial corpus has one valid synthetic baseline and 31 adversarial cases; it is specified, not yet executed.


## 9. Expanded industry comparison: do not compete with the wrong layer

The second research pass adds field-capture, policy-decision and digital-preservation systems. Their established features narrow what CTER should claim.

| System / standard | Demonstrated strength to respect | What CTER should consume or test instead of recreating |
|---|---|---|
| QField / QFieldCloud | Offline field editing with local GeoPackage staging and later synchronization; stable UUIDs are recommended to avoid sync/relation conflicts | Use its GIS and field-editing capabilities. Test whether CTER can preserve authorization and lineage across the exported package and its later synchronization, not whether it can draw maps better |
| ODK Collect / Central | Offline form workflows and encrypted form submission; encryption protects finalized content in transit/storage without the server holding the private key in the encrypted-form mode | Use ODK for collection where suitable. CTER must additionally bind the exact export, source/transform lineage, recipient and release attempt. ODK explicitly warns encryption does not prevent malicious replacement/fabrication and cannot guarantee every local plaintext copy is destroyed |
| Open Policy Agent (OPA) | General-purpose policy decision point and decision logs including decision IDs, policy path, bundle revision, request input, result, timestamps and metrics; supports masking sensitive log fields | Treat OPA as a strong policy-engine/audit baseline. CTER's test is not "we log decisions"; it is whether the decision is bound to exact evidence objects, recipient, transformation graph and final release attempt without disclosing sensitive metadata |
| Oxford Common File Layout (OCFL) | Application-independent repository storage layout designed around completeness, parsability, versioning, robustness and storage diversity; version directories are immutable | Use OCFL-compatible preservation where it fits the custodian's repository. Test restore from an independent provider and verify the CTER evidence frontier/authorization lineage after provider exit |
| IETF BagIt (RFC 8493) | Standardized file packaging and payload manifests for reliable storage and transfer | Reuse BagIt for package transport/inventory; do not claim its package checksums prove authorization, source truth, semantic derivation or safe recipient disclosure |
| W3C ODRL / PROV-O / DCAT | Policy vocabulary; provenance interchange; dataset/distribution catalog vocabulary | Map to these rather than inventing competing generic standards. Keep protected object inventories and sensitive rights context outside public catalog metadata |

### Source references

- [QField offline editing and synchronization](https://docs.qfield.org/get-started/tutorials/advanced-setup-qfc/)
- [ODK encrypted forms](https://docs.getodk.org/encrypted-forms/)
- [ODK form finalization and offline workflow](https://docs.getodk.org/collect-forms/)
- [OPA decision logs and sensitive-field masking](https://www.openpolicyagent.org/docs/management-decision-logs)
- [OCFL 1.1 specification](https://ocfl.io/1.1/spec/)
- [BagIt RFC 8493](https://www.rfc-editor.org/rfc/rfc8493.html)

### What this means for fair benchmarking

Do not put the systems above on one synthetic leaderboard. Run matched, task-specific comparisons:

1. **Field capture:** task-completion rate, invalid/missing submissions, offline duration, recovery after partition, conflict resolution, operator training time, battery/storage/network consumption and language support. Compare QField, ODK, Open Tenure and Cadasta on workflows they actually implement.
2. **Policy decisions:** decision correctness on common fixtures, policy update/revocation delay, explainability, sensitive-log masking and evaluation latency. Compare OPA (and later other policy engines) with the narrow CTER request/package binding, not with Mycelix's authority issuer.
3. **Preservation and exit:** bit-level integrity, metadata completeness, version recovery, independent restore, storage diversity and time to rebuild a declared evidence frontier. Compare BagIt/OCFL-compatible repositories and CTER custody-exit workflows on matched archives.
4. **End-to-end evidence release:** unauthorized disclosure, scope substitution, missing lineage, stale/revoked authority, recipient/key-epoch drift, unknown-outcome replay and provider-exit recovery. This is the composition benchmark CTER is intended to own.

For every comparison, pin software versions, configuration, schema/profile, fixtures, host, hardware, network schedule and operators. Publish all failure cases, excluded cases and operational costs. Keep correctness/security thresholds separate from latency and usability; a faster unsafe exporter must never win a weighted score.

**Research hypothesis:** CTER's differentiator is not any one of offline collection, GIS, a policy engine, a hash, a provenance vocabulary or an archive format. It is the compositional proof boundary that binds an upstream owner-qualified decision to a specific recipient-bound disclosure package, its complete source/transform lineage, the actual release attempt and a later independent recovery. That hypothesis remains unproven until the cross-system adapter and matched field/archive trials exist.


## 10. Egress binding implementation and scale fixture status

The initial egress consumer implementation and its limits are recorded in [CTER Egress Binding Profile v0.1](CTER_EGRESS_BINDING_PROFILE_V0_1.md).

Committed on the CTER research branch:

- versioned [disclosure manifest schema](../../schemas/cter-disclosure-manifest-v0.1.schema.json), with explicit source and output data classes;
- frozen [32-entry adversarial corpus](../../benchmarks/territorial-integrity/egress/cases-v0.1.json): one valid baseline and 31 adversarial cases;
- Rust structural evaluator and 14 focused tests;
- independent no-dependency Node oracle with 13 structural cases;
- [1k/10k/100k/1M scale workload specification](../../benchmarks/territorial-integrity/egress/scale-workloads-v0.1.json);
- exact-path CI workflow coverage for the new schema, corpus, scale profile and Node oracle.

**Execution status:** code and tests are committed but not yet executed. Exact-head workflows must finish successfully before any pass claim. No throughput, latency, memory, storage, energy, production authorization or real egress result is claimed. The evaluator checks digest formatting and specified digest equality relationships, not payload hashing; receipt authentication, trusted time, resolver state and release are external host responsibilities.


## 11. Privacy-preserving coverage is a separate research dimension

[CTER-PRIVATE-COVERAGE-001 #962](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/962) studies whether CTER can prove completeness for an exact committed frontier and query without exposing withheld inventory or revealing metadata.

Initial artifacts:
- [Privacy-preserving coverage profile](CTER_PRIVATE_COVERAGE_PROFILE_V0_1.md)
- [16 coverage invariants](../../benchmarks/territorial-integrity/egress/private-coverage-invariants-v0.1.json)
- [17 synthetic cases](../../benchmarks/territorial-integrity/egress/private-coverage-cases-v0.1.json)

This programme must distinguish object inclusion from query completeness. RFC 9942 COSE Receipts provides VDS receipt machinery for inclusion, consistency, disclosure and non-inclusion properties; it does not by itself prove that every object matching an arbitrary CTER query was included. The latter requires a separately justified partition/completeness argument. The proof primitive is not selected, and the corpus is specified but unexecuted.

Do not award CTER a privacy or completeness score for opaque IDs, encrypted payloads or a receipt reference alone. The benchmark must test whether a verifier can detect omissions and stale frontiers while learning no protected inventory, count, location or low-entropy value beyond its authorized purpose.
