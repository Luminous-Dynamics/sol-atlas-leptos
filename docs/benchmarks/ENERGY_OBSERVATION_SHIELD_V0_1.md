# ENERGY-OBSERVATION-SHIELD-001 — Per-interval independent action admission

**Status:** research specification, v0.1  
**Tracking issue:** [#420](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/420)  
**Related work:** [rolling-horizon uncertainty #352](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/352), [anytime audited search #417 / PR #418](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/418), [solver traces #410](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/410)  
**Evidence class:** specification only; no runtime shield or HIL result is claimed

## Decision

The optimizer proposes a horizon schedule. Before each interval is committed, a separately compiled monitor re-admits the next action against the latest observed state, source/freshness evidence, declared state/model uncertainty, current equipment capability and a future-safe reserve/continuation condition.

A frozen schedule receipt is not a standing permission to execute its future suffix. Each action proposal expires at the next observation/decision boundary. This composes with #352 (rolling horizon), #403 (replan policy), #408 (temporal mesh), #410 (compute provenance), #417 (anytime incumbents) and #351 (schedule auditor). It does not create a second scheduler or generic optimizer.

## Proposal and dispositions

Every action is an expiring, digest-bound proposal:

```text
ActionProposal {
  request_digest, schedule_digest, interval_id, action_digest,
  observed_state_digest, observation_manifest_digest,
  uncertainty_profile_digest, transition_model_digest,
  monitor_executable_digest, issued_at, expires_at
}
```

Exactly one disposition:

| Disposition | Meaning |
|---|---|
| `admit` | All declared immediate and future-safe predicates pass for current admitted evidence |
| `replan` | The uncommitted schedule suffix is invalidated by state/forecast/model deviation |
| `fallback` | Candidate rejected; a separately specified fallback is independently admitted |
| `abstain` | Required observation/model/monitor evidence is unknown or unavailable; no safety claim |

Unknown, stale, conflicting or cross-domain observations must never become safe merely because a prior optimization completed.

## Finite robust shield first

Start with small finite state/action sets and an independent exhaustive reference. Let X_t be the admitted set of possible current states, U_t the proposed action, W_t the declared disturbance/model-uncertainty set, F the pinned transition enclosure, K the immediate invariant set and R the future-safe continuation set.

Admission must establish:

1. Source identity, sequence/epoch, monotonic timestamps, validity, calibration and measurement-error bounds pass.
2. Immediate state/action invariants pass for every admitted state in X_t.
3. The conservative next-state enclosure F(X_t,U_t,W_t) is contained in K using conservative interval arithmetic/outward rounding.
4. The resulting state set remains in R: a valid next-step action or separately declared safe fallback still exists. Reject locally safe actions that strand essential load in the next interval.
5. Proposal, observations, monitor result and any approval remain valid at commit time.

A one-step SOC clamp is insufficient if it consumes the reserve required for the next outage interval. After finite exhaustive qualification, explore larger robust viability kernels, reachable-set approximations or control-barrier filters as distinct methods with explicit assumptions/proof obligations.

## Event semantics

Preserve immutable, separate events for forecast publication, sensor sample/receipt/admission, schedule proposal/audit, per-step action admission, command issue, controller acknowledgement, physical response observation, state reconciliation, suffix invalidation and fallback/replan/abstention.

Delayed, duplicate, reordered, missing and conflicting readings are distinct. A deviation outside the pinned envelope invalidates the uncommitted suffix; it does not rewrite historical evidence. A command is not an actuation receipt; acknowledgement is not proof of physical effect. Monitor timeout/crash/integrity failure yields `abstain`, unless an independently valid fallback can itself be admitted. Operator approval, hardware interlocks, grid-code compliance and actuation authorization remain separate gates.

## Benchmark

### Frozen scenarios

- scheduled load shedding with known timing and duration;
- surprise outage/restoration and equipment faults;
- PV ramps, forecast bias, heavy tails and PV/load correlation shift;
- essential/flexible demand changes;
- SOC and battery/inverter derating;
- stale, missing, duplicate, replayed, reordered and conflicting sensors;
- timestamp skew and calibration/source mismatch;
- command timeout, saturation, delayed acknowledgement, effect mismatch;
- monitor timeout/crash, executable substitution and unavailable fallback;
- locally safe action with no safe continuation at t+1.

### Comparators

1. Frozen schedule without per-step re-admission (control/negative baseline).
2. #352 receding-horizon controller without independent shield.
3. One-step deterministic constraint clip.
4. Exhaustive finite one-step robust shield.
5. Exhaustive finite short-horizon viability-aware shield.
6. Robust/scenario MPC only when physical model, uncertainty, service floor and information timing match.

### Metrics

False-safe/unsafe admission count; false rejects and abstentions by cause; invariant violations; admitted-action coverage; essential unserved energy and longest interruption; outage survival/recovery; reserve/SOC extrema; invalidated schedule suffixes; fallback/replan counts; time to first audited-and-admitted action; solver and monitor runtime separately; command/acknowledgement/physical-effect mismatch; cost/carbon/throughput/degradation proxy; component-wise objective deltas. Keep every case in the denominator and bind exact request, observation, model, monitor, solver and auditor digests.

### Promotion gates

- Zero false-safe admissions on the finite exhaustive panel.
- Every admitted action agrees with the independent reference for declared predicates.
- Stale/missing/conflicting/replayed/out-of-order evidence never silently passes.
- A state/model deviation beyond the declared envelope invalidates the uncommitted schedule suffix.
- Monitor failure never becomes `admit` by default.
- HIL begins only after simulator/reference qualification.

## Standards and South Africa

IEC TS 62898-3-5:2026 (published 11 June 2026) defines HIL, commissioning and periodic testing for microgrid monitoring, control and energy-management systems in grid-connected and isolated microgrids. Use it as the external test-structure reference; obtain the normative text before claiming conformity.

Wattegama et al. (2026) reviewed 127 peer-reviewed publications on resilient microgrid hierarchical control with a South Africa focus. It distinguishes predictable scheduled load shedding from unannounced faults and identifies the need for standardized resilience metrics. Keep those disturbance classes separate in our benchmark.

References:
- IEC TS 62898-3-5:2026: https://webstore.iec.ch/en/publication/84175
- Wattegama et al. (2026): https://doi.org/10.3390/en19030644
- Rolling-horizon programme: https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/352

## Claim ceiling

A passing finite shield/reference panel establishes only agreement for the exact finite model, state/action set and predicates. It does not establish physical completeness, real-plant stability, sensor accuracy, electrochemical safety, field reliability, standards conformity, grid-code approval or permission to actuate equipment. HIL, commissioning and field trials are separate evidence layers.

## Current industry research comparators (2026)

The first finite SOC shield is a reference layer, not a direct replacement for converter-level safety controllers. The next matched comparisons should include:

- **Proximal-reserve control-barrier functions:** a June 2026 *IEEE Transactions on Industrial Informatics* paper reports a distributed nominal controller plus proximal-reserve CBF safety layer, with forward-invariance, closed-loop stability and recursive feasibility analysis, validated by HIL on a four-dispatch-unit autonomous DC microgrid (DOI: https://doi.org/10.1109/TII.2026.3659902). Once our physical model includes voltage/current/converter states, compare transient constraint violations, chattering, recovery and computation on the same plant model. Our present SOC-only shield cannot claim those properties.
- **Attack-resilient distributed secondary control:** a May 2026 *Applied Energy* study combines deception-attack resilience, event-triggered communication and converter-level large-signal modeling with HIL validation (DOI: https://doi.org/10.1016/j.apenergy.2026.127521). Expand future threat panels to message replay, spoofed observations, delay/loss, clock skew and common-mode communications faults; do not claim that simple freshness checks reproduce that controller.
- **Learning-enhanced stochastic MPC:** an *Applied Energy* paper dated November 1, 2026 reports a learning-enhanced stochastic MPC framework for resilient multi-energy microgrids, including cases with 30% battery capacity loss and a reported approximately 40% forecasting-error reduction against its selected baselines (DOI: https://doi.org/10.1016/j.apenergy.2026.128321). These results are specific to the paper's data/model/baselines and have not been reproduced here. A fair comparison needs matched forecasts, degradation assumptions, physical models, horizons and operational objectives.

Testing references:
- [IEC TS 62898-3-5:2026](https://webstore.iec.ch/en/publication/84175), published June 11, 2026, specifies HIL, commissioning and periodic testing of microgrid monitoring/control/energy-management systems.
- [IEEE P2030.8](https://standards.ieee.org/ieee/2030.8/12543/) is an active PAR approved March 26, 2026, proposing procedures to compare common microgrid-controller functions. It is not yet a published standard. Obtain the normative/official documents before claiming conformity.

## Promotion ladder

1. Finite software reference with exhaustive differential tests for all admitted/rejected state/action tuples and uncertainty-envelope boundaries.
2. Dynamic simulator with voltage/current, inverter saturation, line/network constraints, disturbance sets and model mismatch; compare against proximal-reserve CBF and robust MPC on identical equations.
3. Cyber-resilience simulation with replay, spoofing, deception, delay/loss, clock skew, event-triggered updates and common-source compromise.
4. HIL measuring transient limit violations, settling/recovery, command-to-effect mismatch, replan/fallback latency, false-safe admissions and false rejects.
5. Commissioning/periodic tests only with equipment owners and applicable utility/safety processes.

No step may be skipped based on a software-only pass. The finite SOC proof remains conditional on its declared model and uncertainty set.

