# GMD essential-service continuity oracle

A dependency-free Rust reference evaluator for complete, time-indexed essential-service demand/delivery trajectories.

## Why this exists

Asset survival is not equivalent to essential-service continuity. A transformer may survive while a hospital water supply or telecom backhaul fails; an electrical outage may be tolerable when a real, qualified islandable microgrid or backup system maintains service.

The evaluator reports a **per-service resilience vector**, not one score that adds incompatible quantities such as kWh, m³, Mbps or patient throughput.

## Inputs

Each service trajectory declares:
- stable service identity and asset/topology revision;
- a native quantity unit;
- evidence class, source reference and SHA-256 digest of the trajectory artifact;
- minimum service floor as a fraction of required service;
- native-unit quantity tolerance;
- minimum sustained duration required to confirm recovery;
- complete contiguous intervals over the scenario horizon;
- required and delivered quantities over each interval.

Time is represented as elapsed seconds from scenario event t0, not as wall-clock UTC. This preserves scenario ordering even when GNSS or external time is unavailable. Mycelix retains source-clock and receiver-local time-quality evidence separately.

## Metrics per service

- required, capped-delivered, surplus-excluded and unmet native-unit quantities;
- demand-weighted served fraction;
- time availability above the owner-declared floor;
- total floor-violation duration, longest continuous violation and episode count;
- maximum interval deficit fraction;
- whether the service initially meets its minimum floor and full demand;
- time to a sustained minimum-floor recovery after a recorded floor violation;
- time to sustained full-service recovery after a recorded full-service violation.

A recovery mark contains both the beginning of the qualifying interval run and the later time at which its required hold duration is confirmed. Initial service condition is reported separately; an initially healthy service is not falsely reported as having recovered. A momentary supply spike does not establish recovery.

There is deliberately no cross-service aggregate score. Consumers must inspect every critical service floor and the worst service/region.

## Run

From the Sol-Atlas repository root:

    cargo fmt --manifest-path tools/gmd-service-continuity-oracle/Cargo.toml -- --check
    cargo test --manifest-path tools/gmd-service-continuity-oracle/Cargo.toml --locked --offline
    cargo clippy --manifest-path tools/gmd-service-continuity-oracle/Cargo.toml --locked --offline --all-targets -- -D warnings

## Fail-closed semantics

- Intervals must cover exactly [0, horizon] with no gap, overlap, duplicate or reordering.
- Missing observations are not forward-filled and cannot count as delivered service.
- Demand/delivery must be finite and non-negative.
- Surplus is reported but cannot compensate for another interval's deficit.
- Zero-demand intervals count as no-demand, not as unserved service; the whole horizon must contain positive total demand.
- Each service is evaluated in its own declared unit.
- Every input trajectory identifies its evidence class, source digest and asset revision.
- Recovery hold duration must be positive; quantity tolerance may not consume the entire minimum service floor in any positive-demand interval.

## Claim ceiling

Passing synthetic tests proves only arithmetic and time-series evaluator mechanics. It does not simulate electrical power flow, water hydraulics, telecommunications, healthcare delivery, microgrid feasibility, protection, resource dispatch or human behaviour. It cannot certify an actual service continuity or black-start claim. Field/operational claims require owner-authorised measured data and qualified upstream physics.

## Research comparators

- IEEE PES, *Power System Resilience Metrics and Evaluation Methods*, 8 June 2026, DOI 10.17023/7e16-km93.
- Shi, Bai and Wang, *Power distribution system blackstart restoration using renewable energy*, Nature Communications 17, 8498 (17 August 2026), DOI 10.1038/s41467-026-76692-0.

The evaluator operationalizes a small, explicitly defined metric subset; it does not claim conformance to the complete IEEE PES report or any black-start standard.
