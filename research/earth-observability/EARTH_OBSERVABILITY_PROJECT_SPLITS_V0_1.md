# EARTH-OBSERVABILITY-PROJECT-SPLITS-001
## Project-clustered fitting, calibration and blind evaluation v0.1

**Implementation:** `sol-atlas-core/src/project_split.rs`  
**CLI:** `project_split_eval`  
**Input schema:** `PROJECT_DATASET_INDEX_V0_1.schema.json`  
**Input fixture:** `fixtures/project-split-input-synthetic-v0.1.json`  
**Pinned split:** `baselines/project-split-manifest-synthetic-v0.1.json`  
**Parent:** Sol-Atlas #771 / dataset admission #783  
**Status:** deterministic split builder, validator, fixture, schema and SHA-256-bound receipt CLI authored; exact-head CI pending  
**Claim ceiling:** Generic synthetic project IDs only. No UT-Gold-D files have been downloaded, parsed or split.

---

## 1. Why the split unit is a construction project

UT-Gold-D reports 125 utility-survey activities and 959 radargrams across 13 construction projects. Activities, radargrams, traces, annotations, trench images and derived labels from the same construction project share acquisition context, operator practices, geology, urban environment and often common sources.

A random trace/image split can therefore place near-duplicate physical evidence and site-specific signatures in both training and blind partitions. The benchmark would measure interpolation within a known project rather than transfer to a new project.

The indivisible split group is the **whole construction project**. Every child activity inherits its parent project's role. The split builder rejects duplicate activity IDs across projects and validates complete project-role assignment.

For UT-Gold-D, project IDs in the research fixture are synthetic placeholders. Real repository project identities must be admitted only after source inventory and ID linkage review.

## 2. Deterministic leave-one-project-out protocol

The executable builder accepts a project/activity index and a declared number of calibration projects per fold.

For each fold:
- exactly one project is blind;
- the configured number of other projects are calibration-only;
- all remaining projects are fitting-only;
- no project ID may appear in more than one role;
- each project is blind exactly once across the manifest;
- project and activity ordering is canonicalized for deterministic output.

The committed synthetic 13-project pilot uses three calibration projects:
- 9 fit projects;
- 3 calibration projects;
- 1 blind project;
- 13 folds, with each project blind exactly once.

The three-calibration setting is a reproducible pilot configuration, not a universal statistical standard or a claim of sufficient calibration sample size.

The `project_split_eval --sensitivity-sweep` mode now emits SHA-256-bound split-allocation summaries for calibration counts one, two and three. On the 13-project synthetic index these produce:
- 11 fit / 1 calibration / 1 blind;
- 10 fit / 2 calibration / 1 blind;
- 9 fit / 3 calibration / 1 blind.

The sweep verifies role disjointness and that every project is blind exactly once for each allocation. It compares only the partition sizes and deterministic split structure; it does not estimate calibration quality, power or model performance. The physical benchmark should preregister a sensitivity analysis over one, two and three calibration projects per fold (and consider nested/rolling calibration where model selection itself requires a validation layer). Report the effect on training-set size, calibration stability, interval coverage and between-project variability before fixing the physical protocol.

## 3. Leakage invariants enforced by Rust

The split module checks:
- non-empty dataset, project and activity IDs;
- unique project IDs;
- each project has at least one activity;
- activity IDs are unique across the entire dataset index;
- minimum project-cluster count;
- valid calibration-project count;
- manifest dataset identity and split policy;
- every fold ID is bound to its blind project;
- no unknown project IDs;
- each project is assigned exactly one role per fold;
- no project is omitted;
- no project is blind more than once;
- calibration cardinality is exact.

These structural checks do not replace:
- spatial footprint overlap analysis;
- near-duplicate radargram/image detection;
- trace/annotation/trench linkage audits;
- shared preprocessing/model checkpoint audits;
- source lineage and operator/campaign common-cause analysis.

All such audits remain required before physical blind evaluation.

## 4. How to use this in model evaluation

For each fold:
1. Fit model parameters, feature selection and learned preprocessing using only fit projects.
2. Fit calibration mappings, uncertainty scaling and any predeclared decision thresholds using only calibration projects.
3. Freeze code/configuration, weights, transforms, thresholds and inference environment.
4. Reveal the blind project once and score all admissible events under the frozen configuration.
5. Retain per-project result, failure cases, abstentions, excluded truth tiers and input digests.
6. Do not retune from that fold's blind result. A later development iteration must treat that project as development data and obtain a new external blind project for prospective claims.

For methods that require hyperparameter tuning, use nested project-level validation inside the fit-project subset. The calibration-project subset must not be used to select architecture or training hyperparameters unless that role is explicitly changed and a separate calibration set is reserved.

## 5. Small-cluster caution

Thirteen projects are more informative than hundreds of randomly partitioned radargrams, but still a small number of independent clusters. A 13-fold design does not magically create 13 statistically independent replications of a universal deployment population.

Report:
- every project result;
- macro-average and project-level dispersion;
- worst-project and worst-domain result;
- the number of eligible truth clusters per metric;
- cluster-bootstrap or hierarchical uncertainty only when its assumptions and sample size are defensible;
- sensitivity to one/two/three calibration projects;
- project composition and selection bias.

Do not use trace-level bootstrap intervals as if the traces were independent sites. Do not present tight population confidence intervals solely because there are 959 radargrams.

## 6. Separate tracks for positionless and degraded acquisition

A project-level split must not erase the UT-Gold-D acquisition limitations. Per activity/file, preserve:
- GNSS RTK georeferenced;
- wheel-encoder-only;
- mixed/partially georeferenced;
- spatially unresolved;
- wheel-contact quality known/poor/unknown;
- positioning transform and uncertainty provenance.

A file can remain eligible for signal-level event detection without being eligible for localization, asset geometry or network-topology scoring. Each metric has its own eligible denominator, with excluded counts visible.

## 7. Receipt and reproducibility

The `project_split_eval` CLI:
- parses the declared index;
- builds and validates the deterministic split;
- binds the receipt to the exact input bytes with SHA-256;
- compares the split manifest against the pinned synthetic baseline.

CI invokes the shared-core project-split tests and `project_split_eval --check-pinned`. A successful synthetic split check proves only deterministic structural partitioning. It does not prove that real project IDs, activity-to-project lineage or spatial overlap are correct.

## 8. Nonclaims

This is not a claim that the UT-Gold-D source has been admitted, that its 13 clusters are representative of all GPR deployment settings, that three calibration projects are statistically optimal, or that any detector/model has generalized to a held-out project. No physical performance result is produced by the split builder.
