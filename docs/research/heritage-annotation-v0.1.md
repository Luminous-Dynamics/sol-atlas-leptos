# HDB-21: Plural Archaeological Annotation and Adjudication v0.1

## Why this exists

Archaeological labels are interpretations of indirect observations, not automatically objective ground truth. A southern-Gauteng stone-wall study found high analyst variability for individual structures and higher agreement at regional scale. A 2026 archaeological micromorphology study surveyed experts, measured inter-rater reliability and recorded "Unsure" responses rather than treating the reference labels as infallible.

## v0.1 records

- Rater identity is an opaque or pseudonymous ID plus task-scope tags. No global expertise score is inferred.
- Every independent annotation retains task, sample, context, source digest, ontology version, rubric digest, rater, round, decision, alternatives, ordinal confidence, rationale digest and timestamp.
- Decisions distinguish assigned label, unsure, not observable and abstained. None is silently converted into a negative label.
- Repeated ratings by the same rater are permitted in distinct rounds and remain separate records.
- Adjudication references the exact source annotation set and preserves dissent; model visibility is explicitly recorded.
- Plurality is a derived record. A tie remains a tie, and all original ballots remain untouched.

## Agreement metric implemented

Fleiss' kappa is implemented only for complete, balanced nominal human panels within a single task, context and ontology. It refuses incomplete/abstaining panels, unequal ratings per item and repeated same-rater ratings within an item. It reports category prevalence, mean observed agreement, chance agreement and kappa. A missing/undefined kappa under perfect expected agreement is represented as None, not a fabricated score.

Fleiss' kappa is not a universal annotation metric. Ordinal categories require a defensible distance function; continuous measurements require suitable ICC/concordance analysis; geometry needs object matching and boundary metrics. This v0.1 module does not compute those other statistics or confidence intervals.

## Standards and research basis

- Southern-Gauteng stone-walled structure coder reliability: https://doi.org/10.1002/arp.1515.
- Archaeological micromorphology expert survey / inter-rater agreement / unsure responses (2026): https://ore.eu/ore/article/view/1339.
- Heritage Quest citizen-science consensus vs fieldwork at 380 locations: https://doi.org/10.15184/aqy.2024.127.
- Orengo et al. (2026), archaeological AI proxies, labels and validation: https://doi.org/10.1016/j.jas.2026.106571.
- Gattiglia (2025), risks of deterministic archaeological categories: https://doi.org/10.1016/j.culher.2024.11.020.

## Claim boundaries

Consensus is not truth. A majority vote can reproduce a shared conceptual bias. Kappa is design-dependent and prevalence-sensitive. The ordinal confidence field is not a probability. Rater references are not identity authentication. Model agreement with annotations does not replace independent field, material, stratigraphic or geophysical evidence.
