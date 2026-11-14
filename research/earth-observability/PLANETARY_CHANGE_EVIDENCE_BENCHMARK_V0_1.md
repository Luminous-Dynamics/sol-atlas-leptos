# Planetary Change Evidence Benchmark v0.1

**Programme:** PLANETARY-CHANGE-EVIDENCE-001 ([issue #795](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/795))  
**Status:** research protocol / benchmark design; no satellite benchmark executed  
**Date:** 2026-11-14  
**Repository owner:** Sol-Atlas  
**Claim ceiling:** this protocol defines data and evaluation requirements. It does not establish that any dataset has been ingested, any model has been validated, or any Luminous system is superior to an industry baseline.

---

## 1. Executive thesis

Sol-Atlas should be the **space-time evidence system of record for the Luminous planetary-intelligence programme**.

It should not be reduced to a globe renderer, a catalogue of map layers, or a pass-through for Planet AI. It should make every important Earth-change claim inspectable in terms of:

- what was observed, by which source and sensor;
- where and over what support it was observed;
- when the observation was acquired, processed, published and retrieved;
- what processing, calibration and coordinate transformations were applied;
- what was not observed, was obscured, or fell outside the detection envelope;
- which independent evidence supports or contradicts the claim;
- which explanations remain hypotheses;
- which decisions relied on the claim and what happened afterwards.

The programme's differentiator is not a new satellite foundation model by itself. It is a reproducible chain:

**source observation → qualified space-time evidence → competing explanation → targeted investigation → decision value → authorized action → measured outcome → revision of the evidence record.**

The proposed paradigm is an event-sourced, rights-aware **space-time evidence graph**, interoperable with existing geospatial standards.

## 2. Architecture ownership and existing work

This programme composes existing repository work. It must not create a second GIS, source-asset registry, evidence graph, EVSI kernel or governance authority.

| Concern | Owner / existing work | Boundary |
|---|---|---|
| Spatial and temporal evidence; source references; map and timeline; evaluation manifests | **Sol-Atlas** | Does not certify the truth of a source observation |
| Geospatial asset identity and validation | issue #65 | Reuse the renderer-neutral asset model |
| OGC/STAC discovery and ingestion | issue #240 | Reuse established interfaces; discovery is not permission |
| Rights-aware source-asset registry | issue #430 | Preserve original STAC items/extensions; unknown rights remain blocked |
| Ground-sensor observability / POD / PFA / sensor trials | issue #771 | Reuse the qualification discipline, but do not reuse GPR/ERT likelihoods as satellite likelihoods |
| Discrete EVSI and spatial/sequential investigation | PR #133, issue #134 | Reuse the decision kernel; no competing EVSI implementation |
| Engineering artifact lineage and decision receipts | PR #138 | Reuse exact-digest, supersession and revalidation patterns where applicable |
| Physics / environmental process models and counterfactual qualification | **Symtropy** | Model predictions are not observations |
| Hypothesis generation, causal reasoning, sensitivity and investigation planning | **Symthaea** | May propose; may not invent evidence, likelihoods or authority |
| Rights, consent, custody, stewardship, revocation and coordination | **Mycelix** | Governance is not duplicated inside Sol-Atlas |

The existing planetary-energy demonstration in DEMO.md and the source-attributed map projection in issue #791 remain useful as product context. However, reserve values, EROI trajectories, emissions, grid-stress values, P2P trading and economic outputs in a demo must remain **source-declared or simulated** until their exact source versions, rights, calculations and independent validation are recorded.

## 3. Core paradigm: space-time evidence, not map pixels

### 3.1 Separate the objects

A raster, a scene, a detected change, an explanation and a decision are different objects.

The benchmark should preserve at least these logical entities (reuse existing types where they already exist; do not add duplicate structs before the interface inventory):

1. **Source asset** — provider, collection, item and asset identity; exact version or bytes digest; original metadata; licence and allowed purposes.
2. **Observation** — acquisition time or interval, spatial footprint, sensor/platform, bands/modalities, native resolution, view/illumination context and quality flags.
3. **Processing run** — code/container/model revision, parameters, atmospheric correction, co-registration, resampling, compositing, masks, coordinate transforms and output digests.
4. **Evidence statement** — a bounded claim about a class, object, geometry, magnitude or temporal interval, with supporting and contradicting observation references and uncertainty.
5. **Hypothesis** — a possible cause or mechanism, its assumptions, competing explanations and the evidence needed to distinguish them.
6. **Investigation action** — candidate additional observation or field check, with expected information, decision value, costs, rights, access constraints and disturbance limits.
7. **Decision event** — exact evidence/receipt digests, actor, authorization, policy, disposition, conditions and supersession/revocation lineage.
8. **Outcome observation** — the later measurement used to compare an intervention's predicted and actual effects.

A change polygon is not automatically a verified event. A classifier output is not automatically a causal explanation. A recommendation is not authority to act.

### 3.2 State semantics: unknown is not unchanged

The following states must remain distinct at the evidence boundary:

- **observed**: source data were acquired and passed the declared quality checks;
- **no-change-within-envelope**: observations cover the declared support and period, and no change above a stated detection limit was found;
- **unobserved**: no relevant acquisition or usable observation exists;
- **obscured**: cloud, shadow, haze, smoke, snow, water, occlusion or another declared condition prevents a reliable observation;
- **invalid**: required radiometric, geometric, registration, CRS or processing checks failed;
- **below-detection-limit**: a phenomenon may exist but is below the qualified spatial, temporal or radiometric sensitivity;
- **out-of-domain**: conditions fall outside the model's qualified domain;
- **conflicted**: admissible sources disagree beyond the declared tolerance;
- **rights-restricted**: data may exist but the requested use is not authorized.

A missing tile, empty search result, cloud-covered pixel or absent feature must never be silently converted into a negative observation.

For a promoted `no_change_within_envelope` claim, complete coverage and a numeric minimum-change threshold are necessary but not sufficient. The claim must also bind an **independent, target/support/sensor/platform/acquisition-configuration-specific detection-qualification receipt**. That receipt must identify the qualified applicability domain and the detection/false-alarm envelope at the stated threshold (including calibration and test configuration). Sol-Atlas records the exact receipt digest; the appropriate modality/sensor qualification owner authenticates and interprets it. This contract must not infer detectability from a model confidence score, a generic sensor label or two successful acquisitions.

### 3.3 Time is an interval and a lineage

Do not flatten time into one timestamp. Keep separate:

- acquisition start/end (or a bounded/approximate interval);
- observation-valid interval, if different from acquisition;
- processing time and processing revision;
- publication/catalogue time;
- retrieval time;
- the interval between the last qualified observation without the feature and the first qualified observation with it.

The last pair bounds the event time; it does not reveal an exact event time unless additional evidence does.

### 3.4 Spatial support and scale are part of the claim

Each observation and claim must bind:

- CRS, axis order, horizontal datum and epoch where applicable;
- vertical datum and units where relevant;
- the full transform chain and software revision;
- geometry type, coordinate precision, spatial support and boundary policy;
- native and resampled ground sampling distance;
- acquisition footprint and valid-data/quality mask;
- antimeridian, polar and partial-coverage handling.

Do not compare a 10 m Sentinel-2 band with a 20 m or 30 m product as if they had identical spatial support. Do not use tile boundaries as ecological or administrative boundaries. Area estimates must include boundary, registration and classification uncertainty.

## 4. Standards and interoperability

Use existing standards at the external boundary:

- **STAC 1.1** for spatiotemporal asset metadata and extension preservation.
- **STAC API 1.0** and OGC API - Features for discovery/query interfaces where supported.
- **Cloud Optimized GeoTIFF (COG)** for appropriate raster exchange and range access.
- **GeoParquet / Parquet** for suitable vector and tabular analytical interchange.
- Existing OGC APIs for feature, process, coverage and environmental-data access where a specific provider supports them.

STAC is a discovery and metadata contract, not a truth, integrity, licence or qualification authority. A URI is a locator, a digest binds bytes, and a publisher identity or signature is a separate claim. Preserve provider-native identifiers and unknown extension fields. Do not normalize away information that downstream review may need.

## 5. Benchmark design

The benchmark is a suite of independently scored tracks. A model may be strong at detection and weak at transfer, calibration or decision value. Do not collapse these dimensions into one weighted leaderboard that hides critical failures.

### EO-A — Source, rights and reproducibility

Measure:

- percentage of promoted claims with complete source, asset/version, acquisition, footprint, CRS/transform, processing and quality lineage;
- retrieval reproducibility and digest consistency;
- rights-state correctness by purpose: search, internal analysis, training, calibration, blind evaluation, publication, redistribution and commercial deployment;
- provenance completeness and ability to replay from frozen manifests;
- invalidation/revalidation when a source, licence, transform or model version changes.

**Hard gate:** unknown or denied rights cannot satisfy an allowed-purpose check. A public download URL is not a licence grant. Permission to view is not permission to train, publish or redistribute.

### EO-B — Change detection and characterization

Evaluate separately:

- binary change / no-change within a qualified observation envelope;
- class-specific change: built-up expansion, vegetation loss/gain, surface-water extent, burn scars and other narrowly defined classes;
- object/segment localization and boundary quality;
- change magnitude and area-estimation bias;
- earliest/latest plausible event interval and detection delay;
- nominal revisit versus actual and quality-qualified usable-observation cadence;
- time-to-first-qualified observation and time-to-first-actionable claim;
- abstention and unresolved/insufficient-observation outcomes.

The effective revisit is the interval between usable, target-relevant observations, not the satellite's advertised orbital repeat cycle. See [#804 — EO-USABLE-CADENCE-001](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/804) for interval-censored event-time and quality-conditioned cadence qualification.

For each class, define the relevant minimum mapping unit, spatial tolerance, temporal window and consequence of omission/commission before evaluating.

### EO-C — Transfer and robustness

Use spatially, temporally and sensor-held-out evaluation:

- leave-geography/region-out;
- leave-year/season-out;
- leave-sensor/platform/processing-collection-out;
- resolution and scale shift;
- atmospheric/corruption stress, cloud/shadow and illumination variation;
- terrain, land-cover and acquisition-geometry shift.

Random pixel-level splitting is insufficient: adjacent pixels, overlapping patches, mosaics and repeated acquisitions can leak nearly identical context across train and test. Splits must be grouped by independent scene, site, event and time as appropriate.

Report pooled, per-domain and worst-domain performance, plus the in-distribution to out-of-distribution drop.

### EO-D — Calibration, selective prediction and falsification

Report:

- Brier score and log loss for probabilistic outputs;
- expected calibration error with reliability plots and sample counts;
- class-wise precision, recall, F1, IoU and PR-AUC;
- selective risk versus retained coverage;
- uncertainty interval coverage and width;
- false-positive area density (for example, false changed area per km² per declared time window);
- omission/commission rates and unresolved area.

Pixels are spatially correlated, so uncertainty estimates must cluster by independent sites/scenes/events, not treat every pixel as an independent trial.

Required negative controls include:

1. seasonal phenology mistaken for land-cover conversion;
2. cloud, haze, shadow or smoke mistaken for change;
3. sensor or processing-collection transition mistaken for change;
4. co-registration / orthorectification shift producing boundary doublets;
5. no-data, missing tile or invalid QA treated as unchanged;
6. mosaic overlap or duplicate acquisitions leaking across splits;
7. change below the declared minimum mapping unit;
8. out-of-domain scene retained with unjustifiably narrow confidence;
9. stale labels or changed source metadata reused without revalidation;
10. rights revoked or narrowed after an earlier permitted analysis;
11. two sensors sharing a common calibration/registration bias and agreeing incorrectly;
12. a causal story inferred from visual change without independent evidence.

### EO-E — Causal investigation and decision value

Satellite change detection alone does not establish why a change occurred. For each causal hypothesis, record:

- mechanism and causal graph/assumptions;
- alternative explanations and confounders;
- supporting and contradicting observations;
- missing observations and evidence needed to discriminate;
- external records or field observations required;
- the claim ceiling and conditions under which the hypothesis is rejected.

Symthaea may generate and rank hypotheses. Symtropy must own and qualify applicable physical/environmental models. The existing EVSI kernel and spatial/sequential planning work should rank only eligible investigations whose likelihoods, costs and loss model are qualified.

Compare fixed-grid, documented expert, random-eligible, cost-only, entropy-greedy and EVSI/ENBS strategies under the same frozen eligible actions and budget. Report held-out expected loss, realized loss, regret, time, cost, disturbance and whether the additional observation changed the decision. Legal access, rights, cultural duties, safety and ecological constraints are hard constraints, not terms to be traded away in a weighted score.

### EO-F — Operational usefulness and outcome

Measure end-to-end time and cost to produce a reviewable claim, including source discovery, analyst review and independent adjudication. After an authorized intervention, compare pre-registered predictions with subsequent observations. Report both beneficial and adverse/unintended outcomes.

The benchmark is not complete when a model emits a map. It is complete only when the claim can be inspected, challenged, reproduced and compared with later evidence.

### EO-G — Temporal observability and interval-censored event time

Use actual usable observations, not nominal orbital cadence, to measure change-time resolution and actionable latency. Preserve the last qualified no-change observation, first qualified positive observation, intervening unusable acquisitions and resulting event-time interval. Score interval coverage, interval width/sharpness, unresolved fraction, time-to-first-qualified claim and delay under weather, quality, sensor and rights outages. A change that occurs and reverses between usable observations must remain a possible missed event, not be relabelled no-change.

Compose with [#804 — EO-USABLE-CADENCE-001](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/804), which owns the irregular/censored observation timeline corpus and equal-budget cadence-policy tournament.

## 6. Baselines and industry comparison

Reproduce relevant baselines on identical source versions, spatial/temporal splits, labels, compute budgets and scoring code.

| Baseline family | Purpose |
|---|---|
| Persistence / no-change baseline | Detect dataset imbalance and establish the cost of false alarms |
| Spectral-index / threshold baseline (e.g. NDVI/NDWI, where valid) | Transparent sensor-aware reference |
| Random Forest or other classical classifier | Strong low-compute tabular/spectral baseline |
| Siamese U-Net / established CNN change detector | Matched supervised spatial baseline |
| Transformer / geospatial foundation encoder | Pretraining and representation comparison |
| Natural-image foundation encoder | Test whether geospatial pretraining transfers better, especially at changed resolution |
| Ensembles / calibrated probabilistic model | Uncertainty and selective-risk comparison |
| Human expert / documented GIS workflow | Operational reference and adjudication, with expertise/time recorded |

Do not assume a larger or newer foundation model is automatically better. Recent work makes the evaluation gap clear:

- The **FAIR-EO** 2026 preprint compares ten representative architectures over ten heterogeneous change-detection datasets under a common protocol and measures inference efficiency as well as predictive performance. It reports that optimized Siamese U-Nets can beat more complex models when efficiency is included, and that pretraining can help without additional inference cost.
- **EarthShift** evaluates eight geospatial foundation models over eleven tasks and five realistic distribution shifts. Its authors report an average 15–20% out-of-distribution performance drop across geography, time, sensor, source and spatial-scale shifts. Its paired splits are directly relevant to deployment robustness.
- A 2026 calibration study evaluates sixteen frozen encoders across classification and segmentation tasks and reports overconfidence under corruption/shift. Temperature scaling and deep ensembles did not arrest the degradation; a Gaussian-process probe improved ECE under severe cloud but worsened clean-condition ECE, and confidence-based abstention could not reliably defer confidently wrong predictions. Calibration and abstention methods are therefore necessary diagnostics, not a complete safety case.
- **CrossCD-Eval** (October 2026) compares five recent change-detection architectures across CLCD, HRSCD and LEVIR-CD under in-domain, cross-domain and few-shot adaptation. Its domain-aware feature-normalization results are a useful comparator for adaptation, including repeated 10-shot draws; Sol-Atlas should test this against frozen, leakage-resistant geographic/time/sensor holdouts rather than assume test-time feature alignment is always admissible.
- A 2026 remote-sensing change-detection meta-analysis reports high scores on popular datasets but also performance saturation and diminishing returns from parameter scaling. Treat LEVIR-CD/WHU-CD leaderboards as narrow task references, not proof of field transfer.
- **VHR-Bench** (November 2026 preprint) tests ten very-high-resolution datasets at 1.5 m GSD or finer and twelve encoders. Its reported results show that relative performance depends on label fraction and pretraining regime; Sentinel-scale pretraining alone transfers least well under RGB input in its VHR setting.
- **GeoAnalystBench** provides 50 expert-validated geoprocessing workflow/code-generation tasks. It is a useful static workflow comparator, but it does not by itself establish that a geospatial claim is true.
- **GeoAgentBench** (published online October 1, 2026) evaluates dynamic GIS-agent execution with 117 atomic tools and 53 tasks across six GIS domains. It is useful for parameter selection, runtime recovery and tool-chain correctness; its execution score still needs to be paired with independent source/label validation.
- **GISAgentBench** (August 2026 preprint) provides 349 practitioner-sourced multi-step tasks with executable reference trajectories and exact ground-truth output files. Its six-model evaluation reports a best strict, tolerance-aware task completion of 32.7%, illustrating how difficult exact multi-step GIS correctness remains. Its strict output matching is a valuable model for Sol-Atlas deterministic workflow receipts, though its task set and geography remain benchmark-scoped.

### Platform and industry comparators

The benchmark should distinguish infrastructure scale, imagery supply, GIS workflow capability and evidence qualification. These are different competitions:

- **Planet Labs / Planet AI:** a commercial imagery provider and natural-language discovery/analysis workflow over Planet's archive. Planet's own PlanetScope product documentation exposes provider-declared image-quality metadata (including sun angle, off-nadir angle, saturation and registration criteria). Preserve those as source QA evidence, not as independent ground truth.
- **Google Earth Engine:** a public geospatial data catalog, APIs, interactive application environment and planetary-scale cloud compute service. Google's current platform page reports an archive of more than 80 petabytes and over 30 years of imagery/science datasets, updated daily, and exposes AlphaEarth Satellite Embedding data. This is a scale/compute comparator; Sol-Atlas should use it or compatible open services as a possible execution/data substrate rather than attempt to reproduce that infrastructure.
- **Microsoft Planetary Computer:** a large environmental data catalog and cloud platform with STAC-based discovery. Its STAC alignment is a useful interoperability reference; keep the Sol-Atlas evidence contract independent of any one provider.
- **Esri ArcGIS Online Imagery:** a mature managed imagery hosting and raster-analysis product. Its November 10, 2026 update adds raster analysis in ModelBuilder and an Image Change Detection widget in Experience Builder. Compare workflow completion, output validity, auditability and user-review cost, not just feature checklists.
- **Airbus OneAtlas:** a commercial VHR imagery and SAR supply/streaming platform, with tasking and multi-resolution imagery. It is a potential licensed source for cases where 10–30 m public sensors cannot resolve the target, not a baseline dataset by default.

The strategic objective is not to beat all of these platforms at catalog size, cloud throughput, image resolution or GIS feature breadth. It is to sit **across** them as a portable evidence, uncertainty, rights and decision-assurance layer, with adapters that preserve provider-native metadata and do not make one vendor authoritative.

A particularly important South African challenge is **map-generation drift versus physical change**. The Department of Forestry, Fisheries and the Environment's draft Ninth National GHG Inventory report (Government Gazette, 2 May 2024) describes the newer SANLC generation as Sentinel-2-derived and explicitly warns that differences from older Landsat-based land-cover maps can reflect improved spatial/spectral/temporal interpretation and mapping quality rather than real landscape change. The same report gives SANLC 2020 overall map accuracy of 85.47% and mean class accuracy of 84.66%. These are product-level reported metrics, not a licence grant or proof that SANLC is independent reference truth for a Sentinel-2 model. The benchmark must plant cases where a source/sensor/ontology/processing-generation change creates an apparent class transition without a physical event, and score abstention or an unresolved measurement-change explanation rather than rewarding a false event claim.

A Planet Labs comparison should separate product capability from independent model qualification. Planet's November 5, 2026 Planet AI application emphasizes natural-language discovery over its imagery archive, temporal exploration, agent-assisted analysis and scheduled reporting. This is a useful discovery/workflow reference, not an independent geospatial truth benchmark.

**Planet AI terms boundary:** the Planet AI terms published and effective November 5, 2026 prohibit using Planet AI Licensed Materials (the service, Planet Content and Outputs) or Derivative Products to develop, train, fine-tune, validate, benchmark or otherwise improve AI/ML systems or competing products. Enterprise users are limited to internal, noncommercial business use and may not externally publish or distribute those materials/derivations. Do not feed Planet AI Licensed Materials, Outputs or Derivative Products into Luminous training, validation or benchmark corpora. Planet AI's defined Third-Party Content (for example, Landsat or Sentinel surfaced through the app) is subject to separate source terms; do not assume the Planet AI restrictions either replace or grant rights to that underlying data. Planet's separate Data, Catalog, Orders, Subscriptions and Basemaps products also require review of the exact applicable contract and data rights before integration.

## 7. Initial data strategy

### 7.1 Public-data first

Start with data whose exact product and use rights can be established without a commercial agreement:

- **USGS Landsat Collection 2:** the USGS states that its no-cost open-data policy remains in place and Landsat products are public domain; it requests source acknowledgement. Product-level processing, metadata and QA must still be preserved.
- **Copernicus Sentinel-2:** the Copernicus Data Space states Sentinel data are available free of charge and supports optical land monitoring. Sentinel-2 has thirteen spectral bands at 10 m, 20 m and 60 m resolutions; the two-satellite constellation has a nominal five-day revisit at the equator. Apply the separate Sentinel Data Legal Notice to the imagery; portal documentation and other portal content have their own terms. A five-day nominal revisit is not a five-day usable revisit at every target.
- Public benchmark datasets such as EarthShift / FAIR-EO where the dataset licences, label provenance and permitted uses are documented.

Do not assume that an open imagery licence grants rights to unrelated labels, web-portal material, uploaded user data or sensitive derived geometries. Maintain separate permissions for access, processing, model training, evaluation, public display and redistribution.

### 7.2 South African pilot candidate: Jukskei River catchment / Gauteng

This is a candidate pilot, not yet a data-admission decision.

A published study analysed 1987, 2001 and 2015 Landsat imagery over the Jukskei River catchment, reporting built-up and bare-surface increases and vegetation decreases, with reported overall classification accuracies of 85.9%, 87.5% and 92.5% for those respective years. This provides a local historical comparison and a concrete question set; it is not automatically a reusable labelled benchmark or independent ground truth.

The 2020 South African National Land Cover (SANLC 2020) product is a 20 m classification generated from Sentinel-2 imagery and automated mapping procedures. DFFE's draft Ninth National GHG Inventory report reports 85.47% overall map accuracy and 84.66% mean class accuracy on the detailed legend. It explicitly documents that cross-generation map differences can be caused by improved Sentinel-2 interpretation/mapping quality rather than actual landscape change. Because it is Sentinel-2-derived, SANLC 2020 must not be treated as independent ground truth for a Sentinel-2 model without an independence analysis. It is an external product comparator / potential weak-label source only, subject to separate rights, provenance and reference-label review.

The SAEOSS catalogue record currently surfaced for SANLC 2020 has no licence ID and is marked not open; therefore **rights status is unresolved**. Do not ingest or redistribute the raster until the authoritative DFFE terms and any access conditions are confirmed. A public catalogue entry or download link is not a permission grant.

Suggested pilot questions:
- Can built-up expansion be detected across Landsat/Sentinel-2 transitions without confusing sensor changes with real land change?
- Can vegetation and surface-water changes be separated from seasonal and rainfall-driven variation?
- Can the system bound when a change occurred rather than inventing a precise event date?
- Which additional observation or reference source most reduces uncertainty for a real planning/ecological decision?

Before a pilot dataset is frozen, obtain: exact source URI/item, product/collection version, licence/permission, reference-label provenance, class crosswalk, acquisition dates, cloud/QA masks, CRS/datum, spatial resolution, accuracy report, independent sample design and data digest. If any critical permission or reference detail is missing, the dataset remains a candidate not admitted.

## 8. Proposed scorecard

These are proposed gates and stretch hypotheses, not measured results.

| Metric | Qualification requirement | Stretch target |
|---|---|---|
| Evidence lineage | 100% of promoted claims bind exact source/version, support, acquisition interval, QA, transforms and processing revision | Deterministic replay of every published case |
| Negative controls | 0 no-data/cloud/invalid/OOD cases promoted to no-change-within-envelope | Keep zero across all regression corpora |
| Rights | 0 prohibited-purpose admissions | Full revocation/revalidation drill |
| Detection | Class-wise precision/recall/F1/IoU and area error reported | ≥10% relative reduction in worst-domain error vs strongest matched baseline |
| In-domain retention | Always report matched in-domain result | No more than 2 percentage points loss while improving worst-domain error |
| Calibration | Cluster-aware calibration and coverage reported with intervals and widths | Preregistered 90%/95% coverage on independent clusters, with useful selective-risk/coverage trade-off |
| Efficiency | Parameters, peak memory, latency/km², cost/scene and data-volume reported | Pareto improvement in quality, latency and cost |
| Decision value | Equal-budget regret, expected loss, costs, abstentions and constraints reported | ≥20% lower held-out expected decision loss at equal eligible budget |
| Outcome | Predicted vs measured result and adverse effects recorded | Repeated prospective improvement across independent cases |

The 10% / 2 percentage-point / 20% values are hypotheses for planning, not accepted thresholds. Before they become pass/fail gates, estimate baseline variance, independent cluster counts, class prevalence and statistical power. Publish per-class and worst-domain results; never let a high average conceal a catastrophic failure in one geography or class.

## 9. Evidence ladder and execution order

| Level | Evidence | Permitted claim |
|---|---|---|
| T0 | Schema / contract and rights-state tests | Structural validity only |
| T1 | Deterministic synthetic known-answer fixtures and adversarial controls | Synthetic behavior only |
| T2 | Retrospective public datasets with verified rights and documented reference labels | Dataset-scoped retrospective result |
| T3 | Blind geography/time/sensor holdout with independent adjudication | Out-of-domain result for the declared domains |
| T4 | Prospective observations and authorized field/reference validation | Prospective, site- and purpose-scoped evidence |
| T5 | Post-intervention measurement against preregistered predictions | Outcome evidence for the declared intervention |

Do not skip levels by substituting a visually convincing demo, language-model explanation, catalogue score or synthetic result for independent validation.

### Immediate P0 tasks

1. Inventory existing Sol-Atlas asset, STAC, source-rights, timeline, geometry and evidence-receipt contracts; publish the exact reuse map.
2. Create a versioned benchmark manifest referencing existing schema/type definitions rather than cloning them.
3. Register the benchmark tasks, baseline algorithms, metrics, splits, source rights and claim ceilings before data/model selection.
4. Build small synthetic fixtures for time intervals, cloudy/invalid imagery, sensor changes, registration shifts, seasonality, missing coverage, source revocation and label leakage.
5. Reproduce a no-change baseline and one transparent spectral-index baseline before neural models.
6. Select one public retrospective benchmark subset only after verifying label independence and rights.
7. Record exact-head format/build/test/scorer/receipt evidence. Queued, skipped, cancelled or missing checks are not passes.

### P1 tasks

- Reproduce relevant FAIR-EO baselines and EarthShift-style OOD splits.
- Add geospatial foundation encoders and VHR transfer comparisons only where resolution/labels are appropriate.
- Add cluster-aware confidence intervals, calibration and selective-risk evaluation.
- Compose qualified EO likelihoods with the existing EVSI decision kernel; keep sensor-specific likelihood owners explicit.
- Add Symthaea/Symtropy/Mycelix adapters only after each boundary has an executable contract and exact-version receipt.

### P2 tasks

- Permissioned South African pilot with a documented independent reference design.
- Prospective monitoring and post-intervention outcome measurement.
- Separate Planet Data/API integration after contract review; no Planet AI output ingestion into model training/validation/benchmark.

## 10. Nonclaims and safety boundaries

This protocol does not establish that:
- any satellite imagery has been downloaded or ingested;
- the Jukskei/SANLC candidate datasets are licensed for Luminous reuse;
- demo values in DEMO.md are independently verified;
- a model can determine cause from imagery alone;
- map completeness proves ground truth;
- an EO recommendation establishes land access, ownership, consent, permit, environmental approval or safety;
- a model or benchmark has beaten SOTA.

Sensitive locations, community-held data, private land information and infrastructure geometry require purpose-specific access controls and disclosure review. No field survey, land intervention, energy trade, grid operation or policy action is authorized by a benchmark result.

---

## References

1. OGC, [SpatioTemporal Asset Catalog (STAC)](https://www.ogc.org/standards/stac/) and [STAC API 1.0](https://docs.ogc.org/cs/25-005/25-005.html).
2. Tomanič et al. (2026), [A comprehensive and trustworthy benchmark of AI methods for change detection in Earth observation](https://arxiv.org/abs/2608.28247).
3. Doerksen & Kerner (2026), [EarthShift: a benchmark for measuring robustness to real-world distribution shifts in Earth observation](https://earthshift.github.io/).
4. Lehmann et al. (2026), [Beyond Accuracy: Assessing Calibration of Geospatial Foundation Models and Their Sensitivity to Distribution Shifts](https://arxiv.org/abs/2608.16614).
5. Sharma et al. (2026), [When GeoFMs Meet VHR: Benchmarking Foundation Models on Very High Resolution Earth Observation Imagery](https://arxiv.org/abs/2611.08003).
6. [GeoAnalystBench](https://doi.org/10.1111/tgis.70135), first published 2025-10-13; [GeoAgentBench](https://doi.org/10.1080/10095020.2026.2736957), published online 2026-10-01; and [GISAgentBench](https://arxiv.org/abs/2608.01645), August 2026 preprint.
7. USGS, [Landsat Collection 2](https://www.usgs.gov/landsat-missions/landsat-collection-2) and [Landsat public-domain status](https://www.usgs.gov/faqs/are-landsat-data-cloud-still-considered-be-within-public-domain).
8. Copernicus Data Space, [Sentinel-2 mission and products](https://dataspace.copernicus.eu/data-collections/copernicus-sentinel-missions/sentinel-2) and [terms and conditions](https://dataspace.copernicus.eu/terms-and-conditions); ESA, [Sentinel-2 facts and figures](https://www.esa.int/Applications/Observing_the_Earth/Copernicus/Sentinel-2/Facts_and_figures).
9. Planet, [Introducing the Planet AI App: Search. Explore. Act.](https://www.planet.com/pulse/introducing-the-planet-ai-app-search-explore-act/) (2026-11-05), [Planet AI Terms of Use](https://ai.planet.com/terms-of-use/) (effective 2026-11-05; user-category terms and Third-Party Content distinctions) and [PlanetScope product documentation](https://docs.planet.com/data/imagery/planetscope/).
10. [Google Earth Engine](https://developers.google.com/earth-engine/), [Microsoft Planetary Computer](https://planetarycomputer.microsoft.com/docs/), [Esri November 2026 imagery update](https://www.esri.com/arcgis-blog/products/arcgis-online/imagery/whats-new-in-imagery-in-arcgis-online-november-2026), and [Airbus Earth-observation imagery / OneAtlas](https://www.airbus.com/en/products-services/space/earth-observation/satellite-imagery).
11. Al Aghbari et al. (2026), [Beyond Accuracy: Trustworthiness and Assurance for Multimodal Foundation Models in Earth Observation](https://doi.org/10.3389/frsen.2026.1986933); Elkholy et al. (2026), [CrossCD-Eval](https://doi.org/10.1038/s41598-026-70914-7); and Kola et al. (2026), [From architectural evolution to foundation driven paradigms](https://doi.org/10.1186/s43067-026-00367-5).
12. Mawasha (2022), [Detecting land use and land cover change for the past 28-years using multi-temporal Landsat satellite images in the Jukskei River catchment, Gauteng, South Africa](https://sajg.org.za/index.php/sajg/article/view/823/0).
13. Department of Forestry, Fisheries and the Environment, [Draft 9th National Greenhouse Gas Inventory Report for South Africa](https://www.dffe.gov.za/sites/default/files/legislations/unfccc_greenhousegasinventoryreport9_g50607gon4772.pdf), Government Gazette No. 50607 (2024-05-02), SANLC accuracy and Landsat/Sentinel interpretation comparison; [SAEOSS SANLC catalogue record](https://saeoss.sansa.org.za/dataset/?organization=dffe&tags=Boundaries&tags=Climatology-Meteorology-Atmosphere&tags=SPOT&tags=land+cover&tags=land+use).
