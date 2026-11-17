# Highveld topology-removal study — preregistration protocol v0.1

**Parent candidate:** `NEIGHBORHOOD_RESILIENCE_COMMONS`  
**Maturity:** `PROTOCOL_ONLY_NOT_EXECUTED`  
**Status:** research protocol; no topology has been ranked or evaluated  
**Relation to PR #736:** does not alter its frozen three-candidate B1/B2 matrix

## 1. Question

Which dependencies actually create resident-service resilience, and which only create the appearance of resilience through installed capacity, asset availability, central coordination or optimistic routing?

The experiment removes one dependency at a time from each applicable topology, then tests selected compound removals. The independent oracle evaluates whether each household's declared essential-service floor was delivered. It does not infer service from a facility being online, water being in a tank, a battery being charged, a route existing on a map, or an optimizer returning a feasible plan.

This is deliberately an ablation experiment, not a new simulator. Symtropy must provide native event identity, event order/time, paired exogenous traces, resource transitions, household service trajectories and stable-recovery semantics. Sol-Atlas supplies versioned topology/geometry/access constraints. Mycelix supplies governed resource/consent/commitment semantics and qualified dependency evidence. The finite oracle is independently specified and must not reuse the candidate's own success predicate.

## 2. Freeze the comparison before execution

All topology arms in one comparison set must bind the same:
- household identities and resident-declared service floors;
- service definitions, units, demand windows and quality limits;
- spatial geometry, household locations and accessibility requirements;
- exogenous hazard/disruption trace bytes and digest;
- initial resource inventory, source quality, reserve floors and account rules;
- total installed service capacity for the capacity-matched arm;
- lifecycle and operating budget assumptions for the budget-matched arm;
- labour, maintainer, spares, information and repair assumptions;
- component and model versions, configuration digests and applicability domains;
- independent service-floor, resource-conservation and dependency/cut-set oracle versions.

A comparison is invalid if any of these differ silently between arms. If an assumption cannot be matched, expose it as a measured confounder and do not attribute its effect to topology.

Run at least two separate comparisons:
1. **Capacity matched:** equal total installed capacity and service-quality limits.
2. **Budget matched:** equal declared capital plus lifecycle/operating/maintenance budget, with price dates, boundaries and uncertainty visible.

A maintenance-capacity-matched arm is a useful third experiment: hold qualified technician-hours, spares and response coverage constant. Do not combine these into one weighted score.

## 3. Removal ladder

Run the intact reference first, then the following single removals where applicable:

| Removal | Applicability | Question |
|---|---|---|
| `REMOVE_PRIMARY_HUB` | Central hub; layered hybrid | Does loss of the hub break local guarantees or expose a hidden single point of failure? |
| `REMOVE_EACH_DISTRIBUTED_NODE` | Distributed nodes; layered hybrid | What is the marginal service loss for each node and its catchment? |
| `PARTITION_COORDINATION_NETWORK` | Federated mesh; layered hybrid | Do local nodes continue safely without shared coordination, or do stale reservations/double commitments appear? |
| `REMOVE_MOBILE_SUPPORT_UNIT` | Mobile support; layered hybrid | Does route, payload, staff or replenishment dependence dominate nominal mobile capacity? |
| `CUT_PRIMARY_ACCESS_ROUTE` | All arms | Which households become inaccessible, and is there a separately qualified route/delivery fallback? |
| `REMOVE_SHARED_UPSTREAM_FEEDER` | Every arm with that dependency | Are household fallbacks and nominally separate providers still coupled to the same grid/water/control dependency? |
| `REMOVE_REPAIR_AND_SPARES` | Every arm with shared repair dependency | Does a service remain marked available when no qualified maintainer, spare or requalification path exists? |
| `EXHAUST_SHARED_RESERVE` | Every arm with a shared reserve | Are household floors incorrectly met by double-counting reserve, transfers or stock already committed elsewhere? |
| `WITHDRAW_SHARED_DATA_CONSENT` | Every arm using shared household data | Does the system stop the exact disallowed purpose without disabling unrelated local/manual service? |

If a component is absent, the row is `NOT_APPLICABLE` with a reason. It is never silently converted to a zero-loss pass. Every applied removal names the exact component/failure-domain IDs it disables; a label such as "independent" is not proof of independence.

After single-removal results are preserved, run paired-removal interactions only where a preregistered dependency path makes them meaningful. Initial examples: hub plus primary route; distributed node plus repair/spares; network partition plus reserve exhaustion; mobile unit plus route cut; upstream feeder plus local backup dependency. Compound cases must retain the same frozen exogenous trace and identify the additional ablation explicitly.

## 4. No hidden reconfiguration

The first pass is a strict ablation: remove the declared component and do not silently resize, relocate, reroute, reallocate or invoke an unregistered fallback. This isolates dependency contribution.

A second, separately reported adaptive pass may use a predeclared recovery policy. That policy must be versioned and identical in decision authority, information access, time budget and stopping rule across arms where the policy is applicable. It cannot see future events. Report adaptation cost, delay, resource draw, failed actions and residual risk.

Do not let one topology receive bespoke human intervention while another is evaluated autonomously. Human response may be an arm, but it must be explicit, bounded and equally instrumented.

## 5. Output contract

Symtropy owns all event clocks, event IDs, native sequence numbers and service/resource trajectories. Each evaluated case must produce:
- input package digests and exact model/configuration versions;
- topology and component/failure-domain map before and after ablation;
- explicit `APPLIED` or `NOT_APPLICABLE` removal record and reason;
- per-household/per-service requested and delivered quantity, unit, quality, access delay, queue delay, unavailable duration and floor status;
- resource-account opening/inflow/transfer/commitment/loss/reserve/closing balance with unique source-event identity;
- service transition and stable-recovery evidence;
- independent oracle result and disagreements;
- maintenance effort, spare consumption, route/access outcomes and lifecycle cost evidence where available;
- unknown, censored, missing and out-of-domain states without imputation to PASS.

The independent oracle must be able to replay the frozen input package without trusting the candidate's own final status. Keep raw observation, normalized observation, inferred state and qualified conclusion as separate fields.

## 6. Hard gates before Pareto comparison

| Gate | Target |
|---|---:|
| Household essential-floor false passes | 0 |
| Missing required household/service outcomes | 0 |
| Asset availability/proximity/stock mistaken for delivered service | 0 |
| Undetected common-cause dependency against the finite oracle | 0 |
| Duplicate credits, double commitments or silent unit conversions | 0 |
| Wrong-purpose, pre-grant or post-revocation shared-data access | 0 |
| Unknown/out-of-domain state converted to PASS | 0 |
| Positive controls incorrectly blocked | 0 |
| Exact finite known-answer agreement | 100% |

Only arms that pass the same hard gates enter comparative analysis. Report per-household and per-service vectors first: floor breaches, time-to-breach, duration unserved, access/queue delay, recovery, resource reserve, maintenance burden, autonomy, privacy, lifecycle cost and environmental burden. Compare the resulting Pareto set; do not construct an average that compensates for a failed household.

## 7. Owner boundaries

- **Sol-Atlas:** topology identity, geometry, access, site constraints, configuration provenance and requirement-change impact.
- **Symtropy:** native event/time/sequence identity, paired traces, service/resource trajectories, causal replay and stable recovery.
- **Mycelix:** rights/consent, instrument admission, allocation commitments/appeals, offline exchange and qualified dependency/cut-set contracts.
- **Specialist solvers:** electrical, hydraulic, thermal, structural, route and other domain equations.
- **Independent oracle:** finite known-answer service-floor and conservation checks; it must not share the candidate success predicate.

The JSON matrix beside this document is a machine-readable registration of the removal families, not execution evidence. No event timestamps, IDs, probabilities, trajectories or performance values are invented here.

## 8. Claim ceiling and next gates

Current status is `PROTOCOL_ONLY_NOT_EXECUTED`. The existing two-household fixture is a contract smoke test, not an adequate comparative sample. Before running a meaningful study, obtain owner acceptance for the trace/trajectory interface, freeze a non-synthetic or explicitly synthetic input package, bind a qualified dependency oracle and preregister the comparison arms. Field use additionally requires rights-cleared instrument data and instrument calibration/admission.

The expected result is allowed to be that household-local passive capacity dominates a shared topology for a particular service and hazard envelope. The research goal is to identify the minimum viable, maintainable service architecture—not to maximize shared infrastructure.

## References and related work

- [PR #741 — Neighborhood Resilience Commons](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/741)
- [Symtropy interface review #1848](https://github.com/Luminous-Dynamics/symtropy/issues/1848)
- [Mycelix dependency/cut-set interface #5762](https://github.com/Luminous-Dynamics/mycelix/issues/5762)
- [Long-horizon benchmark proposal #744](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/744)
