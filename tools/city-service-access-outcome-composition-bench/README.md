# City service access and realized outcome composition reference v0.1

Standalone Rust reference oracle for #680. This is a read-only composition child: it does not replace CityMorph, the restoration/continuity owners, City In-Use Outcomes, the scope binder, or service-owner evidence.

## Core distinction

The evaluator reports four independent axes:

1. **Restoration:** the exact upstream restoration bridge state and stability qualification.
2. **Potential access:** the exact CityMorph group result. `AccessibleAndAdequatePotential` is only modeled opportunity/capacity under the declared network, mode, time, affordability, eligibility and access assumptions.
3. **Operational evidence:** the exact in-use owner metric result plus the corresponding raw-input metadata binding (evidence class, evidence kind, coverage, calibration and source receipt).
4. **Realized use and resident experience:** direct source-owned delivery receipts are separate from potential access and operational hours; consented resident experience is separate again.

An opening-hours receipt and a favorable network/capacity model do not prove that people received the service. Missing service-use data remains `not_collected`; it is not a zero. Survey non-response or consent denial is not a negative satisfaction result.

## Exact join

Every owner receipt must resolve through one `Bound` common-scope binder entry. Owner, contract/schema revision, receipt ID, source-scope digest, subject ID/digest, functional unit, beneficiary group, scenario, evidence state, rights state, issuer state and common-scope digest are matched. Ambiguous duplicate owner bindings fail closed.

The service frame pins its own source receipt, frame ID/digest, population count, demand quantity/unit, demand state and period. The CityMorph population group and result must match that frame exactly. Actual delivery records must use the same unit, frame and observation window. Zero demand is represented separately from unknown demand and is never used as a divisor.

## Frozen 30-case corpus

The corpus covers restored-but-potential-only, actual delivery, consented experience, consent denial, inaccessible and constrained routes, unknown access, non-restoration, unknown continuity, missing demand denominator, demand mismatch, revoked/restricted frames, scope/window/design mismatches, low operational coverage, evidence-class substitution, zero demand, positive use against a verified-zero frame, conflicting not-collected values, duplicate group output, duplicate source bindings and missing/non-representative evidence.

All fixtures are synthetic. The CLI binds the exact corpus and schema bytes and emits a receipt pinned to the checked-out source commit. Owner-input digests and receipts are treated as declared references; this tool does not re-open or authenticate their bytes.

## Run

~~~sh
cargo fmt --manifest-path tools/city-service-access-outcome-composition-bench/Cargo.toml --all -- --check
cargo test --locked --manifest-path tools/city-service-access-outcome-composition-bench/Cargo.toml
cargo clippy --locked --manifest-path tools/city-service-access-outcome-composition-bench/Cargo.toml --all-targets -- -D warnings
cargo run --locked --quiet --manifest-path tools/city-service-access-outcome-composition-bench/Cargo.toml -- \
  docs/research/city-service-access-outcome/CITY_SERVICE_ACCESS_OUTCOME_CORPUS_V0_1.json \
  docs/research/city-service-access-outcome/CITY_SERVICE_ACCESS_OUTCOME_CORPUS_V0_1.schema.json \
  --source-commit "$(git rev-parse HEAD)" > city-service-access-outcome-composition-receipt.json
~~~

## Claim ceiling

A passing result establishes synthetic cross-owner composition mechanics only. It does not prove live service availability or use, population representativeness, healthcare/clinical quality, resident satisfaction, statutory compliance, causal impact, professional approval or action authority.
