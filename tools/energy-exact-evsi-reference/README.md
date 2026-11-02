# Exact finite-state EVSI reference v0.1

A dependency-free Rust reference implementation of exact expected value of sample information (EVSI) for small decision problems.

## Why it exists

The temporal-mesh allocator in #408 currently ranks optional event windows by an authored conservative avoidable-loss lower bound divided by marginal interval count. That is a useful deterministic heuristic, but it is **not** Bayesian EVSI: it does not model prior state probabilities, observation likelihoods, posterior action changes or observation costs.

This crate supplies a small exact reference oracle so future mesh/sensing policies can be compared with the true decision value on a finite, fully declared model before attempting larger stochastic or continuous models.

## Mathematical contract

States (s) have prior probabilities (P(s)), actions (a) have losses (L(a,s)), and observation outcomes (o) have likelihoods (P(o\mid s)).

Prior Bayes risk:

[
R_0 = \min_a \sum_s P(s)L(a,s)
]

Expected posterior Bayes risk:

[
R_1 = \sum_o P(o)\min_a \mathbb{E}[L(a,S)\mid o]
]

Expected value of sample information:

[
\mathrm{EVSI}=R_0-R_1
]

Expected value of perfect information is calculated as an independent upper bound:

[
\mathrm{EVPI}=R_0-\sum_s P(s)\min_a L(a,s)
]

All probabilities are integer parts per million (ppm). Prior probabilities and each state's observation-likelihood row must sum exactly to 1,000,000. Joint probabilities are accumulated as ppm² and losses remain integer units. EVSI is therefore reported as an exact numerator over (10^{12}); no floating-point posterior normalization, rounding, Monte Carlo sampling or arbitrary scalarization occurs.

Observation cost is converted to the same ppm² denominator. A signal is recommended only when exact EVSI is at least its observation cost.

## Heuristic-vs-exact bake-off

Each candidate carries:
- the existing authored avoidable-loss lower-bound units;
- marginal compute cost units;
- observation acquisition cost in loss units;
- a finite observation-likelihood matrix.

The tool outputs two rankings:
1. lower-bound-per-compute ranking (the current heuristic family);
2. exact net-EVSI-per-compute ranking.

This exposes when a candidate has a high claimed loss bound but conveys little decision-relevant information. In the frozen two-state control, an uninformative signal with a higher authored lower bound is intentionally ranked above a lower-bound noisy signal by the heuristic, while exact EVSI ranks the informative signal first.

## Hard invariants

- Prior probabilities sum to exactly 1,000,000 ppm.
- Every state's observation likelihoods sum to exactly 1,000,000 ppm.
- Every matrix dimension and stable identifier is validated.
- Expected posterior risk cannot exceed prior Bayes risk.
- EVSI cannot exceed EVPI.
- A zero-probability observation is retained but has no posterior policy.
- Reordering states/actions does not change the stable lexical tie-break or EVSI.
- Integer accumulation is checked; exact density comparison avoids overflowing u128 cross-products.
- Decision ties use lexical action IDs rather than input order.

## Test panel

The crate has **20 deterministic tests** covering:
- perfect observation equals EVPI;
- uninformative signals have exactly zero EVSI;
- noisy observations fall between zero and EVPI;
- more informative symmetric signals cannot reduce EVSI;
- observation cost thresholding without rounding;
- zero-probability outcomes;
- state/action ordering invariance and deterministic tie-breaking;
- exact-ranking reversal of an intentionally misleading lower-bound heuristic;
- unprofitable signal demotion;
- malformed priors, likelihood rows, probabilities and matrices;
- duplicate identifiers and zero compute cost;
- u64-scale loss values and u128-safe ratio comparisons.

## Run

```sh
cargo fmt --manifest-path tools/energy-exact-evsi-reference/Cargo.toml --check
cargo check --locked --all-targets --manifest-path tools/energy-exact-evsi-reference/Cargo.toml
cargo test --locked --all-targets --manifest-path tools/energy-exact-evsi-reference/Cargo.toml
cargo clippy --locked --all-targets --manifest-path tools/energy-exact-evsi-reference/Cargo.toml -- -D warnings
cargo run --release --locked --manifest-path tools/energy-exact-evsi-reference/Cargo.toml
```

## Claim ceiling

This is an exact evaluator for a finite declared decision problem, not a calibrated model of Johannesburg load/PV/weather/outage uncertainty. It proves neither the prior nor the likelihoods are empirically correct. The next step is to replace synthetic distributions with timestamped, source-backed South African weather/load/outage/price evidence and then validate calibration and posterior decisions on held-out chronological periods.
