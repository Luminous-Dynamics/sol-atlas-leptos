# PORT-ENERGY-COMPUTE-001 — Port-city energy and compute twin

**Research snapshot:** 19 November 2026  
**Status:** screening architecture, evidence contract and benchmark plan; no site design or performance claim.  
**Tracker:** [#942](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/942)  
**Related programme:** [formulation-matched energy/OPF benchmark](ENERGY_OPF_BENCHMARK_AND_PARADIGM_SHIFT_2026-11-17.md).

## 1. Decision: build the port energy system first; earn the right to put compute offshore

The opportunity is not simply to put servers underwater. It is to make the port-city a coordinated clean-energy, logistics, compute, water and marine-observation system.

The recommended sequence is:

1. **Evidence and digital twin:** establish a source-backed asset registry, hourly load model, grid-connection envelope, metocean and environmental baseline. Start in read-only shadow mode.
2. **Port energy demonstrator:** optimize existing/planned PV, wind, grid supply, batteries, shore power, cargo equipment, cold stores and water systems. Prove dispatch and savings against a matched baseline.
3. **Coastal edge-compute demonstrator:** add a modular, land-based or elevated port-edge compute load that can move delay-tolerant workloads into renewable-surplus periods. Keep latency-critical systems on independently resilient paths.
4. **Floating / nearshore cooling experiment:** only after site-specific bathymetry, ecology, metocean, cable, maintenance and permitting studies. Use a recoverable, serviceable platform and indirect heat exchange.
5. **Subsea compute module:** only if a matched lifecycle study proves a system-level advantage over the coastal land facility and the environmental assessment, cable route, retrieval plan and service model are independently accepted.

**No underwater deployment is selected by this document.** A subsea module is one candidate in a portfolio, not the default. It may reduce mechanical cooling and improve some reliability characteristics, but pressure vessels, subsea power/data cables, marine installation and retrieval, corrosion, biofouling, repair latency, environmental externalities and embodied carbon can erase the apparent advantage.

## 2. Why now — and what is actually demonstrated

### 2.1 Industry state

- Microsoft's Project Natick Phase 2 deployed a 240 kW module with 12 racks, 864 servers and 27.6 PB of disk at the European Marine Energy Centre in Scotland. Microsoft reports a server failure rate one-eighth that of its land control group. The module was powered by locally produced renewable electricity. Its published five-year maintenance-free period is a design plan, not a five-year field-proven service interval. These are valuable engineering data, but one demonstration does not establish a universal cost or environmental advantage. [Microsoft Project Natick](https://www.microsoft.com/en-us/research/project/natick/) · [facts and figures](https://www.microsoft.com/en-us/research/project/natick/facts-and-figures/)
- China's Lingshui/Hainan underwater data-centre project began operation in 2023. A further module connected to the cluster in February 2025 and was reported to accommodate more than 400 high-performance servers. These public figures establish a commercial deployment path, but the public report does not provide a comparable, independently audited PUE, lifecycle carbon, ecological-impact or availability dataset. [State Council / Xinhua, 19 February 2025](https://english.www.gov.cn/news/202502/19/content_WS67b5a14dc6d0868f4e8efcaa.html)
- A January 2026 *Nature Reviews Electrical Engineering* perspective describes underwater data centres supplied by offshore wind, solar and wave energy with seawater cooling. Treat this as a research direction and system concept, not proof that a complete offshore renewable mix is bankable at every site. [Yang et al., 2026](https://www.nature.com/articles/s44287-025-00246-7)
- The IEA's 2026 update reports data-centre electricity demand grew 17% in 2025 and AI-focused data-centre demand grew 50%; its central outlook sees total data-centre electricity consumption roughly doubling from 485 TWh in 2025 to 950 TWh in 2030. This makes energy-aware compute placement and dispatch materially important. [IEA, *Key Questions on Energy and AI*, 16 April 2026](https://www.iea.org/reports/key-questions-on-energy-and-ai)

### 2.2 South African first-screen

**Ngqura is the preferred first energy-system screening case, not yet a chosen underwater site.**

The September 2025 PtX Hub / CSIR load-connection assessment ranked the Port of Ngqura first among eight assessed South African ports for estimated spare capacity and low connection risk: 19 MVA, converted in the study to approximately 17.1 MW using a 0.9 power-factor allowance. The underlying assumptions are 25 MVA firm supply and 6 MVA Notified Maximum Demand; the authors describe this as a pre-feasibility desktop assessment and recommend a supply-authority load application during feasibility. The same assessment estimated 3.6 MW spare capacity and medium connection risk for Durban. These are pre-feasibility screening estimates. The 17.1 MW value is calculated as 0.9 × (25 MVA firm supply − 6 MVA Notified Maximum Demand); the report explains the 0.9 factor as a power-factor allowance. It is not a utility connection offer, a real-time headroom measurement, a feeder-specific limit, or proof that capacity is available when vessels and compute loads coincide. [PtX Hub / CSIR, *Load Connection Capacity Assessment of South African National Ports*, September 2025](https://ptx-hub.org/publication/load-connection-capactiy-assessment-of-south-african-national-ports/)

TNPA issued a tender for a 7 MW renewable-energy plant and 6 MWh battery energy-storage system at Ngqura. The tender scope specified approximately 6,991 kW of solar PV and 9 kW of vertical-axis wind turbines. The connection-capacity study states the project remained in procurement, with tenders closed and under evaluation, as of June 2024. The eTender page shows the tender closing on 16 August 2024. The existence of the tender is evidence of procurement scope, **not** evidence of award, commissioning, operational output, availability or current state of charge. Those facts remain unverified in this screening case. [TNPA tender TNPA/2023/10/0020/46114/RFP](https://transnetetenders.azurewebsites.net/Home/TenderDetails?Id=46114) · [tender technical annexure surfaced on eTenders](https://www.etenders.gov.za/home/Download/?blobName=986dc183-d2c6-4c66-abee-567c92efe9e8.pdf&downloadedFileName=Annexure+F+-+Health+and+Safety+Specification.pdf)

Durban is the important second comparator. The 2025 South African shore-power demand study estimates Durban's peak maximum at 47 MW in FY2018–19 (and 42 MW in FY2024–25, the latest year shown), with average-peak maximum of 20 MW in FY2024–25. Coega/Ngqura's largest listed peak maximum is 12.6 MW in FY2019–20 (9.5 MW in FY2022–23, the latest year with a value shown), with average-peak maximum of 4.3 MW in FY2022–23. The authors construct demand time series from vessel activity and vessel power assumptions; these are modeled maxima, not metered coincident loads or an interconnection study. Durban therefore represents a much larger shore-power stress case, while Ngqura remains a more promising first-grid screening candidate. The years and demand definitions must be retained in every comparison. [Source PDF, Table 3, p. 19](https://ptx-hub.org/wp-content/uploads/2025/10/PtX-Hub-Shore-Power-Demand-for-South-African-Ports-2.pdf). Durban is also a useful international-connectivity candidate: the 2Africa cable lands at the OADC Durban facility. The combination makes Durban a strong future compute-and-shore-power stress case, but its lower pre-feasibility spare-capacity estimate means that a real grid study must precede any load addition. [PtX Hub / CSIR, *Shore Power Demand for South African Ports*, September 2025](https://ptx-hub.org/publication/shore-power-demand-for-south-african-ports/) · [WIOCC 2Africa landing information](https://www.wiocc.net/2africa)

CSIR's 2025/26 programme also identifies sustainable-port hybrid infrastructure and autonomous metocean / water-quality monitoring as national research outputs, with TNPA and regional port authorities as collaborators. This is a natural interface for an evidence-first marine twin. [CSIR Shareholders Compact 2025/26](https://www.csir.co.za/sites/default/files/2025-09/SHAREHOLDERS%20COMPACT%2020256_proof%207.pdf)

### 2.3 Site selection table

| Candidate | Best initial role | Evidence available | Blocking unknowns / cautions |
|---|---|---|---|
| **Ngqura / Coega** | First energy and grid-integration twin | 17.1 MW estimated spare capacity (derived using 0.9 × (25−6) MVA); low screening connection risk; modeled shore-power peak maximum 12.6 MW in FY2019–20; TNPA 7 MW PV/wind + 6 MWh BESS tender | Actual award/commissioning, current feeder headroom, coincident 15-minute load, battery inverter MW, renewable yield, cable route, seabed ecology |
| **Durban** | Later high-demand compute / shore-power stress case | Modeled peak maximum 47 MW in FY2018–19 and 42 MW in FY2024–25; 2Africa landing; 3.6 MW estimated spare capacity | Higher grid-connection risk; modeled vessel demand is not measured coincident load; actual spare capacity at proposed feeder; port congestion, marine hazards and local environmental constraints |
| **Saldanha Bay** | Offshore-wind / industrial-fuels comparison | Deep-water industrial port; offshore wind service/logistics potential in South African siting studies | 0 MW estimated spare capacity in the 2025 connection assessment; industrial and marine-area conflicts require careful study |
| **Richards Bay** | Bulk-port, wind and industrial-energy comparison | 20 MW renewable plant reported as awarded by TNPA in a public media statement; industrial port role | Commissioning and operating performance, actual load/connection envelope, marine ecology, coal-related transition risks |

The site ranking is intentionally multi-dimensional. Ngqura leads the **first energy-model** screen; Durban is compelling for **future compute connectivity and shore-power value**; neither has passed a subsea siting gate. The World Bank / ESMAP strategic framework identifies approximately 95 GW of South African offshore-wind technical potential in suitable areas, but national potential is not a bankable resource estimate for an individual berth or cable corridor. [ESMAP, *A Strategic Framework for Offshore Wind Development in South Africa*](https://www.esmap.org/publications/strategic-framework-offshore-wind-development-south-africa)

## 3. The system architecture

### 3.1 Coupled physical system

Model these as distinct assets, loads and constraints before optimizing them jointly:

- **Generation:** rooftop and carport PV; ground-mounted PV where land-use and glare allow; onshore/coastal wind; offshore wind through a qualified grid connection; floating PV only in sheltered basins after navigation, corrosion and ecological review; wave/tidal only where measured resource and device evidence justify the technology.
- **Storage and flexibility:** batteries for ramps, frequency response and short-duration resilience; thermal storage for cold stores and cooling; flexible compute queues; hydrogen or derivatives for maritime fuel and selected long-duration applications only where lifecycle and round-trip economics justify them. Do not make hydrogen the default data-centre battery.
- **Port demand:** shore power, cranes, yard equipment, reefer containers, workshops, lighting, pumping, desalination, buildings, rail and electric vehicle charging.
- **Compute demand:** port community systems, logistics optimization, digital twins, scientific HPC, marine forecasting, AI training/inference and archival storage. Class each workload by latency, criticality, data locality, privacy, energy flexibility and maximum interruption.
- **Marine environment:** bathymetry, seabed substrate and habitat, current/wave/wind, temperature and salinity profiles, turbidity, dissolved oxygen, acoustic environment, protected-area intersections, fisheries and vessel traffic, cable corridors, sediment movement and cumulative impacts.
- **Heat and water:** prefer indirect, closed-loop heat exchange. Meter freshwater separately from seawater withdrawal and discharge. Model the thermal plume and its seasonal/ecological envelope; do not equate “uses seawater” with “zero water impact”.

### 3.2 Compute placement is a dispatch decision

Subsea or floating compute should be a **controllable demand resource**, not a new inflexible baseload.

- Keep latency-sensitive control systems, port safety systems and real-time operational technology local and independently resilient.
- Shift deferrable training, rendering, batch analytics, simulation, archival processing and selected inference to periods of clean-power surplus, low grid-carbon intensity or low marginal energy cost.
- Define maximum start delay, completion deadline, checkpoint interval, data-transfer budget, workload migration cost and failure recovery for every flexible job.
- Do not move sensitive data offshore merely because energy is available. Data sovereignty, jurisdiction, physical security and operator contracts are independent constraints.
- Compare a coastal onshore edge facility, a floating platform and a subsea module using identical IT equipment, workload traces, network demand and reliability requirements.

### 3.3 Energy-management and control boundary

Start with an advisory system:

`measure → validate → forecast → optimize → independently verify → operator approve → observe`

The first implementation is read-only shadow mode. It must not write to port SCADA, vessel protection, shore-power interlocks, battery protection or grid controls. A future actuation path would require a separate safety case, OT-security review, protection coordination, change authorization, hardware-in-the-loop testing and an operator-owned release process. Unknown or stale observations block new actuation proposals until reconciled.

## 4. Luminous system division of responsibility

| System | Role in this programme | First useful integration | Explicit boundary |
|---|---|---|---|
| **Sol-Atlas** | Geospatial evidence and energy-network view | Port assets, feeders, renewables, storage, loads, cables, bathymetry and protected-area overlays; scenario and evidence comparison | Visualization does not validate a measurement or authorize a design |
| **Symtropy** | Deterministic physical/system replay | Versioned hourly energy balance, storage state, dispatch and event replay; later couple thermal, mooring, platform and marine-process models | A simulation is not field evidence; publish model fidelity and unmodelled physics |
| **Symthaea** | Forecasting, anomaly detection and bounded decision support | Reuse the existing engineering digital-twin / microgrid work: ingest telemetry, maintain uncertainty and prediction residuals, request missing observations, rank workload/dispatch candidates | A forecast or policy proposal is not proof; no direct safety-critical control authority |
| **Mycelix** | Identity, provenance, permissions and evidence lineage | Signed sensor/asset/maintenance/energy events; source and calibration credentials; versioned claims; revocation and dispute trail | Signatures prove origin/integrity, not truth; trust score or consensus must not self-authorize a physical action |

**Existing implementation anchors checked 19 November 2026:** Symthaea contains `symthaea-digital-twin` telemetry and residual primitives plus `symthaea-engineering/examples/microgrid_twin.rs`; Sol-Atlas already has a formulation-matched OPF research contract; Mycelix Supply Chain describes idempotent event ingestion, signed verifiable credentials and hash-linked DKG lineage; Symtropy provides deterministic replay-oriented physical simulation infrastructure. The marine/port integration described here is new work; these repositories do not yet constitute a qualified port digital twin.

Proposed cross-system evidence envelope:

- stable `event_id`, `asset_id`, `schema_version`, sequence number and idempotency key;
- UTC timestamp, time quality, sampling interval and source clock;
- channel, value, SI unit, uncertainty, detection limit and quality flag;
- location plus CRS / vertical datum where relevant;
- sensor make/model, serial or pseudonymous ID, calibration certificate and calibration expiry;
- raw payload digest, parser/version digest, source signature and signer identity;
- model/forecast/optimization version, input-data digest, assumptions and applicability boundary;
- independent verification result, reviewer/approval identity and revocation status.

Do not store large raw time series directly in a consensus ledger. Store content-addressed immutable evidence objects and signed claims pointing to them, subject to retention, privacy and operational-security policy.

## 4.1 Existing cross-project work to compose, not clone

The following open work items already own adjacent problems. This programme adds the port-city / energy-compute case and a shared, source-bound case identity; it should not create duplicate schedulers or physics engines.

- **Symthaea #11572 — MARINE-PORT-ENERGY-CO-DESIGN-001:** ship–port shore-power and berth energy eligibility, interface compatibility, exact delivered/unserved energy and matched emissions boundaries. Reuse its connection/service-window contract.
- **Symthaea #11028 — MARINE-POWER-TRANSIENT-001:** shipboard multi-timescale power-quality and transient assurance. Its documented boundary matters: existing slow energy models are not proof of converter, protection, or transient stability.
- **Symtropy #4582 — PORT-ENERGY-SCHEDULE-VERIFIER-001:** exact power (W), cumulative energy (Wh), connection readiness and schedule oracle. Reuse it for proposed port-energy schedules rather than writing another verifier.
- **Symtropy #3400 / #3404 — POWER-SYSTEM-DIGITAL-TWIN / EVIDENCE:** deterministic energy-system model, source-bound traces and uncertainty handling; extend its evidence model with port and compute assets where appropriate.
- **Mycelix #5974 — MARITIME-CROSS-STACK-GOLDEN-TRACE-001:** cross-owner integration corpus and independent oracle. Extend this with energy, compute, marine-observation and site-evidence traces; do not reimplement the four systems' internal decisions.

The ownership split is therefore: Sol-Atlas owns spatial/network scenario identity and source mapping; Symtropy owns deterministic schedule/physics replay and exact feasibility checks; Symthaea owns bounded prediction and decision support; Mycelix owns identity, permission, provenance, revocation and external-effect receipts. The new port-city case is an integration/benchmark target, not evidence that these interfaces are already wired together.

### Research comparators now available in the adjacent programme

Recent port-energy literature increasingly co-optimizes berth allocation, cranes, shore-power and multi-source dispatch. For example, a 2026 *Journal of Cleaner Production* study models integrated berth/crane/shore-power/multi-source dispatch and reports case-specific cost improvements; a 2026 *Ocean Engineering* study considers port integrated-energy-system planning under uncertainty; another 2026 study coordinates port virtual power plants with vessel and equipment-load uncertainty. Treat these as methods to reproduce under matched data and objective functions, not savings claims transferable to South Africa. Symtropy #4582 tracks the exact citations and comparison caveats.

## 5. Benchmark design — separate tracks, matched inputs

The current Sol-Atlas OPF benchmark already establishes an important rule: same formulation, same data, same machine, same tolerances, independent verifier. Extend that discipline instead of inventing an incomparable “green port score”.

### Track A — energy-system correctness

**Baselines:** grid-only dispatch; fixed/rule-based EMS; same-site PV + battery dispatch; a matched MILP/QP or robust MPC formulation; established power-system toolchains where the model is equivalent.

**Inputs:** identical 8,760-hour load, generation, weather, tariffs, grid-carbon factors, grid constraints, storage efficiencies/degradation, reserve requirements, outages and curtailment. Keep 5-minute or faster transients as a separate dynamic track.

**Metrics:** energy balance error, line/voltage/transformer violations, renewable curtailment, import peak, ramp, reserve shortfall, storage degradation, unserved energy, cost, emissions and independently verified feasibility.

### Track B — port service

Compare the same port demand and vessel call traces across shore-power, cargo-equipment charging, reefer, water and building loads.

**Metrics:** shore-power energy delivered, vessel auxiliary-fuel displacement, port Scope 1/2 emissions, grid peak and ramp, berth service continuity, equipment availability, cost per vessel-hour served, and critical-load survival during outages. Do not count electricity delivered to a ship as zero-carbon unless its hourly source and losses support that claim.

### Track C — compute and cooling

Compare, with the same IT equipment and workload trace:

1. a matched coastal onshore edge facility;
2. an onshore facility with direct liquid cooling / heat recovery;
3. floating nearshore compute with indirect seawater heat exchange;
4. subsea compute with pressure vessel and closed-loop cooling.

**Metrics:** IT kWh per completed workload; facility PUE; hourly carbon-free-energy match; lifecycle carbon per completed workload; freshwater WUE; seawater withdrawal and discharge; cooling parasitic energy; network energy and latency; job deadline misses; availability; maintenance/retrieval downtime; embodied carbon; end-of-life recovery.

### Track D — integrated operation

Compare: (i) fixed port loads + fixed compute; (ii) rule-based port EMS; (iii) energy-only optimizer; (iv) compute-aware optimizer; (v) uncertainty-aware rolling-horizon controller with ecological and reliability constraints. Give all controllers the same forecast and information at decision time. Test forecast error, asset outages, cable faults, weather extremes, tariff shocks, sensor faults and common-cause failures.

### Track E — marine environmental and lifecycle qualification

No single PUE or carbon score can compensate for an unpermitted habitat impact. Establish pre-deployment seasonal baseline, matched reference stations and independently reviewed monitoring for temperature, salinity, dissolved oxygen, turbidity, sediment, benthic habitat, acoustic effects, marine mammals/fisheries, cable route disturbance, invasive species and decommissioning/recovery.

Report observed effects with uncertainty and control-site comparison. The acceptance threshold is the applicable permit and independently agreed ecological envelope—not an arbitrary universal temperature or biodiversity number invented by the software team.

## 6. Proposed targets — research aspirations, not achieved performance

| Dimension | First qualification target | Stretch target / note |
|---|---|---|
| Optimization correctness | 100% independent feasibility checks on the admitted model profile; 0 false passes on negative controls | Publish excluded cases, formulation and exact witness status |
| Renewable integration | At least 80% annual renewable share for the demonstrator, with hourly matching reported | Move toward 90–100% hourly clean-energy matching; annual certificates alone do not count as 24/7 matching |
| Data-centre PUE | ≤ 1.15 at a declared 75–85% IT-load band | ≤ 1.10 at the same boundary and load band; compare under ISO/IEC 30134-2:2026 measurement rules |
| Freshwater | No evaporative freshwater cooling for an ocean-cooled demonstrator; report whole-site WUE | Minimize potable freshwater, while separately publishing seawater withdrawal, discharge, temperature and chemistry |
| Carbon | ≥80% lower lifecycle GHG per useful workload than the matched grid-only baseline before claiming a decarbonization advantage | Approach net-zero lifecycle energy/carbon only after including hardware, vessel work, batteries, cables, maintenance and end-of-life |
| Flexibility | Demonstrate measurable peak-import and curtailment reduction without missed port or compute service requirements | Optimize energy cost, marginal emissions and grid services jointly |
| Reliability | ≥99.9% demand-qualified compute-service availability in the pilot design, with N-1 power/network assumptions made explicit | 99.95%+ with demonstrated independent fallback; no availability claim from simulation alone |
| Economics | Report full TCO and cost per completed compute workload against the matched coastal baseline | Require a positive NPV or an explicit, quantified resilience / port-service benefit; no universal LCOE hurdle transferred from another port |
| Ecology | No permit-envelope exceedance; baseline, reference stations, monitoring and retrieval plan accepted before deployment | Demonstrate recovery after retrieval and disclose adverse as well as beneficial observations |
| Evidence | 100% of site-specific claims linked to source, date, unit, evidence class and applicability | Independent replication and operator-observed data before external performance claims |

The PUE target is intentionally tied to an explicit load band. ITU-T L.1510 (October 2025) cites 1.1 as a best-in-class example and 1.2–1.3 as an industry target at 75–85% load; those values are context, not universal mandates. ISO/IEC 30134-2:2026 provides the measurement framework and does not prescribe a universal pass threshold. PUE alone does not measure water, carbon, workload productivity or marine impacts. [ITU-T L.1510 (10/2025)](https://www.itu.int/rec/dologin_pub.asp?id=T-REC-L.1510-202510-I%21%21PDF-E&lang=e&type=items) · [ISO/IEC 30134-2:2026](https://www.iso.org/standard/30134-2)

### Published port-energy results are comparators, not targets

A 2026 open-access study of a Greek island-port case reports a marine wave/tidal + battery design with 87.96% renewable penetration, an 88.3% lifecycle-carbon reduction, LCOE of €0.179/kWh and a 7.64-year payback under its specific assumptions. A separate 2026 study of a TEN-T port reports a PV/wind/wave configuration at €0.1996/kWh with an 81.6% emissions reduction against its grid baseline. These are useful published comparators, not transferable results for South Africa or proof that wave/tidal is the right Ngqura resource. [Cholidis et al., 30 June 2026](https://link.springer.com/article/10.1007/s40722-026-00509-x) · [2026 multi-objective port-planning study](https://www.sciencedirect.com/science/article/pii/S1755008426001377)

## 7. Paradigm shifts worth pursuing

1. **Port as an energy-compute-water market participant.** Co-optimize port operations, shore power, flexible compute, storage and selected water infrastructure; price operational flexibility rather than treating every load as fixed.
2. **Compute as a virtual storage resource.** Workload queues, checkpointing and migration can absorb renewable surplus without converting electricity into hydrogen and back. This is only useful for workloads with real scheduling slack and acceptable data/network costs.
3. **Ecology-constrained design from day zero.** Marine habitat and cumulative impact are hard feasibility constraints and public evidence layers, not a late-stage compliance appendix.
4. **Reversible ocean infrastructure.** Prefer recoverable modules, standardized connectors, replaceable pressure housings, cable-route minimization and a funded retrieval/end-of-life plan over permanent seabed commitment.
5. **Evidence-carrying operations.** Every forecast, optimization and maintenance recommendation carries input lineage, uncertainty, a validity envelope and independent checks. A recommendation cannot silently become an operating fact.
6. **Multi-fidelity, counterfactual planning.** Use hourly energy optimization for sizing and dispatch; dynamic electrical and thermal models for transients; CFD / marine models for heat and hydrodynamics; HIL and field data for qualification. Never promote a coarse model result into a physical safety claim.
7. **Local value, not only global cloud capacity.** Measure local skills and jobs, supplier participation, port reliability, reduced vessel emissions, affordable connectivity and access to useful compute. A clean facility that extracts value without improving the port-city is not a flourishing infrastructure project.

## 8. Immediate execution plan

### Slice 0 — this branch

- Add a strict, versioned screening-case schema.
- Add an Ngqura case that labels procurement, screening and unknown facts separately.
- Add this architecture / benchmark contract.
- Validate the JSON fixture against the schema and review source links.
- No code path can issue control commands.

### Slice 1 — data and evidence

- Obtain TNPA / port-operator load profiles, feeder diagrams, interval meter data and actual project award/commissioning documents.
- Obtain the PtX Hub study tables and grid-connection assumptions; request a utility/municipal connection study for the exact proposed point of connection.
- Acquire site metocean and bathymetry data, seabed habitat mapping, protected-area overlays, fisheries/vessel use, cable-route constraints and baseline water-quality observations.
- Establish an explicit data-sharing, privacy, retention and operator-consent agreement.

### Slice 2 — reproducible energy benchmark

- Build an 8,760-hour Ngqura reference case with a separate synthetic fixture until operational data are licensed.
- Add storage degradation, grid limits, shore power, port demand and flexible compute as separate variables.
- Compare grid-only, rule-based and matched optimization/MPC controllers.
- Verify energy balance, grid constraints, reserve and service constraints independently; publish raw inputs, solver identities and receipts.

### Slice 3 — physical and operational qualification

- Connect Symtropy replay to the Sol-Atlas scenario identity and source digests.
- Connect Symthaea forecast residuals and uncertainty to the same identity; predictions never replace measurements.
- Anchor Mycelix signed evidence and revocation lineage; no trust score authorizes a plant or vessel action.
- Run read-only shadow operation, then hardware-in-the-loop and operator-led trials under a separate safety case.

### Slice 4 — compare compute locations before any ocean hardware

Produce a common TCO/LCA and service-quality comparison across inland, port-edge, floating and subsea placements. Include the full energy chain, data transport, batteries, vessel movements, cable repair, pressure-vessel manufacture, maintenance visits, retrieval and recycling. Proceed offshore only if it wins on a stated, independently checked objective without breaching ecological or service constraints.

## 9. Non-claims

This specification does **not** claim:
- that the Ngqura renewable plant is operating or that its tender was awarded;
- that 17.1 MW is a firm or currently available connection capacity;
- that Ngqura has an identified subsea data-centre site or suitable cable route;
- that any marine renewable technology is economically viable at the port without local resource measurements;
- that subsea computing is cheaper, lower-carbon, more reliable or ecologically benign than coastal land computing;
- that existing Symthaea, Mycelix, Sol-Atlas or Symtropy components already implement this integrated system;
- that any design, permit, grid offer, construction authorization or safety approval exists.

**Promotion rule:** screening → modeled → independently verified → hardware-in-the-loop → operator-observed → independently qualified. A missing or inconclusive gate remains missing or inconclusive.
