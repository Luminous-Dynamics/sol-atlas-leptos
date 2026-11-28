# Correlated spatial observations and EVSI benchmark v0.1

**Status:** synthetic reference case; no field performance claim.

## Research question

What happens when two surveys are treated as conditionally independent even though they observe the same underlying signal? Multiplying marginal likelihoods then double-counts correlated evidence, exaggerates posterior certainty, and can overstate the value of acquiring both observations.

This benchmark provides an exact joint distribution over all binary observation patterns for each hypothesis. The reference evaluator marginalizes unselected observations to score a selected survey portfolio. It also computes the incorrect conditional-independence result as a negative comparator.

## Synthetic oracle

There are two competing hypotheses and two candidate surveys with identical marginal likelihoods. The two survey outcomes are perfectly correlated conditional on either hypothesis.

- Baseline optimal action expected utility: 60.0.
- Exact joint-model expected utility after both surveys: 80.0 (gross EVSI 20.0).
- Naive conditional-independence expected utility after both surveys: 83.2 (gross EVSI 23.2).
- Independence assumption overstates gross EVSI by 3.2 utility units (16% of the true gross EVSI).
- Under the two-unit budget and equal costs, the planner selects one survey; its net EVSI is 19.9. Buying both adds no decision value under the exact joint model and incurs an additional cost.

This is a constructed counterexample. It is intended to verify probability handling, not to describe actual sensor correlation.

## Model

For (m) surveys, each hypothesis supplies a complete conditional distribution over (2^m) binary outcome patterns. Probabilities must be finite, within [0,1], cover every pattern exactly once and sum to 1. The pattern's character positions use lexicographically sorted survey IDs, making serialization order irrelevant.

For a selected subset, the evaluator sums over all full outcome patterns consistent with the selected observations. This is exact marginalization; it does not assume that unselected outcomes are missing at random beyond the supplied joint distribution.

## Tests

- Correlated observations do not receive duplicated information value.
- Naive independence overstates EVSI in the registered counterexample.
- Candidate ordering does not change the result when joint pattern positions are permuted accordingly.
- Non-normalized, incomplete and malformed joint distributions fail closed.

## Limits

- Binary observations and a complete joint outcome table.
- Exact enumeration; maximum 16 candidate surveys.
- Supplied priors, utilities and costs are not inferred or calibrated.
- The sequential policy evaluator is limited to eight candidates and currently has an exact binary-outcome joint distribution; it is a small oracle rather than a scalable production solver.
- Joint likelihoods must be learned, elicited or estimated from independent campaign data before operational use.

## Run

```sh
cargo test --manifest-path reference/Cargo.toml
cargo run --manifest-path reference/Cargo.toml -- fixtures/correlated-case.json
```
