# Highveld Keep — Computational Design Basis v0.1

**Programme:** Luminous Habitats / CityMaker  
**Execution issue:** [HIGHVELD-KEEP-001 #607](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/607)  
**Parent:** [CITY-MAKER-001 #549](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/549)  
**Research cut-off:** 7 November 2026  
**Status:** E0 concept/design brief; synthetic scenario only  
**Geography:** Johannesburg/Highveld regional reference; no parcel selected

## 1. Design thesis

Highveld Keep is the first South African computational reference dwelling for Luminous Habitats. Its architectural character draws on the experience of a *living citadel*: a generous communal heart, crafted stone-and-timber expression, sheltered galleries, gardens, thresholds and a strong relationship with the land.

It is not a medieval replica, a militarized bunker, a luxury render exercise or a parcel-specific proposal. The design objective is to explore how architecture can be simultaneously:

- beautiful, locally meaningful and human-scaled;
- accessible and usable by a diverse household;
- climate-responsive and comfortable across warm days, cool nights and winter conditions;
- resilient to power, water and communications interruptions;
- proportionately secure without hostile public/private interfaces;
- structurally and fire/life-safety verifiable by qualified professionals;
- manufacturable, inspectable, repairable and adaptable;
- economically and ecologically assessable over its whole life.

The first release defines the design problem, alternatives, inputs, owner boundaries and benchmark. It does **not** define approved construction geometry or engineering results.

## 1A. Industry reality and the programme's position

Highveld Keep is not competing against an industry that only draws static models. By 7 November 2026:
- Autodesk Forma Building Design publicly documents schematic modelling, design automations for floor plans/facades/unit layouts, area metrics, integrated sun-hours/daylight/carbon analysis, and geolocated native Revit handoff.
- Its 4 November 2026 update adds Indoor Daylight in public beta, occupied-hours daylight analysis over a typical year, DXF import/DWG export and additional parking/furniture workflows.
- Forma Site Design already advertises site area, sun-hours, daylight potential, wind, microclimate and embodied-carbon analyses.
- TestFit documents generative zoning filters (height, FAR, building coverage, impervious coverage and dwelling units/acre) plus unit and parking-mix goals.
- Revit remains a detailed model-based BIM authoring baseline; Autodesk describes Forma as extending connected, outcome-based workflows earlier rather than replacing Revit overnight.
- IFC 4.3 and IDS 1.0 already establish important open exchange and information-requirement standards. IDS covers alphanumeric IFC information; it does not prove geometric correctness.

References: [Forma Building Design](https://www.autodesk.com/products/forma-building-design/overview), [Forma November 4, 2026 update](https://blogs.autodesk.com/forma/2026/11/04/whats-new-in-forma-building-design-indoor-daylight-parking-furniture-and-autocad-workflows/), [Forma environmental analysis](https://www.autodesk.com/products/forma-site-design/environmental-impact-analysis), [TestFit Generative Design](https://www.testfit.io/blog/testfit-5-2-better-data-brings-better-generative-design), [IFC specification register](https://technical.buildingsmart.org/standards/ifc/ifc-schema-specifications/), [IDS scope](https://www.buildingsmart.org/standards/bsi-standards/information-delivery-specification-ids/).

The proposed Luminous differentiation is therefore not "we generate more shapes." It is to test whether an open, evidence-carrying workflow can connect local requirements and source data, plural resident intent, verified geometry, domain solvers, product approvals, fabrication acceptance, as-built records and maintenance without losing authority or provenance. Current Highveld work is only E0/E1 and has not demonstrated that advantage yet.

## 1B. Research-led paradigm shifts

Recent reviews help identify where a genuinely useful step-change may exist:

- A 2025 systematic review of 153 performance-driven building-design studies found a strong concentration on façade/envelope and single-building massing problems, with energy/daylight/thermal-comfort objectives prominent. It identified fragmented metrics, limited cross-case or field validation and reproducibility risks. The field already has parametric evolutionary, surrogate/Bayesian and data-driven workflow families; using those names is not innovation by itself.
- A 2025 review of 161 generative-AI architectural-design papers found that image inputs/outputs dominate and most AI use is concentrated in early schematic phases; graph and multimodal representations are a developing direction. Subjective comparison remains common.
- A 2026 review of 114 AI-surrogate building-design studies describes limited exploration of uncertainty-aware optimization and interpretability, alongside challenges in integrating surrogate prediction and optimization.

The Highveld research bets should therefore be:

1. **Intent/requirement graph before image generation.** Compile task requirements, hard routes, privacy zones, preferences and unresolved evidence into a versioned, testable design contract. Images and renders are explanatory views, not the authoritative design.
2. **Evidence-carrying alternatives.** Every candidate carries exact sources, assumptions, geometry revision, solver scope, uncertainty and invalidation dependencies. A visually persuasive option with missing evidence stays E0.
3. **Multi-fidelity search with calibrated uncertainty.** Use cheap geometry/rule checks and surrogate estimates for exploration; spend expensive specialist solver calls where they have the highest expected value of information; re-run every promoted candidate with the exact qualified solver. Detect out-of-domain candidates.
4. **Robustness-first, not typical-year-only optimization.** Design for ordinary conditions and adverse trajectories: heatwave, cold, poor air quality, grid/water interruption, storm/hail, component failure and future climate. Keep scenario uncertainty visible and optimize lower-tail resilience as a separate decision dimension.
5. **Open standards plus independent geometry proof.** Use IFC/IDS contracts for information requirements, but separately validate geometry, route/access constraints, clearances, clashes, topology and as-built equivalence.
6. **Buildability from the first design generation.** Include material source/grade, connection logic, fabrication route, tooling capability, inspection and repair in the search problem. Do not wait for a later contractor to discover the design cannot be made.
7. **Plural resident agency.** Preserve different household preferences and contested interpretations rather than reducing them to a single inferred utility or beauty score.
8. **Operational learning without surveillance by default.** Use opt-in, purpose-limited, locally controlled measurement and maintenance data to calibrate models after occupation; retain manual fallback and resident control.

Sources:
- Huang et al. (2025), 153-study performance-driven building-design review: https://doi.org/10.3390/buildings15244556
- Jang et al. (2025), 161-paper generative-AI architectural-design review: https://doi.org/10.1016/j.autcon.2025.106174
- Zhang et al. (2026), 114-study AI-surrogate modelling review: https://doi.org/10.1016/j.enbuild.2026.117238

## 1C. Social, environmental and dynamic resilience parity

The 2026 literature reinforces that conventional optimization can be multi-objective in name but narrow in what it values. A systematic review of 123 LP/MILP construction-management optimization studies (1985–2024) found that 89% optimized exclusively economic outcomes, 11% included environmental metrics, and 0% included explicit social metrics; 53% were deterministic. These percentages describe that specific mathematical-programming literature, not the entire AEC industry. A 2026 review of building sustainability/resilience integration reports that most studies use sequential links between sustainability and resilience, while bidirectional coupling and dynamic time-dependent models remain rare.

Highveld should test a more complete model without collapsing it into one total score:
- **Household and social outcomes:** accessibility, indoor habitability, privacy, resident autonomy, affordability, safety, service access, ability to adapt, and distribution of benefits/burdens.
- **Workforce outcomes:** worker exposure, ergonomics, skill/training requirements, human intervention, safe aborts, task control and fair full-workflow labor accounting.
- **Environmental outcomes:** operational energy, embodied carbon, water demand/runoff, material provenance, repair/reuse, waste, biodiversity and transport impacts.
- **Economic outcomes:** capital cost, lifecycle cost, maintenance burden, replacement availability, schedule risk, local supply resilience and total accepted output per full-workflow labor-hour.
- **Resilience outcomes:** time-varying thermal/air/water/energy/access service, lower-tail habitability, outage duration, failure containment and recovery time.

Keep these measures as separate objective or acceptance dimensions, with hard rights/safety/accessibility gates non-compensatory. Do not convert resident dignity, worker safety or life-safety into weights that a lower cost or lower carbon total can outweigh. Keep conflicting household preferences and stakeholder disagreement visible.

The key experiment is **bidirectional, time-dependent coupling**. For example, a shading or envelope change affects daylight, heat, ventilation, energy, water, maintenance and panel fabrication; a product or material substitution affects thermal behavior, carbon, cost, repair availability and robot tooling. These are not one-way reports. A change should propagate to affected models, preserve unaffected evidence, and trigger re-analysis only where dependency edges require it.

The research goal is not to build every solver. It is to test whether Symtropy can orchestrate specialist co-simulation with explicit time step, coupling variables, convergence, stability, latency, uncertainty and causal dependency records—while preserving each specialist's authority.

Sources:
- Mohammed et al. (2026), 123-study construction optimization review: https://doi.org/10.1007/s43621-026-03009-y
- 2026 systematic review of environmental sustainability and resilience integration in building design/construction: https://doi.org/10.1016/j.buildenv.2026.114381

## 2. Paradigm shifts to test

### 2.1 From castle object to living habitat

Do not optimize a single dramatic silhouette. Optimize the relationships among arrival, shared life, rest, work, service, landscape, utility systems, maintenance and emergency operation. The building should feel protective because its spaces are legible, welcoming and resilient—not because it looks threatening.

### 2.2 From room lists to task and relationship models

A room name is not proof that a resident can perform a task. Represent activities, reach/access requirements, privacy, acoustics, daylight, service needs and adjacencies separately. Keep residents' different preferences visible; do not collapse them into one household preference vector.

### 2.3 From one preferred design to a Pareto family

Compare at least four genuinely different spatial organizations under identical functional requirements. Keep aesthetic, accessibility, energy, water, safety, carbon, cost, constructability and repairability separate. Hard constraints cannot be traded against a visual or cost score.

### 2.4 From architecture first, production later to design-for-lifecycle

Every expressive feature should have a plausible construction route, inspection point, repair approach and replacement strategy. Structural rhythm, daylight, orientation, wayfinding, drainage and craft can reinforce one another, but a multifunctional feature is not automatically superior; test the trade-offs.

### 2.5 From generic "smart home" to local autonomy and graceful degradation

Smart functions must not depend on cloud connectivity or continuous surveillance. Essential functions should have safe manual operation, privacy-preserving local control and clear fallback states. Automation assists occupants; it does not gain independent authority over physical security or life safety.

### 2.6 From imported style to attributable South African design grammar

Use site ecology, geology, climate, local craft, materials and resident-authored stories. Work with South African architects, artists, craftspeople and communities. Do not copy or assign symbolic meanings to indigenous, religious or cultural motifs without attribution, permission and context.

## 3. Synthetic reference household and program

The first design study assumes a **synthetic household of 4–6 residents**. This is a benchmark fixture, not a claim about the eventual occupants.

The initial program includes:
- four sleeping rooms, including one flexible guest/care room;
- a shared great hall / living and gathering space;
- kitchen, dining and food-storage workflows;
- library / quiet study space;
- work / making / repair space;
- an accessible arrival-level washroom and essential daily-task route;
- laundry, utility, plant, storage and waste/service access;
- vehicle and bicycle arrival;
- shaded outdoor living, courtyard/garden and step-free connections;
- inspectable maintenance access to roof, drainage, services, energy and water plant.

Actual occupancy, room count, care requirements, mobility, privacy, work patterns and cultural practices must be supplied by real residents at a later stage. Do not infer these from age, gender, ethnicity or other demographic proxies.

## 3A. Functional relationship graph — task-level, not geometric

The machine-readable scenario now includes 11 explicit relationships among the synthetic functional-programme nodes. This is a small requirements graph for Symthaea #7790 and the downstream spatial-design owner #1766 to consume; it is **not** a new room ontology, circulation solver or exact geometry representation.

Hard relationships in v0.1:
- Arrival to great hall, kitchen/dining, work/making and outdoor living must have step-free routes.
- Arrival to the accessible washroom must provide arrival-level access.
- Visitor routing from arrival to the great hall must not require passage through sleeping rooms.
- Maintenance/service access between plant and arrival must not require passage through sleeping rooms.

Negotiable preferences in v0.1:
- Kitchen/dining should be adjacent to outdoor living.
- Library/quiet and sleeping rooms should be acoustically separated from the great hall.
- Work/making should be adjacent to service/plant functions.

These links express the *kind of relationship to evaluate*, not a distance, dimension, shortest path, floor-plan layout or approved accessibility standard. A hard relationship is a requirement to be verified by an appropriate spatial/accessibility analysis; a preference may be negotiated with actual residents. Neither is proof that a design satisfies it.

The graph must preserve:
- stable relationship IDs;
- source and target function IDs that exist in the declared programme;
- requirement class (`hard` versus `preference`);
- explicit excluded intermediate functions where privacy/maintenance routes require them;
- reason and provenance;
- uncertainty or stakeholder disagreement when a real household replaces the synthetic brief.

Do not infer paths from adjacency alone, or infer acoustic separation from a room name. Geometry, distances, gradients, door clearances, travel paths, fire egress and acoustics require separate exact geometry and specialist review. The task graph is an input contract for search, not an engineering result.

## 4. Four architectural families

All alternatives must satisfy the same functional program and share the same synthetic site envelope, weather inputs, budget interval and evaluation assumptions.

### A — Courtyard Cloister

A central, accessible garden court with inhabited galleries and a shared great hall. Explore how the courtyard can provide orientation, daylight, shared outdoor living, passive shade and community without creating hidden or unmaintainable spaces. Vary openness, aspect ratio, courtyard depth, shading, planting and public/private thresholds.

### B — Winged Great Hall

A central communal hall with residential wings and a sheltered garden court. Explore a clear social heart, compact service routes and a legible transition between shared and private zones. Test whether the massing creates difficult circulation, inaccessible level changes or excessive envelope area.

### C — Compact Keep

A compact form, potentially two storeys, that tests envelope efficiency, service concentration and material use. Verticality is not presumed to be better: assess step-free access, accessible daily tasks, evacuation, lift dependence if any, maintenance access, heat stratification and family adaptability.

### D — Garden Pavilions

Lower-volume wings joined by covered circulation and arranged around one or more gardens. Test phasing, accessibility, daylight and robotic/panelized production. Evaluate the potential penalties of increased external envelope, longer service runs, weather exposure and maintenance.

These are spatial archetypes, not finished geometry. No exact building dimensions, structural spans, wall thicknesses or room areas are established in v0.1.

## 5. Design grammar

Explore a consistent vocabulary through functional architectural elements:
- a clear arrival threshold and sheltered entrance;
- a generous shared hall with daylight and strong spatial orientation;
- arcades, verandas and covered galleries;
- courtyard or garden sequences;
- a strong roof silhouette where it serves climate, structure or wayfinding;
- local stone or masonry expression paired with a professionally designed structural system;
- timber roof or upper elements only where the species, grade, durability, moisture, fire and supply chain are qualified;
- exposed, legible and maintainable connections where appropriate;
- restrained ornament derived from craft and building logic rather than pasted-on fantasy detail;
- accessible paths integrated into the main spatial composition, not relegated to a secondary route.

Aesthetic intent and poetic composition belong to Symthaea #6091/#6096. Do not create a universal beauty or meaning score. Use blind resident, architect, artist and craft review, and preserve disagreement.

## 6. South African site and climate basis

The initial context is Johannesburg/Highveld at regional scale only. No precise parcel, coordinate, cadastral boundary, slope, north point, zoning or geotechnical condition is assumed.

### 6.1 Candidate weather files

Sol-Atlas #593 and the current South Africa index identify five Gauteng 2011–2025 TMYx candidates for a regional station ensemble:
- `ZAF_GT_Johannesburg-Tambo.Intl.AP.683680_TMYx.2011-2025.zip`
- `ZAF_GT_Johannesburg.Botanical.Gardens.683610_TMYx.2011-2025.zip`
- `ZAF_GT_Grand.Central.AP.682650_TMYx.2011-2025.zip`
- `ZAF_GT_Rand.AP.683463_TMYx.2011-2025.zip`
- `ZAF_GT_Lanseria.Intl.AP.682635_TMYx.2011-2025.zip`

They are **catalogued candidates only**. The actual archives have not been header-inspected or hashed, and their station histories/representativeness are not qualified. The Climate.OneBuilding source page footer states "All Rights Reserved"; I did not locate a file-specific redistribution licence or reuse terms on that page. Therefore the source-rights state remains Unknown, and no archive is redistributed or admitted as an engineering input until terms/permission are confirmed. Preserve all five as separate station-exposure scenarios; do not average them blindly or select one by distance alone. TMY is a typical-weather file for screening, not a heatwave, extreme design event, future climate projection, wind design file or substitute for local measurement.

Before use, capture exact source, station coordinates/elevation, period, time zone, hourly variables, missingness, file SHA-256, reuse rights and representativeness uncertainty. Also create separate historical hot/cold/storm/hail event cases and future climate cases where qualified data exist.

### 6.1A Alternative source-provider research

EPW.tools is recorded as an alternative provider methodology to compare, not as a qualified replacement. Its published description says it prioritizes station observations, uses ERA5/MERRA-2 for gap filling, uses satellite solar products where available, and offers actual-year (AMY), typical-year and future-climate file products. Its validation page reports broad comparisons against ASHRAE design conditions and an hourly validation set based on seven NOAA SURFRAD stations. These are provider-published methods/results and have not been independently reproduced by Luminous or shown to validate a Johannesburg parcel.

Its Johannesburg temperature-trend page reports a 1981–2025 trend from a 0.5-degree NASA POWER/MERRA-2 grid cell; that is regional context, not a station or parcel measurement. Keep its grid trend out of design-weather input until source/variable/uncertainty checks are complete.

The source/methodology comparison is recorded in the scenario JSON as `RESEARCH_ONLY_NOT_INGESTED_OR_QUALIFIED`. The controlled evaluation should compare:
- Climate.OneBuilding TMYx candidates for typical-year screening;
- EPW.tools TMY products, if downloaded and qualified, as a competing typical-year pipeline;
- EPW.tools AMY for an exact measured operational year only when calibrating against that same year's metered indoor/utility observations;
- future-climate files only with explicit model, emissions scenario, horizon, baseline and morphing method;
- independently obtained station observations, where available, as an external comparison authority.

Do not mix TMY, AMY, historical extremes and future projections. A provider's global validation is not local Highveld validation.

### 6.1B Station identity and coordinate cross-checks

The machine-readable scenario preserves four published station-metadata references as **metadata-only cross-checks**:
- ASHRAE's 2021 HOF list gives Johannesburg Botanical (WMO 683610) at approximately 26.157°S, 27.999°E, 5,328 ft, and Johannesburg International (WMO 683680) at approximately 26.143°S, 28.235°E, 5,561 ft.
- NOAA NCEI's GHCND record `SFM00068368` lists Johannesburg International at -26.139°, 28.246°, 1,694.1 m, with daily-summary period ending 27 May 2020; the rendered page does not show completed location/equipment history.
- A current SAWS daily-rainfall bulletin index points to WMO OSCAR/Surface WIGOS `0-20000-0-68368` at -26.1430°, 28.2346°, 1,695 m. The secondary index says its record was rebuilt on 29 October 2026 and reports a prior town-level plotting point about 19.9 km away. The actual WMO record and SAWS station history still need direct retrieval.

These records are not silently merged. The TMYx WMO station code, GHCND network code, current WIGOS identifier and current instrument coordinates must be cross-walked with effective dates, platform history, instrument moves, elevation, reporting schedule and source lineage. A coordinate discrepancy is a source-quality question, not permission to select the newest coordinate or geocode a historical series to a town centroid.

Until that reconciliation is complete, they remain station metadata only: no downloaded observation series, no proof of independent data sources, and no local weather-file qualification.

### 6.2 Climate hypotheses, not conclusions

Compare:
- orientation and winter solar access with summer shading;
- east/west solar control, daylight and glare;
- thermal-mass/insulation combinations and night-time heat rejection;
- roof/ceiling insulation, moisture and condensation details;
- natural ventilation only when outdoor temperature, air quality, security, noise and weather permit;
- hail-resistant, maintainable roof systems and stormwater details;
- passive-first comfort with powered systems represented as separate dependencies;
- rainwater harvesting/storage/reuse with potable/non-potable separation and appropriate protection;
- PV, battery and load management with safe manual fallback;
- indigenous/climate-appropriate shade and habitat planting with low irrigation demand;
- stormwater attenuation, rain gardens and swales as engineered landscape infrastructure.

Use Symtropy #1900's version-pinned EnergyPlus adapter and #1790's solver boundary. A successful software run is not measured comfort, model calibration, health assessment or regulatory approval.

### 6.3 Site gates

Before a real site is selected or any site-specific design claim is admitted, obtain:
- title, zoning, land-use, servitudes, setbacks, height, coverage and municipal approvals context;
- survey, orientation, contours, slope, hydrology, access, adjoining conditions and utility records;
- geotechnical investigation, foundation recommendations and dolomite hazard review where relevant;
- site-specific stormwater, flood, wind, hail, wildfire and ecological constraints as applicable;
- current municipal building-plan process and the applicable versioned South African approval register in Sol-Atlas #604;
- resident/community requirements and any environmental/heritage permissions.

Johannesburg's dolomitic conditions make geotechnical evidence an early hard gate where relevant. The 2003 national dolomite guideline is a historical technical reference, not a substitute for current site-specific professional advice or authority requirements.

## 6C. South African approval pathway and policy watch

The National Building Regulations and Building Standards Act 103 of 1977 remains the core national statutory framework for building standards and local-authority jurisdiction. The actual approval pathway must be determined against the selected parcel, municipality, building classification, current regulations/SANS editions, professional scope and the South African crosswalk in Sol-Atlas #604. This design basis is not legal advice and does not encode remembered SANS clauses as current law.

For a real home project, verify the current NHBRC applicability, builder-registration and home-enrolment obligations. The NHBRC currently states that the home builder enrols the home at least 15 days before construction, and that enrolment enables stage inspections and the described consumer warranty protections. Applicability and any exemption must be resolved by the project’s qualified South African compliance/legal professionals; do not treat the design brief as a determination.

**2026 policy watch:** On 3 July 2026 the Minister of Public Works and Infrastructure launched the National Built Environment and Construction Safety Framework and announced that CBE Public Interest and Safety Regulations would be gazetted, with focus areas including structural and dolomitic occurrences, improved building compliance and certification aligned to SANS 17024. That announcement is a policy-development signal, not proof that the proposed regulations have already been gazetted or are in force. Sol-Atlas #604 should verify the actual Gazette publication, commencement date, final text, transitional provisions and competent-person requirements before any compliance rule is implemented.

References:
- National Building Regulations and Building Standards Act 103 of 1977: https://www.gov.za/documents/national-building-regulations-and-building-standards-act-16-apr-2015-1302
- NHBRC home enrolment: https://www.nhbrc.org.za/why-enrol/
- NHBRC home-builder registration obligations: https://www.nhbrc.org.za/why-register/
- Ministerial launch of National Built Environment and Construction Safety Framework, 3 July 2026: https://www.gov.za/news/speeches/minister-dean-macpherson-launch-national-built-environment-and-construction-safety

## 6D. Secure night ventilation and vector-exclusion evidence

The building brief now composes Symtropy [VECTOR-HABITAT-001 #4095](https://github.com/Luminous-Dynamics/symtropy/issues/4095) and Sol-Atlas [VECTOR-EXPOSURE-ATLAS-001 #618](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/618). These are research contracts, not local efficacy claims.

Do not infer that a designed opening produces useful ventilation in occupied conditions. Secure night ventilation must be tested with the actual screen/mesh and security arrangement, measured airflow, resident-operated state and outdoor air-quality/weather constraints. The new VH-012 negative control requires paired physical-security and airflow evidence and awards no ventilation credit when the opening is closed during occupied night use.

The 2026 randomized 40-house Kenyan pilot provides a rationale for testing screening, cool roofs and actual window-use barriers, but its results are not transferable effect sizes for Johannesburg. Local vector species, seasonal ecology, house construction, occupancy and sampling effort require independent qualification. Vector captures are not clinical disease-incidence outcomes.

Required evidence before any local claim:
- exact screen/mesh product, aperture, installation detail, defect/repair inspection and maintenance plan;
- fire, egress, accessibility, security and ventilation review of the same assembly;
- measured airflow and indoor/outdoor conditions with doors/windows in observed resident-use states;
- locally defined vector species and trap protocol, species identification confidence and captures per trap-night;
- privacy/consent and resident acceptance, with household-level public disclosure prohibited;
- explicit separation of environmental suitability, vector presence, exposure, infection and clinical disease outcomes.

## 7. Security, accessibility and life safety

The goal is a home that feels welcoming and is demonstrably well considered—not a militarized residence.

Design for:
- clearly legible public, visitor, shared, family, private and service thresholds;
- proportionate professional door/window/perimeter specifications;
- natural visibility balanced with privacy;
- maintainable lighting and landscape;
- responder access, address legibility and emergency approach;
- accessible daily routes and independent emergency egress;
- manual operation during grid, network or control failure;
- fire detection, compartmentation, escape and suppression as determined by qualified fire professionals;
- flood, storm, medical, smoke, power, water and communications failure cases.

A room is not automatically a refuge because it has a strong door. Refuge provisions require an explicit qualified fire strategy. No traps, weapons, hazardous deterrents, autonomous enforcement or default resident surveillance.

## 8. Materials and structural pathways

Compare, without presuming a winner:
1. conventional masonry / reinforced concrete / steel baseline;
2. stone/masonry architectural expression with a conventional professionally designed load-bearing system;
3. timber or timber-hybrid roof/upper structure where grade, durability, moisture, fire, pest strategy and qualified supply support it;
4. panelized or precast elements;
5. guarded robotic timber/panel fabrication;
6. additive construction only for specifically qualified components or formwork.

Foundations, load paths, retaining elements, roof structures, connections, fire resistance, durability and non-standard products remain with qualified specialist engineers and the applicable approval pathway in #604. Do not assume locally available material is structurally or environmentally suitable without testing and provenance.

## 9. Design-for-production and repair

Every future component should have:
- stable component ID and geometry revision;
- declared material, grade, source/lot and allowable variation;
- connection, tolerance and inspection requirements;
- manufacturing route, exact tool/workcell capability and calibration;
- handling/lifting/fixture prerequisites;
- assembly sequence and safe access;
- independent measurement and acceptance method;
- repair, replacement and disassembly path;
- manual recovery and safe work method.

First physical work should be a guarded off-site timber/panel cell for metrology, fixtures and dry-run assembly. No autonomous heavy lifting, cutting or public-site work is authorized by this design brief. Use Symtropy #1818/#1834 and Symthaea #4795.

## 10. Four-system architecture

| System | Highveld Keep contribution | Authority limit |
|---|---|---|
| Symthaea | Resident-stated intent, task requirements, alternatives, design grammar, conflict explanation, value-of-information and Pareto exploration | No engineering or statutory certification |
| Symtropy | Exact geometry/topology under #683, deterministic scenario/trajectory orchestration, qualified specialist solver adapters and validity propagation | Generic physics is not structural/fire/thermal certification |
| Sol-Atlas | Site/climate source evidence, parcel and municipal context, utilities, regional comparison and construction-delivery projection | Map is not survey, title, permit or building approval |
| Mycelix | Consent, resident decisions, material provenance, supplier/inspection receipts, corrections/revocation and lifecycle custody | Receipt is not professional sign-off, permit or title |

Third-party solvers retain their specific physical model authority. Qualified architects, structural and civil engineers, fire professionals, quantity surveyors, access professionals, contractors and municipal authorities retain professional and statutory responsibilities.

## 11. First comparison metrics

Keep outcomes as a vector:
- blind aesthetic / experiential / place-identity review;
- task completion, accessibility and wayfinding;
- usable area, circulation and spatial legibility;
- daylight, glare and views;
- thermal trajectories, HVAC energy and passive-service hours;
- ventilation and indoor air quality where supported;
- potable/non-potable water demand, storage and outage behavior;
- fire, egress, emergency approach and security review;
- structural solver error within declared model scope;
- initial and whole-life cost, carbon and water;
- fabrication completeness, tolerance, first-pass yield, defects and rework;
- installation, inspection and commissioning;
- maintenance, repair, replacement, adaptation and disassembly.

No single weighted score can hide hard-gate failure, uncertainty, resident disagreement or unequal outcomes.

## 12. Evidence tiers

- E0: design concept and assumptions.
- E1: deterministic synthetic geometry/requirements fixtures and independent checker.
- E2: version-pinned specialist solver result within independently validated scope.
- E3: calibrated physical component/mock-up measurements.
- E4: supervised site pilot, consent, independent inspection and professional review.
- E5: sustained operation, maintenance, longitudinal outcomes and external replication.

A tier applies per claim, model, revision, site and time. It is not a universal maturity badge.

## 13. References and source notes

- Sol-Atlas #593 Johannesburg heat/source readiness register, including five distinct regional TMYx candidates.
- Sol-Atlas #604 South African building-system/product approval crosswalk.
- Symtropy #1766 spatial design, #1790 adapter boundary, #1848 resilience, #1900 EnergyPlus qualification, #683 geometry provider and #1818/#1834 robotics.
- Symthaea #7790 intent-to-design, #6091/#6096 aesthetic/poetic engineering intent and #7801 robustness/lifecycle qualification.
- DHS/CSIR Red Book: https://www.dhs.gov.za/node/154 and https://www.csir.co.za/neighbourhood-planning-and-design-guide-red-book
- City of Johannesburg building plans: https://joburg.org.za/work_/Pages/Work%20in%20Joburg/Building%20Plans/Building-Plans.aspx
- NHBRC home enrolment: https://www.nhbrc.org.za/why-enrol/
- Climate.OneBuilding 2026 TMYx methodology: https://climate.onebuilding.org/sources/
- South African dolomite guideline (2003, historical reference): https://www.gov.za/sites/default/files/gcis_document/201409/dolomitemanual0.pdf
- IFC 4.3.2.0: https://standards.buildingsmart.org/IFC/RELEASE/IFC4_3/
- NIST Assembly Performance Metrics: https://www.nist.gov/el/intelligent-systems-division-73500/robotic-grasping-and-manipulation-assembly/assembly

## 14. Claim ceiling

This v0.1 package defines a synthetic computational reference design only. It does not establish parcel suitability, buildability, code compliance, structural/fire performance, security efficacy, measured energy/water performance, resident benefit, robot qualification or construction cost. Site, climate, material, geometry and solver evidence remain Unknown until exact sources and validations are attached.
