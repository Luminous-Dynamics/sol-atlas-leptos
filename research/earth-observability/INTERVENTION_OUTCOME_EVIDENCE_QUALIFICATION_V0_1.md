# Intervention Outcome Evidence Qualification (IOEQ) v0.1

**Status:** protocol and synthetic qualification fixtures. No intervention effects are validated by this document.  
**Issue:** [#853](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/853)  
**Jukskei parent:** [#847](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/847)  
**Source admission:** [#852](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/852)

## 1. Research proposition

Environmental projects routinely publish hectares restored, trees planted, tonnes removed, sensors installed, jobs created and modelled benefit estimates. These are useful records, but they do not all measure the same thing.

An assurance system must distinguish:

- **Implementation output:** what was delivered.
- **Operational performance:** whether an asset or service is functioning.
- **Observed physical outcome:** what changed in the environment, as measured by a qualified observation process.
- **Attributable intervention effect:** how much of that change is credibly attributable to a defined intervention relative to a counterfactual.
- **Modelled benefit projection:** what a model estimates under declared assumptions and scenarios.

The paradigm shift is to **type the claim before scoring the claim**. A model's confidence, the prestige of a publisher, the quantity of project outputs, and the number of derivative reports cannot promote a claim to a stronger evidentiary class.

## 2. Claim classes and promotion gates

| Claim class | Example | Minimum evidence | Not sufficient |
|---|---|---|---|
| `IMPLEMENTATION_OUTPUT` | 45 trees planted; hectares treated; tonnes of waste removed | As-built/activity records, scope, time, counting/measurement method | A project target or undated publicity image |
| `OPERATIONAL_PERFORMANCE` | EcoFilter uptime; monitoring-station availability; treatment-cell flow | Commissioning and operating logs, calibrated telemetry, QA and maintenance records | A system installation announcement |
| `OBSERVED_PHYSICAL_OUTCOME` | Tree survival; water-quality concentration/load; bank-erosion rate | Repeated calibrated measurements, explicit target estimand/support, uncertainty and reference design | Planting count, model projection, or unqualified before/after map |
| `ATTRIBUTABLE_INTERVENTION_EFFECT` | Estimated change caused by a restoration intervention | Credible baseline and counterfactual, controls/comparator, exposure measurement, confounder and spillover handling, uncertainty and sensitivity | A post-intervention difference or a BACI label without design diagnostics |
| `MODELLED_BENEFIT_PROJECTION` | SAVi 25-year avoided costs / benefit-cost ratio | Model identity/version, assumptions, scenario, parameter sources and uncertainty/sensitivity | A claim that the projected benefit has already occurred |

These classes are **not a single ordinal ladder**. A modelled projection is a different evidence product, not a weak observed outcome. It must not be silently promoted to one. Implementation and operational records may support observed outcome analyses, but they do not substitute for them.

### 2.1 Claim ceiling

For each claim, record the highest class justified by the admitted evidence. The agent may propose a class; an independent evaluator determines whether the evidence gates are satisfied. A claim may be:
- accepted at its submitted class;
- downgraded to a lower supported class;
- marked `CAUSALITY_NOT_IDENTIFIED`;
- marked `UNRESOLVED_SCOPE`;
- marked `INSUFFICIENT_OBSERVABILITY`;
- rejected for rights, provenance or integrity violations.

Do not hide the downgrade or delete the original submission. Preserve both with a reason code.

## 2.2 Crosswalk to current international restoration and NbS standards

The current external standards reinforce the distinction in this protocol and provide a disciplined way to make the outcome claim more useful.

### Society for Ecological Restoration (SER), Third Edition, June 2026

The SER third edition distinguishes three monitoring purposes:
- **Implementation/compliance monitoring:** whether planned actions were delivered.
- **Effectiveness/performance monitoring:** whether predefined objectives and indicators are being achieved.
- **Impact monitoring:** longer-term ecological, cultural and socioeconomic effects.

That maps naturally to the IOEQ claim classes, but causal attribution is an additional question: SER notes that comparing a restoration site with its baseline and reference model measures recovery, but does not by itself establish that the intervention caused it. For intervention effects, SER recommends pre/post data at a sufficient number of treated and control sites (BACI) and replication where possible. When this is infeasible, baseline change may be described, but not interpreted as causal without adequate design evidence.

SER's Five-star / Ecological Recovery Wheel framework assesses six attributes separately: absence of threats, physical conditions, species composition, structural diversity, ecosystem function and external exchanges. It recommends reference models, baseline inventories, measurable indicators and time-bound recovery objectives. IOEQ therefore records which ecological attributes a case actually measures. Do not collapse them into a single project score that can hide an attribute that worsened. Different attributes may have different recovery trajectories and evidence quality.

### IUCN Global Standard for Nature-based Solutions, Second Edition, October 2025

The IUCN standard has eight criteria and 27 indicators spanning societal challenges, integrated systems, biodiversity/ecosystem integrity, financial feasibility, inclusive governance, trade-offs/safeguards, adaptive management and sustainability/mainstreaming. IOEQ should use it as a complementary **intervention-design and governance crosswalk**, not as a substitute for sensor calibration, reference sampling or causal identification. An NbS can be well-designed and socially inclusive while its physical effect remains unmeasured; conversely, a physical measurement alone cannot establish equitable governance or distribution of benefits.

### FAO Framework for Ecosystem Restoration Monitoring (FERM)

FERM is the UN Decade on Ecosystem Restoration's operational monitoring/reporting framework and provides a geospatial platform, initiative registry and good-practice discovery. It is a useful interoperability target for project identifiers, geometry, intervention types and harmonized indicators. FERM-scale reporting of area under restoration is not a causal evaluation and must not replace local outcome measurements.

### Water-quality instrumentation

For the EcoFilter, the USGS continuous water-quality monitor guidance is a strong methodological comparator for station selection, sensor inspection, cleaning, calibration, record computation, review and reporting. It is not a South African legal requirement; use it as a technical reference and reconcile it with applicable South African standards and the actual instruments/laboratories used.

### Contract additions

The IOEQ case schema now records monitoring type, reference-ecosystem status, the six ecosystem attributes measured, whether indicators were preregistered, and causal-design type. This makes a project summary, a telemetry dataset, a restoration assessment and a causal-effect study machine-distinguishable before scoring.

## 3. Jukskei as a real intervention qualification case

The Jukskei catchment now offers several different intervention records:

### 3.1 SUNCASA programme outputs

The City of Johannesburg says SUNCASA launched in early 2024 and was officially concluded on 26 August 2026. Its closing summary reports:
- 489 ha restored along the Jukskei catchment;
- 47,600 trees planted;
- 664 tonnes of solid waste removed;
- 1,500 jobs across targeted regions;
- a continuous hydrological monitoring network for flood early-warning.

These are City-reported implementation totals. Primary as-built polygons, activity logs, waste measurement records, tree inventories and station registers have not yet been admitted to Sol-Atlas.

### 3.2 Different published totals are not automatically contradictions

IISD's SAVi assessment, published 18 March 2026, describes 469 ha of riparian restoration and 46,000 trees. An IISD Johannesburg profile also reports 469 ha / 46,000 trees. These were published before the City's August 2026 closing report and may use different as-of dates, boundaries, intervention definitions or counting methods.

Do not average, add or call these values erroneous until primary records establish:
- geography and geometry version;
- as-of / cut-off date;
- whether planned, treated, completed, maintained or verified areas are counted;
- whether newly planted, replacement or surviving trees are counted;
- measurement/counting protocol and uncertainty.

### 3.3 EcoFilter

The City describes an EcoFilter in the Upper Jukskei catchment, with modular 800-litre ecological treatment cells and continuous water-quality monitoring. It supports irrigation for community gardens. The City explicitly says it is not designed to provide potable water or resolve catchment-wide pollution by itself.

An installation description supports an implementation/operational claim only. A treatment-effect claim requires paired and time-aligned influent/effluent observations, flow context, commissioning date, operating and bypass logs, calibrated probes and/or laboratory measurements, method detection limits, maintenance history, and a defensible comparison.

Concentration changes alone can reflect dilution. Estimate pollutant mass loads only when flow and concentration measurements support them. Do not infer potable quality or catchment-wide improvement from the EcoFilter.

### 3.4 Jukskei Park 3 tree cohort

A separate report says 45 wild olive trees were planted at Jukskei Park 3 on 25 October 2026 by ASEZ volunteers with University of Johannesburg and Alexandra Water Warriors partners. Treat this as its own cohort and intervention episode unless primary records prove overlap with SUNCASA. A planting count is not survival, canopy establishment or ecosystem recovery. Follow-up needs as-planted coordinates, initial condition, species/provenance, watering, maintenance, replacements and repeated survival/health observations.

### 3.5 Modelled economic value

IISD's SAVi assessment reports projected 25-year net benefits of about USD 8.6 million and USD 3.06 in combined returns for each dollar invested. These are scenario-model outputs conditional on the model and assumptions. They are not measured cash flows, observed reductions in flood damage, measured health improvements or proof of realized environmental impact.

## 4. Causal effect qualification

An observed change and an attributable intervention effect are different claims.

A causal design must state:
- treatment/intervention exposure and dose/intensity;
- target population and estimand;
- time zero, implementation period, expected ecological lag and evaluation horizon;
- baseline and comparator/control units;
- pre-intervention trend evidence;
- concurrent works, upstream/downstream spillovers and interference;
- rainfall, flow, season, fire, land-use and other relevant time-varying confounders;
- missingness, observation availability and measurement-system changes;
- analysis model, assumptions, uncertainty intervals and sensitivity analyses.

Possible designs include replicated BACI, difference-in-differences, interrupted time series, matched controls, synthetic controls or randomized/staggered rollout where legitimate and feasible. The label of a design does not establish that its assumptions hold. A weak baseline or contaminated control lowers the claim ceiling.

### 4.1 Minimum causal diagnostics

- Inspect pre-intervention trends and timing.
- Explain selection of controls and target reaches.
- Test placebo dates/outcomes where meaningful.
- Quantify spatial spillover and interference.
- Account for repeated observations, spatial dependence and clustering by reach/event/acquisition.
- Evaluate sensitivity to plausible unmeasured confounding and control selection.
- Keep model uncertainty and measurement uncertainty distinct.
- Report null, adverse and inconclusive effects.

If these conditions are not met, report descriptive or associational physical change, not causal effect.

## 5. EcoFilter-specific measurement contract

For a treatment system, the measurement design should record:
- as-built process train, cell volume, hydraulic residence time and flow path;
- commissioning/configuration changes and operational hours;
- influent/effluent locations and paired sampling timestamps;
- upstream/downstream location and hydrological relation;
- flow, bypass, overflow and rainfall context;
- analyte, unit, field/lab method, detection/quantification limit, calibration, blanks, duplicates and chain of custody;
- laboratory accreditation/method validation where relevant;
- maintenance, media replacement, cleaning and downtime;
- uncertainty and rules for missing or censored observations.

The primary result should be analyte-specific concentration and, where flow permits, mass-load change with uncertainty. A single aggregate "water quality score" should not conceal an analyte that worsened.

## 6. Tree-cohort measurement contract

For each planting cohort, preserve:
- planting site and point/polygon geometry;
- date, species, provenance, initial height/condition and planting method;
- soil/site preparation, watering, protection and maintenance;
- mortality, replacement and survival inspection dates;
- health, growth and canopy metrics with protocol and observer;
- reference/comparison plot selection and land-manager permission.

Report survival with a defined denominator and follow-up window. Do not count replacements as original-cohort survivors. Do not infer catchment-scale biodiversity or bank-stability benefits from tree survival alone.

## 7. Evidence lineage, rights and reconciliation

Every evidence record binds:
- source-native identity, original publisher/operator and exact record/version;
- published date and as-of date;
- geometry/AOI, temporal interval and support;
- measurement/model/output classification;
- method, software, instrument and calibration;
- content digest and provenance lineage;
- dependencies/common-cause groups;
- uncertainty, missingness and reference independence;
- rights by purpose: access, analysis, derivative creation, publication, redistribution and model training;
- reviewer, claim ceiling, reason code and supersession/retraction links.

A report derived from a project dashboard does not become independent corroboration. A model using the same underlying data does not independently validate those data. An asset's presence does not prove service delivery.

## 8. Benchmark design

The public development corpus should include:
1. source-total reconciliation with different as-of dates/scopes;
2. modelled economic benefits mislabeled as measured physical outcomes;
3. an installed sensor/treatment asset with no admitted paired measurements;
4. planting count mislabeled as tree survival;
5. before/after observation without controls promoted as causal effect;
6. a control contaminated by upstream spillover;
7. sensor/season/processing shift confounded with intervention response;
8. unknown rights or provenance;
9. dependent reference labels;
10. an independently supported synthetic BACI effect;
11. negative/adverse outcome suppressed by a positive aggregate;
12. inadequate baseline / missing observations.

Report by claim class and failure family:
- false promotion rate from output → outcome and outcome → causal effect;
- source-total reconciliation and scope-resolution accuracy;
- causal false-positive rate on placebo/spillover controls;
- effect-estimator bias, interval coverage and power only when a valid truth/reference design exists;
- reference-independence/common-cause miss rate;
- provenance and rights completeness;
- correct downgrade / abstention rate;
- cost per qualified claim.

Do not report a single accuracy score as the headline. Synthetic claim-classification performance does not establish real environmental effects.

## 9. Composition with the wider Luminous architecture

- **Sol-Atlas:** intervention, observation, claim, source and outcome lineage; claim class and qualification state.
- **Symtropy:** physical forward models, hydrology/ecology predictions, model discrepancy and scenario outputs. A simulation remains a model output.
- **Symthaea:** competing explanations, causal hypothesis proposals and discriminating observation plans. A reasoning proposal is not evidence.
- **Mycelix:** source custody, rights, purpose grants, consent and legitimate coordination. It does not make a causal claim scientifically valid.
- **Independent verifier:** reproduces declared computation; replay authenticity is not source truth or causal identification.

Reuse PCIT (#845/#846) for sensor/processing/season invariance and #771 for equal-budget observation value. Do not duplicate those subsystems.

## 10. References

- City of Johannesburg, [SUNCASA closing outcomes](https://joburg.org.za/media_/Newsroom/Pages/2026-News-Articles/Johannesburg-concludes-multimillion-dollar-SUNCASA-climate-resilience-project.aspx), 26 August 2026.
- IISD, [SAVi assessment of nature-based solutions in the Jukskei catchment](https://www.iisd.org/publications/report/nature-based-solutions-johannesburg), 18 March 2026.
- IISD, [SUNCASA Johannesburg profile](https://www.iisd.org/suncasa/johannesburg).
- City of Johannesburg, [Upper Jukskei EcoFilter](https://joburg.org.za/media_/Newsroom/Pages/2026-News-Articles/Joburg-launches-innovative-eco-filter-system-to-address-pollution-in-Jukskei-River.aspx), 24 April 2026.
- Alex News, [45 wild olive trees planted at Jukskei Park 3](https://www.citizen.co.za/alex-news/news-headlines/local-news/2026/11/13/volunteers-plant-45-wild-olive-trees-along-the-jukskei-river-in-alexandra/), 13 November 2026.
- [Best Practices for Monitoring and Assessing the Ecological Response to River Restoration](https://doi.org/10.3390/w13233352) (2021).
- Smokorowski & Randall (2017), [Cautions on using the Before-After-Control-Impact design](https://doi.org/10.1139/facets-2016-0058).
- Gann et al. (2026), [International principles and standards for the practice of ecological restoration, Third Edition](https://doi.org/10.1111/rec.70441), June 2026. Use the 2026 edition, not only the 2019 second edition.
- IUCN (2025), [Second Edition of the Global Standard for Nature-based Solutions](https://iucn.org/press-release/202510/iucn-launches-second-edition-iucn-global-standard-nature-based-solutionstm), with eight criteria and 27 indicators.
- FAO, [Framework for Ecosystem Restoration Monitoring (FERM)](https://www.fao.org/ecosystem-restoration-monitoring/ferm/en), official UN Decade restoration monitoring and initiative registry.
- Wagner et al., USGS, [Guidelines and standard procedures for continuous water-quality monitors](https://doi.org/10.3133/tm1D3), Techniques and Methods 1-D3 (2006).
- Conservation Measures Partnership (2025), [Open Standards for the Practice of Conservation, Version 5.0](https://www.conservationstandards.org/wp-content/uploads/2026/03/CMP-Open-Standards-Report-v5.0-FINAL-v.-2026-03-25.pdf), monitoring, evaluation and learning framework.

## 11. Non-claims

This v0.1 protocol and synthetic corpus do not establish SUNCASA ecological impact, EcoFilter treatment efficacy, tree survival, flood-risk reduction, causal effect, or external validity. Public project summaries do not provide the primary implementation and monitoring data required for those claims.
