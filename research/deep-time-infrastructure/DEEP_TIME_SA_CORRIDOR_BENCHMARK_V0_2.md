# Durban–Gauteng Synthetic Corridor Benchmark
## Protocol v0.2 — TRIM capacity semantics extension

**Status:** 30 synthetic cases; 13 exact references; 17 specification-only controls. No observed South African route capacity, project feasibility or operating performance is claimed.  
**Parent:** [DEEP-TIME-SA-CORRIDOR-001 #895](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/895).  
**Current rail source-intake owner:** [Sol-Atlas TRIM-CORRIDOR-EVIDENCE-001 #400](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/400), composed with [Symtropy TRANSPORT-DATA-001 #1631](https://github.com/Luminous-Dynamics/symtropy/issues/1631) and [SOUTHERN-AFRICA-FREIGHT-BASELINE-001 #3243](https://github.com/Luminous-Dynamics/symtropy/issues/3243).

## 1. Purpose

The first version had 20 synthetic corridor cases. Version 0.2 adds ten cases to encode Transnet Rail Infrastructure Manager's September 2026 Network Statement Version 4 and Annexure 33/33a/33b capacity semantics.

The benchmark does not ingest or assume numeric values from the official 33b workbook. That macro-enabled workbook has not yet been captured or parsed. The new cases qualify what the eventual parser and downstream models must preserve.

## 2. Current evidence model

TRIM Version 4 was published 18 September 2026. Its Annexure 33 Network Register (June 2026 v0.3) is valid for the 2026/27 timetable and identifies General Freight Johannesburg–Durban. It defines the context for section-level theoretical, practical, lost, available, allocated and spare train-path capacity, with daily and weekly resolutions. Annexure 33a supplies station-to-station running times/distances/frequency; Annexure 33b is the section-level capacity workbook, explicitly updated 29 October 2026. Primary index: https://www.transnet.net/TRIM-Networkstatement .

The full source and acquisition-status record is the companion [source register v0.2](DEEP_TIME_SA_CORRIDOR_SOURCE_REGISTER_V0_2.json). The workbook is linked but not acquired; there are no 33b cell values in this benchmark.

## 3. Thirty-case corpus

Cases SA-COR-001 through SA-COR-020 from v0.1 are preserved. Ten rail-capacity semantics cases are added:

| Case | Subject | Required behavior |
|---|---|---|
| 021 | Section-level route bottleneck | Use the minimum available capacity along a serial path |
| 022 | Available vs allocated vs spare | Reconcile spare only under the stated source definition |
| 023 | Theoretical/practical/lost | Preserve distinct values and source-specific definitions |
| 024 | Daily vs weekly | Use the declared operating-day basis, not calendar-day multiplication |
| 025 | Shared route section | Do not sum parallel feeders beyond common-section capacity |
| 026 | Allocated vs operated | Unused allocated paths do not automatically become spare |
| 027 | Scheduled vs realized time | Keep schedule and individual observed run distinct |
| 028 | XLSM parsing safety | Preserve formulas/cached values/metadata; never execute VBA |
| 029 | TRIM coverage scope | Do not label TRIM's graph the entire national rail system |
| 030 | Future plan vs operation | Do not credit uncommissioned/planned capacity as operational |

## 4. Exact synthetic references added

- **SA-COR-021:** sections 12, 9, 8 and 11 train paths/day; demand 10 => route capacity 8, delivery 8, unmet 2; bottleneck S3.
- **SA-COR-022:** operational available 14 paths/week, allocated 9 => spare 5 under the fixture's explicit definition.
- **SA-COR-024:** 4 paths per operating day × 5 declared operating days = 20 paths/week, not 28.
- **SA-COR-025:** two feeders supply 8 and 6 paths, but share a downstream section of capacity 10 => total throughput 10, not 14.
- **SA-COR-027:** one synthetic observed run 150 min vs scheduled 120 min => difference 30 min. One run cannot be promoted to a representative distribution.

The exact arithmetic from all eight v0.1 exact cases is retained unchanged. Total exact references in v0.2: 13.

## 5. Modelling rules

### Capacity layers

Never collapse these into one "rail capacity":
- theoretical;
- practical;
- lost with reasons;
- operationally available;
- allocated/committed;
- spare;
- actually operated paths;
- actual freight payload.

### Section graph

A corridor route is a sequence of section edges. Shared sections, junctions, directionality, closures and reroutes must be explicit. Route throughput is bounded by the joint capacities of its required sections. Two routes that use the same physical section are not independent capacity paths.

### Units and conversion

A train path/day or train path/week is not tonnes/day or tonnes/year. Conversion requires separately qualified train formation, wagon capacity, locomotive/traction, commodity, payload, operating calendar, direction, path occupancy and reliability evidence. Do not invent average payload or days per year.

### Running time

Scheduled point-to-point time (including any stated buffer) is not an observed running-time distribution. Observed times need event-level timestamps, population, period, route version and cancellation/reroute handling.

### Workbook safety

The 33b input is macro-enabled XLSM. Ingestion is read-only data extraction: pin bytes and digest, inspect sheets and hidden states, preserve formulas and cached values, capture cell-level lineage, and never execute macros or external links. The workbook is not yet acquired, so no source workbook values are qualified.

## 6. Comparison baselines

Same route, same time basis, same demand, same constraints and same source versions:
- theoretical-capacity-only baseline (diagnostic only; not operational);
- practical-capacity baseline;
- available-minus-allocated residual where source definitions support it;
- time-dependent section bottleneck / max-flow;
- simple shortest path and capacity-constrained min-cost flow;
- fixed and staged capacity transition policies;
- exhaustive exact oracle on small networks;
- Symtropy #1601 multimodal whole-life cost and #1639 schedule-aware allocation where exact data contracts match.

Do not compare our section-level benchmark directly with port-call CPPI statistics. Port time in port is not inland corridor reliability or door-to-door service.

## 7. Claim ceiling

Passing this synthetic suite proves only the declared finite graph/capacity semantics. It does not prove current 2026/27 spare train paths, actual service, actual freight moved, rail reliability, route safety, throughput in tonnes, corridor cost, or a preferred investment. Real figures remain blocked until 33b/33a are captured, source-qualified and independently verified.
