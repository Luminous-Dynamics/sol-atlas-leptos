# ENERGY-THERMAL-EXERGY-V0.1 — explicit energy quality at a declared boundary

**Status:** reference implementation and frozen synthetic fixture; CI results determine executable verification status.  
**Scope:** one isothermal heat quantity crossing a declared boundary; reference environment is an explicit scenario input.  
**Not in scope:** finite-body cooling curves, non-isothermal profiles, refrigeration/cold exergy, device maps, network dispatch, plant efficiency, lifecycle economics, or physical metering.

## Paradigm shift

First-law accounting answers how much energy crosses a boundary. It does not answer how much reversible work could ideally be obtained from that energy under a stated environment. A kWh of electricity and a kWh of low-temperature heat have equal energy units but are not interchangeable services.

This evaluator adds a distinct thermal-exergy quantity without replacing energy conservation or reducing a multiobjective portfolio to one opaque score.

## Definition and boundaries

For heat quantity Q supplied isothermally at temperature T to a system with explicit reference-environment temperature T0:

\[
B = Q\left(1-\frac{T_0}{T}\right)
\]

T and T0 are absolute temperatures in kelvin. The JSON API accepts Celsius and records both the Celsius inputs and converted kelvin values. The supported case requires T >= T0 > 0 K. T = T0 and Q = 0 yield zero thermal exergy. T < T0 is rejected in v0.1; it must not be silently interpreted as zero, negative exergy, or ordinary heat above ambient.

The result is an idealized reversible-work upper bound for the declared isothermal stream. It is **not** predicted electrical output, achieved work, device efficiency, a heat-pump COP, economic value, or proof of a measured source. Non-isothermal sensible streams need integration over their temperature-dependent heat transfer; applying this formula to the whole stream without an explicit approximation is out of scope.

The field energy_minus_exergy_kwh is a quality distinction, not a dissipative loss. Do not add it to pathway conversion losses. Exergy and energy remain separate and must not be counted as additional energy inputs/outputs.

## Contract and refusal behavior

- Every scenario supplies both the heat temperature and reference-environment temperature; there is no hidden ambient default.
- Absolute temperatures must be finite and strictly positive after conversion to kelvin.
- Heat quantity must be finite and nonnegative; exergy fraction is constrained to [0, 1].
- Cold streams, non-finite derived values, unsupported schema versions and unknown JSON fields fail closed.
- Any evidence class beyond analyst assumption needs a non-empty source reference. Metadata is not authenticated by this evaluator.
- The CLI receipt hashes the exact raw input bytes and compact serialized assessment. A digest binds identity, not the truth of evidence or physical behavior.
- The report carries an explicit claim ceiling and the synthetic flag through the receipt.

## Frozen synthetic known answer

Fixture: 40 kWh of heat at 80 C and a 20 C reference environment.

- T = 353.15 K; T0 = 293.15 K.
- Exergy fraction = 1 - 293.15/353.15 = 60/353.15 = 0.16989947614328194.
- Idealized thermal exergy = 40 × 60/353.15 = 6.795979045731277 kWh.
- Energy quantity minus exergy = 33.20402095426872 kWh (a quality gap, **not a loss**).
- Ambient-temperature heat is a zero-exergy negative control; zero heat produces zero exergy.
- Lower temperature-grade heat must not rank above higher-grade heat at the same Q and T0 under this formula.

The fixture is synthetic and analytical. It establishes equation handling and schema behavior, not a plant performance result.

## Benchmarks and acceptance gates

| Gate | Acceptance target | Why it matters |
|---|---|---|
| Hand-derived known answer | absolute error <= 1e-9 kWh at unit-scale quantities | Catches unit, temperature conversion and formula errors |
| Exergy factor | 0 <= factor <= 1 for supported inputs | Prevents impossible quality factors |
| Ambient and zero-flow controls | B = 0 | Catches hidden ambient defaults and fictitious energy |
| Invalid absolute temperatures | reject T <= 0 K or T0 <= 0 K | Prevents invalid thermodynamic states |
| Cold stream | reject in v0.1 with an explicit error | Avoids silently misclassifying refrigeration/cold exergy |
| Evidence ceiling | non-assumption without source ref rejected; synthetic stays synthetic | Prevents assumptions turning into plant claims |
| Numerical stress | non-finite inputs and non-finite derived values rejected | Preserves fail-closed behavior |
| Temperature sensitivity | at fixed Q and T0, B is monotonic in T; factor remains in [0,1] | Checks qualitative thermodynamic behavior |
| Property-based follow-up | generated valid/invalid ranges, scale-aware tolerance and oracle checks | Broadens beyond a handful of fixtures |
| Profile convergence follow-up | compare declared discretized temperature profiles to an integral reference with predeclared error bounds | Prevents use of the isothermal approximation outside its domain |

For floating-point regression, use absolute tolerance for unit-scale values and a declared scale-aware rule for large magnitudes. Do not hide meaningful residuals behind an arbitrary large global tolerance.

## Industry comparison: match the problem before comparing results

- **SAM:** compare compatible component/plant cases and boundary-level energy outputs where SAM exposes the required quantities. Do not imply this small formula replaces SAM's technology-specific models.
- **REopt:** use for site sizing/dispatch, cost and outage-resilience baselines. Exergy can be added as an explicit reporting dimension; this oracle is not an optimizer.
- **PyPSA:** use for chronological generation, storage, sector coupling and network portfolios. Preserve time, carrier, temperature grade, curtailed supply and unserved service at compatible boundaries.
- **Independent reference:** where possible compare the heat-exergy calculation against a separately written analytical reference, then compare the downstream portfolio outputs under identical inputs and constraints.

Every comparison record should pin tool/version, source data, weather/load interval, boundary, units, assumptions, costs, constraints, time resolution, objective, solver tolerance and input digest. Report discrepancies and missing fields rather than silently imputing them.

## What would count as beating the state of the art?

This module alone cannot establish that. A credible claim needs a repeatable advantage on a matched held-out workload, such as fewer conservation/evidence errors, better calibrated uncertainty, lower decision regret against an exact small-instance oracle, comparable or better solve time, or materially better useful-service delivery under the same cost and constraint budget. Report Pareto fronts for cost, service delivered, exergy, carbon, water, critical materials and resilience; do not compress them into a single arbitrary score.

## Roadmap

1. Couple this oracle to the heat-pump result while preserving its declared delivery temperature.
2. Add typed multi-output ports so cogeneration/recovered heat is represented once and can feed a downstream node without also being counted as a loss.
3. Add an explicit profile/integral contract, then network and chronological dispatch.
4. Compare selected cases with SAM, REopt and PyPSA under matched boundaries and preserve raw inputs/results as digest-bound evidence.
5. Train neither the model nor the benchmark on evaluation cases; publish frozen synthetic known answers and held-out measured records separately.
