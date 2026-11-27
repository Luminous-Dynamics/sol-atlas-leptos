# CHARGE N3 published-trip benchmark v0.1

**Programme:** Sol Atlas EV route assurance (#664)  
**Primary public source:** https://charge.co.za/n3-real-world-ev-tests/  
**Corpus:** \`data/benchmarks/n3_published_trip_summaries_v0_1.json\`  
**Evidence class:** \`operator_published_summary\`  
**Status:** public-summary consistency harness; not raw-data model validation

## Why this is a useful new benchmark

CHARGE publishes real-world N3 trip summaries containing route direction, distance, net elevation change, total ascent/descent, average speed, reported pack size, battery used and energy efficiency. The currently visible results include multiple 185 km uphill/downhill samples across vehicle models, including Volvo, Geely, Audi and iCAUR.

The N3 test page is valuable because it gives us a geographically fixed corridor with opposing elevation profiles, rather than only synthetic drive cycles. It is a public challenge dataset for a first validation increment; it is **not** a complete raw telemetry dataset and does not disclose the full CAN/BMS, GPS, charger-power, weather or metering traces needed for independent model calibration.

This v0.1 corpus curates 13 published trip summaries retrieved on 27 November 2026. It deliberately omits registration identifiers and charging transaction IDs. The sample is selected from the publisher's publicly visible results; it must not be treated as a random or statistically representative sample of the national fleet.

## What the executable audit checks

For each trip it calculates:

1. Published efficiency energy:
   \`energy_kwh = energy_efficiency_kwh_per_100km × distance_km / 100\`
2. A nominal-capacity/SOC arithmetic proxy:
   \`capacity_soc_proxy_kwh = reported_pack_capacity_kwh × battery_used_percent / 100\`
3. Absolute and relative difference between those two publisher-reported quantities.
4. A range consistency proxy:
   \`range_km = distance_km / (battery_used_percent / 100)\`
5. Whether the published ascent minus descent reconciles to the published net elevation change and whether route direction agrees with that sign.
6. Per-make/model/direction sample counts and min/median/max published energy intensity.

Default consistency tolerances are 0.10 kWh and 0.50% for the energy pair, and 2 km for the published full-charge range proxy. These are display-rounding screens, not physical accuracy bounds.

**Critical interpretation:** pack capacity and battery-used percentage may be inputs to the same operator calculation that produces kWh/100 km and range. A close residual is therefore an internal consistency check on co-published fields, not independent validation of battery energy, usable capacity or traction power. The report's claim ceiling makes this explicit.

## What is not in v0.1

- Raw vehicle/CAN/BMS or GPS time series.
- Independent traction-battery DC energy measurement.
- Metered charger AC input, DC output, efficiency or taper curve.
- Charge-event start/end SOC with unambiguous association to a trip.
- Ambient/wind/precipitation, traffic, tyre pressure, payload, HVAC, road surface or driver-control traces.
- Calibrated battery usable capacity, pack SOH, thermal state or vehicle-specific energy model.

Do not infer charger availability or charge delivery from a station label. Do not join charger-session kWh to a trip solely because the two appear together on the page. The session boundary and direction must be resolved from the source operator's data before that association is admitted.

## Integration ownership

- **Symthaea:** qualified pack capability, cell/module/pack uncertainty and battery observations. Do not infer pack power from a scalar SOH value.
- **Symtropy:** exact EV configuration, vehicle energy model, SOC/taper curves, per-leg energy estimate and drive-cycle validation.
- **Mycelix:** vehicle/pack identity, component/configuration lineage and lifecycle provenance.
- **Sol Atlas:** N3 route geometry, elevation context, site/charger identity and spatial/grid evidence.
- **Evidence admission:** the source issuer and artifact digest must be independently qualified. A public webpage snapshot is not a BMS attestation or a meter record.

## Benchmarks to run next

### A. N3 passenger-EV corridor

When permissioned raw data becomes available, evaluate by held-out trip, vehicle model, direction and date:

- MAE/RMSE in kWh/100 km and absolute kWh per leg.
- P50/P90/P95 absolute error and signed bias.
- Coverage of declared 50%, 80%, 90% and 95% prediction intervals.
- Calibration error by uphill/downhill, speed band, temperature band, model and pack age.
- SOC prediction error at each charger arrival/departure.
- Charge energy/time prediction error against independently metered AC and DC quantities.
- Reserve-violation false-feasible rate, with false-feasible a hard non-compensatory metric.
- Performance and abstention when route, vehicle, charger or weather evidence is stale or missing.

Use leave-one-trip-out for initial exploration, then leave-one-vehicle-model-out and date/time-blocked holdouts. Never split windows from the same physical journey between train and test. Preserve repeated runs from the same vehicle as a group when their identity is available.

### B. External depot profiles

The NLR heavy-duty depot dataset provides 81 modeled 24-hour profiles at 15-minute resolution: three fleet vocations (beverage, warehouse, food delivery), fleet sizes 10/50/100, three charging strategies (immediate 100 kW, delayed 100 kW, minimum power), and average/minimum/peak days. This is a different validation domain from the passenger-EV N3 corridor; use it for depot/fleet load prediction and charging-policy comparisons, not as a direct passenger-car route-energy label.

Report 15-minute site-load MAE/RMSE, daily energy-integral residual, peak-demand error, departure completion, unserved energy, grid capacity required, and compute/solver time on identical workloads.

### C. Published grid-capacity reduction study

The 2025 Applied Energy vehicle-valet study reports a 64% average grid-capacity reduction in its 30-vehicle case. Treat it as a **replication target**, not our result. A valid comparison must reproduce the study's duty cycles, cost boundary, valet repositioning, service guarantees, infrastructure and control assumptions before comparing percentages. A fixed-power fleet scheduler without vehicle repositioning cannot claim a valet benefit.

### D. Heavy-duty real-world depot challenge

A 2026 SSRN preprint reports more than 400 trucks and nearly 40,000 tours across nine depots with measured site loads. It is a useful future external challenge set, but it is a preprint and must be methodologically reviewed before becoming a benchmark reference. Separate its measured facility/load evidence from modelled route/vehicle assumptions.

### E. Charging interoperability

For future heavy-duty/MCS extensions, map the test plan to SAE J3271:2025 and IEC 61851-23-3:2026 scopes. Simulation of site/vehicle power is not conformance testing. Keep coupler, charging communications, cooling/cable handling, interoperability, grid interface and bidirectional-power tests as distinct qualification lanes.

## Prospective promotion gates

These are targets, not results achieved by this v0.1 corpus:

- 100% deterministic replay of the same dataset bytes and audit version.
- 100% detection of duplicate trip IDs, invalid direction/elevation sign, impossible percentages, non-finite values and malformed source metadata.
- Zero silently corrected or dropped records; inconsistent public records remain visible as findings.
- 100% of report rows carry a source-backed trip identifier, data class and claim ceiling.
- On raw held-out trips, target at least 90% empirical coverage for nominal 90% prediction intervals, reported by model/direction/temperature strata with sample counts.
- Zero false-feasible reserve violations on the frozen adversarial route corpus.
- For planning improvements, compare FCFS, earliest-departure-first, nearest-charger, fastest-route and constrained optimization on identical route, vehicle, charger, weather, evidence and compute inputs.

No state-of-the-art or field-accuracy claim is made by the public-summary audit. Promotion requires permissioned raw signals, source-specific evidence authentication, independent energy metering, frozen train/test splits and external review.

## Reproducible command

    cargo run -p sol-atlas-core --bin ev_n3_published_trip_audit --locked -- --input data/benchmarks/n3_published_trip_summaries_v0_1.json

The generated report binds the exact input bytes with SHA-256 and records its own deterministic report digest. These hashes establish replay identity, not the truth of the publisher's claims.
