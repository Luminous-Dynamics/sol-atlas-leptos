# ENERGY-RELIABILITY-V0.1 — chronological essential-service resilience

**Status:** reference trace auditor and frozen synthetic outage fixture; CI results determine executable verification status.  
**Scope:** time-ordered service requests and declared deliveries, with interval-aligned grid-outage annotations, demand-quality checks, and interruption metrics.  
**Not in scope:** dispatch optimization, asset sizing, causal proof of outage response, network flow, meter calibration, battery mechanism modelling, forecast uncertainty, or the physical energy balance of a generation/storage portfolio.

## Paradigm shift: resilience is service over time

A daily energy total and a nameplate battery capacity do not prove that a critical service survives an outage. Timing matters: energy delivered after a service failure may not undo the failure, and a small amount of unserved energy can hide a long critical interruption.

This v0.1 evaluator treats resilience as a time-indexed service trace. It preserves every interval and separately reports:
- requested, delivered and unserved service;
- service delivery fractions (undefined when the corresponding demand is exactly zero);
- grid outage exposure;
- essential service deficits that coincide with outages;
- longest continuous essential-deficit run and longest continuous outage-deficit run.

The grid-outage flag is a stress annotation, not proof that a site lost power. Behind-the-meter generation or storage may maintain delivery during a grid outage. Conversely, non-outage deficits remain real service failures and must not disappear from the aggregate.

## Contract

### Chronology

Every interval has a UTC Unix start timestamp and a positive duration in seconds. Intervals must be strictly contiguous and ordered: the next start equals the prior start plus duration. Missing/unknown periods must be explicit intervals; a gap cannot silently shorten an outage or split a deficit run. Duplicate identifiers, overlaps, gaps and timestamp overflow fail closed.

This contract assumes one common interval grid for all services. Different measurement cadences must be reconciled by a separately specified resampling step; do not implicitly duplicate or interpolate demand or delivery.

### Service and evidence

Each service has a stable ID, description, essential flag, required energy form and, for heat, a minimum supply temperature. Every interval must carry exactly one sample for every declared service. Request/delivery quantities are finite and nonnegative; delivered energy must not exceed requested energy. Thermal delivery must carry a valid absolute temperature and must satisfy the declared minimum grade. A trace tagged with any evidence class beyond analyst assumption requires an explicit reference.

These are arithmetic and data-quality checks. Evidence metadata, a SHA-256 receipt, or a publisher signature is not proof that meters were calibrated or that the declared source caused the supplied service. An audited historical trace, forecast, simulation and synthetic fixture are different claim types.

### Metrics and denominators

For every service sample:

\[
E_{\text{unserved}} = E_{\text{requested}} - E_{\text{delivered}}
\]

The evaluator reports totals in kWh and interruption durations in seconds. It calculates an essential-deficit interval when the essential shortfall exceeds the declared numerical tolerance. Consecutive durations accumulate only over adjacent contiguous intervals; an interval with full essential service breaks the run. The outage-deficit run also resets whenever the grid-outage flag is false.

Outage essential delivery fraction is:

\[
f_{\text{outage, essential}} =
\frac{E_{\text{delivered, outage, essential}}}
{E_{\text{requested, outage, essential}}}
\]

When there is no essential service demand during flagged outage intervals, the fraction is null/undefined. It is not 100%, zero, NaN or infinity. This metric is quantity-weighted, not a claim that every critical load was uninterrupted; use interval deficit counts and durations alongside it.

Unserved service is not additional energy. This module has no generation/storage energy ledger, so its delivered totals must not be added to the source-to-service or port graph ledger as extra energy flows. When connected to a future controller/dispatch evaluation, match exact meter/asset boundaries and preserve storage state transitions separately.

## Frozen synthetic known answer

Four contiguous one-hour intervals, one essential electricity service:

| Interval | Grid outage | Requested | Delivered | Unserved |
|---|---|---:|---:|---:|
| 1 | No | 10 kWh | 10 kWh | 0 kWh |
| 2 | Yes | 10 kWh | 8 kWh | 2 kWh |
| 3 | Yes | 10 kWh | 0 kWh | 10 kWh |
| 4 | No | 10 kWh | 10 kWh | 0 kWh |
| **Total** | **2 outage hours** | **40 kWh** | **28 kWh** | **12 kWh** |

Known answers:
- Total duration 14,400 seconds; grid-outage duration 7,200 seconds.
- Outage essential request 20 kWh, delivery 8 kWh, unmet 12 kWh, delivery fraction 0.4.
- Two essential-deficit intervals and 7,200 seconds of deficit.
- Longest consecutive essential deficit 7,200 seconds; longest outage-essential deficit 7,200 seconds.
- Recovery in interval 4 resets the consecutive-deficit run.
- All service and aggregate balance residuals equal zero within numeric tolerance.

This fixture is synthetic and deterministic. It proves accounting behavior, not the resilience of any actual solar/storage system or the duration of a real outage.

## Fail-closed controls and acceptance benchmarks

| Gate | v0.1 acceptance target |
|---|---|
| Frozen outage fixture | 40 requested = 28 delivered + 12 unserved kWh |
| Outage-only deficit accounting | outage request 20 = delivery 8 + unmet 12 kWh |
| Time coverage | every interval strictly contiguous; gaps and overlaps reject |
| Sample completeness | every service exactly once per interval; missing, unknown or duplicate IDs reject |
| Upper bound | delivery greater than requested rejects; never cap silently |
| Heat quality | inadequate heat temperature or missing delivery temperature rejects any positive claimed heat delivery |
| Outage semantics | outage flag plus full delivery reports no outage deficit; non-outage deficits remain in all-trace metrics |
| Zero denominator | no essential outage demand produces a null fraction |
| Deficit duration | consecutive runs reset after service recovery or any non-outage interval for the outage-specific run |
| Reproducibility | raw request and serialized assessment hashes are recorded; synthetic and evidence ceilings remain visible |
| Numerical safety | negative, non-finite, overflowed aggregate or invalid duration values reject |
| Complexity | use indexed service samples; target O(I × S log S) time and O(I × S) report storage for I intervals and S services |

Next add property-based traces and a separate performance benchmark at 8,760 hourly intervals, multiple services, and sub-hourly outage windows. Performance targets must record hardware, toolchain, sample size and measurement method; never report a speedup from an unpinned run.

## Industry comparison: where this layer fits

This trace auditor complements rather than replaces optimizers.

- **NREL REopt** — site/community/microgrid technology sizing and techno-economic optimization for cost, resilience and energy performance. Its stable API is V3 and uses REopt.jl as the optimization engine. For comparison, feed the same load, outage window, asset assumptions and critical-load floor to REopt; compare reported resilience against independently computed interval service shortfall. Sources: [REopt platform](https://www.nlr.gov/reopt), [REopt API V3](https://developer.nlr.gov/docs/energy-optimization/reopt/v3/).
- **PyPSA** — chronological generation, sector-coupling, storage and linearized power-flow optimization at network scale. Its storage balance equations explicitly evolve state of charge across snapshots and apply elapsed-time standing losses. Compare on identical input snapshots, storage initial state, final reserve policy and service floors. Sources: [PyPSA docs](https://docs.pypsa.org/latest/), [storage constraints](https://docs.pypsa.org/latest/api/networks/constraints/).
- **GenX** — constrained linear/MILP planning and dispatch across generation, storage, transmission, demand-side resources and reserve constraints. It is a relevant system-planning comparator when the question expands beyond a single site. Sources: [GenX workflow](https://genxproject.github.io/GenX.jl/stable/User_Guide/workflow/), [model introduction](https://genxproject.github.io/GenX.jl/stable/Model_Concept_Overview/model_introduction/).
- **Eskom demand data** — public historical/system forecast traces can contextualize South African scenarios, but Eskom notes that official-metering-derived data are subject to verification. They are not automatically independent site-load or fully audited microgrid data. Source: [Eskom demand-side portal](https://www.eskom.co.za/dataportal/demand-side/).

### Matched comparison protocol

1. Freeze and hash load traces, time zone/UTC boundaries, outage windows, weather, tariffs, generation availability, storage initial and terminal state, and technology constraints.
2. Separate forecasts, synthetic stress tests and held-out measured intervals; never leak held-out outages into controller tuning.
3. Compare essential unserved kWh, interval deficit count, longest deficit, outage-only deficit, cost, emissions, curtailment, reserve shortfall, runtime and solver status.
4. On tiny synthetic cases, enumerate all feasible dispatch choices and compute exact service-regret against the oracle. On larger cases, report solver gap/status, infeasibility and model limitations.
5. Stress test unseen outage lengths, clustered outages, cloudy periods, heat-grade constraints, time-step changes, and sensor missingness.
6. Publish Pareto fronts over cost, essential-service quality, carbon, water, exergy and resilience. An overall mean delivery fraction is insufficient when a short failure of an essential service dominates the consequence.
7. Claim superiority only on predeclared held-out cases with identical constraints and measurements, and show confidence/robustness bounds. This deterministic trace evaluator itself is not a competitor to REopt, PyPSA or GenX.

## Next steps

1. Connect storage discharge/reserve intervals to the service trace through explicit node/port/source IDs and preserved timestamps.
2. Add service floors by interval, minimum uninterrupted duration, service priority, asset location and outage clusters.
3. Build scenario generation for PV/weather uncertainty and correlated grid outages; report worst-case and percentile service rather than just expected output.
4. Run a South African/Johannesburg benchmark using appropriately licensed public profiles, followed by lawful held-out site load/meter traces when available.
5. Add exact microgrid dispatch enumeration for tiny cases, then compare a controller and REopt/PyPSA/GenX on matched input envelopes.
