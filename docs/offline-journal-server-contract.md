# Offline Journal Authoritative Service Contract v1

**Status:** design/acceptance contract for the future authoritative API. This repository's IndexedDB adapter is client-side only; no server implementation, database schema, or end-to-end proof is supplied by this document.

## Why this is the next critical boundary

A local journal can prove which bytes it intended to send. It cannot decide whether the server applied those bytes before a connection failed. A browser-local dispatch lease reduces simultaneous sends, but it cannot prevent a suspended tab from resuming after lease expiry, nor can it close the crash gap after the server commits but before the client stores the receipt.

The correctness boundary therefore has to be server-side. Transport is at-least-once; the service must provide one durable logical outcome per scoped idempotency key and immutable request fingerprint.

## Request identity and fingerprint

The server must accept the operation envelope and raw payload body, then independently:

1. Parse the body using the protocol's strict RFC 8785 JSON Canonicalization Scheme profile. Reject malformed JSON, duplicate names and unsupported numeric values.
2. Canonicalize it and compute SHA-256 over the exact canonical UTF-8 bytes. Reject if the result differs from `payload_digest_sha256`.
3. Reconstruct the immutable request binding from the validated envelope. The fingerprint must bind all immutable fields, not only the body digest:
   - envelope/schema version;
   - principal or tenant scope;
   - operation ID and aggregate ID;
   - expected aggregate revision;
   - canonical payload digest;
   - explicit supersession/conflict lineage.
4. Use the idempotency key as a unique, separately indexed ledger key; do not include the key itself inside the fingerprint material. Persist an authoritative server-computed fingerprint. Do not trust a client-supplied fingerprint without recomputation.

The shared Rust reference primitive is now available in `sol-atlas-core::offline_idempotency_fingerprint`:

- `canonical_request_fingerprint_material` returns the exact canonical JCS UTF-8 bytes;
- `canonical_request_fingerprint_sha256` returns lowercase SHA-256 over those bytes.

Fingerprint schema v1 binds the tenant scope, operation namespace, envelope schema version, operation ID, aggregate ID, exact expected revision (encoded as decimal text to avoid unsafe IEEE-754 numbers), payload digest, and explicit nullable supersession lineage. It excludes the idempotency key because the server's unique ledger key indexes that fingerprint separately. The server must derive tenant/route scope from authenticated context, not accept an arbitrary client-selected scope as authoritative.

The client-side IndexedDB record also persists an immutable `owner_scope` and `operation_namespace` next to each journal snapshot. Enqueue, pending scan, dispatch-lease renewal and reply reconciliation require an exact match to the current account/tenant and action namespace. A mismatch refuses automatic dispatch and hides the row from the other owner context. The local scope is a guard against accidental account-switch replay, not an authentication claim; the authoritative service must still derive its own tenant and route from the authenticated request and reject any mismatch against the intended operation scope. The dispatch permit exposes the shared request fingerprint so a transport can attach it for diagnostics/correlation, but the server must recompute it from the authenticated scope, route, envelope and body.

The frozen eight-vector corpus at `sol-atlas-core/fixtures/offline-idempotency-fingerprint-v1.json` locks both canonical material bytes and resulting digests, including tenant/namespace changes, operation/aggregate/revision changes, lineage changes, and the separate-key rule. Independent service implementations should consume that same file and compare bytes and digests exactly. The Rust tests only establish the Rust implementation's conformance to this corpus; no second server runtime is implemented here.

## Durable ledger and atomic effects

Suggested relational model (names are illustrative):

`idempotency_ledger`
- scoped unique key: `(tenant_id, operation_namespace, idempotency_key)`;
- `request_fingerprint`, `operation_id`, aggregate ID and expected revision;
- status: `in_progress | applied | rejected | conflict`;
- stable outcome / receipt ID, response status and replayable response body;
- created/updated timestamps and published retention policy.

The local IndexedDB idempotency index is also scope-qualified as `(owner_scope, operation_namespace, raw_idempotency_key)` using an unambiguous tuple encoding; the service ledger must use its own authenticated equivalent. This avoids accidental local collisions across independent tenants/routes but does not replace server-side uniqueness.

The database must enforce uniqueness; a read-before-insert check alone is racy. Every write is scoped to the authenticated principal/tenant. Never reveal another tenant's response when a key collides.

For effects contained in one transactional database, the ledger reservation, aggregate revision check, domain mutation, final idempotency outcome, receipt record and any outbox event should commit in one database transaction. This produces one authoritative commit point.

For external effects (payments, email, messaging, physical device commands, partner APIs), a local database transaction cannot atomically include a remote system. Use a transactional outbox plus downstream idempotency/deduplication and reconciliation; do not claim global exactly-once execution across uncooperative systems.

## Executable reference state machine

Shared Rust core now exports `offline_idempotency_ledger_model::ReferenceIdempotencyLedger`. It is an **in-memory contract oracle**, not production persistence. The reference model executes these semantics:

- `reserve(scope, namespace, envelope, body)` independently validates the envelope and body digest, recomputes the full scoped request fingerprint, and creates at most one active reservation for the scoped idempotency key.
- An identical request while the reservation is active returns `AlreadyInProgress`; an identical request after commit returns `Replay` with the exact stored HTTP status, receipt ID, response bytes and response-body digest.
- Reusing the same scoped key with a changed immutable binding returns a fingerprint conflict without replacing the first record.
- `commit(ticket, outcome)` accepts only the current reservation generation; it returns the same outcome for an identical repeated finalization and refuses a conflicting outcome or stale ticket.
- `abort_before_effect(ticket)` is permitted only for the current reservation and is explicitly named for the case where the caller knows the domain transaction did not commit. It must never be used to clear an unknown external side-effect result.

The native tests cover exact replay after simulated response loss, changed aggregate/revision rejection, per-tenant/per-namespace key isolation, stale reservation fencing, immutable completed outcomes and a 64-contender in-process concurrency scenario.

**Limitation:** the model serializes through an in-memory map/mutex in tests. It does not establish durability, database isolation, cross-process correctness, effect atomicity or protection against server crashes. Its role is to provide an executable oracle that an actual database-backed service adapter must match. The production gate remains the crash/transaction fault matrix below.

## Required server behavior

| Situation | Required behavior |
|---|---|
| New key + valid fingerprint + valid expected revision | Atomically claim key; execute once; persist outcome and receipt with the effect |
| Retry after success with same key and same fingerprint | Return the original outcome and receipt; do not repeat the effect |
| Retry after a deterministic rejection/conflict with same key and same fingerprint | Return the stored original outcome; changed intent requires a new operation/key |
| Same scoped key reused with another fingerprint | Reject with a documented client error (HTTP 422 is recommended by the retrieved HTTPAPI working-group draft); never apply the new payload |
| Same key concurrently in progress | Return a documented in-progress/conflict response, or wait for/replay the definitive result; never create a second logical execution |
| Missing required key | Reject before the non-idempotent effect |
| Server crash before transactional effect commit | No domain effect is committed; retry can claim/complete the operation |
| Server crash after effect/outcome commit but before response | Retry replays the committed outcome with the same receipt |
| Retry after key-retention horizon | Follow a documented expiry policy; do not silently imply indefinite deduplication |

A server response must bind to the original operation envelope. The client reducer already checks reply binding, but this is not authentication. Authenticate the endpoint through the established service channel; if portable audit evidence is needed, define a signed receipt format binding request fingerprint, operation ID, idempotency key, outcome, applied revision, result digest, receipt ID, signer/key ID, and protocol version.

## Retention and replay policy

The resource owner must publish key scope, required header syntax, expiry/retention duration, duplicate-in-progress behavior, deterministic validation-error handling, maximum request size and replay guarantees. Retain the ledger at least as long as the supported offline retry horizon and any business duplicate-hazard horizon. An in-progress operation must not age out while it can still commit. If the system must forget bulky response bodies, retain a tombstone containing the key, fingerprint, receipt/outcome reference, and replay policy so an old retry cannot create a second effect.

The current HTTPAPI working-group document titled *The Idempotency-Key HTTP Header Field* describes unique keys, request fingerprints, replay of completed outcomes, a conflict response for concurrent requests, and rejection when one key is reused with a different payload. It is a working-group Internet-Draft, not proof that any arbitrary endpoint implements those behaviors. Check the current revision/status before choosing the exact wire syntax: [IETF HTTPAPI working-group source](https://github.com/ietf-wg-httpapi/idempotency/blob/ebd731bf7ae4af534725eccee8c5d5bcad3580c0/draft-ietf-httpapi-idempotency-key-header.md).

## End-to-end fault benchmark

Correctness gates are absolute; a speedup cannot offset one duplicate effect or one false receipt.

| ID | Scenario | Hard oracle |
|---|---|---|
| SVC-01 | 100,000 operations with randomized retries, response loss, duplicate delivery and out-of-order replies | Zero duplicate domain effects; one stable outcome per scoped key/fingerprint |
| SVC-02 | Same key with one changed immutable field at a time | 100% rejected before effects; all mutations retained as minimized counterexamples |
| SVC-03 | Same body/digest but changed aggregate, revision, operation ID or lineage | Fingerprint mismatch rejected; body digest alone is not accepted as complete request identity |
| SVC-04 | Concurrent duplicate requests at 1/16/64/256 clients | At most one authoritative logical effect; all losers receive a defined replay/in-progress outcome |
| SVC-05 | Crash injection before and after every durable write / outbox dispatch / response write | Zero committed effects without recoverable ledger outcome; zero second effects after response loss |
| SVC-06 | Tenant A's key and Tenant B's collision attempt | No cross-tenant outcome disclosure or mutation |
| SVC-07 | Key expiry followed by delayed offline retry | Behavior follows an explicit expiry contract and cannot silently duplicate a protected effect |
| SVC-08 | Receipt mutation/tampering and stale applied revision | Authentication/binding failure never promotes client state to received |
| PERF-01 | End-to-end latency at concurrency 1/16/64/256 and payload cohorts 1 KiB/32 KiB/256 KiB/1 MiB | Publish p50/p95/p99, throughput, CPU/storage cost, failures, target hardware and raw samples |
| PERF-02 | Ledger hot-key and many-key contention | Quantify collision behavior; no correctness gate relaxed for throughput |

Report the raw event log with stable operation IDs, attempt numbers, server transaction/receipt IDs, payload digests, injected failure point, expected state and observed state. A test result without its code SHA, server/database version, test seed, exact command and raw evidence is not a release pass.

## Industry comparison and honest claim

The industry baseline is not a local retry queue alone. The established building blocks are (a) client persistence, (b) unique idempotency keys, (c) a server-maintained request fingerprint and outcome ledger, (d) database-enforced uniqueness and transactional effects/outbox, (e) replayable receipts, and (f) fault injection around crash boundaries.

The potential Sol Atlas advantage is the explicit combination of canonical byte identity, immutable payload retention, snapshot revalidation, browser-local claim coordination and a conformance suite spanning client/service boundaries. This is an architectural hypothesis, not a measured lead over Stripe-like APIs, offline-first databases, or other mature systems. The differentiator becomes real only when independent implementations pass the same corpus and crash suite, then publish comparable latency and recovery measurements.
