# Energy temporal-mesh value-of-information allocator v0.1

A dependency-free Rust research benchmark for allocating rolling-horizon temporal resolution under an explicit compute budget.

## Why a separate benchmark

Adaptive temporal mesh and value-of-information optimization are active research topics. In particular, the 2026 Greedy-VoI time-mesh paper already studies block granularity and horizon selection under explicit per-cycle compute budgets: https://www.mdpi.com/2624-6511/9/2/30. This prototype does not claim to invent adaptive meshing. It tests a narrower, auditable contract: mandatory safety windows are refined first; remaining interval budget is allocated by conservative loss lower-bound per marginal interval; every event boundary is preserved exactly; an insufficient safety budget is a hard failure, not a coarse schedule with a green status. The differentiating research question is whether these non-compensating safety and provenance constraints improve the practical value of compute allocation when the resulting mesh is later used by a separate schedule optimizer and audited trajectory pipeline.

It complements the trigger-policy bake-off in #403 and the rolling-horizon programme in #352.

## Strategies

1. `uniform-60m`: coarse reference, low compute, deliberately exposes any safety-resolution violations.
2. `uniform-15m`: fine reference, high compute; reports budget exhaustion rather than claiming an infeasible plan is usable.
3. `safety-first-voi`: exact event-boundary partition, refine all safety-critical windows to 15 minutes first, then repeatedly recompute optional-window lower-bound value per marginal interval after each refinement, including overlapping windows.

Compute cost is currently interval count. This is a transparent structural proxy, not measured solver CPU time. The next calibration step is to fit cost against exact solver traces by horizon, constraint count, binary-variable count, and solver/backend. The 2026 literature also demonstrates the need to model nonlinear battery losses and cyclic degradation in matched SAC/MPC/optimization comparisons (IEEE Access, 6 October 2026, https://doi.org/10.1109/ACCESS.2026.3740101); this crate does not yet model either. Do not interpret interval count as milliseconds.

## Hard invariants

- Mesh covers exactly [0, 24h] with no gaps or overlaps.
- Every source event start/end boundary is preserved.
- Each cell is no longer than its declared resolution.
- Safety-critical windows are all represented at 15-minute resolution or the plan is rejected.
- The selected mesh never exceeds its declared budget.
- Event ordering does not change the selected mesh.
- The verifier recomputes cost, remaining budget, unresolved loss, safety violations and the complete refined-event ID set; it rejects strategy relabelling and mandatory-window coarsening.
- Value-density cross-products use 128-bit arithmetic so large u64 loss bounds cannot collapse into a false tie through saturation.
- Unresolved loss is reported separately from compute cost; it is an event-wise synthetic sum and may count overlapping event bounds separately. There is no arbitrary weighted scalar.

## Frozen synthetic panel

32 deterministic scenarios: 8 event regimes x 4 lower-bound value scales. Event types include evening peak, solar ramp, storm outage, telemetry recovery, equipment derate, price spike, load/PV mismatch, and compound shock. The four compound-shock variants use compute budgets of 30, 33, 36 and 45 cells to force no optional refinement, partial refinement and full refinement while keeping the 120-minute safety window mandatory.

Every field is synthetic and dimensionless. The panel is not calibrated to measured Johannesburg/South African load, PV, tariff, outage or battery data. It is a mechanics test of boundary preservation, resource allocation, and fail-closed behavior.

## Metrics

The CSV report emits per-scenario, per-strategy compute budget/cost, remaining budget, safety-resolution violations, unresolved loss lower bound, acceptance disposition, and selected event windows. Compare the Pareto frontier; do not rank by a single weighted score. The current allocator's greedy rule is a deterministic heuristic, not a globally optimal knapsack solver; the 32-case report makes its choices inspectable.

## Run

```sh
cargo fmt --manifest-path tools/energy-temporal-mesh-voi/Cargo.toml --check
cargo check --locked --all-targets --manifest-path tools/energy-temporal-mesh-voi/Cargo.toml
cargo test --locked --all-targets --manifest-path tools/energy-temporal-mesh-voi/Cargo.toml
cargo clippy --locked --all-targets --manifest-path tools/energy-temporal-mesh-voi/Cargo.toml -- -D warnings
cargo run --release --locked --manifest-path tools/energy-temporal-mesh-voi/Cargo.toml
```

## Validation boundary

`verify_mesh` is a structural checker in the same crate, not an independently implemented verifier. It checks coverage, chronology, exact event boundaries, declared resolution semantics, budget accounting, mandatory safety resolution and the recomputed refined-event set. It does not certify that the event lower bounds or safety labels are physically correct.

## Claim ceiling

This is a finite, synthetic temporal-mesh allocation benchmark. It does not solve dispatch, measure real solver runtime, prove optimality, model electrochemical degradation, predict physical grid stability, or authorize actuation. Every actual dispatch trajectory still requires source-bound generation and the independent #351 schedule auditor.
