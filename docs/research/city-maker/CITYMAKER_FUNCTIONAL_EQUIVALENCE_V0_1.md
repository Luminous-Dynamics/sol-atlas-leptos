# CityMaker functional-equivalence profiles v0.1

**Status:** domain comparison contract proposal. Not a building code, performance specification, LCA calculation, material approval, permit or professional certification.

## Why this exists

Different construction systems can only be compared fairly when they deliver the same declared function. Equal area, mass, material grade, nominal cost or robot cycle time does not prove equivalence.

CityMaker compares nested service levels:
- component / assembly function;
- building function;
- neighbourhood / city service function.

A successful component does not automatically qualify the room, dwelling, neighbourhood or city that uses it. Interfaces, occupancy, operating schedules, service capacity, maintenance and access must be represented at the appropriate level.

## 1. Functional equivalence contract

Every comparison binds:
- scope digest: geography/site, climate file and version, use/occupancy profile, population frame, horizon and scenario family;
- service definition, beneficiaries, quantity, quality, availability windows and minimum floors;
- design/configuration/BIM revision, product/assembly identity, BOM and material lots;
- functional unit, reference flow and denominator;
- lifecycle boundary and included stages;
- mandatory performance requirements, units, threshold, method, applicability and qualified owner;
- operating assumptions, schedules, setpoints, ventilation, water use, maintenance and replacements;
- uncertainty semantic and evidence state for every datum;
- test/measurement method, calibrated instrument identity, date and acceptance authority;
- source, rights/provenance, version and content digest.

A difference in scope, functional unit, service level or lifecycle boundary means NotComparable unless a reviewed crosswalk exists. No implicit unit conversion or LCA allocation is permitted.

## 2. Functional-unit profiles

### FU-COMP-01 — installed assembly service

For the first panel-cell demonstrator, consider one square metre of **net installed, complete reference assembly**, but freeze geometry and system boundary before using it. Include the assembly's share of framing, sheathing, insulation, membranes, finishes, fasteners, connectors, penetrations, seals, offcuts, waste and accessories—not just its headline material.

The profile declares applicable requirements without selecting universal thresholds:
- thermal transmittance / resistance and thermal bridges;
- air, water, vapour and moisture behaviour under declared exposure;
- fire performance of the exact assembly and configuration;
- acoustic performance where required;
- dimensions, tolerances, connections and installation conditions;
- durability, inspection, maintenance and service-life assumptions;
- repairability, replacement, disassembly and end-of-life routes.

Separate profiles are required for roofs, floors, external walls, internal partitions, glazing, doors, shading, structural connections and service modules. A non-load-bearing panel cannot be presented as a structural system.

### FU-BLDG-01 — dwelling service-year

One year of delivered housing service for a declared household and occupancy profile. The reference profile includes:
- usable floor area and actual occupancy;
- indoor environment, thermal comfort and energy/water service;
- accessibility and life safety;
- repair, maintenance and service outage burden;
- housing, utility and operating cost assumptions;
- location-dependent access to essential services and jobs;
- embodied impacts amortized only with a declared service life and allocation method.

Keep physical performance separate from affordability and distribution. Do not allow cost or carbon benefits to compensate for inadequate health, safety, accessibility, water or comfort.

### FU-DIST-01 — neighbourhood service-year

A declared population/group receiving a specified service basket for one year. Specify each service contract separately: housing, potable water, sanitation, energy, mobility, health, education, food, public space, waste, digital access and emergency response.

The denominator is people/groups actually served, not mapped facility count. Service capacity, access, opening hours, reliability, eligibility, affordability and quality stay separate. Unknowns remain visible.

## 3. Non-compensatory gates

Applicable mandatory requirements are resolved before multi-objective comparison:
- legal land-use/building-control approval pathway;
- structural adequacy for declared loads, site conditions and design life;
- fire/life safety;
- moisture protection and durability;
- occupant health, ventilation and thermal comfort;
- universal accessibility;
- construction/install safety;
- material/product qualification and traceability;
- commissioning and maintenance ownership;
- land access, rights and anti-displacement safeguards;
- ecological, water and public-space constraints.

Missing calculations, tests, approvals, qualifications or site data are Unknown/Blocked, not favourable estimates. Speed, cost or carbon improvements cannot offset a failed mandatory gate.

## 4. Required output vector

Report separately:
- each mandatory requirement result;
- accepted functional output;
- first-pass and final acceptance yield;
- install and commission duration;
- total workflow labour by role, including setup, kitting, inspection, rework and recovery;
- lifecycle cost per accepted functional unit;
- lifecycle energy, water, virgin/recovered material, waste and GHG using consistent modules and source vintage;
- in-use performance when measured or modelled under declared conditions;
- maintenance hours, outage duration, replacement and repair burden;
- affordability, accessibility and distributional impacts;
- uncertainty, missing evidence, source coverage and comparability status.

No default scalar city/building score. A policy-weighted view, if commissioned, is a separately versioned deliberation artifact with disclosed weights and sensitivity.

## 5. Matched route protocol

The panel-cell routes B0 conventional, B1 productized/prefabricated, B2 assisted and B3 guarded automation must share:
- same released reference design and BOM;
- same materials, lot acceptance, storage and input quality;
- same function and independent inspection/acceptance predicates;
- same study boundary and included labour/work content;
- same denominator and treatment of rejects/rework;
- same lifecycle and environmental modules.

Report total workflow labour, not robot cycle time. Keep setup, replenishment, cleaning, inspection, rework, supervision and recovery in the declared denominator. Do not hide failed attempts.

## 6. Frozen synthetic case families

The initial known-answer corpus should contain at least 24 cases:
1. different materials achieve the same declared assembly performance;
2. nominal area matches but one system omits required layers;
3. thermal target passes while fire gate fails;
4. moisture evidence is unknown;
5. component passes but integrated interface fails;
6. component success does not imply dwelling-service success;
7. low-cost option has insufficient usable floor area/occupancy service;
8. average performance passes but some rooms violate a declared bound;
9. climate file revision changes comparability;
10. occupancy/use profile changes reference flow;
11. lifecycle study periods differ;
12. EPD declared units/modules differ;
13. replacement interval requires additional lifecycle flow;
14. recovered material has no avoided-virgin credit without allocation policy;
15. carbon improvement violates accessibility or affordability floor;
16. city water total passes but one catchment is undersupplied;
17. mapped service exists but hours/capacity are Unknown;
18. factory QA passes but installed inspection fails;
19. performance relies on unsupported maintenance;
20. simulation passes but physical claim lacks physical evidence;
21. robot route has faster cycle but greater total labour;
22. first-pass yield improves but final yield falls due to seeded defect;
23. BOM revision changes functional reference flow;
24. stale or missing data yields Unknown/NotComparable.

## 7. Evidence ladder

- E1: synthetic contract and independent oracle.
- E2: method reproduction on public/published reference data.
- E3: instrumented lab assembly with calibrated measurements and independent acceptance.
- E4: supervised site/building pilot with responsible professional review.
- E5: sustained operation, maintenance, repair, replacement and independent replication.

Tiers attach per claim. An E1 software PASS does not establish product, building or service performance.

## 8. Comparison and research anchors

- CEN EN 15978:2026 overview: LCA and quantified environmental performance across new/existing buildings and refurbishment; design-for-circularity information: https://www.cencenelec.eu/news-events/news/2026/en-in-the-spotlight/2026-04-17-en-15978-2026/
- EN 15804 EPD information is only comparable when product, declared unit, modules, PCR and validity match.
- IGLC 2026 Industrialized Construction Index has five maturity pillars—standardization, prefabrication, modularization, automation and digitalization—applied in Colombia and France. Use as a maturity crosswalk, not proof of performance superiority: https://www.iglc.net/papers/details/2492
- NIST's robot performance framework composes task-driven perception, mobility, dexterity and safety; its assembly tests require repeated measurements and statistical comparisons: https://www.nist.gov/programs-projects/performance-assessment-framework-robotic-systems and https://www.nist.gov/el/intelligent-systems-division-73500/robotic-grasping-and-manipulation-assembly/assembly

## 9. Ownership

- Sol-Atlas #612 owns this urban functional-equivalence profile and comparison scope.
- Symtropy #3605 owns generic accepted-output and full-workflow labour accounting.
- Symtropy #3773 owns the specific panel-cell comparison.
- Mycelix Living-City owns physical material/energy/water conservation and distributional impact.
- Sol-Atlas #555/#563 own group-specific accessibility and continuity projections.
- Relevant specialist engineering, statutory, measurement and safety owners remain authoritative.

## Claim ceiling

This profile makes alternatives comparable only for the functions, population, geography, time horizon, requirements and lifecycle boundary represented. It does not identify a universally best material, certify compliance, establish actual affordability or safety, or replace community and professional decision rights.
