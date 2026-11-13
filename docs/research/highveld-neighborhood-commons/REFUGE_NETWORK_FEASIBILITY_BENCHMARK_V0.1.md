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


## 7. Discrete max-min fairness comparator

Maximum flow establishes a throughput upper bound, but it does not specify how service should be distributed among cohorts. A feasible maximum-flow result may leave one area with a very low service ratio while another reaches 100%, even where the same total count can be distributed with a better worst-cohort outcome.

The qualifier therefore reports a second, policy-comparison result:

1. Keep the accessibility-protected stage. Accessible-place demand is first served through qualified accessible routes and bounded by the same effective accessible site capacity.
2. For remaining general demand, use **discrete progressive filling**. At each step, consider the cohort with the lowest current total coverage ratio:
   \[
   \rho_g=\frac{A_g+G_g}{D_g}
   \]
   where \(A_g\) is feasible accessible service, \(G_g\) is general service already protected by the fairness quota, and \(D_g\) is total cohort demand.
3. Tentatively add one general place to that cohort. Keep the increment only when a fresh max-flow feasibility check can still satisfy every cohort's current general-service quota simultaneously.
4. A quota that cannot be increased is marked blocked; the process moves to the next lowest service ratio. Because quotas only increase, a failed increment cannot become feasible later as more capacity is consumed.
5. Continue until no cohort can receive another place while preserving all accepted quotas.

The synthetic fixture requires an explicit, unique fairness_tie_break_rank per anonymous cohort. This rank is used **only** when two discrete service ratios are exactly equal. It is visible in the output and must be owner-governed; it is not a moral priority or an identity-derived score. A production temporal policy should test rotating ties and cumulative service debt so the same zone is not repeatedly disadvantaged by a static tie-break.

The oracle reports both the existing accessible-first maximum-throughput result and the max-min comparator side by side:
- total places feasible under each policy;
- minimum and maximum cohort service ratios;
- cohort ratio spread;
- per-cohort accessible/general/total counts;
- explicit tie-break policy;
- no site assignment, reservation or occupant identity.

The benchmark objective is to improve the minimum cohort service ratio **without reducing total feasible throughput**, while preserving the accessible-place floor. Where that cannot be achieved under the same graph and capacity, report the trade-off rather than hiding it in a weighted score. This is a deterministic planning comparator, not an operational allocation policy.

### Known-answer policy comparison target

The synthetic network contains three anonymous cohorts sharing two qualified sites. Under the existing maximum-throughput flow, the service ratios are determined by max-flow traversal rather than a fairness objective. The progressive-fill comparator must:
- preserve the same maximum total feasible places for this fixture;
- strictly improve the minimum cohort service ratio;
- retain all accessible-route, site-eligibility and protected-water constraints;
- return no named or cohort-to-site assignment.

The unit-step algorithm is intentionally a v0.1 reference for small aggregate benchmark instances. It is not yet a large-network optimizer; runtime and graph-size scaling must be benchmarked separately before any larger deployment claim.

## 8. State-of-the-art comparison: throughput and fairness are different axes

Compare future policies on the same frozen demand cohorts, qualified route graph, water-limited site capacities and outage trace:
- nearest-open-refuge baseline;
- capacity-aware maximum-flow feasibility;
- accessible-first maximum-throughput policy;
- discrete max-min cohort service policy;
- any owner-approved lexicographic or rotating service-debt policy.

Report throughput, worst-cohort service, accessible unmet demand, unmet-place-hours, water/seat utilization, route limits and policy runtime separately. Never call one policy "fair" solely because it maximizes a mathematical ratio: the cohort definition, protected classes, tie-break, cumulative history and legitimacy of the governance process all require explicit owner review.
