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
