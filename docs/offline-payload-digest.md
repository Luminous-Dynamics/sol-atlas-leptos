# Offline Payload Digest v1

**Status:** a Rust shared-core canonicalization/digest primitive and frozen conformance corpus are implemented in this branch. The production journal/storage/transport path does not yet make verification mandatory, and the authoritative service implementation is not part of this PR.

## Protocol

For a raw payload JSON string:

1. Parse and validate it under the accepted RFC 8785 / I-JSON profile.
2. Reject duplicate property names, malformed JSON, non-finite/out-of-model numbers, or values the implementation cannot represent faithfully.
3. Produce RFC 8785 JCS canonical UTF-8 bytes.
4. Compute SHA-256 over exactly those bytes and encode it as 64 lowercase hexadecimal characters.
5. Compare the result against the immutable operation envelope's digest before persisting or dispatching the operation. The service must independently repeat the computation and reject idempotency-key reuse if any immutable operation binding differs.

Implementation surface:
- `canonicalize_payload_json`
- `canonical_payload_digest_sha256`
- `verify_payload_digest`
- `OperationEnvelope::verify_payload_json`

`OperationEnvelope::validate` continues to check only digest-string shape and other envelope fields. The caller must invoke payload verification explicitly; the reducer cannot recompute a digest without the original payload bytes. No transport/store adapter in this PR currently enforces that ordering automatically.

## Canonicalization profile

This is RFC 8785 JCS—not merely sorted keys and compact JSON. The profile covers ECMAScript number serialization, UTF-16 code-unit property sorting, Unicode preservation without normalization, UTF-8 output, and strict rejection of values outside the agreed input profile. The protocol schema version pins the algorithm; changing the canonicalization profile requires a schema-version change.

RFC 8785 is published as an **Informational RFC**, not an Internet Standards Track specification. The protocol explicitly adopts its byte-representation rules because independent clients and services must agree on what the SHA-256 digest identifies. Reference: https://www.rfc-editor.org/rfc/rfc8785.html.

## Dependency and supply-chain boundary

The code currently pins the strict Rust JCS implementation at upstream commit `d4b6cdfd72ed0073d2c9ac6e83497234e78403be` from https://github.com/unbounded-ai/jcs. That source advertises MIT/Apache-2.0 licensing, includes raw-text canonicalization with duplicate-key rejection and rejects unsafe integer representations rather than silently rounding them. The immutable Git revision makes resolution reproducible; **it is not a substitute for dependency review**.

Before production use, review the pinned source, audit its actual resolved dependency graph and licenses, and decide whether to vendor/fork it or replace it with a reviewed upstream package. If the implementation is changed, every fixture below and the RFC's full sample corpus must continue to pass. Do not hand-roll a number serializer as a quick fallback.

## Frozen corpus

`sol-atlas-core/fixtures/offline-payload-digest-v1.json` currently contains:
- Six fixed canonical payload/digest vectors, including key reorder/whitespace invariance, IEEE-754 edge formatting, UTF-16 property ordering, Unicode, empty object, and RFC 8785's correctly-rounded decimal parse sample.
- All 26 number serialization samples from RFC 8785 Appendix B, executed directly against the pinned serializer.
- Six invalid payload cases, including duplicate keys, escaped duplicate keys, an imprecise integer, invalid JSON literals, an out-of-range number and trailing data.

The fixtures are synthetic conformance data, not production telemetry. These tests qualify the Rust implementation only; they do not prove server interoperability until the authoritative service runs the same suite.

## Acceptance gates

| ID | Gate | Measurement |
|---|---|---|
| HASH-01 | Zero canonical-byte/digest divergence | All frozen vectors produce exact bytes and SHA-256 across the client and authoritative service |
| HASH-02 | 100% rejection of invalid JCS inputs | Duplicate keys (including escaped aliases), invalid JSON, unsupported numeric values and trailing content produce no digest and no operation dispatch |
| HASH-03 | Zero mismatched payloads reach dispatch or server-side effects | Mutate one payload byte/semantic value while reusing the envelope digest; verification fails before write/send; service repeats the check |
| HASH-04 | Zero retry drift | Same operation ID/key reuses the same canonical payload digest; changed payload requires a new operation identity and key |
| PERF-01 | Measure rather than assume hashing cost | Benchmark payload sizes 1 KiB, 32 KiB, 256 KiB and 1 MiB on a named browser/host, in optimized builds; report throughput, p50/p95/p99, allocations if measurable, and raw samples |
| PERF-02 | Reproducible interop result | Artifact records source SHA, JCS dependency revision, SHA-256 implementation/version, fixture digest, target/runtime, exact commands and failure counterexamples |

Correctness gates are absolute. A low latency measurement must never compensate for a payload-digest mismatch. Performance thresholds should be established from baseline measurements on target field devices before being converted into hard release gates.

## Next integration step

Make verification an unavoidable part of the durable journal ingress API and the service endpoint. Then build a crash/fault test where the same idempotency key is presented with: (a) identical raw JSON under different whitespace/key order, (b) a semantically changed payload, (c) duplicate-key JSON, and (d) a number that falls outside the accepted profile. The identical semantic payload must have the same digest; the remaining cases must fail closed before any side effect.
