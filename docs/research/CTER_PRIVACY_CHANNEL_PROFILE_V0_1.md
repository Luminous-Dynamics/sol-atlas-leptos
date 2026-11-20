# CTER Privacy Channel and Non-Disclosure Qualification Profile v0.1

**Tracking:** [CTER-PRIVACY-CHANNEL-001 #964](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/964)  
**Composes:** CTER issues #951, #955, #960 and #947.  
**Status:** synthetic threat model and qualification contract; not an anonymity proof or production privacy guarantee.  
**Research date:** 2026-11-20.

## 1. The paradigm shift: qualify what can be inferred, not only what was exported

An export gate can correctly reject an unauthorized object in the primary package while still leaking the protected fact through a map tile, search result, count, changed response time, filename, error code, status lookup, log, cache, thumbnail, AI prompt or backup.

The privacy boundary is therefore not the file boundary. It is the complete set of observations available to each audience under each purpose.

CTER should qualify **non-disclosure over declared observable channels**. For two synthetic worlds that differ only in a protected fact, a public observer should not be able to distinguish those worlds wherever the applicable policy requires that fact to remain undisclosed.

This is stronger and more testable than "the coordinates are not in the JSON export." It is not a proof of universal anonymity: it is a falsifiable property over a declared threat model, channel inventory, corpus and execution environment.

## 2. Industry comparison: do not compete on the wrong thing

FAO Open Tenure already documents offline collection, community control, multilingual forms, participatory validation, disputes/challenges, evidence attachments, local encryption and local/cloud synchronization. FAO SOLA offers open-source, LADM-aligned land administration and community tenure tools.

Cadasta's platform already describes offline-ready mapping, secure storage, role-based sharing, integrations and community data protocols. Its current research catalogue includes 2026 *Cadasta Data Guidelines* for responsible, transparent and accountable data practices, expressly emphasizing Indigenous Peoples and Local Communities. This makes a generic claim to "community data sovereignty" or "responsible data governance" a poor differentiation strategy.

W3C Verifiable Credentials 2.0 and associated materials already support data minimization and selective disclosure. W3C Bitstring Status List v1.0 provides a privacy-preserving and space-efficient status distribution model; the W3C overview explains that one-credential/one-URL lookups create privacy and performance concerns.

CTER's opportunity is narrower: **cross-channel non-disclosure qualification tied to exact source/claim, recipient, purpose, disclosure package, status/currentness and custody boundaries**, integrated with these existing systems rather than replacing them.

## 3. Threat model and observable surface

Assume an observer can make repeated/adaptive public requests; compare public snapshots; inspect maps, tiles, catalogues, search, metadata and packages; observe response status/length/timing; query revocation/currentness endpoints; and correlate public information with auxiliary data.

Assume accidental exposure through application telemetry, support tooling, backups, analytics and model/agent workflows is possible. The adversary need not compromise the database or break cryptography.

### Channel inventory

| Channel family | Observable surface to inspect |
|---|---|
| Primary objects | JSON, CSV, GeoPackage, attachments, archive files and payloads |
| Object metadata | filenames, paths, IDs, stable pseudonyms, MIME, EXIF, size, timestamps and low-entropy digests |
| Spatial projection | points, bounds, geometry, generalization, raster/vector tiles, zoom levels and cache keys |
| Search/catalogue | search, autocomplete, filters, pagination, empty results, public catalogue and object counts |
| Aggregates/diffs | totals, dashboards, before/after snapshots, change notifications and query differencing |
| Protocol behavior | status/error code, body length, timing, retries, pagination and rate limiting |
| Operations telemetry | logs, traces, metrics, crash dumps, analytics, support bundles and alert payloads |
| Derived media | thumbnail, preview, OCR, document conversion, generated summaries and embeddings |
| AI/agent | prompt context, retrieved objects, tool-call arguments, caches, traces and output summaries |
| Status/currentness | one-to-one receipt/credential status requests, URLs, network destinations and query timing |
| Relay/transport | bundle metadata, sender/recipient inventories, sync histories and delivery receipts |
| Preservation/exit | backup snapshots, restore packages, withheld/unavailable accounting and provider-exit evidence |

Unknown or undeclared channels block a claimed complete privacy qualification. A list authored by the same application cannot prove that no uninstrumented channel exists: deployments must state the observation boundary and the independent means used to inventory it.

## 4. Formal qualification properties

### 4.1 Protected-world non-interference

For a policy-defined public observer, define two synthetic worlds A and B that differ only in one protected fact. Let **Obs(channel, world, query_schedule)** be the observable response transcript across the declared channels and schedule.

For a strict non-disclosure control:

**Obs(public, A, Q) == Obs(public, B, Q)**

This must include response content, object ordering, status, pagination, sizes and any timing fields explicitly in scope. Real network timing is noisy; timing non-interference requires a specified experiment, repeated trials, declared statistical test and confidence bound. Do not claim timing non-interference from one local comparison.

### 4.2 Query-differencing resistance

A series of individually permitted aggregate answers must not enable reconstruction of a restricted member or location through set differences, overlapping windows, repeated filters or changed cohort membership. Test complete query schedules, not isolated requests.

Differential privacy may be appropriate for some repeated aggregate outputs, but only with:
- explicit neighboring-dataset definition;
- contribution bounds and unit of privacy;
- declared epsilon and delta;
- an auditable, versioned, composable privacy accountant;
- budget enforcement across endpoints, time windows and recipients;
- utility and bias reporting.

Differential privacy is not consent, authorization, community mandate or a universal solution for maps and legal evidence. Do not add it automatically where its assumptions do not fit.

### 4.3 Identifier and commitment privacy

A deterministic digest of a low-entropy identifier, coordinate, small category or predictable document is not assumed anonymous. Test dictionary attacks, linkage with auxiliary information, repeated commitment reuse and cross-recipient linkability.

Use scoped opaque references, keyed commitments or reviewed selective-disclosure proofs when appropriate. A keyed commitment introduces key custody/rotation dependencies and must not become a substitute for community-held custody.

### 4.4 Revocation/currentness query privacy

A one-credential/one-URL status lookup can reveal which subject or receipt a verifier is checking. Evaluate group status lists, batch status resolution and cache behavior. W3C Bitstring Status List is a candidate to reuse where compatible with Mycelix's credential/status stack; Mycelix remains the owner of status resolution.

A status list can reduce direct lookup correlation; it does not establish that the issuer is authorized or that the underlying community decision is legitimate. Bind status freshness and the status-list credential's own verification/currentness context.

### 4.5 Withheld is not absent

The custodian's internal record may distinguish withheld, unavailable, restricted, out-of-scope, not-collected and not-found. A public response may intentionally make several states observationally indistinguishable where the disclosure policy requires existence concealment.

The public response must never turn a private or unavailable record into a positive assertion that no underlying right, claim, evidence or consent exists.

### 4.6 No automatic declassification

Pseudonymization, aggregation, geospatial generalization, redaction, blurring, k-anonymity and file conversion are transformations with residual risk. None is a universal certificate of anonymity.

A threshold such as k greater than or equal to 5 is not a universal safety rule: auxiliary information, repeated releases, distinctive geometry and temporal differencing can still reveal identities or locations. Any threshold or small-cell rule must be declared by the applicable custodian/community policy and tested against auxiliary data.

## 5. Synthetic canary and paired-world protocol

All fixtures are synthetic. Never publish real claimant identifiers, sensitive locations, sacred-site coordinates, protected oral knowledge or partner data in a public corpus.

For each test:
1. Create two controlled worlds with an explicit protected-fact delta.
2. Freeze the public query schedule and channel inventory.
3. Execute both worlds through the same instrumented build.
4. Capture a canonical observable transcript for every declared channel.
5. Search every transcript and generated artefact for synthetic canary tokens and canary-derived commitments.
6. Compare exact outputs where strict non-interference is required.
7. Where bounded statistical leakage is explicitly permitted, publish the statistical test, sample count, confidence interval, privacy budget and false-positive/utility impact.
8. Preserve raw test results privately where they reveal system internals; publish only approved synthetic artifacts and aggregate metrics.

Canaries should be unique per test run and placed in synthetic source fields, filenames, metadata, coordinates, logs and derived assets. A canary hit is a positive signal of leakage. No hit is not proof that no other information can leak.

## 6. Adversarial corpus families

At minimum, freeze cases for:

1. protected fact appears in the primary payload;
2. protected fact leaks through filename or directory path;
3. stable object ID enables cross-recipient tracking;
4. low-entropy digest is recovered by dictionary attack;
5. exact point leaks through vector tiles at a high zoom level;
6. bounding box or simplified geometry reveals a restricted site;
7. tile-cache key or ETag reveals a protected edit;
8. search/autocomplete reveals existence of a restricted record;
9. empty-result behavior distinguishes absent from withheld;
10. count or dashboard change reveals a new protected claimant;
11. overlapping query windows reconstruct a hidden member;
12. response size reveals a protected object or its approximate size;
13. timing/retry behavior distinguishes hidden-object presence;
14. pagination or sort order leaks hidden-object rank;
15. HTTP error codes reveal object existence or authorization state;
16. EXIF/OCR/thumbnail/preview contains hidden values;
17. logs/traces/crash reports/support bundles contain canaries;
18. backup/restore package contains an undeclared channel;
19. AI prompt/retrieval/cache contains a restricted source;
20. status endpoint/URL reveals which credential or receipt is queried;
21. public response says absent when the internal state is withheld;
22. identical world pairs differ because a hidden record changes the public identifier or cache namespace;
23. consent withdrawal changes future release policy but is falsely represented as deletion of prior recipient copies;
24. recipient-specific views are linkable through stable pseudonyms or commitment reuse.

## 7. Benchmarks and targets

All figures below are qualification targets, not achieved results.

| Metric | Target |
|---|---:|
| Canary detections in forbidden channels | 0 |
| Protected-world distinctions in exact non-disclosure controls | 0 |
| Undeclared channels in the declared deployment boundary | 0 |
| Unauthorized stable identifier disclosures | 0 |
| Low-entropy commitment disclosures without approved mitigation | 0 |
| Unauthorized status-query correlation in the frozen test | 0 |
| Exact paired-world transcript agreement where required | 100% |
| Privacy budget overrun under an approved DP policy | 0 |

Also measure false positives and legitimate information suppressed; task completion and time-to-completion; query cost, response latency and cache efficiency; spatial utility and map interpretability; status-check bandwidth and response size; privacy test coverage by channel and threat family; leakage-detection precision/recall on synthetic canaries; and statistical power/confidence bounds for noisy channels.

Do not collapse privacy failures, community acceptability, usability and performance into one weighted number.

## 8. Standards and existing mechanisms

- [W3C Verifiable Credentials Data Model v2.0](https://www.w3.org/TR/vc-data-model-2.0/) — data minimization, proof and selective-disclosure ecosystem.
- [W3C Bitstring Status List v1.0](https://www.w3.org/TR/vc-bitstring-status-list/) — privacy-preserving, space-efficient credential status distribution.
- [W3C ODRL 2.2](https://www.w3.org/TR/odrl-model/) — permission/prohibition/duty and constraints; use community profiles rather than inventing a general policy language.
- [W3C PROV-O](https://www.w3.org/TR/prov-o/) — provenance relationships for data products and transformations.
- [IETF RFC 9942](https://www.rfc-editor.org/rfc/rfc9942.html) and [RFC 9943](https://www.rfc-editor.org/rfc/rfc9943.html) — COSE receipts and SCITT transparency; consume through Mycelix rather than create another ledger.
- Cadasta 2026 Data Guidelines and the existing FAO/Open Tenure/SOLA workflows are direct practice comparators, not capabilities to dismiss.

## 9. Ownership and non-claims

- Mycelix owns identity, issuer trust, consent, authorization, status/currentness and typed receipt resolution.
- Sol-Atlas owns the CTER channel inventory, projection/export boundaries and consumer-side privacy qualification.
- Symthaea may discover possible sensitive outputs and review evidence read-only; its explanations are not authorization decisions.
- Symtropy can generate synthetic spatial scenarios and traces; simulated leakage is not a field measurement.
- Communities and data custodians define applicable restrictions, permitted audiences and acceptable governance processes. A benchmark cannot manufacture that legitimacy.

A PASS on this profile means only that the declared build, corpus, query schedules and channels satisfied the published test properties. It does not prove universal anonymity, absence of unknown channels, legal compliance, consent, community approval or safety against every auxiliary dataset.


## 10. Initial machine-readable and executable slice

The first public artefacts are now committed:

- Channel inventory schema: \`schemas/cter-privacy-channel-inventory-v0.1.schema.json\`.
- Synthetic, explicitly partial inventory: \`benchmarks/territorial-integrity/privacy/channel-inventory-synthetic-v0.1.json\`.
- Paired-world corpus: \`benchmarks/territorial-integrity/privacy/cases-v0.1.json\`.
- Rust fixture evaluator: \`benchmarks/territorial-integrity/relay/reference/src/privacy.rs\`.
- Rust tests: \`benchmarks/territorial-integrity/relay/reference/tests/privacy_conformance.rs\`.
- Independent Node oracle: \`benchmarks/territorial-integrity/relay/reference/privacy-oracle.mjs\`.

The 24-case paired-world corpus has:
- 20 known synthetic leak cases;
- 3 exact non-disclosure controls;
- 1 deliberately undeclared channel that must produce inventory-incomplete.

A tool-side V8 harness parsed the committed schema, synthetic inventory and corpus and matched **24/24** expected fixture dispositions. Counts: 20 leak-detected, 3 non-disclosure, 1 inventory-incomplete. This is not a Node 22 workflow execution, Rust execution, production channel audit, statistical timing study or anonymity proof.

The sample inventory deliberately says \`partial\` and names three illustrative channels. It excludes a third-party CDN edge-log surface, making the incompleteness explicit. The Rust inventory evaluator checks required record structure, duplicate channel references and the presence of an independent coverage-evidence reference for a complete-boundary claim. A reference string is not authentication: the host/independent auditor must still resolve and inspect the evidence.

The paired-world evaluator only compares the JSON observations represented in the fixture, detects synthetic canary strings and flags channel names absent from the declared inventory. It does not inspect live map tiles, application logs, CDNs, caches, production APIs, AI providers or backups. The production channel inventory and independent observation method are prerequisites for any field claim.

The exact-head workflow has been extended to validate the privacy schema, fixture and corpus, and run both Node oracles. At the time of this profile revision those jobs remain subject to live GitHub Actions status; queued is not PASS.
