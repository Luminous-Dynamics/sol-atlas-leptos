# Evidence-bound task review packet v1

**Status:** deterministic completeness evaluator and synthetic test corpus; not a verifier, safety authority or durable record service  
**Module:** `sol-atlas-core/src/task_evidence.rs`  
**Fixture:** `sol-atlas-core/fixtures/task-evidence-packet-v1.json`

## Paradigm shift: from status labels to reviewable claims

A graph says which prerequisites are complete; it does not say what evidence makes that statement defensible. This module models a version-bound evidence packet with explicit requirement IDs, typed evidence, source references, digest encodings, recorder identity and procedure revision. It evaluates whether the packet is complete enough for qualified review.

It intentionally returns `ReadyForQualifiedReview`, **not** `Verified`, `Safe`, `Authorized` or `Passed`. The packet evaluator does not automatically retrieve artifact bytes, recompute hashes during packet evaluation, authenticate producers, validate worker qualifications/signatures, or persist the result. A separate `verify_sha256_bytes` helper recomputes SHA-256 when a trusted adapter supplies exact artifact bytes; the adapter must resolve the correct source, apply any required canonicalization, and authenticate the expected digest source. Server-side identity, authorization and durable decision handling remain integration responsibilities.

## Current guarantees

- Packet graph ID, graph revision and task ID must match the caller's current authoritative projection. The caller must also provide the current authoritative procedure revision and its evidence-requirement set separately; the evaluator rejects packets whose declared requirements omit or rewrite that set. In production, do not populate the context from the packet being evaluated.
- Requirements and evidence references are explicit; duplicate requirement/evidence IDs and undeclared requirement references block readiness.
- Evidence type must match its requirement. Measurement artifacts require a unit and instrument reference.
- SHA-256 encoding must be exactly 64 lowercase hexadecimal characters. Packet evaluation treats that field as metadata; adapters can additionally call `verify_sha256_bytes` to recompute the hash over exact bytes and reject a mismatch.
- Pinned procedure revision must agree at packet, requirement and artifact level.
- Requirements needing an independent review block if the reviewer is missing or is also the recorder.
- Completeness and issue ordering follow stable input order for reproducible UI and test output.
- Requirement lookup is indexed by stable ID instead of rescanning all requirements for every artifact, avoiding quadratic lookup growth as packets scale.

The reviewer ID in the evaluation context is an input from outside this pure module. It is not an authentication or qualification check. An adapter must verify identity/role with the authority source.

## Frozen negative controls

The Rust test corpus includes complete packet, missing evidence, stale graph revision, wrong task, invalid digest, missing measurement unit/instrument, procedure mismatch, missing/self reviewer, duplicate evidence ID, undeclared requirement, wrong kind and duplicate requirement ID. The fixture is synthetic and contains no real service, instrument or job data.

## Benchmark plan

| ID | Target | Required evidence |
|---|---|---|
| EVD-01 | Zero incomplete/stale/malformed packets marked ready for review | Frozen adversarial fixture plus generated mutation corpus |
| EVD-02 | 100% of authoritative requirements have at least one valid matching artifact when ready | Invariant assertions and negative controls removing each required artifact in turn |
| EVD-03 | Zero self-review accepted when independence is required | Reviewer identity equality and whitespace/empty ID tests |
| EVD-04 | Zero stale graph or procedure revision accepted | One-step and many-step graph/procedure revision mutations |
| EVD-05 | Zero packets accepted after omitting or rewriting a required evidence condition | Mutate/remove one authoritative requirement at a time while keeping the rest of the packet valid |
| EVD-05 | 100% deterministic decision/issues for identical packet and context | Replay corpus and compare serialized evaluations |
| EVD-06 | p95 under 25 ms for 10,000 evidence records and 1,000 requirements (initial hypothesis) | Release-mode benchmark on named runner, ≥100 samples, raw distribution and exact SHA |
| UX-01 | ≥95% correct interpretation of missing-evidence explanations | Moderated evaluation with technicians and qualified reviewers, stratified by trade |

The numerical performance target is a hypothesis, not an observed result. Correctness and evidence completeness are hard gates; latency is optimized only after those gates hold.

## Reproducible evaluation benchmark

A Rust example generates deterministic synthetic packets and emits raw nanosecond samples with p50/p95/max, input cardinalities, host architecture/OS, build profile, and the completeness result:

```sh
cargo run --release --manifest-path sol-atlas-core/Cargo.toml --example task_evidence_bench -- 100 1000 10000
```

CI smoke-tests the runner on a small packet; that is not the 10,000-record performance result. The benchmark measures metadata/completeness evaluation only and excludes byte hashing, persistence, authentication and network I/O. Publish the exact source SHA and all raw samples from a named release-mode runner before comparing performance.

## Industry comparison and fair baseline

Offline-capable enterprise field-service products already support work orders, sync status, conflict handling and workflow instructions. Microsoft Dynamics 365 Field Service documents that its offline conflicts are resolved at table/record level rather than field-by-field in the documented mode; the outcome depends on configuration. That creates a well-defined comparison case for the separate offline journal, but it does not imply that the vendor lacks verification or workflow capabilities.

For evidence review, use a common synthetic job packet and compare:
1. whether stale graph/procedure revisions are visible,
2. whether an omitted or malformed artifact can appear complete,
3. whether measurement provenance (unit, instrument, source, procedure revision) stays attached,
4. whether reviewer independence is visible and enforced by the authoritative service,
5. how many UI steps and how much time are needed to resolve each negative control.

Publish platform/version/configuration and raw scenarios. Do not claim to beat a vendor unless an equivalent workflow was actually run. The relevant established foundation for representing provenance is the W3C PROV data model, which describes entities, activities and agents involved in producing or influencing data: [W3C PROV Primer](https://www.w3.org/TR/prov-primer/). This module is a small field-specific metadata contract, not a complete implementation of PROV.

## Next integration steps

1. Define canonical work/task/evidence ownership with Mycelix; map IDs to source records instead of minting client-only authority.
2. Have the server recompute and verify content digests from canonical bytes, validate signatures/roles and atomically compare graph revision before accepting a review decision.
3. Persist the review decision as an append-only event referencing this exact packet digest and procedure revision.
4. Connect Leptos to display issues and enable submission only when the server's preconditions are met; retain accessible text explanations.
5. Keep Bevy as optional spatial context, never the only route to inspect evidence or resolve a blocked task.
