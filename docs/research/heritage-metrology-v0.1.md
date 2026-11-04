# HDB-18: Traceable Metrology and Covariance-Aware Aggregation v0.1

## Purpose

HDB-18 closes two gaps in the HDB-17 architectural measurement graph: a calibration ID alone is not evidence of accuracy, and repeated observations sharing an image/calibration/model do not have independent errors by default.

## Implemented in the first Rust slice

### Independent checkpoint assessment

A metrology certificate records project/site/snapshot scope, CRS, reference survey identity, method-build digest, capture IDs, control point IDs, independent checkpoints, per-checkpoint horizontal and vertical reference survey uncertainty, strata, rights receipt and independence attestation.

The assessment computes product-fit RMSE components and checkpoint-survey components separately, then combines them using the ASPRS quadrature model:

- RMSEH1 from horizontal product-reference residuals;
- RMSEV1 from vertical product-reference residuals;
- RMSEH2 and RMSEV2 from supplied checkpoint-survey accuracy;
- RMSEH = sqrt(RMSEH1² + RMSEH2²);
- RMSEV = sqrt(RMSEV1² + RMSEV2²);
- RMSE3D = sqrt(RMSEH² + RMSEV²).

Control/checkpoint identity overlap is rejected. The `asprs2024_core` profile requires at least 30 independent checkpoints and requires the declared checkpoint survey accuracy to be at least twice as accurate as the declared horizontal and vertical product targets. The report explicitly identifies itself as core statistics only—not full ASPRS conformance. A separate target status is computed from the combined RMSE values. Checkpoint counts by declared stratum are retained.

### Covariance-aware additive area

The initial model accepts only disjoint, calibrated 2D plan-footprint areas and requires an external disjointness receipt. It computes:

`A = Σᵢ Aᵢ`

`u²(A) = Σᵢ uᵢ² + gᵀ R g`

where `gⱼ = sⱼ Σᵢ aᵢⱼ`, `sⱼ` is a shared factor's standard uncertainty, `aᵢⱼ` is its signed component sensitivity, and `R` is the declared correlation matrix between shared factors.

The implementation checks matrix dimensions, symmetry, diagonal, bounds and positive semidefiniteness. Missing sensitivities, duplicate factors/components, invalid areas, missing disjointness evidence and invalid correlation matrices fail closed.

The report gives standard uncertainty only. It does not invent a 90%/95% interval or probability distribution.

## Research basis

- ASPRS Positional Accuracy Standards, Edition 2, Version 2 (2024); USGS summary: https://www.usgs.gov/ngp-standards-and-specifications/adopt-updated-accuracy-standards
- ISO 19157-1:2023: https://www.iso.org/standard/78900.html
- Altadonna, Campisi & Di Santo (2026), scale-constrained multi-source archaeological photogrammetry at Tindari: https://doi.org/10.5194/isprs-archives-XLVIII-2-W12-2026-1-2026

## Claim boundaries

A structurally valid certificate does not authenticate the surveyor, instrument calibration, reference network, signatures, rights or checkpoint independence. A residual report does not prove fitness for a particular archaeological question.

The factor model assumes independent component-specific residuals and uses only the explicit shared-factor covariance matrix for correlated systematic effects. Nonlinear GIS unions, overlapping geometries, 3D surface aggregation, unknown covariance and Monte Carlo distributional validation are not implemented.

## Next qualification work

- JSON Schema conformance using a dedicated Draft 2020-12 validator (Rust fixture tests currently ensure parseability, not full schema conformance).
- Monte Carlo reference tests for factor-model uncertainty.
- Cross-check residual summaries against a geodetic/photogrammetric reference implementation.
- Bind validated certificate digests into HDB-17 metric observation references.
- Independent field validation against survey checkpoints.
