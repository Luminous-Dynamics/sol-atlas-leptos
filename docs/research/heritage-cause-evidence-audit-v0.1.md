# Heritage Cause Evidence Audit (HCEA) v0.1

**Scope:** audit provenance, event specificity, rights, support/contradiction polarity and source independence for a set of competing cause hypotheses. This is not a causal inference engine or archaeological adjudication.

## Why this is distinct from classification

The existing cause evaluator scores 11 causes as multi-label probabilities, separates unknown labels and abstentions, reports Brier/F1 and tracks false archaeological claims. Its evidence references are intentionally lightweight. HCEA adds the evidence structure needed to audit *why* a cause is proposed and what evidence argues against it.

Archaeological change can be caused by agriculture, seasonal vegetation, erosion, hydrology, fire, construction, quarrying, military activity, sensor artifacts or looting. A robust system must preserve alternatives and counter-evidence rather than forcing every anomaly into one of those explanations.

## Evidence record

Every evidence record binds to one event and one cause, and carries:
- support or contradiction polarity;
- source asset SHA-256, source reference, method ID and independence group;
- modality and evidence tier;
- same-event or context-only scope;
- rights state and receipt;
- reviewer/authority references where the evidence tier requires them;
- explicit non-detection status and detectability assessment.

## Independence policy

Independent support/contradiction requires all of:
1. same-event scope;
2. verified rights and a receipt;
3. field-assessed, independently expert-reviewed or instrument-observation tier;
4. a source digest not already counted for that cause/polarity;
5. an independence group not already counted for that cause/polarity.

This is deliberately conservative. The same raw raster interpreted twice is one source. Another tile or asset in the same acquisition group does not create a second independent line. Model-derived, catalogue-only, context-only and rights-unverified records remain visible but do not increase independent support.

## Negative evidence

A non-detection must carry a detectability assessment. It cannot contradict a cause unless the relevant survey could detect the expected target under the actual size/depth, sensor and field conditions. This record only checks that an assessment reference exists; it does not validate that the assessment is scientifically correct.

## Report semantics

- `asserted`: model probability meets threshold and the model did not abstain.
- `has_event_specific_support`: at least one independent same-event support record exists.
- `unsupported_asserted_cause_count`: asserted hypotheses with no independent same-event support.
- `contested`: both support and contradiction records exist; the system does not majority-vote or discard the contradiction.

An asserted cause with no independent support is reported as unsupported, not converted into an adjudicated cause. Human archaeologists and custodial authorities retain decision authority.

## Tests

Tests cover independent source/group counts, repeated source digests, shared acquisition groups, same-source interpretation, context-only evidence, rights-pending evidence, support plus contradiction, unsupported assertions, detectability-qualified non-detections, event mismatch, unresolved/wrong-polarity/duplicate references, rights receipts, SHA-256 validation and reviewer/authority requirements.

## Benchmarks

- Evidence precision and contradiction recall on independently adjudicated events.
- Double-count robustness under duplicate image chips, repeated model runs and alternate interpretations of the same raster.
- Evidence-removal tests: removing all support must render the claim unsupported.
- Counterfactual hard negatives across agriculture, phenology, erosion, hydrology, construction and sensor artifacts.
- Cause Brier/reliability and abstention risk-coverage remain evaluated by `heritage_cause.rs` separately.

## Claim ceiling

HCEA audits the structure of the evidence bundle. It cannot prove source authenticity, geometry overlap, independence-group truth, reviewer competence, custodial authority, rights interpretation or survey sensitivity. It does not infer cause, assess severity or issue an accusation.