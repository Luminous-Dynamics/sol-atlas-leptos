# City Restoration Outcome Bridge Reference v0.1

Rust reference composition evaluator for the frozen 26-case HRO corpus.

## Owner boundaries

This evaluator composes the existing owner reports; it does not implement their domain evaluators again:

- CIV-SERVICE-001 (Symthaea): declared service, beneficiary, floor and measurement requirement.
- CITY-SERVICE-RESTORATION-001 (#646): feasibility, work execution, inspection, commissioning and accepted output.
- CITY-MORPH-ACCESS-001 (#574): interruption, service continuity, temporal trajectory, stable adequacy and recurrence.
- CITY-IN-USE-OUTCOMES-001 (#635): measured outcome, evidence class, calibration, applicability and consent-related boundaries.
- CITY-SCOPE-BINDING-001 (#624): common scope, functional unit, group, scenario and exact owner receipt binding.
- CITY-PANEL-ACCEPTANCE-001: physical output evidence, when required by the service/intervention contract.

The bridge does not create another service ontology or authority. It consumes the declared owner-native result states, preserves their identities, and reports the composition disposition and missing evidence.

## What a result means

RestoredForDeclaredScope means that all requirements represented by this synthetic corpus are met for the exact declared service/scope and current evidence cut. It does not mean the physical district or clinic has actually been restored. The corpus contains synthetic report fixtures only.

The bridge keeps these axes separate:
- plan readiness;
- work execution;
- inspection;
- commissioning;
- service continuity and stability;
- measured outcomes;
- physical output acceptance;
- common-scope comparability;
- resident consent/disclosure.

A hard mandatory floor cannot be compensated by another successful metric. A service that is adequate at one functional unit cannot be extrapolated to another. A later-ingested measurement cannot leak into historical replay. An absent resident survey because consent was not granted is not a negative service outcome. Panel acceptance does not substitute for building commissioning or sustained service.

The recovery trajectory is calculated only from the supplied #574 owner ticks. Unmet demand remains null and its denominator not_established unless a qualified owner provides the denominator.

## Strictness and claim ceiling

The evaluator rejects unknown root, case, input and selected nested fields. It checks owner receipt bindings, temporal visibility, issuer/right/evidence state, mandatory service and hard-floor failures, and exact expected outputs.

It does not authenticate the actual source owners, verify external payload bytes, recompute upstream owner results, assess local statutory or professional compliance, create an action authorization, establish causal impact, measure a physical building, or establish resident wellbeing.

Claim ceiling: synthetic_service_restoration_composition_only.

## Run

From repository root:

~~~sh
cargo fmt --manifest-path tools/city-restoration-outcome-bridge-bench/Cargo.toml --all -- --check
cargo test --locked --manifest-path tools/city-restoration-outcome-bridge-bench/Cargo.toml
cargo clippy --locked --manifest-path tools/city-restoration-outcome-bridge-bench/Cargo.toml --all-targets -- -D warnings
cargo run --locked --quiet --manifest-path tools/city-restoration-outcome-bridge-bench/Cargo.toml -- \
  docs/research/city-restoration-outcome-bridge/CITY_RESTORATION_OUTCOME_BRIDGE_CORPUS_V0_1.json \
  docs/research/city-restoration-outcome-bridge/CITY_RESTORATION_OUTCOME_BRIDGE_CORPUS_V0_1.schema.json \
  --source-commit "$(git rev-parse HEAD)" > city-restoration-outcome-bridge-receipt.json
~~~

Passing the frozen known-answer corpus establishes deterministic E1 synthetic composition mechanics only. E2+ requires actual owner-authorized data, independently validated scope binding and professional review. E3 requires a physical article and metrology. E4 commissioning and E5 longitudinal occupied service outcomes remain separate qualifications.
