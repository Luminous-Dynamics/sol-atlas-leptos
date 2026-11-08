# CityMaker construction-method comparison matrix v0.1

**Status:** research and experiment specification. It is not a design recommendation, cost database, code interpretation, material approval, robot qualification or measured performance result.

**Scope:** above-ground buildings, factory-made building components, assembly, retrofit, public realm and construction logistics. Buried services may appear only as external constraints/interfaces; underground robotics and tunnelling are excluded.

## 1. The central decision change

Do not select a construction method from a material label, a robot demo, a factory utilization number or a single cost-per-square-metre estimate.

Compare **functionally equivalent, independently accepted building outcomes** across the same site, climate, occupancy/service requirements, study period, delivery boundary and evidence rules.

The comparison hierarchy is:

1. **Component / assembly:** e.g. one accepted installed square metre of an external wall or roof assembly meeting the same declared structural, fire, moisture, thermal, acoustic, durability, maintainability and appearance requirements.
2. **Building service:** e.g. one dwelling-year of safe, habitable, accessible housing at a declared occupancy/service level, inside a stated affordability and operating boundary.
3. **District service:** e.g. a completed, operational neighbourhood service basket over a declared period, including delivery, maintenance, access, resilience and distributional outcomes.

A square metre of wall is not equivalent if one solution needs more area, different conditioning energy, more maintenance, shorter replacement intervals or different usable floor area. A dwelling is not equivalent if it has a different service level, occupancy, safety, comfort, accessibility or lifecycle boundary.

**Do not normalize away real differences.** Define a functional unit, then report the raw quantities and the assumptions used to achieve that function.

## 2. Non-compensatory gates before comparison

Every alternative must pass or explicitly resolve the requirements that cannot be traded away:

- statutory land-use, building-control and professional-approval pathway;
- structural system and connection design for the actual site/actions;
- fire strategy, smoke/egress and material/system fire evidence;
- weatherproofing, moisture, drainage, condensation and durability;
- safe installation, temporary stability, lifting and work-at-height controls;
- universal access and usable route/building interfaces;
- indoor health, ventilation and thermal-comfort requirements;
- construction-worker safety and human–robot application risk assessment;
- receiving, inspection, commissioning, operation and maintenance ownership;
- rights, land access, community participation and displacement safeguards;
- required ecological, water, waste and public-space constraints.

A missing specialist calculation, unresolved site condition or unqualified material is **Unknown/Blocked**, not a lower score. Faster, cheaper or lower-carbon estimates cannot compensate for a failed mandatory gate.

## 3. Method portfolio — hypotheses, not a fixed ranking

| Method family | Where it may be competitive | Main constraints / failure modes to test | Promising automation tasks | Critical evidence before a real claim |
|---|---|---|---|---|
| Maintain / repair / retrofit / adaptive reuse | Existing service can be preserved; avoids unnecessary new material and disruption; often the first alternative to test | Unknown as-built condition, hazardous materials, hidden defects, non-conforming legacy systems, access constraints, disruption while occupied, limited upgrade space | Survey, scanning, defect mapping, selective removal, component identification, drilling/cutting under controls, repair kitting, inspection and reinstatement | Condition survey, intrusive investigation where needed, hazardous-material controls, measured retained/replaced quantities, post-repair test and occupied-building safety |
| Site-built masonry / block / brick | Local trades and supply may already exist; flexible on irregular sites and small changes | Labour variability, wet-trade sequencing, mortar curing, material handling, quality variation, thermal/moisture detailing, scaffold and ergonomic burden | Pallet/kitting logistics, block/brick handling assistance, mortar dosing, layout/setting-out, cutting under guarding, joint/finish inspection | Product standards and batch records, workmanship inspection, wall-system tests, moisture/weather details and complete labour/rework baseline |
| Cast-in-situ reinforced concrete frame with infill | Complex geometry and continuity may be achievable with familiar engineering routes | Formwork/rebar labour, curing/sequence dependencies, temporary works, embodied impacts, site congestion, inspection/cover/placement quality | Rebar processing/cage assembly, formwork handling, drilling/tying assistance, concrete placement monitoring, surface/geometry survey | Structural design and temporary-works approval, reinforcement/cover records, concrete batch/test records, curing and defect/rework evidence |
| Precast concrete panels / elements | Repeatable geometry and factory QA may help for repeated layouts; site time can shift toward installation | Transport dimensions/mass, cranes and lifting design, tolerances and connections, storage/handling damage, factory/transport/site coordination | Rebar/formwork preparation, mould setup, panel finishing, dimensional metrology, lifting-point verification, load presentation and alignment assistance | Exact product/element qualification, lifting plans and equipment evidence, connection design, dimensional survey, transport and installation records |
| Cold-formed / light-gauge steel framing | Repetitive light framing, dry assembly and CNC/roll-form workflows can suit productization | Corrosion/fire/acoustic/thermal bridging, fastener patterns, bracing, tolerance and service penetrations, supplier/coil/section traceability | Coil/section handling, roll-forming, cut/mark, hole placement, panel assembly, screw fastening, inspection and kitting | Material/coil certificates, exact section revision, structural/fire/acoustic/moisture design, connection checks and as-built fastener/geometry inspection |
| Timber frame / cassette / panel | Lightweight components, factory cutting and repeatable assemblies; potential for repairable/demountable details | Moisture, biological durability, dimensional variation, knots/defects, fire strategy, acoustic/vibration performance, supply legality and treatment | Scanning/grading, board handling, optimized cutting, drilling/milling, joinery, panel fastening, adhesive control where used, inspection | Timber species/grade/moisture and source evidence, connection/fire/moisture/durability design, process/adhesive controls, calibrated geometry and installation inspection |
| Engineered / mass-timber components (e.g. CLT/GLT where applicable) | Large, repeatable elements and hybrid structural systems may enable off-site manufacture | Qualified supply depth, panel dimensions and transport, connections, fire/moisture, acoustic/vibration, opening/penetration coordination and repair | CNC machining, panel handling, drilling/fastening, dry fit, robotic or assisted placement guidance, metrology | Product-specific declarations and structural qualification, fire and moisture strategy, connection/diaphragm design, lift/transport proof and as-built inspection |
| Panelized hybrid systems | Can separate structure, insulation, services, windows and finish layers into coordinated replaceable assemblies | Interface complexity, tolerance accumulation, thermal bridges, penetrations, service coordination, transport and site closure quality | Gantry/robotic cutting, pick-and-place, framing, fastening, adhesive/sealant application, MEP insertions, vision/metrology, automated documentation | Interface specifications, full-wall/mock-up testing, fire/moisture/acoustic/thermal evidence, manufacturing QA and receiving/installation acceptance |
| Volumetric modular | Repeated room/functional modules with enough repetition and transport/cranage feasibility | Transport envelope/route, lifting and temporary stability, tolerance at module joins, services connections, fire/acoustic interfaces, site access and capital utilization | Jigs, framing/MEP assembly, dimensional QA, module handling, crane/placement assistance, connection inspection | Complete module/system qualification, transport/lift engineering, connection and service tests, factory/site interface acceptance, commissioning and maintainability |
| Additive construction / 3D concrete printing | Geometry or formwork reduction may be useful for bounded components and qualified process windows | Printable mix/material availability, reinforcement, interlayer bonding, anisotropy, curing/environment, durability, inspection, scale and regulatory acceptance | Material batching, controlled extrusion, bead/geometry metrology, path planning, nozzle/flow monitoring, controlled finishing | Mix/process qualification, interlayer and reinforcement strategy, full-scale component tests, durability/fire/moisture/structural evidence and independent inspection |
| Earth-based / earthen systems (e.g. compressed earth block, rammed earth) | Local material and low-processed construction may be attractive in suitable contexts | Soil variability, stabilization/additives, water erosion, moisture, structural/seismic/fire requirements, quality control and certification route | Soil testing/sorting, controlled mixing/pressing, block handling, panel/element inspection, moisture measurement | Geotechnical/material characterization, mix/stabilizer evidence, water/moisture and structural tests, code route, batch control and workmanship QA |

These families are not mutually exclusive. The best deliverable may be a hybrid: retained structure + new panelized envelope + factory-made service modules + site-built foundations or interfaces. A construction method portfolio should include the maintain-existing baseline and combinations where the boundaries are clear.

## 4. Design for manufacturing and assembly is a city-design variable

The design should expose, early enough to change:

- structural grid, spans, openings and repetition;
- component family, variants and unique-part count;
- connection type, fastener access, reversibility and inspection access;
- tolerance chain and datum strategy between trades/factory/site;
- transport dimensions, mass, centre of gravity, lifting points and laydown needs;
- workcell access, tool reach, fixture strategy and safe human/robot separation;
- sequence, temporary support, weather protection and hold points;
- service zones, access panels, replaceable equipment and future upgrades;
- maintenance, repair, disassembly and end-of-life access;
- local supplier availability and qualified substitute routes.

A design that is cheaper to manufacture but impossible to transport, lift, connect, inspect or repair is not a better city-building method.

## 5. Specialized robotics portfolio

Do not set a universal humanoid as the baseline. Compare the right mechanism for the task:

| Robot / automation family | Appropriate first tasks | Key measured capability | Hard boundary |
|---|---|---|---|
| Fixed gantry / Cartesian cell | panel framing, drilling, fastening, adhesive/sealant or finish application under a declared envelope | dimensional repeatability, tool access, cycle distribution, setup and changeover | fixed guarded cell only until physical integration and risk assessment qualify more |
| Industrial arm + fixture | cutting, machining, drilling, fastening, sanding or controlled placement | pose accuracy, payload, tool force/torque, calibration drift, fixture dependence | industrial-cell safety and application-specific tooling/guarding |
| Mobile material-handling platform | kitting, component presentation, internal factory logistics | payload, stability, stopping, route completion, manual recovery and uptime | no public/occupied-site autonomous movement without separately qualified system and legal operating envelope |
| Mobile manipulator | supervised pick/present/align tasks in semi-structured spaces | perception, dexterity, reachability, recovery rate, handoff and uncertainty | human-approved, limited-scope task; not a general construction authority |
| Survey/metrology platform | as-built geometry, layout checks, panel/connection inspections | measurement uncertainty, registration, repeatability and coverage | sensor reading is not engineering approval |
| Lifting / positioning assistance | present and align panels or modules under approved lift plans | load state, pose, alignment, communication, stop and recovery evidence | lifting equipment, rigging and qualified operator remain under their governing safety/engineering processes |

A task should only be assigned to a robot when the declared task, machine/tool configuration, calibration, work envelope, payload, material, fixture, site condition and safety prerequisites match the current evidence. Unknown capability blocks the claim.

## 6. Matched experiment design

### 6.1 Common article and functional requirement

First reference article: a **non-load-bearing modular wall/panel assembly** with a frozen design revision, declared openings and interfaces, repeatable materials and documented acceptance tests. It is deliberately not a structural wall or a building-code approval exercise.

Freeze:
- design and BOM revision;
- materials, lots, grades and storage conditions;
- functional unit and boundary;
- dimensions/tolerances and measurement-system capability;
- connection/fastener specification;
- inspection and acceptance predicates;
- work content and inclusions/exclusions;
- operator/training/shift assumptions;
- plant/facility and tooling configuration;
- source data, scenario, seed and evaluator revisions.

A later article may add a structural or weather-exposed function only under a new professional design and test plan.

### 6.2 Routes

- **B0 — conventional baseline:** representative manual/site-built or conventional workshop route, precisely described.
- **B1 — productized/prefabricated:** standardized panel and fixtures, human-operated manufacture and installation.
- **B2 — assisted cell:** ergonomic material presentation, guided alignment, metrology and selected tool assistance; humans retain task control.
- **B3 — guarded automation:** fixed, risk-assessed cell for bounded repeatable operations. The system is not assumed to run unattended.
- **B4 — optional alternative:** a second material/system route (e.g. steel vs timber/hybrid) only if the same functional and acceptance requirements can truly be met.

The routes need not use identical machines; they must share the same output specification, boundary, acceptance protocol and denominator.

### 6.3 Fair-trial protocol

Use randomized or blocked run order where practical to limit learning/material-order effects. Preregister repetitions and analysis before inspecting results. Include operators across relevant experience levels; make training time and intervention burden visible. Keep all failed/rejected attempts in the denominator. Use calibrated instruments and an independent acceptance evaluator. Separate synthetic fixtures, simulation, recorded replay, instrumented lab articles and field pilot results.

Do not set the headline productivity target before establishing a trustworthy B0 baseline. A proposed improvement is not achieved until the measured comparison, uncertainty and guardrails support it.

## 7. Metrics — report independent dimensions

**Accepted output and productivity**
- accepted units / scheduled production hour;
- accepted units / total workflow labour-hour, including setup, kitting, replenishment, cleaning, supervision, inspection, rework, recovery and training as declared;
- first-pass yield and final acceptance yield;
- p50/p90 cycle time, lead time, changeover and schedule adherence;
- manual interventions, recoveries and restart time.

**Quality**
- dimensional error distribution and uncertainty;
- defect class, severity and rate;
- rework and scrap per accepted unit;
- missed-critical-defect count;
- connection/fastener/adhesive/finish inspection results as applicable;
- measurement-system identity and calibration status.

**Economics**
- fully burdened cost per accepted unit;
- capital/tooling and amortization assumptions;
- material, transport, energy, consumable, maintenance and labour costs;
- rejected output, rework and warranty/repair exposure;
- cost variance and sensitivity to utilization/volume.

**Human and safety**
- hazardous exposure removed and new hazard exposure introduced;
- ergonomic load, awkward postures, repetitive handling and lifting burden;
- entries into guarded/exclusion zones, unexpected stops and recovery events;
- supervision/monitoring attention and task interruption;
- task skill retained, training time, role distribution, usability and meaningful human override.
- Perceived safety and usability are supplementary human-factor outcomes, not a safety certification.

**Lifecycle / environment**
- virgin material, reclaimed material, water and energy per accepted functional unit;
- construction waste, offcuts, rework and measured recovery by destination/grade;
- transport and packaging burden;
- embodied/operational impacts within a common lifecycle boundary;
- repairability, replacement time, part/tool complexity and maintenance burden;
- in-use service and replacement assumptions, not only factory gate impacts.

Do not collapse these into a universal efficiency score. Any weighted policy must be separately versioned, have declared authority, publish units/normalization/weights and show sensitivity. Default output is a vector and Pareto set.

## 8. Benchmark targets and evidence ladder

### E1 — synthetic software correctness
- 100% agreement with an independently written finite-case oracle.
- Zero candidate survives a seeded hard-gate violation.
- Zero promotion of synthetic/simulated results to measured or qualified physical claims.
- Exact-head, exact-corpus receipts; deterministic replay and order-invariant output.
- Negative controls for stale calibration, wrong revision, wrong unit, missing inspection, geometry/reach conflict, duplicate material, missing load/transport limit and failed safety gate.

### E2 — matched external/reference-method comparison
- Compare robot subskills with NIST task-oriented assembly measures for perception, grasping/manipulation, joining/fastening and human–robot interaction where the task is applicable.
- Use multiple repetitions and statistical reporting; do not translate a small-parts task-board result into a whole-building productivity claim.
- Report context-of-use and differences from the reference task/protocol.

### E3 — instrumented laboratory article
- Same frozen article and acceptance standard across B0–B3.
- Independently calibrated metrology and documented material/lot identity.
- Preregister repetitions and analysis; report raw trial outcomes, p50/p90, uncertainty, defects and labour by role.
- Zero known critical acceptance failures in accepted output. All misses and rejected items remain visible.
- No safety claim without the relevant risk assessment, engineered controls, validation and accountable commissioning.

### E4 — supervised field pilot
- Same acceptance and evidence rules in actual site conditions.
- Record weather/site changes, logistics, workforce, setup, recovery, stoppages and service disruption.
- Independent inspection and responsible professional review.
- Measure installation and downstream performance, not just factory output.

### E5 — sustained operation and replication
- Repeated production across lots, shifts, operators, maintenance cycles and demand variation.
- Independent replication by a second team/site or a clearly bounded external adopter.
- Lifecycle repair/replacement and actual operating performance.

## 9. Industry / state-of-the-art comparison

### Industrialization maturity

The 2026 Industrialized Construction Index (ICI) measures five distinct pillars: standardization, prefabrication, modularization, automation and digitalization, and reports a multi-case application across residential projects in Colombia and France. CityMaker should crosswalk those five dimensions separately, then measure actual outcomes independently. Maturity is not productivity and cannot replace quality, cost, schedule or safety evidence.

Autodesk's 2026 survey of 869 industry respondents reports regular/standard use of productization, DfMA and prefabrication at 37–40%; respondents using industrialized construction often report quality and predictability benefits. This is survey evidence about adoption/perception, not a matched causal experiment or universal performance baseline.

### Robotic capability measurement

NIST's Performance Assessment Framework for Robotic Systems treats perception, mobility, dexterity and safety as components that must be measured and composed into task-specific system performance. NIST's task-board work includes grasping/manipulation, joining/fastening and deformable-part tasks, and its assembly guidance emphasizes repeat trials and statistical comparison. CityMaker should adopt this task-based discipline and extend it to large, variable building components, transport interfaces, installation and accepted building outputs.

A 2026 IEEE review of on-site construction automation organizes deployments around assembly/placement, inspection/mapping and drilling/surface operations; it identifies evolving site geometry, incomplete digital information, human presence, perception robustness, integration, safety and workforce adaptation as core challenges. These are explicit stress dimensions in the CityMaker benchmark, not footnotes.

### Environmental and lifecycle method

UNEP's 2025–2026 Global Status Report says buildings and construction account for about 37% of global emissions and nearly 50% of global material extraction. Those are global sector aggregates, not local Johannesburg factors. The report strengthens the case for measuring embodied material, operational energy, affordability and resilience together.

Use the current applicable building LCA method and database version. CEN published EN 15978:2026 as a harmonized method covering new and existing buildings and refurbishment, lifecycle stages and design-for-circularity information. ISO 14040/14044 remain the general LCA principles/framework and requirements/guidelines. Do not combine EPDs, local data and generic databases without documenting geography, age, modules, cut-offs and declared/functional units.

### Safety scope

ISO 10218-1:2025 addresses industrial robots; ISO 10218-2:2025 addresses industrial robot applications and cells. They are relevant to bounded factory-cell engineering within their scope. They do not independently authorize unrestricted robots in public or occupied spaces, mobile construction applications, lifting systems or every hazard of a construction process. Each application needs its own risk assessment and relevant standards/regulatory path.

### Construction productivity and prefabricated MEP — 8 November 2026

A useful cross-industry benchmark is the **Cambridge Framework for Construction Productivity: Measure · Benchmark · Improve** (published 30 September 2026). It was developed using data from more than 25 UK projects and structures productivity at project, work-package and activity levels, connecting production rates, labour productivity, production flow, variability and delivery blockers. CityMaker should align its work-package/activity-level measures with this framework rather than invent a competing generic construction-productivity ontology. Its distinct contribution is to connect those established measures to robot/task-level execution, exact design/material/configuration identity and independently accepted output.

- Cambridge framework summary: https://www.construction.cam.ac.uk/news/new-cambridge-framework-aims-transform-how-construction-productivity-measured-and-improved
- Full report repository and DOI: https://doi.org/10.17863/CAM.135013

For prefabricated MEP, a February 2026 five-project study identifies tolerance management as a major unresolved productivity/quality issue. It proposes principles emphasizing communication, detailed design, quality control and buffer assembly; none of the five projects applied every principle. CityMaker's service-cassette trial now requires a tolerance chain from the host as-built datum through module manufacture, buffer allowance, installation and final interface verification.

- Li et al. (2026), *Tolerance management in prefabrication of mechanical, electrical, and plumbing installations*: https://doi.org/10.1080/01446193.2026.2626735

Singapore's BCA describes prefabricated MEP as integrated off-site subassemblies and reports time savings up to 60% in selected project contexts. This is a useful industry example and hypothesis, not a transferable target; the CityMaker baseline must be established on a matched article with the complete workflow boundary.

- BCA Prefabricated MEP System (updated 1 October 2026): https://www1.bca.gov.sg/growth-and-transformation/productivity/design-for-manufacturing-and-assembly-dfma/prefabricated-mechanical-electrical-and-plumbing-mep-system/

A September 2026 study of 491 modular projects (303 permanent and 188 relocatable) uses gross square metres completed per day to analyse productivity by building type and project size. This can inform stratification and warns against pooling unlike project sizes/types, but gross area/day is not sufficient for our primary measure because it does not independently establish quality, functional equivalence, whole-workflow labour, service performance or lifecycle burden.

- Hasan & Choi (2026), *Correlation between Size and Productivity for Modular Buildings*: https://doi.org/10.1061/JMENEA.MEENG-7605

## 10. City-scale implication: co-design the production geography

The construction-method comparison is only one level. At city scale, compare where the production ecosystem should exist:

- component/panel factories and their catchments;
- repair, refurbishment and salvage hubs;
- timber/steel/concrete/board suppliers and qualified alternatives;
- kitting, storage, consolidation and last-mile logistics;
- workforce access, training and maintenance skills;
- energy/water capacity and backup;
- construction staging and temporary public-realm disruption;
- future repair/replacement access and material recovery.

Do not infer local productive capability from a map pin or declared warehouse. Use Mycelix #4814 for supplier/inventory/dependency facts, Symtropy #3768 and #3773 for service/workcell feasibility and panel cell trials, Symthaea #5774/#8051/#7608 for capability/design/value reasoning, Mycelix #5149 for accountable material and project provenance, and Sol-Atlas #569/#573 for read-only spatial and delivery projections.

## 11. Research anchors

- Sanchez & Attouri (2026), *Industrialized Construction Index (ICI): a project indicator for industrialization assessment*, IGLC 34, DOI 10.24928/2026/0184: https://www.iglc.net/papers/details/2492
- NIST, *Performance Assessment Framework for Robotic Systems*: https://www.nist.gov/programs-projects/performance-assessment-framework-robotic-systems
- NIST, *Assembly Performance Metrics and Test Methods*: https://www.nist.gov/el/intelligent-systems-division-73500/robotic-grasping-and-manipulation-assembly/assembly
- Kimble et al. (2022), manufacturing grasping/manipulation/assembly task boards, DOI 10.3389/frobt.2022.999348: https://www.nist.gov/publications/performance-measures-benchmark-grasping-manipulation-and-assembly-deformable-objects
- IEEE (2026), *Construction Robotics for On-Site Automation: A Review of Research and Applications*, DOI 10.1109/ISoIRS70157.2026.11545216: https://ieeexplore.ieee.org/abstract/document/11545216
- Autodesk (2026), *State of Design & Make: Spotlight on Industrialized Construction*: https://www.autodesk.com/design-make/articles/2026-state-of-design-make-spotlight-on-industrialized-construction
- UNEP / GlobalABC (2026), *Global Status Report for Buildings and Construction 2025–2026*: https://wedocs.unep.org/items/dd0a1d99-dba3-46da-93ab-ef2a26fc3c14
- CEN (2026), EN 15978:2026 overview: https://www.cencenelec.eu/news-events/news/2026/en-in-the-spotlight/2026-04-17-en-15978-2026/
- ISO 14040 LCA framework: https://www.iso.org/standard/37456.html
- ISO 14044 LCA requirements/guidelines: https://www.iso.org/standard/38498.html
- ISO 10218-1:2025 industrial robots: https://www.iso.org/standard/73933.html
- ISO 10218-2:2025 industrial robot applications and cells: https://www.iso.org/standard/73934.html

## Claim boundary

This matrix is a method-selection and experiment-design aid. It does not recommend one material/system for Johannesburg, establish actual costs or productivity, certify a building/robot, validate local supply chains, or represent community preference. A defensible claim of superiority needs identical functional requirements, full boundaries, measured outcomes, uncertainty, independent acceptance and review by the appropriate professionals and affected people.


## 12. Thermal service and climate resilience must be a first-class design output

Construction-method comparisons cannot end at embodied carbon, annual energy or nominal U-value. Each functionally equivalent assembly and building alternative must be evaluated in the context of its climate, occupant/service profile and operating constraints.

Report, where the qualified method supports them:
- summer daytime peak and hot-night persistence;
- indoor operative temperature and overheating duration/degree-hours under a declared method;
- winter nighttime comfort and heating need;
- humidity/moisture and condensation risk;
- natural-ventilation availability when outdoor air quality, temperature, wind, security and noise permit it;
- peak cooling demand and grid-outage behaviour;
- water demand for any evaporative strategy;
- maintenance, cleaning, repair and component replacement;
- uncertainty under observed, typical, extreme and future weather profiles.

A typical meteorological year is useful for energy screening but does not establish performance during an observed heatwave or a future climate. Modelled trajectories and measured indoor outcomes must retain separate evidence labels.

Use the existing Symtropy thermal and resilience owners:
- #1766 resident-led spatial design;
- #1848 time-dependent habitability under heat/cold and utility outages;
- #1900 version-pinned EnergyPlus adapter;
- #1790 solver-neutral simulation boundary.

Sol-Atlas [CITY-HEAT-RETROFIT-ATLAS-001 #593](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/593) will spatially project outdoor exposure, building cohorts, modelled thermal trajectories, measured outcomes and retrofit delivery capacity. It must not replace those solvers or turn ward-scale population/weather evidence into per-building indoor temperatures.

Recent research reinforces the need for evidence-rich field comparison:
- A randomized pilot published on 5 January 2026 in 40 rural African houses found different day/night responses across cool roofs, mat ceilings and cross-ventilation; its effects are specific to that study's climate and housing, not transferable Johannesburg coefficients. This strengthens the case for measured local prototype trials rather than simulation-only rankings.

References:
- https://doi.org/10.1038/s41591-025-04104-9
- https://doi.org/10.1038/s44359-026-00177-y
- https://climate.onebuilding.org/
- https://data.ashrae.org/standard140/


## 12. South African approval and current standards watch (checked 7 November 2026)

This method matrix is global in its research comparisons, but any actual Johannesburg building or construction-system choice must follow the South African regulatory, municipal and product-assessment pathway. The software must therefore store **jurisdiction + rule/standard edition + effective date + building/use class + approving authority + design responsibility + evidence references** as part of every hard gate. Do not apply a UK/EU/US method or certificate as if it were South African approval.

### National and local building approval

- The National Building Regulations and Building Standards Act 103 of 1977 is the national statutory basis. A local authority's building-control process and current applicable parts of SANS 10400 / referenced design standards must be checked for the actual building and use. The official South African Government Act page is an entry point, not a complete live register of every amended standard: https://www.gov.za/documents/national-building-regulations-and-building-standards-act-16-apr-2015-1302
- Do not assume a single "latest SANS 10400" date applies to all parts. The applicable edition and referenced standards are part-specific. Verify against SABS catalogue and the municipality before a design gate is closed.
- National Building Regulations A14 expressly requires the constructed work not to compromise the design intent of a compliant design solution; this is a reminder that as-built execution and construction QA are part of code compliance, not a separate visual handover exercise. Use the current official regulation text and qualified local professionals for interpretation: https://www.acts.co.za/national-building-regulations-and-standards-act-/r2378_a14__construction
- South African energy-efficiency requirements include SANS 10400-XA. Verify the current published edition and local submission requirements; do not use a secondary explainer as the final legal source.

### Agrément South Africa — alternative and non-standardized systems

Agrément South Africa's published notice says its performance criteria for building systems, walling systems and building products were under review across structural strength/stability/serviceability, fire, water penetration/damp-proofing, acoustics, thermal/energy/condensation, durability and technical drawings, with a target completion date of **31 October 2026**. As checked on **7 November 2026**, the public notification page still states the target date but does not itself provide confirmation that revised criteria have been issued. Therefore:
- criteria revision status = **Unknown / verify directly with Agrément South Africa**;
- do not assume the deadline means the revisions are published or effective;
- do not infer that a new construction system is approved because it resembles a previously certified product;
- retrieve the actual active certificate and conditions for the exact system, manufacturing site, design, application and use;
- confirm whether the system falls within a deemed-to-satisfy pathway or requires an Agrément assessment, and record any limitations and validity reviews.

Agrément's process states that it evaluates the fitness for purpose of non-standardized construction products/materials/systems using performance-based criteria and monitors the certificate holder's quality system after approval. A certificate is scoped evidence, not an unrestricted approval for every configuration or site: https://agrement.co.za/certification-process/

Primary status pages:
- performance-criteria review notice: https://agrement.co.za/important-notification-review-of-agrement-south-africa-performance-criteria/
- current criteria index: https://agrement.co.za/asa-performance-criteria/
- certificate process: https://agrement.co.za/certification-process/

### International productivity guidance — PAS 4010:2026

BSI's project page identifies PAS 4010:2026 as a specification for improving productivity in infrastructure development and delivery and lists a publication start date of 5 November 2026. The ICE's public launch event is scheduled for **9 November 2026**. As of 7 November, it is therefore reasonable to align our research crosswalk with its stated lifecycle-productivity themes, but we should not claim formal conformity, certification or completed adoption. Retrieve the published specification before a clause-by-clause crosswalk; do not rely on the consultation draft.

Its stated scope includes productivity management for new and adapted infrastructure and the lifecycle of maintained, repurposed and retrofitted assets. It is UK infrastructure guidance, not South African building law:
- BSI project: https://standardsdevelopment.bsigroup.com/projects/2024-03460
- BSI product page: https://www.bsigroup.com/en-GB/insights-and-media/insights/brochures/pas-4010-improving-productivity-in-infrastructure-development-and-delivery-specification/
- ICE launch: https://www.ice.org.uk/events/upcoming-events/launching-pas-4010-the-new-standard-for-infrastructure-productivity-register-interest

### Environmental performance and information management

- CEN describes EN 15978:2026 as harmonizing whole-life environmental assessment for new, existing and refurbished buildings, including circularity information, system boundaries and use of EPD information. It is a useful cross-method reference, not automatically the South African regulatory pathway: https://www.cencenelec.eu/news-events/news/2026/en-in-the-spotlight/2026-04-17-en-15978-2026/
- ISO 19650-1 and -2 second editions are surfaced in ISO's catalogue as DIS (Draft International Standard) records. Do not cite the drafts as published replacements for the current editions. The general principle remains useful: information management is broader than 3D geometry and should be driven by the decision/use and asset lifecycle, but current normative editions must be checked at procurement/contract date: https://www.iso.org/obp/ui?_escaped_fragment_=iso%3Astd%3Aiso%3A19650%3A-1%3Adis%3Aed-2%3Av1%3Aen

### Required machine-readable gate fields

Every method/system evidence record should include:

```text
jurisdiction
authority
building_use_class
rule_or_standard_id
edition_or_revision
publication_status
effective_from / effective_to
deemed_to_satisfy_or_alternative_solution
certificate_id + exact scope + conditions + expiry/validity
design_professional + responsibility
tested_configuration + manufacturing_site
site_conditions + climate/exposure class
inspection_and_commissioning_records
source_url + retrieval_date + digest
status = current | stale | draft | unknown | superseded | revoked
```

If any legally or technically required gate is unresolved, the method remains Unknown/Blocked for that use. A strong synthetic result, a foreign certificate or a vendor declaration cannot override this.
