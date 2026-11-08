# Highveld Keep — South African resilience binding v0.1

**Owner:** [HIGHVELD-RESILIENCE-BINDING-001 #630](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/630)  
**Parent:** [HIGHVELD-KEEP-001 #607](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/607)  
**Companion package:** [Highveld computational basis #608](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/608)  
**Manifest:** `HIGHVELD_KEEP_RESILIENCE_BINDING_V0_1.json`  
**Schema:** `HIGHVELD_KEEP_RESILIENCE_BINDING_V0_1.schema.json`  
**Research cut-off:** 8 November 2026  
**Evidence tier:** E0 integration contract; no real climate file or physical model admitted.

## 1. Purpose: connect existing owners, do not reproduce them

Highveld Keep already identifies the important South African climate and disruption families. Symtropy #1848 already owns time-varying habitability trajectories, paired event traces, resident-level service floors, stable recovery and non-compensatory ranking. Symtropy #1900 owns the EnergyPlus adapter qualification; #1790 owns solver-neutral modelling boundaries; #1892 owns decision-relevant measurement selection. Sol-Atlas #593/#611 own regional climate context and weather-source qualification.

This package adds the missing **South African binding layer**: it maps the Highveld candidates and evidence gaps to those owner contracts. It does not copy their event oracle, paired-outcome evaluator, Pareto engine, weather pipeline or thermal solver.

The central unit is a *paired scenario trace*, not a render or an annual scalar. Every design alternative in one comparison must consume the same exact exogenous trace identity, revision, digest, time basis, timestep and horizon. Candidate policies may use only observations available at the decision time; future-event leakage is forbidden.

## 2. South African weather classes must remain distinct

The five catalogued 2011–2025 Gauteng TMYx candidates are preserved separately in the manifest. Their file hashes are null, file-specific reuse rights are Unknown, and station/site representativeness is not established. Catalogue presence is not qualification.

| Evidence class | Appropriate use | Not a substitute for |
|---|---|---|
| TMY/TMYx | Typical-year screening after exact-file qualification | Heatwave, cold extreme, design storm, future climate |
| Actual meteorological year / observed episode | Historical event and operating-year comparison | Future projections or parcel measurement without representativeness evidence |
| Design condition | A declared engineering sizing/check basis | Typical-year energy or historical event record |
| Future climate | Scenario sensitivity with model/ensemble/pathway/period/grid/quantile | Observed weather or an unlabeled “2050 weather file” |
| Site measurement | Local comparison within instrument/time/placement scope | Regional representativeness or other variables not measured |

The Climate.OneBuilding South Africa catalogue lists the five candidate files, but its source page currently carries an “All Rights Reserved” notice and no file-specific redistribution permission was located in the existing research. Therefore, this adapter keeps hashes null and rights Unknown until #611 obtains lawful terms and inspects the exact archives. Do not mirror or package those files on the strength of public download access alone.

Station identity also remains a first-class uncertainty. WMO/NOAA/SAWS coordinates and instrument histories must be reconciled over their effective dates; a station code in a filename does not prove that the sensor remained at one coordinate or shared the same exposure for the whole record.

## 3. Nine Highveld stress families

The manifest separates typical-year screening, observed hot spells/hot nights, cold nights, future climate, grid outage during heat, water interruption, severe weather, air-quality-constrained ventilation, and component/control fault with repair delay.

Each scenario is presently `BLOCKED` or `UNKNOWN` pending its exact source, trace, model and scope inputs. A synthetic deterministic event sequence may test the Symtropy evaluator mechanics, but it must carry an explicit synthetic-fixture label and never be represented as observed South African weather or utility behaviour.

No numeric comfort threshold, outage duration, water autonomy, pollutant limit, wind design pressure, rainfall intensity, or resident service floor is invented here. Such values must be attributable to an applicable standard, qualified model, source record, resident-authorized requirement or appointed professional. Otherwise they remain null/Unknown and cannot support an admissible design PASS.

## 4. Resident services remain non-compensatory

The adapter preserves separate service domains: resident task-hours by person/task, thermal trajectory, essential electricity, potable/non-potable water, ventilation/indoor air quality, accessibility/egress, and repair/stable recovery.

Thresholds are deliberately Unknown in v0.1. The household profile is synthetic; actual resident requirements have not been collected. Requirements must not be inferred from demographic proxies. Dissent and separately held preferences must remain visible.

The paired comparison is admitted only after:
1. exact common exogenous trace binding;
2. current candidate/design/geometry/material/occupancy/solver revisions;
3. complete required inputs and explicit units;
4. solver convergence, warnings, validation scope and uncertainty declaration;
5. per-resident/per-task hard floors;
6. owner-native evidence and claim ceiling.

Only then may the Symtropy #1848 owner output be used to compare the remaining candidates. The adapter does not perform local dominance or choose a universal winner.

## 5. Risk-proportionate VVUQ

NIST AMS 100-85 (published 5 October 2026) recommends aligning computational-model credibility effort with model influence and decision consequence, and explicitly treats uncertainty propagation and empirical metrology together. This supports a staged assurance ladder:

- Software capability: use applicable ASHRAE Standard 140-2023 tests, including weather-format interpretation, under lawful access. This tests software capabilities; it does not calibrate a Highveld project model.
- Input/scope validation: exact geometry, construction, systems, material, weather, schedules, occupancy, units and boundary conditions.
- Numerical verification: convergence, warnings, missing outputs, numerical residuals and repeatability.
- Independent reference comparison: analytical, accepted reference or measured evidence in the declared applicability domain.
- Calibration and uncertainty: frozen calibration/held-out split, residual distributions, sensitivity, interval coverage where justified, and out-of-domain abstention.
- Professional/field review: separate scope-specific review and measured evidence for consequential claims.

The official EnergyPlus release register identifies version 26.2.0, released 30 September 2026. It is a sensible *version-pinned candidate* for the first whole-building thermal adapter, not an already qualified Highveld engine. The adapter must record the exact platform package, binary digest, inputs, weather, schedules and output artefacts once selected.

## 6. Value of information: measure only what can change the decision

The manifest identifies five decision-relevant unknowns:
- station identity, relocation and local exposure;
- real parcel survey, orientation and microclimate;
- resident-stated task schedules and comfort/service priorities;
- local outage and water-continuity evidence;
- EnergyPlus residuals and held-out validity for the declared domain.

The stopping rule is decision-based: do not collect measurements simply because more data are available. Prioritize an observation when its possible outcomes can change hard-gate admissibility, remove an uncertainty-driven ranking reversal, or change which candidate remains on the feasible frontier. Without calibrated likelihoods, do not report expected monetary value of information.

## 7. Industry comparison: use the right baselines

| Baseline | What it already does | Luminous comparison boundary |
|---|---|---|
| Autodesk Forma / Revit | Connected site/building design, early analysis and detailed BIM workflows | Do not claim layout generation or BIM handoff as unique; measure evidence and revision continuity |
| NLR BEopt 3.1.1 + EnergyPlus 25.2 | Residential discrete-option sweeps and cost/energy optimization | Use exact matched dwelling inputs; replace obsolete bundled costs with current sourced South African costs |
| EnergyPlus 26.2.0 + ASHRAE 140-2023 | Whole-building thermal/energy modelling and software-capability tests | Compare only within the qualified model envelope; Standard 140 is not project calibration |
| IBPSA BOPTEST | Standardized building-control emulators, APIs and KPIs across test cases | Relevant to operational-control comparison, not a substitute for envelope/design geometry qualification |
| IFC 4.3.2.0 + IDS 1.0 | Open asset exchange and alphanumeric information requirements | Preserve IFC/IDS compatibility; independently test geometry, trajectories and physical performance |

The Luminous hypothesis is not “better than EnergyPlus” or “better than Forma.” It is that an interoperable, evidence-carrying workflow can preserve the identity and validity of inputs and claims as an architectural choice moves through climate analysis, outage testing, repair planning and maintenance. That proposition remains unproven until a matched experiment is run.

## 8. Required next gates

**G0 — Binding integrity:** schema and semantic checks, exact owner contract versions, nine scenario identities, five unqualified weather candidate identities, and all Unknown/null statuses preserved.

**G1 — Evidence admission:** #611 obtains exact files under lawful rights, inspects headers/period/units/quality, reconciles station histories and determines representativeness. No parcel station is selected until a surveyed site exists.

**G2 — Model qualification:** #1900 completes the version-pinned EnergyPlus 26.2.0 software/input/reference gates. Then establish a Highveld-specific calibration/held-out dataset and applicability domain. Use ASHRAE Standard 140-2023 as a software capability suite, not as whole-project validation.

**G3 — Paired trajectory:** submit at least two exact candidate revisions against the same qualified event traces to Symtropy #1848; preserve per-resident service floors, uncertainty and stable recovery. Require the Symtropy exact-head verifier receipt.

**G4 — Field/authority:** only independently reviewed, appropriately measured and professionally scoped outcomes may support real building, health, statutory or safety claims.

## 9. Synthetic and physical claim ceiling

This is an E0 South African adapter contract. Schema conformance or a synthetic Symtropy oracle PASS establishes only that the crosswalk is internally structured and that the declared evaluator semantics are bound. It does not establish Johannesburg climate representativeness, indoor temperature, thermal comfort, health impact, energy/water autonomy, accessibility, structural/fire/weatherproofing safety, statutory compliance, construction feasibility or SOTA superiority.

## Sources checked 8 November 2026

- NIST AMS 100-85, published 5 October 2026: https://doi.org/10.6028/NIST.AMS.100-85
- EnergyPlus 26.2.0 official release, 30 September 2026: https://github.com/NatLabRockies/EnergyPlus/releases/tag/v26.2.0
- ASHRAE/IBPSA Standard 140-2023 resource index: https://www.ashrae.org/technical-resources/bookstore/ashrae-building-energy-simulation-and-performance-modeling-resources
- BEopt 3.1.1 release details (May 14, 2026): https://www.nlr.gov/buildings/beopt
- BOPTEST framework: https://ibpsa.github.io/project1-boptest/
- IFC 4.3.2.0 official specification: https://technical.buildingsmart.org/standards/ifc/ifc-schema-specifications/
- IDS 1.0 standard scope and geometry limitation: https://www.buildingsmart.org/standards/bsi-standards/information-delivery-specification-ids/
- Climate.OneBuilding South Africa candidate catalogue: https://climate.onebuilding.org/WMO_Region_1_Africa/ZAF_South_Africa/index.html
- SAIAT SANS 10400-XA Edition 3 specialist update: https://cpd.saiat.org.za/sans-10400-xa-3rd-edition-cc
