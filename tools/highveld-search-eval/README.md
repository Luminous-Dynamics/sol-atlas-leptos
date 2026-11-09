# Highveld Search Eval v0.1

Offline Rust auditor for matched multi-objective benchmark-result bundles.

## Claim ceiling

The fixture in this directory contains **mathematical synthetic objective vectors only**. It is deliberately not a Highveld building simulation, architectural performance estimate, climate file, engineering result or benchmark of a commercial product.

A successful result means only that the bundle satisfies the declared benchmark-accounting rules and that the evaluator can derive the reported Pareto set / two-dimensional hypervolume under its exact mathematical fixture. It does not show that an algorithm is better for architecture, or that any building candidate is feasible, safe, accessible, compliant, affordable, buildable or comfortable.

The tool does not:
- generate architectural alternatives;
- calculate building energy, daylight, structure, fire, water, acoustics or cost;
- validate geometry, IFC files, climate data or material certificates;
- independently authenticate a declared SHA-256 value or a solver output;
- authorize building or robotic work.

A future Highveld result bundle must reference qualified geometry, exact solver/model/input revisions, and the relevant professional authority. Synthetic fixtures cannot be promoted to physical evidence.

## Run locally

From repository root:

```sh
cargo fmt --manifest-path tools/highveld-search-eval/Cargo.toml --all -- --check
cargo test --locked --offline --manifest-path tools/highveld-search-eval/Cargo.toml
cargo run --locked --offline --manifest-path tools/highveld-search-eval/Cargo.toml -- tools/highveld-search-eval/fixtures/mathematical-reference-v0.1.json
```

The crate is an isolated Rust 2024 nested workspace. `serde_json` is its only third-party dependency. No network, GIS, database, renderer, model or specialist solver is required.

## Bundle contract v0.1

The JSON fixture shows the complete first schema:
- exact fixture ID/digest, evidence tier, claim ceiling and enumeration state;
- shared objective schema digest, objective direction/unit and explicit normalization bounds;
- predeclared normalized hypervolume reference point;
- required hard gates with authority-scope labels;
- methods with algorithm/version/code digest, fixture/objective digest, stochastic state, candidate budget, high-fidelity and low-fidelity call budgets, wall-time budget and human-time budget;
- deterministic or repeated per-seed runs;
- per-run accounting split into training, tuning, validation and final high-fidelity calls, low-fidelity calls, elapsed wall time in milliseconds, human minutes and candidate records;
- candidate input and revision digests, explicit reported-feasibility declaration, dependency state, verification kind, solver-result digest, convergence, complete-input flag, validation scope, an explicit warnings array, hard-gate states, per-gate evidence-reference digests and objective values.

### Fail-closed evaluation

Candidate feasibility declarations are mandatory; missing declarations are not silently interpreted as false. Every candidate must include an explicit warnings array (empty only when no warnings are reported), exact revision/result digest-shaped identifiers, and exactly the declared hard-gate set. Each hard-gate status must have a corresponding evidence-reference digest-shaped identifier. These identifiers are not cryptographically verified against underlying artifacts in v0.1; the evaluator checks format and presence only.

A candidate only enters the feasible Pareto set when:
1. all required hard gates are explicitly `passed`;
2. dependency state is current;
3. a high-fidelity/exact fixture evaluation is supplied (not surrogate-only);
4. solver convergence and required-input completeness are true;
5. a non-empty validation scope is present;
6. warnings, if any, are acknowledged; and
7. all objective values are present and finite.

A reported-feasible candidate that does not satisfy these conditions raises `FALSE_FEASIBLE_DECLARATION`. A candidate that passes actual checks but is labelled infeasible is counted as a false reject, not silently removed. Failed/unknown hard gates are never traded against a better objective value.

Matched comparison requires the same fixture digest, objective schema digest, candidate budget, high-fidelity call budget, low-fidelity call budget, wall-time budget and human-time budget. Each run must report actual low-fidelity calls and elapsed wall time in milliseconds; both must stay within their predeclared budgets. High-fidelity training/tuning/validation/final calls must all be accounted for and stay within budget. This prevents an apparent search-quality improvement from hiding an asymmetric surrogate-evaluation allowance or runtime budget. The final-call ledger must contain at least one final call for every candidate labelled as high-fidelity. Stochastic methods require at least 20 unique seeds in this initial protocol.

### Metrics included

- objective normalization into a minimization convention, including maximize objectives;
- hard-gate/dependency/verification feasibility;
- per-run feasible Pareto set and dominated-candidate exclusion;
- false-pass and false-reject counts;
- exact two-objective hypervolume against a predeclared normalized reference point;
- explicit high-fidelity/low-fidelity call, wall-time and human-time accounting;
- exact reference-front derivation only from a declared, complete exhaustive finite enumeration.

### Metrics deliberately unavailable

IGD+, GD+ and epsilon indicators are not emitted in v0.1. They require a separately validated exact reference-front/metric contract and are not included simply to make a benchmark report look comprehensive. Hypervolume is currently exact only for two normalized minimization objectives; it is reported unavailable for other objective counts.

## Fixture interpretation

`mathematical-reference-v0.1.json` contains:
- one exact 11-candidate enumeration with a known six-point Pareto front;
- one 20-seed search-method fixture with the same per-run candidate, high-fidelity, low-fidelity, wall-time and human-time budgets;
- dominated candidates;
- one candidate with a very attractive objective pair but a failed hard gate;
- one candidate with an unknown hard gate;
- one surrogate-only candidate;
- one deliberately conservative false reject.

The 20-seed result is a structural test fixture repeated over seeds, not an optimizer performance experiment. Do not quote its hypervolume or Pareto coverage as evidence about NSGA-II or any real software.

## Determinism and evidence

The CLI reads the exact file supplied at invocation, validates the declared digests' format and equality across methods, and outputs deterministic JSON. It does not recalculate or cryptographically verify the declared fixture/code/objective SHA-256 values in v0.1; those digest strings are identity fields to bind upstream processes, not a substitute for fetching and hashing the underlying content.

Evidence ceiling: E1 synthetic benchmark-auditor software tests only.
