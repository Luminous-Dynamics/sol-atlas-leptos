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
