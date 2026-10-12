# South African Mineral Prospectivity — Baseline Reproduction Audit

**Status:** Literature-derived preflight; source supplement not yet ingested  
**Reference:** Nwaila et al. (2024), *Natural Resources Research* 33, 2357–2384, [DOI 10.1007/s11053-024-10390-w](https://doi.org/10.1007/s11053-024-10390-w)  
**Scope:** Capture the published workflow accurately before claiming any reproduction or improvement.

## 1. Facts extracted from the article

The article reports:

- 38 covariates for national-scale South African mineral prospectivity mapping.
- 593 positive labels for the PGE–Cr–Ni–Cu target family and 1,073 positive labels for Witwatersrand-type Au.
- Inputs standardized to EPSG:4326 and integrated to H3 resolution 7. The paper describes about 5.16 km² mean cell area and roughly 1.22 km edge length.
- Regional Bouguer gravity station spacing of about 14 km.
- Regional magnetic data compiled from block surveys collected during the 1980s through late 1990s, with flight lines about 1 km apart and flight heights of 100–150 m; derivative/filter products can accentuate noise.
- The study's model space varied 13 feature-space dimensions, 8 algorithms, 2 model-selection metrics, and 5 sets of sampled negative labels: 13 × 8 × 2 × 5 = 1,040 candidate workflows per target family.
- The described training procedure withheld 25% of training data for testing and used four-fold cross-validation for model construction, guided by F1 or ROC-AUC. The paper also assessed geospatial spatial selectivity and used a variable-spatial metric for workflow/model merging.
- Point/vector interpolation methods varied by data type, including Gaussian processes, quadratic inverse-distance weighting, kernel density and triangulated irregular networks.
- The article acknowledges incomplete national geodata coverage, including insufficient coverage of national geochemical surveys for large-scale prospectivity mapping.
- Faults were assembled from multiple sources without deduplication because the authors considered the effect modest at national scale; that assumption should be stress-tested before any higher-resolution reuse.

These are **reported paper facts**. They are not evidence that this repository has independently reconstructed the 38 covariates, labels, or output maps.

## 2. Supplementary-material blocker

The publisher lists Supplementary Material 1 as a single PDF of approximately 142,100 KB and describes it as containing the supporting dataset compilation and prediction results. The article page says the supplementary materials are embedded in one PDF for convenience, with links inside the PDF for individual materials.

No copy of this PDF, extracted table, embedded file, or prediction output has been downloaded, decoded, hashed, or checked in this repository as of this audit.

### Reproducible extraction plan

1. Record the canonical publisher URL, access time, reported content length, HTTP metadata and source/license notice before downloading.
2. Download to a **non-repository working directory**; calculate SHA-256 and BLAKE3 digests and retain the original bytes unchanged. Do not commit the 142 MB PDF.
3. Inspect document metadata, page count, file/object structure, attachments, and URI/link annotations. Determine whether the data are structured embedded files, linked files, tables, figures, or a mixture; do not assume one format.
4. If attachments or linked assets exist, fetch only from the cited publisher destinations and inventory each item separately with rights and source metadata.
5. For PDF tables, extract candidate text/tables using reproducible tools and record their page/region, parser version, options, and extracted artifact hashes. Inspect representative pages visually when extraction quality is uncertain; report missing symbols, column alignment failures and OCR use explicitly.
6. Compare recovered headers, row counts, label counts, feature names, units and spatial IDs against statements in the paper. Manually inspect a stratified sample of extracted rows and all schema anomalies.
7. Preserve three distinct artifacts: immutable source PDF, raw extracted artifacts, and normalized analytical data. Normalization must never overwrite or masquerade as original material.
8. Publish only the derived data that the verified license permits. An article's CC BY 4.0 license may not apply to marked third-party material or source data governed by separate terms.

**Stop condition:** if the supplement does not expose machine-readable source tables or sufficiently documented data, classify M1 as blocked or partial and ask the corresponding data custodians/authors for the missing files. Do not rebuild approximate covariates and call the result an exact reproduction.

## 3. Reproduction mode must be distinct from improved validation

### Faithful reproduction mode

- Use the same target-family definitions and label semantics.
- Reconstruct the documented input set and feature processing as far as source assets allow.
- Preserve EPSG:4326 and H3 resolution 7 for comparison with the paper's grid products.
- Reimplement only documented workflow variations that can be determined from the article/supplement. Identify unspecified implementation details as unknown.
- Compare recovered output maps, labels and stated spatial/variable-domain metrics against the publication.

### Improved-evaluation mode

Do not silently overwrite reproduction choices with new choices. A second pipeline should:

- Use spatially grouped holdouts that keep an entire deposit, lobe, intrusion, goldfield or suitable geological district out of training, with buffer rules recorded in advance.
- Test whether any random cell-based holdout inflates scores relative to independent-region generalization.
- Keep unlabeled ground separate from verified barren ground; repeat experiments under multiple explicitly named unlabeled/negative sampling strategies.
- Run ablations for each data family and its derivative products, with shared upstream-source lineage recorded to avoid counting correlated derivatives as independent evidence.
- Report PR-AUC with prevalence; hit-rate/recall within the top 1%, 5%, and 10% of eligible valid area; fraction of eligible area nominated; enrichment over simple baselines; fold-to-fold dispersion; and separate measures of coverage, extrapolation, consensus and model disagreement.
- Compare calibrated probabilities only if calibration is fit without leakage and evaluated on an independent spatial test set.
- Preserve negative results and previously nominated areas that later fail review or field investigation.

The original article's 25% holdout/four-fold CV is part of the reproduction target. It is not sufficient by itself to demonstrate performance on a genuinely new geological district.

## 4. Target-specific spatial validation

### PGE–Cr–Ni–Cu / Bushveld Complex

Design spatial groups around geological units and intrusions (including appropriate lobe/satellite-intrusion groupings). The model should not receive near-duplicate cells around a known mineralized horizon in both train and test. Because the Bushveld is exceptionally well-studied, use it to test the benchmark and known-system recovery, not as sole proof of greenfield discovery ability.

### Witwatersrand-type Au

Separate producing goldfields and basin-scale settings into groups where feasible. Distinguish deposit labels from mines, mine infrastructure, production points, prospect occurrences and resource polygons. Avoid features or data products created after the event being predicted when running a temporal backtest.

For either target, choose holdout geometry before viewing the final predictions. If the available data do not support several meaningful independent regions, report limited validation power rather than repeatedly splitting neighboring H3 cells and treating folds as independent evidence.

## 5. Why deep-time simulation is not an M1 dependency

The first benchmark is a national-scale spatial integration and model-selection exercise. Adding a plate-reconstruction or geochemical simulation before reproducing the base evidence creates more assumptions without resolving the primary ingestion blocker.

GPlates/PyGPlates or a specialized geochemical/thermal solver may become useful for a declared hypothesis, but only in a later ablation where the reconstruction/solver version, temporal assumptions, physical parameters, and incremental predictive value can be evaluated independently. Symtropy's current generic rigid-body engine is not a substitute for a validated ore-genesis model.

## 6. M1 acceptance checklist

- [ ] Source PDF captured and hashed, or unavailable status documented.
- [ ] Attachments/links and source reuse terms inventoried.
- [ ] Extracted/raw/normalized artifacts separated and hashed.
- [ ] Target and covariate schemas documented; reported label counts investigated.
- [ ] CRS/H3/feature transformation behavior documented; true source resolution preserved.
- [ ] Missing inputs and uncertain implementation details enumerated.
- [ ] Reproduction classified as exact, partial, or blocked with concrete evidence.
- [ ] No "improved prediction" claim before independent spatial validation and baseline comparison.

## References

- Nwaila et al. (2024), article and supplement listing: <https://link.springer.com/article/10.1007/s11053-024-10390-w>
- DOI: <https://doi.org/10.1007/s11053-024-10390-w>
