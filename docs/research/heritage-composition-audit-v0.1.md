# HDB-24A — fail-closed probability composition audit v0.1

## Objective

Audit whether HDB-11 detection evidence and HDB-15 field-sweep geometry can populate the corresponding two factors in HDB-22's factorized non-detection chain without changing estimands, double-counting a factor, or promoting an area ratio to a target-location probability.

This is an audit and proposal stage only. It does not construct all six HDB-22 stages or calculate a posterior.

## Why this is necessary

HDB-22 v0.1 requires these six conditional stages:

1. target survival;
2. exposure given survival;
3. observability given survival and exposure;
4. target coverage given survival, exposure and observability;
5. detection given survival, exposure, observability and coverage;
6. false positive given original absence.

HDB-11 v0.2 declares the detection probability's conditioning set. Only the exact four-event set (survived, exposed, observable, actually covered) corresponds to HDB-22's current conditional-detection stage.

The following scopes are intentionally rejected by this adapter:
- joint_per_survey_unit: composite probability; HDB-22 v0.1 has no composite mode, so there is nowhere safe to put it;
- conditional_on_target_presented_to_sensor: includes an additional conditioning event not represented in HDB-22 v0.1;
- other_declared_conditioning_set: requires a separately reviewed factor map.

HDB-11 must also represent a qualified searched non-detection, not a detection, inaccessible search, or inconclusive observation.

## HDB-15 geometry is not a coverage probability

The HDB-15 report's swept-area fraction is:

buffered transect union area / survey unit area.

That is a geometric diagnostic. The adapter recomputes the ratio from the supplied geometry receipt and checks it against the HDB-15 report, but never copies it into the HDB-22 probability field.

Instead, the adapter requires separate target-location evidence that:
- binds to the exact HDB-15 source report, survey unit, CRS, transect geometry, survey-unit geometry and spatial-union receipt;
- states either a uniform-area target-location model or a target-specific spatial distribution;
- includes a target-location distribution digest, intersection geometry digest, and coverage probability receipt reference/digest;
- declares the coverage probability interval semantics;
- justifies the uniform-area assumption for the target class and sampling frame when that model is selected.

A candidate-specific or non-uniform location model must be supported by its own intersection receipt. A geometric union ratio is never silently substituted.

## v0.1 accepted mapping

For a factorized request, the audit emits two proposals only:

- HDB-11 Wilson 95% interval → HDB-22 ConditionalDetection, provided the HDB-11 scope equals the exact four-event conditioning set;
- separate target-location intersection probability → HDB-22 FootprintCoverage, provided its receipt binds to the exact HDB-15 geometry.

It preserves source profile, calibration, method/build, source-data, split, validation, analytical-rights and rights receipt fields required by HDB-22. The HDB-15 geometric area fraction is reported separately. The report explicitly states that the complete HDB-22 stage set is not present and no posterior was calculated.

The remaining survival, exposure, observability and false-positive factors must come from their own compatible evidence. This adapter does not estimate them.

## Context and arithmetic integrity

- HDB-11 calibration and observed contexts must match exactly; the observation must use the profile digest and be qualified.
- HDB-11 empirical POD must equal detected eligible PSUs / eligible PSUs; total = eligible + excluded; the Wilson 95% bounds must contain the empirical estimate.
- HDB-11 rights must be verified and its access tier cannot be unknown; a separate rights decision receipt must explicitly permit research analysis and match the source decision identifier/reference.
- HDB-15 requires the same purpose-specific research-analysis decision; `verified` is not treated as equivalent to `allowed`.
- HDB-15 report/profile/geometry digests and the source-report digest must be present and consistent.
- HDB-15 effective and buffered sweep widths must be finite and the buffered width must match the geometry receipt; its held-out Brier score, transect counts and seeded-target counts must be valid and the Brier score must meet the threshold in the profile binding.
- A context-alignment receipt binds HDB-11 profile/context to HDB-15 profile and the exact HDB-22 context identifier.
- A second event-alignment receipt binds HDB-11's exact covered-by-actual-footprint conditioning event to the HDB-15 geometry and target-location probability.
- HDB-15 swept area / survey-unit area must reproduce its reported geometric coverage fraction.
- Target-location probability evidence must bind the source report and exact geometry digests.
- Probability interval semantics are explicit: point estimate, deterministic bound, frequentist confidence interval or Bayesian credible interval. Confidence levels are required only for the last two.

Digest syntax is checked, not cryptographic authenticity. The evaluator cannot authenticate issuers or inspect source bytes. Importantly, a generic verified-rights status does not itself establish that research reuse is permitted: this audit requires separate, source-bound decision receipts for the exact research_analysis purpose for both HDB-11 and HDB-15. Restricted, prohibited or undetermined decisions fail closed, even when the upstream survey report says the receipt was verified.

## Explicit rejection controls

The Rust tests reject:
- joint POD in a conditional-detection slot;
- composite mode while HDB-22 v0.1 has no composite update path;
- presented-to-sensor and custom conditioning sets;
- unqualified or detected HDB-11 observations;
- inconsistent counts or POD arithmetic;
- non-finite coverage measurements;
- mismatched target class, HDB-11 profile, HDB-22 context, survey unit, CRS or geometry;
- HDB-15 area-ratio arithmetic drift;
- absent target-location distribution / intersection receipts;
- unjustified uniform-area assumptions;
- invalid or underspecified probability interval semantics.

## Benchmark programme

The next HDB-24B step should compare the naïve product (negative control), correct factorized chain, and composite joint model on synthetic event graphs with known dependence and controlled misspecification.

Measure posterior bias, Brier/log score, interval coverage and width, false-absence rate, calibration by context, and the impact of removing or swapping each receipt. The adapter must achieve 100% exact factor-mapping agreement with a hand-authored oracle across the finite contract corpus, and zero acceptance of each P0 negative control.

No field data or real archaeological probability is included here.
