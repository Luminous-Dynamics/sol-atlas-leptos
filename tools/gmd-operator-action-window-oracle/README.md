# GMD operator action-window oracle

A dependency-free Rust evaluator for a **supplied candidate mitigation action**. It estimates whether approval, preparation, mobilisation and execution can fit inside an asset/region impact-arrival window, while preserving separate evidence, authority, resource and scope gates.

## Core distinction

Timing feasibility is not authorization.

The result echoes all four precondition states and includes `execution_authorized: false`. It preserves every blocker even when the summary readiness value uses a priority order.

The result has two independent dimensions:
- readiness: ready for operator review, awaiting approval, blocked, stale or indeterminate;
- timing: conservative fit, arrival-dependent/at-risk, or missed window.

A candidate may be timing-feasible while still awaiting a human decision. The evaluator never creates a switching, dispatch, protection, load-shed, public-warning or crew-dispatch command.

## Time semantics

All times are elapsed seconds from a declared decision reference instant t0:
- earliest/latest impact arrival are forecast bounds relative to t0;
- stage durations include approval, preparation, mobilisation and execution;
- conservative deadline = earliest arrival minus safety buffer;
- latest safe deadline = latest arrival minus safety buffer.

Classification:
- ConservativeFit: maximum completion duration <= conservative deadline.
- ArrivalDependentAtRisk: conservative fit fails, but minimum completion can still fit before latest safe deadline.
- MissedWindow: minimum completion cannot fit before latest safe deadline.

No midpoint substitution is permitted. A wide event-arrival interval remains visible as an at-risk region.

## Preconditions

- Stale evidence -> StaleEvidence.
- Pending human authority -> AwaitingHumanApproval.
- Denied authority -> BlockedByAuthority.
- Unavailable resources -> BlockedByResources.
- Scope conflict -> ScopeConflict.
- Any unknown evidence/authority/resource/scope -> Indeterminate.

An Approved action must have zero remaining approval duration. A successful readiness state is only ReadyForOperatorReview, not authorization to execute. A candidate with zero total maximum completion duration is rejected rather than treated as an instantaneous fit.

## Cost and action class

Operational, maintenance and capital action classes are preserved for stratified analysis. Cost bands and reversibility are descriptive, owner-defined fields; this crate does not compare monetary values across operators or optimize a financial score.

## Machine-readable example

- Input schema: `docs/research/gmd-action-window/GMD_ACTION_WINDOW_INPUT_V0.1.schema.json`.
- Sample input: `docs/research/gmd-action-window/GMD_ACTION_WINDOW_SAMPLE_INPUT_V0.1.json`.
- Output schema: `docs/research/gmd-action-window/GMD_ACTION_WINDOW_RESULT_V0.1.schema.json`.
- Expected result: `docs/research/gmd-action-window/GMD_ACTION_WINDOW_SAMPLE_RESULT_V0.1.json`.

The example has 1,800–3,900 seconds of completion time, a conservative deadline of 3,000 seconds and latest safe deadline of 4,800 seconds. It is therefore ArrivalDependentAtRisk and AwaitingHumanApproval. Execution authorization is false.

## Run

    cargo fmt --manifest-path tools/gmd-operator-action-window-oracle/Cargo.toml -- --check
    cargo test --manifest-path tools/gmd-operator-action-window-oracle/Cargo.toml --locked --offline
    cargo clippy --manifest-path tools/gmd-operator-action-window-oracle/Cargo.toml --locked --offline --all-targets -- -D warnings

CI runs a Python-standard-library preflight and independent sample arithmetic check; this is not production Python or a full JSON Schema validator.

## Claim ceiling

P0 proves deterministic time-bound arithmetic and precondition disposition only. It does not validate forecast accuracy, action effectiveness, resource sufficiency, legal/operational authority, electrical feasibility, operator acceptance or service outcomes. Those require qualified forecast/physical models, owner procedures and independently observed exercises.
