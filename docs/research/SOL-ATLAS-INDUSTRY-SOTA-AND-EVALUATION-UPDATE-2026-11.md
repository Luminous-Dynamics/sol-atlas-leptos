# Sol-Atlas industry SOTA and evaluation update — November 2026

**Status:** research plan and literature synthesis. No comparative performance result is claimed.

## 1. Industry baseline

The OGC JSON-FG standard is now a published 1.0.0 standard. Its purpose is to extend GeoJSON for feature typing, additional geometry/CRS cases and temporal information while retaining GeoJSON compatibility. OGC API - Features is modular: Part 1 Core is primarily discovery and GET-based feature retrieval; further parts address CRS, filtering and schemas. Sol-Atlas should use JSON-FG and OGC API - Features as transport foundations, not treat either as a complete claim, rights or evidence-governance model.

The current pygeoapi 0.24.0 documentation describes it as an OGC API implementation and lists OGC API - Features as a reference implementation and OGC API - Records as implemented. GeoServer's current implementation-status page reports Parts 1 and 2 as passing compliance tests, Part 3 and CQL2 as implemented without CITE tests, and Part 5 Schemas as a draft proposal not implemented. These distinctions matter: benchmark each advertised capability independently and pin versions; do not compare products by a single OGC badge.

Recent heritage research makes the competitive landscape more sophisticated than a simple globe or 3D viewer:

- A 2026 metrological digital-twin study describes uncertainty budgets and multi-sensor geometric validation for cultural heritage.
- A 2026 Scan-to-HBIM review synthesizes 120 publications and highlights inconsistent reporting across RMSE, IoU and level-of-detail metrics, as well as the need for transparent, comparable benchmarks.
- A 2026 review of heritage-building digital twins synthesizes 204 studies and identifies interoperability, AI validation and long-term preservation as continuing challenges.
- A 2026 use-centred HBIM / heritage digital-twin framework argues that technical completeness and geometric quality alone do not establish operational usefulness.

Sol-Atlas cannot differentiate itself merely by attaching uncertainty values to 3D geometry. Its research claim should be that uncertainty, evidence relations, provenance, rights, alternative interpretations and downstream decisions remain jointly inspectable and testable through the lifecycle.

## 2. Benchmark tracks beyond the first exchange corpus

The first 16 invariants and 17 negative controls are an exchange-qualification corpus. The next benchmark release should include independent tracks:

| Track | Measurement | Required reporting |
|---|---|---|
| Geometry / metrology | Point-to-plane distance, cloud-to-cloud distance, registration error, dimensional bias, uncertainty coverage | Per sensor, surface, scale, material and occlusion stratum; units and confidence level |
| Semantic reconstruction | Element classification macro-F1, boundary IoU, instance precision/recall, relation extraction precision/recall | Expert adjudication protocol, disagreement and abstention |
| Evidence graph integrity | Referential integrity, relation precision/recall, duplicate/conflict detection | Per relation class; no counter/context promotion |
| Hypothesis integrity | Alternative retention, unsupported-selection rate, adjudication trace completeness | Human decision authority remains explicit |
| Uncertainty calibration | Brier score / log score where probabilistic; empirical interval coverage and mean interval width for geometric estimates | Reliability diagrams, sample counts, confidence intervals, stratified calibration |
| Rights and privacy | Capability non-escalation, disclosure tests, public-output non-interference | Zero accepted rights escalations; zero unauthorized observable differences |
| Interoperability | Semantic field preservation, transform correctness, loss-ledger completeness | Per adapter and multi-hop route, not one aggregate percentage |
| Reproducibility | Canonical semantic projection, source/config/tool digests, replay determinism | Exact inputs, versions, CRS database and environment |
| Decision utility | Change in expert time, survey prioritization quality, expected information gain, conservation decision regret | Prospective study, control workflow and community-defined outcomes |
| Operations | Throughput, p50/p95 latency, memory, storage, cold start and recovery | Matched hardware, frozen workload and concurrency |

Do not collapse these into one leaderboard score. Publish a scorecard and hard-gate failures separately. A privacy breach or fabricated provenance cannot be compensated for by better IoU.

## 3. Calibration and abstention are first-class outputs

A system should not be rewarded for being confidently wrong. For each model-generated geometry, classification or claim:

1. Publish the target variable and its operational definition.
2. Publish a predictive distribution or interval, not only a point estimate.
3. Evaluate coverage on independent site-level holdouts.
4. Measure interval sharpness only after coverage is calibrated.
5. Report worst-stratum calibration and selective risk as the system abstains.
6. Compare to simple baselines (survey-only, nearest-neighbour / rule-based, and human-only) before expensive learned models.

For a nominal 95% geometric interval, the first target is empirical coverage compatible with 95% within a pre-registered tolerance on independent site holdouts. The tolerance must be set by sample size and a binomial confidence interval, not chosen after results are seen. A model with wider intervals may be better calibrated but less useful; report both coverage and width.

## 4. Matched SOTA comparison protocol

Use at least these matched baselines:

- **Survey / metrology baseline:** registered raw observations and independently checked control points.
- **Conventional GIS baseline:** a pinned OGC Features stack (pygeoapi and GeoServer), with GDAL/OGR conversion.
- **Scan-to-HBIM baseline:** a pinned, documented workflow with expert refinement.
- **Learned perception baseline:** a contemporary segmentation / detection model evaluated on the same site holdouts.
- **Human expert baseline:** blinded experts with the same evidence package and time budget.
- **Sol-Atlas ablations:** remove typed evidence relations, provenance binding, rights enforcement, uncertainty calibration, or hypothesis retention one at a time.

All methods receive the same source observations, permitted evidence, compute/time budget and reference labels. Hold out complete sites, source collections and, where relevant, acquisition campaigns. Report per-site results and worst-stratum performance. Pre-register metrics and thresholds. Keep a sealed test set and independent evaluator. Include confidence intervals and negative controls.

## 5. Falsifiable research hypotheses

- **H1 — semantic preservation:** Sol-Atlas profile-aware routes preserve more decision-relevant semantics than unprofiled GIS exports, without claiming that ordinary GIS systems are defective when a capability is outside their advertised scope.
- **H2 — epistemic integrity:** typed contradiction/context associations and explicit hypothesis sets reduce unsupported claim promotion relative to a single-record / single-label workflow.
- **H3 — rights safety:** pre-projection policy evaluation prevents restricted-input changes from altering public outputs, within the declared threat model.
- **H4 — decision utility:** value-of-information ranking reduces expert survey effort or improves discovery yield under a fixed budget.
- **H5 — calibration:** stratified uncertainty reporting improves the reliability of downstream conservation decisions compared with point estimates and uncalibrated confidence scores.

Each hypothesis needs a null hypothesis, independent data, ablations and a falsification criterion. Until those experiments run, they remain research hypotheses rather than performance claims.

## 6. Sources

- OGC JSON-FG 1.0.0: https://www.ogc.org/standards/json-fg/
- OGC API - Features: https://www.ogc.org/standards/ogcapi-features/
- OGC API - Features / Common Schemas: https://docs.ogc.org/is/23-058r2/23-058r2.pdf
- pygeoapi 0.24.0 (2026-07-28): https://docs.pygeoapi.io/en/stable/
- GeoServer OGC API - Features implementation status: https://docs.geoserver.org/latest/en/user/services/features/status/
- Moyano et al. (2026), metrological framework for cultural-heritage digital-twin measurement systems: https://doi.org/10.1016/j.measurement.2026.122271
- Performance benchmarking of scan-to-HBIM workflows (2012-2025), Automation in Construction (2026): https://doi.org/10.1016/j.autcon.2026.106848
- Tootoonchi and Hess (2026), heritage-building digital-twin review: https://doi.org/10.5194/isprs-annals-XI-4-2026-435-2026
- Doria et al. (2026), use-centred HBIM and heritage digital-twin evaluation: https://www.mdpi.com/2571-9408/9/8/309


## 2026-11-28 update: adapter-readiness reality check

A new third-party validation route is available: Geonovum's OGC Checker CLI validates JSON-FG documents against JSON-FG 1.0.0. The repository documents the CLI invocation and its JSON-FG requirement coverage. It tests Core schema validity, metadata and several geometry/time requirements, but it does not test Core valid-geometry, cannot test axis-order, and does not dereference referenced feature schemas for the types-schemas requirement. These gaps mean its result must be described as independent partial conformance evidence, not complete Annex A coverage.

Sources:
- Geonovum OGC Checker: https://github.com/Geonovum/ogc-checker
- OGC JSON-FG 1.0.0 normative Annex A: https://docs.ogc.org/is/21-045r1/21-045r1.html

The current GDAL stable JSON-FG driver documentation says it implements a draft version of the standard. Its documented feature coverage includes feature type, CRS, feature-level time, and limited Polyhedron/Prism support. Therefore GDAL JSONFG is a candidate conversion route, not a JSON-FG 1.0.0 conformance claim.

Source: https://gdal.org/en/stable/drivers/vector/jsonfg.html

pygeoapi 0.24.0 documents GeoJSON as the default feature output; custom formatters are configured separately. A separate pygeoapi-plugins project documents a JSON-FG formatter. This should be treated as a candidate formatter requiring pinned integration tests, not built-in JSON-FG 1.0 conformance.

Sources:
- https://docs.pygeoapi.io/en/stable/publishing/ogcapi-features.html
- https://pypi.org/project/pygeoapi-plugins/

Sol-Atlas response: add Geonovum validation to CI at a pinned version, retain a negative control, publish adapter readiness as not-executed until evidence exists, and require full semantic round-trip plus field-disposition receipts before qualifying any adapter.


## 2026-11-28 update: raise the comparison bar beyond basic calibration

### Existing GIS capability is a serious baseline

ArcGIS Pro Geostatistical Analyst already exposes mean error, RMSE, average standard error, mean standardized error, root-mean-square standardized error, 90% and 95% interval coverage, and—where supported by Empirical Bayesian Kriging (EBK) models—average CRPS. It also distinguishes cross-validation from validation on a held-out subset. Therefore, Sol-Atlas must not claim that reporting RMSE, interval coverage, standardized residuals, or CRPS is itself a differentiator. The matched benchmark should include these metrics where the compared provider exposes predictive distributions, and should record when a method cannot produce uncertainty outputs ([Esri Cross Validation](https://doc.esri.com/en/arcgis-pro/latest/tool-reference/geostatistical-analyst/cross-validation.html); [Esri Cross-Validation and Validation](https://doc.esri.com/en/arcgis-pro/latest/help/analysis/geostatistical-analyst/performing-cross-validation-and-validation.html)).

A stronger contribution is to evaluate the *support and decision object* around those predictions: whether the same qualified support is used; whether the functional is fit for purpose; whether local/functional uncertainty remains calibrated under the deployment regime; whether restricted dependencies are withheld; and whether the recommended action remains robust as priors, likelihoods, costs, boundaries and utilities change.

### Validation regime is part of the performance claim

A 2026 tiered validation paper proposes random folds, a size-matched random control, coordinate clusters, nearest-neighbour graph communities, administrative holdouts and an external regional stress test. In its Baltimore case study, random validation was more optimistic than spatially separated and external-region tests; even coordinate clustering remained optimistic relative to external-region transfer. The authors also report repeated-seed variability, spatial autocorrelation and sensitivity to block scale ([Jiang, Wu & Liu, 2026](https://doi.org/10.1016/j.mex.2026.104072)).

Sol-Atlas's validation contract is now organized as T0–T5. The companion split-integrity auditor is being implemented in [PR #996](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/996). It validates partition metadata only; it does not generate folds or prove performance. The required reporting ladder is:

- T0/T1: random-mixture interpolation diagnostic and matched training-size control;
- T2/T3: coordinate-buffered and graph-community separation within the sampled region;
- T4: administrative-unit holdout;
- T5: external region and independent campaign challenge.

The benchmark receipt must state the highest tier actually run, dataset/support digests, sample counts, train/evaluation sites, buffer scale where applicable, seed variability, calibration and coverage, width/sharpness, residual spatial autocorrelation and worst-stratum performance. A lower-tier pass cannot be promoted into a higher-tier claim.

### Industry positioning: integrate the engines, qualify their outputs

| Existing capability | What it already does well | Sol-Atlas should add rather than duplicate |
|---|---|---|
| GIS / geostatistical platforms | Spatial data management, interpolation, cross-validation, prediction uncertainty and map products | Cross-provider support contracts, semantic claim qualification, evidence/rights lineage and decision-regret evaluation |
| Geological modelling platforms | Domain-specific geological interpretation, uncertainty ensembles and drillhole-planning workflows | Independent comparison of competing interpretations and action consequences across providers, with applicability and evidence receipts |
| Infrastructure digital-twin platforms | Federation of engineering, reality, GIS, IoT/sensor and enterprise/project data | Verify whether federated evidence is fit to justify a particular operational decision, with explicit uncertainty, support and rights constraints |
| Spatial ETL / conversion tools | Broad format and schema transformation | Multi-hop semantic preservation receipts, derived-field non-interference and machine-readable field-loss accounting |

This positioning avoids a weak "we are a better GIS" argument. Sol-Atlas should be a vendor-neutral qualification and investigation layer that accepts outputs from capable GIS, geology, metrology, reality-capture and digital-twin systems. It should only claim a measurable advantage where matched experiments show one.

### Additional interoperability target: columnar geospatial storage

GeoParquet 2.0.0, released 2026-11-05, is built around Parquet's native GEOMETRY/GEOGRAPHY logical types, CRS metadata on geometry columns and row-group spatial statistics. It is a valuable analytical-storage comparison, but not a replacement for JSON-FG feature exchange. Its geometry/CRS metadata does not by itself define Sol-Atlas evidence relations, claim contracts, rights or decision semantics. The adapter matrix now registers GeoParquet 2.0.0 as a candidate storage route, still not executed or qualified ([GeoParquet 2.0.0 specification](https://geoparquet.org/releases/v2.0.0/)).

### Proposed stretch targets for matched independent evaluation

These are prospective targets, not current achievements:

1. **Exchange integrity:** 100% of declared profile fields receive a terminal disposition per adapter hop; zero silent semantic loss; zero accepted rights escalation; zero unauthorized public-output changes under restricted-source mutation controls.
2. **Spatial decision advantage:** at matched evidence rights, support, stakeholder utility and acquisition budget, achieve at least 20% lower realized decision regret, *or* at least 20% lower acquisition effort at matched regret, against the strongest preregistered incumbent. Report confidence intervals and worst-site/stakeholder outcomes; fail the superiority claim if the independent sample cannot resolve the target.
3. **Uncertainty quality:** evaluate proper scores, interval coverage and width, worst-site/sensor/community performance and calibration drift across the declared T0–T5 regimes. Coverage without a predeclared useful-width ceiling is not sufficient.
4. **Investigation efficiency:** compare correlation-aware sequential EVSI and robust portfolio selection with random, highest-variance, highest-entropy, nominal-EVSI, expert and archive-first baselines. Report net regret reduction per unit cost, not only gross information gain.
5. **Operations:** report deterministic replay, p50/p95 latency, peak memory, throughput, storage, recovery and failure/abstention rates under fixed hardware and workload.

The 20% target is a stretch threshold for a prospective, powered independent study—not an estimate inferred from the synthetic fixtures. The benchmark must be allowed to falsify Sol-Atlas's superiority hypothesis.

### Comparator reading

- Esri, [Cross Validation (Geostatistical Analyst)](https://doc.esri.com/en/arcgis-pro/latest/tool-reference/geostatistical-analyst/cross-validation.html).
- Esri, [Using cross validation to assess interpolation results](https://doc.esri.com/en/arcgis-pro/latest/help/analysis/geostatistical-analyst/performing-cross-validation-and-validation.html).
- Jiang, Wu & Liu (2026), [A tiered spatial validation workflow for geographic prediction models](https://doi.org/10.1016/j.mex.2026.104072).
- [GeoParquet 2.0.0 release and specification](https://geoparquet.org/releases/v2.0.0/).
