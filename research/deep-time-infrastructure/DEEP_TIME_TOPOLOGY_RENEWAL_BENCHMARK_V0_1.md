# Deep-Time Topology Renewal Benchmark
## Protocol v0.1

**Status:** ten synthetic transition cases defined; five hand-computable references are included; no physical or real-site validation is claimed.  
**Parent:** [DEEP-TIME-INFRASTRUCTURE-001 #886](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/886)  
**Work item:** [DEEP-TIME-TOPOLOGY-RENEWAL-001 #894](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/894)  
**Composes:** service-continuity corpus #890; Symtropy #3894/#5452/#5474/#1601/#3279/#3347/#451; Symthaea #7636; Sol-Atlas #377/#115; Mycelix #5149.

## 1. Research thesis: infrastructure as a renewing topology

Operational resilience studies how a declared network responds to disruptions and recovers. Deep-time infrastructure adds a different problem: the network itself is expected to change.

Ports move, channels silt, coastlines migrate, watersheds shift, bridges and tunnels reach end of life, settlements redistribute, and transport modes change. Essential services may remain necessary even when original assets or alignments are no longer viable.

The research object is therefore a **time-indexed service topology and its transition policy**. The target is not an immortal port or canal. It is a region that can replace, relocate, split, reconnect, retire and restore infrastructure while maintaining essential services and ecological functions.

## 2. Paradigm shifts

### 2.1 From asset survival to service transfer

Measure whether freight, water, healthcare access, power, mobility and ecological processes remain available—not whether the same asset survives.

A service transition is valid only if replacement capacity becomes available before old capacity is retired, or if a declared interruption is explicitly accepted. A new terminal's existence does not prove that transfer links, power, staff, water, access or capacity are ready.

### 2.2 From static graphs to time-expanded topology

Every asset/service relationship has a validity interval, commissioning/retirement state, capacity trajectory, prerequisite and common-cause dependencies, geographic context, source and authority evidence.

Represent time explicitly. Reject impossible state overlaps, missing prerequisites, unknown units and use of future observations at earlier decision points.

### 2.3 From redundancy by count to independence by cause

Two routes are not independent merely because they have different geometries or asset IDs. They may share a substation, control plane, telecom backhaul, floodplain, workforce, supplier, water source or approval authority.

Represent cross-sector dependencies as typed directed hyperedges and common-cause groups. Preserve physical, digital, geographic, logical, economic, ecological and institutional dependency classes. Unknown dependencies are unknown—not independent.

A 7 November 2026 comparative review by Shamsi, Helmrich and Deegan catalogues 24 energy/water/transport tools in its defined review scope, seven of which explicitly support system-of-systems interdependency assessment. It also notes uneven treatment of broader dependency classes and a complexity/usability trade-off. This is a scoped literature sample, not a universal industry census or performance leaderboard. [DOI: 10.1007/s10669-026-10162-2](https://doi.org/10.1007/s10669-026-10162-2).

### 2.4 From construction cost to whole-life material and carbon metabolism

Every alternative must expose its declared lifecycle boundary: demand avoidance/operational options; material extraction and manufacture; transport/construction; operation and maintenance; refurbishment/replacement; decommissioning/reuse/recycling/disposal.

The 5 March 2026 UK NISTA Whole Life Carbon Management Handbook is a current public-sector comparator aligned with PAS 2080:2023. We adopt whole-life accounting as a comparator; we do not treat jurisdiction-specific governance as universal law or invent carbon factors. [NISTA guidance](https://www.gov.uk/government/publications/whole-life-carbon-management).

### 2.5 From one optimum to option-preserving portfolios

A portfolio should preserve the ability to switch, split or relocate when evidence changes. Option value must be evaluated against monitoring cost, action lead time, switching cost, irreversibility and the risk of waiting too long.

No probabilities should be invented for deep-time scenarios. Use robust feasibility, scenario-wise service floors, regret and satisficing unless a defensible probability model exists.

## 3. Time-expanded transition contract

A candidate transition action should minimally carry:
- action ID and type (build, upgrade, operate, monitor, switch, relocate, split, retire, decommission, restore);
- affected asset/service IDs and from/to states;
- start time, duration, commissioning time and validity interval;
- prerequisite actions and dependencies;
- capacity gained/lost over time and service-demand class;
- water, energy, material and carbon ledgers;
- lifecycle cost boundary, uncertainty and source for every coefficient;
- rights, consent, ecological and competent-authority gates;
- trigger source, sampling cadence, observation delay and action lead time;
- failure modes, common-cause groups and fallback actions.

A transition policy is a directed decision graph. It may branch only on observations available at the decision time. It may not peek at the future scenario identity.

## 4. Ten initial benchmark cases

The machine-readable register contains ten cases:
- TNR-001/002 — overlap and service gaps during port/corridor relocation;
- TNR-003 — drought waterway substitution with environmental-flow protection;
- TNR-004 — sediment-driven retirement and habitat restoration;
- TNR-005 — polycentric settlement distribution and group-level service floors;
- TNR-006 — common power/control dependencies behind nominally distinct routes;
- TNR-007 — repair-versus-replace whole-life carbon known answer;
- TNR-008 — trigger observation and action lead-time failure;
- TNR-009 — minimax-regret policy under uncalibrated scenario probabilities;
- TNR-010 — asset decommissioning and material recovery.

Five are hand-computable synthetic controls; the remainder are explicit specifications pending complete numeric fixtures and qualified external evidence. The companion case register is a specification, not a claim that all ten have executable oracles.

### 4.1 Hand-computable known answers

**TNR-001 — zero-gap service transfer**

| Period | Old capacity | New capacity | Demand | Delivered |
|---|---:|---:|---:|---:|
| 0 | 100 | 0 | 80 | 80 |
| 1 | 60 | 40 | 80 | 80 |
| 2 | 0 | 80 | 80 | 80 |

Expected total unmet service = 0 units.

**TNR-002 — premature retirement**

Old/new/demand by period: (100,0,80), (0,0,80), (0,80,80). Expected delivery: 80, 0, 80. Unmet service = 80 units in one declared interval. The policy fails the no-gap control.

**TNR-003 — drought substitution**

Demand = 100 tonnes/day. Drought waterway = 30, rail = 40, road = 10. Expected delivery = 80; unmet = 20 tonnes/day. Required/proposed ecological flow = 25/25 m3/day, so the environmental-flow gate passes. The result must report unmet demand; capacity cannot be invented.

**TNR-007 — repair versus replacement**

Synthetic ten-year fixture; all emissions factors are supplied by the fixture and are not real-world factors.

- Repair: 120 tCO2e embodied + 12 tCO2e/year × 10 + 10 end-of-life = **250 tCO2e**.
- Replace: 220 tCO2e embodied + 8 tCO2e/year × 10 + 20 end-of-life = **320 tCO2e**.
- Repair is 70 tCO2e lower for this declared boundary and horizon, with half the material input (30 vs 60 tonnes).

**TNR-008 — late trigger**

Signal crosses at t=4; observation delay=2; action lead time=3; service threshold t=8. Trigger observed at t=6; action completes at t=9. It is one time unit late. It cannot be credited as preventive adaptation.

**TNR-009 — minimax regret**

No probabilities are provided.

| Strategy | Coastal stable | Coastal loss | Drought | Maximum regret |
|---|---:|---:|---:|---:|
| Fixed coastal | 100 | 10 | 100 | 70 |
| Staged dual | 80 | 70 | 70 | **30** |
| Inland rail | 65 | 80 | 80 | 35 |

Scenario-wise best utilities are 100, 80 and 100. The exact minimax-regret choice is staged dual, with worst-case regret 30. This is a known answer over this declared finite table only.

## 5. Comparison protocol

Compare methods using the same time horizon, initial service topology, demand, scenario support, available observations, hard constraints, lifecycle boundary, compute budget and admissible actions.

Baselines: fixed asset/alignment; no-build and operational improvement; static master plan plus contingency list; staged trigger-based policy; relocatable/distributed portfolio; greedy lowest initial cost; minimax/minimax regret; exhaustive exact policy oracle on small finite cases.

Report service delivered, unmet service-hours, worst-group service, transition gap, recovery/reconfiguration time, minimum cut/common-cause failure domains, water/ecological constraints, material inputs, whole-life cost/carbon, stranded assets, trigger lead-time margin, regret and evidence completeness.

Do not publish a single resilience score that lets low cost or high throughput compensate for a breached legal, ecological or essential-service constraint.

## 6. Industry comparison and claim discipline

- Shamsi et al. (2026) provide a recent system-of-systems interdependency assessment framework and tool review. Compare dependency-class coverage and usability; do not treat their 24-tool inventory as a universal market census.
- Symtropy #3894 owns service-cascade simulation and recovery. Reuse it for operational disruption trajectories; this work adds the topology-transition horizon.
- Symtropy #5452 owns matched evaluation against IN-CORE, NIST Community Resilience Planning Guide and relevant standards. Reuse its external comparison protocol.
- Symtropy #5474 owns model/dataset lineage envelopes. Reuse it for imported model results.
- Symtropy #1601 owns multimodal whole-life social-cost optimization. Reuse transport and lifecycle objectives; test transition continuity and deep-time change here.
- NISTA 2026 WLC guidance aligned to PAS 2080:2023 is a whole-life carbon comparator. Project-specific factors, scopes and data quality must remain explicit.
- PIANC, USACE and specialist hydraulic/coastal models remain physical engineering references; this benchmark does not replace validated solvers.

No superiority claim is allowed without matched task, source versions, scenario support, authority boundaries, resources and independent adjudication.

## 7. Proposed acceptance

- Ten unique transition cases with explicit specification/fixture maturity.
- At least five exact synthetic known answers reproduced by a separately implemented oracle.
- Zero avoidable service gap on declared zero-gap controls.
- Zero false-safe result on water, rights, ecology, authority or source-quality negatives.
- Zero future-information leakage.
- Correct detection of late triggers and hidden common causes.
- Exact minimax-regret agreement on the enumerated known-answer table.
- Whole-life carbon and material boundary disclosed; no invented factors.
- Deterministic replay for pinned transition fixtures.
- Real-site transition claims remain blocked until local physical, ecological, community, legal and financial evidence is qualified.

These are targets, not measured results.

## 8. Claim ceiling

A synthetic pass establishes only declared transition semantics on the frozen finite fixtures. It does not establish geological prediction, actual port relocation feasibility, safe navigation, water entitlement, carbon-factor validity, ecological recovery, community consent, financing or project authorization.
