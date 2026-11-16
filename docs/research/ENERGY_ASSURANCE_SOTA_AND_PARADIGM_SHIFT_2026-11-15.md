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
| pandapower 3.5.6 (current stable) / 4.0 development line | Distribution-oriented power flow, AC/DC OPF, contingency analysis and state estimation | Same balanced/unbalanced assumptions, element models, control settings and transformer conventions | Cross-tool semantic audit, rights-aware source provenance and refusal on unsupported model projections |
| PowerIO 0.12.1 | Rust-first multi-format grid data compiler with source retention, typed models, diagnostics and MATPOWER/PSS/E/PowerModels/pandapower/PyPSA exchange | Same exact source files, field inventory, conversion options, semantics and diagnostic policy | Sol-Atlas should build on its broad interoperability rather than duplicate it; compete on independent admission, model validity, operational evidence, service outcomes and assurance receipts |
| matpowercaseframes 2.1.1 | Python MATPOWER case reader and DataFrame representation | Exact same `.m` bytes, supported fields, units and ordering | Third independent parser for mutation tests and cross-checking PowerIO's interpretation |
| Grid2Op 1.12.5 | Sequential grid-operation environments and controller/agent evaluation with interchangeable power-flow backends | Same environment, chronics, action space, protection rules, backend, reward and agent budget | Independent safety admission, evidence freshness, explicit unknowns and service-floor metrics |
| NREL REopt API v3 / REopt.jl | Building, campus, community and microgrid technology sizing, cost, resilience and energy performance | Same load, tariffs, technologies, outage duration, constraints, finance and objective | Evidence provenance, network context, independent post-solve audit and equity/service-floor outcomes |
| SimBench | Public distribution-to-extra-high-voltage networks, profiles and time-series comparisons | Exact selected grid code, version, scenario, time zone, profiles, licensing and data conversion | Auditable source intake, cross-system evaluation, South African transfer testing and service-accountability layer |

### Grounded reference points

- PGLib-OPF v23.07 publishes PowerModels.jl / Ipopt baseline objectives. For the IEEE 14-bus case, its table reports DC objective $2,051.5/h and AC objective $2,178.1/h; for IEEE RTS 24-bus, $61,001/h and $63,352/h; and for IEEE 57-bus, $34,773/h and $37,589/h. These are reference objective values, not Sol-Atlas results. The table explicitly notes that solve-time figures exclude approximately 2–5 seconds of Julia JIT and use HSL MA27; a default linear solver is stated to be 2–6x slower. Do not use the published timings without those qualifications. Source: https://github.com/power-grid-lib/pglib-opf/blob/master/BASELINE.md
- PyPSA 1.4.0 was released 9 October 2026. Its release notes include behavior changes affecting rolling-horizon unit commitment, ramp limits, and security-constrained optimization. A benchmark must pin the exact release and cannot treat "PyPSA" as a timeless result. Source: https://docs.pypsa.org/latest/release-notes/
- pandapower 3.5.6 was released on 7 October 2026 and is the published stable release at this snapshot. Its documentation exposes power flow, contingency analysis, OPF, short-circuit and state estimation. The 4.0 documentation/changelog is the development line, not the current stable release; it describes angle-difference constraints in OPF and additional transformer conversion behavior. Sources: https://pypi.org/project/pandapower/ and https://pandapower.readthedocs.io/en/latest/ and https://pandapower.readthedocs.io/en/stable/about/changelog.html
- PowerIO 0.12.1 (MIT OR Apache-2.0) is a current Rust-first compiler for power-system data formats. Its published facade includes a typed balanced network, source retention, diagnostics and readers/writers for MATPOWER, PSS/E, PowerModels JSON, pandapower JSON, PyPSA CSV and other formats. It is an important direct interoperability comparator, so Sol-Atlas should integrate it as an independent parser rather than reimplement its format surface. Sources: https://github.com/eigenergy/powerio and https://crates.io/crates/powerio
- matpowercaseframes 2.1.1 is a current Python parser for MATPOWER cases (MIT license). Use it as a third parser only after confirming exact field/semantic coverage and matching the same source bytes. Source: https://pypi.org/project/matpowercaseframes/
- Grid2Op explicitly targets sequential power-grid decision-making and allows different power-flow backends; its documentation supports heuristic, optimization, MPC and learning-based agents. The GitHub release page lists 1.12.5 (12 June 2026) as its latest release at this snapshot. Sources: https://grid2op.readthedocs.io/en/latest/ and https://github.com/Grid2op/grid2op/releases
- RL2Grid is a complementary control benchmark built on RTE's Grid2Op framework. It standardizes tasks, state/action spaces and rewards, and includes operator-informed heuristics and safety constraints. It is a more relevant future comparator for sequential control than a static DC solver. Source: https://arxiv.org/abs/2503.23101 (29 March 2025).
- REopt API v3 is the stable API line and uses REopt.jl as its optimization engine. Its stated scope includes buildings, campuses, communities and microgrids. Sources: https://developer.nlr.gov/docs/energy-optimization/reopt/v3/ and https://developer.nlr.gov/docs/energy-optimization/reopt/
- SimBench provides 13 grid datasets spanning low to extra-high voltage; the University of Kassel project description states annual 15-minute time series for one year. The official download page specifies ODbL and Database Contents License terms for the datasets. Sources: https://simbench.de/en/download/datasets/ and https://www.uni-kassel.de/eecs/en/sections/sustainable-electrical-energy-systems/our-developments/data-and-models.html
- A power-system digital-twin review highlights the lack of consensus fidelity standards, the sensitivity of results to sensor precision and temporal alignment, and recommends "known-fidelity" frameworks that declare acceptable errors and integration quality per application. Source: https://ietresearch.onlinelibrary.wiley.com/doi/full/10.1049/dgt2.70015

### An additional interoperability baseline: PowerIO

PowerIO changes the competitive boundary. A broad, source-aware format compiler already exists; Sol-Atlas should not spend its scarce engineering effort rebuilding every import/export format.

The useful architecture is:
- **PowerIO** as one independent parser/normalizer and a source-preserving format adapter.
- **Sol-Atlas intake** as a stricter domain-admission layer: rights, version, bytes, semantic profile, explicit supported-field set, source claim ceiling and exact field-differential receipt.
- **Independent physics** as a distinct DC/AC solver and auditor chain.
- **Operational assurance** as the differentiator: freshness, uncertainty, service-floor outcomes, authorization boundaries, incident reconciliation and per-beneficiary evidence.

A first PowerIO differential adapter has been added to the MATPOWER evaluation branch. It parses the same pinned PGLib bytes with PowerIO 0.12.1 and compares bus, generator, branch, load-aggregate and cost-row fields against the Sol-Atlas projection. It must report discrepancies instead of normalizing them away. Its CI is an independently pinned comparator workflow; the result is pending until exact-head validation completes.

PowerIO's public performance note is based on an August 2026 candidate snapshot at version 0.10.0, not the current 0.12.1 release. Its reported case193k parser times and comparisons must not be quoted as a current 0.12.1 result until rerun on pinned hardware with the same PGLib case corpus and updated matpowercaseframes 2.1.1. Source: https://github.com/eigenergy/powerio/tree/main/evals/performance

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

### H. From reward-shaped safety to an independent runtime shield

A 2026 preprint on hierarchical grid control separates a high-level learned policy from a deterministic runtime safety shield. It evaluates nominal operation, forced line-outage stress and zero-shot transfer to an unseen grid. Its direction supports an architecture in which learning proposes abstract actions while an independent physical model and shield decide whether an action is admissible. This is a research comparator, not evidence that any particular shield is certified. Source: https://arxiv.org/abs/2604.14032 (15 April 2026).

A November 2025 review of safe reinforcement learning for power-system control distinguishes safety layers from integrated optimization and examines frequency regulation, voltage control and energy management. This reinforces the need to measure safety as its own outcome rather than fold it into reward alone. Source: https://doi.org/10.1016/j.rser.2025.116022.

For Sol-Atlas, benchmark the shield and policy separately:
- **Proposal quality:** useful actions found, solve time, objective/service value and coverage of feasible alternatives.
- **Shield correctness:** false-safe and false-reject rates against an independent frozen oracle, with zero false-safe as a promotion target on the declared finite panel.
- **Abstention quality:** unsupported/out-of-envelope states correctly rejected, without counting abstention as success.
- **Robustness:** unseen topology, N-1/N-k outages, forecast/sensor shift, stale or contradictory telemetry and common-cause failures.
- **Recovery:** time and service deficit after fallback or replan; a controller that never violates limits but causes avoidable prolonged outages is not a sufficient result.

Do not report "zero unsafe actions" without naming the independent oracle, finite scenario panel, physical model, tolerances, and untested conditions.

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


## 9. Three-oracle qualification ladder (new implementation)

The next qualification step is no longer to write more parser code. Sol-Atlas now has three intentionally separate evidence paths on the active integration branch:

1. **Sol-Atlas internal path:** strict MATPOWER v2 admission → source-bound typed projection → fixed-injection DC reference kernel → separate receipt auditor → complete single-branch outage sweep.
2. **Independent semantic parser oracle:** PowerIO 0.12.1 parses the exact original bytes independently. The Rust differential compares the supported bus, generator, branch and generator-cost fields, including loads/shunts re-aggregated to bus rows. The sole declared representation mapping in the comparator is MATPOWER zero-valued branch ratings ("unlimited") to PowerIO positive infinity; any other discrepancy must be surfaced and classified.
3. **Independent numerical solver oracle:** MATPOWER 8.1 rundcpf loads the original .m file and computes a DC solution independently of the Sol-Atlas projection and solver. The comparator binds both results to the same Git blob SHA-1, compares base MVA and reference-bus identity, normalizes bus angles to the reference bus, compares per-bus net injections, and compares both PF/PT branch-end flows and branch identities.

This splits three questions that must not be conflated:
- **Did we interpret the source consistently?** The PowerIO field differential addresses the declared field inventory.
- **Did our implementation solve the stated DC equations consistently?** The independent receipt auditor addresses the Sol-Atlas receipt.
- **Did an independent implementation obtain the same declared DC solution?** MATPOWER 8.1 rundcpf addresses the numerical reference comparison.

A zero-mismatch result from either external oracle is only scoped to the exact source files, parser/solver versions, declared fields, DC assumptions and tolerances. PowerIO is a parser/model interoperability oracle, not an independent physical measurement. MATPOWER's DC solve is a numerical reference, not AC validation or field telemetry.

### Frozen initial solver-differential tolerances

| Quantity | Initial tolerance |
|---|---:|
| Bus angle relative to reference | \(10^{-7}\) degrees |
| Bus net active injection | \(10^{-6}\) MW |
| Branch PF and PT | \(10^{-6}\) MW |
| System base MVA | \(10^{-10}\) MVA |

These tolerances are contract values for the first pinned IEEE 14/RTS 24/IEEE 57 panel, not claims about field-model accuracy. If any reference implementation produces a systematic difference, first investigate equation convention, phase-shift sign, tap interpretation, slack treatment, units, and output semantics. Do not loosen a threshold merely to make CI green.

The MATPOWER reference workflow pins release commit 1a828c7af590714499284e36ee9c81273388c594. Its TSV input carries the original source digest, MATPOWER version, base MVA and reference bus. Its JSON output preserves every numerical mismatch and tolerance.

### Next expansion after parity

Once all three cases pass both external differentials:
- extend the parser corpus with MATPOWER cases containing DC lines, multiple cost rows, switched shunts, generator capability columns, phase shifters, offline devices, multiple islands and unusual angle bounds;
- add controlled one-field-at-a-time source mutations and require the strict importer to reject malformed or unsupported forms, while both external parsers' behavior is logged separately;
- expand from three PGLib cases to a declared case matrix (including larger PGLib cases) and preserve per-case receipts;
- add a separate DC-OPF objective/bound comparison. Do not infer OPF quality from DC power-flow parity;
- only then move to AC and chronological / distribution / sequential-control domains.

The promotion rule is strict: **source-field parity, receipt consistency and reference numerical parity are independent gates**. Passing one must never imply the other two.


## 10. PTDF/LODF screening with mandatory escalation

MATPOWER 8.1 exposes makePTDF and makeLODF, and explicitly supports marking bridge-branch columns as NaN because removing a bridge causes islanding. PyPSA expresses the same branch outage distribution factor relationship and identifies the singular case where the outaged branch is the only connection between two regions. Sol-Atlas should compete by making this singularity an explicit decision state and validating every usable prediction against exact re-solves—not by pretending that LODF applies to every outage.

Reference implementations:
- MATPOWER 8.1 makePTDF: https://matpower.org/documentation/ref-manual/legacy/functions/makePTDF.html
- MATPOWER 8.1 makeLODF: https://matpower.org/documentation/ref-manual/legacy/functions/makeLODF.html
- PyPSA contingency analysis and BODF formula: https://docs.pypsa.org/latest/user-guide/optimization/contingencies/

For an in-service outage k, the linear post-contingency flow estimate is:

\[
f_l^{(k)} = f_l + LODF_{lk} f_k,\qquad
LODF_{lk} = \frac{PTDF_{lk}}{1-PTDF_{kk}}\quad(l\ne k),\qquad
LODF_{kk}=-1
\]

The denominator is a topology/sensitivity warning. If it is non-positive or within a frozen tolerance of zero, Sol-Atlas returns an exact-solve abstention. It never publishes a plausible-looking predicted-safe flow for an islanding bridge.

The implemented fast path:
1. solves the intact fixed-injection DC case;
2. constructs and factorizes one reduced susceptance matrix per energized island;
3. computes branch-transfer PTDF columns;
4. derives LODF outage-flow predictions and Rate A overload flags;
5. emits every in-service outage, with either a full predicted flow vector or an explicit exact-resolve abstention.

The separate validation path re-solves every outage with the direct DC reference kernel. It measures branch-flow residuals, false-safe branches/outages, false alarms, and abstentions that could have been solved exactly. The exact re-solves are intentionally not included in the fast-path runtime.

### Qualification targets

| Metric | Initial promotion target |
|---|---:|
| Coverage | 100% of in-service single-branch outages represented |
| False-safe branch count | 0 on frozen PGLib + adversarial panel |
| False-safe outage count | 0 on that panel |
| Resolved flow error | max absolute error <= 1e-6 MW against exact DC re-solve |
| Bridge / near-bridge handling | 100% explicitly abstained or sent to exact fallback |
| Exact-solve unknowns | reported separately; never recoded as a pass |
| False alarms | report count and overload magnitude; do not hide them |
| Unnecessary abstentions | report count/rate where exact solve succeeds |
| Runtime | warm/cold p50/p95/p99 for screen and exact sweep, same host and same network |
| Memory | peak resident memory for PTDF/LODF factors and outage predictions |

PGLib regression panels: IEEE 14 bus (20 outages), IEEE RTS 24 bus (38), and IEEE 57 bus (80). These are initial numerical verification sets, not evidence of large-grid performance or operational security.

### What would count as a real improvement over current practice?

- **Useful selective computation:** run the cheap sensitivity screen on all eligible outages, spend exact-solve compute on bridge/near-bridge cases and on cases whose predicted margin is close to a decision threshold.
- **Risk-weighted escalation:** exact-resolve any outage whose predicted flow uncertainty interval overlaps a thermal limit; do not use a point estimate as an unconditional safety decision.
- **Topology-aware refusal:** make bridges, weak cutsets, islands, unknown connectivity and unsupported model fields first-class states.
- **Evidence-quality propagation:** widen flow/limit uncertainty when source fidelity, telemetry freshness, parameter calibration or topology identity is weak. A mathematically small LODF residual cannot repair poor source evidence.
- **Service consequence ranking:** after technical screening, prioritize exact analyses by essential-service exposure and recovery cost rather than only largest MW overload.
- **Independent external cross-check:** compare both PTDF/LODF factors and post-outage flows against MATPOWER 8.1 makePTDF/makeLODF in addition to the direct in-repository solver.

Proposed performance goal after correctness is established: at least 3x warm p50 improvement over exhaustive direct re-solves on a declared 100+ bus benchmark panel, with zero false-safe outcomes and no more than 5% unnecessary abstentions. This is a target, not a measured result; small cases may not benefit because matrix factorization and report construction have fixed overhead.

### Non-claims

The current LODF implementation is fixed-injection linear DC screening. It does not redispatch generators, allocate new slacks after islanding, optimize corrective actions, model generator outages, validate AC voltage/reactive constraints, or certify N-1 operational security. Even a zero-error PGLib result only validates the declared equations and frozen cases. It is not field calibration or permission to operate a grid.
