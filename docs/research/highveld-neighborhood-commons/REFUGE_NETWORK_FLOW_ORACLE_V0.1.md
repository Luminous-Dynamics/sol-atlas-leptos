# Highveld refuge network max-throughput reference oracle v0.1

**Study ID:** HIGHVELD-REFUGE-NETWORK-FLOW-001  
**Parent:** HIGHVELD-THERMAL-SERVICE-REFUGE-001 / HIGHVELD-REFUGE-AVAILABILITY-001  
**Maturity:** SYNTHETIC_AGGREGATE_REFERENCE_COMPARATOR_NOT_FIELD_VALIDATED  
**Purpose:** provide a small, independent Edmonds–Karp max-flow reference for a simple accessible-first, general-second refuge capacity benchmark. This is a comparator, not the primary network policy engine.

## 1. Scope relative to the existing refuge stack

The repository already has three more specialized pieces:

- highveld-refuge-network-feasibility is the primary multi-zone engine. It evaluates site and route state, freshness, shared water/capacity, progressive max-min fairness across anonymous cohorts, and an ordered supply/demand degradation sequence with unmet place-hours and recovery/degradation metrics.
- highveld-refuge-capacity-oracle is a bounded exact backtracking feasibility oracle for a complete group-level witness. It has a declared envelope and reports search-limit exhaustion as indeterminate.
- highveld-refuge-shared-capacity-audit checks a proposed aggregate plan against one shared site ledger; it is an audit, not an optimizer.

This binary is deliberately smaller. It provides a conventional deterministic maximum-throughput reference against which fairness-aware policies can be compared. It does not replace the primary network engine, the bounded exact witness oracle or the proposed-plan audit. The comparison must use the same frozen sites, demand, route graph and service evidence; the current fixture is a synthetic unit case, not yet a cross-binary parity test.

## 2. Why retain a simple max-throughput reference

A fairness-aware service policy and a pure maximum-throughput flow answer different questions. The maximum-throughput reference asks: what is the greatest aggregate number of places that can be served under this one frozen graph and set of capacity limits? It may leave a worse area-level distribution than progressive max-min service. That makes it useful as a baseline, not as a morally preferred allocation policy.

The v0.1 reference policy is:

1. **Accessible-place flow first.** Accessible demand can use only sites with a reachable, evidence-backed route and a qualified accessible route, bounded by each site's accessible free capacity.
2. **General-place flow second.** Remaining general demand can use reachable sites' non-accessible free capacity, bounded by safe water left after the accessible pass.
3. **No cross-class substitution.** General places do not satisfy accessible demand; unused accessible places are not reclassified as general in v0.1.
4. **One site edge per capacity pool.** A site appears once in each flow graph, so demand areas cannot count the same seats twice.
5. **Deterministic graph order.** Areas and sites are sorted by stable IDs before the max-flow graph is constructed.

## 3. Input and finite envelope

The input contains:
- anonymous area IDs and total/accessible place requests;
- candidate site IDs, eligibility state and free total/accessibility/water capacity;
- a shared water quantity in litres per place for the specified time window;
- route reachability and travel-time evidence for each area/site edge;
- separate accessibility evidence for accessible demand;
- source evidence references and an independent-oracle reference.

The implementation envelope is at most 64 demand areas, 64 refuge sites and 1,000,000 aggregate place requests. These are implementation bounds, not claims about community scale. Exceeding them is an input error, not an invitation to return a partial answer.

## 4. Network and accounting

For each class, the oracle builds:

\[
source \rightarrow demand\ area \rightarrow qualified\ refuge \rightarrow sink
\]

The site's capacity edge exists exactly once per pass. Accessible flow is subtracted from that site's water before the general pass.

For each site:

\[
C_{nonaccess,free}=C_{total,free}-C_{accessible,free}
\]

\[
C_{water,places}=\left\lfloor\frac{W_{free}}{L_{safe/place}}\right\rfloor
\]

The accessible pass is bounded by accessible free places and the water-derived limit. The general pass is bounded by non-accessible free places and the remaining water-derived limit.

Route and accessibility exclusions are reported separately. A reachable but inaccessible route can still be considered for general demand, but cannot be counted for accessible demand. Straight-line distance is never used as a substitute for route evidence.

## 5. Synthetic known answer

Two demand areas request 15 places total, including four accessible places. Two sites offer 14 total free places, four accessible places and sufficient water for all seats. The shared central hub is the only route option for one general-demand segment; another demand area has a route to a distributed pod that exceeds its declared travel-time bound and fails accessibility qualification.

The registered result is:
- accessible requested: 4; served: 4; unmet: 0;
- general requested: 11; served: 10; unmet: 1;
- total requested: 15; served: 14; unmet: 1.

These are synthetic expected values, not actual refuge locations/capacities or service observations.

## 6. Relationship to state-of-the-art and benchmark use

Recent urban heat network research models heat-risk sources and cooling-buffer sinks, identifies critical nodes/corridors and tests robustness against attack/random disruption. Recent refuge-location research considers time-sensitive accessibility, operating hours and shelter types. Static accessibility baselines such as 2SFCA answer a valuable spatial coverage question but do not by themselves prove time-specific shared capacity or water service.

Use the stack together:
- static 2SFCA / route reachability: spatial coverage baseline;
- this max-throughput oracle: simple aggregate throughput reference;
- highveld-refuge-network-feasibility: event-time, shared-resource, progressive max-min and time-series service policy engine;
- highveld-refuge-capacity-oracle: bounded exact complete-witness check;
- highveld-refuge-shared-capacity-audit: independent validation of a proposed plan.

The next meaningful benchmark is a same-input cross-comparison across these engines with identical route graphs, site capacities, water commitments, anonymous demand cohorts and disruption chronology. Report total coverage, minimum cohort service ratio, accessible unmet demand, worst-area gap, water headroom, degradation duration, recovery time and common-cause cut sensitivity. Do not call the simple maximum-throughput result the fair allocation or operational answer.

## 7. Acceptance and claim ceiling

- zero site capacity double-counting;
- zero accessible demand served through general capacity;
- zero water budget overrun;
- zero blocked, unknown or over-time route accepted;
- zero accessibility evidence missing for accessible demand;
- deterministic ordering and 100% known-answer agreement;
- per-area unmet demand and route/accessibility exclusions visible.

**Status:** SYNTHETIC_AGGREGATE_REFERENCE_COMPARATOR_NOT_FIELD_VALIDATED. No real site, demand area, route, capacity, water or resident data is admitted. This is not a reservation, assignment or emergency dispatch system.
