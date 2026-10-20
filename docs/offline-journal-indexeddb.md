# Browser IndexedDB adapter v1

**Status:** first browser persistence adapter draft. The adapter is exported from `src/state/offline_journal_indexeddb.rs`; it is not yet wired into a user-facing workflow or transport client. Exact-head WASM/native CI and browser fault-injection evidence are release gates.

## Storage model

Database `sol-atlas-offline-journal`, version 2, has four key-addressed object stores (upgrade creates the new lease store for any existing v1 database):

- `operations`: operation ID → serialized `JournalEntrySnapshotV1`;
- `idempotency_keys`: idempotency key → operation ID;
- `pending_operations`: operation ID → scheduling marker for entries in `Prepared` or `OutcomeUnknown`;
- `dispatch_leases`: operation ID → versioned lease record containing a cryptographically random owner token and a wall-clock expiry.

Object stores use out-of-line keys. Recovery checks the storage key against `envelope.operation_id` using `JournalEntry::from_snapshot_for_operation`, then validates snapshot schema, canonical payload bytes, SHA-256 and state/attempt invariants. The pending store is only a scheduling index: every result is reloaded and validated, and a stale/missing index is surfaced rather than treated as evidence that an operation is safe to send.

## Transaction semantics

### Prepare

`insert_prepared(envelope, payload_json)` first constructs the payload-bound `JournalEntry`, so malformed JCS or a digest mismatch never reaches storage. Inside one readwrite transaction covering all three stores, it checks both operation-ID and idempotency-key indexes. Identical insertion of the same envelope/canonical payload is idempotent and returns the stored state; operation-ID reuse with changed immutable content or an idempotency key pointing elsewhere is rejected. A new record, idempotency mapping and pending marker are added in the same transaction.

### Dispatch boundary

`begin_attempt(operation_id)` reads and validates the snapshot and lease under one readwrite transaction scoped to the operation, pending and dispatch-lease stores. If a still-active lease exists, it refuses to issue a second permit. Otherwise it writes a cryptographically random lease token with a 30-second expiry, the `OutcomeUnknown` snapshot, attempt count and pending marker in the same transaction. It returns a `DispatchPermit` **only after awaiting the transaction's `complete` event**. The permit contains the canonical payload bytes, digest, operation identity, attempt count, owner token and expiry. A transport should accept this permit and send its stored bytes rather than reconstructing a request from UI state. If `crypto.randomUUID()` is unavailable, lease creation fails closed.

`renew_dispatch_lease(&mut permit)` extends the claim only while the stored owner token still matches and the lease has not expired. A suspended/stale holder cannot revive an expired lease. The caller must renew before the deadline when a legitimate request may exceed 30 seconds; this function does not start a heartbeat by itself.

A permit is not a network result and is not proof that the payload reached the server. This draft does not include the network transport, cross-tab request scheduler, or server endpoint. The lease suppresses concurrent permits while active but cannot guarantee exactly-once dispatch: a suspended tab can resume after lease expiry, and a process can crash after sending but before receipt reconciliation. Server-side idempotency is still mandatory.

### Reply reconciliation

`apply_reply` loads the operation snapshot inside a readwrite transaction, rehydrates it, calls the pure reducer, then writes the updated snapshot, updates/deletes the pending marker, and deletes the local dispatch lease atomically. A mismatched reply or invalid state transition causes no journal write. The receipt binding still is only correlation: it does not authenticate the service. A future transport/service layer must authenticate the service and make signed receipt verification available when receipts are intended to be independently auditable.

### Recovery reads

`load` validates every recovered row. `list_pending(after, limit)` enumerates the scheduling index, reloads each candidate and verifies that it is still pending; the returned page is not snapshot-isolated because records may change while pages are fetched. A scheduler must restart full recovery scans from the beginning and call `begin_attempt` before any dispatch. A missing operation referenced by the pending index is a hard error rather than silently skipped.

## Durability truthfulness

Every readwrite transaction requests `durability: "strict"` where the browser accepts the transaction-options overload. If the overload throws a `TypeError`, the adapter retries without options and records `BrowserDefaultFallback`. `DispatchPermit::durability_policy()` reports the policy used for that permit's pre-dispatch transaction. No strict-durability guarantee is claimed when the fallback is in effect.

The browser API describes strict durability as a request to verify writes have reached a persistent medium before completion. It is still a hint, not a backup: origin eviction, user data clearing, profile corruption, disk failure, device loss and browser bugs remain possible. Transaction `complete` is the commit boundary available to the application; it must be observed before issuing the permit. Never depend on `unload` handlers to flush journal rows.

References:
- [MDN: IDBDatabase.transaction](https://developer.mozilla.org/en-US/docs/Web/API/IDBDatabase/transaction)
- [MDN: IDBTransaction complete event](https://developer.mozilla.org/en-US/docs/Web/API/IDBTransaction/complete_event)
- [MDN: Using IndexedDB](https://developer.mozilla.org/en-US/docs/Web/API/IndexedDB_API/Using_IndexedDB)

## Required conformance and fault tests

This draft adds no claim that the following browser-fault tests have run. They are the merge-to-production gate for the storage adapter:

| ID | Fault / invariant | Required oracle |
|---|---|---|
| IDB-01 | Identical insert replay | One immutable operation identity; attempts/state/receipt never reset |
| IDB-02 | Operation ID reused with different envelope or body | Explicit collision, no row/index mutation |
| IDB-03 | Idempotency key reused for another operation | Explicit collision, no duplicate key mapping |
| IDB-04 | Failure during any one of the three store writes | Whole transaction aborts; none of the snapshot/key/pending writes remain |
| IDB-05 | Crash before attempt transaction commits | No `DispatchPermit`; no network dispatch |
| IDB-06 | Crash immediately after attempt commit, before socket write | Recovery sees `OutcomeUnknown`; retry uses exactly the same canonical bytes, digest and idempotency key |
| IDB-07 | Server apply followed by lost response | Server retry/receipt lookup returns prior outcome; one logical server side effect |
| IDB-08 | Crash/abort while applying terminal reply | Recovery sees prior unresolved state or full terminal state, never a partial snapshot/index update |
| IDB-09 | Snapshot stored under wrong primary key | `from_snapshot_for_operation` rejects the record |
| IDB-10 | Quota exhaustion, blocked upgrade or database close | Typed error; never hand out a dispatch permit after an uncertain or aborted transaction |
| IDB-11 | Two tabs begin attempt or reconcile simultaneously | No state regression; immutable identity and receipt binding stay valid; duplicate logical effects remain zero at service |
| IDB-12 | Strict durability unsupported | Fallback is explicit and surfaced to diagnostics; no claim of strict behavior |
| IDB-13 | Stale/missing pending index | Error or a full recovery rebuild path; no silently lost unresolved operation |
| IDB-14 | Malformed/noncanonical/digest-mismatched stored snapshot | Recovery rejects before the scheduler can dispatch or report receipt |
| IDB-15 | Two tabs claim same operation concurrently | A browser test starts two readwrite claims from separate connections; at most one active lease is committed and at most one permit is returned while that lease remains active |
| IDB-16 | Lease expiry / suspended tab | A second tab cannot claim before expiry but can reclaim after expiry; explicitly demonstrate that a paused original tab can resume later, so server-side idempotency remains mandatory |
| IDB-17 | Missing secure UUID source | No fallback to predictable tokens; claim fails closed without mutating attempt count or pending state |
| IDB-18 | Stale lease renewal | Renewal succeeds only with the current token before expiry; an expired lease or superseded token cannot be revived |

## Benchmarks

Do not invent an IndexedDB baseline. Measure on named supported browsers and at least one representative field device after conformance tests pass:

- cold recovery of 1k, 10k and 100k records; publish p50/p95/p99 and raw samples;
- insert and pre-dispatch commit latency, with requested/observed durability policy recorded;
- page fetch scaling at sizes 50/100/500, plus memory allocated if measurable;
- transaction-abort recovery rate and time;
- canonical payload size/bytes per operation, with 1 KiB, 32 KiB, 256 KiB and 1 MiB payload cohorts.

Integrity gates are absolute: zero sends after failed/indeterminate pre-dispatch commits, zero lost acknowledged transitions in the injected crash matrix, zero mismatched payloads dispatched, zero duplicate logical service effects, zero missing index rows silently skipped, and zero snapshot-key mismatches accepted. Performance does not compensate for an integrity failure.

## Explicit limits

- This adapter is currently a draft and is not wired into the product's UI or a live API.
- Pure lease-policy tests cover expiry boundaries and malformed bindings, but the adapter has not been browser-tested; exact-head WebAssembly compilation plus two-tab/crash fault tests remain required.
- No service authentication, signed receipt, authoritative server idempotency table, server payload digest verifier, network retry layer, automated lease heartbeat, encryption, export/backup or device-loss recovery is included. The proposed service contract and end-to-end acceptance gates are in `docs/offline-journal-server-contract.md`.
- IndexedDB does not provide an unconditional power-loss guarantee simply because the application observed a completed transaction.
