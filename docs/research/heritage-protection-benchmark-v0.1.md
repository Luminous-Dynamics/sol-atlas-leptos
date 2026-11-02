# Heritage Protection Benchmark (HPB) v0.1

**Snapshot:** 2026-11-02  
**Scope:** monitoring, threat/event detection, and preservation response. This benchmark is separate from the Heritage Discovery Benchmark (HDB); discovery and protection results must not be collapsed into one leaderboard score.

## 1. Why a separate benchmark is necessary

A model that says “this site has been looted” answers a different question from a model that identifies the month of disturbance, gives an early warning, tracks repeated incidents, or helps an authority allocate a limited monitoring budget.

The current literature illustrates the difference:

### Microsoft site-state looting classifier (February 2026)

The study *Satellite-Based Detection of Looted Archaeological Sites Using Machine Learning* reports a curated dataset of 1,943 Afghanistan archaeological sites (898 looted, 1,045 preserved) and PlanetScope monthly mosaics at 4.7 m/pixel spanning 2016–2023. An ImageNet-pretrained CNN with spatial masking reports F1 0.926; the strongest feature/embedding traditional ML baseline reports F1 0.710.

This is a site-state classification result on a specific curated dataset. It is not a regional archaeological discovery score, a month-localisation result, or proof of cross-country performance.

References:
- Microsoft Research publication: https://www.microsoft.com/en-us/research/publication/satellite-based-detection-of-looted-archaeological-sites-using-machine-learning/
- Code and dataset documentation: https://github.com/microsoft/looted_site_detection

### Microsoft WATCH: month-level temporal change detection (May 2026)

WATCH evaluates 1,943 archaeological sites in Afghanistan using PlanetScope RGB+NIR monthly mosaics at 4.7 m/pixel from 2017–2024. It includes three method families:

1. **TED — Temporal Embedding Distance:** training-free temporal deviation scoring.
2. **SSCD — Self-Supervised Change Detection:** reconstruction, forecasting and latent-novelty signals.
3. **WS — Weakly Supervised:** temporal localisation using sparse event-month labels.

Only 117 of the 898 looted sites had reliably known event months. Reported TED results include 55.0% exact-month recall with SatMAE and 92.5% recall within a ±3-month window with selected embeddings. The repository documents qualitative transfer checks across Syria, Turkey, Pakistan and Egypt; those qualitative checks must not be presented as equivalent to a fully labelled, quantitative external benchmark.

The repository is MIT-licensed, but the README explicitly says it does not ship the PlanetScope dataset or model checkpoints. Reproducing the method therefore requires separately authorised source imagery, checkpoints and runtime dependencies.

References:
- Paper / preprint: https://arxiv.org/abs/2605.08160
- Microsoft WATCH repository and protocol: https://github.com/microsoft/WATCH
- MIT code licence: https://github.com/microsoft/WATCH/blob/main/LICENSE

### S.A.D.A.: accessible, human-in-the-loop analysis

S.A.D.A. consolidates spectral indices, PCA, anomaly detection, clustering and local spatial autocorrelation into a guided GUI workflow. Its contribution is accessibility and reproducibility rather than a new automatic threat detector. It is a useful workflow/usability comparator, not a direct temporal-localisation model.

Reference: https://doi.org/10.1016/j.culher.2026.06.026

## 2. HPB task families

| Task | Question | Primary outputs |
|---|---|---|
| HPB-1 Site-state classification | Is a site in a looted/disturbed state, confirmed preserved for a window, or unknown? | PR-AUC, F1, sensitivity, specificity, calibration |
| HPB-2 Event-month localisation | When did a known event occur? | Exact-month recall, recall within ±1/±3/±6 months, median localisation error |
| HPB-3 Repeated-event tracking | Can multiple separate disturbances at the same site be detected without merging them? | One-to-one event recall, episode precision, event-count error |
| HPB-4 Early warning | Can the system raise useful alerts before recorded events? | Early-warning event rate, lead-time distribution, false alerts per observed site-month |
| HPB-5 Robust change monitoring | Does the system distinguish threat signals from seasonality, agriculture, construction, weather, sensor changes, missing imagery and other non-heritage causes? | Per-cause false-alert rates, missing-data robustness, held-out-region metrics |
| HPB-6 Operational triage | Which alerts should qualified analysts review first under a limited time/field-response budget? | Validated events per analyst-hour, recall at fixed alert budget, time-to-review |

A single event detector cannot receive credit for good site-state classification alone. HDB discovery metrics remain in HDB.

## 3. Label semantics: no false negatives from missing knowledge

Ground truth has exactly three semantic states:

- **KnownEvents:** a complete list of event months for a declared monitoring window. Only this state supports event recall and unmatched-alert analysis.
- **ConfirmedNoEventForWindow:** a custodian/researcher has verified no target event occurred during the declared interval. Only this state contributes verified negative months.
- **Unknown:** the event status is not known. This contributes neither positive nor negative labels.

A month with no valid acquisition is missing. An observation with a null score is missing. Neither is a zero-risk observation. Sparse event labels must not be expanded into a blanket negative series.

The Rust evaluator uses an integer month ordinal (year × 12 + month − 1) and retains explicit monitoring-window bounds.

## 4. Metric definitions

### Event localisation

- **Exact-month recall:** true event months with a matched alert episode in the same month divided by all known event months.
- **Recall within temporal tolerance:** true event months with a one-to-one matched alert within the registered ±N month window divided by known event months.
- **Median absolute localisation error:** median month distance among matched event/alert pairs.
- **Repeated-event matching:** one alert episode can match at most one event, and one event can match at most one alert in the event-localisation score.

### Early warning

- **Early-warning event rate:** known events with a distinct one-to-one alert match 1..N months before the event divided by known events.
- **Lead-time distribution:** report median and quantiles, not only a mean. A warning one month before and one six months before have different operational value.
- Early-warning and confirmation scores are separate. Do not hide the sensitivity/false-alert trade-off in one arbitrary weighted score.

### False-alert burden and coverage

- **False alerts:** alert episodes not matched either as a localised event or as an early warning within the declared complete event window, plus every alert episode in an explicitly confirmed no-event window.
- **False alerts per 100 observed site-months:** false alert episodes divided by months with valid observations in explicitly labelled windows, multiplied by 100.
- **Observation coverage:** observed valid-score months divided by all months in explicitly labelled windows.
- Unknown periods do not enter the denominator. Publish unknown and unobserved coverage separately.

The v0.1 evaluator merges adjacent alert months into episodes and uses a deterministic one-to-one temporal matching procedure. Threshold, merge-gap, localisation tolerance and early-warning horizon are explicit report fields.

## 5. Required comparison protocol

Reproduce the relevant WATCH baselines first:

- Same site list and event-month labels.
- Same imagery product, spatial masks and month coverage.
- Same train/validation/test partitions and label cut-off.
- Same foundation-model embedding families where licences and checkpoints permit.
- Same temporal margin, top-K and alert-threshold semantics.
- Same treatment of missing acquisitions.

Then compare additional methods: simple temporal differencing, handcrafted spectral/texture features, rolling-median TED, SSCD-like self-supervised forecasts, weakly supervised temporal sequence models, change-point detection, and calibrated multimodal ensembles.

Do not compare F1 from the site-state looting classifier with event-month recall from WATCH. Report task-specific scorecards.

## 6. Research targets

These are aspirations for preregistered experiments, not current Sol-Atlas results:

1. **Reproducibility gate:** reproduce WATCH exact-month and ±3-month event recall within 1 absolute percentage point on the same evaluation split before attempting to beat it.
2. **Localisation improvement:** target at least +5 absolute percentage points in ±3-month event recall and +5 points exact-month recall at no worse false-alert burden on the identical benchmark.
3. **Early warning:** publish recall at 1–3 and 4–6 month lead windows with false-alert rate, rather than reporting lead time alone.
4. **Geographic transfer:** report a quantitative per-country/per-region result on independently labelled blind data. Aim for no more than a 5-point degradation in primary recall relative to in-domain evaluation, subject to comparable label quality and task prevalence.
5. **Operational utility:** target at least 20% fewer analyst-hours per independently confirmed incident at a matched event-recall floor.
6. **Calibration:** report reliability curves and Brier score only where event/non-event labels are complete for the evaluated temporal windows.
7. **Repeated incidents:** beat single-event baselines on sites with independently annotated recurring disturbances without increasing false alerts per site-month.

If these targets are not met, report the failure modes and keep the strongest published baseline as the comparator. No target is a guarantee.

## 7. Hard alternative explanations and robustness tests

Every region should include hard negatives and confounders appropriate to its ecology and land use: agricultural ploughing, seasonal vegetation, erosion, river/flood changes, wildfire, new roads, quarrying, construction, military activity where safe/ethical to label, shadows, cloud/haze, compositing artefacts and sensor calibration changes.

Required stress tests:
- missing contiguous months and irregular revisit periods;
- different seasonal windows and interannual climate variation;
- spatial registration errors and quality degradation;
- sensor/platform transitions and resolution changes;
- single vs repeated events;
- class prevalence changes and previously unseen disturbance types;
- event-month uncertainty intervals rather than false precision where the event date is not known.

Radar must not be presented as a universal canopy-penetration or subsurface detector. A change anomaly is not proof of looting.

## 8. Governance and safety

Exact locations of vulnerable sites, burials, sacred places and active looting indicators must not leak into public alerts. Public interfaces receive only an authorised generalised representation. Exact geometry and response details require access-control checks at API/storage/export boundaries, not just UI hiding.

An alert is a hypothesis requiring qualified review. It is not proof of a crime or permission for intervention. Heritage authorities, land custodians and affected communities decide whether and how to investigate or publish information.

## 9. Scope of the initial implementation

The Rust module sol-atlas-core/src/heritage_threat.rs implements deterministic metric calculation over synthetic time-series records. It does not download satellite data, infer events, train models, reproduce WATCH, establish a true no-event label, or decide response actions. A later runner must add site-level bootstrap confidence intervals and provenance-bound external evaluation manifests.
