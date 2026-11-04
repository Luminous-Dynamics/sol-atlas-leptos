# HDB-14 — Context-Conditioned Representation and Model Routing v0.1

## Scientific motivation

Zabaleta-Santisteban et al. (2026), *Archaeological Prospection*, DOI 10.1002/arp.70033, apply 17 relief visualization techniques to 0.5 m ALS terrain models at Kuélap and Cambolín. The best final-test configuration differs by site: combined VAT at Kuélap (mAP 0.38, F1 0.72) and slope at Cambolín (mAP 0.37, F1 0.77). The reference labels are expert interpretation of ALS DTMs, not independent field verification. The paper also has metric naming distinctions between its Table 6 average-precision percentages and Tables 7–8 mAP/AP. Reproduction must pin the exact table, evaluation phase and metric definition.

This motivates a general hypothesis: representation selection should be context-conditioned, but only where external site-level evidence validates the selected configuration.

## Decision boundaries

- HDB-14 selects an already evaluated representation/model for the observations in hand.
- HDB-6 / PR #446 selects which new observation should be acquired.
- HDB-11 / PR #514 assesses whether a non-detection is informative.
These are separate decision contracts.

## v0.1 selection contract

Each candidate profile binds exact task, feature class, sensor modality, channel semantics, landscape context, representation transform, model build, metric definition, primary metric, reference tier, review budget, split manifest, data manifest, evaluation role, training-site IDs, per-validation-site results and rights receipt.

Landscape context is exact-match categorical in v0.1: ecology/biome, geomorphology, relief complexity, canopy/vegetation, preservation/visibility and resolution. No cross-context interpolation is implicit.

Only `selector_validation` profiles with held-out-site or held-out-region splits are eligible. Training and validation site IDs must not overlap, validation IDs must be unique, and every profile must contain at least one declared training site. Outer-test profiles and random-pixel/random-tile splits are ineligible.

## Conservative selection

The primary utility lower bound is normalised/oriented by the preregistered metric definition digest, higher being better. That digest pins the underlying metric, matching/IoU semantics, any transformation and uncertainty procedure. Profiles with different task, metric ID/digest, reference tier or review budget cannot be ranked together.

Every validation site must meet all quality floors:
- primary metric utility lower bound;
- precision lower bound;
- false candidates per km² ceiling;
- coverage floor;
- review seconds per candidate ceiling;
- calibration pass.

Rights must be verified and accompanied by a receipt. Any site failing any mandatory floor rejects the entire profile. Profiles are ranked lexicographically by worst-site primary utility lower bound, then worst-site precision lower bound, then worst-site false-candidate burden, then profile ID for deterministic tie-breaking.

This is deliberately not a blended scalar score: quality, false alarm, coverage, calibration, cost and rights remain separate hard constraints.

## No-profile behavior

When no exact-context profile qualifies, the selector returns `abstain_no_qualified_profile`. It does not select the globally highest-scoring profile as a fallback, transfer a profile to a new biome/sensor/resolution, or infer performance from a test set.

## HDB-14 benchmark protocol

1. Split complete sites/landscapes into model-training, selector-validation and untouched outer-test partitions. Where enough sites exist, use nested group cross-validation or nested leave-region-out evaluation.
2. Compare a global fixed representation, expert-default representation, context-conditioned selector and representation ensemble.
3. Keep a test-site oracle best representation only as an unattainable upper bound; never call it a deployable method.
4. Evaluate on outer sites not used for profile selection. Report worst-site and lower-tail performance, selector regret to oracle, selective risk/coverage, false candidates/area, task-specific AP/mAP/F1/IoU, calibration, review effort, compute and energy.
5. Stratify by canopy/vegetation, slope/relief, feature morphology, sensor, resolution, season, soil/moisture and missing-channel condition.
6. Repeat by proxy family. Cropmark, stone wall, LiDAR enclosure and looting change detection do not share a universal metric.

## Negative controls

- Outer-test labels/performance cannot influence selection.
- Pixel-random and random-tile results cannot qualify a profile for external generalisation.
- Training/validation site overlap fails.
- Mixed tasks, target definitions, reference tiers, channel semantics, metric digests, matching thresholds or review budgets cannot be ranked.
- A weak site cannot be hidden by a strong mean.
- Pending/unknown/denied rights fail.
- Unsupported context yields abstention rather than silent fallback.
- The selector never declares archaeological significance or identification.

## v0.1 outputs and claim ceiling

The report records selected profile/transform/model digests, worst-site utility/precision/false-candidate values, candidate/qualified/rejected profile counts and exact reasons for rejection. It sets outer-test usage, silent fallback and archaeological identification flags to false.

The evaluator is a deterministic policy function over supplied summaries. It does not authenticate split manifests, rights, site independence, metrics, source data or model weights. Synthetic tests validate selector logic only; real superiority requires rights-cleared datasets, a nested site/region design, independent adjudication and outer-test results.
