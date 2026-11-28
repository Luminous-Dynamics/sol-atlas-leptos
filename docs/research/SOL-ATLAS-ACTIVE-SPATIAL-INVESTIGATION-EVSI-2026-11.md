# Sol-Atlas active spatial investigation: decision-value research and benchmark plan

**Date:** November 2026  
**Status:** research / synthetic reference benchmark. No field performance or SOTA advantage is claimed.

## Executive thesis

Sol-Atlas should not optimize for the most uncertain map cell, the most informative sensor reading, or the most complete digital twin in isolation. It should optimize for **the next authorized observation that is most likely to improve a real decision, net of acquisition cost and risk**.

This is a transition from passive spatial representation to active, evidence-governed investigation:

`observe → update competing hypotheses → evaluate decisions → value candidate observations → acquire evidence → independently verify → adjudicate → preserve the decision trail`

The system should optimize what to inspect next while making clear what it does not know, what evidence is allowed to be used, and who retains decision authority.

## 1. Why this is a meaningful paradigm shift

### 1.1 From entropy minimization to decision value

Entropy and uncertainty reduction are useful diagnostics, but neither directly measures whether a decision improves. Lark, Chagumaira and Milne argue that spatial uncertainty measures should engage with end-user decision processes and formal decision analysis; they identify value-of-information theory as a promising basis for rational data acquisition ([Spatial Statistics, 2022](https://doi.org/10.1016/j.spasta.2022.100619)). USGS/NIST work similarly frames geospatial information as something with monetary and non-monetary acquisition and use costs, whose value is realized through decision pathways ([Glynn et al., 2022](https://doi.org/10.3389/fenvs.2022.805214)).

The benchmark therefore distinguishes:

- **Expected information gain (EIG):** expected reduction in uncertainty over hypotheses.
- **Expected value of sample information (EVSI):** expected improvement in the best decision after an observation is acquired.
- **Net EVSI:** EVSI minus the declared cost of collecting, processing and using the observation.

The objective is not to minimize entropy. It is to select the affordable evidence portfolio with the greatest expected improvement to the decision.

A direct spatial-sampling predecessor already used expected value of information and misclassification costs to choose sample locations, comparing it with random and minimum-kriging-variance sampling ([Where and When Should Sensors Move?](https://doi.org/10.3390/s121216274)). Sol-Atlas should generalize this decision-centric logic across survey methods, geospatial hypotheses, provenance, rights and human decisions—not claim to have invented EVSI.

### 1.2 From a finished map to an active investigation loop

A static twin records what is believed about a site. An active investigation system decides which missing observation could resolve an important uncertainty, compares acquisition methods and costs, and revises its priorities as evidence arrives.

This changes the planning unit from "where is data missing?" to "which observation can change which authorized decision, under what uncertainty and cost?"

### 1.3 From one reconstruction to explicit competing hypotheses

The system should retain rival explanations—original fabric, later repair, intervention, damage, reconstruction—until evidence and a recorded decision justify a selection. A model's top class is not an adjudication. The benchmark should penalize unsupported hypothesis collapse and report the provenance of every selection.

### 1.4 From confidence scores to calibrated uncertainty and abstention

Every candidate observation model needs calibration on site-level holdouts. Report interval coverage and interval width, reliability, selective risk as the model abstains, and performance on the worst sensor/material/occlusion stratum. An uncalibrated 0.95 confidence score is not a 95% guarantee.

### 1.5 From metadata labels to rights-aware capability security

The planner must not assume that a record being discoverable means its contents can be retrieved, disclosed, redistributed, used commercially or used for model training. Rights dimensions are separate capabilities. Evaluation should test non-escalation and non-interference, not merely the presence of a rights field.

### 1.6 From file conversion to semantic accountability

A successful import/export is not proof of interoperability. Each adapter hop must emit a field-disposition ledger: every source field is preserved, transformed with an explicit mapping, omitted with a reason, blocked with a reason, or not yet assessed. A route cannot be called complete while fields remain unassessed or source/target bytes are not digest-bound.

## 2. Synthetic reference result now in the branch

The new Rust reference crate is intended to be a small exact oracle for a finite EVSI problem. It uses a synthetic heritage decision with three hypotheses, two possible conservation actions, four candidate surveys, binary outcomes and a three-unit effort budget.

| Measure | Synthetic oracle |
|---|---:|
| Prior best action | Preserve in situ |
| Prior expected utility | 72.000 |
| Selected survey | Stratigraphic microsection |
| Expected post-survey utility | 79.125 |
| Gross EVSI | 7.125 |
| Declared utility cost | 2.500 |
| Net EVSI | 4.625 |
| Highest-EIG candidate | Ornamental subtype catalog |
| Highest-EIG information gain | 0.653 bits (approximately) |
| Highest-EIG survey's gross EVSI | 0.000 |

This is a deliberately constructed counterexample: ornamental subtype catalog has the highest expected information gain, but does not change the optimal action. A stratigraphic microsection has lower entropy reduction but positive decision value. The example is a correctness fixture, not a real-world model or empirical SOTA comparison.

The evaluator assumes conditional independence of survey outcomes given the hypothesis, supplied likelihoods and priors, binary outcomes, supplied utilities and costs, and exhaustive enumeration of at most 16 survey candidates. It is not a production planner. Likelihood calibration, correlated sensors, adaptive sequential surveys and real stakeholder utility elicitation remain open.

## 3. Industry and SOTA comparison

### 3.1 Geospatial interoperability

OGC JSON-FG 1.0.0 extends GeoJSON to address additional geometry, CRS, temporal and feature-type use cases ([OGC announcement, 21 May 2026](https://www.ogc.org/announcement/ogc-releases-the-features-and-geometries-json-json-fg-standard/)). It should be used as a standards-based exchange representation, not treated as an evidence, rights or decision model by itself.

The current pygeoapi 0.24.0 documentation is dated 28 July 2026 and describes a broad OGC API implementation including Features and Records ([official docs](https://docs.pygeoapi.io/en/stable/)). GeoServer's implementation status is capability-specific: core and CRS support should be distinguished from filtering, CQL2, transactional operations and schema support, which vary by release and test status ([GeoServer implementation status](https://docs.geoserver.org/latest/en/user/services/features/status/)). Compare exact versions and capabilities—not product names or one global "OGC compliant" label.

GeoParquet 2.0 should be evaluated as a columnar analytical storage/exchange route, not as a semantic governance layer. GDAL/OGR remains a practical conversion baseline. These routes need actual executed adapter tests; listing them in a plan does not establish interoperability.

### 3.2 Heritage reconstruction and metrology

A 2026 review of 120 Scan-to-HBIM papers identifies persistent inconsistencies across RMSE, IoU and level-of-detail reporting, alongside hybrid human/algorithm workflows and the need for transparent benchmarking ([Waqar, Automation in Construction, 2026](https://doi.org/10.1016/j.autcon.2026.106848)). A 2026 heritage digital-twin review of 204 studies identifies interoperability, AI validation and long-term preservation as continuing problems ([Tootoonchi & Hess, 2026](https://doi.org/10.5194/isprs-annals-XI-4-2026-435-2026)). A recent metrology study grounds its approach in GUM and measurement standards, reinforcing that geometric accuracy must be accompanied by uncertainty budgets, not just a nominal RMSE ([Moyano et al., Measurement, 2026](https://doi.org/10.1016/j.measurement.2026.122271)).

The competitive gap is not simply another 3D reconstruction model. Sol-Atlas can aim to connect geometry, uncertainty, source evidence, contradictory interpretations, rights, provenance and the value of the next observation in one testable workflow.

### 3.3 Active learning and sensor placement

Do not assume that entropy-based active learning is obsolete. A 2025 evaluation found entropy performed better than other acquisition functions in 72.5% of acquisition steps across four datasets and warned that no single-model method decisively dominated in general settings ([Gashi et al., Pattern Recognition Letters, 2025](https://www.sciencedirect.com/science/article/pii/S0167865525002697)). The correct response is stronger domain-specific evaluation, not a blanket claim that EVSI always wins.

A 2026 systematic mapping review of 63 urban sensor-placement studies reports only three operational field evaluations and four documented public-code links; it also says the heterogeneous evidence does not support a universal algorithm ranking ([Kakoulli & Chatzifristofis, Electronics, 2026](https://doi.org/10.3390/electronics15194545)). This is an opportunity for a reproducible benchmark ladder with operational evidence, not proof that any specific optimizer is superior.

## 4. Benchmark suite and qualification rules

The benchmark should be multi-axis. Do not combine all measures into one weighted score: rights breaches, fabricated lineage and privacy leakage must be hard-gate failures.

| Track | Metrics | Baselines |
|---|---|---|
| Decision value | Net EVSI, expected regret, decision loss, cost per utility gain | Human experts, fixed priority, random, EIG, entropy, clairvoyant upper bound |
| Survey efficiency | Survey effort, travel/time cost, repeat observations, yield per unit cost | Grid, nearest-uncertain, max entropy, heuristic expert plan |
| Calibration | Brier/log score, reliability, interval coverage and width, selective risk | Empirical frequency, survey/metrology baseline, calibrated model variants |
| Geometry/metrology | Point-to-plane, cloud-to-cloud, control-point error, dimension bias, uncertainty coverage | Independent TLS/total station/GNSS or qualified reference survey |
| Evidence quality | Per-relation precision/recall, missing-link rate, duplicate/conflict detection | Human-adjudicated evidence graph and rule baseline |
| Hypothesis handling | Unsupported selection rate, alternative retention, adjudication trace completeness | Single-label workflow and human-only workflow |
| Rights/privacy | Rights escalations, restricted-record observability, public projection non-interference | Deliberately mutated restricted records and negative controls |
| Interoperability | Required semantic retention, explicit loss disposition, geometry/CRS equivalence after each hop | JSON-FG, pygeoapi, GeoServer, GDAL/OGR, GeoParquet |
| Reproducibility | Byte digests, canonical semantic projection, deterministic replay | Independent runner / evaluator |
| Operations | p50/p95 latency, throughput, RAM, storage, cold start and recovery | Matched workloads and hardware |

### Benchmark protocol

1. Hold out complete sites, source collections and acquisition campaigns. Never split scans from one site across training and test partitions.
2. Pre-register hypothesis definitions, decision actions, utility elicitation, likelihood estimation, outcomes, costs and metrics.
3. Evaluate candidate methods with the same observations, time/effort budgets, permissions, data access and reference labels.
4. Compare both decision quality and calibration. Report mean, confidence intervals, per-site outcomes and worst-stratum results.
5. Keep a sealed holdout and independent evaluator; publish exact software, configuration, CRS database and dataset digests.
6. Include negative controls: misleadingly high entropy, sensor drift, missing data, contradictory evidence, restricted records, correlated surveys, rare classes, out-of-distribution materials and no-decision-value cases.

### Initial targets (proposed, not measured)

- **Rights/privacy hard gate:** zero unauthorized disclosure, zero rights escalations and zero public-output change under restricted-only input mutations.
- **Interop hard gate:** 100% of required fields either semantically preserved or explicitly transformed with a reviewed mapping; zero silent losses. Every other disposition must carry a reason.
- **Reproducibility:** identical canonical semantic projection and plan for three repeated runs with identical frozen inputs and environment.
- **Evidence integrity:** zero contradiction/context promotion; report precision/recall per relation class with confidence intervals.
- **Calibration:** nominal 95% intervals should have empirical coverage statistically compatible with 95% on independent site holdouts; report width and worst-stratum coverage. Pre-register acceptable binomial confidence bounds rather than selecting a tolerance post hoc.
- **Decision improvement:** aspirational initial target of at least 20% lower expected regret or 20% lower acquisition effort at matched regret versus the best non-EVSI baseline, with a positive confidence interval across independent sites. Do not claim this target is achieved by the synthetic fixture.
- **Operational scale:** measure p50/p95 latency, throughput, RAM and storage across small / medium / large sites under fixed hardware; do not set a universal runtime threshold before the reference workload is measured.

## 5. How Sol-Atlas can move beyond current practice

1. **Decision-conditioned sensing:** use decision utility and acquisition cost to rank where/what to survey, instead of maximizing coverage or entropy alone.
2. **Multi-stage adaptive plans:** update the next observation after each survey result. The planner should explain what outcome would change the plan.
3. **Robust EVSI:** model uncertain priors, likelihoods, utilities and costs; report sensitivity ranges and robust/least-regret plans, not one brittle optimum.
4. **Correlation-aware evidence:** handle redundant sensors and correlated sources explicitly so the planner does not pay twice for the same underlying information.
5. **Spatially aware experiment design:** value both the location and the modality—TLS, photogrammetry, multispectral, archive research, intrusive survey—subject to permissions, access, safety and conservation constraints.
6. **Human-authorized adjudication:** keep decisions and authority separate from model recommendations; retain rejected and unresolved hypotheses with their reasons.
7. **Evidence-first digital thread:** bind decisions and geometry to source, method, uncertainty, transformations and the exact data that influenced the result.
8. **Operational external validation:** run on real sites with independent control measurements, community participation and repeat visits; publish limitations and negative results.

## 6. Next engineering gates

- Run the new Rust reference crate formatting, tests, strict Clippy and synthetic oracle in CI.
- Extend the exact oracle to correlated/conditional survey outcomes, sequential adaptive plans and Monte Carlo approximation with confidence bounds.
- Add sensitivity analysis for uncertain priors, likelihoods, decision utilities and observation costs.
- Build baseline implementations for random, fixed priority, max-EIG and expert ranking on identical fixtures.
- Link candidate survey locations and methods to a spatial cost/risk surface while keeping rights, access and conservation constraints explicit.
- Only after this small oracle is stable, connect the planner to Sol-Atlas evidence records and real survey campaigns.

## References

- Lark, R. M., Chagumaira, C. & Milne, A. E. (2022). [Decisions, uncertainty and spatial information](https://doi.org/10.1016/j.spasta.2022.100619). *Spatial Statistics*, 50, 100619.
- Glynn, P. et al. (2022). [Value of information and decision pathways: Concepts and case studies](https://doi.org/10.3389/fenvs.2022.805214). *Frontiers in Environmental Science*.
- [Where and When Should Sensors Move? Sampling Using the Expected Value of Information](https://doi.org/10.3390/s121216274). *Sensors*.
- Gashi, E., Deng, J. & Elezi, I. (2025). [Deep active learning: A reality check](https://www.sciencedirect.com/science/article/pii/S0167865525002697). *Pattern Recognition Letters*, 197, 188–194.
- Kakoulli, E. & Chatzifristofis, S. A. (2026). [From Geometric Coverage to Task-Aware Observability](https://doi.org/10.3390/electronics15194545). *Electronics*, 15(19), 4545.
- Waqar, A. (2026). [Performance benchmarking of scan-to-HBIM workflows for cultural heritage (2012–2025)](https://doi.org/10.1016/j.autcon.2026.106848). *Automation in Construction*, 184, 106848.
- Tootoonchi, R. & Hess, M. (2026). [Digital Twin in Heritage Buildings: A Comparative Literature Review](https://doi.org/10.5194/isprs-annals-XI-4-2026-435-2026). *ISPRS Annals*.
- Moyano, J. et al. (2026). [Metrological framework for digital twin-based measurement systems in cultural heritage](https://doi.org/10.1016/j.measurement.2026.122271). *Measurement*, 285, 122271.
