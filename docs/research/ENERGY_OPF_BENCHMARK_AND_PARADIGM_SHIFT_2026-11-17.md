# Sol-Atlas Energy Optimization: Benchmark and Paradigm-Shift Plan

**Research snapshot:** 17 November 2026  
**Status:** benchmark contract and research targets; no claim of SOTA performance.

Related work: [ENERGY-NETWORK-QUAL-001 #332](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/332), [cross-solver receipt #908](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/908), [exact-rational proof #916](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/916), [benchmark matrix #919](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/919).

## 1. Executive thesis: time-to-qualified solution

Sol-Atlas should not compete only on the number of seconds needed to return an objective value. The engineering product is:

`source bytes + formulation identity + normalized model + solver build + solution + independent witness + applicability boundary + receipt`

Measure **time-to-qualified-solution**: model build, preprocessing, solve, independent verification and reproducible evidence, with each component separately reported.

The paradigm shift is to treat optimization as proof-carrying engineering. A solver proposes; an independent verifier checks. The resulting evidence is bound to the exact source, model convention, software build and applicable asset/configuration. Unknown or unsupported conditions remain unknown instead of being silently promoted.

## 2. Industry landscape (checked 17 November 2026)

| System | Current public reference checked | What it is useful for | Important comparison caveat |
|---|---|---|---|
| MATPOWER | 8.1 (released 12 July 2025) | Power-system power-flow and OPF reference, including the `makeBdc` convention | Pin exact source/dependency commits; the solver success flag is not an independent certificate |
| PGLib-OPF | v23.07 is the latest tagged release shown | IEEE PES AC-OPF benchmark suite, MATPOWER-format cases | Published baseline hardware/software is historical and solve times are approximate |
| PowerModels.jl | current docs expose DCP and DCMP families, plus AC/conic formulations | Common modeling layer for comparing mathematical formulations | MATPOWER's `makeBdc` must be compared with `DCMPPowerModel`, not default `DCPPowerModel` |
| HiGHS | v1.15.1 (2 July 2026) | Open LP/QP baseline: simplex, interior point, PDLP, active-set QP and HiPO | Appropriate for convex DC-OPF LP/QP; not an AC-OPF nonlinear modeling system |
| Gurobi | Optimizer 14.0 | Strong commercial LP/QP/MIP and nonlinear baseline | Vendor release performance claims are not PGLib measurements; run our own matched panel |
| Ipopt | PGLib public baseline uses Ipopt 3.14.4 | Nonlinear AC-OPF benchmark baseline with PowerModels.jl | Local NLP optimum must not be mislabeled a global optimum |

Primary sources:

- [MATPOWER 8.1 release notes](https://github.com/MATPOWER/matpower/blob/master/docs/relnotes/MATPOWER-Release-Notes-8.1.md) and [makeBdc reference](https://matpower.org/documentation/ref-manual/legacy/functions/makeBdc.html).
- [PGLib release history](https://github.com/power-grid-lib/pglib-opf/releases), [PGLib project](https://github.com/power-grid-lib/pglib-opf) and [published baseline](https://github.com/power-grid-lib/pglib-opf/blob/master/BASELINE.md).
- [PowerModels formulation details](https://lanl-ansi.github.io/PowerModels.jl/dev/formulation-details/).
- [HiGHS releases](https://github.com/ERGO-Code/HiGHS/releases) and [solver documentation](https://ergo-code.github.io/HiGHS/stable/).
- [Gurobi 14.0 What's New](https://www.gurobi.com/product/whats-new) and [current parameter reference](https://docs.gurobi.com/projects/optimizer/en/current/reference/parameters.html).

### Formulation firewall: DCP is not DCMP

PowerModels documents that `DCPPowerModel` uses branch susceptance `-x/(r²+x²)` and omits transformer tap ratios and phase shifts. `DCMPPowerModel` uses `-1/x`, includes taps and phase shifts, and is designed to match MATPOWER calculations.

Therefore:

- Compare Sol-Atlas MATPOWER `makeBdc` with MATPOWER 8.1 and PowerModels `DCMPPowerModel`.
- Keep `DCPPowerModel` as a separate formulation diagnostic, not as a pass/fail solver-parity reference.
- Treat differences in (r,x), tap, phase-shift, shunt, angle-limit, flow-limit, slack and cost semantics as model differences until disproven.

### Historical PGLib timings are not a current leaderboard

The PGLib public baseline reports PowerModels.jl v0.19.9, Ipopt.jl v1.4.1 / Ipopt 3.14.4, PGLib v23.07 and dual Intel 2.10 GHz CPUs. It explicitly notes that timings are approximate, exclude roughly 2–5 seconds of Julia JIT, and depend on the linear solver. Use those results as historical reference points, not as current claims of comparative speed.

## 3. Benchmark tracks

| Track | Problem class | Candidate baselines | Required evidence |
|---|---|---|---|
| A. Source/model fidelity | MATPOWER parsing, units, topology, cost and device interpretation | Sol-Atlas parser, PowerIO, MATPOWER | source identity, exact field differential, unsupported-field mutation controls |
| B. DC power flow | fixed injections, taps, phase shifts, islands and offline devices | MATPOWER 8.1, Sol-Atlas reference kernel | nodal residuals, reference angle, branch PF/PT, topology/island receipt |
| C. Convex DC-OPF | linear/convex quadratic cost, line/angle/generator limits | MATPOWER/MIPS, PowerModels DCMP, HiGHS, optional Gurobi | independent primal + KKT audit, complementarity, gap; exact witness for rational cases |
| D. AC-OPF | nonlinear AC power balance, voltage/reactive/thermal constraints | MATPOWER, PowerModels.jl + Ipopt, optional commercial solvers | AC residuals and limits; local/global/relaxation claim labels |
| E. Security-constrained and chronological | N-1, corrective redispatch, reserves, ramping, storage, unit commitment | matched MILP/QP/NLP models and solvers | all-contingency coverage, intertemporal feasibility, recourse and reserve evidence |
| F. Distribution and unbalanced systems | three-phase feeders, DER and inverter limits | OpenDSS / PowerModelsDistribution and matched tools | phase-resolved balance, neutral/ground representation, measurement correlation |
| G. Dynamic and physical | RMS, EMT, controller, HIL and field | independent dynamic solvers, vendor models and test rigs | exact equipment/firmware/configuration, calibration, metrology, test protocol and observation |

Do not combine DC-OPF, AC-OPF, convex relaxations, unit commitment and physical field validation into one leaderboard. They answer different questions.

## 4. Fair-run protocol

Each run must freeze and report:

- Source Git blob SHA-1 and SHA-256; case version; operating condition; data and configuration digests.
- Exact mathematical formulation, including base MVA, (r,x), taps, phase shifts, shunts, branch limits, angle limits, cost curve semantics, slack handling and per-unit conversions.
- Solver executable/build commit, dependency versions, algorithm, tolerances, options, linear solver, thread count, CPU, RAM, OS and BLAS.
- Cold-start and warm-start status, model-build/preprocessing time, solve time, independent-verification time, time-to-qualified-solution, peak memory and exit status.
- Objective, normalized primal residuals, dual feasibility, stationarity, complementary slackness, primal-dual gap, violations and independent witness disposition.

For performance claims, use at least ten repetitions on pinned hardware and report median, p95 and dispersion. Separate JIT compilation and model-build time from solve time, but include all of them in time-to-qualified-solution. Never compare a warm/cached run with a cold run, and never loosen tolerances after seeing results.

## 5. Aspiration gates (not measured achievements)

| Dimension | Target |
|---|---|
| Independent correctness | 100% pass across supported, matched-formulation cases |
| Negative controls | 0 false pass; all identity, sign, unit, stale-receipt and constraint mutations rejected |
| Numerical quality | For well-conditioned convex DC-OPF cases: normalized primal, dual/stationarity residuals and relative gap target (le 10^{-8}), preregistered per case |
| Coverage | Full PGLib v23.07 corpus within the admitted model profile; unsupported cases explicitly recorded |
| Reproducibility | 100% deterministic source/model/solver receipts and repeatable output digests |
| Performance | After parity, target median time-to-qualified within 1.25× of the fastest matched solver on the shared panel, with better receipt/verification latency |
| Scale | Grow from IEEE 14 / RTS 24 / IEEE 57 to IEEE 118 / 300 and larger PGLib networks; measure memory and verification cost |

The 1.25× performance target is an aspirational engineering gate, not a present achievement. First establish parity and independently auditable correctness.

## 6. Differentiating research directions

### 6.1 Proof-carrying optimization

Require every candidate to carry its primal point and dual witness. An independent verifier reconstructs constraints, checks identities, stationarity, dual signs, complementarity and primal-dual gap. For rational LPs, move beyond checking floating-point residuals: reconstruct an exact primal/dual witness from a simplex basis and verify it with checked rational arithmetic.

The two-bus exact proof is the first calibration case, not a general MATPOWER proof.

### 6.2 Exact and interval verification

Use floating-point solvers for speed and discovery; exact rational verification for small rational LP fixtures; interval or rigorously bounded arithmetic for larger problems. When an interval contains zero ambiguously or conditioning prevents a sound decision, return inconclusive/unknown, not pass.

### 6.3 Counterfactual operating envelopes

Move from a single dispatch value to a verified decision surface: constraint margin, marginal cost, shadow price, binding-set stability, and which contingencies or asset/configuration changes reverse the action. A shadow price is useful only when its model, unit, sign, source and validity domain are known.

### 6.4 Multi-fidelity validity contracts

DC → AC → RMS → EMT → HIL → field is not an automatic promotion ladder. Each model output must declare the exact asset/configuration, operating envelope, output-specific validity axes, numerical method, uncertainty and claim ceiling.

### 6.5 Time-to-qualified engineering

Optimize for a candidate already accompanied by source lineage, explicit assumptions, constraint coverage, independent validation and evidence receipt. This is the product distinction over a solver that merely returns a low objective.

## 7. First execution slices

1. Finish the exact-rational two-bus proof and preserve its mutation controls.
2. Complete exact-head MATPOWER primal, independent reference and numerical KKT gates.
3. Add a matched PowerModels `DCMPPowerModel` adapter and a HiGHS LP/convex-QP adapter; retain DCP only as an explicitly different-formulation diagnostic.
4. Generate a versioned manifest from the pinned PGLib v23.07 cases, operating-condition coverage and explicit out-of-profile reasons.
5. Publish raw CSV/JSONL run receipts plus deterministic summaries. Failed and not-evaluable cases must remain in the denominator and must not disappear from aggregate scores.

## 8. Non-claims

No current claim that Sol-Atlas beats HiGHS, Gurobi, Ipopt, MATPOWER or PowerModels in speed, solution quality, scalability or physical accuracy. Those claims require formulation-matched experiments, pinned hardware, independently verified outputs and public run receipts.
