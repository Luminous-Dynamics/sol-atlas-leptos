# Water measurement benchmark contract v0.1

**Status:** preregistered synthetic benchmark targets; no scenario corpus has been generated or scored. The thresholds below are project targets, not standards, observed results, or a claim of superiority.

## Paradigm shift

Do not optimize for the largest entropy reduction or the most attractive map of sensor positions. Optimize for **reliable decisions under model uncertainty**, with decision relevance, costs, failure modes and distribution shift exposed.

That leads to six requirements:

1. **Decision value over information volume.** Report expected information gain (EIG) and expected value of sample information (EVSI) separately. A perfectly informative sensor can have zero EVSI if learning the state never changes the best decision. The Rust toy calculator now has a test for that counterexample.
2. **Model uncertainty is part of the experiment.** Do not assume the hydraulic model is exact. Vary demand profiles, roughness, boundary conditions, leak magnitude/location, sensor noise and missingness, and model form. EPANET supports both demand-driven analysis (DDA) and pressure-dependent demand analysis (PDA); under DDA, full nodal demand can remain imposed even when calculated pressures become negative. Therefore, do not interpret the current DDA W0 smoke case as a realistic low-pressure service model. Record the selected demand model and its parameters with every scenario.
3. **Generalization over row-wise random splits.** Hold out parameter/noise regimes and, where feasible, entire network topologies. Near-duplicate runs from the same topology/configuration must not leak across development and held-out sets.
4. **Robustness over average-only wins.** Preserve per-scenario outcomes, worst-case strata, failed solver runs, corrupted channels and no-fault controls. A strong mean cannot hide a catastrophic topology-specific regression.
5. **Calibration over confidence theatre.** Report probability calibration (Brier score) and decision regret, not only top-k accuracy. Confidence scores without synthetic or empirical ground truth do not establish reliability.
6. **Reproducibility over narrative.** Freeze scenario, seed and acceptance manifests before evaluation; bind all results to solver/source/binary, model configuration, scenario, channels and output hashes. Independently recompute reported metrics.

Research context: information-theoretic water-network sensor placement has considered relevance and redundancy; EPANET-generated pressure perturbations have been used for synthetic leak-localization tests; and recent work explores jointly placing sensors and calibrating drainage models. These are methodological precedents, not evidence that this implementation is superior. See the sources below.

## Benchmark gates

### Gate 0 — mathematical and contract controls

Required controls:

- Perfect sensor on a two-hypothesis toy model: EIG = 1 bit and known EVSI from the declared loss matrix.
- Uninformative sensor: EIG = 0 and EVSI = 0 within the declared numeric tolerance.
- **Decision-irrelevant perfect sensor:** EIG = 1 bit, EVSI = 0 when the optimal decision is the same under every hypothesis; it must not be selected when its declared cost is positive.
- Incompatible cost/loss units disable net-value ranking.
- Invalid probabilities/dimensions/IDs/units fail closed; zero-probability outcomes never produce NaNs.
- Permuting hypothesis order or measurement-outcome order leaves decision-value and information metrics invariant.
- Metamorphic and negative tests execute on the exact commit used for any claim.

The current Rust tests are authored; until an exact-head run is observed, they must not be described as passing.

### Gate 1 — solver and scenario integrity

Start with the existing tiny W0 model solely as a plumbing/smoke test. Then create a larger, versioned synthetic corpus across at least three distinct network topologies. Record for each scenario: topology and model hashes, solver version and binary digest, demand model (DDA/PDA), headloss model, boundary conditions, leak/fault class and magnitude, parameter draws, sensor placement/noise/dropout, seed, units, convergence state and output digests.

Mandatory negative controls include:

- no leak / no fault;
- sensor bias or drift that must not be silently classified as a physical leak;
- missing, duplicated, delayed and corrupted channels;
- parameter combinations outside the development envelope;
- a topology/model mismatch;
- a non-converged or malformed solver run.

A failed run is a failed/unavailable case, not a zero-valued measurement and not a successful detection. Report its count and retain its failure record.

### Gate 2 — fair strategy comparison

Compare the preregistered strategies:

- information-directed;
- random selection with frozen matched seeds;
- spatial-coverage greedy;
- fixed heuristic.

All strategies use the same candidate measurement sites, scenario cases and feasible sensor budgets. Report the budget itself; do not let one strategy quietly use more sensors or more observations. Freeze hyperparameters and criteria before scoring the held-out cases.

**Primary efficacy target:** on held-out scenarios, information-directed selection should improve top-3 localization accuracy by at least **5 absolute percentage points** over the strongest baseline, with a paired per-scenario bootstrap 95% confidence interval for the difference strictly above zero. This is a proposed research target; failure to meet it means no superiority claim, not a reason to tune the threshold after seeing results.

**Robustness guardrail:** on every reported held-out topology/model-form stratum, top-3 accuracy must not trail the strongest baseline by more than **5 percentage points**. Where detection delay is meaningfully defined, the p95 delay must not regress by more than **5%** versus the strongest baseline.

**No-fault guardrail:** false-positive-rate target is at most **5% point estimate**, and the Wilson 95% upper bound is at most **10%**. Publish numerator and denominator. If the corpus does not contain enough no-fault cases for this bound to be informative, expand the corpus rather than weaken the criterion.

**Corpus target:** at least **1,000 labelled scenario cases across at least three network topologies**, with at least **20% held out by grouped parameter/noise regimes**. Use leave-one-topology-out evaluation where feasible. A smaller or single-topology corpus is explicitly a pilot and cannot support topology-generalization claims.

### Gate 3 — report decision quality, uncertainty and cost

Publish per-strategy and per-topology results for:

| Metric | Direction | Required interpretation |
|---|---|---|
| Top-k localization accuracy | Higher | Primary efficacy metric; state k and denominator |
| p95 detection delay | Lower | Only when time-series/detection semantics are defined |
| False-positive rate on no-fault controls | Lower | Always report numerator, denominator and interval |
| Brier score | Lower | Probability quality against labelled synthetic truth |
| p95 decision regret | Lower | Selected-decision loss minus the oracle decision under the same frozen loss matrix |
| Worst-case held-out localization | Higher | Preserve the worst topology/parameter/noise stratum |
| Sensor count and acquisition cost | Lower, where comparable | Do not equate sensor count with lifecycle cost |
| EIG and EVSI | Diagnostic / decision value | High EIG without EVSI or a better decision is not a win |
| Solver/scenario failure rate | Lower | Keep failures in the denominator; no silent dropping |

Do not collapse these into an opaque scalar score. Show a Pareto view and individual guardrails. Net value is only arithmetic when cost and loss are genuinely comparable in the same declared unit; matching unit strings alone do not validate an economic assumption.

### Gate 4 — reproduction and claim ceiling

A second run must use the frozen corpus/seed manifest and pinned solver. An independent checker recomputes metrics from scenario-level records. Define the numeric tolerances before running. If the output cannot be reproduced, report the discrepancy and withhold a reproduction claim.

Even a successful synthetic benchmark supports only a claim about that synthetic test domain. Field relevance requires authorised, time-aligned observations; calibration and validation on independent periods; uncertainty and data-quality assessment; privacy/security review; independent engineering review; and human authorization. No synthetic result authorises sensor installation, operational changes or autonomous actuation.

## Implementation status and next executable step

- Rust expected-information/EVSI calculation and SHA-bound CLI: implemented in the public PR, but exact-head execution is not yet observed.
- Decision-irrelevance and permutation-invariance tests: added in this revision; execution remains unverified until tests actually run.
- Four strategy comparison and scenario-integrity protocol: specified.
- Acceptance targets: now frozen in `sol-atlas-core/fixtures/water-engineering/w0-measurement-experiment.synthetic.json` before any strategy scores exist.
- Scenario corpus, hydraulic-response likelihoods, sensor-noise calibration, strategy execution and result manifest: **not generated / not run**.

The next implementation milestone should be a **deterministic synthetic scenario generator and runner** that produces hashed, labelled pressure-response cases from pinned EPANET runs (then a separate SWMM track), validates scenario outputs, and creates a train/held-out manifest without crossing topology or parameter-regime boundaries. The information-directed method must consume the same evidence available to each baseline; it must not receive hidden ground truth.

## Model-uncertainty ensemble evaluation (2026-10-16)

A single prior/likelihood table can make a sensor look optimal by assuming precisely the relationship we want to test. The new [robust ensemble evaluator](https://github.com/Luminous-Dynamics/sol-atlas-leptos/blob/e1281e026bc985119d0d6b0270036ab03b75cfe1/sol-atlas-core/src/water_measurement.rs) runs the expected-information/EVSI calculation across two or more declared probability models. For each candidate measurement, it reports minimum and maximum EIG and EVSI across the supplied ensemble. It computes minimum/maximum net value only when all models share an explicit comparable cost/loss basis, and selects a candidate only when its worst-case net value is positive.

The ensemble validator fails closed if model variants change hypothesis IDs/order, candidate decisions, loss matrix, declared units, cost comparability, action IDs, outcome IDs/order, or acquisition cost. It also rejects duplicate model IDs, invalid individual models, and non-finite aggregates. The [Rust CLI](https://github.com/Luminous-Dynamics/sol-atlas-leptos/blob/944d609fd565c372870c9df13dc0577369bf4ab7/sol-atlas-core/src/bin/water_measurement_robust_evaluate.rs) binds the output to the exact input bytes using SHA-256. The [ensemble JSON Schema](https://github.com/Luminous-Dynamics/sol-atlas-leptos/blob/b5bed5c7a9caa24c0dc26b7349fa888902febe23/sol-atlas-core/schemas/water-measurement-ensemble-v0.1.schema.json) and [toy fixture](https://github.com/Luminous-Dynamics/sol-atlas-leptos/blob/5606b52a42fdda955df5a0355ee79960afb56301/sol-atlas-core/fixtures/water-engineering/w0-measurement-ensemble.toy.synthetic.json) are wired into the existing schema and native-test workflow.

The deliberately adversarial fixture has two equally weighted hypotheses and the same loss/cost basis, but changes the likelihood model so the action named `perfect_sensor` becomes uninformative in one variant. Its expected control is an EIG range of 0–1 bit, EVSI range of 0–2 toy loss units, and net-value range of -1 to +1; therefore the robust evaluator must **not** select it. These are expected test values from hand-authored math, not results from a hydraulic simulation. The fixture name is not treated as evidence about the sensor; the supplied likelihood tables are what determine the calculation.

The experiment fixture received a versioned **v0.1.1 amendment dated 2026-10-16**, before scenario generation or strategy scoring: it adds a ninth acceptance criterion for uncertainty-ensemble reporting and two metrics (`robustness.minimum_evsi` and `robustness.evsi_spread`). Existing numerical efficacy thresholds were left unchanged. The amendment is recorded in the fixture's own notes, rather than silently editing the frozen protocol.

This extrema-based calculation is a finite-ensemble stress test, not a formal guarantee against unknown models or a substitute for posterior model averaging, calibration, or real-data validation. Its main value is exposing **assumption fragility** instead of silently trusting one probability table. The next physical-model milestone remains a deterministic scenario generator driven by pinned EPANET runs, with operating-condition perturbations and topology/grouped held-out splits. Existing literature has explicitly studied sensor placement under demand and operating uncertainty, including [sensor placement and leakage localization considering demand uncertainties (2014)](https://doi.org/10.1016/j.proeng.2014.11.242) and [robust sensor placement under uncertain operating conditions (2014)](https://doi.org/10.1016/j.proeng.2014.11.453). Those studies motivate the method; they do not establish performance of this code.

**Validation status:** four new robust-ensemble unit tests and a CI check are authored, but have not been observed executing on the exact current head. JSON Schema validation and formatting are likewise not claimed as passed. The experiment protocol and hydraulic corpus remain `NotRun`.

## Deterministic hydraulic scenario-corpus pilot (2026-10-16)

The next stage now has an implementation path separate from the single restricted-pipe smoke test:

- [Scenario plan schema](https://github.com/Luminous-Dynamics/sol-atlas-leptos/blob/main/sol-atlas-core/schemas/water-scenario-corpus-plan-v0.1.schema.json)
- [Five-case W0 pilot plan (LPS)](https://github.com/Luminous-Dynamics/sol-atlas-leptos/blob/main/sol-atlas-core/fixtures/water-engineering/w0-scenario-corpus-plan.synthetic.json)
- [Nine-case Net3 pilot plan (GPM)](https://github.com/Luminous-Dynamics/sol-atlas-leptos/blob/main/sol-atlas-core/fixtures/water-engineering/net3-scenario-corpus-plan.synthetic.json)
- [Rust scenario mutation, execution and evidence module](https://github.com/Luminous-Dynamics/sol-atlas-leptos/blob/main/sol-atlas-core/src/water_scenario_corpus.rs)
- [CLI](https://github.com/Luminous-Dynamics/sol-atlas-leptos/blob/main/sol-atlas-core/src/bin/water_scenario_corpus_run.rs)
- [Run-manifest schema](https://github.com/Luminous-Dynamics/sol-atlas-leptos/blob/main/sol-atlas-core/schemas/water-scenario-corpus-run-v0.1.schema.json)

The W0 plan binds the 523-byte base INP by SHA-256 and runs five cases: one no-fault baseline, plus 2 and 5 L/s additional-demand perturbations at J1 and J2. A second plan now binds the pinned upstream [EPANET Example Network 3 (Net3)](https://github.com/OpenWaterAnalytics/EPANET/blob/4d8d82ddc260fce216af9321fc3d9a4646ac6827/example-networks/Net3.inp) by SHA-256 `d95b068194ba747605bea851d608333e577ac667658aaa1680cf1c07e43faf53` and byte length 30,063, then declares nine cases: one no-fault baseline, plus 10 and 25 GPM junction-demand perturbations at nodes 101, 103, 105 and 109. Source inspection found 92 Net3 junction records and verified the declared observation nodes occur in the [JUNCTIONS] section; the source has exactly one OPTIONS UNITS GPM declaration and non-negative numeric demands. Net3 is an upstream example topology, not a real utility network.

The Rust runner rejects base-input SHA/length drift, a plan/input unit mismatch, unsafe paths/IDs, duplicate scenario/node IDs, missing target junctions, non-finite or out-of-range perturbations, malformed units/report directives, and malformed pressure rows. It changes only the third field of the declared junction row. Before generating a solver input, it deterministically ensures `REPORT NODES ALL` and `REPORT LINKS ALL` without altering or re-identifying the original source bytes. Every manifest carries the native flow/pressure units (`L/s` and metres for LPS; GPM and psi for GPM), report policy, raw source hash, generated-input hashes, report/binary/stdout/solver hashes, and per-scenario success/failure records. Missing pressure is explicit unavailable data, never a zero. Stale outputs are removed before each invocation.

The CI job is wired to run both plans against the same pinned EPANET executable, archive generated inputs and result/failure manifests even after failure, verify each plan SHA and manifest digest, and require all 5 W0 and all 9 Net3 cases to complete with measurements before the corpus gate passes. The run-manifest schema is updated to bind each pressure value to a declared unit system. This is still source-code and contract inspection, not evidence that the runner, schema validator or solver has executed successfully.

This is now a **two-topology hydraulic response plumbing pilot**, not a leak-localization efficacy benchmark. The added junction demand remains a deterministic synthetic withdrawal proxy, not a physical leak. Together the plans contain 14 scenario runs but only two topologies, four observation nodes on Net3, no sensor-placement strategy scores, no calibrated sensor-noise model, no pressure-dependent demand assessment, and no field evidence. The preregistered target remains at least 1,000 labelled cases across at least three topologies with grouped held-out regimes; pilot success cannot satisfy that benchmark.

The scale gap is important: a 2026 study reports more than 47,000 Net3 scenarios in a leak-localization benchmark and distinguishes the easier binary detection task from harder localization ([*Benchmarking Diverse Learning Architectures for Leak Detection and Localization in Water Distribution Systems*](https://doi.org/10.1016/j.jwpe.2026.110114)). The EPA's WNTR documentation also identifies Net3 as a sample water-distribution network ([WNTR documentation](https://usepa.github.io/WNTR/)); we use it here only for a pinned, reproducible synthetic pilot. Research on sensor placement under demand uncertainty supports perturbing operating conditions rather than trusting one idealized model ([DOI: 10.1016/j.proeng.2014.11.242](https://doi.org/10.1016/j.proeng.2014.11.242)). EPANET 2.2 documents pressure-dependent demand as an analysis capability that a later low-pressure realism gate should consider ([official manual](https://usepa.github.io/EPANET2.2/)).

**Execution status:** the runner, plans, schemas and CI wiring have been read back from the public branch. Exact-head Actions queries for the previous head were empty; checks for the new current head remain to be queried after the branch update. Neither corpus has been observed executing yet, so no hydraulic pilot result is claimed. Rust compilation, formatting and JSON Schema runtime validation are not claimed as passed.

## Leakage-resistant pressure-signature localization baseline (2026-10-16)

The two-topology pilot now has a separate evaluation path that turns completed run manifests into reproducible *within-topology* localization baseline metrics:

- [Rust evaluator](https://github.com/Luminous-Dynamics/sol-atlas-leptos/blob/main/sol-atlas-core/src/water_localization_benchmark.rs)
- [CLI](https://github.com/Luminous-Dynamics/sol-atlas-leptos/blob/main/sol-atlas-core/src/bin/water_localization_benchmark_evaluate.rs)
- [Evaluation output schema](https://github.com/Luminous-Dynamics/sol-atlas-leptos/blob/main/sol-atlas-core/schemas/water-localization-evaluation-v0.1.schema.json)

The evaluator requires a fully completed, unit-consistent, SHA-bound run manifest; exactly one no-fault baseline; at least two withdrawal magnitudes; consistent observations; and every declared candidate node represented at each magnitude. It builds a pressure-change signature by subtracting the baseline pressure at each declared observation node. It then performs leave-one-magnitude-out folds and compares deterministic Euclidean-nearest-example and cosine-nearest-example baselines. The frozen candidate space comes from the manifest's declared observation nodes, not from held-out target labels. The predictor receives only held-out pressure features and labelled training signatures; target labels are consulted after rankings are produced, for metric calculation.

The output reports per-case ranked candidates, top-1 accuracy, top-k accuracy (k=min(3, candidate count)), mean reciprocal rank (MRR), and fold-level metrics, along with the exact input-manifest SHA-256 and units. The CI workflow is wired to produce and validate a separate evaluation artifact for W0 and Net3. Its checks enforce metric/schema integrity and exact fold/sample counts, **not** a minimum accuracy. That distinction lets the first run expose bad assumptions instead of tuning the acceptance gate to desired results.

Six evaluator unit tests are authored: two-magnitude fold outcomes, no-fault exclusion from candidates, incomplete-run rejection, missing/undeclared sensor rejection, unit inconsistency rejection, and refusing a one-magnitude dataset. They have not yet been observed executing.

**What this baseline does not establish:** the candidate classes are restricted to the declared observation nodes (two in W0, four in Net3), and the held-out dimension is only demand magnitude within each topology. The evaluator does not yet calculate average topological distance (ATD), false-alarm rate, detection delay, sensor-placement value, confidence calibration, sensor-noise robustness, held-out topology performance, or real-leak localization. This is an initial leakage-resistant scoreboard, not a substitute for a fully specified 1,000-case, three-topology benchmark.

The May 2026 Net3 study reports more than 47,000 simulated scenarios, over 95% binary detection accuracy across its evaluated models, and around 75–77% top-1 localization for ensemble methods under its protocol ([Ranganathan et al., *Journal of Water Process Engineering*](https://doi.org/10.1016/j.jwpe.2026.110114)). Those figures are context, **not directly comparable targets** until topology, target candidate set, split protocol, noise and label semantics are matched. Prior information-theoretic placement research jointly considered sensor relevance and redundancy and evaluated both accuracy and average topological distance ([2022 study](https://pmc.ncbi.nlm.nih.gov/articles/PMC8779686/)); adding graph-distance metrics and a real candidate-site set is a planned extension.

**Exact-head status:** the evaluator module, CLI, schema and two per-topology workflow steps have been read back from the public branch. The latest Rust/CI run is not yet known to have started or completed; no evaluator, solver, unit-test, formatter or schema-validation pass is claimed.

## Sources

- [US EPA — EPANET software, capabilities and manuals](https://www.epa.gov/water-research/epanet). The official manual documents DDA/PDA and warns that DDA can impose full demand even when calculated pressures become negative.
- [Khorshidi, Nikoo & Sadegh, *Water Research* (2018)](https://doi.org/10.1016/j.watres.2018.06.050), value of information and transinformation entropy for sensor placement.
- [Pressure sensor placement for leak localization (2022)](https://pmc.ncbi.nlm.nih.gov/articles/PMC8779686/), synthetic EPANET pressure-change scenarios and sensor redundancy.
- [Bayesian sensor placement and simultaneous drainage-model calibration (2025)](https://doi.org/10.1080/19942060.2025.2473992), motivation for considering sensor placement alongside calibration/model uncertainty.

The cited work motivates the design; it does not validate Sol-Atlas's implementation or the numerical acceptance targets above.


## Topology-aware localization error (2026-10-16)

The localization evaluator now takes two explicit inputs: the executed scenario manifest and the *original* EPANET INP source. It verifies that the INP byte digest matches \`base_input_sha256\` from the manifest, parses junction/reservoir/tank nodes plus pipe/pump/valve endpoints, checks that the manifest's observation and target junctions exist in the source topology, and confirms the source's declared \`OPTIONS UNITS\` agrees with the manifest.

For every held-out test, the evaluator still ranks candidates using pressure-change signatures only. After ranking is complete, it compares the actual target with the predicted top-1 node on an undirected graph and records shortest-path link hops. Each model reports mean topological error, nearest-rank p95 error, worst observed error, and disconnected-prediction count. The output also records the input topology SHA, node/link counts, and fixed metric semantics. Disconnected candidates are \`null\`, not a fabricated large number, and are counted separately. These values are diagnostic only; graph hops are not pipe-length distance, hydraulic influence, travel time, or repair cost.

This closes an important metric gap but not the candidate-universe gap: the current corpus still generates targets only at the two W0 or four Net3 declared observation junctions. It does **not** rank all 92 Net3 junctions or pipe segments, and it does not measure physical leaks. The next experimental milestone is a frozen target universe across all valid junctions plus pipe-segment candidates, followed by grouped held-out topology/operating-regime evaluation and explicit sensor-budget comparisons.

Research context: a 2023 *Water Research* paper combines graph signal processing and redundancy-aware sensor placement (https://doi.org/10.1016/j.watres.2023.120313), while a 2026 study describes joint treatment of network structure, hydraulic states, and risk propagation (https://doi.org/10.1016/j.ress.2026.112709). A May 2026 Net3 benchmark reports 47,000+ scenarios and identifies localization as a harder task than binary detection (https://doi.org/10.1016/j.jwpe.2026.110114). These papers motivate the metrics; none validates this implementation.

**Validation status:** source inspection confirms the evaluator has topology parsing, a source-digest gate, unit-system matching and graph-hop summary fields; tests and CI assertions are authored. Rust compilation, formatter execution, graph-metric tests and actual EPANET output remain unverified until exact-head workflows execute.


## Separate sensor sites from target hypotheses; exhaustive Net3 junction pilot (2026-10-16)

A sensor is where pressure is observed; a target hypothesis is where a fault or withdrawal may be located. Those sets must not be conflated. The localization evaluator now derives its frozen target-class universe from all labelled target junctions in the scenario corpus, while it constructs features only from the run manifest's declared observation nodes. A target is not required to have a sensor installed at that same junction. Every target is required to be represented at each held-out magnitude, preventing a fold from silently omitting a class from the model's training set.

The new [Net3 all-junction plan](../../sol-atlas-core/fixtures/water-engineering/net3-all-junction-localization-plan.synthetic.json) covers all **92 junctions** in the pinned EPANET Net3 source with two demand-addition magnitudes, 10 and 25 GPM, plus one no-fault control: **185 planned solver cases**. The four fixed observation nodes remain 101, 103, 105 and 109. If execution succeeds, the evaluator will rank all 92 target-junction classes for each of 184 withdrawal cases; CI validates the completed manifest, ranked-candidate count, unit contract, output schema and graph-distance diagnostics. No minimum accuracy threshold is tuned into this plumbing gate.

This changes the first localization experiment from four target classes to the full junction set without pretending that synthetic added demand is a physical leak. It still uses one example topology, two magnitudes, one deterministic hydraulic model and four fixed observation nodes. There is no sensor-noise or model-form variation, no grouped cross-topology holdout, no pipe-segment hypothesis class, no costed/optimized sensor placement, and no field data. The exhaustive plan is currently an experiment *specification*; solver output and localization performance are not results until the exact-head workflow produces and validates artifacts.

The next benchmark stages are: (1) generate scenario families across pressure-dependent demand and demand patterns; (2) inject calibrated sensor bias, noise, dropout, clock skew and topology/model mismatch as separate nuisance classes; (3) expand hypotheses from junctions to pipes/segments and distinguish leakage from legitimate withdrawal; (4) preregister grouped operating-regime and leave-one-topology-out splits; (5) compare fixed, random, coverage-greedy and information-directed layouts at matched sensor budgets; and (6) report calibration, top-k accuracy, MRR, mean/p95 topological error, false alarms on a broad no-fault corpus, p95 detection delay, EVSI/regret and solver-failure rates separately.
