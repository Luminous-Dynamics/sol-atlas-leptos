# Spatial-Economic Intelligence benchmarks

**Status:** benchmark protocol and proposed targets; no performance result has yet been measured or claimed.  
**Contract version:** v0.1  
**Scope:** evidence validation, qualitative dependency groups, intervention comparison, common-cause overlap, uncertainty-bounded service-function adequacy, and network max-flow/min-cut composition in `sol-atlas-core`. The timing harness measures four computational surfaces; its targets remain unmeasured until executed on a named runner.

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
| Network flow oracle correctness | 100% of series bottleneck, parallel-route, unknown-link, inactive-link, uncertainty-bound and min-cut fixtures match hand-calculated expected values | No summing capacities across a shared bottleneck; min-cut IDs must match the actual declared lower-bound graph |
| Topology completeness safety | 0 adequate/unavailable decisions when topology is partial or unknown | Partial network data must return indeterminate, even if computed subgraph flow is zero or exceeds demand |
| Flow conservation | Relative max-flow/min-cut value difference <= 1e-9 on deterministic synthetic networks, subject to the declared numeric scale | Any unexplained discrepancy blocks use of the result as a min-cut explanation |
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
| Network flow, small | 100 networks / 300 directed links / 400 observed capacity-and-demand records: p95 <= 10 ms | Every flow and minimum cut matches its synthetic oracle |
| Network flow, medium | 1,000 networks / 3,000 links / 4,000 observed records: p95 <= 100 ms | Batch evidence index built once; deterministic result ordering |
| Network flow, large | 10,000 networks / 30,000 links / 40,000 observed records: p95 <= 1,000 ms | No missing scenarios; topology-incomplete networks remain indeterminate; all results match the expected bottleneck oracle |
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

The harness emits one JSON line for each workload size (100, 1,000 and 10,000 groups/services/networks). Each line includes group/edge/evidence counts; p50/p95/max latency for intervention comparison, common-cause assessment/overlap, batch functional adequacy and batch network flow/min-cut; expected outcome counts; and serialized result sizes. Each measured run is checked against an untimed serialized reference and synthetic correctness oracle. The functional-adequacy and network-flow timings validate a batch of observed capacity/demand records and index the shared corpus once. The network fixture is a series bottleneck feeding two parallel routes; it asserts the expected max flow, min cut, and degraded state for every network. Output serialization is checked outside timing. The harness does not claim that timing targets passed. Use an external process monitor (for example `/usr/bin/time -v`) to measure peak RSS, and archive the stdout with the commit SHA and environment manifest.

## Pilot impact gates (separate from synthetic performance)

These are candidate project goals to ratify with the pilot's affected communities and operators before field use. They are not industry standards and must not be declared achieved from synthetic tests.

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
