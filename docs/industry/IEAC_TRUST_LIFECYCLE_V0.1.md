# IEAC temporal assurance lifecycle and dependency impact v0.1

**Status:** Typed dependency contract, bitemporal invalidation event, synthetic key-compromise corpus and standalone Rust impact auditor. The current fixture is synthetic; it does not verify the cryptographic authority of the invalidation event.  
**Tracks:** [IEAC-TRUST-LIFECYCLE-001 / #396](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/396).  
**Composes with:** SCITT decision history #375, structural history auditor #382, signed envelope #363, source-native evidence #369, selective disclosure #384, witness observation #387, outcome reconciliation #390 and cross-layer benchmark #392.

## 1. Paradigm shift

A verification pass is not a permanent certificate. A key can be compromised, a source record corrected, evidence rights revoked, a model superseded, a verifier found vulnerable, a policy changed or an outcome corrected after the original decision.

**Assurance is a time-bounded claim over an exact dependency graph.** Later changes create append-only invalidation/supersession events; the system computes the exact downstream impact and schedules targeted re-verification.

The chain is:

`artifact change → authenticated invalidation event → dependency impact analysis → targeted re-verification → append-only review/supersession receipt`

The original record is never silently edited or deleted. A past decision remains a record of what was decided with the evidence available then. Its continued admissibility, current use, causal attribution and remediation requirements may change.

## 2. Standards composition

### 2.1 SCITT — authentic statements and transparency history

RFC 9943 requires issuer and Transparency Service key compromises to be communicated clearly so statements can be rejected or receipts ignored. It also separates signature/transparency guarantees from the relying party's decision about which issuers to trust. IEAC must add its own authorized invalidation policy; a valid SCITT receipt is not a permanent endorsement of the statement's content or issuer.

### 2.2 TUF — monotonic trust metadata and recovery

The Update Framework provides mature patterns for:
- monotonically increasing versions and rollback detection;
- expiration and freeze-attack detection;
- delegated authority and revocation;
- root/key rotation and threshold authorization;
- safe recovery after key compromise.

IEAC adopts these principles for assurance state and dependency graphs, not TUF's software-update roles verbatim. TUF is a useful design baseline, not a replacement for SCITT or domain-specific revocation authorities.

## 3. Bitemporal semantics

Every invalidation event must separate three times:

| Time | Meaning | Example |
|---|---|---|
| `effective_at` | When the cause is asserted to have taken effect | Earliest known key-compromise boundary |
| `recorded_at` | When the relying system learned/recorded the event | Incident discovered or revocation received later |
| `trusted_at` | Independently verified timestamp/SCITT registration time of the invalidation event | Trusted transparency receipt time |

These timestamps are not interchangeable. A revoker's self-asserted `effective_at` is not independently proven merely because the invalidation event is signed. The event must have authorized scope, supporting evidence, a trusted registration and the appropriate independent review.

The v0.1 event distinguishes `verified`, `authority_asserted`, `unknown` and `disputed` effective-time confidence. Only a verified effective-time boundary can request retroactive-from-effective-time semantics. Asserted, unknown or disputed boundaries must remain prospective, re-verification-required, superseded-only or quarantined.

## 4. Typed dependency graph

The closed Draft 2020-12 contract is `schemas/industry-assurance-dependency-graph-v0.1.schema.json`.

Node identity is type-separated and content-addressed:

`node_id = node_type + ":sha256:" + artifact_sha256`

The graph captures source measurements, signed statements, TS receipts, issuer/TS keys, schemas, transformations, models, policies, authority grants, decisions, authorizations, executions, outcomes, verifier builds and trust bundles.

Edges point from an upstream dependency to the dependent artifact. Examples:
- signing key → signed statement (`signed_by`);
- source measurement → derived statement (`derived_from`);
- signed statement → transparency receipt (`registered_as`);
- receipt → decision (`evidence_for`);
- policy → decision (`governed_by`);
- decision → authorization → execution → outcome.

The graph snapshot digest is SHA-256 of the canonical JSON graph object with `graph_snapshot_sha256` removed. The auditor recomputes it. Runtime also checks type-separated node IDs, digest equality, edge resolution, uniqueness, scope and DAG acyclicity.

The graph is a dependency record, not an ontology of physical truth. An edge means only the declared relationship; the referenced owner must establish whether the relationship is authentic and semantically correct.

## 5. Append-only invalidation event

The event contract is `schemas/industry-assurance-invalidation-event-v0.1.schema.json`.

It binds:
- invalidation authority identity and monotonic sequence/predecessor;
- exact target node type and artifact digest;
- cause class and domain/subject/purpose scope;
- effective, recorded and trusted times;
- effective-time confidence and dispute state;
- evidence digests and authorization receipt;
- whether the event is prospective, retroactive, requires re-verification, supersedes a prior claim or remains quarantined;
- `action_authority = none`.

The event's SCITT statement and timestamp receipt are separate evidence dependencies. The current Rust auditor does not verify the COSE signatures, SCITT receipt, revoker authorization or truth of the incident evidence. It validates structure and computes the deterministic dependency impact from the synthetic inputs only.

## 6. Impact classes are not one boolean

Each dependent node receives separate historical and current-reliance classifications:

| Impact class | Meaning |
|---|---|
| `no_impact` | No in-scope dependency path from the invalidated node |
| `historical_record_only` | Preserve historical record and its supported at-the-time validity |
| `reverify_required` | A dependent operational/attribution claim must be re-evaluated |
| `claim_invalidated` | The invalidation applies to the claim's relevant time/scope |
| `quarantined_pending_review` | Effective time, scope, authority or evidence is uncertain/disputed |

The report separately states:
- `historical_authenticity`: `valid_at_time`, `invalid_at_time`, `unknown`, or `not_applicable`;
- `current_reliance`: `eligible`, `reverify_required`, `ineligible`, `quarantined`, or `not_applicable`.

This prevents the common error of treating “not safe to rely on now” as “the historical record never existed.”

### 6.1 Key-compromise boundary

The synthetic key-compromise fixture tests a critical timing nuance:

- A signed statement with an exact trusted transparency registration before the verified compromise boundary may retain historical authenticity under a policy that accepts that evidence.
- A statement registered after the compromise boundary is **not automatically proven to have been signed after it**. It could have been signed earlier and submitted later. Without trusted signing-time evidence, the statement is quarantined pending review, not declared forged.
- Decisions depending on the quarantined statement are quarantined.
- Downstream authorizations, execution records and outcome observations are preserved as records but require re-verification/review of their authorization and attribution. A database update cannot undo a physical intervention.
- Unrelated source records outside the scope/dependency closure remain unaffected.

This preserves the difference between issuer signature time, TS registration time, event discovery time and the time an industrial decision was made.

## 7. Standalone auditor

The new package is `tools/industry-trust-lifecycle-auditor`. It is separate from `industry-runtime-admission` and its candidate evaluator/checker.

The `industry-trust-lifecycle-audit` binary:
1. rejects duplicate JSON object member names;
2. recomputes the graph snapshot digest;
3. verifies typed node IDs and unique nodes/edges;
4. checks all graph edge endpoints and rejects cycles;
5. verifies invalidation target node type/digest and scope;
6. checks sequence/predecessor shape, trusted timestamp reference and bitemporal policy conditions;
7. traverses the dependency DAG deterministically;
8. computes impact class, historical authenticity, current reliance and path explanation for every graph node;
9. reconciles exact per-class counts and expected fixture results;
10. writes a replayable impact report.

The current implementation deliberately accepts only the synthetic fixture format. It does not yet consume a production trust registry or general invalidation event stream. The package uses the existing pinned `serde`, `serde_json` and `sha2` dependency versions but has its own Cargo package and lockfile; an independent second implementation is still required before high-assurance promotion.

## 8. Synthetic key-compromise fixture

The frozen fixture has 17 dependency nodes and 16 edges:
- one issuer key used by two signed statements;
- a pre-boundary statement/receipt/decision/action/execution/outcome chain;
- a post-boundary statement/receipt/decision/action/execution/outcome chain;
- source and policy dependencies that are upstream rather than descendants;
- one water-domain source record outside the energy invalidation scope.

Expected 17-node impact distribution:
- 4 `no_impact`;
- 6 `historical_record_only`;
- 3 `reverify_required`;
- 1 `claim_invalidated`;
- 3 `quarantined_pending_review`.

All identities, timestamps and digests are synthetic placeholders or frozen fixture values. The fixture does not establish that a key was compromised or that a real statement was invalid.

## 9. Benchmarks and promotion gates

Benchmark dependency DAGs at 10², 10³, 10⁴ and 10⁵ nodes with shallow/deep chains, shared dependencies, high fan-out/fan-in, multiple domains and unrelated adjacent artefacts.

Mutation families:
- prospective key rotation and valid root update;
- compromise boundaries before, after, overlapping or unknown relative to trusted registration;
- late discovery with a delayed recorded/trusted time;
- rights revocation for only one purpose;
- source correction affecting only true descendants;
- model, schema, policy and verifier supersession;
- duplicated, delayed, concurrent and out-of-order invalidation events;
- forged/out-of-scope revocation authority;
- version/high-water rollback;
- graph cycles, orphan edges, duplicate typed IDs and node type confusion;
- attempt to edit or delete a historical receipt.

Promotion targets:
- 100% affected-descendant recall on frozen graphs;
- zero unrelated-node false invalidations;
- 100% rejection of forged/out-of-scope invalidation targets;
- 100% deterministic idempotency under exact replay;
- 100% detection of authority sequence rollback and conflicting same-sequence events;
- zero deletion/mutation of prior evidence;
- unknown/disputed effective time never promoted to a definitive retroactive claim;
- report p50/p95/p99 propagation latency, visited nodes/edges, memory, event bytes and re-verification workload.

Compare:
1. snapshot-only validity (no propagation);
2. naive invalidation of every descendant;
3. full graph re-evaluation;
4. dependency-aware targeted re-verification.

Measure both recall and precision. Invalidating everything is not success. The graph must find every impacted claim while leaving unrelated claims usable.

These are synthetic conformance targets, not estimated real-world compromise rates or proof of source truth.

## 10. Next stages

1. Bind invalidation statements to real SCITT Signed Statements, Receipts and authenticated timestamps.
2. Add a separately provisioned revoker/operator trust registry and threshold authorization policy.
3. Persist monotonic high-water state and test rollback, crash recovery and concurrent event ordering.
4. Consume actual ITIP source corrections, rights changes, trust/key rotations, model/version supersession and verifier build vulnerability notices.
5. Compare the Rust impact auditor with a separately developed implementation on the same frozen graph corpus.
6. Connect the resulting impact report to IEAC re-admission and domain-owned corrective-action workflows.

## 11. Claim ceiling

This protocol computes impact only from declared, authenticated inputs; the current fixture supplies synthetic inputs. It does not establish that a key was compromised, rights were revoked, a source correction is true, a model is invalid, a verifier is vulnerable or a physical intervention was unsafe. Those determinations remain with the owning security, identity, rights, data, model or operational authority and must themselves be independently evidenced.


## 9.1 Implemented synthetic impact scale ladder

The focused Rust test `impact_propagation_scale_ladder_has_exact_precision_and_replay` now exercises the actual v0.1 graph validator and impact auditor at 100, 1,000, 10,000 and 100,000 nodes. Each generated DAG contains a single in-scope compromised issuer-key root, a deterministic binary tree of dependent execution records, and disconnected out-of-scope water-domain source records.

For every size, the test independently derives the expected affected set from the generator's parent relation, compares every reported node classification, requires zero false positives and zero false negatives, and repeats the full validation/impact computation to require an identical canonical report digest. The workflow retains the exact-head `--nocapture` output as an artifact, including node/edge counts, expected impacted/unaffected counts and one in-process validation-plus-impact timing sample.

This first scale ladder is a structural precision/determinism/performance smoke benchmark, not a field benchmark or a complete comparison against TUF/SCITT implementations. Its timing excludes fixture generation and process startup and is one sample per size; repeat runs, confidence intervals, persistent high-water-state tests, concurrent invalidation ordering, and independent implementation comparison remain later gates. The algorithm currently emits a classification for every graph node, so these timings include full output materialization rather than claiming O(affected-subgraph) total cost.


## 9.2 Append-only event-chain verification

The auditor now includes an in-memory batch verifier for authority invalidation events. It canonical-digests every event, orders delivery by the declared authority sequence, checks predecessor digests, requires contiguous sequence numbers beginning at 1, rejects reused event IDs, and enforces monotonic `recorded_at` and `trusted_at` across the ordered chain.

Exact duplicate replay of the same event ID, sequence and canonical content is idempotent. A different event at an already-used sequence, a reused ID with different content, a gap, a broken predecessor digest or backwards recorded/trusted time is rejected before impact classification. Tests cover out-of-order delivery, exact replay, conflicting same-sequence events, ID reuse, sequence gaps, bad predecessors and time rollback.

This is only an in-memory chain consistency check over supplied events. It does not authenticate the issuer signature, revoker authority, trusted timestamp proof, or persist a monotonic high-water mark across process restart. Durable anti-rollback storage and concurrent/crash-recovery behavior remain explicit follow-on work.

## 9.3 Canonical UTC time ordering

Event-chain ordering now accepts only canonical whole-second UTC timestamps in `YYYY-MM-DDTHH:MM:SSZ` form. The parser validates separators, digits, hour/minute/second ranges, calendar day ranges and leap years before lexical ordering is used. This intentionally excludes offsets and fractional seconds in v0.1 so two accepted timestamps have one representation and lexical order matches chronological order. A broader timestamp representation would require a canonical conversion layer before comparison.

Regression cases cover valid leap-day values, impossible calendar dates, hour overflow, offsets, fractional seconds and invalid months.

## 9.4 Authority scope versus dependency impact scope

Event authority scope constrains the **direct target nodes**: `validate_event` still rejects a key/source revocation whose declared domains, subjects or purposes do not cover the target. Once that target is accepted, impact propagation follows the exact typed dependency edges even when a downstream consumer has a different local domain or purpose. A water-domain decision can consume an energy-domain signed statement; excluding it solely because its own domain differs would create a false negative.

Propagation to a cross-domain descendant does not grant the revoker direct authority over that descendant. The auditor reports it as `reverify_required` or `quarantined_pending_review`; it does not mark it `claim_invalidated` merely because it is reachable. Disconnected out-of-scope records remain `no_impact`. The regression fixture exercises an energy-key → water-source → water-decision path plus a disconnected water-domain control, and the scale ladder mixes cross-domain descendants with disconnected same-/different-domain controls.

This separates three predicates that must not be collapsed: (1) whether the event authority may target the root, (2) whether an exact dependency path exists, and (3) what action/claim class is permitted for each downstream node.

## 9.5 Explicit input and graph budgets

The standalone auditor now fails closed above these v0.1 input limits: 128 MiB per fixture file, 100,000 dependency nodes, 500,000 dependency edges, 10,000 direct invalidation targets and 10,000 events per in-memory chain batch. Node/edge counts are checked before graph cloning and canonical digest calculation; target and event counts are checked before per-item traversal. Boundary tests accept the 100,000-node benchmark and reject oversized node, edge and target collections.

These are implementation resource ceilings, not a claim that all graphs of the maximum size fit every deployment's memory/time budget. Deployments should set lower operational limits where appropriate; extending beyond v0.1 requires a new bounded-memory/streaming design and fresh scale evidence.