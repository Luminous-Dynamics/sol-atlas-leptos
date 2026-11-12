# Highveld compound-disruption synthetic corpus v0.1

This directory contains a small, deterministic **synthetic-only** fixture set for [HIGHVELD-COMPOUND-DISRUPTION-CORPUS-001](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/735).

It is an input/known-answer corpus, not a new event clock, weather archive, resilience evaluator, building simulator, hazard forecast, probability model, or engineering approval. It is designed to feed the existing owners in Sol-Atlas #630/#694/#700 and Symtropy #1848/#1892/#4507/#4531.

## Files

- `HIGHVELD_COMPOUND_EVENT_V0.1.schema.json` — closed Draft 2020-12 JSON Schema.
- `SYNTHETIC_CORPUS_V0.1.json` — seven timeline scenarios plus nine adversarial evidence controls with explicit event windows, scopes, temporal resolutions, expected interval relations and one no-report/scope-mismatch control.

## Contract semantics

- Every case is `SyntheticControlledEvent`. No observation, outage frequency, local hazard occurrence, causal dependency or physical outcome is claimed.
- Event intervals are half-open: `[start_offset_s, end_offset_s)`. Equal end/start boundaries mean `meets`, not `overlaps`.
- Time is integer seconds from a scenario-local `t0`; there is no implicit local timezone, wall clock, daylight-saving conversion or external epoch.
- A temporal relation is not a causal relation. `semantic_links` are explicit declarations; they are never inferred from interval overlap.
- Scope is part of identity. A system-level record is not a feeder, dwelling, room or tap observation. Temporal interval relations and scope-join dispositions are separate: two records can overlap in time while remaining NOT_COMPARABLE.
- `no_report`, `unknown`, `censored` and `observed_normal` are distinct record states. In particular, `no_report` must never be promoted to `observed_normal` or “no outage”.
- A candidate architecture must receive the same exact exogenous scenario event bytes and declared information budget. Future events are hidden unless an explicitly labelled oracle arm is being evaluated.
- All quantities, event times and severities in this corpus are synthetic fixtures; do not reuse them as Highveld weather, outage durations, service thresholds or forecast probabilities.

## Fixture coverage

1. Hot day/night overlapping a grid interruption, water interruption and replacement-part delay.
2. Cold event overlapping a grid interruption and HeatBank fault, with a later repair delay.
3. Poor outdoor-air event overlapping ventilation restriction, grid interruption and network loss; manual fallback is declared separately and is not evidence of adequate ventilation.
4. Initial outage, restoration, repair-part unavailability and a repeated outage; the first outage and restoration `meet` at a half-open boundary.
5. A coarse system-level water-status record with a dwelling/tap `no_report`; the join is explicitly `NOT_COMPARABLE`.

## Adversarial controls

The companion top-level `adversarial_controls` list contains nine synthetic controls for timezone ambiguity, coarse/fine temporal resolution, right-censoring, conflicting observations, rights revocation, stale source revision, future-information leakage, unmatched candidate traces and unsupported common-cause claims.

These controls define conservative expected dispositions; they do not define real-world legal rights, universal temporal-alignment tolerances, empirical hazard probabilities or an authorized policy.

6. Heavy-rain stress overlapping grid interruption, drainage-pump fault and parts delay; entirely synthetic and not a site flood assessment.
7. Peak household service concurrency overlapping shared-plant fault, parts delay and a declared household-task change.

## Reproduction and qualification boundary

The expected interval relations are hand-authored known answers for a finite set. A future Rust consumer should independently recompute all timeline relations and adversarial-control dispositions and reject missing IDs, duplicate IDs, invalid/empty intervals, unknown event kinds, invalid resolutions, scope escalation, mismatched trace bytes and weakened expected dispositions.

The authoring environment did not contain `rustc` or `cargo`; this commit therefore does not claim Rust compilation, Rust tests, Clippy, hosted CI, event-source admission or any physical qualification. The JSON files are intended for structural parsing and review only until an exact-head executable validator passes.

## Next promotion gates

- Extend the corpus with independently authored hostile cases for timezone ambiguity, coarse/fine time resolution, censored endpoints, conflicting records, source revision changes, rights revocation, unsupported common-cause links and candidate information leakage.
- Bind real climate data only through Sol-Atlas #611/#593, utility evidence only with source/rights/scope qualification, and building measurements only through Mycelix #6025.
- Keep observed joint events, partial observations, synthetic stresses, future-climate scenarios and context-only records in separate lanes.
- Connect admitted scenario identities to Symtropy #1848; do not add a parallel timeline or recovery evaluator.

## Claim ceiling

Synthetic temporal-composition fixtures only. This corpus does not prove local event co-occurrence, causal dependencies, probabilities, building habitability, service continuity, comfort, safety, affordability, climate representativeness, code compliance or industry superiority.
