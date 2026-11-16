# Deep-Time Infrastructure Dependency Taxonomy and Cross-Repository Crosswalk
## v0.1

**Status:** semantic crosswalk and benchmark guidance; it does not define a replacement dependency graph or authority schema.  
**Parent:** DEEP-TIME-INFRASTRUCTURE-001 #886.  
**Related:** DEEP-TIME-CORPUS-001 #890; DEEP-TIME-TOPOLOGY-RENEWAL-001 #894.

## 1. Why a single graph edge is not enough

Infrastructure resilience depends on several relationships that look similar in a drawing but have different semantics. A water pump may be physically connected to a power feeder; two routes may share a floodplain; a service may require both power and control telemetry; a construction action may require legal permission; a fish population may require continuous habitat and sediment processes.

Do not flatten all of these into an untyped edge and then propagate availability using one generic rule.

A comparative review published 7 November 2026 identified 24 energy/water/transport interdependency tools within its defined review scope, seven of which explicitly supported system-of-systems interdependency assessment. It found strong emphasis on physical dependencies and weaker integration of broader dependency classes. This is a useful research motivation, not a universal tool-market census. Shamsi, Helmrich & Deegan (2026), *Toward system-of-systems resilience*, https://doi.org/10.1007/s10669-026-10162-2 .

## 2. Dependency taxonomy

| Class | Example | Correct semantic treatment | Benchmark failure to detect |
|---|---|---|---|
| Physical / material | pump requires feeder; lock requires water and power | flow, capacity, threshold, AND/OR; physical stocks remain with domain ledger | treating a pipe/feeder as unlimited or assuming all dependencies are binary |
| Digital / control | both terminals use one SCADA, telecom backhaul or control-plane build | shared-cause hyperedge; outage propagation only through declared service/control semantics | counting nominal routes as independent when one controller disables both |
| Geographic / hazard | road and rail cross the same floodplain or subsidence basin | shared spatial hazard/failure domain with time-varying footprint and uncertain exposure | multiplying independent failure probabilities for co-located assets |
| Logical / service | hospital requires power AND water; alternative access route is OR | typed dependency groups (ALL/ANY), degraded service and thresholds | generic edge semantics that turn redundancy into mandatory dependency or vice versa |
| Economic / supply chain | two projects depend on one supplier, fuel source, spare part or export route | capacity, lead-time, substitution, concentration and market-stress model | treating different vendors as independent when upstream production is shared |
| Institutional / authority | corridor requires land access, water entitlement, cross-border agreement or operating authorization | hard admissibility gate and time-bounded approval evidence; not a probability-weighted service edge | allowing a high optimizer score to compensate for missing rights or consent |
| Ecological / process | waterway requires environmental flow, fish passage, sediment continuity and connected wetlands | state variables, thresholds, seasonal windows, ecological lag and independent outcome ledger | representing nature as a static polygon or a compensable penalty after design selection |
| Evidentiary / model | multiple model outputs share same input dataset, calibration, test harness or model family | provenance/dependency lineage and model-independence qualification | counting correlated model outputs as independent confirmations |

## 3. Preserve semantic distinctions

### 3.1 Dependencies are not necessarily common causes

A directed service dependency says that service A needs B. A common-cause relationship says that a single failure domain can affect A and C together. A shared data lineage says that two observations or model results are not independent evidence. These must remain distinct relationship types.

### 3.2 AND / OR semantics are not interchangeable

- ALL-of: every required parent must be available above its declared threshold.
- ANY-of: one or more admissible alternatives may satisfy a service requirement.
- K-of-N: a declared number of alternatives is required.
- Threshold: a continuous parent-service fraction must exceed a specified level.
- Hard authority gate: a permission/rights condition is admissibility, not service capacity.

Do not reduce these to a common multiplication or minimum rule without declaring and validating the exact model.

### 3.3 Unknown is not independent, zero, or absent

When the dependency inventory is incomplete, stale, disputed or outside the relevant time window, return UNKNOWN/INCOMPLETE for that question. Do not silently infer no shared cause. Distinguish:
- shared dependency established;
- no declared common domain within a complete and current inventory;
- unknown or contested dependency;
- probabilistically modelled dependence;
- unsupported dependency semantics.

Even NO_DECLARED_COMMON_DOMAIN is not proof of physical or statistical independence beyond the declared inventory.

### 3.4 Time is part of the dependency

A source may have depended on one power or control domain at event time and another today. Dependency closure must be evaluated against the observation's relevant valid-time interval and an exact inventory snapshot/frontier. A current dependency graph must not rewrite historical causality.

## 4. Compose existing Luminous owners

| Owner | Existing work | What this programme consumes |
|---|---|---|
| Symtropy | #3894 service cascades | ALL/ANY service propagation, degraded service, time trajectories, recovery and beneficiary loss |
| Symtropy | #5452 matched resilience evaluation | matched comparison with IN-CORE/NIST and existing resilience practice |
| Symtropy | #5474 external model evidence envelope | exact model, dataset, artifact and analysis lineage |
| Symtropy | #1601 multimodal whole-life social cost | network cost, passenger/freight service, lifecycle and externality ledgers |
| Mycelix | #5923 time-qualified dependency hypergraph | typed transitive dependency closure, valid-time semantics, incomplete inventory and exact snapshot binding |
| Mycelix | #5762 multi-valued/transitive operational dependencies | multiple identities per role/domain, partial overlap and bounded exact cuts |
| Sol-Atlas | #406 common-cause-aware evidence independence | common-cause conflict/hypergraph and bounded independent-set qualification |
| Sol-Atlas | #115 maritime atlas/provenance | exact maritime data identity, CRS/vertical datum, source time and read-only evidence projection |
| Sol-Atlas | #377 geographic opportunity discovery | coast–hinterland candidate geography; not a new location suitability model |
| Symthaea | #7636 adaptive pathways | policy generation, triggers, information value and transition choice |
| Mycelix | #5149 construction provenance/approval | rights, authority, consent and construction approval evidence |

No second generic graph, provenance ledger, GIS, service-cascade engine, transport optimizer or authorization system should be created in the deep-time workstream.

## 5. Cross-domain qualification suite

Add at least one positive and one adversarial control for each dependency class:

1. Physical: pump loses power; test continuous flow and storage semantics against declared resource ledger.
2. Digital/control: two terminal routes share one controller; shared outage must disable both only where the control dependency is operationally required.
3. Geographic: road and rail share one flood footprint; correlated outage must not be treated as independent.
4. Logical/service: hospital service requires ALL-of power/water and ANY-of two access routes; preserve both operators exactly.
5. Economic/supply chain: distinct contractors share one upstream transformer manufacturer or fuel source; detect concentration without assuming deterministic joint failure.
6. Institutional: water-transfer corridor has technical capacity but missing entitlement; reject eligibility, do not down-rank.
7. Ecological: navigation remains feasible only if a protected flow threshold and fish passage are preserved; report ecological state separately.
8. Evidentiary/model: two hydraulic model results use the same bathymetry and calibration dataset; count model diversity and input independence separately.

All tests must preserve the source, uncertainty, valid-time and claim-ceiling metadata. The same test may have multiple effects, but one semantic class must not be inferred from another without an explicit, qualified relation.

## 6. Benchmark measures

Report separately:
- dependency class coverage and unsupported class count;
- known, shared, unknown and complete dependency inventory counts;
- false-independent and false-independent-corroboration rate;
- ALL/ANY/K-of-N operator correctness;
- time-valid transitive closure agreement;
- cascade path traceability and update-order determinism;
- authority/rights gate false-accept rate;
- ecological threshold false-safe rate;
- matched service-loss and restoration outcomes;
- compute, memory, model complexity and analyst usability.

Do not average false-safe outcomes into a composite score. A false-independent claim or invalid authority acceptance is a hard failure.

## 7. Claim ceiling

This crosswalk identifies semantic requirements and owner boundaries. It does not prove that any dependency inventory is complete, that a real network is safe, that two suppliers are statistically independent, that an ecological threshold is sufficient for a particular site, or that a project has legal approval. External models remain limited to their exact context of use and captured input lineage.
