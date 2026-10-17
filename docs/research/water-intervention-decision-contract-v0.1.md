# Water intervention decision contract v0.1

**Status:** research and schema contract; not a deployed decision engine or a validated engineering method.  
**Research date:** 2026-10-15.

## The paradigm shift

The default workflow in many engineering projects is model -> dashboard -> recommendation. Sol-Atlas should instead treat the **intervention decision packet** as the unit of engineering evidence:

`authorised observations -> model/run evidence -> feasible interventions -> uncertainty analysis -> constraint-first comparison -> human decision -> independently verified outcome`

A hydraulic model remains a specialised calculation, not a digital twin by declaration. A good dashboard can make a bad assumption easier to see, but cannot make it true. The new contract therefore binds alternatives to named run bundles and requires each outcome to retain its unit, scope, time window, evidence class, comparability state and uncertainty state.

This addresses an active gap: the American Water Works Association's 2026 survey reports interest in AI, predictive analytics and digital twins, while noting that many utilities lack well-defined implementation plans. A defensible engineering plan needs explicit qualification and decision gates, not a promise of “real-time intelligence.” [AWWA, *Model or Digital Twin?* (15 April 2026)](https://awwa.onlinelibrary.wiley.com/doi/full/10.1002/awwa.70075).

## Five design shifts

1. **Optimise service, not a proxy.** Pipe velocity or simulated headloss is a diagnostic. Decision metrics should include service reliability, pressure compliance, unmet demand where represented, water quality where represented, energy, whole-life cost, environmental effects and affected populations. Do not count renewal kilometres as proof of better service.
2. **Unknown is a first-class result.** Missing pressure data, incomparable annual indicators, an uncalibrated roughness estimate, and a failed solver are not zero values. The packet preserves unavailable and non-comparable metrics and prevents them from becoming ranked gains.
3. **Hard constraints before Pareto trade-offs.** Applicable health, safety, regulatory and minimum-service constraints must be pass/fail/unknown before a candidate enters any trade-off frontier. A lower cost or energy metric must never compensate for an unresolved mandatory safety/service constraint. Keep the outcome vector; do not hide policy judgments in an unreviewed weighted score.
4. **Decide what to measure next.** Record material unknowns and the next evidence actions that could reduce them. Once deterministic runs are reproduced, declared sensitivity/sweep cases should test whether a candidate remains preferable when demand, roughness, boundary conditions and other uncertain inputs vary. Quantitative value-of-information estimates can come later; this v0.1 contract records the actionable evidence backlog without pretending it has computed one.
5. **Close the loop with independently observed outcomes.** Preserve the pre-intervention baseline. Recommendation, human authorization, completed work, and verified service outcome remain separate states. A repair record is not itself proof of water saved or improved service.

## Why the physical model boundary matters

The US EPA describes EPANET as a model for extended-period hydraulic and water-quality analysis in pressurised distribution networks. Its user manual distinguishes demand-driven analysis from pressure-driven analysis; under low-pressure conditions, that choice can materially affect how demand shortfall is represented. Sol-Atlas must record the analysis mode rather than silently treating the two interpretations as interchangeable. Sources: [US EPA EPANET project](https://www.epa.gov/water-research/epanet), [EPANET 2.2 User Manual](https://usepa.github.io/EPANET2.2/).

Keep SWMM as a separate adapter for appropriate rainfall-runoff, stormwater and sewer cases. Do not force drinking-water distribution, building plumbing, and stormwater hydraulics into one generic graph solver simply because they share a map. Shared evidence contracts are useful; interchangeable physics models are not.

Local service outcomes matter. South Africa's 2023 No Drop reporting described national non-revenue water increasing from 37% in 2014 to 47% in 2023, while the March 2026 Green/Blue/No Drop programme update reiterates the need to assess drinking-water quality, wastewater management and water-use efficiency together. These are reasons to retain distinct outcome dimensions—not proof that this prototype can explain the causes or select the right pipe. Sources: [South African Government, 2023 No Drop findings](https://www.gov.za/news/media-statements/water-and-sanitation-releases-2023-full-blue-drop-report-05-dec-2023), [South African Government, 27 March 2026](https://www.gov.za/news/media-advisories/government-activities/minister-pemmy-majodina-releases-green-drop-report-and).

## What this commit adds

- `water-intervention-decision-v0.1.schema.json`: a Draft 2020-12 contract for the baseline, candidate alternatives, typed outcomes, hard constraints, uncertainty review, equity review, comparison policy, decision state and nonclaims.
- `w0-intervention-decision.synthetic.json`: a deliberately **not-run** fixture. It uses unavailable metrics with null values and explicit reasons; no invented hydraulic results or ranking.
- A CI schema/fixture pair so the contract shape is checked alongside the existing water-engineering and public-observation bundles.

The schema is a shape gate, not a semantic verifier. The Rust semantic validator now enforces packet-local identifier uniqueness, unavailable-value/null consistency, uncertainty-bound consistency, run-state/digest consistency, hard-constraint evidence semantics, cross-case unit/scope/period consistency when a metric is marked comparable, and preconditions for recording a planning-only decision. It does not resolve or authenticate referenced run bundles, so it is not a cryptographic provenance verifier. Pareto calculation, uncertainty statistics, statistical robustness, causal attribution, and artifact-resolved run-bundle verification remain follow-up work. Do not display a ranked recommendation as qualified until those remaining checks, actual pinned-solver evidence, and independent engineering review exist.

## Next paradigm shift: optimise the next measurement

Once the baseline and candidate can be reproduced, the highest-value next action may be *not* to change a pipe. It may be to obtain the one observation that distinguishes between competing explanations and changes which intervention is worth considering.

This is a research-backed direction, not a new claim of capability. Peer-reviewed water-network studies have used **Value of Information (VOI)** and information-theoretic measures to select pressure-sensor locations for leak detection/contamination warning and to reduce redundant information across sensors. See [Khorshidi et al., *Water Research* (2018)](https://doi.org/10.1016/j.watres.2018.06.050), [*Pressure Sensor Placement for Leak Localization* (2022)](https://pmc.ncbi.nlm.nih.gov/articles/PMC8779686/), and [Comboul & Ghanem, *Value of Information in the Design of Resilient Water Distribution Sensor Networks*](https://ascelibrary.org/doi/10.1061/%28ASCE%29WR.1943-5452.0000259). These are methodological precedents, not proof that a Sol-Atlas implementation will outperform existing approaches.

### Decision-directed evidence acquisition

Represent possible information-gathering actions as explicit, reviewable records: a short pressure-logging campaign, meter validation, a targeted leak survey, a water-quality sample, an asset-record/site inspection, or verification of an annual-report table cell. Each action should state its question, cost and latency assumptions, access permission, affected area, required safety controls, expected evidence type, and the uncertainty it might reduce.

Where a defensible loss function and parameter model exist, the conceptual decision criterion is expected reduction in decision loss minus information-acquisition cost:

`net VOI(a) = current minimum expected loss - expected post-measurement minimum loss - acquisition cost(a)`

The expectation is over possible measurement outcomes, not over a made-up single forecast. This number should not be emitted until the outcome likelihoods, loss/cost basis and relevant uncertainty have been declared and reviewed. Entropy reduction can be a useful diagnostic for information gain, but **information gain is not automatically service benefit or monetary value**. When the model cannot support a defensible VOI estimate, show the candidate actions and their unresolved assumptions without numeric ranking.

A future synthetic test should compare an information-directed strategy with explicit baselines such as random placement, coverage-only placement and a fixed heuristic. Report detection/localisation performance, redundant information, sensor count/cost, sensitivity to misspecified leak/demand/roughness scenarios, and failures—not merely the best-case score. Freeze the scenario corpus before comparing strategies, keep negative controls, and report results across all evaluated cases.

### Executable research protocol and fair baselines

The machine-readable companion is `sol-atlas-core/schemas/water-measurement-experiment-v0.1.schema.json`, with the deliberately **not-run** plan at `sol-atlas-core/fixtures/water-engineering/w0-measurement-experiment.synthetic.json`.

The experiment requires four comparison strategies to share one frozen scenario corpus and matched seed policy:

- **Information-directed:** choose the next synthetic sensor/measurement to maximise a declared information objective; do not call it decision value unless a defensible loss and cost model supports that estimate.
- **Seeded random:** a reproducible chance baseline, not an arbitrary unseeded straw man.
- **Spatial-coverage greedy:** a transparent coverage-first baseline.
- **Fixed heuristic:** a declared, operationally interpretable rule, defined and frozen before results are examined.

Measure top-k leak localisation on synthetic labelled truth, detection delay, sensor count, information redundancy and held-out worst-case localisation. Preserve every scenario failure and unavailable metric. Do not turn information entropy into an implied financial benefit, blend dimensions into a hidden score, tune to held-out cases, or select acceptance thresholds after seeing results. The schema now blocks an evaluation marked `Completed` unless (a) model state is `Completed`, (b) the scenario corpus is generated and hashed, (c) acceptance criteria are frozen, and (d) all four baseline strategies have result records.

This is still only a **protocol**. Costs and expected information/decision value are intentionally unestimated; the corpus contains zero generated scenarios; evaluation state is `NotRun`; fieldwork is explicitly unauthorised. The next implementation should generate synthetic leak/demand/roughness/noise cases from a pinned solver, freeze training vs held-out manifests, and implement the four strategies before reporting comparative performance.

Research precedents: [information-theoretic pressure sensor placement for leak localisation (2022)](https://pmc.ncbi.nlm.nih.gov/articles/PMC8779686/) demonstrated its method using EPANET-generated pressure-change scenarios on the Hanoi network; [optimal sensor placement using Value of Information and transinformation entropy](https://www.sciencedirect.com/science/article/abs/pii/S0043135418305001) compares an information-theoretic approach to other sensor-placement recommendations. These papers motivate the benchmark, not a claim that the proposed Sol-Atlas strategy is better. The first independent question is whether it outperforms seeded random and transparent coverage/heuristic baselines on held-out synthetic cases under matched assumptions.

### How this changes the Sol-Atlas / Symthaea / Mycelix division

- **Sol-Atlas:** owns the spatial evidence packet, permissible measurement-action catalogue, comparison surface and reproducible test corpus.
- **EPANET / SWMM:** generate domain-appropriate synthetic hypotheses/results within their validated capabilities; neither substitutes for field observations.
- **Symthaea:** may propose competing explanations, identify which assumptions discriminate between them, and suggest an evidence-gathering plan. It must return structured hypotheses tied to specific model inputs and testable outcomes, not free-form certainty.
- **Symtropy:** may contribute catchment/terrain/flow context where its own conservation and model-fidelity contracts support it; it must not invent municipal pipe topology.
- **Mycelix:** can preserve who proposed, authorised, collected, reviewed and resolved each evidence action, including access restrictions and correction history. Provenance is not itself proof that a measurement is accurate.

Do not publish sensitive pipe/sensor locations or operational weaknesses by default. Public outputs should disclose only reviewed aggregates; exact topology, telemetry and proposed inspection locations require data-owner permission and an explicit threat/privacy review.

**Proposed next executable gate:** implement the measurement-action and test-corpus contracts on synthetic-only data; exercise the decision gate with fail-closed fixtures; then compare VOI/information-theoretic placement with simple baselines under reproducible EPANET scenarios. Real sensor deployment remains out of scope until the synthetic comparison is independently reproduced and qualified reviewers approve the operational/data-handling plan.

## Qualification ladder

- **Contract-only:** schema/fixture validation passes; no model result is implied.
- **Reproduced synthetic:** exact inputs, solver build identity, reports, continuity checks and output hashes agree on an exact run. This proves the controlled test was reproduced, not that a municipal model is accurate.
- **Uncertainty-screened synthetic:** declared parameter/range sweeps report their coverage and failures; do not cherry-pick only favourable samples.
- **Empirically validated model:** calibration and withheld observations, error distributions and known limitations are reviewed by a qualified engineer.
- **Shadow decision pilot:** historical replay and prospective recommendations are checked against later evidence; decisions and physical actions remain under accountable human authority.
- **Verified intervention outcome:** post-work observations are compared with a frozen baseline, accounting for changing demand, operating conditions and measurement uncertainty.

The long-term opportunity is not to automate every engineering decision. It is to make engineering claims **reproducible, comparable where justified, uncertainty-aware, and accountable from source observation through to service outcome**.
