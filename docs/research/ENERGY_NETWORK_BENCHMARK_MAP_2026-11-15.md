# Sol-Atlas Energy Network Benchmark Map
**Date:** 2026-11-15  
**Parent:** [ENERGY-NETWORK-QUAL-001 / #332](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/332)  
**Related:** #308 exact finite dispatch oracle; #332 network-constrained dispatch; #352 rolling-horizon dispatch; #420 independent action admission; #147 joint water–power service resilience  
**Status:** research and benchmark protocol; targets below are proposals, not results.

## 1. The paradigm shift

Do not build another energy dashboard or a single optimizer that returns a number called "optimal." Build a **service-first, evidence-carrying network decision system**:

`source evidence -> network model -> model-fidelity audit -> candidate plan -> independent feasibility/optimality checks -> service-floor assessment -> human-authorized action -> observed outcome -> reconciliation`

The globe is a spatial index and decision interface, not the digital twin itself. A map location, capacity estimate, resource layer or model prediction must not silently become a verified asset, available MW, deliverable MWh, grid connection, or served household.

Every result should carry:
- immutable source and model identities, dates, geography, units and data rights;
- a declared physical and accounting boundary;
- exact solver/model version, configuration, hardware and time limits;
- independent feasibility and objective-recomputation receipts;
- uncertainty, unsupported physics, missing data and explicit abstention;
- per-service and per-community outcomes, not only aggregate system cost.

This composes existing Sol-Atlas work rather than replacing it: #308/#332/#352/#420 own dispatch, network and runtime admission; #293 owns source-to-service portfolios; #662 owns South African resource evidence; #147 owns joint water–power service resilience.

## 2. Industry comparators are complementary, not interchangeable

| Problem class | Primary reference | What it is good for | What Sol-Atlas must match |
|---|---|---|---|
| Transmission AC/DC OPF | PGLib-OPF + PowerModels.jl / MATPOWER | Public network cases, AC/DC OPF formulations and relaxation comparisons | Exact case, formulation, objective, constraints, solver, tolerances and hardware |
| Linear network planning / operations | PyPSA 1.4.0 | Network-constrained dispatch, unit commitment, rolling/multi-period and security-constrained workflows | Snapshot weights, network equations, investment boundary, commitment, contingencies and solver |
| Distribution and multi-voltage simulation | pandapower 4.x + SimBench | AC/DC and unbalanced/three-phase power flow, diagnostics, OPF/state estimation, grid data and time series | Same topology, component semantics, time series, voltage level, control and phase assumptions |
| Sequential grid control | Grid2Op / L2RPN | Sequential decision-making environments, agents/controllers, time-series scenarios and power-flow backends | Same action space, backend, chronics, reward, compute budget and illegal-action rules |
| Facility / campus / community microgrid | NLR REopt API v3 / REopt.jl | Technology sizing, techno-economics, energy dispatch and resilience | Same facility boundary, tariffs, technology options, outage assumptions, load and financial inputs |
| Finite exact reference | Sol-Atlas bounded oracle (#308) | Complete enumeration and Pareto ground truth for declared tiny discrete models | Exact same finite model, energy quantum, rounding and constraints |

Sources and current reference points:
- PGLib-OPF's published baseline uses PowerModels.jl 0.19.9, Ipopt.jl 1.4.1 / Ipopt 3.14.4, PGLib v23.07 and a dual Intel 2.10 GHz host with 128 GB RAM. Its listed solve times exclude roughly 2–5 seconds of Julia JIT; it uses HSL MA27, while the default linear solver is reported as 2–6x slower. Raw runtime comparisons without reproducing these boundaries are invalid. [PGLib baseline](https://github.com/power-grid-lib/pglib-opf/blob/master/BASELINE.md)
- The pinned PGLib v23.07 typical-condition reference rows report: IEEE 14-bus (DC $2,051.5/h, AC $2,178.1/h, QC/SOC relaxation gaps 0.11%/0.11%); IEEE RTS 24-bus (DC $61,001/h, AC $63,352/h, gaps 0.02%/0.02%); IEEE 57-bus (DC $34,773/h, AC $37,589/h, gaps 0.16%/0.16%). These are published comparator values under that source's setup, not Sol-Atlas targets or reproduced results. [PGLib baseline](https://github.com/power-grid-lib/pglib-opf/blob/master/BASELINE.md)
- PyPSA 1.4.0 was released on 9 October 2026. Its release notes include security-constrained multi-investment-period behavior changes, rolling-horizon unit-commitment fixes and explicit voltage-angle-difference limits. Pin the exact version and behavior profile rather than citing "PyPSA" generically. [PyPSA 1.4.0 release notes](https://docs.pypsa.org/latest/release-notes/)
- SimBench supplies 13 grid datasets from low to extra-high voltage with assigned load/generation time series. Its focus is Germany, so it is useful for reproducible cross-voltage method tests but is not a substitute for South African network evidence. The datasets use ODbL and individual contents use the Database Contents License. [SimBench download and licences](https://simbench.de/en/download/) · [SimBench dataset description](https://simbench.readthedocs.io/en/stable/about.html)
- Grid2Op is a sequential decision-making framework used for L2RPN competitions and supports multiple power-flow backends. It is a controller benchmark, not evidence that an agent is safe on a real utility grid. [Grid2Op documentation](https://grid2op.readthedocs.io/en/latest/)
- pandapower documents AC/DC and asymmetric three-phase power flow, OPF, diagnostics and state estimation; its latest documentation identifies the 4.0.0 series. [pandapower documentation](https://pandapower.readthedocs.io/en/latest/)
- NLR identifies REopt API v3 as the stable API and REopt.jl as its optimization engine. It is a facility/community/microgrid sizing and techno-economic comparator, not a transmission AC-OPF replacement. [REopt API v3](https://developer.nlr.gov/docs/energy-optimization/reopt/v3/)

## 3. Benchmark ladder

### A. Intake and model fidelity — start here

Cases pinned in the initial importer slice:
- PGLib-OPF v23.07 IEEE 14-bus: 14 buses, 5 generators, 20 branches.
- PGLib-OPF v23.07 IEEE RTS 24-bus: 24 buses, 33 generators, 38 branches; includes legacy `areas` compatibility metadata.
- PGLib-OPF v23.07 IEEE 57-bus: 57 buses, 7 generators, 80 branches.

Required metrics:
- source byte identity, release/commit, licence and attribution;
- exact row/column counts and source-to-typed-field parity;
- unsupported-field count, unknown-statement count and rejected-record count;
- duplicate IDs, unknown references, invalid status/bounds, bad units and invalid topology counts;
- connected components and reference-bus consistency;
- no silent defaults or field drops.

Acceptance targets (proposed):
- zero silently ignored model fields;
- 100% equality for every supported source field against a separately implemented MATPOWER/PowerModels parser;
- every unsupported field causes a typed rejection or an explicit, separately reviewed extension;
- deterministic receipts for identical input bytes;
- zero parser false-accepts in the frozen malformed-input and mutation corpus.

A source digest is byte identity, not evidence that the source itself is correct. Import acceptance is not power-flow feasibility.

### B. Static network feasibility and OPF

Keep DC-OPF, AC-OPF, QC/SOC relaxations and unbalanced distribution power flow as separate profiles.

For each model:
- independently recompute nodal active/reactive balances as applicable;
- check generator P/Q limits, voltage bounds, reference angles, branch thermal ratings, tap ratios and phase shifts;
- report maximum and distribution of constraint residuals in declared per-unit / MW / MVAr units;
- recompute objective values from source parameters and returned decisions;
- preserve primal incumbent, dual/best bound, relative gap, termination reason and time limit separately;
- compare at least two independent implementations on the same supported formulation.

Proposed first gates:
- zero constraint violations beyond the declared numerical tolerance in every accepted result;
- no accepted result with missing or unsupported constraints;
- objective recomputation agrees within the declared solver/numerical tolerance;
- on small cases, the bounded independent oracle enumerates all supported discrete choices and detects dominated or infeasible decisions;
- a local AC-OPF optimum is never called globally optimal without a valid global bound/certificate for the original non-convex model.

Do not claim superiority by comparing an AC formulation to a DC result, a relaxation bound to a local optimum, or different system boundaries.

### C. Chronological operation and resilience

Use SimBench time series only after pinning the exact dataset release, license, timestamps, scaling and missingness. Add South African profiles only when the evidence and rights are independently qualified.

Compare:
1. perfect-foresight offline bound (not an online policy);
2. deterministic receding-horizon dispatch;
3. scenario/stochastic dispatch;
4. robust-set dispatch;
5. replan-trigger policies (#403);
6. independent per-interval action admission (#420).

Report:
- essential and flexible unserved energy separately;
- longest continuous essential-service deficit;
- reserve/SOC trajectory and terminal reserve;
- curtailment, grid import/export, operating cost and emissions;
- storage throughput/degradation proxy with model limitations;
- time to first feasible, time to proven optimum, optimality gap and audit cost;
- solver calls, late/infeasible solves, replans, rejected actions and abstentions;
- restoration time and service-floor recovery by service cell/community.

Proposed gates:
- 100% of accepted schedules pass the independent schedule auditor;
- zero false-safe admissions on the frozen finite-state reference panel;
- unknown, stale or conflicting telemetry never improves a safety classification;
- all hard service floors are preserved or explicitly classified as infeasible/unknown;
- Pareto alternatives remain visible; cost, carbon and reliability are not hidden in one arbitrary weighted score.

### D. Sequential control and grid resilience

Use Grid2Op/L2RPN for a matched controller benchmark only after the simulator backend, topology, exogenous time series, action space, reward and compute budget are fixed. Add cyber/telemetry faults and N-1/N-k events as separately enumerated stress families.

Report:
- overload duration and magnitude;
- disconnected/served load and essential-service loss;
- illegal/ineffective actions and topology changes;
- recovery time, action count and intervention cost;
- false-safe, false-reject, fallback and abstention rates;
- wall-clock and compute budget per observation/action.

A high reward does not prove safety. The independent action-admission shield and human/utility authority boundary remain separate from the agent.

### E. Facility and community service

Use matched REopt v3 cases for facility/community technology sizing and economic resilience. Compose the result with water, thermal comfort, communications, health, mobility and compute service only when each service has an explicit time profile, asset boundary and qualified dependency graph.

The key score is not nameplate MW or asset uptime. It is **reliable useful service delivered to each declared beneficiary group**, with cost, carbon, water, land, material and access constraints reported independently.

## 4. How Sol-Atlas can compete differently

1. **Evidence before optimisation.** Treat every data import and model revision as a qualified artifact. A solver cannot repair an incorrect or unrepresentative network model.
2. **Exact small problems, verified large problems.** Use exhaustive finite oracles where complete enumeration is possible; use scalable solvers at larger sizes, then independently audit feasibility and objective values. An anytime incumbent is not a proven optimum.
3. **Decision-aware uncertainty.** Spend sensing and compute where new information can change a service decision before its deadline; forecast accuracy alone is not the operational objective.
4. **Service-first and distribution-aware.** Keep hospital, water, household, communications and other essential floors explicit by location and time. An aggregate optimum must not conceal a local service failure.
5. **Model discrepancy and abstention.** When topology, device behavior, sensor calibration or physics are not supported, widen uncertainty or abstain rather than inventing precision.
6. **Digital thread, not one giant model.** Symthaea proposes and explores counterfactuals; Symtropy owns physics/twin execution evidence; Sol-Atlas owns spatial/portfolio composition; Mycelix owns identity, rights, approvals and accountability. None self-authorizes the others.
7. **Operational learning with receipts.** Bind forecast, proposed action, authorization, actual actuation, observed effect and reconciliation as distinct events. A planned action is not an executed action.

## 5. Performance targets without benchmark gaming

Do not set a global "beat SOTA by X%" goal. Establish a matched baseline for each lane, then optimize the Pareto surface.

Initial proposed targets (unmeasured):
- importer: zero silent drops; exact source identity; field parity on the three pinned cases;
- DC-OPF: objective gap <= 0.1% to the best matched reference on supported cases, with independent feasibility checks;
- AC-OPF: report best-known primal objective and valid lower bound separately; no global-optimality claim without proof;
- online dispatch: zero false-safe admissions on the published adversarial finite panel;
- service resilience: no protected-floor regression under the declared robustness rule;
- runtime: publish cold/warm p50, p95 and p99 on fixed hardware, solver threads and budgets before selecting latency targets;
- robustness: report leave-case-out, topology, forecast, sensor, geography and source-shift results separately.

A candidate is not "better" if it has a lower objective but misses essential load, omits constraints, consumes more compute beyond budget, or uses future information.

## 6. Next engineering sequence

1. Complete strict, multi-case MATPOWER intake for PGLib 14/24/57.
2. Add independent field-parity receipts via a pinned MATPOWER or PowerModels.jl parser.
3. Add explicit importer support/rejection matrices for MATPOWER v2 extensions, including generator capability curves and extended cost forms; do not silently accept them.
4. Create a small hand-calculated DC network corpus and an independent DC-OPF feasibility/objective checker.
5. Add PGLib 118/300 and larger cases only after small-case parity is established.
6. Import one pinned SimBench MV/LV profile with explicit ODbL/DbCL handling, then add time-series chronology and distribution-level constraints.
7. Run matched PyPSA/PowerModels/MATPOWER comparisons and REopt facility cases; publish exact run manifests and raw result receipts.
8. Integrate the verified outputs with #352/#403/#420 and the Highveld service-floor work.

**Claim ceiling:** this note specifies research and benchmark work. It does not claim that Sol-Atlas currently solves network AC/DC OPF, beats any named comparator, qualifies a South African grid model, or authorizes real grid operation.
