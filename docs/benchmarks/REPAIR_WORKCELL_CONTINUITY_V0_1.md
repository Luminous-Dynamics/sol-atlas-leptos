# Repair Workcell Continuity V0.1 — Synthetic Regional Service Comparison

**Status:** research prototype and synthetic reference evaluator; no production deployment or field result  
**Owner issue:** [TOOLCHAIN-LOCAL-001 #161](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/161)  
**Source baseline:** `Luminous-Dynamics/sol-atlas-leptos@522938aa723bc7f8f62b2dbd2b7f9999e24b55a1`  
**Implementation:** `sol-atlas-core/src/repair_continuity.rs`  
**Input schema:** `schemas/spatial-economic-repair-continuity-v0.1.schema.json`  
**Synthetic fixture:** `schemas/examples/repair-workcell-continuity-v0.1.json`

## Paradigm shift: repairability is a sustained service, not a product attribute

A repairable product or a workshop with nominal tools does not establish that a region can restore failed equipment within a useful time. Repair continuity depends on the whole constrained system: qualified capability, calibrated instruments, actual-compatible parts, available stock, staffing, power, routes, finite service hours and disruption timing.

V0.1 compares three architectures against identical demand and a shared declared disruption schedule:

- **B0 — centralized depot:** centralized stock and multi-skill capability, with longer travel and an explicit central-route dependency.
- **B1 — local service cell:** shorter travel and limited local stock/capability.
- **B2 — distributed shared reserve:** a local/regionally distributed cell using finite stock delivered from a declared reserve.

The implementation is deliberately narrower than the long-term regional self-reliance programme. It models guarded low-voltage repair profiles only: fans, sensors, small pumps and simple controller boards. It is not an authorization or procedure for mains/high-energy equipment, sealed refrigerant systems or other hazardous jobs.

## Separation of ownership

This module owns only the regional comparison of declared resources against a matched demand fixture.

- Work-order identity and verified restoration outcomes belong to the canonical repair/work-order owner.
- Exact-model/revision compatibility and approved substitute decisions must come from the canonical parts owner.
- Actual stock, reservations, receipts, consumption and replenishment belong to the canonical inventory/manufacturing ledger.
- Calibration authority, trained-person qualification and safety checks must be supplied as qualified evidence from their existing owners.
- Routing and travel estimates must come from the canonical transport/geographic owner.
- Source truth, snapshot age and authority are not proven by this evaluator; evidence metadata is carried forward rather than treated as an attestation.

A public supplier listing is not confirmed stock; a compatible label is not proof of exact revision fit; an expired or unknown calibration is not qualified; a scenario schedule is not an observed work order. The evaluator will not create a second inventory ledger or claim a completed restoration.

## Contract and semantics

The Rust input contract is mirrored by the v0.1 JSON Schema and a small synthetic JSON fixture. It describes service windows, repair demand, time-indexed dependency observations, three architecture candidates, workcells, qualification/calibration, inventory pools and stock lots.

Key behaviors:

1. **Matched comparison:** all architectures receive the same demand and declared disruption input. Candidate order is canonicalized, and result collections are emitted in stable order.
2. **Unknown stays unknown:** absent staffing/power/route or part-fit evidence can lead to an indeterminate result. Unknown effective capacity and unallocated capacity are represented by optional values in the capacity ledger; they are not silently serialized as zero.
3. **Finite resources:** a stock lot is consumed at most once, and service hours are reserved against one cell/window capacity budget. The v1 scheduler does not split work across windows.
4. **Constrained dispatch:** work is ordered by structural scarcity, due window, declared priority and stable request ID; within a request, the selected feasible assignment minimizes modelled turnaround, then uses stable window/cell tie-breaks.
5. **Evidence is not truth:** `observed` evidence requires a source identifier, URI, date-shaped snapshot and 64-character SHA-256. This is structural provenance validation, not verification of the source or of the stock claim.
6. **Explicit claim ceiling:** every output states `synthetic_modelled_feasibility_only_not_verified_restoration`. Scenario counts are not probabilities, and an assignment is not evidence that work occurred.
7. **Bounded work:** input counts have explicit limits; demand × workcell × window fit evaluations are capped at 2,000,000; part requirements are capped at 64 per demand; stock lots are capped at 10,000 per candidate; and the upper bound of total part requirements × workcells × windows × stock lots is capped at 20,000,000 inspections. Arithmetic overflow and over-budget requests fail closed.

### V0.1 simplifications to keep visible

- A request must fit fully within one window; task splitting, technicians' individual calendars, queues with interruption/resumption and stochastic arrivals are not modeled.
- Parts from multiple pools for one request use a single modeled pickup time equal to the maximum transfer time. This is a declared simplification, not a routing solution.
- Derated staffing/power scale effective available service-hours by the minimum declared fraction; a production model needs sourced calibration of that relationship.
- No failure probability is inferred from the number of synthetic scenarios.
- Cost, emissions, waste, procurement decisions, unique beneficiaries, safety approval and actual repair quality are intentionally not invented in the first slice.

## Frozen fixture interpretation

The fixture has one fan repair request and two eight-hour windows. It intentionally places the central stock arrival in the later window while making the central service route unavailable in that same window. The local candidate has a compatible assumed stock lot before the interruption. The distributed candidate has a declared later replenishment route and a response window that allows the modelled delay.

This is a semantic test, **not a Johannesburg/Gauteng empirical statistic**. Values are synthetic assumptions specifically chosen to expose the difference between topology, stock timing and route interruption.

The fixture tests that each candidate sees the same request, that outcomes reconcile to the request denominator, that the central path cannot simply ignore the route interruption, and that a later shared reserve remains tied to a specific availability window.

## Initial benchmark gates (targets, not yet measured)

| Gate | Initial target |
|---|---:|
| Matched inputs | 100% of candidates process exactly the same demand IDs and declared disruption snapshot |
| Fixture oracle | Exact expected request status, assignment window and stock use |
| Unknown-resource handling | Zero unknown staffing, power, route, part-fit or stock evidence converted into confirmed availability |
| Capacity integrity | Zero assignments above the cell/window's effective known capacity; unknown capacity remains explicitly unknown |
| Inventory integrity | Zero duplicate stock IDs, duplicate pool access, overspend or repeated consumption of one lot; unknown routes do not contaminate unrelated part requirements |
| Calibration integrity | Zero requests confirmed using missing, expired or unknown required instrument calibration |
| Evidence validation | All malformed observed evidence and invalid digest inputs rejected |
| Determinism | 100/100 repeated runs over canonicalized fixture produce identical result data |
| Input permutation | 100/100 demand/candidate/workcell/inventory permutations preserve canonical output |
| Bounded-resource controls | Requests above 2,000,000 fit evaluations, 20,000,000 estimated stock inspections, 64 part requirements/request, or 10,000 stock lots/candidate fail closed |\n| Negative controls | 100% of duplicate stock, incompatible revision, missing evidence, delayed stock, unknown inventory route and capacity-overrun controls reject or remain explicitly unresolved |
| Throughput envelope | Set measured p50/p95 once named-runner results exist for 100 / 1,000 / 10,000 demand items; no speed claim yet |
| Physical success | No claim from this evaluator; actual restoration needs a controlled, independently verified pilot |

The first local gates are correctness and honesty, not a latency target. Only after the finite-resource semantics are qualified should larger sweeps and measured p50/p90 response-time comparisons be introduced.

## Next research steps

1. Qualify formatting, native core tests and a browser/WASM build at an exact source head. Queued workflow state is not a pass.
2. Strengthen contract tests for unknown resource states, source freshness, expired calibration, incompatible revision, shared-reserve depletion, duplicated cell identity and exact boundary behavior.
3. Cross-check small cases against a second independent scheduler/accounting oracle, then use deterministic generated fixtures to exercise input-order invariance and capacity conservation.
4. Consume authoritative repair, part-fit, inventory, calibration and route records only after those interface owners are version-pinned and validated. Do not duplicate their ledgers.
5. Run a physical pilot with real demand, confirmed stock, actual queue/repair/verification times and safety-qualified protocols. Keep synthetic, desk-researched, historical and physical evidence classes separate.
6. Only then compare centralized, local and distributed options against a preregistered, representative disruption family. Report P50/P90 time, percentage of all requests with verified useful outcome, referrals, rework, unserved/indeterminate demand, inventory drawdown and critical dependency concentration separately.

## Sources

- [TOOLCHAIN-LOCAL-001 issue #161](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/161)
- [Industrial capacity evidence contract, PR #160](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/160)
- [Spatial-economic resilience analysis, PR #116](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/116)
- [Open Repair Data Standard](https://standard.openrepair.org/standard.html)
- [NIST Policy on Metrological Traceability](https://www.nist.gov/calibrations/traceability)
