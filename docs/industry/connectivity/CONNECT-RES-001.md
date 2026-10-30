# CONNECT-RES-001 — Power-aware telecommunications resilience profile v0.1

**Status:** research/design contract  
**Parent:** [Issue #323](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/323)  
**Date:** 2026-10-30  
**Claim ceiling:** design specification only; no deployed telecom capability or field result is claimed.

## 1. Purpose

Add telecommunications infrastructure as a new industry vertical by answering a narrow, operationally useful question:

> Under a declared set of power, site, and backhaul failures, which critical connectivity services remain supportable, where are the bottlenecks, and which feasible interventions improve worst-case service without violating protected service floors?

The first vertical is a **resilience-planning twin**, not a general network-management system. It should make a planner's assumptions explicit and produce a reproducible comparison between the current topology and candidate interventions.

## 2. Why this vertical, and why now

The repository work inventory already has active, substantial work on water/power resilience, agricultural systems, manufacturing, mining, materials, and industrial maintenance. The current Sol-Atlas issue/PR search did not identify a telecommunications-specific vertical. A bounded connectivity-resilience profile therefore expands the portfolio without creating a second water or farm workstream.

South African context is useful but must be interpreted carefully:

- ICASA's *State of the ICT Sector Report* (31 March 2026) publishes 2025 urban population coverage by province and technology. Its 5G table varies materially by province. These are regulator-reported aggregate indicators, not geolocated site measurements or evidence of indoor reception.
- The DBSA *Digital Infrastructure Investment Study 2025* reports that 4.3% of deep-rural areas had any 5G access in its analysis. It also describes the absence of a current, publicly available government database for broadband planning, and says its map used ICASA and public infrastructure data. Treat the percentage as a dated study output with its own geographic and methodological limits, not as a live national coverage oracle.
- ICASA's Quality of Service programme measures voice-network accessibility and retainability in selected towns, townships, farms, rural areas, major roads and economic nodes. These measurements can ground narrowly scoped comparisons where the measurement geography, date, sample and KPI definition are available. They do not establish arbitrary data throughput, latency or end-to-end availability.

Sources:
- ICASA: https://www.icasa.org.za/uploads/files/The-State-of-the-ICT-Sector-Report-of-South-Africa-31-March-2026.pdf
- ICASA QoS reports: https://www.icasa.org.za/pages/quality-of-service-reports
- DBSA: https://www.dbsa.org/sites/default/files/media/documents/2026-03/South%20Africa%27s%20Digital%20Infrastructure%20Investment%20Study%202025.pdf

## 3. Scope and explicit exclusions

### In scope

A declared, time-indexed graph of connectivity service dependencies:

- access sites / points of presence;
- backhaul links and alternate paths;
- site power inputs and backup-energy resources;
- service-demand cells, aggregated to a declared geographic and service-class resolution;
- critical service classes and non-compensatory minimum service floors;
- explicit failure domains and common-cause scenarios;
- restoration and repair assumptions;
- provenance, unit, model-version and scenario identity.

Compute only the results justified by the chosen model lane: path reachability, bounded path capacity, site energy autonomy, service-floor compliance, outage exposure and recovery bounds.

### Not in scope for v0.1

- radio propagation prediction, antenna optimization, spectrum allocation or RF planning;
- detailed PHY/MAC/packet-level 4G/5G/6G simulation;
- subscriber-level traffic, personal data, or individual location histories;
- operator OSS/BSS, billing, customer-care, inventory or enterprise-resource planning;
- automatic RAN configuration, spectrum changes, network actuation or emergency dispatch;
- probability-of-failure estimates inferred from counts of synthetic scenarios;
- claims of operator SLA compliance, national coverage, regulatory compliance or production readiness.

These exclusions are deliberate. Detailed RF and RAN simulation already have specialized reference tools. Sol-Atlas should compose with them rather than pretend that a graph model is a radio model.

## 4. Semantic boundaries

Do not conflate these quantities:

1. **Geographic coverage prediction** — a model's estimate of RF service at a location under declared propagation, antenna, frequency and load assumptions.
2. **Network reachability** — a path exists through the declared access/backhaul topology.
3. **Capacity adequacy** — the available path/site capacity meets a declared demand envelope.
4. **Observed end-user quality of service** — a measurement at a declared place, time, device, technology, KPI and sampling method.
5. **Delivered service** — the service was actually delivered to a declared user or aggregate service population.

A positive outcome in one category must not be promoted into another. In particular, a coverage polygon is not proof of usable service; a connected graph is not proof of adequate capacity; a modelled service floor is not observed QoS.

## 5. Minimal domain contract

The first executable slice should define a versioned input bundle. Exact JSON field names can be finalized during implementation, but the semantics below are mandatory.

### 5.1 Site

A stable site ID, location and coordinate reference; site type; supported service classes; declared site capacity; power-supply dependencies; backup equipment; upstream backhaul attachment; evidence references; and effective time interval.

Site identity is not a subscriber identity. Do not embed personal or device-level identifiers.

### 5.2 Backhaul link

A stable link ID; endpoint IDs; capacity interval and unit; availability state or explicit scenario status; medium/type; shared-risk/failure-domain IDs; restoration assumptions; evidence references; and effective interval.

A physical fiber route, an IP path and a logical service path may be different objects. Do not count logical redundancy as physical diversity unless the underlying dependencies are known.

### 5.3 Power and backup resource

A power feed has an explicit source and measurement point (for example, feeder, site input or equipment terminal). Backup resources declare usable energy, discharge/power limits, efficiency or derating assumptions, recharge source, and any fuel or replenishment dependencies.

For a simple battery-only interval, the accounting identity is:

    energy_next = energy_start + energy_in - energy_out - losses

The implementation must reject non-finite values, incompatible units, negative physical quantities and impossible depletion. It must not silently clamp unexplained energy deficits to zero. Nameplate capacity, measured state of charge and estimated usable energy are separate fields.

A site with positive battery energy is not necessarily available: the required power, cooling, access equipment, synchronization and backhaul must also be available under the declared model.

### 5.4 Service cell

An aggregate geography/time cell with a declared demand envelope, service class, weight and denominator definition. Weights can represent population exposure, premises, public-service demand or another declared aggregate—but those meanings must not be mixed.

Unknown denominators remain unknown. Do not silently treat a site coverage footprint, census population, active subscriptions and actual users as interchangeable.

### 5.5 Failure and recovery scenario

A stable scenario ID, time grid, explicit failed/degraded assets, common-cause domains, repair/replenishment assumptions, evidence class and scenario source. Scenario sets are enumerated experiments, not probability distributions unless a separately calibrated statistical model is introduced.

## 6. Responsibilities across the Luminous stack

| Project | Responsibility in this vertical | Must not claim |
|---|---|---|
| Sol-Atlas | Geospatial inventory, topology, dependency graph, service-cell exposure, scenario evaluation, intervention comparison and reporting | RF coverage or delivered QoS without the appropriate model or observation |
| Symtropy | Independently validated physical submodels where required, beginning with bounded backup-energy and recovery dynamics | A full telco simulator or validated battery model merely because a generic physics engine runs |
| Symthaea | Propose diverse interventions and discriminating experiments; explain trade-offs and identify information gaps | That generated designs are feasible, optimal or safe without independent checks |
| Mycelix | Identity, scoped permissions, provenance, maintenance/work coordination and multi-organization receipts | Source truth or operational completion based only on a ledger receipt |
| Luminous Platform | Deployment, access boundaries, audit and operational integration when the vertical is ready | Production readiness before security, recovery, service and operator acceptance evidence |

The first synthetic benchmark should be runnable without Mycelix or live provider connections. Integration is a later, separately gated milestone.

## 7. Benchmark programme

Use four distinct lanes. Results from one lane must not be placed on another lane's leaderboard.

### Lane A — topology and service continuity

Build tiny synthetic networks with hand-computable paths, bottlenecks and alternate routes. Compare:

- current-topology / no-intervention baseline;
- simple N-1 path-reserve rule;
- deterministic priority-greedy protection rule;
- bounded exhaustive oracle on small instances.

Metrics: reachability, max-flow or declared path-capacity bounds, per-class service-floor compliance, unserved demand, min-cut exposure and worst-case service-cell deficit.

### Lane B — backup-energy and site autonomy

Use hand-calculated energy-balance fixtures first. Then add degraded usable capacity, variable load, recharge windows and explicit replenishment delays.

Metrics: lower/upper autonomy hours, energy deficit, recovery interval, unused reserve and resource cost. A simulator must report sampled recovery bounds honestly; do not invent an exact continuous recovery time between samples.

### Lane C — RF and radio-network references (later)

Use **ITU-R P.1812-8 (09/2025)** for path-specific terrestrial propagation in its declared 30 MHz–6 GHz scope. It is a reference for a radio-propagation lane, not for backhaul topology or service restoration.

Use **5G-LENA / ns-3 NR** for matched 5G New Radio PHY/MAC/network experiments. Declare channel, carrier, numerology, antenna, scheduler, traffic, mobility, topology and random seeds. Do not compare a graph reachability result to throughput/latency results as if they were the same task.

A later hardware-in-the-loop lane may use **Colosseum**, which provides RF/channel emulation and protocol-stack experimentation. Do not imply that a simulated Sol-Atlas result has passed Colosseum.

References:
- ITU-R P.1812-8: https://www.itu.int/rec/R-REC-P.1812-8-202509-I/en
- 5G-LENA: https://apps.nsnam.org/app/nr/
- Colosseum: https://colosseumwireless.readthedocs.io/en/latest/

### Lane D — field and regulator observations (later)

Import ICASA reports only after extracting the exact KPI, period, geography, technology, sample and unit. Compare like with like. Retain the report as an aggregate regulator observation, not as per-tower ground truth.

Operator site inventories, route maps and network telemetry require explicit authorization and data-sharing terms. OpenCelliD or OpenStreetMap may be evaluated as discovery sources only after checking current licensing, attribution, access and completeness. No dataset is automatically approved for training or operational use because it is downloadable.

## 8. Required metrics

Report each metric with its denominator, unit, scenario and evidence class.

- service-cell and service-class floor-compliance lower/upper bounds;
- demand-volume served and unmet, by scenario and cell;
- maximum normalized deficit and between-cell disparity;
- outage duration and sampled time-to-recovery bounds;
- site backup autonomy under each declared load/derating case;
- backhaul bottleneck capacity, path redundancy and min-cut exposure;
- intervention cost/resource/energy bounds;
- Pareto frontier with hard service floors applied before ranking;
- uncertainty interval coverage, abstention/indeterminate rate and false-confident-availability count;
- exact runtime, input digest, evaluator identity, environment and output digest.

A single weighted score must not trade away emergency communications or another declared essential-service floor. If no alternative meets the floors, report no feasible candidate.

## 9. Acceptance gates for v0.1

1. **Known-answer correctness:** 100% agreement with hand-calculated fixtures for path reachability, bounded path capacity, simple backup-energy accounting and service-floor aggregation.
2. **Fail-closed unknowns:** zero missing, stale, non-finite, inadmissible or unit-incompatible inputs may create a positive robust-availability result.
3. **Common-cause semantics:** a declared shared failure domain affects every assigned asset; a common cause is not inferred from proximity or naming.
4. **Denominator honesty:** missing demand, population or coverage denominators yield indeterminate outcomes, never invented zero or full coverage.
5. **Fair comparison:** all alternatives use the same declared topology, scenario set, time grid, information budget, resource budget and service floors.
6. **Determinism:** canonical input permutations produce byte-identical output; duplicate IDs, invalid references, non-finite arithmetic and digest mismatch fail closed.
7. **Independent verification:** a second implementation/checker reconstructs path, energy and service-floor results from the input bundle; it does not call the candidate generator or selector.
8. **Evidence ceilings:** synthetic, regulatory aggregate, modelled RF, simulated RAN, measured QoS, HIL and field outcomes remain distinct labels.
9. **Security boundary:** no production credentials, subscriber personal data, or actuation interfaces in the synthetic benchmark.

## 10. What would count as beating the baseline

Do not set a universal percentage-improvement target before the baseline, resource constraints and target decision are frozen.

A candidate may be reported as robustly better only when:
- it satisfies every protected service floor in the declared scenario set;
- it improves at least one predeclared objective relative to the named baseline;
- it does not worsen another protected objective beyond a predeclared tolerance;
- comparisons use equal information, scenario, compute and resource budgets;
- the independent verifier reproduces the result;
- held-out topology and load/failure cases do not reveal a hidden catastrophic failure.

For the RF and packet-level lanes, superiority requires matched ITU/5G-LENA/Colosseum reference configurations and independently inspected results. For field claims, it requires authorized real observations and prospective comparison. Synthetic wins are engineering progress, not a claim of industry-leading field performance.

## 11. Phased execution

- **P0 — contract:** freeze typed entities, units, evidence classes, service floors and claim ceilings.
- **P1 — executable kernel:** implement a small deterministic graph/power/service evaluator plus hand-calculated fixtures and an independent checker.
- **P2 — robust planning:** add common-cause/N-k panels, bounded intervention enumeration, Pareto reporting and held-out topologies.
- **P3 — reference adapters:** add separately versioned ITU-R propagation, 5G-LENA and/or Colosseum comparison adapters only where needed.
- **P4 — South African pilot readiness:** secure permissioned site/backhaul/power inventories and compatible QoS observations; perform data-quality and privacy review before importing.
- **P5 — shadow pilot:** compare recommendations with operator/planner decisions; no actuation. Promote only with measured reliability, recovery, security and operational acceptance evidence.

## 12. Risk register

| Risk | Control |
|---|---|
| Public coverage aggregates mistaken for site-level truth | Preserve granularity, source and denominator; prohibit upscaling |
| Logical backhaul diversity mistaken for physical route diversity | Require shared-risk/physical-dependency evidence |
| Battery nameplate treated as usable autonomy | Require state, derating, load and interval accounting |
| Synthetic outage counts described as probabilities | Explicit scenarios; probability lane requires calibration |
| Optimizer trades away critical connectivity | Hard service floors before objective ranking |
| RF/packet model mismatch | Separate benchmark lanes and task-specific metrics |
| Unauthorized or sensitive operator data | Permission gate, minimization and access control |
| Model drift or stale inventories | Effective-time intervals, freshness checks and explicit indeterminate states |

## 13. Assurance references

- NISTIR 8620 (2026), *Digital Twins Workshops Summary Report*: highlights interoperability, VVUQ, cybersecurity and workforce readiness as manufacturing digital-twin research priorities. https://www.nist.gov/publications/digital-twins-workshops-summary-report
- NISTIR 8625 (2026), *Open Radio Access Network (O-RAN) Near-Real Time RAN Intelligent Controller (RIC) Security*: useful security reference for later Open RAN/RIC work; not a certification. https://www.nist.gov/publications/open-radio-access-network-o-ran-near-real-time-ran-intelligent-controller-ric-security

## 14. Decision

Proceed with the topology + backup-energy + protected-service-floor v0.1 as the first implementation target. Defer detailed RF, RAN, billing and operator-automation features until the relevant reference engines, permitted data and assurance needs are explicit.

## 15. South African baseline update (ICASA March 2026)

The ICASA *State of the ICT Sector Report of South Africa* (31 March 2026) records 2025 national population coverage of **99.50% for 4G/LTE** and **58.02% for 5G**. Its rural population coverage table reports 5G coverage by province as: Eastern Cape 7%, Free State 25%, Gauteng 74%, KwaZulu-Natal 15%, Limpopo 29%, Mpumalanga 63%, North West 16%, Northern Cape 13%, and Western Cape 34%.

The report defines rural coverage as inhabitants within range of a mobile-cellular signal. This is not proof of indoor service, capacity under load, backhaul availability, end-user throughput, voice retainability or service during a power outage. Keep those separate in the data model.

The same report records 2025 licensee expenditure of approximately **R387.7 million on batteries** and **R426.8 million on generators**, compared with approximately R173.8 million and R211.5 million respectively in 2024. This establishes that backup-power expenditure is a material industry concern. It does not establish the number of functioning backup systems, their autonomy, fuel availability, service uptime, or causal effect on outages.

The DBSA *Digital Infrastructure Investment Study 2025* separately reports 5G access in only 4.3% of deep-rural areas in its analysis. This is a different study, population/geographic definition and methodology from ICASA's province-level rural population coverage table. Preserve the source and denominator and do not calculate a direct delta between the two figures.

These indicators strengthen the reason to test an operator-neutral resilience-planning use case: identify the sites and communities where power, backhaul and service demand jointly create the greatest continuity risk, and evaluate transparent interventions before committing capital. They do not justify claiming that Sol-Atlas can independently map actual 5G coverage or predict real site uptime.

Sources:
- ICASA, *The State of the ICT Sector Report of South Africa*, March 2026, sections 4.4 and 4.5: https://www.icasa.org.za/uploads/files/The-State-of-the-ICT-Sector-Report-of-South-Africa-31-March-2026.pdf
- DBSA, *South Africa's Digital Infrastructure Investment Study 2025*, section 5.4: https://www.dbsa.org/sites/default/files/media/documents/2026-03/South%20Africa%27s%20Digital%20Infrastructure%20Investment%20Study%202025.pdf
