# OPF Benchmark Contracts v0.1

This package makes the benchmark programme from [ENERGY-OPF-BENCHMARK-001 #919](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/919) machine-readable.

## Files

- `benchmark-manifest-v0.1.schema.json`: strict schema for versioned case panels and pinned reference stacks.
- `benchmark-run-receipt-v0.1.schema.json`: strict schema for a single solver execution and independent qualification record.
- `pglib-v23.07-typ-starter-manifest-v0.1.json`: three-case TYP starter panel with exact Git blob identities, the pinned MATPOWER 8.1 dependency stack, current exact-head evidence snapshot, and explicitly context-only PGLib historical baseline values.
- `benchmark-run-not-run-fixture-v0.1.json`: intentionally `not_run` receipt proving that schema validation does not invent a solver result.
- `tools/validate_opf_benchmark_contracts.py`: CI-only schema and cross-record checks.
- `.github/workflows/opf-benchmark-contracts.yml`: pinned schema validation and negative mutation tests.

## Important comparison rules

- MATPOWER 8.1 `makeBdc` uses `-1/x` and includes taps and phase shifts. Match it with PowerModels `DCMPPowerModel`.
- PowerModels `DCPPowerModel` uses `-x/(r^2+x^2)` and omits transformer taps and phase shifts. The manifest labels it diagnostic-only for MATPOWER parity.
- The public PGLib baseline results are kept as context only; they are not matched-formulation evidence for this MATPOWER profile.
- The starter panel covers only PGLib TYP cases 14, 24 RTS and 57 IEEE. It is not the full PGLib v23.07 corpus.
- The evidence snapshot records exact workflow IDs and PR head. It shows primal audit, fixed-injection DC reference differential, PowerIO source differential and MATPOWER DC network evaluation passes on that head. It does not claim HiGHS/Gurobi/PowerModels solver parity, AC-OPF global optimality, security-constrained dispatch, or physical validation.

## Receipt semantics

A solver's `optimal` outcome is not sufficient for `qualification.disposition=pass`. The CI semantic validator requires independent witness evidence and the appropriate set of checks for numerical KKT, exact optimality and physical claims. Missing evidence remains `unknown`, `not_evaluable` or `not_run`.

Run timing is split into model build, preprocessing, solve, verification and end-to-end. Time-to-qualified includes all five stages. Performance claims require pinned hardware and at least ten repetitions; report median, p95 and dispersion.
