# IEAC decision-time evidence frontier v0.1

**Status:** Structural temporal-classification contract, Rust auditor and synthetic conformance corpus. This is not a timestamp authority, signature verifier, source-truth verifier or decision evaluator.  
**Tracks:** [IEAC-TIME-001 / #413](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/413).  
**Composes with:** SCITT statement history #375 / event-history auditor #382; witness observations #387; bitemporal invalidation and dependency impact #396 / #397; cross-layer false-assurance benchmark #391 / #392; outcome reconciliation #389 / #390.

## 1. The new assurance boundary

A decision needs two different historical views:

1. **As-known-at-decision:** the exact evidence artifacts that the relying party could access by the decision cutoff.
2. **Later-known evidence:** evidence that became accessible only after the original cutoff, including late measurements, corrections and outcomes.

A later correction can have an earlier domain-valid time without having been available to the original decision-maker. Preserve both facts and make a new reassessment; do not rewrite the historical decision.

This issue is deliberately different from #396. The trust-lifecycle auditor propagates later revocations, corrections and vulnerabilities through a dependency graph. The decision-frontier auditor asks which exact evidence could have entered the original decision at its cutoff.

## 2. Do not collapse four different times

For every evidence artifact, preserve separate concepts:

| Time | Meaning | What it does not establish |
|---|---|---|
| `valid_time` | When the source says the fact applies in the domain | When the relying party knew it |
| `observed_at` | When the source says the observation occurred | When the exact artifact was retrievable |
| `source_recorded_at` | When the source says it recorded/published the item | Independent availability to the relying party |
| `first_available.time` | Time bound for the exact artifact becoming accessible to a specified relying party | Physical truth or universal publication |

The decision cutoff is a separate, independently supported time interval. Each `TimeBound` records a normalized UTC instant, uncertainty before/after, clock identity, basis, verification status, proof digest, verifier build and verification-policy digest.

The v0.1 Rust tool accepts only whole-second UTC timestamps of the form `YYYY-MM-DDTHH:MM:SSZ`. It rejects timezone offsets, fractional seconds and leap seconds rather than silently normalizing or guessing. The uncertainty interval is explicit and bounded to one day for this first profile. A future profile can add a richer time-scale/clock model after a dedicated conformance corpus exists.

## 3. Existence is not availability

Three standards are useful, but their semantics must not be stretched:

- **SCITT / RFC 9943 and COSE Receipts / RFC 9942:** a receipt establishes the claims supported by the registered Signed Statement and VDS proof. It does not, by itself, show that a specific relying party had retrieved or could access the exact artifact by an earlier decision cutoff.
- **RFC 3161:** a trusted timestamp token can provide proof-of-existence of the datum before a stated time, subject to TSA policy and token validation. Existence is not the same as accessibility to a particular relying party.
- **TUF:** signed metadata, expiration, monotonic versions and role separation are useful models for trusted metadata freshness and rollback/freeze resistance. TUF is not a decision-time evidence database or a source of physical truth.

For that reason, v0.1 distinguishes **positive availability evidence** from **proof of first availability**.

Positive availability evidence can establish that an artifact was available to the specified relying party by a particular time:

- `relying_party_ingest_receipt` backed by an authenticated ingest log;
- `authenticated_access_log` backed by an authenticated access log;
- `sealed_cache_manifest` backed by the pinned trusted-clock policy.

A positive access/ingest event after the decision cutoff proves availability by that later time. It does **not** prove the artifact was unavailable before then. Therefore a post-cutoff access event is `unknown`, not automatically `excluded_after_cutoff`.

To classify an artifact as first available after the cutoff, require `complete_first_availability_ledger` plus a separately bound coverage proof for the exact relying-party and evidence-channel identity. The proof must establish a complete, independently verified observation interval beginning no later than the earliest possible decision cutoff and continuing through the latest possible first-access time. Both coverage endpoints carry their own clock uncertainty and are verified under the pinned verifier build and policy. Missing, partial, disputed, out-of-scope or temporally insufficient coverage remains `unknown`.

A SCITT receipt, RFC 3161 token or source assertion is retained as evidence but classifies as `unknown` for relying-party availability unless accompanied by a separate access/ingest proof. Those existence proofs must not be relabelled as access proofs. Every availability proof binds the exact evidence artifact digest and the relying-party/evidence-channel scope.

The `verification_status`, verifier-build digest and policy digest are externally produced facts. The tool checks that the digests match the verifier build and policy pinned by the snapshot; it does **not** verify those signatures/proofs itself. The outer IEAC trust policy must independently authenticate the provenance of this input and the pin.

## 4. Interval comparison and abstention

For a trusted availability interval (A=[A_{lo}, A_{hi}]) and cutoff interval (C=[C_{lo}, C_{hi}]):

- Candidate-included only when (A_{hi} \le C_{lo}).
- Excluded as late only when (A_{lo} > C_{hi}).
- Otherwise the intervals overlap, so the relation is `unknown`.

A source-reported old `observed_at` never overrides a later first-availability interval. An earlier `valid_from` never backdates the time at which a correction became available.

The output uses `candidate_included`, not `admitted`. This binary performs temporal classification; it does not make the final IEAC admissibility decision. If the cutoff cannot be trusted, all evidence remains unknown. If a proof is explicitly failed, the input is invalid. Missing, disputed, unverified, wrong-scope, existence-only or overlapping time facts remain unknown.

The auditor always reports `history_completeness = not_established`. A snapshot can reconstruct only the evidence supplied to it; it cannot prove that the issuer or operator disclosed every relevant artifact.

## 5. Rust auditor

Package: `tools/industry-decision-frontier`  
Binary: `industry-decision-frontier-audit`

Run:

```sh
cargo fmt --manifest-path tools/industry-decision-frontier/Cargo.toml -- --check
cargo clippy --manifest-path tools/industry-decision-frontier/Cargo.toml --all-targets --locked -- -D warnings
cargo test --manifest-path tools/industry-decision-frontier/Cargo.toml --locked
cargo run --manifest-path tools/industry-decision-frontier/Cargo.toml --locked -- \
  schemas/examples/ieac-decision-frontier-snapshot-synthetic-v0.1.json \
  /tmp/ieac-decision-frontier-report.json
```

Exit semantics:
- `0`: structurally valid and no unresolved time classifications;
- `3`: valid but incomplete/indeterminate (expected for the mixed synthetic corpus);
- `2`: invalid input, inconsistent binding or failed required proof verification fact.

The report hashes the exact input file bytes. It does not claim JCS canonicalization for the snapshot; raw-byte digesting avoids pretending ordinary JSON serialization is a general RFC 8785 implementation.

## 6. Frozen synthetic corpus

The checked-in 13-item corpus includes:

1. Two independently supported pre-cutoff observations.
2. An observation made earlier but ingested after the cutoff.
3. A backdated source correction received later.
4. A SCITT receipt proving registration but not relying-party access.
5. An RFC 3161 timestamp token proving existence but not access.
6. An availability interval overlapping the cutoff interval.
7. Missing availability evidence.
8. An ingest proof scoped to another relying party.
9. A source assertion without an independent access proof.
10. A late outcome observation.
11. An ingest proof without a verified external result.

Expected distribution: 2 candidate-included, 2 excluded-after-cutoff and 9 unknown. The two exclusions have complete channel-history coverage proving first availability after cutoff; ordinary late access observations remain unknown. These are synthetic contract outcomes, not measured operational accuracy or risk.

## 7. Benchmarks and promotion gates

### Conformance / safety

- 100% exact expected classification on the frozen positive and negative corpus.
- Zero late evidence included in the original as-known candidate set.
- Zero backdated corrections that alter the original snapshot.
- 100% unknown classification for unverified, scope-mismatched, existence-only or overlapping mandatory time evidence.
- 100% refusal of malformed timestamps, duplicate JSON fields, unknown properties, verifier-build substitution, policy substitution and artifact-digest substitution.
- 100% of reports bind the exact input bytes, decision, profile, evaluator build, time-verifier build and time-verification policy.

### Utility / coverage

- Report candidate coverage separately from excluded and unknown proportions.
- Report false exclusion and abstention rates; an implementation that returns unknown for everything is safe but not useful.
- Measure reason-code precision/recall against independently authored temporal ground truth.
- Preserve the latest-known set separately from the original cutoff set and test that new evidence never mutates the original result.

### Stress / performance

- 10, 100, 1,000, 10,000 and 100,000 evidence rows.
- Shallow and long correction/reassessment sequences.
- Large shared source feeds, high fan-out and multi-tenant visibility differences.
- Clock uncertainties from milliseconds through hours; measure the abstention curve as intervals widen.
- p50/p95/p99 classification latency, throughput, memory, bytes read/written, report size and replay cost.

### Statistical integrity

Separate exhaustive deterministic controls from randomized robustness trials. If randomized trials are added, preregister generators and dependence assumptions, preserve every timeout/unknown, and report per-stratum and worst-group bounds. Synthetic conformance is not a field-risk probability.

## 8. Baselines and industry comparison

Compare on identical evidence bytes, time facts, relying-party scope, clock uncertainty and hardware:

1. **Latest-snapshot replay:** demonstrates hindsight leakage when it reuses late corrections.
2. **Source-timestamp-only replay:** tests the common but unsafe assumption that old source timestamps prove prior availability.
3. **Single-time bitemporal replay:** compares valid/effective time with system/recorded time but lacks artifact-level relying-party availability.
4. **SCITT/RFC 3161-only replay:** tests existence/registration evidence without an access proof.
5. **Positive-access-only replay:** treats a later fetch as evidence of prior absence (deliberately unsafe negative baseline).
6. **IEAC decision frontier:** adds exact artifact binding, relying-party/channel scope, verifier/policy pins, interval comparison and complete-history evidence before asserting first availability after cutoff.

IEAC should outperform these baselines on the frozen temporal-mismatch corpus while retaining useful candidate coverage. Do not claim superiority over a production historian or event-sourcing platform without a matched deployment comparison and independently sourced ground truth.

## 9. Claim ceiling and next integration

This tranche does not:
- verify COSE, SCITT receipts, RFC 3161 tokens or access-log signatures;
- prove source truth or complete source disclosure;
- decide final IEAC admissibility;
- prove that an artifact was physically usable by a human/operator beyond the declared access semantics;
- perform dependency invalidation or causal outcome estimation;
- grant action or actuation authority.

Next integration should consume an independently pinned time/provenance verifier report and the temporal dependency graph from #397, then append the resulting decision snapshot/report as SCITT-bound evidence. Later reassessments must reference the original snapshot digest and append a new event; never overwrite the original decision-time record.
