# Landscape cross-repository chain conformance v0.1

**Status:** executable synthetic structural-conformance proposal. It is not a physical-model qualification, ecological outcome evaluation, or field-study result.

## Why this benchmark exists

The landscape program is intentionally split across system boundaries:

- **Sol-Atlas:** versioned project geometry, evaluation plans, source metadata, and observations.
- **Symtropy:** scenario admission and model outputs.
- **Symthaea:** read-only comparison/review with explicit abstention when evidence cannot justify a ranking.
- **Mycelix:** consent-aware stewardship, authority claims, and participant-governed records.

The upstream drafts already define these pieces separately. This conformance increment tests whether they actually reference the **same exact project revision** and whether unsupported claims remain blocked as records move between repositories.

## Research finding and competitive posture

The Landscape Architecture Foundation's guide offers more than 100 candidate metrics across 33 benefit categories, emphasizing selection of methods tailored to each project's goals. Its Landscape Performance Series includes over 200 case-study briefs. This is a useful incumbent reference, not something a new schema should claim to replace. See https://www.landscapeperformance.org/guide-to-evaluate-performance and https://www.lafoundation.org/what-we-do/research/landscape-performance.

A 2025 review of 862 urban ecosystem-service publications identified roughly 47 methods, 1,130 equations and 1,190 parameters, and reported that more than 60% of reviewed studies did not specify parameter sources or values. That is a strong reason to make input provenance and method/version traceability testable contracts rather than prose-only expectations. See https://doi.org/10.1016/j.landurbplan.2024.105215 and https://www.sciencedirect.com/science/article/pii/S0169204624002147.

OGC SensorThings API Sensing 1.1 supplies a relevant interoperable observation-model baseline; STAplus addresses user-owned observations. This project does not reimplement those standards or activate their Tasking/actuator capabilities. The current Sol-Atlas draft contains a read-only Sensing adapter. See https://docs.ogc.org/is/18-088/18-088.html and https://www.ogc.org/standards/sensorthings/.

The intended differentiator is not merely drawing, simulation, or a total score. It is a reproducible, provenance-bound design → evaluation plan → observation → scenario → stewardship → review chain with principled abstention, immutable references, and testable claim boundaries. Whether this improves design outcomes remains an empirical question.

## Implementation

The standalone Rust package `tools/landscape-chain-conformance/` reads the frozen artifacts in `fixtures/landscape/chain-conformance-v0/` and constructs a digest-linked, synthetic chain. It records both raw-byte digests (source project, alternative, source catalog and metric registry) and canonical-JSON digests for derived artifacts. The report includes all artifact digests so a consumer can reproduce the chain locally.

The benchmark checks:

1. Distinct baseline and alternative snapshot identities.
2. Exact source bytes, schema IDs, scope, coordinate frame and georeferencing status.
3. Evaluation-plan linkage to the precise project, scope and intervention/comparison site-reference bytes.
4. Observation linkage to the project digest and an existing feature ID, preserving the synthetic evidence class.
5. Scenario linkage to design bytes, boundary-condition bytes and normalized evaluation-plan bytes.
6. Mycelix linkage to the same project and scenario, with private/no-coordinate-disclosure defaults and no claimed publication consent.
7. Symthaea linkage to the same baseline and alternate design, metric registry, source catalog and scenario; consequence-bearing metrics remain blocked, the scenario result remains unexecuted, and the review abstains from ranking.
8. Seven negative controls: stale project digest in an observation, scope mismatch, missing feature ID, synthetic-to-observed relabel, public exact-sensitive-location exposure, scenario digest disagreement, and a one-byte snapshot mutation.

## Reproduction

```sh
cargo fmt --manifest-path tools/landscape-chain-conformance/Cargo.toml -- --check
cargo test --locked --manifest-path tools/landscape-chain-conformance/Cargo.toml
cargo run --locked --manifest-path tools/landscape-chain-conformance/Cargo.toml
```

The default CLI uses the frozen fixture directory relative to the Cargo package. An explicit fixture-directory argument can be supplied for future fixture variants.

## Target benchmark ladder

- **Contract integrity:** zero broken links and zero accepted negative controls across the frozen fixture suite.
- **Cross-repository mutation coverage:** add single-field and one-byte mutations for every binding edge; require each injected stale/mismatched reference to be rejected by the intended invariant.
- **Exact replay:** byte-identical reports for identical input bytes and pinned executable/toolchain; all inputs and outputs have SHA-256 identities.
- **Throughput:** measure load, validation, normalization and report emission for 10, 100, 1,000 and 10,000 observations/features; report raw p50/p95 and memory on pinned hardware. These are software-integrity workloads, not physical-performance benchmarks.
- **Forecast quality:** use the separate Symtropy forecast verifier with site/season-held-out data, proper independent units, declared baselines and calibrated prediction intervals. Do not use random neighbouring pixels as independent samples.
- **Field impact:** run the Sol-Atlas preregistered pilot protocol (BACI or matched comparison where feasible). Freeze the project/scenario digest, outcomes, method, study window, missingness rules and analysis before unblinding. When controls or statistical power are inadequate, classify the result as descriptive/inconclusive rather than causal.

The first gate is the synthetic structural contract. Passing it must not make a real metric computable or change an upstream draft into a validated release.

## Upstream dependencies pinned for this proposal

The manifest records the precise draft-PR commits used as contract sources. These are draft branches, not all merged releases. The harness's adapters are deliberately schema-specific; it does not claim to invoke the upstream validators. Before enabling it as a cross-repository release gate, follow each upstream PR revision, align schema versions, and run the authoritative validator from each owner repository.

## Boundaries

- All sample observations and boundary values are synthetic and must remain labelled synthetic.
- The source catalog remains metadata-only; no Johannesburg source is represented as ingested or licensed for redistribution.
- Schematic local drawing units are not geographic coordinates or survey-grade geometry.
- No scenario model is executed; no real runoff, heat, biodiversity, accessibility, economics, or causal impact is calculated.
- Private coordinates and participant/landholder identities must not be made public by default.
