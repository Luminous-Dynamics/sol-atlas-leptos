# HDB-24A — factor-scope-safe evidence projections v0.1

## Purpose

HDB-24A is the first executable composition slice between the HDB-11A v0.2 probability-of-detection estimand and HDB-15's field-sweep/actual-GIS-coverage report semantics. It projects only two factors into the HDB-22 v0.1 factorized stage contract:

1. target-specific footprint coverage, supported by HDB-15 geometry plus a separate target-location/intersection receipt;
2. conditional sensor detection, supported by HDB-11A only when its POD is explicitly conditional on survival, exposure, observability and actual footprint coverage.

It does not compute a posterior. HDB-22 still needs independently supported survival, exposure, observability and false-positive stages.

## Critical semantic constraints

### HDB-11A

Accepted only when:
- the report is v0.2, rights-qualified and context matched;
- the observation is a qualified non-detection, not a detected or indeterminate result;
- the POD scope is exactly `ConditionalOnTargetSurvivedExposedObservableAndCovered`;
- the conditioning set is exactly survival, exposure, observability and actual footprint coverage;
- minimum eligible independent PSUs are met;
- profile, calibration-data, split, validation, method and rights provenance are present.

`JointPerSurveyUnit` is composite. HDB-22 v0.1 has no composite likelihood path, so HDB-24A rejects it rather than reusing the joint value as a conditional detection factor.

### HDB-15

HDB-15's reported coverage fraction is the GIS union of buffered transects divided by survey-unit area. HDB-24A validates its arithmetic and carries it as `hdb15_area_fraction_descriptive_only`. It is **never** copied into the target coverage probability.

The HDB-24A caller must also supply a target-location probability receipt with:
- the same target class, survey event, survey unit and CRS;
- exact match to HDB-15's coverage-union geometry and spatial-union receipt;
- a declared uniform-area or target-specific spatial model;
- a non-empty target-location distribution/model digest;
- a probability interval, independent intersection receipt, validation split, method build, source data, rights decision and receipt.

A target-specific probability interval is therefore distinct from the simple area ratio. Uniform-area weighting remains an explicit assumption, never a silent default.

## Projections

The adapter emits:
- a HDB-22-compatible `FootprintCoverage` stage projection;
- a HDB-22-compatible `ConditionalDetection` stage projection.

Both retain method/profile/build/calibration/split/validation/source/rights/access provenance. The coverage stage points at the GIS union geometry and spatial receipt, plus the target-location distribution and candidate-specific intersection validation receipt. The sensor stage preserves the HDB-11 Wilson interval without changing its endpoints.

These projections mirror HDB-22's stage fields while HDB-11A, HDB-15 and HDB-22 remain in separately stacked PRs. A later integration change should convert the projections to the native `StageEvidence` type rather than maintaining duplicate contracts indefinitely.

## Rejected cases

The evaluator rejects:
- composite survey-unit POD while HDB-22 v0.1 has no composite path;
- unqualified, mismatched or rights-undetermined HDB-11/HDB-15 reports;
- scope mismatch or missing conditioning events;
- mismatch in event, candidate, target class, survey unit or CRS;
- HDB-15 coverage area arithmetic mismatch;
- target-location receipt mismatch or missing probability/model/distribution;
- malformed report, profile, split, geometry, method, validation or rights digests.

It never assumes independent repeated visits. It never claims archaeological absence/identification or field action authority.

## Remaining HDB-24 work

- HDB-22 v0.2: introduce an explicit composite survey-unit detection update that consumes a joint HDB-11 POD and omits survival/exposure/observability/coverage factors already represented by that calibration.
- HDB-24B: make duplicate-factor accounting explicit in a full HDB-22 composition manifest, including which stage receipt contributes which event.
- HDB-24C: native conversion from the HDB-11A/HDB-15 source report types to HDB-22's native `StageEvidence`, with exact survey-unit, context, event, snapshot, CRS, rights and digest binding.
- Simulation study: compare naive multiplication (negative control), factorized chain, composite process and misspecified conditioning under controlled dependence; report posterior bias, Brier/log score, interval coverage/width and false-absence rate.

## Claim ceiling

This is software-contract qualification on synthetic fixtures only. It does not authenticate source evidence, prove a target-location distribution, establish archaeological presence/absence, demonstrate field performance or beat published models.
