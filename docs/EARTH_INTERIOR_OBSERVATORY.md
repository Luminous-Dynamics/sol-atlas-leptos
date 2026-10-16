# Earth Interior Observatory

Status: research-and-contract phase; no live-data or predictive-performance claim is made by this document.

## Purpose

Build a reproducible, evidence-bounded model of Earth's changing magnetic field and selected deep-interior dynamics. Combine geomagnetic observations, seismic waveform-derived quantities, reference models, and physics-based synthetic experiments. Keep observations separate from model estimates and simulations in data, APIs, UI, and published conclusions.

This is not a claim that Earth's inner core can be directly observed or monitored in real time. The magnetic field is measured at/near Earth's surface and from satellites; outer-core flow is inferred from field evolution under assumptions; inner-core rotation/deformation hypotheses are primarily constrained by seismic records and geophysical models.

## Why this is timely

- ESA's Swarm Level-0 products are time-ordered satellite measurements; Level-1 products are calibrated instrument series; Level-2 products are higher-level scientific field/source models. Their processing level and source-model semantics must remain visible: https://www.esa.int/Applications/Observing_the_Earth/FutureEO/Swarm/Data_products
- CHAOS-8 describes the time-dependent near-Earth geomagnetic field over 1999–2025. It is a strong scientific comparator, not automatically an independent test baseline when test observations contributed to its fit: https://link.springer.com/article/10.1186/s40623-025-02352-0
- WMM2025 is a navigation reference model valid through 2029. Its published error model explicitly notes varying uncertainty by field component and location, including elevated declination uncertainty where the horizontal field is weak. Use it as a relevant external baseline, not as a high-resolution core-flow solution: https://www.ncei.noaa.gov/products/world-magnetic-model
- USGS exposes time-bounded geomagnetic observatory data with explicit data types, sampling intervals and output formats: https://www.usgs.gov/tools/web-service-geomagnetism-data
- EarthScope's FDSN services provide seismic station metadata and waveform data. Source access, station metadata, event pairing and archive citations must be retained: https://service.earthscope.org/
- A 2024 Nature study analysed 143 pairs of repeating earthquakes spanning 1991–2023 and reported a pattern interpreted as inner-core super-rotation followed by sub-rotation. Treat the paper as a reproducibility target, not a universal ground-truth label: https://www.nature.com/articles/s41586-024-07536-4
- A peer-reviewed 2026 study reconstructed core-surface flow from satellite and ground observations over 1997–2025 and reported a change in the inferred Pacific flow around 2010: https://jsedi.episciences.org/articles/17268
- A 2025 study emphasizes that inferred core-surface flow is sensitive to geodynamo priors and inversion setup; these dependencies are part of the observable, not implementation details to hide: https://www.sciencedirect.com/science/article/pii/S0031920125000172

## Paradigm shift

Conventional maps place a field model and observations on a globe. This work should instead produce an auditable *state-estimation product* with the following attached to every result:

1. The exact inputs and captured-artifact digests, retrieval/observation times, processing chain, source/product versions, licence state and model/configuration identity.
2. The claim class: direct/processed observation, model inference, published reference model, or simulation.
3. Units, coordinate frame, time scale, validity interval, uncertainty interpretation and the assumptions needed to interpret the value.
4. An identifiability statement: which state dimensions are constrained, which are weakly constrained, and which physically distinct solutions remain consistent with the evidence.
5. Reproducible comparison against named baselines and explicit held-out data.

### Novel research directions

- **Ensemble inversion, not a single authoritative answer.** Compare alternative physical priors, source-separation settings and inversion families. Report where the inferred state is robust and where the posterior spreads across materially different solutions.
- **An identifiability budget.** Use synthetic forward experiments with known truth, null-space tests and sensitivity analyses to measure which core-flow modes can or cannot be recovered from the selected observations.
- **Cross-modal constraints without causal overreach.** Examine temporal correspondence between magnetic secular variation, seismic inner-core observables and geodetic signals. Correlation is a prompt for competing hypotheses, not proof of a causal link.
- **Active information value.** Estimate which additional station, orbit segment, seismic event pair, time window or improved metadata would reduce uncertainty most. This is a research prioritization aid, not autonomous control of observatories.
- **Independent reproduction.** Preserve frozen inputs, configuration digests, model output, benchmark summaries and a replay procedure so another researcher can reproduce or falsify a result.
- **Negative controls by design.** Shuffled event pairs, phase mismatches, time shifts, missing-station simulations and corrupted/duplicated records should be expected to fail. A system that cannot reject plausible-looking false positives is not ready to report new discoveries.

## Responsibility boundaries across the four projects

- **Sol Atlas (sol-atlas-core)** owns renderer-independent, versioned observation/estimate contracts and the eventual interactive geospatial/time visualization. A rendered layer is never the source of scientific truth.
- **Symthaea** owns multimodal inference, anomaly triage, model comparison, calibrated uncertainty, and held-out forecasting/reconstruction evaluation. The cognitive model must not bypass the physics baselines or promote a hypothesis to a fact.
- **Symtropy** owns forward-simulation and synthetic-observability experiments: generate states with known truth, simulate sensor coverage/noise, test recovery, and compare computational approaches. A synthetic result must remain labelled as simulated.
- **Mycelix** owns shared provenance and reproducibility receipts, source/version attestations, review lineage, and conflict visibility. Distributed agreement can establish who attested to what; it cannot make a geophysical claim physically true.

Keep adapters and model APIs at explicit boundaries. Start with frozen public datasets and offline fixtures; do not couple core types to a provider SDK, network client, database, or renderer.

## Initial contract implemented in sol-atlas-core

geophysics.rs begins the schema boundary without implementing a live downloader or an estimator.

- ObservationRecord distinguishes geomagnetic vector observations in local North-East-Down coordinates from seismic differential travel-time observations.
- Every observation carries its time window, optional WGS84 position, source/product/record identity, quality state and optional SHA-256 of the captured source artifact.
- StateEstimate represents radial magnetic-field estimates, core-surface flow estimates and inner-core differential-rotation estimates with explicit units, method/model identity, input references, assumptions, uncertainty interpretation and an identifiability declaration.
- Validation rejects unsupported schema versions, invalid coordinates/time windows, non-finite values, negative standard uncertainties, malformed SHA-256 digests, empty evidence for non-simulation estimates and identifiability claims that omit their named ambiguity.
- Unknown quality, location, licence, artifact digest or uncertainty remains unknown; no default "verified" state is created.
- A locator does not authenticate its publisher, and a matching SHA-256 only establishes byte equality with the captured artifact. Source trust and signed provenance require a separate layer.

This contract is an initial internal seam, not yet a stable external interchange format. Before wider use, review it with geophysics/seismology domain experts and compare its units/time metadata with the exact data products selected for the first adapter.

## Benchmark ladder

### B0 — schema and evidence integrity

- Deterministic serialization round-trips preserve values and explicit unknown states.
- Invalid time windows, out-of-range coordinates, NaN/Infinity, negative uncertainty, malformed digests, duplicate observation IDs and missing evidence fail closed.
- Reordering a set of independent input records does not change a normalized manifest digest.
- Every published estimate resolves its model/configuration identity, evidence references and assumptions. A digest is not mislabelled as publisher authentication.

### B1 — geomagnetic reconstruction

Use frozen Swarm and ground-observatory subsets, and published CHAOS-8 / IGRF-14 / WMMHR2025 / WMM2025 products for clearly scoped comparison tasks. Do not compare against a model fitted using the same held-out observations and call the result independent validation.

Metrics:
- N/E/D component bias, MAE and RMSE in nT; total vector error; declination/inclination error where the field geometry makes these meaningful.
- Error by location/latitude, local time, disturbance level, instrument/product, and observation-source class.
- Residual autocorrelation, data coverage, and the fraction of data rejected as suspect/missing.
- 50/80/90/95% interval calibration and sharpness, where the model actually emits probabilistic intervals.

Initial proposed performance gate (a project acceptance target, not a literature standard): on at least three rolling-origin cutoffs, reduce the primary held-out vector RMSE by at least 5% against the strongest admissible baseline, with paired 95% confidence intervals for improvement excluding zero; no predeclared major geographic/source stratum may regress by more than 10%. If no baseline is beaten, report that plainly and keep the established baseline.

### B2 — outer-core surface-flow inference

- Reproduce the broad published 1997–2025 flow reconstruction as a *reproduction target*, preserving its assumptions and input cutoff.
- Test recovery on ensembles of synthetic geodynamo outputs with known ground-truth flow, including sensor gaps, plausible noise, alternative flow priors and deliberate model misspecification.
- Quantify posterior coverage, parameter bias, sensitivity to priors, null-space ambiguity and between-model spread.
- Test whether the reported 2010 Pacific reversal is recovered when the corresponding period is held out from model choice/tuning.
- Report robust features shared across admissible methods separately from prior-sensitive structures. Do not state a unique interior flow field where the inverse problem does not identify one.

Proposed initial gate: empirical coverage of nominal 90% intervals between 85–95% and nominal 95% intervals between 90–98% across the declared synthetic test ensemble, with failure cases retained. Thresholds should be revisited after the domain-specific sample size and data-generating ensemble are agreed.

### B3 — inner-core seismic inference

- Freeze event-pair lists, station metadata, phase picks, waveform preprocessing, correlation methods and data windows before held-out evaluation.
- Evaluate waveform similarity and differential travel-time error on held-out event pairs and held-out receiver paths; report sensitivity to filters, station changes, phase selection and timing metadata.
- Compare rotation-only, deformation/anisotropy and other plausible explanatory families where those are distinguishable from the chosen data.
- Add negative controls: shuffled event pairs, time-shifted traces, inappropriate phase controls, duplicate inputs and mismatched station metadata.
- Report the time resolution honestly. Current approaches infer evolution over years to decades; do not label a reconstructed historical estimate as a real-time inner-core measurement.

Success means independent reproduction and better-calibrated, better-generalizing hypotheses—not a visually smooth animation.

### B4 — cross-modal inference and information value

- Compare geomagnetic-only, seismic-only, geodetic-only and fused models on held-out observations.
- Require the fused model to improve the predeclared target metric without worsening calibration or hiding modality-specific failures.
- Use ablations and synthetic truth to establish whether each modality adds information beyond the model priors.
- Rank candidate additional observations by expected uncertainty reduction, and test whether that ranking predicts actual improvement in held-out reconstruction.

### B5 — reproducibility and operations

- Replaying a frozen manifest produces the same normalized outputs or documents any nondeterminism and tolerance.
- Ingestion is idempotent; duplicates and conflicting revisions remain inspectable.
- Track source freshness and ingestion completeness, but never infer a current Earth state merely because a scheduled fetch succeeded.
- CI status is attached to an exact commit SHA. Queued, cancelled, missing or stale-head checks are not passes.

## Delivery sequence

1. **Contract and benchmark definition** (this change): versioned typed records, validation and research/acceptance plan.
2. **Frozen-data adapter**: choose one Swarm/observatory product after inspecting its schema, license, timestamp semantics and quality flags. Save fixture manifests and checksums; no runtime network dependency in the test suite.
3. **Magnetic baseline evaluation**: compute baseline residuals and rolling-origin splits before introducing an ML or HDC inference model.
4. **Synthetic flow observability**: use Symtropy to generate known-truth scenarios and test inversion identifiability before claiming novel flow discoveries.
5. **Seismic reproduction**: reproduce a published inner-core result from immutable event/station selections, then add blind controls.
6. **Symthaea model comparison**: introduce multimodal inference only after baseline and provenance contracts work.
7. **Mycelix evidence receipts and Sol Atlas 4D views**: expose dataset lineage, model revision, uncertainties, disputed interpretations, and historical time navigation.

## Research/source links

- ESA Swarm data products: https://www.esa.int/Applications/Observing_the_Earth/FutureEO/Swarm/Data_products
- CHAOS-8: https://link.springer.com/article/10.1186/s40623-025-02352-0
- WMM2025 and error model: https://www.ncei.noaa.gov/products/world-magnetic-model
- USGS geomagnetic observatory data API: https://www.usgs.gov/tools/web-service-geomagnetism-data
- EarthScope waveform and station services: https://service.earthscope.org/
- Inner-core waveform reversal study (Nature, 2024): https://www.nature.com/articles/s41586-024-07536-4
- 2010 core-surface flow reversal study (jSEDI, 2026): https://jsedi.episciences.org/articles/17268
- Prior sensitivity in core-flow inversion (Physics of the Earth and Planetary Interiors, 2025): https://www.sciencedirect.com/science/article/pii/S0031920125000172
- Joint Bayesian magnetic field/core-flow inference (jSEDI, 2026): https://jsedi.episciences.org/articles/17320
