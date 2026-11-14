# Highveld refuge network max-throughput reference oracle v0.2

**Study ID:** HIGHVELD-REFUGE-NETWORK-FLOW-001  
**Parent:** HIGHVELD-THERMAL-SERVICE-REFUGE-001 / HIGHVELD-REFUGE-AVAILABILITY-001  
**Maturity:** SYNTHETIC_AGGREGATE_REFERENCE_COMPARATOR_NOT_FIELD_VALIDATED  
**Purpose:** provide an independent Edmonds–Karp maximum-throughput reference for the accessible-first, general-second refuge capacity benchmark. It is a comparator for the existing progressive max-min engine, not a new primary allocation engine.

## 1. Scope relative to the existing refuge stack

The repository already has three more specialized pieces:

- \`highveld-refuge-network-feasibility\` is the primary multi-zone engine. It evaluates site and route state, freshness, shared water/capacity, progressive max-min fairness across anonymous cohorts, and an ordered supply/demand degradation sequence with unmet place-hours and recovery/degradation metrics.
- \`highveld-refuge-capacity-oracle\` is a bounded exact backtracking feasibility oracle for a complete group-level witness. It has a declared envelope and reports search-limit exhaustion as indeterminate.
- \`highveld-refuge-shared-capacity-audit\` checks a proposed aggregate plan against one shared site ledger; it is an audit, not an optimizer.

This binary is deliberately smaller. It provides an independent conventional maximum-throughput reference against which the primary fairness-aware policy can be tested. It does not replace the primary network engine, the bounded exact witness oracle or the proposed-plan audit.

## 2. Policy alignment

The reference implements the primary engine's snapshot semantics:

1. **Accessible-place flow first.** Accessible demand can use only sites with a reachable, fresh, evidence-backed route and qualified accessibility evidence, bounded by each site's effective accessible capacity.
2. **General-place flow second.** After accessible flow, all remaining effective site capacity is available to general demand, including unused accessible-designated seats. This follows the primary engine's residual-total-capacity rule; it is not a blanket no-substitution constraint.
3. **Shared water budget.** Effective site capacity is limited by both free total places and \`floor(free_safe_water_l / water_l_per_person)\`. Accessible service consumes site capacity/water before the general pass.
4. **Per-cohort travel bounds.** Each demand area supplies its own maximum travel-time limit. A global bound remains as a fallback for the standalone fixture.
5. **Freshness and site qualification.** The primary-case adapter admits sites only when their service, water, capacity, rights/quality and site-status evidence is qualified and fresh; routes require current route and accessibility freshness records.
6. **Deterministic graph order.** Areas and sites are sorted by stable IDs before graph construction.

The allocation rule is a maximum-throughput reference, not a fairness objective. The primary engine separately computes progressive max-min cohort coverage; both results should be compared on identical inputs.

## 3. Same-input primary fixture replay

The Rust adapter reads the primary \`REFUGE_NETWORK_FEASIBILITY_CASE_V0.1.json\` fixture, carries each cohort's demand and route-time bound into the reference schema, derives free total/accessibility/water balances from the same site records, excludes sites with unknown or stale critical evidence, and keeps only the routes to qualified sites with route evidence. The source fixture and excluded-site list are attached to the output.

The current primary fixture's independently observed max-throughput reference is:
- total: 12 of 20 places;
- accessible: 6 of 6;
- general: 6 of 14.

The reference oracle must reproduce those totals on the adapted case. The primary progressive max-min policy's separate cohort vector is not expected to match maximum-throughput cohort distribution; that difference is a benchmark result, not a defect.

This is a synthetic cross-binary parity test for one snapshot. It is not yet a broad corpus, physical route validation, or temporal replay. Time-series identity/order/freshness remains owned by Symtropy and the primary engine.

## 4. Input and finite envelope

The input contains:
- anonymous area IDs and total/accessible place requests;
- per-area maximum route travel time;
- candidate site IDs, eligibility state and free total/accessibility/water capacity;
- water litres per place for the snapshot horizon;
- route reachability, route freshness, accessibility and accessibility freshness evidence;
- source evidence references and an independent-oracle reference.

The implementation envelope is at most 64 demand areas, 64 refuge sites and 1,000,000 aggregate place requests. These are implementation bounds, not claims about community scale. Exceeding them is an input error, not an invitation to return a partial answer.

## 5. Network and accounting

For each class, the oracle builds:

\[
source \rightarrow demand\ area \rightarrow qualified\ refuge \rightarrow sink
\]

The site's effective total-capacity edge exists exactly once per flow pass. Accessible flow is subtracted from that site's capacity and water before the general pass. Unused accessible-designated seats are released to general demand under the primary engine's current benchmark rule.

\[
C_{effective}=\min(C_{total,free},\lfloor W_{free}/L_{safe/place}\rfloor)
\]

\[
C_{accessible,effective}=\min(C_{accessible,free},C_{effective})
\]

\[
C_{general,residual}=C_{effective}-C_{accessible,served}
\]

Route and accessibility exclusions are reported separately. A reachable but inaccessible route may be considered for general demand, but cannot be counted for accessible demand. Straight-line distance is never used as a substitute for route evidence.

## 6. Known answers

The standalone synthetic fixture requests 15 places total, including four accessible places; it offers 14 free places and enough water for those seats. Expected result: 4/4 accessible, 10/11 general, 14/15 total.

A second test verifies capacity release: with a 10-place site, four accessible-designated seats, one accessible request and nine general requests, the remaining three accessible-designated places are released after accessible demand is met, and all ten total places can be served.

The cross-binary primary fixture adapter test verifies 6/6 accessible, 6/14 general and 12/20 total against the primary engine's max-throughput result. All fixtures are synthetic known answers.

## 7. Relationship to state-of-the-art and benchmark use

Recent urban heat network research models heat-risk sources and cooling-buffer sinks, identifies critical nodes/corridors and tests robustness against attack/random disruption. Recent refuge-location research considers time-sensitive accessibility, operating hours and shelter types. Static accessibility baselines such as 2SFCA answer a valuable spatial coverage question but do not by themselves prove time-specific shared capacity or water service.

Use the stack together:
- static 2SFCA / route reachability: spatial coverage baseline;
- this max-throughput oracle: independent snapshot throughput reference;
- \`highveld-refuge-network-feasibility\`: primary event-time, shared-resource, progressive max-min and time-series service policy engine;
- \`highveld-refuge-capacity-oracle\`: bounded exact complete-witness check;
- \`highveld-refuge-shared-capacity-audit\`: independent validation of a proposed plan.

Do not interpret a higher aggregate flow as a superior service policy if it worsens minimum cohort service, accessible unmet demand, temporal continuity or worst-area gaps. The next meaningful benchmark is a broader same-input corpus with identical routes, capacities, water commitments, anonymous demand cohorts and disruption chronology. Report total throughput, minimum cohort service ratio, accessible unmet demand, worst-area gap, water headroom, degradation duration, recovery time and common-cause cut sensitivity.

## 8. Acceptance and claim ceiling

- 100% agreement with the primary engine's reported snapshot maximum-throughput total on the adapted fixture;
- zero site capacity double-counting;
- zero accessible demand served through a route without qualified accessibility evidence;
- zero water budget overrun;
- zero blocked, unknown, stale or over-time route accepted;
- deterministic ordering and 100% known-answer agreement;
- per-area unmet demand, route exclusions and accessibility exclusions visible.

**Status:** SYNTHETIC_AGGREGATE_REFERENCE_COMPARATOR_NOT_FIELD_VALIDATED. No real site, demand area, route, capacity, water or resident data is admitted. This is not a reservation, assignment or emergency dispatch system.
