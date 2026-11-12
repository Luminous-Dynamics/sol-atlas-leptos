# Highveld multi-zone refuge capacity feasibility benchmark v0.1

**Study ID:** HIGHVELD-REFUGE-NETWORK-FEASIBILITY-001  
**Parent:** HIGHVELD-THERMAL-SERVICE-REFUGE-001 / HIGHVELD-REFUGE-AVAILABILITY-001  
**Maturity:** SYNTHETIC_AGGREGATE_FLOW_ORACLE_NOT_FIELD_VALIDATED  
**Purpose:** test whether multiple anonymous demand areas can be served by a shared refuge network without double-counting capacity, accessible places or protected safe-water reserves.

## 1. Why site eligibility is not network service

A single-site availability filter can correctly say that several refuges are eligible. It cannot establish that the same free seats and safe water can serve several demand areas simultaneously. Counting every eligible site independently can overstate service when route choices overlap and multiple cohorts compete for the same capacity.

This benchmark therefore adds an aggregate network-feasibility layer:
- demand cohorts are anonymous spatial/service groups, not identified residents;
- route reachability and travel-time evidence are supplied by Sol-Atlas;
- site/service/capacity/water evidence is supplied by the responsible owners;
- a max-flow oracle checks how much aggregate demand can be served by the network;
- no named assignment, booking, dispatch or route computation is produced.

## 2. Shared resource and accessibility model

Each scenario declares a common safe-water planning basis in litres per place for the required operating duration. This is a benchmark assumption supplied by the competent service authority; it is not a universal potable-water standard.

For each qualified refuge site:

\[
C_{free}=C_{total}-O_{current}-R_{committed}
\]

\[
C_{accessible,free}=C_{accessible}-O_{accessible}-R_{accessible,committed}
\]

\[
W_{free}=W_{available}-W_{committed}-W_{protected,reserve}
\]

\[
C_{water}=\left\lfloor\frac{W_{free}}{L_{water/place}}\right\rfloor
\]

\[
C_{effective}=\min(C_{free},C_{water})
\]

\[
C_{accessible,effective}=\min(C_{accessible,free},C_{effective})
\]

Unknown/failed site service, water rights/quality, operating window, route/accessibility or evidence means the site/edge is unavailable for the affected case. The method never treats unknown water as zero use or reserve as free capacity.

## 3. Aggregate max-flow oracle

The synthetic reference uses a declared two-stage feasibility objective:

1. **Accessible demand first:** maximum flow from each anonymous cohort's accessible-place demand to sites reachable through qualified accessible routes, bounded by each site's effective accessible capacity.
2. **Remaining general demand second:** maximum flow for the remainder of each cohort's total demand through reachable routes, bounded by the site's effective total capacity remaining after the first stage.

This is a benchmark policy to preserve the accessible-place floor, not a moral allocation rule or operational booking recommendation. Future variants should compare other declared policy objectives, including max-min cohort service ratios and lexicographic protections, under owner-approved governance.

The oracle returns:
- requested and feasible accessible places;
- requested and feasible general places;
- total feasible demand and unmet demand;
- per-cohort aggregate service counts and service ratios;
- minimum cohort service ratio;
- eligible-site and qualified-route counts.

It does **not** return a cohort-to-site assignment, identifiable route, named occupant, booking or actuation. A maximum-flow feasibility result is an upper bound under the frozen graph and capacity assumptions, not a promise that live operations can realize the flow.

## 4. Matched scenarios and negative controls

The synthetic known-answer case includes three anonymous demand cohorts and:
- a local site with a qualified accessible route;
- a second site with a lower water-limited capacity than its nominal seat count;
- a site excluded because thermal service is unknown;
- route edges with different travel-time and accessibility characteristics.

Required negative controls include:
- accessible route withdrawn or unknown;
- route blocked or over the cohort's declared travel-time bound;
- overlapping cohorts competing for the same site capacity;
- site water sufficient for seats nominally but insufficient after commitments/reserve;
- capacity accounting that violates total/accessibility bounds;
- thermal/ventilation/site closure or rights/quality unknown;
- network partition without qualified local directory;
- duplicate cohort, site or route identities.

Positive controls must remain feasible when a qualified local directory and route/site evidence remain available during network partition.

## 5. Benchmark metrics

Report:
- total/accessible/general demand and feasible counts;
- total and accessible unmet demand;
- per-cohort service ratio and minimum cohort service ratio;
- route availability by cohort and access class;
- total free, accessible free and water-limited effective site capacity;
- sites excluded by each dependency reason;
- sensitivity to water basis and route travel-time policy;
- solver time and graph size (when benchmarking runtime);
- held-out uncertainty only when input capacities/routes themselves carry qualified uncertainty.

Do not optimize only total served count: a network can serve many people while leaving one cohort with no accessible option. Report worst-cohort service and accessible unmet demand alongside total flow. No weighted average may compensate for a protected access or water-floor breach.

## 6. State-of-the-art comparison and boundaries

- **Sol-Atlas:** source of route graph, spatial/accessibility constraints and route provenance.
- **Mycelix:** source of rights/quality/reserve and capacity commitment evidence.
- **Symtropy:** source of event time, route/service availability chronology and replay.
- **Highveld oracle:** aggregate capacity-feasibility check across the declared graph; no route engine, live reservation system or human allocation decision.

The contribution is to test cross-site shared constraints and unmet service at demand-cohort level, not to claim a new state-of-the-art routing or evacuation optimizer. Compare any future operational policy with conventional nearest-open-refuge, capacity-aware assignment and accessibility-constrained assignment on the same graph and demand trace. Preserve the same route times, capacity and water facts across all baselines.

## 7. Claim ceiling

**Status:** SYNTHETIC_AGGREGATE_FLOW_ORACLE_NOT_FIELD_VALIDATED. The input uses synthetic demand, routes, sites and capacities. No actual refuge, resident, route, water allocation, booking, dispatch or emergency recommendation is produced.
