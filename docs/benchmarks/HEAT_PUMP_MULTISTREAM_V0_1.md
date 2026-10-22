# HEAT-PUMP-MULTISTREAM-V0.1 — first-law accounting and COP benchmark

**Status:** research and implementation contract. CI status is authoritative for executable results; this document does not claim plant validation.  
**Scope:** one bounded, steady-state heat-pump interval, with separate source-heat and compressor-electric streams.  
**Intended first use:** heat-grade matching and recoverable-waste-heat screening before adding time-coupled storage or portfolio optimization.

## 1. Why this model boundary matters

A single-stream converter cannot represent a heat pump correctly. A heat pump transfers source heat and adds work:

`Q_{hot}=Q_{source}+W_{compressor}`

The model therefore takes source-heat availability and compressor-electricity availability as separate streams. It carries auxiliary electricity separately, applies hard input limits, computes heat delivered after the declared distribution loss, and distinguishes compressor COP from system COP.

- Core COP: `gross condenser heat / compressor electricity`.
- System COP: `useful heat delivered / (compressor electricity + auxiliary electricity)`.
- Carnot heating bound: `T_hot / (T_hot - T_cold)`, using absolute temperatures in kelvin.
- Service-side first-law balance: `used source heat + used compressor electricity = useful delivered heat + distribution loss`.

An input COP or performance promise is not treated as evidence. The core COP is derived from explicit energy-flow quantities. A value above the declared Carnot bound is rejected, but a value below that bound does not prove real equipment can achieve it: exchanger approaches, part-load behavior, defrosting, cycling, refrigerant constraints, fouling and metered auxiliary demand remain outside v0.1.

## 2. Frozen synthetic known-answer case

Input assumptions:

- Source at 20 °C; useful delivery at 80 °C.
- 30 kWh of source heat and 10 kWh of compressor electricity available and accepted.
- 1 kWh auxiliary electricity.
- 10% declared heat distribution loss.
- Evidence class `analyst_assumption`; fixture explicitly marked synthetic.

Expected results:

| Quantity | Expected value |
|---|---:|
| Gross heat output | 40 kWh |
| Distribution loss | 4 kWh |
| Useful heat delivered | 36 kWh |
| Compressor COP | 4.0 |
| System COP, including auxiliary electricity | 36/11 ≈ 3.2727 |
| Carnot heating COP upper bound | 353.15 / 60 ≈ 5.8858 |
| First-law balance residual | 0 kWh |
| Total electricity consumed | 11 kWh |

This is an arithmetic known-answer, not a measured plant result. The Carnot bound is necessary but not sufficient for real-world performance.

## 3. Negative controls and acceptance gates

- Zero source heat, zero compressor electricity and zero auxiliary use must result in zero heat output.
- Non-zero source heat with zero compressor work must not be accepted as heat-pump operation in the declared source/delivery temperature lift.
- Observed core COP must not exceed the Carnot heating COP by more than the declared numeric tolerance.
- Source temperature must be physically above absolute zero; useful-delivery temperature must be strictly greater than source temperature for this heating mode.
- Source and compressor availability must be separately bounded; every curtailed input stream remains reported and no input cap may be ignored.
- Loss fraction must lie in [0, 1]; NaN, infinity and negative energy inputs are rejected.
- Non-assumption evidence classes require an evidence reference. Digest integrity proves input/report identity, not metering accuracy, calibration or source truth.
- Repeated typed assessment must serialize deterministically. Receipt verification must bind exact input bytes and compact JSON assessment.
- Unknown JSON fields and unsupported schema versions fail closed.

Numeric energy-balance tolerance of 1e-9 kWh applies to the small unit-scale fixture. Scale-aware tolerances are required before asserting that same absolute tolerance on arbitrarily large site data.

## 4. Industry comparison: use the right comparator for each layer

This implementation is not yet a detailed heat-pump performance simulator. It is a small first-law accounting oracle and evidence boundary.

- **NLR System Advisor Model (SAM):** established techno-economic analysis for PV, battery storage, CSP, industrial process heat, wind, marine, geothermal and biomass. Use SAM for compatible plant and process-heat cross-checks; compare at the same boundary and weather/input data. https://sam.nlr.gov/
- **REopt:** facility-level optimization for cost, generation/storage sizing, hourly dispatch and resilience; published capabilities include thermal storage and heating/cooling loads. Use it for sizing/dispatch comparisons once this branch has time-coupled heat and electricity stores. https://www.nlr.gov/reopt/about
- **PyPSA:** multi-sector energy-system optimization with intertemporal storage, elastic demand, generation, and linearized AC/DC network modelling. Use it for larger portfolio/network cases, not as an assumed oracle for every device-level heat-pump detail. https://docs.pypsa.org/latest/
- **IEA Heat Pump Monitor 2026:** emphasizes harmonized reporting because heat-pump taxonomies and performance reporting vary across regions. Preserve source/delivery temperatures, useful heat, compressor power and auxiliary power as separate fields so comparisons are interpretable. https://www.iea.org/reports/heat-pump-monitor-2026/key-findings

Our proposed differentiator is not that this module predicts heat-pump performance better than those tools. It provides a deterministic, testable physical-accounting gate and evidence-aware output that can be cross-checked against domain-specific simulators. A later claim of superiority requires a matched, independently reproducible evaluation.

## 5. Proposed benchmark ladder

| Stage | Benchmark / target | Evidence required before claiming success |
|---|---|---|
| A | First-law residual ≤ 1e-9 kWh on this unit-scale fixture | Exact-head automated test |
| B | 100% rejection of declared invalid temperature/COP/energy/evidence negative controls | Test report including all negative controls |
| C | COP and delivered-heat agreement with analytical known answers | Independent hand calculation and serialized golden output |
| D | Model-versus-device fit: MAE/RMSE and interval coverage by temperature lift, part load and climate | Independent performance maps and metered test series |
| E | Against SAM/REopt/PyPSA where model boundaries overlap | Same input data, units, time periods, economic year and declared assumptions |
| F | Useful heat cost and flexibility benefit beat baseline by a prespecified threshold without violating carbon/water/service constraints | Held-out weather/load cases, uncertainty intervals and sensitivity analyses |

Do **not** set an arbitrary field-accuracy percentage until a sufficiently broad independent dataset is pinned. First characterize error by temperature lift, part-load ratio, operating mode and weather. Do not compare one tool's ideal COP with another tool's whole-system seasonal COP.

## 6. Current South African relevance

The South African IRP 2025 identifies flexibility as a material requirement as variable renewable energy shares rise. On 7 October 2026, the government described a proposed first section 34 determination prioritizing 4.6 GW of battery storage and 5 GW of gas-to-power for immediate system flexibility and dispatchable supply. This is policy/procurement direction, not proof that either capacity has been delivered. https://www.gov.za/news/media-statements/minister-kgosientsho-ramokgopa-prioritises-battery-storage-and-gas-power

The heat-pump work fits alongside, rather than instead of, grid storage and firm supply. Its local use cases should be site-grounded: industrial heat recovery, mine-water district heating/cooling, data-centre waste heat and water/wastewater infrastructure. Model the required process temperature and hourly load before choosing a technology.

IRP 2025 source: https://www.dmpr.gov.za/Branches/Mining-Minerals-Petroleum-Policy-Development/IRP-2025

## 7. Next steps after this slice

1. Add a stateful store model with opening/closing inventory, charge/discharge limits, round-trip efficiency, standing loss and terminal reserve. State continuity is required across adjacent intervals; charge/discharge cannot create energy.
2. Couple heat pumps and stores across time without collapsing thermal and electrical carriers into a single unlabeled scalar.
3. Add measured performance maps / source-evidence records and uncertainty intervals, retaining unknown values as indeterminate.
4. Add hourly tariff/carbon/water bounds and a service-level comparison against electric resistance, gas boiler, heat-recovery and solar-thermal alternatives.
5. Cross-check compatible cases against SAM; move facility scheduling comparisons to REopt and system/network comparisons to PyPSA as model scope grows.

**Principle:** use the smallest model whose accounting is correct, then extend its boundary only when the extra state and physics can be independently tested.
