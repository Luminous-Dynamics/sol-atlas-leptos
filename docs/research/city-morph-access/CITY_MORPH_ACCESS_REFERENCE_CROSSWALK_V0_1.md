# CityMorph industry and research crosswalk v0.1

This is a method comparison and benchmark plan, not a claim that CityMorph has implemented or outperformed every comparator.

## Capability comparison

| System / method | Established strength | Important boundary | CityMorph benchmark / complement |
|---|---|---|---|
| Conveyal R5 / Analysis | Schedule-aware multimodal routing, many-to-many travel times, cumulative-opportunity accessibility and departure-window variation | Routing results depend on timetable/network assumptions; documented Conveyal methodology assumes on-time vehicles, schedule knowledge, unimpeded boarding and perfect reliability in its baseline model | Treat R5 as a routing reference. Match graph, GTFS, origins, destination opportunities, modes, times and impedance; then add service capacity, opening, price/eligibility, universal-access and source-confidence checks |
| r5r | R5 interface for accessibility and travel-time matrices; minute-level departure sampling; median/percentile travel times; step, exponential, fixed-exponential, linear and logistic decay | Accessibility output remains dependent on chosen decay function, cutoff, user tolerances, data coverage and assumptions | Reproduce step and gravity/decay variants on the same inputs; expose parameters and sensitivity rather than selecting hidden weights |
| OpenTripPlanner 2 | Production multimodal trip planning and journey routing | Trip-planning parity does not itself qualify service capacity, affordability or urban-form trade-offs | Compare matched door-to-door paths and modes where timetable data exist; do not rebuild specialist routing without a demonstrated gap |
| ArcGIS Urban | Scenario-based urban plans, zoning/land-use metrics, shadow analysis, green-space and site suitability; suitability model uses reclassified criteria and weighted-sum scores | Scores depend on criterion scaling and chosen weights; suitability and scenario dashboards do not establish actual service usability or resident outcomes | Export equivalent scenario indicators; preserve raw criteria/units, disclose weights and provide a no-composite Pareto view alongside any user-requested weighted view |
| UN SDG 11.2.1 | Standardized indicator for convenient access to public transport, disaggregated by sex, age and disability where data permit | A stop-distance/coverage indicator alone is not a full trip, affordability, reliability or service-capacity assessment | Reproduce the official indicator exactly, document where CityMorph extends it and avoid silently changing the definition |
| UN SDG 11.7.1 | Built-up-area share that is open space for public use, with disaggregation and quality/ accessibility details where data permit | Area share does not prove safe, inclusive or usable access to each space | Calculate the official base indicator, then separately report network accessibility, public-use verification, quality and universal access |
| PEAI (June 2026) | Combines service availability, infrastructure quality, housing affordability and user-centred insight for active mobility | Its weights/tolerances and study context are model-specific; a local adaptation is not an exact reproduction | Reproduce its published case first; map each construct and source; then conduct a separate Johannesburg validation rather than copying its weights |
| Conventional 15-minute city | Accessible public communication and a simple baseline for proximity/time | Distance/time thresholds omit capacity, operating hours, affordability, eligibility, infrastructure conditions, disability needs and resident preference | Keep as a transparent baseline, not the target or an all-purpose city score |

## Core paradigm shift

**From map proximity to evidence-qualified opportunity.**

Each service opportunity must carry distinct evidence for:
1. spatial existence and location;
2. network reachability by mode and relevant time;
3. service availability at the time needed;
4. capacity relative to demand;
5. affordability and eligibility;
6. universal accessibility / step-free constraints;
7. quality and user experience where evidence exists;
8. source authority, date, precision, completeness and uncertainty.

A point on a map proves only that a point was mapped. A route proves only a path under the graph and impedance assumptions. A reachable service with insufficient capacity is not an adequate opportunity. Unknown evidence must never become a favorable default.

## Evaluation hierarchy

### Tier 0 — finite correctness
- All frozen synthetic cases match an independently implemented oracle.
- 100% correct known-route results on small reference graphs.
- Zero favorable access when a required input is stale, missing, revoked, inaccessible or unit-incompatible.
- Zero duplicate facility capacity / group-demand accounting.
- Permutation-invariant normalized results.

### Tier 1 — published-method reproduction
- Exact UN SDG 11.2.1 and 11.7.1 calculations on documented known-answer fixtures.
- PEAI reproduction on its published, rights-cleared case with source data, exact cutoff definitions, tolerances and weights.
- Reproduce R5/r5r travel-time percentiles and cumulative-opportunity calculations on the same graph / GTFS bundle.
- Report every divergence and its cause, not only a correlation coefficient.

### Tier 2 — local validity
- Validate population, service locations and route network against independent local/official evidence.
- Compare modelled accessibility with appropriately designed, consent-based field audits and resident surveys.
- Preserve sampling uncertainty and avoid person-level mobility surveillance.
- Evaluate by neighbourhood, mode, time of day and population group.

### Tier 3 — decision utility
- Compare urban alternatives with identical service floors, populations, constraints and budget/finance assumptions.
- Show Pareto-efficient options and dominated alternatives.
- Sensitivity-test population growth, service capacity, housing cost, network interruptions, timetable reliability and uncertainty.
- Measure transition costs and service gaps by construction phase.

### Tier 4 — performance
Only after semantic correctness is established:
- 10k / 100k / 1M origin and opportunity stress profiles, with declared hardware and dataset sizes;
- cold/warm data loads, graph-build time, query throughput, memory and incremental update cost;
- deterministic replay and exact corpus/source digests;
- no throughput target may weaken abstention, source-currentness or evidence gates.

## Metrics that matter more than a blended score

| Dimension | Primary measures |
|---|---|
| Network | path existence, mode-specific cost, detour ratio, barrier sensitivity, travel-time percentiles |
| Opportunity | cumulative reachable opportunities, gravity-decay opportunity, known usable service capacity |
| Adequacy | supply/demand shortfall, group-level unmet demand potential, capacity concentration |
| Equity | access distributions, p50/p90/worst-decile, between-group gap, population share without known adequate access |
| Affordability | service fare/price plus housing-cost burden and generalized journey cost, kept separate |
| Universal access | step-free route and facility coverage, accessible crossing / stop evidence, unknown share |
| Reliability | sensitivity to departure time, service outages, route closures, timetable freshness |
| Evidence | source coverage, freshness, licence state, uncertainty, false-pass rate, abstention behaviour |
| Transition | service interruption duration, phase-to-phase access, construction burden and recovery time |

No single metric substitutes for the others. Any weighted score must publish its normalization, weights, missing-data treatment, sensitivity and distributional consequences; the default report is a vector and Pareto set.

## References

- Conveyal R5: https://github.com/conveyal/r5
- Conveyal methodology: https://docs.conveyal.com/analysis/methodology
- r5r accessibility API: https://ipea.github.io/r5r/reference/accessibility.html
- r5r accessibility guide: https://ipea.github.io/r5r/articles/accessibility.html
- OpenTripPlanner 2 documentation: https://docs.opentripplanner.org/
- ArcGIS Urban analysis: https://doc.esri.com/en/arcgis-urban/latest/get-started/get-started-analysis.html
- ArcGIS Urban suitability: https://doc.esri.com/en/arcgis-urban/latest/help/help-suitability.html
- ArcGIS Urban scenarios: https://doc.esri.com/en/arcgis-urban/latest/help/help-scenarios.html
- UN Goal 11 and indicators: https://sdgs.un.org/goals/goal11
- UN-Habitat 11.2.1 metadata: https://unstats.un.org/sdgs/metadata/files/Metadata-11-02-01.pdf
- UN-Habitat 11.7.1 metadata: https://unhabitat.org/sites/default/files/2022/08/sdg_indicator_metadata-11.7.1.pdf
- PEAI: https://doi.org/10.1016/j.urbmob.2026.100217

## Claim ceiling

CityMorph should not claim SOTA superiority from a synthetic corpus or an interface feature comparison. A defensible claim requires matched inputs, published reference implementations, exact method reproduction, independent validation and clear uncertainty. The planned differentiator is a more complete evidence/decision layer, not a blanket claim that one engine routes better than R5 or OTP.


## Johannesburg Universal Design Access Plan (UDAP) — local validation reference (2026-11-09)

JDA announced completion of the City's Universal Design Access Plan on 9 June 2026. The public description says the plan uses facility audits, surveys, stakeholder engagement and capacity building; covers Rea Vaya, Metrobus and other operators; and assesses the entire passenger journey from trip planning and pedestrian access to boarding, vehicles, transfers, information and destination access. The City's August 2026 explanation identifies continuing issues in pedestrian links, passenger information, facility consistency, vehicle/platform interfaces, maintenance, staff training and intermodal integration.

**What this contributes:** a locally authored audit and intervention framework for expanding CityMorph's universal-access evidence dimensions beyond the graph's step-free boolean. It is especially relevant to station/stop approach, crossings, wayfinding, ticketing/fare interfaces, vehicle/platform gaps, onboard access, transfer and information-system accessibility.

**What it does not yet provide to this repository:** the full plan document, facility-level audit tables, the risk register, machine-readable station/vehicle identifiers, audit scores, survey instruments, sample coverage, dated operator/service revisions, funded actions or reuse terms. The source register records only the public completion announcement as VerifiedPublic and leaves rights Unknown. No station or route is treated as accessible because it appears in the plan.

**Next benchmark increment:** when JDA releases the full outputs, map each travel-chain audit field to an owner-native CityMorph constraint and test missing/stale/unknown facility evidence. Join each audited facility to the network graph by stable identifier and position, with explicit match uncertainty. Preserve the distinction between a reported barrier, a planned intervention, an implemented intervention and an independently re-audited barrier removal.

Sources:
- JDA completion announcement: https://www.jda.org.za/jda-completes-landmark-universal-design-access-plan-to-advance-inclusive-public-transport-in-johannesburg/
- City/JDA explanation of travel-chain coverage and unresolved barriers: https://joburg.org.za/media_/Newsroom/Pages/2026-News-Articles/JDAs-plan-puts-people-at-the-heart-of-Joburgs-public-transport.aspx
- City completion summary: https://joburg.org.za/media_/Newsroom/Pages/2026-News-Articles/JDA-completes-universal-design-plan-for-public-transport.aspx

## Repository ownership correction (2026-11-06)

A repository-wide audit found existing robust multi-objective owners:
- Sol-Atlas #131 owns site-selection Pareto, sensitivity and value-of-information.
- Symtropy #1874 / RDO-002 owns the base robust frontier and dominance certificates.
- Symtropy #2529 defines guaranteed interval dominance for mixed minimize/maximize objectives, with uncertainty semantics kept explicit (first implementation admits deterministic envelopes only).
- Sol-Atlas #214 owns staged portfolio selection under budgets and dependencies.

CityMorph will not implement another Pareto solver. Urban alternatives will be mapped to the owner-native interval-frontier contract with scope, metric, hard-floor and uncertainty-semantic bindings. Statistical confidence intervals, vendor ranges and scenario quantiles must not be relabelled deterministic envelopes.
