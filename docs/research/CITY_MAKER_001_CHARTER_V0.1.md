# CITY-MAKER-001 — CityKit Charter v0.1

**Status:** research and benchmark contract; not an approved city plan, engineering design, construction specification, or operating authority.  
**Parent:** [CITY-MAKER-001](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/549)  
**Scope:** above-ground city systems, buildings, factories, public realm, logistics and construction.

## 1. Thesis

A city must be designed together with the productive system that makes, installs, operates, repairs and adapts it.

The design object is therefore not only a land-use plan or a collection of BIM objects. It is a versioned dependency graph connecting:

`human needs -> service capacity -> spatial form -> assets -> assemblies -> materials/processes -> productive equipment -> work packages -> accepted articles -> operating services`

The graph must remain inspectable across design, procurement, fabrication, assembly, commissioning, operation, repair, replacement and end of life.

This charter defines the minimum semantics and benchmark boundaries for CityKit v0.1. It does not create a new city simulator, GIS, construction ontology, structural solver, manufacturing engine, work dispatcher, robot authority or provenance authority.

## 2. Scope boundary

### Included

- Housing and daily-life facilities.
- Health, education, culture, recreation and civic facilities.
- Food markets, storage, cold-chain and bounded urban/peri-urban production.
- Water, sanitation, energy, thermal services and digital infrastructure as city service systems.
- Streets, walking/cycling/transit interfaces, freight, loading and logistics.
- Factories, workshops, warehouses, component suppliers, repair and recycling facilities.
- Parks, green/blue infrastructure, shade, public space and ecological interfaces.
- Above-ground construction, prefabrication, workcells, assembly, inspection, maintenance and deconstruction.

Buried networks may be represented as interfaces, capacities, risk zones and external dependencies. Underground robot fleets, tunnelling, mining and subterranean construction are not part of this phase.

### Excluded authority

A CityKit pass cannot establish planning permission, land rights, community consent, structural adequacy, fire/life safety, utility approval, environmental compliance, affordability, economic viability, commissioning, or authority to dispatch or actuate a robot. These remain with responsible participants, professional engineers, qualified operators and applicable authorities.

## 3. Hierarchical model

Represent the city at linked scales:

1. **Region:** supply sheds, watersheds, energy and freight connections, regional ecosystems and labour/skills availability.
2. **City:** population and demand scenarios, urban form, major service capacity, mobility and industrial geography.
3. **District / neighborhood:** service reach, local production and repair, public realm, utilities and everyday mobility.
4. **Block / site:** access, staging, geometry, parcels, buildings, public space and construction interfaces.
5. **Building / facility:** function, occupancy/use, structure, envelope, services, equipment and operations.
6. **Assembly / component:** parts, materials, interfaces, tolerances, connections, inspection and replacement.
7. **Process / capability:** people, machines, tools, energy, inputs, skills, calibration, throughput, quality and dependencies.

A coarse representation may answer only questions for which its declared information and validation envelope are sufficient. Refinement is required when a query crosses that envelope; display LOD never supplies engineering authority.

## 4. Core graph semantics

The first schema should support these typed records. Exact JSON/Rust field names are to be frozen in the implementation issue after the owner audit.

### Service requirement

- Stable ID and revision.
- Service family and description.
- Population / user / demand basis; normal, peak and disruption scenarios.
- Capacity, quality, continuity and operating-window requirements with units.
- Geographic reach and accessibility requirements.
- Provider/operator role and dependencies.
- Required evidence, hard constraints, and unresolved questions.

### Asset and assembly

- Stable ID, revision, parent/host and spatial reference.
- Functional basis and intended service contribution.
- Component tree, quantities, materials and interfaces.
- Performance requirements and separately owned evaluation receipts.
- Design, fabrication, as-built, inspection, commissioning and maintenance states.
- Access, repair, replacement, adaptation and end-of-life constraints.

### Productive capability

- Output or service produced.
- Alternative production routes (OR) and required prerequisites within a selected route (AND).
- Inputs, material grades, energy, water, process equipment, tooling and skills.
- Rate/capacity with units and evidence class.
- Setup, changeover, uptime, maintenance, waste and quality.
- External supplier/import dependencies and source freshness.
- Distinct states: available, operational, maintainable, repairable, replacement-part capable, productive-equipment reproducible, and expansion-capable.

An imported capability can satisfy a city's present need but must not be reported as locally reproduced. Capability cycles cannot bootstrap themselves without grounded prerequisites.

### Work package and robot capability

A work package declares exact site/geometry/material revisions, task, tolerances, sequence, access, environment, safety prerequisites, inspection hold points and acceptance evidence.

A robot profile binds platform/configuration, tool, firmware/controller, coordinate frames, calibration, workspace, payload, material compatibility, tolerances, speed assumptions, energy, setup, maintenance, safety functions, operator role and evidence scope. Unknown values remain unknown. Simulation, vendor statements and synthetic tests do not qualify physical capability.

### Evidence and decision

Every material claim carries source identity, revision, digest where available, units, method, tool/version, assumptions, uncertainty, date/currentness, scope and claim ceiling. Assertions, measurements, independent evaluations, approvals and legal permissions remain different types. Corrections, revocations, disputes and supersession must be representable.

## 5. City form and construction alternatives

CityKit must compare alternatives against the same declared service basket, demand profile, site constraints and study period.

Initial urban archetypes:

- compact mixed-use / transit-oriented;
- polycentric neighborhood network with distributed services and production;
- lower-rise incremental / courtyard / block fabric with local prefab and repair capacity.

These are hypotheses, not an exhaustive typology or a universal ranking. Include maintain-existing / retrofit and no-build baselines.

Initial construction families:

- site-built masonry, concrete, steel and timber;
- panelized / platform and open-building systems;
- volumetric modular where transport and lifting limits permit;
- precast and prefabricated structural, façade and utility assemblies;
- mass timber and hybrid systems where supply, fire, moisture, durability and lifecycle evidence support them;
- robotic machining, handling, fastening, welding, inspection and assembly;
- bounded additive construction applications with material, reinforcement, interlayer bond, durability and acceptance evidence;
- repair, adaptive reuse, component harvesting and selective replacement.

No material or construction technique is presumed to dominate. Evaluate the complete delivered service, including foundations/site interface, transport, lifting, installation, commissioning, maintenance and end of life.

## 6. Benchmark structure

### B0 — Schema and evidence integrity

- Reject duplicate/dangling IDs, incompatible revisions, missing units/coordinate frames and inconsistent references.
- Preserve unknown, unsupported, stale, disputed and rejected states.
- Detect digest mismatch, stale approval, revoked certification and superseded revision.
- Produce deterministic normalized outputs bound to exact inputs and code revision.

### B1 — Service and resource feasibility

- Independently recompute declared service capacity and resource balances.
- Detect capacity shortfalls, unserved demand, impossible operating windows and shared-resource overcommitment.
- Test water, energy, food, waste, logistics and maintenance disruptions.
- Report geographic reach separately from facility counts.

### B2 — Buildability and manufacturing

- Check declared access, workspace, reach, lifting, handling, staging and installation sequence.
- Check process precedence, workcell/resource conflicts and supplier outages.
- Compare accepted output, not nominal output: first-pass yield, rework, scrap, defects and inspection coverage.
- Keep cost, time, carbon, energy, water and worker exposure separate, with explicit denominators and evidence.

### B3 — Robotics and assembly

Use task-level boards inspired by NIST assembly tests and contemporary assembly benchmarks, then add construction-relevant operations: beam/panel presentation, alignment, fastening, connection insertion, flexible service routing, façade/door/window installation, inspection and replacement.

Measure success by task and held-out condition, pose/placement error, force/torque envelope, cycle-time distribution, intervention burden, damage, energy, tool wear, uptime and safe recovery. Report confidence intervals and failures; do not compare unmatched tasks or output quality.

### B4 — Urban outcomes

Report housing and service adequacy, access to jobs/healthcare/education/food/transit/public space, network connectivity, public-realm distribution, heat/shade, water/energy reliability, ecological impacts, affordability and distribution of burdens. Do not use a hidden composite city-quality score.

### B5 — Lifecycle and adaptation

Measure maintenance effort, service interruption, fault diagnosis, component replacement access, repeat assembly/disassembly, recoverable mass and quality, adaptability and actual end-of-life outcomes. Keep prospective reuse benefits separate from measured current-life impacts.

## 7. P0 acceptance invariants

- 100% agreement with an independently written oracle on a frozen finite synthetic corpus.
- Zero false acceptance of seeded hard-constraint, identity, evidence, capability, material-accounting or resource violations.
- Zero silent promotion of unknown/stale/unsupported/simulation-only/uncommissioned claims.
- Zero unauthorized dispatch/actuation transitions.
- Deterministic replay of equivalent inputs and explicit divergence for changed inputs.
- Every result reports denominator, exclusions, unknowns, failure reasons, input digests and exact code/configuration identity.

These qualify the contract and verifier only. They do not qualify a real building or city.

## 8. Comparative protocol

Compare against:

1. Conventional construction and human-skilled baseline on matched work and acceptance criteria.
2. Current industrial automation / prefabrication practice, with named equipment and configuration.
3. Relevant construction-robotics demonstrations or products, only where public evidence supports a matched task.
4. Existing district/city service delivery and maintain-existing / retrofit alternative.
5. Independent solver or professional reference for each engineering domain actually evaluated.

Freeze requirements, output quality, study period, site/environment, source data, software and hardware versions, operator skill, tuning and compute/time budgets. Report total system cost, not just robot purchase price or cycle time. Unavailable or unmatched competitor evidence is `unassessed` / `not_comparable`, not a loss or a win.

No numeric productivity superiority target is set before a matched baseline and trial protocol exist. The initial hard targets are correctness, auditability and safe non-promotion. Comparative improvement targets are then preregistered per task.

## 9. Synthetic first case

Create a parameterized mixed-use neighborhood service bundle with explicit:

- population and demand profiles;
- climate, topography, water and hazard assumptions;
- housing, health, education, food, civic, public-space and mobility services;
- energy, water, digital and waste interfaces;
- factory/repair/warehouse and material-supply routes;
- building/component alternatives and work packages;
- candidate robot/tool profiles;
- maintenance and disruption scenarios.

Compare the three urban archetypes and maintain-existing baseline using common requirements. All synthetic quantities must be visibly marked synthetic. Do not insert guessed real-world cost, vendor throughput, material strength, carbon factors, supplier capacity or local conditions.

## 10. First implementation sequence

1. Audit current owner contracts and implementation status before defining duplicate fields.
2. Freeze CityKit v0.1 service, asset, capability, work-package and evidence identifiers.
3. Create a tiny synthetic fixture set and an independent reference oracle.
4. Add negative controls for missing evidence, capability mismatch, resource overcommitment, stale approvals, unit/frame mismatch and authority promotion.
5. Connect the city-level contract to existing spatial projections and owner-native Symthaea/Symtropy/Mycelix evidence; preserve each owner's authority.
6. Expand from service/resource feasibility to buildability, workcell comparison and one instrumented laboratory assembly.
7. Only then run a larger neighborhood scenario and prospective field pilot.

## 11. Research anchors

- NIST, [Assembly Performance Metrics and Test Methods](https://www.nist.gov/el/intelligent-systems-division-73500/robotic-grasping-and-manipulation-assembly).
- Kimble et al., [Performance Measures to Benchmark the Grasping, Manipulation and Assembly of Deformable Objects](https://doi.org/10.3389/frobt.2022.999348).
- [WorkBenchMark](https://workbenchmark.github.io/), LEGO Duplo robotic assembly benchmark (RoboCup Symposium 2026).
- Chen & Adel, [Advancing robotic assembly in construction](https://doi.org/10.1016/j.autcon.2025.106370), *Automation in Construction* (October 2025).
- NIST, [Additive Construction in Practice — Realities of Acceptance Criteria](https://www.nist.gov/publications/additive-construction-practice-realities-acceptance-criteria).
- ISO, [10218-2:2025](https://www.iso.org/standard/73934.html), industrial robot applications and robot cells.
- ISO, [20887:2020](https://www.iso.org/standard/69370.html), design for disassembly and adaptability.
- UN Statistics Division, [SDG 11: Sustainable Cities and Communities (2026)](https://unstats.un.org/sdgs/report/2026/Goal-11).

3D concrete printing is included as a bounded process alternative, not a general city-building solution. Current reviews describe continuing challenges around printable material composition, reinforcement, interlayer bonding, durability, standardization and lifecycle impact. Industrial robot safety standards apply to their declared workplace/application scope and do not authorize unrestricted robots in occupied public space.

## 12. Claim ceiling

CityKit v0.1 may establish schema integrity, known-answer synthetic feasibility, deterministic resource accounting and explicit evidence gaps. It does not establish that a proposed city is safe, legal, affordable, economically viable, sustainable, structurally adequate, socially accepted, physically buildable or operationally resilient.

A city becomes a demonstrated result only through qualified domain evaluations, as-built inspection, commissioned services, measured operations and sustained independent evidence.
