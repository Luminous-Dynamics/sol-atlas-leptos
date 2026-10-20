# METHANE-ACTION-001: Evidence-to-mitigation benchmark contract

**Status:** Research specification / proposed benchmark. Not an operational system, regulatory certification, validated detector, emissions inventory, or claim of state-of-the-art performance.  
**Date:** 2026-10-19  
**Scope owner:** Sol-Atlas research tranche; intended interfaces to Symthaea, Symtropy, and Mycelix.

## Executive decision

Build a benchmark around the complete methane intervention loop, not just plume detection:

`observe → qualify evidence → associate source/asset → estimate emissions and uncertainty → prioritize inspection/repair → verify outcome`

The research hypothesis is that an evidence-aware, geospatially grounded workflow can improve the *amount of methane actually mitigated per unit of inspection effort and cost* while making fewer unsupported claims than disconnected detection, reporting, and work-order systems.

This document defines a falsifiable evaluation plan. It contains no measured performance results. A synthetic contract pass must never be presented as field validation; satellite alerts alone are not complete ground truth for either recall or false-positive rates.

## Why this is a worthwhile wedge

The problem is actionable and measurable, but data coverage remains incomplete:

- The IEA's *Global Methane Tracker 2026* estimates that around 70% of fossil-fuel methane emissions could be abated with existing technology and that more than 35 Mt could be avoided at no net cost under average 2025 energy prices. It identifies upstream oil and gas as the leading near-term target. These are sector-level estimates, not a promise that a particular asset or intervention will be cost-effective. [IEA, Key findings](https://www.iea.org/reports/global-methane-tracker-2026/key-findings)
- The OGMP 2.0 2026 report describes 163 participating companies in 90 countries, covering 43% of global oil and gas production, with 69% of reported emissions now based on measurement-based data. This reflects progress but does not imply complete global or facility-level observation. [OGMP 2.0, 2026 report](https://www.ogmpartnership.org/events/ogmp-20-2026-report-measurement-mitigation-better-methane-data-enabling-action)
- DOE/NETL explicitly researches field-scale performance of methane detection and quantification technologies across production, processing, and transport, including valves, pneumatic controllers, compressors, and tanks. We should compare against calibrated field evidence where obtainable, not only against generated examples. [NETL, Methane Emissions Quantification](https://www.netl.doe.gov/oil-gas/ngi/quantification)
- UNEP's Methane Alert and Response System (MARS) already connects satellite observations, alerts, notifications, and mitigation follow-up. It is a relevant operational comparator and potential source of historical event records—not a blank space for us to claim novelty. [UNEP MARS](https://www.unep.org/en/what-we-do/energy/methane/methane-alert-and-response-system)
- The Carbon Mapper data platform provides plume/source and scene-level products through documented APIs, with data from multiple airborne and satellite instruments. Its access, attribution, and commercial-use terms must be checked before each use; do not republish source data merely because an endpoint is public. [Carbon Mapper data](https://carbonmapper.org/data) · [API documentation](https://api.carbonmapper.org/api/v1/docs)

The opportunity is to make the gap between a detection and a verified mitigation decision measurable and auditable, while respecting existing specialist systems.

## Paradigm shifts

### 1. From inventory factors to evidence-bounded observations

A standardized emission factor can support planning, but must not be silently treated as a measurement of a particular event. Keep these distinct:

- **Measured observation:** a dated instrument output with calibration/quality metadata and a source artifact.
- **Remote-sensing retrieval:** a sensor- and method-specific estimate with its uncertainty and detection limits.
- **Inventory estimate:** activity data multiplied by an emission factor.
- **Inference:** a model's estimate derived from other evidence.
- **Scenario:** an explicitly hypothetical value used for stress tests.

Never add values from these classes together without a stated, validated reconciliation method.

### 2. From a plume record to an intervention outcome

A plume is an observation at a time; a source is an inferred ground location observed one or more times; a facility is an asset/organization association that may itself be uncertain. UNEP's MARS data dictionary explicitly distinguishes plume observations from source locations. Preserve those identities separately and retain the match confidence, method, and alternatives.

A work order marked “closed” does not prove that emissions fell. Mitigation requires a post-intervention observation or a clearly labeled unresolved verification state.

### 3. From single-objective ranking to constrained portfolio decisions

Prioritize candidate actions using expected avoided methane, uncertainty, inspection/repair cost, operational risk, access, time-to-mitigation, and evidence quality. Keep methane mass, energy, water, cost, worker safety, and community exposure as separate dimensions. Safety, access authorization, environmental restrictions, and minimum evidence requirements are hard gates, not soft penalties that a high financial score can overcome.

Keep a no-action / wait-for-more-evidence option in the choice set. A model must be allowed to say “insufficient evidence to rank these safely.”

### 4. From model confidence to calibrated, falsifiable confidence

Symthaea may propose competing explanations and rank the next useful observation. The system must expose uncertainty, calibrate probabilities on held-out data, test alternative causes, and abstain outside its validated operating domain. “The model agrees with itself” is not evidence.

### 5. From siloed dashboards to portable, independently checkable receipts

Record the chain from source observation to asset association, model/version, recommendation, authorization, repair, and post-repair verification. Mycelix may provide append-only provenance and authorized cross-organization evidence exchange, but cryptographic integrity only says which bytes/claims were committed and by whom; it does not prove that the original reading was true.

## Four-project division of responsibility

| Project | Proposed responsibility | Must not claim |
|---|---|---|
| **Sol-Atlas** | Source-pinned geospatial observations, facility/source association, coverage gaps, spatial joins, and regional context. | That mapped or nominal capacity is operating capacity; that a nearby asset is proven to be the source; or that missing observations imply zero emissions. |
| **Symthaea** | Hypothesis ranking, anomaly assessment, uncertainty/calibration evaluation, expected-value-of-information ranking, and explanation of why a recommendation was made. | That a prediction establishes physical truth, that internally generated hypotheses validate themselves, or that a synthetic benchmark establishes field accuracy. |
| **Symtropy** | Scenario/counterfactual and constrained intervention scheduling once validated physical and operational models are available. | That a generic physics engine is a calibrated fluid-dynamics, atmospheric-dispersion, refinery, or pipeline simulator without separate validation. |
| **Mycelix** | Permissioned provenance, actor/action records, model and artifact digests, review, repair, and verification receipts. | That signatures, hashes, or decentralization prove sensor truth, regulatory compliance, or a real-world emissions reduction. |

The integration is a proposed architecture. No end-to-end methane-to-mitigation capability is established by this document.

### Reuse existing research primitives instead of creating parallel stacks

- **Symthaea experiment selection:** [PR #8034](https://github.com/Luminous-Dynamics/symthaea/pull/8034) describes a prior-free, maximin standardized-separation selector for choosing an experiment that distinguishes rival models under predictive uncertainty and relative cost. If its API is qualified and available on the selected base, use it to rank the **next measurement** that would best distinguish competing source hypotheses. Do not relabel its score as Bayesian expected information gain, a calibrated posterior, a leak-rate estimator, or a repair optimizer. Its own documented assumptions (including conditional independence) must remain visible.
- **Symtropy simulation/experiment conventions:** [Physics Experiment Atlas PR #1981](https://github.com/Luminous-Dynamics/symtropy/pull/1981) is a useful reference-case and receipt pattern, but it is not a validated methane dispersion, pipeline-flow, or refinery-process simulator. Any domain-specific physical model needs separate calibration, assumptions, numerical validation, and held-out physical tests.
- **Mycelix outcome receipts:** [process-run receipt PR #5228](https://github.com/Luminous-Dynamics/mycelix/pull/5228) already captures a useful general distinction among simulation outputs, instrument observations, calibration claims, and review dispositions. Reuse its evidence-layer semantics or existing shared types where compatible instead of creating a competing ledger/cryptography system. Do not transplant a manufacturing receipt as though it validated a gas measurement; the methane-specific adapter must bind its own sensor/source, units, calibration, and intervention evidence.
- **Sol-Atlas flaring evidence:** [gas-flaring admission PR #94](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/94) already addresses source/terms review, snapshot identity, and purpose-gated data admission for annual flare-volume estimates. Reuse its exact-source/provenance principles, but keep annual flaring-volume records distinct from individual methane-plume observations and from verified avoided methane.

These links describe existing research branches, not automatic integration or a claim that every referenced PR is merged or qualified. Before coupling any of them, pin the exact commit/API, verify exact-head checks, and add cross-repository contract fixtures.

## Benchmark tracks

### Track A — Evidence and data admission

Test invalid and incomplete records before any model is evaluated:

- missing source identity, source artifact digest, timestamp, coordinate reference, gas species, units, quality flag, or instrument/method identifier;
- stale or absent calibration, unknown detection threshold, invalid uncertainty, non-finite/negative rates, impossible units, timestamp inversion, duplicate observation IDs, and mismatched digests;
- confused identities: plume observation reused as a unique source; source merged with facility without a match record; repeated overpasses double-counted as separate facilities;
- unknown status, unobserved asset, or missing data silently converted to “no emissions”;
- source terms or usage rights missing or incompatible with intended use.

**Gate:** malformed or unqualified inputs must be rejected or explicitly quarantined with a reason. Unknown stays unknown. Validating a digest establishes byte identity, not the truth or calibration of the underlying observation.

### Track B — Event detection and source association

Compare event-level detection and asset/source association across instrument, geography, weather/season, facility class, and emission-rate bands. Use baselines including:

1. published event/alert outputs where labels permit a fair comparison;
2. simple threshold/rule baselines;
3. a strong conventional statistical or machine-learning baseline;
4. the best available specialist/vendor baseline, where licensing and access permit;
5. our evidence-aware workflow.

Do not calculate recall or false-positive rates from positive-alert-only lists. A dataset of reported plumes does not provide the full universe of sensor opportunities, missed events, or true negatives. For those metrics, require scene/flight coverage, reviewed negatives, or controlled releases with independent ground truth.

#### Empirical comparator and test design

A 2024 *Atmospheric Measurement Techniques* single-blind evaluation tested nine satellite methane-sensing systems using 82 controlled releases during overpasses and collected 134 emissions estimates from six analysis teams. The paper reports 80 estimates correctly identified (58%), with 46 true-positive detections, 34 true-negative non-detections, 41 false negatives, and zero false positives. This is a useful published challenge reference—not an industry-wide current SOTA score, not a direct baseline for a differently sampled corpus, and not a guarantee of performance in another basin or operating condition. [AMT (2024), *Single-blind test of nine methane-sensing satellite systems from three continents*](https://amt.copernicus.org/articles/17/765/2024/).

The key lesson for our benchmark is to report **detection probability and false-alert behavior jointly**. A detector with zero false alarms in one test can still miss a meaningful number of releases. Use the same blind release opportunities and predeclared scoring rules when comparing systems; stratify by release-rate band, wind speed/variability, scene quality, satellite/instrument, and basin. Keep no-detection, no-coverage, and quality-rejected cases distinct.

The U.S. EPA's Advanced Methane Technology Alternative Test Method program asks for performance evidence across each basin and environment where a technology is intended to operate; controlled releases are encouraged, and comparable validation data are needed when controlled releases are unavailable. Our offline benchmark should therefore be treated as a development gate—not as proof of regulatory eligibility or field applicability. [EPA, Oil and Gas Alternative Test Methods](https://www.epa.gov/emc/oil-and-gas-alternative-test-methods) · [EPA Methane Advanced Technology Program guidance (May 2024)](https://nepis.epa.gov/Exe/ZyPURL.cgi?Dockey=P101GVV6.TXT).

**Add two 2026 comparators, without mixing their scopes:**

- An independent, single-blind TADI controlled-release study in Europe evaluated eight commercial systems spanning satellite, drone and continuous monitoring. It reports false-positive rates from 0% to 11%, quantification regression slopes from 0.09 to 1.13, and R² from 0.08 to 0.97. Wind—especially low and variable wind—materially affected quantification. The paper reports a wide spread in minimum detected flow across technologies (0.01–210 kg/h), because the test platforms and release scenarios differed. These ranges are not a single cross-industry ranking or a universal detection limit. Use the paper's participant-specific confusion counts and applicability notes when constructing matched baselines. [McManemin et al., *Atmospheric Measurement Techniques* 19 (2026), 923–934](https://amt.copernicus.org/articles/19/923/2026/).
- A July 2026 GHGSat constellation preprint fits detection probability as a function of plume signal-to-noise ratio, emission rate, wind, retrieval noise and spatial resolution. For its stated median controlled-release conditions (3 m/s wind, 7 mmol m⁻² column-density noise, 27 m resolution), it reports Q50 = 99.0 ± 5.3 kg/h; its estimated quantification slope also changes with the wind source. It is a useful example of a condition-dependent probability-of-detection curve, but applies to the studied sensor product and test campaigns—not to every sensor or terrain. Treat it as a preprint unless its publication status is independently confirmed at study freeze. [Ramier et al., EGUsphere (2 July 2026), DOI 10.31223/X5N78R](https://egusphere.copernicus.org/preprints/2026/egusphere-2026-3662/).
- The EPA's live approved-method list, as retrieved for this research tranche, includes Sensia's Mileva and Caroline methods with a stated leak-resolution criterion below 1 kg/h for periodic screening (approval records dated 12 June 2026, with linked documents updated in August). This is an **application-specific program criterion**, not evidence that every product version has a universal 1 kg/h detection limit in every environment. Freeze the specific approval document, scope, product/version and conditions before using it as a comparator. [EPA approved alternative test methods](https://www.epa.gov/emc/oil-and-gas-alternative-test-methods).

The main paradigm change that follows is: **score the whole probability-of-detection curve and its operating context, not one global sensitivity number**. For each sensor/product/version, estimate probability of detection by release-rate band and relevant conditions (wind speed/variability, retrieval noise, resolution, surface/scene quality, topography, and tasking/coverage); separately estimate false-alert probability on matched zero-release opportunities. Use blind or independently administered opportunities, release-rate-specific uncertainty, and held-out locations/time campaigns. A fixed-threshold Q50 is not enough on its own, and cross-vendor comparisons are not fair unless opportunity set, conditions, reference uncertainty, and scoring rules align.

The TADI protocol also shows why **knowing the observation schedule is a confounder**: participants knew the release start/stop times, and the authors warn that this could overstate detection capability. Our primary challenge should hide release timing and source location from the submitted system wherever safely feasible. Where operations require an announced schedule, classify that result as semi-blind and report it separately from fully blind testing. Preserve no-submission, failed-quantification, valid non-detection and invalid-coverage states independently; post-hoc cleanup must not turn no-submission cases into correct negatives or silently erase them from the opportunity denominator.

### Track C — Emission-rate quantification and uncertainty

Evaluate only against a stated reference measurement appropriate to the sensor's scope and temporal/spatial footprint. Record rate units, averaging window, reference instrument, uncertainty of the reference, environmental conditions, and any non-detect/censoring rule.

Report at least:

- median absolute error and median absolute log10 error for positive reference rates;
- bias by order of magnitude and facility/source category;
- 50%, 90%, and 95% interval coverage and interval width;
- performance at the detection limit and below/near/above it;
- error by sensor, weather/scene quality, geography, and source type.

Do not present precision without recall/coverage and uncertainty; do not score a non-detect as zero emission.

Before scoring any measurement-derived estimate, require an explicit instrument/platform ID and retrieval/measurement-method version. For in-situ reference readings, require a calibration artifact with exact-byte digest and a validity interval that contains the observation timestamp; bind the calibration to the same instrument identity. Reject expired, mismatched, absent, or unverifiable calibration records. For satellite and airborne products, preserve the provider's product/version and quality class; do not assume a declared instrument ID proves calibration or that all sensor products share the same detection limit.

This is a key benchmark control: a supposed gain must not come from mixing instruments, retrieval versions, calibration periods, or reference quality between candidate and baseline. Report per-instrument/per-method results and confidence intervals before aggregating them.

### Track D — Inspection prioritization and intervention selection

At a fixed budget (crew-hours, site visits, or money), measure:

- confirmed sources found per unit inspection effort;
- independently confirmed methane mass identified;
- predicted versus realized avoided methane after repair;
- time from first valid observation to alert, review, field confirmation, repair, and verification;
- cost per unit of **verified** methane reduced;
- ranking quality under imperfect and delayed observations;
- the share of recommendations withheld because authorization, access, safety, or evidence requirements failed.

Compare policies at equal cost and equal operational constraints. A policy cannot win by assuming unlimited crews, privileged access, or a perfect ground-truth feed that competitors do not receive.

### Track E — Closed-loop verification

Evaluate outcome before/after intervention with a pre-declared protocol. Account for revisit timing, variability in emissions, weather, production state, sensor detection limits, and alternative causes. Use repeat measurements or independent post-repair verification where possible.

Keep these outcomes separate:

- detection/notification;
- operator acknowledgement;
- inspection completed;
- repair reported complete;
- measured emissions reduction;
- independently verified mitigation;
- unresolved / not observable.

A ticket closure, model assertion, or absence of a subsequent satellite detection is not by itself proof of zero emissions.

### Track F — Operational resilience and evidence integrity

Inject controlled faults: missing scenes, stale data, duplicate observations, clock shifts, sensor calibration expiry, revoked access, model version drift, unavailable services, and contradictory reports. Test replay, source/model digest binding, audit completeness, safe abstention, and recovery.

For process-safety context, use the applicable site safety-management system and relevant recognized indicators such as API RP 754 as references; this methane benchmark is not a substitute for process hazard analysis, alarm management, or safety certification. [API RP 754, 4th edition announcement (2026-08-06)](https://www.api.org/news-policy-and-issues/news/2026/08/06/api-publishes-fourth-edition-process-safety-performance-indicators-standard)

## Provisional targets: ambitious gates, not SOTA claims

These are proposed engineering gates to ratify after establishing a baseline and measuring the reference-data uncertainty. They are not external standards or reported results.

| Dimension | Initial target to test | Guardrail |
|---|---|---|
| Evidence admission | 100% rejection/quarantine for authored invalid-input and provenance negative controls. | No malformed fixture silently accepted; no invalid input becomes a zero-emission observation. |
| Detection | On a representative, independently labeled holdout, either ≥10 percentage-point absolute recall gain at a matched false-alert burden **or** ≥25% fewer false alerts at matched recall versus the strongest fair baseline. | Report both measures, per-stratum results, and recall at fixed false-alert operating points; published satellite tests show missed releases remain material even when false positives are rare. No hidden trade of misses against alert burden. |
| Quantification | Pilot target of median absolute log10 error ≤0.30 dex (roughly a factor of two) when reference uncertainty and observation scope make that target meaningful. | Report bias and interval coverage; suspend the target for a stratum if the reference is not sufficiently accurate. |
| Uncertainty | Nominal 90% intervals achieve 85–95% empirical coverage on held-out site/time groups. | Report interval width and subgroup coverage; broad but useless intervals do not count as success. |
| Prioritization | ≥20% more independently confirmed methane mass located per fixed field-work budget, or ≥20% lower cost per verified unit reduced, versus the best fair baseline. | Include full operating/inspection costs and maintain safety/access gates. |
| Mitigation loop | ≥20% lower median time from valid, actionable alert to verified mitigation in a prospective pilot. | Compare like-for-like workflows, record right-censoring and unverified cases, and do not infer causality from an uncontrolled before/after comparison. |
| Integrity/replay | 100% detection of authored tamper, stale-version, duplicate-ID, and unauthorized-action negative controls; deterministic replay for the same pinned inputs and tool version where deterministic execution is specified. | Do not equate a passing control fixture with security certification. |

A result is a benchmark win only if the holdout is predeclared, the baseline is strong and fairly matched, provenance is preserved, negative controls pass, and the improvement survives per-site/time/source stratification. If baseline performance is already stronger, record that and improve the research question instead of lowering the gate.

## Dataset strategy and limits

### Public retrospective sources

1. **UNEP MARS** plume/source exports and data dictionaries: [download page](https://methanedata.unep.org/download-dataset) · [plume/source data dictionary](https://methanedata.unep.org/dict-mars-plumes) · [terms of use](https://methanedata.unep.org/terms-of-use). Use it first for event chronology, source grouping, notification/response context, and geographic coverage—not as an automatic complete detection benchmark. The current data dictionary states that plumes from Sentinel-2, Landsat-8/9, EMIT, PRISMA, and EnMAP were requantified after **20 February 2026** and that the export is **CC BY-NC-SA 4.0 and non-commercial**. Thus older and newer exports must be treated as different artifact/retrieval versions, and MARS-derived assets must not enter a commercial product without separately established rights. Detected-only plume lists and notification criteria also create selection bias; they are not a full opportunity census.
2. **Carbon Mapper** data API and product definitions: [data portal](https://carbonmapper.org/data) · [API docs](https://api.carbonmapper.org/api/v1/docs) · [product guide](https://carbonmapper.org/articles/product-guide). The API warns that results may contain **uncalibrated Tanager first-light data** and exposes product-quality, gas, instrument, and platform fields. Preserve provider product/version, calibration status, instrument/platform, and the original quality classification. Exclude uncalibrated first-light products from reference-grade quantification unless independently qualified; otherwise keep them explicitly flagged and reported as a separate exploratory stratum. Respect current usage restrictions and reference rather than redistribute records unless permitted.
3. **OGMP 2.0 company factsheets**: useful for reporting-maturity context and understanding the difference between inventory and measurement-based reporting, not event-level detection ground truth. [OGMP 2.0 2026 report](https://www.ogmpartnership.org/events/ogmp-20-2026-report-measurement-mitigation-better-methane-data-enabling-action)
4. **DOE/NETL field validation**: preferred route to suitable controlled-release and field-reference evidence. Confirm specific dataset access, release terms, units and reference uncertainty before treating any record as ground truth. [NETL methane quantification program](https://www.netl.doe.gov/oil-gas/ngi/quantification)

### Split design

- Freeze source retrieval date, provider/API query, source product/retrieval version, terms snapshot, source artifact URL, attribution/license, byte length and SHA-256 for every downloaded artifact. For MARS, record whether a record comes from a pre- or post-20-February-2026 requantification snapshot; do not silently mix versions.
- Split by **site/source and time**, not random rows. Repeated plumes from the same source must not leak between train and test; validate the corpus-pinned split manifest before model selection.
- Preserve instrument/platform, product version, calibration state, and provider quality class. Treat uncalibrated products as an explicit exclusion or separate exploratory stratum, not as verified reference truth. Test geography and season shift and publish a separate out-of-domain set.
- Do not use private customer telemetry in a public corpus without explicit rights and a redaction plan.
- Label synthetic cases and scenarios unambiguously. Synthetic fixtures are for schema, boundary, and fail-closed tests only; they provide no evidence of real detector accuracy or achievable savings.

### Complete opportunity census

Before computing a headline score, freeze the complete expected set of valid measurement opportunities and reference labels for the evaluation. Require exactly one response row per expected case and system. Missing, extra, duplicate, or malformed rows suppress the headline metrics; do not silently score only successful detections. Rows must explicitly distinguish **covered**, **not tasked**, and **invalid scene**, alongside **detected**, **explicit non-detection**, **abstention**, and **no output**. No-coverage cases are not true negatives. Positive events that were not observed remain misses in end-to-end recall; report recall conditional on coverage separately so the two questions cannot be conflated.

Report confusion counts, end-to-end recall, recall conditional on coverage, false-alert rate among covered metered-zero opportunities, coverage rate, abstention rate, and rate-quantification errors. Include per-stratum counts and 95% Wilson score intervals for per-system binomial rates, including denominators. These intervals are descriptive for an individual system; comparing systems on shared scenes, sites, or time windows requires paired or cluster-aware inference rather than treating the estimates as independent. A tiny sample must show its uncertainty and cannot substantiate a SOTA claim.

### Paired baseline comparison

A baseline and candidate must be scored on the **same complete opportunity census**. Each case therefore has both an analysis `stratum_id` (what subgroup should be reported) and a distinct `cluster_id` (the sampling dependence unit, e.g. site × overpass/time window). They must not be conflated: stratification is for reporting; clustering is for uncertainty.

Compare candidate-minus-baseline end-to-end recall and false-alert rate on covered metered-zero cases. Report baseline/candidate point estimates, denominators, discordant paired event counts, seed, and deterministic cluster-bootstrap intervals that resample whole clusters instead of treating every row as independent. Missing/extra/duplicate submissions invalidate the paired comparison.

The current Rust reference implementation withholds cluster-bootstrap intervals unless there are at least 10 independent clusters and at least 1,000 bootstrap replicates. These are conservative implementation gates, not universal evidence that any data set has enough independent information; the cluster definition and held-out sampling design remain decisive. If too few clusters exist, report point estimates with an explicit “interval omitted” note and do not promote the result as an uncertainty-qualified SOTA improvement. Reuse these paired comparisons on predeclared site/time holdouts; do not select the baseline or operating point after inspecting final holdout results.

### Split integrity and leakage gate

Pin the exact raw corpus-manifest bytes by SHA-256 before assigning partitions. Require every eligible case exactly once in train, validation, and test, with an optional separately controlled blind-test split. Split by dependence/source groups before fitting transformations, choosing thresholds, tuning hyperparameters, or comparing candidates. Reject any case manifest whose cluster identity, stable source group, or exact case-artifact digest crosses partitions. A row-level random split is not a defensible site-, source-, or campaign-generalization test when related observations can cross the boundary.

A split validator can check internal integrity, not certify that a cluster is independent or that no semantic leakage remains. The experiment owner must document the intended generalization axis (new overpasses at known sites, unseen sites, unseen basins, new sensors, or future time periods) and choose group keys accordingly. Keep a second, access-controlled blind holdout for the final comparison where practical; do not repeatedly tune to it.

### Post-intervention verification and realized mitigation

A plume alert, a repair work order, and a claimed emissions reduction are different evidence types. Do not infer avoided methane directly from the count of alerts, a closed work order, or a single post-repair non-detection.

For a first outcome contract, require a pinned intervention/work-order artifact, a pinned verification report, a reviewed association to the same stable source, one pre-intervention in-situ methane reading, and at least two post-intervention follow-ups across a predeclared time span. Follow-ups must use the same qualified instrument and processing method by default, with calibration valid at each observation time. Use a qualified cross-instrument comparison only when an independent calibration/bridging study is present and separately approved.

When the baseline and all follow-ups are quantified, report the median post-repair rate and baseline-minus-median point difference. When post-repair readings are non-detections, retain their detection limits and report bounds instead of coercing them to zero. With declared symmetric absolute uncertainty bounds, a conservative sustained-reduction interval is formed from the baseline lower/upper bounds and the most conservative post-measurement bounds. A positive lower bound is evidence consistent with sustained rate reduction over the sampled windows, not proof of causal attribution or total tonnes avoided.

Do not multiply that rate difference by an assumed operating duration. Estimating avoided mass requires an independently justified duration/activity model, temporal variability, repair scope, and uncertainty propagation. Report verified mitigation per inspection hour and per dollar only when both the reduction evidence and the measured effort/cost denominator are available.

## Reproducible experiment protocol

Every evaluated run should emit a machine-readable receipt containing:

- benchmark contract version and corpus manifest digest;
- source identifiers, dates, licensing/attribution metadata, and immutable artifact digests;
- code commit, model/configuration digest, dependency/toolchain versions, and random seed where applicable;
- split assignment, metrics, uncertainty, subgroup counts, missingness, exclusions, and explicit failure/abstention counts;
- baseline identity and configuration;
- negative-control outcomes and a concise claim scope.

The receipt must bind to an exact code and data version. A pending or queued workflow is not a pass; validation must be reported against the exact evaluated commit. An independently reproducible result is preferred to a vendor or model author's unverified summary.

## Implementation sequence

### Phase 0 — Contract and evidence inventory (this specification)

- Review this contract against Sol-Atlas's gas-flaring evidence-admission work and current industrial-capacity semantics.
- Confirm data-source access terms and define exact provenance fields.
- Keep this PR documentation-only; it introduces no new capability claim.

### Phase 1 — Rust-first offline contract evaluator

Create a small, dependency-bounded Rust package or core module to validate event identities, units, non-finite/negative values, observation/source/facility separation, quality/uncertainty metadata, artifact digests, and explicit unknown states. Add adversarial fixtures for duplicate observations, forged/mismatched digests, invalid timestamps, stale calibration, and missing source identity. CI should run tests directly for the package rather than assuming a workspace-wide command covers it.

### Phase 2 — Source-pinned retrospective corpus

Build small source adapters only after inspecting provider schemas and terms. Save retrieval manifests and digests; do not commit restricted imagery or raw telemetry. Publish counts and known selection biases before publishing metric results.

### Phase 3 — Baselines and calibration

Implement transparent threshold/statistical baselines first, then compare a learned model and the full decision pipeline. Freeze site/time holdouts and score scripts before looking at final holdout metrics.

### Phase 4 — Field validation and prospective pilot

Partner with an operator or independent measurement team. Establish ground truth, reference instrument accuracy, operating constraints, and repair verification before evaluating any live workflow. Begin in advisory mode; no autonomous physical actuation is authorized by this project.

## Go/no-go and claim discipline

Do not proceed to operational recommendations if the test cannot establish source identity or the evidence needed for the action. Do not infer emissions from missing observations, infer successful mitigation from ticket closure, or publish a percentage reduction without a credible baseline/counterfactual.

Do not call the system “better than SOTA” unless an independently reviewed, predeclared experiment beats named baselines on representative held-out data and reports uncertainty, failures, costs, and relevant subgroup results. If no fair ground truth or complete negative set exists, publish a limitation and the test needed to resolve it rather than a leaderboard score.

## References

- IEA, [Global Methane Tracker 2026 — Key findings](https://www.iea.org/reports/global-methane-tracker-2026/key-findings)
- OGMP 2.0, [2026 Annual Report](https://www.ogmpartnership.org/events/ogmp-20-2026-report-measurement-mitigation-better-methane-data-enabling-action)
- UNEP, [Methane Alert and Response System (MARS)](https://www.unep.org/en/what-we-do/energy/methane/methane-alert-and-response-system), [dataset download](https://methanedata.unep.org/download-dataset), [data dictionary](https://methanedata.unep.org/dict-mars-plumes)
- Carbon Mapper, [data portal](https://carbonmapper.org/data), [API](https://api.carbonmapper.org/api/v1/docs), [product guide](https://carbonmapper.org/articles/product-guide)
- U.S. DOE NETL, [Methane Emissions Quantification](https://www.netl.doe.gov/oil-gas/ngi/quantification)
- American Petroleum Institute, [API RP 754 4th edition announcement](https://www.api.org/news-policy-and-issues/news/2026/08/06/api-publishes-fourth-edition-process-safety-performance-indicators-standard)
- NIST, [Framework for a Digital Twin in Manufacturing](https://www.nist.gov/publications/framework-digital-twin-manufacturing)
