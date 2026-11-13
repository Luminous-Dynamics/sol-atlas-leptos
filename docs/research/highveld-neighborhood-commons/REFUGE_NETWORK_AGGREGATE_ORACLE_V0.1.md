# Aggregate neighborhood refuge capacity and common-cause oracle v0.1

**Study ID:** HIGHVELD-REFUGE-NETWORK-001  
**Parent:** HIGHVELD-THERMAL-SERVICE-REFUGE-001 / HIGHVELD-REFUGE-AVAILABILITY-001  
**Maturity:** SYNTHETIC_AGGREGATE_ORACLE_NOT_FIELD_VALIDATED  
**Purpose:** determine whether multiple anonymous demand zones can be served by a shared refuge network without double-counting the same places, accessible capacity or protected water.

## 1. Why an aggregate oracle is necessary

A site-by-site eligibility filter is necessary but not sufficient. Three demand zones can each see the same refuge as having five free places; that does not create fifteen places. A map may appear covered while all routes converge on one building, its shared utility or one bridge/road cut. A network can have nominal aggregate capacity but no route from the demand zone that needs it.

This oracle takes anonymous zone-level demand groups and one shared inventory of sites. It never expands demand into named people, books places or routes occupants.

## 2. Resource accounting

For every refuge site:
- free total places = total capacity − current occupancy − total committed reserve;
- free accessible places = accessible capacity − accessible occupancy − accessible committed reserve;
- free safe water = available safe water − committed water − protected reserve;
- water-supported places = floor(free safe water / policy-supplied water per place for the required duration);
- serviceable total places = min(free total places, water-supported places);
- accessible serviceable places = min(free accessible places, serviceable total places).

The safe-water-per-place requirement is an input from the responsible authority/service plan, not a normative constant invented by this program. Negative/unknown water balance blocks the site.

Accessible places are a subset of total places, not an additive capacity. Accessible demand is matched first to accessible serviceable capacity using a maximum-flow witness. The remaining serviceable total capacity is then offered to general demand. This protects accessibility demand and prevents a site seat or litre from being consumed twice.

## 3. Network allocation witness

The oracle uses a deterministic bipartite max-flow calculation:
- source → anonymous demand-zone node;
- demand-zone node → site only when the zone/site route evidence is reachable, within the declared travel bound and, for accessible demand, accessibility-qualified;
- site → sink with its remaining qualified capacity.

It emits zone-to-site counts as a mathematical planning witness, never individual assignments. Accessible demand is routed first; general demand is routed over remaining capacity.

This is a conservative, deterministic witness, not a global multi-objective siting optimiser. If it finds a complete feasible witness, the aggregate capacities/routes are sufficient under the supplied inputs. If it finds a shortfall, the reported witness shortfall is not necessarily proof that no alternative routing exists; an independent exhaustive or optimisation oracle is required before making an infeasibility claim in a real deployment.

## 4. Common-cause disruption

Each site declares failure-domain memberships. The input case declares active failure domains. A site in an active domain is unavailable even when its local equipment is otherwise qualified. This makes shared grid, water-main, access-bridge, operator and communications cuts explicit.

Local fallback can remain eligible through a network partition only when its own directory, local operating state, route evidence, services, water and capacity remain qualified. It cannot inherit a remote site's capacity or evidence.

## 5. Synthetic known-answer case

The committed fixture contains two demand zones, three sites and one shared utility failure domain. In the no-failure case, the flow witness can serve all demand while preserving accessible capacity. When the shared domain fails, two larger sites are removed and the local-autonomy pod remains; the resulting unmet demand is reported rather than hidden by the original aggregate seat count.

All quantities and site/domain IDs in the fixture are synthetic. They are not site data or local refuge recommendations.

## 6. Metrics and acceptance

Report:
- total, accessible and general demand by anonymous zone;
- served and unmet totals by class and zone;
- free total/accessibility/water capacity per site;
- flow witness counts per zone/site;
- route travel bound and evidence references;
- active common-cause cuts and their affected sites;
- coverage fraction, worst-zone shortfall and capacity stranded by route constraints.

Acceptance targets:
- zero double-counted site capacity across zones;
- zero accessible demand served by non-accessible seats/routes;
- zero protected-water reserve usage;
- zero unavailable/unknown/failed sites included in flow;
- zero duplicated site or zone IDs accepted;
- zero route edges without source evidence;
- 100% flow conservation and independent max-flow oracle agreement;
- preserve positive control where a fully reachable and qualified supply can serve the group demand;
- correctly expose demand shortfall under seeded shared failure domains.

## 7. Comparison with current practice

Cooling-centre siting studies have used geospatial/multi-criteria coverage and walking-distance analyses; these help locate spatial gaps but do not necessarily prove real-time operating status, capacity after prior commitments, protected safe-water sufficiency or common-cause service continuity. A 2025 five-city study reports that only 32.2–50.7% of cooling centres are within walking distance of the most vulnerable populations and only 21.2–49.4% of populations with high energy burden have access in those study cities. These findings are context-specific, not Johannesburg measurements.

The 2026 Gauteng heat-action-plan scoping/benchmarking review identifies district thresholds, graded activation and cooling-camp models as potentially useful local response patterns. The World Bank's Johannesburg/Ekurhuleni urban-heat report recommends community cool spaces among broader low-cost-dwelling, public-space and warning interventions.

Highveld's differentiator is to join geospatial route evidence with live capacity, water entitlements, accessibility and common-cause failure—not replace municipal heat-action plans or decide who receives refuge.

Sources:
- [Where to cool off: a geospatial framework for placement of cooling centers (2025)](https://www.ornl.gov/publication/where-cool-geospatial-framework-placement-cooling-centers)
- [Temperature-Related Health Impacts: A Scoping Review and Benchmarking Exercise to Inform a Heat Action Plan (Gauteng, 2026)](https://pmc.ncbi.nlm.nih.gov/articles/PMC12829453/)
- [Urban heat in Johannesburg and Ekurhuleni: impacts and mitigation options (World Bank, 2026)](https://www.preventionweb.net/publication/documents-and-publications/urban-heat-johannesburg-and-ekurhuleni-impacts-and)

## 8. Claim ceiling

**Status:** SYNTHETIC_AGGREGATE_ORACLE_NOT_FIELD_VALIDATED. No real resident, refuge, route, capacity, water source, failure domain or allocation is admitted. Output is aggregate mathematical feasibility evidence only; no reservation, occupant assignment or emergency dispatch occurs.
