# Built-environment source payload byte verification v0.1

**Status:** bounded streaming Rust verifier and exact-head qualification workflow added on a branch stacked on the source descriptor work. Compilation, Clippy, tests, and workflow qualification are pending.
**Parent:** [#500 — independently rehash descriptor-backed source payloads](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/500).
**Composes with:** [source descriptor sidecar PR #496](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/496), [descriptor benchmark #494](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/494), [trust/canonicalization #347](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/347), and [coverage qualification #488](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/488).

## Purpose

The descriptor verifier in #496 checks whether a payload digest claim is internally consistent with its declared digest basis and content-addressed reference. It does not read the object. Therefore the label `locally_retrieved_and_hashed` is still a descriptor assertion until a separate verifier opens the referenced bytes and independently hashes them.

This module introduces that separate step:

`BuiltEnvironmentCase + SourceDescriptorCatalog + Resolver -> bounded byte verification report`

The resolver is a caller-supplied I/O boundary. The verification core owns the byte counter, hard limits, SHA-256 computation, expected digest and length comparison, and deterministic per-record results. It does not make network requests itself.

## Digest-basis treatment

| Basis | Verifier behavior | Positive result means |
|---|---|---|
| `locally_retrieved_and_hashed` | Reopens the content-addressed object and recomputes digest/length | The resolver-provided bytes match the descriptor's claim |
| `source_owner_attested` | Rehashes bytes if the caller makes them available; a digest alone is not sufficient | Byte equality is established, but the source-owner attestation is not authenticated |
| `external_reference` | Opens the referenced content-addressed object and recomputes digest/length | The referenced bytes match the declared digest |
| `unavailable` | Does not call the resolver and does not invent a digest | Explicitly unavailable; incomplete |

For local and external content-addressed objects, the reference must be exactly `cas:sha256:<expected-digest>`. For source-owner-attested inputs, the resolver can use the descriptor's expected digest as its lookup key or apply a caller-defined source lookup policy. The verifier does not infer a URL or fetch from the network.

## Bounds and outcomes

- Fixed streaming buffer is caller-configurable from 1 byte to 1 MiB.
- A per-payload byte ceiling and a catalogue-wide byte ceiling are enforced.
- The hash is computed incrementally; payloads are never loaded in full by the verifier.
- Interrupted reads retry; other stream errors are distinct from resolver-open errors.
- Not-found, resolver rejection, resolver I/O error, stream read error, resource exhaustion, digest mismatch and length mismatch have separate outcomes.
- Digest/length contradictions are invalid; unavailable data and resource limits are incomplete/indeterminate.
- Invalid catalogues are rejected before resolver access.

The implementation deliberately treats a stream that reaches the exact byte ceiling as indeterminate because it does not read an extra byte merely to test EOF. To obtain a conclusive result at the largest admissible payload size, configure the limit above that size. This preserves the hard bound on bytes consumed.

## Report and claim ceiling

The report binds:
- exact typed-case representation digest;
- exact descriptor-catalogue representation digest;
- evaluation time and verifier build digest (missing build identity is incomplete; malformed build identity/time is invalid);
- configured per-object, total-byte and buffer ceilings;
- each record key, digest basis, object reference, expected and observed digest/length;
- bytes hashed and resolver-open attempt;
- deterministic per-record and catalogue-level outcome counts.

Byte verification does not authenticate the resolver or storage backend, source-owner namespace, source signature, transparency receipt, rights evidence, source snapshot completeness, professional review or engineering correctness. The report explicitly keeps rights authentication and source-attestation authentication as `not_performed`, and action authority as `none`. A successful rehash is never sufficient for `NoImpact`, approval or actuation.

A successful rehash may discharge only the record-specific “digest not locally reverified” or “external payload bytes not retrieved” finding. It cannot resolve missing schema/media metadata, missing rights, missing source records, incomplete coverage or untrusted issuer identity.

## P0 tests

- exact digest and length match;
- one-byte payload mutation;
- descriptor byte-length mismatch;
- source-owner-attested digest rehashed through the resolver;
- missing object;
- invalid catalogue rejected before resolver access;
- hard byte limit remains indeterminate with no digest-match claim.

Additional tests planned before promotion:
- resolver-open I/O error and rejection;
- stream read failure after a partial read;
- zero-byte payload;
- catalogue-wide budget exhaustion after one or more verified objects;
- malformed build identity and evaluation time;
- independent SHA-256 vectors and interrupted reads.

## Benchmarks

P0:
- 100% exact digest/length agreement against an independently computed byte oracle;
- 100% detection of one-byte mutation, truncation, extension, wrong object, digest substitution and length substitution;
- no verification on IO failure or resource exhaustion;
- peak memory bounded by buffer size plus fixed metadata, independent of payload size;
- every descriptor and every resolver outcome retained in denominators.

P1:
- 1 KiB, 1 MiB, 100 MiB and 1 GiB payload strata;
- 64 KiB, 256 KiB and 1 MiB buffer comparisons;
- p50/p95/p99 throughput, CPU, peak RSS, bytes read, object-open latency, retry behavior and concurrency;
- local CAS, filesystem and object-store resolvers compared independently from the deterministic streaming hash core.

P2:
- rights-cleared held-out source payloads from at least two independent source-system families;
- source-owner review of whether the content-addressed object denotes the original source bytes;
- trust-root, signed statement, revocation and authority qualification under #347.

These are proposed acceptance gates, not achieved benchmark results.
