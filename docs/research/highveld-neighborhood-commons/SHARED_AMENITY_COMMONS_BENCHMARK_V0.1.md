# Spatial amenity provisioning benchmark v0.1

**Study ID:** SHARED-AMENITY-COMMONS-001  
**Tracking issue:** https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/940  
**Parent:** Sol-Atlas CITY-NEIGHBOURHOOD-REFERENCE-001 (#596) / CITY-REFERENCE-DISTRICT-001 (#615)  
**Maturity:** BENCHMARK_PROTOCOL_AND_CASE_REGISTRY_ONLY  
**Execution:** NOT_EXECUTED — 50 registered synthetic design cases; no numeric inputs or result claims  
**Geographic profile:** Johannesburg / Highveld reference method; portable with jurisdiction-specific rules  
**Claim ceiling:** no proven savings, resident preference, site suitability, legal entitlement, service capacity, affordability or safety claim

## 1. Correct problem boundary: where should a service be provided?

This is a **spatial provisioning comparison**, not a second shared-amenity operating platform.

The question is: given the same resident-authored task, population frame, amenity service, service quality, time horizon and evidence envelope, should the function be provided privately in each dwelling, in one central facility, through several local nodes, through a private/shared hybrid, or by an external provider?

The benchmark compares spatial consequences: catchment reach, route accessibility, time/cost burden, proximity to transit and other daily destinations, distribution of access, land opportunity cost, phasing and resilience to facility/provider loss. It does not decide who receives a booking or establish that a facility is open, safe, legally owned or adequately funded.

### Canonical owner map — compose, do not duplicate

| Owner | Canonical responsibility | Sol-Atlas #940 consumes |
|---|---|---|
| Sol-Atlas #555 / PR #564 | Effective access: network reachability, walk/cycle impedance, opening, affordability, eligibility, step-free access and reachable-capacity potential | Group-specific access outcomes and their evidence/currentness. Do not implement another routing engine. |
| Sol-Atlas #623 / PR #624 | Common district scope digest and owner-evidence envelope | Same population, service frame, geography, horizon, scenario, functional-unit and owner revision binding. |
| Sol-Atlas #612 | Functional equivalence for component, dwelling service-year and neighbourhood service-year | Correct denominator and conversion boundary. |
| Sol-Atlas #816 | Whole-household cost of place: land/tenure, finance, servicing, utilities, transport, maintenance and access affordability | Household affordability and cost vectors; #940 does not recalculate these. |
| Mycelix #6328 / draft PR #6329 | Canonical shared-amenity asset, service, beneficiary, access, finance, reserve, maintenance, governance and handback contract; draft corpus has 33 synthetic cases | Versioned service definition, asset and operating evidence references. The draft contract remains subject to owner review. |
| Mycelix #5901 | Matching and constrained allocation policy | Allocation outcome evidence where relevant; #940 does not decide moral priority or build a second allocator. |
| Mycelix #912 | Capital-to-commons finance and stewardship transition | Capital claim and stewardship profile where a provision alternative uses this model. |
| Symtropy #3768 | Generic service/capacity/resource feasibility | Qualified capacity and resource receipts; no second service-capacity solver. |
| Symtropy #3957/#4065/#4074 and Mycelix #5001 | CEA full-boundary resource/nutritional service, greenhouse climate/harvest control and food cooperative lots/fulfilment | Greenhouse/growing service evidence when a local productive amenity is considered. No second crop, climate or food inventory model. |
| Sol-Atlas #607/#904 | Highveld Keep reference dwelling; resident expression and optional experience packages | Resident-defined private/accessory/shared/external amenity alternatives; do not create another room or style catalogue. |
| Sol-Atlas #694/#737/#840 | Essential household services, neighbourhood resilience commons and multifamily building design | Existing authoritative service floors and building/district configurations; not reimplemented here. |

The manifest marks candidate amenity functional units as `CANDIDATE_NOT_OWNER_APPROVED`. This is deliberate: Mycelix #6328/#6329 has the canonical service contract and may revise these units. A schema-valid mapping is not owner approval.

## 2. Paradigm shifts

1. **From facility count to service opportunity.** A map pin or floor area is not usable service. Access requires route, time, opening, eligibility, affordability and an owner-qualified service record.
2. **From centralization to service topology.** A facility may be private, accessory, central, distributed, hybrid or external. The best topology depends on population density, travel network, demand, land cost and failure domains.
3. **From distance to effective access.** Use network and multimodal evidence, not Euclidean radius alone. Treat step-free routes, gradients, crossings, opening windows, price, eligibility and time-of-day as part of access.
4. **From average access to distributional guarantees.** Report P50/P90, worst-decile, no-access cohorts, and local service-capacity bottlenecks. A good district average cannot erase a resident group with no usable route or no eligible service.
5. **From static plan to phased service network.** Model construction/occupancy phases, demand emergence, provider opening/closure and alternate service routes. A masterplan can be spatially attractive but fail before its promised facilities arrive.
6. **From isolated assets to robust alternatives.** A central facility can fail as a single point of service failure. Distributed options can provide substitutes, but may multiply staffing, maintenance and land burdens. Measure both.
7. **From one opaque score to Pareto choices.** Keep access, service, affordability, land, privacy, environmental burden and resilience separate. Residents choose among feasible trade-offs; no weighted score can override rights or hard accessibility floors.

## 3. Matched spatial provision alternatives

| Arm | Provision topology | Required same-scope assumptions |
|---|---|---|
| PRIVATE_DUPLICATION | Each household has its own version of the selected amenity or equipment | Same service definition, quality, opening/use window, household population and resident requirements. |
| CENTRAL_SHARED | One shared node serves the declared catchment | Site location, owner-native service capacity, operating/access window, routes and catchment boundaries. |
| DISTRIBUTED_SHARED | Multiple smaller shared nodes serve sub-catchments | Node locations/capacities, route graph, overlapping catchments, common dependencies and fallback topology. |
| HYBRID_SHARED_PRIVATE | Private minimum plus selected shared services | Exact private floor, shared supplement, fallback path and duplication avoided. |
| EXTERNAL_COMMERCIAL | Existing external providers supply equivalent service | Verified provider/service, price, opening, capacity/eligibility evidence, route and continuity. |

Every arm must provide an equivalent declared service basket. When it cannot, report `NOT_COMPARABLE`; do not quietly change the functional unit, quality, population or evaluation period.

## 4. Resident-facing amenity service basket

These are candidate service classes to locate and compare, not a requirement to build all of them. Functional units are candidate mappings for Mycelix #6328 review.

| Amenity | Candidate functional unit | Spatial question |
|---|---|---|
| Community hall | accessible community-room-hour | How many nodes and what catchment are needed for ordinary use and events? |
| Community kitchen | accepted meal-service | Is a local shared kitchen reachable at the required time, or is private/external provision equivalent? |
| Coworking / library / study | accessible seat-hour | Does distributed access reduce travel without making staffing/connectivity uneconomic? |
| Workshop / tool library / repair | usable equipped-workshop-hour | Should workshops cluster, distribute by catchment, or remain private/accessory? |
| Guest suites | accessible guest-night | Can shared guest accommodation substitute for some private spare-room demand without violating resident privacy or access needs? |
| Shared laundry | accepted laundry-load | What density and walking catchment justify one laundry node versus in-home equipment? |
| Greenhouse / nursery / food growing | usable harvest kg by crop/grade | Which sites have lawful water, climate and food-network evidence, and which producer/pickup nodes can serve residents? |
| Pool / wellness | candidate operable pool-lane-hour | What is the access and resource burden of private versus shared access? Public-pool service remains blocked/out-of-scope absent applicable operator and statutory evidence. |
| Gym / recreation | usable activity-space-hour | Do distributed smaller facilities improve coverage or duplicate equipment/staffing? |
| Non-essential amenity plant | enabling asset only | Plant is attributed to the named amenity service; it is never counted as a separate resident-facing service. |

No universal `amenity unit` is allowed. A guest-night, seat-hour, meal, pool-lane-hour and crop/grade kilogram are not interchangeable. A spatial access result does not promote an owner service claim.

## 5. Ten spatial scenarios — 50 registered cases

The manifest crosses five provision arms with these ten synthetic spatial contexts:

1. `COMPACT_HIGH_DENSITY` — short route network, high resident concentration and potential peak demand.
2. `LOW_DENSITY_DISPERSED` — longer distances and lower per-node utilization.
3. `MIXED_USE_TRANSIT_ADJACENT` — amenity catchments overlap transit and daily destinations.
4. `SITE_EDGE_LONG_WALK` — residents on the site edge face longer access to central facilities.
5. `ACCESSIBLE_ROUTE_DISCONTINUITY` — direct geometric proximity exists but step-free/usable route continuity does not.
6. `UNEQUAL_POPULATION_DISTRIBUTION` — population is unevenly distributed within a nominally equal radius.
7. `PHASED_OCCUPANCY_GROWTH` — residents and facilities enter service in stages.
8. `LAND_SCARCITY_COMPETING_USE` — amenity land competes with housing, public realm, productive work and ecological uses.
9. `EXTERNAL_PROVIDER_DISTANCE_OR_CLOSURE` — external service exists but is distant, costly, ineligible or closes.
10. `MULTI_NODE_CATCHMENT_REBALANCING` — distributed nodes and catchments change after a node/provider loss or phased growth.

All 50 case records remain `REGISTERED_DESIGN_ONLY`, `NOT_EXECUTED`, `NOT_SUPPLIED` and `NO_RESULT`. The manifest is a case registry, not an execution receipt.

## 6. Spatial model and owner-native evidence

Each candidate comparison must bind, through Sol-Atlas #623:
- the same boundary, CRS, population frame, demand/task definition, period, service scope, functional-unit version and scenario;
- exact amenity service/asset references from Mycelix #6328, with current owner revision and claim ceiling;
- access network and service-reach evidence from CityMorph #555, with mode, time, route state, impedance, accessibility and source currentness;
- owner-native capacity/service evidence from Symtropy #3768 and allocation evidence from Mycelix #5901, where needed;
- cost/affordability outputs from Sol-Atlas #816, not an invented per-room cost;
- any capital-to-commons or stewardship assumptions from Mycelix #912;
- design/home revision and resident intent from Sol-Atlas #607/#904;
- source and resolution limits, and the exact upstream receipt or schema revision.

Do not duplicate shortest-path routing, GIS, facility booking, allocation, capacity, service, finance, asset, maintenance or governance engines. The Sol-Atlas layer combines the owner receipts into spatially comparable alternatives and highlights missing, stale, mismatched or inaccessible claims.

## 7. Metrics — keep the outcome vector visible

The machine-readable manifest freezes eight reporting vectors:

| Vector | Authority / interpretation |
|---|---|
| SERVICE_OPPORTUNITY_AND_AVAILABILITY | Read-only owner evidence of usable service windows and service existence. |
| EFFECTIVE_ACCESS_AND_TIME | CityMorph #555 route/time/access results. |
| REACHABLE_CAPACITY_POTENTIAL | Capacity reachable by a group; not allocated supply or bookings. |
| SPATIAL_TOPOLOGY_AND_LAND | Sol-Atlas comparison of node locations, catchments, competing land uses and phase. |
| HOUSEHOLD_AFFORDABILITY_OUTPUTS | Sol-Atlas #816 cost-of-place output; not recomputed by this benchmark. |
| RESOURCE_ENVIRONMENT_OWNER_OUTPUTS | Qualified Symtropy/other owner outputs; no shadow resource ledger. |
| ROBUSTNESS_AND_RECONFIGURATION | Alternative routes/nodes, substitution, phase and option-value outcomes. |
| DISTRIBUTIONAL_EQUITY_AND_PRIVACY | Cohort-level access gaps with declared consent and disclosure bounds. |

### Effective access
- network travel time and generalized travel cost by mode and relevant time window;
- median, P90 and worst-decile travel burden, plus share of residents with no known usable route;
- population/task-weighted reachability to service classes;
- cumulative opportunities and reachable service options within resident-defined travel budgets;
- step-free route availability and route discontinuity for declared accessibility requirements;
- opening-window fit, eligibility and affordability filters;
- external-provider availability and provider-loss sensitivity.

### Service opportunity and capacity
- reachable, owner-qualified capacity potential per demand unit (not allocated supply);
- service demand, reachable capacity and aggregate facility capacity kept as separate quantities;
- facility capacity counted once in city/category totals;
- number of independent reachable providers/nodes and their common dependencies;
- queue/wait/capacity results only from the canonical Mycelix/Symtropy owner receipts.

### Distribution and equity
- access distributions by declared, consented task/function cohorts;
- neighborhood-level and within-neighborhood access gaps;
- low-access/no-access populations and affected resident tasks;
- no demographic proxy is used to infer a person's amenity preference.

### Land, cost and environmental burdens
- land opportunity cost and housing/public-realm/productive/ecological alternative use;
- lifecycle/household affordability vector from #816;
- travel time and trip burden per accepted service unit;
- water/energy/resource intensities only from qualified owner receipts;
- privacy, noise, congestion, maintenance and operator burden where evidenced.

### Robustness and phasing
- access preserved after facility/provider/route loss;
- time to viable substitute or recovery, from owner-native continuity evidence;
- phase at which each service becomes accessible to residents;
- service gap duration while construction or transition occurs;
- sensitivity to population growth, density, land price, travel time and provider availability.

Do not compute one blended amenity score. Report the Pareto frontier and the full distribution.

## 8. Break-even and option-value analysis

Do not hard-code a universal density at which sharing becomes optimal. For each service and matched area, vary population, demand frequency, land cost, facility size, catchment radius, opening window, travel impedance, cost and failure assumptions.

Identify:
- demand/density at which shared provision becomes less costly than private duplication under the #816 cost boundary;
- density/radius at which added travel and queue burden outweighs duplicated capital savings;
- utilization at which central provision requires a second node;
- whether a distributed node reduces tail access time or just duplicates low-use assets;
- whether external providers dominate private/shared investment at low utilization;
- how phasing and node redundancy change future feasible options.

Report the break-even surface and uncertainty, not one magic threshold. A shared node can be cheaper on average and still be rejected because it fails a declared travel, accessibility, privacy or service floor.

### Robust spatial provisioning and option value

Treat an amenity network as a **reconfigurable service topology**, not a once-built set of rooms. A central node, two smaller local nodes, a temporary pop-up, an external provider and a private fallback are different topology states. A design can preserve future options by reserving a route, service connection or convertible space without immediately building the full facility.

For each candidate, test:
- **robust reachability:** which resident tasks retain a usable service route when one node, route, provider or phase is lost;
- **reconfiguration cost:** how much land, equipment, staffing, retrofit work and time is required to add, split, relocate or repurpose a node;
- **option value:** whether a phased or modular provision strategy preserves the ability to respond to uncertain occupancy, service demand, climate, tariff and land-use changes;
- **common-cause exposure:** whether nominally distributed nodes share the same provider, route, utility, operator, ownership or supply dependency;
- **transition gaps:** how long residents lose service while a facility is built, moved, upgraded or transferred to a new operator.

Do not invent probabilities for the ten scenario families. Where defensible event frequencies do not exist, report a scenario envelope, worst-case access, and non-dominated alternatives. Use expected-value or CVaR summaries only when the probability model, sampling basis and time period are separately evidenced. Minimax-regret comparisons are permitted only against a declared objective or resident-authored preference model; do not create a universal regret score.

This is where distributed provision can create value beyond shorter average travel: it can preserve substitute service paths and maintain future design freedom. It can also be worse because multiple small facilities duplicate maintenance, operators and land. Both effects must be visible.

## 9. Non-compensatory gates

- No alternative passes when the required resident task has no known usable route or eligible service.
- A map marker is not available service; missing/stale opening, capacity, eligibility or price remains `UNKNOWN`.
- A capacity-potential measure is not an allocation and does not prove demand is served.
- No service capacity is double-counted across group-specific catchments.
- A direct line or Euclidean radius cannot override a barrier or missing step-free path.
- Unknown or stale upstream evidence cannot become zero cost, unlimited capacity or open access.
- Legal rights, public-health, building, fire, pool and regulated service approvals remain with their proper authorities and Mycelix/qualified owner records.
- A nominal cost or density advantage cannot compensate for displacement, inaccessible routes or a breached mandatory service floor.
- A candidate owner functional unit remains unapproved until Mycelix #6328 accepts it.

## 10. External baselines and fair comparison

### Routing and effective accessibility

CityMorph #555 is the existing Sol-Atlas access reference. Its current PR head, `6777acd894027671dbd5decf1ccb02aeda1564ac`, has a successful exact-head workflow: the 24-case synthetic corpus, 14 Rust tests, rustfmt, strict Clippy, locked dependency resolution and exact-head receipt steps all completed successfully on 9 November 2026. This qualifies only the synthetic access oracle and source-readiness contract; the Johannesburg source register remains a research inventory, not ingested GIS or transit data.

Compare CityMorph with the following:
- **Conveyal R5 / Conveyal Analysis:** many-to-many, schedule-aware transit accessibility and travel-time distributions by departure window. The published methodology assumes perfect service reliability, complete schedule knowledge and no crowding/accessibility boarding barriers in its baseline routing assumptions. It is a strong timetable-routing reference, not physical ground truth. https://docs.conveyal.com/analysis/methodology
- **OpenTripPlanner 2:** mature multimodal route planning using open data such as GTFS and OpenStreetMap, plus real-time updates/alerts where supplied. Compare exact feed/network vintages and query semantics. https://docs.opentripplanner.org/en/latest/
- **PEAI (Journal of Urban Mobility, June 2026):** a Perception-Aware Equitable Accessibility Index combining service availability, infrastructure quality, housing affordability and resident-centred time tolerances/weights. Its Vancouver example reports 99% within median cycling tolerances and 74.5% within walking thresholds, but only 37.9% in the framework's “truly accessible” category. This demonstrates why proximity-only results can mislead. Reproduce the published dataset, definitions and weights before any matched comparison; do not copy the composite as a universal Sol-Atlas score. https://doi.org/10.1016/j.urbmob.2026.100217
- **UN SDG 11.2.1:** convenient public-transport access uses walking-network distances (500 m to low-capacity transit and 1 km to high-capacity transit in the cited metadata), together with accessibility, frequency and safe/comfortable-stop criteria. Treat this as a transparent policy baseline, not a measure of all daily services. https://unstats.un.org/sdgs/metadata/files/Metadata-11-02-01.pdf
- **UN SDG 11.7.1:** public open-space share/access metadata. It measures a different dimension from private or shared amenity-service catchments; do not conflate public-space area with delivered amenity service. https://unhabitat.org/sites/default/files/2022/08/sdg_indicator_metadata-11.7.1.pdf

Sol-Atlas should not claim to beat R5/OTP at core routing. Its competitive advantage is the evidence layer above specialist routes: owner-qualified service capacity, service currentness, eligibility, affordability, step-free access, distributional outcomes, common-scope binding, privacy and abstention when evidence is missing.

A universal 15-minute cut-off is not a substitute for task-specific resident requirements, service hours or accessibility constraints.

### Housing / design tools

Autodesk Forma, Finch3D, TestFit and Hypar are relevant to site/layout option generation. They are adjacent comparators, not direct equivalents to Mycelix asset operations or CityMorph network accessibility. Compare them on identical site, design brief, unit mix, resident tasks, hard constraints, time budget and downstream information exchange. Do not claim superiority from option count or render speed.

### Construction and delivery tools

Singapore BCA PPVC, Hong Kong MiC/MiMEP and the 491-project ASCE modular-building study are relevant construction productivity references. They are not amenity operating savings or direct South African performance forecasts. Use equivalent installed/commissioned service and complete-workflow labor/cost boundaries.

## 11. Candidate stretch targets — not measured outcomes

| Target | Proposed benchmark objective | Qualification guardrail |
|---|---|---|
| Common scope | 100% of alternatives use identical scope digest, population, tasks, functional unit, horizon and evidence policy | Otherwise `NOT_COMPARABLE`. |
| Access hard gates | 100% of mandatory step-free / resident-declared access requirements either pass on owner evidence or remain `UNKNOWN/BLOCKED` | No inference from straight-line distance. |
| Evidence missingness | Zero missing/stale/wrong-scope route, service, capacity, price or opening inputs treated optimistically | Unknown never becomes zero or unlimited. |
| Capacity integrity | Zero group access potentials promoted to allocations; zero duplicate facility capacity | Allocation remains Mycelix-owned. |
| Tail accessibility | Candidate stretch: at least 20% lower P90 generalized access burden than the matched central-only baseline on cases where a distributed/hybrid option is feasible | No cohort may lose mandatory access; preserve cost/land/service Pareto trade-offs. |
| Provider resilience | 100% of declared single-node/provider failure cases expose the affected population, service and substitute path | Do not claim recovery without continuity evidence. |
| Resident preference | 100% of private/shared/hybrid alternatives remain opt-in, reversible design options until residents decide | No demographic inference, prestige upsell or automatic centralization. |
| Evidence portability | Every output binds exact upstream schema/revision, source status, scenario, scope digest and verifier version | An exact hash is identity, not truth. |

The 20% P90 objective is a stretch hypothesis to test, not a forecast. If a matched baseline shows another feasible topology is better, retain the measured Pareto result rather than force the target.

## 12. Execution sequence

1. **E0 — contract freeze:** spatial alternatives, 10 scenario IDs, amenity candidate units, owner map and 5×10 registry.
2. **E1 — registry integrity:** Rust auditor checks exact identity, 50 unique arm/scenario pairs, candidate functional-unit mapping, non-approved owner alignment, owner dependencies, source register and NOT_EXECUTED state.
3. **E2 — common-scope binding:** consume Sol-Atlas #623 and exact owner receipts from #555, Mycelix #6328, #816, Symtropy #3768 and other relevant systems. Payload bytes remain unverified unless fetched.
4. **E3 — matched spatial runs:** freeze a small set of synthetic neighborhood layouts and run through existing CityMorph #555. Compare central/distributed/private/hybrid/external provision with identical populations and service inputs.
5. **E4 — independent baselines:** compare effective access with specialist R5/OTP outputs and the published SDG/UN-Habitat indicator methods, with the same network and source inputs.
6. **E5 — local readiness:** only after rights-cleared population, street, transit, service and site data have been acquired and ingested can a Johannesburg analysis be attempted.
7. **E6 — physical pilot:** select one low-risk unoccupied amenity mock-up only after canonical asset/service contract, responsible operator, professional approvals and instrumented acceptance are present.
8. **E7 — operations and replication:** after at least 12 months of service/access/affordability/maintenance evidence, compare multiple sites, densities and governance forms.

## 13. Johannesburg / Highveld constraints

On 18 November 2026, Johannesburg Water announced Level 2 restrictions effective **21 November 2026**. Municipal drinking water may not be used to irrigate gardens, lawns, parks and public spaces. The City notice says the restrictions apply across residential, commercial and institutional consumers. The published Level 2 tariff schedule is stated to take effect separately from **1 December 2026**. A greenhouse, food garden or landscape alternative must declare lawful water source, quality and use; a stored, treated, greywater or rainwater supply is not automatically suitable for any given use. Source: https://joburg.org.za/media_/Newsroom/Pages/2026-News-Articles/City-implements-Level-2-water-restrictions-to-stabilise-supply.aspx

Do not use regional climate or soil priors as plot-level yield, microclimate or water-right evidence. For greenhouse/growing comparisons, consume Sol-Atlas #107/#105, Symtropy #3957/#4065/#4074 and Mycelix #5001. Keep planned harvest, forecast, observed lot and fulfilled supply as distinct states.

## 14. Claim ceiling

This is a protocol and registry, not an executed urban accessibility study. No source GIS, population grid, transit feed or facility-capacity data is claimed to be ingested by this PR. No design, legal, service, accessibility, cost, environmental or safety claim is qualified. The 50 cases remain NOT_EXECUTED. A valid manifest audit means only the registry is internally consistent.

## 15. Home, experience and food-network composition

Highveld Keep #607 remains the first reference dwelling; Home Expression Library #904 owns resident-directed expression and optional experience packages such as guest pavilion, workshop, conservatory/greenhouse, pool, music studio and library. This benchmark consumes those options and asks whether each selected task is private, accessory, shared, distributed or externally provided.

The greenhouse is a resident-facing productive amenity only when accepted crop/grade-specific harvest service and lawful water/site evidence exist. Symtropy #3957 owns whole-facility CEA resource/nutritional accounting; #4065 owns spatial climate/airflow; #4074 owns harvest-constrained control. Sol-Atlas #107 owns Highveld food/climate readiness; #105 owns the privacy-preserving food-network projection; Mycelix #5001 owns product lots and fulfilment. Do not duplicate those models or treat a predicted harvest as physical inventory.

The pool/wellness option is a spatial service comparison only. Public-pool operation remains out-of-scope or blocked unless applicable professional, operator, water-quality, public-health and statutory evidence is explicitly bound. A location map does not certify the pool.

Resident-authored requirement and preference remains distinct from engineering feasibility, legal rights, cost estimates and operator claims. Optional experience packages are not defaults or prestige upsells.



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