# Federated Source Inventory Completeness — Research and Benchmark Contract v0.1

**Status:** v0.1 Rust verifier, schema, and synthetic fixtures implemented on a draft stacked branch; external source authentication remains separate  
**Scope:** source-system inventory claims feeding the built-environment source descriptor catalogue and IEAC projection  
**Claim boundary:** the current v0.1 verifier can qualify internal consistency within a declared query boundary. It does not authenticate that boundary or prove that the declared source set is the entire real-world source universe.

## 1. Why this is the next architectural boundary

The source descriptor catalogue validates one exact BuiltEnvironmentCase against its declared owner-native records. This answers whether each declared typed case record has a descriptor, whether declared native identity components agree with the typed record, and whether digest, rights vocabulary, schema metadata, snapshot and temporal fields satisfy the descriptor contract.

It does **not** answer whether the connector enumerated every record in the upstream source; whether all pages, partitions, tenants, regions, revisions and tombstones were included; whether a source-side filter was silently applied; whether the source changed during enumeration; whether other relevant source owners were omitted; or whether absence of a returned record constitutes negative evidence.

These questions are about inventory scope and enumeration, not record identity. Do not collapse them into missing-descriptor counts, not-found states, or a confidence score.

## 2. Paradigm shift: from connector output to proof-carrying inventory

A connector should not merely return records. It should return an inventory receipt that makes the enumeration boundary independently inspectable.

The intended pipeline is:

1. **Discover:** identify expected source owners and systems for a declared asset/project boundary.
2. **Freeze:** obtain a source snapshot token, watermark, immutable export ID, or explicitly record that the source cannot provide snapshot consistency.
3. **Enumerate:** retrieve every page/partition under a canonical query scope, preserving page order and cursor ancestry.
4. **Reconcile:** compare reported totals, returned identities, tombstones, duplicates, filters and exclusions.
5. **Attest:** authenticate the receipt issuer/key and verify the receipt signature and trust status in a separate verifier.
6. **Qualify:** issue a bounded inventory disposition with precise nonclaims.
7. **Project:** only then compose the record descriptors into IEAC. A descriptor catalogue cannot promote an incomplete inventory to complete.

The decisive principle: **absence is evidence only when the relevant source scope was enumerated completely under a stable and authenticated snapshot.** Otherwise absence means unknown.

## 3. Proposed data model

The v0.1 schema and Rust validator are implemented on the stacked draft branch; source adapters and independent trust verifiers remain separate.

### 3.1 Inventory manifest

A v0.1 manifest should carry:

| Field | Purpose |
|---|---|
| inventory_id | Stable ID for this enumeration operation |
| inventory_schema_version | Exact manifest schema |
| source_owner_namespace | Owner namespace from the source identity contract |
| source_system_id | Native source system / tenant / repository identity |
| connector_id + connector_build_sha256 | Adapter identity and exact build |
| query_scope_sha256 | Digest of canonical query and filter semantics |
| query_scope_summary | Human-auditable, non-secret statement of included domains, time range and partitions |
| access_scope | Source assertion that enumeration is complete within the declared source-policy query, caller-authorized subset only, unknown, or denied; a limited/unknown scope is incomplete, denied is not comparable |
| snapshot_token / snapshot_sha256 | Source snapshot or export identity; null must remain explicit |
| enumeration_started_at / enumeration_finished_at | Enumeration time interval |
| source_watermark | Source-side high-water mark, sequence, revision or event offset, if supported |
| page_receipts[] | Page ordinal, page digest, item count, predecessor receipt digest, cursor transition and terminal marker |
| reported_total_count | Source-reported count under the declared query |
| enumerated_unique_count | Count of unique owner-native records returned |
| duplicate_count | Repeated owner-native identities during enumeration |
| excluded_count + exclusion_reasons[] | Records excluded by declared policy, rights or query conditions; each excluded item is bound to an enumerated native-identity digest, and each identity may appear under at most one exclusion reason |
| tombstone_count | Explicitly observed deletes/retractions |
| pagination_exhausted | True only when the source's authoritative terminal condition was observed |
| snapshot_consistent | Whether every page belongs to the same source snapshot/watermark |
| truncation_detected | Explicit truncation/error signal |
| issuer_key_id + attestation_ref | References for a separately verifiable source attestation |
| limitations[] | Unsupported features, unavailable partitions, rate-limit interruptions and other nonclaims |

Do not include raw secrets, access tokens, internal credentials, or protected query predicates in a public receipt. Hash exact canonical query bytes and publish an appropriately redacted scope summary.

### 3.2 Page receipt chain

Each page receipt should bind:

- a per-item exact source-payload digest array, positionally aligned with the owner-native identity digest array, so repeated native identity can be checked for conflicting payload bytes without retaining payload content; The generic verifier compares these hashes but does not fetch the payload bytes; the hashes remain unauthenticated source/connector claims.

- inventory ID and snapshot token/digest;
- 0-based ordinal and stable partition identifier;
- digest of the exact serialized page payload;
- returned item count;
- first and last source-native cursor/sequence when available;
- predecessor receipt digest;
- next cursor digest or an explicit terminal marker;
- response timestamp and source revision/watermark.

The verifier must reject a skipped ordinal, repeated cursor, broken predecessor link, duplicate terminal page, a page from a different snapshot, or a terminal marker that does not agree with the source API's documented completion semantics.

A cursor chain proves only that the returned pages form the declared chain. It does not prove the source API itself exposed every relevant record. That requires source-specific completeness semantics and an authenticated source claim. The Rust verifier also requires a per-record payload digest for every page item and rejects one owner-native identity paired with conflicting payload digests across pages or partitions. Matching duplicate identities remain counted as duplicates rather than silently merged. These payload digests are still source/connector assertions: the verifier does not retrieve or independently hash those source bytes.

### 3.3 Bounded inventory disposition

Use explicit dispositions, never a single confidence number:

- complete_within_declared_query: every required partition/page was enumerated; pagination terminated; counts and snapshot constraints reconcile; no undeclared filter/truncation is known.
- complete_with_declared_exclusions: enumeration completed but explicit, policy-bound exclusions exist. This is not equivalent to an unfiltered inventory.
- incomplete_or_indeterminate: a page, partition, snapshot guarantee, count, cursor, tombstone stream or trust check is missing or contradictory.
- invalid_input: broken digest chain, malformed cursor sequence, contradictory counts, invalid source identity or malformed manifest.
- not_comparable: the source cannot expose enough semantics to compare its inventory to the benchmark's declared gold scope.

The first two are bounded to the exact source, tenant, snapshot, query, time range and partitions in the manifest. Neither says that all real-world sources were discovered.

### 3.4 Two separate completeness claims

Keep these claims distinct:

1. **Record-set completeness:** all records in a source query/snapshot boundary were enumerated.
2. **Source-universe coverage:** all relevant owners/systems for a project or asset were identified and included.

The first can be tested against source-side counts, cursor/page semantics and an independent export. The second requires an independently maintained source-discovery inventory, change-control, owner attestations and explicit treatment of unknown systems. Never infer source-universe coverage from complete pagination.

## 4. Threat model and negative controls

The verifier must handle both malicious and accidental failures:

- missing or skipped page;
- duplicate page, cursor replay, cursor loop or false terminal marker;
- count mismatch, count overflow, unstable count, or count from a different query;
- page substitution from a different snapshot or tenant;
- source mutation during enumeration;
- authorization-limited response presented as unrestricted;
- source access denied but represented as an empty inventory;
- caller-authorized subset or unknown access scope presented as a source-policy-complete inventory;
- silent default filter, retention window, pagination cap, or result truncation;
- deleted/tombstoned records omitted from the current-state endpoint;
- late-arriving events and retroactive corrections;
- duplicate owner-native identities with equal and conflicting payload digests;
- content digest equal across different owner identities;
- invalidated, revoked or expired signing keys;
- valid signature over a false or incomplete source claim;
- connector upgrade that silently changes mapping/filter behavior;
- source API outage, rate limiting, partial response or undocumented pagination semantics.

A valid signature authenticates the signer and signed bytes under a trust policy. It does not make the source's claim true. The benchmark must preserve separate fields for signature verification, source authority, snapshot semantics, enumeration completeness and semantic correctness.

## 5. Benchmark protocol

### 5.1 Gold corpora

Create source-specific, independently authored gold inventories. The gold inventory must not be generated by the connector or validator being evaluated.

Required corpus families:

- IFC/BIM records and relationships with IDS property constraints;
- CAD/PLM assemblies, revision histories, parts and engineering-change records;
- permits, inspection records, utility assets and geospatial observations;
- manufacturing material lots, certificates, process events and quality records;
- event streams containing late arrivals, corrections and tombstones.

Every corpus has a declared universe, source-owner namespace, snapshot boundary, expected record set, expected relationships, and intentionally withheld/inaccessible records.

### 5.2 P0 acceptance gates

| Metric | P0 target |
|---|---:|
| False complete_within_declared_query on a truncated/incomplete source | **0** |
| Accepted page-chain gaps, cursor replays or cross-snapshot pages | **0** |
| Duplicate owner-native records silently merged | **0** |
| Count mismatch accepted as complete | **0** |
| Tombstone or late-event omission silently treated as current absence | **0** |
| Inaccessible or undeclared source treated as empty | **0** |
| Exact page digest / manifest digest replay | **100%** |
| Deterministic disposition for frozen manifests | **100%** |
| Unknown/unsupported query semantics reported as not_comparable or incomplete | **100%** |

Run at least 1,000 deterministic mutations per negative-control family in CI; retain a larger held-out property-based/fuzz corpus for nightly or release qualification. Report the exact mutation seed, input digest, verifier build and result for every failure.

### 5.3 Coverage and performance metrics

Measure separately:

- record precision/recall against the independent source inventory;
- relationship precision/recall against the independent gold graph;
- owner/source discovery precision and recall;
- complete-within-query precision (the most safety-critical metric);
- incomplete-query detection recall;
- false absence rate;
- tombstone and late-arrival detection latency;
- page enumeration throughput and p50/p95/p99 latency;
- digesting, signature verification and reconciliation CPU/RAM overhead;
- bytes and metadata retained per million records;
- recovery time after cursor expiry, source outage and connector restart;
- cost per million reconciled records and per impacted engineering decision.

The most important metric is **false completeness**, not average completeness. A system that returns unknown too often is operationally inconvenient; a system that claims complete when it has silently missed a page can invalidate engineering decisions.

## 6. Industry comparisons

The benchmark should build on existing work rather than claim that federation, digital-twin composition, traceability or operational ontologies are new:

- **ISO 23247-6:2026** specifies integrated, unified and federated manufacturing digital-twin composition. Sol-Atlas should prove that each composed view carries a verifiable source-inventory boundary and loss report, not claim to invent federation.
- **NIST IR 8536 (final, 9 September 2026)** defines an industry-neutral manufacturing supply-chain traceability meta-framework, including cryptographic links, temporal provenance and selective disclosure while preserving existing sector standards. The additional Sol-Atlas question is whether each traceability segment was actually enumerated completely and whether the engineering conclusions relying on it remain current.
- **IFC 4.3.2.0 / ISO 16739-1:2024** is the official current IFC 4.3 release. Pair IFC fixtures with IDS validation and report semantic loss, not just parse success.
- **buildingSMART's 2026 IDS survey** found that different tools can produce different results for the same IDS file; 131/186 respondents (70%) wanted some form of IDS software certification. Cross-tool conformance, exact reason codes and independently authored gold fixtures belong in the benchmark.
- **Bentley iTwin** already exposes source synchronization/connectors, changesets, validation/clash detection, changed-element review, webhooks and spatial alignment. Compare the inventory semantics and evidence export that can actually be observed, not vendor internals that cannot be exported.
- **Siemens Digital Twin Composer** combines 2D/3D twin data with real-time physical data and supports large-scale simulation/virtual commissioning. The Sol-Atlas differentiator should be portable evidence qualification and cross-owner invalidation around these outputs, not a claim to replace the simulation environment.
- **Palantir Ontology** already integrates data, logic, actions and security into operational workflows. Compare portable, independently replayable evidence and bounded source-completeness claims; do not claim that object/link/action modelling is novel.

Useful primary references:
- ISO: https://www.iso.org/standard/87426.html
- NIST final report: https://csrc.nist.gov/pubs/ir/8536/final
- buildingSMART IDS survey: https://www.buildingsmart.org/ids-survey-results-2026/
- buildingSMART IFC versions: https://www.buildingsmart.org/standards/bsi-standards/industry-foundation-classes/
- Bentley APIs: https://developer.bentley.com/apis/
- Siemens Digital Twin Composer: https://www.siemens.com/en-us/company/digital-transformation/industrial-metaverse/introducing-digital-twin-composer/
- Palantir Ontology: https://www.palantir.com/docs/foundry/architecture-center/ontology-system

## 7. How this changes IEAC and decision impact

Once source-inventory receipts exist, the decision pipeline can distinguish:

- a record was not present in a complete query boundary;
- a record was not returned because a query was incomplete;
- a record was excluded by rights or policy;
- a source was not discovered or was not connected;
- a source was inaccessible or incomparable;
- a record existed at the decision watermark but was later deleted/revised;
- a receipt's signer or trust key was later revoked.

Only the first case can support a bounded negative finding. All other states must remain unknown, incomplete, excluded, or invalid according to the evidence.

Downstream NoImpact must require a validated dependency graph **and** a sufficiently complete source inventory for the declared impact scope. An invalid or incomplete inventory cannot be converted into a green no-impact result by a correct graph traversal.

## 8. Current v0.1 implementation status

The draft branch now includes:
- a strict Rust manifest/page-receipt model and Draft 2020-12 JSON Schema;
- per-partition ordinal, cursor, terminal, predecessor-receipt, snapshot, watermark and time-order checks;
- separate raw page-item and unique owner-native identity count reconciliation;
- exact per-record payload-digest alignment, matching duplicate accounting, and conflict rejection for the same native identity with different payload digests;
- tombstone identity binding and policy-bound exclusions with unique identity accounting;
- expected-owner presence reconciliation kept distinct from per-source query completeness;
- composed owner-closure validation that requires every required owner manifest to validate independently;
- a descriptor-to-inventory projector path that matches owner namespace, source system, native identity digest, source payload digest and snapshot digest for every descriptor;
- an explicit `no_impact_eligible: false` output and blockers, including when every synthetic inventory is internally complete. Source access scope is now explicit: limited and unknown scopes are incomplete, and access-denied empty results are `not_comparable`, never empty.

The projector receipt binds the inventory-closure validation digest and descriptor-binding result. It also binds a digest of the caller-supplied expected-owner register and records each manifest's owner namespace, source system, connector/build, query digest, snapshot semantics/digest, page count, manifest digest and per-manifest disposition. The register digest proves which expected set was evaluated; it does not authenticate the register or prove it is globally complete. A missing, incomplete, excluded, incomparable or invalid inventory, or any descriptor/inventory identity, payload or snapshot disagreement, downgrades the projected graph claim ceiling to `incomplete_or_indeterminate`.

**Important serialization limit:** page-receipt hashes and current validation hashes use the Rust struct's `serde_json` field-order serialization profile. They are not yet a declared cross-language canonical JSON format such as RFC 8785 or deterministic CBOR. They must not be used as interoperable signature inputs until that profile is explicitly frozen and independently implemented.

**Verification limit:** current tests use synthetic manifests. Payload digests, connector identity, source snapshot references, owner namespaces, page receipt digests and attestation references remain claims until a separate byte verifier and trust policy authenticate them. Neither a structurally complete inventory nor a matching descriptor binding authorizes `NoImpact`, construction, or actuation.
## 9. Remaining implementation sequence

1. Keep this contract and synthetic positive/negative manifests frozen; expand them with independent cross-tool corpora.
2. Implemented on the draft branch: the Rust SourceInventoryManifest model and JSON Schema live in a separate v0.1 module, leaving the existing descriptor schema additive and backwards-compatible.
3. Implemented on the draft branch: page chains, query digest shape, snapshot and watermark bindings, counts, tombstones, exclusions, and duplicate identity/payload consistency. Canonical query semantics remain source-adapter-specific and are not independently reconstructed by the generic verifier.
4. Keep cryptographic signature verification as a separate verifier input with explicit trust-policy results; not implemented by this module and never conflate it with structural validation.
5. Implemented as a draft composition API: join owner inventory closure and per-descriptor native identity/payload/snapshot bindings into IEAC projection receipts, with claim-ceiling downgrade on missing or conflicting coverage.
6. Add independent full-inventory gold oracles and mutation tests.
7. Add change-event/tombstone watermarks to revalidation-frontier certificates.
8. Only then compare end-to-end completeness, latency and cost against independent connector exports and vendor interfaces.

**Non-goal:** this proposal does not authenticate a source merely because it declares a snapshot hash or attestation reference. Local byte hashing, signature verification, signer authorization, rights verification, source truth, source discovery and engineering correctness remain independent obligations.
