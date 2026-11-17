# Shared refuge capacity and service-gap audit v0.1

**Study ID:** HIGHVELD-REFUGE-ALLOCATION-AUDIT-001  
**Parent:** HIGHVELD-THERMAL-SERVICE-REFUGE-001 / HIGHVELD-REFUGE-AVAILABILITY-001  
**Maturity:** SYNTHETIC_GROUP_LEVEL_PLAN_AUDIT_NOT_OPERATIONAL

## Purpose

The single-site availability filter cannot establish that multiple demand areas can all consume the same spare seats, accessible places or safe-water reserve. This audit consumes a proposed group-level plan and checks it against one shared site-capacity ledger.

It is deliberately not an assignment optimizer. It does not allocate named residents, optimize people, reserve seats, book a refuge, compute routes or direct emergency movement.

## Conservation rules

For every site, sum all group allocations before comparison:

\[
\sum_g n_{g,s} \leq C_{s,\mathrm{free}}
\]

\[
\sum_g a_{g,s} \leq A_{s,\mathrm{free}}
\]

\[
\sum_g w_{g,s} \leq W_{s,\mathrm{free}}
\]

For every group, allocations cannot exceed its declared demand in any dimension. Accessible places are a subset of total places. An allocation requires upstream site eligibility and a route marked reachable/accessibility-qualified with a non-empty route evidence reference.

A site may appear in many group allocations, but its capacity is counted only once. Duplicate group/site rows are invalid; callers must aggregate them with traceable source events before evaluation.

## Outputs and fairness

The audit reports separately:
- total places served/requested;
- accessible places served/requested;
- safe-water litres covered/requested;
- each demand area's residual service gap;
- site-level total, accessible and water utilisation;
- minimum group fulfilment ratio in each dimension;
- maximum group gap in each dimension.

There is no composite fairness score. A high total fulfilment ratio cannot hide an inaccessible allocation, water shortfall or one area's complete service failure. A feasible plan with residual demand is reported as FEASIBLE_WITH_UNSERVED_DEMAND, not full service.

## Falsification and targets

Known-answer controls include a valid multi-group positive control, total-seat double-spending, accessible-capacity double-spending, safe-water double-spending, blocked/unknown route, unavailable site, duplicate group/site row, per-group over-allocation, unknown site ID and duplicate group identity.

Targets: zero accepted total/accessibility/water overcommitments; zero allocation to unknown, unavailable or unreachable sites; zero duplicate rows silently accepted; preserve the positive control; 100% traceability; no personal data required.

**Status:** SYNTHETIC_GROUP_LEVEL_PLAN_AUDIT_NOT_OPERATIONAL. The fixture is synthetic. The audit validates a proposed aggregate plan but does not create or commit an allocation.
