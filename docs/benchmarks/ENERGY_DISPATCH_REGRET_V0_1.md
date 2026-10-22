# ENERGY-DISPATCH-REGRET-001 — baseline regret versus exact oracle v0.1

Status: draft. No measured performance claim is made until the Rust workflow has run and the corpus is published.

## Purpose

The exact oracle is a calibration instrument for tiny finite problems. This layer adds a deliberately simple, deterministic controller so we can measure avoidable shortfall and cost rather than declaring success because a heuristic or optimizer returned a schedule.

## Baseline policy contract

Version: `pv-grid-battery-priority-v0.1`.

1. Meet essential demand before flexible demand.
2. Allocate available PV directly to essential then flexible demand.
3. Use grid imports to serve remaining essential demand, then flexible demand, subject to the import limit; grid imports are exactly zero during declared outages.
4. Discharge the battery only for remaining load, after available PV and grid. The selected integer discharge output may not exceed residual load, so energy cannot disappear into an unmodelled dump load.
5. Preserve terminal reserve as a hard floor during the trajectory. This is conservative and can be suboptimal compared with schedules that temporarily go below reserve and recharge later.
6. Charge only from remaining PV surplus, never simultaneously with discharge. Use the same declared rational efficiency and floor rounding as the finite oracle.
7. Return a schedule only when all bus balances close and terminal reserve is met. If this policy fails while the exact model is feasible, record a **baseline feasibility failure**; do not fabricate an objective vector.

The controller intentionally does not optimize the tariff or carbon. It prefers grid import to battery discharge whenever grid supply is available. This is a transparent choice designed to expose cost and emissions regret. A stronger policy is a future candidate, not a hidden change to this baseline's semantics.

## Comparison semantics

The oracle emits the full non-dominated Pareto frontier. This benchmark explicitly selects the first alternative under the oracle's documented lexicographic ordering:
1. essential energy unserved;
2. flexible energy unserved;
3. operating-cost units;
4. grid-carbon grams;
5. curtailed PV;
6. maximize terminal state of charge.

This is a declared benchmark reference policy, not a universal social welfare function. The report also preserves the full objective tuple, but a single scalar score is intentionally omitted.

For a feasible baseline, report signed per-objective deltas against that one selected reference. For minimized dimensions, `baseline - reference` is positive when the baseline is worse. For terminal SOC, `reference SOC - baseline SOC` is positive when the baseline ends with less energy. A negative delta on one dimension can coexist with a worse result on another; interpret the vector, not a sum. If the baseline is infeasible, its objective and delta vector are null and the failure reason is recorded.

## Independent schedule verification

The comparison path rechecks every schedule in the exact oracle's Pareto frontier and every schedule emitted by the baseline through `energy_dispatch_verifier.rs`. That module does not call the schedule generator. It independently checks interval metadata against the request, demand metadata, grid caps/outage import, PV use/curtailment, essential/flexible demand reconciliation, integer bus balance, SOC continuity, efficiency rounding, charge/discharge bounds, no simultaneous charge/discharge and terminal reserve. Forged residual fields cannot override the independently calculated balance.

This remains a model-consistency verifier. It does not authenticate meter readings, hardware behavior, or field evidence.

## Frozen demonstrations

### PV-charge before outage

Use `schemas/examples/energy-dispatch-pv-outage.synthetic.json`. The PV-first baseline should charge two 1 kWh quanta before an outage and discharge them to deliver two essential quanta. It should match the exact oracle's zero essential shortfall and zero grid import in this synthetic lossless case.

### Deliberate grid-first cost regret

A one-interval case starts with 1 SOC quantum and 1 quantum of essential demand, no PV, grid import capacity of 1, and grid cost of 10 units per quantum. The baseline buys from the grid before discharging the battery, so its cost is 10. The exact lexicographic oracle can discharge the battery and pay zero grid cost, exposing a cost delta of +10 for the baseline. That trade-off is measured in the declared model only.

### Visible heuristic failure

When the opening SOC is below a hard terminal reserve and the only possible way to recharge is from grid energy, the baseline may fail because it deliberately does not grid-charge. The exact model can still produce a feasible schedule. The comparison must record this as a failure rate, not pretend the controller handled it.

## Metrics and acceptance

Report at minimum:
- baseline feasibility rate and count of hard-constraint failures;
- essential and flexible unserved energy;
- cost and grid-carbon objective deltas as separate signed quantities;
- PV curtailment and terminal SOC;
- longest essential deficit and outage-only deficit, computed by applying the reliability evaluator to generated service traces in the next iteration;
- exact-oracle search expansions and feasible-complete-schedule count;
- input digest, exact model/policy versions, quantum, build identity and stable output digest;
- wall time and peak memory when the execution environment can measure them.

Proposed target gates (not results):
- 100% independently rechecked hard constraints on every schedule the baseline labels feasible.
- Frozen outage known-answer recovered with zero essential shortfall.
- The grid-first fixture reports +10 cost units of baseline regret under the exact declared assumptions.
- Zero hidden objective substitution: every metric delta is explicit, and baseline infeasibility remains a separate observable result.
- Run at least 10,000 deterministically generated bounded cases before publishing any aggregate controller result; separate development, tuning, validation and held-out test cases.
- Publish full distributions and worst cases, not only an average. Keep essential load-shedding independent of cost/carbon scores.

## Industry context and comparison protocol

The reference systems solve broader optimization problems and should not be described as beaten by this small heuristic or oracle:
- [REopt platform](https://www.nlr.gov/reopt/about) optimizes distributed energy planning and operation, with cost and resilience analyses.
- [PyPSA objective documentation](https://docs.pypsa.org/latest/user-guide/optimization/objective/) represents dispatch and investment costs under the modelled constraints.
- [GenX model introduction](https://genxproject.github.io/GenX.jl/stable/Model_Concept_Overview/model_introduction/) covers coupled planning, dispatch, storage, reserves and network power-flow decisions.

For a valid external comparison, translate the same case and record solver/model versions, objective, initial/terminal SOC, efficiencies, grid/outage limits, time-step durations and any discretization gap. Compare decisions and hard residuals, not just solver status. Since the oracle has one storage device and a tiny discrete horizon, it is a truth-set generator for that finite model—not a substitute for commercial-grade techno-economic analysis or large-scale network optimization.

## Limitations and next work

This baseline ignores tariff when dispatching, never charges from the grid, has no forecast or receding horizon, no self-discharge/degradation, no multi-battery coordination, no AC flow, no inverter constraints and no uncertainty model. Keep its version fixed as a benchmark baseline. Next add:
1. independent schedule re-audit through a separate verifier;
2. generated-case corpus and reliability traces for deficit-run metrics;
3. one cost/carbon-aware heuristic as a second comparison policy;
4. matched input/output translation to REopt/PyPSA on cases whose assumptions are exactly representable;
5. held-out regret, feasibility rate and worst-case analysis.
