# Johannesburg / Highveld Landscape Performance Pilot v0

**Status:** preregistration protocol draft. No site has been selected, no sensor data have been collected, and no outcome is established.  
**Owner:** Sol-Atlas coordinates the design/evidence contract; Symtropy owns qualified models; Symthaea performs read-only evidence reviews; Mycelix records authorized consent, decisions and stewardship.  
**Related:** [LAND-DESIGN-001](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/190), [LAND-BENCH-001](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/195), [LAND-METRICS-001](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/197).

## 1. Research principle: a design is a falsifiable intervention

Treat a landscape plan as a versioned hypothesis about measurable outcomes, not a set of attractive symbols. Each claim must name a metric, baseline, target/time horizon, method, required inputs, source digests, uncertainty and a falsification condition. Do not allow a renderer, AI critique or governance approval to turn “could help” into “has helped.”

The Landscape Architecture Foundation explicitly recommends selecting defensible metrics and methods that fit a project’s goals across environmental, social and economic benefit categories ([metric/methods guide](https://www.landscapeperformance.org/guide-to-evaluate-performance)). The pilot should add independent measurements and transparent failures to that practice, not replace site-specific expertise.

## 2. Do not pretend all scales are one experiment

Start with **one primary treatment typology at one consented, accessible pilot site**. Decide the exact site, intervention and primary outcome before looking at post-intervention values. The location is intentionally unselected here; exact land coordinates and participant details must not be added to public artifacts.

The protocol and shared metric contract span four scales, but each scale requires a distinct study design:

| Scope | Example intervention family | Primary evidence needs | What it cannot prove alone |
|---|---|---|---|
| Site / parcel | Shade trees, planting, rain garden, permeable surfaces, outdoor comfort | Surveyed geometry, soils, water pathways, species schedule, microclimate observations | District or watershed-scale impact |
| District | Street/tree-canopy network, shaded routes, connected public green space | Comparable network data, repeated heat/access observations, maintenance and access coverage | Whole-city equity effects without population and privacy design |
| Watershed | Runoff retention, infiltration, riparian restoration | Catchment boundaries, rainfall event records, drainage, soil/infiltration, flow measurements | Regional hydrological effect from a few isolated site sensors |
| Bioregion | Habitat connectivity and restoration mosaic | Habitat/species observations, land tenure/consent, landscape-scale disturbance and monitoring | Species recovery or causal ecological effects from satellite vegetation indices alone |

A parcel-level pilot can validate workflow and measurement contracts; it cannot qualify every scale’s ecological or hydrological models.

## 3. Study design

### Preferred design

Use a replicated Before–After–Control–Impact (BACI) design where feasible, with multiple pre-intervention observations, one or more matched comparison sites, consistent measurement methods, and post-intervention observations. Site selection, controls, exclusions, sample windows, primary metric, analysis method and the smallest decision-relevant effect must be preregistered before treatment.

BACI is intended to separate intervention effects from background temporal change and site differences, but its assumptions need checking; monitoring guidance recommends repeated baseline observations, representative time spans, multiple controls where feasible, and sampling aligned to ecological response time ([restoration monitoring review](https://doi.org/10.3390/w13233352); [BACI design and power](https://doi.org/10.1111/2041-210X.13287)). A simple before/after comparison without controls is exploratory and must not be described as causal.

### Baseline and follow-up

- Record repeated observations before construction/planting, spanning relevant local conditions. For a short pilot, one wet/dry seasonal cycle is a minimum operational starting point—not proof of interannual robustness.
- Continue monitoring through installation, establishment, and at least one relevant seasonal cycle afterward. Trees and habitat have longer response times; predeclare longer follow-up for survival, canopy, habitat and maintenance outcomes.
- Log rainfall, temperature, wind, drought/heat episodes, maintenance, irrigation, disturbance, nearby development and sensor downtime as potential confounders.
- Keep a contemporaneous baseline/control plan. If access to adequate controls or baseline measurements is impossible, downgrade the claim class to descriptive/observational and say what causal inference is not supported.

## 4. Metric protocol

Choose a small set of primary and secondary metrics for the selected pilot; do not try to optimize every metric simultaneously.

| Family | Candidate observation | Minimum method requirements |
|---|---|---|
| Heat / comfort | Near-ground air temperature, surface temperature and/or shade exposure at declared times | Calibrated paired sensors, placement/height/shading metadata, time synchronization, nearby weather covariates, missing-data rules |
| Water | Rainfall event and measured captured/outflow volume; soil infiltration where relevant | Calibrated rain/flow instruments, defined catchment boundary, event window and initial conditions, independent unit/mass balance checks |
| Ecology | Tree survival, species identity, establishment condition, crown growth or predeclared habitat indicator | Versioned taxonomy, field protocol, repeated observations, suitable local seasonal windows and independent quality review |
| Access / equity | Route availability/continuity and travel time to shade/refuge/services; optional consented user-reported comfort | Route graph and audit method, accessible-route standard/version, sampling frame, accessible consent and privacy-preserving aggregation |
| Materials / lifecycle | Material quantities, reuse fraction, embodied-carbon range and replacement/maintenance costs | Actual quantity take-off, factor/EPD version, declared life-cycle stages, base year/currency and sensitivity analysis |
| Stewardship / economics | Maintenance tasks completed, accepted role/cadence, operating cost and failures | Versioned obligations, accountable consented party reference, receipts/logs, review/expiry state and declared cost boundary |

“Tree symbol count” is not canopy cover. “Planting-bed symbol count” is not native biodiversity. A displayed path stroke is not a surveyed path centreline. Satellite-derived vegetation indices can support broad screening but do not by themselves prove local species presence or ecological recovery.

## 5. Preregistering thresholds and statistical power

Do not hard-code a universal improvement target into the product. Before intervention:
1. Define a primary outcome and the direction of benefit, plus harm and equity guardrails.
2. Estimate baseline variance and autocorrelation from pre-intervention/control observations.
3. Define the smallest effect that would change a real decision, and compute sampling/power needs with a documented method.
4. Fix inclusion/exclusion rules, sensor calibration schedule, outlier handling, missing-data treatment, and the analysis model.
5. Pin the preregistration and analysis-code digest in the evidence record before looking at post-treatment outcomes.
6. Report effect estimates, uncertainty intervals, sample coverage, attrition, adverse outcomes and sensitivity to model choices—not just a p-value or percent improvement.

If the data are too weak to distinguish the target effect from natural variability, report “inconclusive,” not success.

## 6. Source/CRS/license gate

Each proposed dataset must have an exact product/sheet/collection ID, acquisition/edition date, resolution, CRS and axis order, vertical datum if relevant, documented accuracy, no-data behavior, attribution, reuse licence/access terms and a SHA-256 digest of the acquired bytes. Store raw and processed artifacts separately.

The current Johannesburg/Highveld catalog is metadata-only and has seven candidate sources, zero ingested sources. CD:NGI topographic/orthophoto products remain license-review-required; no source is automatically downloaded, cached, rehosted or treated as construction-grade. Never make coordinate transformations from the canvas’s arbitrary local drawing units.

## 7. Privacy, consent and stewardship

Before site selection or monitoring, record the authority to access the land and collect each class of observation; land access, legal ownership, community consent, image consent and publication consent are distinct. Use Mycelix’s versioned consent/stewardship profile, but never interpret a consent record as proof that the site geometry or a scientific claim is true.

- Exact sensitive coordinates, participant identities, contact details and protected-species locations stay private by default.
- Publish coarse site context only after explicit authorization and risk review.
- Field observations include method, unit, timestamp, observer/role pseudonym where appropriate, uncertainty, source digest and access policy.
- Accepted maintenance obligations name an authorized role/reference, cadence, trigger, resource status and review date; an assigned task is not proof that it was completed.
- Withdrawal changes future access/publication state without pretending already distributed external copies can be recalled.

## 8. Benchmarks: product quality and field validity are separate

### Application/tooling benchmarks
- 100 / 1,000 / 10,000 / 100,000-feature frozen native scenes; same hardware/toolchain/seed/input digest.
- Native validation/serialization/hash p50/p95 and raw samples; browser edit/pan/zoom p50/p95, dropped frames, memory and accessibility task success measured separately.
- Zero coordinate reinterpretation; exact feature/property/CRS round-trip; zero silent source-license promotions.
- Blinded task comparison for matched workflows versus QGIS and suitable professional landscape tools, reporting completion, rework, interoperability errors and deliverable quality rather than raw FPS alone.

### Scientific benchmarks
- Calibration and known-answer tests for each physical model; independent hand-calculable or field reference cases.
- Compare predicted effects with held-out control/impact observations; report uncertainty coverage, bias, error by site/season and failures/out-of-domain cases.
- Track unsupported-claim rate, false-confidence rate, reproducibility from pinned inputs and the proportion of metrics correctly blocked by missing evidence.
- Keep UI/kernel performance, model validation, and field outcomes as three independent test layers.

## 9. Stop conditions

Do not advance from screening to analytical claims when any of the following apply:
- no authority/consent for site access, observation or publication;
- source licence or attribution is unresolved for the intended use;
- missing/invalid CRS, datum, geometry, units or sensor calibration;
- control/baseline quality cannot distinguish an intervention effect from temporal/site variation;
- required evidence is absent, stale or contradictory;
- safety-critical outputs have not been independently validated;
- an apparent improvement creates a material negative effect on another predeclared outcome or group.

The correct artifact in any of these cases is a reasoned blocker and next-evidence action, not a simulated green “success” badge.

## 10. Pilot exit criteria

A pilot may be called **measurement-qualified** only when:
- the preregistration and exact project/design digests are pinned;
- source terms, coordinate systems, instrumentation and methods pass review;
- raw and processed records have provenance and reproducible normalization;
- baseline/control completeness meets the preregistered design;
- uncertainty, null/negative results, harms and missing data are reported;
- independent review can reproduce the primary outcome from the pinned inputs.

“Measurement-qualified” is not synonymous with construction approval, universal validity, or proof of benefit at untested scales.
