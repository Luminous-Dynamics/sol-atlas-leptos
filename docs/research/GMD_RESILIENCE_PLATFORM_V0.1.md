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
A concrete operational benchmark now exists: during the May 2024 Gannon G5 storm, Transpower New Zealand implemented a preplanned targeted line-disconnection strategy. Published retrospective analysis reports that currents at the Dunedin hotspot peaked around 100–113 A under the mitigation procedure, while the no-mitigation counterfactual was estimated near 200 A; supply to consumers was maintained. This is a much stronger benchmark than a synthetic optimizer-only demonstration ([Mac Manus et al. 2025](https://doi.org/10.1029/2025SW004388)).

Hardware mitigation also has network-wide side effects. Neutral capacitive blockers can reduce DC current through a protected transformer but divert current to other transformers; industry research describes a “whack-a-mole” risk and concerns about overvoltage/fault protection. A 2026 study also examines interactions between neutral-blocking devices, inverter-based PV sources and distance-relay behaviour ([Mac Manus et al. 2023](https://doi.org/10.1029/2023SW003533); [Piesciorovsky et al. 2026](https://doi.org/10.1016/j.epsr.2025.112190)).

The mitigation-planning work is tracked in [Sol-Atlas GMD-MITIGATION-PLANNING-001 #703](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/703). It evaluates network-wide redistribution, AC/protection feasibility and service continuity—not merely the reduction at a single transformer. A plan that makes the target asset safer but moves unacceptable stress to another asset is not a successful mitigation.


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
### Operational R2O and system-level risk comparators

- **UK SWIMMR:** nine research-level models were moved into operational code for the Great Britain high-voltage grid, high-pressure gas pipelines and rail network. The programme added variometer coverage and a three-year magnetotelluric campaign across 53 sites, combining L1 solar-wind inputs, ground observatories, empirical machine learning and numerical MHD forecasting. This is a mature research-to-operations comparator, not just a model paper ([Beggan et al. 2025](https://doi.org/10.1029/2025SW004364)).
- **USGS coupled physics–engineering–economic risk model (2026):** a US example estimated daily economic losses of about $1.81B for a 250-year GMD scenario from transformer thermal heating alone, with disruption affecting approximately 5.1 million people and 135,000 businesses. The authors label this a conservative lower bound that excludes voltage collapse, cascading failures and restoration costs. The lesson is that service and economic consequences must be explicit benchmark outputs, not an afterthought ([USGS, 4 September 2026](https://www.usgs.gov/publications/major-space-weather-risks-identified-coupled-physics-engineering-economic-modeling)).
- **New Zealand operational mitigation:** the May 2024 Transpower line-reconfiguration strategy gives us a real counterfactual for evaluating mitigation and service preservation; it is tracked in [Sol-Atlas #703](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/703).


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
Recent measured work reinforces that transformer response cannot be represented by a universal current threshold. Feng et al. (2026, DOI [10.1029/2026SW004966](https://doi.org/10.1029/2026SW004966)) report a roughly 7 A effective-GIC headroom and approximately 0.115 MVAr/A response for one 220 kV single-phase autotransformer, with different behaviour for a nearby three-phase unit. Liu et al. (2026, DOI [10.1016/j.epsr.2026.113139](https://doi.org/10.1016/j.epsr.2026.113139)) report a lagged transient saturation response under GIC pulses. These are reasons to model winding configuration, effective ampere-turns, pulse duration and thermal/magnetic history—not fleet-wide thresholds.

The follow-on transformer-response work is [Symtropy GMD-TRANSFORMER-DYNAMIC-001 #4528](https://github.com/Luminous-Dynamics/symtropy/issues/4528). It composes the GIC reference kernel with the existing transformer thermal qualification work [PWR-TX-HST-001 #2899](https://github.com/Luminous-Dynamics/symtropy/issues/2899), rather than creating a competing thermal engine.


GIC is an input to transformer response, not a damage verdict. Harmonics, reactive-power consumption, half-cycle saturation, hot-spot temperature, protection response and voltage stability require appropriate specialist models and/or measured/OEM evidence. Couple these outputs into AC power flow and dynamic security analysis as distinct qualified stages. Do not invent universal damage thresholds.

### 5.4 Service outcomes

Translate grid consequences into owner-defined, time-qualified service claims. For example, water service needs pressure, flow, quality, duration, demand and population coverage; hospital power needs critical-load coverage, transfer behaviour, fuel/battery autonomy and tested restoration. The evaluator may compose owner-approved thresholds; it must not invent them.

### Evidence from the May 2024 measured-data study

The 2026 Wilkerson et al. study is now the first pinned real-data baseline for this programme:

- Four-site TVA operator replay: the independent exploratory replay in [GMD_MAY2024_TVA_REPLAY_V0.1.md](GMD_MAY2024_TVA_REPLAY_V0.1.md) reproduces the published Table 1 (r^2) and polarity-qualified PE values at Bull Run, Montgomery, Union and Widows Creek.
- 47 measured GIC sites and 17 magnetometer sites were assembled. The filtered comparison of the generic reference model against measured GIC covered 43 sites; Appendix A1 is the next exact-reproduction target ([Symtropy #4521](https://github.com/Luminous-Dynamics/symtropy/issues/4521)).
- The paper explicitly describes the generic reference model as a fallback when detailed real-time network configuration (in-service lines, line resistances and substation circuit particulars) is unavailable. TVA's site model uses utility-specific representation; it is therefore a stronger baseline for those four sites.
- The study finds weak average GIC time-series correlation between sites (~0.3 across 1,081 pairs), and even sites under 10 km apart can have correlations near zero or one. This is a strong warning against nearest-neighbour GIC interpolation or latitude-only risk maps.
- The study's physical chain reinforces the architecture: derive (mathbf{E}(t)) from measured/modelled (mathbf{B}(t)) and Earth-response data; integrate (mathbf{E}(t)) along transmission-line paths to obtain induced voltages; then use line/transformer resistances, grounding and actual connectivity to solve network GIC.

This changes our prioritisation: qualify *local conductivity + line geometry + actual network topology + transformer grounding* before adding more elaborate learned forecasting. A learned model may fill bounded gaps or quantify discrepancy, but it must not replace missing physical evidence with a confident interpolated field.

The branch also includes two separate follow-on research records:
- [South African data access/evidence matrix](GMD_SOUTH_AFRICA_DATA_ACCESS_MATRIX_V0.1.md), tied to Sol-Atlas [#697](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/697). It identifies SANSA SANDIMS, legacy Grassridge/Matimba, Botrivier under-line observations and MT evidence, while explicitly separating academic access from commercial data rights.
- [Adaptive residual calibration pilot](GMD_ADAPTIVE_CALIBRATION_V0.1.md), tied to Symthaea [#11370](https://github.com/Luminous-Dynamics/symthaea/issues/11370). This is a post-hoc, past-only within-event diagnostic: it improves some May 2024 site forecasts but does not transfer universally to the GMU reference model and is not independent-event evidence.

The reproducible offline Rust verifier is separately tracked in Sol-Atlas [#699](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/699); the 43-site appendix replication is Symtropy [#4521](https://github.com/Luminous-Dynamics/symtropy/issues/4521).

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
South Africa is not a blank research region. Published work includes Grassridge/Hermanus modelling (2008), finite-element modelling using multiple observatories (2015), data-driven transfer functions/network parameter ensembles (2020), and regional MT surveys such as SAMTEX with over 700 sites. SANSA's research portal also describes MT inversion work using 10 stations. These are distinct data/model families; deep lithospheric conductivity maps do not automatically constitute storm-frequency near-surface Earth-response transfer functions.

The South African model-family comparison is tracked separately in [Sol-Atlas SA-GIC-SOTA-BASELINE-001 #696](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/696); source rights and historical measurement access are tracked in [#697](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/697). Do not assume that a published paper, research project page, or public station metadata grants access to raw GIC or utility network data.


South Africa is a strategic starting point because SANSA's Space Weather Warning Centre is based at Hermanus and provides an established regional scientific and warning capability. Use its products as one input family.

The pilot should begin with a **data-owner and rights matrix**, not an invented complete asset map:
- SANSA: space-weather products, magnetometer coverage, availability, cadence, access and reuse terms.
- Eskom / relevant transmission and distribution owners: permissioned network topology, transformer metadata, grounding, GIC/neutral-current measurements, protection studies and contingency procedures.
- Universities / geophysical institutions: local MT surveys, conductivity models, transfer functions, geophysical uncertainty and data-use rights.
- Water boards, municipalities, hospitals, telecoms and fuel/food operators: dependency edges, critical service thresholds, backup autonomy and restoration constraints.

SANSA's current research portal lists a new project titled “Inversion of South African magneto-telluric data to determine conductivity profiles.” This is a concrete institutional collaboration lead for a South African conductivity/MT workstream, not evidence that the underlying survey data, transfer functions, geographic coverage or derived profiles are publicly released or ready for grid modelling. Request data extent, frequency coverage, processing/quality records, uncertainty, permissions and a path to joint validation from SANSA and relevant academic partners. This research pass has still not established an openly accessible, sufficiently dense national transmission GIC/transformer measurement corpus or a nationally complete 3-D MT transfer-function product. That remains an evidence gap, not a low-risk result. Do not infer low risk from geographic latitude alone.

**South African prior art makes this a real replication opportunity, not a greenfield assumption.** Ngwira (2008) used GIC data and Hermanus geomagnetic observations to improve layered Earth-conductivity modelling; Matandirotya, Cilliers and van Zyl (2015) used finite-element conductivity modelling, magnetic observations from two South African observatories and a coefficient-based network model, comparing with measured GIC during the late-October 2003 storm. Heyns et al. (2020) developed data-driven transfer functions and network-parameter ensembles to quantify uncertainty from sparse magnetometer and node measurements; Heyns (2021) examined operational modelling and uncertainty across the coupled chain. SAMTEX collected MT data at more than 700 southern-African sites, with published 3-D inversions. These studies provide candidate baselines and a local scientific lineage; their data, methods and intended-use limits must be independently reconstructed before reuse.

SANSA's specific current project description says it will use surface impedances from 10 South African MT stations, interpolate impedance-derived conductivity to locations without MT stations to estimate the E-field needed for power-line GIC, and compare against SAMTEX transects. This is a concrete collaboration path, but it does not imply access to the underlying 10-station data or a complete operational national field product.

Start with synthetic/abstracted network cases and public scientific data. Bring in owner data only after explicit permission, classification and privacy review. The first real pilot should be read-only and co-designed with the relevant utility and engineering professionals. The SANSA MT-inversion project is a specific initial contact for the conductivity-data pathway.

## 9. Data and reference-source register

All dataset ingestion must independently check version, licence, data rights, field units, temporal resolution, coverage and source integrity.

| Source | Use | Important boundary |
|---|---|---|
| NOAA SWPC Geoelectric Field Models | 1-D US/Canada and geographically limited 3-D EMTF E-field products; archived E-field and geomagnetic products | NOAA documents coverage and validation limitations; not a globally complete model. https://www.spaceweather.gov/products/geoelectric-field-models-1-minute |
| NOAA SWPC 3-D empirical EMTF method | Local Earth-response/MT transfer-function model | Preserve MT site coverage and transfer-function validity. https://www.spaceweather.gov/products/geoelectric-field-1-minute-empirical-emtf-3d-model |
| USGS, *Mapping a Carrington Storm* (2025) | Methodological benchmark for spatially varying extreme-event E-field reconstruction | CONUS study; historical event is reconstructed, not directly measured. https://www.usgs.gov/publications/mapping-a-carrington-storm |
| May 2024 GIC public data / SWERVE repository | Measured 1-second TVA series, TVA operator 1-minute model, GMU reference-model outputs, magnetometer and geospace-model data; the four-site replay is recorded in [GMD_MAY2024_TVA_REPLAY_V0.1.md](GMD_MAY2024_TVA_REPLAY_V0.1.md) | Pin repository commit and per-file blobs; the Zenodo record declares MIT, but check embedded source-provider terms individually. https://doi.org/10.5281/zenodo.20090223 · https://github.com/lucywilkerson/SWERVE-2024-05-10 |
| May 2024 US GIC measurements (Space Weather, 2026) | 47 GIC sites and 17 magnetometer sites; measured vs operator/reference models | Pin paper, data supplement and allowed reuse; preserve site/time holdouts. https://doi.org/10.1029/2025SW004758 |
| May 2024 New Zealand transformer study | GIC, harmonics and reactive-power observations | Site and asset scope only; not universal transformer thresholds. https://doi.org/10.1029/2024SW004235 |
| BPA GIC monitoring | Public measured GIC series at 11 transformer substations | Verify terms, station/site coverage and data vintage before use. https://www.bpa.gov/energy-and-services/power/operation/geomagnetically-induced-currents |
| EarthScope MT / EMTF repositories | Geophysical transfer-function and conductivity evidence | Site-specific; requires quality, frequency and coverage review. https://www.earthscope.org/data/ |
| SANSA Space Weather / Geomagnetism research portal | African space-weather monitoring and a listed South African MT-conductivity inversion research project | A collaboration lead only; confirm observation availability, MT survey coverage, transfer functions, uncertainty, access and reuse terms. https://spaceweather.sansa.org.za/ · https://atlas.sansa.org.za/projects/theme/geomagnetism |
| South African GIC prior-art corpus | Ngwira (2008), Matandirotya et al. (2015), Heyns et al. (2020), Heyns (2021 thesis), and SAMTEX 3-D MT inversions (>700 sites) | Candidate baselines, not evidence of present-day public grid/GIC access. Reconstruct source versions, event windows, station/site metadata and data-use rights. https://doi.org/10.1029/2008SW000408 · https://doi.org/10.1002/2014SW001135 · https://doi.org/10.1016/j.epsr.2020.106546 · https://open.uct.ac.za/items/23955613-3ada-48b6-951c-232eaf8d75b8 · https://doi.org/10.1029/2021JB023117
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
- [Sol-Atlas GMD-MAY2024-REPLAY-001 #695](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/695) — four-site measured/operator/reference baseline reproduction.
- [Sol-Atlas SA-GIC-SOTA-BASELINE-001 #696](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/696) — replicate and compare South African 1-D, 3-D MT/FEM and data-driven GIC model families.
- [Symtropy GMD-PHYSICS-001 #4461](https://github.com/Luminous-Dynamics/symtropy/issues/4461), under [NATURAL-HAZARDS-000 #2232](https://github.com/Luminous-Dynamics/symtropy/issues/2232)
- [Symthaea GMD-INTELLIGENCE-001 #11358](https://github.com/Luminous-Dynamics/symthaea/issues/11358), separate from [GEO-INFER-001 #7688](https://github.com/Luminous-Dynamics/symthaea/issues/7688)
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


---

## 14. State-of-the-art and regional evidence update — 2026-11-10

This update corrects and extends the original v0.1 research survey. It does not change the claim ceiling or imply a new Luminous physical model has been validated.

### 14.1 Operational industry is more mature than a single-model comparison suggests

The UK Space Weather Innovation, Measurement, Modeling and Risk (SWIMMR) programme has operationalised nine models into robust codes for GIC estimation across the Great Britain high-voltage transmission network, high-pressure gas pipelines and railways. It added three variometer sites and undertook a three-year MT campaign at 53 sites, combining ground observations, L1 solar-wind inputs, empirical ML and numerical MHD. This is a strong research-to-operations comparator, not merely an academic paper: https://doi.org/10.1029/2025SW004364.

The September 2026 Fujian MM-CNN-MHA study is a more recent single-substation forecast comparator. It reports 60-minute-ahead GIC forecasts at one 220-kV substation. Direct GIC prediction reports 0.27 A RMSE in its model comparison; predicted-E-field followed by a physical GIC calculation gives 0.26 A; persistence is 0.28 A rounded. The study also reports the observed 3.28 A peak was predicted at 1.03 A by the field-mediated forecast and 0.6 A by direct GIC prediction, due in part to very sparse high-amplitude samples. It explicitly compares event-threshold detection, not just point error: https://doi.org/10.1029/2025SW004900.

**Implication:** Luminous should not aim to out-CNN a one-site model. The credible differentiator is independently reproducible, multi-event and cross-site qualification; uncertainty and tail calibration; physical model transfer limits; common-cause dependency analysis; and the translation to critical-service continuity.

### 14.2 Add a sustained-current hazard class

A July 2026 Space Weather case study documents GIC up to approximately 20 A for roughly 90 minutes during an otherwise moderate New Zealand geomagnetic storm. It argues that sustained moderate currents can saturate transformer cores and create harmonic distortion, separate from short-duration peak/thermal risk: https://doi.org/10.1029/2026SW005032.

The benchmark must therefore contain two orthogonal stress families:
- impulse/peak exposure: peak, rise rate, timing, short-duration saturation and peak-related response;
- sustained/cumulative exposure: duration above asset-specific thresholds, integrated current exposure, cumulative thermal state, harmonic duty and recovery/cool-down.

Do not invent universal transformer damage thresholds. The transformer owner/OEM defines qualified curves and limits.

### 14.3 Model complete conductor topology

A 2025 study of a Portuguese network found that including shield wires changed simulated substation grounding-resistor GIC by about -24%, transformer-winding GIC by about +11%, and line current by about +2.5% for that particular system: https://doi.org/10.1016/j.epsr.2025.111540.

These values are not general correction factors. They demonstrate that shield wires and grounded return paths can materially alter the circuit model. The Symtropy contract must represent their connectivity, grounding and impedance explicitly, or mark the component unsupported rather than silently omit it.

### 14.4 South African evidence: correct the regional baseline

South Africa has a documented history of GIC observations and modelling:
- Ngwira et al. (2008) used Grassridge GIC data with Hermanus geomagnetic observations to derive a multilayer conductivity model and improve modelling for that particular site pair: https://doi.org/10.1029/2008SW000408.
- Ngwira et al. (2009) found model performance degraded with increasing magnetometer-to-GIC-site distance and the calibrated model was specific to a station/site pair: https://doi.org/10.1029/2009SW000478.
- GIC monitoring in southern Africa was reported to have started in 1998 through the EPRI Sunburst system. The 2008 work used 2-second Grassridge GIC averaged to one minute for the October 2003 event. The 2009 work cautioned that limited monitoring and station separation reduce transferability.
- SANSA's 2015/16 Annual Report documents magnetometers installed under/near a 400-kV line near Botrivier, recording GIC during the 17 March 2015 storm: https://www.sansa.org.za/wp-content/uploads/2018/05/SANSA-AR-2015-16.pdf.
- Nahayo et al. (2019) used the SANSA geomagnetic network, local MT impedance and measured GIC to study the Saint Patrick's Day 2015 event. It describes spatial variation in southern African E-fields and local/global index differences: https://doi.org/10.17159/sajs.2019/5204.
- Nahayo et al. (2022) used Grassridge and Matimba GIC observations with Hermanus and Hartebeesthoek magnetometers for the 2003 and 2015 storm periods; it notes the two utility sites did not provide good-quality GIC records over the same periods: https://doi.org/10.1029/2022SW003092.
- A 2026 African geomagnetic-proxy study includes Hermanus alongside four other African observatories, showing station- and event-dependent local GIC-proxy behaviour: https://doi.org/10.1016/j.asr.2026.04.031.

Therefore the South African gap is **not** an absence of prior measurements or science. The unresolved issue is access to continuous, current, permissioned and independently replayable records with exact sensor metadata, GIC site definitions, MT uncertainty, transformer/topology revisions, and owner-approved redistribution/analysis terms. The next data-source work is [GMD-SA-HISTORICAL-DATA-001 #697](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/697).

### 14.5 New experimental result: causal adaptive calibration is a hypothesis, not a win claim

The May 2024 TVA four-site replay now reproduces the publisher's four TVA and four GMU reference-model rows from a pinned dataset. The separate [adaptive calibration pilot](GMD_ADAPTIVE_CALIBRATION_V0.1.md) explores past-only rolling bias/gain correction. It reduces error at the four TVA sites in a single-event prequential test but slightly worsens the GMU reference results at two sites. This is model- and site-dependent. The follow-up work is [GMD-ONLINE-CALIBRATION-001 #11370](https://github.com/Luminous-Dynamics/symthaea/issues/11370).

This experiment is deliberately not called SOTA: its hyperparameters are post-hoc, all observations are from one storm, and within-event temporal blocks are not independent events. It motivates a preregistered multi-event and cross-site benchmark.

### 14.6 Updated benchmark requirements

In addition to Section 6's original metrics, benchmark reports must include:
- event-based POD, POFD, HSS, threshold precision/recall, false-alarm rate and lead time for predeclared GIC/E-field thresholds;
- peak magnitude/time error and quantile/tail calibration, not only overall RMSE;
- duration above asset-specific, owner/OEM-qualified thresholds and time-integrated exposure proxies;
- model performance stratified by sensor site, local geology, conductor orientation, shield-wire/ground-return topology, transformer type and event morphology;
- leave-one-event-out, leave-one-site-out and geographic transfer evaluations;
- calibration/abstention under sensor polarity, clock, topology, grounding, model revision and observability changes.

A result that improves average pointwise RMSE while missing rare high-current peaks or sustained moderate episodes is not an operational improvement.


### 14.7 First-party data access routes and an operational feed comparator

A concrete SANSA data access path is now identified: **SANDIMS** (https://sandims.sansa.org.za/). SANSA describes it as the repository for geomagnetic, ionospheric and magnetospheric data from its instrumentation network, including sites across South Africa and Namibia. Instrument metadata includes GPS location, manufacturer, principal investigator and sampling interval; the portal supports date-range searches and temporary-FTP data bundles. SANSA states academic research access is free subject to registration, while commercial use requires a contract. For Luminous Dynamics' commercial work, this is a rights gate—not a detail to defer until deployment. SANDIMS is a magnetic/space-environment data source; it does not automatically provide GIC transformer measurements, owner network topology or permission for operational derivatives. See [GMD-SA-HISTORICAL-DATA-001 #697](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/697).

A second public operational-format comparator is Bonneville Power Administration's rolling GIC data endpoint: https://transmission.bpa.gov/business/operations/gic/gic.txt. BPA documents measured transformer-neutral currents from 11 substations and a five-minute refresh; the linked TSV endpoint declares a rolling four-day time window and its update timestamp. This is useful for testing ingestion, source freshness, timestamp, gap handling and rolling-window snapshot logic. It is **not** a pinned historical benchmark: capture immutable dated copies with file digests, preserve the retrieval time, and verify reuse terms before publishing derivatives.

A 2026 North Island New Zealand study also demonstrates why regional magnetic-input transfer needs its own validation. It combines new long-period MT data (56 sites from the 2022–2023 survey) and older MT data with local OAKS and distant EYR magnetometer inputs. For the May 2024 Gannon event at Henderson T1, the 24-hour OAKS-driven result reports correlation 0.85, performance parameter 0.98 and RMSE 2.47 A, compared with EYR 0.55 / 0.77 / 7.19 A. The authors report that distant EYR input (>200 km south) can overstate local modeled current and distinguish this from a realistic local observation. This is an especially useful benchmark for source-station distance, spatial gradients and model-transfer validity: https://doi.org/10.1029/2026SW004997.

**New research requirement:** measure spatial-input transfer error independently of circuit-solver error. For every event/site, record the actual magnetic-input station, distance and spatial coverage; compare nearest/local station, regional interpolation and distant reference station; propagate source-location mismatch into E-field and GIC uncertainty. A model cannot report a site-calibrated field outside its supported spatial footprint without an explicit out-of-domain disposition.


### 14.8 South African transformer design and monitoring evidence

A public eTenders copy of Eskom transformer specification 240-68973110 Rev 2 contains a conditional GIC compatibility clause. For units with solidly earthed neutrals where GIC compatibility is specified in Schedule AB, it describes a 10 A neutral-terminal withstand criterion for 30 minutes under specific operating conditions and requires an external neutral CT proposal for in-service measurement. The copy is classified Controlled Disclosure, warns that downloaded versions are uncontrolled and lists a February 2023 next-review date. Treat it as a procurement/design lead only; obtain the current authorised Eskom revision and verify applicability to actual units before using it as a requirement or test threshold: https://www.etenders.gov.za/home/Download/?blobName=88a96272-1959-484f-818d-f2716e313c57.pdf&downloadedFileName=Eskom_Spec_ST_240-68973110_Rev_2%5B1%5D.pdf.

This gives a concrete South African utility data question: which in-service transformers have neutral CTs intended or suitable for quasi-DC GIC measurement, what are their sampling/bandwidth/polarity/calibration characteristics, and are readings archived with exact winding, grounding, topology, transformer revision and thermal/harmonic context? It does not establish that the fleet has those CTs or that the spec remains current.

Keep neutral-terminal current distinct from common-winding and series-winding effective GIC. An autotransformer's neutral measurement is not automatically equal to every winding's quasi-DC current; use the exact winding topology and ratio under a qualified transformer model.


## 15. Forecast benchmark extension

The forecast task is now separated from historical event reconstruction. See [GMD_FORECAST_BENCHMARK_V0.1.md](GMD_FORECAST_BENCHMARK_V0.1.md) and [Symthaea GMD-FORECAST-EVENT-QUAL-001 #11423](https://github.com/Luminous-Dynamics/symthaea/issues/11423).

The 2026 MM-CNN-MHA single-substation 60-minute forecast is an important comparator, but it is not directly comparable with the May 2024 four-site event-reconstruction replay. The benchmark must match issue-time inputs, sensor definition, site, horizon, event splits and physical-model boundary. In particular, any event used to fit empirical GIC coefficients cannot also be labelled an independent holdout.

The target is not a larger neural network in isolation. It is held-out, calibrated, multi-event and multi-site prediction whose physical and service consequences are independently checked. The benchmark RFC defines the model tournament, leakage controls, tail metrics, transfer tests and proposed gates; none of those gates are achieved results yet.
