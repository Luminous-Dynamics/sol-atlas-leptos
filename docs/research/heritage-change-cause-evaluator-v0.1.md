# Heritage Change-Cause and Abstention Evaluator v0.1

**Scope:** evaluate cause hypotheses for a detected change candidate. This is not a change detector and does not determine heritage significance.

## Problem

An anomaly is not a cause. Archaeological excavation/looting must be separated from agriculture, seasonal vegetation, erosion, hydrology, fire/clearance, modern construction, quarrying, conflict-related change, sensor/processing artefacts and other known causes. Several causes may coexist. When evidence is inadequate the correct action is abstention, not a forced class.

## Evaluation semantics

- CompleteCauses: independently adjudicated and exhaustive within the v0.1 taxonomy; one or more labels.
- ConfirmedNoRelevantChange: reviewed negative control; every taxonomy cause is negative for the evaluated sample/window.
- Unknown: insufficiently investigated or unresolved; never converted to a negative.
- Predictions contain one probability per taxonomy class and an explicit abstention flag.
- Any asserted cause at or above the threshold must include at least one evidence reference.
- Macro F1, per-class metrics and Brier score are conditional on non-abstained fully labelled samples. Coverage and abstentions are reported separately.
- Archaeological false-claim rate is a first-class metric: false looting/excavation assertions per 100 covered labelled candidates.

## Why the separation matters

Recent heritage condition work reported a phenology-aware temporal change detector with less than 8% false alarm rate under cross-season pairings in its three-site study, compared with 15–34% for selected baselines. These results concern a specific UAV landscape-heritage setting, not an archaeological looting benchmark. They nonetheless reinforce the need to measure seasonal pseudo-change and task-specific failure modes instead of treating all anomalies as heritage threats.

2026 archaeological computer-vision best-practice and AI-methodology papers emphasise archaeologically informed validation, reproducible source/preprocessing/training/post-processing, explainability, transfer and ethical responsibility. The HCCA protocol makes this operational by keeping cause and significance separate.

References:
- https://www.nature.com/articles/s41598-026-63132-8
- https://doi.org/10.1016/j.jas.2026.106577
- https://doi.org/10.1016/j.jas.2026.106571
- https://doi.org/10.1016/j.culher.2026.06.026

## v0.1 limits

The taxonomy is a starting point, not a universal ontology. Local and Indigenous categories, language and context must remain available. A label is meaningful only with provenance and appropriate reviewer authority. The evaluator does not verify the authenticity of adjudication references, source imagery, or rights.

Heritage significance and intervention remain the responsibility of qualified archaeologists, custodians, descendant/Indigenous communities and heritage authorities. A high cause probability is not proof of a crime and must not automatically trigger a public alert or field intervention.