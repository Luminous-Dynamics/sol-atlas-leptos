# Power Fabric Maintenance Qualification v0.1

**Status:** design/contract proposal; synthetic fixture only.  
**Parent:** [POWER-NETWORK-QUAL-001 #332](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/332)  
**Maintenance operations issue:** [#923](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/923)  
**Asset/manufacturing lifecycle:** [Mycelix #6297](https://github.com/Luminous-Dynamics/mycelix/issues/6297)  
**Physical model applicability:** [Symtropy #5607](https://github.com/Luminous-Dynamics/symtropy/issues/5607)

## Decision

Build a small, independent **maintenance-availability projection and service-impact receipt** between canonical asset lifecycle records and Sol-Atlas network/service studies.

Do not create a second:
- asset registry, product configuration, BOM, work-order or maintenance event ledger;
- condition-monitoring/RUL model;
- general OPF/AC/EMT solver;
- chronological service evaluator.

Mycelix remains the canonical identity and lifecycle owner. Symtropy remains the physical model, condition and operating-envelope owner. Sol-Atlas projects the exact maintenance state into network/chronology studies and independently verifies service impact.

## 1. Core model

The source event and the operational availability projection are distinct records:

`Mycelix lifecycle event -> validated projection -> interval availability state -> network/service replay`

Every projection must bind:
- exact serialized asset identity;
- exact installed configuration and its digest;
- canonical source maintenance/work-order event reference and digest;
- model profile(s) and output applicability where health/condition influences the decision;
- UTC time basis, planned and observed timing, and evidence freshness;
- required isolation, inspection, test, and return-to-service authority references;
- explicit unknown/unsupported disposition.

### Availability states

| State | Operational meaning |
|---|---|
| Available | Evidence admits the exact asset/configuration as available for the declared interval and duty |
| Degraded | Asset remains available only within a declared reduced capability envelope |
| PlannedOutage | Scheduled isolation/maintenance makes the asset unavailable |
| ForcedOutage | Unplanned failure/event makes the asset unavailable |
| Quarantined | Asset is isolated pending investigation/disposition |
| Unknown | Evidence is insufficient, stale, conflicting or outside the model/profile envelope; do not assume available |

A planned outage is not a failure. A forced outage is not a planned maintenance event. Unknown must not be collapsed into Available.

### Event lifecycle

The projection should distinguish at least:
1. proposed/scheduled;
2. authorized;
3. isolation initiated;
4. isolated / work in progress;
5. work complete, inspection pending;
6. inspection/test accepted;
7. return-to-service authorized;
8. available under the new exact configuration;
9. failed / quarantined / rejected / cancelled.

A planned end time is not proof of completion. When actual completion or required post-work evidence is missing, the asset remains unavailable or Unknown, according to the declared policy.

## 2. Hard invariants

1. **Identity:** event asset ID and configuration digest must match the target in the network model/service graph.
2. **Time:** intervals use a declared UTC time basis; timestamps, duration, ordering and overlaps are validated.
3. **No implicit return:** planned completion never makes an asset available. Return requires actual completion plus all profile-required acceptance evidence and authority.
4. **No stale reuse:** a configuration, firmware, material or repair change invalidates downstream physical-model and qualification receipts according to dependency closure.
5. **Part compatibility:** a replacement part must bind to an approved part/configuration revision; nominal dimensions or supplier name alone do not establish equivalence.
6. **Capability:** a degraded state must reference a qualified output-specific envelope; no scalar health score may invent a derating.
7. **Isolation:** an asset cannot simultaneously be isolated for work and used by the operational network model.
8. **Conflicts:** overlapping work orders, inconsistent state transitions, duplicate event identities and contradictory availability evidence fail closed.
9. **Authority:** an event record or digital signature is not itself permission to energize, switch, reset protection or return to service.
10. **Unknown:** missing condition, part compatibility, crew capability, test evidence or time certainty remains explicit and may force HOLD/REPLAN.

## 3. MaintenanceAvailabilityTraceV1

This is a derived, study-scoped projection, not a canonical lifecycle ledger.

Minimum envelope:
- schema version and deterministic trace ID;
- exact source event references and source digests;
- study/case digest, network topology revision and model-profile ID;
- target asset/configuration identity;
- ordered state intervals with UTC start/end and state;
- planned vs actual timing, including uncertainty or late-completion status;
- available/degraded capability reference, only when qualified;
- isolation/inspection/test/return authority references;
- projection disposition: Complete, Incomplete, Conflicted, NotEvaluable;
- claim ceiling and explicit limitations.

For each asset and interval there must be exactly one effective state. Conflicting events cannot be resolved by “last write wins”; the conflict must be surfaced.

## 4. MaintenanceServiceImpactReceiptV1

The service-impact receipt binds:
- exact maintenance availability trace digest;
- exact network/source/model profile digest;
- exact dispatch candidate/solver receipt;
- exact chronological service request and service graph;
- independent replay/verifier version;
- per-interval asset availability, topology and capability;
- network feasibility / infeasibility disposition;
- reserve and storage trajectory;
- per-location essential/flexible service requested, delivered and unmet;
- recovery time, longest essential deficit, ENS/EENS and outage-linked deficit;
- all unsupported, unknown, stale, conflicting or excluded evidence.

A schedule must not be called feasible only because the optimization model returned success. Every interval is independently replayed with the maintenance state applied.

## 5. Frozen synthetic case

Use a deliberately small panel with three buses, two supply paths, one constrained corridor, one storage resource, and essential and flexible demand at different buses.

Events:
- interval 1: all assets available;
- intervals 2–3: one corridor asset in PlannedOutage for authorized maintenance;
- interval 3: one supply asset also enters a declared Degraded state, with its reduced limit unresolved in the negative-control variant;
- interval 4: corridor work is marked complete, but the post-work inspection is missing, so the asset remains unavailable;
- interval 5: inspection/test is accepted and return-to-service authority is present; the exact post-maintenance configuration becomes available;
- adversarial variant: replacement component revision differs from the approved configuration;
- adversarial variant: maintenance exceeds the planned end time and the schedule tries to use the asset early.

All numerical values and state events are synthetic. This fixture tests identity, timing and fail-closed semantics; it is not a real network study or a maintenance procedure.

## 6. Baseline policies

Compare on identical cases and information horizons:

1. Fixed calendar window, dispatch unchanged.
2. Fixed calendar window with feasible redispatch and storage.
3. Condition-based scheduling using only evidence available at decision time.
4. Risk-aware scheduling over explicit condition/RUL uncertainty.
5. Network- and service-aware co-optimization of maintenance start, dispatch and reserve.
6. Clairvoyant schedule with future failure and repair information, labelled upper bound only.

Keep hard safety/isolation rules outside objectives. Essential-service floors cannot be traded for cost, carbon or utilization improvements. Preserve the complete Pareto alternatives or explicit lexicographic policy; no undocumented weighted-sum collapse.

## 7. Industry comparators and what they do not cover

- **PyPSA:** its v1.3.0 release added maintenance scheduling for Generator, Link and Process components; v1.4.0 (9 October 2026) includes further security-constrained, multi-period, seasonal-rating and operational-model changes. It is a strong chronological planning comparator, but the documented maintenance component scope does not amount to universal line/transformer/switchgear maintenance qualification.
- **Grid2Op/L2RPN:** a sequential power-system operation environment whose actions include maintenance operations. It is useful for sequential policy comparison on matching grid/action semantics, not a manufacturing lifecycle record.
- **REopt API v3:** useful for site, building, campus, community and microgrid technology sizing, economics, dispatch and resilience; not an asset-level factory acceptance or transmission maintenance qualification engine.
- **MATPOWER / PowerModels / HiGHS / Tellegen:** network optimization references. Their solver output does not establish equipment manufacturing quality, physical health, protection behavior or post-repair test acceptance.

References:
- [PyPSA v1.4.0 release notes](https://docs.pypsa.org/latest/release-notes/)
- [PyPSA roadmap](https://pypsa.org/roadmap)
- [Grid2Op](https://github.com/Grid2op/grid2op)
- [NLR REopt API v3](https://developer.nlr.gov/docs/energy-optimization/reopt/)

## 8. Benchmark metrics

**Asset/work execution:** planned vs actual outage duration, maintenance deferral, parts/crew wait, diagnosis time, repair/restore time, repeat work, rework, post-repair test turnaround, and inspection/qualification completeness.

**Network and service:** infeasible intervals, constrained corridors, available transfer capability under the declared model, served/unserved essential and flexible demand, ENS/EENS, longest essential deficit, per-location worst-case service, reserve trajectory and recovery time.

**Prognostics (only where qualified):** false alarm, missed failure, RUL error, interval coverage/calibration, out-of-domain detection and abstention. Predictions are not observations.

**Lifecycle:** cost per actually delivered essential-service unit, replacement/repair costs and material recovery with declared functional units/boundaries; keep embodied carbon and circularity Unknown unless input evidence is admitted.

Report p50/p95 and worst-case outcomes. Aggregate availability or mean cost must not conceal a critical-location interruption.

## 9. Promotion gates

- 100% exact source event, asset and configuration identity for in-scope events.
- Zero unverified return-to-service transitions.
- Zero use of an asset while isolated, quarantined or unknown.
- 100% rejection/abstention on the frozen negative-control corpus.
- 100% of reported schedules independently replayed against chronology, network availability, reserve and essential-service floors.
- All missed, late, incomplete and conflicting events visible in the final report.
- At least two matched external policy/solver baselines before any competitive performance claim.
- Physical/field claims require calibrated test articles, competent review and independent evidence; synthetic pass rates cannot be promoted to field reliability.

## 10. Multi-physics boundary

Maintenance impact must consume the correct Symtropy model regime for the claim:
- network OPF for steady-state feasibility;
- RMS dynamics for electromechanical/control recovery;
- EMT for switching, converter current limiting, harmonic and protection interactions;
- regime-appropriate EM (magnetostatic/MQS, eddy-current, full-wave) for equipment-specific field/loss/insulation questions;
- thermal RC/CFD and structural/electromechanical models where heating, force, vibration or ageing affects the qualified envelope.

Reuse [Symtropy SPECTRA-SOLVER-001 #3396](https://github.com/Luminous-Dynamics/symtropy/issues/3396), [GMD-TRANSFORMER-DYNAMIC-001 #4528](https://github.com/Luminous-Dynamics/symtropy/issues/4528), and [POWER-FABRIC-DYNAMIC-MODEL-001 #5607](https://github.com/Luminous-Dynamics/symtropy/issues/5607). A static DC availability result is not an EMT, protection, thermal or mechanical qualification.

## 11. Explicit non-claims

This specification does not establish real asset health, RUL accuracy, maintenance safety, transformer capability, protection coordination, grid-code compliance, outage authorization, physical grid truth, or permission to energize. It defines an auditable software boundary for later qualified engineering work.
