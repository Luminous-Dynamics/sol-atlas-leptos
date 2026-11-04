# SA Economic Portfolio Oracle

A small, standalone Rust reference evaluator for [SA-PORTFOLIO-OPT-001](../../docs/research/SA-ECONOMIC-PORTFOLIO-001.md).

This is a **synthetic contract benchmark**, not a production national-investment planner.

## Run

From the repository root with Rust/Cargo installed:

```sh
cargo test --manifest-path tools/sa-economic-portfolio-oracle/Cargo.toml
cargo run --manifest-path tools/sa-economic-portfolio-oracle/Cargo.toml
```

The default command evaluates the frozen synthetic input, checks its exact feasible set and Pareto frontier against the expected manifest, and prints a JSON receipt with the input SHA-256 digest. Optional positional arguments can select an alternate input and expected manifest:

```sh
cargo run --manifest-path tools/sa-economic-portfolio-oracle/Cargo.toml -- INPUT.json EXPECTED.json
```

The evaluator enumerates every project subset and rejects inputs above 20 projects rather than silently truncating. V0.1 additionally freezes the eight project IDs, two named unweighted scenarios, four maximization objectives and interaction semantics. It is a correctness oracle, not the larger-instance optimization method.

## Important limits

- All costs and objective values are arbitrary synthetic units.
- Scenario cases are stress cases, not probabilities.
- The frontier preserves non-dominated alternatives; it does not choose society's priorities or a single winner.
- The UNKNOWN-gate decoy is deliberately assigned implausibly high values to verify that a failed evidence gate cannot be compensated by apparent benefit.
- Input hashing identifies the exact bytes evaluated. It does not authenticate a publisher or establish the truth of data.
- Passing the tests does not establish real ROI, causal service improvements, jobs, engineering validity, legal permission or investment authority.
