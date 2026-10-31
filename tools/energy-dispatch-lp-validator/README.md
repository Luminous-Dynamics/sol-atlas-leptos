# Energy Dispatch Independent LP Validator v0.1

A standalone Rust validator for the restricted integer-LP dialect emitted by
`tools/energy-dispatch-mip-reference`.

The validator does **not** depend on or import the emitter. It parses the LP
artifact, independently derives the expected integer model from the source
request, and compares objective, variables, domains, bounds, constraints and
right-hand sides exactly.

## Run

```sh
cargo fmt --manifest-path Cargo.toml --all
cargo test --locked
cargo clippy --all-targets --locked -- -D warnings

cargo run --locked -- \
  ../../schemas/examples/energy-dispatch-pv-outage.synthetic.json \
  ../../target/energy-dispatch-mip-reference/essential_unserved_quanta_min.lp \
  ../../target/energy-dispatch-mip-reference/essential_unserved_quanta_min.translation.json
```

The qualification workflow generates the standalone lockfile from the exact pinned dependency versions and uploads it with the source and receipts. For a local first run, generate it with:

```sh
cargo generate-lockfile --manifest-path tools/energy-dispatch-lp-validator/Cargo.toml
```

The CLI accepts one request JSON, one named LP file and one receipt destination.
The LP filename identifies the objective and direction (for example
`grid_carbon_grams_max.lp`). It exits nonzero on every parse, metadata or
semantic mismatch.

## Independence and model semantics

- The request DTO is independently declared with unknown fields denied.
- The expected model is derived from request values using checked integer
  arithmetic; the validator does not call emitter functions or import the
  emitter crate.
- The parser accepts only this tool's restricted one-line LP profile:
  objective, named constraints, zero-based variable bounds, binary declarations,
  general-integer declarations and one terminal `End` marker.
- Unrecognized sections, duplicate declarations/rows/metadata, unknown symbols,
  missing bounds, trailing content, malformed signs and unsupported syntax fail
  closed.
- Integer charge/discharge efficiency is checked as two exact inequalities,
  preserving floor-division semantics.
- Model comparison checks exact coefficient maps, relations, RHS values,
  objective direction, variable domains and bounds. It does not use floating
  point or relax row/variable semantics.
- The LP source digest and model schema/objective/direction metadata are required
  and must bind to the exact request and filename.

## Receipt

A successful receipt records the validator ID/version, request SHA-256, LP
SHA-256, objective identity, reference semantic-model SHA-256 and parsed
semantic-model SHA-256. Those semantic digests must be equal.

The current SCIP/VIPR workflow consumes the receipt before each solve.
It checks that the receipt is equivalent and that its request digest, LP digest
and semantic-model digest match the exact request and LP bytes being solved.
Those bindings are carried into the solver result row. After VIPR checking, a
proof-binding JSONL record additionally commits the translation-receipt digest,
solver certificate digest, VIPR verification-log digest, validator version,
SCIP binary digest, VIPR checker digest and pinned VIPR source commit.

The temporal test oracle is now extended beyond the original one-interval case.
A two-interval charge-from-PV followed by an essential-load outage case
exhaustively compares 65,536 bounded integer assignments between the independent
LP semantic model and a separately written source-request schedule evaluator.
The evaluator checks SOC carry-forward, floor efficiency, mode exclusivity,
per-interval energy balance, load/PV partitions, outage import limits and
terminal reserve. A dedicated negative control accepts the known feasible
charge-then-discharge schedule and rejects it when the preceding SOC state is
corrupted. Three- and four-interval cases and broader request-domain generation
remain open qualification work.

This does not make the receipt a mathematical proof itself; it makes it an
auditable translation claim that is cryptographically associated with the
subsequent proof artifact and the exact solver/checker binaries.

## Current tests

The manually authored one-interval known-answer LP exercises all 13 variables,
13 constraints, the objective and all LP sections. Negative controls change a
coefficient, widen a bound, remove an integer declaration, delete a row,
inject an unsupported section, append trailing content, duplicate a section,
substitute source metadata and substitute the direction. Additional tests cover
request parsing, artifact naming and byte-size limits.

The first implementation tranche includes 1,000 deterministic semantic
mutations and a dedicated workflow that independently regenerates and validates
all twelve frozen-fixture objective/direction LPs. The SCIP/VIPR workflow also
runs the validator before each solve and emits twelve proof-binding records.
Remaining qualification work:
- expand exhaustive request-domain and multi-interval feasibility agreement
  beyond the one-interval tiny fixture;
- richer mutation classes for all individual constraints, domains and metadata;
- mutation of request/model/receipt/certificate associations and proof-log paths;
- a stable portable receipt encoding profile and explicit certificate schema version.


## Claim ceiling

A pass establishes that the parsed LP is semantically identical to the
independently derived integer model for the exact request and objective. It does
not establish that SCIP solved the model, that a VIPR proof is valid, that the
physical model is complete, that input evidence is authentic, or that an
operational dispatch is safe or authorized.
