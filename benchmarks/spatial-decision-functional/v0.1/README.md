# Spatial decision functional reference v0.1

This directory contains a **small exact synthetic oracle**, not a geostatistical, covariance, posterior, GIS or network-simulation engine.

It applies a declared functional to provider-supplied synthetic joint worlds after checking the exact support contract. The initial frozen corpus contains seven synthetic cases. The first reference supports:

- `weighted_mean`: area/volume/line-weighted mean over the complete target support;
- `weighted_total`: weighted total over the complete target support;
- `line_integral`: weighted line/corridor integral;
- `thresholded_measure`: measure of support strictly above a declared threshold;
- `connectivity`: source-to-target reachability over each supplied graph world.

## Run

From the repository root:

```sh
cargo fmt --manifest-path benchmarks/spatial-decision-functional/v0.1/reference/Cargo.toml -- --check
cargo test --manifest-path benchmarks/spatial-decision-functional/v0.1/reference/Cargo.toml
cargo run --manifest-path benchmarks/spatial-decision-functional/v0.1/reference/Cargo.toml -- \
  benchmarks/spatial-decision-functional/v0.1/fixtures/spatial-functional-cases-v0.1.json
```

The CLI emits a JSON report to stdout.

## Input and claim ceiling

The fixture uses explicitly synthetic, hand-authored finite worlds whose weights sum to one and are declared as `exact_probability`. It tests arithmetic and support semantics only. The weights are not learned posterior probabilities, empirical sampling frequencies, calibration evidence or a real-world uncertainty model.

The reference:
- binds each result to a support ID, support revision, geometry-digest identifier, coordinate-reference label, vertical-datum label, coordinate-epoch label, units and declared target measure;
- checks that the geometry digest is 64 hexadecimal characters, but does not resolve geometry bytes or recompute the digest;
- consumes supplied cell measures and checks that they sum to the declared target measure; it does not calculate polygon/mesh area or volume;
- performs no CRS transformation, geodesic calculation or dimensional unit conversion;
- uses the **full target-support measure** as the denominator for a weighted mean;
- reports known-support fraction separately;
- never renormalizes the observed fragment and calls it the full-region mean;
- requires defensible value bounds to emit a bounded result where support is unknown;
- otherwise reports `UNKNOWN`, with no full-support point estimate;
- uses joint finite worlds for thresholded extent and connectivity;
- preserves unknown graph edges as lower/upper reachability bounds;
- does not treat marginal edge probabilities as a joint network model.

The report carries the machine-readable claim ceiling `SYNTHETIC_FINITE_REFERENCE_ONLY`. Passing these fixtures establishes only exact arithmetic and declared finite-world behavior. It does not qualify field data, geometry bytes, CRS handling, cell measures, a covariance model, a posterior, a physical sensor, or a real spatial decision.
