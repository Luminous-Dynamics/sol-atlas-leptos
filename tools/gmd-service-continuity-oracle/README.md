# GMD essential-service continuity oracle

A dependency-free Rust reference evaluator for complete, time-indexed essential-service demand/delivery trajectories.

## Why this exists

Asset survival is not equivalent to essential-service continuity. A transformer may survive while a hospital water supply or telecom backhaul fails; an electrical outage may be tolerable when a real, qualified islandable microgrid or backup system maintains service.

The evaluator reports a **per-service resilience vector**, not one score that adds incompatible quantities such as kWh, m³, Mbps or patient throughput.

## Inputs

Each service trajectory declares:
- stable service identity, owner reference, geographic/jurisdiction reference and asset/topology revision;
- a native quantity unit;
- evidence class, source reference and SHA-256 digest of the trajectory artifact;
- minimum service floor as a fraction of required service;
- native-unit quantity tolerance;
- maximum allowed aggregation interval, set to the resolution needed by the service use case;
- minimum sustained duration required to confirm recovery;
- complete contiguous intervals over the scenario horizon;
- required and delivered quantities over each interval.

Time is represented as elapsed seconds from scenario event t0, not as wall-clock UTC. This preserves scenario ordering even when GNSS or external time is unavailable. Mycelix retains source-clock and receiver-local time-quality evidence separately.

## Metrics per service

- required, capped-delivered, surplus-excluded and unmet native-unit quantities;
- demand-weighted served fraction;
- time availability above the owner-declared floor during positive-demand intervals;
- total floor-violation duration, longest continuous violation and episode count;
- maximum interval deficit fraction;
- whether the service initially meets its minimum floor and full demand;
- time to a sustained minimum-floor recovery after a recorded floor violation (with initial condition reported separately);
- time to sustained full-service recovery after a recorded full-service violation.

A recovery mark contains both the beginning of the qualifying interval run and the later time at which its required hold duration is confirmed. Initial service condition is reported separately; an initially healthy service is not falsely reported as having recovered. A momentary supply spike does not establish recovery.

There is deliberately no cross-service aggregate score. Consumers must inspect every critical service floor and the worst service/region.

## Machine-readable contract and synthetic case

- Input schema: `docs/research/gmd-service-continuity/GMD_SERVICE_CONTINUITY_INPUT_V0.1.schema.json` (JSON Schema Draft 2020-12).
- Example input: `docs/research/gmd-service-continuity/GMD_SERVICE_CONTINUITY_SYNTHETIC_CASE_V0.1.json` (synthetic hospital power and potable-water trajectories; no real facility or service floor is implied).
- Output schema: `docs/research/gmd-service-continuity/GMD_SERVICE_CONTINUITY_RESULT_V0.1.schema.json`.
- Expected output: `docs/research/gmd-service-continuity/GMD_SERVICE_CONTINUITY_SYNTHETIC_RESULT_V0.1.json`, independently recomputed from the input and checked in CI.

The schema expresses structure and scalar bounds. CI also runs a Python-standard-library structural preflight over the example. Cross-field interval continuity, exact horizon coverage, per-service maximum aggregation duration and numerical metric rules are enforced by the Rust evaluator, not by JSON Schema alone. The preflight is not a complete JSON Schema Draft 2020-12 validator. A separate standard-library reference calculation recomputes the synthetic result metrics and compares them to the expected-result fixture.


### Independently recomputed synthetic example

For the included five-minute example, using one-minute intervals:

| Service | Required | Capped delivered | Unserved | Served fraction | Floor availability |
|---|---:|---:|---:|---:|---:|
| Hospital critical power | 50 kWh | 42 kWh | 8 kWh | 0.84 | 0.80 |
| Hospital potable water | 9.0 m³ | 6.9 m³ | 2.1 m³ | 0.7667 | 0.80 |

Both services first fall below their declared floor in the third interval. The floor recovery run begins at 180 s and is confirmed at 240 s; full-service recovery begins at 240 s and is confirmed at 300 s. These figures were independently recomputed from the synthetic input arithmetic. They are not Rust execution results, real hospital thresholds or physical resilience evidence.

## Run

From the Sol-Atlas repository root:

    cargo fmt --manifest-path tools/gmd-service-continuity-oracle/Cargo.toml -- --check
    cargo test --manifest-path tools/gmd-service-continuity-oracle/Cargo.toml --locked --offline
    cargo clippy --manifest-path tools/gmd-service-continuity-oracle/Cargo.toml --locked --offline --all-targets -- -D warnings

## Temporal resolution boundary

The floor calculation applies at the declared interval resolution. An interval-level energy or volume total cannot prove uninterrupted delivery inside that interval: excess later in the same interval could hide a short outage. Choose intervals at or below the owner-declared maximum aggregation duration, preserve the source sampling/cadence in the surrounding evidence receipt, and separately qualify instantaneous voltage, frequency, pressure, link integrity or other quality limits with the appropriate physical/measurement owner. This crate does not interpolate high-frequency service continuity from coarse aggregates.

## Fail-closed semantics

- Intervals must cover exactly [0, horizon] with no gap, overlap, duplicate or reordering.
- No interval may exceed the owner-declared maximum aggregation duration; coarse data cannot be promoted into finer-grained continuity evidence.
- Every service must declare a non-zero maximum interval duration and a non-zero sustained-recovery hold.
- Missing observations are not forward-filled and cannot count as delivered service.
- Demand/delivery must be finite and non-negative.
- Surplus is reported but cannot compensate for another interval's deficit.
- Zero-demand intervals count as no-demand, not as unserved service; they are excluded from the floor-availability denominator and cannot bridge a sustained-recovery window. The whole horizon must contain positive total demand.
- Each service is evaluated in its own declared unit.
- Every input trajectory identifies its evidence class, source digest and asset revision.
- Recovery hold duration must be positive; quantity tolerance may not consume the entire minimum service floor in any positive-demand interval.

## Claim ceiling

Passing synthetic tests proves only arithmetic and time-series evaluator mechanics. It does not simulate electrical power flow, water hydraulics, telecommunications, healthcare delivery, microgrid feasibility, protection, resource dispatch or human behaviour. It cannot certify an actual service continuity or black-start claim. Field/operational claims require owner-authorised measured data and qualified upstream physics.

## Research comparators

- IEEE PES, *Power System Resilience Metrics and Evaluation Methods*, 8 June 2026, DOI 10.17023/7e16-km93.
- Shi, Bai and Wang, *Power distribution system blackstart restoration using renewable energy*, Nature Communications 17, 8498 (17 August 2026), DOI 10.1038/s41467-026-76692-0.

The evaluator operationalizes a small, explicitly defined metric subset; it does not claim conformance to the complete IEEE PES report or any black-start standard.
