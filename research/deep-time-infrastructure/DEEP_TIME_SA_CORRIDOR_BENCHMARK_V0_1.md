# Durban–Gauteng Synthetic Corridor Benchmark
## Protocol v0.1

**Status:** 20 synthetic cases; eight exact references; no observed corridor performance or project feasibility claim.  
**Parent:** [DEEP-TIME-SA-CORRIDOR-001 #895](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/895).  
**Composes:** Sol-Atlas #377/#115/#809; Symtropy #1601/#3199/#3894/#5452/#5474; Symthaea #7790/#7636; Mycelix #5149/#5923.

## 1. Purpose

Build a corridor benchmark before using real route, port, rail or demand data. It qualifies modelling semantics; it must not manufacture a South African capacity estimate from public summaries.

The companion machine-readable register contains 20 stable IDs. Eight are exact synthetic references; twelve are specification-only and require full numeric or source-qualified evidence before execution. Every output is bounded to synthetic semantics.

## 2. Exact references

| Case | Inputs | Exact expected result |
|---|---|---|
| SA-COR-001 | Serial edge capacities 100, 75, 60; demand 90 | Flow 60; unmet 30; min-cut 60 |
| SA-COR-002 | Two edge-disjoint paths with capacities 20 and 25; demand 60 | Flow 45; unmet 15 |
| SA-COR-003 | Demand 100; port 90; rail 70; dry-port handling 80; drayage 60 | Flow 60; unmet 40 |
| SA-COR-004 | Old/new capacities (80,0), (60,20), (0,80); demand 80 | Delivery [80,80,80]; no transition gap |
| SA-COR-005 | Old/new capacities (80,0), (0,0), (0,80); demand 80 | Delivery [80,0,80]; one-period unmet service 80 |
| SA-COR-006 | Two nominal paths both require SUB-1; SUB-1 fails | Zero paths and zero deliverable capacity |
| SA-COR-007 | Rail capacity 60 fails; road capacity 40 survives; demand 80 | Delivery 40; unmet 40 |
| SA-COR-020 | Three strategies across normal, rail-disruption and port-disruption futures | Maximum regrets: road 15, rail 47, staged multimodal 7; staged multimodal selected |

All figures are invented test inputs. They do not correspond to observed Transnet capacity, TEUs, corridor demand, route reliability or capital costs.

## 3. Remaining cases

The twelve specification-only cases cover:
- planned versus commissioned capacity;
- announced future train operator start versus observed service;
- correlated flood exposure on road and rail;
- shared digital control planes;
- alternative gateway compatibility;
- canal/water-transfer hard stops;
- group-level service floors;
- whole-life social-cost/carbon completeness;
- OD demand data quality;
- port cargo-dues metrics versus design capacity;
- stale network statements;
- compound port/rail/power disruption and shared recovery resources.

Missing information must produce an incomplete/abstain disposition.

## 4. Fair baselines

Compare no-build/observed current operation (once source-qualified), road-dominant, rail with road drayage, inland terminal/dry-port, a fixed master plan with contingency list, staged service transfer, time-dependent shortest path, capacity-constrained min-cost flow, greedy allocation, minimax and minimax regret without invented probabilities, and exhaustive small-graph oracles.

Reuse Symtropy #1601's multimodal objective and #3894's service-cascade kernel. External comparisons should follow #5452/#5474 and require exact task, data, model revision, scope, scenario information, resource budget and independent adjudication.

## 5. Required metrics

- delivered tonnes/TEUs only after commodity, unit and source definitions;
- unmet demand and OD data completeness;
- path bottleneck, max flow, min-cut and common-cause groups;
- actual service-hours lost, transition gap and recovery latency;
- port dwell/turnaround only with valid observations and definitions;
- lifecycle financial, fiscal and social cost ledgers;
- whole-life carbon with documented factors and boundary;
- energy, safety and local air pollution;
- group-level accessibility and essential-service floors;
- water/ecological impacts and authority status;
- source completeness, age, rights and datum.

Do not pool incompatible tonnes, TEUs, trips, cargo dues, capacity, design targets and observed throughput. Do not let a composite score compensate for a hard rights, safety or ecological failure.

## 6. Promotion gates

Synthetic -> public-data reproduction -> matched scenario comparison -> local physical/engineering qualification. Each promotion requires pinned evidence and independent review. Planned capacities and future policy milestones must remain separate from observed operation.

## 7. Claim ceiling

Passing these cases demonstrates only exact synthetic graph, transition and decision semantics. It does not establish observed corridor capacity, actual freight demand, rail availability, alternative gateway feasibility, sea-level exposure, local water balance, preferred investment, port design, navigation safety, legal authority, community consent or construction feasibility.
