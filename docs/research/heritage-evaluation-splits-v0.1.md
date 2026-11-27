# HDB-19: Spatially Independent Evaluation Split Manifests v0.1

## Purpose

HDB-19 records what a heritage-AI evaluation is actually testing. It preserves paper-faithful splits while separately screening source-pixel overlap, duplicated physical entities, site/sensor/epoch overlap, block assignments and spatial exclusion buffers.

## Profiles

- `paper_faithful_random_tiles`: reports overlap counts without rejecting the split. Its claim ceiling is paper-split reproduction only. The report distinguishes a supplied reference-manifest digest (`reference_manifest_digest_recorded`), a seeded protocol (`seeded_protocol_reproduction`) and an unseeded protocol reconstruction (`protocol_reconstruction`). A supplied hash is recorded but is not independently authenticated or compared with a canonicalized copy of the current manifest.
- `source_footprint_independent`: rejects source-pixel window intersections, duplicate source tile digests and physical identity crossing across train/validation/test. Different non-overlapping tiles from one source capture are permitted and reported.
- `buffered_spatial_blocks`: applies source-footprint checks, requires a common projected metric CRS, unique block IDs across roles and a buffer justified by an empirical correlation range or declared deployment distance. The v0.1 screen uses axis-aligned metric bounds, not a polygon engine.
- `leave_one_site_out`: requires declared held-out sites to occur only in test, no source pixel overlap, no duplicate tile digest and no physical-identity crossing.
- `cross_sensor_transfer` / `cross_epoch_transfer`: require declared sensor or epoch identities to occur in test only; these estimate different tasks from unseen-site generalisation.

## Leakage screens

- Parent acquisition pixel windows use half-open rectangle semantics: edge-touching windows do not overlap; positive-area intersections do.
- Source capture identity is bound to one SHA-256 digest within a manifest.
- Stable physical feature IDs are checked across partition roles.
- Axis-aligned metric bounds are screened for buffer separation in the buffered spatial profile.
- Buffer-excluded samples remain in the manifest and require an explicit reason.
- Split seed/protocol digests and dataset digest are recorded.

## Limitations

A matching seed does not prove random assignment. A source-window intersection screen does not detect all derived-feature leakage. An axis-aligned bounding-box gap is a conservative screen, not proof of polygon-level non-intersection. The validator cannot authenticate dataset custody, tile/source file digests, or the empirical correctness of a spatial autocorrelation range. The 2026 literature shows effective sample size can be much lower than pixel count under spatial dependence; choose block/buffer distance from the intended deployment estimand and empirical correlation diagnostics, not a universal constant.

## Research basis

- Uziel et al. (2026), JAS 185, 106430, DOI 10.1016/j.jas.2025.106430.
- Accuracy assessment recommendations on overlapping chips and map-level metrics: https://doi.org/10.3390/rs13132591.
- Karasiak et al. (2022), spatial dependence in remote-sensing accuracy: https://doi.org/10.1007/s10994-021-05972-1.
- Frontiers in Remote Sensing (2025), block selection based on autocorrelation range and intended use: https://doi.org/10.3389/frsen.2025.1531097.
- Young (2026), effective sample size and spatial block holdout bounds: https://doi.org/10.1109/TGRS.2026.3739158.
- Comer et al. (2022), archaeological predictive modelling and sampling strategy: https://doi.org/10.1016/j.jasrep.2022.103824.
