# Energy observation shield reference v0.1

A dependency-free Rust reference model for independently re-admitting one expiring dispatch action against a finite, explicitly declared observation and future-condition envelope.

This is a **research checker**, not a controller, optimizer, simulator, HIL result, certification or actuation system.

## What it checks

The monitor accepts a proposed interval action only if the declared predicates hold for **every** possible current state in the admitted observation envelope and **every** declared next-interval operating condition.

It independently recomputes:
- charge gain and discharge delivery using integer-floor rational efficiency;
- SOC after the action, including capacity and withdrawal constraints;
- grid outage/import limits;
- the prohibition on simultaneous charging and discharging;
- exact bus-energy balance;
- essential-service floor and demand bounds;
- whether enough PV/grid/battery energy remains for the next-step essential-service floor while preserving the declared future reserve.

The equations intentionally match the finite integer-quantum semantics of the existing Sol-Atlas dispatch request, but the monitor is separately implemented: it does not call the optimizer, import its helpers, or trust a solver-provided `soc_after` or `safe=true` flag.

The future condition is an explicit finite one-step envelope. For each current-state successor SOC, the checker conservatively calculates the maximum permitted next-step battery withdrawal after preserving `future_reserve_quanta`, then checks the declared future scenario's essential-service floor. Every state/scenario pair must pass.

## Dispositions

| Disposition | Meaning |
|---|---|
| `Admit` | Proposed action passes all declared current-state and future-condition checks |
| `FallbackAdmitted` | Proposed action failed, but a separately digest-bound fallback passed the same current-state/future checks |
| `Replan` | Proposal expired or the action failed a modeled predicate; no action is selected |
| `Abstain` | Observation, identity, timestamp, calibration, policy or proposal envelope is invalid/unknown |

There is no implicit default action. A missing, stale, conflicting, replayed, reordered or wrong-source observation abstains. Monitor failure/timeout must be handled as abstention by the external process supervisor.

## Evidence binding

The report carries:
- proposal digest;
- original request/schedule/action digests;
- selected action digest (including fallback identity);
- observation digest;
- policy, battery-model, uncertainty-profile, future-envelope and monitor digests.

The crate treats digest fields as **already computed identifiers supplied by a trusted caller**. It does not canonicalize/serialize or hash request, telemetry, policy or action bytes. The caller must compute each digest from the exact bytes under the pinned serialization profile and verify the policy/model/monitor artifacts before calling. A digest value alone is not proof that the source, sensor, calibration, policy owner or executable is authentic.

The observation includes source/calibration identities, epoch, sequence, sample/receive/expiry timestamps, and a finite list of possible current states. A production adapter must construct that state set using a separately reviewed uncertainty/calibration method. Passing one fabricated or under-conservative state set would invalidate the conditional argument.

## Run

From repository root:

```sh
cargo fmt --manifest-path research/energy-observation-shield/Cargo.toml --check
cargo check --locked --all-targets --manifest-path research/energy-observation-shield/Cargo.toml
cargo test --locked --all-targets --manifest-path research/energy-observation-shield/Cargo.toml
cargo clippy --locked --all-targets --manifest-path research/energy-observation-shield/Cargo.toml -- -D warnings
```

Rust 1.98.1 is pinned. The crate has no third-party dependencies.

## Initial adversarial test panel

The unit tests cover:
- all-current-state and all-future-scenario acceptance;
- action locally within SOC bounds but with no safe next-step continuation;
- one unsafe state inside a multi-state observation envelope;
- stale and replayed observations;
- missing, conflicting and wrong-source observations;
- schedule-state prediction outside the admitted SOC envelope;
- recomputed exact bus-balance rejection;
- fallback admission against the same current and future envelope;
- fallback without its own digest rejected;
- empty future envelope and expired proposal;
- observation expiry and impossible sample timestamp;
- failure with no valid fallback;
- rational efficiency floor behavior;
- exact report binding for proposal, request, schedule, action, observation, policy, model, uncertainty, future envelope and monitor.

These are finite software tests, not field/plant evidence.

## Important limits

This v0.1 reference checks only the declared integer-quanta microgrid contract and one-step continuation envelope. It does not model:
- AC/DC network voltages, frequency, fault currents, inverter dynamics or protection;
- electrochemical battery thermal/ageing/safety constraints;
- uncertain transition intervals beyond the explicit finite scenario set;
- multi-step viability beyond the next interval;
- time-varying power limits or ramp constraints not represented in the request;
- telemetry authenticity, cryptographic signatures, trusted clocks or sensor calibration truth;
- HIL, commissioning, regulatory/grid-code compliance or human/operator authorization.

The next scientific step is a **bounded multi-step viability reference** for small state/action sets, differential-tested against exhaustive reachability. Only after that should larger robust viability kernels, reachable-set approximations or CBF/MPC filters be compared. A finite checker must never describe its admitted envelope as the full physical plant.

## Current external comparators and promotion ladder

The v0.1 integer-SOC shield is not a direct competitor to converter-level safety controllers. The following 2026 literature should inform later matched comparisons:

- **Proximal-reserve CBF control:** a June 2026 *IEEE Transactions on Industrial Informatics* paper combines a nominal distributed controller with a proximal-reserve control-barrier safety layer, claims forward invariance, closed-loop stability and recursive feasibility, and validates the method in HIL on a four-dispatch-unit autonomous DC microgrid. This is an important HIL/controller baseline once Sol-Atlas represents the relevant voltage/current/converter states; the present SOC-only model cannot reproduce those claims. DOI: https://doi.org/10.1109/TII.2026.3659902
- **Attack-resilient distributed secondary control:** a May 2026 *Applied Energy* paper studies deception attacks, communication-efficient event-triggered control and converter-level large-signal microgrid behavior with HIL validation. Add data integrity, replay, delay, packet loss and correlated communication failure to the future attack panel, but do not equate this integer telemetry fixture with an attack-resilient secondary controller. DOI: https://doi.org/10.1016/j.apenergy.2026.127521
- **Learning-enhanced stochastic MPC:** a November 1, 2026 *Applied Energy* paper reports a learning-enhanced stochastic MPC framework for resilient multi-energy microgrids, including operation under 30% battery capacity loss and a reported approximately 40% forecast-error reduction against its selected baselines. These are paper-specific results, not reproduced Sol-Atlas results; compare only after matching data, degradation assumptions, forecast horizons and operational objectives. DOI: https://doi.org/10.1016/j.apenergy.2026.128321
- **Testing framework:** IEC TS 62898-3-5:2026 specifies HIL, commissioning and periodic testing for microgrid monitoring/control/energy-management systems. IEEE P2030.8 is an active PAR (approved March 26, 2026) defining controller-function testing procedures; it is not yet a published standard. Obtain the normative/official documents before asserting conformity.

### Promotion ladder

1. **Finite software reference:** exhaustive differential tests for every admitted/rejected state/action tuple, including uncertainty-envelope boundaries.
2. **Dynamic simulator:** add voltage, current, inverter saturation, line/network constraints, disturbances and model mismatch; compare the shield with proximal-reserve CBF/robust MPC on identical models.
3. **Cyber-resilience simulation:** replay, spoofing, deception, communication delay/loss, clock skew, event-triggered updates and common-source compromise; compare with the attack-resilient distributed-control literature.
4. **HIL:** measure transient constraint violations, settling/recovery, command-to-effect mismatch, replan/fallback latency, and false-safe/false-reject outcomes using the relevant IEC/IEEE test structure.
5. **Commissioning and periodic evidence:** only with equipment owners and required safety/utility processes. Simulation and software receipts do not certify hardware.

References:
- Proximal-reserve CBF / HIL: https://doi.org/10.1109/TII.2026.3659902
- Attack-resilient event-triggered secondary control / HIL: https://doi.org/10.1016/j.apenergy.2026.127521
- Learning-enhanced stochastic MPC: https://doi.org/10.1016/j.apenergy.2026.128321
- IEC TS 62898-3-5:2026: https://webstore.iec.ch/en/publication/84175
- IEEE P2030.8 active PAR: https://standards.ieee.org/ieee/2030.8/12543/

