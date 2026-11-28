# Sol-Atlas active spatial investigation benchmark v0.1

**Status:** synthetic reference benchmark; no field-study performance claim.

This benchmark asks a decision-centric question: given competing spatial/heritage hypotheses, candidate observations, explicit likelihoods, action utilities and a finite survey budget, which observation portfolio should be acquired next?

It distinguishes:

- **Expected information gain (EIG):** how much an observation is expected to reduce uncertainty over hypotheses.
- **Expected value of sample information (EVSI):** how much an observation is expected to improve the best downstream decision before observation costs.
- **Net EVSI:** gross EVSI minus a declared cost expressed in the same decision-utility units.

EIG is reported as a diagnostic baseline, not the optimization objective. A highly informative observation can have zero decision value when it separates hypotheses that imply the same action.

## Included

- `fixtures/decision-case.json`: a fully synthetic two-action / three-hypothesis problem with four candidate observations.
- `reference/`: a Rust evaluator that enumerates feasible observation portfolios, computes exact expected posterior decision utility under conditional independence, subtracts explicit utility costs, and reports EIG rankings.
- Unit tests for the expected synthetic oracle, information-gain/decision-value divergence, candidate-order invariance, decision-equivalent hypotheses, prior normalization and likelihood bounds.

## Synthetic oracle

The prior best action is `preserve_in_situ` with expected utility 72.0. The exhaustive EVSI planner selects `stratigraphic_microsection`, yielding expected post-observation utility 79.125 and net EVSI 4.625 under a budget of three effort units.

The EIG-only ranking selects `ornamental_subtype_catalog` first (approximately 0.653 bits), while this observation leaves the optimal action unchanged and has zero gross EVSI in this fixture. This is a constructed counterexample, not an empirical comparison of survey methods.

## Assumptions and limits

- Candidate tests are conditionally independent given the hypothesis.
- Each test has a binary outcome with known likelihoods.
- Hypothesis priors and action utilities are treated as supplied inputs, not learned or calibrated by this evaluator.
- Survey costs are expressed in the same utility units as action payoffs; they are arbitrary synthetic values here.
- The exhaustive search is capped at 16 survey candidates and is intended as a small reference oracle, not a production planner.
- Correlated sensors, adaptive multi-stage plans, continuous outcomes, spatially varying likelihoods, stakeholder-specific utility, safety constraints and rights constraints are not yet modeled.

A production Sol-Atlas planner must not turn this synthetic oracle into operational probabilities. It must elicit or estimate likelihoods, calibrate them on held-out sites, publish sensitivity analysis over priors and utilities, include the cost of false positives/negatives, and retain a human decision authority.

## Run

From this directory:

```sh
cargo test --manifest-path reference/Cargo.toml
cargo run --manifest-path reference/Cargo.toml -- fixtures/decision-case.json
```
