# Sol-Atlas Spatial Investigation: Industry Benchmark Matrix v0.1

**Status:** research benchmark specification, not a product ranking or completed comparison  
**Parent:** [#988 — SPATIAL-LIKELIHOOD-QUAL-001](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/988)  
**Draft:** [PR #989](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/989)

## 1. Competitive question

The competitive question is not whether Sol-Atlas has more GIS features, more 3D rendering, more data formats, or another uncertainty metric. Existing products already perform substantial parts of this work.

The testable question is:

> Can a vendor-neutral spatial evidence and investigation layer compose specialist GIS, geostatistical, subsurface-modelling, reality-data and infrastructure-twin outputs into an independently reproducible, rights-aware, uncertainty-qualified decision process with lower realized decision regret or lower total investigation burden?

No product is presumed inferior. Feature presence is not evidence of workflow performance, and absence of a feature from public product literature is not evidence that the product lacks it.

## 2. Industry capabilities that must be represented fairly

This matrix is a scope baseline from public vendor documentation checked 28 November 2026. It does not claim that every feature is licensed, configured, available in every product tier, or suitable for every customer workflow.

| Industry system | Established capability to include | What Sol-Atlas must not claim as novel | Potential matched test |
|---|---|---|---|
| Esri ArcGIS Pro / Geostatistical Analyst | Ordinary/simple/universal kriging, EBK, EBK 3D, probability/indicator methods and areal interpolation; prediction, standard-error, threshold-probability and quantile surfaces; EBK cross-validation exposes CRPS and 90/95% interval coverage | Spatial interpolation, geostatistics, exceedance-probability surfaces, areal interpolation and common pointwise predictive diagnostics | Compare on identical spatial blocks and whole sites. Separately test pointwise surfaces, aggregate/functional uncertainty and independent block-support reference values; do not infer the latter from point CRPS alone |
| Seequent Leapfrog Works + Central | 3D geological modelling from boreholes and other ground evidence; Central 2026.2 project revisions, branches for alternative hypotheses, history, local-copy lineage and four permission levels (Owner, Editor, Viewer+Export, Viewer); published geological model objects include surfaces and volumes | 3D ground modelling, model revisions, alternative branches and collaborative access management | Compare exact model revision/support, alternative geological interpretations, uncertainty/unknown volume, evidence lineage and decision consequence; do not treat revision history as independent physical validation |
| Safe Software FME Form / Workbench | FME Form 2026.3 Data Inspector verifies 2D/3D geometry and location, attributes, format, schema, feature counts and process outputs; Workbench provides 500+ transformers across 3D, coordinate, data-quality, raster, point-cloud and spatial-analysis categories | Format conversion, geometry/attribute transformation, broad spatial ETL and intermediate-output inspection | Run pinned reader/transformer/writer paths on the identical evidence corpus; compare actual semantic preservation, CRS/units, relation/rights loss, geometry changes, replay and failure recovery |
| Bentley iTwin Platform | APIs for digital-twin management, synchronization/exports, reality capture/modeling, GIS/geospatial features, sensor data, schedules, reporting, review/issues, validation/clash detection, visualization and workflow automation; synchronization supports change tracking/ledger and transformations | Infrastructure digital-twin composition, engineering/reality/sensor data integration, synchronization, change tracking and visualization | Compare exact synchronization inputs/outputs, revision and change detection, model/asset identity, provenance and permissions; test whether Sol-Atlas adds decision-functional uncertainty and evidence admissibility rather than duplicating iTwin management |
| OGC standards and validators | Modular interoperable APIs and exchange standards such as OGC API - Features and JSON-FG; formal conformance classes and available validator routes | Compliance with an exchange format, API implementation, or independent document-level validation as such | Execute standards checks and multi-hop adapters; measure normative conformance separately from semantic preservation of Sol-Atlas-specific evidence and rights meaning |

Public references:
- Esri EBK and output surfaces (prediction, standard error, probability, quantile): https://doc.esri.com/en/arcgis-pro/latest/tool-reference/geostatistical-analyst/empirical-bayesian-kriging.html
- Esri supported geostatistical methods, including EBK 3D and areal interpolation: https://doc.esri.com/en/arcgis-pro/latest/help/analysis/geostatistical-analyst/kriging-in-geostatistical-analyst.html
- Esri cross-validation and validation (including EBK CRPS/90%/95% intervals): https://doc.esri.com/en/arcgis-pro/latest/help/analysis/geostatistical-analyst/performing-cross-validation-and-validation.html
- Esri cross-validation output: https://doc.esri.com/en/arcgis-pro/latest/tool-reference/geostatistical-analyst/cross-validation.html
- Seequent Leapfrog Works: https://www.seequent.com/products-solutions/leapfrog-works/
- Seequent Central: https://www.seequent.com/products-solutions/seequent-central/
- Seequent Central 2026.2 project revisions, alternative branches and permissions: https://help.seequent.com/Works/2026.2/en-GB/Content/central/central-projects.htm
- Seequent Central imports of revision-bound model surfaces and volumes: https://help.seequent.com/Works/2026.2/en-GB/Content/central/central-import.htm
- Safe Software FME Quick Translator / documentation: https://docs.safe.com/fme/2026.3/html/FME-Form-Documentation/FME-Form/QuickTranslator/Welcome_FME_QuickTranslator.htm
- FME Form 2026.3 Data Inspector: https://docs.safe.com/fme/2026.3/html/FME-Form-Documentation/FME-Form/DataInspector/AboutTheDataInspector/FMEDataInspector_about.htm
- FME Form 2026.3 transformer categories: https://docs.safe.com/fme/2026.3/html/FME-Form-Documentation/FME-Form/Workbench/transformers_locating.htm
- Bentley iTwin platform: https://developer.bentley.com/
- Bentley iTwin APIs (digital twin, reality capture, sensors, synchronization, validation, workflow automation): https://developer.bentley.com/apis/
- Bentley synchronization and export change tracking: https://developer.bentley.com/api-groups/synchronization/
- Bentley iTwin Synchronization API: https://developer.bentley.com/apis/synchronization/
- OGC JSON-FG: https://www.ogc.org/standards/json-fg/
- OGC API - Features: https://www.ogc.org/standards/ogcapi-features/

## 3. Where Sol-Atlas could differentiate

The proposed differentiator is the **composed evidence-to-decision contract**, not any one downstream solver or viewer.

A full investigation record should join:

1. Claim and decision: exact question, decision alternatives, utility/loss or hard constraints, decision owner and consequence class.
2. Evidence: supporting, contradicting and context-only records; source identity, provenance, time, spatial support and evidence freshness.
3. Measurement: sensor/instrument identity, calibration validity, observation envelope, uncertainty budget, coordinate frame and reference truth.
4. Model: method, input/output digests, version, assumptions, model discrepancy, alternative hypotheses and supported domain.
5. Rights: source-use purpose, actor, recipient, spatial/temporal disclosure scope, restrictions and derived-field dependency closure.
6. Investigation policy: eligible candidates, expected net decision value, robust sensitivity, correlated dependence, resource budget and stopping rationale.
7. Outcome: independently scored decision quality, realized regret, acquired evidence, and whether the observation changed the decision.

The hypothesis is that this contract reduces untraceable or overconfident cross-system decisions. It remains unproven until matched trials.

## 4. Matched experiment arms

Use the same admitted inputs, decision question, decision loss, budgets, access conditions, spatial support and blind truth across arms.

- **B0 — specialist-native workflow:** each specialist product performs its native task with its normal documented workflow.
- **B1 — specialist tools plus manual handoff:** native tools are connected by a documented human-led integration process.
- **B2 — Sol-Atlas composition:** the same specialist outputs are imported into Sol-Atlas; no specialist solver is replaced.
- **B3 — evidence/decision augmentation:** B2 plus qualified observation likelihoods, rights-aware derivation, correlation-aware EVSI and robust policy analysis where all required inputs are admitted.
- **B4 — independent finite oracle:** a separately implemented evaluator for bounded synthetic known-answer cases; for physical cases, the evaluator uses blinded reference truth and preregistered scoring.

Do not compare a fully licensed commercial product operated by an expert with a partial research adapter operated by its author and present that as a product-level ranking. Record operator expertise, configuration, enabled modules, training, elapsed time, compute, licenses and unsupported workflows.

## 5. Workflow-specific scorecards

### A. Spatial prediction and uncertainty
Compare ArcGIS-supported interpolation/EBK baselines and appropriate open geostatistical baselines on identical spatial-block and whole-site holdouts.

Report MAE/RMSE as secondary descriptive metrics, plus log score/CRPS where the predictive distribution is available, interval coverage and width, standardized residual diagnostics, worst-site/sensor/regime coverage, selective risk and cluster-level uncertainty. Use conformal baselines only when the exchangeability/mixing assumptions and calibration split are stated.

### B. 3D subsurface interpretation
Compare Leapfrog Works workflows with the Sol-Atlas evidence composition layer on identical admitted borehole, geophysical and reference data. Report geometry/stratigraphic discrepancies, unobserved volume, alternative-model sensitivity, decision-critical uncertainty, provenance completeness and the evidence used to revise the model.

A visual match or plausible geological interpretation is not a physical validation result. Physical truth requires qualified reference methods and domain review.

### C. Spatial ETL and semantic round-trip
Use FME and open-source GDAL/OGR/GeoServer/pygeoapi routes as adapter comparators. Freeze tool and plugin versions, source bytes, projection/axis order, geometry types, temporal roles, units and input manifests. Compute the loss ledger from actual output bytes and independently parse the outputs.

Report:
- byte equality where byte preservation is expected;
- semantic field retention by declared field identity;
- geometry displacement/topology/validity changes;
- time, CRS, axis-order and unit changes;
- evidence-link and provenance retention;
- rights/visibility leakage through direct and derived fields;
- unsupported/unassessed fields rather than silently ignoring them;
- execution and recovery receipts.

JSON-FG validation is one conformance layer, not a proof of complete semantics or OGC certification.

### D. Infrastructure twin and change workflow
Compare the integration patterns supported by iTwin and other available owner tools using the same engineering/reality/sensor revision set. Measure synchronization delay, changed-object detection, revision binding, provenance, authorization, evidence freshness, reproducible replay and operator intervention.

Do not claim iTwin lacks these capabilities; test the exact workflows and extension points.

### E. Evidence-to-investigation decision
This is the main cross-stack contest. Compare:
- expert survey plan;
- fixed grid / fixed modality;
- random eligible plan;
- nearest/cheapest eligible plan;
- entropy/EIG greedy;
- each vendor's native decision workflow where available and configured;
- plug-in EVSI;
- posterior-predictive/correlation-aware EVSI;
- robust EVSI;
- clairvoyant constrained upper bound.

Primary outcome: realized decision regret at equal total eligible acquisition budget. Secondary outcome: total acquisition effort/cost at matched regret. Preserve hard constraints as non-compensatory. Report regret distributions, tail outcomes, policy switches, calibration burden, number of observations, elapsed time and compute.

## 6. Cross-system integrity challenges

The shared blind corpus should include:
- CRS and axis-order ambiguity;
- vertical datum and coordinate epoch mismatch;
- resampling / generalization / minimum mapping unit shifts;
- changed source, sensor, configuration or processing revision;
- stale or expired calibration;
- partial/no-data/occluded support;
- correlated repeated observations;
- common-cause dependencies hidden behind different sensor/vendor IDs;
- contradictory evidence and alternative hypotheses;
- source restriction that propagates through public claims/geometry/metadata;
- rights expiration/revocation after initial ingestion;
- adapter field loss, unsupported geometry or changed temporal semantics;
- interrupted exchange, partial writes, stale snapshots and replay/recovery;
- duplicated/derived data presented as independent corroboration.

Positive controls are mandatory: an always-abstain system, a permanently withheld projection and a converter that refuses every adapter route must not pass.

## 7. Primary outcomes and gates

Hard gates:
- zero unauthorized release in the defined test channels;
- zero silent semantic loss in the declared field inventory;
- 100% explicit accounting of every field as preserved, transformed, withheld, unsupported or not assessed;
- zero false promotion of no-data/no-detection/OOD to absence;
- zero false independence claims on shared-cause fixtures;
- 100% exact known-answer agreement on the frozen bounded oracle corpus;
- no unresolved required measurement/rights gate hidden by a weighted score.

Comparative targets (hypotheses, not achieved):
- >=20% lower out-of-sample realized expected regret at equal eligible acquisition budget; or
- >=20% lower total acquisition effort at matched regret;
- lower manual evidence reconciliation time without reducing reviewer comprehension;
- complete replay of the declared evidence frontier in an independent environment;
- better worst-stratum calibration/coverage without unacceptable interval inflation.

Set sample size and effect thresholds from an independently estimated baseline variance and power/precision analysis. Do not tune metrics after seeing the held-out outcome.

## 8. First feasible tournament

Start with one bounded, retrospective site-investigation decision where:
- lawful source and output rights are documented;
- a complete set of native input bytes and tool versions can be frozen;
- independently surveyed reference truth exists with an uncertainty budget;
- specialist software access is actually available;
- the site/campaign split can be blinded;
- the decision and hard constraints are approved by the relevant competent domain owner.

Run a synthetic known-answer corpus first, then a retrospective campaign, then a prospective permissioned campaign only after the physical and governance gates are met.

No public data catalogue entry, vendor product description, synthetically generated receipt or benchmark pass grants land access, permission to excavate, utility-marking authority, engineering certification or legal determination.

## 9. Interpretation rule

Results must be reported by workstream and qualification tier. A tool can win one component and lose another. No single combined leaderboard score should erase:
- unsupported claims;
- rights/authority violations;
- measurement under-coverage;
- excessive abstention;
- high false discovery;
- operational overhead;
- unreproducible adapter routes;
- or failure on critical site/sensor strata.

A credible outcome may be that ArcGIS is better at a given geostatistical task, Leapfrog is better at geological model authoring, FME is better at a translation workflow, and iTwin is better at an infrastructure synchronization workflow—while Sol-Atlas adds value as an evidence and decision composition layer. That is a stronger and more defensible position than pretending to replace these specialist systems.
