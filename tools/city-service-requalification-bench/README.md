# CityMaker service requalification reference v0.1

**Status:** synthetic-first state/composition oracle; exact-head qualification pending  
**Parent issue:** [CITY-SERVICE-REQUALIFICATION-001 #656](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/656)  
**Parent architecture:** [CityMaker owner-impact router PR #651](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/651)  
**Temporal semantics:** [CITY-TEMPORAL-REPLAY-001 #653](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/653)  
**Branch:** `research/city-service-requal-v0.1`

## Purpose

Compose three independent outcome families:

1. **Service state** — adequate, constrained, interrupted, inaccessible or unknown.
2. **Requalification state** — whether exact required owner obligations are open, blocked, invalid or discharged by matching receipts.
3. **Restoration state** — whether a plan is feasible, work is running, inspection/commissioning is pending, measured outcomes are incomplete, or restoration is demonstrated for the declared scope.

A feasible plan is not completed work. A completed work order is not inspection. Commissioning is not measured service adequacy. A measurement that omits a required population group cannot demonstrate restoration for that group.

This crate composes declared owner reports; it does not calculate service thresholds, simulate physical networks, authenticate report bytes, certify engineering, prove that the source-owner set is globally complete, or authorize construction/operation.

## Contract

The query binds:
- exact change event ID/digest and native subject/revision;
- world-valid, knowledge, trust and evaluation cutoffs;
- historical-known-then versus currently-reconstructed-history perspective;
- district/jurisdiction scope and functional unit;
- required population groups;
- expected service owner, owner repository, contract/schema/runtime revisions and report kind.

A service report is accepted only as `contract_bound_unverified` when its owner/contract/event/subject/scope/time bindings match. Report payload bytes and owner authentication remain explicitly unverified.

Requalification obligations discharge only through a receipt bound to the exact obligation, owner, event, asset revision, scope, functional unit and evaluation time.

Restoration demonstration requires exact inspection and commissioning receipts plus a valid, sufficiently sampled outcome measurement covering every required population group and a live measurement interval. This is a composition gate, not a substitute for professional acceptance.

## Corpus

`CITY_SERVICE_REQUALIFICATION_CORPUS_V0_1.json` contains 26 frozen synthetic cases covering:
- unaffected service;
- feasible but unexecuted and infeasible plans;
- completed work awaiting inspection or commissioning;
- commissioning without measured outcomes;
- fully receipt-bound measured restoration;
- partial/constrained restoration;
- missing population coverage and missing owner reports;
- wrong event/scope/time/revision bindings;
- invalid or absent obligation discharge receipts;
- missing obligations;
- insufficient sample size and stale measurements;
- failed inspection and unknown plans;
- duplicate service reports and untrusted system time; late correction excluded from historical knowledge but included in current reconstruction of the same earlier world time.

Expected output records service, requalification and restoration states separately. The corpus is synthetic; no physical city performance is implied.

## Run

From the repository root:

    cargo fmt --manifest-path tools/city-service-requalification-bench/Cargo.toml --all -- --check
    cargo test --locked --manifest-path tools/city-service-requalification-bench/Cargo.toml
    cargo clippy --locked --manifest-path tools/city-service-requalification-bench/Cargo.toml --all-targets -- -D warnings
    cargo run --locked --quiet --manifest-path tools/city-service-requalification-bench/Cargo.toml -- \
      docs/research/city-service-requalification/CITY_SERVICE_REQUALIFICATION_CORPUS_V0_1.json \
      --source-commit <40-lowercase-hex-sha>

The CLI emits a receipt binding exact corpus bytes and source commit. Replace the example SHA with the exact commit under evaluation.

## Future owner integrations

The first integration candidates are:
- Sol-Atlas #574 service continuity evidence;
- Sol-Atlas #642 restoration-feasibility output;
- Sol-Atlas #644/#651 impact frontier and owner routing;
- Symtropy #4181 service-cassette inspection/manufacturing evidence.

Owner adapters must supply their actual service-state, inspection, commissioning and measurement contracts before this synthetic reference can be treated as a live operational consumer.

## Claim boundary

A PASS means only that the evaluator reproduces the 26 synthetic state-composition expectations. It does not establish source authenticity, legal rights, service-owner completeness, engineering adequacy, physical safety, successful work execution, field commissioning, actual service restoration or authority to reopen/operate an asset.
