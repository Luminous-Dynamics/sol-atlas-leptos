# Offline Journal Persistence and Crash-Consistency Contract

**Status:** adapter contract and fault-injection plan; no durable backend is implemented by this document or by `offline_journal_store.rs`.  
**Depends on:** [Offline Journal Protocol v2](offline-journal-protocol.md)  
**Contract module:** `sol-atlas-core/src/offline_journal_store.rs`

## Why this is the next boundary

The reducer can reject a receipt that belongs to another request, but it cannot establish that the journal row and queue indexes survive a crash, that two tabs cannot race, or that a server applied an operation only once. The next architectural step is not a generic `save(entry)` API. It is a transaction-shaped storage port whose public methods make the safe ordering difficult to bypass.

The new `OfflineJournalStore` port therefore exposes only validated operation insertion, a pre-dispatch attempt transition, reply reconciliation, reads, and bounded pending pages. It deliberately does not accept an arbitrary caller-mutated `JournalEntry`.

## Required storage semantics

Every backend must provide these guarantees before it can be called a durable adapter:

1. **Atomic insert and uniqueness.** Insert the immutable envelope and its initial `Prepared` state in one transaction. Enforce uniqueness of operation IDs and idempotency keys across the entire journal. Repeating insertion with the same operation ID and exactly equal envelope returns the existing entry without resetting its state; identity reuse with changed envelope or a reused key fails without mutation.
2. **Persist before dispatch.** `begin_attempt` atomically changes `Prepared` to `OutcomeUnknown` and increments attempts, then returns only after its documented commit boundary. The caller must not perform network I/O before that return. A failed or indeterminate commit is not permission to send.
3. **Atomic reconciliation.** Read the current row, run the pure reducer against a private copy, and commit its state in one transaction. Never persist a rejected/misbound reply. Use serializable transaction semantics or compare-and-swap so simultaneous tabs cannot overwrite a newer state with an update based on a stale read.
4. **Durable receipt and indexes.** Terminal state, receipt reference, applied revision, and any queue/index metadata update in one atomic transaction. Recovery must not discover an operation as pending after the same durable transaction has committed it as received.
5. **Bounded recovery reads.** Return pending entries in a deterministic order with bounded pages. A cursor is not a snapshot token; each full recovery pass restarts from the beginning. A scheduling loop must not assume a single page is the entire queue.
6. **Errors remain visible.** Transaction abort, quota failure, blocked upgrade, corrupt record, schema mismatch, unavailable storage, or failed read-back is a typed storage failure. Never map a storage error to `Received`, silently drop a row, or clear the queue as a recovery shortcut.
7. **Migration is fail-safe.** Schema migration is atomic or versioned so interruption cannot leave partially migrated entries. Unknown state discriminants and unknown fields are quarantined for inspection or cause the affected record to remain unresolved; do not coerce them into a known terminal state.
8. **Authority stays server-side.** The durable adapter does not substitute for server idempotency, revision preconditions, authentication, authorization, or receipt lookup. Client-generated IDs and local state are not proof of server application or authorization.

## Crash-cut fault matrix

A backend conformance suite must inject process termination, transaction abort, transport loss, and storage failure at each cut below. Every test records the adapter, schema, exact source SHA, storage policy, fault point, expected state, and actual state.

| ID | Fault cut | Required recovery behavior |
|---|---|---|
| CUT-00 | Before the prepared insert begins | No row, no dispatch, no success indication |
| CUT-01 | During prepared insert transaction | Entire insert commits or no row is visible; retrying identical insert is safe |
| CUT-02 | After prepared commit, before `begin_attempt` | Row remains `Prepared`; no network request was sent |
| CUT-03 | During attempt-state transaction | Transaction commits fully or fails; caller sends only after successful commit |
| CUT-04 | After `OutcomeUnknown` commit, before socket write | Recovery keeps the same operation identity and can retry the same envelope |
| CUT-05 | Server applies operation, response is lost | Client remains uncertain; server-side dedupe/receipt lookup yields the original outcome on retry |
| CUT-06 | Response received, before local reconciliation starts | Journal remains unresolved; retry/lookup uses the same identity |
| CUT-07 | During terminal-state transaction | Commit is atomic; read-after-restart finds either prior unresolved state or full terminal state, never a partial terminal record |
| CUT-08 | Reply binding mismatch or future reply schema | No state mutation; visible unresolved/error state |
| CUT-09 | Two tabs attempt the same operation concurrently | One stable logical operation; no attempt-count lost update or terminal-state regression |
| CUT-10 | Idempotency key reused with changed payload or operation | Insert/server request is rejected; existing operation remains unchanged |
| CUT-11 | Quota exceeded, database closed, or transaction aborted | Explicit storage error and retained recovery path; do not claim durability or send after failed pre-dispatch commit |
| CUT-12 | Process restart while paginating pending operations | A fresh scan from the first page eventually finds every still-pending operation |
| CUT-13 | Interrupted database version upgrade | Previous or new schema is usable; no silently lost journal entries |
| CUT-14 | Corrupt or unknown-version stored record | Isolate and surface the record; never silently discard or mark received |

The oracle is stronger than “the app opens after restart.” For every operation, the test must compare pre-crash and post-recovery immutable envelopes, attempt counts, state, receipt reference, applied revision, and the fake server's logical side-effect count.

## Browser backend: IndexedDB is persistent storage, not an absolute durability promise

A browser adapter is the natural first target for the Leptos CSR application, but its contract must report what its storage system actually guarantees.

- Group an operation row and all indexes that must agree within one IndexedDB transaction. Do not clear and rebuild stores in separate transactions.
- Where supported, request `readwrite` transaction durability `strict` for critical journal transitions, and observe transaction completion/abort. Do not silently present the same guarantee when only default or relaxed durability is available.
- Feature-detect transaction options. Record the requested policy and observed browser/runtime in diagnostics; tests must cover browsers where the option is unsupported.
- Consider requesting persistent storage through the Storage API, but treat denial, quota exhaustion, user data clearing, private browsing, device loss, and browser/profile corruption as real failure modes. Persistence permission is not a backup.
- Avoid relying on unload handlers to flush data. Commit before showing a task as durably queued or before dispatching its network request.
- Keep sensitive payload fields minimized; browser-origin isolation is not encryption against same-origin script compromise. Encryption keys, retention, logout, shared-device, and recovery behavior require a separate threat model.

The IndexedDB API documents a durability hint (`strict`, `relaxed`, or `default`), but the selected hint is not a substitute for fault injection against the actual browser and storage hardware. See [MDN: IDBDatabase.transaction](https://developer.mozilla.org/en-US/docs/Web/API/IDBDatabase/transaction), [MDN: transaction durability](https://developer.mozilla.org/en-US/docs/Web/API/IDBTransaction/durability), and [MDN: IndexedDB terminology and limitations](https://developer.mozilla.org/en-US/docs/Web/API/IndexedDB_API/Basic_Terminology).

## Native/reference backend

A native SQLite backend can serve as a deterministic test and field-device option if it is appropriate for the eventual deployment. Evaluate WAL, transaction isolation, `synchronous` settings, checkpointing, backup/restore, and filesystem/device behavior explicitly. Do not treat “SQLite with WAL” alone as proof that every acknowledged write survives sudden power loss. Keep the database and its WAL together during backup and recovery. See [SQLite Write-Ahead Logging](https://www.sqlite.org/wal.html).

The browser and native adapters should implement the same conformance suite, with storage-specific expected durability claims. Do not compare throughput until configurations and durability policies are equivalent.

## Benchmarks and acceptance gates

| ID | Acceptance gate | Measurement procedure |
|---|---|---|
| PERSIST-01 | Zero acknowledged local transitions lost in tested crash cuts | Repeat every CUT-00–CUT-14 across cold restart and abrupt process termination; publish denominator and failures |
| PERSIST-02 | Zero sends after a failed or indeterminate `begin_attempt` commit | Inject transaction abort, timeout, and read-back failures; instrument the transport boundary |
| PERSIST-03 | Zero duplicate logical server effects | Fault-inject after server commit/before response; replay same key and digest; assert side-effect counter equals one |
| PERSIST-04 | Zero silent mutation on corrupt, unknown, or misbound data | Parser/storage corruption corpus; compare exact serialized state before and after rejected input |
| PERSIST-05 | 100% conflict-resolution lineage integrity | Validate original operation, new operation ID/key, superseded ID, remote base revision, and reason |
| PERSIST-06 | Zero lost updates under two-tab races | Concurrent attempt/reconcile loops with deterministic barriers and randomized schedules |
| PERF-01 | p95 cold recovery of 10,000 pending operations under 250 ms | **Initial hypothesis only**, measure per named browser/device/backend in optimized builds, publish raw samples and percentile method |
| PERF-02 | Bounded-page query scales with returned rows rather than loading the full journal | Compare 1k/10k/100k entries, vary page size, report p50/p95/p99, allocated bytes if measurable, and query-plan/index evidence |
| UX-01 | At least 95% of interrupted scripted tasks recover without data loss or a silent state change | Fixed task script and denominator; usability result stays separate from integrity gates |

Integrity gates are absolute pass/fail requirements, not percentages or weighted scores. Performance targets are hypotheses until measured; do not average a data-loss failure away with a fast query.

Every benchmark artifact must include:
- source commit SHA and test/fixture digest;
- storage backend, schema version, durability request and observed support;
- device, operating system, browser/runtime, build mode and filesystem/storage medium where available;
- fault schedule seed, sample count, raw results, and p50/p95/p99 calculation method;
- failure details and counterexamples, not just the final score.

## Comparison against industry practice

Microsoft Dynamics 365 Field Service documents configurable offline conflict behavior, including record/table-level conflict handling. ServiceNow Field Service documents offline task execution and an outbox of actions taken while offline. These are established capabilities and should remain part of the baseline comparison, not straw-man competitors. See [Microsoft Learn](https://learn.microsoft.com/en-us/dynamics365/field-service/mobile/offline-data-sync) and [ServiceNow: Working offline](https://www.servicenow.com/docs/r/field-service-management/work-order-management/work-offline-fieldservice-mobile.html).

The differentiator worth proving is not “offline mode exists.” It is an independently executable protocol contract plus crash-consistency evidence: exact receipt binding, server dedupe under response loss, durable pre-send ordering, concurrent-tab safety, and transparent evidence lineage.

A fair head-to-head needs equivalent task models, data sizes, device/storage constraints, network-fault schedules, and defined outcomes. Vendor product documentation is not enough to infer hidden transactional guarantees. Until identical-workload measurements exist, claim a protocol/testability advantage only where demonstrated and do not claim overall superiority to Dynamics, Salesforce, or ServiceNow.

## Explicit non-goals for this increment

- No browser IndexedDB implementation is included here.
- No server endpoint, durable idempotency table, cryptographic payload canonicalizer, transport retry loop, encryption, or device-loss backup is implemented here.
- No performance result has been measured by this contract.
- This contract can constrain future implementations; it does not prove a future adapter satisfies it. The adapter must pass the exact same conformance and fault-injection suite before being enabled in production.
