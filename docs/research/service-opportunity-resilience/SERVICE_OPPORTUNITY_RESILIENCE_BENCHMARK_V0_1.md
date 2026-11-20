# Service-opportunity resilience benchmark and reference oracle v0.1

**Tracking issue:** [CITY-SERVICE-OPPORTUNITY-RESILIENCE-001 #959](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/959)  
**Base integration:** [temporal amenity evaluator PR #957](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/957)  
**Maturity:** synthetic reference oracle and contract only; not executed  
**Geographic profile:** portable contract; no Johannesburg data admitted  
**Claim ceiling:** no real-world service reliability, safety, equity, legal, resident-outcome or physical-validation claim

## 1. Objective

Qualify whether a resident cohort can complete a declared service task over time, including when facilities, routes, operators, capacity or supporting infrastructure are disrupted.

The unit of analysis is:

**scenario × phase × cohort × service × functional unit × time interval**

This benchmark joins upstream evidence; it does not recreate it.

## 2. Paradigm shift: asset restoration is not capability restoration

A route may reopen before a facility is commissioned. A building may be repaired while capacity remains below the service floor. A service may briefly resume and fail again. A nominally separate facility may share the same operator, bridge, power feed or communications link.

Therefore the result is a time-indexed opportunity state, not an asset status or resilience badge.

The evaluator consumes four mandatory predicates for each alternative and cohort:

- **Route:** CityMorph's owner-qualified access result.
- **Service:** Mycelix's owner-qualified operating/calendar result.
- **Capacity:** Symtropy's owner-qualified service/resource feasibility result.
- **Suitability:** resident-declared requirements and their owner-qualified evidence.

Every predicate is bound to exact common scope, population frame, service, functional unit, time basis and source revision.

## 3. Normative state algebra

### Alternative state

For a scope-matched alternative over one interval:

1. If any mandatory predicate has a current, owner-qualified QUALIFIED_UNAVAILABLE result, the alternative is UNAVAILABLE.
2. Otherwise, if any predicate is UNKNOWN or STALE, the alternative is UNKNOWN.
3. Only if every mandatory predicate is QUALIFIED_AVAILABLE is the alternative AVAILABLE.
4. A scope, population, service, functional-unit or time-basis mismatch is NOT_COMPARABLE and cannot be repaired by matching display names.

A proven current negative predicate can establish that one alternative is unavailable even if another predicate is unknown: the conjunction cannot be satisfied. Conversely, stale or missing evidence alone cannot establish unavailability.

### Resident-task opportunity across alternatives

- **AVAILABLE:** at least one in-scope alternative is fully qualified.
- **UNKNOWN:** no alternative is qualified available, and at least one potentially relevant alternative remains unresolved.
- **NOT_COMPARABLE:** no alternative is available or unresolved, but one or more candidate alternatives cannot be compared because scope binding differs.
- **UNAVAILABLE:** all declared, in-scope alternatives are known unavailable.

The candidate universe must be declared. UNAVAILABLE means all alternatives in that declared universe are proven unavailable; it is not a claim that no unregistered alternative exists.

### Time and interval semantics

All fixture ticks are 900-second synthetic ticks. Intervals are half-open, [start,end). Adjacent intervals with the same state can be coalesced only when their boundaries touch exactly and their evidence/scope continuity is established. Gaps are never silently bridged.

The reference corpus uses four intervals: [0,2), [2,4), [4,6), and [6,8). These are test discretizations, not observations or probabilities.

## 4. Recovery semantics

Distinguish:
- first available opportunity;
- first opportunity restoration after a known loss;
- owner-qualified stable service recovery;
- stable opportunity recovery, which additionally requires the composed opportunity to be AVAILABLE at the owner-attested stable tick.

The evaluator must consume Symtropy's stable-recovery evidence. It must not infer stable restoration from a single good sample or a transient return to service.

A route/service/capacity restoration mismatch remains unavailable until all mandatory opportunity predicates are again qualified.

## 5. Failure domains and redundancy

Failure-domain identifiers are not probabilities. Two assets with different labels are not automatically independent. A complete declared dependency map is necessary for a bounded analysis, but even that does not establish physical independence without owner-qualified evidence.

The benchmark therefore reports:
- current qualified usable alternatives;
- declared shared dependency identifiers;
- dependency-map completeness;
- alternatives lost in each explicit fault set.

It does not estimate independent failure probabilities, MTBF or real-world uptime from synthetic cases.

## 6. Frozen synthetic reference corpus

The machine-readable oracle is SERVICE_OPPORTUNITY_RESILIENCE_REFERENCE_ORACLE_V0_1.json.

It contains **16 hand-authored synthetic scenarios**, including:
- independent local fallback;
- shared-operator common-cause loss;
- a reachable destination without a full-duration service opportunity;
- split route/service restoration;
- insufficient qualified capacity;
- cohort-specific suitability failure;
- stale service evidence;
- overlapping compound disruption;
- recurrent outage before stable recovery;
- temporary substitute during construction;
- incomplete versus complete calendar evidence;
- half-open interval adjacency;
- scope mismatch;
- unknown dependency-map completeness.

The expected output includes per-interval opportunity states, alternative-state counts, coalesced intervals, first availability, restoration after known loss, stable restoration and dependency-map completeness.

Expected results are independent hand-calculated golden answers. The later implementation must not generate or rewrite the expected values from its own algorithm.

## 7. Cross-project ownership

| Evidence | Owner | This evaluator must not |
|---|---|---|
| Route and cohort access | CityMorph #555 | Recompute routes |
| Shared scope | Sol-Atlas #623 | Invent a second scope digest |
| Temporal service fit | Sol-Atlas #956 | Rebuild calendars or booking |
| Departure distributions | Sol-Atlas #958 | Infer a CDF or assign weights |
| Compound event evidence | Sol-Atlas #735/#736 | Invent event chronology or local observations |
| Event clock | Symtropy #1739 | Create a second clock |
| Physical/service and stable-recovery trajectory | Symtropy #1848 | Infer physical restoration |
| Capacity/resource feasibility | Symtropy #3768 | Recalculate owner capacity |
| Asset/service/operations | Mycelix #6328/#6329 | Create a parallel asset registry |
| Allocation/booking | Mycelix #5901 | Grant access or allocate capacity |

A service-calendar completeness assertion is not a Mycelix field until that owner approves it. A calendar's completeness must be bounded to the exact period, service and time basis it covers.

## 8. State-of-the-art comparison: compare the right layer

| System / method | Established capability | Luminous comparison boundary |
|---|---|---|
| Conveyal R5 | Network routing, opportunity accessibility, departure-window travel-time distributions and time percentiles | Consume identical outputs; add owner-qualified full-duration service and disruption composition. Do not claim better routing. |
| OpenTripPlanner 2 | Multimodal routing and accessibility-aware route configuration, including treatment of unknown wheelchair data | Preserve accessible / inaccessible / unknown distinctions. Do not reinterpret unknown as accessible. |
| Cumulative opportunity | Count reachable opportunities under declared thresholds | Baseline for interpretable access coverage. |
| Gravity-based access | Opportunity weighted by impedance/decay | Baseline for graded access; publish decay assumptions. |
| Competition-based / 2SFCA family | Supply-demand competition and capacity pressure | Baseline where input demand/supply and catchment assumptions are defensible. |
| PEAI (2026) | Proximity, availability, suitability, affordability and awareness | Reproduce published method before adaptation; keep dimensions visible and user weights explicit. |
| B0 manual incumbent workflow | Specialist systems used separately with human integration | Record integration errors, operator time, missingness and decision trace. |
| B1 Luminous composition | Same specialist outputs plus scope/time/revision/trajectory receipts | Test false-positive reduction, selectivity, cohort visibility and replayability. |
| B2 independent oracle | Hand-computable finite cases | Exact conformance target; not a field truth set. |

### Important research caution

More complex accessibility metrics do not automatically predict lived travel behaviour better. A 2026 comparative study across Toronto, Montréal and Vancouver reported that cumulative-opportunity and gravity-based measures had higher explanatory power for transit mode share than more complex competition-based measures, with a negligible difference between the former two. The benchmark must therefore test predictive/decision value and cost, not reward complexity for its own sake.

PEAI's 2026 Vancouver example demonstrates that time-threshold proximity can differ substantially from a multidimensional assessment: 99% within median tolerable cycling times and 74.5% within acceptable walking thresholds, versus 37.9% classified as truly accessible under its framework. Reproduce the published method; do not copy its weights as universal policy.

## 9. Benchmark ladder

| Gate | Test | Target |
|---|---|---:|
| Contract | Exact scope/service/population/time binding | 100% match or explicit NOT_COMPARABLE |
| Safety/selectivity | Known unavailable versus clean available controls | 0 false positives and 0 false negatives on oracle |
| Evidence integrity | stale/unknown/incomplete evidence | 0 optimistic promotions |
| Interval integrity | boundary, adjacency, overlap, gap and recurrence | 100% exact oracle agreement |
| Resilience | shared dependencies and overlapping disruptions | 100% declared fault-set agreement; no invented probabilities |
| Recovery | asset, route, capacity and stable-service restoration | 0 false stable-restoration claims |
| Distribution | cohort-specific service-floor failures | 0 hidden mandatory failures |
| Replay | source revisions, exact head and fixture digest | deterministic exact-head receipt |
| External validation | rights-cleared held-out local evidence | preregistered; independent review required |
| Operational validation | instrumented, unoccupied demonstrator | repeated service, maintenance and recovery evidence |

The synthetic acceptance targets are:
- 100% independent golden-answer agreement.
- 0 false AVAILABLE.
- 0 false UNAVAILABLE.
- 0 unknown/stale evidence promotion.
- 100% clean-case acceptance.
- 100% scope/time/functional-unit mismatch detection.
- 0 duplicate service/capacity accounting.
- 0 false stable-restoration claims.
- 0 probability or independence inference without supporting evidence.

## 10. Empirical validation design

The synthetic corpus qualifies the state algebra only. It cannot validate residents' lived access.

A later Johannesburg study should:
1. Use the existing CityMorph #561 source-readiness and rights pathway.
2. Preregister the geographic boundary, population frame, resident/cohort requirements, services, outcome measures and missingness policy.
3. Compare R5/OTP and the Luminous layer on the exact same network and route outputs.
4. Reproduce PEAI and cumulative/gravity/competition-based baselines with matched definitions.
5. Hold out neighbourhoods and/or time periods.
6. Validate against independently collected, consented and rights-cleared service-use or resident-reported unmet-opportunity outcomes.
7. Publish confidence/uncertainty, selection bias, nonresponse, coverage and all denominators.
8. Keep booking histories and movement data minimized and privacy-protected.

No local dataset is claimed acquired or validated by this protocol.

## 11. Sources

- Conveyal R5 methodology: https://docs.conveyal.com/analysis/methodology
- OpenTripPlanner accessibility documentation: https://docs.opentripplanner.org/en/latest/Accessibility/
- PEAI, Beyond time: A multidimensional framework for assessing accessibility via active transportation, Journal of Urban Mobility 9 (June 2026), 100217: https://doi.org/10.1016/j.urbmob.2026.100217
- Which access matters? A comparative analysis of accessibility metrics and their impacts on commuting, Journal of Transport and Land Use (2026): https://doi.org/10.5198/jtlu.2026.2832


## 11. Decision-relevant evidence acquisition and service debt

Unknown opportunity states should lead to a deliberate investigation decision, not an indiscriminate request for more data.

Compose Symtropy [SPATIAL-VOI-001 #1892](https://github.com/Luminous-Dynamics/symtropy/issues/1892). Do not create another generic value-of-information engine.

For each unresolved service claim, identify:
- which missing owner evidence blocks the decision;
- which lawful, rights-cleared measurement or source request could resolve it;
- which observation outcomes are possible under the declared finite state set;
- whether any outcome changes robust-feasible alternatives, the Pareto set or a hard service-floor decision;
- measurement cost, time, privacy burden and implementation feasibility;
- residual unknowns that remain after the measurement.

Without defensible priors, likelihoods and calibration, do not report expected monetary VOI or expected probability of a decision change. Report worst-case decision impact, outcome-conditional feasible/Pareto sets, state-set reduction, action cost and residual blockers. A high-volume survey that cannot change any admitted decision may be less valuable than a small, targeted inspection that resolves one decisive service constraint.

### Service debt

Where an owner-qualified demand basis exists, report accumulated unmet service as a dimensional quantity over time rather than a generic resilience score.

For a declared cohort and service functional unit:

`service_debt = integral(unmet_service_rate(t), t0..t1)`

For discrete intervals, this is the sum of unmet service units per interval multiplied by interval duration. The unit might be resident-workshop-hours, household-water-volume, or another service-specific measure. Never add unlike service units together. If the demand rate, service capacity, time basis or cohort denominator is not qualified, report the missing basis and retain UNKNOWN rather than fabricating service debt.

Service debt complements (and does not replace):
- known unavailable duration;
- unresolved duration;
- time to first restoration;
- time to stable restoration;
- worst-cohort service-floor breaches.

## 12. Claim discipline and implementation order

This package is a reference oracle and protocol only. It is not an executable evaluator, not CI evidence, not an empirical study and not field qualification.

Implementation order:
1. Have the independent golden answers reviewed.
2. Freeze the closed schema and fixture digest.
3. Implement a Rust evaluator that consumes these inputs without owning routes, event clocks or capacity.
4. Add metamorphic and adversarial tests (input order, exact half-open boundaries, stale evidence, recurrence, shared dependencies).
5. Produce exact-head receipt bound to commit, schema, fixture and owner revisions.
6. Integrate Symtropy #1892 only for measurement/action decision impact; do not make a duplicate VOI engine.
7. Only then run B0/B1 comparison on identical specialist inputs.


This package is a reference oracle and protocol only. It is not an executable evaluator, not CI evidence, not an empirical study and not field qualification.

Implementation order:
1. Have the independent golden answers reviewed.
2. Freeze the closed schema and fixture digest.
3. Implement a Rust evaluator that consumes these inputs without owning routes, event clocks or capacity.
4. Add metamorphic and adversarial tests (input order, exact half-open boundaries, stale evidence, recurrence, shared dependencies).
5. Produce exact-head receipt bound to commit, schema, fixture and owner revisions.
6. Only then run B0/B1 comparison on identical specialist inputs.
