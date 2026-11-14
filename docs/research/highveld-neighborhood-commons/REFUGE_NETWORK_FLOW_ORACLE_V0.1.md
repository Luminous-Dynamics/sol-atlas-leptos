# Highveld refuge network capacity and equitable flow oracle v0.1

**Study ID:** HIGHVELD-REFUGE-NETWORK-FLOW-001  
**Parent:** HIGHVELD-THERMAL-SERVICE-REFUGE-001 / HIGHVELD-REFUGE-AVAILABILITY-001  
**Maturity:** SYNTHETIC_AGGREGATE_FLOW_ORACLE_NOT_FIELD_VALIDATED  
**Purpose:** measure whether multiple anonymous demand areas can collectively reach enough qualified refuge capacity without double-counting places, accessible capacity or safe water.

## 1. Why per-site eligibility is not enough

A refuge may individually appear to have free capacity, while multiple demand areas all depend on those same places. Counting each site's capacity once per demand area creates fictional supply. The network benchmark therefore computes a capacitated bipartite flow between anonymous demand areas and qualified refuge sites.

The output is aggregate place counts and unmet demand by category. It contains no named occupants, personal route history, booking or site assignment. The flow is a planning oracle, not an evacuation/dispatch system.

## 2. Lexicographic allocation policy

The synthetic v0.1 policy is explicit:

1. **Accessible-place demand first.** Maximize served accessible-place demand over reachable and accessibility-qualified routes, using only accessible refuge capacity.
2. **General-place demand second.** Use remaining safe-water budget and non-accessible-place capacity for general demand.
3. **No cross-class substitution by default.** General places are not counted toward accessible demand. Unused accessible places are not silently reclassified as general capacity. A later policy may permit release only through an owner-authorized, time-bounded rule.
4. **No water reserve borrowing.** Site water is converted into a maximum place capacity using the common declared litres/person for the required period. Existing commitments and protected reserves have already been deducted by the availability evidence owner; no reserve can be spent.
5. **Deterministic maximum flow.** Demand areas and sites are sorted by stable IDs. Routes are admitted only when their state, travel-time limit and evidence references qualify.

This is a lexicographic policy, not a universal moral allocation rule. Real deployment requires public/asset-owner governance and appeal.

## 3. Frozen input contract

The case contains:
- anonymous demand-area IDs with total and accessible place counts;
- site IDs and prequalified remaining total/accessibility capacity and safe-water litres;
- one common safe-water litres-per-person requirement for the evaluated time window;
- per-area route edges to candidate sites, with route state, accessibility state, travel time and evidence references;
- site availability, capacity and water evidence references;
- a maximum travel-time rule.

The uniform water-per-person quantity is a v0.1 simplifying assumption. The input rejects duplicate demand IDs, duplicate site IDs and duplicate area-to-site edges. It does not assume the same route is usable for all access profiles.

## 4. Flow and accounting

For each class, the oracle builds a bipartite capacitated graph:

\[
\text{source}\rightarrow\text{demand area}\rightarrow\text{qualified site}\rightarrow\text{sink}
\]

The accessible pass is bounded by each site's accessible free places and its water-derived place limit. Its actual output is deducted from site water before the general pass. The general pass is bounded by the site's non-accessible free places and remaining water-derived place limit.

Per site:

\[
C_{nonaccess,free}=C_{total,free}-C_{accessible,free}
\]

\[
C_{water,places}=\left\lfloor\frac{W_{free}}{L_{safe/person}}\right\rfloor
\]

The flow on an area→site edge is an aggregate count only. Every site capacity edge exists once in the network, so two demand areas cannot claim the same place twice.

## 5. Output metrics

- requested / served / unmet accessible places by demand area;
- requested / served / unmet general places by demand area;
- total served and unmet places;
- accessible and general coverage ratios;
- per-site accessible/general use and residual water;
- excluded route/site evidence counts and reasons;
- exact demand, route, capacity, water and oracle input references;
- deterministic ordering and oracle revision.

Coverage is reported by demand area. A high total coverage rate cannot hide an area with no reachable refuge. Do not compress all area-level gaps into a single weighted score.

## 6. Positive and negative controls

The registered synthetic control has two anonymous demand areas and two sites. Both areas can use the central hub, but only one can use the distributed pod within the route-time bound. The network contains 14 total free places, 4 accessible places and enough water for 14 places, against 15 total demand. The oracle must serve all four accessible requests and no more than ten of eleven general requests.

Negative controls include duplicate site/demand/route IDs, unknown site references, route blocked/unknown/over-time, accessibility not qualified, capacity below demand, protected water reserve deducted, and missing evidence references. A site with enough total capacity but no accessible free places must not satisfy accessible demand.

## 7. Acceptance targets

- zero double-counted seats across demand areas;
- zero accessible demand served by general capacity;
- zero water allocation beyond qualified remaining litres;
- zero route edge accepted when blocked, unknown or beyond the declared travel-time bound;
- zero missing route/capacity/water evidence accepted;
- zero duplicate identities accepted;
- 100% deterministic agreement with the finite max-flow oracle;
- positive controls preserved and every unserved aggregate count reported.

These are proposed benchmark targets. No field performance or equity outcome is implied.

## 8. Industry comparison

Recent urban-heat network work applies circuit/network analysis to heat-risk source and cooling-buffer sink relationships, identifies critical corridors and nodes, and tests robustness to attack/random disruption. Shelter-location research also evaluates time-sensitive accessibility, operating hours and typology. Highveld's narrow contribution is a reproducible *capacity and resource-constrained flow oracle* with explicit accessible-place and water budgets, coupled to the existing evidence-gated site availability filter.

This does not replace Sol-Atlas route calculation, Symtropy event chronology, Mycelix resource commitments, or local public authority decisions. It turns their evidence into an independently checkable aggregate service-coverage result.

**Status:** SYNTHETIC_AGGREGATE_FLOW_ORACLE_NOT_FIELD_VALIDATED. The fixture is fictional. No real demand areas, routes, sites, capacity, water or resident data are admitted.