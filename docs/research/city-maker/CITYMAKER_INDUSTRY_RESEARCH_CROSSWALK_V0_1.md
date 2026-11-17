# CityMaker Industry Research Crosswalk v0.1

**Programme:** Luminous Habitats / CityMaker  
**Parent:** [CITY-MAKER-001 #549](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/549)  
**Crosswalk issue:** [CITYMAKER-INDUSTRY-CROSSWALK-001 #600](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/600)  
**Research cut-off:** 8 November 2026  
**Status:** desk-research and benchmark proposal; no market superiority or physical performance claim

## Executive finding

The AEC industry already has excellent specialist tools. Autodesk is connecting early-stage Forma workflows with detailed Revit authoring; Rhino/Grasshopper is a mature parametric design environment; Bentley iTwin integrates engineering data, reality data and operational sensor streams; EnergyPlus/OpenStudio and Ladybug/Honeybee provide building-performance analysis; OpenSees provides specialist structural/geotechnical simulation; IFC and IDS support open information exchange; NIST provides reproducible robot manipulation and assembly test methods.

Luminous should not attempt to replace these systems with one giant CAD/BIM/GIS/physics/robotics package.

The differentiating opportunity is a **verified design-to-production-to-operations loop** that:
1. translates resident/civic intent into explicit requirements and design alternatives;
2. retains geometry, source, units, coordinate-frame, scope and revision identity;
3. invokes the best qualified specialist analysis for each physical question;
4. preserves solver validity limits, uncertainty, convergence and unknowns;
5. checks constructability, process capability, robot/tool/calibration and logistics;
6. connects procurement, material provenance, fabrication, installation and inspection;
7. compares as-built and operational observations against the exact design and model revisions;
8. measures whole-life results, distributional impacts and maintenance.

This is an orchestration, evidence and decision-quality opportunity—not a claim that the four Luminous systems already deliver a production-ready building design platform.

## 1. Competitive baseline

### 1.1 AEC authoring and early design

| Industry system | Demonstrated strength / intended role | Boundary and opportunity for Luminous |
|---|---|---|
| **Autodesk Forma + Revit** | Autodesk's June 2026 direction positions Forma for connected, cloud-based, outcome-aware AEC workflows and Revit as central to detailed model-based BIM. September 2026 announcements describe additional AI-assisted option exploration and connected workflows across planning, design and construction. | Treat as a major commercial baseline. Do not claim it lacks generative design or lifecycle integration. Benchmark data handoff, exact revision binding, solver evidence portability, open export, auditability and as-built/operations continuity. |
| **Rhino + Grasshopper** | Mature visual parametric modelling. Rhino documents native geometry analysis and an ecosystem of structural and environmental add-ons, including Karamba3D/SOFiSTiK and Ladybug Tools. This is a strong baseline for shape exploration and designer-authored rule systems. | Symthaea should not simply reproduce Grasshopper. Differentiate through intent/requirements compilation, explicit constraint conflicts, multi-stakeholder Pareto search, evidence-aware abstention, value-of-information and production/lifecycle coupling. |
| **Revit / Archicad / other BIM authoring tools** | Detailed information-rich model authoring and coordination workflows, with large established professional ecosystems. | Consume authoring output through open/versioned exchange. Do not require users to abandon tools they already know. Measure geometry/property fidelity, identity preservation and round-trip losses. |
| **OpenBIM / IFC** | Vendor-neutral machine-readable built-asset information exchange. buildingSMART lists IFC 4.3.2.0 (IFC 4.3 ADD2) as its latest official release, published as ISO 16739-1:2024. IFC 4.3 adds infrastructure-domain coverage. | Adopt IFC as an exchange boundary where suitable. Do not create a competing proprietary universal asset schema. Map unsupported discipline/authority data explicitly rather than pretending IFC covers every engineering semantic. |
| **buildingSMART IDS 1.0** | Computer-interpretable exchange requirements for IFC object types, classifications, properties, materials and relationships; official since June 2024. | IDS is limited to alphanumeric information and does not cover geometry. Pair it with separately qualified geometry, spatial-frame and topology checks. An IDS pass is not a geometric, structural, fire or statutory approval. |

**Sources:** Autodesk's 30 June 2026 Forma/Revit direction (https://adsknews.autodesk.com/en/views/forma-revit-evolution/); Autodesk's 15 September 2026 Forma/AI workflow announcement (https://adsknews.autodesk.com/en/news/autodesk-forma-ai-aec-connected-workflows-2026/); Rhino Grasshopper (https://www.rhino3d.com/en/for/grasshopper/); buildingSMART IFC (https://www.buildingsmart.org/standards/bsi-standards/industry-foundation-classes/); buildingSMART IDS (https://www.buildingsmart.org/Standards/Bsi-Standards/Information-Delivery-Specification-Ids/).

### 1.2 Engineering and environmental analysis

| Industry system | Demonstrated strength / intended role | Boundary and opportunity for Luminous |
|---|---|---|
| **EnergyPlus** | Open-source building energy and water simulation engine developed by the U.S. Department of Energy. Used for whole-building energy analysis. | Do not recreate it in Symtropy. Integrate through a version-pinned adapter with weather file, construction, schedules, internal loads, HVAC/system assumptions, convergence and calibration/validation records. |
| **OpenStudio** | NLR describes it as an open-source collection of tools and SDK around EnergyPlus for creating, simulating and analyzing building energy models and strategies. | Use as one possible orchestration/model authoring route. Test model translation, assumptions and output reproducibility; do not treat a model run as measured energy performance. |
| **Ladybug Tools / Honeybee** | Rhino/Grasshopper-connected tools that use Radiance for daylight and EnergyPlus/OpenStudio for energy/thermal modelling; documents solar/radiation, glare, HVAC, thermal comfort, airflow-network and condensation studies. | Treat as a strong parametric environmental design baseline. Luminous's potential advantage is integrated requirements/evidence/authority management and whole-life coupling, not replacing Radiance or EnergyPlus. |
| **OpenSees** | Framework for simulating dynamic response of structural and geotechnical systems; documents use across natural hazards including fire, wind, earthquake and tsunami/wave actions. | Use specialist validated solver adapters for bounded problem classes. Symtropy's generic rigid-body/game physics must never be represented as structural engineering. |
| **Commercial structural packages** | ETABS, SAP2000, Robot Structural Analysis, STAAD and other packages are entrenched in professional structural engineering. Their use and acceptance depend on design codes, engineer competence, model assumptions and verification. | Select one or more domain tools per problem class after qualified practitioner input. Compare against analytical references and independent solvers. No one software package is a universal physical oracle. |
| **Daylight / lighting / acoustics / CFD** | Specialist tools exist for each physical field and often have different geometric, boundary and material requirements. | Do not compress daylight, glare, acoustics, air quality, thermal comfort and ventilation into one comfort score. Maintain separate field-specific evidence, units, assumptions and claim envelopes. |

**Sources:** OpenStudio / NLR (https://www.nlr.gov/buildings/openstudio); Honeybee (https://www.ladybug.tools/honeybee.html); OpenSees documentation (https://opensees.github.io/OpenSeesDocumentation/); EnergyPlus (https://energyplus.net/).

### 1.3 Digital twins and lifecycle operations

| Industry system | Demonstrated strength / intended role | Boundary and opportunity for Luminous |
|---|---|---|
| **Bentley iTwin Platform** | API platform for digital-twin applications that combine engineering data from diverse tools with reality data and sensor/IoT information; supports lifecycle visualization and analysis. | This is a serious commercial benchmark. Luminous should not claim "we have a digital twin" as differentiation. Benchmark exact model-to-reality lineage, change detection, stale-source behavior, provenance, offline operation, open export, authority boundaries and operational decision traceability. |
| **Common Data Environments / ISO 19650 processes** | Information management across design, delivery, handover and operations; requirements and roles, not one specific software product. | Build evidence and receipts that fit professional information-management processes. Keep roles, approvals, information requirements, status and handover traceable. |
| **Asset management / FM platforms** | Work orders, asset registers, maintenance schedules, sensor feeds, building operations and lifecycle services. | Connect rather than replace. Test whether installed assets, component passports, maintenance events, commissioning and replacement history remain linked to the as-built object and accepted design revision. |

**Sources:** Bentley iTwin (https://developer.bentley.com/itwinplatform/); ISO 19650 series overview (https://www.iso19650.org/); ISO/DIS 19650-1:2026 (draft record: https://www.iso.org/obp/ui?_escaped_fragment_=iso%3Astd%3Aiso%3A19650%3A-1%3Adis%3Aed-2%3Av1%3Aen).

**Standards status warning:** the cited ISO/DIS 19650-1:2026 is a draft (DIS), not a published replacement. Current project requirements must pin the formally published edition applicable at the time and jurisdiction. Do not implement draft terminology as if already mandatory.

### 1.4 Construction robotics and additive manufacturing

| Baseline | Demonstrated strength / scope | Luminous opportunity and limitation |
|---|---|---|
| **NIST Assembly Task Boards (ATB)** | NIST develops task-based tests for peg insertion, gear meshing, electrical connectors, threading/fastening, cables and deformable assemblies. Multiple repetitions and statistical comparisons are part of credible performance evaluation. | Adapt the task-board philosophy to construction-specific operations: panel alignment, timber joint fit-up, fastening, cable/duct handling, window/door installation and inspection. Do not claim a manufacturing task-board pass proves jobsite performance. |
| **NIST performance assessment framework** | Decomposes robot performance into perception, mobility, dexterity and safety, then composes these in task-based performance models. | Use as a stronger baseline than one scalar autonomy percentage. Measure the complete workcell, operator intervention, fixtures, sensing, calibration, load, environment, safety and accepted output. |
| **Dusty Robotics FieldPrinter 2** | Vendor documentation describes a BIM-to-field layout robot that prints model-derived full-scale lines/points onto slabs, with stated ±1/16 in (±1.6 mm) print accuracy under the vendor's specified setup and surveyed-control requirements. | Benchmark layout error against calibrated survey control and total-station measurements. Vendor specs are not independent comparative test results. Preserve control-point, stationing, model revision and environmental conditions. |
| **Additive construction** | ISO/ASTM 52939:2023 specifies quality-assurance requirements for additive construction processes and structural/infrastructure elements, across materials/process categories. | Use printing only for components or applications whose material, reinforcement, interlayer bond, durability, inspection and acceptance properties are qualified. The standard itself does not provide design approval, material property characterization/testing or printing-facility EHS coverage. |
| **Industrial robot safety** | ISO 10218-1:2025 addresses industrial robot safety; ISO 10218-2:2025 addresses integration and safety of industrial robot applications/cells, including commissioning, operation and maintenance. Part 2's scope excludes contexts where the public or non-working adults have access. | Apply standards within their stated scope. A guarded factory cell and a mobile platform in a public residential street are not the same safety case. Site robotics need additional applicable mobile-robot, machinery, occupational-safety and public-interface assessment. |

**Sources:** NIST grasping/manipulation/contact-safety project (https://www.nist.gov/programs-projects/grasping-manipulation-and-contact-safety-performance); NIST assembly test boards (https://www.nist.gov/el/intelligent-systems-division-73500/robotic-grasping-and-manipulation-assembly/assembly); NIST performance assessment framework (https://www.nist.gov/programs-projects/performance-assessment-framework-robotic-systems); Dusty FieldPrinter specifications (https://support.dustyrobotics.com/hc/en-us/articles/52682349645851-FieldPrinter-Specs); ISO/ASTM 52939:2023 (https://www.iso.org/standard/81177.html); ISO 10218-1:2025 (https://www.iso.org/standard/73933.html); ISO 10218-2:2025 (https://www.iso.org/standard/73934.html).

**Safety-critical interpretation:** ISO 10218-2:2025 is for industrial robot applications and cells in industrial settings. Its scope explicitly excludes service-robot/public-access situations and mobile-platform mobility from its direct coverage. It is not blanket approval for robots operating around residents, visitors, children or the public.

## 1.5 Current SOTA delta — 8 November 2026

This section supersedes any older statement that Forma's early-design workflow lacks integrated indoor-daylight, parking or furniture capabilities. The industry baseline must be updated as products change; a stale comparator is a benchmark defect.

### Autodesk Forma Building Design — 4 November 2026 release

Autodesk's 4 November 2026 release notes describe three important capabilities for Forma Building Design:
- **Indoor Daylight (public beta):** occupied-hours daylight analysis over a typical meteorological year, with a linked workflow to Revit.
- **Parking automation:** layout generation and editing in the schematic design workflow.
- **Furniture workflow:** 2D furniture layout capability with model/visualization effects in the 3D environment.
- **AutoCAD exchange:** DXF import and DWG export are included in the update.

This makes the early-design baseline more capable than a generic "massing + sun hours" comparator. The Forma workflow must be represented in the next matched experiment, including these exact capabilities, release date, entitlement/availability, supported file paths and human interventions. Indoor Daylight must be compared on matched geometry, schedules, weather evidence class and occupied-hour settings; a public beta feature is recorded as beta, not assumed production-qualified.

**It removes several weak Luminous differentiation claims:** early daylight screening, parking layout automation, furniture layout and a basic Revit-linked schematic workflow are not unique Luminous value propositions.

The stronger Luminous hypothesis is narrower and more difficult to prove: preserve exact requirement, source, model, revision and authority semantics across the entire chain, then demonstrate correct selective invalidation and independently verifiable handoff into fabrication, installation, commissioning and operations.

Source: Autodesk Forma team, 4 November 2026, https://blogs.autodesk.com/forma/2026/11/04/whats-new-in-forma-building-design-indoor-daylight-parking-furniture-and-autocad-workflows/

### Research synthesis — where generative design still under-delivers

A 2025 systematic review of 153 performance-driven generative building-design studies reports a literature dominated by building-scale design problems and conventional energy/daylight/thermal objectives. It highlights fragmented metrics, uneven validation and a need for reproducibility and cross-case assessment.

Luminous should not interpret this as proof that existing commercial tools are weak; the review spans research workflows, not a controlled product comparison. It does suggest a valuable research gap for our benchmark: move beyond single-building design objectives to **matched delivered-service outcomes across household, building, production and neighbourhood scales**, and require independent evidence for every physical metric.

Source: Buildings (2025), Performance-Driven Generative Design in Buildings: A Systematic Review, https://doi.org/10.3390/buildings15244556

### Risk-proportionate model credibility

NIST AMS 100-85 (5 October 2026) recommends scaling verification, validation and uncertainty-quantification effort with the influence of a computational model and the consequence of the decision it informs.

For CityMaker, this means a rendering or early daylight option study can have a lightweight research-grade credibility package, while structural, fire, egress, indoor-air, health, statutory or construction decisions require stronger domain-specific reference data, validation and qualified review. The same evidence tier must not be reused for every domain merely because the files share a project identifier.

Source: NIST AMS 100-85, https://doi.org/10.6028/NIST.AMS.100-85

### Urban digital-twin lifecycle evidence

A 2026 review of 114 urban-digital-twin publications is discoverable on the publisher site and proposes four interacting layers: reference, entity/semantic, state and feedback synchronization. **Temporal caveat:** the publisher currently assigns the article an issue date of **15 November 2026**, later than this crosswalk's 8 November 2026 research cut-off. Treat it as forthcoming/watch-list material, not as an already-published source in the current evidence baseline. Do not cite it as established evidence until the stated publication date or a separately verified online-first publication record is confirmed.

The four-layer framing is directionally consistent with the work already assigned to CityMaker #623–#625, but that architectural recommendation is not dependent on treating the forthcoming article as published: shared scope is necessary but insufficient; integrated city states also need owner-native identity, valid/record/observation times, freshness, correction, revocation, intervention and post-intervention outcome evidence. Sol-Atlas should coordinate read-only projections and state gaps, not invent a competing sensor platform or event ledger.

Watch item (issue date 15 November 2026): Sustainable Cities and Society, https://www.sciencedirect.com/science/article/abs/pii/S2210670726009455

### Building data interoperability: use the standard, measure beyond it

NIST's Building Digitization and Semantic Interoperability programme was updated 2 November 2026. It identifies the continuing difficulty of integrating building data from diverse lifecycle systems and points to semantic models and machine-readable exchange as necessary for analytics and building controls.

The practical implication is to retain IFC 4.3.2.0 / ISO 16739-1:2024 and IDS 1.0 as stable exchange baselines, while treating geometry identity/topology, external source provenance, temporal currentness, solver evidence and professional authority as separate proof obligations. IDS checks are not geometry or physical-performance checks.

Sources:
- NIST, https://www.nist.gov/programs-projects/building-digitization-and-semantic-interoperability
- buildingSMART IFC release register, https://technical.buildingsmart.org/standards/ifc/ifc-schema-specifications/
- buildingSMART IDS scope, https://www.buildingsmart.org/standards/bsi-standards/information-delivery-specification-ids/

### Required comparator update

The next preregistered Forma-vs-Luminous lane must test at least:
1. identical intent, task programme and site/geometry fixture;
2. current Forma Building Design release and exact beta/feature entitlement;
3. Indoor Daylight configuration, typical-year input and occupied-hours assumptions;
4. parking and furniture outputs, where applicable;
5. DXF/DWG/IFC/Revit handoff and exact identity/geometry/property round-trip;
6. time-to-first-feasible and full human intervention time;
7. evidence lineage, missing/unknown inputs, revision invalidation and stale-result detection;
8. specialist-solver and measured/physical validation only where the task calls for it.

Use an integrated human-led best-of-breed workflow as a serious comparator. If a workflow cannot be made equivalent on geometry, information, solver budget or human time, label that comparison as not comparable; do not manufacture a ranking.

## 2. What the newest research says about robotics adoption

A June 2026 systematic review examined 375 construction-robotics studies and identifies persistent real-world deployment barriers despite potential for productivity and occupational-health gains. A 2026 PRISMA scoping review of 52 AEC human-robot collaboration studies notes that much literature remains task-centric and short-term, with convenience participants rather than end users.

This implies the most defensible Luminous strategy is not "more autonomy" as an abstract goal. It is **more qualified work delivered across variable conditions with fewer failures, rework and hazardous exposures**.

The first physical demonstrator should therefore be a guarded off-site timber/panel fabrication and metrology cell, not an autonomous residential construction fleet.

**Sources:** Xu et al. (June 2026), *Construction robotics: a systematic review of robot types, applications and human robot collaborations*, https://doi.org/10.1016/j.rineng.2026.110406; Vermeulen et al. (2026), *Reconceptualising Human-Robot Collaboration in AEC: A PRISMA Scoping Review*, https://doi.org/10.22260/ISARC2026/0133.

## 3. Standards and open exchange register

| Standard / guidance | Status at research cut-off | Use in Luminous | Important limit |
|---|---|---|---|
| IFC 4.3.2.0 / ISO 16739-1:2024 | Published; latest official IFC release found | Open asset/infrastructure exchange | Not a complete ontology for every discipline, approval or engineering state |
| IDS 1.0 | Published buildingSMART standard, approved 1 June 2024 | Alphanumeric property/object exchange requirements | Does not check geometry |
| ISO 19650-1:2018 and current published series parts | Published editions remain applicable pending formal revisions | Lifecycle information management process | Pin exact project/jurisdiction requirements; not a software product |
| ISO/DIS 19650-1:2026 | Draft under development | Track change and future migration needs | Not yet a published standard |
| ISO 7817-1:2024 | Published | Level of information need | Does not prove physical validity |
| ISO/ASTM 52939:2023 | Published | Additive construction quality-assurance process | Does not cover design approval/material-property tests/printing-facility EHS |
| ISO 10218-1:2025 / -2:2025 | Published | Industrial robot and application/cell safety within scope | Does not automatically cover mobile robots in public occupied environments |
| South African DHS/CSIR Red Book (2019, v1.1) | Published national neighbourhood planning and design guide | South African settlement baseline | Guidance, not a site-specific permit or substitute for current local regulation |

## 4. Luminous responsibility and authority map

| Layer | Owns | Must not claim |
|---|---|---|
| **Symthaea** | Explicit requirements, intent-to-design translation, alternatives, multi-objective reasoning, uncertainty, conflicts, value-of-information and evidence-bounded improvement proposals | Structural certification, building-code approval, land rights, community consent or solver capability |
| **Symtropy** | Deterministic geometry/spatial/resource/process semantics and orchestration of validated solver adapters within declared model envelopes | Generic physics as validated structural, fire, thermal, daylight, geotechnical or life-safety engineering |
| **Mycelix** | Consent, authorized decisions, material/supplier provenance, custody, inspection/approval records, correction/revocation and dispute evidence | A provenance receipt as permit, title, engineer sign-off or physical capability |
| **Sol-Atlas** | Geospatial context, site suitability evidence, service access, urban form, logistics, Pareto comparison, public-interest impacts and source coverage | Map/rendering outputs as engineering approval, safety certification or source truth |
| **Specialist solver** | One bounded physical model class, with inputs, assumptions, validation evidence, uncertainty and version | Universal truth outside its calibration/validation envelope |
| **Qualified professional / authority** | Discipline-specific review, professional acceptance and statutory approval within legal scope | A software-generated status as a substitute for professional judgment |
| **Robotics / construction executor** | Physically authorized and safety-engineered task execution after exact prerequisites and commissioning | Simulation pass, AI plan or task proposal as actuator authority |

## 5. Three matched reference objects

### A. Highveld Keep — one dwelling

A South African residential reference model, not an imported European castle copy:
- central courtyard and shaded covered circulation;
- great hall/family space, private work/rest areas and service/arrival threshold;
- step-free routes and usable access;
- Highveld solar orientation, summer shade, thermal mass/insulation and ventilation studies;
- water harvesting/storage/reuse with potable/non-potable separation;
- solar PV, battery/load management and manual fallback;
- qualified fire, egress, responder-access and security review;
- local stone/timber/earth/masonry alternatives with material provenance;
- repair access, replaceable components and realistic fabrication packages.

Compare at least three architecture/material/production alternatives under common occupancy, site, room/task, budget and performance assumptions.

### B. Living Hamlet — 4–12 synthetic dwellings

A small cluster with shared courtyard, street frontage, public-space link, local service node, energy/water systems, emergency access and landscape. Compare six morphology baselines: controlled-entry/gated, courtyard cluster, active-frontage block, polycentric neighbourhood, retrofit/repair-first and maintain-existing/no-build.

Keep access, safety, ecology, affordability, construction phase, service capacity, public access and spillovers as separate outcomes.

### C. Guarded panel/timber workcell

A low-consequence and instrumentable physical manufacturing demonstrator:
- surveyed metrology and fixture validation;
- standardized and variable/reclaimed material cases;
- scan-to-cut and joint/tolerance stack-up;
- human-only, assisted and guarded robotic routes;
- supervised handling, fastening, inspection and rework;
- calibrated as-built measurement and batch/material provenance.

Do not progress to autonomous heavy lifting, cutting or public-site work until the task-specific risk assessment, guarding, safety-rated controls, interlocks, commissioning and operator training are established.

## 6. Competitive benchmark protocol

For every comparison:
1. Freeze functional requirements and study boundary.
2. Pin all candidate geometry, parameters, data vintages, solver versions and configuration files.
3. Use identical material, weather, occupancy, load, service and budget assumptions where comparison requires them.
4. Separate measured, simulated, vendor-claimed, synthetic and unknown evidence.
5. Use independent reference calculations or qualified third-party checks.
6. Blind expert review of aesthetics and usability where possible.
7. Run physical tests with calibrated metrology and repeat trials.
8. Publish failed cases, exclusions, confidence intervals, p50/p90/p95 and Pareto trade-offs.
9. Preserve baseline and candidate artifacts so results can be reproduced.
10. Do not say "better than SOTA" based on a feature list, one demo or one synthetic pass.

### Matched baselines

- Architect + established BIM + specialist consultants.
- Rhino/Grasshopper + specialist plugin workflow.
- Commercial connected AEC workflow such as Forma/Revit.
- Open source parametric + OpenBIM + EnergyPlus/OpenStudio + OpenSees path where applicable.
- Luminous orchestration with the same specialist solvers and tools.
- Conventional manual, assisted and robotic production methods for the same accepted physical output.

### Metrics

**Design / user value**
- Requirement coverage and unresolved mandatory conflicts.
- Blind aesthetics, character, usability and place-identity reviews.
- Step-free task completion, accessibility and wayfinding.
- Daylight, glare, thermal, acoustic, privacy and ventilation outcomes reported separately.

**Interoperability**
- IFC/IDS round-trip preservation.
- Geometry and spatial-frame equality tested separately.
- Source, unit, revision, scope, material and object identity preservation.
- Missing-field and unsupported-domain detection.

**Engineering**
- Error against analytical/validated references.
- Solver convergence and model-domain validity.
- Uncertainty coverage, calibration and held-out error.
- False acceptance on negative controls.

**Construction**
- First-pass yield, tolerance distributions and accepted-output rate.
- Rework, defect severity, installation time and site disruption.
- Material loss, embodied impacts, energy and water per accepted functional unit.
- Cost/schedule per accepted asset, not nominal model quantity.

**Robotics**
- NIST-style task success, perception/mobility/dexterity/safety decomposition.
- Placement/fit-up accuracy, cycle-time distribution, intervention burden and recovery.
- Energy, tool wear, maintenance, calibration, downtime and worker exposure.
- Transfer across material variability, geometry, lighting, fixtures, workcell and site conditions.

**Operations**
- As-built mismatch, defect closure, repair time and asset uptime.
- Maintenance and replacement cost.
- Actual measured operational energy/water and carbon where available.
- Adaptation, reuse, disassembly and verified recovery.

### P0 goals

- No silent loss of geometry, units, frame, scope, source, revision or authority through exchange.
- No PASS for missing units, unconstrained supports, incomplete load cases, unsupported physics or non-convergence.
- 100% of high-impact requirements bind to a source and reviewer role.
- 100% of missing/stale/unsupported inputs remain Unknown or limited.
- Zero false acceptance of seeded hard-gate violations in frozen synthetic suites.
- Deterministic replay for identical pinned inputs wherever the underlying tool supports it.

### P1 superiority threshold

Luminous may claim comparative improvement only after a preregistered matched benchmark demonstrates a statistically and practically meaningful gain in an outcome such as:
- design-to-fabrication completeness;
- accepted output per labour-hour or total resource;
- first-pass yield and defect rate;
- installed and whole-life cost;
- embodied/operational carbon and water;
- maintenance or outage duration;
- adaptation/repair cost.

No gain counts if accessibility, safety, durability, fire/egress, worker health, privacy, affordability, resident experience or ecology materially degrades. Report the complete Pareto frontier and confidence intervals.

## 7. Where Luminous can genuinely differentiate

The most credible route to differentiation is a small number of demonstrably stronger capabilities:

1. **Authority-preserving engineering integration:** exact evidence and claim scope survive cross-tool handoff.
2. **Multi-fidelity design search:** cheap exploration followed by qualified specialist checks and calibrated physical experiments.
3. **Design-for-production and repair:** fabrication, assembly, inspection, maintenance and deconstruction are first-class design variables.
4. **Whole-neighbourhood optimization:** home-to-service accessibility, public realm, safety, utilities and productive capacity are evaluated together.
5. **Construction evidence loop:** design intent, fabrication, installation and as-built inspection are linked without treating the model as truth.
6. **South African contextual design grammar:** climate, ecology, local materials, local crafts, resident-stated practices and affordability are not afterthoughts.
7. **Open and locally operable systems:** exportable IFC/data, privacy, offline functionality, manual fallbacks and local maintenance capacity.

This is a more defensible competitive thesis than claiming a new AI architect, one universal physics engine or a fully autonomous construction city.

## 8. South African context and near-term regulatory watch

The DHS/CSIR Red Book is a relevant national starting point: it covers neighbourhood layout, open space, housing/social facilities, transport, water, sanitation, stormwater, energy, safe communities and universal design (2019, v1.1). Use it as a local planning baseline; do not create a competing generic South African manual.

The South African Government has extended the public-comment period for the 2026 National Building Regulations and Building Standards Amendment Bill to **12 November 2026 at 12:00**. The bill remains a draft proposal during this process; it is not enacted law. Luminous may consider a transparent, professionally reviewed submission on performance-based approval, innovation evidence, digital records and accountability, but no submission should be sent without explicit authorization and legal/professional review.

Sources:
- DHS Red Book: https://www.dhs.gov.za/node/154
- CSIR Red Book: https://www.csir.co.za/neighbourhood-planning-and-design-guide-red-book
- Extension notice: https://www.gov.za/documents/notices/national-building-regulations-and-building-standards-act-extension-submission

## 9. Immediate next actions

1. Audit exact implementation and maturity for the listed Luminous owner issues and PRs; issue text alone is not implementation proof.
2. Add narrow adapters only after exact source API, data schema, units, revision and evidence fields are confirmed.
3. Freeze Highveld Keep and Living Hamlet synthetic input manifests.
4. Select one specialist solver per physical domain and construct independent analytical reference fixtures.
5. Reuse NIST ATB methods when designing the first construction-robot test board; extend with construction-specific components and field conditions.
6. Keep the safety benchmark (#591/#598) independent from this broader engineering crosswalk.
7. Select the first physical workcell based on instrumentability, low consequence, maintainability and local supply—not novelty.
8. Raise claims from E0 to E1/E2/E3/E4/E5 only on exact, scoped evidence.

## 10. References and access date

All sources accessed 7 November 2026.

- Autodesk News, *Forma and Revit: what's changing, what's staying the same, and what comes next* (30 June 2026): https://adsknews.autodesk.com/en/views/forma-revit-evolution/
- Autodesk News, *How Autodesk Forma and AI are advancing a more connected future for AEC* (15 September 2026): https://adsknews.autodesk.com/en/news/autodesk-forma-ai-aec-connected-workflows-2026/
- Rhino Grasshopper: https://www.rhino3d.com/en/for/grasshopper/
- Bentley iTwin Platform: https://developer.bentley.com/itwinplatform/
- buildingSMART IFC (latest official release IFC 4.3.2.0): https://www.buildingsmart.org/standards/bsi-standards/industry-foundation-classes/
- buildingSMART IDS 1.0: https://www.buildingsmart.org/Standards/Bsi-Standards/Information-Delivery-Specification-Ids/
- ISO/DIS 19650-1:2026: https://www.iso.org/obp/ui?_escaped_fragment_=iso%3Astd%3Aiso%3A19650%3A-1%3Adis%3Aed-2%3Av1%3Aen
- OpenStudio / National Laboratory of the Rockies: https://www.nlr.gov/buildings/openstudio
- Ladybug Tools Honeybee: https://www.ladybug.tools/honeybee.html
- OpenSees: https://opensees.github.io/OpenSeesDocumentation/
- NIST robotic grasping/manipulation/contact safety performance: https://www.nist.gov/programs-projects/grasping-manipulation-and-contact-safety-performance-robotic-systems
- NIST Assembly Task Boards: https://www.nist.gov/el/intelligent-systems-division-73500/robotic-grasping-and-manipulation-assembly/assembly
- Dusty FieldPrinter specifications: https://support.dustyrobotics.com/hc/en-us/articles/52682349645851-FieldPrinter-Specs
- ISO/ASTM 52939:2023: https://www.iso.org/standard/81177.html
- ISO 10218-1:2025: https://www.iso.org/standard/73933.html
- ISO 10218-2:2025: https://www.iso.org/standard/73934.html
- Xu et al. (2026), construction robotics systematic review (375 studies): https://doi.org/10.1016/j.rineng.2026.110406
- Vermeulen et al. (2026), AEC human-robot collaboration scoping review (52 studies): https://doi.org/10.22260/ISARC2026/0133

---
**Claim ceiling:** this is a competitive research crosswalk and a proposal for matched benchmarks. It makes no claim that Luminous currently outperforms Autodesk, Bentley, specialist solvers, construction-robot vendors or existing professional workflows.