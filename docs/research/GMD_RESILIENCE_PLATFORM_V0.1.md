# Geomagnetic Disturbance Resilience Platform — Research Architecture v0.1

**Status:** Research design / benchmark contract. Not an operational system.  
**Version:** 0.1.0  
**Date:** 2026-11-10  
**Owners:** Luminous Dynamics research programme; domain-owner participation required for any real infrastructure study.  
**Claim ceiling:** E0 design until the synthetic, analytical, reference and measured-evidence gates below are independently passed.

## 1. Purpose and central thesis

Build an open, evidence-qualified architecture that helps electricity and essential-service systems anticipate, withstand and recover from severe geomagnetic disturbances (GMDs), including plausible Carrington-class scenarios.

The objective is not to predict one exact future Carrington event, nor to promise that infrastructure will remain undamaged. The objective is to preserve **essential services under uncertainty** and reduce time, cost and inequity of recovery.

A planetary-scale event can degrade several coupled systems: high-voltage networks, transformers, GNSS, satellite services, HF radio, telecommunications, data centres, water pumping, fuel logistics, hospitals and food distribution. A useful resilience system therefore needs a chain from physical forcing to service outcomes, with uncertainty and evidence retained at each boundary.

**Core proposition:** move from storm-category awareness to a continuously updated, spatially grounded, asset- and service-conditioned view of risk and recoverability.

A G-scale alert is useful situational awareness. It is not a local transformer-risk estimate, an assurance of grid failure, or a command to operate equipment.

## 2. What changes the field: eight paradigm shifts

### 2.1 From storm magnitude to local induced-field and asset consequences

A single planetary index or geomagnetic latitude is too coarse to determine asset risk. Geoelectric fields depend on time-varying magnetic fields and the local Earth response, which in turn depends on conductivity structure. Induced voltage also depends on the orientation and path of a conducting line. Transformer design, winding configuration, grounding and network topology further affect current.

Treat the physical chain explicitly:

`solar-wind / geomagnetic forcing
 -> magnetic variation at ground
 -> Earth conductivity response
 -> geoelectric field E(t,x)
 -> line-integrated induced voltage
 -> quasi-DC network currents
 -> transformer GIC exposure
 -> qualified transformer / grid response
 -> essential-service consequence`

Each arrow is a typed, versioned handoff. Do not substitute a dashboard colour for any step.

### 2.2 From point forecasts to calibrated multi-model uncertainty

Use ensembles and model-discrepancy terms. Preserve disagreements between official forecasts, regional observations, 1-D Earth-response approximations, 3-D magnetotelluric transfer functions and operator models. A confident model outside its validity domain should abstain or widen its bounds.

Separate aleatoric variation, measurement error, parameter uncertainty, numerical error, model discrepancy and unknown data coverage. Do not compress them into one unexplained confidence score.

### 2.3 From asset protection to essential-service continuity

A transformer is important because of the services that depend on it. Map causal dependency chains: grid -> pumping -> potable water; grid -> telecom -> emergency dispatch; grid -> cold chain -> food/medicine. Keep the asset graph, service claims, affected populations, time horizon and adequacy thresholds explicit.

Measure service delivered, not just assets surviving. A site can be energised while water pressure, quality, coverage or hospital backup autonomy remains inadequate.

### 2.4 From reactive planning to precomputed robust action envelopes

Before an event, evaluate operator-approved options such as topology reconfiguration, load prioritisation, reactive-power reserve, transformer monitoring, islandable microgrids, storage and staged restoration. Produce a set of feasible, authority-bounded alternatives rather than one brittle optimum.

Optimisation must preserve critical-service floors and report tail outcomes, fairness and recovery time alongside cost. Any proposed switching sequence remains a study result until reviewed and authorised by the grid operator.

### 2.5 From cloud dependence to partition-tolerant federation

Assume that ordinary internet, cellular backhaul, satellite links, GNSS time and cloud services can be unavailable or degraded. Local nodes must continue within bounded, pre-authorised autonomy limits. Messages must preserve issuer, scope, time uncertainty, provenance, expiry and correction history. Reconnection must reconcile causally and expose conflicts.

Less externally verifiable freshness must never increase authority. A valid signature proves message origin/integrity under the key model; it does not prove the physical observation is true or that an action was executed.

### 2.6 From one universal simulator to qualified specialist models

Do not build a single solver that purports to simulate the Sun, magnetosphere, ionosphere, conductivity structure, GIC network, transformer saturation, AC grid dynamics, communications and society. Integrate specialist models through explicit contracts. Qualification attaches to a model, output, region, timescale and parameter envelope—not to a product name.

### 2.7 From mean-case optimisation to tail-risk and common-cause resilience

Evaluate correlated failures: shared corridors, common substations, common-mode transformer design, shared telecom paths, common fuel suppliers, common data feeds and correlated model errors. Independent-looking observations are not independent when they share sensors, calibration, model inputs or processing pipelines.

Report worst-region, worst-service and worst-event performance, plus the Pareto frontier for service continuity, cost, resource use, recovery and equity. Do not let a strong global average hide a catastrophic regional tail.

### 2.8 From impressive visuals to evidence and outcomes

A spatial view is valuable only when each layer says whether it is measured, inferred, modelled, simulated, stale, incomplete or unknown. Visual quality is assessed separately from physical validity. Every recommendation must be replayable from exact source/model/configuration revisions and must show the evidence that would falsify it.

## 3. Four-system division of responsibility

| System | Responsibility | Must not claim |
|---|---|---|
| **Symtropy** | Specialist physical/numerical work: Earth-response input contracts, E-field integration, line-voltage and quasi-DC GIC reference calculations, scenario simulation and downstream model coupling. | A global magnetosphere or transformer-damage solver without specialist equations, data and validation. |
| **Sol-Atlas** | Spatial asset/dependency graph, CRS and time-aware geospatial evidence, coverage gaps, network geometry, critical-service relationships, exposure and recovery resources. | That a map is complete, a site is safe, or a grid operator's asset model is authenticated merely because a feature exists. |
| **Symthaea** | Consequence-centric evidence assimilation, uncertainty and model-disagreement reasoning, calibrated risk assessment, counterfactual comparison and decision-support explanations. | Authority over solar physics, source truth, engineering limits, protection settings or physical operations. |
| **Mycelix** | Federated identity, scoped emergency information exchange, store-carry-forward, offline operation, conflict visibility, audit and causal reconciliation. | That signatures make false telemetry true, that partitions preserve global authority, or that the emergency Civic cluster is production-ready. |

**Authority boundary:** Symtropy computes; Sol-Atlas contextualises; Symthaea evaluates and proposes; Mycelix authenticates and coordinates. Infrastructure owners and qualified human operators retain operational authority. No component in this programme may directly actuate grid breakers, alter relay settings, shed load, dispatch generation or issue public emergency commands.

## 4. State of the art and fair industry comparison

The field already has serious operational and engineering capabilities. The programme should extend them and demonstrate matched improvements, not imply that existing tools are simplistic.

| Reference capability | What it already does well | Luminous opportunity / limitation to address |
|---|---|---|
| **NOAA SWPC operational products** | GMD forecasts and geoelectric-field estimates. The US/Canada 1-D product covers a broad region; an empirical 3-D EMTF product uses magnetotelluric transfer functions in more limited CONUS coverage. | Extend spatially where defensible local conductivity/MT and operator data exist; compare against SWPC products at matched sites and cutoffs. NOAA itself calls for more validation against E-field/GIC measurements. |
| **NERC TPL-007-4** | Defines North American GMD benchmark-event and transmission-planning assessment requirements, including a reference waveform. | Build a globally usable, geographically resolved consequence and service-continuity evidence architecture. Do not treat compliance with one regional standard as global resilience. |
| **ESA Space Weather Service Network** | Specialist support for power-system operators, including GIC monitoring and short-term forecasting. | Compose space-weather evidence with local asset dependencies, service impacts, offline coordination and recovery. |
| **Utility/operator models and measurements** | Use owner-specific network, transformer and grounding data; these are the essential comparators for any real deployment. May 2024 published measurements include 47 GIC sites and 17 magnetometer sites; at matched TVA locations, operator-derived GIC correlation exceeded 0.8, with reported prediction efficiency around 0.4–0.7. | Demonstrate whether regional field modelling and evidence composition improve out-of-area performance without weakening calibration. Never assume an open model beats an operator's proprietary model. |
| **BPA public GIC data** | Provides measured GIC time series at 11 transformer substations, useful for replay and method development. | Reproduce a transparent public baseline before claiming improved forecasting or physical realism. |
| **PowerModels.jl, PGLib-OPF, PyPSA, SimBench** | Strong comparators for AC/DC optimal power flow, power-system optimisation and chronological network/energy studies. | Use downstream of qualified GIC/transformer-stress models. They are not by themselves GIC induction, transformer saturation, harmonic, protection or transient-stability solvers. |

### Fair-comparison rules

1. Freeze the strongest admissible baseline before evaluating a new method.
2. Match exact input observations, spatial coverage, forecast cutoff, topology, Earth model, transformer parameters, solver formulation and hardware where runtime is compared.
3. Pin versions, licences, source digests, transformations, solver tolerances and time limits.
4. Compare against both official products and operator-native models when available.
5. Publish absolute performance, paired deltas, confidence intervals, calibration, failure strata and cost to reproduce.
6. A lower RMSE with worse interval coverage, a better objective with infeasible power flow, or a faster run with less complete physics is not an unqualified win.
7. No "beats SOTA" statement until independent parties can reproduce the same held-out comparison.

## 5. Physical model boundaries

### 5.1 Geomagnetic forcing and Earth response

Start from official geomagnetic observations and documented event products. E-field estimates require an Earth-response model. A 1-D layered conductivity model is a useful baseline, not a universal truth. Where measured magnetotelluric impedance/transfer functions exist, preserve their location, frequency range, uncertainty, calibration and validity domain. Do not silently fill spatial gaps with a 3-D-looking interpolation.

### 5.2 Induced voltage and network GIC

For an oriented network segment `i -> j`, the induced voltage is the path integral of the geoelectric vector field:

`V_ij(t) = integral_along_path E(t, x) · dl`

Sign, path geometry, coordinate frame and units must be explicit. The first Symtropy implementation should be a deterministic quasi-DC reference kernel for tiny networks, with an independent analytic oracle. Solve the declared resistive network including transformer winding and grounding paths under a documented convention. Keep per-phase, transformer-effective and neutral-current conventions distinct.

### 5.3 Transformer and grid response

GIC is an input to transformer response, not a damage verdict. Harmonics, reactive-power consumption, half-cycle saturation, hot-spot temperature, protection response and voltage stability require appropriate specialist models and/or measured/OEM evidence. Couple these outputs into AC power flow and dynamic security analysis as distinct qualified stages. Do not invent universal damage thresholds.

### 5.4 Service outcomes

Translate grid consequences into owner-defined, time-qualified service claims. For example, water service needs pressure, flow, quality, duration, demand and population coverage; hospital power needs critical-load coverage, transfer behaviour, fuel/battery autonomy and tested restoration. The evaluator may compose owner-approved thresholds; it must not invent them.

## 6. Benchmark programme

The benchmark is layered. Passing an earlier layer does not imply a later one.

### E0 — Contract and source integrity

- Versioned manifest; immutable dataset/model/version references and licences.
- Explicit units, coordinate systems, magnetic/geographic frame, timestamp/time-scale/cadence and interpolation policies.
- Separate measured, modelled, reconstructed, synthetic and unknown states.
- Strict schemas; reject unsupported fields, duplicate IDs, invalid units, missing required metadata and inconsistent temporal cuts.
- Exact source, model, transformation and configuration digests.

### E1 — Analytical and synthetic known-answer tests

Start with tiny hand-computable networks: a single straight line in a uniform E field, reversed line orientation, two-node loop, branched network, parallel paths and grounded multi-transformer network.

Negative controls: wrong CRS, wrong units, sign reversal, stale field, omitted branch, missing ground return, zero/duplicate path length, unsupported transformer connection and non-finite input. Positive controls are mandatory; a system that rejects everything has not passed.

**P0 targets:** zero unit/coordinate/sign errors in the frozen corpus; 100% exact known-answer agreement within predeclared numeric tolerances; zero silent component omission; zero false "no impact" outcomes when a required source, corridor, grounding path or asset parameter is missing.

### E2 — Differential reference-model benchmarks

Use pinned NOAA 1-D/3-D E-field products where geographic coverage overlaps; NERC TPL-007-4 reference event; a specialist GIC calculator or operator model; and independent open reference implementations where formulation and licence permit.

Report separately:
- E-field vector MAE/RMSE (V/km), direction error, frequency-dependent residuals and spatial-gradient error.
- Induced-voltage and GIC peak/time-series error, peak timing, correlation, prediction efficiency and site-stratified error.
- Power-flow residuals and branch/voltage/generator feasibility for downstream OPF; never relabel an OPF pass as GIC validation.
- Maximum absolute and normalised residuals, numerical convergence and unexplained differential.
- Data coverage and excluded/unsupported component counts.

### E3 — Measured event and geographic holdouts

Event families:
- March 1989 benchmark event (observational basis for TPL-007-4).
- October 2003 storms.
- May 2024 Gannon storm (measured US GIC/magnetometer corpus; NZ transformer response observations; UK response/model comparisons).
- Additional public BPA measured GIC series where terms permit.
- 1859 Carrington and May 1921 only as **uncertain historical reconstruction ensembles**, not exact ground-truth waveforms.
- Synthetic scenarios spanning dB/dt, conductivity, line orientation/length, grounding resistance, transformer parameters, topology and missingness.

Split by event and geography, not random adjacent time rows. Use rolling-origin cutoffs for forecasting and hold out stations/regions/source models. Report sample counts, event-clustered confidence intervals, calibration and worst-stratum errors.

### E4 — Service-continuity and recovery scenarios

Simulate compound failures across electricity, water, telecom, healthcare, fuel, food and transport. Use owner-defined service floors and resource limits. Compare:
- current documented operator/municipal contingency plans;
- no-action / naive threshold rules;
- deterministic nominal optimisation;
- robust or distributionally robust planning;
- multi-objective Pareto planning with critical-service floors and explicit equity constraints.

Metrics: critical service served-energy/time, unserved essential demand, longest interruption, recovery-time distribution, resource feasibility, spare-part and crew constraints, tail loss, region/population inequity and regret against an offline clairvoyant upper bound. The clairvoyant result is a theoretical bound, not an operationally achievable baseline.

### E5 — Offline federation / exercise

Extend Mycelix SOV-DPI-005 with incident-message scenarios under 1, 6, 24 and 72-hour partitions; node restart; stale/revoked credentials; clock uncertainty; duplicate/reordered bundles; false but correctly signed observations; split-view/equivocation; and partial reconciliation.

Measure message durability/loss, priority delivery latency, duplicate suppression, causal ordering, conflict visibility, reconciliation convergence, bytes/storage/CPU and unauthorised authority/disclosure. Target zero authority escalation and zero unauthorised disclosure. Report physical service continuity only from independently verified service-owner outcomes—not successful message delivery.

## 7. Proposed targets (not achieved results)

Targets below are project proposals, not existing benchmark standards or current Luminous performance.

| Dimension | Initial target after corpus freeze | Qualification note |
|---|---|---|
| Analytical physics | 100% of frozen small-case known answers pass; zero unit/sign/CRS errors | E1 only; independent oracle required. |
| Missing evidence safety | Zero false no-impact / low-risk conclusions in negative controls | Fail closed or abstain; incomplete asset inventory is not evidence of safety. |
| E-field/GIC prediction | At least 10% lower held-out GIC MAE than strongest admissible baseline, while maintaining calibration and no predeclared major stratum regression >10% | Aspirational; only meaningful with matched measured sites/events and a frozen test set. |
| Uncertainty calibration | Nominal 90% intervals achieve 85–95% empirical coverage on adequately sized held-out strata | Report sample sizes and confidence bands; small samples remain inconclusive. |
| Transformer response | No damage-threshold claims until matched measured/OEM validation exists | Physics/thermal specialist review required. |
| Critical service | Meet the service-owner's pre-registered minimum floor in each declared exercise scenario | Never infer a generic "hospital-safe" or "water-safe" state. |
| Offline communications | 72-hour partition test with zero authority escalations, zero unauthorised disclosure and deterministic conflict visibility | Does not guarantee operation under arbitrary physical destruction. |
| Reproducibility | Exact replay from pinned corpus/model/config/source digests; independent verifier receipt | Queued/skipped CI is not a pass. |

The 10% MAE target is not a universal superiority threshold. If a matched corpus is too small, measurements are not independent, or baseline model inputs differ, report "insufficient evidence" instead of relaxing the protocol.

## 8. South Africa and African pilot

South Africa is a strategic starting point because SANSA's Space Weather Warning Centre is based at Hermanus and provides an established regional scientific and warning capability. Use its products as one input family.

The pilot should begin with a **data-owner and rights matrix**, not an invented complete asset map:
- SANSA: space-weather products, magnetometer coverage, availability, cadence, access and reuse terms.
- Eskom / relevant transmission and distribution owners: permissioned network topology, transformer metadata, grounding, GIC/neutral-current measurements, protection studies and contingency procedures.
- Universities / geophysical institutions: local MT surveys, conductivity models, transfer functions, geophysical uncertainty and data-use rights.
- Water boards, municipalities, hospitals, telecoms and fuel/food operators: dependency edges, critical service thresholds, backup autonomy and restoration constraints.

This research pass has not established a sufficiently dense, open South African transmission GIC/transformer measurement corpus or national 3-D MT transfer-function coverage. That is an evidence gap, not a low-risk result. Do not infer low risk from geographic latitude alone.

Start with synthetic/abstracted network cases and public scientific data. Bring in owner data only after explicit permission, classification and privacy review. The first real pilot should be read-only and co-designed with the relevant utility and engineering professionals.

## 9. Data and reference-source register

All dataset ingestion must independently check version, licence, data rights, field units, temporal resolution, coverage and source integrity.

| Source | Use | Important boundary |
|---|---|---|
| NOAA SWPC Geoelectric Field Models | 1-D US/Canada and geographically limited 3-D EMTF E-field products; archived E-field and geomagnetic products | NOAA documents coverage and validation limitations; not a globally complete model. https://www.spaceweather.gov/products/geoelectric-field-models-1-minute |
| NOAA SWPC 3-D empirical EMTF method | Local Earth-response/MT transfer-function model | Preserve MT site coverage and transfer-function validity. https://www.spaceweather.gov/products/geoelectric-field-1-minute-empirical-emtf-3d-model |
| USGS, *Mapping a Carrington Storm* (2025) | Methodological benchmark for spatially varying extreme-event E-field reconstruction | CONUS study; historical event is reconstructed, not directly measured. https://www.usgs.gov/publications/mapping-a-carrington-storm |
| May 2024 US GIC measurements (Space Weather, 2026) | 47 GIC sites and 17 magnetometer sites; measured vs operator/reference models | Pin paper, data supplement and allowed reuse; preserve site/time holdouts. https://doi.org/10.1029/2025SW004758 |
| May 2024 New Zealand transformer study | GIC, harmonics and reactive-power observations | Site and asset scope only; not universal transformer thresholds. https://doi.org/10.1029/2024SW004235 |
| BPA GIC monitoring | Public measured GIC series at 11 transformer substations | Verify terms, station/site coverage and data vintage before use. https://www.bpa.gov/energy-and-services/power/operation/geomagnetically-induced-currents |
| EarthScope MT / EMTF repositories | Geophysical transfer-function and conductivity evidence | Site-specific; requires quality, frequency and coverage review. https://www.earthscope.org/data/ |
| SANSA Space Weather | African regional monitoring, forecasts and advisories | Product use and archival access should be confirmed with SANSA. https://spaceweather.sansa.org.za/ |
| NERC TPL-007-4 | North American reference benchmark and planning standard | Regulatory comparator; not a global operational rule. https://www.nerc.com/globalassets/standards/reliability-standards/tpl/tpl-007-4.pdf |
| PGLib-OPF / PowerModels.jl / PyPSA / SimBench | Downstream network optimisation and power-flow comparator families | Match formulations; none alone establishes GIC or transformer physics. https://github.com/power-grid-lib/pglib-opf · https://docs.pypsa.org/ · https://simbench.de/en/ |

## 10. Immediate implementation sequence

1. **Freeze evidence and contracts:** manifest schema, sources, units/CRS/time conventions, geographic coverage, negative controls and claim ceiling.
2. **Symtropy P0:** pure deterministic quasi-DC GIC reference kernel plus independent hand-computable oracle. No renderer dependency.
3. **Sol-Atlas P0:** ingest the typed field/network/asset outputs, preserve spatial/temporal provenance and expose missing coverage; do not implement a second physics solver.
4. **Symthaea P0:** baseline and uncertainty evaluation harness; source-native product comparisons before a cognitive predictor is admitted.
5. **Mycelix P0:** incident envelope, durable signed store-carry-forward and reconciliation tests composed with SOV-DPI-005.
6. **Cross-stack replay:** a frozen end-to-end synthetic scenario with exact digests and a receipt; only then add open measured event replays.
7. **Pilot partnership:** seek SANSA, utility, university and essential-service-owner review before any real topology or operational data is used.

Separate work into reviewable increments. Prefer exact analytical and reference-model evidence before sophisticated HDC/CfC inference or 3-D visualisation.

## 11. Claim and safety ladder

- **E0 — Design:** architecture, requirements and source inventory only.
- **E1 — Synthetic / analytical:** exact contract tests and known-answer synthetic fixtures.
- **E2 — Reference differential:** matched comparisons with pinned specialist implementations and published benchmark cases.
- **E3 — Measured physical validation:** held-out real geomagnetic/E-field/GIC/transformer measurements, independent review and stated regional applicability.
- **E4 — Operational shadow / exercise:** permissioned owner data, cybersecurity and safety review, operator assessment and measured exercise outcomes. Shadow mode only; no actuation authority.

No tier by itself establishes regulatory compliance, resilience against all Carrington scenarios, or the safety of any real grid. A model valid for one event, region, output or timescale is not automatically valid elsewhere.

## 12. Cross-stack work items

- [Sol-Atlas GMD-RESILIENCE-001 #692](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/692)
- [Symtropy GMD-PHYSICS-001 #4461](https://github.com/Luminous-Dynamics/symtropy/issues/4461), under [NATURAL-HAZARDS-000 #2232](https://github.com/Luminous-Dynamics/symtropy/issues/2232)
- [Symthaea GMD-INTELLIGENCE-001 #11358], separate from [GEO-INFER-001 #7688](https://github.com/Luminous-Dynamics/symthaea/issues/7688)
- [Mycelix GMD-OFFLINE-COORD-001 #6018], extending [SOV-DPI-005 #1030](https://github.com/Luminous-Dynamics/mycelix/issues/1030)
- Existing downstream network benchmark: [Sol-Atlas ENERGY-NETWORK-QUAL-001 #332](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/332)

## 13. Bibliographic notes

- NOAA Space Weather Prediction Center, *Geoelectric Field Models* and *1-minute Empirical EMTF 3D Model*.
- USGS (2025), *Mapping a Carrington Storm*.
- NERC, *TPL-007-4 — Transmission System Planned Performance for Geomagnetic Disturbance Events*.
- Space Weather (2026), DOI 10.1029/2025SW004758, May 2024 US GIC and magnetometer measurements.
- Space Weather (2024), DOI 10.1029/2024SW004235, transformer response observations during the May 2024 storm.

---

**Versioning rule:** this file defines a research protocol. Changes to test splits, datasets, metric definitions, targets or expected outcomes after seeing holdout results must be a new versioned amendment with an explicit rationale. Never tune the test set to make a model pass.
