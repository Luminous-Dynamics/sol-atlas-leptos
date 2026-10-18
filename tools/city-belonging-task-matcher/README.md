# City Belonging Task Matcher

Standalone Rust checker for the frozen synthetic constraint-first task benchmark. It is separate from the Sol-Atlas UI and the HSDS admission checker.

## Run

From the repository root, with Rust and dependencies available:

    cargo run --manifest-path tools/city-belonging-task-matcher/Cargo.toml -- verify docs/research/fixtures/city-belonging-task-benchmark-v0.1.jsonl docs/research/fixtures/city-belonging-task-benchmark-v0.1.manifest.json

Run the unit tests with:

    cargo test --manifest-path tools/city-belonging-task-matcher/Cargo.toml

## Policy

1. Evaluate each user-stated hard constraint independently.
2. A hard constraint is satisfied only by evidence matching the declared operator; missing or unknown inputs yield an explicit unknown, never a positive match.
3. A candidate is eligible only if every hard constraint is satisfied.
4. If no candidate satisfies all hard constraints, report no fit; do not invent an option or silently relax a hard requirement.
5. Soft preferences never remove hard-eligible candidates; they create a separate preferred subset so the user can inspect trade-offs.
6. User-entered preferences apply to the current search; they do not become inferred identity labels or a stored sensitive profile.
7. This matcher consumes synthetic, pre-qualified facts. It does not verify service truth, admit HSDS records, or grant data-reuse permission.

## Integrity and claim ceiling

The checker verifies exact JSONL and embedded-schema SHA-256 values against the manifest before evaluating cases. It emits deterministic per-case constraint receipts and checks expected eligible and preferred candidate sets plus reason coverage for every excluded candidate.

The implementation contains a selected task-shape validator and digest binding, not a full JSON Schema evaluation engine. A matching receipt does not establish full JSON Schema conformance, real-world search quality, service coverage, contact success, safety, belonging, or loneliness reduction. No single global match, compatibility, wisdom, or social-cohesion score is emitted.

## Release targets

- Zero hard-constraint violations on the frozen fixture.
- Zero unknown-to-positive conversions.
- Zero silent hard-constraint relaxations.
- 100% exclusion-reason coverage for ineligible candidates.
- Zero candidates removed solely due to a soft preference.
- Byte-identical receipt for identical fixture, schema and policy versions.
- Measure actual search success, constraint comprehension and wanted next actions separately in later human studies.


## Metamorphic invariants

See the [7-property machine-readable contract](../../docs/research/fixtures/city-belonging-metamorphic-contract-v0.1.json). The Rust test suite now exercises candidate-order invariance, constraint-order commutativity, preference noninterference, constraint monotonicity, irrelevant-fact noninterference and conjunctive hard requirements. These tests remain unexecuted until the scoped CI run completes.
