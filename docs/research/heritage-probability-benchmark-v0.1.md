# HDB-24B — exact event-graph composition benchmark v0.1

## Purpose

Provide a small, deterministic oracle for the central HDB-24 probability-scope claim. This benchmark is synthetic mathematics only. It does not estimate a real archaeological prior, detectability, target distribution or field outcome.

## The chain rule, not independence

For a target originally present, the factorized path is:

`P(D|O)=P(S|O) P(E|S,O) P(V|E,S,O) P(C|V,E,S,O) P(D|C,V,E,S,O)`.

The factors are sequential conditional probabilities. Their product follows the probability chain rule; no mutual-independence assumption is needed. The composite survey-unit probability is the joint result of that same chain.

Therefore:
- factorized path: multiply the five conditional factors;
- composite path: use the joint per-survey-unit POD once, omitting all five component factors;
- invalid path: multiply the joint POD by any or all factors that the joint value already includes.

## Exact oracle

The Rust evaluator accepts a frozen scenario with:
- prior presence probability;
- five sequential conditional event probabilities;
- an independently declared joint POD which must equal their product within 1e-12;
- false-positive probability under original absence;
- scenario, protocol and evaluator digests.

It computes posterior presence probability after both detection and non-detection. It then calculates exact expected Brier score and expected log loss by summing the four joint masses (present/detected, absent/detected, present/missed, absent/missed). There is no Monte Carlo or random seed.

## Example

For prior 0.2, survival 0.8, exposure 0.6, observability 0.7, coverage 0.5 and conditional detection 0.9, the upstream product is 0.168 and joint POD is 0.1512. With false-positive probability 0.05:
- the exact factorized and exact composite posterior after a miss both equal approximately 0.182585;
- the intentionally incorrect joint-POD-times-upstream path uses POD 0.0254016 and returns approximately 0.204122 after a miss;
- its posterior bias is approximately +0.021537, while detection-posterior bias is approximately -0.317829;
- expected Brier rises from approximately 0.155985 to 0.159640;
- expected log loss rises from approximately 0.489912 to 0.499123.

These scores are properties of this illustrative scenario only, not field performance.

## Scope validator

The production factor-scope validator accepts only:
- the canonical five-stage factorized list; or
- the single joint-per-survey-unit factor.

It rejects mixed joint/component lists, duplicates, missing factors and non-canonical order. The double-count computation exists only as an explicitly labelled negative control in the report and must never be accepted as a production factor mapping.

## Acceptance tests

- factorized and composite posterior equality on non-degenerate and degenerate cases;
- declared joint probability must match the sequential conditional product;
- non-finite and out-of-range probabilities fail closed;
- zero-likelihood posterior is represented as undefined;
- exact canonical factor sets pass;
- joint-plus-components, duplicates, missing and reordered factors fail;
- in the non-degenerate negative-control case, posterior bias is non-zero and both proper scores worsen.

## Claim ceiling

A passing synthetic oracle proves only that the code implements the stated probability algebra and factor exclusivity. It does not authenticate HDB-11/HDB-15/HDB-22 receipts, establish real-world conditioning semantics, estimate archaeological absence, or validate model transfer. The HDB-24A adapter remains structural until independent semantic and field review.
