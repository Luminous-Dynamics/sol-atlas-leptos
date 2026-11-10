# Highveld household-service industry crosswalk v0.1

**Research cut-off:** 10 November 2026  
**Parent:** [HIGHVELD-HOUSEHOLD-SERVICE-001 #694](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/694)  
**Purpose:** define fair incumbent baselines and the narrow areas where the Luminous composition should be tested—not to claim superiority from feature lists.

## Executive finding

There is no credible basis for claiming that Luminous should replace Autodesk Forma, Revit, EnergyPlus, Home Assistant, or the relevant ventilation, thermal-comfort and water-reuse standards. They solve different parts of the problem and some have mature capabilities that must be used or compared directly.

The opportunity is a **cross-owner, service-outcome evidence layer** that composes these systems while preserving the exact design, weather, product, measurement, authority and lifecycle state behind each conclusion. This is a testable hypothesis, not a demonstrated industry advantage.

The comparison must distinguish four things:

1. **Capability parity:** can the candidate workflow perform the same task?
2. **Decision quality:** does it find better feasible alternatives under equal constraints and budgets?
3. **Evidence integrity:** can another party reconstruct why a decision was admissible and which exact inputs supported it?
4. **In-use outcome:** did the occupied dwelling deliver the measured service under the real conditions and operating boundary?

Success in one category does not imply success in another.

## Dated competitive and standards baseline

| Area | Established capability / reference | What it already does | Fair Luminous comparison |
|---|---|---|---|
| Early AEC design | [Autodesk Forma Building Design](https://www.autodesk.com/formabuildingdesign), [4 Nov 2026 release note](https://blogs.autodesk.com/forma/2026/11/04/whats-new-in-forma-building-design-indoor-daylight-parking-furniture-and-autocad-workflows/) | Schematic design, early daylight analysis (Indoor Daylight public beta), parking, furniture, DXF import, DWG export and Revit handoff. | Do not claim basic schematic generation, daylight, parking or CAD exchange as unique. Match the same brief, geometry, feature entitlement, human interventions and budget. Test whether resident service constraints and evidence survive later lifecycle transitions. |
| Connected AEC lifecycle | [Autodesk Forma / Revit strategy, 30 Jun 2026](https://adsknews.autodesk.com/en/views/forma-revit-evolution/), [Forma connected-workflow update, 15 Sep 2026](https://adsknews.autodesk.com/en/news/autodesk-forma-ai-aec-connected-workflows-2026/) | Autodesk is extending connected workflows across planning, design, construction and operations, with permissions-aware project context and AI-assisted documentation/review. | Do not reduce the incumbent to disconnected files. Compare against a real connected Forma/Revit workflow and a best-of-breed workflow. The hypothesis to test is owner-neutral evidence continuity across physical services, supplier lots, as-built configuration, consent and post-occupancy outcomes. |
| Whole-building physics | [EnergyPlus 26.2.0 release](https://energyplus.net/epupdate.htm), [OpenStudio](https://openstudio.net/) | Mature open-source whole-building energy and water simulation; EnergyPlus 26.2.0 was released 30 Sep 2026. | Use the qualified release as a specialist solver, not a competing engine to reinvent. Compare adapters, reproducibility, input fidelity, runtime, convergence, uncertainty and measured-model discrepancy. |
| Simulation software qualification | [ANSI/ASHRAE Standard 140](https://www.ashrae.org/technical-resources/bookstore/ashrae-building-energy-simulation-and-performance-modeling-resources) | Standard 140-2023 includes software capability tests and a weather-data-format test suite. It identifies predictive differences and software/input limitations. | Require the relevant Standard 140 cases from the solver owner. Passing Standard 140 is not calibration to a particular Johannesburg parcel or validation of every model output. |
| Thermal comfort | [ANSI/ASHRAE Standard 55-2023](https://www.ashrae.org/technical-resources/bookstore/standard-55-thermal-environmental-conditions-for-human-occupancy) | Standard and adaptive methods; considers air temperature, thermal radiation, humidity, air speed, clothing and activity. | Avoid “thermostat setpoint = comfort.” Measure/model the relevant variables and preserve occupant-specific exposure and preference. Comfort is separate from air quality, acoustics and illumination. This is a technical comparator, not automatically South African law. |
| Residential ventilation | [ANSI/ASHRAE Standard 62.2-2025](https://www.ashrae.org/technical-resources/bookstore/standards-62-1-62-2) | Residential minimum ventilation, local exhaust and source-control framework; the 2025 edition updates filtration and IAQ provisions. | AirCare must report actual outdoor-air delivery, room distribution, pollutant-specific particle removal, filter loading, humidity, noise, energy and maintenance. Meeting a minimum ventilation standard does not guarantee every IAQ outcome. It is a US consensus reference, not a substitute for South African approval. |
| Household automation | [Home Assistant: operation without internet](https://www.home-assistant.io/faq/works-without-internet) | Local controller can continue to operate devices using local protocols during an internet outage; cloud-only device features remain an exception. | Local operation is already available in the ecosystem. Test device/adapter qualification, exact resident authority, revocation, local fail-safe, ambiguity/reconciliation and independent physical outcome evidence—not merely local connectivity. |
| Greywater reuse | [ISO/CD 25446:2026](https://committee.iso.org/standard/90421.html) | Committee Draft under development for building greywater reuse, including source quality, collection, treatment, reuse, safeguards and monitoring. | Treat this as a draft reference, not a published standard or certification. WaterLoop must begin with source-separated, explicitly non-potable service; require physical segregation, locally qualified plumbing review and independent assays for any treatment claim. |
| Digital-twin trustworthiness | [NIST IR 8620, 21 Jul 2026](https://www.nist.gov/publications/digital-twins-workshops-summary-report) | NIST identifies interoperability, VVUQ, cybersecurity and workforce readiness as important digital-twin challenges. | Use these as explicit benchmark dimensions. A federation diagram or schema is not a qualified digital twin; test exact identity, scope, time, rights, data lineage, model limits, correction and invalidation. |

## Product-specific incumbent baselines

The underlying Symthaea product owners already identify several of the strongest comparisons. These must be carried into the Highveld composition rather than presenting their concepts as new.

### WaterLoop

Use three sequential baselines already specified in [WATERLOOP-APPL-001 #10323](https://github.com/Luminous-Dynamics/symthaea/issues/10323):

1. Metered storage plus leak monitoring, without treatment.
2. Source-separated capture/storage and physical diversion for one explicitly permitted non-potable use.
3. A treatment module only if the first two stages show a measured, decision-relevant benefit and a qualified process is available.

A real vendor comparison requires a dated Southern African market/supplier scan, exact product/system revisions, installation requirements, approvals, consumables and spares. Do not pretend one generic greywater unit is the universal incumbent. Compare the same source stream and end-use; meter every inflow, delivered volume, residual, purge, energy and maintenance event. ISO/CD 25446 remains a committee draft under development, not a final certification basis.

### HeatBank

A useful named market comparator already exists: **Cala's intelligent heat-pump water heater**. Cala publicly describes demand learning and adjustment using hot-water use, solar output, energy pricing, grid-emissions information and preferred operating hours; these are vendor product claims, not independent test results. See [Cala's product description](https://www.calasystems.com/), [how it works](https://www.calasystems.com/how-it-works) and [solar integration](https://www.calasystems.com/solar). Availability, installation, service support and local compliance in South Africa must be separately established before selecting it as a physical comparator.

The matched tournament should include:
- conventional heat-pump water heater without predictive coordination;
- a predictive/solar/tariff-aware heat-pump water heater such as Cala, if a comparable unit is accessible;
- heat-pump water heater with dedicated thermal storage;
- the proposed shared thermal-service hub, only after its interfaces and hygiene separation are engineered.

Compare the same inlet/ambient conditions and hot-water draw/temperature profile. Report accepted hot-water service, unmet draws, useful thermal energy, electricity (compressor, pumps, controls and backup), standing/distribution losses, peak demand, recovery, noise, maintenance and whole-life cost. Do not treat a vendor claim such as “up to 50% more hot water at peak” as a measured comparator endpoint.

[IEC 60379:2023](https://webstore.iec.ch/en/publication/61520) is a current method for measuring electric storage water-heater performance, but its stated exclusions and scope must be reviewed before applying it to a heat-pump architecture. It does not specify safety requirements. The test method must not be misrepresented as a heat-pump safety standard or assumed to qualify every HeatBank design.

### AirCare

Use the established comparison arms in [AIRCARE-APPL-001 #10330](https://github.com/Luminous-Dynamics/symthaea/issues/10330):

1. Named portable room air cleaner sized and independently verified by pollutant-specific AHAM CADR.
2. A measured residential ventilation baseline meeting the applicable project requirement.
3. A best-of-breed combined ventilation + filtration system, where available.

AHAM CADR reports separate smoke, dust and pollen clean-air delivery rates; its certified directory identifies models with third-party-verified results. See [AHAM's CADR method](https://ahamverifide.org/ahams-air-filtration-standards/) and [certified cleaner directory](https://ahamverifide.org/directory-of-air-cleaners/). CADR is not proof of outdoor-air ventilation, room distribution, gaseous-contaminant removal, humidity control or an occupied-room health outcome.

Match room geometry, openings, source/outdoor conditions, filter age, airflow, operating mode, noise and energy. Measure at multiple occupied positions. Keep each pollutant and service outcome separate; an aggregate AQI may hide a pollutant-specific failure.

### Additional 2026 research baselines

The May 2026 *Energy and Buildings* Volume 359 papers materially raise the baseline for our research programme. These are peer-reviewed case studies/methods, not universally transferable performance guarantees.

| Research | Demonstrated scope | How Highveld must compare |
|---|---|---|
| Andersen, Zhou & Dargaville, [Multi-variable least-cost optimisation of residential efficiency retrofits](https://doi.org/10.1016/j.enbuild.2026.117253), 15 May 2026 | Couples AccuRate building simulation with tariff/cost model and genetic algorithm; jointly searches envelope, HVAC, domestic hot water, PV and battery configurations over a 15-year annualized cost boundary. Melbourne detached-house case reports roughly 75% bill reduction and ~9-year payback for its selected conditions. | This rules out claiming that Luminous is novel merely because it co-optimizes shell + appliances + PV/storage or uses a genetic algorithm. Reproduce an incumbent-style joint least-cost baseline with equivalent functional service, tariff/capital assumptions and horizon. Add service continuity, resident-specific constraints, uncertainty, supply/repair and metrology, but do not claim the Melbourne savings transfer to Johannesburg. |
| Karanth, Loonen & Hensen, [Assessing implications of control integration in early-stage residential energy system design](https://doi.org/10.1016/j.enbuild.2026.117216), 15 May 2026 | Parametric TRNSYS study of PVT, heat pump and thermal storage in a Dutch terraced-house renovation. Five PV-self-consumption control strategies changed early design decisions; selected strategies reported up to 35% lower annual operational heating emissions than baseline controls in that case. | Controls cannot be a late afterthought. Compare static design with matched control-aware co-design; freeze forecasts, tariff/carbon data and information budgets. Separate policy benefit from physical envelope/product capability and test forecast error, outage behavior, comfort/service floors and rebound. Do not adopt 35% as a Highveld target. |
| Caieiro da Costa et al., [Optimized design solutions for multifamily housing: a multi-objective approach to thermal resilience under heatwave conditions](https://doi.org/10.1016/j.enbuild.2025.116886), 15 Feb 2026 | Multi-objective optimization using historical and future heatwaves, separating thermal resistance, robustness and recovery. The article highlights that a single resilience indicator does not capture the full resilience process. | This is a direct resilience baseline. Highveld should compare the same three stages and report time trajectories/uncertainty rather than only annual loads or a resilience badge. Keep its hot-climate scenario and multifamily boundary distinct from our Johannesburg synthetic 4–6-person dwelling. |
| Manmatharasan, Bitsuamlak & Grolinger, [High-resolution weather-guided surrogate modeling for data-efficient cross-location building energy prediction](https://doi.org/10.1016/j.enbuild.2026.117251), 15 May 2026 | Weekly weather-guided surrogate model with TCN, Transformer and convolutional-autoencoder alternatives; reports promising transfer to unseen sites from one or a few representative locations, with limited degradation across climate zones in its tested corpus. | This is a serious compute-efficiency comparator. Test a frozen surrogate on held-out stations and genuinely unseen climatic contexts, with uncertainty/OOD detection and high-fidelity EnergyPlus re-evaluation of selected candidates. Do not use cross-location transfer to promote an unqualified Johannesburg or parcel weather file. |
| Mitchell, Stephens & Heidarinejad, [Measurement and modeling of residential thermal resilience during a simulated outage](https://doi.org/10.1016/j.enbuild.2026.117221), 15 May 2026 | Three Chicago-area homes, measured cold-weather simulated heating outage, model calibration, DHR metric and thermal response characterization. | Use as a measurement-method comparator only. Preserve the exact published DHR formula/normalization before claiming replication; do not import the case's thresholds or temperatures as Highveld limits. See Symtropy #4507. |

### What these papers mean for Luminous

The differentiator must not be any one of: genetic search, whole-house energy-cost optimization, PV/battery sizing, model-predictive or predictive control, multi-objective heatwave optimization, or a weather-conditioned surrogate. Recent peer-reviewed work already covers these.

The harder and more defensible hypothesis is **evidence-constrained, resident-governed co-design across the building and delivered service**:

1. Preserve a per-resident and per-task service floor as a hard constraint, not a weighted objective.
2. Co-design physical envelope, thermal/electrical/water plant, control policy, repair access and supply/maintenance boundary.
3. Compare independent, shared-utility and hybrid/local-autonomy architectures under identical event traces and information budgets.
4. Use low-cost screening only where applicability is supported; escalate decision-critical cases to version-pinned specialist solvers and independent measurement.
5. Carry exact revisions, weather class, geometry, calibration, uncertainty, source rights, consent and outcome receipts through commissioning and post-occupancy.
6. Show the total lifecycle effect including unserved service, maintenance/repair labor, spares, replacement, noise, privacy, water quality and distribution of benefit.

The benchmark should be allowed to conclude that an incumbent is better for a particular service, climate or cost boundary. Luminous superiority requires matched evidence, not a feature matrix.

The building-surrogate comparison is tracked in Symtropy [BUILDING-WEATHER-SURROGATE-QUAL-001 #4509](https://github.com/Luminous-Dynamics/symtropy/issues/4509), under the generic multi-fidelity admission/escalation owner [#4487](https://github.com/Luminous-Dynamics/symtropy/issues/4487). Thermal-outage metrology is tracked in [#4507](https://github.com/Luminous-Dynamics/symtropy/issues/4507) with its reference arithmetic draft [PR #4508](https://github.com/Luminous-Dynamics/symtropy/pull/4508). These are new work items, not completed qualification.

## Research-frontier reset — 10 November 2026

### Optimization is established practice, not a novelty claim

A systematic review available online from 1 August 2026, *Multi-Objective Optimization and Decision-Making in Building Retrofits: A Systematic Review of Frameworks, Trends, and Research Gaps* (Building and Environment 2026, 115074; DOI: https://doi.org/10.1016/j.buildenv.2026.115074), reviews retrofit work from 2020–2026. It finds envelope retrofits dominate, followed by HVAC and renewables; it identifies SA/ML/MOO/MCDM integration as emerging but fragmented; and reports NSGA-II as the leading algorithm in this literature. The review highlights remaining issues including dynamic carbon, uncertainty, harmonized lifecycle assessment and resilience.

This changes the benchmark requirement: **NSGA-II is a necessary incumbent baseline**, not something Luminous can beat simply by using a different optimizer. Include:
- NSGA-II with declared operators, population, generation/evaluation budgets and stopping criteria;
- at least one strong alternative (e.g. MOEA/D, SMS-EMOA or an appropriately tuned CMA-ES/BO method for the problem structure);
- exhaustive enumeration as an oracle on small discrete problems;
- a transparent passive-first heuristic and a conventional expert-assisted workflow.

Report the optimizer's best-known front, exact-oracle regret/gap on small cases, hypervolume with declared reference point, IGD+, epsilon, feasible rate, evaluation count, wall time and high-fidelity confirmation cost. Do not tune the test suite to favor the Luminous candidate.

### Whole-house co-optimization and control are also existing baselines

The following research baselines are mandatory in any SOTA comparison:

- Andersen, Zhou & Dargaville (Energy and Buildings 359, 15 May 2026, 117253; DOI: https://doi.org/10.1016/j.enbuild.2026.117253) jointly optimizes envelope, HVAC, domestic hot water, PV and battery configurations using an AccuRate-based energy model, tariff/lifecycle-cost model and genetic algorithm. Therefore, shell + plant + PV + storage co-optimization is not novel by itself.
- Karanth, Loonen & Hensen (Energy and Buildings 359, 15 May 2026, 117216; DOI: https://doi.org/10.1016/j.enbuild.2026.117216) studies control integration during early-stage residential energy-system design for PVT, heat-pump and thermal storage. Therefore, control-aware design is not novel by itself.
- Caieiro da Costa et al. (Energy and Buildings, 15 Feb 2026, DOI: https://doi.org/10.1016/j.enbuild.2025.116886) separates thermal resistance, robustness and recovery under heatwave conditions. A single scalar resilience score is inadequate.
- Manmatharasan, Bitsuamlak & Grolinger (Energy and Buildings 359, 15 May 2026, 117251; preprint: https://arxiv.org/abs/2603.11121) studies weekly weather-guided surrogate prediction with TCN, Transformer and convolutional-autoencoder families, including cross-location transfer. Therefore, weather-conditioned surrogate modeling is not novel by itself; held-out site/time qualification and high-fidelity re-evaluation are essential.
- Mitchell, Stephens & Heidarinejad (Energy and Buildings 359, 15 May 2026, 117221; DOI: https://doi.org/10.1016/j.enbuild.2026.117221) combines field measurement, calibrated building modelling and outage resilience metrics across three homes. This is a direct comparator for the Highveld thermal metrology programme.

### Thermal-resilience method: DHR is a specific metric, not generic degree-hours

Al Mitchell's PhiusCon presentation on 8 October 2025 (slide 44) gives the Degree Hour Ratio in the form:

\`DHR = Σ(T_set − T_int) / Σ(T_set − T_out)\`

for the same outage period. Presentation: https://www.phius.org/sites/default/files/2025-10/al-mitchell-PhiusCon_2025-al-mitchell-PhiusCon_2025-Degree%20Hour%20Ratio%20A%20Low-Tech%20Test%20for%20Thermal%20Resilience.pdf

The formula's operational convention (positive-part clipping, above-setpoint samples, exact time weighting, missingness and denominator handling) remains under verification in Symtropy #4507. The generic degree-hour arithmetic in Symtropy PR #4508 is deliberately not labelled DHR. A future Highveld comparison must preserve both the published DHR and a broader metric vector: time-to-breach, breach duration/depth, hot/cold exposure, stable recovery, active-system contribution, energy and measurement coverage. Do not treat the Chicago study's thresholds or measured temperatures as South African criteria.

### South African XA Edition 3 and live regulatory status

SAIAT's current CPD page for SANS 10400-XA Edition 3 says Edition 3 was published in early 2026 and describes a material change in how energy-efficient hot-water requirements are satisfied. The presenter is identified as chair of SABS TC060 and a member of the workgroup responsible for the latest XA review. Source: https://cpd.saiat.org.za/sans-10400-xa-3rd-edition-cc (accessed 10 Nov 2026).

Treat this as credible professional/committee-chair confirmation, but still obtain the exact SABS catalogue record and official standard text, publication metadata, scope and effective date before making a project-specific compliance claim. Do not use the 2025 draft \`SANS 10400-XA:20XX Ed 3\` consultation notice as publication proof. The compliance source register is [Sol-Atlas #604](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/604).

The National Building Regulations and Building Standards Amendment Bill 2026 remains a public-participation proposal at this research cut. The official extension notice sets the comment deadline at **12 November 2026, 12:00**. Sources: https://www.gov.za/news/cabinet-statements/statement-cabinet-meeting-6-may-2026-07-may-2026 and https://www.gov.za/documents/notices/national-building-regulations-and-building-standards-act-extension-submission. Draft Bill provisions must not be applied as operative requirements.

### Resulting research hypothesis

The defensible Luminous hypothesis is not "we invented co-optimization, NSGA-II, AI building simulation, or thermal resilience." It is:

**Can a rights- and authority-aware evidence layer make specialist AEC and physics tools work together across a whole resident-defined service lifecycle—while improving robust decisions and measured service outcomes at an equal or lower total engineering/operating burden?**

The test must be allowed to show that an incumbent or hybrid workflow is better for a service, climate, building type or cost boundary. Publish wins, ties, losses, unsupported strata and exact versions.

## Physical integration bench comparators — 10 November 2026

The proposed unoccupied room-scale bench is tracked in [Symtropy #4531](https://github.com/Luminous-Dynamics/symtropy/issues/4531). Its fair measurement design needs to distinguish product method, lab repeatability, production variation and actual household use.

- **Appliance reproducibility:** IEC TR 63250:2021 covers repeatability, inter-lab reproducibility, uncertainty and round-robin testing, but explicitly excludes production-unit variation and household-use representativeness. Compose Symthaea #10341 to keep those evidence strata separate. [IEC scope](https://webstore.iec.ch/en/publication/65090)
- **HeatBank:** Australia's 2026 Decision RIS compares ISO 19967-1:2019 and EN 16147:2017+A1:2022, with defined draw profiles and ambient conditions; it reports hot-water delivery, reheat, standby/storage loss, energy and COP. IEC 60379:2023 addresses electric storage water heaters, not all heat-pump water-heater architectures, and does not specify safety. [Energy Rating Australia](https://www.energyrating.gov.au/industry-information/publications/decision-ris-energy-efficiency-heat-pump-water-heaters) · [IEC 60379](https://webstore.iec.ch/en/publication/61520)
- **AirCare:** ASHRAE 62.2-2025 is a minimum residential ventilation reference; it does not guarantee pollutant-specific IAQ or thermal comfort. Measure actual outdoor-air delivery and pollutant-specific occupied-zone results. [ASHRAE 62.2-2025](https://www.ashrae.net/technical-resources/bookstore/standards-62-1-62-2) · [scope](https://www.ashrae.org/technical-resources/standards-and-guidelines/titles-purposes-and-scopes)
- **WaterLoop:** ISO/CD 25446 remains under development and is not a final standard or local approval route. [ISO status](https://www.iso.org/standard/90421.html)
- **Building physics:** use analytical, empirical and comparative test classes; a software qualification result does not replace measured test-cell validation. [DOE test procedure overview](https://www.energy.gov/cmei/buildings/articles/test-procedures-building-energy-simulation-tools)

## South African applicability

International standards and products above are technical comparators, not automatic local legal requirements. Every real Highveld project must use the applicable South African editions, municipal requirements, professional determinations and product approvals.

The existing Sol-Atlas [South African building approval crosswalk #604](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/604) remains the source register. The package in this branch does not resolve open edition/effective-date/applicability questions and does not implement building-code checks.

Likewise, a synthetic Johannesburg/Highveld scenario must not be assigned a parcel weather file from a city name. Sol-Atlas #611 owns weather-source identity, rights, representativeness and event-class separation; Symtropy #1900 owns the EnergyPlus adapter/qualification boundary.

## Matched benchmark protocol

Every comparison freezes the same:

- user-visible service request and hard service floors;
- dwelling geometry/model revision and building-system/product revisions;
- weather, outage, occupancy and task traces, with exact bytes/digests;
- information available to each policy at each point in time (no future-data leakage);
- simulator/solver versions, numerical tolerances and high-fidelity call budget;
- human-time budget, tool entitlements and setup/cleanup effort;
- metering boundary, functional unit, observation horizon and exclusions;
- uncertainty, missingness, calibration and evidence class.

Report the incumbent's strongest supported workflow, a manual/control baseline, the Luminous composition and—where helpful—a hybrid workflow. Include failed runs and human intervention. Never use vendor marketing claims as measured performance.

### A. Architecture and option search

Metrics:

- time to first feasible candidate;
- feasible design diversity after removing color/material-only changes;
- hard-constraint false-PASS and false-reject rates;
- hypervolume, IGD+, generational distance and epsilon against exact small reference fronts;
- performance under weight/preference perturbation and uncertain inputs;
- stale-analysis and change-invalidation detection;
- IFC/IDS and geometry round-trip loss;
- human review time and number of corrective interventions.

A search algorithm only wins if final candidates are re-evaluated by the same qualified high-fidelity solvers. Report runtime and solver calls; do not tune on the scored holdout.

### B. Building physics and habitability

- relevant Standard 140 software capability cases;
- independent analytical/reference cases and convergence;
- calibration period separated from held-out validation period;
- hourly/dynamic thermal trajectory, overheating exposure and recovery;
- comfort assessment using the appropriate Standard 55 method and declared occupant assumptions;
- actual ventilation and pollutant-specific room measurements, not one composite AQI;
- predicted-versus-measured error, uncertainty interval coverage and out-of-domain abstention.

A lower annual kWh total is not a win if the design fails the declared service floor, is under-occupied, has worse overheating or relies on unreported occupant behaviour.

### C. Water, thermal and indoor-air services

**WaterLoop:** metered source withdrawal, capture, delivered usable non-potable volume, reject/residual route, mass-balance residual, energy, maintenance and independent quality assays for the specific intended use. No potable or pathogen-removal claim.

**HeatBank:** hot-water service volume/temperature profile, useful thermal energy, electricity by component, source/sink temperatures, storage stratification/loss, peak demand, recovery and service shortfall. No hygiene or pressure-vessel safety claim from a simulation.

**AirCare:** outdoor-air delivery, room distribution, named pollutant-specific concentration/removal, filter loading and pressure drop, humidity/condensation, sound at occupied locations, electrical use and filter service life. CADR or outlet readings alone are not equivalent to occupied-zone performance.

### D. Household service under disruption

For matched hot/cold, grid, water, outdoor-air, equipment, network, consumable and common-mode events, report:

- service delivered and unmet by service/resident;
- time to degradation and reserve depletion;
- duration below each declared service floor;
- time to first restored service and time to stable recovery;
- repeat-fault performance and reserve-capital recovery;
- manual fallback usability and operator intervention;
- repair/spare-part delay and whole-system recovery burden.

Do not multiply marginal hazard probabilities or assume independence without an independently justified model. Compound failures require explicit joint event semantics.

### E. Evidence, agency and lifecycle

- exact provenance and reproducibility;
- correction/revocation and stale-evidence behaviour;
- permission/consent scope and revocation correctness;
- personal data minimization and local-only operation;
- unauthorized command and authority-confusion rate;
- accessibility and resident-specific task outcomes;
- total labour for operation, maintenance, repair and administration;
- lifecycle cost, carbon, water and material flows with declared boundaries.

Keep each outcome as a vector. Rights, consent, accessibility, safety and service floors are non-compensatory; do not bury failures in a scalar weighted score.

## Proposed superiority gates

These are research gates, not achieved results or universal industry thresholds:

1. **Integrity:** zero false PASS across the frozen adversarial corpus; all exact source/revision and scope mismatches detected.
2. **Decision quality:** statistically and practically meaningful improvement over the strongest matched baseline on at least one preregistered primary objective, without worsening any hard constraint.
3. **Robustness:** the advantage survives held-out weather/event traces, model-family perturbation and plausible parameter uncertainty.
4. **Service:** measured service quality and continuity improve at equivalent functional unit and operating conditions—not just simulated annual totals.
5. **Lifecycle:** gains remain after maintenance, consumables, repair labour, embodied materials, replacement and end-of-life are included.
6. **Adoption:** integration, training, licensing, data governance and operational burden are competitive with the incumbent workflow.
7. **Independent reproducibility:** a third party can reconstruct the result from the released permissible inputs, exact source identities and procedure.

Publish wins, losses, ties, unsupported domains and the exact versions tested. A Luminous advantage may be narrow and domain-specific; that is more credible than a universal SOTA claim.

## Recommended strategy

- **Buy/use:** specialist CAD/BIM, geometry kernels, EnergyPlus/OpenStudio, accredited calculations, calibrated meters and qualified laboratory assays.
- **Bridge:** versioned adapters, requirements-to-evidence mapping, owner-native receipts, data/rights metadata and exact change invalidation.
- **Build:** household service contract, cross-service matched resilience benchmark, independent evidence verifier, and resident-authority composition where genuinely absent.
- **Do not build:** a replacement Revit, EnergyPlus, Home Assistant, building-code engine, generic sensor platform or a universal comfort/efficiency score.

The measurable objective is not “more integrated software.” It is less decision error and lower end-to-end burden while delivering equal or better verified household service.

## Claim ceiling

This crosswalk is desk research and benchmark design. It does not establish market-wide product performance, local regulatory compliance, building physics, service efficacy, safety, affordability, resident benefit or Luminous superiority. All products, features, editions and availability must be rechecked at the exact experiment date and entitlement.
