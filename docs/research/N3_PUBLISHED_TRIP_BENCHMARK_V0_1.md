# CHARGE N3 published-trip benchmark v0.1

**Programme:** Sol Atlas EV route assurance (#664)  
**Primary source:** https://charge.co.za/n3-real-world-ev-tests/  
**Corpus:** data/benchmarks/n3_published_trip_summaries_v0_1.json  
**Evidence class:** operator_published_summary  
**Status:** public-summary consistency harness; not raw-data model validation

## Why this is useful

CHARGE describes its N3 route as a public real-world EV benchmark. Its current page lists 95 BEV models available in South Africa, of which 32 are shown as tested on the N3. The published trip cards contain direction, distance, net elevation change, total ascent/descent, average speed, reported pack capacity, battery used, energy efficiency and range. Several cards also show charging-stop energy and temperature summaries, but the public presentation does not consistently establish the session-boundary and battery-side energy semantics needed to join those numbers safely.

The N3 is a geographically fixed 185 km corridor with opposing uphill/downhill profiles. That gives us a useful challenge set for terrain-conditioned prediction and route reserve assessment, beyond synthetic drive cycles. It is not a raw CAN/BMS, GPS, charger meter, or environmental time-series dataset.

This v0.1 corpus contains 26 individually transcribed trip cards across 15 make/model labels, retrieved on 27 November 2026. It includes the later September Geely and iCAUR cards and the 7 and 9 November Alfa Romeo Junior Elettrica runs. Identifiers are local anonymized labels; publisher run IDs and charging transaction IDs are excluded from the local primary keys. The sample is curated from the visible public cards, not sampled randomly, and is not statistically representative of the national fleet.

## Executable audit and source completeness

For each trip the auditor checks:

1. Published-efficiency energy proxy: energy_kwh = (kWh_per_100km × distance_km) / 100.
2. Nominal-capacity/SOC proxy: capacity_soc_proxy_kwh = pack_capacity_kwh × battery_used_percent / 100.
3. Absolute and relative residual when both source fields are actually present.
4. Published range consistency: distance_km / (battery_used_percent / 100), compared with the source's full-charge range field.
5. Ascent minus descent against net elevation change, and the consistency of direction with the sign of that change.
6. Per-make/model/direction counts and min/median/max published energy intensity, using only rows with an actual published efficiency value.

The current frozen corpus has 24 rows where both nominal pack capacity and published energy efficiency are present, and 2 rows where that pairwise comparison is unavailable. Of the 24 matched rows, 23 fall within the display-rounding screen and 1 exceeds it. All 26 rows publish enough information for the current range proxy check. Missing source fields remain null and are counted as unavailable; the auditor does not impute them, silently drop the trips, or count them as passes.

The visible iCAUR V23 uphill card (24 September 2026) reports 49% battery used, 59.9 kWh pack size and 17.16 kWh/100 km. The derived quantities differ by approximately 2.395 kWh, or 7.54% of the efficiency-derived energy. The auditor preserves this as `EXCEEDS_PUBLISHED_ROUNDING`; it does not decide which source field is wrong. Nominal nameplate capacity, usable capacity, SOC calibration and the source's energy calculation may not share the same basis. This is a useful negative result: the benchmark must expose disagreements rather than assume all co-published metrics reconcile.

The maximum range residual across the 26 cards remains approximately 0.48 km. These are consistency residuals among operator-published fields, not model errors or physical measurement accuracy.

Default tolerances are 0.10 kWh and 0.50% for the energy pair, and 2 km for the range proxy. They are explicit display-rounding screens, not empirical uncertainty bounds.

**Critical interpretation:** pack capacity and battery-used percentage may participate in the same upstream calculation that produces published kWh/100 km and range. A close residual can therefore show internal consistency without independently validating battery-side energy, usable capacity, SOC accuracy, traction energy or a vehicle model. The report's claim ceiling says this explicitly.

## What is not in v0.1

- Raw vehicle/CAN/BMS or GPS time series.
- Independent traction-battery DC energy measurement.
- Metered charger AC input, EVSE DC output, efficiency or taper curve.
- Charge-session start/end SOC with unambiguous trip/session identity and energy-meter boundary.
- Ambient wind/precipitation, traffic, tyre pressure, payload, HVAC, road-surface or driver-control traces.
- Calibrated battery usable capacity, pack SOH, thermal state or a vehicle-specific energy model.

Do not infer charger availability or charge delivery from a station label. Do not join charger-session kWh to a trip just because the values appear together on the page. The session boundary, direction, meter point and SOC endpoints must be resolved from operator evidence before such an association is admitted.

## Four-project integration ownership

- **Symthaea:** qualified pack capability, cell/module/pack uncertainty and battery observations. Do not infer pack power from a scalar SOH value.
- **Symtropy:** exact EV configuration, vehicle energy model, SOC/taper curves, per-leg energy estimate and drive-cycle validation.
- **Mycelix:** vehicle/pack identity, component/configuration lineage and lifecycle provenance.
- **Sol Atlas:** N3 route geometry, elevation context, site/charger identity and spatial/grid evidence.
- **Evidence admission:** the issuer and source-artifact digest must be independently qualified. A public webpage snapshot is not a BMS attestation or a meter record.

## External benchmark landscape

### 1. NLR heavy-duty depot profiles

The National Laboratory of the Rockies (formerly NREL) catalog publishes 81 modeled 24-hour depot-load profiles at 15-minute resolution. The matrix covers beverage, warehouse and food-delivery tractor fleets; sizes of 10, 50 and 100 EVs; immediate 100 kW, delayed 100 kW and minimum-power charging strategies; and average, minimum and peak days. The public catalog also links the profile-generation code and an Oncor integration-results workbook.

Source: https://data.nlr.gov/submissions/162  
Open data: https://data.openei.org/submissions/8223  
Generator: https://github.com/NREL/hdev-depot-charging-2021  
Dataset DOI: 10.7799/1787031  
Related Nature Energy paper DOI: 10.1038/s41560-021-00855-0

This is a separate heavy-duty fleet/depot domain, not a passenger-EV N3 label set. Use it for charging-load prediction, site constraints and policy comparisons. Report 15-minute site-load MAE/RMSE, daily energy-integral residual, peak-demand error, departure completion, unserved energy, grid capacity required and compute/solver time on identical workloads.

### 2. Grid-capacity reduction and valet charging

Passow and Rajagopal's Applied Energy 2025 study compares low-capital-cost depot strategies. In its 30-vehicle depot scenarios, adding one valet reduced grid capacity required per vehicle by 64% on average, with 5th/95th percentiles of 55%/73%; reported levelized cost of charging decreased by 13%, with 8%/24% bounds. This is a replication target, not a result achieved by Luminous Dynamics.

Source: https://doi.org/10.1016/j.apenergy.2025.126563

A valid comparison must reproduce the duty cycles, valet repositioning, cost boundary, service guarantees, plug/power-electronics assumptions and charging controls. A scheduler that does not move vehicles on/off plugs cannot claim the paper's valet benefit. Compare FCFS, earliest-departure-first, minimum-power, plug sharing, valet-enabled scheduling and constrained optimization with identical fleet workloads and service constraints.

### 3. Real-world heavy-duty depot operations

A September 2026 SSRN preprint reports an analysis involving more than 400 heavy-duty trucks and nearly 40,000 tours across nine depots over more than three months, combining simulated fleet charging with measured building loads and site-specific grid connection capacities. It is a promising external challenge, but it is a preprint; its methods, licensing, data access and separation between measured and simulated variables must be reviewed before formal benchmarking.

Source: https://papers.ssrn.com/sol3/papers.cfm?abstract_id=7543981

### 4. Megawatt charging interoperability

IEC 61851-23-3:2026 was published on 21 August 2026. It specifies MCS EV supply equipment with an IEC TS 63379 coupler and describes voltage envelopes of up to 1,000 V AC / 1,500 V DC on the supply side and up to 1,250 V DC on the EV side. The standard notes that bidirectional power-flow requirements are under consideration and points to ISO 15118-10 and ISO 15118-20 for digital EVSE/EV energy-transfer communications.

Source: https://webstore.iec.ch/en/publication/69016

SAE J3271_202503 is an SAE Technical Information Report issued on 5 March 2025, organized around coupler, communications/controls, cables/cooling/automated connection, grid and bidirectional use cases, and interoperability test procedures.

Source: https://saemobilus.sae.org/standards/j3271_202503-sae-megawatt-charging-system-electric-vehicles

Simulation of site/vehicle power is not MCS conformance testing. Keep coupler, thermal/cooling, cable handling, communication, interoperability, grid-interface and bidirectional-power qualification as separate evidence lanes.

### 5. Upcoming forecasting literature watch

A publisher record for DOI 10.1016/j.eswa.2026.134259 currently lists an issue date of 15 January 2027, which is after this corpus retrieval date; it is therefore a forthcoming-literature watch item, not a current completed baseline. Its preview describes a demanding EV forecasting evaluation with strictly future 30-second targets, segment-disjoint training/validation/calibration/test sets, five prespecified seeds, chronological/cross-trip/cross-context/cross-dataset regimes, split-conformal uncertainty and policy-level evaluation. It reports that tree ensembles outperform the compared deep sequence models within-domain, while direct cross-dataset transfer fails. We should adopt the evaluation controls as a challenge design, then verify the final paper and data before making a numerical SOTA comparison.

Publisher record: https://www.sciencedirect.com/science/article/pii/S0957417426031659

## What would beat today's practice

The main opportunity is not another isolated RMSE leaderboard. It is an evidence-qualified, decision-oriented EV digital twin in which a route/depot plan is only actionable when the vehicle, pack, charger, site, road and weather claims have provenance, freshness and uncertainty.

The research progression should be:

1. **Reproduce first:** matched baselines on the exact same route, vehicle, duty cycle, weather, pack, charger, evidence and compute inputs. Include persistence/moving-average, linear/physics baselines, LightGBM/CatBoost/XGBoost, and GRU/LSTM/TCN/Transformer families where data justifies them.
2. **Prove transfer:** use leave-one-trip-out only for early debugging, then leave-one-vehicle-model-out, time-blocked, route-blocked and depot-held-out tests. Windows from the same physical trip, vehicle campaign or charging session must never be split across train/test roles.
3. **Make uncertainty decision-safe:** calibrate 50/80/90/95% intervals on a disjoint calibration set. Report PICP, interval width, WIS/CRPS where supported, and coverage by uphill/downhill, speed, temperature, vehicle model and pack-age strata. A nominal 90% marginal interval that undercovers high-demand tails is not adequate for reserve decisions.
4. **Treat false-feasible as non-compensatory:** report reserve-violation false-feasible count/rate and worst-case underestimation separately. Do not trade it away for lower average MAE. Unknown/stale route, pack, charger or grid evidence must lead to an explicit indeterminate state.
5. **Optimize the whole operating system:** jointly evaluate route choice, charging-stop selection, SOC taper, charger queues, depot scheduling, grid import, PV/storage, bidirectional constraints and operational windows. Compare to greedy baselines and published strategies with matched service guarantees.
6. **Price externalities:** report user time, queueing, energy loss, degradation proxy, demand charges, grid upgrade capacity, service completion and compute/solver cost—not just kWh/100 km.
7. **Make every result replayable:** frozen input bytes, schema/version, trust-store digest where applicable, model/configuration SHA, code SHA, split manifest, seeds, solver settings and machine profile must be carried into the report.

## Prospective promotion gates

These are targets, not results achieved by the current public-summary harness:

- 100% deterministic replay of the same dataset bytes and audit version.
- 100% detection of duplicate trip IDs, invalid direction/elevation sign, impossible percentages, non-finite values and malformed source metadata.
- Zero silently corrected, imputed or dropped records; missing fields remain visible and coverage is reported.
- 100% of output rows carry a source-backed ID, evidence class and claim ceiling.
- On permissioned raw held-out trips, target at least 90% empirical coverage for nominal 90% prediction intervals, including high-demand and uphill strata with adequate sample counts.
- Zero false-feasible reserve violations on the frozen adversarial route corpus; any non-zero result blocks promotion.
- On the matched depot workload, replicate the published valet result before claiming improvement; then seek a better Pareto frontier across grid capacity, fleet service, time, cost and degradation.
- For MCS, no standards/conformance claim without lab evidence for the relevant coupler, communications, thermal, mechanical and grid-interaction requirements.

No state-of-the-art or field-accuracy claim is made by this public-summary audit. Promotion requires permissioned raw signals, source-specific evidence authentication, independent energy metering, frozen group/time splits and external review.

## Reproducible command

    cargo run --manifest-path sol-atlas-core/Cargo.toml --bin ev_n3_published_trip_audit --locked -- --input data/benchmarks/n3_published_trip_summaries_v0_1.json

The generated report binds the exact input bytes with SHA-256 and records its deterministic report digest. These hashes establish replay identity, not the truth of the publisher's claims.
