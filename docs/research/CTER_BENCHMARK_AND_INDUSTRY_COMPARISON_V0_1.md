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

**Execution status:** the independent JavaScript evaluator was exercised in a V8 harness against the committed synthetic corpus and matched 13/13 structural cases. This does not count as a Node 22 workflow result. The 14 Rust tests remain unexecuted; exact-head workflows are pending. No throughput, latency, memory, storage, energy, production authorization or real egress result is claimed. The evaluator checks digest formatting and specified digest equality relationships, not payload hashing; receipt authentication, trusted time, resolver state and release are external host responsibilities.


## 11. Privacy-preserving coverage is a separate research dimension

[CTER-PRIVATE-COVERAGE-001 #962](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/962) studies whether CTER can prove completeness for an exact committed frontier and query without exposing withheld inventory or revealing metadata.

Initial artifacts:
- [Privacy-preserving coverage profile](CTER_PRIVATE_COVERAGE_PROFILE_V0_1.md)
- [16 coverage invariants](../../benchmarks/territorial-integrity/egress/private-coverage-invariants-v0.1.json)
- [17 synthetic cases](../../benchmarks/territorial-integrity/egress/private-coverage-cases-v0.1.json)

This programme must distinguish object inclusion from query completeness. RFC 9942 COSE Receipts provides VDS receipt machinery for inclusion, consistency, disclosure and non-inclusion properties; it does not by itself prove that every object matching an arbitrary CTER query was included. The latter requires a separately justified partition/completeness argument. The proof primitive is not selected, and the corpus is specified but unexecuted.

Do not award CTER a privacy or completeness score for opaque IDs, encrypted payloads or a receipt reference alone. The benchmark must test whether a verifier can detect omissions and stale frontiers while learning no protected inventory, count, location or low-entropy value beyond its authorized purpose.


## 12. Mycelix-owned transparency dependency: no duplicate VDS

The privacy-coverage and egress benchmarks now explicitly compose with Mycelix [PULSE-TRANSPARENCY-ANTI-EQUIVOCATION-001 #5942](https://github.com/Luminous-Dynamics/mycelix/issues/5942) and [draft PR #5944](https://github.com/Luminous-Dynamics/mycelix/pull/5944). That PR contains a bounded TLA+ witness-quorum model around an existing SCITT-compatible TS/VDS: 4 witnesses, a maximum of 1 Byzantine witness, 3-of-4 matching checkpoint signatures, durable honest-witness checkpoint locks, and UNKNOWN when a client cannot establish quorum. It includes weak-quorum, double-sign and lock-loss negative controls.

The upstream PR is currently a research model, not a deployed protocol or runtime COSE/VDS implementation, and its own qualification record says TLC has not yet run. CTER therefore consumes the future Mycelix-owned verification result; it does not create another tree, witness committee or transparency ledger.

The end-to-end benchmark now distinguishes three independent claims:

1. **Statement/receipt validity:** the exact signed statement and COSE/VDS receipt verify under Mycelix's authoritative resolver.
2. **Checkpoint agreement:** the exact log/root/tree-size checkpoint has acceptable Mycelix witness/quorum and freshness/equivocation evidence.
3. **Query completeness:** every item matching CTER's exact committed query over the declared source frontier is accounted for and the included subset is bound to the exact package.

The first two do not automatically prove the third. That gap is the focus of #962 and must not be hidden behind a generic verified=true flag.


## 9. November 2026 research correction: privacy and preservation are existing capabilities too

The comparison must include the newest public work, not freeze competitors at a previous release.

- Cadasta's research catalogue now lists its 2026 *Cadasta Data Guidelines: A Framework for Responsible, Transparent, and Accountable Data Practices*, expressly intended to guide ethical data collection, management, sharing and protection, especially for Indigenous Peoples and Local Communities. CTER must not present generic community data governance as its novelty.
- W3C Verifiable Credentials 2.0 supports data minimization and selective disclosure mechanisms; W3C Bitstring Status List v1.0 (Recommendation, 15 May 2025) addresses privacy/performance issues of one-status-URL-per-credential lookups. Mycelix should remain the credential/status/currentness owner; CTER should consume privacy-preserving status resolution where that stack supports it.
- IETF RFC 9942 and RFC 9943 were published in June 2026 as Proposed Standards on the Standards Track. RFC 9942 defines COSE receipts proving properties of Verifiable Data Structures, including inclusion/consistency/disclosure proofs. RFC 9943 defines the SCITT architecture for transparency of single-issuer signed statements. CTER should reuse Mycelix's transparency boundary and not claim a new ledger or generic transparency primitives.
- RFC 8493 BagIt provides package structure and strong fixity/integrity checking, but its own security considerations say it is not designed to resist active attacks. A valid bag is not an authorization proof, source-truth proof or active-attacker defense.
- RFC 8785 JCS is an Informational RFC, not an Internet Standards Track specification. It is a useful candidate canonicalization profile, but the CTER profile must explicitly pin its constraints and test vectors instead of calling it a mandatory Internet Standard.
- NIST IR 8547 reached final publication in October 2026 and describes the transition from quantum-vulnerable to post-quantum signature and key-establishment standards. Long-lived CTER archives therefore need algorithm identifiers, renewal/migration evidence and historical-validation semantics; a digest or signature that verifies today must not be described as timelessly secure.

### New CTER qualification dimension

[CTER-PRIVACY-CHANNEL-001 #964](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/964) adds paired-world non-disclosure tests across spatial tiles, search, counts, error behavior, filenames, logs/traces, previews/OCR, AI retrieval, status lookups, caches, backups and transport metadata. Its first artifacts are:

- \`docs/research/CTER_PRIVACY_CHANNEL_PROFILE_V0_1.md\`
- \`schemas/cter-privacy-channel-inventory-v0.1.schema.json\`
- \`benchmarks/territorial-integrity/privacy/cases-v0.1.json\`
- \`benchmarks/territorial-integrity/relay/reference/privacy-oracle.mjs\`
- \`benchmarks/territorial-integrity/relay/reference/src/privacy.rs\`
- \`benchmarks/territorial-integrity/relay/reference/tests/privacy_conformance.rs\`

The synthetic fixture set currently has 24 paired-world cases: 20 leak detections, 3 exact non-disclosure controls and 1 undeclared-channel/inventory-gap control. A tool-side V8 fixture semantics check matched 24/24 expected dispositions. This is not a Node 22 workflow run or a production runtime privacy audit. The Rust tests and integrated CI remain pending exact-head execution.

### Revised differentiation hypothesis

CTER should not try to win by claiming offline mapping, governance, data minimization, selective disclosure, generic provenance, cryptographic packaging or transparency as inventions. Those have mature standards, products and practice.

The testable differentiation is the composition of:
1. exact upstream authority/consent references;
2. exact source-to-output transformation lineage;
3. recipient/package/release-attempt binding;
4. cross-channel non-disclosure and status-query privacy;
5. contestation and withheld/unavailable semantics;
6. independently restorable evidence frontiers with cryptographic migration history.

The comparison must still use matched datasets, operations, hardware, network traces, users and adversarial schedules. No product-level SOTA superiority is established by the current synthetic reference suite.


The privacy inventory/evaluator slice has since been expanded with:
- a channel-inventory JSON Schema and a synthetic inventory that is explicitly marked partial;
- a Rust structural evaluator that refuses to promote partial coverage to a complete declaration and requires an independent coverage-evidence reference for a complete claim;
- an independent paired-world Node oracle and matching Rust fixture tests;
- exact fixture counts of 20 detected synthetic leaks, 3 no-disclosure controls and 1 undeclared-channel inventory gap.

A tool-side V8 check matched 24/24 fixture expectations. This remains a fixture-semantics result, not a Node 22 workflow run, Rust run, live application audit or privacy guarantee. The workflow includes the new schema, corpus, inventory fixture and oracle; exact-head CI must still complete before any test-pass statement.


## 10. Private coverage implementation status

[CTER-PRIVATE-COVERAGE-001 #962](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/962) now has a versioned schema, Rust partition evaluator, Rust conformance tests and independent Node oracle. The machine-readable corpus contains 30 synthetic cases. A tool-side V8 implementation matched 30/30 expected dispositions: 1 valid partition structure, 3 incomplete, 5 unknown and 21 deny. These are fixture semantics only—not Node 22 CI, Rust execution, a COSE/VDS proof or a production completeness claim.

The new contract requires the exact frontier item set to equal the private partition item set; each item has one compatible scope/disposition; the included set equals package terminal source leaves; the receipt projection binds query/request/recipient/frontier/package; and the public verifier view does not expose item references or counts. Non-inclusion is explicitly restricted to the named frontier version.

RFC 9942 COSE receipts can establish properties of a selected VDS, including inclusion and consistency, but the exact CTER partition predicate remains a separate proof obligation. RFC 9942 proof types are VDS-specific; do not assume portability across VDS implementations. A ZK proof remains only a candidate profile and must be compared with a designated independent-auditor baseline on the same corpus.


The private-coverage comparison now also has a pinned workload specification: \`benchmarks/territorial-integrity/egress/private-coverage-scale-workloads-v0.1.json\`. It compares the designated-auditor baseline, Mycelix's selected VDS/COSE proof profile, and a research-only private partition-predicate proof over identical synthetic 1k/10k/100k/1M frontiers. It measures proof generation and verification separately from partition set-checks and metadata disclosure. No proof primitive, throughput winner or p99 target has been selected or measured.

Current tool-side V8 fixture results: privacy paired-world corpus 24/24, private coverage partition corpus 30/30. Egress exact hidden-transform-input mutation was detected. These are test-fixture semantics, not Node 22 Actions runs, Rust execution or production qualification. The CI workflows are configured to run the committed Node oracles and Rust tests on the exact checkout; queued is still not PASS.


## 13. Composed release binding and VDS proof selection

A further research step joins #960 exact egress binding with #962 private frontier coverage. The new structural evaluator derives terminal source leaves from the egress transformation graph and requires those leaves to equal the private coverage record's included set. It also binds the exact coverage reference, package payload commitment, request, principal, subject, operation, purpose, recipient, resource, jurisdiction, project snapshot, data classes and frontier reference across the two records.

Committed artefacts:
- \`docs/research/CTER_COMPOSED_RELEASE_BINDING_PROFILE_V0_1.md\`
- \`benchmarks/territorial-integrity/egress/composed-release-cases-v0.1.json\` — 17 synthetic cases (2 positive structural controls, 15 expected denials).
- \`benchmarks/territorial-integrity/relay/reference/src/release_binding.rs\` and the \`egress.rs\` terminal-leaf derivation helper.
- Rust conformance matrix and an independent Node composed-release oracle.
- Workflow steps that validate the composed corpus and execute the independent oracle.

A tool-side V8 check matched all 17 composed fixture expectations: 2 valid composed structures and 15 denied substitutions. This is not yet the committed Node 22 workflow or Rust execution. It does not recalculate package hashes over bytes, verify COSE/VDS receipts, resolve actual authority/currentness or transmit a package.

### Proof selection is a separate benchmark, not an implementation assumption

[CTER-VDS-PROOF-QUAL-001 #967](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/967) now tracks the choice of a privacy-preserving completeness proof. The comparison evaluates:
- Mycelix-owned SCITT/COSE VDS receipts for exact statement inclusion and append-only checkpoint consistency;
- sparse authenticated maps for keyed membership/non-membership;
- ordered authenticated dictionaries/range proofs for query completeness;
- private set-equality/zero-knowledge relations for the disjoint partition and included-source equality;
- a hybrid that keeps the transparency ledger and witness/checkpoint owner in Mycelix while CTER consumes a narrowly scoped coverage proof.

The standards boundary is material: RFC 9942 provides a COSE receipt envelope for VDS-specific proof types, but the initial \`RFC9162_SHA256\` profile concretely specifies inclusion and consistency proofs. The general COSE receipt concept includes non-inclusion as a proof family, but that does not mean an arbitrary frontier/query/partition set-equality proof is available from the initial profile. RFC 9162 itself is Experimental. CTER must identify the exact VDS/proof type and Mycelix interface it consumes rather than treating "COSE receipt verified" as proof of query completeness.

The new comparison profile is \`docs/research/CTER_VDS_PROOF_PRIMITIVE_COMPARISON_V0_1.md\`. It specifies 1k/10k/100k/1M benchmarks, proof size/generation/verification costs, metadata leakage, verifier independence, frontier updates, offline use, split-view and long-term migration. No proof primitive is selected and no cryptographic result is claimed.
