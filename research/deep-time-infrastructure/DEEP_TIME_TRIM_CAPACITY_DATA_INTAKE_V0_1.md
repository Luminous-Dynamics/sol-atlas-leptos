# TRIM 2026/27 Rail-Capacity Evidence Intake
## Durban–Gauteng source and semantics addendum v0.1

**As of:** 16 November 2026  
**Programme:** [DEEP-TIME-SA-CORRIDOR-001 #895](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/895)  
**Existing source-intake owners:** [Sol-Atlas #400](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/400), [Symtropy #1631](https://github.com/Luminous-Dynamics/symtropy/issues/1631), [Symtropy #3243](https://github.com/Luminous-Dynamics/symtropy/issues/3243).

## 1. Corrected current source baseline

The earlier demonstrator v0.1 source register used the Government 2024/25 network-statement notice as the only rail network-statement source. That is now insufficient and is retained only as historical provenance.

TRIM's official current publication page states:
- Final Network Statement Version 4 published 18 September 2026;
- Annexure 33 Network Register, June 2026 v0.3;
- Annexure 33a point-to-point running times/distances;
- Annexure 33b Capacity Statement updated 29 October 2026.

Primary index: https://www.transnet.net/TRIM-Networkstatement .

The Network Register PDF is dated 26 June 2026, valid for the 2026/27 timetable under Version 4, and explicitly identifies **General Freight: Johannesburg to Durban** as one of its routes. It says the 33b workbook supplies ten routes of section-level capacity, including:
- theoretical train-path capacity;
- practical train-path capacity;
- capacity lost and reasons;
- operationally available train-path capacity by section at daily and weekly resolution;
- capacity already allocated to other applicants/operators;
- remaining spare capacity.

The same Network Register distinguishes Annexure 33a's station-to-station running times, distances and frequency, and warns that stated times include recovery/buffer assumptions but depend on prevailing conditions. Theft/vandalism, rolling-stock failures, loss of overhead power and route diversions can alter achieved service. It explicitly excludes PRASA and other infrastructure managers from its own network scope.

References:
- [Final Network Statement v4 index](https://www.transnet.net/TRIM-Networkstatement)
- [Annexure 33 Network Register PDF](https://www.transnet.net/getFile.ashx?id=21073)
- [Annexure 33a Running Times XLSX](https://www.transnet.net/getFile.ashx?id=22633)
- [Annexure 33b Capacity Statement XLSM, updated 29 October 2026](https://www.transnet.net/getFile.ashx?id=32419)

## 2. Data acquisition status: do not claim the workbook was parsed

The official website and links were retrieved, and the Annexure 33 PDF text and rendered pages were inspected. The browser identifies Annexure 33b as macro-enabled XLSM. The current web/file retrieval surface rejected the workbook's macro-enabled Excel MIME type; a local download attempt also failed. Therefore the exact XLSM bytes, file hash, sheet names, formula values and section rows have **not** been captured or parsed in this work session.

The direct link is a verified source location, not a captured file. Do not commit or redistribute it until the existing #400/#1631 rights, security and source-review process is complete.

## 3. Critical semantic shift: capacity is a layered ledger, not one scalar

Keep these quantities separate:

| Layer | Meaning | What it does not mean |
|---|---|---|
| Theoretical train-path capacity | Idealized path ceiling for a defined section/time basis | What the infrastructure can practically deliver today |
| Practical train-path capacity | Capacity after declared infrastructure/operational constraints | What is currently available after other commitments |
| Capacity lost | Declared reduction and reason(s) | A number to add back without reviewing source definitions |
| Operationally available capacity | Available train paths at section level, daily/weekly | Tonnes moved or an allocated train path |
| Allocated capacity | Contractually committed/allocated paths to applicants/operators | Actual trains operated or actual payload |
| Spare capacity | Residual capacity after allocations under source definitions | A guaranteed slot or a route-wide throughput number |
| Actual operated paths | Observed trains/services run in a period | A capacity statement or train-path entitlement |
| Actual freight | Observed payload with commodity/time/unit basis | Path availability or designed capacity |
| Scheduled running time | Published point-to-point time including stated buffers | An observed distribution or service guarantee |
| Realized running time/reliability | Actual service outcomes under observed conditions | Recoverable from a schedule alone |

Do not convert train paths/day or train paths/week into tonnes/year unless a separately sourced and qualified train-formation, locomotive, wagon, commodity, payload, directionality, operating-days and realized reliability model exists.

### Route aggregation

A route is a sequence of sections with shared resources, junctions, direction and possibly alternative paths. A route's feasible throughput is constrained by the joint section capacities and allocation interactions. Do not sum every section's capacity along a single corridor. Do not sum parallel route capacities when they share a bottleneck section or another common failure domain.

The first route-level service calculation should use a time-expanded, direction-aware network with explicit shared section edges, allocated-path constraints, closures and reroutes. Unknown section correspondence must remain unresolved rather than snapping nearest GIS lines.

### Daily / weekly resolution

Treat daily and weekly capacity values as different source quantities with their own basis. Do not multiply daily by seven or divide weekly by seven unless the published definition, operating-day calendar, direction and aggregation convention are confirmed. Holidays, maintenance windows, timetable days and directionality may alter equivalence.

### Schedule / reliability distinction

Annexure 33a is not an observed running-time dataset. Keep schedule/buffer, actual transit-time observations, cancellations, reroutes and disruptions in different tables. Use a distribution only where actual timestamped observations and denominator coverage are available.

## 4. Workbook handling contract

Existing #400/#1631 own source ingestion; this addendum defines the requirements to preserve:
- raw workbook bytes, publisher URL, retrieval time, content type and SHA-256;
- workbook format and macro-enabled flag;
- sheet names/visibility, hidden and very-hidden sheets, used range and workbook properties;
- formulas and cached values separately; never execute VBA macros;
- external links, defined names, data validations, merged cells and table definitions where relevant;
- source sheet/cell/range and parser version for every normalized value;
- exact units, row/column headers, route/section identifiers, valid period and direction;
- rejection/discrepancy logs, missingness, duplicate IDs and independent reconciliation.

A pure-Rust reader is viable to test: calamine 0.38.0 (published 2 November 2026) documents support for XLSX/XLSM and formula retrieval, including workbook/VBA inspection APIs. It does not authorize macro execution or replace byte-level custody, rights review, formula/cached-value interpretation or independent validation. See [calamine 0.38.0 changelog](https://docs.rs/crate/calamine/latest/source/Changelog.md), [API documentation](https://docs.rs/calamine/latest/calamine/).

## 5. Added synthetic capacity semantics cases

The corridor benchmark v0.2 adds controls for:
- route bottleneck across section-level available capacities;
- operational availability minus allocated capacity to obtain declared spare capacity;
- daily/weekly basis without implicit seven-day conversion;
- two routes sharing one section bottleneck;
- scheduled versus observed running time;
- capacity loss and theoretical/practical discrepancy;
- allocation versus actual train operation;
- workbook formula/cached value separation and macro non-execution;
- TRIM network scope exclusions;
- investment-plan/TOC future plan versus commissioned and observed service.

Synthetic figures are only contract test inputs. No 33b cells have been consumed.

## 6. Promotion path

1. Resolve workbook terms/security handling and retrieve exact 33b file.
2. Freeze 33b/33a/33/Version4 artifacts with SHA-256, retrieval date and metadata.
3. Parse sheets and cells with the existing Rust intake owner; no macros executed.
4. Reconcile section IDs, route path order, directions and daily/weekly fields against Annexure 33.
5. Cross-check the Johannesburg–Durban route's section coverage and exact expected figures with an independent verifier.
6. Bind section graph to qualified geometry under Sol-Atlas #400; do not fabricate coordinates.
7. Only then expose route capacity to Symtropy #1601/#1639 scheduling and #3243 baseline scenarios.
8. Preserve actual train-operation and freight data as separate later evidence classes.

## 7. Claim ceiling

This addendum establishes current source references and semantic requirements only. It does not establish 2026/27 spare path capacity, operational train count, tonnes moved, actual transit-time distribution, operator utilization, route reliability or a corridor investment recommendation. The Annexure 33b workbook remains unacquired and unparsed at this snapshot.
