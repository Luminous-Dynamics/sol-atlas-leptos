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

### South African applicability

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
