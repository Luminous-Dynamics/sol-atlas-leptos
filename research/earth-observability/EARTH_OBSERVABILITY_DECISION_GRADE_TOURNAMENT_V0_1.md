# EARTH-OBSERVABILITY-DECISION-GRADE-001
## Sensor observability envelopes and equal-budget investigation tournament v0.1

**Parent:** #761 — EARTH-OBSERVABILITY-QUAL-001  
**Status:** research protocol; executable implementation not yet qualified  
**Date:** 2026-11-13  
**Owners:** Sol-Atlas (evidence and benchmark orchestration), Symtropy (sensor/physical qualification), Symthaea (decision-aware investigation planning), Mycelix (rights/custody)  
**Claim ceiling:** research design. The targets below are proposed acceptance rules, not achieved performance. No field measurement, instrument, geological interpretation, survey or engineering decision is qualified by this document.

---

## 1. The next paradigm shift

The first increment establishes the evidence contract: source identity, coordinate/datum semantics, non-detection states, provenance, split isolation, rights scope, and a synthetic transfer benchmark.

The next increment should change the unit of optimisation. Do not optimise for map coverage, point count, model accuracy in isolation, or information gain. Optimise for a **defensible decision under an explicit investigation budget**.

A usable Earth-observability system needs to answer four questions:

1. What physical target could this measurement detect, at what depth, size, contrast and environmental conditions?
2. Where is the measurement informative, and where is the sensor blind or out of its validated domain?
3. Which next eligible measurement most improves a named decision after its cost, delay, access burden and disturbance?
4. What independent observation would falsify or validate the resulting interpretation, and which downstream claims must be re-evaluated when it arrives?

This yields a three-part operating loop:

**qualify sensor → choose investigation → score against independent revelation**

The evidence graph remains the authoritative record. Model estimates remain revisable claims. Symthaea may propose hypotheses and candidate measurements; it cannot authorize acquisition or field work. Symtropy qualifies the sensor and physical assumptions. Mycelix authenticates rights, custody and reviewers. Sol-Atlas retains the spatial/time support, evidence lineage and benchmark receipts.

## 2. Sensor observability envelopes (SOE)

### 2.1 Replace binary coverage with conditional detection

A survey footprint only says where an instrument was deployed. It does not say what the instrument could reliably observe there.

For each sensor configuration and target class, define an observability envelope with the following independent dimensions:

- modality and instrument identity, model, firmware and sensor configuration;
- acquisition geometry, spacing, orientation, coupling and speed;
- target type, minimum feature size, depth, orientation and physical contrast;
- host material / stratigraphy, conductivity or permittivity where relevant, moisture and temperature;
- noise floor, calibration state, processing pipeline and declared processing parameters;
- spatial and temporal support, environmental domain, known exclusions and detection-limit definition.

For target characteristics x, environment e and acquisition configuration c, estimate a detection curve:

**POD(x, e, c) = P(detection | target present, x, e, c)**

This is a conditional claim, not a universal sensor score. Keep target presence, detector output, localization and geological interpretation separate. A sensor may detect an anomaly without uniquely identifying its cause or material.

The SOE must expose the validated domain, the detection probability estimate and interval, false-alarm rate, localization uncertainty, and a reasoned state for regions not supported by the data. The minimum set of result states is:

- **Detected:** qualified detection evidence exists; cause/material may remain ambiguous.
- **No detection within envelope:** target class/configuration/domain was qualified and no detection occurred.
- **Below detection limit:** target could be present but the size/contrast/depth is below the validated capability.
- **Unobserved:** acquisition or footprint does not cover the target support.
- **Out of domain:** environmental or configuration conditions fall outside validation.
- **Unqualified:** calibration, metadata, rights or source integrity is inadequate.
- **Conflicted:** admissible evidence supports incompatible interpretations.

Only “no detection within envelope” may contribute evidence of absence, and only for the named target class and support. It is never a universal assertion that the ground is empty.

### 2.2 Detection qualification experiment

Use a physically meaningful reference target or independently established ground truth. Do not calibrate and score on the same targets.

Split by site, survey line / acquisition campaign, sensor configuration and target instance. Adjacent traces, repeat scans of the same object and neighbouring pixels from one acquisition must stay in one partition. Where feasible, use a calibration set, a locked blind test set and a prospective external site.

Report at least:

- probability of detection with confidence intervals by target size / depth / contrast / environment;
- false alarms per surveyed area, route length or observation time;
- localization error P50 / P95 and spatial uncertainty coverage;
- minimum detectable target and change under the declared operating conditions;
- calibration curve / Brier score and, for continuous score distributions, proper scoring rules;
- coverage, blind-zone fraction, abstention rate and out-of-domain rate;
- detection latency, acquisition time, uptime and cost;
- site-clustered uncertainty intervals, not iid point bootstrap intervals.

Do not collapse different physical modalities into one leaderboard. GPR, ERT, EM, seismic / surface-wave, microgravity, borehole logging and DAS have different forward physics and target sensitivities. Compare them only on the same target definition, support, reference truth and decision.

**Proposed acceptance gate:** every “absence” claim is traceable to a target-specific envelope and a preregistered detection requirement. The acceptable POD lower confidence bound is selected per consequence class before the test; it is not one arbitrary threshold for every target. An unobserved, below-limit, unqualified or out-of-domain result must never be converted into absence. The frozen negative-control suite must contain zero such invalid conversions.

## 3. Equal-budget decision tournament

### 3.1 Compare actions, not just interpolation methods

Define each candidate investigation action as:

**A = (location/support, modality, instrument configuration, sampling design, time window, expected cost, access/permit constraints, disturbance)**

The candidate action catalogue may include borehole, CPT, GPR, ERT, EM, seismic/surface-wave, borehole geophysical log, groundwater monitoring, DAS interrogations, repeat survey or excavation/as-built observations.

First apply hard eligibility gates: lawful access, permits, method authorization, source rights, custodial review, health/safety and environmental constraints. Ineligible actions are not given a large penalty and left in the optimization; they are removed from the feasible action set with a reason receipt. Non-compensable legal, safety, cultural and ecological obligations are constraints, not monetized utility terms.

Among eligible actions, score the expected decision improvement after accounting for measurement likelihood, correlated errors, model uncertainty, action costs, delay and disturbance. Distinguish information gain from decision value.

For current evidence D, action A, latent physical state θ, decision d and declared loss L:

**EVSI(A | D) = min_d E[L(d, θ) | D] − E_Y[min_d E[L(d, θ) | D, Y]]**

**ENBS(A | D) = EVSI(A | D) − C(A)**

Here C(A) must have a declared, compatible cost boundary. When time, access burden, ecological disturbance or cultural impacts cannot be defensibly priced, report them as separate dimensions or treat them as hard constraints. Do not hide them in a single opaque score.

### 3.2 Required tournament baselines

For each frozen case, compare:

A. Existing practice / documented expert plan, if available.  
B. Fixed-grid or fixed-spacing sampling.  
C. Random eligible sampling and cost-only prioritisation.  
D. Nearest / linear / IDW and applicable kriging or spatial-random-field baselines.  
E. Uncertainty- or entropy-greedy sampling.  
F. Mechanism- and decision-specific EVSI / ENBS planner.  
G. Multimodal posterior update with and without each modality (ablation).  
H. Oracle chosen with hidden truth, upper bound only and never deployable.

For a fair result, freeze the action catalogue, total monetary and crew-hour budgets, maximum investigation duration, access/permit constraints, disturbance limits, measurement quality requirements, target truth, priors, likelihoods, model versions and evaluation code before the blind run. Compare feasible plans at equal cost **and** publish Pareto frontiers for cost, delay, disturbance and decision quality. A planner cannot win by acquiring more data, spending more, quietly omitting difficult areas or relaxing the eligibility constraints.

### 3.3 Required decision outputs

Report post-investigation expected loss, realized loss against withheld truth, decision regret, EVSI and ENBS calibration, total cost, elapsed time, access burden, disturbance, samples by modality, spatial coverage and abstentions. Also report plan stability to reasonable changes in prior, cost, consequence and likelihood assumptions.

A good planner must know when a new observation will not change the selected decision. It must also recognize when high information gain does not have decision value.

**Stretch hypothesis retained from v0.1:** at equal investigation budget, at least 20% lower out-of-sample expected decision loss; alternatively, at least 20% lower survey expenditure at equivalent preregistered decision quality. First measure variance across independent cases and expert baselines, then freeze an effect threshold and sample size. No result may be claimed from synthetic fixtures as field savings.

## 4. The falsification ladder: from synthetic to prospective

Use separate evidence tiers with separate headlines.

| Tier | Evidence | Claims allowed |
|---|---|---|
| T0 — contract | schemas, deterministic fixtures, negative controls | parser / contract / reproducibility behavior |
| T1 — analytic synthetic | known geometry, target truth and generated measurement process | algorithm correctness under declared model assumptions |
| T2 — physics synthetic | modality-specific forward model with noise, attenuation, coupling and inversion | model-based performance within simulated assumptions |
| T3 — retrospective physical | permissioned real data with independent blind truth | site/campaign-specific empirical performance |
| T4 — prospective physical | preregistered new acquisition with independent survey/metrology and competent review | prospective generalization for the validated envelope |

T1 is not T2. A scalar proxy is not a GPR, ERT or seismic forward model. T2 is not T3. A simulation does not establish calibration against the physical environment. T3 is not T4. Retrospective tuning may exploit knowledge of the field site.

For synthetic physics, Symtropy owns and documents the forward assumptions and model qualification. Each model must include negative controls and a model discrepancy term or explicit caveat. The benchmark should include cases where two different subsurface structures produce near-identical observations, demonstrating inverse non-uniqueness rather than pretending every inversion has a unique answer.

For physical validation, independent truth may include surveyed excavation exposures, verified borehole core / logs, independently surveyed utility or structure geometry, repeat control measurements, or qualified instrumentation. Its uncertainty and support must be recorded. The observation used to fit, register, calibrate or select a model cannot also be its independent test truth.

## 5. Calibration, selective prediction and spatial uncertainty

A calibrated model is not merely one that reports a probability. Its uncertainty should correspond to empirical errors on independent observations within each declared domain.

Required reporting:

- 50%, 80%, 90% and 95% prediction-interval empirical coverage and mean width;
- CRPS / weighted interval score where predictive distributions exist;
- Brier score / log loss and reliability curves for categorical outcomes;
- selective risk-versus-coverage curves under abstention;
- uncertainty and error stratified by site, geological domain, depth, sensor, campaign and distance from observations;
- covariance and spatial residual dependence; uncertainty from measurement, transform, model and geology kept separate where identifiable;
- performance when one modality is missing, biased, stale, miscalibrated or shifted.

The 95% interval should cover about 95% of independent observations in the specific evaluation population, with uncertainty intervals around empirical coverage. Pooled coverage alone is insufficient if a particular material or depth stratum is miscalibrated. A model that retains narrow confidence under domain shift fails even if aggregate RMSE looks good.

For sparse data, use leave-one-site / domain / instrument / campaign out and a prospective blind site. Do not make random point splits across a single acquisition. Report each fold and worst-performing domain, not only a pooled mean.

## 6. New benchmark corpus v0.2

Extend the 18 v0.1 challenge definitions with deterministic scenario generators and traceable Rust assertions for:

| New case | Scenario | Required disposition / score |
|---|---|---|
| SOE-POD-SIZE-001 | POD varies with target size and depth | target-conditional curve; no universal detection claim |
| SOE-MOISTURE-SHIFT-001 | same GPR setup under dry sand and conductive wet clay | detect calibration shift; abstain / qualify separate envelope |
| SOE-ERT-EQUIVALENCE-001 | two distinct models yield similar ERT responses | retain non-uniqueness / competing interpretations |
| SOE-COUPLING-DAS-001 | ducted vs direct-buried fibre | separate coupling domains and POD curves |
| SOE-DRIFT-CHANGE-001 | correlated registration bias across epochs | no confirmed movement below propagated detection limit |
| SOE-OOD-INSTRUMENT-001 | unseen sensor configuration | OOD / unqualified until bridged by calibration |
| EVSI-MULTIMODAL-001 | candidate location differs by measurement modality | mechanism-aware plan under equal budgets |
| EVSI-REGRET-001 | entropy-greedy plan has high information but unchanged decision | lower decision value than true EVSI plan |
| EVSI-COST-SHOCK-001 | cost and access constraints change | stable feasible plan or explicit infeasibility |
| TRUTH-SELECTION-001 | truth is sampled preferentially near easy detections | detect selection bias; no unbiased performance claim |
| TEMPORAL-RETRACTION-001 | source owner retracts/corrects upstream evidence | immutable history plus stale descendants |
| RIGHTS-EGRESS-001 | source-local adapter tries to emit restricted coordinates | deny egress and emit an accountable denial receipt |

These are additional planned test definitions. They are not generated signal datasets, field campaigns or completed tests.

## 7. Industry comparison: where to integrate and where to differentiate

| Capability | Established strength / current comparator | Sol-Atlas strategy |
|---|---|---|
| Geotechnical factual data management | Bentley OpenGround supports field/lab/project workflows, AGS/DIGGS, REST API and connections to design/model tools | integrate and verify provenance/semantics; do not rebuild its project database |
| Implicit 3D geological modelling | Seequent Leapfrog Works provides dynamic modelling, borings, geophysics extensions, GIS and engineering integration | compare on an identical data pack if licensed; do not claim superior implicit modelling without tests |
| GIS / point cloud / survey | mature systems handle raster/vector, georeferencing, point clouds, adjustment and visualization | reuse standards and specialist tools; qualify coordinate chain, uncertainty and change |
| Frontier multimodal inference | probabilistic borehole + surface-wave fusion and neural 3D geological modelling already exist | compete on held-out geography/instrument, detection envelopes, calibration and decision outcomes |
| Site investigation planning | spatial random fields + Bayesian networks + EVSI/ENBS are published approaches | reproduce first, then compare mechanism-specific actions on equal budget and measured regret |
| TBM continual learning | ring-sequence Bayesian continual learning has been demonstrated on a real twin-line tunnel | compare transfer, new-state detection, forgetting, calibration and independent lithological truth—not accuracy alone |

The product strategy is **interoperability before replacement**. Make Sol-Atlas the trust and decision layer above specialist acquisition, GIS, geological modelling and engineering tools. Vendor benchmarks are NOT_RUN until an identical licensed workflow can be tested; a published product capability is not proof that the competitor passes or fails our test.

## 8. Standards strategy

Use established encodings and profiles; keep the internal evidence/claim model independent of any one serialization.

- OGC MUDDI 1.0 — underground infrastructure and related subsurface/built-space information.
- GeoSciML 4.1 — geological map features, geological time, boreholes and analytical metadata.
- GWML2 2.2.1 — groundwater information exchange.
- OGC SensorThings API — observations and sensing systems; Part 2 Tasking only when tasking workflows are explicitly in scope.
- OGC API – Connected Systems 1.0 — connected sensor/platform systems, deployments and operations.
- OGC API – Features — discrete feature access.
- OGC API – EDR 1.2 — spatiotemporal environmental retrieval; published 8 September 2026.
- STAC — discoverable spatiotemporal assets; OGC 3D Tiles — 3D content streaming.

Conformance is a separate benchmark lane: exact version, conformance class, test suite, test date, failed requirements, extensions and known non-mappings must be reported. A parser for a small profile is not full standard conformance. The EDR 1.2 update is a timely adapter target, not evidence that Sol-Atlas already supports it.

## 9. Evidence receipts and reproducibility

Every benchmark run should emit one immutable receipt binding:

- benchmark and case-manifest digest; case generator version and seed;
- exact source / asset identifiers, source revision and source-content digest where permitted;
- rights grant reference, subject, purpose and scope (or a denial); no source data may enter CI unless explicitly authorized;
- coordinate frame, axis order, horizontal/vertical datum, epoch, transform operation and grid-resource digests;
- instrument / configuration / calibration / processing identities;
- split assignments, independent truth source and truth uncertainty;
- model, code, configuration, dependency lock and runtime identity;
- action catalogue, constraints, cost model, decision loss and budget;
- raw metrics, abstentions, failures, subgroup results and limitations.

A receipt verifies reproducibility and claim lineage; it does not turn a synthetic result into physical truth or a parser pass into survey qualification. Historical receipts remain immutable. New evidence creates new versions and marks affected descendants stale or requiring scoped revalidation.

## 10. First implementation sequence

**P0 — schema and deterministic case fixtures (Sol-Atlas):**
- Add typed SensorObservabilityEnvelope, DetectionObservation, DetectionEvaluationSplit and InvestigationAction structures under sol-atlas-core.
- Add exact support / sensor / condition / target bindings. Reject missing units, missing calibration IDs, unknown configuration, non-finite POD inputs, invalid interval bounds and train/test target identity overlap.
- Create the 12 v0.2 deterministic case definitions and fixtures. Keep synthetic measurement data explicitly tagged; no real coordinates or private data in CI.
- Add negative controls for all invalid absence promotions, rights/evidence egress, and split leakage.

**P0 — detection scoring harness (Symtropy + Sol-Atlas):**
- Add a headless, offline Rust runner that consumes frozen fixtures and emits POD bins/curves, false alarms, localization error, detection-limit and selective-risk receipts.
- Support cluster-aware resampling by site/acquisition/target, or mark confidence intervals NOT_RUN until that method is implemented.
- The first version should test contract and metric implementation against analytic synthetic truth. Do not simulate a sensor as a scalar noise perturbation and call it physics.

**P1 — equal-budget experiment runner (Symthaea + Sol-Atlas):**
- Define a common action schema and feasible-action filter.
- Reproduce fixed-grid, random, cost-only and entropy-greedy planners before adding EVSI/ENBS.
- Use one frozen loss function and cost envelope per case, then compare realized regret against blind truth. Report Pareto frontiers where objectives are not commensurable.
- Add tests showing an infeasible, unauthorized high-information action never enters the recommendation set.

**P1 — reproduce published methods (Symtropy):**
- Reproduce the 2026 surface-wave/borehole reliability-weighted fusion paper and a simple neural or implicit-model comparator where data access allows.
- Reproduce the Fartosy et al. EVSI/ENBS slope and settlement example, including the mechanism-dependent sample-location result.
- For the TBM continual-learning paper, compare known-state accuracy, cross-line transfer, unknown-state detection, calibration and forgetting. Treat TBM proxy labels as proxy truth, not lithology.

**P2 — permissioned physical pilot (partner + Symtropy):**
- First qualify the Johannesburg aerial data files, survey report, coordinate reference, vertical datum, calibration and reuse rights. The City's published 2023 programme description is an acquisition lead, not a qualified dataset.
- Select one bounded site and a named engineering decision with independent reference truth and authorized access.
- Freeze the blind protocol before accessing evaluation labels. Acquire no survey or borehole data without the relevant authority and source-owner permission.

## 11. Competitive targets and claim discipline

| Dimension | Proposed target / gate | Evidence tier needed |
|---|---|---|
| Invalid absence | 0 negative-control promotions from unobserved, below-limit, OOD or unqualified to absent | T0/T1 |
| Provenance / spatial semantics | 100% of admitted claims bind source, support, time, frame and transform identity; unresolved fields remain explicit | T0/T1, then T3 |
| Calibration | nominal 90%/95% intervals achieve coverage consistent with nominal on independent, stratified holdouts; publish width and uncertainty on coverage | T2/T3 |
| Detection | preregister target-specific POD lower bound and false-alarm ceiling; no global modality leaderboard | T2/T3 |
| Transfer | leave-site/domain/instrument/campaign-out; no confidence collapse hidden by aggregate metrics | T2/T3/T4 |
| Decision value | beat fixed-grid, documented expert and entropy-greedy baselines on equal feasible budget; report regret and uncertainty | T1 first, T3/T4 for field claim |
| Stretch advantage | 20% lower expected decision loss at equal budget, or 20% lower cost at equivalent decision quality | independent T3/T4 |
| Rights | zero unauthorized access / process / derive / source egress in frozen negative controls | T0 and adversarial integration tests |
| Revalidation | every affected downstream artifact is marked stale/revalidation-required when relevant inputs change; no historical receipt mutation | T0/T1 |

A result should be reported as implemented, structurally checked, synthetically executed, CI tested on exact head, externally validated, or physically qualified. Do not compress those states into “done”.

## 12. Research references and comparator notes

- Dengyi et al. (2026), *Probabilistic landslide stratigraphic modelling by uncertainty fusion of surface wave inversion and borehole data*, Engineering Geology 109128, online 7 Oct 2026. DOI: https://doi.org/10.1016/j.enggeo.2026.109128. The authors report about 26% lower locally propagated false-positive area than uniform weighting under their adopted configuration. The paper states the uncertainty is conditional/incomplete, does not statistically separate geological spatial variability from measurement/model error, and its boreholes used in fusion are not independent validation truth. Supporting data/code are available from the authors on request.
- Zhang et al. (2026), *U-shaped multi-scale feature fusion network for three-dimensional geological modelling and uncertainty assessment*, Engineering Geology 108727, 21 May 2026. DOI: https://doi.org/10.1016/j.enggeo.2026.108727. The field-case study integrates boreholes and geophysics, identifies five stratigraphic units and karst cavities, and reports 74.68% spatial extrapolation accuracy against validation boreholes. Treat this as a case-specific reference, not a general SOTA ceiling.
- Fartosy et al. (2026), *Optimizing Geotechnical Site Characterization: A Value of Information Approach Using Coupled Spatial Random Fields and Bayesian Networks*, Periodica Polytechnica Civil Engineering 70(2), 598–611, published online 2 Jun 2026. DOI: https://doi.org/10.3311/PPci.43308. The authors report up to 143% improvement in economic efficiency in two case studies; their optimal sample location changes with the decision mechanism. Reproduce assumptions and cost model before comparison.
- *Bayesian continual learning and dynamic uncertainty quantification framework for TBM geological prediction in non-stationary environments*, International Journal of Rock Mechanics and Mining Sciences 205 (Sep 2026), 106610. DOI: https://doi.org/10.1016/j.ijrmms.2026.106610. On real ring-level data from a twin-line Shenzhen intercity tunnel, the study reports 84.76% static accuracy for known states, 45.40% direct cross-line transfer, and 79.9% cross-line accuracy with experience replay. The paper explicitly describes geology labels as operational TBM–ground interaction proxy states, not strict lithological truth.
- OGC API – EDR 1.2, OGC document 19-086r9, published 8 Sep 2026: https://github.com/opengeospatial/ogcapi-environmental-data-retrieval
- OGC GeoSciML 4.1: https://www.ogc.org/standards/geosciml/
- OGC GWML2 2.2.1: https://www.ogc.org/standards/gwml2/
- Bentley OpenGround: https://www.bentley.com/products/openground — geotechnical data management, configurable project records, AGS/DIGGS exchange, API and connected workflows.
- Seequent Leapfrog Works: https://www.seequent.com/products-solutions/leapfrog-works/ — dynamic implicit geological modelling, drilling/site-investigation data, geophysics extension and engineering / groundwater integration.

The 2026 research examples establish that many of the individual ideas are active research, not unclaimed novelty. Sol-Atlas's defensible differentiator is to test whether a single vendor-neutral evidence and qualification layer can connect them while enforcing observability limits, independent truth, rights, calibration, equal-budget decision value and reproducible revalidation.
