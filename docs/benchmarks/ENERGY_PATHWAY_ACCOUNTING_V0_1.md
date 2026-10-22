# Energy pathway accounting v0.1 — benchmark contract

**Purpose:** a small, deterministic, fail-closed single-stream accounting oracle that provides a physically conservative substrate for later generation portfolios.  
**Status:** implemented on a draft stacked branch; test results are not claimed until executed on the exact head.  
**Schema:** Rust API in `sol-atlas-core::energy_pathway`.

## Why this vertical slice first

Industry-standard tools such as NLR's System Advisor Model (SAM) cover technology performance and financial modelling for PV, batteries, CSP, industrial process heat, wind, marine energy, geothermal, and biomass. REopt optimizes distributed generation/storage/load combinations for cost and resilience. PyPSA can simulate and optimize networks, sector coupling, storage and expansion planning. Our goal is not to replace these validated domain tools with a generic scalar model. This module supplies a small, deterministic **accounting oracle and evidence boundary** that can become a shared regression layer and later be cross-checked against those tools.

- SAM: https://sam.nlr.gov/
- REopt overview: https://www.nlr.gov/reopt/about
- PyPSA features: https://pypsa.org/
- NLR Electricity ATB 2025 update, released 2026-09-02: https://www.nlr.gov/news/detail/program/2026/electricity-annual-technology-baseline-2025-update-now-available
- IEA Electricity 2026: grid flexibility, storage and demand response: https://www.iea.org/reports/electricity-2026/flexibility

## Model boundary

One source energy stream passes through an ordered chain of explicitly typed stages. Supported forms are electricity, thermal heat, mechanical, chemical and radiant energy. Every stage records:

- input/output form and dimensionless gross conversion efficiency;
- a required hard input-throughput limit for the scenario horizon;
- explicit auxiliary electricity per kWh of accepted input;
- evidence class, reference and date when the class is stronger than an analyst assumption.

It reports stage input accepted/curtailed, gross output, conversion loss and auxiliary-electric demand separately. For an electrical final output, net electricity is gross delivered output minus auxiliary electrical demand; that result can be negative. The primary-stream accounting identity is:

`source energy = total conversion loss + total curtailed energy + gross delivered energy`

within the declared scalar-stream boundary and numeric tolerance. Auxiliary electricity is a separate external input and is therefore not silently subtracted from thermal energy.

This is not yet a stateful battery/thermal store simulator, heat-pump/COP model, AC power-flow solver, dynamic thermal model, uncertainty propagator, cost model or plant reliability model. It will reject thermal-to-electric stages without explicit hot/cold reservoirs or efficiency above the Carnot upper bound. Heat pumps (COP > 1) require a separate multi-stream contract because electricity input and ambient/source heat must both be counted.

A valid source reference is not proof that a source is true; a signed telemetry record is not calibration; a calculated result is not a field measurement. All results carry the claim ceiling: `calculated scenario; no physical meter verification or source authentication`.

## Correctness targets (gates, not claimed measurements)

| Gate | Acceptance target |
|---|---|
| Stage conservation | `available_input = accepted_input + curtailed_input` within 1e-9 kWh on bounded analytic fixtures |
| Converter conservation | `accepted_input = gross_output + conversion_loss` within 1e-9 kWh on bounded analytic fixtures |
| Whole-path balance | `abs(stream_balance_residual_kwh) <= 1e-9` on unit-scale analytic fixtures |
| Carnot | Every declared thermal-to-electric conversion must be <= (1 - T_c/T_h), with temperatures in kelvin |
| No energy creation | Zero source cannot create non-zero gross output |
| Capacity limits | Stage throughput cannot exceed its explicit limit; excess is curtailed and accounted |
| Evidence boundary | Non-assumption evidence class without a reference fails closed |
| Determinism | Repeated assessment of the same typed request serializes to byte-identical JSON |
| Invalid input | NaN/infinity, negative input, invalid efficiencies, duplicate stage IDs, mismatched energy forms and invalid reservoir temperatures must be rejected |
| Performance | Record p50/p95/max and throughput on the same pinned toolchain/host; do not set a speed target before a baseline exists |

The 1e-9 kWh targets apply to small analytic fixtures, not unbounded arbitrary magnitude floats. Add scale-aware tolerances before accepting very large MWh/GWh values into the same precision contract.

## External comparison plan

1. **Local correctness baseline:** analytically solve 1–5 stage fixtures, with independent expected values and negative controls.
2. **Technology modelling baseline:** build matched PV/battery, CSP/process heat, and waste-heat examples in SAM when a compatible input schema/model exists; compare each model's supported system boundary rather than forcing feature equivalence.
3. **Portfolio baseline:** compare against REopt for facility-level technology sizing/dispatch and against PyPSA for network/intertemporal/sector-coupled cases. These are external reference tools, not dependencies and not assumed oracle truth.
4. **Cost/performance priors:** pin NLR ATB version and the exact CSV row/scenario/geography/year used for every imported assumption. Do not transplant US cost assumptions into South Africa as project estimates.
5. **Real-world calibration:** when available, compare with independently metered input/output and parasitic loads; report bias, MAE/RMSE, interval coverage and systematic error by technology class. No comparison should use a technology's brochure efficiency as measured whole-system output.

## Next steps

- Add a versioned JSON Schema/CLI with input and assessment SHA-256 receipts, then freeze known-answer fixtures.
- Add stateful storage and multiple energy streams as separate model versions (charge/discharge losses, state continuity, terminal reserve, heat rejection and source grade).
- Add cost, carbon, water and material accounts using evidence-bound interval-valued data—not generic defaults.
- Construct a three-site South African case and hold out one weather/load period for evaluation.
- Publish a matched comparison table with SAM/REopt/PyPSA only where model scopes overlap; show discrepancies and failed cases.

**Research principle:** a small model that closes its balance and states its limits is more valuable than a comprehensive simulator that silently creates energy, conflates heat with electricity, or hides missing evidence.
