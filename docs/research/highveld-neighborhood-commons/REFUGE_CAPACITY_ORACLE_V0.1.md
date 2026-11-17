## Aggregate thermal refuge capacity oracle v0.1

**Study ID:** HIGHVELD-REFUGE-CAPACITY-ORACLE-001  
**Parent:** HIGHVELD-THERMAL-SERVICE-REFUGE-001 / HIGHVELD-REFUGE-AVAILABILITY-001  
**Maturity:** BOUNDED_EXACT_FEASIBILITY_ORACLE_SYNTHETIC_ONLY

### Why it is needed

A per-site availability check can say that a refuge has spare seats. It cannot establish that several demand areas can all rely on the same seats at once. Independent per-area checks can double-count shared total capacity, accessible capacity and safe-water reserves.

This oracle evaluates all anonymous demand cohorts and all candidate sites against one shared-resource state. It answers whether a complete, evidence-qualified assignment exists for every declared cohort place. It is a feasibility proof over supplied inputs, not a normative priority policy.

### Anonymous demand

Each cohort supplies a pseudonymous service-cell ID, total places, accessible places as a subset of total places, safe water per place for a declared duration, a maximum travel-time bound and route evidence from Sol-Atlas. No resident name, device ID, health status, movement history or individual trajectory is accepted or needed.

### Shared resource equations

- Free total places = total capacity − current occupancy − committed reserve.
- Free accessible places = accessible capacity − accessible occupancy − accessible committed reserve.
- Free safe water = safe-water available − committed water − protected reserve.

Accessible capacity is a subset of total capacity. Accessible places consume both pools; ordinary places consume the total pool. Water is consumed by both. A site is usable only when its opening window, thermal service, ventilation, safe-water service, rights, quality and staffing/maintenance are all qualified and capacity/service/water evidence references are present. Routes must be reachable, evidenced and within the cohort travel-time bound. Accessible cohorts also require a qualified accessible route.

### Bounded exact search

The Rust implementation uses deterministic depth-first search with backtracking across anonymous place-units and shared site resources. It does not greedily commit a nearest-site choice.

Input envelope: at most 8 sites, 8 demand cohorts, 32 places total, and 100,000 search nodes. If the limit is reached, the answer is INDETERMINATE, not feasible or infeasible. No heuristic result inherits the exactness label.

A feasible witness is for human/operator review only and must be re-read against current owner state before any commitment. It is not a booking or occupant assignment. An infeasible result applies only to the finite supplied graph and constraints; it does not prove that no other site or route exists.

### Known-answer controls

1. Partition-tolerant positive control: all cohorts can be covered across multiple qualified local sites.
2. Double-booking trap: each cohort individually fits the same site, but combined demand exceeds its free capacity.
3. Accessible-route trap: general seats exist, but accessible route or capacity is inadequate.
4. Protected-water trap: demand would only fit by spending protected water reserve.
5. Common-cause outage: unknown/failed ventilation or shared service state removes a site from the graph.
6. Positive-control preservation and full witness replay.

### Industry comparison

A September 2026 building-level heat-risk study assessed indoor thermal exposure and cooling-center accessibility across 214,681 Phoenix residential buildings, finding spatial mismatch between building heat performance and access to cooling centers. A September 2026 urban-park study introduced a cooling-accessibility metric using a physics-informed two-step floating catchment area model and reported large cooling-service blind zones in its study geography. Earlier cooling-center placement research used multicriteria ranking and population coverage across five US cities.

These are useful spatial diagnostic baselines. Highveld's proposed addition is operational: event-time opening and service states, accessible capacity, finite safe-water after reserve, route-specific travel constraints, simultaneous anonymous demand cohorts, common-cause failures and network partitions. The fair comparison is to run the static access baseline and dynamic feasibility oracle on the same site inventory, demand cells and network graph, and attribute differences to the added operational constraints.

### Acceptance targets

- Zero shared-seat overbooking and accessible-capacity double-counting.
- Zero protected-water reserve breaches.
- Zero unknown/failed site dependencies treated as qualified.
- Zero route claims without evidence references.
- 100% witness replay on feasible known-answer cases.
- 100% detection of seeded infeasible controls.
- Exactness reported only when bounded search completes; limit exhaustion is indeterminate.
- No personal identity, health or movement data; no booking or occupant assignment.

### Owners and claim ceiling

Sol-Atlas owns route/accessibility calculation and evidence. Mycelix owns water rights, quality, commitments and capacity attestations. Symtropy owns event time, common-cause chronology and replay. Resident/asset authorities retain decisions on participation and actual commitments.

**Status:** BOUNDED_EXACT_FEASIBILITY_ORACLE_SYNTHETIC_ONLY. No real refuge, site, route, demand, capacity, water or service data is admitted. This is not an emergency dispatch system.