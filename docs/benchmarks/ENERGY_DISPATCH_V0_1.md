# ENERGY-DISPATCH-001 — bounded exact microgrid dispatch oracle v0.1

Status: draft research contract. The synthetic fixture is a known-answer test, not field validation.

## Paradigm shift

Use an **exact small-instance oracle to measure decision regret**. Rather than treating an optimizer's success status as evidence that its result is good, exhaustively enumerate a deliberately small, fully specified integer model. This gives us a reproducible truth set for testing dispatch heuristics, learned controllers, and industrial/open optimization models. Only generalize beyond this finite domain after matched, held-out benchmark runs.

The oracle returns every non-dominated feasible schedule for its declared objectives, or fails closed when the search-expansion or Pareto-frontier limit prevents a complete answer. It never labels a truncated search exact.

## Model boundary and exactness semantics

- One site, one battery, fixed capacities, aligned chronological intervals; no network expansion or capacity-planning decisions.
- Every energy quantity is an integer multiple of `energy_quantum_kwh`. The floating-point quantum only converts a selected schedule into physical units; it does not take part in decisions, feasibility checks or objective comparisons.
- PV availability, grid import limits, essential and flexible demand, grid-outage annotations, integer cost/carbon factors and interval durations are explicit input data.
- A grid-outage interval has zero permissible grid import, regardless of its declared non-outage import limit.
- Charging uses bus-side input quanta; storage gain is `floor(input × charge_efficiency_numerator / denominator)`. Discharge chooses internal SOC withdrawal quanta; bus-side delivery is `floor(withdrawal × discharge_efficiency_numerator / denominator)`. Efficiency ratios must be in `(0, 1]`. This floor rule is part of the model, not a numerical approximation added after optimization.
- Simultaneous charge/discharge is prohibited. SOC cannot go below zero or exceed capacity. Initial SOC and terminal reserve must fit the capacity, and every retained full-horizon schedule meets the terminal reserve as a hard constraint.
- The bus balance is exact in integer quanta:
  `PV_used + grid_import + battery_discharge_delivered = essential_served + flexible_served + battery_charge_input`.
  Available PV is partitioned into used plus curtailed. Charging/discharging losses appear in the state transition and are not mislabelled as load service.
- Objectives are multi-dimensional: minimize essential unserved energy, flexible unserved energy, operating-cost units, grid-carbon grams and PV curtailment; maximize terminal SOC. A schedule is dominated only when another is no worse in every objective and strictly better in at least one. Terminal reserve and other physical constraints are feasibility conditions, never cost penalties.
- The request-level action-expansion cap is restricted to 1–1,000,000. The complete Pareto frontier is also capped at 128 alternatives. Either limit being exceeded returns an error and no assessment. These caps bound runtime and memory; they do not turn a partial search into a result.
- Provenance is propagated, not authenticated. A digest proves identity of bytes/assessment only; it does not establish meter truth, forecast quality, physical safety or model adequacy.

## Frozen synthetic known-answer case

Fixture: `schemas/examples/energy-dispatch-pv-outage.synthetic.json`.

There are two one-hour intervals and 1 kWh per energy quantum. In the first interval, 2 kWh of PV is available and there is no load; in the second, the grid is out and essential demand is 2 kWh. The idealized battery has 2-quanta capacity, unit efficiencies and no terminal reserve requirement.

Known result: charging from 2 PV quanta and discharging 2 SOC quanta in the outage can satisfy all essential demand with zero grid import, zero PV curtailment, zero cost/carbon units and zero terminal SOC. This is an intentionally lossless synthetic fixture to verify bookkeeping, not a battery-performance assumption.

## Mandatory correctness gates

1. Request and report schema are valid Draft 2020-12 schemas; fixture and generated report validate against them.
2. Every returned schedule closes the per-interval bus balance, partitions PV into used/curtailed, respects service demand bounds and storage state, avoids simultaneous charging/discharging, and imports no grid energy during an outage.
3. The reported frontier is pairwise non-dominated; deterministic enumeration gives deterministic serialized output and SHA-256 receipt.
4. Invalid efficiencies, impossible SOC/reserve declarations, chronology gaps/overlaps, missing evidence references for non-assumption inputs, and a candidate cap of one all fail closed.
5. The frozen fixture returns the hand-calculated zero-essential-shortfall schedule. Property/fuzz work should later enumerate thousands of generated bounded cases and independently re-check every output.
6. A search-cap or frontier-cap failure must be an error, never a truncated list with `search_is_exact=true`.

## Benchmark plan and target metrics

First publish the exact synthetic corpus and its model assumptions. For each case, record the input digest, model/version, quantization, candidate cap, action expansions, complete feasible schedules, frontier size, runtime, peak memory, objective vector, terminal reserve, essential unserved energy and longest essential-deficit run (from the service-reliability evaluator).

Proposed acceptance targets—not yet measured results:

- 100% recovery of frozen known answers.
- Zero hard-constraint violations across every reported schedule.
- 100% pairwise non-dominance of the published frontier.
- Deterministic output bytes for identical request bytes and build.
- 100% fail-closed behavior when either completeness cap is reached.
- A future generated suite of at least 10,000 small instances, with independent invariant checks and no silent infeasible outputs.
- For controller evaluations, report objective regret and essential-service shortfall against the exact oracle on held-out cases; do not average essential failures away inside a weighted scalar.
- On larger cases, report runtime/memory and anytime bounds separately from exact small-case results.

## Industry comparison: matched models, not marketing claims

- [NREL REopt](https://reopt.nrel.gov/) / [REopt API](https://developer.nrel.gov/docs/energy-optimization/reopt/): compare matched building/microgrid load profiles, outage windows, tariff, PV/battery assumptions, reserve and terminal SOC. REopt is an optimizer/platform; this tiny oracle is not a replacement.
- [PyPSA](https://docs.pypsa.org/latest/): match snapshots, storage-state constraints, efficiencies, standing losses and import/outage bounds. PyPSA targets broader multi-carrier network analysis and optimization.
- [GenX](https://genxproject.github.io/GenX.jl/stable/User_Guide/workflow/): compare only once the exact case can be represented without changing its boundary, integer resolution or policy. GenX addresses capacity expansion and dispatch problems at a different scale.

For each comparator, publish model translation, versions/solver status, tolerance, exact/continuous resolution differences, feasibility residuals, objective values, runtime and reproducible input/output digests. A solver's 'optimal' status is not evidence of matched semantics. Because this oracle is quantized and small by design, its strongest claim is calibration on its own finite model—not scalability or superiority over REopt, PyPSA or GenX.

## Explicit limitations / next work

This version does not model AC power flow, voltage/frequency constraints, feeder topology, multi-site sharing, thermal storage, heat-pump thermodynamics, PV forecast error, demand uncertainty, degradation, self-discharge, reserve uncertainty, startup/ramp constraints, inverter reactive power or lifecycle emissions. Add these as named, independently tested dimensions rather than silently broadening the claim boundary.

The next benchmark stage should compare a simple greedy baseline and at least one optimizer/controller against the oracle on the same finite model, then add held-out perturbations: longer outages, low SOC, lossy storage, competing essential/flexible loads, changing grid price/carbon, and terminal reserve. Only after those tests should the model be extended toward rolling-horizon control or larger network solvers.


## Independent completeness audit (test-only reference enumerator)

The follow-on reference audit uses a deliberately simple, separately written test-only enumerator. It enumerates every local integer action tuple, composes feasible state transitions across intervals, recomputes objective vectors directly from request fields, and independently derives the Pareto set. It does not call the production search, objective calculator, Pareto helper or schedule verifier.

The comparison checks both:
- the complete feasible-schedule count; and
- equality of the production and reference Pareto objective-vector sets.

A one-interval exhaustive domain covers 448 combinations of battery capacity/initial SOC/reserve, outage, PV, grid bound and essential/flexible demand. A deterministic generated suite covers 1,000 additional one- and two-interval cases, including lossy efficiency floor rounding. A negative control removes a real frontier member and requires the comparator to reject the corrupted assessment. Reference budget exhaustion is an explicit error; partial results are never returned.

This establishes agreement between two implementations over those bounded cases only. Both implementations still encode the same declared model and therefore can share a conceptual modeling error. Hand-derived known answers and later differential comparisons against external model formulations remain necessary. The slow reference enumerator is test-only and is not used in production dispatch.
