# Feature-weighted spatial fidelity for cultural heritage

**Status:** research and reference implementation v0.1  
**System owner:** Sol-Atlas  
**Related issue:** #688  
**External comparator:** SHREC 2026, "Reconstruction of high-frequency geometry in synthetic heritage", *Computers & Graphics* 139 (October 2026), article 104726, https://doi.org/10.1016/j.cag.2026.104726

## 1. What the new literature changes

The SHREC 2026 cultural-heritage track demonstrated that global Chamfer Distance can prefer smooth reconstructions even when they erase high-frequency details such as inscriptions, tool marks and relief. Its Feature-Aware Chamfer Distance (FACD) uses local geometric criticality to make these losses visible. The track compares eight reconstruction configurations, including three team submissions and five organizer baselines.

This is an important benchmark contribution, not a universal proof that one reconstruction family is best. The track's central lesson is that the metric defines what the optimizer is rewarded for preserving.

## 2. Complement FACD with semantic importance

Sol-Atlas adds a complementary reference metric: **Feature-Weighted Symmetric Chamfer (FWSC)**.

The difference is intentional:

- FACD derives feature criticality from local geometry.
- FWSC consumes frozen, externally authored importance weights on reference points.
- The weights can represent a conservator-, curator-, archaeologist- or community-identified feature whose significance is not reducible to curvature.
- Candidate reconstructions cannot choose their own weights.

Examples include an inscription, maker's mark, repaired seam, fragile edge, tool mark, mounting interface or a region designated sensitive by a community. The authority to annotate or disclose such a feature is separate from its geometric importance; Mycelix/CSCP rights and community permissions still govern capture, annotation, indexing, export and publication.

FWSC is not a reimplementation or replacement for FACD. The two metrics should be compared on identical reference data where the official FACD method and compatible fixtures are available.

## 3. Reference implementation

The renderer-agnostic module lives in `sol-atlas-core/src/spatial_fidelity.rs`.

Inputs:
- reference points in metres, each with a finite positive semantic-importance weight;
- reconstructed points in the same calibrated metric coordinate frame;
- one explicit distance tolerance in metres.

Outputs:
- reference-to-reconstruction and reconstruction-to-reference mean errors;
- global symmetric Chamfer;
- feature-weighted symmetric Chamfer;
- global precision, recall and F1 at the declared tolerance;
- feature-weighted precision, recall and F1 at that tolerance.

Both directional errors are returned so a pooled score cannot hide missing reference geometry or unsupported candidate surfaces. Weights are attached only to reference points; reconstructed points inherit the weight of their nearest reference point for the reverse direction.

The implementation is an O(n*m) deterministic reference oracle for modest fixtures. It deliberately does not implement registration, filtering, meshing, normal estimation, scanner calibration, rights evaluation or physical validation. Inputs must already be in the same calibrated coordinate frame. A spatial index and scalable point-cloud implementation are later optimizations, not part of this correctness baseline.

## 4. Evaluation protocol

Freeze semantic annotations before candidate evaluation. Publish the annotation rubric, annotator roles, disagreement, adjudication and any community-defined authority or access restrictions. Do not infer weights from candidate model confidence or tune them after viewing benchmark outputs.

Compare on identical capture conditions and reference geometry:

1. symmetric Chamfer Distance;
2. point-to-plane RMSE where calibrated normals exist;
3. F-score at declared metric tolerances;
4. official SHREC FACD where compatible;
5. FWSC with independent semantic annotations.

Use both:
- synthetic known-ground-truth reliefs to isolate reconstruction behavior; and
- real calibrated reference scans only where capture, derivative and publication rights are explicit.

Hold out objects, capture devices, sites, materials and feature classes. Group near-duplicate captures before splitting. Report feature-class and site worst groups, not only a pooled mean. Report view-synthesis metrics separately from metric geometry.

## 5. Proposed research targets (not achieved results)

At equal or lower capture cost:

- at least 10% relative reduction in FWSC versus the strongest matched open baseline;
- at least 5 percentage-point absolute improvement in feature F1 at a declared tolerance;
- no more than 3% relative regression in global Chamfer or point-to-plane RMSE;
- bootstrap confidence intervals and per-feature/per-site worst-group results.

These are stretch hypotheses. A benchmark result must identify the exact code/data/calibration/annotation digests and cannot be described as universal geometric superiority.

## 6. Next research step: uncertainty and multi-scale reporting

FWSC v0.1 does not yet calibrate pointwise uncertainty. The next version should add:

- per-region and per-scale errors;
- explicit occlusion and unobserved-surface masks;
- uncertainty coverage against independently measured reference error;
- inter-annotator reliability and feature-weight sensitivity analysis;
- robustness when semantic labels or local curvature are noisy.

Do not treat an unobserved surface as a measured failure. Do not let uncertainty inflate a claimed score without separately reporting the raw metric and the uncertainty-aware interpretation.
