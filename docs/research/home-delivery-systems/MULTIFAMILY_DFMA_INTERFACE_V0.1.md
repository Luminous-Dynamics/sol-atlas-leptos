# Multi-family DfMA and lifecycle interface v0.1

**Status:** apartment-specific integration contract for existing assembly/manufacturing owners; not a product catalogue, fabrication release, structural design, fire/acoustic approval or cost estimate.
**Parent:** [HOME-MULTIFAMILY-COMPILER-001 #840](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/840).

## 1. Ownership

Compose:
- [#803 Home Assembly Demand](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/803): required assemblies and design-to-assembly demand.
- [#814 Home DfMA Delivery](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/814): delivery-mode comparison, fabrication, installation and accepted as-built.
- [#811 / PR #812 Material Intelligence](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/811): exact product/version/approval/quote evidence.
- [#622 Robotic Panel Fabrication](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/622): geometry-to-panel production integration.
- [#549 CityMaker](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/549): productive geography, local factories, logistics and capability.
- Symthaea [HABITAT-VERTICAL #8051](https://github.com/Luminous-Dynamics/symthaea/issues/8051): repairable household goods, product/process/factory co-design and supply chain lifecycle.
- [#816 Cost of Place](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/816): household/lifecycle cost and location effects.
- [#840 Multi-family Compiler](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/840): apartment/unit/floor/shared-service identity.

This document owns only the apartment-to-DfMA adapter contract. It does not create a second materials database, estimator, manufacturing execution system, construction dispatcher, robot controller, building ontology or certification register.

## 2. Paradigm: buildings as maintainable assemblies

Do not optimise only for the first installation. A multi-family building contains repeated assemblies and shared systems whose failure or replacement can affect many households.

The target chain is:

`resident/unit requirement -> building design revision -> assembly demand -> exact product/material evidence -> factory capability -> QA and batch record -> logistics/lift -> installation -> inspection -> as-built -> operation/repair/replacement`

Every interface should be designed for:
- exact identity and controlled revision;
- tolerance stack-up and adjustment envelope;
- safe transport, lifting, handling and temporary storage;
- connection and service isolation;
- inspection access and evidence;
- replacement path without disproportionate disturbance to occupied units;
- repair/disassembly and end-of-life;
- local sourcing and capability evidence, not assumed local availability.

Standardization should apply to interfaces where repeatability helps; resident-visible choices and site-specific engineering must not be erased to force a common module.

## 3. Candidate apartment assembly families

These are candidate classes for investigation, not prescribed products or an assertion that they are qualified.

| Assembly family | Why it matters in multi-family housing | Required interface evidence |
|---|---|---|
| Party walls and floor/ceiling assemblies | Fire compartmentation, airborne/impact acoustics, moisture and structural separation affect neighbours | Exact assembly/product revision, tested scope, junction details, penetrations, installation tolerances and inspection |
| Facade / window / shading cassettes | Repetition can support factory QA; orientation, solar gain, daylight, water ingress and maintenance vary by elevation | Panel geometry, fixing/anchor design owner, weather/water test scope, thermal/solar data, fire scope, access and replacement method |
| Wet-core / bathroom modules | Repeated plumbing/electrical/ventilation details can reduce interfaces but create transport and maintenance dependencies | Exact fixture/pipe/duct/electrical positions, isolation, leakage test, drainage, waterproofing evidence, access panels and replacement envelope |
| Service risers and distribution branches | Shared vertical infrastructure drives pressure zones, outages, metering, isolation and cross-floor maintenance | Exact served unit/floor IDs, carrier type, units, pressure/flow or electrical ratings, meter/valve identity, fire-stopping scope and access |
| Common plant skids | Pumps, hot water, ventilation, batteries and switchgear may be prefabricated as maintainable modules | Measured/declared loads, thermal rejection, vibration/noise, isolation, lifting/transport envelope, commissioning and spare/repair path |
| Stairs, lift interfaces and core components | Vertical circulation affects transport, installation sequence, outages and replacement access | Specialist design references, exact opening/shaft identity, installation tolerances, inspection and maintenance access; no accessibility or egress claim from geometry |
| Interior partitions and storage systems | Adaptable layouts can reduce future renovation waste and resident disruption | Non-structural scope confirmation, attachment method, service-zone conflicts, acoustic/fire limitations, disassembly and reconfiguration evidence |
| Balconies, guards and external access components | Repeated elements face corrosion, waterproofing, wind/load, facade and maintenance risks | Exact design/product scope, connection authority, corrosion/environment class, inspection/repair and replacement access |

## 4. Apartment-specific assembly interface record

Every assembly demand should bind:

1. **Identity:** assembly ID, family, instance IDs, building/floor/unit/space references, exact design revision and parent requirement.
2. **Geometry:** source geometry reference, units, coordinate frame, manufacturing envelope, installed envelope, lifting points and declared tolerances. If takeoff or geometry is unavailable, state `NOT_COMPUTED`.
3. **Interfaces:** adjacent assembly IDs, connection type, fasteners/adhesives, joint dimensions, penetrations, service routes, isolation and required installation order.
4. **Performance evidence:** exact tested/assessed claim scope for structural, fire, acoustic, water/moisture, thermal, durability or other properties, with method, lab/professional, date and report digest. An unrelated rating or product-family brochure cannot be transferred to the exact assembly.
5. **Manufacturing:** factory/site process, equipment, tooling, skill, material batch/lot, process window, QA plan, calibration and nonconformance disposition.
6. **Supply and logistics:** manufacturer, supplier, quote date, lead time, origin, stock/production capacity, packaging, storage, route, vehicle, crane/lift and site access. `LOCAL` is an evidence state, not a geographic guess.
7. **Installation:** lifting/handling plan owner, temporary works, sequence, weather limits, tolerances, site measurements, inspection hold points, commissioning and acceptance records.
8. **Occupied-building maintenance:** safe access, isolation, repair/replace module, household disruption, downtime, resident notice, spares, support and service-life assumptions.
9. **End-of-life:** disassembly, reuse/reconditioning, recycling route, hazardous components and material traceability.
10. **Revision invalidation:** changes to geometry, product, supplier, batch, process, connection, fire/acoustic/moisture scope, installation method or site logistics invalidate only dependent claims.

## 5. Transport and replacement are design variables

A module is not manufacturable merely because it fits its final BIM location. The delivery path must be checked:
- factory machine envelope and material handling;
- packaging and weather protection;
- road/vehicle dimensions, route limits and permits;
- site laydown/storage and construction staging;
- crane reach, lifting mass, centre of gravity, rigging and wind conditions;
- lift/shaft/door/corridor turning envelope for indoor movement;
- temporary support, installation sequence and worker access;
- replacement extraction route through an occupied building;
- temporary isolation, resident disruption and safe reinstatement.

Any missing lift study, route survey, structural lifting authority or temporary-works design remains a blocking unknown for the relevant release. This contract records evidence; it does not generate a lift plan.

## 6. Whole-workflow delivery comparison

Compare conventional site-built, panelized, volumetric modular and hybrid approaches using the same resident brief, site, unit mix, fire/acoustic/moisture/thermal requirements, finish scope, authority path, cost date and output quality.

Include:
- design engineering and digital coordination;
- factory mobilisation, overhead, utilisation and learning curve;
- procurement, minimum order, stock, working capital and supplier credit;
- factory labour, tooling, energy, yield, scrap and rework;
- packaging, storage, transport, permits, cranage and insurance;
- site preparation, foundations, temporary works and access;
- site labour, installation, weather downtime, tolerance correction and commissioning;
- inspections, certification, defects, warranty, repair and replacement;
- financing/carrying cost, occupation disruption and lifecycle maintenance.

Do not quote a universal prefabrication savings percentage. A 2026 systematic review of industrialised housing reports variable performance, with literature examples of schedule reduction and cost savings but also cases of substantial cost increases; factory utilisation, transport/cranage, supply-chain maturity, procurement and interface rework materially affect outcomes:
- https://www.mdpi.com/2075-5309/16/3/552

That variability is a reason to measure, not a reason to dismiss offsite production.

## 7. Benchmarks

Hard gates:
- 100% assembly-instance to exact design revision traceability;
- zero quantity `PASS` without geometry, takeoff method, units and source digest;
- zero material/product substitution based only on name or family resemblance;
- zero fabrication release without exact product, supplier, process, QA and revision scope;
- zero installation acceptance without required inspection/hold-point records;
- 100% nonconformance, repair and reinspection history retained;
- zero claim inheritance across unrelated fire/acoustic/structural/moisture/thermal product scope;
- zero as-built claim from design intent alone.

Outcome metrics (separate vectors):
- total installed cost and variance;
- design-to-factory-release and factory-to-occupancy duration;
- factory utilisation, throughput, yield, scrap and first-pass quality;
- transport/lift cost, damage and schedule variance;
- installation hours, fit-up tolerance, interface rework and defects;
- inspection completeness and commissioning first-time-right;
- maintenance access time, replacement time, occupied-unit service disruption and spare availability;
- material reuse/recovery, waste, embodied impact and lifecycle cost.

A density or cost improvement is not accepted if achieved by reducing required unit quality, fire/acoustic separation, accessibility, serviceability or resident control.

## 8. Current synthetic fixture boundary

The current `SYN-MF-COURTYARD-004` is a floor-envelope / space / portal / vertical-core identity fixture. It does **not** contain internal apartment rooms, wall thickness, real structural grids, facade openings, exact product assemblies, quantities, BOMs, manufacturing tolerances or site logistics.

Therefore all apartment assembly quantities, component identities, manufacturing methods, local supplier capacity, lift plans, installed cost and performance remain `NOT_COMPUTED` or `UNKNOWN`. Do not derive an assembly takeoff from the 14 space rectangles.

Next safe step: add resident-authored internal room geometry and explicit wall/opening/shaft interfaces, then bridge selected exact assembly classes to #803/#814 with independent known-answer takeoffs and product evidence.

## 9. Claim ceiling

This interface contract and its synthetic candidate do not demonstrate that any module is safe, code-compliant, fire/acoustically rated, structurally adequate, waterproof, thermally sufficient, locally available, cost-saving, manufacturable, transportable, installable or repairable. Production and site release remain with qualified designers, manufacturers, installers, inspectors and authorities.
