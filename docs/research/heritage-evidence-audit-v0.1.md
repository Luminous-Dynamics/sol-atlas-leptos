# Heritage Evidence Audit (HEA) v0.1

**Purpose:** distinguish evidence that merely exists in a project from evidence that actually tested a specific candidate.

## Why this is needed

A recent systematic review of 132 UAV thermal/multispectral archaeological studies noted that ground-geophysical integration rose to 26.4% in 2021–2026, but explicitly warned that this is an upper bound on independent corroboration: reporting that magnetometry/GPR/ERT was collected does not prove it tested the same aerial anomaly. This is the most consequential distinction for an evidence-led system.

Reference: Ciccone, *Heritage* 9(10), 429 (2026-10-15), https://doi.org/10.3390/heritage9100429.

## Evidence semantics

- `same_candidate`: the evidence actually covers the same candidate geometry/feature instance, not merely the same region or project.
- `independent_measurement`: distinct data source and distinct method; an independent ground measurement is separately identified through its modality.
- `independent_interpretation_same_source`: a qualified different reviewer interprets the same underlying data. This is valuable, but is not independent physical corroboration.
- `shared_source`: a related analysis that reuses the same source/method; cannot count as an independent measurement.
- `supports`, `does_not_support`, `inconclusive`, `not_tested`, and `unavailable` are distinct outcomes.
- A project-level geophysics flag is not enough to count as candidate-level corroboration.

## Positive corroboration gate

A candidate is counted as independently corroborated only where an independent measurement:
1. uses a distinct source digest and a distinct method identifier from the remote-sensing detector;
2. is a ground-verification modality (field survey, magnetometry, GPR, electrical resistivity, excavation);
3. is explicitly related to the same candidate;
4. covers at least the configured fraction of that candidate's geometry;
5. has an explicit outcome of `supports`;
6. has reviewer/authority and rights-receipt references, with rights state `verified`.

Archival and community/custodial evidence remain important contextual evidence and must be represented, but do not become a geophysical measurement merely through inclusion in a single corroboration count.

## Negative-control gate

`does_not_support` only becomes an eligible negative control when the same candidate was actually examined by an independent ground-verification measurement, coverage meets the threshold, rights are verified, and a detectability assessment describes the target size/depth and survey sensitivity. Without that sensitivity evidence, a non-detection is not proof of absence.

`not_tested`, `unavailable`, `inconclusive`, nearby context, regional context and shared-source reviews are never negative controls.

## Report outputs

The audit reports candidate counts, evidence record counts, same-candidate ground measurement counts, independently corroborated candidates, eligible negative controls, same-source independent interpretations, context-only records, untested/unavailable/inconclusive records, and rights-not-verified evidence.

The report is not a quality score. It exposes evidence coverage and limitations rather than hiding them in one composite number.

## Limits

The core validates declared digests/references and the consistency of their relationships; it cannot prove that source digests map to authentic physical evidence, that reviewers have the stated qualifications, that a geometry overlap is correct, or that the stated rights are legally valid. These remain subject to independent audit and custodial authority. Precise site geometry remains protected.