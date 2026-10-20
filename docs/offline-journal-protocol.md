# Offline Journal Protocol v2

**Status:** deterministic core primitive and synthetic fault corpus; not a production sync implementation  
**Module:** sol-atlas-core/src/offline_journal.rs  
**Fixture:** sol-atlas-core/fixtures/offline-journal-fault-corpus-v2.json

## Design change

A network request is not a state transition to success. The client must first durably record the immutable operation envelope, then durably mark the dispatch outcome as unknown before network I/O. Only an authoritative service response can resolve that uncertainty. If the response is lost, the client retries the **same operation identity and payload**, not a newly reconstructed request from mutable form state.

This module is deliberately a pure decision layer. It does not implement a database, encryption, authentication, a network transport, or server-side idempotency. Those boundaries must be supplied and independently tested before production use.

## Protocol invariants

1. **Immutable retry identity.** Each operation carries a stable operation ID, aggregate ID, expected revision, canonical payload SHA-256, and idempotency key. A retry candidate must match the complete original envelope. The service must bind an idempotency key to the original payload digest and reject reuse with different content.
2. **Persist before send.** Persist Prepared; then persist OutcomeUnknown and the incremented attempt count; only then dispatch. A crash after that commit is treated as uncertain even if the process had not yet written bytes to the socket.
3. **Receipt is bound evidence.** Every authoritative reply echoes the exact envelope schema version, operation ID, aggregate ID, expected revision, idempotency key, payload digest, and optional `supersedes_operation_id` (including an explicit null) from the immutable request. The client rejects a reply whose binding differs, even if its receipt shape otherwise looks valid. The request envelope and echoed binding must both include `supersedes_operation_id` explicitly as `null` or a value; omission on either side is malformed, not equivalent evidence. The client reaches Received only from a matching authoritative reply with a receipt reference and an acceptable applied revision. A timeout or malformed/unrecognized response leaves the outcome unknown.
4. **No terminal-state rewriting.** Duplicate identical receipts are idempotent. A different receipt, a rejection after receipt, or another contradictory terminal reply is an error rather than a silent replacement.
5. **Conflict is preserved.** A conflict records the remote revision and a reason code. A resolution is a new operation with a new operation ID and idempotency key, bound to the remote revision and explicitly referencing the superseded operation. The original remains available for audit.
6. **Unknown future states fail closed.** If decoding a future or malformed service reply fails, retain the current journal state and surface an unresolved outcome; never infer success from a parse failure.

## Required durable-adapter behavior

The implementation that persists this model must additionally guarantee:

- Atomic local commit and read-back before dispatch, including across process termination and device restart.
- Unique constraints for operation IDs and idempotency keys across the entire journal.
- Durable server-side deduplication, keyed to the idempotency key and payload digest, with a receipt that can be retrieved after the original response is lost.
- Revision preconditions checked by the canonical service, not trusted from the client.
- A recoverable queue state for rejected, superseded, and conflict-needs-review operations.
- Authentication and authorization reevaluated at the authoritative boundary; an offline draft is not authorization.
- Evidence corrections are appended with provenance instead of deleting the original claim.
- Encryption at rest, scoped local data retention, safe device-loss recovery, and privacy reviews before storing sensitive site data.

A local reducer cannot prove these database, transport, security, or server properties. They require adapters, integration tests, and fault injection.

## Frozen fault corpus

The JSON fixture contains eight synthetic cases: timeout/response loss, retries, duplicate authoritative receipt, explicit rejection, concurrent revision conflict, contradictory terminal response, a conflict reply that does not advance revision, and a receipt whose operation binding is wrong. Rust unit tests replay the corpus against the reducer, assert the exact final state and assert the response classification. These are conformance fixtures, not telemetry or field-test observations.

## Benchmarks to publish

| Benchmark | Gate or initial target | Evidence required |
|---|---|---|
| JRN-01: ambiguous outcome | Zero transitions from unknown to received without a valid authoritative receipt | Frozen reducer corpus plus malformed/unknown reply tests |
| JRN-02: duplicate delivery | Zero duplicate logical effects across the idempotency integration suite | Server-side dedupe test and side-effect counter; reducer tests alone are insufficient |
| JRN-03: retry mutation | Zero requests that reuse identity while changing digest, aggregate, or expected revision | Property tests over mutated envelopes |
| JRN-04: conflict preservation | Zero silent overwrites; every conflict remains inspectable and recoverable | Two-writer revision races, process restarts, and audit-log assertions |
| JRN-05: crash recovery | Zero lost acknowledged local operations across every durable transition | Kill/restart fault injection at each write boundary on target devices |
| JRN-06: unknown protocol data | Zero malformed or future replies interpreted as success | Parser corpus including missing fields, unknown discriminants, and truncated JSON |
| JRN-08: reply/request binding | Zero replies applied when any operation ID, aggregate, expected revision, idempotency key, or payload digest differs | Mutate all seven binding fields independently (including conflict lineage) and verify the entry remains unresolved |
| JRN-07: evidence lineage | 100% of corrected operations link to the prior operation, remote revision, and resolution reason | Graph/reference integrity verifier |
| PERF-01: local queue recovery | p95 rebuild of 10,000 journal entries under 250 ms on a named reference device (initial hypothesis) | Release build, pinned device/storage, cold and warm cache, raw sample distribution |
| PERF-02: reconnect usability | At least 95% of interrupted synthetic tasks recover without data loss or silent state change | Controlled task protocol and participant/task denominator |

The numerical performance target is an initial hypothesis, not a measured result. Measure the current baseline on a named device and storage backend before claiming superiority. Publish the source SHA, fixture digest, run command, raw samples, hardware/runtime, and percentile method with every benchmark.

## Comparison with current industry practice

Official product documentation shows that offline operation is already a standard capability in enterprise field-service apps. The opportunity is not to claim "we support offline too"; it is to define and test the semantics of uncertain delivery, retries, conflicting edits, audit history, and recovery.

| Platform/documented behavior | Published baseline to compare against | Proposed differential to implement and verify |
|---|---|---|
| Microsoft Dynamics 365 Field Service | Offline data synchronization and conflict handling; configured conflict behavior can resolve at a table/record level. | Typed, field-aware conflict classification; preserve the conflicting revisions and require review for critical fields. |
| Salesforce Field Service Mobile | Offline priming, queued drafts and sync; its documented help says the most recently uploaded server change takes precedence, and a conflict can pause the pending-upload queue until resolved. | Keep immutable operation identity; do not turn ambiguous delivery into success; resolve a conflict with a new revision-bound operation while retaining the original. |
| ServiceNow Field Service Management | Cached offline work, later synchronization, and an outbox listing actions taken while offline with timestamps. | Publish a recoverable operation journal with explicit per-operation state, receipt references, duplicate-replay tests, and auditable resolution lineage. |

References: [Microsoft Learn — Configure offline data synchronization](https://learn.microsoft.com/en-us/dynamics365/field-service/mobile/offline-data-sync); [Salesforce Help — Offline Priming FAQ, draft and conflict handling](https://help.salesforce.com/s/articleView?id=sf.mfs_offline_questions.htm&language=en_US&type=5); [ServiceNow docs — Working offline](https://www.servicenow.com/docs/r/field-service-management/work-order-management/work-offline-fieldservice-mobile.html).

These references document selected product behaviors, not complete vendor architectures or every configurable mode. This PR has not been benchmarked head-to-head against the products above and makes no claim that it already beats them. A fair result requires the same workload, equivalent device/storage/network profiles, two-writer conflicts, applied-but-response-lost cases, duplicate delivery, restart/fault injection, and a transparent scorecard. Only claim superiority where our raw results demonstrate it.

## Current limitations

- No durable storage adapter or transactional write-ahead log.
- No end-to-end service idempotency implementation.
- No cryptographic digest computation in this module; the envelope validator checks SHA-256 encoding only. The canonicalizer and digest verifier belong in the adapter.
- No cross-process uniqueness registry, encryption, server authorization, or real field telemetry.
- No Rust test or browser execution is claimed until the exact commit has a completed validation run.
