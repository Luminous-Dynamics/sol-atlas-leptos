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
