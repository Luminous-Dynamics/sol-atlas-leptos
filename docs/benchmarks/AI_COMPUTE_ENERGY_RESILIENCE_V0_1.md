# AI Compute–Energy Resilience Benchmark V0.1

**Status:** proposed contract and synthetic fixture; no performance or facility improvement has been measured.  
**Work item:** AI-COMP-ENERGY-001.  
**Parent context:** [spatial-economic resilience PR #116](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/116), [intertemporal allocation issue #153](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/153), and its exact-oracle implementation candidate [PR #187](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/187).  
**Claim ceiling:** deterministic scheduling and resource accounting under declared synthetic constraints only; no grid-load-flow, thermal-CFD, real-world facility, causal impact, or national-ranking claim.

## 1. Paradigm shift

A data center should not be evaluated only as a building with servers, or only by its Power Usage Effectiveness (PUE). The relevant system is:

**workload and quality target → compute hardware → IT electricity → facility power/cooling/water → grid and local resources → economic and public outcomes.**

The unit of value is accepted, quality-conformant work delivered at the required place and time—not GPUs installed, tokens generated without a quality constraint, or megawatts announced in a project pipeline.

V0.1 defines a compute-specific contract on top of Sol-Atlas's existing service-capacity and intertemporal resource work. It keeps separate:

- requested, scheduled, completed, and quality-accepted work;
- protected-service floors and deferrable demand;
- IT energy and total facility energy;
- nameplate compute capacity and capacity deliverable under each interval's grid, thermal and water limits;
- measured observations, forecasts, commitments, model outputs and unknowns;
- modeled potential heat and heat actually accepted by a compatible customer;
- private operating cost and external or public resource impacts.

The central optimization output is a **Pareto set** and a transparent constraint ledger, not a hidden weighted “green AI” score. Hard service floors and safety/resource limits are constraints, not costs that can be traded away for a better scalar score.

## 2. Boundaries and definitions

The fixture's interval-level **facility_energy_per_it_energy** is a synthetic conversion factor for the interval model. It is **not** a standardized PUE measurement. PUE is calculated from total facility energy divided by IT equipment energy over a defined measurement period; it should be reported separately from this model coefficient.

For each interval, a first-order upper bound on IT power is:

min(site IT rating, thermal IT limit, grid import limit / facility-energy-per-IT-energy)

Water may impose an additional constraint using the declared per-IT-energy water coefficient. This is a deliberately bounded scheduling model: it does not simulate electrical voltage/current, cooling-fluid dynamics, heat-exchanger behavior, equipment transient response or a real utility network. Real deployments need their qualified domain solvers and measured power/thermal/water telemetry.

Unknown inputs must remain unknown. Missing grid headroom is not zero demand or unlimited supply; missing marginal emissions is not zero-carbon electricity; absent heat-reuse demand is not a recovered-heat benefit.

The CLI deserializes the versioned envelope with strict root and nested records; unrecognized keys fail parsing instead of disappearing from the model. A non-null `storage` profile is explicitly `Unsupported` until horizon-wide charge/discharge/reserve accounting is implemented. Deferrable workloads must be interruptible, and multiple deferrable workloads must share a declared quality-profile identity; otherwise no schedule is emitted. The thermal cap is an explicit input, not inferred from ambient temperature or free-form failure tags.

## 3. Benchmark layers and industry baselines

This benchmark complements, rather than replaces, established benchmarks.

| Layer | Reference | What it covers | What remains outside it |
|---|---|---|---|
| AI task performance and quality | [MLPerf Inference](https://mlcommons.org/benchmarks/inference-datacenter/) | Defined workloads, datasets/quality targets, throughput or latency scenarios | Grid location, time shifting, water, heat customers, national dependencies |
| Online generative-AI endpoint frontier | [MLPerf Endpoints](https://docs.mlcommons.org/endpoints-submission-docs/understand/what-is-mlperf-endpoints/) | Throughput versus per-user interactivity over concurrency, with quality gates | Its `system_tps_per_kw` uses provisioned-power estimates, not an interval wall-power energy meter; pair it with measured energy per accepted work |
| System-level AI power | [MLCommons Power](https://mlcommons.org/working-groups/benchmarks/power/) | Wall/system power and energy measurement for applicable MLPerf runs | Multi-site resilience and public-resource opportunity costs |
| Server energy efficiency | [SPECpower_ssj2008](https://spec.org/power_ssj2008/) | Standardized server-side Java workload and power/performance behavior | GPU AI workload equivalence; facility/grid/site system optimization |
| Open facility engineering | [OCP Data Center Facility](https://www.opencompute.org/projects/data-center-facility/) and [Rack & Power](https://www.opencompute.org/projects/rack-and-power/) | Open facility, rack, power, cooling and serviceability approaches | Whether scheduled work creates more useful national or public value |
| Facility/resource metrics | [The Green Grid PUE](https://www.thegreengrid.org/node/372) and [DCRE v1](https://www.thegreengrid.org/resources/library-and-tools/wp93-data-center-resource-effectiveness-dcre-metric) | Facility overhead; broader energy/water/resource-context metrics | Workload quality and end-to-end resilience/economic trade-offs |
| National/regional capacity | Sol-Atlas network, evidence, service-floor and intervention contracts | Spatial dependencies, time-bound capacity, resource competition, uncertainty and recovery | Physical facts not present in attributed evidence or qualified models |

The differentiator is system-level comparison under matched workload quality and explicit shared-resource constraints. It is not fair to compare an MLPerf score to an unrelated normalized work unit. When hardware is available, task-level energy must be measured with appropriate wall-power instrumentation and a fixed quality target. The synthetic work units in the fixture below are only for correctness testing.

### Research context

The IEA's updated analysis reports data-centre electricity consumption of approximately 485 TWh in 2025 and projects around 950 TWh in 2030 in its central outlook; it also notes that grid/equipment bottlenecks constrain near-term deployment. These are scenario estimates, not certain outcomes: [IEA, Key Questions on Energy and AI](https://www.iea.org/reports/key-questions-on-energy-and-ai/executive-summary).

The U.S. DOE/Lawrence Berkeley National Laboratory's 2024 report provides estimates and a scenario range for U.S. data-centre electricity use: [2024 United States Data Center Energy Usage Report](https://bies.lbl.gov/publications/2024-lbnl-data-center-energy-usage-report).

OCP explicitly frames rack-power engineering as a grid-to-gates problem, supporting evaluation across interdependent power and IT interfaces: [OCP Rack & Power](https://www.opencompute.org/projects/rack-and-power/).

The current MLPerf Endpoints v1.0 guidance makes an important boundary explicit: throughput/interactivity is a Pareto frontier with quality gates, while its normalized `system_tps_per_kw` is based on a provisioned-power model derived from rated component power plus overhead, not the energy actually read by a meter during a run. We should preserve the official endpoint frontier for serving quality, then add repeated wall-power measurements and quality-accepted work per kWh as a separate measured axis; do not relabel a provisioned-power estimate as measured efficiency. [MLPerf Endpoints measurement definitions](https://docs.mlcommons.org/endpoints-submission-docs/understand/what-is-mlperf-endpoints/)

A complementary grid-facing proposal, [*A net-grid-benefit test for interconnecting AI data centres* (npj Environmental Social Sciences, 4 June 2026)](https://www.nature.com/articles/s44432-026-00013-5), argues for evaluating interconnection contributions against a passive-load counterfactual, including auditable hour-by-hour clean-power matching with deliverability screens, verified fast frequency-response/regulation telemetry, and useful waste-heat recovery where feasible. We should benchmark these as distinct, verifiable services—not as automatic credits or a universal interconnection rule. A claimed grid-service benefit needs event telemetry, response latency, duration, rebound and a clear counterfactual.

A practical capacity-state example: on 19 October 2026, Reuters reported Microsoft withdrew from a proposed 50 MW UK data-centre deal after grid-power connection timing shifted into the 2030s. This is evidence about one project, not a universal estimate. The model should therefore distinguish **announced**, **contracted**, **commissioned**, and **firm-deliverable** capacity, and only the last should count as assured supply for a scenario's declared interval and conditions. [Reuters, 19 October 2026](https://www.reuters.com/business/media-telecom/microsoft-pulls-out-uk-data-center-deal-with-nscale-telegraph-reports-2026-10-18/)

### Grid contribution: prove additionality, not intent

The next benchmark layer should compare a candidate's actions against a passive-load counterfactual with the **same accepted workload, quality, deadlines and facility boundary**. Report grid effects separately from private site cost:

- **Locational relief:** MW and MWh reduced at the constrained node, affected hours, and the qualified grid model used. Do not assume that shifting demand automatically avoids generation or transmission investment.
- **Response service:** event-triggered response latency, MW/min ramp, sustained MWh, duration, telemetry completeness, dispatch success rate, recovery time and rebound peaks.
- **Clean-power deliverability:** hourly matching, location/connection deliverability and separately reported average versus marginal emissions factors. Annual contract matching alone does not prove hourly physical delivery.
- **Resource burden:** water by source and local stress, storage degradation, heat actually accepted at usable temperature, and pumping/heat-pump energy. Optional heat revenue cannot count as cooling safety.
- **No double-counting:** the same battery or interruptible load cannot be credited simultaneously for incompatible grid services; declare reserve commitments and availability windows.
- **Distribution and robustness:** show whether shifting moves congestion or water stress to another region and how benefits change under outages, hot weather and uncertain grid factors.

Accept a grid-service claim only when the baseline, boundary, telemetry, uncertainty and external solver/model version are explicit. A schedule can be optimal for the operator's local cost objective and still fail a system-level additionality test. That is why the scorecard retains the full Pareto set instead of manufacturing a single sustainability score.

These sources motivate the problem and provide comparison context; they do not validate this synthetic model.


### Electrical architecture as an explicit comparison, not a presumption

The Open Compute Project's 800 VDC work is a current industry reference for high-density AI racks and more integrated power conversion/storage interfaces: [OCP, 800 VDC transition](https://www.opencompute.org/news/powering-the-next-era-of-ai-data-center-infrastructure-through-800vdc/). The benchmark should support AC, hybrid and DC-native candidate topologies through measured or source-qualified parameters rather than declaring one topology universally better.

For each comparable design, measure or model separately:
- source-to-rack and rack-to-accelerator conversion losses;
- copper/conductor mass and distribution losses;
- protection, fault isolation, serviceability and restoration time;
- UPS/storage efficiency, availability and dispatch limits;
- rack-density and cooling effects;
- installation capital, commissioning time, retrofit disruption and lifecycle cost.

The first comparison must use the same delivered workload, quality target, facility boundary and reliability/service constraints. Claims about newer voltage architectures should be source-attributed design hypotheses until supported by measured equipment or independently qualified system models.

## 4. Scenario contract

The machine-readable shape is defined by **schemas/spatial-economic-ai-compute-energy-flexibility-v0.1.schema.json**. The example **schemas/examples/ai-compute-energy-flexibility-four-interval.synthetic.json** is a tiny, hand-checkable positive control.

A scenario declares:

1. **Time grid:** ordered, non-overlapping intervals and duration.
2. **Site:** IT rating and stable site identity.
3. **Grid:** maximum facility import per interval, with optional attributed price and marginal-emissions factors.
4. **Thermal envelope:** interval-level upper bound on IT power after ambient/thermal derating.
5. **Water envelope:** maximum operational cooling water and water per IT-energy coefficient for each interval.
6. **Workload:** required work units, energy per unit, release/deadline indices, service-quality-profile identity, and minimum accepted work per interval.
7. **Evidence and claim ceiling:** source references for real data; the synthetic fixture explicitly says it is not an observation or national forecast.

The semantic validator must additionally enforce cross-field rules the JSON Schema cannot fully express: unique IDs, array lengths aligned to the interval count, valid release/deadline bounds, service floors not exceeding required work, compatible units, finite values, and valid power/energy conversions. Never infer an absent interval, unit, resource or workload profile.


## 4A. Executable reference evaluator

The Rust core uses no external optimization solver for its bounded reference evaluator and native JSON CLI. Run individual policies or compare the reference-policy frontier against the **same exact scenario bytes**:

```sh
cargo run -p sol-atlas-core --bin ai_compute_energy_assess -- schemas/examples/ai-compute-energy-flexibility-four-interval.synthetic.json earliest-ready
cargo run -p sol-atlas-core --bin ai_compute_energy_assess -- schemas/examples/ai-compute-energy-flexibility-four-interval.synthetic.json lowest-cost
cargo run -p sol-atlas-core --bin ai_compute_energy_assess -- schemas/examples/ai-compute-energy-flexibility-four-interval.synthetic.json lowest-carbon
cargo run -p sol-atlas-core --bin ai_compute_energy_assess -- schemas/examples/ai-compute-energy-flexibility-four-interval.synthetic.json pareto
```

The CLI emits a machine-readable JSON envelope with top-level `receipt` and `assessment` fields. Its versioned output contract is **`schemas/ai-compute-energy-run-report-v1.schema.json`**, so downstream verifiers can validate the report shape without importing the Rust implementation. The policies are `earliest-ready`, `lowest-cost` (sort by attributed electricity price × facility-energy factor), and `lowest-carbon` (sort by attributed carbon intensity × facility-energy factor). The `pareto` mode returns all three candidate assessments plus the non-dominated policies across modelled electricity cost and attributed grid carbon; this is a frontier over the three reference policies, not a proof of the global feasible frontier. If any candidate or objective attribution is invalid, unsupported or indeterminate, the comparison preserves candidate diagnostics and withholds the frontier. Missing price/carbon factors for any interval with eligible capacity make the corresponding objective indeterminate; unknown values never silently become zero. Finite negative wholesale electricity prices are allowed because negative-price intervals can occur in real markets; carbon intensity and physical resource bounds remain nonnegative.

Exit codes are explicit: 0 for a completed feasible/capacity-limited assessment; 2 for protected-floor infeasibility; 3 for indeterminate required resource/objective inputs; 4 for unsupported request shapes; 5 for invalid inputs; 64 for CLI usage errors; 65 for JSON parse errors; 66 for file-read errors; 70 for serialization errors.

### Exact-input run receipt

Every CLI output wraps the result in a `receipt` plus `assessment`. The receipt contains the SHA-256 digest and byte length of the **exact UTF-8 scenario bytes parsed**, plus `assessment_sha256`, the SHA-256 digest of the compact `serde_json` serialization of the assessment value (before pretty-printing the outer report). It also records the requested mode, scenario schema version and fixture identity, `synthetic` flag, and claim ceiling. The same semantic input JSON with different whitespace intentionally yields a different input digest; changing the assessed result changes the assessment digest. The receipt's verification-boundary text explicitly says these digests bind input/result identity; they do not validate whether the scenario's underlying factual claims are true. The public Rust helper `verify_scenario_receipt` re-parses the exact raw input with the strict `Request` contract and recomputes the receipt; it returns false for altered input bytes, mode, metadata or assessment contents. A successful digest verification establishes identity/reproducibility only—not empirical validity, model correctness or authorship.

The digests are intended for reproducible replay, Mycelix provenance linkage and reviewer receipts. For independent reconstruction, record the evaluator commit SHA and environment alongside the receipt; the report digests alone do not attest to software identity, data licensing or physical accuracy.

The evaluator's unit tests parse the committed synthetic JSON via `include_str!`, rather than maintaining a second copied fixture. This prevents the test fixture and code-side request from silently drifting apart. The CLI is useful for deterministic local inspection and integration with other stack components; it does not elevate synthetic data to physical evidence.

## 5. Synthetic oracle case

The four-interval fixture uses abstract work units and invented limits. It must never be labelled as a real AI model, U.S. grid, Johannesburg facility, or industry result.

- Every interval lasts one hour.
- The site IT rating is 80 kW.
- Grid import limits are 20, 10, 10 and 20 kW.
- The synthetic facility-energy multipliers are 1.15, 1.35, 1.40 and 1.15.
- Thermal IT caps are 80, 60, 50 and 80 kW.
- Cooling-water limits are 100, 100, 2 and 100 litres; water coefficients are 0.1, 0.4, 0.5 and 0.1 L/kWh of IT energy.
- A protected workload requires six accepted units in every interval. A deferrable batch workload requests 80 units by the end of the horizon. Each unit uses 0.5 kWh of IT energy.
- Protected floors are hard constraints. Every policy must satisfy them first; deferrable work uses only residual capacity.

For this finite fixture, the hand-calculated maximum deferrable delivery is 66 units: interval ceilings are 28, 8, 2 and 28 units after the protected floor and power/water constraints. The expected results are therefore 24/24 protected work units and 66/80 deferrable work units, with all resource limits held. Freeze these values only as the fixture's oracle, not as an optimizer benchmark or field-performance claim.

The test is useful because a nominal site rating of 80 kW is not the deliverable capacity in every interval. The grid interface, thermal limit and water constraint can each change the feasible scheduling envelope. A scalar average or nameplate value would hide that fact.

### Synthetic scheduling counterfactual

With deferrable demand set to 40 units in the same four-interval fixture, the three policies produce these synthetic schedules:

| Policy | Deferrable units by interval | Modelled electricity cost | Modelled grid carbon |
|---|---|---:|---:|
| Earliest-ready | `[28, 8, 2, 2]` | $3.700 | 24,227.5 gCO₂ |
| Lowest-cost | `[12, 0, 0, 28]` | $1.978 | 12,305 gCO₂ |
| Lowest-carbon | `[2, 8, 2, 28]` | $3.401 | 10,025 gCO₂ |

These are derived from the fixture's explicitly synthetic factors—not facility measurements, observed grid emissions, or evidence that a real workload can move without changing quality and latency. The counterfactual demonstrates that price and carbon minimization are distinct objectives: the evaluator exposes the trade-off instead of collapsing it into an opaque score. All three schedules preserve the six-unit protected floor in every interval.

A May 2026 paper using Alibaba workload traces reports that about 20% of data-center power came from latency-tolerant tasks suitable for shifting and that its LAD-Flex strategy could shed up to 22% load during flexibility windows. This is a workload- and scenario-specific result, not a universal fleet property: [Caprara et al., *Data center workload flexibility for power system demand response: Evidence from Alibaba traces*](https://www.sciencedirect.com/science/article/pii/S0142061526003820). It supports testing workload flexibility, but does not prove this synthetic scheduler beats the published approach; a fair comparison needs comparable trace, flexibility-window, and quality/service assumptions.

A June 2026 ACM Sustainability Week study, [*To Defer or To Shift? The Role of AI Data Center Flexibility on Grid Interconnection*](https://doi.org/10.1145/3765611.3815593), reports that flexibility's grid-planning effects are location- and scenario-dependent; more flexible load does not automatically reduce required generation capacity. The benchmark should therefore report locational congestion, import limits, connection readiness, and capacity-expansion effects separately instead of treating every shifted kWh as a system-level benefit.

A timely real-world failure mode is interconnection rather than server efficiency: [Reuters, 19 October 2026](https://www.reuters.com/business/media-telecom/microsoft-pulls-out-uk-data-center-deal-with-nscale-telegraph-reports-2026-10-18/) reported that Microsoft withdrew from a planned 50 MW UK data-centre deal after expected grid power availability moved into the 2030s. This is a specific reported project, not a general estimate; it motivates keeping **announced, contracted, commissioned, and deliverable** capacity as distinct states in the spatial-economic model.

## 6. Baseline ladder

Use identical scenario bytes, task quality profile, work demand, deadlines, service floors and resource bounds for every candidate.

- **B0 — fixed/earliest-ready:** deterministic earliest-ready scheduling within the residual capacity left after protected service floors and all hard constraints.
- **B1 — power-cap-aware greedy:** respects interval grid, site and thermal bounds before admitting deferrable work.
- **B2 — resource-aware scheduling:** additionally respects water limits and uses attributed time-dependent prices/emissions where present.
- **B3 — joint optimization:** co-optimizes deferrable workloads with shared power, water and optional storage over the horizon. Optional heat reuse is a separate resource stream, never an assumed credit.
- **Exact oracle:** exhaustive search on tiny integer fixtures; a candidate must match the feasible optimum and objective for the exact supported domain.
- **Independent scalable baseline:** documented LP/MILP/CP-SAT solver or other independently implemented optimization method on larger cases. Pin version, solver parameters, stopping tolerance and optimality gap.

A heuristic is not “better than SOTA” because it beats a weak baseline. The comparison must include the strongest applicable baseline that can represent the same constraints, and disclose any constraints it cannot model. The exact oracle is for correctness on small instances, not a claim of superior scalability over industrial solvers.

### Small-instance optimizer oracle (added after the initial v0.1 slice)

The Rust test suite checks both the lowest-cost and lowest-carbon greedy allocators against an independent exhaustive enumerator over every feasible integer allocation. It covers six tariff profiles and six carbon-intensity profiles at demand sizes 1–20 (240 profile-demand combinations), including negative and near-tied tariffs for cost minimization. A second test creates 64 deterministic pseudo-random resource-capacity scenarios by varying the binding water limit; each scenario is checked against both objective oracles, adding 128 comparisons. Across these bounded tests, there are 368 objective-vs-exhaustive comparisons. Assertions cover exact work conservation, interval capacity bounds, protected-service floors, repeat determinism, and equality with the globally minimum objective. Regression checks also verify that finite raw values whose derived cost or carbon objective overflows are rejected before sorting.

This oracle validates only the supported four-interval, homogeneous-workload domain used by the test. It does not establish scalability, prove the general implementation formally, or compare against a production LP/MILP/CP-SAT solver. The next benchmark step is randomized/property-based small-instance differential testing, followed by a pinned independent solver for larger cases.

## 7. Scorecard: do not collapse the Pareto frontier

Always report the following independently:

| Dimension | Metric | Interpretation and guardrail |
|---|---|---|
| Usefulness | accepted work / requested work, by workload | Count only output meeting the declared quality profile |
| Service | per-class completion, deadline misses, floor shortfall and p95/p99 latency where applicable | Zero protected-floor violations are a hard gate |
| Compute energy | measured or modelled IT kWh / accepted work | Keep measured and modelled results distinct |
| Facility overhead | facility kWh and IT kWh; observed period PUE where measured | Do not call interval multipliers PUE |
| Grid | peak import kW, curtailed work, load profile, interconnection-limit binding intervals | Power capacity is time- and location-specific |
| Carbon | consumption-weighted emissions using attributed time/location factors | Unknown factors remain unknown; do not substitute annual averages silently |
| Water | operational water by source/type, WUE, and water consumption per accepted work unit | Add site-specific water-stress context such as The Green Grid's Water Usage Impact (WUI); do not equate low water consumption with zero impact |
| Heat reuse | heat accepted at usable temperature minus associated pumping/heat-pump energy; customer uptime | No credit for heat merely rejected or technically recoverable |
| Economics | energy cost, capital/maintenance assumptions, cost per quality-accepted work unit | Keep assumptions and financing horizon visible |
| Resilience | accepted work during named outage/heat/grid events; service degradation and recovery | Declared scenarios are not probabilities; report coverage and uncertainty |
| Distribution | impacts on essential services, other industries and communities where evidence supports them | Do not let an aggregate gain conceal protected-service failure |
| Confidence | observed/modelled/forecast/committed/unknown coverage and provenance | No hidden imputation or false precision |

For national/regional presentation, add a dependency ledger: commissioned versus announced capacity; grid-connection status; deliverable compute under peak conditions; equipment/import concentration; local energy and water opportunity costs; workforce and service access; and model/data vintage. Aggregate only compatible measures with aligned geography/time and publish the denominator and missing-data coverage. Do not emit a single geopolitical rank from this one benchmark.

## 8. Proposed acceptance gates and improvement targets

All targets below are proposed and unmeasured until an exact revision is run.

| Gate | Target | Fail-closed interpretation |
|---|---:|---|
| Small known-answer oracle | 100% exact agreement | Any difference in protected floors, work counts or resource bounds fails |
| Resource conservation/capacity | 100% of interval rows valid | No negative stock, power overrun, water overrun or double allocation |
| Unknown/invalid evidence | 0 unsafe positive conclusions | Missing interval/price/emissions/source/units cannot silently become zero or unlimited |
| Protected services | 0 accepted floor violations | Infeasible floors return explicit infeasible/indeterminate status, never PASS |
| Determinism | 100/100 identical canonical outputs | Input-order permutations must preserve the canonical result |
| Scheduling improvement hypothesis | In at least 3 of 4 resource-constrained synthetic cases, ≥10% more deadline-satisfying deferrable work than B0, or ≥15% lower peak grid import at equal accepted work | Improvement is measured only under identical quality, service floors and resource constraints; if not attained, report the result rather than relax constraints |
| Pareto integrity | All non-dominated candidates retained | Never conceal resource trade-offs behind one weighted score |
| Small/medium/large timing | Report p50/p95/max and memory before setting release gates | Establish a named reference runner and baseline first; proposed latency thresholds are not currently authoritative |
| Real-hardware promotion | Matched workload quality, wall-power measurement and repeated runs | Simulator outcomes cannot qualify physical facility performance |

The 10%/15% thresholds are research hypotheses for selecting useful experiments, not expected or established results. A candidate that increases work by violating a hard service floor, relying on missing water data, or using an incomparable quality target is a failure.

## 9. Adversarial cases to add next

1. **Heat wave + grid cap:** simultaneous thermal derating and reduced import headroom; all protected floors and limits remain explicit.
2. **Water constraint:** cooling-water cap binds before IT nameplate; resource scarcity is visible rather than averaged away.
3. **Storage look-ahead:** a myopic scheduler empties storage early and harms a later protected interval; compare with the temporal allocator from issue #153 / PR #187 once integrated.
4. **Heat customer outage:** recoverable heat demand falls to zero; compute cooling remains safe on the independent rejection path.
5. **Interconnection bottleneck:** high announced IT capacity but limited commissioned power delivery; report deliverable capacity, not project announcements.
6. **Common-cause event:** a shared substation or transmission failure affects multiple sites; do not treat them as independent capacity.
7. **Unknown/mismatched factors:** an emissions factor is expired, a unit differs, or a site/time interval is missing; disposition is indeterminate.
8. **Fairness pressure:** deferrable commercial demand competes with declared essential services; no scalar objective can override protected floors.

## 10. Cross-stack integration

- **Sol-Atlas:** owns scenario geography, evidence provenance, service/capacity assessment, intervention comparison and later national/regional aggregation.
- **Symtropy:** supplies validated scenario simulation and temporal resource/energy experiments; it must expose model versions, assumptions and limits.
- **Symthaea:** may search for candidate schedules and hypotheses, but every candidate must pass the same deterministic constraints and independent verifier.
- **Mycelix:** can retain source lineage, review/attestation history and challengeable evidence records. A digest authenticates identity only when verified; it does not prove the underlying claim true.

The campus-level design must follow Symthaea's [symbiotic infrastructure profile](https://github.com/Luminous-Dynamics/symthaea/issues/6146): useful thermal/resource coupling is optional and bounded; compute cooling and primary physical safety cannot depend on an unqualified heat customer or AI service. Symtropy's [energy-delivery issue](https://github.com/Luminous-Dynamics/symtropy/issues/2085) is a downstream system-model consumer, not a substitute for electrical-grid or nuclear licensing models.

## 11. External demand-response reference and metric bridge

A useful comparison is not only a grid-aware scheduler objective: the output must be evaluated using a recognized demand-response tracking rule and reported workload-quality effects. The 2026 ACM E-Energy paper by Acun et al., [*Investigating Power Consumption Flexibility of AI Data Centers for Demand Response Participation*](https://doi.org/10.1145/3744255.3798112), reports 18%–55% flexibility relative to average power for its studied regulation-service-reserve (RSR) scenarios and up to 20% power reduction in its emergency-demand-response (EDR) scenarios. Those are study-specific results from its AI workload traces and power–performance profiles, **not** a transferable target or a result from Sol-Atlas.

The authors' open-source [FlexDC-Sim](https://github.com/peaclab/flexdc-sim) reference repository is pinned for comparison at commit [`ea50bd582b5d1b55fcb09ea46899fa9d325cce3b`](https://github.com/peaclab/flexdc-sim/tree/ea50bd582b5d1b55fcb09ea46899fa9d325cce3b), BSD-3-Clause. Its experiment configuration specifies time step, duration, utilization, server count/idle power and the grid-provider signal; cluster/job profiles specify node idle power, job power–performance ranges and QoS. It exports a power trace including target, realized power and tracking error, and a job trace with arrivals/start/end and QoS-related fields. This is a more detailed node/job simulation than Sol-Atlas v0.1's aggregate per-interval work model. We therefore compare only an explicitly matched subdomain and do not treat either simulator's output as metered facility truth.

The new dependency-free Rust module `sol-atlas-core/src/demand_response_metrics.rs` gives Sol-Atlas a shared measurement layer. The `demand_response_assess` CLI reads strict traces and emits receipt-bound JSON. It supports two intentionally distinct RSR paths: (1) recompute error from an explicitly aligned target/measured power pair using the exact declared reserve, and (2) score a source-normalized signed `trackingError` series without recomputing source timing or denominator. The second path is for parity against a reference simulator's exported error series; it does not validate that source's time alignment or power measurement. EDR accounting separately reports baseline/event energy, gross curtailment, rebound and net change. Input/output contracts are in `schemas/demand-response-trace-v1.schema.json` and `schemas/demand-response-assessment-report-v1.schema.json`; a synthetic known-answer fixture is in `schemas/examples/demand-response-trace-v1.synthetic.json`. The receipt hashes exact input bytes and compact metric JSON. CI tests arithmetic, fail-closed behavior, receipt identity, strict JSON parsing, and schema/mutation behavior. These are metric-accounting tests, not measurement or causal-effect tests.


- **RSR tracking:** for aligned power pairs, normalized absolute tracking error is `abs(measured_kW - target_kW) / reserve_kW`. For source-normalized errors such as FlexDC-Sim's signed `trackingError` column, the CLI scores the supplied series' absolute values without silently rebuilding source alignment or denominator. It reports two separate decisions: `p90_threshold_satisfied` matches the pinned implementation's interpolated p90 check (`p90 <= 0.3`), while `exceedance_rate_requirement_satisfied` is a supplementary strict criterion (`fraction(error > 0.3) < 10%`). They are not equivalent: one error of 0.8 among ten samples yields interpolated p90=0.08, so the p90 threshold passes while exactly 10% exceed the threshold and the strict rate criterion fails. The cross-validation run must report both rather than silently choosing whichever looks better.
- **EDR curtailment:** compare an aligned declared baseline against event power and report baseline/event kWh, net energy reduction, gross curtailment energy, rebound energy, count of rebound intervals, and peak event power above baseline. A negative net reduction remains visible when rebound exceeds gross curtailment.
- **Evidence gates:** missing samples, mismatched vector lengths, non-finite values, negative power, zero/invalid reserve and derived-value overflow are rejected. The module computes accounting metrics only; it cannot prove that an input baseline is a valid causal counterfactual, and no live trace has been run yet.

One denominator difference must be reconciled before bit-for-bit FlexDC-Sim parity: its runtime policy adds `0.0001` to reserve in a denominator, while this helper uses the declared positive reserve exactly and rejects zero. The helper intentionally does not use epsilon to turn an invalid reserve into a valid experiment.

### Cross-validation gates

1. Pin the FlexDC-Sim commit, full input/config digests, dependencies and random seeds; keep their software external and preserve license/citation attribution.
2. Use one explicitly aligned signal, time grid, power unit, server/job profile and workload seed in both systems. Map only inputs whose semantics are verified.
3. Compare the realized power trace and job/QoS trace against Sol-Atlas metrics, with the exact-oracle scheduler retained for small aggregate cases. A mismatch may reflect model boundaries (node-level power capping vs aggregate interruptible work) rather than an optimizer defect.
4. Report tracking-error p90, strict exceedance fraction, response delay, peak target deviation, QoS/deadline loss, event energy, net reduction, rebound and run-to-run variation.
5. Do not generalize FlexDC-Sim's RSR/EDR ranges to different workloads, sites, control intervals, hardware, carbon factors, water limits or interconnection conditions without matched evidence.

The active work item is [AI-COMP-ENERGY-003 / issue #256](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/256). The current reference metric module is dependency-free Rust; FlexDC-Sim itself remains an external reference and no third-party code/runtime dependency has been added to production Rust.

## 12. Reference sources

- [IEA — Key Questions on Energy and AI](https://www.iea.org/reports/key-questions-on-energy-and-ai/executive-summary).
- [DOE/LBNL — 2024 United States Data Center Energy Usage Report](https://bies.lbl.gov/publications/2024-lbnl-data-center-energy-usage-report).
- [MLCommons — MLPerf Inference: Datacenter](https://mlcommons.org/benchmarks/inference-datacenter/).
- [MLCommons — MLPerf Endpoints](https://docs.mlcommons.org/endpoints-submission-docs/understand/what-is-mlperf-endpoints/): multi-point throughput/interactivity Pareto frontier with model-quality gates.
- [MLCommons — Power working group](https://mlcommons.org/working-groups/benchmarks/power/).
- [SPECpower_ssj2008](https://spec.org/power_ssj2008/).
- [Open Compute Project — Data Center Facility](https://www.opencompute.org/projects/data-center-facility/).
- [Open Compute Project — Rack & Power](https://www.opencompute.org/projects/rack-and-power/).
- [The Green Grid — PUE](https://www.thegreengrid.org/node/372).
- [The Green Grid — Data Center Resource Effectiveness v1](https://www.thegreengrid.org/resources/library-and-tools/wp93-data-center-resource-effectiveness-dcre-metric).
- [FlexDC-Sim — PEACLab demand-response simulator, pinned source revision](https://github.com/peaclab/flexdc-sim/tree/ea50bd582b5d1b55fcb09ea46899fa9d325cce3b) (BSD-3-Clause).
- [Acun et al. (2026), *Investigating Power Consumption Flexibility of AI Data Centers for Demand Response Participation*, ACM E-Energy](https://doi.org/10.1145/3744255.3798112): RSR tracking formula and studied RSR/EDR flexibility outcomes.

**Current result:** a bounded capacity evaluator, greedy reference scheduler, cost/carbon Pareto comparison, exact-input SHA-256 run receipt, exhaustive small-instance test oracle, synthetic fixture, and dependency-free RSR/EDR metric helpers with fail-closed input and overflow checks. No FlexDC-Sim run, matched trace, hardware experiment, measured facility improvement, production joint optimizer, or SOTA result is claimed.
