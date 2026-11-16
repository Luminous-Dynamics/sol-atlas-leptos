# Finite Network-Constrained DC Dispatch Oracle v0.1

A small, exhaustive reference instrument for the network-constrained dispatch programme in Sol-Atlas #332.

## What it solves

The oracle enumerates every non-reference generator output on an explicitly declared integer MW quantum. The reference-bus generator is not treated as an unconstrained mathematical slack: its output is derived from total demand minus all other generation, then checked against its own Pmin/Pmax.

For each complete assignment it:
1. constructs fixed net injections at every non-reference bus;
2. solves the declared linear DC network with the independent reference kernel;
3. sends the candidate receipt through a separately implemented DC auditor;
4. checks the reference generator balancing output, thermal Rate A limits and optional symmetric branch angle-difference bounds;
5. counts the assignment as feasible, declared-infeasible, solver-unknown or auditor-rejected;
6. maintains the complete nondominated frontier over integer operating-cost units and CO2e grams.

The oracle refuses a search whose Cartesian product exceeds the case's hard combination budget. It never returns a truncated frontier marked exact. Solver-unknowns and auditor rejections also prevent an exact Pareto-completeness claim.

## Three-bus known answer

The frozen analytic case contains 3 buses, 3 generators and 3 branches with 4 MW fixed demand. It has 16 non-reference generator assignments, of which 9 satisfy reference-generator bounds and network checks. The expected complete cost/carbon Pareto objective pairs are:

| Operating-cost units | CO2e grams |
|---:|---:|
| 16 | 32 |
| 18 | 23 |
| 20 | 14 |
| 22 | 5 |

These are synthetic integer accounting units, not money or a measured emissions estimate. The fixture is a known-answer test of enumeration, network constraints and Pareto completeness.

Run:

    cargo test --manifest-path tools/dc-opf-finite-oracle/Cargo.toml --locked --offline
    cargo run --manifest-path tools/dc-opf-finite-oracle/Cargo.toml --locked --offline -- --self-test

## Why this is a paradigm shift

Instead of asking only whether a solver says "optimal", retain a small domain where the complete finite feasible set is enumerable. This gives a calibration oracle for:
- checking missed feasible schedules and false-feasible claims;
- measuring optimality and Pareto regret of larger optimizers;
- validating cost/emissions trade-offs without collapsing them into a single arbitrary score;
- exercising network constraints with known outcomes before scaling to continuous DC-OPF;
- measuring the error introduced by power quantization separately from solver approximation.

The next comparison is to compile the same tiny cases to PyPSA and PowerModels/MATPOWER with matched fixed injections, branch constraints and objective coefficients. The finite oracle's quantum must be represented identically; continuous reference results should be compared to the lattice optimum as a lower-bound/cross-check, not mislabeled as a same-domain result.

## Exactness and model boundary

"Exact" means complete enumeration of this finite declared model and independent auditing of each returned DC solution. It does not mean continuous optimization or physical truth.

The current profile deliberately excludes:
- AC voltage magnitude, reactive power and resistance losses;
- generator commitment, startup/ramp/reserve constraints;
- storage, chronology, uncertainty and corrective redispatch;
- island-specific reference generators or multiple reference islands;
- load shedding (all declared demand is fixed and must be served);
- phase-angle controls other than fixed branch tap and phase shift;
- market settlement, tariffs, currency conversion or real-world emissions claims.

An islanding/unsupported solve is counted as unknown and blocks the exact frontier claim. source_identity is a caller-supplied identity label; pass an exact source digest when this case is derived from a pinned source.

## Promotion targets

- 100% assignment coverage within the declared finite search budget.
- Zero auditor false accepts on the frozen mutation panel.
- Zero objective recomputation mismatch.
- Zero missing/duplicate candidate assignments.
- Exact cost/carbon Pareto frontier on every published tiny case.
- Continuous DC-OPF matched comparisons only after field-parity and numerical-reference gates in #332 are green.

This crate is a research oracle, not an operational dispatch engine or an AC-OPF replacement.
