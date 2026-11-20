# Spatial Provisioning: SOTA Crosswalk and Qualification Protocol v0.1

**Programme:** Sol-Atlas Shared Amenity Commons #940 / draft PR #941  
**Executable synthetic child:** #952  
**Maturity:** research protocol; no field performance result

## 1. The field boundary

The relevant product is not a routing engine, site-plan renderer, or universal neighbourhood score. It is an **evidence-qualified service-topology comparison**:

> Given the same resident needs, population, functional unit, time period, geographic/network scope and resource boundary, which private, central, distributed, hybrid or external provision topology offers the strongest feasible service opportunity—and what happens to each resident group when evidence is missing or a dependency fails?

The system consumes specialist routing, amenity-service, capacity, affordability, legal and engineering evidence. It compares those outputs under common scope and exposes non-compensatory constraints. Crucially, route qualification (CityMorph) and period-specific service availability (Mycelix) are independent predicates: a resident can reach a facility that is closed, unstaffed, ineligible or otherwise unable to deliver the declared service. It must not silently replace an upstream authority or promote a planning option into a construction or allocation decision.

## 2. SOTA crosswalk

| Reference system / method | Established scope to respect | What Sol-Atlas should add without duplicating it |
|---|---|---|
| Conveyal R5 | Network isochrones, opportunity access, schedule-sensitive travel-time distributions and percentile analysis | Bind path/time outputs to Mycelix service-availability state for the declared functional unit and time window, plus owner-qualified capacity potential, eligibility, affordability, step-free access, resident tolerances and population-group outcomes |
| OpenTripPlanner 2 | Multimodal trip planning, transit timetable and accessibility profile handling | Preserve the distinction between accessible, inaccessible and unknown; bind access outputs to the actual amenity/service record and its operating state |
| PEAI (2026) | Multidimensional active-mobility accessibility, including service availability, infrastructure quality, housing affordability and user-centred insight | Reproduce the published case and method before adaptation; add independently qualified service/capacity and resilience evidence without assuming PEAI weights are universal |
| UN SDG 11.2.1 | Access to public transport using pedestrian-network distances and transport service classes | Use as a comparable public-transport indicator, not as a universal amenity-access guarantee |
| UN SDG 11.7.1 | Share of built-up area allocated to open public space for all, with a distinct public-space indicator boundary | Keep public-space amount separate from quality, legal public-use status, safe route and actual usable service |
| Autodesk Forma, Finch3D, TestFit, Hypar and manual design practice | Early site, massing, layout, feasibility and option generation | Compare downstream service outcomes and evidence completeness, not number or visual appeal of generated options |
| PPVC / MiC / MiMEP and modular-construction studies | Off-site manufacturing, assembly and construction productivity under defined project categories | Keep construction productivity in its own matched study; connect it to accepted dwelling/neighbourhood service-years only when all factory, transport, install, inspection, commissioning and rework boundaries are included |

### Sources and method notes

- Conveyal R5 documentation describes isochrones and opportunity access, and supports travel-time percentile distributions rather than only one mean. Its percentiles are useful as routing evidence, not as a full service-availability guarantee: https://docs.conveyal.com/analysis/methods
- OpenTripPlanner documents that wheelchair accessibility metadata in GTFS/NeTEx is often binary and incomplete, with configurable treatment of unknown accessibility and trip/stop accessibility: https://docs.opentripplanner.org/en/latest/RouteRequest/
- The 2026 PEAI paper is *Beyond time: A multidimensional framework for assessing accessibility via active transportation*, Journal of Urban Mobility 9 (June 2026): https://doi.org/10.1016/j.urbmob.2026.100217
- UN SDG 11.2.1 metadata defines pedestrian-network access to public transport and service-class thresholds; it is not a generic access rule for every amenity: https://unstats.un.org/sdgs/metadata/files/Metadata-11-02-01.pdf
- UN SDG 11.7.1 metadata concerns the share of built-up area allocated to open public space: https://unstats.un.org/sdgs/metadata/files/Metadata-11-07-01.pdf

The PEAI operationalizes five dimensions—proximity, availability, suitability, affordability and awareness. The paper's Vancouver example reports that conventional time-threshold coverage and its multidimensional "truly accessible" category diverge materially. That is a reason to reproduce the published method and assess dimensions separately—not to copy one city's weights or to claim a universal correction factor.

## 3. Temporal usability: travel arrival must meet service availability

The temporal-opportunity child [#956](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/956) adds a narrow join across two independent owner facts:

1. CityMorph #555's route/access result, including its qualified arrival-time interval.
2. Mycelix #6328's operational state and service-availability window for the exact functional unit.

The trip result is not recomputed. The amenity calendar is not inferred from a map point. The requested service duration must fit wholly inside a current operational window on the same declared scope and time basis.

This closes a specific false-positive path: someone can reach a workshop at 16:55, but a one-hour workshop service is not available when the facility closes at 17:00. A route that reaches a facility before close is not enough; the whole declared service interval must fit.

The join uses explicit half-open intervals `[start, end)`. For uncertain arrival-time results, every possible arrival in the declared range must fit before the output can be AVAILABLE. A mixture of fitting and non-fitting possibilities stays UNKNOWN; the evaluator does not assume a resident can choose the favourable arrival.

The calendar-completeness state in the synthetic adapter is an integration-envelope proposal, not an already-approved field in Mycelix #6329's current JSON Schema. It distinguishes a complete schedule with a known closed gap from an incomplete feed where the same missing interval must remain UNKNOWN. Mycelix must approve this assertion or supply an equivalent owner-native field before production integration.

### Fair industry comparison

- **Conveyal R5:** consumes departure-window travel-time distributions and time percentiles. Its documented default assumes perfectly reliable transit operations—on-time vehicles, full schedule information, and no crowding or boarding/accessibility constraints. We therefore use its travel-time outputs as a routing comparator and do not claim that the amenity-window join improves transit reliability or route calculation. https://docs.conveyal.com/analysis/methodology
- **OpenTripPlanner 2:** documents that wheelchair accessibility metadata in GTFS/NeTEx is often binary and incomplete, with explicit configuration for unknown trips, stops and elevators. We preserve upstream UNKNOWN and never promote it to accessible. https://docs.opentripplanner.org/en/latest/Accessibility/
- **PEAI (2026):** integrates proximity, availability, suitability, affordability and awareness. Its Vancouver application reports 99% cycling-time and 74.5% walking-threshold reachability, versus 37.9% under its multidimensional truly-accessible category. This makes it important to benchmark service availability separately from proximity. We should reproduce PEAI before adaptation and not claim this interval join replaces or outperforms the index. https://doi.org/10.1016/j.urbmob.2026.100217

The novel Luminous contribution is not a new router or PEAI clone. It is a verifiable join of specialist route evidence with canonical facility/service availability, exact functional unit, scope, source revision and time basis. That is a cross-owner correctness layer; it must be tested against the same route and calendar inputs as the incumbent workflow.

## 4. Fair comparison protocol

Every competitor and Luminous result must use the same:

1. Geographic boundary, network snapshot, population frame and resolution.
2. Origin points / population weights and resident-group definitions.
3. Departure times, timetable period, mode, routing profile, slope/crossing/step-free attributes and allowed unknown-data policy.
4. Destination/service definition, operating windows, capacity basis, eligibility, quality and affordability rules.
5. Functional unit and time horizon.
6. Exclusions, source vintage, rights, uncertainty, compute settings and software version.

Separate errors and differences by layer:
- **Network/routing difference:** path, generalized time, transfer/waiting and reliability.
- **Evidence/usability difference:** keep route qualification, Mycelix service availability for the declared period, eligibility, affordability, mandatory accessibility, capacity and resident tolerance as separate evidence states. A reachable but closed facility is not an effective service opportunity.
- **Capacity difference:** owner-qualified total capacity and demand; never multiply facility capacity by the number of groups who can reach it.
- **Distribution difference:** coverage and burden for each declared cohort, not only a weighted city mean.
- **Topology difference:** independent nodes, common-cause dependencies, substitution and phasing.
- **Governance/rights difference:** who owns, benefits, sets access rules, funds maintenance and can appeal or replace an operator.

When routing paths differ, attribute the divergence to the routing layer before claiming that the Luminous decision layer is better. R5/OTP should remain consumed comparators unless a separate routing benchmark justifies replacement.

## 4. Paradigm shifts worth pursuing

### A. Amenity topology as a service mesh

Treat amenities as a network of service nodes with explicit coverage, capacity, access windows and dependency domains. A facility is not "provided" merely because it appears on a plan. A resident's service opportunity is qualified only when the route/access result and the owner-native service record agree for the same scope and time.

### B. From access averages to resident-defined service floors

Make hard access constraints resident-declared and cohort-visible. A districtwide median cannot offset a resident group with zero step-free access. Any unresolved mandatory constraint stays UNKNOWN/BLOCKED. Do not set one universal travel threshold for every task or household.

### C. From shortest distance to dependable opportunity

Compare the best qualified options and their alternatives. A central node may have excellent routine access but one failure domain; distributed nodes can improve alternatives but may share an operator, power feed or bridge. Measure common-cause dependencies explicitly. Do not equate facility count with independent resilience.

### D. From static masterplan to reconfigurable service topology

Evaluate service topology by occupancy phase, facility opening/closure, population shift, route interruption and operator failure. The output should expose which groups lose access, which substitute remains, and what evidence is needed to reconfigure—not autonomously reassign residents.

### E. From one “best plan” to a rights-constrained Pareto set

Hard gates first; Pareto comparison second. Keep access, affordability, capacity, land opportunity cost, resources, privacy, distribution and lifecycle costs as separate axes. Residents and accountable institutions choose among feasible trade-offs. Never allow a weighted sustainability or cost score to compensate for a breached right or mandatory access requirement.

### F. From design option to auditable service promise

Translate design intent into testable, time-bounded service commitments: what service, for whom, where, during which window, with which minimum capacity and what fallback. The promise is only as strong as its owner evidence and its expiry/revocation rules.

## 5. Benchmark ladder

| Stage | Benchmark | Acceptance target | Evidence ceiling |
|---|---|---|---|
| E0 Contract | Same scope, population, service, functional unit, time and policy | 100% matched or `NOT_COMPARABLE` | Structural/synthetic |
| E1 Integrity | Unknown, stale, wrong-scope, duplicate and malformed evidence | Zero optimistic promotions; zero duplicate capacity | Software tests |
| E2 Routing cross-check | R5 and OTP on identical inputs and profiles | Publish error distributions and explain every material divergence; no predeclared pass tolerance before baseline | Software/replay |
| E3 Access reproduction | Reproduce PEAI published case and selected SDG 11 methods | Method-level reproduction before modifications | Published-data reproduction |
| E4 Service topology | Compare all five provision arms, separate service and access metrics | 100% independent-oracle agreement on frozen synthetic cases | Synthetic-only |
| E5 Distribution and resilience | Worst-served cohort, outage alternatives, common-cause domains, occupancy phases | Every declared mandatory floor passes or remains explicitly blocked; every failure scenario names affected groups and substitutes | Synthetic, then owner-qualified |
| E6 Real-world validation | Rights-cleared local data, held-out neighbourhood/time window, resident review and field observations | Preregistered method, external review, uncertainty and missingness reported | Local observational |
| E7 Operational proof | Instrumented pilot, maintenance/renewal, resident outcomes and audited costs | Repeatable performance across seasons and service periods | Physical/operational |

### Proposed metrics (never hide denominators)

- Population-weighted effective-access coverage.
- P50/P90 and worst-decile generalized access burden **among the effectively served population**, always alongside the coverage and unserved/unknown populations.
- No-access, unknown-access, known-service-unavailable and outside-tolerance population shares, separately.
- Mandatory step-free / resident-declared access gate.
- Reachable owner-qualified capacity potential, aggregate demand and shortfall, with once-only facility accounting.
- Minimum independent facility/operator/utility/route failure domains per group.
- Time to restore an acceptable service opportunity after a declared disruption.
- Phase coverage gap and temporary service debt before permanent amenities open.
- Household cost-of-place outputs from #816, not recalculated in #940.
- Privacy exposure and required disclosure, reported separately from efficiency.

## 6. Research targets, not claimed results

1. **Zero false positives** for a qualified service opportunity when the required route, owner service, time, eligibility, price or mandatory-access evidence is absent, stale or out-of-scope.
2. **100% exact-scope match** or `NOT_COMPARABLE`.
3. **100% independent-oracle agreement** on a frozen synthetic corpus.
4. **Zero duplicate capacity accounting** and zero capacity-potential-to-allocation promotion.
5. **Zero mandatory-right compensation:** no weighted score can erase a hard access failure.
6. **100% failure-case visibility:** the service, population groups, unavailable options, common-cause dependencies and known substitutes are exposed.
7. **20% candidate P90 burden reduction** versus a matched central-only topology where distributed/hybrid provision is feasible, with no regression in mandatory access for any declared group. This is a hypothesis to test, not an optimisation constraint or result claim.
8. **Robustness:** no option is labelled dominant if its advantage disappears under a declared, plausible input-uncertainty range; report Pareto stability/regret rather than hiding sensitivity in one score.

## 7. Next research questions

- What is the smallest additional owner evidence needed to distinguish “reachable” from “usable and available now”?
- Which topologies reduce correlated failure rather than merely adding nominal nodes?
- How does service access change between the first occupied dwelling and full build-out?
- Which amenities have high enough concurrency or low enough utilization to justify private, shared or external provision?
- What privacy-preserving aggregates let residents see service fairness without exposing individual movement or booking histories?
- Which service promises can be backed by enforceable ownership, maintenance reserves, operator replacement and handback rights?

## 8. Johannesburg empirical-data path: reuse CityMorph #561

The local-data owner is [CityMorph data readiness #561](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/561), with its existing machine-readable source register and data-readiness note on [draft PR #564](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/564). Do not create a second CoJ/GCRO/Stats SA source inventory inside #940.

Current source candidates and limits (checked 20 November 2026):

- **CoJ Corporate Geo-Informatics:** the public CGIS material says the Spatial Data Inventory records dataset description, format/type, date created, data owner/contact and usage limits. Different municipal entities may own the layers; direct custodian permission and reuse terms matter. The inventory link visible today is labelled 2024. Digital-data requests are directed to `cgisprojects@joburg.org.za`. See https://eservices.joburg.org.za/sites/cgismaps/Pages/default.aspx.
- **GCRO:** the Data Portal currently lists Quality of Life Survey, Gauteng boundaries, public infrastructure, land-cover/urban-footprint, demographics and biodiversity. Some items are directly downloadable; some require a landing page or registration. Rights remain dataset-specific. See https://www.gcro.ac.za/outputs/dataportal/.
- **Stats SA:** the Ward-level Small Area Population Estimates release (posted 20 November 2025) includes spreadsheets, methodology and ward shapefiles. Keep these as ward-level estimates, not household/building population truth. See https://www.statssa.gov.za/?p=18967.
- **WorldPop:** the currently visible South Africa 2025 population grid is modelled at 30 arc (approximately 1 km) and labelled alpha R2025A. Use only as a separately versioned sensitivity surface, never as official neighbourhood ground truth. Its product page states CC BY 4.0 for the dataset family and separate ODbL conditions for OSM/Microsoft-derived inputs. See https://hub.worldpop.org/geodata/summary?id=79327.
- **Transport gap:** an authoritative public Johannesburg GTFS feed was not confirmed in this check. Keep GTFS/AVL unknown or request-only until a publisher, service dates, feed integrity, update cadence and licence are verified. The existing #561 register also warns against using stale JRA roads as current routing truth.
- **Water constraint:** the City states that Level 2 restrictions commence 21 November 2026; municipal drinking-water irrigation of gardens, lawns, parks and public spaces is prohibited at all times. The separate Level 2 tariff start is 1 December 2026. A water-dependent amenity needs a dated lawful water source, intended-use and quality receipt; the restrictions do not automatically authorize all greywater or treated-effluent reuse. See https://joburg.org.za/media_/Newsroom/Pages/2026-News-Articles/City-implements-Level-2-water-restrictions-to-stabilise-supply.aspx.

No data were acquired, licensed, ingested or spatially validated by this research update. The sources above are candidates and constraints, not evidence of current amenity capacity or access.

## 9. Claim discipline

A successful synthetic evaluator means the implementation agrees with its frozen synthetic fixtures. It does not show that an arrangement is affordable, accessible, safe, legally valid, preferred by residents, sustainable or physically operable in Johannesburg. Real claims require rights-cleared source data, owner-native receipts, an independently reviewed method and field validation.
