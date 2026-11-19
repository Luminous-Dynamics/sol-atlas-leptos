# Shared amenity commons benchmark v0.1

**Study ID:** SHARED-AMENITY-COMMONS-001  
**Parent:** Sol-Atlas CITY-NEIGHBOURHOOD-REFERENCE-001 (#596) / CITY-REFERENCE-DISTRICT-001 (#615)  
**Issue:** https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/940  
**Maturity:** BENCHMARK_PROTOCOL_AND_CASE_REGISTRY_ONLY  
**Execution status:** NOT_EXECUTED  
**Geographic profile:** Johannesburg / Highveld reference; method intended to be jurisdiction-portable  
**Claim ceiling:** no evidence yet that sharing is cheaper, preferred, safer, more accessible, or more resilient

## 1. Paradigm shift: compare delivered service, not amenity counts

A site plan may show a pool, workshop, greenhouse or library without proving that any resident can use it when needed. Availability depends on capacity, opening hours, route accessibility, staffing, utilities, maintenance, cost, governance, privacy and the actual demand trace.

This benchmark asks:

> For the same resident-defined service basket, population, geography, time horizon, accessibility requirements and operating scenarios, when does a shared asset deliver better whole-life value than private duplication or an external commercial service?

Sharing is a hypothesis, not a presumption. Centralization may lower duplicated capital but raise travel time, queues, operator dependence, privacy risks, common-cause failure and deferred maintenance. Distributed or hybrid options may outperform both centralized sharing and private duplication.

The unit of comparison is **accepted resident-facing amenity service over time**. Capital cost per room, amenity count, nominal floor area, utilisation percentage or a rendered masterplan cannot stand in for delivered service.

## 2. System boundary and owner responsibilities

This is a Sol-Atlas spatial/service comparison profile. It composes existing project owners:

| System | Owns | Does not claim |
|---|---|---|
| Sol-Atlas | Spatial alternatives, access/network projection, context and comparable reference scope | Title, structural approval, utility engineering or owner-native ledgers |
| Symtropy | Deterministic scenarios, resource/service trajectories, physical adapter evidence and fail-closed comparison | A universal building solver or statutory building approval |
| Symthaea | Resident brief reasoning, engineering alternatives, product/process/factory co-design | Legal consent, independent inspection or certification |
| Mycelix | Ownership/consent/provenance, accountable decisions, lifecycle events, stock-flow and stewardship evidence | Legal title by token, democratic legitimacy by account count, or engineering safety |
| Qualified practitioners / authorities | Applicable design, statutory, professional, inspection and operating decisions | Automated replacement by a software receipt |

Existing owners to compose, not duplicate: Sol-Atlas #555, #563, #596, #612, #615, #623; Symtropy #1687, #1766, #1790, #1848, #1900, #4181, #4314, #4531, #5335; Symthaea #5949, #6032, #8051; Mycelix #987, #954, #3095, #4808, #4816, #5149.

## 3. Comparison arms

Each arm must deliver the same declared service basket and pass the same non-compensatory requirements.

| Arm | Model | Required explicit inputs |
|---|---|---|
| PRIVATE_DUPLICATION | Households individually own and maintain a specified facility/equipment | Per-home capital, floor/land allocation, replacement, utility and maintenance needs |
| CENTRAL_SHARED | One common facility serves the reference population | Capacity, service schedule, access network, queue policy, staffing, utilities and reserve |
| DISTRIBUTED_SHARED | Multiple local shared facilities divide demand | Facility locations, capacities, routes, independent/common dependencies and operating model |
| HYBRID_SHARED_PRIVATE | Private minimum provision plus selected common facilities | Which service remains private, which is shared, fallback and duplication avoided |
| EXTERNAL_COMMERCIAL | Service purchased outside the development | Verified provider, price, opening hours, capacity, travel, eligibility and continuity |

An arm with missing provider, cost, capacity, scope or service evidence is UNKNOWN or NOT_COMPARABLE. Do not replace unavailable data with a zero cost or unlimited capacity.

## 4. Reference amenity basket

These are distinct service archetypes. A neighbourhood does not need every facility; resident-authored briefs and local demand determine inclusion.

| Archetype | Service to measure | Critical dependencies |
|---|---|---|
| Tool library / maker workshop | Tool availability, booking, safe access, repair and return | Custody, qualified supervision where required, power, consumables, maintenance |
| Shared laundry | Accepted wash/dry cycles and availability | Water quality/quantity, energy, drainage, ventilation, equipment uptime |
| Guest suites | Accessible overnight stays | Booking fairness, cleaning, privacy, maintenance, safeguarding |
| Greenhouse / food garden | Productive growing area and usable harvest/service | Water entitlement/quality, climate, inputs, staffing, soil/compost and maintenance |
| Pool / wellness | Safe, accessible operating hours and user capacity | Water, filtration, energy, trained operation, inspection, cleaning and reserve |
| Community kitchen / dining | Meal-preparation and gathering service | Food hygiene, ventilation, fire safety, cleaning and accessible equipment |
| Library / study / coworking | Usable quiet/work/study hours | Power, connectivity, acoustic separation, opening, privacy and seating |
| Childcare / learning | Qualified care/learning places and hours | Licensing, staffing ratios, safeguarding, access, sanitation and emergency provisions |
| Sports / park / public realm | Accessible use-hours and recreation capacity | Land, shade, lighting, water constraints, maintenance and safe routes |
| Shared utility plant | Delivered utility service and recovery continuity | Engineering qualification, isolation, metering, operators, spares and failure domains |

**Pool and greenhouse caution:** Johannesburg Level 2 water restrictions take effect 21 November 2026 and prohibit municipal drinking-water irrigation of gardens, lawns, parks and public spaces. A pool or greenhouse proposal must declare its water source, quality, permitted use, refill/top-up, evaporation, treatment, leakage and drought operating policy. Do not assume that harvested rainwater or greywater is automatically suitable for a given use.

## 5. Registered scenarios

v0.1 registers ten common scenario families. Exact numerical traces, probability distributions, demand values and local costs are deliberately unpopulated; they must be supplied by an identified source or clearly labelled synthetic fixture before execution.

1. **ORDINARY_DEMAND** — ordinary operating profile and declared household/service schedule.
2. **PEAK_SIMULTANEOUS_DEMAND** — coincident demand, queues, booking contention and overflow.
3. **SEASONAL_OR_EVENT_SURGE** — holiday, community event, seasonal growing or school-calendar variation.
4. **GRID_OR_WATER_INTERRUPTION** — declared utility interruption, reserves, passive fallback and service-floor degradation.
5. **STAFFING_OR_OPERATOR_OUTAGE** — absence, provider insolvency or operator replacement.
6. **MAJOR_REPAIR_AND_PARTS_DELAY** — planned maintenance, unplanned failure, replacement parts and return-to-service.
7. **OPERATING_COST_INFLATION_AND_RESERVE_SHORTFALL** — utility, labour, insurance, consumable and replacement-cost shock.
8. **ACCESS_ROUTE_DISRUPTION_OR_LIFT_OUTAGE** — route closure, vertical transport outage and alternative access.
9. **HOUSEHOLD_COMPOSITION_AND_DEMAND_CHANGE** — changed occupancy, tasks, schedules and service preferences.
10. **GOVERNANCE_CONFLICT_OR_OPERATOR_CAPTURE_ATTEMPT** — disputed decision, conflicted procurement, unauthorized rule change or attempted capture.

The five arms crossed with ten scenarios create **50 registered comparison cases**. The case registry is not an execution result.

## 6. Measurement model: keep the outcome vector visible

### Service delivery
For each amenity and time interval, preserve requested, eligible, scheduled, delivered, rejected, cancelled, unavailable and unmet demand. Distinguish a service-hour from a person-hour, booking, cycle, meal, overnight stay or other functional unit. Define the accepted-output predicate before comparing arms.

### Access and time
Record door-to-door route and travel time, access-window fit, accessible-route availability, wait-time distribution, queue abandonment, booking lead time, opening hours and privacy. A facility is not accessible merely because it lies inside a site boundary.

### Capacity and utilization
Report rated capacity, safe operating capacity, observed available capacity, peak concurrency, utilization by time window, overflow and unsatisfied demand. Utilization is not a substitute for availability or equitable service. A highly utilized facility can still be undersized.

### Lifecycle economics
Include capital, design, approvals, land opportunity cost, financing, transport, installation, staffing, operator contracts, cleaning, energy, potable and non-potable water, consumables, insurance, maintenance, repairs, replacement, reserve funding, disruption, decommissioning and residual value. Report:
- total discounted lifecycle cost under an explicitly named discount/price convention;
- cost per accepted service unit;
- cost per served household-year;
- cost per resident-facing service-hour where the denominator is meaningful;
- distribution of cost and benefit, not just development-wide averages.

Do not double-count transfers between resident levy, cooperative budget, operator revenue and supplier expenditure as if they were separate social resource costs. Keep cash-flow, financial incidence and real resource consumption ledgers distinct.

### Environmental and resource accounting
Where supported by declared methods and sources, report electricity, fuel, water by quality class, materials, waste and lifecycle carbon per accepted service unit. State boundaries and exclusions. An environmental label or supplier assertion is not measured impact.

### Reliability and recovery
Report service availability, fault detection, time to isolate, repair time, parts delay, recommissioning/reinspection time, restoration, repeat failure, common-cause dependency and fallback service. Distinguish a service that is physically present from one that is operable.

### Equity and participation
Report whether the service can meet explicitly resident-authored tasks and access requirements, by opt-in functional cohorts and use cases. Preserve differences between people. Do not infer needs or amenity preferences from age, diagnosis, household size, income or other demographic proxies. Never let a neighbourhood average hide an affected cohort with zero usable service.

### Governance and stewardship
Record the legal/organizational owner, beneficiary constituency, decision rights, operating authority, procurement controls, asset-purpose protections, conflicts, complaints and appeals, member exit, privacy, insolvency continuity, operator replacement, reserve oversight and handback. Participation counts alone do not establish meaningful influence.

## 7. Break-even analysis and Pareto comparison

No universal break-even density should be hard-coded.

For each amenity service, compare the full annualized and lifecycle cost curves over population/demand, facility capacity, service frequency, land cost, travel time, queueing, maintenance burden, staffing model and failure exposure. Identify:
- utilization/demand at which shared provision becomes lower-cost than private duplication;
- utilization at which a shared facility becomes congested or requires a second node;
- density/radius at which access time begins to outweigh capital savings;
- reserve and staffing requirements that change the result;
- sensitivity to land, interest, utility, labour, parts and replacement prices.

Present the feasible Pareto frontier for cost, delivered service, access time, privacy, reliability, resource use and distributional outcomes. Hard legal, safety and accessibility requirements are constraints, not weighted objectives. Do not collapse unlike outcomes into a single “amenity score.”

## 8. Governance and legal form profiles

The project must select its legal structure only after local legal and land-tenure review. Candidate forms may include:
- sectional-title common property and a body corporate;
- housing cooperative or shared deferred ownership, if properly established;
- a communal property association only where statutory eligibility and land-history conditions apply;
- nonprofit/social-housing institution with resident participation;
- contractual amenity operator with enforceable resident oversight;
- federated or polycentric stewardship across distinct asset owners/operators.

For every profile, separate:
1. asset/title holder;
2. asset-purpose guardian or constitutional steward;
3. resident/user representative body;
4. day-to-day operator;
5. independent reviewer/auditor;
6. relevant public/statutory authority.

Specify ordinary decisions versus reserved/constitutional matters, voting and representation, conflicts of interest, operator selection/removal, budget and reserve limits, access rules, complaints, appeal, records/privacy, insolvency, member exit, transfer/encumbrance and operational handback. An ordinary majority must not silently gain authority to privatize an asset-locked commons. The lawful mechanism and protection must be confirmed by counsel; a software ledger cannot create or substitute for legal title.

## 9. South African and interoperability references

- SHRA Social Housing Norms and Standards contain communal-housing requirements, including rooms with shared amenities, common bathrooms, kitchens and common-area amenities: https://www.etenders.gov.za/home/Download/?blobName=09797a42-8553-4615-a677-dab34af1c251.pdf&downloadedFileName=Norms.pdf
- Social Housing Regulations include design criteria for internal services, lifecycle maintenance obligations, metering, range of accommodation options and cooperative/shared deferred ownership provisions: https://www.acts.co.za/social_housing_act_2008/r51_20__housing_design_criter
- Sectional Titles Schemes Management Act 8 of 2011: https://www.gov.za/documents/sectional-titles-schemes-management-act
- Sectional Titles Act 95 of 1986: https://www.gov.za/documents/sectional-titles-act-17-sep-1986-0000
- Communal Property Associations Act 28 of 1996: https://www.gov.za/documents/communal-property-associations-act. The 2024 amendment announcement describes its scope and changes; eligibility is not universal: https://www.gov.za/news/media-statements/president-cyril-ramaphosa-signs-law-communal-property-associations-amendment
- buildingSMART IFC 4.3 / ISO 16739-1:2024: https://www.buildingsmart.org/standards/bsi-standards/industry-foundation--classes/
- buildingSMART IDS 1.0 for machine-readable alphanumeric information requirements and exchange validation: https://www.buildingsmart.org/standards/bsi-standards/information-delivery-specification-ids/

The standards crosswalk must pin the exact instrument, edition, amendments, effective status, applicability, municipality, use/occupancy, tenure and reviewer. An IFC/IDS exchange pass does not establish a building's physical performance or legal compliance.

## 10. Industry baseline and what would count as better

### Construction productivity is not amenity value
Singapore BCA states PPVC can raise productivity by up to 40% in manpower/time savings depending on project complexity. HDB reported a 25% site-productivity improvement against a projected 0.6 m²/man-day baseline on its first Construction Transformation Project. These figures are external construction references, not shared-amenity savings and not direct South African performance predictions:
- https://www1.bca.gov.sg/growth-and-transformation/productivity/design-for-manufacturing-and-assembly-dfma/prefabricated-prefinished-volumetric-construction-ppvc/
- https://www.hdb.gov.sg/hdb-pulse/news/2026/advanced-construction-technologies-at-hdbs-first-ctp-helped-achieve-25-productivity-improvement

### South African market baseline
NHBRC publishes an Approved Innovative Building Technologies dynamic database and research on structural performance of registered systems: https://www.nhbrc.org.za/innovative-building-technologies/
Agrément South Africa publishes active certificates, which must be checked for exact product, revision, scope and status: https://agrement.co.za/active-certificates/

These are baseline sources and qualification routes, not proof that an unreviewed system is approved.

### Luminous differentiation hypothesis
The research contribution is not “we have a digital twin” or “we use prefabrication.” It is the ability to join, on one exact reference scope:
- equivalent delivered service, not just building output;
- access, queue and capacity under real schedules;
- full lifecycle and reserve economics;
- water/energy/material constraints;
- repairability, replacement and handback;
- consent-aware multi-party ownership and governance;
- interoperable, revision-bound evidence that can be independently replayed.

We should claim superiority only after matched tests demonstrate better results without relaxing quality, accessibility, safety or service floors.

## 11. Proposed gates — targets, not results

- 100% of cases bind the same population, service, period, site, demand trace and operating conditions across comparison arms; otherwise mark NOT_COMPARABLE.
- 100% of seeded missing, stale, revoked, wrong-scope, corrupt or contradictory critical inputs remain UNKNOWN, BLOCKED or INVALID; zero optimistic defaults.
- 100% of seeded queue/capacity, travel/access, power/water, staffing, operator, maintenance and reserve failures are visible in the correct outcome.
- Exact agreement with hand-calculated synthetic reference examples for all deterministic capacity, queue, unit-conversion and stock-flow rules.
- Zero unserved demand silently dropped from reports.
- Zero lifecycle claims with unreported boundary exclusions.
- 100% of major common assets have a condition, inspection date, expected renewal horizon, replacement-cost source and reserve policy before a lifecycle-adequacy claim.
- Zero access, legal, life-safety or engineering constraints compensated away by a weighted cost or sustainability score.
- Every recommendation exposes its evidence state, uncertainty, scope, claim ceiling and responsible decision-maker.

### Future physical/operational target gates
These require a real site, consenting residents, measured cost/demand, independent evaluation and qualified operators. Establish targets only after baseline collection and power analysis. Do not fabricate numerical savings or utilization targets now.

## 12. Execution stages

1. **E0 — contract freeze:** schema, case registry, same-scope rules, units, outcome vectors, legal/authority boundaries and source register.
2. **E1 — registry integrity:** deterministic Rust auditor for frozen input identity, exact 5×10 coverage, functional-unit mapping, owner exclusions and claim-state invariants; adversarial mutations and a byte-bound summary receipt. This is not an operational service, capacity, cost or allocation evaluator.
3. **E2 — owner-native composition:** Sol-Atlas scope binding #623; Symtropy #3768 capacity/resource outputs and #1848 resilience where applicable; Mycelix #5901 allocation, #987 governance, #4808 accounting and lifecycle records; Symthaea engineering/process estimates. Read-only adapter composition; no schema shadow ledgers.
4. **E3 — measured baseline:** one amenity at a time. Collect lawful demand, actual operating cost, access, capacity, condition, utility use, staff hours and resident feedback with consent and privacy safeguards.
5. **E4 — non-occupied test cell:** choose the lowest-risk asset with the highest information value (tool-library custody workflow, laundry water/energy mock-up, greenhouse water loop, or pool plant mock-up). No mains/potable/occupied installation without qualified review and approvals.
6. **E5 — bounded live pilot:** independent assessment, matched comparison, resident governance, funded maintenance reserve and explicit stop/withdrawal process before any public-facing claim.

## 13. Claim ceiling

This protocol and registry are design artifacts. They do not demonstrate that shared amenities are cheaper, legally owned, adequately funded, accessible, safe, resilient or preferred by residents. Synthetic cases do not establish physical performance. A hash proves artifact identity, not source truth. An approval receipt is scoped to its authority and artifact revision. All empirical and statutory claims require exact sources, local applicability review and evidence from the relevant project.


### Additional 2026 benchmarks and local cost anchors (evidence classes kept separate)

| Reference | Reported evidence | Correct use in our benchmark | What it does not prove |
|---|---|---|---|
| Singapore BCA PPVC guidance (updated 1 Oct 2026) | Up to 40% manpower/time savings depending on project complexity; up to 30% floor-cycle time reduction; typical reported project savings of 1–2 months, with some 4+ months. Its Brownstone case reports up to 4 months and 20% project-level manpower savings. | Reference for factory-built volumetric construction, crane/logistics, design-for-manufacture, and end-to-end labour/time accounting. | Not a guarantee for low-rise South African homes, not a shared-amenity saving, and not comparable unless baseline and full workflow boundaries match. |
| Singapore HDB Construction Transformation Project (18 Sep 2026) | 25% site-productivity improvement against a projected 0.6 m²/man-day baseline, attributed to a package of advanced technologies and more productive methods. | Reference for programme-level site-productivity measurement; benchmark the package and its context, not one device. | Not an isolated technology effect or direct South African benchmark. |
| Modular Buildings project dataset (ASCE, 9 Sep 2026) | Study analyses 491 modular projects (303 permanent, 188 relocatable) across multiple sectors; productivity is gross m²/day and patterns differ by building category and project size. | Use as an external dataset reference and require building-type/size stratification. | Gross area/day is not accepted dwelling-service output; the abstract says data may be available from authors on request. |
| Modular 3D-printed housing techno-economic analysis (Journal of Cleaner Production, 14 Jan 2026) | A 120 m² Australian detached-home case models 20–30% lower upfront cost, 50–70% shorter construction time, 80–95% lower on-site labour, up to 40% lower material use and up to 55% lower embodied emissions versus a conventional modular baseline. Results are sensitive to scale and material prices. | A high-ambition scenario for matched techno-economic analysis and sensitivity to equipment utilization, material costs and project pipeline. | A modelled Australian case, not a South African project result, not proof of regulatory approval or measured in-service performance. |
| South African DPWI 2026/27 Annual Performance Plan, citing Stats SA December 2025 data | Average completed-building costs reported at R7,698.03/m² for dwelling houses below 80 m²; R9,010.23/m² for houses 80 m² and above; R10,994.73/m² for flats/townhouses. Plan values differ from completed-building values. | A dated national macro sanity check for early costing; preserve size/type and plan-vs-completed distinction. Replace with project-specific, current QS estimates and actual invoices before investment decisions. | Not a quote, not Johannesburg-only, and not necessarily inclusive of land, bulk services, finance, professional fees, abnormal ground conditions, VAT, furnishing or lifecycle operation. |

Sources:
- BCA PPVC: https://www1.bca.gov.sg/growth-and-transformation/productivity/design-for-manufacturing-and-assembly-dfma/prefabricated-prefinished-volumetric-construction-ppvc/
- HDB CTP: https://www.hdb.gov.sg/hdb-pulse/news/2026/advanced-construction-technologies-at-hdbs-first-ctp-helped-achieve-25-productivity-improvement
- Hasan & Choi (2026), *Correlation between Size and Productivity for Modular Buildings*, Journal of Management in Engineering: https://doi.org/10.1061/JMENEA.MEENG-7605
- Nguyen et al. (2026), *Techno-economic analysis of 3D printed modular housing: Productivity, cost, and environmental assessment*, Journal of Cleaner Production: https://doi.org/10.1016/j.jclepro.2025.147461
- South African DPWI Annual Performance Plan 2026/27, pp. 54–55, Stats SA building-cost series: https://www.publicworks.gov.za/PDFs/documents/StrategicPlans/Annual_Performance_Plan-26_27.pdf

### Candidate Luminous stretch targets (hypotheses to test, not forecasts)

Only after we establish a matched local baseline, scope and acceptance protocol, use the following as ambitious pilot hypotheses:

| Dimension | Candidate stretch objective | Qualification condition |
|---|---|---|
| Repeatable construction assemblies | At least 40% fewer on-site labour hours per independently accepted assembly than the matched local baseline | Include off-site labour, equipment, transport, lifting, setup, inspection, rework and commissioning in separate totals; do not hide displaced labour. |
| Site programme | At least 30% shorter critical-path time for a selected repeatable work package | Same scope, weather/shift calendar, crew capability, access, crane/plant, quality and handover boundaries. |
| Whole-life affordability | At least 15% lower fully burdened cost per equivalent dwelling-service-year or accepted amenity-service unit | Land, services, finance, staffing, utilities, maintenance, replacement, reserve, disruption and end-of-life are included or explicitly excluded. |
| Quality and rework | At least 30% less rework labour/material per accepted output | Independent acceptance and defect taxonomy; no reduced inspection or acceptance threshold. |
| Energy and water | At least 20% lower delivered energy and potable-water intensity for the same declared service | Same climate, occupancy and comfort/service floor; no credit for degraded performance. Water-source and use rights remain explicit. |
| Shared amenity access | At least 95% of resident-authored priority service requests delivered within their agreed access/wait window | Report each cohort and service class separately; 100% of mandatory accessibility and life-safety requirements remain hard gates. |
| Asset stewardship | 100% of major shared assets have current condition, renewal horizon, replacement-cost source, named operator and approved reserve policy | For sectional-title profiles, map the current statutory maintenance/repair/replacement requirements; do not generalize that regime to every tenure type. |

These are deliberately demanding research targets. A project that fails to beat them may still be valuable, but we should not call it better than state of the art. Report confidence intervals, sensitivity, adverse cases and the full Pareto frontier. The appropriate comparator changes with housing type, site, density, climate, tenure and local supply chain.

## 14. Scope boundary against adjacent Luminous work

The repository audit identified adjacent work that must remain authoritative. This benchmark is a **daily-use and productive-amenity comparison profile**, not a second essential-services resilience or apartment-design programme.

### Included in this profile
- Everyday shared-use amenities: learning, study/work, guest accommodation, laundry, routine tool access and making, community dining, childcare, recreation, sport and non-emergency food growing.
- Whether a household should own, share, use a distributed facility, use a hybrid, or buy an equivalent external service.
- Service-specific functional units, access/queue outcomes, cost allocation and lifecycle service value.
- Maintenance and governance of those specific amenity assets.

### Explicitly excluded / delegated
- **Essential household service floors and household resilience:** Sol-Atlas [HIGHVELD-HOUSEHOLD-SERVICE-001 #694](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/694).
- **Shared emergency energy/water/thermal refuge/communications/charging/repair logistics and neighborhood-resilience hub topology:** Sol-Atlas [HIGHVELD-NEIGHBORHOOD-RESILIENCE-COMMONS-001 #737](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/737). #940 may consume its receipts but must not redefine those assets or claims.
- **Apartment/floor/building design, vertical circulation and multi-family household-to-building composition:** Sol-Atlas [HOME-MULTIFAMILY-COMPILER-001 #840](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/840).
- **City-scale reference district, service basket and spatial integration:** Sol-Atlas #596/#615.
- **Common scope binding and owner-evidence envelope:** Sol-Atlas #623 and its existing PR #624. Do not create another scope digest format.
- **Generic service/capacity/resource feasibility:** Symtropy [CITYKIT-SYNTH-001 #3768](https://github.com/Luminous-Dynamics/symtropy/issues/3768). Consume its exact owner-native output; do not build a second general service-capacity oracle here.
- **Matching, reservation and constrained allocation policy:** Mycelix [#5901](https://github.com/Luminous-Dynamics/mycelix/issues/5901). This benchmark evaluates resulting service, unmet demand, fairness diagnostics and capacity evidence; it does not choose who morally deserves a slot.
- **Resident-led layout, accessibility and habitability:** Symtropy #1687/#1766/#1848.
- **Physical household water, thermal and room-scale service integration:** Symthaea #10323/#10329 and Symtropy #4531/#5335. This profile consumes those exact outcomes.
- **Legal ownership, stewardship, consent, asset lock and financial/physical lifecycle source records:** Mycelix #987/#954/#3095/#4808/#4816.

A routine shared tool library can be evaluated here as a resident amenity. A community emergency repair depot, strategic spares pool or resilience repair hub belongs to #737. A common laundry room is included here; a shared water-backup plant serving essential household use is not. A greenhouse is included as a resident productive amenity; emergency food security and shared water infrastructure remain with their respective resilience/resource owners. A pool plant is an enabling cost/resource component of the pool service, not a separately credited community utility service.

## 15. Amenity functional units: compare equivalent service, not unlike outputs

Every amenity class receives its own functional unit and acceptance definition. Never convert a pool person-hour, guest night, childcare place-hour and kilogram of produce into one invented universal “amenity unit”. A portfolio may show a vector of service/cost outcomes or a resident-authored bundle of separate service floors; it must not collapse unlike outputs into one scalar without a disclosed and justified preference model.

| Amenity class | Candidate functional unit | Minimum definition that must be frozen |
|---|---|---|
| Tool library / routine maker workshop | accepted tool-user-hour | Identified tool, allowed use, access window, working condition, safe-return/inspection and downtime treatment |
| Shared laundry | accepted laundry load | Load class/mass, service quality, cycle, rewash/reject treatment, water/energy boundary and turnaround |
| Guest suites | accessible guest-night | Room/service class, occupancy, accessible route/features, cleaning/turnover and booking window |
| Greenhouse / food garden | usable harvest kg by crop/grade | Crop identity, grade/quality, safety boundary, harvest point, losses and growing-cycle horizon |
| Pool / wellness | operable swimmer-hour | Independently declared operating envelope, capacity, water-quality/plant state, accessibility and closure time |
| Community kitchen / dining | accepted meal service | Meal/service definition, declared dietary/allergen information, food-safety process and service window |
| Library / study / coworking | accessible seat-hour | Seat/workspace class, hours, connectivity/acoustic service where promised, access and interruption |
| Childcare / learning | licensed place-hour | Applicable license/registration, service age/class, staffing and safety conditions, attendance and closure state |
| Sports / recreation | usable court-hour or recreation-space-hour | Activity, surface/lighting/accessible use condition, opening window, maintenance and weather closure |
| Non-essential amenity plant | supporting asset only | Pump/filter/ventilation/plant availability and cost is attributed to the service it enables; it is not a standalone beneficiary service unit |

Functional-unit equivalence requires the same service definition, quality class, beneficiary/task, opening window, geography/access boundary, period/season and lifecycle boundary. If the definitions differ, return NOT_COMPARABLE or show separate service classes. A calendar hour of opening is not necessarily a delivered user-hour. Nominal capacity is not accepted output.

## 16. State-of-the-art comparison protocol for design software

Autodesk Forma's 2026 Building Layout Explorer is an experimental AI-assisted multifamily layout exploration capability. Autodesk says it can generate/compare options against unit mix and circulation/core placement, and explicitly describes its current output as an exploration starting point rather than a dependable finished design. Its growing connected workflow links site planning, building design, Revit, Civil 3D, site/subsurface context and project data. References:
- https://blogs.autodesk.com/forma/2026/10/14/what-autodesk-forma-announced-au-2026/
- https://adsknews.autodesk.com/en/news/autodesk-forma-ai-aec-connected-workflows-2026/

Do not claim to beat Forma by generating more layouts or faster images. A fair matched benchmark should give each system the same resident/unit brief, site/context package, unit mix, hard constraints, time budget and downstream exchange task. Compare:
- feasible candidate yield under mandatory constraints;
- diversity of genuinely different spatial families;
- resident/task coverage and accessibility conflict visibility;
- daylight/thermal/acoustic/service analysis evidence and unknown handling;
- design-to-IFC/IDS information retention and change propagation;
- time to first credible option and time to independently reviewable package;
- false-feasible rate, missing constraints and professional correction effort.

Autodesk's experimental feature is one competitor, not the entire industry baseline. Include manual architect workflow and at least one other applicable layout/planning tool when a real comparison begins. Preserve software versions, feature flags, input files, operators and reviewer protocol. Never compare an AI concept sketch with a fully documented professional design package.

For construction productivity, the 2026 ASCE analysis of 491 modular projects uses gross square metres completed per day and finds differences by building category and size (https://doi.org/10.1061/JMENEA.MEENG-7605). That is useful for context, but our distinctive functional unit should remain the independently accepted assembly and, higher up, the equivalent dwelling/neighbourhood service-year. Report gross area/day alongside complete-workflow labour and accepted service; do not substitute one for the other.


## 17. Broader AEC state-of-practice comparator matrix

A credible comparison must cover separate classes of tools and delivery systems. Product pages below establish advertised or documented capabilities, not independent proof of superior outcomes.

| Comparator | Documented capability / reported benchmark | How Luminous should compare | Evidence class |
|---|---|---|---|
| Autodesk Forma Building Layout Explorer | Experimental multifamily layout generation/comparison against unit mix, circulation and core placement; Autodesk describes it as an exploration starting point, not a dependable final design. | Same unit mix, site, brief, hard constraints and time budget; measure feasible yield, diversity, false-feasible rate, constraint coverage, review effort and exchange loss. | Vendor-described experimental capability |
| Finch3D | Product page describes firm plan libraries, generated layouts, embedded accessibility/local-code rules and rapid iteration from feasibility to detailed layouts. | Give it the same authored standards, accessibility rules, unit mix and candidate brief; measure independently checked constraints, not vendor-described option counts. | Vendor capability claim |
| TestFit Site Solver | Product page describes site/unit-mix scenario exploration, financial/area metrics, imported standards, Revit integration and a claim of exploring 7× more scenarios. | Compare site yield, unit mix, parking/circulation, local zoning/service constraints, feasibility false positives, export integrity and total review effort. | Vendor capability/marketing claim |
| Hypar | Product updates describe editable windows/CAD components and a 300,000 ft², eight-building test-fit case delivered in six weeks with IA Interior Architects. | Use an identical site/program and compare model completeness, rule transparency, interoperability, time to a reviewed package and rework. | Vendor product update / vendor case study |
| Singapore BCA PPVC | Up to 40% manpower/time savings and up to 30% floor-cycle reduction, depending on project complexity; project case variation is material. | Matched complete-workflow labour, programme, transport/lifting, quality and handover boundary; do not transfer a Singapore result to South Africa as a forecast. | Government programme guidance and case examples |
| Hong Kong MiC / MiMEP | The High Productivity Construction programme promotes off-site prefabrication, multi-trade integration and plug-and-play. Official guidance states MiMEP can reduce on-site MEP installation time, manpower and material waste by at least 30%; MiMEP had been adopted in more than 160 new and RMAA projects by 2026. | Compare complete installed and commissioned MEP service, including factory labour, transport, installation, fit-up, inspection, commissioning, rework and maintenance; separate MiC volumetric modules from MiMEP service cassettes. | Government programme and adoption data |
| ASCE 491-project modular study | Analyses 303 permanent and 188 relocatable modular projects; reports gross square metres/day and differences by sector and size. | Stratify by housing type, project size and permanency; report gross area/day but retain accepted assemblies and dwelling-service-year as the Luminous functional units. | Peer-reviewed project dataset |
| South African NHBRC / Agrément | NHBRC publishes dynamic approved-IBT databases; Agrément certificates apply to specific systems and scopes. | Verify exact product, configuration, revision, certificate status, intended use, jurisdiction and site conditions. Treat directory presence as a search lead, not blanket approval. | National qualification pathway / registry |

References (checked 19 November 2026):
- Autodesk Forma AU 2026: https://blogs.autodesk.com/forma/2026/10/14/what-autodesk-forma-announced-au-2026/
- Finch3D product: https://www.finch3d.com/product
- TestFit Site Solver: https://www.testfit.io/product/site-solver
- Hypar product updates: https://docs.hypar.io/what%27s-new-or-product-updates
- Hong Kong Project Strategy and Governance Office, High Productivity Construction: https://www.psgo.gov.hk/en/high-productivity-construction.html
- Singapore BCA PPVC: https://www1.bca.gov.sg/growth-and-transformation/productivity/design-for-manufacturing-and-assembly-dfma/prefabricated-prefinished-volumetric-construction-ppvc/
- ASCE modular project study: https://doi.org/10.1061/JMENEA.MEENG-7605
- NHBRC approved Innovative Building Technologies: https://www.nhbrc.org.za/innovative-building-technologies/

### Required head-to-head protocol

1. Freeze a shared input package: parcel/geometry rights, site boundary, unit mix, resident tasks, accessibility requirements, zoning/approval data, climate inputs, service demand, budget/finance assumptions and evaluation horizon.
2. Record exact product versions, deployment region, enabled features, operator skill, configuration, plan/subscription tier and export settings for every competitor.
3. Use equal time budgets and equal evidence inputs. Keep manual professional workflows as a comparator, not as an assumed weak baseline.
4. Keep concept generation, code/rule checks, engineering analysis, IFC/IDS exchange, construction documentation and physical acceptance as different maturity gates.
5. Measure feasible yield, true spatial diversity, hard-constraint recall, false-feasible rate, change-impact completeness, model-exchange semantic loss, correction time, accepted-output productivity, whole-life cost and post-occupancy service.
6. Have independent architects, engineers, QSs, access reviewers and residents evaluate the same blinded candidate packages under a preregistered protocol.

A layout generator is not “better” because it creates 10,000 options. It is better only if it reliably surfaces more genuinely feasible, useful, diverse and reviewable solutions with lower total human correction burden and no hidden loss of safety, accessibility, service quality or resident agency.

### Luminous stretch targets — ambition beyond vendor feature claims

These remain unmeasured targets, not expected results:
- 100% hard-constraint traceability from resident brief through selected design and exchange package; zero silent loss of mandatory requirement IDs.
- Zero false-feasible candidate promoted to a qualified design state.
- At least four materially distinct spatial families retained in the Pareto set when the brief admits them; diversity is measured by topology/use, not cosmetic surface variation.
- 100% of candidate recommendations expose unresolved requirements and the professional role needed to resolve them.
- At least 30% reduction in total professional correction/rework hours to reach an independently reviewable package on a matched brief.
- At least 20% reduction in end-to-end design-to-accepted-output time on a repeated, locally buildable work package, including manufacturing, transport, installation, QA and commissioning.
- At least 20% lower fully burdened cost per equivalent dwelling-service-year, with identical service floors and lifecycle boundaries.

The targets are not all necessarily jointly attainable. Report the Pareto frontier and the evidence behind each achieved target; never trade mandatory access/safety/legal requirements for speed or price.


## 18. Compose the dream home, optional experiences and neighborhood food system

The benchmark is not a new architectural catalogue. It consumes the existing Highveld and Home Freedom work:

- **Highveld Keep #607** is the initial South African computational reference dwelling with four distinct spatial families: Courtyard Cloister, Winged Great Hall, Compact Keep and Garden Pavilions. It already defines the synthetic resident/task programme, accessible arrival-level essentials, kitchen/dining, great hall, library, work/making, sleeping rooms, service/storage and courtyard/garden.
- **Home Expression Library #904** owns resident-directed style blends and optional experience packages, including library, workshop, conservatory/greenhouse, guest pavilion, pool/water feature, spa/sauna/plunge, music studio and other additions. An experience package is a functional space/system option with cost, service, spatial, utility and maintenance implications—not merely a decorative tag.
- **Household service composition #694** owns the link from a home design to appliance, water, thermal, air and household service floors.
- **Multifamily compiler #840** owns apartment/floor/building configurations and vertical shared facilities.

Therefore, do not create another room catalogue or design generator here. Consume the resident-authored brief and exact design revision, then compare whether an amenity belongs in the private dwelling, an accessory structure, a shared facility, a distributed neighbourhood node or an external provider.

### 18.1 Home and experience configuration

For each optional experience package, record:
- resident-authored task and reason; preference may be required, preferred, optional, phase_later, declined or undecided;
- exact home/candidate and revision;
- spatial area, adjacency, access, privacy and servicing needs when known;
- build assembly/product demand and supply evidence;
- capital, operating, replacement and maintenance implications, each with source/uncertainty;
- service dependencies and manual fallback;
- whether the same function can instead be shared or purchased externally.

A pool is not a mandatory sign of a dream home. A greenhouse is not necessarily economical on an individual plot. A shared guest suite can remove duplicated spare bedrooms for some households but is not equivalent for households requiring permanent private accommodation. Those are resident-defined alternatives, not defaults imposed by the optimiser.

### 18.2 Greenhouse and food-growing composition

A greenhouse is a productive residential amenity only when it delivers a defined resident-facing service such as usable harvest. It must not be evaluated by planting area or rendered greenery alone.

Use the following owner split:

| Owner | Responsibility | What this benchmark consumes |
|---|---|---|
| Sol-Atlas #107 AGRO-CLIMATE-001 | Highveld/region-specific food, climate, soil and water evidence readiness | Currentness, geography, spatial resolution and uncertainty; not a plot-level forecast |
| Sol-Atlas #105 ATLAS-FOOD-001 | Privacy-preserving spatial food-network projection | Permitted producer/pickup/aggregate-demand/fulfilment view, not inventory or household identity |
| Symtropy #3957 CEA-FULL-BOUNDARY-001 | Whole-facility resource and nutritional-service accounting | Accepted harvest/service, electricity, heat, water, nutrients, labour and lifecycle boundary |
| Symtropy #4065 CEA-CLIMATE-001 | Spatial temperature, humidity, CO2, airflow, moisture and heat-rejection dynamics | Qualified climate/airflow outputs and calibration/applicability limits |
| Symtropy #4074 CEA-HARVEST-CONTROL-001 | Harvest-constrained lighting/climate control comparison | Declared harvest target, resource/control policy and modelled-versus-measured status |
| Mycelix #5001 MYC-FOOD-001 | Producer cooperative, typed product quantities, lots, orders, fulfillment and provenance | Source/lot/accepted-delivery references; planned harvest must never be treated as inventory |

Candidate functional unit: kg of usable harvest by declared crop and grade. Keep crop identity, grade, quality, losses, growing-cycle horizon and harvest point. Report separately:
- usable harvested mass and rejected/lost mass;
- electricity and useful heat per kg by crop/grade;
- potable and non-potable water by source/quality class per kg;
- nutrient/seed/substrate inputs and residual streams;
- labour and maintenance hours;
- area/volume and throughput per cycle;
- market/household service delivered versus forecast/planned production;
- capital, replacement, operating and lifecycle cost.

Do not compare lettuce, herbs, tomatoes, microgreens and fruit as interchangeable kg. Nutritional service may be reported when an admitted composition method and accepted harvest composition are present; do not infer nutritional value from mass alone. A modeled yield is not a measured yield. A harvest forecast is not a lot. A mapped producer is not inventory. A food-network projection is not food-safety approval.

### 18.3 South African water and climate constraints

For Johannesburg, the Level 2 restrictions effective 21 November 2026 prohibit using municipal drinking water to irrigate gardens, lawns, parks and public spaces. The greenhouse and landscape design must therefore declare source and lawful use for every water flow: municipal potable, rainwater, greywater, treated wastewater or other separately qualified source. Do not presume that a stored or treated stream is fit for crop contact, washing produce, pool top-up, irrigation or any other use.

The Highveld climate profile must not assume that a regional climate prior predicts a specific parcel's microclimate, soil condition, water availability or crop yield. Site testing, local rights, source quality, seasonal continuity, soil history and operator capacity are prerequisites for a physical installation.

Landscaping should prioritize climate-appropriate and indigenous planting, shade, low-irrigation design, soil health, stormwater attenuation and low-maintenance planting. Rain gardens, swales and stormwater systems must be engineered for the site rather than treated as decorative water features.

### 18.4 Pool and wellness boundary

The pool remains a resident-facing amenity service class with candidate unit operable swimmer-hour. #940 compares whether private, shared or external access is more appropriate, the operating window, availability, cost, access, water/energy intensity and downtime.

It does **not** design or certify pool hydraulics, electrical safety, chemical treatment, water quality, public-health controls, structural support, barriers or statutory pool safety. These require separate qualified design, operation and inspection. Symtropy #4531 room-scale household service integration and #5335 WaterLoop can provide bounded mock-up/service-system evidence where applicable; neither is a pool approval.

A pool option must include source water, evaporation, leakage, treatment/backwash, energy, cleaning, seasonal closure, accessibility, operating staff, inspection and maintenance reserve. Water restrictions and lawful water source take precedence over discretionary amenity targets.

### 18.5 Dream-home and commons comparison

For the same resident-authored task, compare:
- private room/equipment inside the dwelling;
- a separate accessory structure on the household plot;
- a shared amenity operated by the residents/community;
- several smaller shared nodes;
- an external commercial provider;
- omission or later phasing, where the resident accepts that option.

The result is a service/cost/access vector and a resident decision brief. A lower-cost shared design may be rejected by a resident who values privacy, independent access, permanent guest accommodation or a particular workshop. A private facility may be unaffordable to maintain. Preserve the preference and its rationale; do not infer a preferred arrangement from household type or demographic profile.

The benchmark becomes especially useful when linked to #607 and #904: the same Highveld reference home can be explored as a restrained core dwelling, an expressive dream-home configuration, and a household embedded in a neighborhood with shared alternatives—without treating the most expensive configuration as the default or claiming the same layout suits every person.
