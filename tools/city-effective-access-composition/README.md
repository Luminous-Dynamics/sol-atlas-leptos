# City Effective Access Composition Reference v0.1

This crate composes synthetic owner-output fixtures. It does not route, calculate capacity, ingest meters, authorize work, or infer individual service receipt.

## Non-substitution contract

- CityMorph reachable capacity is a potential opportunity, not actual delivery.
- Restoration qualification does not prove that a service is reachable for every beneficiary group.
- An in-use service receipt is evidence for the declared operational metric, not proof of visits or outcomes beyond that metric.
- Beneficiary delivery is a separate optional owner lane. Missing delivery evidence is NotProvided, never zero.
- Resident experience and disclosure are separate from the technical service disposition.

The composition requires service, functional-unit, subject, scenario, beneficiary group, population-frame, demand, common-scope, operating-window and observation-window bindings to match. Evidence authority and rights are checked independently of scope comparability.

## Corpus

The 28 frozen known-answer cases cover restoration failure/instability/revocation, potential access constraints, unknown and inaccessible routes, operational service failure, source and scope mismatch, demand/frame mismatch, delivery missing/stale/below-demand, an aggregate-versus-subgroup counterexample, and consent-denied experience evidence.

All fixtures are synthetic. They do not qualify Johannesburg services, public delivery, resident coverage, equity, causal impact, regulatory compliance or deployment readiness.

## Run

```sh
cargo fmt --manifest-path tools/city-effective-access-composition/Cargo.toml --all -- --check
cargo test --locked --manifest-path tools/city-effective-access-composition/Cargo.toml
cargo clippy --locked --manifest-path tools/city-effective-access-composition/Cargo.toml --all-targets -- -D warnings
cargo run --locked --quiet --manifest-path tools/city-effective-access-composition/Cargo.toml -- \
  docs/research/city-effective-access-composition/CITY_EFFECTIVE_ACCESS_COMPOSITION_CORPUS_V0_1.json \
  --source-commit "$(git rev-parse HEAD)"
```

## Input hardening

The command-line receipt path validates original JSON bytes before map creation: duplicate decoded keys (including escaped spellings) and trailing content are rejected. Corpus validation rejects unknown nested fields and missing required fields. These checks protect the synthetic oracle contract; they do not replace source-owner signature verification or real data-rights review.

## Current owner-integration boundary

The access fixture now uses CityMorph's native `reason_codes` field. However, CityMorph #564 currently exposes group-level capacity potential plus accessible/unknown service ID lists; it does not yet emit a complete service-scoped opportunity row for a named facility. The `opportunity.service_id` in this prototype is therefore an explicit synthetic binding, not a claim that a direct owner-native per-facility receipt already exists.

Issue #686 defines the missing CityMorph owner output: a deterministic per-service opportunity row with route, availability, affordability, eligibility, step-free and capacity states. Until that owner contract is implemented and qualified, this PR remains a synthetic composition prototype rather than an executable CityMorph integration.
