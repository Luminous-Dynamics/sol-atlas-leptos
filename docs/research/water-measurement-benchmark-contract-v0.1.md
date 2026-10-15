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

## Sources

- [US EPA — EPANET software, capabilities and manuals](https://www.epa.gov/water-research/epanet). The official manual documents DDA/PDA and warns that DDA can impose full demand even when calculated pressures become negative.
- [Khorshidi, Nikoo & Sadegh, *Water Research* (2018)](https://doi.org/10.1016/j.watres.2018.06.050), value of information and transinformation entropy for sensor placement.
- [Pressure sensor placement for leak localization (2022)](https://pmc.ncbi.nlm.nih.gov/articles/PMC8779686/), synthetic EPANET pressure-change scenarios and sensor redundancy.
- [Bayesian sensor placement and simultaneous drainage-model calibration (2025)](https://doi.org/10.1080/19942060.2025.2473992), motivation for considering sensor placement alongside calibration/model uncertainty.

The cited work motivates the design; it does not validate Sol-Atlas's implementation or the numerical acceptance targets above.
