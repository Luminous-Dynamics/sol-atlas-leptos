# Source payload byte verification v0.1

**Status:** bounded streaming verifier implemented. On 2026-11-04, exact-head Rust formatting, strict Clippy, and core unit tests passed at commit `74b5271a84b95a94f3487f7d5ec1fa606a37a30b`; report-schema validation is pending on the final schema/fixture head.
**Issue:** [#500 — independently rehash source payloads](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/500).
**Depends on:** [#492 / PR #496 — owner-native descriptor catalogue](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/496).
**Composes with:** [#347 — trust roots and signed statements](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/347), [#488 — source coverage qualification](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/488), and [#484 — independent impact oracle](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/484).

## 1. The claim being added

The descriptor catalogue can state that a payload was locally retrieved, that a SHA-256 digest was computed, and that a content-addressed object reference exists. Structural validation can check that those fields agree with each other. It does not open the object or hash the bytes.

The byte verifier adds exactly one new claim:

> The bytes returned by the caller-supplied resolver for this exact content-addressed reference were streamed through this verifier and their observed SHA-256 (and declared byte length, when present) matched the descriptor.

That claim is deliberately narrower than source authenticity, rights, attestation, inventory completeness, engineering correctness, or authorization.

## 2. Execution model

`verify_catalog_payload_bytes` first validates the descriptor catalogue against the exact serialized `BuiltEnvironmentCase`. Invalid catalogues are rejected before the resolver is called.

The storage adapter is supplied as a resolver closure. The verifier owns:
- the requested opaque content-addressed reference;
- a fixed 64 KiB read buffer;
- incremental SHA-256;
- observed byte count;
- per-object hard byte limit;
- expected digest and optional length comparison;
- deterministic per-record findings and final disposition.

No whole-payload `Vec<u8>` is allocated. The live payload buffer is fixed-size; the report itself grows with the number of descriptors and records, so aggregate report memory remains (O(n)) in descriptor count.

A one-byte probe distinguishes an exact-limit object ending at EOF from an object that exceeds the configured limit. In that case the reported `bytes_read` may be one byte greater than the configured limit; no digest conclusion is made for an over-limit stream.

## 3. Result semantics

| Result | Meaning | Reliance |
|---|---|---|
| `verified` | Observed bytes match declared digest and optional length | Byte equality only |
| `source_bytes_unavailable` | No object reference or resolver could not open it | Incomplete |
| `read_failed` | Stream failed after opening | Incomplete |
| `byte_limit_exceeded` | Per-object resource bound reached | Incomplete |
| `invalid_descriptor` | Digest/reference basis is internally contradictory | Invalid |
| `digest_mismatch` | Recomputed SHA-256 differs | Invalid |
| `length_mismatch` | Recomputed byte count differs | Invalid |

Catalogue outcomes:
- `all_resolvable_payloads_verified`: descriptor validation is internally consistent and every descriptor payload reference was successfully rehashed; it still grants no source trust or action authority.
- `incomplete_or_indeterminate`: one or more payloads were unavailable, a read failed, a limit was reached, verifier identity is missing, or the descriptor catalogue was incomplete.
- `invalid_input`: invalid descriptor catalogue, invalid verifier context, digest mismatch, or length mismatch.

The report carries the exact case representation digest, descriptor catalogue digest, verifier build digest, verification time, resource limit, descriptor-validation disposition and findings, per-record expected/observed digest and length, and counts/denominators. It also exposes `invalid_finding_count` and `incomplete_finding_count`. Any invalid verifier/report finding is non-compensable and forces `invalid_input`, even if all payload bytes individually match.

## 4. Digest-basis behavior

- `locally_retrieved_and_hashed`: reopen the exact CAS reference and recompute the digest; the descriptor's assertion is not trusted without this check.
- `source_owner_attested`: a supplied CAS object may be independently opened and rehashed, but owner identity/signature remains unauthenticated.
- `external_reference`: a reference alone is not a proof; the resolver must return bytes that are then rehashed.
- `unavailable`: no digest or object is invented; the record remains unavailable.

A byte digest proves equality to a declared digest, not that the resolver returned bytes from the original source owner. A malicious resolver can return arbitrary bytes; the hash check detects mismatch but cannot authenticate the resolver or the mapping from source owner to CAS object. A separately qualified CAS adapter, object-store access policy, and source-owner attestation are required for that claim.

## 5. Standards alignment: compose, do not reinvent

This work should reuse the standards ecosystem rather than create a parallel trust stack:

- **NIST IR 8536 (final, 9 September 2026):** manufacturing traceability meta-framework for interoperable pedigree and secure cross-organization linkage. Use it as the manufacturing traceability comparison and data-sharing context; the local byte verifier supplies one evidence primitive, not a competing framework.
- **IETF RFC 9943 (June 2026), SCITT architecture:** standard-track architecture for signed supply-chain statements and transparency receipts. #347 should evaluate how source-owner statements, signer authority, receipts, history and revocation compose with IEAC. SCITT's transparency receipt is not a byte rehash and does not prove an engineering relationship is complete.
- **W3C Verifiable Credentials Data Model 2.0 (Recommendation, 15 May 2025):** suitable for interoperable issuer/holder/verifier claims where a credential is the right abstraction. A credential can make an issuer claim verifiable, but does not itself establish the truth of an engineering measurement or source-universe completeness.
- **ISO 23247-6:2026:** digital-twin composition baseline. The differentiator is independently checkable byte, identity, coverage, temporal and engineering decision evidence over the composed view—not federation itself.

## 6. Qualification targets

P0:
- exact SHA-256 agreement with an independent known-answer oracle;
- zero false verified results under one-byte mutation, truncation, extension, wrong-reference and digest-substitution controls;
- exact-limit allowed; one byte over limit must refuse with no digest conclusion;
- interrupted and short reads must preserve the correct digest;
- IO failure must never become digest mismatch or verified;
- invalid descriptor/verifier bindings must not call the resolver;
- no rights, owner authentication, attestation authentication, source completeness, or action-authority promotion.

P1:
- 1 KiB, 1 MiB, 100 MiB and 1 GiB payloads;
- 64 KiB, 256 KiB and 1 MiB buffer comparisons;
- p50/p95/p99 throughput, CPU time, peak RSS, bytes read, resolver latency and retries;
- concurrent streams, interrupted/partial reads, corrupted CAS values and cold/warm object stores;
- record-count scaling at 10, 100, 1,000, 10,000 and 100,000 descriptors;
- measure report memory separately from in-flight payload-buffer memory.

P2:
- held-out source bytes from at least two independent source-system families, subject to rights;
- independent source-owner validation that the CAS reference denotes original native bytes;
- SCITT/VC trust evaluation, key rotation, equivocation and revocation handled as a separate qualification track.

No aggregate score can compensate for one false byte-verification result. Report every object, unavailable record, failed open, timeout, retry and byte-limit exclusion. A faster verifier that silently skips unavailable or corrupt objects is not better.
