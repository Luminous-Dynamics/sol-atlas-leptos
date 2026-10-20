# Portable offline journal bundle v1

**Status:** shared Rust bundle codec, validation tests, consistent read-only IndexedDB export, and isolated staging of untrusted imports are implemented. Server reconciliation, safe activation/promotion, encryption, source authentication and a user-facing workflow are not implemented.

## Problem and design shift

An offline-first journal should not be tied to one browser profile or one physical device. But “copy the database file” is not a portable protocol: browser stores may not be directly exportable, schema versions differ, and a copied pending row can be mistaken for server-authoritative truth.

Bundle v1 treats export as a versioned protocol artifact. It carries a single owner scope, each operation's immutable namespace and validated JournalEntrySnapshotV1, and an integrity digest over the canonical manifest. Import must verify the entire bundle before performing any local mutations.

## Wire model

The canonical JSON bundle contains:

- schema_version: bundle schema version 1, distinct from journal envelope and snapshot versions;
- owner_scope: original opaque account/tenant scope;
- records: records sorted by snapshot.envelope.operation_id, each containing operation_namespace and a full snapshot, including original canonical payload bytes, journal state and attempt count;
- integrity_sha256: lowercase SHA-256 over RFC 8785 canonical UTF-8 bytes of the unsigned manifest (schema_version, owner_scope, records).

The digest is stable across insignificant JSON whitespace and object-key ordering. Record ordering is explicitly normalized and then enforced. Duplicate operation IDs, unsupported schemas, empty scopes/namespaces, malformed state/attempt combinations, noncanonical payloads and payload-digest mismatches are rejected.

Rust API: sol_atlas_core::offline_journal_bundle::{OfflineJournalBundleV1, OfflineJournalBundleRecordV1}.

The IndexedDB adapter exposes `export_bundle(owner_scope)`. It reads operation keys and values within one readonly transaction, validates every stored owner-bound record and its snapshot, filters to the requested owner, and delegates sorting/checksum creation to the core codec. This gives the export one consistent database read view rather than composing a bundle from many individually timed reads. It returns plaintext canonical JSON and does not change local data.

- OfflineJournalBundleV1::create(owner_scope, records) validates, sorts and seals the bundle.
- to_json() returns canonical JCS JSON only after validation.
- from_json(input) rejects unknown fields, validates every snapshot, and verifies the manifest digest.
- validate_for_owner(authenticated_owner_scope) requires equality with an independently supplied current account scope. The bundle's self-declared scope is never identity proof.

## Import protocol requirements

The codec intentionally performs validation only. A production restore adapter should use a single write transaction and follow this order:

1. Parse and validate the whole bundle before opening a write transaction.
2. Obtain the destination owner scope from the authenticated account/session layer, and require validate_for_owner to succeed. Never “adopt” an unrecognized bundle by assigning the currently logged-in user to it.
3. Preflight every operation ID, scoped idempotency key, namespace and immutable envelope/payload identity against existing records. Exact duplicates may be treated idempotently; any differing collision aborts the entire import before changes.
4. Merge records without resetting state or attempt count. Recreate derived pending-index entries from validated state; do not import dispatch leases or transfer a permit/lease token.
5. Never mark an operation received solely because the bundle contains a receipt reference. The current receipt model is correlation, not a signed attestation from the server.
6. Commit the imported records and all derived indexes atomically. If storage quota, key collision, schema upgrade or any write fails, leave the prior local state unchanged.
7. Before retrying OutcomeUnknown, send the original idempotency key and exact canonical payload bytes. Safe retry still depends on authoritative server-side idempotency and request-fingerprint enforcement.

The IndexedDB adapter implements read-only export plus atomic staging outside the live journal. Staging does not activate an operation or establish its state as authoritative. Safe promotion is intentionally absent until an authenticated service reconciliation/receipt-verification contract exists. This must not be advertised as one-click device migration.

## Security and truthfulness

The digest detects accidental corruption; it is NOT a MAC or digital signature. Anyone who can edit a bundle can recompute an unkeyed digest. Snapshot state is therefore untrusted input and is revalidated, while server outcomes must be independently reconciled with the authoritative service.

Bundles contain complete operation payloads and may include sensitive personal, household or civic data. This codec does not encrypt at rest or in transit. A future UI must clearly warn before writing a plaintext bundle, avoid automatic cloud upload, and offer authenticated encryption with recovery-key handling. Losing the recovery key must be an explicit failure mode rather than a secret bypass.

Owner scope prevents accidental cross-account restore but is not a credential. A service must derive its tenant from authenticated request context, not this file. An owner mismatch should block import without revealing or mutating the other account's local records.

## Conformance and fault matrix

| ID | Test | Required outcome |
|---|---|---|
| BND-01 | Same bundle serialized, parsed and serialized repeatedly | Byte-identical canonical JSON and stable integrity digest |
| BND-02 | Whitespace or object-key reorder only | Validation succeeds; canonical output is identical |
| BND-03 | Payload, attempt count, state, namespace or owner changed without digest update | Import validation fails |
| BND-04 | Payload/state is invalid even when a bundle writer recomputes the outer digest | Inner snapshot validation still fails |
| BND-05 | Duplicate operation IDs or unsorted hand-authored records | Reject duplicate IDs; enforce normalized sorted order on the wire |
| BND-06 | Unknown bundle/snapshot fields or future version | Fail closed with an explicit version/schema error |
| BND-07 | Bundle owner differs from authenticated target | Reject before IndexedDB write; no records or indexes change |
| BND-08 | Exact operation already exists locally | Idempotent no-op only after comparing owner, namespace, envelope, payload and state lineage |
| BND-09 | Same operation ID or scoped key but changed immutable contents | Abort the entire import; keep pre-import state byte-identical |
| BND-10 | Quota/transaction abort halfway through index reconstruction | Zero partial imports |
| BND-11 | Bundle holds OutcomeUnknown with old receipt reference | Preserve uncertainty; independently reconcile using the same server idempotency key |
| BND-12 | Terminal state has a forged receipt reference and recomputed bundle digest | Never promote it to an authenticated server result |
| BND-13 | Restore/import repeated after success | No state reset and no duplicate logical server effect |
| BND-14 | Lease record omitted from export | Restore reclaims through the regular lease protocol; no transferred token grants dispatch authority |

## Performance targets (proposed, not measured)

Benchmark against the exact same synthetic corpus and hardware:

- Bundle sizes: 1,000 / 10,000 / 100,000 records, with payload cohorts 1 KiB / 32 KiB / 256 KiB; add a separately labelled large-payload stress case.
- Measure create/validate/parse/canonical-encode throughput, p50/p95/p99 latency, peak memory, bundle bytes per record, and digest verification cost.
- Proposed initial target: validate 10,000 typical records in under 1 second on a named modern desktop, with no unbounded extra copies of every payload. Treat this only as a hypothesis until measured; 100,000-record restore is a scale test, not a promised browser UI operation.
- Correctness is an absolute gate: zero malformed snapshots accepted, zero owner-scope mismatches accepted, zero partial imports, zero lost attempt/state lineage, zero duplicate logical effects in the server fault suite.

Record the exact Git SHA, Rust/compiler/browser versions, CPU/RAM, input corpus digest, seed, command and raw per-run samples. Publish median and tail latency; a single best run is not evidence of performance superiority.
