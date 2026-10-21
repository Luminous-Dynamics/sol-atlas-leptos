# FlexDC-Sim Cross-Validation Plan v0.1

**Status:** the pinned FlexDC-Sim reference workflow completed successfully (run #3 / ID `39518611387`, tested Sol-Atlas head `1267897bab2534100cded8f054740df81de40231`). It produced and validated paired source-native and same-row RSR metrics for 3,601 simulator samples. This is a reproducible simulator-metric result—not wall-metered facility energy, an evaluated alternative scheduler, or proof of grid benefit.  
**Related code:** `sol-atlas-core/src/demand_response_metrics.rs` and the `demand_response_assess` CLI in this branch.  
**Tracking work item:** [AI-COMP-ENERGY-003 / issue #256](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/256).

## 1. Reference implementation and attribution

- Repository: [PEACLab FlexDC-Sim](https://github.com/peaclab/flexdc-sim).
- Pinned commit: [`ea50bd582b5d1b55fcb09ea46899fa9d325cce3b`](https://github.com/peaclab/flexdc-sim/tree/ea50bd582b5d1b55fcb09ea46899fa9d325cce3b).
- Software license stated by its README: BSD 3-Clause. The associated ACM E-Energy 2026 paper states CC BY 4.0 for the paper itself; keep code and paper licenses distinct and preserve attribution/citation.
- Reference paper: Acun et al., [*Investigating Power Consumption Flexibility of AI Data Centers for Demand Response Participation*](https://doi.org/10.1145/3744255.3798112), ACM E-Energy 2026.
- This plan consumes the external simulator as a pinned baseline. No FlexDC-Sim implementation code is copied into the Rust production module and no external Python runtime dependency is added to Sol-Atlas.

The paper reports 18%–55% power flexibility relative to average power for its evaluated RSR cases and up to 20% power reduction in its EDR cases. These are study-specific outputs from the authors' workloads, measured power-performance profiles and simulator setup—not a general law or a Sol-Atlas result. The pinned implementation defines the normalized error as `abs(P_measured - P_target) / reserve` and calculates the linear-interpolated p90 of the absolute exported `trackingError` series; its runtime check flags violation when that p90 is greater than 0.3. We separately report the fraction of observations with error >0.3 and a strict `<10%` exceedance-rate statistic, but **do not treat that statistic as equivalent to the implementation's p90 check**. For example, one error of 0.8 among ten samples produces interpolated p90=0.08 (p90 criterion passes) while exactly 10% of samples exceed 0.3 (strict exceedance-rate criterion fails). These details follow the [pinned metric extractor](https://github.com/peaclab/flexdc-sim/blob/ea50bd582b5d1b55fcb09ea46899fa9d325cce3b/src/peacsim/extract_power_metrics.py), [pinned runtime policy](https://github.com/peaclab/flexdc-sim/blob/ea50bd582b5d1b55fcb09ea46899fa9d325cce3b/src/peacsim/aQA_runtimepolicy.py), and the [Acun et al. paper](https://doi.org/10.1145/3744255.3798112).

## 2. Pinned reference configuration artifacts

All blob IDs below refer to the pinned commit above. They are identifiers for reproducible review, not signatures of real-world truth.

| Input/artifact | Blob SHA-1 | Selected semantics |
|---|---|---|
| `configs/experiment/exp_low_util.ini` | `554bb80355e85fb0c3fbf73e4d8a4f1521fa2184` | 1 s simulation timestep; 3,600 s duration; seed 20; target utilization 0.6; 50 servers; 120 W configured idle power; normalized power-provider signal with 4 s granularity; fixed signal start rather than randomized start |
| `configs/cluster/cluster.ini` | `9acf360f05e88e82e812c8d412979a91dcbe3399` | Per-cluster-node-type idle watt values; keep distinct from the experiment's server-count setting |
| `configs/workload/W2-short-qos3445.ini` | `c7f5e7d9e91de0ef17a4995b4d07cd896dc436f1` | Four inference job profiles (ResNet, GPT-2, Llama, Bloom) with declared min/max execution times, min/max job power and QoS values |
| `configs/policy/example_flexDC_simulation.ini` | `8a12f500ed8116b99e7e6b79010d5ab76be85051` | AQA policy with P=24,216 W; R=9,626 W; ratios and a path to precomputed weights |
| `src/peacsim/output/optimization/W2-simulated_annealing_low_util_availability=False_20250308130817/opt_weights.csv` | `f84164c63ba4215f92bb41c174955ec6e041db85` | Four job-type weights used by the policy configuration |
| `src/peacsim/output/optimization/W2-simulated_annealing_low_util_availability=False_20250308130817/configurations.json` | `96f2158470b61345d1571924a645078edc02bc53` | Recorded config for the weight-optimization run; inspect before assuming it matches the evaluation run exactly |
| `data/ISO24h.csv` | `6f7a8e4275d00ad43702f8a94d0a085b79f21f65` | Provider signal values used by the configured reference run. Its external provenance/license and interpretation still require review before generalizing it to a specific ISO/market |
| `src/peacsim/extract_power_metrics.py` | `6f325cd54994a70effce50a0329509179df7408c` | Uses pandas linear-interpolated p90 of the `trackingError` field for its saved tracking metric |
| `src/peacsim/iso_signal.py` | `80e3ab8f63e60188d6e1fa677b4f05fb69ae29c8` | Normalizes and samples the provider signal; signal time granularity and simulator step are different |
| `src/peacsim/simulator.py` | `ab2510fd9b73f6e62c2169c1e95b389c513d0f05` | Main loop, power target selection, node/job updates, tracked-error timing, power trace, QoS report |
| `src/peacsim/extract_qos_metrics.py` | `83c61b1cf14f90be90cc02cc01687ea7382d5f43` | Summarizes job QoS by type and counts violations against per-job thresholds |

The example policy references a precomputed optimizer output. To reproduce the results, pin both the optimizer configuration and weights; do not characterize an optimizer-derived configuration as globally optimal. The reference config is a *comparison profile*, not evidence that its inputs match Sol-Atlas v0.1 automatically.

## 3. Important measurement and alignment traps

**Power units.** FlexDC-Sim configuration and `power_trace.csv` power columns are in watts. Sol-Atlas metric input expects kW, so divide target and measured powers by 1,000 and reserve watts by 1,000. This preserves the normalized ratio when all three values are converted consistently.

**Time bases.** The simulation updates once per second, but the provider target signal has 4-second granularity. Preserve the original `time` field and the exact conversion/indexing. Do not join series on row number alone after filtering or resampling.

**Tracked error versus output-row fields.** In the pinned `simulator.py`, `update_cluster` assigns the signed error as `(self.real_power - last_power) / (R + 0.0001)` once the current time reaches the signal granularity. `self.real_power` is the total from the previous cycle's policy update and `last_power` is the prior signal interval's target cap. The loop then executes the current runtime policy, recomputes `real_power`, and writes the current `target` and `realSum`. Thus the exported `trackingError` is a **lagged source-native series** relative to same-row `target`/`realSum`; before the 4-second signal granularity, the branch explicitly writes zero. The `error_violate()` p90 check uses the entire exported column including startup zeros. Separately, `final_output()` excludes the first row for its mean tracking-cost calculation, and `extract_power_metrics.py` contains a different helper that computes `quantile(abs(realSum-target), 0.9) / total_reserve` from same-row fields without the runtime epsilon. These are three different constructions and must not be conflated.

The adapter emits **both** views from the same source CSV. The `rsr_reference` section scores the exported signed `trackingError` series, takes absolute values and computes the source p90 without rebuilding source timing or denominator. The `rsr` section uses same-row `target` and `realSum`, converts both from W to kW, and divides their absolute difference by the exact configured reserve (9,626 W) rather than adding the runtime epsilon. Thus the report can quantify both the source-native result and a clearly labelled same-row reconstruction. It is a diagnostic contrast—not a correction claiming the latter is the source's true service response.

The committed synthetic paired fixture intentionally demonstrates the difference: nine source errors are zero and one is 0.8, so source-native p90 is approximately 0.08 and passes the p90 threshold; same-row target/realSum rows at times 4–6 and 8–9 differ by 0.4 reserve, making five aligned samples exceed 0.3 and aligned p90 approximately 0.4, which fails the p90 threshold. These are fabricated known-answer inputs to test interpretation, not observed FlexDC-Sim output. The report also carries source revision/field, sample timing, denominator convention, raw CSV digest, exact run-manifest digest and notes. In a real replay, both series must come from the same run artifacts before the difference is interpreted as a timing effect.

**Reserve denominator.** The runtime code divides by `R + 0.0001`; a separate utility computes an error from `realSum - target` and divides by `total_reserve`. Sol-Atlas's initial power-pair helper uses exact positive reserve and rejects zero, so parity should report this numerical boundary instead of hiding it behind an epsilon.

**Workload QoS.** FlexDC-Sim's job trace measures arrival, start/end time, sojourn time and job-type QoS. Sol-Atlas's current aggregate evaluator has homogeneous work units and protected floors, not this node/job execution model. Only compare an explicitly mapped aggregate metric, not raw work units with inference jobs or raw completion count with QoS-qualified accepted work.

**Energy boundary.** The simulator's power trace is modeled from job/server power-performance profiles and idle power. It is not a physical facility wall meter, PUE measurement, grid-flow model, carbon trace, water meter or verified grid-service telemetry. The output can support reference-simulator comparisons only.

## 4. Adapter and scorecard

### Executable source adapter

The new Rust binary `sol-atlas-core/src/bin/flexdc_trace_adapter.rs` converts a pinned FlexDC-Sim `power_trace.csv` into the strict input consumed by `demand_response_assess`:

```sh
cargo run -p sol-atlas-core --bin flexdc_trace_adapter -- \
  /path/to/power_trace.csv /path/to/run-manifest.json > /tmp/flexdc-rsr-input.json
cargo run -p sol-atlas-core --bin demand_response_assess -- \
  /tmp/flexdc-rsr-input.json > /tmp/flexdc-rsr-report.json
```

The adapter supports only the numeric CSV export shape from the pinned source revision `ea50bd582b5d1b55fcb09ea46899fa9d325cce3b`. It is intentionally fail-closed: quoted CSV fields, missing headers, non-numeric/non-finite tracking errors, blank rows, a missing or duplicate time row, a nonuniform timestamp grid, mismatched manifest row count, and nonzero startup errors before the configured signal interval are rejected. It does not silently drop or resample records. The run manifest must pin six exact configuration references (experiment, cluster, workload, runtime policy, power signal and optimization weights) to the source commit's Git blob identities. Those SHA-1 IDs identify source objects; the adapter also hashes the exact manifest bytes and power-trace bytes with SHA-256 and carries both digests into the emitted input.

The manifest must state the simulation duration/step, signal step, random seed, exact command, run identifier, source revision, exact Python major/minor/patch version, the pinned QoS percentile (0.9), power cap/reserve and whether this artifact is synthetic. A non-synthetic run is rejected unless it carries a relative path to a captured runtime-environment manifest and its SHA-256 digest (for example, a complete package-version freeze and relevant environment details). The adapter resolves that path relative to the run-manifest file, reads the actual bytes and recomputes the digest; a self-declared hash without the artifact is insufficient. The upstream `requirements.txt` is explicitly a contributor/development requirements file, while `setup.cfg` lists broad runtime requirements without fully pinned versions, so source commit plus requirements file alone does not guarantee reproducible dependency resolution. A practical capture recipe for a real run is to save interpreter/platform information and a sorted complete package listing to the declared relative file, compute SHA-256 on those exact bytes, archive that file with the power trace and run manifest, and retain the three digests together. The environment-manifest digest is therefore a separate required identity for a real run. The CLI output contains both a source-native `rsr_reference` view and a separately recomputed same-row `rsr` view. The source metadata records the lagged timing semantics and `R + 0.0001` denominator; the reconstructed view uses exact configured R. Both preserve original time IDs. The manifest's `synthetic` flag is propagated rather than hard-coded: the committed test fixture sets it to true, while a future adapter input may set false for an actual simulator execution. In either case the claim ceiling stays `metric_accounting_only`, because simulator output alone is not physical facility metrology. Artifact/config hashes bind identities but cannot prove that the stated run actually occurred.

The committed 10-row CSV and run manifest are **synthetic adapter tests**, not an executed simulator run. The fixture models signal targets at 4-second boundaries and a lagged source error with nine zero values and one 0.8 value. Its same-row `target`/`realSum` pairs intentionally include five 0.4-reserve errors; source p90 is about 0.08 while same-row p90 is about 0.4. The run-manifest fixture names real source config Git blobs but explicitly states the CSV was not generated from those files; it also sets `synthetic: true` and leaves the environment digest null. The adapter propagates this flag, and real runs must set `synthetic: false` and supply the runtime manifest digest. The fixture must never be reported as FlexDC-Sim performance evidence.

The schema workflow runs the adapter into the metric CLI and validates both produced JSON documents against the Draft 2020-12 input and report schemas. It also asserts the p90/strict-rate counterexample survives this end-to-end conversion. This verifies the adapter/metric contract; it does not execute the FlexDC-Sim repository or show a simulator parity result.

Create a derived, versioned adapter input with one row per ordered sample and these required fields:

- source repository commit, config blob hashes, seed, output artifact hashes and adapter version;
- sample time with original time unit, signal index and target source;
- mapped target power and realized cluster power, with source unit and converted kW values;
- workload/job trace digest plus the mapping from job type to the Sol-Atlas accepted-work definition (or a declaration that the measures are not equivalent);
- missing/out-of-range row counters, excluded-row reasons and sample alignment note;
- claim ceiling `simulator_metric_comparison_only` unless separate physical measurements and provenance support a stronger claim.

The same derived trace should be scored by both FlexDC-Sim's published outputs and Sol-Atlas's Rust metric CLI. Save its exact bytes and SHA-256. Do not change the seed, time slice, target signal, work mix, QoS thresholds, or reserve while comparing policies.

### Required paired metrics

| Metric | Calculation / comparison rule |
|---|---|
| RSR reference decision | Compare exported `trackingError` p90 against 0.3 using the source's linear-interpolated quantile definition |
| RSR exceedance-rate statistic | Separately report `count(error > 0.3) / n` and whether it is strictly below 10%; do not substitute it for FlexDC-Sim's p90 decision |
| Recomputed aligned error | Independently compute errors only after documenting exact target/sample alignment and denominator; compare p90 and exceedance-rate outputs to the source-native series |
| Target tracking | Mean absolute error, p90 error, max error, sign/ramp response and time above threshold |
| QoS | Per-job-type QoS/deadline violations, unfinished/queued job counts, completed/accepted jobs, and sojourn time distribution |
| EDR | Baseline/event kWh, gross curtailment, rebound, net reduction, peak overshoot and unserved/unfinished workloads |
| Timing | Response delay, recovery time, sample interval and alignment policy |
| Determinism | Identical source commit/config/seed/output digest across repeated runs, or an explanation of nondeterminism |
| Independent solution | Exact exhaustive oracle on small compatible instances; FlexDC-Sim's simulated annealing is a candidate heuristic configuration, not itself proof of a global optimum |

## 5. Falsification and acceptance

- First reproduce FlexDC-Sim's own baseline output from pinned artifacts and same environment. Archive the run manifest, dependency lock, config digests, seed, metrics and output hashes.
- Test one hand-calculable RSR sequence and one EDR sequence with known answers in both tools. Reconcile the 4-second signal, 1-second simulation update and tracking-error row timing before comparing real output.
- Repeat fixed-seed runs before testing variable seeds; measure variation and state whether any nondeterminism appears.
- Require zero unsupported input mappings and no silent missing-value drops. Unknown energy, carbon, water and facility impacts stay unknown.
- Report all model-boundary mismatches. Only compare schedules that preserve equivalent QoS constraints and workload availability; otherwise call it a model comparison, not a policy win/loss.
- Do not claim that flexibility reduces generation or transmission capacity. This needs a separate power-system model and a declared passive-load counterfactual.
- Do not claim SOTA superiority until the same workload, target, QoS, power boundary, target samples, hardware profile and measurement rule are matched against the strongest applicable baseline.

## 6. First executed reference result

The first actual pinned run was completed by GitHub Actions workflow [run #3 / ID `39518611387`](https://github.com/Luminous-Dynamics/sol-atlas-leptos/actions/runs/39518611387) at Sol-Atlas code head `1267897bab2534100cded8f054740df81de40231`. It checked out FlexDC-Sim commit `ea50bd582b5d1b55fcb09ea46899fa9d325cce3b`, used Python `3.10.16`, seed `20`, 3,600-second simulated duration with one-second steps and four-second signal granularity, AQA power cap P=24,216 W, reserve R=9,626 W, and QoS percentile 0.9. The run manifest has `synthetic: false` and includes the captured runtime-environment manifest and digest.

**Evidence bundle:** [download the run artifacts](https://github.com/Luminous-Dynamics/sol-atlas-leptos/actions/runs/39518611387/artifacts/12310455207). The bundle contains the raw `power_trace.csv`, `job_table.csv`, `runtime-environment.txt`, exact `run-manifest.json`, adapted trace, receipt-wrapped assessment report and paired summary. Archive digest: `sha256:c3edb6e9f2db00b55a13c1f06c685a5c8624f8e4ce6a03f554626d46c9131501`.

### Paired RSR results

| Metric | Source-native `trackingError` | Same-row reconstructed `target`/`realSum` |
|---|---:|---:|
| Samples | 3,601 | 3,601 |
| Mean absolute normalized error | 0.0508884 | 0.0427334 |
| p90 absolute normalized error | 0.0630000 | 0.0602846 |
| Maximum normalized error | 1.5040 | 1.500613 |
| Samples with error > 0.3 | 123 / 3,601 | 122 / 3,601 |
| Exceedance fraction | 3.4157% | 3.3879% |
| P90 threshold (p90 ≤ 0.3) | Pass | Pass |
| Supplementary strict exceedance rate <10% | Pass | Pass |

Difference in p90 (same-row minus source-native): −0.00271535. Do **not** interpret the slightly lower same-row value as a scheduler improvement. It is a comparison of two definitions over the same simulator run: FlexDC-Sim's signed source-native lagged series uses the prior signal cap and `R + 0.0001`; the reconstruction uses same-row `target`/`realSum` with exact declared `R`. Both variants pass the p90 check in this scenario, and both have fewer than 10% of samples above 0.3. The difference is small here but the synthetic known-answer test demonstrates that it can reverse a pass/fail decision in other distributions.

### Workload/QoS context

The simulator's per-job-type p90 QoS degradation values were 2.400 (ResNet), 2.989 (GPT-2), 3.167 (Llama) and 4.614 (Bloom), against configured thresholds 3, 4, 4 and 5. No job type crossed its configured p90 threshold in this run. However, the simulator also reported a 1.69784% per-job QoS threshold exceedance ratio among 3,475 jobs (about 59 jobs). These are different QoS summaries; retain both rather than claim zero job-level misses. The mean execution slowdown by type was approximately 7.93%, 7.26%, 6.00% and 5.28%, respectively. These outputs describe the reference schedule only; they do not compare Sol-Atlas scheduling against it.

### Provenance digests

- `power_trace.csv`: SHA-256 `f70501a7b80a6bdcfbae120ebc7e3564c9e32606b164ca25fda6e38816627365`
- `run-manifest.json`: SHA-256 `d94a7c5ad902ab25928d093acc90f80aa7af774eb5fd9904e4711053943c5912`
- `runtime-environment.txt`: SHA-256 `919e9affff8da271991769d0c52726f1fb73b038860b7e6285434f4a98c07a40`
- adapted input JSON: SHA-256 `7b047147567fd7fe712b5c6a95fa3db2fb96ce45070d1756d43fe5c5b5556d9c`
- assessment report JSON: SHA-256 `7a6d3a1fd0dfacb32c1f47f43e4d2af0b674bf91b33ca70467aee6f2fba44c15`
- report receipt: input SHA-256 `7b047147567fd7fe712b5c6a95fa3db2fb96ce45070d1756d43fe5c5b5556d9c`; metrics SHA-256 `33b44cb2536633afec86c3f613f5844e7efa5e07c84ee08285814c6cf88ae842`.

The reference workflow passed its end-to-end schema checks. The associated repository Check at the same code head passed `fmt`, native tests/all-target compilation and the WASM check. This establishes reproducible execution and report-contract validity, not physical metrology. The emitted power series is modeled by FlexDC-Sim's job/server power-performance profiles and idle power, not wall-metered facility energy. No CO₂, water, facility overhead, or avoided-generation credit is inferred.

## 7. Next execution gate

1. The new `.github/workflows/flexdc-reference.yml` executes the pinned upstream simulator with exact Python/dependency versions and the reviewed configuration. It captures the full environment manifest, run manifest, power trace, job table, and receipt-bound paired RSR report as a GitHub Actions artifact.
2. Inspect the exact-head run and its uploaded artifact. Confirm that the manifest’s config hashes, environment digest, row count and source revision match the workflow and that both output schemas validate.
3. Compare the source-native p90 and same-row recomputed p90, including threshold/exceedance counts, without assuming they should agree. Review the source job table and QoS outputs as a separate workload-quality dimension.
4. Only then append actual simulator metrics to the benchmark record. These remain modeled simulator evidence; they do not establish physical facility energy, site-grid benefit or SOTA superiority. The committed 10-row fixture remains the separate synthetic known-answer test.
