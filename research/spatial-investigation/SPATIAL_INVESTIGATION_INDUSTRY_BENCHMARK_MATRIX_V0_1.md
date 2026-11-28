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
| Esri ArcGIS Pro / Geostatistical Analyst | Empirical Bayesian Kriging and other geostatistical workflows; cross-validation and held-out validation; standardized errors; 90/95% interval coverage; CRPS and validation quantiles for supported models | Spatial interpolation, predictive standard errors, cross-validation, and some probabilistic validation | Compare identical held-out spatial blocks/sites; evaluate predictive score, calibration, interval width, worst-stratum coverage and reproducible outputs |
| Seequent Leapfrog Works + Central | 3D geological ground models from borehole/site data; dynamic model updates when new data arrives; geophysics and GPR context; contaminant geostatistics and hypothesis alternatives; Central project revisions, branching, access permissions and audit history | 3D subsurface modelling, hypothesis alternatives, model revision history, data updates and collaborative review | Compare a bounded site-investigation workflow: data admission, model update, alternative interpretation, uncertainty disclosure, evidence used, revision trace and decision consequence |
| Safe Software FME Form / Workbench | Broad format translation; geometry and attribute transformation; spatial ETL; inspection of geometry, attributes, formats, schemas, feature quantities and process outputs | Generic format conversion, geometric/attribute transforms, data inspection or a large connector catalogue | Run the same frozen corpus through FME reader/transformer/writer routes; compare actual byte/semantic retention, geometry error, field-loss accounting and reproducibility |
| Bentley iTwin Platform | APIs/services connecting engineering design data, reality data, GIS, IoT/sensor data, schedules, operational and enterprise data; digital-twin management, synchronization, access control, change tracking, reporting and visualization | Infrastructure digital-twin integration, access control, change tracking, reality-data integration and multi-source visualization | Compare an infrastructure change/investigation workflow across source synchronization, exact revision binding, evidence provenance, permissioned projections and replay |
| OGC standards and validators | Modular interoperable APIs and exchange standards such as OGC API - Features and JSON-FG; formal conformance classes and available validator routes | Compliance with an exchange format, API implementation, or independent document-level validation as such | Execute standards checks and multi-hop adapters; measure normative conformance separately from semantic preservation of Sol-Atlas-specific evidence and rights meaning |

Public references:
- Esri cross-validation and validation: https://doc.esri.com/en/arcgis-pro/latest/help/analysis/geostatistical-analyst/performing-cross-validation-and-validation.html
- Esri cross-validation output: https://doc.esri.com/en/arcgis-pro/latest/tool-reference/geostatistical-analyst/cross-validation.html
- Seequent Leapfrog Works: https://www.seequent.com/products-solutions/leapfrog-works/
- Seequent Central: https://www.seequent.com/products-solutions/seequent-central/
- Seequent Central project histories/permissions: https://help.seequent.com/Works/2026.2/en-GB/Content/central/central-projects.htm
- Safe Software FME Quick Translator / documentation: https://docs.safe.com/fme/2026.3/html/FME-Form-Documentation/FME-Form/QuickTranslator/Welcome_FME_QuickTranslator.htm
- FME Data Inspector: https://docs.safe.com/fme/html/FME-Form-Documentation/FME-Form/DataInspector/AboutTheDataInspector/FMEDataInspector_about.htm
- Bentley iTwin platform: https://developer.bentley.com/
- Bentley iTwin APIs: https://developer.bentley.com/apis/
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
