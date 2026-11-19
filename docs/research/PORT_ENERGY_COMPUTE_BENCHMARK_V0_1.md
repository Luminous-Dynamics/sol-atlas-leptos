# Port-Energy-Compute Nexus: Benchmark Contract and Research Plan

**Protocol:** PORT-ENERGY-COMPUTE-001  
**Version:** 0.1  
**Research snapshot:** 19 November 2026  
**Status:** evidence-screening and benchmark design; not a site selection, feasibility study, construction authorization, grid-connection offer, environmental approval, or performance claim.

Parent work: [Sol-Atlas #942 — PORT-ENERGY-COMPUTE-001](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/942).

Related owners (compose; do not duplicate):
- [Sol-Atlas #654](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/654): rights-aware maritime spatial/evidence projections.
- [Symtropy #2721](https://github.com/Luminous-Dynamics/symtropy/issues/2721): architecture generation and independently verified energy-system twins.
- [Symtropy #5241](https://github.com/Luminous-Dynamics/symtropy/issues/5241): reproducible power-fabric benchmark and matched industry baselines.
- [Symtropy #6209](https://github.com/Luminous-Dynamics/symtropy/issues/6209): deterministic, provenance-preserving marine forecast replay.
- [Mycelix #4882](https://github.com/Luminous-Dynamics/mycelix/issues/4882): separate energy-market intent, physical delivery evidence and settlement.
- Symthaea already contains standalone grid physics, generation and storage scheduling, thermofluids, engineering and simulator bridges.

## 1. Executive decision

**Yes to integrated clean-energy and compute infrastructure in maritime/port cities. No to assuming that an underwater data centre is automatically the best first asset.**

The design target is a **port-city energy and compute commons**: one time-aware, evidence-carrying system model for electricity, port services, ship-at-berth power, renewables, storage, hydrogen, water, compute workloads, heat, grid constraints, fibre connectivity, marine ecology and community governance.

The first build should be a **shore-first, subsea-optional** architecture:

1. Establish the port's real electricity, shore-power, logistics, water, fibre and land-use constraints.
2. Co-design renewable generation, storage, flexible demand and port electrification.
3. Add modular onshore/nearshore compute only where demand, grid, fibre, land and heat-reuse economics justify it.
4. Treat floating or subsea compute as a competing design alternative—not a presumption—and admit it only after bathymetry, seabed habitat, metocean, cable, thermal-plume, permitting, maintenance and lifecycle evidence exists.

The differentiator is not a lower PUE in isolation. It is **time-to-qualified system design**: a proposal whose physics, constraints, costs, carbon, water, ecology, provenance, uncertainties and failure modes can be independently reconstructed.

## 2. Why this is timely — and what the industry has already done

### Underwater compute is no longer hypothetical

| Reference | Publicly reported position | What it establishes | What it does not establish |
|---|---|---|---|
| Microsoft Project Natick | The North Sea trial reported a server failure rate one-eighth that of its land-based control group. The experiment ended and the programme was formally closed in 2024. | A sealed subsea module can be operated and studied; the reported reliability result warrants replication. | A commercial cost advantage, universal reliability advantage, or suitability for every seabed and grid. |
| Hainan/Lingshui, China | The commercial underwater data-centre project began operating in March 2023. A February 2025 report describes an added module with more than 400 high-performance servers and multiple commercial users. | Commercial subsea compute and modular expansion are real deployment classes. | Independent lifecycle cost, environmental performance or a directly comparable PUE result. |
| Shanghai Lingang, China | The wind-linked underwater project entered operation in May 2026. Official reports describe a 24 MW planned capacity, direct offshore-wind integration and seawater cooling; the city reports PUE around 1.15 and electricity use 22.8% below a conventional land-based comparison. | Offshore renewable co-location and subsea compute are now a direct industry comparator. | That every phase is operating at the full planned capacity, that the comparison is independently audited, or that the energy is 24/7 carbon-free. |

Sources: [Microsoft Project Natick](https://www.microsoft.com/en-us/research/project/natick/); [China State Council/Xinhua on Hainan](https://english.www.gov.cn/news/202502/19/content_WS67b5a14dc6d0868f4e8efcaa.html); [Shanghai Government on Lingang operations](https://english.shanghai.gov.cn/en-Latest-WhatsNew/20260601/89a94ac072f842eeacf82fcf0cce8261.html); [Lingang official performance summary](https://www.lingang.gov.cn/html/website/lg/English/CityofInnovation/InnovationProjects/2021462605065908225.html).

### Conventional data-centre performance is improving too

Uptime Institute's 2025 Global Data Center Survey reports an average PUE of **1.48 for facilities commissioned within the preceding five years**, and **1.44 for facilities of 20 MW or larger**. Those are useful industry reference points, not universal design limits. Shanghai's reported ~1.15 is a stronger public comparator, but it must be compared on equivalent measurement boundaries, load, weather, annualization and independent metrology.

PUE alone cannot establish that a facility delivers more useful compute per kWh, is lower-carbon, uses less freshwater across its supply chain, is more reliable, or causes less marine impact. Keep facility metrics and system outcomes separate.

Sources: [Uptime Institute Global Data Center Survey 2025](https://datacenter.uptimeinstitute.com/rs/711-RIA-145/images/2025.Annual.Survey.Report.pdf?version=0); [ISO/IEC 30134-2:2026 — PUE](https://www.iso.org/standard/30134-2); [ISO/IEC 30134-6:2021 — ERF](https://www.iso.org/standard/71717.html).

### Port energy integration is also an active research field

Recent port-energy research already combines offshore wind, hydrogen and electricity storage, uncertainty-aware scheduling and ship microgrid demand. One 2025 Applied Energy study reports a real-world coupled hydrogen-electricity port case where multi-timescale scheduling saved 25.42% of cost and reduced CO₂ emissions by 14.78% in that case. This is a research comparator, not a transferable guarantee or a result achieved by this project.

Sources: [Applied Energy, multi-timescale port hydrogen-electricity scheduling (2025)](https://www.sciencedirect.com/science/article/pii/S0306261925006154); [Applied Energy, offshore-wind zero-carbon port microgrids under uncertainty (2025)](https://www.sciencedirect.com/science/article/pii/S0306261925014023); [IMO 2023 GHG Strategy](https://www.imo.org/en/ourwork/environment/pages/2023-imo-strategy-on-reduction-of-ghg-emissions-from-ships.aspx).

## 3. First South African screening case

The first case should be **Ngqura for energy-system screening**, with **Durban as the high-demand port comparison**. This does not select an underwater data-centre site.

The September 2025 PtX Hub / CSIR load-connection assessment estimates the following spare capacity at national port substations:

| Port | Estimated spare capacity | Estimated MW equivalent | Connection risk |
|---|---:|---:|---|
| Ngqura | 19 MVA | 17.1 MW | Low |
| East London | 18 MVA | 16.2 MW | Low |
| Gqeberha | 15 MVA | 13.5 MW | Low |
| Richards Bay | 11.5 MVA | 10.35 MW | Low |
| Mossel Bay | 7 MVA | 6.3 MW | Medium |
| Durban | 4 MVA | 3.6 MW | Medium |
| Cape Town | 0 MVA | 0 MW | High |
| Saldanha Bay | 0 MVA | 0 MW | High |

These are **pre-feasibility screening estimates**, not reserved capacity, a connection offer, a firm export/import limit, or proof that upstream network constraints are absent. The study itself notes that upstream authority consultation and grid studies are required where demand exceeds the estimated headroom.

A separate September 2025 shore-power demand study identifies Durban as having the highest shore-power demand among the assessed South African ports, followed by Cape Town. That makes Durban an important load/connectivity comparison even though its estimated spare capacity is much lower than Ngqura's.

**Interpretation:** Ngqura appears to be the more tractable first energy-system screening case based on published substation headroom. Durban is the more demanding test of how shore power, port electrification and flexible compute compete for constrained capacity. Neither statement establishes land availability, fibre latency, subsea suitability, permitting, power quality or project economics.

Sources: [PtX Hub / CSIR — Load Connection Capacity Assessment of South African National Ports (September 2025)](https://ptx-hub.org/wp-content/uploads/2025/09/PtX-Hub-Load-Connection-Capacity-Assessment-of-South-African-National-Ports_final.pdf); [PtX Hub / CSIR — Shore Power Demand for South African Ports (September 2025)](https://ptx-hub.org/publication/shore-power-demand-for-south-african-ports/). A machine-readable evidence seed is included alongside this protocol.

## 4. Paradigm shifts

### 4.1 Compute becomes a controllable grid resource

Treat non-urgent compute as a **time-flexible industrial load**, not an always-on demand sink. Separate workloads by service class:

- **Hard real-time / critical:** port safety, navigation support, emergency response, operational control and contracted low-latency inference. These are not casually shed.
- **Latency-bounded:** enterprise inference and interactive services with explicit latency and availability envelopes.
- **Deferrable:** batch training, rendering, scientific processing, indexing, archival work and data transformation with declared deadlines.
- **Replicable:** work that can migrate between a port-region onshore site and geographically independent sites without violating data-sovereignty, security or service constraints.

The scheduler should shift eligible workloads toward hours of renewable surplus, low marginal carbon, low congestion and lower cost—subject to deadlines, data gravity, network capacity, cooling limits and recovery/rebound constraints. It must account for **the energy required to catch up later**; a load shed now is not a permanent reduction if the queue creates a rebound peak.

The value of flexibility must be calculated from chronological trajectories and system adequacy, not a fixed percentage of theoretical workload deferrability. Reuse Symtropy's existing chronological adequacy work and report product-specific flexibility (market-serving, grid-serving and system-serving) separately.

### 4.2 Co-optimize the port, not just the data centre

The objective function must include the interacting services:
- ship-at-berth shore power and hotel loads;
- cranes, cargo handling, reefer containers, pumps and port vehicles;
- local homes, public services and industrial tenants;
- onshore/offshore wind, solar and other site-qualified resources;
- batteries, thermal storage and long-duration storage;
- electrolysers and hydrogen derivatives where offtake and lifecycle evidence support them;
- desalination and water/wastewater infrastructure;
- onshore, floating and subsea compute alternatives;
- transmission, distribution, substation, fibre and cable-landing constraints;
- marine habitat, thermal discharge, noise, seabed disturbance and cumulative impacts.

Use a **Pareto frontier** rather than a single opaque weighted score. A low-cost plan that violates environmental or resilience constraints is infeasible, not a clever trade-off.

### 4.3 Make the thermal system an explicit resource network

Seawater cooling can lower mechanical cooling demand, but it does not make heat disappear. Model:
- IT and facility heat generation by interval;
- supply/return temperatures, heat-exchanger approach and pumping loads;
- marine intake, discharge, entrainment and plume dispersion;
- seasonal stratification, currents, tides and ambient temperature;
- fouling, corrosion, biofouling, cleaning chemicals and maintenance;
- heat-pump lift and exergy penalties;
- actual nearby heat demand and pipe distance.

Recover heat only when a real, metered offtaker exists—such as suitable industrial preheating, a district/greenhouse loop or a properly assessed water process. Low-grade compute heat is not automatically a useful desalination heat source. If there is no viable heat customer, report the energy as rejected heat rather than claiming reuse.

For subsea designs, measure seawater intake/withdrawal volume, temperature difference and plume exposure separately from freshwater WUE. A low or zero freshwater WUE must not hide marine thermal or ecological impacts.

### 4.4 Move from annual renewable accounting to time-matched clean energy

Annual renewable certificates or annual energy matching can hide fossil-heavy operating hours. Report:
- renewable energy factor (REF) using the relevant standard boundary;
- hourly (and where data allow, 15-minute) clean-energy matching;
- residual grid energy and its time/location-specific emissions;
- curtailment avoided and renewable energy actually consumed;
- storage losses, charge origin, state of charge and degradation;
- additionality and contract/attribute ownership;
- backup fuel use and lifecycle emissions.

The system should prefer direct renewable use, storage and flexible compute before diverting scarce clean power into low-efficiency conversion loops. Hydrogen is best treated as a separately evaluated long-duration, industrial or maritime fuel pathway—not assumed to be the default way to store every surplus kWh.

### 4.5 Replace a single digital twin with evidence-bounded multi-fidelity twins

A planning model is not a physical approval. Use separate fidelity levels:
- hourly multi-year resource adequacy and techno-economic dispatch;
- 15-minute port/compute/energy scheduling;
- network power flow and protection-study interfaces;
- thermal-fluid and marine plume models;
- detailed CFD / hydrodynamics / structural assessment where justified;
- hardware-in-loop and metrology;
- field observations and independent environmental monitoring.

Each result must state the exact asset/configuration, model version, inputs, calibration, uncertainty, validity envelope and claim ceiling. DC-OPF, AC-OPF, RMS, EMT, CFD and physical observations are different evidence classes—not automatic promotions.

### 4.6 Build ecological observability and local legitimacy into the system

Environmental monitoring is a design input and an operating guardrail, not an afterthought. Baseline before/after/control/impact measurements should cover temperature, salinity, dissolved oxygen, turbidity, noise, relevant chemical indicators and site-specific biological indicators. Use seasonal and extreme-weather coverage. The site-specific environmental authorization sets the legal limits; do not invent a universal allowable thermal plume temperature.

Make ownership, benefits, procurement, energy claims, impact reports and grievance/correction mechanisms legible to port workers, local communities, municipalities, Indigenous/customary rights holders where applicable, and the public. Do not expose sensitive vessel, personnel or infrastructure telemetry merely to make the system auditable.

## 5. Cross-project architecture and authority boundaries

| Project | Proper role in this programme | Must not be treated as |
|---|---|---|
| **Sol-Atlas** | Spatial/temporal context: port assets, substations, shipping lanes, cable landings, protected areas, land use, source coverage, rights, bathymetry references and evidence overlays. | A source of live berth status, navigational clearance, legal permission or physical truth just because a feature appears on a map. |
| **Symtropy** | Deterministic replay, chronological energy/compute simulation, physical twins, storage/thermal-fluid trajectories, failure/common-cause scenarios, uncertainty and benchmark harnesses. | An accredited grid study, marine environmental approval, HIL result or field validation without the relevant evidence. |
| **Symthaea** | Forecasting, anomaly detection, scenario generation, constrained decision support and diverse design proposals over explicit interfaces. | The protective relay, safety PLC, grid operator, port authority or sole verifier of its own recommendation. |
| **Mycelix** | Identity and permissions, append-only provenance, asset/community claims, governance, investment/ownership, energy-market intent, authenticated evidence references and settlement lifecycle. | Proof that a physical quantity was delivered merely because an agent wrote a record to a DHT. |

### Authority hierarchy

`sensor / source evidence → validated state estimate → constrained proposal → independent verification → authorized local execution → metered observation → reconciliation → public/private receipt`

- Certified protection, approved local PLCs and the responsible system operator retain physical authority.
- Symthaea proposes; an independently implemented verifier checks hard constraints and source/model identity.
- A stale forecast, Mycelix outage, Symtropy fault or network partition cannot remove local protection or prevent safe standalone operation.
- Mycelix records who asserted, verified, approved, delivered and settled each claim. It does not collapse those distinct actors into one boolean.
- Unknown outcome means no new mutation until reconciled. Fail closed at the authority boundary.

## 6. Benchmark suite

Do not publish one leaderboard that conflates energy optimization, data-centre efficiency, grid capacity, ecological safety and physical deployment. Keep the tracks distinct and publish the raw receipts.

| Track | Matched baseline | Primary measures | Qualification target |
|---|---|---|---|
| A. Site evidence and spatial joins | Manual GIS + source documents; independent second reviewer | Source coverage, date/currentness, coordinate/datum correctness, rights, provenance completeness, unknown visibility | 100% accounting for in-scope source fields; zero silent conversion of unknown to safe/available |
| B. Port/grid feasibility | Grid-only reference; independently checked OPF and load-flow solvers | Nodal balance, voltage/thermal limits, contingency coverage, reserve margin, unmet critical load | 100% of admitted hard constraints independently rechecked; zero false-feasible results in negative controls |
| C. Chronological dispatch | Grid-only, rule-based EMS, conventional optimization/MPC | Total cost, import/export, renewable curtailment, storage cycles/degradation, reserve violations, rebound, unserved energy | Same source traces, weather, tariffs and load; report p50/p95 and all scenario failures |
| D. Compute energy efficiency | Matched onshore air/liquid-cooled and nearshore alternatives | Standard PUE/WUE/CUE/ERF/REF plus workload throughput per IT kWh, utilization, latency and energy per completed job | Report ISO/IEC 30134 PUE at a declared annual measurement boundary and match load factor/ambient conditions. Shanghai's reported ~1.15 is a subsea comparator, not the global SOTA: ENTSO-E's May 2026 report notes advanced cooling designs below 1.1 and leading hyperscalers below 1.05. Do not set a universal PUE win threshold; compare total energy per service-equivalent useful work and full resource impacts. |
| E. Clean-energy integrity | Annual matching vs hourly matching | REF, hourly clean-energy coverage, residual carbon intensity, curtailment and attribute provenance | Stretch objective: ≥95% REF and ≥90% hourly clean-energy matching; no certificate-only claim of 24/7 carbon-free operation |
| F. Water and heat | Matched cooling topology; independently checked thermal balance | Freshwater WUE, seawater intake volume, pump energy, ERF, heat grade, discharge temperature, plume and ecological indicators | No freshwater cooling consumption in the closed-loop design target; site-specific marine limits met with measured margin |
| G. Flexibility and adequacy | Inflexible compute; rule-based shifting; matched MPC | ELCC-like contribution, LOLE/EUE/CVaR, deadline misses, recovery rebound, latency/SLA violations | No adequacy degradation against the frozen reference; capacity value only reported when the full chronological model identifies it |
| H. Reliability and cyber-resilience | Single-node and conventional redundant site | Availability, failover, recovery time, common-cause cuts, cable/fibre loss, cooling loss, grid islanding, cyber isolation | Pilot service target ≥99.9%; production stretch ≥99.99% with independent evidence; zero unsafe control transitions in frozen fault suite |
| I. Lifecycle economics and climate | Matched land-only and port-only reference cases | CAPEX/OPEX, LCOE/LCOS, cost per useful compute, cost per port service, lifecycle CO₂e, embodied materials, repair logistics | No lower-cost/lower-carbon claim without matched lifecycle boundaries; candidate stretch: ≥30% lower operational CO₂e for the combined services |
| J. Community and port outcomes | Port-only counterfactual and documented benefit-sharing plan | Shore-power served, vessel auxiliary emissions avoided, local jobs/training/procurement, energy affordability, public reporting and grievance resolution | Report disaggregated outcomes and distribution of benefits; no aggregate social score that can mask harm |

The numeric values in the right-most column are **aspirational research gates**, not measured results or regulatory thresholds. The PUE and availability goals require a stated boundary and workload/service definition. For carbon, the 30% figure is a testable hypothesis against a matched counterfactual, not a guaranteed effect.

### Fair-run protocol

Every run freezes and reports:
- source bytes, source edition, rights/terms, retrieval date and SHA-256;
- site geometry, CRS, horizontal/vertical datum and coordinate epoch;
- asset inventory, ratings, units, topology and unknown fields;
- weather year(s), wind/solar series, ocean conditions and forecast information boundary;
- vessel and shore-power profile, port service demand, compute mix, deadlines and SLA;
- tariff, grid marginal/average emissions, renewable contract/attribute treatment and fuel lifecycle factors;
- storage chemistry, power/energy ratings, efficiency, temperature, degradation and reserve policy;
- model and solver versions, build hashes, hardware, tolerances, random seeds and all run options;
- model-build, solve, verification and total time-to-qualified-solution;
- objective values, primal/dual residuals where relevant, violations, constraint margins, uncertainty and rejected cases.

Use matched annual weather/load traces (at least 8,760 hourly points) for adequacy and annual resource comparisons; 15-minute traces for detailed port dispatch; faster dynamics only in a separate network/control study. Synthetic fixtures must be labelled synthetic and never be blended into observed site records. At least ten repeated runs are required for performance timing on pinned hardware; publish median, p95 and dispersion. Do not drop failures, censored cases or not-evaluable cases from the denominator.

## 7. Candidate targets and comparison posture

The credible initial industry comparison is:

- **Subsea PUE:** Shanghai reports about 1.15; use ≤1.15 as an aspiration, not as a claim of superiority. First reproduce the same boundary, load and annualization.
- **Conventional new-build PUE:** Uptime's 2025 survey gives 1.48 for facilities commissioned within five years and 1.44 for 20 MW+ facilities. These are useful context only; the best comparison is a matched site and workload.
- **Port energy optimization:** published 2025 research reports 25.42% lower cost and 14.78% lower CO₂ in one real-world hydrogen-electricity port case. The Luminous benchmark must reproduce its own case and baselines before claiming any advantage.
- **Reliability:** Natick's 1/8 server failure rate is a compelling experimental result, not a bankable fleet reliability distribution. We need multi-year operational evidence, common-cause analysis, repair/replace logistics and an independent land-based comparator.
- **Carbon:** beat the matched baseline on lifecycle and hourly operational emissions, not by shifting load or emissions outside the measurement boundary.
- **Usefulness:** measure completed compute under the same quality, throughput, latency and deadline conditions. A lower PUE that completes less useful work is not a win.

**Our candidate advantage should be verified integration quality:** better source-to-model continuity, constraint coverage, cross-domain optimization, ecological visibility, counterfactual explanation, failure handling and independently auditable decisions. Speed or a lower objective is secondary to valid, safe, reproducible improvement.

## 8. First implementation slices

1. **Freeze the schema and evidence seed.** Start with the companion South African screening JSON; keep all non-published site facts explicitly unknown.
2. **Build the synthetic reference case.** Use declared hypothetical hourly loads for port, shore power, water and compute; renewable generation, battery and optional hydrogen; tariff and carbon data; fixed service and environmental constraints. Do not label it a South African operating profile.
3. **Connect the Sol-Atlas context.** Join port, substation, shipping lane and cable-landing context by exact source/reference, preserving CRS/datum, source coverage and rights. Protected areas and bathymetry remain unresolved until rights-cleared data are acquired.
4. **Reuse the OPF programme.** Keep MATPOWER 8.1 / PGLib v23.07 and formulation-matched PowerModels DCMP / HiGHS comparators from the existing energy benchmark. Keep AC-OPF, dynamic controls and CFD as separate tracks.
5. **Extend Symtropy chronologically.** Compare fixed compute with deadline-aware flexible compute; include post-curtailment rebound, weather-correlated renewable shortfall, battery degradation, cable failure, grid outage, cooling-loop fault and a no-communications mode.
6. **Constrain Symthaea.** Allow proposals only through typed, bounded interfaces; test against rule-based and optimization/MPC baselines. Symthaea must not be its own verifier.
7. **Bind Mycelix evidence.** Keep market offer, dispatch request, controller acceptance, physical meter evidence, delivery reconciliation and settlement as separate append-only states.
8. **Promote only with new evidence.** Real metering, utility network studies, seabed surveys, environmental baselines, HIL and regulatory review are separate gates. A synthetic pass cannot promote a site or physical design.

## 9. Open questions that must remain open

- Bathymetry, seabed geology, habitat and marine-protected-area overlap for every candidate.
- Metocean conditions, wave/current loading, corrosion, biofouling, anchoring and subsea repair strategy.
- Submarine fibre/power cable route, landing capacity, latency, path diversity, repair access and common-cause exposure.
- Actual port and ship load time series, vessel call profiles, berth-level shore-power compatibility and future electrification demand.
- Utility-confirmed import/export capacity, fault levels, voltage quality, protection coordination, curtailment and connection timeline.
- Actual renewable resource time series, permitting, offtake, additionality and grid emissions.
- Data-centre workload, hardware, service quality, cooling topology, utilisation and annual metered facility energy.
- Heat customer, temperature grade, distance, seasonality and contract.
- Environmental baseline, discharge/thermal plume model, monitoring design and site-specific authorization limits.
- CAPEX, insurance, financing, decommissioning, repair logistics, tax/import, data sovereignty and local benefit-sharing model.

Until these questions are answered, this programme can rank **investigation priority**, not approve a physical site.

## 10. Primary references

1. [Microsoft Research — Project Natick](https://www.microsoft.com/en-us/research/project/natick/)
2. [China State Council/Xinhua — Hainan underwater intelligent computing cluster, 19 February 2025](https://english.www.gov.cn/news/202502/19/content_WS67b5a14dc6d0868f4e8efcaa.html)
3. [Shanghai Government — wind-powered undersea data centre operations, 1 June 2026](https://english.shanghai.gov.cn/en-Latest-WhatsNew/20260601/89a94ac072f842eeacf82fcf0cce8261.html)
4. [Lingang official — underwater data-centre performance summary](https://www.lingang.gov.cn/html/website/lg/English/CityofInnovation/InnovationProjects/2021462605065908225.html)
5. [Uptime Institute — Global Data Center Survey 2025](https://datacenter.uptimeinstitute.com/rs/711-RIA-145/images/2025.Annual.Survey.Report.pdf?version=0)
6. [ISO/IEC 30134-2:2026 — Power Usage Effectiveness](https://www.iso.org/standard/30134-2)
7. [ISO/IEC 30134-6:2021 — Energy Reuse Factor](https://www.iso.org/standard/71717.html)
8. [ITU-T L.1510 (2025) — environmental KPIs for digital infrastructure](https://www.itu.int/rec/dologin_pub.asp?id=T-REC-L.1510-202510-I%21%21PDF-E&lang=e&type=items)
9. [Applied Energy — port hydrogen-electricity scheduling, 2025](https://www.sciencedirect.com/science/article/pii/S0306261925006154)
10. [Applied Energy — offshore-wind zero-carbon port microgrids, 2025](https://www.sciencedirect.com/science/article/pii/S0306261925014023)
11. [PtX Hub / CSIR — South African port load-connection capacity, September 2025](https://ptx-hub.org/wp-content/uploads/2025/09/PtX-Hub-Load-Connection-Capacity-Assessment-of-South-African-National-Ports_final.pdf)
12. [PtX Hub / CSIR — South African port shore-power demand, September 2025](https://ptx-hub.org/publication/shore-power-demand-for-south-african-ports/)
13. [IMO — 2023 Strategy on Reduction of GHG Emissions from Ships](https://www.imo.org/en/ourwork/environment/pages/2023-imo-strategy-on-reduction-of-ghg-emissions-from-ships.aspx)

---

**Claim ceiling:** This document establishes an evidence-backed research and benchmark direction. It does not demonstrate that a subsea data centre is economically or ecologically superior, that a candidate port has firm spare capacity, that any proposed hybrid plant is feasible, or that any Luminous system is ready to dispatch or control real port infrastructure.


## 11. Research update — objective-specific flexibility and grid-safe compute (19 November 2026)

A new article became available online on **14 November 2026**:

Yang et al., *Power-energy profiling of AI data center workloads reveals objective-specific grid flexibility and coordination risks*, **Advances in Applied Energy**, DOI [10.1016/j.adapen.2026.100322](https://doi.org/10.1016/j.adapen.2026.100322).

Its Alibaba workload analysis covers 600,245 AI jobs. It reports that the top 5% by average power intensity and top 5% by cumulative energy overlap by only 19.10%, while the highest-energy 5% account for 85.23% of estimated job energy in the studied trace. It finds that TOU and carbon objectives select different workload populations. Independent shifting increased aggregate peaks in the studied cases; a no-new-peak admission rule retained 73.88–82.51% of TOU and 54.36–59.72% of carbon objective value.

This materially changes the programme's workload-flexibility design:

1. Do not model "flexible compute" as one percentage. Maintain separate power-intensity, total-energy, duration, deadline slack, checkpoint cost, useful-work, hardware and location profiles.
2. Generate distinct candidate schedules for demand response, TOU, carbon, local congestion and adequacy objectives.
3. Independently replay the **aggregate** PCC/system trajectory after every proposed move. Reject moves that create a new peak or violate time-varying PCC, feeder, ramp, reserve or service envelopes.
4. Include deferred-work recovery over the full horizon. A dispatch-window-only comparison can hide a rebound peak later.
5. Compare objective-only optimization with no-new-peak admission and network-aware constraints. Report the objective value sacrificed for the grid-impact constraint.
6. Preserve the difference between site-peak screening, feeder/transformer deliverability, system adequacy and dynamic voltage/frequency qualification.

The first synthetic reference case is being developed separately in Symtropy under [PORT-NEXUS-OBJECTIVE-FLEX-001 #6211](https://github.com/Luminous-Dynamics/symtropy/issues/6211). Its small 8-slot case deliberately demonstrates the mechanism; it is **not** a reproduction of the cited study or South African load data.

### New benchmark gates

| Measure | First gate | Stretch research target |
|---|---|---|
| Objective fidelity | Distinct price, carbon, peak and local-congestion schedules; report job-selection overlap | Improve objective value without reducing service-equivalent work |
| New-peak admission | Zero admitted candidates exceeding the frozen full-horizon peak cap | Match a time-varying locational PCC/feeder envelope |
| TOU value retention | Reproduce published no-new-peak methodology on the exact matched trace where accessible | >82.51% retained benefit, with all service/grid hard gates intact |
| Carbon value retention | Reproduce published no-new-peak methodology on the exact matched trace where accessible | >59.72% retained benefit, with all service/grid hard gates intact |
| Workload quality | Same useful work, deadlines, latency and checkpoint obligations across strategies | Held-out workload/site/hardware generalization |
| Rebound | Full-horizon replay, with no deferred work omitted | Repeated events with realistic recovery/fatigue constraints |

The stretch percentages deliberately exceed the upper ends of one published study's scenario range. They are research hypotheses, not current Luminous results, universal targets or an apples-to-apples leaderboard claim.

### Grid interaction is a separate qualification axis

ENTSO-E's May 2026 report, *Data centres and the power system: expected trends, challenges, and opportunities*, notes that UPS-based architectures can rapidly disconnect large loads during minor disturbances; modern AI facilities can experience multi-megawatt load changes over milliseconds. It also distinguishes workload, cooling and on-site-generation flexibility, and explains why grid-safe operation must precede grid-supporting participation.

Accordingly, the port-compute benchmark must separately test:
- voltage/frequency ride-through and local protection boundaries;
- ramp-rate control, disturbance-driven disconnection and staged reconnection;
- UPS/BESS state, critical-load reserve and cooling continuity;
- commanded demand response versus involuntary load trip;
- post-event recovery and synchronized workload restarts.

A sudden protective trip is not a successful demand-response service. The initial programme remains read-only and does not authorize OT/SCADA or protection-system writes.

Source: [ENTSO-E — Data centres and the power system: expected trends, challenges, and opportunities (May 2026)](https://eepublicdownloads.blob.core.windows.net/public-cdn-container/clean-documents/Reports/2026/FINAL_ENTSO-E_Data_Centres_260430.pdf).


## 12. Industry update — event-triggered port microgrids, standards and outcome evidence (19 November 2026)

### 12.1 Port-operations baseline: berth/logistics-aware energy scheduling

Li et al., *Microgrid energy optimal scheduling in ports incorporating the bidirectional event trigger mechanism*, *Journal of Cleaner Production* 568, article 148666 (15 June 2026), DOI [10.1016/j.jclepro.2026.148666](https://doi.org/10.1016/j.jclepro.2026.148666), provides a stronger port-energy comparator than a generic EMS.

Its Tianjin Dongjiang Port case jointly schedules berth and quay-crane assignments, renewable use, grid procurement and storage in a day-ahead hourly MILP. A real-time layer updates at 15-minute intervals and re-solves a one-hour horizon only when a bidirectional logistics-side / energy-side event trigger fires. Reported case outcomes include 18.64% and 22.44% lower operating cost than the study's MPC and logistics-event-trigger baselines, with online solution time lower by 88.90% and 77.76%, respectively.

These are case-specific published results, not a direct apples-to-apples comparison with this programme. They establish a mandatory comparator.

**New paradigm:** berth and logistics operations should not be a fixed load trace fed into an energy optimizer. Their decisions create shore-power and cargo-handling demand. Compute jobs, in turn, are a new flexible service class competing for the same headroom. The integrated problem must co-schedule berth/shore power, cranes/yard, compute queues, renewable supply, storage and other port services across multiple time scales.

The first fair comparison should reproduce the published day-ahead hourly + event-triggered 15-minute design on its own case where data and method are available. The Luminous extension should then test whether adding compute workload contracts and aggregate PCC/feeder admission yields a better cost/service/grid/compute Pareto frontier without degrading port logistics service.

Source: [ScienceDirect — BETM port microgrid scheduling](https://www.sciencedirect.com/science/article/pii/S0959652626012072).

### 12.2 Grid-interconnection and disturbance-readiness comparator

NERC's Board adopted its first three Computational Load Ownership & Operations standards on 26 October 2026. NERC submitted them to FERC on **16 November 2026**; NERC currently lists the CLO standards as pending regulatory filing.

- CLO-001-1: computational-load interconnection, studies and modelling data.
- CLO-002-1: operational data, communications and response to operating instructions.
- CLO-003-1: protection coordination and disturbance monitoring.

This is an emerging U.S. bulk-power reliability framework—not yet a final FERC-approved obligation as of this snapshot, and not a South African grid-code requirement. Use it as an engineering comparison only; maintain a separate South African/NTCSA requirements profile before any domestic project claim.

The port-compute evidence contract should preserve separate identities for facility owner/operator, PCC boundary, planning model, operating data, communications path, disturbance record and protection-study reference. Intentional demand response must not be confused with involuntary disturbance-driven disconnection.

Sources: [NERC Large Loads Action Plan](https://www.nerc.com/initiatives/large-loads-action-plan); [NERC CLO standards status](https://www.nerc.com/standards/reliability-standards/clo); [NERC filing coverage, 17 November 2026](https://www.publicpower.org/periodical/article/nerc-seeks-ferc-approval-proposed-reliability-standards-tied-integration-computational-loads).

### 12.3 Environmental outcome reporting — evidence as infrastructure

Alavi-Borazjani et al., *Environmental KPI Reporting in Ports: Patterns, Gaps, and Opportunities for Systemic Change*, *International Journal of Environmental Research* 20, article 166 (25 May 2026), DOI [10.1007/s41742-026-01130-4](https://doi.org/10.1007/s41742-026-01130-4), benchmarks 403 environmental KPIs across 25 ports. It finds uneven and fragmented disclosures, with outcome-oriented and integrated indicators under-represented.

This is a useful market gap for Sol-Atlas and Mycelix. We should not compete by publishing the largest KPI catalogue. We should make each KPI reproducible and decision-relevant:
- definition, formula, unit and boundary;
- source artifact, owner, timestamp and rights;
- meter/calibration or model/assumption identity;
- uncertainty and coverage;
- target and matched counterfactual;
- observed result versus installed capacity or announced plan;
- review, correction and supersession history.

The key outcome set should join port throughput/service, ship auxiliary energy avoided, shore-power delivery, grid import/ramp, renewable matching, compute useful work, emissions, water, habitat/thermal observations, climate resilience and local benefit distribution.

Source: [Springer Nature — Environmental KPI Reporting in Ports (2026)](https://link.springer.com/article/10.1007/s41742-026-01130-4).

### 12.4 Subsea data-centre capacity: operating versus planned

Shanghai Lingang entered operation in May 2026. Public reporting identifies an initial 2.3 MW demonstration phase and a 24 MW planned overall capacity. Reported seawater cooling, offshore-wind connection, 22.8% electricity reduction and zero freshwater use are useful comparison claims, but are not independently audited lifecycle results in this benchmark.

Do not compare the planned 24 MW with operating capacity. Match phase, IT load/utilisation, meter boundary, cooling auxiliary load, weather and environmental scope.

Sources: [Shanghai Government / China Daily, 1 June 2026](https://english.shanghai.gov.cn/en-Latest-WhatsNew/20260601/89a94ac072f842eeacf82fcf0cce8261.html); [Shanghai project launch description](https://english.shanghai.gov.cn/en-Strength-Scitechcenter/20250611/c60460fe62e440d08b62be991cade0c9.html).

## 13. New execution priorities

1. **Multi-timescale co-scheduling:** day-ahead berth/crane/shore-power/compute/energy plan; 15-minute event-triggered updates; faster local controls remain independently authorized.
2. **Bidirectional event detection:** trigger on both logistics deviations and energy/grid deviations; extend to compute queue/deadline, renewable forecast, PCC headroom and marine conditions. Measure both avoided optimization calls and the cost of delayed response.
3. **Shared constraint ledger:** PCC, feeder, transformer, export, ramp, reserve and connection headroom must not be allocated twice across shore power, compute, cooling and storage.
4. **Service-equivalent benchmarking:** never accept lower port throughput, missed shore-power windows, compute deadline misses, or unserved critical load as hidden savings.
5. **Evidence-first environmental outcomes:** distinguish capacity installed, project announced, commissioned, measured and independently audited. Publish only claims supported by the corresponding evidence class.

The current Symtropy tool is intentionally narrower: it independently checks fixed workload placements against declared PCC and feeder trajectories. It is an admission primitive for this larger port-logistics/energy/compute coordination programme, not the joint optimizer or a grid-study substitute.


## 14. Ocean cooling and subsea cable resilience — research update (19 November 2026)

### 14.1 Warm-water hybrid cooling is the meaningful comparator

Mahmood, Bayhan and Al-Ansari, *Thermo-economic analysis of depth-optimised hybrid seawater cooling for offshore data centres in the shallow Arabian Gulf*, Applied Thermal Engineering 309 (November 2026), DOI [10.1016/j.applthermaleng.2026.133740](https://doi.org/10.1016/j.applthermaleng.2026.133740), models a 15 MW data centre and a three-mode cooling system:
1. direct seawater cooling where intake conditions permit;
2. partial water-side economiser plus mechanical chiller;
3. chiller-only fallback when seawater is too warm.

The model reports 17.0 GWh/year system electricity versus 33.1 GWh/year for its air-cooled baseline and 19.5 GWh/year for a binary water-side economiser; it estimates a 22.1% reduction in 20-year lifecycle cost. A ±1°C seawater temperature bias changes modeled annual electricity cost by −6.4% to +7.4%.

These are modelled results under a particular Arabian Gulf boundary, not field measurements and not a South African prediction. The exact energy boundary must be reconciled before numerical comparison with a whole-facility PUE or another cooling system.

**Spatial implication for Sol-Atlas:** a candidate must expose seasonal and depth-resolved seawater temperature, stratification, currents, bathymetry, intake/discharge geometry, recirculation possibility, cable landing and maintenance access. A surface-temperature map or nearest-shore distance is insufficient evidence for a cooling design.

Source: [Applied Thermal Engineering — DOI 10.1016/j.applthermaleng.2026.133740](https://doi.org/10.1016/j.applthermaleng.2026.133740).

### 14.2 Subsea cable route diversity must be spatial and failure-domain aware

Palmer-Felgate, *Global Submarine Cable Faults and Repair Dynamics: A Statistical Analysis*, first published 16 September 2026, DOI [10.3233/NICSP260018](https://doi.org/10.3233/NICSP260018), analyses 3,224 valid submarine telecommunications cable repair records across 137 countries. It reports 150–200 telecom cable repairs per year, approximately 80% associated with fishing and anchoring, recurring multiple-cable incidents and repair delays approaching 50 days in some contexts.

**Strict scope boundary:** these are global submarine telecommunications cable observations. They are not failure-rate statistics for submarine power cables, a site-specific risk probability, or a forecast for a South African route.

**Spatial paradigm:** two lines on a map are not necessarily two independent routes. A Sol-Atlas route comparison must expose common landing stations, beach manholes, ducts, trench/corridor overlap, shared geohazards, fishing/anchoring/dredging exposure, common substations, inland backhaul and repair-permit dependencies. Unknown failure-domain overlap must remain unknown; do not promote logical redundancy to physical independence.

Source: [SAGE / NATO Science for Peace and Security — DOI 10.3233/NICSP260018](https://doi.org/10.3233/NICSP260018).

### 14.3 Cross-stack execution link

Symtropy has started [OCEAN-COMPUTE-THERMAL-CABLE-001 #6228](https://github.com/Luminous-Dynamics/symtropy/issues/6228) and draft [PR #6229](https://github.com/Luminous-Dynamics/symtropy/pull/6229). It adds a machine-readable stress matrix for hybrid cooling, marine heatwave/fouling/failure cases, cable common-cause and repair-delay scenarios. The source JSON has 10 thermal and 12 cable scenarios and has been parsed successfully. The same PR now includes a standalone 0D steady-state seawater heat-balance verifier and a frozen fixture with three synthetic cooling modes plus four hostile cases; its JSON parses and the three mode balances were independently recalculated. Rust tests remain authored but unexecuted.

Sol-Atlas remains the owner of spatial context and evidence lineage. It should supply source-qualified geography and expose uncertainty; Symtropy runs bounded physical/recovery scenarios; Symthaea proposes alternatives; Mycelix records permissions, approvals and observations. Mycelix [ASSURE-PORTFOLIO-CUT-002 #5762](https://github.com/Luminous-Dynamics/mycelix/issues/5762) remains the owner for multi-valued/transitive common-cause dependency identities; Symtropy should consume those references rather than implement a second dependency graph. None of these layers turns a map or a simulation into a marine permit, cable route approval or proof of thermal safety.


## 15. Multi-timescale coordination: compute, cooling and port operations (19 November 2026)

### 15.1 Seaport VPP under heterogeneous load uncertainty

Zhang et al., *Scenario-driven day-ahead–intraday coordination of seaport virtual power plants under heterogeneous load uncertainty*, Sustainable Energy Research 13, article 38 (11 September 2026), DOI [10.1186/s40807-026-00269-0](https://doi.org/10.1186/s40807-026-00269-0).

The study models ship shore-power and electric heavy-duty truck battery-swap loads as scenarios, derives crane demand from terminal operations, and uses day-ahead dispatch plus intraday rolling correction. Its case reports 9.79% lower total operating cost and 52.11% lower maximum electricity-purchase deviation with storage, with an 18.4% reduction in execution-stage cost under the proposed coordination method. An important reproducibility boundary: its main case treats wind/PV output as a forecast sequence rather than independently stochastic input.

Sol-Atlas should preserve the different source semantics: arrival/berth-derived demand scenarios, operation-derived crane demand, storage state, and forecast-vintage renewable supply. Do not describe all source/load series as equally stochastic if the reference study did not.

Source: [Springer Nature — seaport VPP coordination](https://link.springer.com/article/10.1186/s40807-026-00269-0).

### 15.2 Liquid-cooled compute demand response is electro-thermal

*Exploring the Demand Response Potential of Liquid-Cooled Data Centers*, Applied Energy 412 (1 June 2026), DOI [10.1016/j.apenergy.2026.127679](https://doi.org/10.1016/j.apenergy.2026.127679), uses an electro-thermal coupled model and coordinates server and liquid-cooling response. This reinforces that the compute cluster is not the only flexible resource: cooling pumps, heat exchangers, chillers and thermal state have their own dynamics and constraints.

Source: [Applied Energy — liquid-cooled data-centre demand response](https://www.sciencedirect.com/science/article/pii/S0306261926003314).

### 15.3 Thermal storage can shift peaks without reducing annual energy

*AI-based model predictive control of chiller plant with thermal energy storage combining a water-side economizer system in data centers*, Energy and Buildings (15 August 2026), DOI [10.1016/j.enbuild.2026.117668], reports up to 11% lower chiller energy, 5% lower whole-cooling-system energy and 4% lower cost in its modeled case.

DOE FEMP's data-centre M&V addendum notes that chilled-water TES can shift demand from high-cost periods and add resilience even when annual chilled-water-plant electricity rises. Accordingly, report separately:
- peak kW and duration;
- cooling-plant kWh and whole-facility kWh;
- tariff cost and carbon emissions on their own time/location signals;
- thermal reserve / autonomy and resilience;
- chiller/pump constraints, losses and service temperature.

Do not call all peak-shifting or cost-shifting "energy savings".

Sources: [Energy and Buildings — chiller/TES/WSE MPC](https://www.sciencedirect.com/science/article/abs/pii/S0378778826007280); [DOE FEMP Data Center M&V Addendum](https://www.energy.gov/sites/default/files/2025-09/femp_mv_5_0_data-center-addendum_092025.pdf).

### 15.4 New multi-domain re-plan trigger work

Symtropy has started [PORT-NEXUS-MULTIRATE-EVENT-001 #6255 / draft PR #6257](https://github.com/Luminous-Dynamics/symtropy/pull/6257), a read-only, std-only Rust event-trigger monitor with a 12-case synthetic fixture and 21 authored tests (not yet executed). It composes the BETM-style logistics/energy trigger with additional signals for compute deadline slack, cooling margin, thermal-store reserve and evidence freshness.

Sol-Atlas should provide spatial and source-qualified inputs with separate observation/valid/first-available/decision times. The monitor rejects future-available evidence and marks missing/stale critical signals incomplete. A trigger only requests re-planning; it does not reserve power, change schedules or actuate physical assets.

Benchmark against fixed 15-minute and hourly replanning, one-sided energy triggers, logistics+energy BETM, and the multi-domain trigger. Any reduction in optimizer invocations is only an improvement when hard-grid/thermal misses and port/compute service outcomes do not worsen.


## 16. Port-city energy–water nexus: compute heat and seawater desalination (19 November 2026)

### 16.1 Data-centre heat can be a water-service input—but only with a qualified sink

Yang, Konstantinou and Hou, *Enabling grid-interactive data center-desalination coordination through thermo-electric coupling-based waste heat utilization*, Applied Energy 408 (1 April 2026), DOI [10.1016/j.apenergy.2026.127428](https://doi.org/10.1016/j.apenergy.2026.127428), links compute-workload heat generation to seawater reverse-osmosis feedwater temperature and temperature-/salinity-dependent RO specific energy use. Its two-stage stochastic planning/operation study reports an 86.5% total annualized cost reduction in its specific case, with waste-heat synergy contributing 17%.

The 86.5% is a system-level case result and must not be attributed solely to heat recovery. The 17% is the more appropriate heat-synergy comparison, but still requires exact-boundary reproduction before direct benchmarking.

Spatial/evidence implications:
- verify an actual, time-matched downstream water demand before assigning value to recovered heat;
- preserve feed source, flow, temperature, salinity/species, intended-use boundary and rights;
- locate intake/outfall, receiving-water body, brine discharge and protected/ecologically sensitive areas;
- preserve the receiving environment and environmental authorization as separate evidence;
- do not equate feedwater preheat with product water, RO performance improvement or potable-water quality.

Sources: [Applied Energy](https://www.sciencedirect.com/science/article/pii/S0306261926000802); [University of Hong Kong repository](https://hub.hku.hk/handle/10722/379334).

### 16.2 Heat pump versus ORC depends on temperature and market conditions

Hong et al., *Thermal design-based waste heat utilization strategies for liquid-cooled data centers in third-generation district heating networks*, Energy Conversion and Management 368 (15 November 2026), DOI [10.1016/j.enconman.2026.122025](https://doi.org/10.1016/j.enconman.2026.122025), models a 120 MW liquid-cooled data centre with 50–60°C waste heat and compares high-temperature heat pumps with ORC power recovery. It reports an average COP of 3.46 for one selected heat-pump working fluid and 5.80% thermal efficiency for the ORC fluid, with the preferred option depending on the electricity-to-heat price ratio (crossover regions around 2.7 and 3.4 in that case).

The study points out that where liquid cooling already avoids chiller duty, heat-pump recovery may add net electrical load unless it displaces a genuine heat source elsewhere. This should shape the spatial screening: a nearby process/building/heat sink must be characterized by temperature, flow, load profile, distance, delivery infrastructure, ownership and contracted offtake—not merely distance to a data centre.

Source: [Energy Conversion and Management](https://www.sciencedirect.com/science/article/pii/S0196890426009945).

### 16.3 Dual-purpose seawater systems are research comparators, not site guarantees

Chen et al., *A dual-purpose seawater evaporation system for data center cooling and freshwater production: design and performance analysis*, Applied Thermal Engineering 287 (February 2026), article 129458, models seawater evaporation for simultaneous data-centre cooling and freshwater production. Its two study configurations report freshwater generation around 10,004 and 10,744 tonnes/day and power reductions of 41.5% and 40.0% against their study baselines.

These are modeled outcomes, not guaranteed commercial output, qualified potable water or proof of acceptable brine/thermal discharge. Keep the proposed architecture separate from conventional SWRO and evaluate feed chemistry, recovered water quality, brine concentration, outfall impacts and component maturity.

Source: [Applied Thermal Engineering](https://www.sciencedirect.com/science/article/abs/pii/S1359431125040505).

### 16.4 Cross-stack integration

Symtropy opened [PORT-NEXUS-WATER-HEAT-001 #6258 / draft PR #6259](https://github.com/Luminous-Dynamics/symtropy/pull/6259). It creates a matched application benchmark and synthetic 0D heat-interface fixture. The fixture partitions 10 MW IT heat into 6.27 MW useful feedwater preheat, 1.5675 MW transfer loss and 2.1625 MW residual cooling rejection. It deliberately leaves RO SEC change and qualified product-water volume null because no admitted RO temperature/salinity response or independent product-water evidence is supplied.

Sol-Atlas remains the spatial owner for actual site water demand, intake/outfall, feedwater source and chemistry evidence, brine receiving environment, water rights and environmental constraints. Symtropy owns deterministic coupling and scenario replay; the water process, quality and closure authorities remain #5355/#2152/#4958; Mycelix carries source/approval/metrology evidence. No heat-balance pass becomes a potable-water or discharge-compliance claim.


## 17. South African paired-site evidence update — Ngqura / Coega and Durban (19 November 2026)

### 17.1 Use two complementary cases, not one premature winner

The September 2025 PtX Hub / CSIR load-connection assessment ranks Ngqura first for reported spare connection capacity and lowest connection risk (19 MVA, estimated 17.1 MW at the report's 0.9 power-factor assumption, low risk). East London follows at 18 MVA / 16.2 MW, then Gqeberha at 15 MVA / 13.5 MW. Durban is 4 MVA / 3.6 MW, medium risk. Cape Town and Saldanha report zero spare capacity in that pre-feasibility study.

The companion PtX Hub / CSIR shore-power study, using vessel activity from 2018–2025, identifies Durban as the highest shore-power-demand port, followed by Cape Town. Its 2024–2025 metric table gives Durban a 42 MW peak-utilization maximum and 20 MW peak-average maximum. Those figures are study-specific historical load estimates, **not** a firm simultaneous connection requirement or a utility offer.

This suggests:
- **Ngqura / Coega** as the first integration-study candidate: more reported pre-feasibility headroom and an adjacent industrial ecosystem with existing project records worth interrogating.
- **Durban** as the high-demand, constrained-grid stress test: a valuable case for workload flexibility, berth/shore-power coordination, storage and event-triggered operation.

Neither is selected for construction. Spare substation capacity is not a committed connection; demand rank is not a measured current load profile.

Sources: [PtX Hub / CSIR Load Connection Capacity Assessment](https://ptx-hub.org/fr/publication/load-connection-capactiy-assessment-of-south-african-national-ports/); [PtX Hub / CSIR Shore Power Demand](https://ptx-hub.org/publication/shore-power-demand-for-south-african-ports/); [IKI publication record — connection assessment](https://www.international-climate-initiative.com/en/iki-media/publication/load-connection-capacity-assessment-of-south-african-national-ports-new696e3fd8d74ab589496258/).

### 17.2 Ngqura / Coega project records: model status, not nameplate ambition

- **7 MW hybrid renewable + 6 MWh BESS:** TNPA tender TNPA/2023/10/0020/46114/RFP sought design, permitting, build, commissioning and seven years of operations/maintenance for 7 MW solar PV plus vertical-axis wind and 6 MWh storage. The official eTender entry is closed. As of this screening update, commissioning/as-built meter evidence is not established; do not count the system as operating supply.
- **Solar-powered seawater desalination:** TNPA's June 2024 media statement sought two solar-powered seawater desalination plants, including approximately 0.5 ML/day for Ngqura and 0.8 ML/day for Port Elizabeth. The RFP is not proof of operating output, product-water quality or water rights.
- **Coega Green Ammonia:** SLR's project notice dated 21 January 2026 states that the proposed 1,400 MW green-hydrogen electrolyser and ammonia plant received environmental authorization on 16 January 2026. This does not prove FID, construction, grid connection, renewable supply, water supply or operating demand. Treat it as a proposed future industrial load and potential infrastructure/renewables coordination partner, not an existing load or energy source.
- **Ngqura LNG pathway:** TNPA signed a 25-year terminal operator agreement for a planned LNG regasification facility in May 2026. The cited public statements describe a long-dated facility and a gas-to-power pathway. It is not existing generation, not renewable energy, and must be assessed explicitly for fuel lock-in, methane and lifecycle emissions rather than labeled clean.

Sources: [Official TNPA Ngqura renewable tender](https://transnetetenders.azurewebsites.net/Home/TenderDetails?Id=46114); [TNPA June 2024 media statement on renewable and solar desalination RFPs](https://www.transnetnationalportsauthority.net/Corporate%20Affairs/Press%20Releases/Media%20Statement%20-%20TNPA%20seeks%20proposals%20for%20renewable%20energy%20projects%20in%20Nelson%20Mandela%20Bay-June%202024.pdf); [SLR Coega Ammonia authorization notice](https://www.slrconsulting.com/afr/public-documents/coega-ammonia/); [SAnews TNPA LNG agreement](https://www.sanews.gov.za/index.php/south-africa/transnet-signs-deal-r22-billion-gas-facility-project).

### 17.3 Adjacent Coega industrial renewable/storage node

A separate private industrial-energy project strengthens the Coega-area evidence picture, but must not be mistaken for port-owned capacity.

Fedgroup's 30 October 2026 press release, carried by Bizcommunity and reported by Engineering News, says:
- a 7 MWp solar PV plant supplies Coega Steels' new rolling mill;
- the company/funder reports nearly 6 million kWh generated to that date;
- a 50 MWh BESS was nearing commissioning;
- the rolling mill has 150,000 tonnes/year capacity;
- a further 19.8 MWp solar expansion is planned by the end of 2027.

A 6 November 2026 report says the additional 19.8 MWp phase's funding arrangement had not been finalized. Thus the solar project is a reported existing/operating asset, the BESS commissioning/operating status still needs primary evidence, and the 19.8 MWp expansion remains planned.

Sources: [Fedgroup press release via Bizcommunity, 30 Oct 2026](https://www.bizcommunity.com/PressOffice/PressRelease.aspx?ai=272996&cid=752422); [Engineering News, 30 Oct 2026](https://www.engineeringnews.co.za/article/fedgroup-funds-renewable-energy-infrastructure-for-coega-steels-new-rolling-mill-2026-10-30); [The National, 6 Nov 2026](https://thenational.co.za/news/2026-11-06-fedgroup-backs-coega-steels-energy-expansion-amid-rising-power-demand/).

This is a separate private Coega SEZ industrial site, not a TNPA asset. Geographic proximity does not establish a shared PCC, wheeling, export, aggregation, reserve, dispatch, heat or water right. Obtain the exact meter boundary, as-built and commissioning evidence, interval solar generation, BESS power/usable energy/SoC/availability, and any written network/contract rights before including it in a port-city dispatch portfolio.

### 17.4 Marine observation access: useful leads, not automatically qualified design data

- **Ngqura:** CSIR WaveNet identifies a directional Datawell Waverider buoy at 33°50'00"S, 25°43'00"E, in 21 m of water, approximately 2 km off the breakwater tip. The public information page describes half-hourly transmission and near-real-time processing/storage. The public pages carry an explicit warning that displayed values are estimates for display and should not be used to draw conclusions. For engineering use, obtain the owner data product and its timestamps, QC flags, calibration, measurement depth, coverage, uncertainty, access rights and applicability.
- **Durban:** CSIR's WaveNet history page states that a Teledyne RD Instruments ADCP has operated in real time off the Port of Durban since 2002, providing ocean-current and wave information. The current station status, sensor configuration, depth bins, uptime, calibration, data rights and historical gap structure must be reconfirmed before use.

These sources create an actionable marine-data intake path for the paired benchmark, but do not replace site bathymetry, vertical temperature/salinity profiles, benthic habitat surveys, receiving-water baseline, plume validation, cable route survey or environmental authorization.

Sources: [CSIR WaveNet Ngqura station information](https://wavenet.csir.co.za/OnlineData/Ngqura/ngqura.htm); [CSIR WaveNet Ngqura wave data page](https://wavenet.csir.co.za/OnlineData/Ngqura/ngqurawaveD.htm); [CSIR WaveNet history and Durban ADCP note](https://wavenet.csir.co.za/history.htm).

### 17.5 Paired-site qualification sequence

1. Start a data-room and metering request for Ngqura/Coega: utility-confirmed import/export, current firm supply, upstream network constraints, meter series, shore-power inventory, status of renewable/BESS/desalination procurement, Coega industrial project load and water plans.
2. Run Durban as a parallel stress-test case: verify actual berth-level ship load profiles, transformer/feeder constraints, operating data access and the scope of its CSIR ADCP record.
3. Acquire WaveNet raw/owner-qualified data and reconcile source/observed/first-available/decision times through Symtropy #6209 and Sol-Atlas source provenance.
4. Use Mycelix [PORT-CITY-SERVICE-GOLDEN-TRACE-001 #6343 / draft PR #6344](https://github.com/Luminous-Dynamics/mycelix/pull/6344) as the shared service-equivalence comparator; no owner-specific system is duplicated.
5. Keep subsea compute as a later option only after onshore integration, water, power, fibre diversity, marine habitat, bathymetry, thermal plume and cable repair evidence is independently qualified.

New owner issue: [ZA-PORT-CITY-PAIRED-SITE-001 #948](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/948).


## 18. Commercial subsea-compute competitor evidence (19 November 2026)

### 18.1 Underwater data centres are now commercial comparators

The industry benchmark must move beyond treating Microsoft's Project Natick as the only serious subsea reference.

- **Hainan / Lingshui, China:** Shanghai Municipal Government's Xinhua republication (11 June 2025) reports Hainan UDC in operation since December 2022; a further module was added in February 2025 with an additional module for over 400 high-performance servers. The report describes ten companies signed up for capacity, including AI training/inference and marine-science use. The reported "30,000 gaming PCs" and 7,000 queries/s figures lack the workload, quality, precision, latency and energy boundaries needed for a matched benchmark.
- **Shanghai Lingang / HiCloud:** industry reporting in May 2026 says a 24 MW underwater data centre reached full commercial operation. DCD describes a June 2025 launch, October construction completion and February 2026 trials; Offshore Wind reports a 2.3 MW first phase scaled to 24 MW. Both report PUE about 1.15 or below, but attribute performance to Chinese media or developer claims rather than an independently audited common meter boundary.
- **Material depth conflict:** DCD reports modules around 35 m below the surface; Offshore Wind reports 10 m underwater. This is now encoded as CONFLICTED_UNKNOWN in Symtropy #6283. Sol-Atlas must not select either value for pressure, route, bathymetry or environmental calculations until primary as-built documentation resolves it.
- **Microsoft Natick:** a research pilot, retrieved in 2020, remains a valuable experimental reliability reference; Microsoft reports one-eighth the server failure rate of its land control. It is not a current commercial deployment or an independently generalized availability/MTTR result.
- **Nautilus EcoCore:** adjacent liquid-cooling technology; not a submerged UDC.
- **Panthalassa Ocean-3 / Aikido floating concepts:** floating offshore compute architectures must remain a separate comparator class until operating evidence establishes status.

Sources: [Shanghai Municipal Government / Xinhua (11 June 2025)](https://english.shanghai.gov.cn/en-Strength-Scitechcenter/20250611/c60460fe62e440d08b62be991cade0c9.html); [China government / Xinhua — Hainan UDC (19 Feb 2025)](https://english.www.gov.cn/news/202502/19/content_WS67b5a14dc6d0868f4e8efcaa.html); [DCD — Shanghai Lingang UDC (20 May 2026)](https://www.datacenterdynamics.com/en/news/hiclouds-offshore-wind-powered-underwater-data-center-up-and-running-off-coast-of-shanghai-china/); [Offshore Wind — Lingang UDC (18 May 2026)](https://www.offshorewind.biz/2026/05/18/china-puts-worlds-first-offshore-wind-powered-underwater-data-centre-into-operation/); [Microsoft Research — Project Natick](https://www.microsoft.com/en-us/research/project/natick/); [Symtropy subsea-compute industry benchmark #6280 / draft PR #6283](https://github.com/Luminous-Dynamics/symtropy/pull/6283).

### 18.2 What Luminous must do differently

The product cannot be "we can put servers underwater". Commercial competitors already report that capability. Our differentiator should be a **Port-City Compute and Ocean-Energy Commons**:
- onshore operational core for maintenance, security, low-latency services and local fallback;
- subsea modules as optional batch/HPC/AI capacity for workloads that tolerate latency and data movement;
- offshore wind / renewables, grid, storage and port shore-power coordinated against one PCC/feeder envelope;
- workload placement across onshore/floating/subsea compute with deadline, data-locality, migration and rebound constraints;
- liquid cooling and seawater rejection with explicit heat-source/useful-sink/loss/rejection balances;
- heat reuse only where the local process/water sink is real, time-matched, metered and quality-qualified;
- cable route/common-cause, repair vessel, module retrieval and degraded-mode evidence;
- independent marine/habitat/thermal-plume and cyber/OT trust qualification.

### 18.3 Metric boundary discipline

PUE below 1.15 is a reported Lingang developer/media benchmark, not an independently audited result in the sources above. Whole-facility PUE, module PUE, site grid import, renewable fraction and compute energy per useful workload must be kept separate.

ASHRAE recommends using heat reuse where a viable external sink exists and tracking useful reuse with ERE/ERF alongside PUE, while also emphasizing water-aware cooling. ITU-T L.1328 defines ERF around energy reused outside the data-centre boundary that substitutes for energy otherwise consumed externally; heat extraction alone does not earn useful-reuse credit.

Sources: [ASHRAE AI data-centre energy and thermal efficiency](https://www.ashrae.org/technical-resources/topics-and-initiatives/ai-data-center/energy-and-thermal-efficiency); [ITU-T L.1328 (2025)](https://www.itu.int/rec/dologin_pub.asp?id=T-REC-L.1328-202509-I%21%21PDF-E&lang=e&type=items).

New owner benchmark: [SUBSEA-COMPUTE-INDUSTRY-BENCH-001 #6280](https://github.com/Luminous-Dynamics/symtropy/issues/6280).


## 19. South African port-city demand-response product qualification (19 November 2026)

### 19.1 Local products have materially different response envelopes

The port-city benchmark must distinguish actual South African programme terms rather than import generic flexibility assumptions.

**Eskom Distribution Instantaneous Demand Response (IDR)** public programme page states:
- minimum entry of 10 MW;
- full load reduction within 6 seconds of notification;
- sustain for 10 minutes;
- up to three 10-minute events per day;
- day-ahead scheduling and on-the-day frequency-triggered dispatch;
- special metering requirements;
- standby payment, no energy payment.

Source: [Eskom IDR programme](https://www.eskom.co.za/distribution/instantaneous-demand-response-programme/).

**SAWEM Market Code Rev 2.3 (February 2026)** defines Instantaneous Reserve as generation or demand-side managed load available to respond fully within 10 seconds to a frequency drop and sustain that response for at least 10 minutes. It includes day-ahead reserve and real-time dispatch provisions.

Source: [NTCSA SAWEM Market Code Rev 2.3](https://www.ntcsa.co.za/wp-content/uploads/2026/02/Market-Code-v2.3.pdf).

The Eskom six-second programme profile and the SAWEM ten-second definition are not the same product. Publication of the Market Code does not prove that an individual port or data centre is registered or eligible.

**NTCSA Supplemental Demand Response** public page describes:
- notice from 30 minutes to 6 hours;
- minimum entry equal to the greater of 500 kW or 15% of average plant demand;
- a minimum two-hour response, once per day under the published description;
- 30-minute interval remote metering;
- standby payment plus energy payment.

Source: [NTCSA Supplemental DR programme](https://www.ntcsa.co.za/supplemental-demand-response-programme-2/).

**NRS 048-9 Edition 3** concerns emergency load reduction and restoration. Mandatory emergency load reduction is distinct from voluntary/contracted demand response, and authority remains with the relevant System Operator/utility.

Source: [NRS 048-9 Edition 3](https://www.kaya959.co.za/wp-content/uploads/2024/04/NERSA-Approved-Version-of-NRS-048-9-2023-Edition-3.pdf).

### 19.2 Regulatory status must be time-qualified

NERSA's September 2026 Electricity Regulation Projects Dashboard, last updated 15 September, listed approval of the Market Code as in finalisation with an end-October target and trading rules in consultation. That snapshot cannot establish whether the subsequent final regulatory decision occurred. NTCSA publishes Market Code Rev 2.3 dated February 2026, but project participation, registration, reserve qualification and current settlement/implementation status must be confirmed with NERSA/NTCSA/Eskom.

Sources: [NERSA September 2026 regulatory tracker](https://www.nersa.org.za/files/files/2026/09/ELRStakeholderDashboard-September2026-Rev1-HighRes.pdf); [NTCSA Market Code publication page](https://www.ntcsa.co.za/energy-market-services/wholesale-market-code/).

### 19.3 What this means for Ngqura and Durban

- Ngqura's 17.1 MW estimated spare substation capacity is not 17.1 MW of flexible demand, contracted IDR or available response.
- Durban's 3.6 MW estimate is not flexible demand or response capacity. It is below the published 10 MW IDR entry level for a single resource, but this does not rule in or out aggregation under programme-specific rules.
- The supplemental programme's 500 kW / 15% threshold may be a more accessible starting point, but eligibility, baseline, metering, event response, aggregation and contractual terms need direct confirmation.
- A 30-minute meter may meet the published supplemental programme prerequisite but cannot prove a six-second IDR response.
- For every site, collect real metered baseline, participant and point-of-delivery IDs, special meter/telemetry specifications, response profile, sustain and recovery/rebound, event authority, call frequency, compensation and non-delivery rules.

The source register and screening fixture now include four South African product categories and a paired-site applicability matrix. The fixture deliberately leaves actual site DR eligibility unknown pending direct operator evidence.
