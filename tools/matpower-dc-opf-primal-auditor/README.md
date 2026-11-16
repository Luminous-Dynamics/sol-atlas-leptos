# MATPOWER 8.1 DC-OPF Primal Auditor v0.1

A Rust-only independent auditor for candidate solutions produced by MATPOWER 8.1. It does not optimize and it does not treat the solver's success flag as a feasibility certificate.

## What is independently checked

For the exact original MATPOWER source bytes and a separately produced TSV receipt, the auditor checks:
- source Git blob SHA-1, MATPOWER version, model profile and system base MVA;
- complete bus, generator and branch records with stable source row identities;
- generator dispatch against Pmin/Pmax;
- each branch's DC flow equation, including tap ratio and phase shift;
- both reported branch-end active-power flows;
- every bus's active-power balance;
- branch Rate A and angle-difference limits;
- a separately recomputed model-2 polynomial generation-cost objective against MATPOWER's reported objective.

Initial numerical tolerances are explicit in AuditOptions. A candidate that violates a constraint returns a structured report with each violation; malformed receipts, unsupported source profiles and identity mismatches fail closed.

## Model profile firewall

The accepted profile is matpower_8_1_makeBdc_dcopf. It uses the MATPOWER makeBdc convention: branch susceptance is based on 1/x, with transformer taps and phase-shift injections. The auditor rejects a receipt that claims a different convention.

Do not compare this profile directly against PGLib's published DC objective column without matching formulations. PGLib's published baselines are generated with PowerModels.jl, whose DCPPowerModel uses x/(r²+x²) and omits taps/phase shifts; PowerModels' DCMPPowerModel is the variant documented to match MATPOWER. The formulation choice is part of the benchmark identity.

## Scope and non-claims

v0.1 supports one model-2 generator cost row per generator, linear or convex quadratic costs, and zero startup/shutdown cost. It does not support piecewise-linear cost functions or non-convex polynomial objectives.

A passing report establishes **primal feasibility under the declared MATPOWER DC model and independent objective recomputation**. It does not prove optimality or a valid global lower bound. In particular, it is not a dual/KKT certificate, AC-OPF result, physical measurement, N-1 security proof or operational authorization.

A future optimality checker must independently validate the solver's dual multipliers, stationarity, complementarity and primal-dual gap for the exact convex model before any global-optimality claim is permitted.

## Run

With MATPOWER 8.1, its pinned MP-Opt-Model/MIPS/MP-Test dependencies and GNU Octave installed:

    cargo test --manifest-path tools/matpower-dc-opf-primal-auditor/Cargo.toml --locked --offline
    cargo build --manifest-path tools/matpower-dc-opf-primal-auditor/Cargo.toml --locked --offline
    octave-cli --quiet --no-gui --eval "addpath('tools/matpower-reference-differential'); run_matpower_dcopf('case.m', '/tmp/case.tsv', 'vendor/matpower', 'vendor/mp-opt-model', 'vendor/mips', 'vendor/mptest', 'SOURCE_GIT_BLOB_SHA1')"
    tools/matpower-dc-opf-primal-auditor/target/debug/matpower-dc-opf-audit case.m /tmp/case.tsv

The dedicated workflow executes MATPOWER 8.1 DC optimal power flow and audits PGLib IEEE 14, RTS 24 and IEEE 57 candidate receipts. It publishes the TSV and JSON receipts.

## Reference equations

For an in-service branch f -> t:

    tap_eff = (tap == 0) ? 1 : tap
    b = 1 / (x * tap_eff)
    Pf = baseMVA * b * (Va_f - Va_t - shift_rad)
    Pt = -Pf

The bus injection check is:

    sum(Pg at bus) - Pd - Gs - sum(branch active outflow) = 0

The objective is evaluated with Horner's method from the MATPOWER polynomial cost coefficients in descending-power order. The v0.1 cost profile is restricted to linear/convex-quadratic model 2.
