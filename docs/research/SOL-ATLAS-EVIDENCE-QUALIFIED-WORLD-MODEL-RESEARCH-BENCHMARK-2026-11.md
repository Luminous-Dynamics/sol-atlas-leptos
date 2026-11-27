# Sol-Atlas evidence-qualified world models — research and benchmark direction (2026-11)

## Executive position

Sol-Atlas should not compete with GIS, photogrammetry, HBIM, remote-sensing foundation models or digital-twin platforms by attempting to replace their strongest capabilities. It should become the **evidence and claim qualification layer that connects them**.

The proposed contribution is an evidence-qualified world model: an interoperable representation of physical and social systems in which geometry, relationships, time, uncertainty, provenance, rights, competing hypotheses and decisions remain distinguishable and inspectable.

This is a research direction and benchmark proposal, not a claim that Sol-Atlas currently exceeds commercial GIS, HBIM, remote-sensing or digital-twin systems.

## What the current field establishes

### 1. Heritage world models are an emerging architectural proposal

Pavlidis, Sevetlidis and Arampatzakis, *Towards Heritage World Models* (13 June 2026), propose extending semantic digital twins with prediction, memory, uncertainty-aware reasoning and intervention evaluation. It is a position paper, not a validated deployed system or a comparative superiority study.

Its research agenda is highly relevant: explicit unknown / contested / poorly observed states, auditable traces, action-conditioned predictions, multi-level evaluation, missing-data robustness, explanation quality, governance, human override and cross-site benchmarks.

Sol-Atlas should build the evidence and claim semantics needed to make those promises testable:
- https://doi.org/10.3390/heritage9060233
- https://www.mdpi.com/2571-9408/9/6/233

### 2. Geospatial foundation models are advancing under label scarcity

ESA Φ-lab's March–April 2026 archaeological-prospection pilot compared TerraMind and RemoteCLIP with U-Net and Vision Transformer baselines using seven-band Sentinel-1 / Sentinel-2 / elevation inputs. Its preliminary mini-dataset covered about 20% of the study area; the researchers identify full-area scaling, object-level metrics and an out-of-region Iran test as future work.

Implications for Sol-Atlas:
- compare foundation-model features with task-specific baselines on identical source packages;
- evaluate label efficiency at 100%, 80%, 50%, 25%, 10% and 5% where the corpus supports it;
- report pixel-, object- and site-level performance;
- hold out whole sites and regions, not random neighbouring tiles;
- report domain-shift degradation and abstention rather than extrapolating a local validation score.

Source: https://cin.philab.esa.int/databases/projects/geospatial-foundation-models-for-archaeological-prospection-tasks

### 3. Geometric quality is increasingly treated as a metrology problem

The 2026 cultural-heritage digital-twin metrology paper describes uncertainty-aware geometric assessment across heterogeneous acquisition systems, referencing GUM and geometric/surface-texture standards. The scan-to-HBIM literature also evaluates automation, geometric accuracy, semantic depth and expert refinement.

This supports a key design rule: a high-resolution mesh is not a measurement certificate. Sol-Atlas should bind acquisition conditions, registration, coordinate reference, datum, uncertainty representation and independent check measurements rather than equating polygon density with accuracy.

Sources:
- https://doi.org/10.1016/j.measurement.2026.122271
- https://doi.org/10.1016/j.autcon.2026.106848

### 4. Digital-twin composition and digital threads are formalizing

The ISO 23247 series now includes:
- ISO 23247-5:2026, Digital thread for digital twin (published June 2026);
- ISO 23247-6:2026, Digital twin composition (published July 2026).

Part 6 distinguishes integrated, unified and federated composition approaches. Sol-Atlas should prefer federated composition where specialist owners retain authority over source systems and domain models, while Sol-Atlas preserves the claim/provenance layer and shared cross-system references.

Sources:
- https://www.iso.org/standard/87425.html
- https://www.iso.org/standard/87426.html

### 5. Geospatial interoperability has a concrete standards path

Use existing standards rather than invent a competing transport format:
- OGC API – Features Part 1: Core corrigendum 1.0.1 for feature access;
- OGC API – Features Part 2: CRS by Reference 1.0.1 when non-WGS84 CRS handling is required;
- OGC API – Records Part 1: Core 1.0.0 for resource/metadata discovery;
- OGC Features and Geometries JSON (JSON-FG) Part 1: Core 1.0.0 for GeoJSON-compatible extensions, richer CRS and geometry typing.

Features, Records and JSON-FG solve different problems and should not be conflated. OGC also provides a conformance validator for Features Parts 1 and 2; Sol-Atlas should target formal conformance where the profile applies.

Sources:
- https://www.ogc.org/standards/ogcapi-features/
- https://ogcapi.ogc.org/records/
- https://www.ogc.org/standards/json-fg/
- https://www.ogc.org/announcement/ogc-releases-the-features-and-geometries-json-json-fg-standard/

### 6. Open data must not erase community authority

The London Charter requires users to understand whether a visualization depicts an existing state, evidence-based restoration or hypothetical reconstruction, and the nature and extent of factual uncertainty. It also emphasizes source provenance, paradata, method documentation and hypothetical dependencies.

For sensitive cultural and Indigenous data, FAIR-style discovery and interoperability are not sufficient by themselves. CARE principles add collective benefit, authority to control, responsibility and ethics. Sol-Atlas must model audience, purpose, jurisdiction, custodian and derivative-use restrictions; it must not treat a public citation or a dataset download as universal permission.

Sources:
- https://www.london-charter.org/principles.html
- https://www.london-charter.org/documentation.html
- https://www.gida-global.org/care
- https://doi.org/10.1038/s41597-021-00892-0

## Industry comparison: compete on a different axis

| Industry capability | Established strength | Sol-Atlas strategy |
|---|---|---|
| GIS platforms and spatial data infrastructure | Mature editing, analysis, cartography, spatial databases and standards-based services | Integrate through OGC Features / Records / JSON-FG; do not recreate a full GIS desktop |
| Photogrammetry and 3D reconstruction | Sensor processing, registration, mesh generation and texture quality | Consume source-native outputs; qualify acquisition lineage, uncertainty and held-out measurement accuracy |
| HBIM / heritage digital twins | Object semantics, asset lifecycle, inspection and maintenance workflows | Preserve epistemic state, provenance, counter-evidence, chronology and claim ceilings across exports |
| Earth-observation foundation models | Multimodal representation learning, segmentation, detection and label efficiency | Benchmark on site-held-out and region-held-out data; record domain shift, observability and abstention |
| Heritage world-model proposals | Dynamic state, predictive risk, memory and human-supervised decision architecture | Provide executable evidence/uncertainty/governance contracts and falsifiable benchmark cases |
| Institutional / community knowledge systems | Custodial authority, contextual interpretation and participation | Preserve plural accounts, contestation, access tiers, version history and withdrawal semantics |

The differentiator should not be a composite "world accuracy" score. It should be **claim-level reliability across systems that were not built by the same organization**.

## Paradigm shifts to implement

### A. From a single world state to a set of evidence-compatible worlds

Maintain a hypothesis ensemble where observations do not identify one reconstruction. Preserve hypotheses that remain observationally equivalent; never collapse them merely because one is visually cleaner or easier to render.

### B. From evidence lists to typed evidence relationships

Every claim should distinguish support, contradiction and context. Counter-evidence must be retained and visible without becoming positive support. Evidence identity, lineage, rights and derivation dependencies must survive serialization and cross-system projection.

### C. From confidence scores to calibrated, decision-relevant uncertainty

Store uncertainty in the space/time/variable where it belongs. Calibrate intervals against independent measurements. Report sharpness and coverage together. Uncertainty reduction is not automatically useful: the system should estimate whether a new survey or observation changes a decision.

### D. From a digital twin to a federated world-model composition

Specialist owners retain their authoritative models: Symtropy for physical simulation, Mycelix for rights / identity / attestations, Symthaea for hypotheses and investigation proposals, and Sol-Atlas for spatial/temporal evidence composition and visualization. No projection becomes a source of authority merely because it is rendered.

### E. From passive documentation to reversible, human-supervised investigation

The system can rank additional observations by expected decision value, recommend review, and preserve why it abstained. It must not autonomously authorize excavation, disclose restricted coordinates, grant title or execute physical intervention.

### F. From one benchmark to a public adversarial qualification suite

Publish matched tasks, independent holdouts, frozen inputs, baselines, negative controls, uncertainty coverage, rights-boundary tests and exact-head receipts. Report failures and unsupported claims alongside headline performance.

## HDB-12 benchmark suite

The following are proposed targets, not achieved results. Thresholds must be preregistered per reconstruction class and supported by enough independent validation units.

| Track | Measurement | Proposed target / gate |
|---|---|---|
| HDB-12A measured geometry | Median / 95th-percentile point-to-surface error, registration error, completeness, omission by feature class | Beat expert and conventional best-fit baselines at matched resolution and acquisition conditions; report absolute units and confidence intervals |
| HDB-12B uncertainty calibration | Empirical coverage at nominal 50 / 80 / 90 / 95% levels; interval width / volume; risk-coverage | For nominal 90% intervals, target 87–93% held-out site-level coverage where sample size is adequate; report 95% confidence intervals and stratify by site, feature, sensor and preservation state |
| HDB-12C hypothesis integrity | Recovery of known compatible hypotheses, false certainty, hypothesis-set coverage, evidence-removal sensitivity | 100% of registered non-identifiability controls retain all known compatible alternatives; zero false observed claims from simulated-only inputs |
| HDB-12D landscape relations | Network topology F1, connectivity, adjacency, catchment consistency, temporal-layer preservation | Compare with expert-authored and single-best-fit relation graphs on whole-site holdouts |
| HDB-12E evidence associations | Support / contradict / context relation integrity, counter-evidence retention, duplicate/overlap rejection | 100% rejection of registered relation swaps and association overlap; zero counter/context promotion into positive support |
| HDB-12F rights and disclosure | Rights non-escalation, coordinate access, purpose-bound derivative access | Zero registered disclosure or rights-escalation control failures |
| HDB-12G investigation value | Expected decision value, observation cost/delay, decision change, realized post-observation utility | Beat fixed survey schedules and uncertainty-greedy selection at matched budget; compare to a non-deployable oracle upper bound |
| HDB-12H interoperability | OGC conformance, round-trip preservation of CRS, geometry type, lineage and epistemic class | 100% retention of mandatory evidence semantics across supported round trips; no loss of restrictions or claim ceiling |
| HDB-12I reproducibility | Replay from source digests, method/build digest, parameters and seeds | 100% exact replay for deterministic tracks; stochastic tracks report bounded reproducibility and uncertainty |

### Required baselines

1. GIS / 3D best-fit workflow without explicit uncertainty or evidence relations.
2. Single maximum-likelihood reconstruction.
3. Expert-authored reconstruction with documented sources and paradata.
4. Bayesian or ensemble reconstruction with declared priors and posterior predictive checks.
5. Evidence-constrained ensemble with counter-evidence, dependence auditing, OOD abstention and value-of-information recommendations.

Compare on the same source package, units, CRS, resolution, holdout and compute budget. Do not allow photorealism, model size or render latency to compensate for unsupported historical claims.

### Holdout and leakage protocol

- Split by complete site / landscape / region and, where possible, acquisition period and sensor campaign.
- Withhold surveyed surfaces and dated artefacts from training, registration, prior selection and hyperparameter tuning.
- Keep annotation and adjudication teams blind to model outputs until scoring.
- Record duplicates, near-duplicates, overlapping tiles and derivative-source dependence.
- Publish per-site results and worst-stratum results; do not hide weak environments in a global average.
- Require an external adjudication panel for claims of historical interpretation, rights and cultural significance.

## First executable milestones

1. Complete HDB-12 support / contradiction / context association validation (child PR #981).
2. Add synthetic multi-world cases where two or more incompatible geometries explain the same observations.
3. Add independent geometry interval calibration and site-held-out scoring.
4. Add OGC JSON-FG / Features / Records round-trip fixtures that preserve epistemic and rights metadata through a sidecar/profile where the base standard has no native field.
5. Add an offline investigation tournament comparing expert schedule, fixed interval, uncertainty-greedy, decision-aware value-of-information and oracle-upper-bound policies.
6. Only then open a rights-cleared pilot with custodial approval, acquisition metadata and independent check measurements.

## Non-claims

This document does not establish that Sol-Atlas is SOTA, OGC-certified, ISO 23247-conformant, a validated heritage world model, or a qualified archaeological reconstruction system. Alignment with a standard or a published research direction is not certification. All proposed benchmark targets remain prospective until exact-source, independent, reproducible evaluations support them.
