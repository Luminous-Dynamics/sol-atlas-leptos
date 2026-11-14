# Multi-family Shared-Service Benchmark v0.1

**Status:** preregistered synthetic topology / service-evidence benchmark. No hydraulic or operational result has been executed.
**Parent:** [MULTIFAMILY-SHARED-SERVICE-001 #844](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/844).
**Building:** [SYN-MF-COURTYARD-004](multifamily-building-synthetic-v0.1.json).
**Service topology:** [SYN-MF-SERVICE-DISTRIBUTION-001](multifamily-service-distribution-synthetic-v0.1.json).
**Stress corpus:** [16-case / 20-event authored manifest](multifamily-service-stress-corpus-v0.1.json).

## 1. Research question

Can the design/delivery workflow prove that resident-defined household services are delivered at the correct apartment boundary, at the correct time, with the correct resource class and quality evidence, through normal operation, maintenance, scarcity, common-cause failure and recovery?

The measured outcome is not the building's nominal tank volume or pump rating. It is the service actually delivered at each household boundary, with uncertainty and quality qualification retained.

`source -> storage / plant -> pressure zone / riser -> unit boundary -> resident service -> qualified delivery -> deficit / recovery trajectory`

The system must represent who owns each shared asset, who maintains it, which units it serves, what evidence supports each claim, who controls scarcity allocation, and how residents can understand and challenge those decisions.

## 2. Why this matters in Johannesburg

Johannesburg Water's 29 October 2026 statement reported severely constrained supply systems: high demand and excessive draw-off were limiting reservoir and tower recovery, resulting in low pressure, intermittent supply and prolonged outages in several areas. It noted that higher-lying areas were particularly affected as reservoir and tower levels declined:
- https://www.johannesburgwater.co.za/blog/2026/10/29/johannesburg-water-implements-urgent-interventions-to-stabilise-severely-constrained-systems/
- https://www.johannesburgwater.co.za/blog/2026/10/21/johannesburg-water-urges-residents-to-reduce-consumption-as-high-demand-strains-supply-systems/

Gauteng Provincial Government's 13 February 2026 update attributed disruptions across Gauteng metros to interacting factors including ageing infrastructure, demand, leaks, power interruptions affecting pumping and maintenance backlogs:
- https://www.gov.za/news/media-statements/gauteng-provincial-government-addresses-water-challenges-13-feb-2026

These statements are dated, system-level observations, not a prediction of any specific apartment building's service. They establish why a multi-family building must model source pressure, floor/pressure zones, reserve, pump power, unit-boundary delivery and restoration as separate evidence domains.

South Africa's revised Compulsory National Water and Sanitation Services Standards (Government Gazette 52814, 6 June 2025) defines SANS 241 and SANS 10252 and places drinking-water fitness and monitoring with the responsible water services institution. It also defines a supply zone by its pressure-zone/reservoir or bulk-supply context:
- https://www.gov.za/sites/default/files/gcis_document/202506/52814gon6292.pdf

SANS 10252-1:2018 (Edition 3.2) describes principles for design, installation and testing of building water installations; current project use must be confirmed against the controlled SABS edition, amendments, local rules and applicability:
- https://www.mosselbay.gov.za/storage/documents/cb942e034d7e086691fa18d41a15b553.pdf

A software fixture must never hard-code a universal minimum pressure, litres/person/day, outage allowance, water-quality limit or storage duration. Bind exact project rules, edition, effective date, authority and professional applicability instead.

## 3. Current industry and solver baseline

### Hydraulic solvers: compose, do not recreate

**US EPA EPANET 2.2** is an established open, public-domain application for extended-period hydraulic and water-quality simulation of pressurised pipe networks containing junctions, pipes, pumps, valves, tanks and reservoirs. Its documented outputs include flow, pressure, tank levels, constituent concentration, water age and source tracing; it supports pressure-dependent demand and pump energy analysis:
- https://www.epa.gov/water-research/epanet
- https://github.com/USEPA/EPANET2.2
- https://usepa.github.io/EPANET2.2/

**Bentley OpenFlows Water** markets WaterCAD, WaterGEMS and HAMMER workflows for hydraulic, water-quality, scenario, transient and pump/valve analysis, with CAD/GIS/SCADA integration and optimization features:
- https://www.bentley.com/products/openflows-water/
- https://www.bentley.com/globalassets/content/pds-openflows-water-ltr-en-lr.pdf

These tools are the hydraulic-analysis comparators. Their model outputs still depend on correct boundary conditions, calibration, topology, demand profiles, product versions and professional interpretation. Municipal distribution-system capabilities must not be assumed to be a complete internal building-plumbing design or approval workflow.

### Pressure-zone and booster-system baselines

Grundfos describes zoned high-rise water supply as a way to constrain pressure variation between lower and upper floors, with basement booster, cascade booster and roof-tank hybrid configurations:
- https://www.grundfos.com/za/learn/research-and-insights/water-supply-systems-in-zones

These are real architecture choices to compare, not a universal recommended pressure or a project-specific design. Pump selection, pressure zoning, storage, redundancy, water age, isolation, backflow protection, commissioning and fire-service separation remain under the relevant engineering/authority owners.

### Research evidence and gap

A 27 October 2026 *Journal of Building Engineering* review synthesized 53 high-rise residential water-distribution studies. It reports that much of the literature focuses on isolated components such as pump scheduling or tank sizing, often uses simplified hydraulic models and limited case studies, and has comparatively limited real-building validation, long-term maintenance/resilience and lifecycle treatment:
- Zhang et al., DOI 10.1016/j.jobe.2026.117679: https://www.sciencedirect.com/science/article/pii/S2352710226025015

This is the differentiating opportunity: **do not attempt to beat EPANET at hydraulics or Grundfos at booster equipment.** Connect specialist hydraulic results, utility-quality evidence, per-unit service boundaries, common-cause dependencies, service-deficit intervals, resident governance, lifecycle cost and revision invalidation into one reviewable evidence chain.

## 4. Four architecture configurations to compare

All configurations receive the exact same building, unit programmes, source boundary, demand profiles, weather/outage traces, pipe/material options, local rule profile and measurement uncertainty. The current synthetic fixture contains none of the physical quantities needed to calculate these designs; these configurations are a benchmark plan only.

1. **Central shared plant and storage:** one central source/storage/booster arrangement with shared risers.
2. **Cascade pressure zones:** staged booster groups and independently identified floor/pressure zones.
3. **Distributed zone plant/storage:** localised zone reserves and boosters with independently isolatable branches.
4. **Hybrid with selective local reserves:** central plant and reserve combined with floor-zone isolation and limited local reserve.

Add an independent household-level reference where technically, legally and economically appropriate. It is not assumed to be more resilient or cheaper.

Compare the complete cost and risk envelope: plant/room footprint, capital, pumping energy, water age, pressure/flow, leakage, storage hygiene, maintenance staff, spares, redundancy, isolation time, common-mode failure, service charges, resident disruption, replacement and embodied materials.

## 5. P0 machine-readable artifacts

- `multifamily-service-distribution.schema.json`
- `multifamily-service-distribution-synthetic-v0.1.json`
- `multifamily-service-stress-corpus.schema.json`
- `multifamily-service-stress-corpus-v0.1.json`
- `src/bin/home-multifamily-service-qualifier.rs`
- `.github/workflows/home-multifamily-qualifier.yml`

The current synthetic distribution topology contains:
- 6 resource classes: potable water, non-potable water, electricity, useful thermal, wastewater and communications;
- 34 nodes, including source, storage, plant, zone, riser, meter, equipment and apartment boundaries;
- 30 typed edges, including resource flows and separate power/control dependencies;
- 16 unit-service boundary records (four services for each of four synthetic apartments);
- 16 synthetic event records;
- a separate 16-case / 20-event stress corpus with one positive control and 15 adversarial stress controls.

The fixture explicitly binds to the exact synthetic building identity and unit/floor mapping. All pressure, flow, water quality, capacity, demand, delivered quantity, storage change, losses, tariff, service floor, allocation policy and conservation status remain UNKNOWN. The non-potable path is separate and receives no potable-use credit. The control dependency does not assert an actual cloud or network requirement.

The Rust qualifier checks schema/maturity identity, unique resources/nodes/edges/events, unit and floor identity, resource-class consistency, edge endpoints and flow types, service-to-resource semantics, exact unit-boundary mapping, event-to-node references, unknown quality/capacity/quantity preservation, authorised-allocation evidence, accounting-state promotion and directed path reachability. It does not calculate hydraulic performance or execute the stress simulations.

## 6. Synthetic adversarial corpus and intended dispositions

The separate corpus covers:
- source interruption;
- pump / common-power failure;
- one-floor riser isolation;
- non-potable treatment unavailable;
- measurement / sensor observability loss;
- floor-zone outage;
- allocation-policy change;
- common communications failure;
- maintenance isolation;
- spare-part delay;
- restoration followed by recurrent outage;
- unknown or unqualified capacity/quality and boundary substitution.

A valid synthetic positive control proves only that the authored topology is internally connected and typed. It does not show that a litre of water, watt-hour or joule of heat has ever been delivered.

The Rust qualifier currently checks the authored stress-corpus identity, case split, event identity/order, event-to-node references and claim ceiling. It does not yet mutate and execute all 15 adversarial stress cases. That is a distinct next test-harness increment; do not label the manifest executed until the runner actually replays each case against the independent oracle.

## 7. Hard qualification gates

- 100% of household services map to the exact unit, resource class, boundary and design revision.
- Zero resource-flow edges cross resource or quality classes.
- Zero potable-use credit from unknown or unqualified treatment/source evidence.
- Zero unit-service PASS from aggregate tank, pump or building capacity alone.
- Zero unit-level service-floor breach hidden by building averages.
- Zero duplicate reserve allocation or resource creation; a physical conservation PASS requires measured input, measured storage change, unit delivery, declared losses, an exact time window and uncertainty.
- Zero common-cause dependency hidden by treating pump, storage, controls, communication and power as independent.
- 100% event-to-asset and event-to-affected-unit traceability.
- 100% exact revision binding for source, pump, valve, meter, pressure zone, unit served, control policy and measurement/calibration.
- 100% affected evidence invalidation after a relevant source/asset/topology/policy/product/rule change.
- Zero allocation-policy promotion without authority, rules, resident notice and dispute/appeal pathway.
- Zero probabilities inferred from this synthetic event manifest.

## 8. Metrics: report separate distributions, not one score

### Hydraulic / physical service (specialist solver and measured pilot)
- minimum/maximum and interval pressure at every unit boundary;
- flow and delivered volume by unit/time window/resource quality class;
- storage level and time-to-reserve-depletion;
- pump efficiency and electrical input (kWh-e) per delivered cubic metre;
- water age, source contribution, disinfection/quality measurements and backflow/cross-connection results;
- hydraulic convergence, calibration residuals, sensor uncertainty and out-of-domain states.

### Household service and resilience
- quality-qualified service delivered and deficit interval by unit;
- minutes below resident-defined/professional service floors;
- time to first breach, reserve depletion, restoration and stable recovery;
- worst-unit, worst-floor, median and lower-tail results;
- service affected by pump, power, communications, valve, treatment, spare and maintenance events;
- access to safe manual operation and resident-visible service state where applicable.

### Governance / equity
- allocation policy, authority, resident notification, appeal and dispute status;
- who bears scarcity, service interruption, utility cost and maintenance disruption;
- per-unit service distribution by floor/zone/tenure without exposing private usage histories;
- show alternatives and consequences rather than silently ranking household claims.

### Lifecycle / affordability
- installed capital, plant-room/floor area, design and professional cost;
- operating electricity, water charges, maintenance, calibration, filters/consumables, spare parts and replacements;
- service charge per unit/month and cost volatility;
- embodied materials and replacement/disposal route;
- tie whole-household results into [#816 Cost of Place](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/816).

Do not collapse life-safety, water quality, rights, service floors and affordability into a weighted scalar.

## 9. Matched comparison protocol

Compare:
1. Conventional professionally designed and commissioned building-services package.
2. EPANET 2.2 model and result package, where its network physics and assumptions fit the problem.
3. Bentley OpenFlows Water / equivalent commercial hydraulic workflow, where available and comparable.
4. A manufacturer-selected zoned booster/plant design (for example, a documented Grundfos/Wilo/Xylem package) with exact product/revision and professional sizing.
5. Luminous's evidence-carrying service-topology and unit-boundary qualification layer, consuming externally validated solver and measurement results.

Freeze the same network, source, unit mix, demand trace, pressure boundaries, storage, controls, pipe/material data, tariffs, event traces, engineering scope and cost date. Record product/solver version, model files, assumptions, calibration set, time-step, convergence, warnings, human effort, configuration, licenses and missing inputs.

Compare:
- solver-to-solver pressure/flow/energy agreement on known-answer reference networks;
- measured-to-model error on calibrated held-out pilot traces;
- unit-level breach detection recall and false-pass rate;
- common-cause and dependency coverage;
- time to identify which apartments are affected by a source/asset/policy change;
- invalidation precision/recall and stale-PASS count;
- whole-life cost completeness and unit service distribution;
- resident comprehension and dispute resolution, measured separately.

A synthetic topology check must never be scored against a hydraulic solver as if they solve the same task. A hydraulic engine's numerical accuracy must not be conflated with the end-to-end workflow's evidence traceability.

## 10. Proposed improvement targets

Hard gates in section 7 are non-negotiable. Quantitative performance hypotheses should be frozen only after baseline and variance studies:

- zero false unit-service PASS on known-answer and sealed adversarial controls;
- 100% agreement with independent resource-path and unit-boundary oracles;
- target at least 25% lower time to identify affected units and revalidation scope after a topology revision than a matched manual evidence review, without increasing missed dependencies;
- target at least 20% reduction in essential-service breach minutes for a matched hybrid configuration versus a matched central-only configuration, without transferring deficits to another unit or neighbour;
- target at least 10% lower lifecycle cost per unit for a matched design only where physical performance, service floors, quality and maintenance are equivalent; otherwise publish the Pareto frontier;
- target zero unexplained mass-balance residual beyond declared measurement uncertainty on a calibrated pilot.

These are preregistered research hypotheses, not current results, code thresholds or promises of savings.

## 11. Physical pilot gates

No pilot should be run on an occupied building without competent engineering design, owner/operator permission, safety review, resident notice, privacy protection and appropriate authority approvals.

A permissioned pilot must have:
- calibrated source and unit-boundary meters;
- pressure transducers at representative floors and unit endpoints;
- water-quality sampling/chain-of-custody by a competent laboratory where potable claims are assessed;
- pump electrical input and useful thermal output metered as separate carriers;
- exact asset, riser, valve, meter and unit mapping;
- common time base and uncertainty/clock synchronization;
- controlled outage/maintenance tests approved by the responsible operator;
- no intentional degradation of resident service to create a benchmark case;
- independent data adjudication, resident complaint/appeal and incident stop conditions.

## 12. Claim ceiling

The current schemas, fixture and authored stress corpus establish no capacity, pressure, flow, potability, quality, delivery, service continuity, outage probability, fair allocation, energy efficiency, cost savings, compliance or resident outcome. The Rust qualifier is a topology/evidence-integrity check only. No hydraulic or building-services calculation, live control, isolation, load shedding, installation or operational change is authorized.
