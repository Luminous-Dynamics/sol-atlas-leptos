# HDB-11 — Heritage Observability and Detection-Limit Evaluation v0.1

## Purpose

A recorded non-detection is not equivalent to evidence of absence. Its evidential value depends on whether the feature would likely have been observable under the actual sensor, processing, season, environment, spatial resolution, data quality and survey coverage.

HDB-11 provides an auditable, conservative evaluator for that question. It complements HDB-8 / the probability-sampled predicted-negative audit (PR #473): that evaluator samples predicted-negative landscape units and estimates population error; HDB-11 defines when a survey-level non-detection has adequate, context-matched sensitivity to contribute negative evidence.

## Scientific basis and limitations

Archaeological field-survey research has long documented that visibility and geomorphology affect whether archaeological remains are detectable. A negative field observation can be misleading where vegetation, surface visibility, erosion, deposition or survey practice reduce detection opportunity. Visibility research also shows that field-walking visibility assessment is not always standardised. Sweep-width / seeded-target studies provide a stronger calibration idea: test survey detection against a known population of targets, then estimate detection probability under declared conditions.

v0.1 therefore calibrates only on independently selected, known-positive primary sampling units (PSUs). It restricts its statistical claim to a probability-sampled independent-site Bernoulli design and reports a Wilson score interval. It does not pretend that a Wilson interval fixes an unrepresentative calibration frame or spatial dependence hidden inside a PSU.

## Exact-context profile

A calibration profile binds:
- archaeological feature class;
- sensor modality and exact sensor/configuration digest;
- processing build digest;
- environment / visibility stratum;
- season / phenophase stratum;
- resolution class;
- declared sampling design;
- minimum independent PSU count and lower-bound PoD threshold;
- minimum coverage and usable-data fractions.

A target observation must match the whole profile. There is no implicit interpolation across sensors, environments, seasons, feature classes, processing versions or resolutions. Such transfer requires its own external validation and profile.

## Calibration procedure

1. Preregister the target feature class, observation context, profile digest, primary sampling unit, sample design, minimum number of PSUs, confidence level and lower-bound requirement.
2. Sample independent sites / PSUs where presence of the declared target feature is independently established through field assessment, independent expert review or appropriate instrument observation.
3. Run the exact declared sensor and processing configuration. Record whether each target was detected.
4. Count at most one calibration outcome per independent PSU. Multiple pixels, chips, dates, repeated interpretations or overlapping tiles from the same site do not create additional independent trials.
5. Require verified rights and a rights receipt for every calibration reference.
6. Estimate empirical PoD and its two-sided 95% Wilson score interval.
7. Evaluate the target observation against profile identity, coverage, valid-data fraction, visibility and rights gates.

Wilson interval:
`p = x/n`, `z = 1.959963984540054`, `denom = 1 + z²/n`,
`center = (p + z²/(2n))/denom`,
`half = z sqrt(p(1-p)/n + z²/(4n²))/denom`.
Bounds are clipped to [0,1]. This is a binomial interval under the supported independent-PSU design; design-based cluster or unequal-probability estimators are future work, not silently approximated here.

The v0.1 policy floor is 20 independent PSUs. This is a minimum engineering guard, not a claim that n=20 is sufficient for every feature class, consequence or confidence requirement. The preregistered lower confidence bound threshold determines qualification; a strong point estimate alone is insufficient.

## Four dispositions

- `detected`: observation recorded a detection; this does not identify the feature as archaeological.
- `qualified_non_detection`: the exact matched survey was completed with sufficient calibration, sensitivity lower bound, coverage, valid data, visibility and verified rights. This is informative negative evidence only; it does not prove absence.
- `indeterminate_non_detection`: no detection, but at least one evidential gate failed.
- `not_surveyed`: no qualifying observation was attempted.

The report never emits `confirmed_absent` or `no_archaeology`. A separate human/expert evidence process is required to make archaeological interpretations.

## Negative controls

The evaluator must withhold qualification for:
- a mismatched feature class, sensor, sensor configuration, processing build, environment, season or resolution;
- duplicate PSU or site IDs;
- fewer than the preregistered independent PSU minimum;
- a Wilson lower bound below the preregistered PoD threshold;
- insufficient spatial coverage or usable data;
- partial, blocked or unknown visibility;
- pending, denied or unknown rights or missing rights receipt;
- model-only or catalogue-only presence references;
- non-finite fractions, malformed hashes or invalid input;
- unsupported unequal-probability / clustered sampling designs.

No repeated acquisitions are combined as though conditionally independent in v0.1. Distinct source digests are not proof of independent observations.

## Benchmark programme

Report by feature class × sensor × environment × season × resolution:
- empirical PoD, Wilson lower and upper bounds, and independent PSU count;
- false-qualified-non-detection rate on adversarial and independently adjudicated controls;
- coverage and indeterminate / abstention rate;
- calibration reliability and degradation on held-out sites, regions and biomes;
- effects of canopy, vegetation, soil moisture, illumination, terrain, cloud/nodata, sensor degradation and spatial resolution;
- analyst hours and survey cost per informative, independently reviewed result.

Baselines:
1. Unsafe control: treat every model/survey non-detection as a negative.
2. One global PoD estimate for every context.
3. Per-sensor PoD without environmental or seasonal stratification.
4. Expert visibility rating alone.
5. Exact-context calibrated PoD with the preregistered Wilson lower-bound gate.

Primary evaluation must be site/region disjoint and must not use calibration PSUs as test observations. Report domain-specific support and confidence intervals. Do not average away a biome or sensor where the calibration is unsupported.

## Reproducibility and governance

Pin the source observation, sensor/configuration, processing build, calibration trials, split manifests, rights receipts, profile digest and evaluator build. Exact coordinates and vulnerable site metadata remain behind the relevant custodial/access policy. Public benchmark fixtures use synthetic site IDs. Community and custodial authorities retain control over culturally sensitive observations and whether a survey is appropriate.

## v0.1 deliverables and claim ceiling

The Rust module is a pure evaluator. It performs no network acquisition, remote-sensing detection, field survey or archaeological adjudication. Structural receipt references are not authenticated by this module. Synthetic fixtures validate the contract and arithmetic only; real PoD estimates require rights-cleared, independently assessed, representative field/survey data.

Follow-on work:
- design-based estimators for stratified, unequal-probability and cluster/PSU sampling after statistical review;
- conditional-independence / repeated-observation models with dependence evidence;
- hierarchical calibration with explicit partial pooling and conservative OOD abstention;
- calibration transfer across regions only after prospective external validation;
- integration with HDB-8 predicted-negative probability audit and HDB-6 sequential survey planning.
