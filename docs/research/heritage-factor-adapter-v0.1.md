# HDB-24A — factor-scope-safe HDB-11A / HDB-15 → HDB-22 adapter v0.1

## Objective

This adapter composes three existing contracts only when their probability semantics agree:

- HDB-11A v0.2: a calibration POD with an explicit conditioning set;
- HDB-15 v0.1: a fieldwalking sweep-width model and GIS-computed spatial-union receipt;
- HDB-22 v0.1: a one-event, survival/exposure/observability/coverage/detection/false-positive Bayesian update.

The adapter constructs a complete HDB-22 input and runs the actual HDB-22 evaluator. It does not invent a posterior formula or alter HDB-22's conditional-probability chain.

## Only supported probability factorization

HDB-24A v0.1 accepts exactly this HDB-11A scope:

`ConditionalOnTargetSurvivedExposedObservableAndCovered`

with the ordered conditioning set:

1. target survived;
2. target exposed;
3. target observable;
4. target covered by the actual footprint.

The calibration trials contributing to HDB-11A's POD denominator must all have evidence that all four conditions were satisfied. The POD Wilson interval is then mapped to HDB-22's `ConditionalDetection` stage, which has precisely the same conditioning semantics.

The caller must supply already-qualified HDB-22 stages for:

- Survival;
- Exposure;
- Observability;
- FalsePositive.

The adapter supplies only:

- FootprintCoverage from HDB-15 plus a target-location probability receipt;
- ConditionalDetection from HDB-11A's 95% Wilson POD interval.

The completed six-stage input is passed through `evaluate_non_detection`; HDB-24 returns both the adapted input and the HDB-22 posterior report.

## Intentionally rejected scopes

- **JointPerSurveyUnit**: rejected. It is a composite survey-process probability and cannot be put into HDB-22's conditional sensor-detection slot while separately multiplying survival/exposure/observability/coverage.
- **ConditionalOnTargetPresentedToSensor**: rejected in v0.1 because HDB-22 does not yet encode a separate target-presentation event in its conditioning ontology.
- **OtherDeclaredConditioningSet**: rejected. No generic condition-to-factor mapping is guessed.

A later HDB-22 v0.2 may introduce a factorization enum and a dedicated composite-detection stage. That should be a separate versioned change with separate proofs and negative controls; v0.1 does not pretend to support it.

## Coverage semantics

HDB-15's `receipt_reported_coverage_fraction` is the area of the GIS-computed union of buffered transects divided by survey-unit area. It is not automatically the probability that a particular archaeological target fell inside the searched footprint.

HDB-24A therefore requires a distinct `TargetLocationCoverageReceipt` bound to the exact HDB-15 survey unit, survey-unit geometry, transect geometry and spatial-union receipt.

Two explicit models are supported:

- `UniformAreaWeightedWithinSurveyUnit`: the target-location probability band must be a point value exactly equal (within 1e-9) to HDB-15's geometric coverage fraction. The uniform spatial distribution assumption has its own digest.
- `TargetSpecificSpatialDistribution`: the receipt supplies a target-specific probability band, target-location distribution digest, method/build, source, split, calculation receipt, validation receipt and rights decision. The adapter does not derive this probability from the area fraction.

Planned transect metadata, mismatched geometry, an unbound location model, missing rights, invalid probabilities or unknown access tier are rejected. The GIS operation itself remains external; the adapter validates receipt structure and equality, not the GIS computation or authenticity.

## Context and provenance binding

HDB-24 requires an explicit context mapping receipt because HDB-11, HDB-15 and HDB-22 do not share identical context identifiers. It binds target class and HDB-22 context ID to HDB-11/HDB-15 profile digests and canonical context digests.

HDB-11 and HDB-15 provenance bindings require method/build, calibration data, source data, validation split/receipt, rights decision, analytical permission, rights receipt and access tier. The HDB-15 spatial union and the separate target-location calculation/validation are retained as distinct receipts.

The adapter checks digest syntax, references, rights status and equality constraints. It does not cryptographically authenticate issuers, inspect original source bytes, adjudicate archaeological truth, or prove calibration independence. Those remain separate provenance and field-validation responsibilities.

## Fail-closed conditions

The adapter rejects:
- unqualified HDB-11 non-detections, context mismatch or wrong feature class;
- non-fieldwalking HDB-11 modality in this HDB-15-specific adapter;
- unsupported POD scope or missing/reordered conditioning events;
- inadequate eligible calibration sample or malformed Wilson interval;
- HDB-11 processing-build mismatch;
- unqualified or rights-incomplete HDB-15 reports;
- HDB-15 profile / fit / calibration / split receipt mismatch;
- survey-unit, survey-unit geometry, transect geometry or spatial-union receipt mismatch;
- uniform location model whose probability differs from the geometric area fraction;
- target-specific location receipt with invalid digests, missing rights or invalid probability band;
- a template that is not `SearchedNoDetection`;
- a template that already includes coverage or conditional-detection stages, preventing silent replacement;
- missing Survival, Exposure, Observability or FalsePositive stages;
- any HDB-22 validation error after assembly.

## Benchmark and acceptance plan

Synthetic software-contract tests require:
1. valid factorized composition yields a six-stage HDB-22 input and an evaluated posterior;
2. joint POD is rejected;
3. sensor-presentation and arbitrary custom scopes are rejected;
4. geometric area fraction cannot be used when the uniform-location probability does not match;
5. target-specific location models require their own receipt;
6. geometry and survey-unit mismatches fail closed;
7. build/context/rights mismatches fail closed;
8. pre-existing adapted stages cannot be overwritten;
9. inaccessible, unsurveyed and inconclusive states cannot be forced into a negative-evidence update.

For the eventual field benchmark, compare:
- direct six-stage, independently sourced HDB-22 factorization;
- HDB-24A-adapted HDB-11/HDB-15 factors;
- an oracle calculation on fully adjudicated, controlled survey data.

Report posterior calibration, Brier/log score, Wilson coverage, false-positive burden, subgroup calibration by visibility/crew/landform, and sensitivity to the target-location distribution. Use independent survey projects/sites as inferential units. No stone, transect or pixel pseudo-replication.

## Claim ceiling

HDB-24A is an adapter contract and synthetic integration test. It is not field validation, a claim that HDB-11 or HDB-15 estimates are empirically correct, proof of archaeological absence, or authorization to survey, excavate, publish sensitive locations or act on a posterior.
