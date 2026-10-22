# ENERGY-STORAGE-STATE-V0.1 — state continuity, losses and service coverage

**Status:** implementation and benchmark contract; executable CI results are authoritative.  
**Scope:** one bounded store for one external carrier (electricity or useful heat), over an ordered finite series of intervals.  
**Goal:** a small accounting oracle for intertemporal storage behavior before coupling generation, heat pumps, dispatch and networks.

## 1. Paradigm shift: model time, state and service—not a storage efficiency scalar

A single round-trip efficiency cannot answer whether a store can serve a load at a particular hour. Storage value depends on opening state, charge/discharge power, energy capacity, standing loss, charge/discharge conversion efficiency, time-step length, chronology, and the terminal state.

The model therefore:
- Carries stored energy continuously across ordered intervals.
- Applies standing losses per hour, compounded over each interval duration.
- Bounds charge input by offered energy, charge power × duration, and remaining capacity adjusted for charge efficiency.
- Bounds useful discharge by requested output, discharge power × duration, and retained stored energy adjusted for discharge efficiency.
- Reports curtailed charging, unmet discharge demand, standing loss, charge loss, discharge loss, and final reserve shortfall separately.
- Rejects simultaneous nonzero charge/discharge requests within one interval to prevent artificial same-timestep arbitrage; finer sub-interval resolution can represent legitimate within-hour switching.
- Marks the output as calculated scenario evidence, not measured state-of-charge or verified device performance.

This is deliberately a same-carrier model for electricity and useful heat. Hydrogen production/consumption, pumped-hydro hydraulic state, and coupled thermal/electrical conversion require explicit process models and should not be smuggled into this contract as a generic efficiency.

## 2. Frozen two-interval synthetic fixture

Configuration: 20 kWh capacity, 0 kWh initial inventory, 12 kW charge limit, 10 kW discharge limit, charge efficiency 0.90, discharge efficiency 0.80, 1% standing loss per hour, and 3.5 kWh terminal reserve.

**Interval 1 — charge:** offer 10 kWh, no discharge. The store accepts 10 kWh, stores 9 kWh, loses 1 kWh at charging, and closes at 9 kWh.

**Interval 2 — discharge:** one hour at 1% standing loss loses 0.09 kWh, leaving 8.91 kWh. Deliver 4 kWh to the load: 5 kWh stored energy is withdrawn, including 1 kWh discharge loss. The final store state is 3.91 kWh.

| Metric | Expected result |
|---|---:|
| Accepted charging input | 10 kWh |
| Curtailed charging input | 0 kWh |
| Useful output delivered | 4 kWh |
| Unserved discharge request | 0 kWh |
| Standing loss | 0.09 kWh |
| Charging loss | 1 kWh |
| Discharging loss | 1 kWh |
| Total loss | 2.09 kWh |
| Closing inventory | 3.91 kWh |
| Terminal reserve met | Yes |
| Trajectory balance residual | 0 kWh |

Check:

`opening inventory + accepted charge input = closing inventory + useful discharge + standing loss + charging loss + discharging loss`

For this fixture, `0 + 10 = 3.91 + 4 + 0.09 + 1 + 1`. The expected values are analytically calculated test answers, not a measured battery result.

## 3. Coupled reference case

The integration test `sol-atlas-core/tests/heat_pump_storage_coupling.rs` composes the heat-pump and storage contracts across three boundaries:

1. A synthetic heat pump uses 30 kWh source heat plus 10 kWh compressor electricity, losing 4 kWh in distribution and delivering 36 kWh useful heat.
2. The thermal store accepts only as much of that 36 kWh as its 20 kWh capacity and 90% charging efficiency permit; the excess remains explicit as curtailed input.
3. In the next interval, the store delivers 10 kWh useful heat at 80% discharge efficiency and retains 7.5 kWh of stored energy.

The test requires the heat-pump and storage balances to close independently, checks that the heat output splits into accepted and curtailed charging energy, and checks the store's accepted input against final inventory, useful discharge and modeled losses. The case remains synthetic; it proves interface accounting under the declared assumptions, not plant behavior.

## 4. Fail-closed correctness gates

- Opening inventory must not materially exceed capacity; closing inventory must remain in the physical range.
- Charge/discharge efficiencies must lie in (0, 1]; standing loss fraction must lie in [0, 1].
- Each duration must be finite and positive; each energy/power field must be finite and nonnegative.
- Charge and discharge cannot both be requested in the same interval in v0.1.
- A hard input power limit is multiplied by that interval's duration. Energy capacity and power capacity must never be conflated.
- Storage inventory cannot be created without an accepted charge input, and no discharge may exceed retained inventory times discharge efficiency.
- Unserved demand and terminal reserve shortfall must remain visible; the evaluator must not label them as served or rewrite them away.
- Evidence classes beyond analyst assumptions require a source reference. Hashes bind run identity, not physical truth.
- Deterministic re-run bytes, unknown-field rejection, schema conformance and receipt verification are test gates.
- The 1e-9 kWh balance target applies to unit-scale fixtures; large-scale comparisons need a declared scale-aware absolute/relative tolerance.

## 5. Competitive positioning

This contract is an accounting layer, not a substitute for existing device, site or network software.

- **SAM** provides detailed performance and financial models for technologies including batteries, CSP/thermal storage and renewable generators. It is useful for compatible component/plant cross-checks: https://sam.nlr.gov/
- **REopt** sizes and dispatches distributed generation, battery/thermal storage and flexible loads, including resilience scenarios. It is the relevant next comparator for site-level energy-supply portfolios: https://www.nlr.gov/reopt/about
- **PyPSA** supports intertemporal storage, generation, flexible demand, multi-sector coupling and network power-flow approximations at larger system scales: https://docs.pypsa.org/latest/
- **South Africa's IRP 2025** identifies power-system flexibility as a core challenge as variable generation increases: https://www.dmpr.gov.za/Branches/Mining-Minerals-Petroleum-Policy-Development/IRP-2025
- On 7 October 2026, the South African government described a proposed initial determination prioritizing 4.6 GW of battery energy storage and 5 GW of gas-to-power for immediate flexibility and dispatchable supply. These are policy/procurement figures, not a statement that this capacity is already operating: https://www.gov.za/news/media-statements/minister-kgosientsho-ramokgopa-prioritises-battery-storage-and-gas-power

Our differentiation should be auditable state continuity, first-law invariants, explicit service-deficit accounting, evidence boundaries and deterministic fixtures that can cross-check larger models. We should not claim better dispatch economics than REopt or PyPSA without a matched evaluation.

## 6. Benchmark ladder

| Stage | Proposed target | Required evidence |
|---|---|---|
| State integrity | 100% of interval opening states match the preceding interval close | Multi-interval deterministic regression |
| Conservation | Residual ≤ 1e-9 kWh on the unit-scale fixture | Analytic known-answer and independent calculation |
| Bounds | Zero capacity, power, efficiency or state-bound violations | Property tests and adversarial fixtures |
| Service | Correct separation of delivered discharge, unserved request and reserve shortfall | Over-demand, power-limited and reserve-limited cases |
| Reproducibility | Byte-identical repeated reports; exact input/report SHA-256 matches | CLI receipt test and schema workflow |
| Device calibration | Publish bias, MAE/RMSE and interval coverage against metered trajectories | Independent measured input/output and state-of-charge datasets |
| Portfolio value | Compare lifecycle cost, critical-hour deficit, curtailment, losses and reserve violations against matched baselines | Same chronological load/renewable/tariff data; cross-tool parity where supported |

Do not compare annual round-trip efficiency to hourly dispatch performance, or modelled storage capacity to a field-metered service claim. Preserve assumptions about time resolution, degradation, temperature, ambient conditions, lifetime, replacement and warranty.

## 7. Next tranche

1. Calibrate charge/discharge efficiency and standing losses by temperature, state of charge, power and technology from public or licensed measured datasets.
2. Add multi-rate/timestamp semantics, explicit gap detection, and unit-typed input/output power limits.
3. Add cycle aging/degradation and replacement, with parameter provenance and uncertainty intervals.
4. Extend the current synthetic heat-pump → thermal-store coupling into timestamped site profiles while preserving separate carrier boundaries.
5. Compare the coupled model against SAM on compatible storage/solar cases, REopt for site dispatch, and PyPSA for intertemporal/network portfolios.
6. Add exact small-instance dispatch as an oracle before introducing larger heuristics; score feasibility, service deficits, regret, runtime and energy accounting on identical cases.

**Rule:** storage shifts energy through time; it does not create energy. A simulation must preserve that fact at each transition and over the complete trajectory.
