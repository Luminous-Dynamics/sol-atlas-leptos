# IEAC evidence-channel completeness v0.1

**Status:** Structural channel-manifest and completeness-attestation contracts with a synthetic four-stage checkpoint/query fixture. Not yet a production access-log verifier.  
**Issue:** #419 — IEAC-AVAIL-001.  
**Composes with:** decision frontier #413 / PR #415; SCITT statement history #375; event-history auditor #382; witness observations #387; bitemporal lifecycle #396 / PR #397.

## 1. The bounded claim

A post-cutoff access event proves that an artifact was accessible by that time. It does not prove that the artifact was inaccessible earlier.

To classify an artifact as *first available after* a decision cutoff, the relying party needs an independently verified record that covers the full pre-decision interval and binds the later first-access event.

This profile makes only a bounded claim:

> Under this declared relying-party, tenant, subject, purpose, event taxonomy and access-capture policy, a signed query found zero qualifying access events from the coverage-start checkpoint through a decision-boundary checkpoint that is later than the decision cutoff; the first qualifying access event occurred afterward and is included before the coverage-end checkpoint.

It is not a universal statement that the artifact did not exist, was not accessible through another path, or was never disclosed by its source.

## 2. Four-stage sequence

The attestation requires four ordered stages:

1. **Coverage-start checkpoint** — its latest possible time is no later than the earliest possible decision cutoff.
2. **Decision-boundary checkpoint** — its earliest possible time is strictly later than the latest possible decision cutoff. This closes the entire relevant pre-decision interval.
3. **Signed zero-match query** — a query for the exact relying-party/channel/tenant/subject/purpose/artifact/event-type predicate, over sequence interval `(coverage_start.sequence, decision_boundary.sequence]`, returning exactly zero qualifying events.
4. **First-access event and coverage-end checkpoint** — the first qualifying event is after the decision boundary, has a receipt binding the exact artifact, and is included no later than the end checkpoint.

Two consistency receipts bind start → decision-boundary and decision-boundary → end. All three checkpoints, the query and event share the same ledger, sequence namespace and epoch.

Synthetic example:

| Evidence point | Sequence | Time (UTC) |
|---|---:|---|
| Coverage start | 90 | 08:00:00 |
| Decision cutoff | — | 09:10:00 ± 30s |
| Decision boundary | 100 | 09:11:00 |
| First access | 110 | 09:25:00 ± 2s |
| Coverage end | 120 | 09:30:00 |

The zero-match query is bound to sequence range `(90, 100]`. The first access event is beyond that range and included before the end checkpoint.

## 3. Channel manifest

`industry-ieac-evidence-channel-manifest-v0.1.schema.json` declares:
- relying-party, channel, log service, operator, tenant, subject and purpose identities;
- exact event types that count as access (ingest, read, cache hit, replica fetch, export, delegated access and others);
- capture-policy digest/version, instrumented paths, unmonitored paths and known bypass paths;
- durability, event-loss handling and policy-change semantics;
- ledger, sequence namespace, epoch, monotonicity, checkpoint cadence, retention and reset semantics.

A channel cannot claim completeness for paths it does not instrument or explicitly exclude. Any unmonitored/bypass path that can expose the artifact blocks a complete absence claim for that scope.

## 4. Completeness attestation

`industry-ieac-availability-completeness-attestation-v0.1.schema.json` binds the exact channel-manifest digest and repeated channel identity; tenant, subject pseudonym, purpose and artifact digest; decision cutoff and coverage interval; all three checkpoint identities, roots, exact signed-byte digests, receipts, ledger/namespace/epoch and sequence; both consistency receipts; first-access event digest, artifact, type, sequence, time and inclusion receipt; and the zero-match query identity, explicit predicate binding, sequence interval, zero result, signed statement, signer identity, signature digest, receipt and verifier facts. The channel manifest pins an allow-list of authorized completeness-query issuers; a correctly signed query from an unapproved issuer is not admissible.

For a `complete` result, hard requirements are:
- all mandatory time, query, checkpoint and continuity verification facts are independently verified under pinned build/policy identities;
- coverage-start checkpoint is at or before the earliest possible decision cutoff;
- decision-boundary checkpoint is strictly after the latest possible decision cutoff;
- coverage interval spans the start through the first-access event;
- query range exactly equals the start-to-boundary sequence interval and has zero matches;
- first-access sequence is greater than the boundary sequence and no greater than end sequence;
- all checkpoints and events share the same ledger, namespace and epoch;
- no dropped events, sequence gaps, unresolved outages, backfills, policy changes, bypass paths, retention loss or degraded clock health are declared.

The workflow cross-checks relationships that JSON Schema cannot express across objects/files. The schema validates structure; it does not authenticate the signer, TS receipt, log operator or query result.

## 5. VDS interoperability boundary

Do not confuse consistency with non-inclusion.

RFC 9942 describes COSE Receipts generically and notes that VDS proof formats differ. Its concrete `RFC9162_SHA256` profile defines inclusion and consistency proofs; do not treat those receipts as a non-inclusion proof.

This v0.1 fixture uses a separately signed, verifier-checked channel query for the exact pre-decision sequence interval. It uses VDS consistency receipts only to link checkpoints. A deployment must pin the actual VDS proof-type registry and verifier support matrix. A VDS that does not explicitly support a required proof type is refused for that claim.

A signed zero-match query is only as trustworthy as its query engine, capture policy, operator authority, retention, and verifier. An opaque digest is not a completeness proof.

## 6. Frozen synthetic corpus

The initial corpus contains one strict channel manifest, one synthetic complete attestation, checkpoints at sequences 90/100/120, one first-access event at sequence 110, and one signed-query placeholder covering `(90, 100]`.

Every digest and proof is synthetic placeholder data. No real keys, receipts, log roots, query signature or operational access events are supplied. The fixture demonstrates structure and relationships only.

## 7. Promotion benchmarks

**Safety / completeness:** 100% fail-closed detection of sequence gaps, rollback, same-sequence equivocation, reset, policy changes and scope substitution; zero complete status with outage, bypass, retention loss, dropped event, backfill, degraded clock or unknown proof state; 100% binding among manifest, query predicate/range, checkpoint chain, first-access event, relying party and artifact digest; zero use of RFC9162_SHA256 consistency receipts as non-inclusion proofs; deterministic, idempotent replay.

**Utility:** report completeness precision/recall, false-late-exclusion rate, unknown/abstention, channel coverage, query failure, reason-code precision/recall; stratify by relying party, tenant, event type, operator, VDS, outage and bypass. Rejecting every case is not success: positive complete cases must be recognized while planted incomplete cases remain unknown.

**Scale/cost:** sequences at 10², 10³, 10⁴, 10⁵ and 10⁶ records; p50/p95/p99 query and proof latency; checkpoint/receipt/query bytes; CPU, memory, storage, egress and retention overhead; rotation, outage, backfill and recovery behavior.

**Baselines:** source timestamp only; positive post-cutoff access only; lower/upper checkpoint consistency only; complete channel attestation with exact signed query; same data with injected bypasses, event loss, gaps and policy changes.

These are synthetic conformance targets, not field risk probabilities. Production claims require independent channel-owner review and operational measurement.

## 8. Claim ceiling

A complete attestation supports only **absence of a qualifying event within the named, authenticated channel, capture policy, scope and sequence interval**. It does not prove universal non-access, source truth, absence in undisclosed systems, complete issuer disclosure, or decision correctness. No action authority is created.

## 9. Next engineering increment

Implement a standalone Rust checker that consumes independently verified checkpoint/query facts, cross-binds identities and intervals, and returns `complete`, `partial`, `unknown` or `invalid`. Do not reimplement SCITT COSE or Merkle cryptography. Add an independently authored oracle and compare it against the candidate checker on sequence/gap/bypass controls.
