# Civic Systems Intelligence: beyond a spatial-temporal-dynamical model

**Status:** research and architecture proposal; not a claim of a validated product or model.  
**Initial home:** Sol-Atlas spatial / evidence interface.  
**Cross-project intent:** Symthaea (reasoning and hypotheses), Mycelix (provenance, challenge and governance), Symtropy (dynamics and intervention simulation), Sol-Atlas (spatial context, observations and interface).  
**Decision:** do not build a universal “health score” or treat one simulation as a source of truth.

## Executive thesis

A spatial, temporal, dynamical model is necessary for understanding complex systems, but it is not sufficient.

It can describe **where** a state occurs, **when** it occurs, and **how** a modeled state changes. It does not, by itself, establish whether the data are trustworthy, why a change occurred, whose interests an objective serves, how institutions respond strategically, what evidence would falsify a model, or which intervention is robust under deep uncertainty.

The proposed paradigm is a **federation of evidence-linked models**: several explicit, scoped models with typed interfaces, shared provenance, causal claims, plural objectives, intervention tests, and public challenge mechanisms. The interface can offer an integrated view; the underlying evidence and models must remain separable and inspectable.

This is a design direction, not a claim that the current repositories already implement these capabilities end to end.

## 1. What spatial + temporal + dynamical modeling misses

| Lens | Question it answers | Failure if omitted |
|---|---|---|
| Spatial | Where are assets, people, conditions, exposures and dependencies? | Spatial averages hide local failure and unequal exposure. |
| Temporal | What changed, over what interval, and with what lag? | The model confuses snapshots, trends and delayed effects. |
| Dynamical | How do stocks, flows, constraints and feedback loops evolve? | Static correlations miss accumulation, depletion, cascades and path dependence. |
| Causal | What mechanism may explain a change; what would happen under an intervention? | Correlation is presented as cause, or a simulation is mistaken for a counterfactual. |
| Epistemic | Where did each claim come from; how uncertain, stale, biased or dependent is the evidence? | Repeated sources appear independent; missing data are mistaken for good outcomes. |
| Institutional / agentic | Who acts, decides, adapts, coordinates, defects, learns or changes the rules? | Institutions are treated as passive objects; incentives and strategic responses vanish. |
| Normative / distributional | Which outcomes matter, to whom, subject to what rights and floors? | A weighted average hides severe harms to a minority or sacrifices one group for aggregate gain. |
| Decision / control | Which feasible intervention improves which objectives at what cost and risk? | The platform describes problems but cannot support accountable action. |
| Adversarial / reflexive | How can actors manipulate the metric, evidence, model or interface once it affects rewards? | Goodhart effects, strategic reporting and propaganda degrade the system after launch. |
| Learning / governance | Who can challenge assumptions, correct records, revise the model and judge outcomes? | Errors become institutionalized and affected people are only data subjects, not participants. |

These lenses should not be forced into one giant ontology or one global model. They should be composable, domain-scoped contracts.

## 2. Paradigm shifts to aim for

### A. From “one world model” to model federation

Use purpose-limited models—such as water reliability, fiscal sustainability, bank liquidity, ecosystem condition and public-health capacity—with explicit units, time horizons, geographic scales, assumptions and validation records.

Connect them through typed dependencies, not undocumented shared variables. A water model may depend on electricity availability; a fiscal model may depend on maintenance costs; their coupling must say how the outputs map between models, at which time step, with what uncertainty, and with what evidence.

The system should support competing models of the same phenomenon. Model disagreement is useful evidence, not something an ensemble should silently average away.

### B. From map layers to claims with provenance

A map pin should be only a view of a claim-bearing entity. Every published metric or assertion should trace through:

1. source record or observation;
2. unit, scope, date and geographic reference;
3. transformation and version;
4. model and parameter set;
5. uncertainty and missingness treatment;
6. reviewer / attestation / challenge state;
7. displayed claim and its exact reproduction inputs.

Use established provenance concepts where practical instead of inventing incompatible semantics. W3C PROV models entities, activities and agents involved in producing information; the UN SEEA provides structured, spatially explicit approaches to ecosystem extent, condition, services and assets. Reuse and map to these domain standards rather than replacing them.

### C. From prediction-only to intervention-aware causal reasoning

Keep four artifact types distinct:

- **Observation:** a measured or reported value.
- **Hypothesis:** a proposed causal mechanism, with alternatives.
- **Forecast:** a prediction for a stated target, horizon and information cutoff.
- **Intervention estimate:** a conditional estimate of what may change under a specified action versus a stated comparator.

A running simulator proves only that the implementation ran. It does not validate its causal structure. Policy impacts need historical backtests, natural experiments, randomized evidence when feasible, or clearly labeled assumptions and sensitivity analysis when causal identification is not possible.

Every consequential model should disclose what would falsify it and which observations distinguish competing explanations.

### D. From aggregate optimization to plural, constrained objectives

Do not define “health” as a single scalar by default. Present a profile with:

- objective-specific metrics and directionality;
- uncertainty and observation coverage;
- trends and change rates;
- distribution by relevant groups and locations;
- critical floors, rights constraints and ecological limits;
- trade-offs and Pareto-efficient alternatives;
- sensitivity to weights and assumptions.

Some objectives may be non-substitutable constraints rather than tradable quantities. A monetary gain should not automatically offset a violation of a legal right, safety limit or critical ecological threshold. Decisions must identify who has authority to set those constraints and how they can be challenged.

If a composite score is offered for a narrow use case, it must be optional, versioned, have transparent weights, and include sensitivity analysis. Never publish a universal country, company, bank or ecosystem “health rank” as though it were a context-free fact.

### E. From expert-only models to socially robust model governance

Affected communities should be able to contest boundaries, missing variables, proxy measures and distributional assumptions before a model is treated as authoritative—not merely comment on the final chart. Participation should be resourced, accessible and ongoing, with protections against powerful stakeholders dominating.

A 2023 review of participatory simulation modelling and a March 2026 scoping review of public involvement in computational policy modelling both discuss the importance—and practical difficulty—of public involvement and power dynamics in modelling. That makes governance part of model quality, not optional outreach.

### F. From passive dashboard to active learning loop

A model should generate testable questions and identify the next observation with the highest expected decision value. The platform should say, for example, “the conclusion is sensitive to an uncertain feeder outage rate; measuring that rate is more valuable than adding another general indicator.”

Use pre-registered forecasts, decision logs and post-intervention evaluations. When outcomes differ from forecasts, preserve the failure, inspect the residuals, revise the model transparently, and score the revision on future data.

### G. From trusted network to adversarial evidence system

Cryptographic signatures and append-only logs establish who made a claim and whether it changed; they do **not** prove the claim is true. Multiple attestations may share a source or incentive. Track source dependence, conflicts of interest, coverage and correction history.

Threat-model metric gaming, synthetic documents, coordinated reporting, compromised sensors, stale data, selective disclosure, model extraction and attempts to use public health indicators for coercive individual profiling. Treat “unknown”, “contested” and “not independently verified” as valid states.

### H. From one timeline to regimes, ensembles and deep uncertainty

The future is not a deterministic slider. Support multiple scenarios, ensembles, shocks, delayed effects, tipping points and plausible structural changes. Distinguish parameter uncertainty, observation uncertainty, model uncertainty and radical uncertainty—cases where a useful probability distribution is not known.

A visually smooth trajectory is not evidence that the real system will follow it. For example, a fixed period-to-phase mapping in a toy secular-cycle visualization must be labeled illustrative until it passes out-of-sample validation against a clearly defined target. It must not be used as an empirical warning signal merely because it produces a phase label.

## 3. Target architecture: six planes and one map

1. **Observation plane** — typed records and source metadata; original values remain distinct from normalized values and derived estimates.
2. **Provenance plane** — source identities, content digests, licences, transforms, attestations, dependencies and correction history.
3. **Model plane** — versioned domain models with explicit scope, assumptions, training / calibration periods and independent validation.
4. **Causal and scenario plane** — dependency graphs, interventions, comparators, uncertainty propagation and ensembles.
5. **Objectives and governance plane** — multiple objectives, critical floors, stakeholder-defined trade-offs, challenge / appeal routes and decision ownership.
6. **Evaluation plane** — preregistered predictions, baseline comparisons, calibration, failure analysis, policy outcome tracking and model change history.
7. **Spatial interface** — a useful way to explore all of the above by place and scale, but not the authority that determines truth.

Cross-project roles:

- **Sol-Atlas:** geographic entities, exposure, assets, observations, provenance-aware maps and multi-scale exploration.
- **Symtropy:** bounded simulation interfaces, dependency networks, resource / stock-flow behavior, shock propagation and deterministic replay where the model permits it.
- **Symthaea:** candidate hypothesis generation, evidence synthesis, alternative explanations and uncertainty-aware narrative; generated claims remain traceable to sources and model outputs.
- **Mycelix:** signed receipts, consent and access policy, independent attestations, dispute workflows and public governance records.

These are proposed responsibilities. Each project needs explicit integration contracts; this table is not proof that integrations are implemented.

## 4. Minimum data contracts

Before building an all-purpose interface, standardize a small set of conceptual records. Exact Rust types and serialization must be designed and tested separately.

### Observation
- stable observation and metric identifiers;
- entity and optional geography identifiers;
- value **and** unit, with an explicit missing-value reason;
- observation time / period and publication time;
- measurement method, scope, coverage and licence;
- source URI and immutable content digest where legally possible;
- uncertainty representation with defined semantics;
- transformation and schema versions.

### Model run
- model ID / commit or artifact digest;
- parameter-set digest and configuration;
- inputs and their digests;
- explicit spatial and temporal scope;
- assumptions, excluded mechanisms and known limitations;
- random seed and environment where relevant;
- execution status, output digest and independent verification result.

**First Rust implementation (draft / not yet runtime-verified):** `sol-atlas-core::model_run` defines a schema-versioned `ModelRunManifest`, references immutable input/output artifacts using SHA-256 digests, records bounded scope, assumptions, run times and execution status, and declares bitwise, tolerance-bounded, or seeded-stochastic reproducibility. Structural tests cover malformed digests, duplicate artifact IDs, invalid periods, missing outputs, absent failure reasons, bad tolerances and missing stochastic seeds. The validator is not a model-validity certificate.

The current manifest stores digests but does not yet compute them, canonicalize and hash the manifest itself, verify referenced bytes, capture a complete runtime/container fingerprint, or independently replay a run. Those are separate follow-on tasks and must not be inferred from successful deserialization or structural validation.

The contract is intentionally a narrow Rust representation, not a replacement provenance standard. An interoperability adapter should map model runs to W3C PROV concepts (entities, activities and agents) and, where research packages are exported, consider RO-Crate for the bundle of data, code, configuration and metadata. Keep the domain contract stable while avoiding unnecessary invention of new provenance semantics.

### Claim / assessment
- exact observation or model-run inputs;
- the claim type: observation, inference, forecast, scenario or recommendation;
- uncertainty, evidence gaps, source-dependence notes and challenge status;
- evaluation target and due date for any forecast;
- named decision owner for any recommendation.

### Policy alternative
- objective vector and critical constraints;
- cost range, distributional effects, dependencies and reversibility;
- assumptions and comparison baseline;
- predicted effects with uncertainty and time horizon;
- evaluation plan, exit conditions and accountable owner.

Never let missing values silently become zero. Never conflate the uncertainty of a measurement with the model's confidence, or a cryptographic signature with truth.

## 5. Benchmark suite and initial release gates

These are proposed engineering gates, not current performance claims or universal scientific standards. Thresholds should be preregistered before evaluation and revised only with a documented justification.

| Benchmark | Initial gate | Evidence required |
|---|---|---|
| Provenance completeness | 100% of externally visible assessment claims carry a machine-verifiable path to their inputs, transformations and model version, or are explicitly marked untraceable and excluded from scored output. | Fixtures with missing links, stale sources, altered digests and licence restrictions. |
| Reproducibility | Same pinned inputs, configuration, model artifact and deterministic seed reproduce the same canonical output digest, or document the exact nondeterministic tolerance. | Independent replay and negative controls; do not treat queued CI as a pass. |
| Schema integrity | Invalid units, incompatible periods, out-of-scope geography, impossible sentinel values and silent missingness fail closed. | Property tests, fuzzing and adversarial fixtures. |
| Probabilistic calibration | Evaluate Brier score / log loss for probabilities and coverage of prediction intervals on held-out cases; report sample size and subgroup results. An initial 90% interval target of roughly 85–95% empirical coverage is only a screening gate with adequate samples, not a universal guarantee. | Rolling-origin evaluation, calibration plots, baseline comparison and confidence intervals. |
| Causal validity | No causal-effect claim is released as established solely from a correlational fit or a simulator run. Each claim names its identification strategy, comparator and assumptions. | Historical backtests, negative controls, sensitivity analysis and independent methodological review. |
| Baseline skill | Beat a simple, preregistered baseline on a held-out primary metric before claiming predictive improvement; otherwise publish the neutral or worse result. | Baseline definition, frozen split and reproducible scores. |
| Assumption sensitivity | Publish which assumptions can change the sign or preferred ordering of an outcome; unstable rankings are labeled unstable. | One-at-a-time and joint sensitivity analysis, alternative weights and leave-one-source-out tests. |
| Distributional fidelity | Aggregate outputs can be decomposed by relevant geography / cohort, with privacy-preserving suppression where needed; critical group harms cannot disappear in a weighted average. | Simpson's-paradox fixtures, small-cell privacy tests and floor-constraint checks. |
| Model comparison | Preserve disagreement among plausible models; explain which observations would discriminate between them. | Multi-model scenario comparisons and model-ensemble calibration. |
| Adversarial resilience | Metric manipulation, correlated attestations, duplicate sources, stale evidence and poisoned input cannot silently increase confidence. | Threat-driven red-team fixtures, duplicate-lineage test, corruption detection and correction drills. |
| Decision usefulness | Demonstrate improvement over existing practice on a predefined decision outcome, forecast, or avoidable loss. | Prospective or quasi-experimental evaluation, not user-satisfaction alone. |
| Public comprehension | Users can distinguish observation, causal claim, forecast, scenario and value judgment in comprehension tests. | Preregistered, diverse-user study, including uncertainty and opposing-view understanding. |

Do not make one aggregate benchmark score. A system could be well calibrated but socially harmful, or have excellent provenance but no useful predictive skill. Release status must be multidimensional.

## 6. First vertical slice: evidence-linked service resilience

Use an intentionally bounded infrastructure case—e.g. electricity-to-water-service dependencies in a city—before scoring countries or ranking institutions.

### Minimal scope

1. Ingest a small, publicly documented dataset with clear licensing and time coverage.
2. Preserve source values; normalize into explicitly typed observation records.
3. Represent assets, service areas and dependencies, with the basis for each link.
4. Add one simple baseline model and one explicit dependency / shock scenario.
5. Show current observation, historical trend, scenario, uncertainty, evidence gaps and source lineage separately.
6. Publish a replayable assessment manifest and tests for stale, missing, duplicated and adversarial input.
7. Have a domain reviewer and affected users challenge whether the model's boundary and outputs correspond to lived service conditions.

Do not call a scenario a forecast. Do not infer causality from a spatial overlay. Do not claim that a facility is operational because it appears on a map. Do not present a citywide aggregate without indicating service gaps across neighborhoods.

### First implementation sequence

- **P0 — contracts + threat model:** write the observation / model-run / claim contracts; list failure modes and out-of-scope claims.
- **P1 — evaluation harness:** fixtures, baseline metrics, calibration / sensitivity reports and negative controls.
- **P2 — one domain model:** transparent service reliability and dependency propagation with explicit assumptions.
- **P3 — map integration:** expose evidence lineage and model limitations directly from each visible output.
- **P4 — governance loop:** correction, challenge, model version history and forecast review.
- **P5 — cross-project adapter:** only after the first vertical slice passes, connect Symthaea / Mycelix / Symtropy through stable contracts.

## 7. Non-goals and safety boundaries

- No universal “objective health score” for nations, people, banks or ecosystems.
- No ranking of citizens by political beliefs, loyalty, inferred morality or social worth.
- No automated policy authority: the tool informs decisions; accountable humans and lawful institutions remain responsible.
- No causal language without a causal argument and uncertainty.
- No claim that decentralization, a signature, an LLM explanation, or model complexity guarantees truth.
- No hidden normative weights or irreversible decisions based on an opaque score.
- No production dependence on unverified scraped values where a claim is consequential.
- No release gates weakened to obtain a passing benchmark.

## 8. Research basis and references

- OECD, *Systemic Thinking for Policy Making*: emergence, nonlinearities, radical uncertainty, experiments and stakeholder engagement. https://www.oecd.org/en/publications/systemic-thinking-for-policy-making_879c4f7a-en/full-report.html
- UN SEEA, *Ecosystem Accounting*: spatial ecosystem extent and condition, ecosystem-service flows and links to economic activity. https://seea.un.org/en/methodology/ecosystem-accounting
- UN SEEA, *Central Framework*: environmental flows, asset stocks and environment-related economic activity. https://seea.un.org/en/methodology/seea-central-framework
- W3C, *PROV Primer* / *PROV-O*: interoperable provenance representations. https://www.w3.org/TR/prov-primer/ and https://www.w3.org/TR/prov-o/
- IMF, *Financial Soundness Indicators*: established indicators for financial-sector vulnerability analysis. https://data.imf.org/Datasets/FSIC
- IMF, *Interconnectedness and Contagion Analysis: A Practical Framework*: cross-sector and cross-border propagation in financial systems. https://www.imf.org/en/publications/wp/issues/2019/10/11/interconnectedness-and-contagion-analysis-a-practical-framework-48717
- *Conceptualising the value of simulation modelling for public engagement with policy* (2023): https://pmc.ncbi.nlm.nih.gov/articles/PMC10680332/
- *Towards socially robust policy modelling* (published 11 March 2026): public involvement, power dynamics and inclusive computational policy modelling. https://link.springer.com/article/10.1186/s12961-026-01473-6

## Decision requested from engineering

Treat this proposal as the architecture and benchmark contract for a staged prototype. The next code change should be a narrow, testable observation / provenance type or evaluation fixture—not an all-encompassing score or a large speculative agent simulation.
