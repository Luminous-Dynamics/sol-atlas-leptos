# Highveld compound-disruption evidence and benchmark v0.1

This directory contains a deterministic synthetic event corpus, adversarial controls, a cross-tool household-resilience benchmark contract, an industry comparison and a South African source-readiness register for [HIGHVELD-COMPOUND-DISRUPTION-CORPUS-001 #735](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/735).

It is an evidence/benchmark package, not a new event clock, weather archive, resilience evaluator, building simulator, hazard forecast, probability model, physical measurement platform, or engineering approval.

## Files

- `HIGHVELD_COMPOUND_EVENT_V0.1.schema.json` — closed Draft 2020-12 schema for the synthetic event corpus.
- `SYNTHETIC_CORPUS_V0.1.json` — seven deterministic timeline scenarios and nine adversarial evidence controls.
- `HIGHVELD_BENCHMARK_CONTRACT_V0.1.json` — machine-readable benchmark arms, matched-comparison requirements, owner crosswalk, scorecards, metrics and stage gates.
- `INDUSTRY_COMPARISON_V0.1.md` — current public capability comparison for Forma, TestFit, EnergyPlus, ASHRAE 140, BOPTEST and building semantics; proposed differentiation and fair head-to-head design.
- `SOUTH_AFRICA_SOURCE_READINESS_V0.1.md` — South African climate, electricity, water and household-survey source readiness, evidence classes and acquisition gates.
- `SERVICE_EVENT_HANDOFF_V0.1.md` — proposed evidence-to-service-event projection boundary; prevents upstream utility/hazard/component events from being mistaken for household service loss/restoration.
- `SERVICE_EVENT_HANDOFF_CONTROLS_V0.1.schema.json` and `SERVICE_EVENT_HANDOFF_CONTROLS_V0.1.json` — closed schema and 12 synthetic known-answer projection controls with 13 per-service expected dispositions and explicit observation-availability time.

- `B1_B2_PAIRED_EXPERIMENT_PROTOCOL_V0.1.md` — preregistration, blinded B1/B2 workflow, matched-input invariants, independent-oracle rules, acceptance gates and proposed neighborhood-scale architecture boundary.
- `B1_B2_PAIRED_RUN_PACKET_V0.1.schema.json` — closed run-packet schema for matched specialist-engine pairs, composition receipts, per-service decisions and oracle expectations.
- `src/bin/highveld-paired-audit.rs` — Rust CLI that checks paired engine identity, receipt bindings, per-service oracle agreement, hard-floor false passes and future-information leakage. It does not run specialist tools or implement the native Symtropy service evaluator.

## Synthetic corpus semantics

- Every event scenario is `SyntheticControlledEvent`. No observation, outage frequency, local hazard occurrence, causal dependency or physical outcome is claimed.
- Event intervals are half-open: `[start_offset_s, end_offset_s)`. Equal end/start boundaries mean `meets`, not `overlaps`.
- Time is integer seconds from a scenario-local `t0`; there is no implicit local timezone, wall clock, daylight-saving conversion or external epoch.
- A temporal relation is not a causal relation. Semantic links are explicit declarations; they are never inferred from interval overlap.
- Scope is part of identity. A system-level record is not a feeder, dwelling, room or tap observation. Temporal interval relations and scope-join dispositions are separate: two records can overlap in time while remaining `NOT_COMPARABLE`.
- `no_report`, `unknown`, `censored` and `observed_normal` are distinct record states. A missing notice or missing record is not normal service.
- Every paired candidate must receive the same exact exogenous trace identity/digest, initial state and information budget. Future events are hidden except in a separately labelled oracle upper bound.
- All times, quantities and severities in the corpus are synthetic; do not reuse them as Highveld weather, outage durations, service thresholds or forecast probabilities.

## Benchmark paradigm

The machine-readable contract defines five arms: documented manual workflow (B0), incumbent specialist toolchain (B1), the same incumbent engines plus Luminous evidence composition (B2), the best disclosed hybrid (B3), and a finite exhaustive oracle for tiny synthetic cases only (B4).

The intended competitive hypothesis is **not** that Luminous replaces Forma, TestFit, EnergyPlus, BOPTEST or ASHRAE 140. It is that evidence composition can improve cross-tool decision integrity: source scope and rights remain visible; revisions invalidate only dependent claims; compound service stresses are matched; resident-defined service floors remain non-compensatory; and decisions can be replayed from exact inputs and receipts.

No universal resilience scalar may hide per-resident hard-floor failures. Physical outputs remain unknown until the relevant model, instrument and reference-method owners qualify them.

## Evidence-to-service event boundary

An infrastructure outage is not automatically a household-service outage. Symtropy #1848/#1849 remains the only owner of native service transitions and trajectories. The handoff document records the currently inspected native event shape (`id`, `generation`, `tick`, `sequence`, `kind`; `kind` is `lost` or `restored`) but deliberately does not guess the identity, tick or sequencing rules. A service transition requires a qualified named-service outcome and owner-approved clock mapping; upstream events stay in the evidence crosswalk.

## Existing owner boundaries

- Sol-Atlas #607/#630/#611/#593: reference dwelling, Highveld stress binding and climate-source admission.
- Sol-Atlas #694: household-service definitions and architecture candidates.
- Sol-Atlas #735/#736: compound-event evidence and synthetic fixtures.
- Symtropy #1848/#1849: deterministic event clock, paired policies, service trajectories, recovery and Pareto decisions.
- Sol-Atlas #700/#633: service outcome binding and read-only post-occupancy outcomes.
- Mycelix #6025: building instrument-channel admission.
- Symtropy #1846: evidence validity and scoped invalidation.
- Symtropy #1892/#1922: finite-state decision impact and adaptive measurement policies.

Do not implement a second timeline, service evaluator, thermal solver, weather database, instrument passport, post-occupancy evaluator or generic value-of-information engine in this package.


## B1/B2 paired-run audit

The native CLI validates a submitted paired-run packet; it does not execute Forma, TestFit, EnergyPlus, BOPTEST or Symtropy, and it does not establish physical performance.

Run the built-in synthetic harness self-test:

```sh
cargo run --bin highveld-paired-audit -- --self-test
cargo test --bin highveld-paired-audit
```

Audit an externally generated packet:

```sh
cargo run --bin highveld-paired-audit -- --input paired-run.json
```

The audit requires identical specialist engine versions and raw engine input/output digests across B1/B2, checks B2 receipt binding and per-service rights/scope states, compares B2 decisions with the independent oracle, and rejects future information or hard-floor false passes. B1 oracle mismatches are reported as baseline outcomes rather than treated as packet-integrity failures. A PASS means only that this submitted synthetic/experiment packet satisfies the audit contract; it does not mean the industry experiment has been run or that the physical model is qualified.

## Validation status

The event corpus at original implementation head `6205f9ad96303a97a03a1ecb99261b040f092664` passed exact-head hosted Check run [#7105](https://github.com/Luminous-Dynamics/sol-atlas-leptos/actions/runs/43183337599): `cargo fmt --all --check`, `cargo check --target wasm32-unknown-unknown`, and `cargo test --bin sol-atlas-leptos`.

The benchmark, industry/source-readiness and service-handoff documentation commits are separate additions after that green corpus head. Their final exact-head workflow status must be checked before treating the expanded PR as green. This package itself still adds no Rust validator or physics code. Structural checks on the corpus do not qualify empirical sources or physical models.

## Promotion gates

1. Keep observed joint, observed partial, synthetic, future-climate and context-only evidence lanes separate.
2. Admit real climate inputs only through Sol-Atlas #611/#593; admit utility records only with source/rights/scope/coverage evidence; admit physical measurements only through Mycelix #6025.
3. Translate service loss/restoration only through the exact owner-native Symtropy #1848 interface. Preserve source evidence separately; never pass raw hazard or utility notices as if they were measured service events.
4. Run the paired comparison with identical geometry, task requirements, exogenous trace, initial state, resource budget and information budget.
5. Promote physics claims only after applicable Standard 140 capability tests, model-specific VVUQ, held-out reference observations and risk-proportionate review.
6. Use the unoccupied Symtropy #4531 test cell only after instrument admission, independent reference methods and a safety-reviewed protocol.

## Claim ceiling

This package supports synthetic event-composition semantics, benchmark design and source-acquisition planning. It does not prove observed South African joint events, event probabilities, causal dependencies, household service delivery, habitability, comfort, safety, health protection, affordability, climate representativeness, code compliance, physical resilience or industry superiority.
