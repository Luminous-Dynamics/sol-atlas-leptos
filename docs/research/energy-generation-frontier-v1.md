# ENERGY-GENERATION-FRONTIER-001 — Evidence-ranked generation and conversion research

**Status:** research contract / proposed benchmark, not a validated technology ranking  
**Evidence cut-off:** 2026-10-22  
**Primary geography:** South Africa, with transferable international reference cases  
**Purpose:** extend the existing compute-energy resilience and bounded-Pareto work into a source-to-service energy portfolio model. Novelty is not a goal by itself: a candidate must improve measured useful service, resilience, lifecycle cost, or another explicit objective against a matched baseline.

## 1. The paradigm shift: model energy pathways, not isolated devices

The common comparison—one generator versus another by headline nameplate capacity or LCOE—misses the systems that determine whether energy is useful. Represent each pathway as a graph:

`resource → converter → conditioning → optional storage → distribution → delivered service`

Keep four categories distinct:

- **Primary resource:** sunlight, wind, geothermal heat, ocean gradients, organic waste, nuclear fuel, or recoverable industrial waste heat.
- **Converter:** PV, CSP/steam cycle, thermophotovoltaic (TPV), organic Rankine cycle (ORC), supercritical-CO2 cycle, thermoelectric generator, Stirling engine, or combustion engine.
- **Storage / carrier:** batteries, sensible/latent/thermochemical heat storage, pumped hydro, or hydrogen/derivatives. Storage moves energy in time; it is not a primary source.
- **Delivered service:** net electricity at a boundary, heat at a stated temperature, cooling, water, mobility, or a specified industrial process.

This graph exposes cascading conversion: use high-grade heat for electricity where justified, then cascade lower-grade heat to process heat or hot water, and include rejected heat, pumping, cooling, inverter losses, storage losses, and backup supply. A high gross efficiency must not hide parasitic loads or poor service availability.

## 2. Candidate slate: evidence level and decision relevance

The entries below are research candidates, not endorsements or claims of local feasibility. Each numeric result must remain attached to its original boundary, date, system scale, and uncertainty.

| Candidate | What is genuinely interesting | Current evidence / boundary | Initial priority |
|---|---|---|---|
| **Solar PV + solar-thermal / thermal storage / flexible loads** | Pair low-cost daytime electricity with thermal loads and dispatchable stored heat/electricity; reduce curtailment before inventing a new source | South Africa already has operational PV and CSP; the 100 MW Redstone CSP plant is a local dispatchable-storage reference. Compare real output, storage duration, cost and outages rather than assuming a hybrid always wins. [Redstone case](https://bnkp.ndb.int/case-studies/renewable-energy-development-project) | **P0 — matched baseline and deployment** |
| **Industrial heat recovery: ORC, sCO₂, TPV, thermoelectric** | Turn otherwise rejected heat into useful power, while still using remaining heat if there is a real sink | Converter suitability depends on source temperature and duty cycle. A 2022 TPV paper measured (>40\%) device conversion at emitter temperatures of 1,900–2,400 °C; this is not round-trip efficiency or the net efficiency of an installed plant. [Nature TPV result](https://www.nature.com/articles/s41586-022-04473-y). The 2026 thermoelectric review identifies tellurium-free materials as an important low-temperature waste-heat research path, while emphasizing device/system challenges. [Nature Reviews Materials, 2026](https://www.nature.com/articles/s41578-026-00923-5) | **P0 — quantify recoverable heat by temperature band** |
| **Solar process heat + phase-change storage + heat pumps / mechanical vapour recompression** | Provide reliable clean process steam, often more economically than converting all sunlight to electricity and back to heat | South Africa's Coal Zero by Green Steam pilot is scheduled for March 2026–August 2027 and is designed around continuous 178 °C steam; it remains a demonstration to measure, not proven fleet performance. [Project](https://www.absolicon.com/funded-projects/project-coal-zero-by-green-steam/) | **P0 — high local industrial relevance** |
| **Landfill gas / wastewater sludge / anaerobic digestion** | Dispatchable electricity and/or heat plus methane mitigation and waste treatment | The CSIR lists landfill and biomass among relevant municipal supply options. A 2026 study estimates 70–75 GWh from proposed new cells at Johannesburg's Robinson Deep, but that is a modelled projection based on collection and conversion assumptions, not metered generation. [CSIR energy](https://www.csir.co.za/what-we-do/natural-environment/energy) · [Robinson Deep study, 2026](https://www.ijesd.org/show-222-2186-1.html) | **P0 — site-specific feedstock + methane accounting** |
| **Mine-water thermal networks / geothermal heat pumps** | Reuse flooded-mine water as a thermal reservoir for space/process heating and cooling; avoid labelling heat delivered by a heat pump as electricity generation | A 2026 review reports feasibility depends strongly on network heat loss, heat-exchanger mineralisation, capital costs and local energy/carbon prices. This is a service/heat pathway first, not a claim of material net electric output. [Mine-water heat study](https://www.sciencedirect.com/science/article/pii/S0196890425013330) | **P1 — evaluate only with site and demand evidence** |
| **Enhanced / superhot-rock geothermal** | Potential firm, weather-independent heat with high-temperature conversion and possibly process heat coproducts | PNNL's Newberry demonstration describes two or more wells at about 4.25 km and 425 °C; its stated success criterion includes (>40\text{ MW}_{th}) of economic steam. That is a project target, not a measured electric output or a South African resource estimate. [PNNL project report](https://www.pnnl.gov/publications/newberry-shr-demonstration-project-bipartisan-infrastructure-law-enhanced-geothermal) | **P1 — high upside, high drilling/resource risk** |
| **Salinity-gradient power from brine + wastewater (pressure-retarded osmosis first)** | Recover part of the chemical potential in co-located concentrated brines and low-salinity streams; potentially pair energy recovery with treatment | A 2025 process-feasibility model reported PRO approaching US$0.26/kWh for a hypersaline-brine case, while RED and nanopore systems recovered less than 10% of theoretical energy under the tested real-condition assumptions and remained expensive. Treat this as model evidence, not bankable plant economics. [2025 assessment](https://www.sciencedirect.com/science/article/pii/S2213343725031860) | **P2 — only where a real brine/wastewater pair exists** |
| **Ambient-radiation / radiative-cooling mechanical harvesting** | A device can use radiative cooling against the sky to create a temperature gradient without a conventional fuel; niche uses may include passive ventilation and low-power sensing | Outdoor experiments reported (>0.4\text{ W}/\text{m}^2) of mechanical power, with (>6\text{ W}/\text{m}^2) described as potential. The latter is not the demonstrated field result. This is not a utility-scale power candidate on present evidence. [Science Advances, 2025](https://pmc.ncbi.nlm.nih.gov/articles/PMC12609158/) | **P2 — bounded niche study, not grid supply** |
| **Fusion pathways** | A potentially large long-term firm-power resource if net electric plants become technically and economically viable | 2026 plasma/coil milestones do not establish commercial net-electric generation. Reuters reported Helical Fusion's future test schedule and that no commercially viable fusion reactor exists as of its report. [Reuters, 29 Sep 2026](https://www.reuters.com/world/asia-pacific/japans-fusion-energy-startup-begin-power-on-tests-pilot-reactor-2027-2026-09-29/) | **Watchlist only — no near-term supply credit** |

## 3. Localisation: choose sites and use cases before ranking technologies

For South Africa, ground the work in the national IRP and local evidence, not imported cost curves alone:

- Use [South Africa's IRP 2025](https://www.dmpr.gov.za/Branches/Mining-Minerals-Petroleum-Policy-Development/IRP-2025) for the latest national planning assumptions and published input data.
- Use the [CSIR Energy Centre](https://www.csir.co.za/what-we-do/natural-environment/energy) and its [Wind Atlas](https://www.csir.co.za/what-we-do/natural-environment/energy) for locally maintained energy-system evidence.
- Use [NLR's 2025 Electricity Annual Technology Baseline](https://atb.nlr.gov/electricity/2025/technologies) as a versioned, comparative reference for familiar generation and storage technologies—not as South African project prices without adjustments for financing, import costs, grid connection, water, local labour and currency.
- Model Johannesburg/Gauteng use cases around industrial and data-centre heat, wastewater/sludge, landfill gas, embedded PV, batteries, grid bottlenecks and resilience. The Robinson Deep estimate should be tracked as a **forecast** until project-specific metered data exists.
- Model CSP/solar-thermal and wind where the spatial resource and grid conditions support them; do not infer a national resource potential from one project.

## 4. Shared data contract for candidate technologies

A candidate record must be auditable and versioned, with source-level provenance for every measured or assumed field.

**Identity:** candidate ID, source/converter/storage/service role, technology family, version, geography, application scale and declared system boundary.

**Performance:** gross and net electric output; heat/cooling output with temperature and quantity units; parasitic demand; availability/capacity factor definition; ramp/start time; degradation; storage charge/discharge efficiency and duration; lifetime and replacements. Keep instantaneous power (kW/MW), energy (kWh/MWh), thermal energy, and useful service distinct.

**Economics:** installed CAPEX, fixed and variable O&M, fuel/feedstock, grid connection, replacement, decommissioning, finance assumptions and currency/date; calculate both LCOE and service-specific cost where meaningful. Report whole-system costs and the counterfactual, not just component costs.

**Externalities and resilience:** lifecycle emissions and boundary, water withdrawal/consumption, land, critical materials, toxic/by-product streams, noise, safety hazards, repair skill/parts availability, failure domains, recoverability, and impacts on essential-service floors.

**Evidence:** source URL/DOI, publisher, publication date, exact page/table or quoted metric definition, source artifact digest when available, measured/modelled/target/vendor status, sample size, uncertainty/interval, geography, and reviewer. A citation proves the provenance of the claim, not that the claim is true.

Use explicit maturity labels: **physics/theory**, **lab result**, **component demonstration**, **field pilot**, **operating commercial plant**, **replicated operational fleet**, or **projection/target**. Do not collapse these into an unsupported single TRL or confidence number. Missing input means *unknown/indeterminate*, not zero or a favourable default.

## 5. Benchmark programme

### A. Thermodynamic and accounting correctness

- Energy in/out balances must close on analytically solvable fixtures within a declared numerical tolerance.
- Net output must subtract pumps, compressors, blowers, cooling, conversion auxiliaries and storage losses; assert that reporting boundaries are consistent.
- Exergy destruction must be non-negative within numerical tolerance; idealized heat-engine efficiency must not exceed the Carnot bound for the declared hot/cold reservoirs.
- Storage shifts energy; it must not create net energy. A zero resource, zero temperature delta, absent salinity delta, impossible duty cycle or zero fuel-flow negative control must not produce unexplained generation.
- Units and time bases must be dimensional and explicit. Reject NaN, infinity, overflow, invalid periods and contradictory records.

### B. Useful-service benchmark, not nameplate benchmark

Run matched site/load profiles against at least:
1. PV + wind + battery reference portfolio;
2. PV + wind + storage + dispatchable biogas/landfill-gas where a sustainable feedstock exists;
3. CSP/solar process heat + thermal storage where the thermal load is a genuine match;
4. waste-heat recovery behind a separately metered thermal source;
5. candidate geothermal or salinity-gradient system only with resource-specific data.

Report net MWh delivered, critical-hour firm capacity, unserved load, curtailment, ramp/availability, useful heat by temperature band, lifecycle cost, water, carbon, land, replacement schedule and common-mode failure exposure. Run both ordinary years and declared stress cases: low wind/solar, heat-wave, drought/water constraints, grid separation, converter outage, storage degradation, feedstock shortfall and price shocks.

### C. Optimisation and scientific validity

- Preserve an unweighted Pareto frontier across at least net useful energy/service, cost, carbon, water, material criticality and essential-service coverage. Do not hide trade-offs in a single weighted score.
- For small fixtures, enumerate a bounded exact frontier and use it as an oracle. Compare larger heuristics by hypervolume/regret against that oracle on matched scenarios; fail closed on candidate-budget overruns rather than calling a truncated set exact.
- Include ablations: remove the candidate technology, remove storage, remove demand flexibility, ignore parasitic loads, collapse uncertainty, and remove spatial/network constraints. The complex model earns its keep only if it improves out-of-sample service/cost/resilience over simpler baselines.
- Use held-out weather and load years; publish forecast calibration and interval coverage, not only point estimates. Use independent source snapshots and deterministic input/output digests.
- Keep scenario counts separate from probabilities. Do not claim physical reliability, causal impact or bankability from a synthetic simulator alone.
- Acceptance requires reproducible artifacts, explicit source revisions, negative controls, deterministic results and a separate claim ceiling: *synthetic model*, *literature-calibrated model*, *field-metered evaluation*, or *independently audited result*.

## 6. Repository responsibilities

- **Sol-Atlas:** spatial resource evidence, source-to-service graphs, uncertainty-aware scenarios and bounded multiobjective portfolio comparison; integrate with the existing compute-energy/Pareto work rather than duplicating it.
- **Symthaea:** scientific hypothesis generation, physical-admissibility checks, evidence grading, sensitivity analysis and adversarial testing of economic/efficiency claims. Keep speculative economics separate from measured plant evidence.
- **Symtropy:** deterministic digital twins of resource profiles, converters, thermal/electric storage, parasitic loads, degradation, outages and service demand. Test conservation and failure recovery before adding higher-level control.
- **Mycelix:** provenance/attestation of production, metering, maintenance, costs and community delivery; represent new technology families without assigning them trust or reward merely because of their type. Keep source identity, meter evidence, production, claims and acceptance separate.

## 7. First research tranche

1. Freeze the candidate registry and evidence cut-off; keep source values plus derived values and formula version separate.
2. Build three end-to-end worked cases: (a) PV + battery for an essential load, (b) solar thermal + storage for a real industrial heat demand, (c) landfill/wastewater biogas with methane and parasitic-load accounting.
3. Add a literature-calibrated waste-heat cascade model as the first converter comparison, with ORC/TEG/TPV only enabled over sourced temperature ranges.
4. Add superhot geothermal, salinity-gradient and radiative-cooling cases as *uncertain research candidates*, with explicit resource and maturity gates.
5. Compare the exact frontier and simple baselines before adding more mechanisms. Publish what loses as well as what wins.

**Non-goal:** declaring a new generator commercially viable from a paper abstract, vendor projection, synthetic fixture or theoretical efficiency. The goal is a better falsifiable process for discovering which combinations provide more reliable useful energy at lower total cost and impact.
