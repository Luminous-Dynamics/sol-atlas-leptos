# Energy replanning policy bake-off v0.1

A dependency-free, synthetic-known-truth research instrument comparing when to request a replan under imperfect forecasts and changing observations.

This is intentionally a trigger-policy benchmark, not another optimizer, dispatch model, or actuator. The per-scenario execution matrix uses evaluator-only solve/audit outcomes. In addition, exact-head CI builds the standalone #351 schedule-auditor executable and runs a frozen H=2 accepted trajectory plus a mutated rejected trajectory through that separate process, verifying request/schedule SHA-256 and actual auditor-binary SHA-256 in both receipts. That CLI smoke-test is not yet wired to every policy-generated candidate schedule. It sits alongside the standalone schedule auditor and the broader rolling-horizon programme in issue #352.

## Paradigm

The comparison target is the decision to spend compute and operational switching cost on a new schedule:

1. fixed-cadence baseline;
2. forecast-residual trigger;
3. forecast-uncertainty trigger;
4. decision-value trigger (estimated avoidable service loss versus solver plus switching cost);
5. hybrid with a hard reserve-margin override.

All policies receive the same immutable, time-indexed observations. The policy interface receives only one current DecisionObservation; it does not receive a scenario, hidden future, event label, outcome, or evaluator truth. A common input-quality gate returns Abstain for stale, missing, or conflicting telemetry before any policy is invoked.

The evaluator has a separate ScenarioTruth object containing material-event observability time, response deadline, counterfactual service-loss values, solver completion outcomes and synthetic independent-auditor outcomes. It uses that truth only after policy decisions are fixed. A replan request counts as event recovery only when the solver completes by the response deadline and the synthetic audit outcome is Accepted. Missing/ambiguous execution receipts, timeout, infeasibility, cancellation, late completion, audit rejection or audit unavailability never count as a usable schedule.

## Frozen synthetic panel

The initial panel has 32 scenarios: four variants each of nominal operation, slow forecast drift, sudden outage, nuisance forecast spike, uncertainty shock, telemetry gap, high-value drift and heavy-tailed shock. Execution truth includes late completion, timeout, infeasibility, rejected audit and unavailable audit cases.

It is a deterministic authored/generative synthetic panel. It is not a real weather, load, tariff, asset, or grid trace.

## Metrics

The report keeps separate columns for scenario count, event count, timely detections, missed events, false triggers, redundant triggers, solver requests, completions, failures, late completions, audit rejections, audit unavailability, accepted replans, abstentions, synthetic service-loss units, replanning-cost units, total-penalty units and mean time-to-audited-usable-schedule.

There is no single scalar winner and no authority output. All loss/cost fields are dimensionless benchmark units, not kWh, MWh, currency or measured battery degradation. The total penalty is a synthetic unit-comparable accounting field for this panel only; policy comparison should also inspect the underlying service-loss and compute/switching components.

## Independent auditor CLI qualification in exact-head CI

The workflow uses the standalone auditor as a separate executable. It copies the auditor package to a temporary source tree, normalizes formatting in that copy only, runs locked tests/Clippy and builds the release CLI with `CARGO_TARGET_DIR=target/energy-replan-policy-bakeoff/auditor-target`. It records both the original checked-out source file digests and the normalized build-copy digests, plus the toolchain and release-binary hash; the PR checkout is required to remain clean.

The auditor dependency graph is committed in `tools/energy-dispatch-schedule-auditor/Cargo.lock` (SHA-256 `0ab5e2fd068a53ece3a48b38e649a447190102043fb656967a1cae259639a23c`). The workflow asserts that digest before use and compiles with `--locked`.

The CLI smoke-test freezes these exact inputs:
- request SHA-256: `e1fc548c7d82e6aa9f95ace133f25c650df6f8c51018ee3a7bf16a980b8af804`;
- accepted H=2 schedule SHA-256: `fbeef792114c62eb516b8e0d92fd771f71932670a4d2249a11d0ced1f8a39397`;
- mutated H=2 schedule SHA-256: `6c43b474312ab870e65fbdb9e22093f0e78aaca53337fcee35626bf7c60058df`.

Both returned receipts must bind the exact request/schedule digests, schema/case identity and actual built auditor-binary SHA-256. This is a fixed known-answer and mutation smoke-test, not yet a run of the auditor on each policy-generated dispatch candidate.

## Claim ceiling

Passing this model establishes only that the typed observation boundary and finite synthetic benchmark behave as specified, and that the separate auditor CLI accepts its frozen H=2 known answer and rejects its frozen mutation with exact receipt bindings. It does not establish per-policy-candidate auditor integration, optimal dispatch, physical/electrochemical battery behavior, real-site resilience, forecast calibration, real auditor implementation correctness beyond the tested profile, safety certification, an operator's authority, or permission to actuate hardware. Every proposed/executed schedule in a future integrated runner must still be submitted to the independent trajectory auditor.

## Run

```sh
cargo fmt --manifest-path tools/energy-replan-policy-bakeoff/Cargo.toml --check
cargo check --locked --all-targets --manifest-path tools/energy-replan-policy-bakeoff/Cargo.toml
cargo test --locked --all-targets --manifest-path tools/energy-replan-policy-bakeoff/Cargo.toml
cargo clippy --locked --all-targets --manifest-path tools/energy-replan-policy-bakeoff/Cargo.toml -- -D warnings
cargo run --release --locked --manifest-path tools/energy-replan-policy-bakeoff/Cargo.toml
```
