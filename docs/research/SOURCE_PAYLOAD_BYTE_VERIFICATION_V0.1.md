# Source Payload Byte Verification v0.1

**Status:** Rust streaming verifier implemented; exact-head qualification is pending.
**Parent:** [#500 — independently rehash source payloads](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/500).
**Composes with:** [#492 / PR #496 descriptor catalogue](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/496), [#486 / PR #491 projector](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/491), [#347 trust and revocation](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/347).

## Objective

The descriptor catalogue describes a payload digest and how that digest was obtained. Descriptor consistency alone does not prove that the referenced payload object still contains those bytes.

This module opens the exact content-addressed object through a caller-supplied resolver, reads it in bounded chunks, counts bytes, computes SHA-256 independently, and compares both digest and declared byte length.

The verifier is deliberately an I/O boundary plus a deterministic hashing core. The resolver chooses storage and opens an object; the verifier owns the stream limit, byte count, hash, expected digest comparison, and result.

## Supported result states

| Result | Meaning |
|---|---|
| `verified` | Stream ended normally; recomputed digest and optional byte length match |
| `bytes_unavailable` | Descriptor has no usable payload object/bytes or declares unavailable |
| `object_not_found` | Exact CAS object does not exist in the resolver |
| `read_failed` | Resolver open or stream read failed |
| `byte_limit_exceeded` | Per-object or aggregate hard byte budget was exceeded |
| `digest_mismatch` | Recomputed SHA-256 differs from the descriptor claim |
| `byte_length_mismatch` | Stream length differs from the descriptor claim |
| `invalid_descriptor` | Digest/reference basis is internally contradictory |
| `not_attempted_invalid_catalog` | Descriptor catalogue was rejected before opening any object |
| `not_attempted_aggregate_limit` | Aggregate byte budget was exhausted before the object was opened |

Digest mismatch and length mismatch are invalid evidence. Missing objects, read failures, and resource limits are incomplete/indeterminate: they do not become digest mismatches or successes.

## Resource bounds

- Buffer size is fixed per invocation and must be between 1 byte and 1 MiB.
- Per-object and aggregate byte ceilings are independent.
- On exceeding a byte limit, the verifier reads at most one sentinel byte beyond the ceiling to distinguish an exact-size EOF from an over-limit stream.
- The verifier never accumulates the complete payload in memory.
- Interrupted reads are retried; other I/O failures are reported distinctly.
- Invalid catalogues and invalid verifier settings are rejected before object resolution.

The report binds the exact case representation digest, descriptor catalogue digest, object reference, expected and observed digest/length, verifier version/build identifier, resource ceilings, and per-record outcomes.

## Digest-basis semantics

- `locally_retrieved_and_hashed`: re-open and hash the exact CAS object; the descriptor's earlier claim is independently checked.
- `source_owner_attested`: if bytes are made available, independently re-hash them; otherwise retain the owner-attested digest as unverified.
- `external_reference`: a reference alone is not a byte proof. The resolver must open the exact CAS object and the verifier must re-hash it.
- `unavailable`: do not fabricate a digest and do not attempt a resolver read.

The CAS reference must equal `cas:sha256:<expected-digest>`. A different object reference is rejected before reading.

## Claim boundaries

A byte verification success proves only that the bytes returned by the resolver match the descriptor's declared SHA-256 and optional byte length.

It does **not** authenticate:
- source owner namespace or issuer identity;
- signature, transparency receipt, key status, or revocation state;
- rights or permitted use;
- whether the source snapshot/page inventory was complete;
- whether the source statement is true;
- whether an engineering result or decision is correct;
- whether anyone has authority to approve or act.

The report therefore fixes source-owner authentication, rights authentication, source-attestation authentication as `not_performed`, and action authority as `none`. The verifier build digest is recorded but not authenticated by this module.

A fully byte-verified report is not a fully trusted engineering decision. Downstream reliance must still compose the independent descriptor, inventory, trust, temporal, coverage, engineering-domain and authority receipts.

## Qualification targets

P0:
- exact agreement with independent SHA-256 and byte-length recomputation;
- zero acceptance of a one-byte mutation, truncation, extension, wrong object reference or digest substitution;
- zero digest success on resolver error or byte-budget exhaustion;
- memory usage bounded by buffer size plus fixed metadata, independent of object size;
- invalid catalogue results in zero resolver opens;
- an incomplete descriptor catalogue remains incomplete after successful byte checks;
- no trust, rights, source-completeness, engineering or action-authority promotion.

P1:
- 1 KiB, 1 MiB, 100 MiB and 1 GiB payload strata;
- 64 KiB, 256 KiB and 1 MiB buffer comparisons;
- p50/p95/p99 throughput, CPU time, peak RSS, bytes read, interrupted/partial reads, and recovery;
- aggregate budgets, concurrent readers, zero-length payloads and adversarial resolvers;
- separate object-store adapter comparisons from the deterministic byte-hash core.

P2:
- rights-cleared held-out payloads from at least two source-system families;
- independent source-owner verification that each CAS reference represents the original native payload bytes;
- #347 trust/signature/revocation integration under a separately reviewed policy.

These are proposed qualification gates, not achieved production results.
