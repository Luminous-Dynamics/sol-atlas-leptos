# HDB-22: Survival- and Detectability-Aware Non-Detection v0.1

## Purpose

HDB-22 separates the processes that determine whether an originally present archaeological feature can produce a detection today.

For a target originally present, the v0.1 chain is:

P(detect | originally present) =
P(survival) × P(exposure | survival) ×
P(observability | survival, exposure) ×
P(footprint coverage | observable) ×
P(detection | survived, exposed, observable, covered).

This is a chain rule over explicitly conditional probability stages, not an assumption that the latent events are mutually independent.

## Bayesian update for one non-detection

The module uses:
- P(no detection | originally present) = 1 − P(detect | originally present);
- P(no detection | originally absent) = 1 − P(false positive | originally absent).

It updates the prior with Bayes' rule. Every probability is supplied as a calibrated lower/upper band and tied to context, target class, calibration profile, data, independent validation split, semantics contract and receipt. For a searched no-detection, the coverage band must carry the exact footprint receipt recorded by the observation. Method- and effort-conditioned bands must match the actual method and effort-profile digests.

The posterior range is a conservative outer interval over the rectangular ranges of the input bands. It is not automatically a confidence or credible interval. The calibration method must justify any stronger interpretation.

## Observation states

Only SearchedNoDetection triggers a negative-evidence update. Detected, NotSearched, NotObservable and Inconclusive remain records with explicit no-update reasons. A nominal planned survey is not evidence of spatial coverage; the actual footprint receipt is required.

## Repeated observations

v0.1 assesses exactly one survey event. It does not multiply repeated no-detections as independent evidence. Shared preservation, sensor, seasonal visibility, terrain, source imagery or crews can induce dependence. Combining repeated visits requires a separately validated joint/conditional dependence model.

## Integration guidance

- HDB-11 sensor probability of detection informs conditional detection/observability, subject to matching context and sensor.
- HDB-15 sweep-width and spatial-coverage receipts inform field survey effort/footprint terms. Avoid double-counting effective sweep width as both footprint coverage and conditional detection.
- HDB-18 calibrates metric measurement uncertainty; uncertainty in geometry may affect the footprint receipt.
- HDB-19 ensures survival/observability calibrations and final evaluation use independent spatial/site splits.
- HDB-20 records provenance/fixity/rights for calibration and survey receipts.

## Research basis

- Edwards et al. (2026), Wadi Quṣaybah: https://doi.org/10.1016/j.jas.2026.106568
- Banning et al. (2017), field survey detection/sweep width: https://doi.org/10.1007/s10816-016-9274-2
- Archaeological predictive-model inventory and sampling bias: https://pmc.ncbi.nlm.nih.gov/articles/PMC7529236/
- Bayesian statistics in archaeology: https://doi.org/10.1146/annurev-anthro-102317-045834

## Limitations

The v0.1 scalar model does not estimate survival/exposure/detection bands from raw field data, infer geomorphic process histories, or authenticate receipts. It does not combine repeated surveys, model spatial dependence, implement a hierarchical site model, or validate a real-world prior. A high posterior is not proof of existence; a low posterior is not proof of historical absence. Field, geophysical, stratigraphic, chronometric and excavation references remain separate validation tiers.
