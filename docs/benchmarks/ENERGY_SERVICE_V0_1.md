# ENERGY-SERVICE-V0.1 — demand-matched useful energy and service deficits

**Status:** reference implementation and frozen synthetic known-answer fixture; CI is the executable verification authority.  
**Scope:** one time-boundary graph assessment; each terminal useful-service output is bound to one explicit demand with energy-form and, for heat, minimum-temperature requirements.  
**Not in scope:** chronological reliability, network location/congestion, voltage/frequency quality, pressure/flow constraints, indoor comfort dynamics, service scheduling, plant metering, or optimization.

## Paradigm shift

A first-law-balanced portfolio is not automatically useful. It can produce a physically valid quantity of energy in the wrong carrier, at the wrong temperature, at the wrong time/place, or in excess of the actual load. An output label cannot establish usefulness.

This layer binds a terminal output to a stated consumer demand. It preserves two independent ledgers:

1. **Physical energy ledger:** energy crossing node boundaries, internal transfers, terminal disposition and graph balance.
2. **Service ledger:** requested quantity, compatible delivered service, unserved amount and essential-service deficits.

Unserved demand is **not** an energy flow and is never inserted into the physical balance. Service satisfaction is assessed against a declared requirement; it is not proof from a meter, nor an optimizer's predicted dispatch.

## Contract and fail-closed rules

- A service request has a unique ID, readable description, essential/flexible flag, required energy form, nonnegative requested quantity, and optional explicit source output.
- Thermal-heat demand must provide a minimum supply temperature in Celsius above absolute zero. Nonthermal demand must not specify a temperature in v0.1.
- Each demand has at most one terminal source output. Each terminal output marked `useful_service` must be linked to exactly one demand. Split an output into separate graph ports when it serves different demands or when surplus must be rejected/curtailed.
- Source output form must match the demand form. Delivered heat must meet the declared minimum temperature grade.
- Delivered quantity comes from the graph's un-routed terminal remainder. A source cannot deliver more than the demand requests; an over-sized service output must be explicitly split or the scenario is rejected.
- A demand without a declared compatible source is reported with zero delivery and full shortfall. The evaluator does not guess a connection.
- Unknown fields, duplicate demand IDs, duplicate output claims, absent useful-service links, invalid values, incompatible carrier, invalid temperatures and insufficient grade fail closed.
- Service data must refer to the same scenario ID and synthetic/evidence envelope as the graph.
- Both graph and service assessment balances are reported. The service residual is requested - delivered - unserved. The graph balance is independently checked before service matching.
- A SHA-256 receipt binds exact raw request bytes and the compact serialized assessment. It does not establish truth, calibration, load criticality, or that the declared service was met.

The `essential` flag only partitions reporting in v0.1; it is not yet a reliability guarantee or an optimization objective. Time, location, outage exposure, minimum continuous runtime, and service-quality envelopes are later constraints.

## Frozen synthetic known answer

A synthetic conversion node receives 100 kWh of chemical energy and outputs:

| Terminal output | Quantity | Disposition |
|---|---:|---|
| Electricity | 35 kWh | Useful-service candidate |
| Process heat | 14 kWh at 80 C | Useful-service candidate |
| Exhaust heat | 51 kWh at 150 C | Rejected |

Demand claims:

| Service demand | Requested | Delivered | Unserved | Quality |
|---|---:|---:|---:|---|
| Essential site electricity | 40 kWh | 35 kWh | 5 kWh | Electricity matches |
| Essential process heat | 20 kWh | 14 kWh | 6 kWh | 80 C supply satisfies >=75 C minimum |
| **Total** | **60 kWh** | **49 kWh** | **11 kWh** | Service residual = 0 |

Physical closure remains: 100 kWh external input = 35 + 14 + 51 = 100 kWh terminal disposition. The 11 kWh service deficit does **not** create or delete physical energy. The fixture proves accounting-contract behavior only; it is not plant evidence.

## Acceptance benchmarks

| Benchmark | v0.1 target |
|---|---|
| Service known answer | 60 requested = 49 delivered + 11 unserved kWh |
| First-law independence | Graph remains 100 in = 100 terminal out despite 11 kWh service shortfall |
| Demand accounting | Each row: requested = delivered + unserved within scale-aware tolerance |
| Useful port claims | Every useful-service terminal output has exactly one demand link |
| Demand/output overclaim | Reject if output remainder exceeds demand; require an explicit surplus/rejected port |
| Carrier matching | Reject electricity-to-heat or other wrong-form assignment |
| Thermal grade | 80 C may serve a >=75 C demand; must not serve >=85 C |
| Duplicate claims | One output cannot be counted against two service IDs |
| Missing connection | Entire request remains unserved; no implicit routing or zero-filled connection |
| Essential floor | Report requested, delivered and unserved essential service independently from total service |
| Numerical bounds | Nonnegative finite quantities and non-finite aggregate/derived values reject |
| Reproducibility | Input bytes and result have recorded SHA-256 digests; the synthetic flag remains explicit |

Next expand tests with property-generated demand/output quantities, multiple services and output splits. Then add chronology and location, and compare service shortfall under outages against an independent solver or exact small-instance oracle.

## Industry comparison: compete at the right layer

This v0.1 layer is **not an optimizer**. It adds service-claim auditing to optimization/plant models; it does not replace them.

- **NREL SAM** (https://sam.nlr.gov/): a plant/component techno-economic comparator for PV, batteries, CSP, process heat and other power technologies. Use matched technology, weather, cost, and energy boundaries.
- **NREL REopt** (https://developer.nlr.gov/docs/energy-optimization/reopt/): optimize an appropriate facility or microgrid for cost and resilience, then independently assess whether outputs satisfy declared services and outages. REopt API V3 uses REopt.jl as the optimization engine, per the current API docs.
- **PyPSA** (https://docs.pypsa.org/latest/): network-scale, chronological optimization for sector coupling and storage. Its roadmap already lists a multi-port Process component with explicit conversion rates (21 April 2026) and process/link delays (17 February 2026). Therefore **multi-port modelling alone is not a defensible novelty or superiority claim**. Differentiation needs independent invariants, matched service floors, provenance/claim ceilings, fail-closed missing data, adversarial tests, and measured decision-quality improvements.
- **PVGIS / European Commission JRC** (https://photovoltaic-geographic-information-system.ec.europa.eu/api/v6/docs): an open source of hourly solar resource and PV estimates, with current public coverage described as 2013–2024. The v6 API release notes record a 23 July 2026 response-format break; every benchmark must pin API version, request, response bytes and digest.
- **Eskom Data Portal** (https://www.eskom.co.za/dataportal/demand-side/): official metering-derived weekly demand and recent hourly actual/forecast traces. The portal explicitly states data remain subject to verification. Preserve that caveat and pin retrieval time; don't treat forecast, metering and independently audited records as equivalent evidence classes.
- **CSIR South African power-generation statistics** (https://www.csir.co.za/media-hub/media-room/csir-releases-statistics-power-generation-south-africa-2024): aggregated output/capacity comparisons across coal, nuclear, hydro, pumped storage, OCGT, wind, PV, CSP and embedded/private PV. It is system context, not a substitute for site-level interval load data.

### Matched evaluation protocol

1. Freeze location, time zone, weather and load intervals, tariffs, emission factors, asset costs/lives, converter boundaries, storage openings/reserves, outage traces and source data digests.
2. Keep training/calibration cases separate from held-out evaluation days and sites. Preserve raw inputs, tool versions, solver configuration and returned output.
3. Compare on the same scope: cost/NPV, essential-service energy not served, hours below service floors, curtailment, losses, emissions, water, materials, resilience runtime, solve time and reproducibility.
4. For bounded synthetic cases, enumerate the exact feasible set and report optimality gap/regret. For larger cases, report the solver's optimality status/tolerance and infeasibility, not just the objective value.
5. Publish Pareto fronts across cost, carbon, water, essential service, exergy and resilience. Don't hide trade-offs in an arbitrary single "sustainability" score.
6. Claim to outperform a comparator only on a predeclared, held-out, matched-boundary metric with confidence intervals or robust-case analysis. An accounting test passing is not an industry-performance win.

## Next steps

1. Extend service contracts to hourly intervals, service location, criticality and minimum duration.
2. Connect explicit demand matching to stateful storage and thermal-exergy outputs.
3. Ingest signed and calibrated metering as separate evidence layers; signatures identify publishers, not measurement truth.
4. Build a pinned Johannesburg/South Africa benchmark with public weather estimates and system-demand traces, then site-specific load records where lawful.
5. Compare optimized portfolios against REopt/PyPSA and matched SAM plant cases; retain a simple exact oracle for small instances.
