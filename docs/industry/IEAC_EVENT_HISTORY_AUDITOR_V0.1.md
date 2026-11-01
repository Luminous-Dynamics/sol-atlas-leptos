# IEAC event-history auditor v0.1

**Status:** Synthetic-fixture structural auditor; no SCITT COSE or VDS verifier is included.  
**Tracks:** [IEAC-TRANSPARENCY-002 / #376](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/376).  
**Composes with:** [SCITT event payload profile / PR #375](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/375).

## 1. Purpose and claim boundary

The `industry-event-history-audit` binary audits event-payload digests, issuer-local sequence structure, predecessor links, and cross-issuer causal graph structure in the frozen SCITT synthetic event corpus.

It is deliberately not a COSE implementation, SCITT verifier, VDS proof verifier, relying-party authorization engine, or action gate. A structurally consistent bundle is not proof that its signatures or receipts are valid, that the issuer submitted every statement, or that the represented measurements and outcomes are true.

The first fixture has four events from three issuer streams. Its event graph has three explicit cross-issuer causal edges and one issuer-local predecessor edge. This is why the combined graph contains four distinct directed edges.

## 2. Implementation boundary

The first implementation is a separate binary and independent graph-audit path under the existing `industry-runtime-admission` Cargo package. It does not call the candidate evaluator or the existing admission checker. It reuses the package's pinned `serde_json` and `sha2` dependencies, so it is not yet a separately built supply-chain artifact or a fully independent cryptographic verifier.

### 2.1 Restricted JCS profile

For this frozen event payload profile, the binary computes JCS bytes only for:
- ASCII object member names and strings;
- safe integral JSON numbers in the IEEE-754 interoperable integer range;
- JSON nulls, booleans, arrays and objects.

For this subset, ASCII byte ordering is identical to UTF-16 code-unit property ordering, and safe integers use unambiguous base-10 serialization. It emits the RFC 8785 string escapes for quote, backslash, controls and the standard short escape characters.

The binary refuses non-ASCII strings/member names, fractional or exponent-form values that deserialize as non-integral numbers, and integers outside the safe range. This is **not** a general RFC 8785 implementation. The profile's synthetic event fixture is intentionally inside this subset. Production adoption must use a general-purpose conformant JCS implementation, its full test vectors and cross-implementation verification over the exact detached payload bytes.

A strict JSON visitor rejects duplicate object member names before conversion into a JSON value. This avoids the common parse-and-overwrite ambiguity for duplicate names.

### 2.2 Stream checks

Issuer stream identity is the tuple:

`(issuer_id, subject_id, issuer_stream_id)`

For each stream, the auditor:
- rejects duplicate sequence claims;
- requires sequence 1 to have a null predecessor;
- requires every later event to name a lowercase SHA-256 predecessor;
- verifies predecessor digest equality whenever the immediately previous sequence is present;
- marks a missing prefix or sequence gap as incomplete rather than pretending the earlier history is empty.

The auditor does not infer that an observed sequence range is the complete lifetime history of an issuer. It has no verified checkpoint or signed stream-head input in v0.1.

### 2.3 Causal graph checks

The auditor resolves every `causal_parent_payload_sha256` reference against the bundle's event digest index. A missing parent is incomplete; a present parent from a different decision case or subject is invalid. It combines predecessor and causal-parent edges and applies Kahn's topological algorithm; a directed cycle is invalid.

The auditor does not use SCITT registration order as event issue order. Issuer-local sequence and causal references are separate application semantics.

## 3. Results and exit semantics

The output contract is `schemas/industry-scitt-event-history-audit-report-v0.1.schema.json`.

| Exit code | Report status | Meaning |
|---:|---|---|
| 0 | Reserved | Not currently emitted by the synthetic-only profile |
| 2 | `invalid` | Digest, sequence, predecessor, scope or graph integrity control failed |
| 3 | `valid_incomplete` | Structural checks completed, but complete history is not established; fail closed for any policy requiring completeness |
| 4 | No complete report guarantee | Input unsupported, malformed, unreadable or outside the restricted canonicalization subset |

A `valid_incomplete` result is **not an admission pass**. The CLI exits 3 for it. Even if a future bundle includes checkpoint-shaped fields, v0.1 cannot authenticate the checkpoint and will not promote the history to complete.

The report's `matched_event_payload_digests` field counts event payloads whose restricted-subset JCS digest matched the fixture's expected digest map. It does not imply signature verification, receipt binding or signer authority.

## 4. Not checked

The report records that this binary does not check:

- COSE Signed Statement signatures or protected CWT `iss` / `sub` claims;
- COSE Receipt signatures or receipt-to-statement binding;
- VDS inclusion, consistency, non-inclusion or non-equivocation proofs;
- transparency-service or issuer trust roots, key authorization, freshness or revocation;
- authenticity of stream checkpoints;
- whether an issuer created additional events and withheld them;
- source measurement truth, model validity, physical fitness-for-purpose or outcome correctness.

These facts must come from separately pinned SCITT and IEAC verification components, and their exact executable/policy/trust bundle identities must be bound into the final audit evidence.

## 5. Frozen tests and promotion

The binary includes tests for:
- ASCII key ordering, JSON string escapes and refusal of unsupported Unicode/fractional numbers;
- duplicate JSON member names;
- all four fixture digest matches and the expected 4-event / 3-stream / 4-edge graph;
- missing stream prefix and missing causal parent as incomplete;
- duplicate issuer sequence as invalid;
- causal cycle as invalid;
- event digest mismatch as invalid.

CI invokes the binary at the exact PR head. The expected fixture exit code is 3 and the report must say `valid_incomplete`; a digest-mutated negative control must exit 2 and report `invalid`.

The immediate next implementation stages are:
1. Replace the restricted JCS subset with an independently audited general JCS implementation and RFC 8785 conformance corpus.
2. Consume exact SCITT verification fact bundles from an independently pinned verifier; keep receipt issuer and statement issuer policies separate.
3. Add authenticated issuer stream-head checkpoints and VDS consistency-audit observations, with no promotion unless signatures, trust roots, freshness and anti-rollback state are checked independently.
4. Split the graph auditor into its own Cargo package and build it independently from the evaluator/checker, then cross-check results with a second implementation.

## 6. Benchmark targets

For the frozen structural corpus:
- all golden payload digest comparisons match;
- 100% of duplicate-sequence, predecessor-mismatch, cross-scope and cycle mutations fail closed;
- 100% of missing-prefix, sequence-gap and dangling-parent fixtures report incomplete rather than complete;
- zero incomplete histories receive a complete status;
- report exact event/stream/edge counts and corpus digests.

For scale testing, add fixed event graphs at 10², 10³, 10⁴ and 10⁵ events, varying issuer count, causal fan-in, chain depth, gap rate and malformed-edge density. Record p50/p95/p99 runtime, memory, event throughput and report size. Compare against an independent implementation and a deliberately naive baseline; freeze hardware and corpus digests before publishing performance claims.

These are synthetic conformance targets, not real-world false-admission or physical-risk estimates.
