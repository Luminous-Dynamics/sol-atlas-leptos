# Energy Assurance: SOTA Comparison and Research Strategy

**Snapshot date:** 2026-11-15  
**Status:** research strategy and benchmark contract; not a performance claim.

## 1. Strategic thesis

Sol-Atlas should not attempt to replace every mature power-system solver. It should build the evidence and assurance layer that makes heterogeneous models, optimizers, observations and human decisions comparable and reviewable.

The proposed pipeline is:

`source and rights -> admitted model -> model-fidelity evidence -> candidate solution -> independent physical audit -> service-floor assessment -> authorized decision -> observed outcome -> reconciliation`

A globe is a spatial index and interface, not a digital twin by itself. A candidate network, capacity estimate or dispatch schedule is not a verified physical asset, available MW, deliverable MWh, interconnection approval or served household.

The product differentiator should be **known-fidelity, evidence-carrying infrastructure decisions**:
- Every model has a declared validity envelope and explicit exclusions.
- Every important result binds to exact source inputs, parser/profile version, solver and configuration.
- The producing solver and the independent assessor are separate.
- Feasible, infeasible, unsupported, stale and unknown are different states.
- Aggregated economics never hide essential-service shortfalls or the distribution of harm.
- A proposed schedule expires; fresh observations and independent admission are required before each action.
- A physical effect is not accepted until separately observed and reconciled.

## 2. Industry and open-source baseline map

The table identifies the correct comparison domain. These tools are complements, not interchangeable implementations.

| System | Strong comparison domain | What Sol-Atlas must match before claiming an advantage | Gap where Sol-Atlas can differentiate |
|---|---|---|---|
| MATPOWER + PowerModels.jl | Established power-flow and OPF formulations; PGLib reference results | Exact network model, limits, costs, solver tolerances, reference bus, tap/phase semantics, objective and hardware | Source/rights lineage, model admission, independent receipt checks and beneficiary-service accounting |
| PyPSA 1.4.0 | Multi-period energy systems, network-constrained optimization, rolling horizon, unit commitment and security-constrained optimization | Same topology, time weights, availability, storage, constraints, formulation, solver and version | Evidence-fidelity envelope, independent action admission, uncertainty and evidence quality propagated to service outcomes |
| pandapower 4.0 documentation / 2026 release line | Distribution-oriented power flow, AC/DC OPF, contingency analysis and state estimation | Same balanced/unbalanced assumptions, element models, control settings and transformer conventions | Cross-tool semantic audit, rights-aware source provenance and refusal on unsupported model projections |
| Grid2Op 1.12.5 | Sequential grid-operation environments and controller/agent evaluation with interchangeable power-flow backends | Same environment, chronics, action space, protection rules, backend, reward and agent budget | Independent safety admission, evidence freshness, explicit unknowns and service-floor metrics |
| NREL REopt API v3 / REopt.jl | Building, campus, community and microgrid technology sizing, cost, resilience and energy performance | Same load, tariffs, technologies, outage duration, constraints, finance and objective | Evidence provenance, network context, independent post-solve audit and equity/service-floor outcomes |
| SimBench | Public distribution-to-extra-high-voltage networks, profiles and time-series comparisons | Exact selected grid code, version, scenario, time zone, profiles, licensing and data conversion | Auditable source intake, cross-system evaluation, South African transfer testing and service-accountability layer |

### Grounded reference points

- PGLib-OPF v23.07 publishes PowerModels.jl / Ipopt baseline objectives. For the IEEE 14-bus case, its table reports DC objective $2,051.5/h and AC objective $2,178.1/h; for IEEE RTS 24-bus, $61,001/h and $63,352/h; and for IEEE 57-bus, $34,773/h and $37,589/h. These are reference objective values, not Sol-Atlas results. The table explicitly notes that solve-time figures exclude approximately 2–5 seconds of Julia JIT and use HSL MA27; a default linear solver is stated to be 2–6x slower. Do not use the published timings without those qualifications. Source: https://github.com/power-grid-lib/pglib-opf/blob/master/BASELINE.md
- PyPSA 1.4.0 was released 9 October 2026. Its release notes include behavior changes affecting rolling-horizon unit commitment, ramp limits, and security-constrained optimization. A benchmark must pin the exact release and cannot treat "PyPSA" as a timeless result. Source: https://docs.pypsa.org/latest/release-notes/
- pandapower's current documentation exposes power flow, contingency analysis, OPF, short-circuit and state-estimation functionality. Its 2026 release notes also describe an OpenDSS feeder converter and redispatch development in its PandaModels interface. Source: https://pandapower.readthedocs.io/en/latest/ and https://pandapower.readthedocs.io/en/stable/about/changelog.html
- Grid2Op explicitly targets sequential power-grid decision-making and allows different power-flow backends; its documentation supports heuristic, optimization, MPC and learning-based agents. The GitHub release page lists 1.12.5 as its 2026 release. Sources: https://grid2op.readthedocs.io/en/latest/ and https://github.com/Grid2op/grid2op/releases
- REopt API v3 is the stable API line and uses REopt.jl as its optimization engine. Its stated scope includes buildings, campuses, communities and microgrids. Sources: https://developer.nlr.gov/docs/energy-optimization/reopt/v3/ and https://developer.nlr.gov/docs/energy-optimization/reopt/
- SimBench provides 13 grid datasets spanning low to extra-high voltage; the University of Kassel project description states annual 15-minute time series for one year. The official download page specifies ODbL and Database Contents License terms for the datasets. Sources: https://simbench.de/en/download/datasets/ and https://www.uni-kassel.de/eecs/en/sections/sustainable-electrical-energy-systems/our-developments/data-and-models.html

## 3. Paradigm shifts worth pursuing

### A. From "digital twin" as a label to a known-fidelity contract

Every layer should say what it can and cannot represent. A source-derived GIS point, planning estimate, asset register, survey, synchronized telemetry point and independently witnessed measurement must never collapse into the same visual category.

Proposed evidence tiers:
1. **Declared:** stated by a source or user, not independently checked.
2. **Structurally admitted:** schema, units, identity, rights metadata and required fields pass.
3. **Semantically cross-checked:** a second parser/model or a hand-audited reference agrees within a declared tolerance.
4. **Physically calibrated:** the model is compared against independent observations in a declared operating envelope.
5. **Operationally monitored:** state, freshness, drift, interventions and residuals are continuously evaluated.
6. **Reconciled:** predicted and observed outcomes are compared, with discrepancies preserved.

A tier is scoped by asset, time window, operating region and measurement boundary. There is no global "verified twin" bit.

### B. From solver self-certification to two-key computation

A producer proposes a result. A separately implemented verifier checks the result against the admitted inputs and declared equations. The verifier must reconstruct balances, limits, identities and objective values rather than accepting producer-supplied booleans.

For stronger assurance, the second implementation should eventually be an independently maintained codebase or a reference tool such as MATPOWER/PowerModels/PyPSA—not merely a second function that shares all parser and model assumptions.

### C. From single optimum to a qualified decision set

Expose Pareto alternatives and hard constraints separately:
- essential-service deficit;
- nonessential-service deficit;
- cost and affordability;
- emissions and embodied resources;
- water and other coupled resource requirements;
- resilience duration and recovery;
- uncertainty and evidence quality.

Do not hide essential-service failure inside an arbitrary weighted sum. A lower cost must not buy its way past a protected floor.

### D. From forecast accuracy to value of information

A forecast is useful only insofar as it changes a decision. Measure whether another meter, survey, inspection, weather update or field observation reduces expected service loss or uncertainty enough to justify its cost and delay.

Benchmark decision value, not only RMSE. Include situations in which the model should request more evidence, replan, choose a conservative fallback, or abstain.

### E. From schedule validity to expiring action proposals

A mathematically valid schedule is not a durable command. At each boundary, re-admit the next action against current telemetry, freshness, uncertainty, equipment state and a future-safe reserve condition. Missing, delayed, conflicting or out-of-envelope observations must cause replan, fallback or abstention—not automatic execution.

### F. From aggregate energy to useful service and distribution

The goal is not maximum MW or lowest average cost. The goal is reliable useful service: lighting, refrigeration, communications, water pumping/treatment, thermal comfort, medical loads and productive work.

Publish per-beneficiary and per-community outcomes, essential unserved energy, longest interruption, outage recovery time, affordability and concentration of failures. Do not let aggregate averages conceal a community that lost service.

### G. From one-way simulation to adversarial falsification

The digital twin should actively search for the conditions under which its own recommendation stops being valid: line outages, islanding, stale sensors, topology changes, unit mistakes, time shifts, missing profiles, common-cause failures and out-of-distribution weather/load. Preserve negative controls and count false accepts, false rejects and abstentions.

## 4. Benchmark ladder and target gates

These are proposed qualification targets, not results already achieved.

| Gate | Benchmark | Minimum evidence for promotion |
|---|---|---|
| 0. Deterministic foundations | Hand-calculated 2–5 bus and tiny dispatch cases | Exact known answers, dimensional checks, mutation negatives, deterministic outputs |
| 1. Import fidelity | PGLib-OPF v23.07 IEEE 14/24/57; then selected SimBench network | 100% field parity for the declared supported profile; zero silent drops; every unsupported field rejects or is explicitly preserved as unused metadata |
| 2. DC flow parity | Same fixed injections/topology in Sol-Atlas and MATPOWER or PowerModels.jl, then PyPSA | Per-branch flow and per-bus balance agreement within frozen tolerances; independent recomputation of all accepted receipts |
| 3. DC-OPF quality | Matched convex DC-OPF models | Report primal feasibility, independent objective recomputation, incumbent, valid bound, gap, termination reason and time budget separately; target <=0.1% relative gap on small, well-conditioned cases where the comparator establishes a valid bound |
| 4. AC-OPF fidelity | PGLib AC cases with a separate AC formulation | P/Q balance, voltage, reactive capability and branch constraints independently checked; local optimum and global bound reported separately; no global claim from local NLP convergence |
| 5. Chronological operation | SimBench 15-minute profiles, storage, outages and forecast splits | Leakage-resistant rolling-horizon results; essential-service floors, longest deficits, terminal reserve and recovery time reported |
| 6. Contingency and control | N-1/N-k, Grid2Op/L2RPN or RL2Grid-compatible panels | Complete contingency coverage; no false-safe admission on frozen adversarial tests; solver/auditor unknowns and abstentions remain visible; matched action budgets |
| 7. Physical calibration | Independently instrumented microgrid / feeder / facility | Calibrated model envelope, sensor/source lineage, synchronization, uncertainty, drift and residual evidence; independent operator/engineering review |

### Required reporting dimensions

Every benchmark row should bind:
- exact input source and content digest; rights and attribution;
- parser, normalized schema and model profile versions;
- topology, units, base MVA, time zone, interval weighting and boundary choices;
- solver, version, formulation, tolerances, threads, hardware and time limit;
- candidate objective, incumbent, best bound, reported gap and termination reason;
- independent feasibility receipt and recomputed objective;
- service floors, outage exposure, unserved service and beneficiary distribution;
- warm/cold p50/p95/p99 runtime, memory, energy use where measured, and time to first feasible candidate;
- all rejected, unsupported, missing, stale and unresolved cases.

A queued, skipped, timed-out or cancelled CI run is not a pass. A local test result, CI result, benchmark artifact and physical measurement are separate evidence classes.

## 5. How to beat the state of the art without making false comparisons

Do not compete on an unqualified composite "SOTA score". Pick a matched wedge and prove a better result.

**Wedge 1 — Evidence correctness.** Against every comparator, attempt the same stale, malformed, altered, rights-invalid and identity-mismatched inputs. Target zero silent drops and zero false acceptance on the frozen negative-control panel. Publish false rejects and abstentions too.

**Wedge 2 — Independent verification.** Take another solver's result and independently recompute feasibility and objective values from exact source inputs. This can create value even when the optimizer itself is not faster or better.

**Wedge 3 — Service-aware optimization.** On identical loads, outages, assets and horizons, compare the full service/cost/carbon/resilience frontier. Demonstrate better essential-service outcomes at equal cost, or lower cost at equal protected service, with a valid feasibility check.

**Wedge 4 — Robust sequential decisions.** Match Grid2Op or PyPSA rolling-horizon environments, use held-out chronics and unseen topologies, and compare policies under identical compute and action budgets. Include a deterministic independent shield. A learned controller is a proposer, never the sole safety authority.

**Wedge 5 — South African transfer.** Only after the benchmark harness is stable, admit South African grid and resource evidence with source rights, geography, timestamps, units and confidentiality review. German SimBench is a method benchmark, not a South African population or network proxy.

Only claim superiority when two independent reference stacks reproduce the same model, data boundary, feasible set, objective and stopping budget, and a blinded held-out panel confirms the result.

## 6. Immediate implementation sequence

1. Complete the current dependency chain: MATPOWER intake (#871), DC reference solver (#874), independent DC receipt auditor (#876), then the fixed-injection branch-outage sweep (#880).
2. Add a public normalized import projection with explicit field-by-field source-to-model mapping. Keep MATPOWER fields that are present but unused distinct from fields rejected as unsupported. Do not infer generator redispatch from a reference-bus slack injection.
3. Run at least two independent parsers on the exact PGLib bytes and compare every supported field before trusting network-scale results.
4. Create the small DC-OPF oracle and matched reference adapters only after DC flow parity. Keep this distinct from the existing one-site microgrid dispatch oracle (#308).
5. Add one selected SimBench MV feeder and a fully frozen profile/time-series manifest; verify licensing before redistribution.
6. Add full security-constrained operation only after generator redispatch, limits, reserve, corrective actions and post-contingency AC validation have explicit model contracts.
7. Use measured South African data only after source admission, rights and confidentiality gates are met.

## 7. Implementation language and tool strategy

Keep the trusted core Rust-first: source intake, typed model boundaries, immutable receipts, deterministic auditing, contingency orchestration and action-admission contracts. Keep model-critical logic out of untrusted browser code.

Do not force every mature reference implementation to be rewritten in Rust. Use pinned Python/Julia toolchains as isolated benchmark adapters where they are the native ecosystem for PyPSA, pandapower, MATPOWER/PowerModels and REopt. Adapters exchange frozen, versioned fixtures and result receipts; they do not become production authorities. Leptos/WASM can remain the interface layer, with JavaScript confined to the narrow browser boundary where necessary.

The goal is a Rust-owned evidence and assurance plane interoperating with the best domain solvers—not an ideological language rewrite that weakens comparisons.

## 8. Explicit non-claims

This strategy does not establish that current Sol-Atlas has AC-OPF, a full DC-OPF optimizer, calibrated grid twins, generator redispatch, N-1 security, field-validated models, South African grid truth, operational control or SOTA performance. The present DC kernel and branch-outage sweep are bounded research instruments. Each future capability must earn its own evidence tier and claim ceiling.
