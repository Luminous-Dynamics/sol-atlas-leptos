# Spatial-Economic Intelligence benchmarks

**Status:** benchmark protocol and proposed targets; no performance result has yet been measured or claimed.  
**Contract version:** v0.1  
**Scope:** evidence validation, qualitative dependency groups, intervention comparison, common-cause overlap, uncertainty-bounded service-function adequacy, network max-flow/min-cut, network intervention comparisons, Pareto-based network portfolio comparisons, service-recovery trajectories, exhaustive bounded N-k failure sweeps, and distributional service-floor assessment in `sol-atlas-core`. The timing harness measures nine computational surfaces; its targets remain unmeasured until executed on a named runner.

## Principles

1. **Correctness gates performance.** A faster result is not acceptable if a disputed edge contributes to a conclusion, a missing condition is treated as available, an inactive relation is counted, or an unknown baseline is mislabeled as recovery.
2. **Separate model output from reality.** Scenario deltas measure changes in modelled qualitative state, not whether an intervention occurred or caused a real-world outcome.
3. **Reproducible runs.** Pin the commit, Rust compiler, compilation profile, OS/architecture, CPU/RAM, scenario fixture, group/edge/evidence counts, sample count and warm-up count. Emit raw benchmark output and keep the synthetic fixture deterministic.
4. **Report distributions, not only averages.** Record p50/p95 and maximum; report the exact sample count and do not compare measurements from materially different machines as though they were controlled.
5. **Conformance is separate from speed.** OGC's API conformance approach uses external black-box tests for observable behavior. Sol-Atlas must run the appropriate official conformance suite before claiming that an implemented API conforms; this core benchmark does not confer OGC conformance.

## Proposed acceptance targets (not measured yet)

Targets apply to a named reference runner, release profile, warmed process, one comparison at a time and a deterministic in-memory fixture. Establish and publish the reference runner before treating the latency limits as release gates.

| Gate | Proposed target | Fail-closed / interpretation |
|---|---:|---|
| Outcome classification | 100% of the nine qualitative baseline/after state pairs match the documented transition table | No ambiguous pair is silently mapped to recovery or degradation |
| Evidence negative controls | 0 unsafe positive conclusions across proposed, disputed, superseded, withdrawn, malformed-time, expired, unknown-ID, duplicate-ID, wrong-kind and subject-mismatch fixtures | Any false available/satisfied conclusion blocks release |
| Common-cause exposure safety | 100% of active/inactive/unknown, in-window/out-of-window and blocked-evidence cases match the contract; 0 blocked domains reported as exposed or not-triggered | Exposure is not an asset failure; out-of-scope is not safe; common-cause analysis must not mutate operational states |
| Functional-adequacy correctness | 100% of golden scenarios produce the expected adequate/degraded/unavailable/indeterminate class and capacity/coverage bounds | Unknown remains indeterminate; observed values must exactly match cited evidence; mismatched units are rejected |
| Functional-adequacy negative controls | 0 accepted cases with duplicate evidence IDs, wrong entity, quantity/unit/uncertainty mismatch, expired evidence or unsupported observed basis | Reject conflicting evidence instead of averaging or silently preferring one source |
| Service-floor correctness | 100% of cells and community aggregates match declared floor thresholds and weight arithmetic | Meets-floor, below-floor and indeterminate weights remain separate; a weighted average cannot erase per-community results |
| Service-floor uncertainty safety | 0 indeterminate weight counted as definitely meeting or definitely below floor | Status derives only from conservative lower and optimistic upper coverage bounds |
| Service-floor order invariance | 100/100 input permutations serialize identically | Stable community/cell ordering; duplicate cell IDs and duplicate service/community pairs fail closed |
| Network flow oracle correctness | 100% of series bottleneck, parallel-route, unknown-link, inactive-link, uncertainty-bound and min-cut fixtures match hand-calculated expected values | No summing capacities across a shared bottleneck; min-cut IDs must match the actual declared lower-bound graph |
| Topology completeness safety | 0 adequate/unavailable decisions when topology is partial or unknown | Partial network data must return indeterminate, even if computed subgraph flow is zero or exceeds demand |
| N-k stress exhaustiveness | 100% of single-, double- and triple-link failure combinations plus all declared failure bundles are evaluated for requests within budget; 0 successful partial sweeps | Over-budget requests fail explicitly; combinations and declared event bundles are deterministic; unknown/inactive links are not treated as active single-link failure candidates |
| Stress materialization budget | At most 10,000 failure cases and 2,000,000 total scenario-link references per exhaustive request | Oversized case/graph products are rejected before cloning the full scenario batch |
| Worst-case oracle | 100% agreement with exhaustive independent oracles on small networks, including all ties for worst-case lower/upper flow loss | Failure-set IDs and worst-case tie sets must be reproducible under input permutation |
| Stress monotonicity | No link-failure case may improve conservative or optimistic max-flow beyond numeric tolerance | Any monotonicity violation is a correctness failure, not a resilience gain |
| Flow conservation | Absolute max-flow/min-cut value difference <= 1e-9 for the deterministic test fixtures | Any unexplained discrepancy blocks use of the result as a min-cut explanation |
| Recovery bracket correctness | 100% of fixtures identify the first robustly adequate sample and its preceding sample correctly | Report a sampled bracket, never an exact unobserved physical recovery time |
| Recovery deficit integration | Absolute difference <= 1e-9 from the hand-calculated trapezoidal normalized-deficit oracle on deterministic fixtures | Lower deficit integral must never exceed upper deficit integral; label result equivalent full-service-deficit seconds, not physical volume |
| Recovery sampling-gap safety | 100% of trajectories with any gap above their declared maximum are rejected | No silent extrapolation/interpolation across long missing periods; no sustained claim beyond the final sample |
| Determinism | 100/100 repeated runs produce byte-identical canonicalized result data for the same inputs | Runtime measurements may vary; output data must not |
| Small comparison | 100 groups / 200 dependency edges / 200 evidence records: p95 <= 50 ms | Initial target; no target is claimed until measured |
| Medium comparison | 1,000 groups / 2,000 edges / 2,000 evidence records: p95 <= 250 ms | Same runner and binary as small comparison |
| Large comparison | 10,000 groups / 20,000 edges / 20,000 intervention evidence records: p95 <= 2,000 ms | No dropped groups; recovered count must match fixture oracle |
| Common-cause overlap, small | 100 groups with two alternatives each: p95 <= 5 ms for domain assessment + overlap correlation | Exactly 100 overlap pairs; every pair matches the fixture's shared-feeder oracle |
| Common-cause overlap, medium | 1,000 groups with two alternatives each: p95 <= 50 ms | Exactly 1,000 overlap pairs; output bytes stable against reference |
| Common-cause overlap, large | 10,000 groups with two alternatives each: p95 <= 500 ms | Exactly 10,000 overlap pairs; no dropped group, wrong-time join or false warning |
| Functional adequacy, small | 100 services / 200 observed capacity-and-demand records: p95 <= 5 ms | All 100 results match the golden degraded-capacity oracle |
| Functional adequacy, medium | 1,000 services / 2,000 observed records: p95 <= 25 ms | Evidence corpus validated once; output order stable by scenario ID |
| Functional adequacy, large | 10,000 services / 20,000 observed records: p95 <= 250 ms | No missing assessments; every capacity/demand measurement preserves unit, uncertainty and evidence trace |
| Service-floor assessment, small | 100 service cells / 200 observed measurements / up to 17 communities: p95 <= 10 ms | Per-cell statuses, weighted totals and golden bounds match the oracle |
| Service-floor assessment, medium | 1,000 service cells / 2,000 observed measurements / up to 17 communities: p95 <= 50 ms | Evidence corpus validated once within the shared service-assessment batch; output order deterministic |
| Service-floor assessment, large | 10,000 service cells / 20,000 observed measurements / up to 17 communities: p95 <= 500 ms | No missing cells or lost weights; all community and aggregate bounds reconcile |
| Network flow, small | 100 networks / 300 directed links / 400 observed capacity-and-demand records: p95 <= 10 ms | Every flow and minimum cut matches its synthetic oracle |
| Network flow, medium | 1,000 networks / 3,000 links / 4,000 observed records: p95 <= 100 ms | Batch evidence index built once; deterministic result ordering |
| Network flow, large | 10,000 networks / 30,000 links / 40,000 observed records: p95 <= 1,000 ms | No missing scenarios; topology-incomplete networks remain indeterminate; all results match the expected bottleneck oracle |
| Exhaustive N-2 stress, small | 10 active links / 56 cases (55 enumerated combinations + 1 declared common-cause bundle): p95 <= 10 ms | All cases included; bundle disables two parallel routes together; worst-case output matches the synthetic oracle |
| Exhaustive N-2 stress, medium | 20 active links / 211 cases (210 enumerated combinations + 1 declared common-cause bundle): p95 <= 50 ms | All cases evaluated; input order does not change the result manifest |
| Exhaustive N-2 stress, large | 40 active links / 821 cases (820 enumerated combinations + 1 declared common-cause bundle): p95 <= 250 ms | Every case evaluated; all worst-case pair sets are returned; no sampling or truncation |
| Network-intervention correctness | 100% of golden interventions match baseline/candidate flow, min-cut, delta and effect oracles | A model delta does not claim physical work or causal impact; conservative-bound regression and mixed bound movement are distinct; incomplete topology never yields a service-wide effect |
| Pareto portfolio correctness | 100% of candidate plans receive the expected non-dominated layer and sorted dominating-plan IDs on hand-constructed trade-off sets | Equal objective vectors do not dominate; input permutation preserves fronts; no hidden weighted score replaces the objectives |
| Network intervention, small | 100 plans / 100 networks / 300 links: p95 <= 25 ms | Each plan changes explicit assumptions only; exact baseline/candidate result bytes match the golden batch oracle |
| Network intervention, medium | 1,000 plans / 1,000 networks / 3,000 links: p95 <= 200 ms | Evidence corpus indexed once per batch; stable ordering by (scenario ID, plan ID) |
| Network intervention, large | 10,000 plans / 10,000 networks / 30,000 links: p95 <= 2,000 ms | No dropped plans; candidate overrides remove superseded capacity/availability evidence IDs; exact synthetic flow deltas match oracle |
| Pareto portfolio, small | 10 plans x 10 scenarios / 100 candidate comparisons: p95 <= 25 ms | Exact output bytes match the reference; front-1 count and dominated-plan count are stable |
| Pareto portfolio, medium | 50 plans x 10 scenarios / 500 candidate comparisons: p95 <= 125 ms | Pairwise dominance and front-layer assignment included; scenario cases remain unweighted |
| Pareto portfolio, large | 100 plans x 10 scenarios / 1,000 candidate comparisons: p95 <= 250 ms | Stable plan ordering, no missing plan/scenario pair, every dominator list sorted and deterministic |
| Recovery trajectory, small | 100 samples / 200 observed capacity-and-demand records: p95 <= 5 ms | First adequate sample, preceding-sample bracket, deficit area and evidence trace match the golden oracle |
| Recovery trajectory, medium | 1,000 samples / 2,000 observed records: p95 <= 40 ms | Evidence indexed once, maximum sample gap enforced, output is deterministic |
| Recovery trajectory, large | 10,000 samples / 20,000 observed records: p95 <= 500 ms | No missing snapshot; exact sample count and recovery bracket; lower/upper integrated deficit bounds match oracle |
| Peak resident memory | <= 512 MiB for the large fixture | Measure with an OS process monitor; the Rust harness does not measure RSS itself |
| Reconciliation safety | Every economic prototype keeps physical quantities, obligations and financial settlement separately reconcilable | No single synthetic “value” or reputation score may replace ledger-level checks |
| Spatial interoperability | 100% of applicable mandatory tests in the selected official OGC conformance suite before making a conformance claim | Only applies when the corresponding spatial API adapter is implemented |

The latency and memory values above are **engineering targets proposed for this project**, not industry-standard requirements and not current performance claims. If a reference runner demonstrates that a target is unrealistic, preserve the raw data, explain the bottleneck, and amend the target through a reviewed change—not by suppressing difficult cases.

## Running the first CPU benchmark

From the repository root:

```sh
SEI_BENCH_WARMUP=5 \
SEI_BENCH_SAMPLES=100 \
cargo bench --bench spatial_economic_intervention
```

The harness emits one JSON line for each main workload size (100, 1,000 and 10,000 groups/services/networks/trajectory samples). Each line includes group/edge/evidence counts; p50/p95/max latency for intervention comparison, common-cause overlap, batch functional adequacy, distributional service-floor assessment, batch network flow/min-cut, batch network-intervention comparisons, service-recovery trajectories, and exhaustive network stress; expected outcome counts; and serialized result sizes. Each measured run is checked against an untimed serialized reference and synthetic correctness oracle. The functional-adequacy, network-flow, and recovery-trajectory timings validate observed capacity/demand evidence and index the shared corpus once per evaluated batch. Service-floor timing runs those same capacity/demand scenarios through explicit community cells and essential coverage thresholds, then verifies weighted floor-met/below-floor/indeterminate totals and per-community disparity bounds against an untimed reference. Network-intervention timing compares each baseline/candidate pair while reusing the evidence index. Recovery fixtures have a known degraded interval, then a final adequate sample; the harness checks the recovery time bracket and trapezoidal normalized-deficit oracle for each workload size. The added exhaustive-stress workload evaluates every 1- or 2-link failure set for networks with 10, 20 and 40 active links (55, 210 and 820 combinations respectively), plus one declared common-cause event that disables two alternate routes together (56, 211 and 821 total cases). It checks exact candidate/evaluated counts, declared-event provenance, worst-case loss and all tied worst-case sets, and fails rather than sampling. Unit tests compare 64 small deterministic directed networks against an independent exhaustive min-cut oracle and exercise a 10,000-link chain to guard against recursive call-stack limits. Output serialization is checked outside timing. The harness does not claim that timing targets passed. Use an external process monitor (for example `/usr/bin/time -v`) to measure peak RSS, and archive the stdout with the commit SHA and environment manifest.

## Pilot impact gates (separate from synthetic performance)

These are candidate project goals to ratify with the pilot's affected communities and operators before field use. They are not industry standards and must not be declared achieved from synthetic tests. For distributional review, ratify the essential floor and denominator with affected communities before evaluating an intervention; track worst-community floor compliance and the maximum between-community gap separately from total flow. Do not optimize a fairness metric by relabelling missing or indeterminate service data.

| Gate | Candidate target | Evidence required |
|---|---:|---|
| Reviewable-response cycle time | >= 20% lower median time from qualified disruption evidence to a human-reviewed response plan versus the documented manual baseline | Paired, representative exercises; report p50/p95 and all excluded cases |
| Decision traceability | 100% of recommended actions expose the baseline, assumptions, relevant evidence IDs, temporal scope, uncertainty/blockers and expected trade-offs | Deterministic audit over every recommendation in the evaluation set |
| Economic event integrity | 0 duplicate effects or unbalanced postings in adversarial tests for retries, partial delivery, rejection, refund/reversal and settlement reconciliation | Explicit event/idempotency keys and independent physical/obligation/financial reconciliation |
| Field outcome efficacy | Candidate target: >= 10% relative improvement in one pre-registered primary outcome (for example water-delivery reliability or food-loss reduction), with an evaluation interval that rules out no improvement | Qualified observations and a pre-registered counterfactual/study design; report uncertainty and limitations |
| Distributional safety | No critical guardrail regression for a pre-agreed region or affected group, even if the aggregate result improves | Stratified outcome and harm metrics, review by affected stakeholders, appeal path |

A 10% field-efficacy target is a proposed decision threshold, not a claim that the system can currently deliver it. If the evaluation is underpowered, confounded, missing key populations, or has no credible counterfactual, report **inconclusive**, not success. Do not choose or change the primary outcome after seeing results. Improvement in cycle time alone does not establish improved water service, food security or social benefit.

The pilot should use a vector of physical, economic and distributional outcomes rather than a single composite “prosperity” score. A faster recommendation is not beneficial if it increases losses, excludes a community or moves cost into an untracked ledger.

## Required benchmark families

### 1. Semantic and adversarial correctness

- Enumerate group sizes 1 through 8 across available, unavailable and unknown inputs for all-required, any-one and at-least-k semantics.
- Cover inactive relationships, evidence-blocked edges, and thresholds larger than the number of active members.
- Ensure assumed conditions are labelled as assumptions; observed conditions require a matching, admissible, temporally valid observation.
- Preserve negative controls for unknown/duplicate evidence IDs, invalid record structure, wrong evidence kind, wrong subject, disputed support and expired support.
- Verify the six output-delta classes and all nine pairs of qualitative group states.
- Shuffle group, relationship and evidence input order; confirm canonical output is invariant.

### 2. Scale and performance

- Scale groups, relationships and evidence together, as well as stress imbalance (many groups sharing one prerequisite; few groups with many alternatives).
- Report validation-only time separately from full baseline/intervention comparison when a separate timing surface exists.
- Measure p50/p95/max, groups per second, evidence records checked, output size and peak resident memory.
- Capture one baseline after compiler/cache warm-up; do not use queued CI or an unmeasured developer-machine impression as performance evidence.

### 3. Spatial and economic integration

- For spatial adapters, include CRS/axis-order correctness, bounding-box edge cases, geometry validity, temporal query behavior, pagination and license/provenance retention. Run the applicable official suite rather than treating generic JSON validity as interoperability.
- For future economic adapters, test partial delivery, rejection, return/refund, reversal, duplicate/idempotent messages and independent physical/financial reconciliation.
- For outcome evaluation, pre-register baseline, outcome metric, comparison window, uncertainty treatment, negative controls and human-review criteria before claiming impact.

## Research basis

- OGC API – Features documents spatial API capabilities and the available conformance test suite tests externally observable behavior: https://www.ogc.org/standards/ogcapi-features/ and https://cite.ogc.org/teamengine/about/ogcapi-features-1.0/1.0/site/
- NIST's reliability handbook shows that parallel/redundant models depend on explicit component-independence assumptions; qualitative dependency rules must not be presented as quantified probabilities without supporting data: https://www.itl.nist.gov/div898/handbook/apr/section1/apr183.htm and https://www.itl.nist.gov/div898/handbook/apr/section1/apr184.htm
- OpenTelemetry's performance benchmark guidance emphasises a defined workload/configuration and warm-up/reporting protocol: https://opentelemetry.io/docs/specs/otel/performance-benchmark/

This protocol is intended to make improvements measurable and reproducible—not to make benchmark scores a proxy for truth, public benefit, fairness or operational safety.


## Water-service balance batch

The separate water_service_balance benchmark evaluates 100, 1,000 and 10,000 synthetic water-service scenarios. Each scenario has distinct source, pump, treatment, distribution, storage and demand entities; the serial stages have a known treatment bottleneck. The run asserts the expected degraded state, ordered delivery-capacity bounds, storage-capacity limits, explicit distribution-loss reconciliation, zero-within-tolerance whole-ledger mass residuals and byte-identical result serialization. It reports p50/p95/max latency and result bytes; it does not claim field accuracy or measured performance before execution.

Run with WATER_BENCH_WARMUP=5 and WATER_BENCH_SAMPLES=100 using cargo bench --bench water_service_balance.

The model's conservation gate is 1e-9 m3 per aggregate balance path. Initial proposed p95 targets are 5 ms / 25 ms / 250 ms for 100 / 1,000 / 10,000 services, respectively. These targets are local engineering hypotheses, separate from hydraulic correctness. The adapter boundary must be validated against EPA EPANET fixtures before making pressure, head-loss, water-quality, or real-network claims.


## Water-service trajectory batch

The time-dependent water benchmark evaluates contiguous synthetic trajectories with 10 intervals each at 100, 1,000 and 10,000 total intervals. Each trajectory has normal service, an explicit source outage, storage depletion, and a restoration interval. The benchmark asserts exact interval count, deterministic serialized outputs, conserved volume, bounded ending storage, cumulative deficit totals, and a sampled recovery classification. It does not claim measured field accuracy.

Run with WATER_TRAJECTORY_BENCH_WARMUP=3 and WATER_TRAJECTORY_BENCH_SAMPLES=30 using cargo bench --bench water_service_trajectory. Proposed p95 budgets for 100 / 1,000 / 10,000 total intervals are 15 ms / 100 ms / 500 ms. These are engineering hypotheses, not industry standards or current results. The report should be attached to the exact commit and runner manifest.


## Water-service trajectory intervention comparison

The paired-counterfactual harness runs 100, 1,000 and 10,000 baseline/candidate pairs, each with 10 contiguous hourly intervals. The baseline has a two-interval source outage; the candidate supplies a synthetic 50 m3/h during those intervals. The golden result is a 100 m3 reduction in cumulative unmet demand per pair, with both trajectories first returning to robust adequacy at the same sample. Candidate outputs are explicitly marked as modeled counterfactuals rather than causal evidence.

Run with WATER_INTERVENTION_BENCH_WARMUP=3 and WATER_INTERVENTION_BENCH_SAMPLES=30 using cargo bench --bench water_trajectory_intervention. Proposed p95 budgets for 100 / 1,000 / 10,000 comparison pairs are 50 ms / 500 ms / 5,000 ms. Each sample asserts paired time-grid validity, expected effect class, exact synthetic deficit delta, result counts, deterministic output bytes, per-step bound ordering, and separate baseline/candidate evidence-reference arrays. These are proposed local engineering targets, not measured performance or external standards.

## Coupled water-energy trajectories

The water-energy benchmark evaluates 100, 1,000 and 10,000 contiguous intervals. Each interval limits a 100 m³/h mechanical pump with synthetic 40 kW pump-terminal power and 0.5 kWh/m³ operating specific energy, yielding an 80 m³/h energy-limited flow bound. The fixture is synthetic and is not a field accuracy claim.

Run with `WATER_ENERGY_BENCH_WARMUP=3` and `WATER_ENERGY_BENCH_SAMPLES=20` using `cargo bench --bench water_energy_coupling`. The JSON report records p50, p95, max latency, measured sample count, interval count, output bytes, the effective flow bound and whether each **proposed** p95 target was met:

| Coupled intervals | Proposed p95 target |
| ---: | ---: |
| 100 | 20 ms |
| 1,000 | 150 ms |
| 10,000 | 1,000 ms |

These are local engineering hypotheses, not external standards or measured performance. Attach each report to its exact commit, compiler, runner and host manifest; correctness and latency must be reported separately. A target pass does not imply hydraulic validity. The first oracle comparison remains the qualified EPANET corpus in issue #139, using only quantities with compatible physical meaning and explicit tolerances.

## Paired water-energy counterfactual comparison

The paired benchmark compares a synthetic 20 kW baseline pump with a 40 kW candidate pump at the same 0.5 kWh/m³ operating-regime estimate and a 100 m³/h nameplate limit. The deterministic oracle is a 40 m³ reduction in unmet demand per interval, with the result classified as a robust modeled improvement. The harness measures the full baseline-plus-candidate comparison and checks the oracle on every run.

Run with `WATER_ENERGY_INTERVENTION_BENCH_WARMUP=3` and `WATER_ENERGY_INTERVENTION_BENCH_SAMPLES=20` using `cargo bench --bench water_energy_intervention`.

| Paired intervals per side | Proposed p95 target |
| ---: | ---: |
| 100 | 40 ms |
| 1,000 | 300 ms |
| 10,000 | 2,000 ms |

These are unmeasured engineering targets. A performance target can be marked met only from a report tied to the exact commit, compiler, runner and host manifest. Correctness gates and latency numbers are separate, and even an exact synthetic-oracle match does not establish that a real hydraulic system follows the aggregate model.

**Benchmark metadata interpretation:** The water-energy benchmark JSON attempts to record the source revision, Rust compiler, CI runner/host architecture, output size and `/proc/self/status` `VmHWM` when available. `VmHWM` is a process-wide high-water mark since process start; because the three workload sizes run in one process, it is **not** an isolated per-workload peak. For comparable RSS by workload size, run each size in a fresh process or measure each invocation with an external process monitor. Missing metadata remains null rather than being fabricated.

## Joint water-energy stress sweep with protected service floors

The benchmark uses 100 independent service-cell trajectories, each with 10 aligned hourly intervals. It sweeps 10 / 100 / 900 declared common pump-terminal power-degradation scenarios (11,000 / 101,000 / 901,000 service-step assessments including nominal baselines), each scenario affecting the same named time step across all service cells. Every sample asserts complete evaluation of the declared scenario count, exact evaluation-budget and override accounting, the expected zero worst-case floor-compliance lower bound, and byte-identical serialized output against an untimed reference before reporting output bytes.

Run with `WATER_ENERGY_STRESS_BENCH_WARMUP=1` and `WATER_ENERGY_STRESS_BENCH_SAMPLES=3` using `cargo bench --bench water_energy_stress_sweep`.

| Declared failure scenarios | Service-step assessments including baseline | Proposed p95 target |
| ---: | ---: | ---: |
| 10 | 11,000 | 500 ms |
| 100 | 101,000 | 5,000 ms |
| 900 | 901,000 | 45,000 ms |

These are proposed engineering budgets, not measured results. The harness records scenario and override counts, p50/p95/max, source revision/compiler/runner metadata, output bytes, and a process-wide RSS high-water mark when available. `VmHWM` is a process-wide high-water mark since process start, not an isolated per-workload peak; use fresh processes or an external process monitor for per-size memory comparison. The evaluator caps all service-step assessments at 1,000,000 and total scenario overrides at 100,000. The next scalability step is compact/streamed aggregation before raising those bounds.

## Paired intervention comparison across joint-failure suites

The dedicated `water_energy_stress_intervention` harness compares a 100-cell × 10-interval baseline and candidate across 10 declared pump-terminal power-loss scenarios (22,000 paired service-step assessments for each comparison). It runs 10 / 100 / 500 repeated evaluations of the same deterministic candidate pair inside each sample to measure repeated-comparison throughput without holding hundreds of full trace outputs in memory. Every comparison asserts robust improvement, zero protected-floor regression, complete scenario evaluation and a stable summary; each sample's serialized output must match an untimed reference.

Run with `WATER_ENERGY_STRESS_INTERVENTION_BENCH_WARMUP=1` and `WATER_ENERGY_STRESS_INTERVENTION_BENCH_SAMPLES=3` using `cargo bench --bench water_energy_stress_intervention`.

| Repeated evaluations per sample | Paired service-step assessments | Proposed p95 target |
| ---: | ---: | ---: |
| 10 | 220,000 | 1 s |
| 100 | 2,200,000 | 10 s |
| 500 | 11,000,000 | 50 s |

These targets are unmeasured engineering hypotheses. This measures repeated individual comparisons, not a compact multi-candidate batch API; a batch API that evaluates the shared baseline once and returns compact per-candidate/per-scenario summaries remains a worthwhile optimization. The full single-candidate comparison keeps detailed per-step provenance. See [issue #150](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/150). The harness records revision/compiler/runner metadata, p50/p95/max, output bytes and process-wide RSS high-water mark where available.


## Compact multi-candidate stress intervention batch

The new batch API evaluates one baseline and multiple candidate stress sweeps against the identical declared failure suite. It validates pairing before expensive evaluation, builds the evidence index once, evaluates the baseline once, sorts summaries by stable candidate ID, and emits compact candidate summaries with worst-case unmet-demand, delivered-volume and protected-floor bounds plus side-specific evidence IDs. It does not serialize each candidate's full per-step trace; use the detailed single-candidate API on shortlisted options.

The batch is bounded independently of per-sweep limits: at most 1,000 candidates, at most 1,000,000 total service-step assessments across the baseline plus all candidates, and at most 100,000 total declared overrides. Caller budgets must be positive and cannot exceed those hard caps. Over-budget work fails closed before model evaluation.

Run with \`WATER_ENERGY_STRESS_BATCH_BENCH_WARMUP=1\` and \`WATER_ENERGY_STRESS_BATCH_BENCH_SAMPLES=3\` using \`cargo bench --bench water_energy_stress_intervention_batch\`.

| Candidates | Service cells × intervals × cases per sweep | Batch assessment budget | Proposed compact-batch p95 target | Proposed repeated-full-trace p95 target |
| ---: | ---: | ---: | ---: | ---: |
| 10 | 100 × 10 × 11 | 121,000 | 1.5 s | 3 s |
| 30 | 100 × 10 × 11 | 341,000 | 4 s | 8 s |
| 80 | 100 × 10 × 11 | 891,000 | 12 s | 24 s |

The harness times both the compact batch and repeated full-trace single-candidate comparisons on the same candidates. It checks that every candidate's compact effect, protected-floor outcome, envelope bounds and evidence IDs exactly match the full comparison; it also asserts deterministic compact output bytes and complete candidate evaluation. The output records source/compiler/runner metadata, p50/p95/max, output bytes, theoretical assessment work avoided by baseline reuse, and a process-wide RSS high-water mark.

These are **unmeasured engineering targets**, not published standards or performance claims. The process RSS high-water mark is shared across benchmark cases and is not isolated per workload. Timing and correctness are separate acceptance gates; no hydraulic accuracy or real-world causal effect follows from synthetic agreement.


## Shared-resource constrained allocation

The shared-resource allocation benchmark stresses one aligned snapshot with every service connected to ten shared resource constraints. Resource capacity is present once per resource ID, rather than copied per service. Workloads use deterministic balanced connectivity so their expected equal-coverage policy outputs are hand-calculable.

Run with SHARED_RESOURCE_ALLOCATION_BENCH_WARMUP=2 and SHARED_RESOURCE_ALLOCATION_BENCH_SAMPLES=10 using cargo bench --bench shared_resource_allocation.

| Services | Shared resources | Usage constraints | Conservative work units | Expected coverage | Proposed p95 |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 100 | 10 | 1,000 | 505,000 | 1.0 | 50 ms |
| 500 | 25 | 5,000 | 12,525,000 | 0.5 | 500 ms |
| 1,000 | 50 | 10,000 | 50,050,000 | 0.5 | 2 s |

The fixture uses 100 m³/h demand per service and 10,000 m³/h capacity per declared resource, with each service consuming one capacity unit per delivered rate unit on ten resources. The first workload has 100 services evenly sharing ten constraints and reaches full coverage; the latter two have 200 users per resource and reach 0.5 coverage. This balanced connectivity exercises throughput with one dominant fill round; the suite needs an additional staggered-bottleneck profile to exercise many successive freeze rounds. The benchmark asserts service count, relationship count, per-service coverage, protected-floor compliance, resource-capacity residuals, and byte-identical serialized output against a reference assessment.

The harness records revision/compiler/runner metadata, p50/p95/max, output bytes, work units, and /proc/self/status VmHWM when available. RSS is a process-wide high-water mark shared by the workload sizes; use fresh processes for per-size memory comparisons. All latency targets are **unmeasured engineering hypotheses**, not standards or current performance claims.

Correctness gates are independent of speed: no resource is spent more than once, every service and resource relationship is explicit, weighted floor categories reconcile, indeterminate outcomes are preserved, evidence references validate, and input permutations serialize identically. The current policy is equal-coverage progressive filling, not a physical network solver or a universally optimal allocation policy.


### Staggered-bottleneck iteration profile

The benchmark also runs 100 services over ten shared resources with one usage edge per service and stepwise capacity limits. Each resource has ten consumers; its capacity is set so its group reaches coverage 0.1, 0.2, …, 1.0 in turn. This forces **ten successive freeze rounds** instead of allowing all active services to stop in the first round. The expected protected-floor compliance lower bound is 0.8 for a floor of 0.3, and the benchmark asserts all ten conservative and optimistic iterations, each resource constraint, per-service coverage, deterministic output bytes, and the weighted-floor oracle.

Proposed p95 target for this 100-service / 10-resource / 100-usage staged case: **50 ms**. It uses 50,500 estimated work units and is intended to expose iterative overhead alongside the balanced throughput cases above. The target remains unmeasured.



## Shared-resource allocation: independent analytical oracle sweep

The shared-resource unit tests now include 100 deterministic generated complete-bipartite cases: 2–8 services, 1–5 resources, varied positive demands, varied per-service/per-resource coefficients, and declared shared capacities. The oracle is independently computed from the closed form for this restricted topology:

\[
c^* = \min\!\left(1,\ \min_r \frac{C_r}{\sum_i d_i a_{ir}}\right)
\]

where every service consumes every resource, \(d_i\) is demand and \(a_{ir}\) is resource usage per unit delivered. Under equal-coverage progressive filling, every service has coverage \(c^*\); its delivery is \(d_i c^*\). This is a deliberately narrow analytic oracle—not a substitute for a general LP oracle or a proof that the model represents a physical network.

Each generated case checks exact service/resource counts, coverage against the independent formula, allocated delivery against demand times coverage, no resource overspend, and byte-relevant assessment equality after reversing service/resource/usage order. The deterministic LCG seed is fixed in source so any counterexample is replayable without adding a random-number dependency.

**Correctness acceptance targets:** 100/100 generated cases agree within absolute coverage tolerance \(10^{-8}\), delivery-rate tolerance \(10^{-7}\), and resource-capacity tolerance \(10^{-7}\); 0 overspend findings; 100/100 reversed-input cases return identical assessments. These targets are assertions in the test, not a claim that the new test has passed in CI.

**Next independent oracle:** expand beyond complete-bipartite cases to small sparse incidence graphs, compare the progressive-fill result with exhaustive discrete max-min/lexicographic enumeration and an independent LP implementation, and report policy differences rather than treating every feasible allocation as interchangeable. HiGHS is a candidate for an offline differential harness because it solves sparse LP/MIP/QP problems; it should not become an implicit production dependency solely to test this Rust core. Hydraulic quantities still require separate EPANET comparison where semantics align.


### Sparse incidence-graph leximin oracle sweep

A second deterministic unit-test oracle now generates 100 sparse graphs with three service cells and four shared resources. Every service is constrained by at least one resource, every resource has at least one consumer, and capacities are integer multiples of the 100 m³/h service demand with unit resource-use coefficients. For these deliberately bounded instances, the progressive-fill breakpoints lie on the 1/6 coverage grid; the independent oracle exhaustively enumerates a finer 1/30 grid (31³ = 29,791 coverage tuples per case) and selects the lexicographically maximal sorted feasible coverage vector.

Each case checks the sorted policy outcome, the exact-grid breakpoint assumption, no resource over-allocation, and identical assessment after reversing service, resource and usage order. The fixed deterministic seed and failing incidence/capacity manifest make counterexamples reproducible. This check tests a narrow unit-coefficient linear model; it does not establish general LP equivalence or physical network validity.

**Acceptance target:** 100/100 generated cases agree with the independent exhaustive oracle, with no capacity overspend greater than 1e-7, and 100/100 reversed-input assessments match exactly. These are encoded expectations, not a statement that the Rust tests have passed on the current head.

Next, independently cross-check larger sparse graphs with heterogeneous demands and resource-use coefficients using a staged leximin LP. HiGHS is a candidate offline solver for sparse LP/MIP/QP formulation checks, without adding it to the production Rust dependency graph. Compare only the same objective and tie-breaking semantics. For hydraulic validity, keep reference comparisons separate: EPA describes EPANET as an extended-period hydraulic and water-quality simulator covering flow, pressure, tank level and constituents, which goes beyond this allocator's declared linear capacity contract (https://www.epa.gov/water-research/epanet).


### Sparse shared-resource scaling profile

The registered `shared_resource_allocation` harness now includes a sparse case with **1,000 services, 50 shared resources and 3,000 usage constraints** (three resource edges per service). Capacities are derived from each resource's declared degree so every resource is exactly binding at 0.5 coverage under the fixture's 100 m³/h demand and unit coefficients. Each sample validates the 1,000 result rows, expected uniform coverage, capacity feasibility, and deterministic serialized output before emitting timing data.

| Services | Resources | Usage constraints | Conservative estimated work | Expected coverage | Proposed p95 target |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 1,000 | 50 | 3,000 | 15,015,000 | 0.5 | 1,000 ms |

This complements the 1,000-service / 50-resource / 10,000-constraint balanced profile (proposed p95 <= 2,000 ms) by isolating sparse incidence overhead while maintaining a hand-computable oracle. Both targets are **unmeasured hypotheses**; correctness is a separate acceptance gate, and a synthetic linear allocation result is not a hydraulic simulation result.
