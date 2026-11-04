# Source payload catalogue resource budgets v0.1

**Status:** sequential aggregate budget verifier ported onto the current qualified descriptor base; exact-head qualification of this rebased branch is pending.
**Issue:** [#504 — aggregate resource budgets](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/504).
**Composes with:** [#500 payload byte verification / PR #503](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/503), [#492 source descriptors / PR #496](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/496), [#488 coverage qualification](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/488), and the pre-registered [#338 IBAP run manifest](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/338).

## Why per-object limits are insufficient

A 64 KiB streaming buffer and a maximum size per object bound an individual read. They do not bound the total work of a catalogue:

- many individually small payloads can trigger unbounded aggregate reads;
- many object opens can amplify storage requests;
- large descriptor inventories can produce large result vectors and report memory;
- retry loops and concurrency can multiply I/O beyond the simple sum of payload sizes.

The v0.1 policy therefore makes all resource ceilings explicit and applies them deterministically.

## API

The verify_catalog_payload_bytes_with_budget API accepts the case, descriptor catalogue, verifier context, a versioned CatalogResourceBudget, and a caller-owned object resolver.

The v0.1 budget includes:

| Bound | Meaning |
|---|---|
| max_descriptors | Maximum descriptors eligible for evaluation |
| max_total_payload_bytes | Maximum bytes incorporated into payload hash streams across the catalogue |
| max_total_bytes_read | Maximum bytes returned by resolver streams, including one-byte probes |
| max_bytes_per_payload | Existing per-object hard byte limit |
| max_resolver_open_attempts | Maximum resolver/open attempts across all descriptors |
| max_retries_per_payload | Must be zero in v0.1; retries are disabled |
| max_report_bytes | Maximum serialized budget-report size, including truncation metadata |

Execution is sequential (max_concurrency_used = 1). This is a deliberate, auditable first implementation. Wall-clock deadline and cancellation are left for a later injected-clock/cancellation-token API rather than reading nondeterministic wall time inside the core hash function.

## Byte accounting

Two byte counters are reported separately:

- total_payload_bytes_hashed: bytes incorporated into the streaming SHA-256 state;
- total_bytes_read: bytes returned by resolver streams, including the one-byte probe used to distinguish a stream ending exactly at a limit from a longer stream.

The per-object stream verifier reads at most the lesser of its per-object cap and the remaining aggregate payload/read budgets (reserving the probe byte). Therefore the aggregate read budget is a hard cap at the Read interface. The report never hides probe bytes inside hashed-payload totals. Resolver open failures remain separately classified as object-not-found, access-denied, and generic resolver-open failure; stream read failures are separate.

An exact content boundary is accepted when the stream ends at EOF at that size. A payload one byte longer is stopped at the reserved probe, receives byte_limit_exceeded, and produces no digest conclusion for that object.

## Determinism and skipped work

Descriptors are processed in canonical record_kind:case_record_id order, independent of input vector order. This determines which records receive a finite budget when a catalogue exceeds its descriptor, open-attempt, or byte budget.

A record not opened because a budget is exhausted receives the explicit skipped_budget status. It is not included in verified_count, and all skipped records remain in the original descriptor_count denominator. Budget exhaustion never becomes a digest mismatch or a successful verification.

The resolver retry limit is fixed at zero in v0.1. The report exposes retry count and sequential concurrency so later versions can add controlled retries/parallelism without changing the claim semantics.

## Bounded report and truncation

The report itself is subject to max_report_bytes. If the complete per-record detail cannot fit:
- records are removed in descending canonical record-key order;
- findings are then removed in deterministic reverse order;
- descriptor-validation findings are removed last;
- each omitted section retains an exact omitted count and a SHA-256 over length-prefixed serialized entries in removal order;
- the full descriptor denominator and pre-truncation status counters remain available;
- truncation forces incomplete_or_indeterminate.

The minimum supported report budget is 8 KiB so the fixed report envelope can be preserved. The serialized_report_bytes field reports the actual serialized size and is checked against the configured maximum.

This keeps memory and serialized output bounded without silently dropping the denominator or claiming that a truncated report is complete.

## Non-compensable result rules

- A digest/length mismatch or invalid descriptor remains invalid_input.
- A missing object, read error, per-object limit, aggregate limit, descriptor-count limit, resolver-open limit, missing verifier build identity, or report truncation remains incomplete.
- An incomplete descriptor catalogue remains incomplete even when every available payload byte stream matches its declared digest.
- Invalid descriptor catalogues and invalid budget/verifier contexts are rejected before the resolver is opened.
- Payload-byte verification does not authenticate source owner, snapshot, rights, signatures, source-universe completeness, or engineering correctness.
- No action authority and no NoImpact eligibility are granted.

## Qualification programme

P0:
- exact aggregate Read bytes never exceed max_total_bytes_read;
- hashed bytes never exceed max_total_payload_bytes;
- resolver opens never exceed max_resolver_open_attempts;
- descriptors evaluated never exceed max_descriptors;
- no skipped record is marked verified;
- exact-boundary object is accepted; one-byte extension is stopped;
- input vector permutations produce the same canonical evaluation order and per-record outcomes;
- report serialization never exceeds max_report_bytes, and truncation preserves counts plus deterministic omission digests;
- incomplete descriptor validation cannot be upgraded by successful byte reads.

P1 benchmark strata:
- descriptor counts: 10, 100, 1,000, 10,000, 100,000;
- payload sizes: 0, 1 KiB, 1 MiB, 100 MiB, 1 GiB;
- aggregate budget levels at 25%, 50%, 75%, 100%, and one byte below/above the exact boundary;
- object-open budgets and report truncation at 0%, 1%, 10%, and 50% of descriptors;
- slow/short/interrupted readers, resolver failures, zero-byte entries, and high descriptor fan-out;
- p50/p95/p99/worst-case time, total bytes read, bytes hashed, resolver calls, peak RSS, report bytes, skipped count and per-record denominators.

P2:
- rights-cleared source-object stores from at least two independent source-system families;
- separate measurements of adapter latency and deterministic hash-core throughput;
- trust, rights, revocation and source-inventory evidence remain separately qualified under #347 and #497.

## Industry comparison

This is not a new traceability standard or source-of-truth store. It is one bounded verification primitive in a composed assurance system. Compare the complete pipeline against the public NIST IR 8536 traceability principles and applicable vendor/object-store adapters only on equivalent object scope. Report inaccessible records and unresolvable references as unavailable, never as empty or verified.

The differentiator is not maximum throughput alone. It is a deterministic, independently replayable bound on the total cost of verification while preserving every skipped object, denominator and limitation.
