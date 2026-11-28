# Tiered Spatial Validation Contracts v0.1

**Status:** synthetic split-integrity oracle; not a model-performance result.

This benchmark turns the validation-regime label into a machine-checkable precondition. It accepts a frozen protocol digest and explicit train/evaluation observation IDs, then verifies that the partition satisfies its declared tier before any model metrics may be scored.

## Tiers

| Tier | Contract | Maximum claim label |
|---|---|---|
| T0 | Random-mixture diagnostic, no observation identity overlap | Within-mixture interpolation diagnostic only |
| T1 | Random control with training-set size matched to a declared spatial split | Same as T0; controls for training-size effects |
| T2 | Coordinate-buffered holdout; projected CRS and metre units declared; observed minimum train/evaluation distance meets threshold | Within-region spatial separation only |
| T3 | Graph-community IDs are disjoint between train and evaluation | Within-region graph-community separation only |
| T4 | Administrative units are disjoint | Administrative-unit transfer only |
| T5 | Regions and campaigns are both disjoint | External region and campaign transfer challenge |

The auditor also requires:
- unique observation identifiers and no train/evaluation identity overlap;
- every partition identifier resolves to a registered observation;
- SHA-256-shaped protocol and dataset digests;
- a protocol sealed before model scoring;
- deterministic output independent of case and split-array ordering.

## Run

\`\`\`sh
cargo fmt --manifest-path reference/Cargo.toml -- --check
cargo test --manifest-path reference/Cargo.toml
cargo clippy --manifest-path reference/Cargo.toml --all-targets -- -D warnings
cargo run --manifest-path reference/Cargo.toml -- fixtures/validation-regimes-v0.1.json
\`\`\`

## Boundaries

This is a split-integrity auditor, not a fold generator, spatial dependence estimator, model trainer, or CRS transformation engine. T2 uses Euclidean distance on already projected coordinates in a declared metre-based CRS; CRS correctness and transformation provenance must be established upstream. A T5 pass validates only the partition metadata contract; it does not prove that an external sample is representative, metrologically sound, statistically sufficient, or independent in every causal sense.

A validation score must always be reported with its tier. A random-fold score cannot be relabelled as external transfer. Do not interpret T0–T5 as a single monotone score: each answers a different deployment question. T2/T3 are within-region separation diagnostics; only T4/T5 can support claims about jurisdiction or external campaign transfer, and the exact scope of those claims must remain explicit.

## Research reference

Jiang, Wu & Liu (2026), [A tiered spatial validation workflow for geographic prediction models](https://doi.org/10.1016/j.mex.2026.104072), motivates the tiered regimes, matched-size control, graph-community splits, administrative holdouts and external regional stress test. The reported Baltimore experiment is a paper-specific result, not a Sol-Atlas benchmark score.
