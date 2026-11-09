# CityMaker construction-method comparison matrix v0.1

**Status:** benchmark protocol, not an engineering design, approved system, measured result or preferred construction method.  
**Scope:** above-ground buildings, facilities, public realm and construction/assembly.  
**Primary consumers:** Sol-Atlas CityMaker #549; Sol-Atlas construction comparison #146 and repair-first assemblies #149; Symtropy panel-cell #3773 and circular-build #1856; Symthaea BUILT-ENV #6032 / HABITAT-VERTICAL #8051.

## 1. Paradigm: choose a production system, not a material or robot

The city should not standardize on one construction method. It should standardize **interfaces, evidence, information and acceptance** where this creates value, while preserving method diversity for site, climate, geometry, use, workforce, supply and lifecycle context.

A useful default hypothesis is a layered and repairable built environment:

- A long-lived support/infrastructure layer where durability and service life justify it.
- Replaceable, inspectable building systems and service zones.
- Productized/panelized components where repetition supports repeatability.
- Volumetric or precast modules only where transport, lifting, interface and site conditions support them.
- Local site-built work where geometry, access, repair context, labour or supply makes it appropriate.
- Retrofit/adaptive reuse before replacement where the existing asset can safely and lawfully meet the required service.
- Robotic production/assembly only for tasks with a qualified cell, tolerances, safety case, metrology and total-workflow advantage.

This is a set of hypotheses to test, not an assumed optimum.

## 2. Functional equivalence before comparison

Two construction alternatives are comparable only when they provide the same declared service basis over a shared evaluation scope.

Every comparison freezes:
- building/facility use and occupancy/service definition;
- required capacity and service hours;
- floor area or other functional output, with the exact denominator;
- performance and durability requirements;
- geography, climate, site conditions and boundary;
- study period and lifecycle cut-off rules;
- design revision, product/BOM and material performance class;
- quality, inspection and handover predicates;
- required maintenance/repair and replacement assumptions;
- accessibility, affordability, displacement and public-interest constraints;
- uncertainty and evidence rules.

“Same square metres” is not sufficient when occupancy, service, durability, thermal performance, accessibility, structural requirements or operating hours differ.

No method is feasible if a mandatory structural, fire, moisture, durability, health, accessibility, workplace-safety, legal or site constraint is failed or unresolved. Such requirements are hard gates, not weighted objectives.

## 3. Construction-method family matrix

| Method family | Where it may fit | Key dependencies / failure modes to test | Robotization opportunities | Evidence required before promotion |
|---|---|---|---|---|
| Retain / repair / adaptive reuse | Existing buildings and infrastructure with viable remaining service potential | Survey uncertainty, hidden defects, hazardous materials, legacy interfaces, utility capacity, disruption to occupants, statutory restrictions | Survey/metrology, inspection, non-destructive testing, selective removal, surface preparation, service replacement, component handling | Exact existing/as-built identity, condition survey, relevant material/structural/fire/moisture evidence, change-impact analysis, independent inspection |
| Conventional site-built masonry / concrete / steel / timber | Low repetition, constrained access, local skill/material availability, bespoke geometry, incremental construction | Weather exposure, site congestion, variable workmanship, temporary works, rework, material handling, quality dispersion, worker exposure | Layout, drilling/cutting assistance, material delivery, rebar processing, masonry placement assistance, inspection, lifting/handling | Material/lot acceptance, method statements, qualified engineering, workmanship tests, dimensional/quality inspection, site-specific safety controls |
| Panelized / platform construction | Repeated walls, floors, roofs, façades and service modules; distributed or staged projects | Panel tolerances, connection interfaces, transport dimensions, lifting points, weatherproofing at joints, site receiving/staging, design freeze and revision control | CNC cutting, kitting, fastening, adhesive application where qualified, panel inspection, lift preparation, placement/alignment assistance | Frozen BOM/design revision, material lot evidence, calibrated metrology, joint/system qualification, factory and site acceptance, logistics envelope |
| Volumetric modular | Repeated room/plant modules where transport and lifting can be solved | Oversize route constraints, crane capacity, module weight, transport vibration, stacking/interface tolerance, fire/acoustic compartmentation, utility connections, local approvals | Automated subassembly, module handling, fixture-controlled assembly, inspection and logistics | Complete route/vehicle/lifting/site envelope, module and connection qualification, tolerance stack, utility integration, inspection and commissioning evidence |
| Precast concrete | Repeated structural or enclosure components with controlled production and viable transport/lifting | Formwork/rebar variability, lifting inserts, curing, dimensional tolerances, transport damage, connection/site tolerance, embodied impacts and local plant capacity | Rebar/cage production, formwork assistance, concrete placement/finishing, curing/inspection, lifting and placement support | Mix/design/plant/lot evidence, reinforcement/connection checks, curing records, lifting and handling assessment, independent dimensional and performance tests |
| Timber / mass timber / hybrid | Suitable design/supply chains, low mass and prefabrication benefits where moisture/fire/acoustic/structural design is qualified | Moisture ingress, biological durability, fire performance, connection design, supply/grade variability, transport/storage, protection during construction | Grading/scanning, CNC cutting, drilling, fastening, sanding, kitting, panel handling, inspection and repair | Species/grade and source/lot evidence, moisture state, connection/system qualification, fire/acoustic/moisture/durability evidence, as-built inspection |
| Additive construction / 3D printing | Bounded geometries and components where process/material qualification and inspection are established | Printable material window, reinforcement, interlayer bond, anisotropy, curing, environmental sensitivity, dimensional verification, durability and standardization | Deposition path, toolpath inspection, material feed monitoring, robotic deposition, scan-to-model comparison | Qualified process/material profile, batch and environmental records, calibrated machine, coupon and full-scale tests, independent inspection, context-of-use qualification |
| Hybrid / mixed method | Most realistic building/facility portfolios combining methods by component and phase | Interface ownership, incompatible tolerances, scope gaps, duplicated or missing evidence, logistics and sequencing | Integrate task-specific cells and human trades; robot does not need to complete the whole building | Exact interface/control documents, cross-system commissioning, independent verification and whole-life comparison |

No row is inherently better. The method portfolio should be selected by context and evidence, not ideology.

## 4. Industrialization maturity crosswalk

Report the five dimensions identified by the 2026 Industrialized Construction Index independently:

1. **Standardization:** product variants, interface reuse, tolerance compatibility, approved component families, exception rate.
2. **Prefabrication:** share of work completed off-site, factory/site split, handling/transport burden, site assembly duration and quality at handover.
3. **Modularization:** module/interface count, interchangeability, replacement/repair time, mismatch and rework.
4. **Automation:** task coverage by production route, cycle-time distributions, human intervention, exception recovery and operating envelope.
5. **Digitalization:** exact design/BOM/process/inspection lineage, as-built discrepancy, evidence receipt completeness and change/recommissioning effort.

Reference: Sanchez & Attouri (2026), *Industrialized Construction Index: a project indicator for industrialization assessment*, IGLC 34. https://doi.org/10.24928/2026/0184

Maturity is not a performance outcome. Keep the ICI pillars separate from accepted-output, cost, schedule, quality, safety, lifecycle and workforce measures.

## 5. Robotics should follow task suitability

The 2026 review of 375 construction-robotics studies found research concentrated in structural tasks while planning, maintenance and demolition were underexplored; operator-led workflows still predominate. This argues for a lifecycle-wide, task-specific portfolio rather than a universal humanoid premise.

Reference: Xu et al. (2026), *Construction robotics: a systematic review of robot types, applications and human robot collaborations*, Results in Engineering 30, 110406. https://doi.org/10.1016/j.rineng.2026.110406

Priority task families for CityMaker:
- survey, layout, scan and metrology;
- timber/sheet/board cutting, machining, joinery and kitting;
- steel cutting, drilling and connection preparation;
- panel, façade, door/window and MEP rack assembly;
- material movement, loading, storage and staging;
- placement, alignment, fastening and inspection assistance;
- façade/building inspection and maintenance;
- repair, selective replacement and planned disassembly.

A robot capability must be bound to exact as-built platform/tool/configuration, payload, work envelope, coordinate frames, calibration, material/process compatibility, safety functions, operating domain, maintenance and evidence. A task success does not create structural or statutory acceptance.

NIOSH's 7 October 2026 bulletin stresses that robotics can reduce hazardous exposure but also introduce new hazards involving human–robot interaction, unauthorized entry, sensing, communications and software. The panel-cell and physical qualification owners must treat both sides explicitly: https://www.cdc.gov/niosh/bulletin/2026/safety-in-construction-robotics.html

## 6. Required outcome measures

Do not collapse the following into one “construction productivity” or “sustainability” score.

### Delivered service and quality
- Accepted functional units per project/period.
- First-pass yield and final independent acceptance.
- Defect classes, rework hours, scrap mass and nonconformance closure.
- Dimensional, interface and performance conformance.
- Service interruption during construction and handover/commissioning completeness.

### Whole workflow productivity
- Accepted output per total workflow labour-hour.
- Setup, engineering, kitting, material replenishment, supervision, monitoring, inspection, rework, cleaning and recovery time.
- p50/p90 cycle and end-to-end lead time.
- Site duration, temporary works and sequencing delay.
- Resource contention and missed delivery windows.

### Cost and lifecycle
- Installed cost on a common scope.
- Lifecycle cost under a declared study period and replacement schedule.
- Maintenance and repair burden, downtime and spare/tooling dependency.
- Design/change/recommissioning cost.
- Cost variance and cost uncertainty.

### Environmental and material
- A1–A5, B, C and D lifecycle stages separately where applicable.
- Virgin material, qualified component reuse, grade-retained recovery, downcycling and residuals.
- Energy, water, transport and operational inputs.
- Embodied and operational GHG with explicit data sources and boundary.
- Module D benefits kept separate from A–C primary results; no speculative recycling credit subtracted from primary impact.

### Human and safety
- Hazardous exposure removed and new exposure introduced.
- Manual handling, awkward posture, dust, noise, heat and repetitive work.
- Guarded-zone incursions, unsafe restart, safe-stop response, false stops and operator intervention.
- Training, task usability, workload and skill burden.
- Injury and near-miss reports under a declared method and exposure denominator.

### Adaptability and recovery
- Diagnosis, access and repair time.
- Component replacement without destructive demolition.
- Disassembly time, damage and recoverable mass by grade/function.
- Requalification after change.
- Availability of spare parts, tooling, skills, service operators and reverse logistics.

## 7. Matched experimental protocol

For a non-load-bearing panel demonstrator:

1. Freeze one panel family, released design/BOM revision, function, tolerances, materials and acceptance predicates.
2. Compare a site-built baseline, manual off-site assembly, assisted mechanization and guarded robotic cell where the physical safety/commissioning prerequisites exist.
3. Use the same accepted functional output, material/lot criteria, inspection, work scope, delivery window and functional unit.
4. Include all workflow labour/time; do not count robot cycle time alone.
5. Block or randomize trials by material lot, operator/team, shift and environmental conditions; preregister how allocation is chosen.
6. Use calibrated, independent measurement and a ground-truth system outside the robot's relevant fault domain.
7. Preserve all rejected/indeterminate trials and exclusions in the denominator.
8. Report p50/p90, uncertainty, sample counts, critical failures and repeatability across shifts/lots/configurations.
9. Reassess when design, tool, material lot, calibration, firmware or workcell configuration changes.
10. Add disassembly and repair tests only after the relevant connections/components have declared performance and qualification criteria.

No absolute percentage-improvement target is set before a measured baseline and relevant stakeholder/engineering margin are declared. A meaningful early result is non-inferior accepted quality and safety under matched requirements, plus a measured improvement in a predeclared operational outcome.

## 8. Claim and authority boundaries

- Synthetic case pass is software evidence only.
- Simulation is evidence within a declared solver context of use.
- Physical trial evidence requires calibrated instruments and independent acceptance.
- Building/factory profile conformance is not structural qualification or code compliance.
- Material passports do not prove that material is reusable or fit for a new use.
- A robot completing a task does not prove the resulting assembly is qualified.
- Cost/carbon models do not establish procurement or statutory approval.
- Community consent, worker representation, statutory authority and professional responsibility remain distinct.

## 9. Owner composition

- Sol-Atlas #549 owns urban alternatives and service/method comparison.
- Sol-Atlas #146 owns method-screening and robust comparison.
- Sol-Atlas #149 owns repair-first/reversible assemblies.
- Sol-Atlas #569 and #573 own spatial material-loop and delivery-feasibility projections.
- Symtropy #3773 owns the bounded panel-cell benchmark; #3505 owns physical ground-truth / ODD qualification.
- Symtropy #1856 owns repair/reuse/recycle/replacement intervention comparison and mass/grade semantics.
- Symtropy #3768 owns synthetic CityKit service/productive-capability feasibility.
- Symthaea #6032 owns built-environment realization profiles; #8051 owns factory/product/supply-chain co-design.
- Mycelix #4814 owns production/dependency networks; #5149 owns supplier/batch provenance, custody and accountable approvals; Living-City PR #5035 owns physical urban metabolism.

Adapters should reference owner-native receipts and preserve their authority. Do not fork their solvers or ledgers.
