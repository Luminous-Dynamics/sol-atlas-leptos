# Energy Technology Paradigm Shifts and Competitive Benchmark Protocol v0.1

**Status:** research strategy and benchmark contract, 9 November 2026. This document separates current market evidence, research demonstrations, South African resource context and proposed Luminous targets. It does not declare a technology winner or certify a site.

## Executive thesis

The next generation of energy infrastructure will not be won by a single record-efficiency turbine, solar cell or battery. It will be won by the architecture that delivers the **most useful, reliable, affordable and maintainable energy service** from a measured resource, with the lowest total system and lifecycle burden.

The Luminous stack should therefore move from a technology catalogue to an evidence-qualified product and service system:

`site + time-varying resource + service demand + network + converter + storage + manufacturing + construction + operations`

Symthaea proposes diverse architectures and experiments. Sol-Atlas supplies source/site/economic evidence and bounded multiobjective comparisons. Symtropy executes qualified physical and system twins. Mycelix preserves provenance, rights, approval and change history. Independent engineering, site operators and authorities retain safety and actuation authority.

## 1. The evidence ladder — global industry context as of November 2026

### Generation cost and deployment

IRENA's *Renewable Power Generation Costs in 2025* (published July 2026) reports global weighted-average LCOE values for projects commissioned in 2025 of approximately:

| Technology | Global weighted-average LCOE (USD/MWh) |
|---|---:|
| Onshore wind | 33 |
| Solar PV | 44 |
| Hydropower | 62 |
| Offshore wind | 78 |
| Geothermal | 89 |
| Concentrating solar power | 115 |

These are global plant-level weighted averages, not South African EPC quotations, firm-delivery costs, grid-connection costs or project bids. They should be used as a starting comparator only after matching financing, currency year, capacity factor, grid boundary, resource quality and cost inclusions.

The IEA Global Energy Review 2026 reports around 800 GW of renewable capacity additions in 2025, including more than 600 GW of solar PV and about 160 GW of wind. This scale makes solar/wind manufacturing and grid integration the commercial baseline; it does not mean that every site should build the same mix.

IRENA's 2026 24/7-renewables analysis reports firm solar-plus-storage costs around USD 54–82/MWh in high-quality resource regions under its assumptions. The benchmark is a complete delivery profile, not standalone PV LCOE. Project comparisons must preserve the specified annual, monthly and peak demand-fulfilment constraints.

Sources:
- IRENA, *Renewable Power Generation Costs in 2025*: https://www.irena.org/Publications/2026/Jul/Renewable-Power-Generation-Costs-in-2025
- IEA, *Global Energy Review 2026 — Solar PV and Wind*: https://www.iea.org/reports/global-energy-review-2026/technology-solar-pv-and-wind
- IRENA, *24/7 Renewables Outcompete Fossil Fuels on Costs* (6 May 2026): https://www.irena.org/News/pressreleases/2026/May/24-7-Renewables-Outcompete-Fossil-Fuels-on-Firm-Costs

### Solar: efficiency is no longer the whole problem

A 2026 Nature Sustainability study reports 33.0% certified perovskite–silicon tandem cell efficiency, 23.2% module efficiency for its reported blade-coated module, and 99.8% retention after 1,200 hours of continuous light soaking for a specified single-junction cell. Those are valuable laboratory and pilot-scale results, but they are not a commercial module fleet, a long-term outdoor degradation curve or a warranty-compatible product.

The industry bottleneck is increasingly the product system: large-area yield, throughput, encapsulation, thermal/UV/moisture stability, reverse-bias/shading behaviour, lead containment, recycling, bankability and factory economics. Luminous should compete on **lifetime delivered kWh per area, capital, material and maintenance burden**, not chase a laboratory cell-efficiency record while ignoring module yield and service life.

Potential product directions:
- conventional silicon PV as the bankable baseline;
- bifacial/tracker/agrivoltaic and building-integrated layouts where site conditions support them;
- tandem PV as an emerging product only after module-level independent qualification;
- solar thermal and high-temperature thermal storage for matched industrial heat loads;
- PV + battery + thermal storage + demand response where useful electricity and heat profiles justify the extra interfaces.

Source: https://www.nature.com/articles/s41893-026-01916-6

### Wind: optimize the plant, not the turbine nameplate

The global weighted-average 2025 LCOE reference is about USD 33/MWh for onshore wind and USD 78/MWh for offshore wind. Floating offshore wind remains much more project-dependent; published European data show high project LCOE and a lack of standardization across floating substructures. Wind performance is determined by the entire plant: site-specific wind distribution, turbine power curve, wake, availability, foundations, connection, curtailment, logistics, blade lifetime, maintenance access and end-of-life.

Competitive research:
- site-calibrated wake and terrain models with independent met-mast / lidar validation;
- modular drivetrain, blade, power-electronics and condition-monitoring interfaces;
- repairable blade and composite recovery pathways;
- floating foundation designs only where bathymetry, metocean conditions, vessel capacity and cable economics justify them;
- hybrid solar/wind/storage schedules matched to hourly load and grid conditions.

Do not compare a wind turbine power curve with an integrated plant's net output or assume a higher hub height automatically gives lower delivered cost.

### Hydropower: low-head opportunity plus full-scale validation

The IRENA global 2025 weighted-average hydropower LCOE is about USD 62/MWh, but a single global value hides major differences between large hydro, small hydro, conduit recovery, pumped storage, hydrokinetic and refurbishment.

The 2026 low-head hydropower validation review identifies a shortage of full-scale test capability for many low-head powertrains and calls for repurposing underused hydraulic infrastructure as test facilities. The 2026 shallow-water hydrokinetic review reports 15–45% laboratory-to-field performance degradation across its reviewed device families. That range is **not** a universal derating factor for PATs or conventional head-based turbines; it demonstrates why field-scale evidence is essential.

Luminous differentiators:
- preserve water delivery, pressure, ecological flow and water quality as hard constraints;
- recover pressure that can safely be removed from already-required water conveyance;
- maintain separate hydraulic, shaft, generator, power-electronics, auxiliary and net-output ledgers;
- use equipment-specific measured maps and refuse extrapolation;
- qualify low-head designs on a full-scale rig before site transfer;
- compare fish passage, sediment continuity and water quality as separate environmental outcomes for open-water hydro.

South African water-distribution PAT papers and the Leliefontein installation provide valuable local leads. Leliefontein's published 2021 paper describes a full-scale bidirectional pump/PAT station with SCADA and bidirectional metering, but the paper's cumulative meter values conflict with its annual and lifetime estimates. The disputed totals stay unresolved until the operator or authors confirms the unit and raw registers.

### Geothermal: the technology is changing, but site geology still rules

A major 2026 industrial milestone is Fervo's company-reported commercial operation of Cape Station GeoBlock 1 at 33 MW net on 30 September 2026. Its 9 November 2026 company release says the first GeoBlock is delivering under its PPA, while adjacent blocks remain in commissioning. This is a real commercial execution comparator, but the public release does not provide raw independently audited interval meter data, long-term reservoir decline or an independent cost audit.

A 2026 study using data from 514 geothermal plants reports a negative historical global experience rate of roughly -20% for conventional hydrothermal deployment, with major country-level variation. This warns against assuming geothermal costs fall automatically with cumulative capacity. EGS and closed-loop systems have much shorter deployment histories and need their own evidence class.

South African geothermal evidence is different: the published 2017 prospectivity study uses subsurface temperature, heat-flow, geochemistry and geology to identify possible **low-enthalpy** anomalies. It calls for new temperature, structural, hydrogeological and economic investigations. This supports further exploration and direct-heat screening, not a claim of a commercial high-temperature electricity resource.

Luminous strategy:
- first screen low-enthalpy direct heat for industrial, agricultural, building and mine-water applications;
- distinguish direct heat, heat pumps, binary-cycle power, hydrothermal, EGS and closed-loop systems;
- for EGS, measure drilling speed/cost, temperature, flow, injectivity, thermal drawdown, water loss, seismicity and parasitic pumping;
- require a complete subsurface-to-net-electricity model and long-duration field data before bankability claims.

Sources:
- Fervo 9 Nov 2026 results: https://fervoenergy.gcs-web.com/news-releases/news-release-details/fervo-energy-reports-third-quarter-2026-results
- Geothermal experience rates: https://doi.org/10.1016/j.isci.2026.117113
- South African geothermal prospectivity: https://doi.org/10.17159/sajs.2017/20170092

### Marine energy: separate tidal range, current, wave and OTEC

The DFFE marine spatial-planning report identifies four distinct South African marine-resource themes: Agulhas Current, Southern Ocean wave climate, offshore wind and ocean thermal gradients off northern KwaZulu-Natal. It states that the South African tidal range is too small for appreciable **tidal-range** power. That statement must not be confused with current-driven extraction from the Agulhas Current or wave conversion.

The report cites theoretical coastal wave power locally up to 100 kW/m. This is a wave resource statement, not electrical capture or annual energy. European project-cost references remain very high for tidal-stream and wave energy, with wide ranges and technology/project dependence. This is a research and pilot opportunity rather than an immediate utility-scale cost leader.

Priority:
1. refresh metocean data, bathymetry and environmental constraints for current, wave, offshore wind and OTEC separately;
2. identify survivability, access, mooring/foundation, cable and maintenance-window requirements;
3. use small, instrumented demonstrations with high-quality net-energy metering;
4. reject tidal-range barrages as a South African priority unless new site-specific evidence contradicts the DFFE assessment;
5. treat Agulhas-current and OTEC concepts as long-horizon hypotheses, not bankable resources.

Sources:
- DFFE Marine Spatial Planning National Data: https://www.dffe.gov.za/sites/default/files/reports/research/oceans/marinespatialplanning_nationaldata.pdf
- European marine renewable LCOE/deployment context: https://blue-economy-observatory.ec.europa.eu/eu-blue-economy-sectors/marine-renewable-energy_en

### Storage: choose by duration, duty and service

The relevant storage comparison is not simply USD/kWh:
- lithium-ion: fast response and strong supply-chain maturity for short-duration balancing;
- pumped storage hydro: very large system energy capacity and long-duration duty where terrain, water, environmental approval and civil works support it;
- compressed-air storage: a 2026 review reports broad 50–70% round-trip-efficiency ranges for large configurations, with thermal storage integration, compressor/expander part-load performance, geological storage and cycling fatigue as key variables;
- flow batteries and other electrochemical options: assess duration, electrolyte, balance-of-plant, degradation, safety and supply chain;
- thermal storage: often valuable where the actual load is heat, because it avoids unnecessary conversion to electricity and back;
- hydrogen / other molecules: compare full power-to-molecule-to-power or direct industrial use, not electrolyser efficiency alone;
- gravity / liquid-air / CO₂ / other novel storage: require bankable full-system evidence, not a vendor round-trip target.

For every storage option, report power (MW), usable energy (MWh), duration (h), round-trip efficiency, auxiliary consumption, standby loss, degradation, cycles, ramp, response, calendar life, availability, safety, replacement and cost per delivered MWh over its duty.

Source: https://www.nature.com/articles/s44359-026-00150-9

## 2. Paradigm shifts for Luminous

### Shift A — from nameplate generation to useful-service design

Every project begins with the demand and service envelope: electricity by interval, heat by temperature grade, water, mobility, communications and critical loads. The target is service delivered during the hours it is needed, not annual energy alone.

### Shift B — from one technology to a configurable portfolio

Candidate architectures should combine sources and carriers only when there is a measurable benefit: PV + wind + BESS; PV + thermal storage + heat pump; geothermal heat + industrial process; hydro conduit recovery + PV for local critical loads; offshore wind + storage; or pumped hydro / CAES for longer-duration balancing. Retain single-technology baselines and no-build options.

### Shift C — from average resource to uncertainty-aware design

Use hourly resource and demand distributions, extreme-weather events, outage windows, forecast error and correlated failures. Report P50/P90 yield only when the probability model is defensible. Unknown input or missing covariance must not silently become a favourable design number.

### Shift D — from bespoke plants to product platforms

Standardize replaceable modules, interfaces, measurement boundaries, installation steps, inspection, commissioning and repair. The product must be designed for available suppliers, manufacturing process capability, transport, lifting, site access and end-of-life from the start.

### Shift E — from simulator to evidence execution

Keep the chain:
`intent → authority → execution → observation → independent verification → reproducible receipt`

A digital twin is not field truth. A test report is not a licence. A cryptographic receipt proves which bytes were reviewed, not that a supplier statement is truthful. The design system must preserve source class, uncertainty, revision and the exact claim ceiling.

### Shift F — from energy-only to exergy and material productivity

For heat systems, match temperature grade and exergy; do not claim a heat-pump COP as an equivalent electricity-generation efficiency. Include water, land, materials, critical minerals, embodied carbon, manufacturing yield, repair and recycling in the Pareto frontier.

## 3. Benchmarks that should define "better than SOTA"

Every comparison must freeze input data, geography, service requirement, time window, financing, technology boundary and compute/information budget.

| Benchmark family | Core metrics | Acceptance / aspiration |
|---|---|---|
| Resource prediction | bias, RMSE, P50/P90 coverage, forecast error, seasonal drift | Held-out site/year results; uncertainty intervals must have measured coverage |
| Net generation | net MWh, capacity factor, availability, parasitics, curtailment | Compare at identical metering boundary; no gross/net substitution |
| Firm service | hourly load served, critical-hour shortfall, ELCC, outage duration, restart | Zero hard critical-service violations in frozen synthetic acceptance cases |
| Storage | RTE, duration, usable energy, degradation, ramp, standby loss | Compare at matched power, duration, cycling and dispatch duty |
| Hydro/water | pressure/flow/service floors, leakage, ecological release, water balance | Zero hard water-service breaches; independent flow/energy balance closes within instrument uncertainty |
| Thermal / geothermal | useful heat by temperature band, exergy, pumping load, thermal decline, well productivity | Match source/sink temperature and duty; no COP/thermal MW confused with net electric output |
| Manufacturing | yield, first-pass acceptance, throughput, part count, rework, lead time, local-content evidence | Product-level yield and cost, not lab-cell/turbine component efficiency alone |
| Construction | accepted output per crew-hour, outage/tie-in time, rework, safety hold points | Matched task and site conditions; independent inspection and as-built reconciliation |
| Reliability | availability, forced outage, MTTR, spare lead time, degradation, survivability | Long enough field exposure to support the stated warranty/reliability horizon |
| Lifecycle economics | CAPEX, OPEX, replacement, financing, net LCOE/LCOS, firm-service cost | Report sensitivity and Pareto trade-offs; no missing costs treated as zero |
| Environmental | water consumption, habitat/fish impacts, land/sea use, materials, end-of-life | Hard limits remain hard; no weighted score can compensate for an unacceptable impact |

### Matched competitor ladder

- **Solar PV / wind:** IRENA global 2025 cost data; NREL SAM and independent project-level data; REopt/PyPSA/GenX for matching dispatch and planning subproblems.
- **Hybrid firm power:** IRENA's 24/7 solar-plus-storage benchmark; exact hourly load-fulfilment and grid-boundary reproduction.
- **Hydro:** OWA-EPANET for water networks; IEC hydraulic-machine model/field acceptance standards where applicable; measured PAT and low-head rig maps; full-scale environmental and service measurements.
- **Geothermal:** commercial EGS plant operating evidence (for example Cape Station's first GeoBlock) and historical cost data; separate hydrothermal, EGS, closed-loop and low-enthalpy comparisons.
- **Marine:** EMEC/European marine deployments and reliability/survivability guidance, but adjust resource, metocean, vessel and consent assumptions to South Africa.
- **Storage:** NREL/SAM and utility-scale operational benchmarks for batteries, pumped hydro, CAES and thermal systems with matching dispatch duty.

## 4. Luminous research programme — order of effort

1. Build a South African hourly data and evidence layer; distinguish public maps, modelled reanalysis, operator telemetry, meter series, surveys and laboratory observations.
2. Reproduce the solar/wind/BESS baseline and exact bounded portfolio oracle before optimizing exotic architectures.
3. Complete the water-service-first PAT reference and independently qualified uncertainty budget; pursue operator meter data for Leliefontein, Queenswood and other South African sites.
4. Screen industrial heat, mine-water thermal networks and low-enthalpy geothermal before claiming a power-plant prospect.
5. Treat marine current, wave and OTEC as metocean/engineering demonstrations; do not assume resource density implies economic energy.
6. Build modular product and manufacturing benchmarks for the first chosen hydro/thermal product; record factory/site accepted output and full lifecycle cost.
7. Integrate all candidates in Energy Systems Studio with fault replay, deterministic run receipts, Pareto frontier and independent conservation checks.

## 5. Rules for SOTA claims

A result can be called better only when:
- the comparator uses the same location, resource time series, load, service requirement and system boundary;
- the exact baseline and version are pinned;
- output is measured at the same net delivery point;
- training/calibration and blind test sites/times are separate;
- cost, environmental and reliability assumptions are disclosed;
- uncertainty and sample size are reported;
- losing candidates and failure cases are preserved;
- physical safety, permits and operator approvals are not inferred from simulation.

The target is not to win one benchmark by hiding costs or weakening constraints. The target is a reproducible Pareto improvement in delivered service, cost, reliability, maintainability, environmental impact and evidence quality.


## 6. South African mine-shaft pumped storage and gravity storage

A further South African storage research route is to re-use appropriate mine shafts and connected workings as part of an underground pumped-storage (UPHS) system. The 2015 South African Energy Storage Technology and Market Assessment explicitly identified unused/abandoned mineshafts as an initial high-head pumped-hydro screening option, but it did not identify a feasible shaft or complete a site-specific engineering study. This is a research lead, not a mine asset shortlist.

South Africa's baseline is already substantial conventional reservoir pumped storage: Eskom's 2024 fact sheet describes Drakensberg at 1,000 MW, Palmiet at 400 MW and Ingula at 1,332 MW. Eskom's October 13, 2026 Tubatse RFQ describes a separate conventional reservoir scheme with up to 1.5 GW / approximately 21 GWh, four reversible pump-turbine units and up to 14 hours at full output. Tubatse is a development, not operating capacity—and it is not an underground mine project.

### Do not conflate the three mine-related architectures

1. **Underground pumped hydro:** water moves between qualified upper and lower reservoirs using reversible pumps/turbines. The capacity depends on usable water volume and net head; performance requires hydraulic loss, water containment and cyclic geomechanical evidence.
2. **Solid gravity energy storage:** a solid mass is raised/lowered through a shaft or other mechanism. It has different energy density, hoist, braking, structural, response and lifecycle characteristics. A shaft-based solid-gravity device is not pumped hydro.
3. **Mine-water thermal storage:** heat is stored/exchanged in mine water and surrounding rock, usually with heat pumps. It is owned separately in the mine-thermal programme; thermal MWh cannot be counted as pumped-hydro electrical MWh.

A combined mine energy hub may integrate underground pumped hydro, PV/PVT, BESS and thermal systems, but each carrier and storage state requires its own measured boundary and balance.

### What must be proven before any underground storage capacity is counted

- a surveyed upper/lower reservoir geometry and usable operating volume;
- hydraulic head and conveyance route;
- shaft/lining/rock mass condition, fractures, connected workings and subsidence evidence;
- groundwater connectivity, leakage, water availability and permitted abstraction/reinjection;
- water chemistry, contamination, sediment, corrosion and treatment;
- cyclic water pressure, saturation effects, rock-mass stability and containment;
- pump/turbine maps and grid-side import/export metering;
- transient pressure/surge, water-hammer and emergency-stop analysis by qualified specialists;
- closure/remediation liabilities, access, maintenance, environmental impacts and independent inspection.

The 2025 abandoned-coal-mine workflow highlights heterogeneous rock and low permeability as challenges for stability and hydraulic tightness. The 2026 gravity-energy systematic review (107 studies) identifies low intrinsic energy density, infrastructure dependence, inconsistent techno-economic assessment and limited long-term operating data. A 2026 integrated abandoned-mine hub paper reports a modelled 2.22 GWh storage and COP 5.54; those values are scenario outputs, not field results.

### Competitive benchmark

Compare B0 no-build/closure, B1 conventional surface-reservoir PHS, B2 mine-shaft UPHS, B3 connected-mine lower-reservoir UPHS where proven, B4 surface + underground hybrid, B5 solid-mass shaft gravity, B6 BESS/CAES, and B7 a hybrid renewable portfolio. Hold useful grid service, power, usable energy, duration, response, cycle frequency and cost horizon constant. Report net grid-side round-trip efficiency, usable MWh, MW, duration, water loss, pumping energy, auxiliary load, availability, maintenance, civil/rehabilitation cost, closure liability and independent evidence coverage. Storage is not primary generation: it shifts energy and returns less than it uses.

The existing Symtropy [BROWNFIELD-UNDERGROUND-001 #1951](https://github.com/Luminous-Dynamics/symtropy/issues/1951) owns reuse-versus-new-build/no-build and site-use evidence. New Symtropy [MINE-SHAFT-PUMPED-STORAGE-001 #4424](https://github.com/Luminous-Dynamics/symtropy/issues/4424) specializes liquid pumped-hydro shaft qualification. Symtropy [MINE-THERMAL-STORE-001 #4070](https://github.com/Luminous-Dynamics/symtropy/issues/4070) owns mine/cavern thermal storage. These owners must compose and must not duplicate their physical models.

Sources:
- South African Energy Storage Technology and Market Assessment: https://www.eskom.co.za/eia/dx/wp-content/uploads/migrated/HexBESS/Appendix-H-1-South-African-Energy-Storage-Technology-and-Marketing-Assessment-1.pdf
- Eskom existing PHS fact sheet: https://www.eskom.co.za/wp-content/uploads/2024/07/HY-0001-Pumped-Storage-Scheme-and-Water-Transfer-Rev-12.pdf
- Eskom Tubatse project RFQ: https://www.eskom.co.za/eskom-green-launches-strategic-partner-selection-process-for-up-to-1-5gw-21gwh-of-tubatse-pumped-storage-scheme-set-to-be-south-africas-largest-energy-storage-system-ess-to-integrate-inc/
- Colas et al. (2025) abandoned-mine storage geology: https://doi.org/10.1016/j.est.2024.115101
- 2026 Gravity Energy Storage systematic review (107 studies): https://doi.org/10.3390/eng7090463
- Integrated abandoned-mine hub modelling (2026): https://doi.org/10.1016/j.seta.2025.104757
