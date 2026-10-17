# Spatial Overlay Adversarial Benchmark v0.1

**Status:** proposed benchmark contract; Rust matrix test authored, exact-head CI not yet verified.  
**Scope:** receipt interpretation and candidate-gate semantics only. This does not validate a geometry engine, source truth, land status, planning law, or engineering safety.

## Paradigm shift: test the decision boundary, not just examples

Example-based tests show that a few anticipated cases behave as expected. High-consequence site screening also needs an explicit model of how evidence can fail: coverage gaps, stale overlays, uncertain geometry, rejected sources, coordinate reference mismatches, and conflicting or missing inputs.

This benchmark treats **false clearance** as the primary safety failure. If a hard constraint has not been affirmatively cleared by admissible evidence, the candidate must not be labelled eligible. A high preference score cannot offset this gate. A spatial relation of `Disjoint` is not a finding of absence unless coverage, geometry validity, CRS alignment, source policy/state, accuracy, and freshness all support that inference.

The new Rust test `exhaustive_clearance_matrix_never_promotes_uncertainty_to_eligibility` enumerates the full product:

| Dimension | States |
| --- | --- |
| Spatial relation | intersects, disjoint, indeterminate |
| Source coverage | complete, partial, unknown |
| Spatial accuracy | adequate, insufficient, unknown |
| Source evidence state | verified, estimated, unknown, disputed, stale, restricted |
| Source policy | accepted, needs evidence, rejected |
| Geometry validity | valid, invalid, unknown |
| CRS | matching, mismatching |
| Overlay freshness | current, outside policy age |

This produces **5,832 deterministic combinations**. The test asserts the expected constraint finding and candidate gate for every combination. Under the current policy, only an accepted, verified, current, accurate, valid, CRS-aligned `Disjoint` result with complete candidate coverage can clear the single hard constraint. A high-quality intersection establishes presence even when source coverage is partial; uncertainty in other prerequisites prevents that inference.

## Promotion gates

These are targets for staged validation, not claims that the repository has already met them.

### Stage A — semantic safety

- **False-clear count: 0** across the generated matrix, pinned fixtures, mutation tests, and missing-receipt controls.
- **Unresolved-hard gate:** every missing, unknown, invalid, stale, insufficient, rejected, or indeterminate hard result yields `NeedsEvidence`, unless an independently confirmed hard intersection already excludes the candidate.
- **Permutation invariance:** reordering receipts or candidates must not change findings or gates.
- **Duplicate handling:** duplicate candidate/constraint receipts must fail explicitly; never select one arbitrarily.
- **Reproducibility:** the same policy and exact input digests produce byte-stable canonical output, once a canonical serialization contract is defined.
- **Negative controls remain present:** at least one case per failure mode must demonstrate that disabling the relevant guard causes the test suite to fail.

### Stage B — real geometry adapter

The current receipt test assumes the adapter's declared relation. It does not establish whether polygon operations are correct. The geometry adapter therefore needs a separate pinned corpus whose expected results are produced independently and reviewed.

Minimum initial corpus: 30 hand-reviewed geometry pairs covering boundary touches, contained polygons, disjoint polygons, holes, multipolygons, self-intersections, non-finite coordinates, near-coincident edges, narrow slivers and axis/CRS mistakes. Expand to at least 1,000 differential cases before a production pilot, with an independently generated or verified reference set.

- **False-clear count: 0** against the reference corpus.
- Boundary touch is conservatively a presence/intersection for a hard constraint; it must not become disjoint through rounding.
- Invalid or non-finite geometry is rejected or marked invalid, never repaired silently into an eligibility claim.
- No implicit CRS conversion or axis-order inference.
- Report false positives as well as false clears; conservative over-blocking has costs and should be visible rather than hidden.
- Pin geometry-library version, adapter version, CRS definitions, coordinate units, precision policy and input digests in each receipt.

### Stage C — scale and operations

Start with a measured baseline before adopting a latency promise. Benchmark separately:
1. receipt interpretation (no geometric operations),
2. vector geometry operations,
3. data ingestion/validation,
4. end-to-end candidate screening.

Run 10,000 candidate/constraint receipt pairs and representative polygon workloads. Record p50/p95/p99 latency, peak memory, throughput, invalid-input behavior, and hardware/toolchain. Do not combine the geometry-engine latency with receipt-evaluation latency or report a throughput number without the corpus and machine details.

## Adversarial mutation operators to add next

For each valid eligible fixture, mutate one attribute at a time and assert it cannot remain eligible where the mutation invalidates a required proof:

- change complete coverage to partial/unknown;
- age the overlay past policy expiry;
- change accepted/verified evidence to rejected, stale, disputed or unknown;
- introduce a CRS mismatch or invalid geometry;
- replace disjoint with indeterminate;
- remove a required receipt;
- alter the candidate geometry digest between two layer receipts;
- duplicate a candidate/constraint pair;
- reorder all receipts and candidate IDs;
- corrupt the declared digest or remove engine/version provenance.

These are counterfactual tests: “what single fact would make this candidate unsafe to clear?” They should be generated from positive fixtures so each guard has a demonstrated, reviewable failure if removed.

## Decision-level targets beyond geometry correctness

Correct GIS predicates are necessary but insufficient for correct site choices. Once the decision pipeline is integrated, measure:

- **Safety:** zero hard-constraint false clears in all pinned red-team suites.
- **Evidence coverage:** publish the fraction of hard constraints affirmatively cleared, not only a single suitability score.
- **Decision stability:** quantify shortlist/rank changes under plausible source updates, criteria weights, and measurement intervals.
- **Investigation value:** measure whether selected surveys can change an actual decision and whether the expected loss reduction exceeds the survey cost; never monetize non-compensable legal, rights, or safety duties.
- **Distributional impact:** report who gains and who bears costs across affected communities and groups; do not collapse these dimensions into a single weighted score without explicit authority and review.
- **Auditability:** any recommendation can be reconstructed from a versioned policy, source evidence, geometry receipt, candidate digest and decision output.
- **Abstention quality:** count both unsafe clearances and avoidable deferrals. A system that always says “unknown” is safe from false clearance but not useful; progress should reduce unjustified uncertainty without weakening the clearance rule.

## Standards and implementation anchors

- [OGC Features and Geometries JSON (JSON-FG) conformance tests](https://docs.ogc.org/is/21-045r1/21-045r1.html) explicitly test valid geometry and CRS axis-order requirements. Use standards-based conformance tests as inputs to the adapter corpus, not as proof that source data is factually correct.
- [OGC API - Features conformance test suite](https://cite.ogc.org/teamengine/about/ogcapi-features-1.0/1.0/site/) illustrates implementation-independent black-box testing for externally visible geospatial behavior.
- [GeoRust `geo`](https://github.com/georust/geo) provides planar geometry and topology operations; the adapter must still state its CRS/units assumptions and validate against independent references.
- [NIST AI Risk Management Framework FAQs](https://www.nist.gov/itl/ai-risk-management-framework/ai-risk-management-framework-faqs) and the [NIST AI Resource Center](https://airc.nist.gov/) emphasize testing, evaluation, verification, validation, validity and reliability. This benchmark applies that general discipline to the decision boundary; it is not an AI RMF certification.

## Current evidence boundary

- The 5,832-case matrix is generated deterministically by Rust test code; it is not a real-world parcel dataset.
- Existing 21-case JSONL overlay fixtures remain the small, inspectable reference examples.
- The independent JavaScript mirror previously matched those 21 fixture expectations, but that does not validate the Rust implementation or geometry operations.
- The workflow now has a dedicated `cargo test --manifest-path sol-atlas-core/Cargo.toml` step. The root app-binary test alone does not execute unit tests in the `sol-atlas-core` path dependency, so the explicit step closes a real test-discovery gap.
- The prior exact-head run passed native app tests and WASM compilation but failed formatting. Formatter changes, the 5,832-case matrix, and receipt-order invariance test have since been committed. The newest head's exact CI result remains unverified until the current run completes.
- Real GIS adapter work and its differential geometry corpus remain a separate requirement tracked in [issue #141](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/141).

