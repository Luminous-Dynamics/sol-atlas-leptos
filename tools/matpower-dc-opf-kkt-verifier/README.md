# MATPOWER 8.1 DC-OPF Numerical KKT Verifier v0.1

**Status:** runner-independent Rust implementation draft; exact-head CI and independent reference execution pending.

Formatting is checked with the workflow's pinned stable Rust toolchain across the prerequisite primal auditor and this verifier. Formatting success is a prerequisite only; it is not a compilation, numerical-reference or physical-validation result.

This crate independently checks numerical KKT conditions for the bounded MATPOWER 8.1 `makeBdc` DC-OPF profile. It consumes the source-bound primal receipt from the adjacent `matpower-dc-opf-primal-auditor` crate and an additional set of named dual multipliers exported by the pinned MATPOWER 8.1 reference script.

It does not optimize. It does not call MATPOWER. It does not trust the solver's success flag, reported KKT norm, or reported primal-dual gap.

## Claim boundary

A passing result means the exact source-bound convex DC-OPF model and candidate satisfy the verifier's separately computed primal, dual-sign, stationarity, and complementarity checks within the declared floating-point tolerances.

**This is numerical KKT evidence, not an exact-arithmetic proof of global optimality.** The report explicitly sets `formal_global_optimality_proof=false` and `numerical_gap_is_rigorous_lower_bound=false`. The numerical dual-infimum and primal-dual-gap values are estimates; they are useful diagnostics but are not rigorous lower bounds because all calculations use floating-point arithmetic.

For an exact global-optimality proof, a later version would need a rigorously bounded arithmetic path (for example, exact-rational reconstruction or interval arithmetic with a validated residual correction). A local solver termination code cannot substitute for that work.

The result does not establish AC-OPF feasibility, AC system stability, N-1 security, protection, physical model accuracy, grid connection, or operational authority.

## Supported model

The initial profile is deliberately narrow:

- MATPOWER 8.1 DC OPF using the `makeBdc` formulation.
- Variables: bus voltage angles in radians and active generator dispatch in per unit.
- Constraints: active-power balance (`Pmis`), branch active-flow limits (`Pf`), branch angle-difference limits (`ang`), and variable bounds on `Va` and `Pg`.
- Generator costs: MATPOWER polynomial model 2, constant, linear, or convex quadratic rows; non-convex quadratic costs are rejected by the upstream primal auditor.
- Startup and shutdown costs must be zero.
- Source extensions or unsupported constraints must be rejected by the source/primal admission layer rather than silently omitted.

MATPOWER's internal variables and constraints are scaled in per-unit. The verifier preserves that scaling when applying dual multipliers and evaluating stationarity. It maps each dual row back to the original external bus ID or source row using the `results.order.*.i2e` mapping in the MATPOWER exporter.

## Receipt format

The receipt includes the existing source-bound primal rows and the following dual rows:

```text
DUAL_META   1   lower_upper_nonnegative
DUAL_BUS    <bus_id>      <mu_lower> <mu_upper>
DUAL_VA     <bus_id>      <mu_lower> <mu_upper>
DUAL_GEN    <source_row>  <mu_lower> <mu_upper>
DUAL_FLOW   <source_row>  <mu_lower> <mu_upper>
DUAL_ANGLE  <source_row>  <mu_lower> <mu_upper>
```

All multiplier values must be finite. Lower- and upper-bound multipliers must be non-negative. The verifier requires exact identity coverage: no missing, duplicate, or extra multiplier rows are accepted.

- `DUAL_BUS` contains the lower/upper multipliers for the nodal active-power equality in internal bus order, remapped to external bus IDs.
- `DUAL_VA` contains variable-bound multipliers for voltage angles, mapped to external bus IDs.
- `DUAL_GEN` contains generator-dispatch variable-bound multipliers, mapped to original generator source rows.
- `DUAL_FLOW` contains `Pf` lower/upper multipliers for exactly the in-service branches with finite nonzero Rate A.
- `DUAL_ANGLE` contains angle-difference lower/upper multipliers for exactly the branches selected by MATPOWER's `makeAang` rules.

The dual exporter in `tools/matpower-reference-differential/run_matpower_dcopf.m` uses MATPOWER's named `results.lin.mu` and `results.var.mu` fields and maps internal result order back to source identities. These values are observations from the producing solver, not certificates by themselves.

## Independent checks

1. Re-run the existing primal auditor on the primal-only portion of the receipt.
2. Reconstruct the MATPOWER 8.1 `makeBdc` branch susceptance, tap and phase-shift terms from the original source.
3. Reconstruct nodal active-power balance and every admitted flow/angle constraint.
4. Require complete dual identity coverage and non-negative lower/upper multipliers.
5. Recompute stationarity for every voltage-angle and generator-dispatch variable.
6. Recompute lower/upper slack and complementary-slackness products for each inequality and variable bound.
7. Reconstruct the convex quadratic Lagrangian and report its numerical infimum/gap estimate separately from the KKT residuals.

The implementation assembles `Bbus^T * lambda` branch-by-branch; it does not create a dense bus-by-bus matrix.

## Metrics

The JSON report keeps separate:

- maximum nodal equality residual (p.u.);
- maximum branch-flow bound violation (p.u.);
- maximum angle-difference violation (rad);
- maximum generator-bound violation (p.u.);
- maximum fixed-reference-angle violation (rad);
- maximum dual-sign violation;
- maximum stationarity residual for `Va` and `Pg`;
- maximum and summed complementarity products;
- primal objective, Lagrangian value at the candidate, numerical dual-infimum estimate, and numerical primal-dual-gap estimate.

A single scalar score is intentionally not used to conceal a failed KKT family.

## Benchmark fixtures

P0 tests in this crate:

- Hand-calculated two-bus quadratic dispatch with analytically known objective and nodal multipliers.
- Two-generator, two-bus active-line case: a 40 MW line limit binds, the cheap generator supplies 40 MW, the expensive generator supplies 60 MW, and a nonzero line shadow price is required for stationarity.
- Incorrect bus multiplier mutation that must fail stationarity.
- Incorrect active-line shadow-price mutation that must fail stationarity.
- Negative dual multiplier that must fail dual feasibility.
- Missing multiplier identity that must fail closed.
- Nonzero multiplier on a slack line limit that must fail complementary slackness.

The dedicated workflow runs both hand-derived fixtures through the pinned MATPOWER 8.1 stack, exports primal and named dual receipts, then independently verifies them with the Rust implementation. These are synthetic known-answer cases, not evidence of physical grid performance.

The adjacent PGLib IEEE 14, RTS 24, and IEEE 57 source cases are the next reference differential once the MATPOWER 8.1 dual export workflow executes. The PGLib published DC objective column uses PowerModels' DCP convention; it must not be compared directly with MATPOWER's makeBdc profile. Use a matched MATPOWER 8.1 reference run or PowerModels DCMP when checking numerical parity.

## Run

With the pinned MATPOWER 8.1 / MP-Opt-Model 5.0 / MIPS 1.5.2 / MP-Test 8.1 reference stack and GNU Octave available:

```sh
cargo test --manifest-path tools/matpower-dc-opf-kkt-verifier/Cargo.toml --locked --offline
cargo clippy --manifest-path tools/matpower-dc-opf-kkt-verifier/Cargo.toml --locked --offline --all-targets -- -D warnings

octave-cli --quiet --no-gui --eval "addpath('tools/matpower-reference-differential'); run_matpower_dcopf('case.m', '/tmp/case.tsv', 'vendor/matpower', 'vendor/mp-opt-model', 'vendor/mips', 'vendor/mptest', 'SOURCE_GIT_BLOB_SHA1')"

tools/matpower-dc-opf-kkt-verifier/target/debug/matpower-dc-opf-kkt-verify case.m /tmp/case.tsv

# Binding-line known-answer fixture
# The pinned workflow runs the matching MATPOWER 8.1 solve first, then:
tools/matpower-dc-opf-kkt-verifier/target/debug/matpower-dc-opf-kkt-verify \
  tools/matpower-dc-opf-kkt-verifier/fixtures/two_bus_binding_line.m \
  /tmp/matpower-binding-line.tsv
```

The CLI exits successfully only when the primal auditor and every numerical KKT family pass. The JSON report retains the exact claim ceiling.

## Upstream references

- MATPOWER 8.1 `dcopf_solver`: named variable/linear dual extraction and scaling.
- MATPOWER 8.1 `opf_setup`: `Pmis`, `Pf`, `ang`, `Va`, `Pg` model construction.
- MATPOWER 8.1 `opf_execute`: named `results.lin.mu` and `results.var.mu` result blocks.
- MATPOWER 8.1 documentation: https://matpower.org/doc/ref-manual/legacy/functions/opf.html
- MATPOWER `mp.opt_model.parse_soln`: https://matpower.org/doc/mpom/classes/mp/opt_model.html
