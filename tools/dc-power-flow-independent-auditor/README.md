# DC Power-Flow Independent Auditor v0.1

A standalone Rust checker for candidate DC power-flow receipts. It depends only on the public data types of the small-network reference solver and does not call the solver.

## Independent checks

From source topology and the candidate bus-angle vector, the auditor independently reconstructs:
- energized connected components and exactly one reference bus per component;
- slack injection as the negative sum of specified injections in each island;
- each active branch flow, including transformer tap and phase-shift terms;
- per-bus net outflow and nodal balance residual;
- Rate A overloads and the declared feasibility flags.

It checks receipt identity, field completeness, numeric finiteness, source-to-result bus/branch ordering, reported aggregates and all three feasibility flags. It returns a claim-ceiling-tagged report or a typed rejection reason.

## Important limits

This is independent *receipt consistency* for the declared linear DC model. It does not independently validate source truth, AC physics, reactive power, losses, generator limits, transient stability, protection, or real-grid safety. The solver and auditor are separate crates with separate implementations, but both consume the same typed input model; field-parity against an external parser is a separate gate.

Pass the same numerical thresholds used by the solver so the feasibility flags have the same declared semantics. A pass does not grant dispatch or switching authority.

## Run

```sh
cargo test --manifest-path tools/dc-power-flow-independent-auditor/Cargo.toml --locked --offline
cargo clippy --manifest-path tools/dc-power-flow-independent-auditor/Cargo.toml --locked --offline --all-targets -- -D warnings
```
