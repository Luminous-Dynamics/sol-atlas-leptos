# Energy Solver Interoperability: SOTA Snapshot and Luminous Differentiation

**Snapshot:** 16 November 2026  
**Status:** research / benchmark strategy; not a performance claim by Luminous Dynamics.

## Executive decision

The recent Rust-native power-systems landscape has moved. Luminous should not duplicate established solver and format surfaces simply to claim feature parity. We should compose the strongest available specialist tools and build the assurance layer that ties their outputs to exact inputs, physical assets, operating envelopes, measured service and lifecycle decisions.

The immediate competitive reference is **eigenergy/Tellegen + PowerIO**. Tellegen is a Rust-native engine for DC power flow, DC optimal power flow, AC power flow and the Jabr SOCWR relaxation, with KKT-based forward/adjoint sensitivities; its browser/WASM and Svelte packages provide a local case-to-visualization workflow. PowerIO is its typed, source-aware grid-format and calculation-preparation layer. Their public code and documentation are materially more advanced than a simple DC solver or a data importer.

That does not mean Tellegen replaces the entire engineering stack. Its documented nonlinear AC-OPF feature is an opt-in development path, not the shipping DC OPF / AC PF / SOCWR surface. Its multiconductor IVR OPF is described as experimental and separately qualified. Solver mathematics, local physical validation, source admission, dynamic model fidelity, HIL and utility acceptance remain distinct questions.

**Luminous should aim to be solver-independent, evidence-native power infrastructure engineering—not another general-purpose OPF solver.**

## 1. Comparator map

| System | Established strength | Appropriate Luminous use |
|---|---|---|
| MATPOWER 8.1 | Widely used reference implementation for AC/DC power flow and OPF; named primal/dual results | Pinned numerical reference; independent source-bound primal and KKT audit |
| PowerModels.jl | Mathematical formulations for power-system optimization and PGLib comparisons | Independent formulation/reference comparison, with exact DCP/DCMP/AC model identities |
| Tellegen | Rust-native DC PF, DC OPF, AC PF, SOCWR and KKT sensitivity engine; browser/WASM stack | Additional independent solver/reference; do not fork its solver surface |
| PowerIO 0.12.1 | Typed source-aware compiler for grid data, format readers/writers, diagnostics and calculation preparation | Independent parser/projection oracle and interoperability adapter; not a solver |
| PyPSA 1.4.0 | Chronological, multi-period energy-system and network optimization | Planning, storage, rolling-horizon and network-constrained chronological comparisons |
| pandapower 3.5.6 | Distribution-oriented power flow, OPF, contingency analysis, short circuit and state estimation | Distribution/feeder models and independent import/analysis comparison |
| OpenDSS | Distribution simulation, controls and time-series feeder analysis | Distribution-domain reference where model profiles match |
| PowerFactory / PSCAD / RTDS | Mature RMS/EMT and real-time simulation ecosystems | Specialist dynamics and HIL/real-time comparisons; not a static DCOPF runtime contest |
| REopt API v3 / REopt.jl | Distributed energy, facility/campus/microgrid sizing, economics and resilience | Site-level technology sizing and resilience under matched facility boundaries |
| Grid2Op / RL2Grid | Sequential grid-operation and agent/controller evaluation | Future sequential-control and runtime-shield evaluation |

These products are not interchangeable. A solver comparison is only meaningful when the mathematical formulation, topology, input fields, operating limits, objective, storage chronology, units, solver budget and output semantics are aligned.

## 2. Tellegen: current capabilities and what its results say

Repository inspected: https://github.com/eigenergy/tellegen  
Inspected revision: 199c1e730bc5127b8ccbe89ee85046eb1fb4756f

The public README and architecture describe:
- DC power flow and DC OPF using a B-theta formulation;
- polar Newton AC power flow;
- Jabr SOCWR, a convex relaxation of AC OPF;
- a shared sensitivity architecture spanning DC OPF KKT, AC PF Newton and SOCWR KKT systems;
- forward/adjoint modes and finite-difference comparisons;
- Rust engine, native CLI/server, Python bindings, browser/WASM engine, Svelte viewer and WebMCP adapter;
- PowerIO for source parsing, typed network/problem preparation and exchange.

Important boundaries in Tellegen's own documents:
- the exact polar nonlinear AC-OPF route is opt-in/development-only, and is not the same public shipping capability as DC OPF / AC PF / SOCWR;
- the experimental multiconductor IVR OPF has separate qualification and portability boundaries;
- its independent IVR physical validator reconstructs KCL/KVL/device links, load/control laws, selected limits and objective over prepared electrical data, but the docs state that this validator shares PowerIO preparation and cannot detect a physical constraint that was already dropped before preparation. The import diagnostics guard unsupported, malformed, field-dropped and record-dropped cases; Luminous should test those exact boundaries, not assume a gap.

### Published PGLib snapshot (not an independent Luminous result)

Tellegen's committed benchmark snapshot identifies PGLib-OPF v23.07 and reports 198 case/variant rows overall: 143 solved, 50 caveat, 2 failed and 3 skipped.

Its formulation roll-ups report:
- DC OPF: 148 reproduced, 45 consistent, 0 mismatch, across 193 cases with a DC comparison.
- SOCWR: 183 reproduced, 6 consistent, 0 mismatch, across 189 cases with a SOCWR comparison.

The benchmark-result provenance identifies an earlier Tellegen revision and PowerIO 0.7.1, while the inspected current workspace manifest uses PowerIO 0.12.1. Treat this as a dated, pinned benchmark artifact—not as a re-run of the current head. It is also not evidence that every case solved, that all caveats vanished, or that AC-OPF was solved globally.

Sources:
- Tellegen README: https://github.com/eigenergy/tellegen
- Benchmark result snapshot: https://github.com/eigenergy/tellegen/blob/main/docs/src/benchmark-results.md
- Benchmark methodology: https://github.com/eigenergy/tellegen/blob/main/docs/src/methodology.md
- Validation scope and limitations: https://github.com/eigenergy/tellegen/blob/main/docs/src/validation.md
- Formulation / shipping architecture: https://github.com/eigenergy/tellegen/blob/main/docs/src/architecture.md
- PowerIO: https://github.com/eigenergy/powerio
- PowerIO Julia binding: https://github.com/eigenergy/PowerIO.jl

## 3. What this means for Luminous

### Do not build duplicates

Do not spend the next engineering cycle reimplementing:
- a generic MATPOWER/PSS-E/OpenDSS/etc. compiler;
- a standard DC OPF solver;
- generic AC PF or SOCWR;
- a generic KKT sensitivity framework;
- a map-first browser interface that already exists in multiple products.

Instead, pin and integrate relevant tools behind strict adapters. Reimplement only when a measured gap exists and the new component can be validated more credibly than the tool it replaces.

### Build the evidence and assurance plane

Luminous should own the binding between:
- original source bytes, rights, parser and complete field inventory;
- model profile, equations, assumptions, validity envelope and uncertainty;
- exact equipment / module / as-built configuration / firmware identity;
- solver build, inputs, options, output and all omitted/unsupported semantics;
- independent primal, dual, KKT and conservation checks;
- asset commissioning, manufacturing deviations, sensors and calibration;
- operating events, faults, maintenance, repairs, requalification and observed useful service.

A solver's success flag, a parser's lossless source echo, a converged result, a zero residual or two solvers agreeing does not independently establish physical truth.

### Cross-system ownership

- **Symtropy:** physical system composition, solver adapters, time-domain / multi-physics coupling contracts, fault replay and digital-twin validity.
- **Symthaea:** scientific hypotheses, design alternatives, model/experiment selection, causal diagnosis and engineering optimization proposals.
- **Sol-Atlas:** source admission, network/resource/geospatial context, solver-independent auditing, dispatch/service outcomes and benchmark receipts.
- **Mycelix:** rights, identity, custody, provenance, calibration lineage, supplier/process/asset records and authorized governance.
- **Specialist solvers and qualified laboratories:** domain-specific numerical and physical truth within their validated envelopes.

The current Symtropy issue [#5607](https://github.com/Luminous-Dynamics/symtropy/issues/5607) and draft [PR #5609](https://github.com/Luminous-Dynamics/symtropy/pull/5609) define the exact asset/configuration-bound physical-model profile and validation receipt. Sol-Atlas [#899](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/899) and [#902](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/902) define source-bound primal and numerical KKT checks. New cross-solver integration work is tracked by [#908](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/908).

## 4. Immediate matched benchmark

First compare the exact MATPOWER 8.1 makeBdc/DCOPF profile, not a mixture of nominally “DC” formulations.

### P0: hand-derived cases
1. Two-bus uncongested quadratic-cost case.
2. Two-bus, two-generator, active 40 MW line-limit case with nonzero line shadow price.
3. Tap / phase-shift known-answer variant.
4. One-sided angle limit, generator upper/lower bound, inactive constraint and degenerate multiplier variants.

The existing Sol-Atlas primal auditor (#899) and KKT verifier (#902 / draft PR #905) begin with this exact scope. The active-line fixture is intended to prevent an equality-only positive control from passing while line-dual logic is wrong.

### P1: public PGLib cases
Only after P0 and the exact semantic comparison pass:
- IEEE 14;
- RTS 24;
- IEEE 57;
- then larger cases selected in advance.

For every case bind the exact original MATPOWER file digest and separate formulation profiles:
- MATPOWER 8.1 makeBdc DCOPF;
- MATPOWER DCMP if that is the actual comparator formulation;
- PowerModels DCP;
- PowerModels DCMP;
- Tellegen DCOPF.

Do not treat these as interchangeable. Differences in taps, phase shifts, branch admittance, base conversions, shunts, reference/slack treatment, line constraints, generator costs or angle limits can change the optimization problem.

### P2: parity metrics
Report separately:
- exact source-field and diagnostics parity;
- per-bus angles and active injections;
- per-generator dispatch and bounds;
- each branch's from/to flows and limits;
- objective and cost components;
- nodal marginal values / prices;
- primal residuals, dual-sign violations, stationarity and complementarity;
- primal-dual gap and whether it is a valid lower bound;
- formulation support coverage and NotComparable disposition;
- solve/build/parse time, p50/p95/p99 runtime, memory and time-to-first-feasible only under matched hardware/build budgets.

### P3: chronology and service
After solver parity, compare:
- SimBench time-series, storage transitions and outages;
- nonanticipative scenario dispatch and forecast error;
- N-1 / corrective-action feasibility and AC validation;
- essential-service floors, unmet service, longest deficit, terminal reserve and recovery;
- model validity/abstention under stale, missing, conflicting and out-of-domain evidence.

Do not promote a static DCOPF result to transient stability, protection, grid-forming capability or physical service reliability.

## 5. Differentiation and proposed SOTA targets

The achievable competitive wedge is not necessarily an optimizer that is 2% faster. It is an end-to-end workflow that makes it much harder to accept a wrong engineering decision.

Proposed goals—not achieved results:
- 100% exact identity binding from source bytes through result receipt.
- Zero silent dropping/defaulting of an in-scope field.
- Zero acceptance of stale / mismatched asset, firmware or topology records.
- Zero false-valid admissions on a frozen source mutation and negative-control corpus.
- 100% field and formulation coverage disclosure; unsupported semantics are explicit.
- Independent primal/objective/KKT evidence for all supported convex DCOPF candidates.
- Held-out physical/model correlation before any field-performance claim.
- Essential-service floor violations never hidden by aggregate cost or weighted score.
- Dependency-aware invalidation and explicit requalification after material configuration changes.
- Full design → procure → manufacture → as-built → commission → operate → repair → requalify lineage.

For numerical targets, freeze tolerance by equation/reference accuracy and numerical conditioning before test execution. Do not choose per-case tolerances to manufacture parity. Floating-point KKT is numerical evidence, not an exact-rational proof. AC local convergence is not a global optimality certificate.

## 6. Safe expansion to electromagnetics and dynamics

Do not create one universal EM solver. Route each engineering question to the suitable regime:
- network power flow / OPF for steady-state network decisions;
- RMS dynamics for slower electromechanical and averaged control questions;
- EMT for converter switching, current limits and fast transients;
- impedance/admittance scans for weak-grid and multi-converter small-signal interactions;
- FEM/BEM/FDTD or calibrated field extraction for local transformer, winding, busbar, cable and enclosure field questions;
- circuit/PDN models for parasitics and switching-equivalent circuits;
- thermal RC/CFD, electrochemistry and structural models where those physical dependencies alter the claim.

The missing integration is the exact applicability contract: model revision, asset/configuration, frequency/time/thermal/load envelope, coupling timebase, calibration, uncertainty and output-specific validity. Use Symtropy #5607 / PR #5609 to build that without duplicating Tellegen or PowerIO solvers.

## 7. Non-goals and honesty boundary

This strategy does not establish that Luminous has reproduced Tellegen's published results, beaten any solver, or obtained field-validated grid physics. PR #905 and PR #5609 are draft, with exact-head CI and physical qualification pending. A new solver comparison must first pass semantic matching and independent checks.

No live switching, protection-setting changes, dispatch or grid operation is authorized by these software artifacts. Grid-code compliance, interconnection, professional engineering sign-off, HIL and field validation remain separate requirements.

## References and snapshots

- Tellegen repository inspected revision: 199c1e730bc5127b8ccbe89ee85046eb1fb4756f.
- Tellegen benchmark results and methodology links above; benchmark provenance is distinct from the inspected current workspace dependency versions.
- PGLib-OPF v23.07 baseline: https://github.com/power-grid-lib/pglib-opf/blob/master/BASELINE.md
- MATPOWER 8.1: https://matpower.org/
- PyPSA releases: https://docs.pypsa.org/latest/release-notes/
- pandapower releases: https://pypi.org/project/pandapower/
- REopt v3: https://developer.nlr.gov/docs/energy-optimization/reopt/v3/
