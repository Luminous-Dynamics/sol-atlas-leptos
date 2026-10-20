# Offline Journal Snapshot v1

**Status:** a validated, versioned Rust snapshot representation and recovery validator are implemented in this branch. No durable IndexedDB/SQLite adapter is included.

## Snapshot contents

`JournalEntrySnapshotV1` stores a versioned tuple:

- `schema_version`: snapshot serialization version, independent of the service envelope schema;
- immutable `OperationEnvelope`;
- `canonical_payload_json`: the exact canonical RFC 8785 JSON body whose SHA-256 is in the envelope;
- `JournalState`;
- `attempt_count`.

The snapshot is a transport/storage representation, not authoritative evidence. Parsing the snapshot must not itself promote an operation to received or authorize sending.

## Export and recovery rules

`JournalEntry::to_snapshot` exports the internal immutable payload bytes alongside the envelope and state. A storage adapter must atomically persist this snapshot with any indexes required to find the operation.

`JournalEntry::from_snapshot` fails closed unless all of the following are true. If the adapter loaded the snapshot from a key-addressed store, it must use `JournalEntry::from_snapshot_for_operation(expected_id, snapshot)` so the storage key and `envelope.operation_id` are independently checked for equality:

1. The snapshot schema version is recognized.
2. The envelope validates under the declared operation protocol.
3. The payload is valid under the strict JCS profile.
4. Re-canonicalizing the stored payload returns the exact same UTF-8 bytes; otherwise the stored string is valid JSON but not a canonical snapshot.
5. The recomputed SHA-256 equals the envelope digest.
6. Stored state and attempt count are compatible: `Prepared` has zero attempts; `OutcomeUnknown` and terminal states have at least one potentially dispatched attempt; receipt/conflict revisions and reason/reference fields are well formed.

Unknown snapshot fields, unknown state discriminants, unsupported versions, mismatched digests, or inconsistent state are errors, not best-effort defaults. The implementation intentionally does not coerce an old or corrupt record into `Received`.

## Required fault tests

The frozen/current conformance suite covers:
- round-trip of an `OutcomeUnknown` entry with exact payload-byte preservation;
- round-trip of a terminal receipt state;
- non-canonical JSON bytes;
- a syntactically valid but mismatched payload digest;
- attempt/state mismatch;
- unknown snapshot version;
- unknown state discriminant and unknown snapshot fields.

These are pure-Rust serialization and rehydration tests. They do not prove browser transaction durability, power-loss survival, simultaneous-tab isolation, or server idempotency. Those need a concrete adapter plus process-crash/transaction-abort fault injection.

## Next persistence step

Implement the browser IndexedDB adapter in the Leptos application with two hard ordering rules: persist the complete snapshot transactionally before dispatch, and resend the exact stored canonical body on every retry. Persisting the envelope and payload in separate transactions is not acceptable. The transaction must not be reported successful until its completion is observed, and the backend must report the requested/available durability policy rather than imply guarantees it cannot provide.

For critical state, request IndexedDB transaction durability `strict` where supported, feature-detect that support, and report the runtime/storage mode. This remains a browser durability hint; storage eviction, user clearing, profile corruption and device loss still need explicit recovery behavior. See [MDN: IDBDatabase.transaction](https://developer.mozilla.org/en-US/docs/Web/API/IDBDatabase/transaction), [MDN: IndexedDB basic terminology](https://developer.mozilla.org/en-US/docs/Web/API/IndexedDB_API/Basic_Terminology), and [MDN: Using IndexedDB](https://developer.mozilla.org/en-US/docs/Web/API/IndexedDB_API/Using_IndexedDB).

## Acceptance gates

| ID | Gate | Evidence |
|---|---|---|
| SNAP-01 | 100% valid snapshot round-trip equivalence | Compare full immutable envelope, state, attempt count, payload digest and canonical bytes before/after encode/decode |
| SNAP-02 | Zero corrupt/non-canonical/misbound snapshot recovery | Mutation corpus must yield an explicit error and never a constructed entry |
| SNAP-03 | Zero payload-byte drift across recovery/retry | Recovered canonical bytes exactly equal the original prepared entry |
| SNAP-04 | Zero impossible state/attempt combinations restored | Exercise every state with valid/invalid attempt counts and revision invariants |
| SNAP-05 | Zero unknown-schema/unknown-state promotion | New schema/discriminant and unknown fields remain unresolved errors |
| SNAP-06 | Zero storage-key misattribution | Mutation-test a database row under a different primary key; recovery rejects the snapshot unless the storage key equals the immutable operation ID |
| PERF-01 | Snapshot encode/decode latency | Measure 1k, 10k and 100k entries on named reference runtimes; report p50/p95/p99 and raw samples |
| PERF-02 | Recovery throughput | Measure cold recovery in release builds on each target browser/device; keep correctness failures as hard fail, never averaged against speed |

The initial performance target should be selected after measuring a baseline on the intended field device. Integrity gates are absolute; throughput is not allowed to compensate for data loss or a false acknowledgement.
