# Earth Interior Observatory benchmark receipt verifier

This standalone Rust utility is deliberately separate from the inference/benchmark code. It validates a receipt against the local Draft 2020-12 schema, recomputes SHA-256 for declared artifacts and the benchmark specification, checks the exact Git HEAD and worktree cleanliness, and evaluates fail-closed qualification requirements.

It does **not** establish that a geophysical model is physically correct. It verifies protocol, provenance and artifact integrity only.

## Build and test

```sh
cargo test --manifest-path tools/geo-receipt-verifier/Cargo.toml
cargo clippy --manifest-path tools/geo-receipt-verifier/Cargo.toml --all-targets -- -D warnings
```

## Verify an executed but not yet promoted receipt

Use **two clean checkouts**:

- `EVALUATED_ROOT`: the exact code commit being benchmarked, with a clean worktree.
- `VERIFIER_ROOT`: a separate checkout containing the verifier implementation, with a clean worktree and a distinct commit SHA. It may be a reviewed verifier commit from the same public repository or a separate verifier repository. It must not be the same directory or the same commit as the evaluated checkout.

The verifier cross-checks this distinct identity, both worktree states, and the verifier repository origin. This prevents a receipt from qualifying by treating the code-under-test's own checkout as an independent verifier. It is still a provenance control, not proof that an independent human organization reviewed the code or that retained provider JSON is authentic.

Receipt/schema paths are relative to `EVALUATED_ROOT`; artifact locators are relative to `--artifact-root`. Both roots are canonicalized and traversal/symlinks escaping the declared roots are rejected.

```sh
EVALUATED_ROOT=/absolute/path/to/evaluated-clean-checkout
VERIFIER_ROOT=/absolute/path/to/separate-verifier-clean-checkout
ARTIFACT_ROOT=/absolute/path/to/frozen-artifacts

cargo run --quiet \
  --manifest-path "$VERIFIER_ROOT/tools/geo-receipt-verifier/Cargo.toml" -- \
  "$EVALUATED_ROOT" \
  docs/earth-interior/benchmark-run-receipt.schema.json \
  /absolute/path/to/provisional-receipt.json \
  --artifact-root "$ARTIFACT_ROOT" \
  --verifier-root "$VERIFIER_ROOT" \
  --target-status qualified_synthetic > /tmp/geo-verification-report.json
```

The provisional receipt should have `qualification_status: "executed_unqualified"`. The report records the qualification-core digest, target status, separate verifier HEAD/origin/worktree status, individual checks, and outcome. Keep the report outside both checkouts so creating it does not dirty either worktree. The report is a proposal for promotion and still requires an independent review of the recorded provider/run evidence.

After that review accepts the report, the final receipt's `qualification_evidence` records the report SHA-256 plus the exact verifier repository URL and commit. The final receipt must then be verified with the **same clean verifier checkout at the same exact commit**, using `--verifier-report`:

```sh
cargo run --quiet \
  --manifest-path "$VERIFIER_ROOT/tools/geo-receipt-verifier/Cargo.toml" -- \
  "$EVALUATED_ROOT" \
  docs/earth-interior/benchmark-run-receipt.schema.json \
  /absolute/path/to/final-receipt.json \
  --artifact-root "$ARTIFACT_ROOT" \
  --verifier-root "$VERIFIER_ROOT" \
  --verifier-report /tmp/geo-verification-report.json
```

The verifier's qualification-core digest uses compact JSON serialization with deterministic object-key ordering, removes `qualification_evidence`, and normalizes `qualification_status` to `executed_unqualified`. This makes the digest stable across the provisional and attested receipt without a self-referential hash cycle.

## Exit codes

- `0`: structurally and integrally verified; when a promoted status is requested, its gates were eligible and any supplied verifier report checked out.
- `1`: invalid schema/receipt or artifact-integrity failure.
- `2`: requested qualification is not eligible or its attestation is incomplete/invalid.
- `64`: command-line usage error.

A zero exit code for `executed_unqualified` means the receipt was processed correctly; it does not mean the benchmark passed.

## Trust boundary

- No runtime network access is used for schema references; the schema and artifacts are supplied locally.
- Artifact locators are relative to `--artifact-root`; path traversal and symlinks escaping that root are rejected.
- A queued, skipped, cancelled, missing, or stale-head workflow is never treated as passing evidence. Each required CI claim must reference a locally retained workflow-run JSON artifact; its run ID, name, head SHA, status, conclusion, and URL must agree with the receipt.
- All declared gates and controls must pass for promotion.

- The selected benchmark specification declares `required_ci_checks`. The verifier requires the receipt check list to contain exactly that set—no omitted or duplicate check names/IDs—and then verifies each retained workflow-run JSON, preventing the receipt from shrinking its own required CI gate.
- A promoted status requires explicit required-CI evidence: every listed check must be `completed` with conclusion `success` on the exact code commit. A pending, queued, skipped, cancelled, missing, stale-head, or failed check is never a pass.
- Promotion requires metric rows with sample counts and an explicit uncertainty method; `none` is not sufficient.
- Synthetic qualification requires both a withheld truth-state artifact and synthetic observations, plus passing null-signal and truth-leakage controls.
- Source-adapter qualification requires hashed source payload bytes and passed unit, temporal-coordinate, and variable-shape audits.
- Real-observation evaluation requires a hashed holdout manifest, the actual observation artifact, and passed leakage and reference-overlap audits.
- CI JSON payloads are digest-checked and cross-compared offline; this does not cryptographically authenticate GitHub as the payload origin. Review the run URL/provider evidence independently.
- The utility does not authenticate a publisher and does not replace a separately reviewed verifier, domain review, or physical validation.

## Current implementation slice

The verifier now includes 14 authored regression tests for schema acceptance, canonical receipt digests, path traversal, Git URL normalization, strict gates/controls, exact-head CI evidence, metric uncertainty metadata, and synthetic truth visibility. A CI workflow was added for `cargo fmt --check`, tests, and Clippy. These commands have **not yet been executed in this environment**; the current feature-branch CI runs remain pending/queued until a runner reports results. Do not call the implementation Rust-qualified until the exact commit's workflow completes successfully.

The verifier checks retained workflow-run JSON offline against the receipt and the payload digest. It does not cryptographically authenticate that a JSON file originated from GitHub, so the linked run URL and provider provenance remain independently auditable. Receipt eligibility is a protocol/integrity check, not proof of geophysical truth.

## Calibration research addition

The model-falsification contract is now version 0.1.1 with lane `L6_data_conditioned_calibration`. It compares pooled simulation-based calibration to coverage and predictive scores stratified by predeclared observation regimes, tests cases where pooled coverage hides conditional under-coverage, and evaluates posterior-correction candidates only where valid known-truth calibration cases exist. It explicitly forbids presenting simulated posterior calibration as calibration proof for the real Earth's hidden core state. Research leads: [Posterior SBC (2026)](https://doi.org/10.1007/s11222-026-10825-9) and [Flow Matching Calibration for SBI under Model Misspecification (ICML 2026)](https://proceedings.mlr.press/v306/ruhlmann26a.html).
