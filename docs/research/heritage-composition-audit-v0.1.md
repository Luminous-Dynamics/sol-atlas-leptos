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

A more subtle distinction also matters: HDB-15's effective sweep width is the integral of a lateral detection-probability curve, an equivalent perfect-detection width. It is not a hard physical boundary, and it does not mean that detection is certain inside the buffer or impossible outside it. Consequently, the HDB-15 effective-sweep buffer cannot itself define HDB-22's binary footprint-coverage event.

Instead, the adapter requires a separate, independently receipted physical observation domain and target-location evidence that:
- binds to the exact HDB-15 source report and profile as calibration provenance;
- contains its own physical observation-domain geometry digest and receipt, based on the recorded observer path and declared view boundary or a sensor's valid-data footprint—not the effective-sweep-width buffer;
- binds to the survey unit, CRS, raw transect geometry and survey-unit geometry;
- states either a uniform-area target-location model or a target-specific spatial distribution;
- includes a target-location distribution digest, intersection geometry digest, and coverage probability receipt reference/digest;
- proves that HDB-11's calibration conditioning used the exact same physical observation-domain digest and target-location distribution digest;
- declares zero detection outside that domain under the observation protocol; if detection tails remain outside it, this six-stage factorization is not valid and a spatial detection-likelihood integral is required;
- declares the coverage probability interval semantics;
- justifies the uniform-area assumption for the target class and sampling frame when that model is selected.

A candidate-specific or non-uniform location model must be supported by its own intersection receipt. A geometric union ratio is never silently substituted. HDB-15's effective-sweep-width union remains a separate diagnostic/provenance object and is not used as the physical observation-domain geometry.

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
- A second event-alignment receipt binds HDB-11's exact covered-by-actual-footprint conditioning event to the same independently receipted physical observation domain and target-location distribution used by HDB-24A.
- HDB-11's conditioning-domain and target-location distribution digests must exactly equal the target-coverage receipt digests.
- The declared physical domain must have zero observation probability outside it; otherwise a separate spatial detection-likelihood integral is required and HDB-24A refuses the factorization.
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
- attempts to use its effective-sweep-width buffer as the physical observation domain;
- missing physical observation-domain receipt, nonzero detection outside the domain, or mismatch between HDB-11's conditioned domain/distribution and the coverage model;
- absent target-location distribution / intersection receipts;
- unjustified uniform-area assumptions;
- invalid or underspecified probability interval semantics.

## Benchmark programme

The next HDB-24B step should compare the naïve product (negative control), correct factorized chain, and composite joint model on synthetic event graphs with known dependence and controlled misspecification. If the actual observation process has nonzero detection outside a chosen geometric domain, HDB-24C must instead integrate a spatial detection function over the target-location distribution and surveyed path, including lateral detection tails.

Measure posterior bias, Brier/log score, interval coverage and width, false-absence rate, calibration by context, and the impact of removing or swapping each receipt. The adapter must achieve 100% exact factor-mapping agreement with a hand-authored oracle across the finite contract corpus, and zero acceptance of each P0 negative control.

No field data or real archaeological probability is included here.
