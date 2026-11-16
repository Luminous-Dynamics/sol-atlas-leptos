# Home Expression Library and Experience Packages v0.1

**Issue:** [HOME-EXPRESSION-LIBRARY-001 #904](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/904)  
**Parent:** [Home Freedom Compiler #800](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/800)  
**Status:** Research contract and synthetic fixture; not a production design generator  
**Date:** 2026-11-16

## 1. Objective

Let each resident author the architectural expression and experiences of their home: choose one design language, blend several, create a custom direction, or leave style undecided. Let the same resident include ambitious optional spaces and amenities without hiding their physical, financial, water, energy, maintenance or delivery consequences.

The platform should support an attainable, repair-first starter home and a highly bespoke estate with the same respect for agency and the same evidence discipline. A higher budget does not make an unsafe or unpermitted design feasible. A restrained home is not a lesser outcome.

## 2. Four layers that must not be conflated

1. **Spatial typology:** courtyard, detached, row/terrace, duplex, pavilion, apartment, multi-generational, live-work, incremental core, retrofit-first.
2. **Architectural expression:** resident-authored form, composition, material, craft, colour, sensory and symbolic intent.
3. **Experience packages:** optional spaces or systems (library, observatory, spa, indoor pool, maker studio, etc.) that change the building's area, structure, services, cost and maintenance.
4. **Qualification:** the exact geometry, site, rights, building regulations, specialist simulations, professional reviews, supplier and construction evidence. These remain owned by the relevant specialists.

A style choice is not geometry. A renderer is not a structural solver. A schema-valid package is not a quote or a permit.

## 3. Starting architectural language library

These are search and conversation labels, not immutable style definitions, cultural categories or claims of authenticity. Each can be rejected, blended, renamed or replaced with resident-authored references.

| Family | Initial direction to explore |
|---|---|
| Highveld stone-and-timber contemporary | Tactile masonry, timber, deep shade, protected outdoor living |
| Cape/courtyard-informed vernacular | Courtyard, threshold, local climate response, material restraint |
| Contemporary / regionally authored modernism | Modern expression rooted in resident- and community-supplied references |
| Modernist | Clear geometry, functional legibility, spatial continuity |
| Mid-century modern | Human-scaled proportions, indoor/outdoor connection, crafted detail |
| Biophilic / organic | Living systems, natural textures, framed landscape |
| Tropical modern | Deep shade, cross-ventilation concepts, rain protection, outdoor rooms |
| Mediterranean courtyard | Enclosed garden, shaded arcades, textured surfaces |
| Japanese-inspired minimal | Restraint, thresholds, carefully framed views, craft |
| Industrial | Exposed construction logic, robust surfaces, adaptable workspace |
| Farmhouse / craft | Honest materials, domestic warmth, practical utility |
| Art Deco | Geometric ornament, crafted metalwork, strong composition |
| Brutalist / monolithic | Mass, texture, structural expression |
| Gothic Revival | Verticality, rhythm, ornament and crafted light |
| Heritage restoration | Conservation-led repair and context-specific retention |
| Solarpunk | Visible ecological systems, productive landscape, repairable technology |
| Minimalist | Reduction, clarity, proportion, controlled material palette |
| Maximalist | Layered colour, collections, texture and expressive rooms |
| Contemporary classic | Durable proportions with contemporary services and detailing |
| Custom | A resident-authored language with references and review context |

**Cultural and heritage guardrail:** The system must not infer culture, religion, identity or a supposedly authentic regional style from a resident's name, address, country, photograph or budget. Regionally meaningful references should be provided by the resident, with source, rights and contextual review status retained. A style label is an invitation to explore, not permission to appropriate.

## 4. Extraordinary experience library

### Spatial drama and architecture
- Double-height great hall, dramatic stair, daylit atrium, bridge gallery, sculptural lightwell.
- Indoor grove, winter garden, conservatory, orangery, enclosed courtyard, rooftop garden.
- Skybridge, cantilevered view deck and sheltered roof terrace (all require structural, fall, weather and access review).
- Hidden room or book-lined passage (must not compromise escape, accessibility or fire safety).

### Learning, culture and creation
- Grand library, private archive, reading gallery, art gallery, sculpture court.
- Music room, recording studio, rehearsal room, listening room.
- Maker laboratory, workshop, electronics bench, fabrication room, ceramics/woodworking studio with suitable extraction, fire and storage controls.
- Observatory with roof dome, telescope room and dark-sky terrace.

### Wellness, sport and leisure
- Cinema, immersive media room, gaming/arcade room.
- Spa, sauna, cold plunge, indoor pool, hydrotherapy room.
- Climbing wall, gym, dance/yoga studio, sport court, bowling or games room.
- Guest pavilion, multi-generational annex, private retreat suite.

### Food, landscape and ecological systems
- Indoor/outdoor kitchen, chef's kitchen, bakery, fermentation room, cellar.
- Greenhouse, edible garden, productive conservatory, aquaponics/hydroponics only where technically justified.
- Rain garden, visible water channel, reflection pool, water cascade, natural swimming pond.
- Solar pergola, battery/utility room, passive shading gallery, repair and resource workshop.

These are a menu, not recommendations. Some are expensive, maintenance-intensive, water-intensive or structurally demanding. The library never presumes that an amenity is affordable, appropriate, sustainable, safe or locally buildable.

## 5. Data contract

The machine-readable contract is `HOME_EXPRESSION_PROFILE_V0.1.schema.json`; the paired synthetic example is `HOME_EXPRESSION_PROFILE_SYNTHETIC_HIGHVELD_V0.1.json`; the non-exhaustive identifier/description catalogue is `HOME_EXPRESSION_CATALOG_V0.1.json`.

Every profile binds to:
- one exact Home Freedom brief ID;
- one exact design revision/artifact reference;
- known constraint IDs from the source brief;
- a resident-authored expression statement;
- protected qualities and explicitly avoided motifs;
- chosen / acceptable / avoided / undecided style preferences;
- optional experience packages and their delivery phase;
- impact states, evidence references and review plans.

Style blends use resident-authored percentages only to express emphasis. The percentage is not a beauty score, a probability, a quality ranking or a target the optimizer should maximize. For a blended profile, contributing PRIMARY and SECONDARY weights sum to 100. AVOID and ACCEPTABLE entries do not contribute.

### Experience impact completeness

Each package carries ten impact domains:

1. Area and circulation.
2. Capital cost.
3. Lifecycle operating cost.
4. Energy and peak demand.
5. Water and drainage.
6. Materials and carbon.
7. Maintenance and repair.
8. Accessibility.
9. Structure, fire, moisture and acoustics.
10. Sourcing and delivery.

For each domain, the profile records one of:
- `UNKNOWN`: no defensible value is available; numeric values remain null.
- `NOT_APPLICABLE`: a documented reason says the domain does not apply.
- `ESTIMATED`: a value or interval has a declared basis and unit.
- `MEASURED`: exact observation and method evidence are referenced.
- `QUALIFIED`: an appropriate specialist/authority has qualified the claim in the stated scope.

The synthetic fixture deliberately marks all impacts UNKNOWN. It must not invent costs, thermal benefits, embodied-carbon reductions, water efficiency or structural feasibility.

### 5.1 Multi-resident boundary

The v0.1 expression profile is bound to one exact source brief and one design revision. It must not be interpreted as a household-wide consensus or as proof that every resident accepts a proposed style/amenity blend.

For a shared home, preserve separate resident-authored intent and consent scopes. Do not average competing style weights, expose private preferences, silently relax one person's non-negotiable constraint, or assume agreement from a majority or from a generated candidate. Conflicting requirements remain visible and may produce separate candidates or an explicit unresolved state. The dedicated multi-resident owner is [HOUSEHOLD-PLURAL-INTENT-001 #907](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/907). Mycelix identity/consent and construction provenance remain separate owners.

## 6. Paradigm shifts

### 6.1 Style becomes a constraint-aware design variable, not a skin

Current workflows can begin with an inspiration image or an architectural typology and resolve cost, physics and constructability later. This programme carries style intent as editable structured input from the beginning. It allows the resident to compare multiple interpretations before committing to a single expression.

### 6.2 Amenities become service packages, not decorative checkboxes

An observatory is not just a roof icon. It changes roof geometry, structure, waterproofing, solar exposure, access and maintenance. An indoor pool changes water demand, humidity, filtration, structure, energy and whole-life cost. The package is not complete until its service and engineering consequences are described or explicitly UNKNOWN.

### 6.3 Every design is a revisable, evidence-linked composition

A style, finish or amenity change links to the existing Home Design Revision Impact owner (#832). Only unaffected evidence may remain valid, and only when unchanged dependency closure is demonstrated. Evidence that covers an earlier design revision must not silently certify a new revision.

### 6.4 Preference diversity is measured without one universal taste function

The system should preserve many legitimate aesthetic directions. It must not train residents toward one supposed ideal, use a global "beauty" score or make luxury the objective. Use resident choice, explicit exclusions, review notes and transparent Pareto comparisons.

### 6.5 Design-for-maintenance is part of luxury

A hidden pump room that cannot be serviced, a glass roof that cannot be cleaned, or an indoor waterfall that causes persistent moisture problems is not a high-quality amenity. Evaluate the maintenance path, replacement access, spare parts and service interruption alongside the visual experience.

## 7. Integration map

| Owner | Contract to reuse |
|---|---|
| Sol-Atlas #800 / #801 | Home Freedom resident brief and candidate set |
| Sol-Atlas #824 / PR #827 | Resident spatial-intent graph |
| Sol-Atlas #833 | Spatial programme |
| Sol-Atlas #806 | Loss-aware intent and accessibility bridge |
| Sol-Atlas #831 | Synthetic geometric layout qualifier |
| Sol-Atlas #832 | Revision and evidence invalidation graph |
| Sol-Atlas #811 / PR #812 | Material/component evidence and substitution |
| Sol-Atlas #803 / #814 | Assembly demand and DfMA delivery |
| Sol-Atlas #816 | Whole-household cost of place |
| Sol-Atlas #837 / #694 / #700 | Household appliances and service outcomes |
| Symthaea #6091 | Aesthetic, sensory and symbolic intent |
| Symthaea #6096 | Poetic engineering composition / PoeticMove |
| Symthaea #7790 / #7801 | Intent-to-design and lifecycle-aware alternatives |
| Symtropy #1766 / #1848 / #1900 | Spatial design, resilience and EnergyPlus qualification |
| Mycelix #5149 | Construction provenance, authority and approval receipt boundaries |

Do not create a second CAD kernel, BIM ontology, building physics engine, supplier system, cost model, consent authority or resident identity system.

## 8. Industry comparison protocol

Compare matched tasks rather than claiming that one integrated app "beats BIM."

| Comparator | What it is strong at | Matched Sol-Atlas evaluation |
|---|---|---|
| Finch3D | Generative layouts, firm plan libraries, real-time data and BIM geometry | Time to editable layout; resident requirement retention; code/accessibility constraints; geometry quality |
| TestFit | Site configuration, zoning constraints, typology iteration, pro forma and exports | Parcel/site feasibility, buildable area, yield, quantity/cost hand-off and false-feasible rate |
| Autodesk Forma | Early environmental analyses: sun, daylight, wind, noise, microclimate, solar and carbon | Site and massing alternatives under identical location/weather/context assumptions |
| Revit | Detailed parametric BIM, documentation, collaboration and interoperability | Information completeness, revision/change propagation, IFC round-trip loss and handover |
| IFC 4.3.2.0 + IDS 1.0 | Open model exchange and machine-readable delivery requirements | Entity/property/unit/provenance preservation and IDS conformance |
| EnergyPlus / OpenStudio | Detailed whole-building energy and water simulation | Same geometry, weather, schedules, internal loads, systems and model assumptions |
| Architect-led / quantity-surveyor workflow | Professional interpretation, local authority knowledge, design judgement and cost expertise | Same brief/site, paid time, iteration count, fee, cost basis and professional review level |
| Hybrid incumbent + compiler | Existing specialist toolchain with a resident brief and evidence layer | End-to-end time, rework, provenance, handover and lifecycle completeness |

### Important scope limits

- Finch3D claims layout generation grounded in firm standards and local codes; measure its actual output and the selected supported code profile, not only marketing language.
- TestFit's Site Solver focuses on parcel-to-pro-forma feasibility. Its own product page describes U.S.-oriented parcel/soil integrations, so a South African comparison needs a matched imported/verified local data path rather than assuming native municipal coverage.
- Forma's early analyses do not substitute for a detailed building energy model, professional structural design, fire-engineering review or municipal approval.
- Revit is the incumbent detailed BIM/documentation comparator, not a single universal solution for resident governance, evidence provenance or household operations.
- EnergyPlus is a simulation comparator; its output is only as trustworthy as the model, weather, assumptions and validation.

Official product/standard sources, checked 2026-11-16:
- Finch3D: https://www.finch3d.com/product
- TestFit Site Solver: https://www.testfit.io/product/site-solver
- Autodesk Forma environmental impact analysis: https://www.autodesk.com/za/products/forma/environmental-impact-analysis
- Autodesk Revit features: https://www.autodesk.com/ae/products/revit/features
- buildingSMART IFC: https://www.buildingsmart.org/standards/bsi-standards/industry-foundation-classes/
- buildingSMART IDS: https://www.buildingsmart.org/standards/bsi-standards/information-delivery-specification-ids/
- EnergyPlus 26.2: https://energyplus.readthedocs.io/en/v26.2.0/

## 9.1 Current research frontier (checked 2026-11-16)

A particularly relevant recent comparator is Kembhavi et al., **"An evaluation guided multiagent framework for generating floor plans that comply with municipal building regulations"**, *Discover Artificial Intelligence*, published 2026-10-15, DOI 10.1007/s44163-026-02481-9. Its LayoutGen system uses retrieval, role-separated agents, deterministic geometric evaluation and iterative refinement. Across 200 runs, the paper reports a reduction from 3.7 to 0.3 detected violations per plan relative to an unretrieved, unrefined single-shot baseline using the same backbone.

The comparator is promising, but its claim boundary matters:
- evaluated single-storey, axis-aligned rectangular residential rooms on RPLAN/MSD-derived configurations;
- counted topological, geometric and encoded dimensional errors (minimum room area, minimum room width and critical width);
- left setbacks, floor-area ratio, egress, stair geometry, parking and ventilation outside the generation-loop checks;
- demonstrated rule-repository swapping across three Indian municipal regimes with comparable dimensional-rule structure, not cross-country generality;
- does not establish structural, MEP, water, cost, whole-life or construction qualification.

**Our target:** match the useful neuro-symbolic pattern—generation informed by deterministic independent feedback—but require exact source rule provenance, hard-gate semantics, unknown/stale/conflicted outcomes, revision-bound evidence, broader specialist checks and competent professional authority. A 92% relative reduction on a narrow subset is not a zero-false-pass guarantee for an occupied dwelling.

Useful research baselines:
- **RPLAN:** historical residential plan corpus for graph/geometry and distributional layout comparisons. Use it only for a clearly scoped generative subtask; do not treat its distribution as a universal cultural or functional ideal.
- **Tell2Design:** paired natural-language instructions and RPLAN plans for instruction-conditioned generation. Evaluate paired edit fidelity separately from resident agency.
- **MSD:** additional building-layout benchmark population for cross-dataset generalization; preserve the actual split and source task.
- **Human/architect review:** use blinded paired comparisons for resident-defined usability and preference; never substitute an LLM aesthetic score for resident judgement.

A 2025 review of computer-aided building layout generation identifies floor-plan, scene-layout and other building-layout generation as distinct tasks with different benchmark datasets and metrics. Therefore report image realism metrics (e.g., FID), graph/topology validity, vector geometry, constraint satisfaction, functional circulation, accessibility and resident judgement as separate results. Do not collapse them into one leaderboard.

References:
- LayoutGen (2026): https://link.springer.com/article/10.1007/s44163-026-02481-9
- Computer-Aided Layout Generation for Building Design: A Review (2025): https://arxiv.org/abs/2504.09694
- Tell2Design / RPLAN is discussed in the LayoutGen paper; the benchmark must pin the exact public dataset version and test split before comparison.

## 9.2 Independent certification and local review

Green Star, Net Zero and EDGE are certification routes, not design generators or substitutes for local statutory approvals. The Green Building Council South Africa certification process requires an appropriately accredited professional and an independent technical assessment. Treat these as downstream assurance/certification comparators, not as an automatic feature-score output from the home compiler:
https://www.gbcsa.org.za/the-certification-process/

For South African building work, local-authority approval and the applicable National Building Regulations/SANS 10400 path remain explicit. Municipal application requirements differ; an example municipal submission route lists site/layout plans and other discipline documents according to the scope of work:
https://www.capetown.gov.za/city-connect/apply/planning-building-and-development/building-plan-applications/submit-a-building-plan/

## 9. Benchmarks and targets

Use the existing 100-case Home Freedom corpus (#819) as the feasibility oracle. Add style/experience counterfactuals to it; do not create a competing source of truth.

### P0 — contract correctness
- 100% exact link to the source brief ID and revision ID.
- Zero silent changes to hard constraints or resident exclusions.
- 100% of selected/considered packages carry all ten impact domains.
- Zero numeric zero-fills for UNKNOWN costs, water, energy or performance.
- 100% of resident-required packages cite a real non-negotiable source constraint.
- 100% exact requalification of changed dependencies; unaffected evidence survives only with proven unchanged dependencies.
- No automatic aesthetic approval, construction authorization or professional certification.

### P1 — generation and comparison
- At least three materially distinct candidates when at least three are genuinely feasible; do not fabricate candidates when fewer are feasible.
- Preserve the feasible Pareto set across cost, household services, climate resilience, material availability, repairability and resident-stated preferences.
- Measure option diversity (style/space/material/experience), preference preservation and resident revision acceptance separately.
- Report time-to-first-editable-option, time-to-first-feasible-option, number of resident corrections, and downstream professional rework.
- Report all failed, uncertain and out-of-domain cases, not only successful generations.

### P2 — physical and delivery performance
- Thermal comfort in occupied degree-hours using pinned weather files and occupancy schedules.
- Energy and peak demand by end use, not an unqualified kWh total.
- Water withdrawal, recovered water, fit-for-purpose delivered water, residuals and service interruptions by source/end use.
- Whole-life cost with dated local rates, quote provenance, uncertainty intervals and distributional burden.
- Embodied carbon by declared boundary and data source.
- Fabrication time, installation tolerance, labour, rework, waste, repair access and material substitution.
- Post-occupancy satisfaction and adaptation outcomes, only with resident consent and comparable measurement.

A synthetic schema pass is not physical validation. A simulated performance improvement is not a measured one. A matched industry trial must use the same site, programme, code assumptions, climate, service boundary, review maturity and cost basis.

## 10. South African deployment boundary

Start with Johannesburg / Highveld as a regional research profile, not a universal default. Maintain the current municipality, land-use rights, survey, servitudes, soil, service connections and applicable editions/amendments of the National Building Regulations and SANS 10400 as explicit inputs.

The South African building-plan route requires local-authority approval and relevant plans and supporting documentation; the local authority and competent professionals remain the approval path. The compiler must never infer approval from an IFC export, IDS pass, geometry check or simulated result.

## 11. Claim ceiling

This increment provides:
- a versioned resident expression and experience package contract;
- a synthetic profile;
- a deterministic Rust qualifier for contract-level invariants;
- a research and benchmark protocol.

It does **not** provide:
- a production architectural style recommender or image renderer;
- an actual floor plan or BIM model;
- cost estimates, vendor quotes or supplier availability;
- structural, fire, thermal, moisture, acoustic, accessibility or municipal approval;
- a claim that an amenity is safe, sustainable, buildable or affordable.

The correct result for missing evidence is UNKNOWN, not optimism.
