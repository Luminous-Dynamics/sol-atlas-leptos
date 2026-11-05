# HDB-20: Executable Stone and Wall Metrics v0.1

## Scope

HDB-20 implements two raster-mask evaluators:
- stone-instance matching: confidence-ranked deterministic one-to-one matching at a declared IoU threshold, with F1 at a declared operating confidence and 101-point interpolated AP;
- wall scoring: pixel-union precision/recall/F1 plus per-segment coverage by the union of opposite-class masks at all declared thresholds.

The evaluator pins dataset, model, model-build, predictions, labels, HDB-19 split, evaluator, metric protocol, rights receipt and evaluation unit metadata.

## Metric policy

### Stone instances

Predictions are sorted by confidence descending, with object ID as deterministic tie-breaker. Each prediction is paired with the unmatched GT instance having its highest IoU, provided IoU meets the configured threshold. One prediction and one GT instance can be used at most once.

F1 is computed after applying the configured operating confidence cutoff. AP101 uses all predictions in score order, computes precision/recall after each prediction, forms the precision envelope and averages interpolated precision at recall levels 0.00 to 1.00 in 0.01 increments.

The upstream size eligibility rule is recorded as an ID and description. The v0.1 metric engine does not infer centimetres from masks. For faithful reproduction of the source study's 15×15, 20×20 and 25×25 size rules, the preprocessing adapter must implement and verify the authors' exact size semantics.

### Walls

Predicted segments below the confidence cutoff are excluded. Pixel metrics are computed on the union of all active predicted segment masks and the union of all GT segment masks.

For a GT segment, coverage is the fraction of its pixels included in the prediction union. For a predicted segment, coverage is the fraction of its pixels included in the GT union. Multiple predicted fragments can therefore jointly cover one reference segment. Report coverage at every requested threshold (e.g. 0.7, 0.6, 0.5).

This is explicit v0.1 policy; it must be compared with the source authors' own implementation before any paper-equivalent score is claimed.

## Claim boundaries

Synthetic tests verify evaluator arithmetic and deterministic policies. They do not establish that the published dataset, model, preprocessing, split or annotations have been reproduced. The source artifacts and authors' evaluator must be independently inspected. Hash-shaped provenance fields are metadata only; the caller must compute and independently verify the real file digests. Rights receipt references are not authenticated here. Raster masks simplify polygon geometries and depend on the declared rasterization grid.

## Research basis

- Uziel, Zohar & Jaffe (2026), *Journal of Archaeological Science* 185, 106430: https://doi.org/10.1016/j.jas.2025.106430
- Dataset/code: https://doi.org/10.5281/zenodo.15673668
- ArcGIS Pro Detect Objects Using Deep Learning: https://doc.esri.com/en/arcgis-pro/latest/tool-reference/image-analyst/detect-objects-using-deep-learning.html
- ArcGIS object detection accuracy and postprocessing tools: https://doc.esri.com/en/arcgis-pro/latest/arcpy/image-analyst/deep-learning-geoprocessing-functions.html
- PIX4Dmapper outputs: https://support.pix4d.com/hc/en-us/articles/115002439503

## Next qualification work

- full Draft 2020-12 schema validation;
- test the score engine against a hand-calculated external oracle and the paper's official script;
- geometry-preserving polygon evaluator for source vector layers;
- AP50–95 and calibration / selective-risk analysis;
- per-site clustered bootstrap intervals;
- rights-aware acquisition adapter for the Zenodo files;
- paper-faithful and HDB-19 spatial transfer result receipts.
