# Energy Dispatch Independent LP Validator v0.1

A standalone Rust validator for the restricted integer-LP dialect emitted by
`tools/energy-dispatch-mip-reference`.

The validator does **not** depend on or import the emitter. It parses the LP
artifact, independently derives the expected integer model from the source
request, and compares objective, variables, domains, bounds, constraints and
right-hand sides exactly.

## Run

```sh
cargo fmt --manifest-path Cargo.toml --all -- --check
cargo test --locked
cargo clippy --all-targets --locked -- -D warnings

cargo run --locked -- \
  ../../schemas/examples/energy-dispatch-pv-outage.synthetic.json \
  ../../target/energy-dispatch-mip-reference/essential_unserved_quanta_min.lp \
  ../../target/energy-dispatch-mip-reference/essential_unserved_quanta_min.translation.json
```

The standalone lockfile is generated and committed by the qualification workflow.
For a local first run, generate it with:

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

This v0.1 receipt is a translation-validation artifact. The next integration
step is to require its exact LP digest and semantic digest in the SCIP/VIPR
certificate admission manifest, so certificates cannot be detached from the
request and translation validation that justified the model.

## Current tests

The manually authored one-interval known-answer LP exercises all 13 variables,
13 constraints, the objective and all LP sections. Negative controls change a
coefficient, widen a bound, remove an integer declaration, delete a row,
inject an unsupported section, append trailing content, duplicate a section,
substitute source metadata and substitute the direction. Additional tests cover
request parsing, artifact naming and byte-size limits.

This is an initial semantic validator tranche, not yet the full adversarial
qualification corpus. The next targets are:
- at least 1,000 deterministic named mutations with false-accept accounting;
- all 12 generated LPs against the frozen synthetic fixture;
- exhaustive small-request agreement against independent feasible-schedule
  enumeration;
- mutation of request/model/receipt/certificate associations;
- certificate-admission integration and receipt bundle provenance.

## Claim ceiling

A pass establishes that the parsed LP is semantically identical to the
independently derived integer model for the exact request and objective. It does
not establish that SCIP solved the model, that a VIPR proof is valid, that the
physical model is complete, that input evidence is authentic, or that an
operational dispatch is safe or authorized.
