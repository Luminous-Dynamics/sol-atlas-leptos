# Expected information gain and value-of-measurement calculator v0.1

**Status:** executable mathematics contract with an illustrative toy fixture; not a hydraulic model, calibrated probability model or engineering decision service.

## Research basis

The next experiment asks not only where sensors should go, but **which measurements can change a decision, given uncertainty in the system and uncertainty in the model itself**.

This is motivated by water-network research rather than an untested claim of novelty:

- [Khorshidi, Nikoo & Sadegh, *Water Research* (2018)](https://doi.org/10.1016/j.watres.2018.06.050) use Value of Information and transinformation entropy to identify sensor placements that provide useful, non-redundant information for contamination-warning decisions.
- [Pressure Sensor Placement for Leak Localization (2022)](https://pmc.ncbi.nlm.nih.gov/articles/PMC8779686/) uses EPANET-generated pressure-change scenarios and balances information relevance against redundancy.
- [Bayesian sensor placement and simultaneous drainage-model calibration (2025)](https://doi.org/10.1080/19942060.2025.2473992) combines sensor placement and model calibration, suggesting a useful extension beyond sensor optimisation that assumes the model parameters are already known.

The implementation below is intentionally narrower: it verifies the arithmetic for explicitly declared finite categorical priors, likelihood tables, decision-loss matrices and measurement costs. It does not implement parameter calibration or a hydraulic likelihood model.

## Quantities are separate

Given hypotheses (H), prior (P(H)), potential measurement outcome (Y), and declared likelihoods (P(Y mid H)), expected information gain is

`I(H;Y) = H(H) - \sum_y P(y) H(H \mid Y=y)`

It is measured in bits. It is not monetary value, reduction in outage duration, or proof of better decisions by itself.

Given candidate decision losses (L(d,h)), expected value of sample information is

`EVSI = min_d E[L(d,H)] - E_Y[min_d E[L(d,H) \mid Y]]`

For this non-negative decision-loss formulation EVSI should not be negative (up to floating-point tolerance). Net value is computed only if measurement cost and decision loss use the same explicitly declared unit; otherwise the tool returns no cost-adjusted ranking. Even compatible units merely make arithmetic possible—the loss/cost basis must still be justified before any real decision.

## Executable implementation

- `sol-atlas-core/src/water_measurement.rs` implements validation, entropy, posterior updating, expected loss, EVSI and conditional net value.
- `sol-atlas-core/src/bin/water_measurement_evaluate.rs` provides a Rust CLI that reads a declared model and emits a structured result envelope. The envelope includes the SHA-256 of the exact input bytes, so the displayed calculation can be bound to the source model rather than only its identifier.
- `sol-atlas-core/fixtures/water-engineering/w0-measurement-model.toy.synthetic.json` includes a perfect-information and an uninformative-sensor control. It is designed to prove calculation behaviour, not water-network behaviour.
- `sol-atlas-core/schemas/water-measurement-model-v0.1.schema.json` and the existing CI schema gate protect the input shape. Rust performs stricter row/column normalization and cross-field semantic checks.
- CI now independently checks the toy result against its source file SHA-256 and expected values: the perfect sensor must produce 1 bit of information gain, EVSI 2 toy loss units, and net value 1; the uninformative sensor must produce zero information gain and zero EVSI.

Negative and boundary tests cover invalid priors, malformed likelihood dimensions, duplicate identifiers, mismatched unit claims, incomparable costs and zero-probability outcomes. A perfect sensor should yield 1 bit in the two-hypothesis toy case; an uninformative sensor should yield zero information gain and zero EVSI.

## Safety and claim ceiling

The result explicitly declares `illustrative_calculation_only`. No EPANET/SWMM input is invoked, the toy likelihoods are not inferred from hydraulics, and the selected measurement action is not a fieldwork instruction. Before using the calculator on generated hydraulic scenarios, the next gate must bind each probability table to a frozen corpus and pinned solver artifacts, estimate or justify sensor noise/priors, test on held-out scenarios, and obtain independent technical review. For real utilities, data permissions, privacy/security review and human authorization remain separate requirements.
