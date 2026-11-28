# Decision-Critical Spatial Disagreement Benchmark v0.1

This benchmark compares admissible spatial hypotheses by their decision consequences, not merely by prediction distance or ensemble variance.

## Executable reference

The Rust reference consumes finite per-support-unit action-loss tables. For every support unit shared by every hypothesis, it:

1. derives each hypothesis's optimal action set;
2. identifies action disagreement separately from numeric prediction range;
3. selects a local minimax-regret action without assigning probabilities to hypotheses;
4. reports weighted regret under every admissible hypothesis;
5. excludes unknown and incomplete support from comparison instead of treating it as zero, safe, or agreement.

The frozen fixture contains counterexamples in both directions:

- identical scalar predictions but different optimal actions;
- large prediction differences but an identical optimal action;
- a narrow, low-area corridor whose consequence-weighted regret remains visible;
- partial and explicitly unknown support.

Run:

\`\`\`sh
cargo fmt --manifest-path reference/Cargo.toml -- --check
cargo test --manifest-path reference/Cargo.toml
cargo clippy --manifest-path reference/Cargo.toml --all-targets -- -D warnings
cargo run --manifest-path reference/Cargo.toml -- fixtures/disagreement-cases-v0.1.json
\`\`\`

## What this reference does not claim

This is a finite synthetic action-loss oracle. It does not parse geometry, infer topology, calculate predictions from rasters, estimate hypothesis probabilities, establish causal completeness, calculate acquisition EVSI, or validate stakeholder utility. The action-loss tables are supplied inputs. Real integration must source them from qualified spatial functionals (#991), robust decision evaluation (#987), calibrated likelihoods (#989), and governed evidence lineage (#983 / Mycelix). Correlated and sequential observation evaluation remains in #985.

## Benchmark metrics

Primary: comparable-support coverage, action-disagreement support mass, model-specific optimal action sets, local minimax-regret action, weighted regret by hypothesis, and worst local/aggregate regret.

Diagnostic only: weighted prediction range. Prediction disagreement is never substituted for decision disagreement.

## Qualification ladder

Synthetic finite oracle → replay against real provider outputs → independently metrologized multi-site evaluation → preregistered external comparison. A workflow pass qualifies the executable fixture only; it does not qualify field performance or establish an industry SOTA claim.
