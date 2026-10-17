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
- An explicit origin class separates instrument measurements, calibrated/processed observations, paleomagnetic proxies, published reference-model outputs, simulation state outputs, and synthetic forward-model observations. Instrument/processed observational records require an absolute time basis; paleomagnetic proxies require an age-before-present basis; simulation state outputs may use a named model-relative clock. A model's known-truth state array is not itself a synthetic sensor observation.
- Time support distinguishes absolute Unix time, named model-relative seconds, and ages-before-present with explicit reference epoch and optional validated discrete chronology distribution. Tests check reversed model-time windows, a valid age distribution, origin/time-basis compatibility, and rejection of a distribution whose probabilities do not sum to one. Acquisition time is kept separate from the time of the phenomenon represented by the observation.
- Every observation carries its time support, optional WGS84 position, source/product/record identity, quality state and optional SHA-256 of the captured source artifact.
- StateEstimate represents radial magnetic-field estimates, core-surface flow estimates and inner-core differential-rotation estimates with explicit units, method/model identity, input references, assumptions, uncertainty interpretation and an identifiability declaration.
- Spatial support is typed independently from value: a geodetic point, a core-mantle-boundary point, or a global scalar. Validation rejects incompatible combinations, such as a core-boundary flow represented as a global scalar. Radial-field sign convention and the reference frame for inner-core rotation are explicit.
- Validation rejects unsupported schema versions, invalid coordinates/time windows, non-finite values, negative standard uncertainties, malformed SHA-256 digests, empty evidence for non-simulation estimates and identifiability claims that omit their named ambiguity.
- Unknown quality, location, licence, artifact digest or uncertainty remains unknown; no default "verified" state is created.
- A locator does not authenticate its publisher, and a matching SHA-256 only establishes byte equality with the captured artifact. Source trust and signed provenance require a separate layer.

This contract is an initial internal seam, not yet a stable external interchange format. Before wider use, review it with geophysics/seismology domain experts and compare its units/time metadata with the exact data products selected for the first adapter.

## Current cross-repository work items

- Sol Atlas contract and project benchmark plan: https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/118
- Symthaea inference/validation work: https://github.com/Luminous-Dynamics/symthaea/issues/7688. It composes existing state-estimation/calibration work (#3759), model credibility/VVUQ (#6122), and benchmark portfolio selection (#6270).
- Symthaea discrete chronology uncertainty: https://github.com/Luminous-Dynamics/symthaea/issues/7725. It benchmarks Bayesian marginalization over age support against mean-age/MAP-age plug-in baselines, with proper scoring rules, coverage, lock-in lag separation, and negative controls.
- Machine-readable chronology benchmark contract: `docs/earth-interior/benchmark-paleo-chronology.json`. It is a specification only; no scores have been produced yet.
- Machine-readable active observation design contract: [`docs/earth-interior/benchmark-adaptive-observation-design.json`](earth-interior/benchmark-adaptive-observation-design.json). It compares information-gain, D/A-optimal, robust-design and model-discrimination policies under equal budgets; this is also a specification only, with no scores or sensor-network recommendations produced yet.
- Machine-readable model-criticism contract: [`docs/earth-interior/benchmark-model-falsification.json`](earth-interior/benchmark-model-falsification.json). It combines in-family simulation-based calibration, posterior predictive falsification, prior/model-family sensitivity, deep-ensemble disagreement, blind negative controls and explicit abstention. It is protocol-only and has not been executed.
- Machine-readable recoverability-frontier contract: [`docs/earth-interior/benchmark-resolution-frontier.json`](earth-interior/benchmark-resolution-frontier.json). It distinguishes posterior precision from data resolution using the resolution matrix, degrees of freedom for signal, cross-talk, blinded injection/recovery and prior-ablation tests. It includes a closed-form linear-Gaussian reference; no empirical benchmark run is claimed. Implementation tracking: [Symtropy GEO-RES-001 #1775](https://github.com/Luminous-Dynamics/symtropy/issues/1775).
- Symtropy known-truth simulation/identifiability work: https://github.com/Luminous-Dynamics/symtropy/issues/1728. It composes the existing exact-head qualification harness (#1264) and the Earth-fluid applicability boundary (#515); now also includes synthetic age-support and nonlinear-trajectory controls for the chronology benchmark.
- Public geodynamo payload acquisition/fixture task: https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/121. The file identity is pinned, but the source payload remains not downloaded/hash-verified until the exact version is independently acquired.
- Mycelix should compose, rather than duplicate, scientific artifact typing and derived-claim authority: https://github.com/Luminous-Dynamics/mycelix/issues/2410 and https://github.com/Luminous-Dynamics/mycelix/issues/2408. These remain the provenance/evidence handoff; this repository does not create a second canonical evidence ledger.

## Multi-timescale reference benchmark discovered (2026-10-16)

A strong known-truth candidate is Julien Aubert's 2026 geodynamo simulation integrated for roughly 1,700 years of physical model time. The paper reports simulation fields and dynamics compared against historical geomagnetic/geodetic observations. Its public IPGP replication dataset (version 2.0; Licence Ouverte / Open Licence 2.0) contains core-surface magnetic field, velocity field and magnetic-diffusion outputs from documented simulation cases. See:

- Paper: https://jsedi.episciences.org/articles/17790
- Replication dataset/version: https://dataverse.ipgp.fr/dataset.xhtml?persistentId=doi%3A10.18715%2FIPGP.2026.mm0sx2m0
- Dataset DOI: https://doi.org/10.18715/IPGP.2026.MM0SX2M0
- Candidate file manifest: [Aubert 2026 case-C surface magnetic field](earth-interior/manifest-aubert-2026-bsurf-c.json)

The candidate manifest pins DataVerse file ID 20149, filename `Bsurf_C.mat`, dataset version 2.0, provider-reported MD5, source metadata URL, and intended benchmark use. It intentionally leaves local SHA-256 null and marks the payload as not downloaded or verified. This is a useful manifest-before-bytes pattern rather than a false claim that we already hold the source artifact.

Use this as an openly documented simulation truth source, not as the Earth's actual observed trajectory. The first adapter should select a small, well-documented case after reading the dataset README, inspect array shapes and units, and freeze the exact data-file IDs, version, declared licence, source metadata, local SHA-256 digests and parsing transform. Do not commit the 302.5 MB archive or derived bulk arrays to Git; keep a lightweight manifest and deterministic retrieval/verification instructions. If the files cannot be retrieved or verified, keep the benchmark blocked rather than fabricating fixtures.

### Why the time axis needed a contract change

A UTC-only timestamp is not valid for physical model seconds, and a present-day acquisition timestamp is not the age of an archaeological or palaeomagnetic sample. The `TimeWindow` contract therefore distinguishes:

- `absolute_unix` — the timestamp window on the declared Unix/POSIX basis;
- `model_relative` — elapsed seconds tied to an explicit simulation/model epoch;
- `age_before_present` — a named age-reference convention, a youngest/oldest age interval, and optionally a discrete age-probability support tied to a named chronology model.

`ObservationRecord.acquired_at_unix_ns` is distinct from the phenomenon's `time`. An age distribution must be non-empty, strictly ordered, fall within the declared interval, identify its chronology model, and have probabilities summing to one within a stated validation tolerance. This preserves the kind of chronological uncertainty that recent Bayesian geomagnetic reconstructions marginalize rather than silently replacing it with a point date. See: https://jsedi.episciences.org/articles/17320

### Chronology-marginalization benchmark pinned

A machine-readable specification is tracked at `docs/earth-interior/benchmark-paleo-chronology.json`. It converts the 2026 joint Bayesian field/flow reconstruction paper into an independently authored benchmark contract:

- Compare discrete age marginalization with mean-age and MAP-age plug-in baselines on the same frozen synthetic known-truth cases.
- Add exact tiny-support tests for stable log-sum-exp, and a nonlinear trajectory where plug-in age evaluation diverges from marginalized likelihood.
- Generate narrow, broad, skewed, multimodal, and outlier-weight chronology supports; separate depositional/lock-in lag from dating uncertainty.
- Score held-out log predictive density/CRPS, field/flow bias and RMSE, interval calibration, negative-control rejection, and runtime scaling.
- Preserve chronology prior, likelihood, physical prior, posterior age distribution, and inferred field/flow as distinct artifacts.

The specification is not yet executed and does not claim to reproduce the authors' implementation. The referenced paper is CC BY-NC-ND 4.0; this work uses bibliographic metadata and an independently written test protocol rather than copying or adapting article figures/text. The scientific basis is the paper's abstract describing discrete chronology-age marginalization and synthetic geodynamo validation: https://jsedi.episciences.org/articles/17320

### Proposed simulation benchmark protocol

1. Freeze one documented Aubert simulation case as a known-truth trajectory; keep the true core state and any hidden test portion unavailable to an estimator under evaluation.
2. Define a separately versioned observation operator for synthetic satellite/ground geometry, source separation, sample cadence, noise and missingness. Preserve the truth arrays, synthetic observations, transforms and estimator outputs as separate artifact roles.
3. Hold out contiguous time intervals and, where scientifically defensible, an entire documented simulation case. Randomly splitting adjacent time samples alone is not adequate because temporal dependence can leak between train and test.
4. Run a model-recovery baseline before Symthaea. Compute a local Jacobian from the simulator or an explicitly documented finite-difference perturbation; run the observability diagnostic on that exact linearization, with predeclared physical parameter scales and measurement-error scales.
5. Score state recovery, local identifiability, posterior/interval calibration, prior sensitivity, out-of-case generalization and compute cost. Repeat across simulation cases and noise/coverage conditions; never award a pass from one seed.
6. After synthetic qualification, evaluate on frozen satellite/observatory measurements using model cutoffs that exclude the held-out observations. Simulation success is necessary for a credible inversion pipeline, but not sufficient to establish performance on the real Earth.

The 2026 CHAOS-8 model itself uses satellite and observatory data and an improved temporal regularization informed by geodynamo simulations; it also documents source separation and small-scale, rapidly varying features. Use it as an important reference and leakage-control challenge, not independent ground truth where its input data overlap with the test fold: https://link.springer.com/article/10.1186/s40623-025-02352-0

## Local observability diagnostic added

`sol-atlas-core::observability` adds a small, dependency-free local inverse-problem diagnostic for modest parameter counts. It requires an explicit `ObservabilityContext` naming the analysis, model/version, linearization, and immutable input manifest. It also accepts stable observation/parameter IDs, parameter scales and observation/error scales. It analyzes the dimensionless Jacobian formed by scaling parameter increments and observation residuals, then reports singular values, numerical rank, condition number only when full column rank is found, approximate unobservable directions, and the absolute/relative rank tolerances used. The report preserves the context so the diagnostic cannot be separated accidentally from its declared experiment identity.

The implementation uses a one-sided Jacobi SVD rather than forming the normal-equation matrix (J^T J), avoiding the unnecessary loss of numerical precision caused by squaring the condition number. Unit tests cover full rank, dependent observations with an unobservable direction, explicit scale effects, malformed matrix shape, non-finite inputs, invalid scales/tolerances, preservation of experiment context, and deterministic repeatability.

**Claim ceiling:** this identifies local sensitivity/identifiability for the supplied linearization. It does not establish global identifiability, validate the physical model, distinguish a bad prior from missing information, or prove that a real Earth parameter is observable. Scale selection and tolerance must be predeclared and included in the experiment manifest. Large or highly ill-conditioned inverse problems should use a reviewed numerical linear algebra backend.

### Linear-Gaussian resolution implementation added

`sol-atlas-core::resolution` now contains a small dense linear-Gaussian diagnostic alongside the local Jacobian observability tool. It computes posterior covariance, the prior-conditioned data-resolution matrix, diagonal resolution by named parameter, off-diagonal cross-talk norms, and degrees of freedom for signal. Input matrices preserve correlated observation errors and cross-parameter prior covariance. Positive-definite systems use Cholesky factorizations and triangular solves; malformed shapes, asymmetric or ill-conditioned covariance matrices, invalid tolerances, non-finite values, and duplicate IDs fail closed.

Thirteen Rust unit tests are authored for the exact two-mode reference, cross-talk, correlated errors, prior-dominance, exact null modes, parameter-order invariance, malformed shapes/IDs, non-finite inputs/tolerances, invalid covariance, and preservation of experiment identity. A separate JavaScript mathematical mirror matched the analytic H0 values and exercised off-diagonal, correlated-covariance, prior-dominance and non-diagonal-prior cases. The Rust test now loads the machine-readable benchmark JSON directly, so fixture and implementation can be kept in lockstep. Neither the mirror nor the JSON parse is a Rust compile/test. The implementation is for small, well-conditioned problems only and explicitly warns against using it as a large-system numerical backend or global-identifiability proof.

## Benchmark ladder

### B0 — schema and evidence integrity

- Deterministic serialization round-trips preserve values and explicit unknown states.
- Invalid time windows, out-of-range coordinates, NaN/Infinity, negative uncertainty, malformed digests, duplicate observation IDs and missing evidence fail closed.
- Reordering a set of independent input records does not change a normalized manifest digest.
- Every published estimate resolves its model/configuration identity, evidence references and assumptions. A digest is not mislabelled as publisher authentication.
- Local observability fixtures cover a known full-rank matrix, exact dependent columns and their expected null direction, scale normalization, and explicit rank tolerance. The result records the analysis/model/linearization and input-manifest IDs so the matrix and chosen scale/tolerance can be reconstructed.

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

### Canonical benchmark-run receipts and fail-closed qualification

All executed Earth Interior Observatory benchmark suites now point to the shared JSON Schema at [`benchmark-run-receipt.schema.json`](earth-interior/benchmark-run-receipt.schema.json). The schema is for an *attempted run*, not for an unexecuted benchmark specification. Every attempt should emit a receipt, including failed, cancelled, or interrupted runs; silence is not a pass.

A qualified receipt binds the benchmark-specification digest, exact code commit, clean worktree, frozen configuration digest, toolchain/environment, random-number generator and seed set, immutable artifacts, metric values and uncertainty, negative-control outcomes, gate decisions, and an independent verifier result digest. Qualified synthetic runs must show that the truth artifact was withheld from the selector until scoring. Source-adapter qualification additionally requires a verified payload digest and passed unit, time-axis, and variable-shape audits. Real-observation evaluation additionally requires passed holdout-leakage and reference-overlap audits.

The schema deliberately rejects a qualified status when any declared gate or negative control did not pass, the worktree is dirty, execution did not complete, or independent-verifier evidence is absent/not accepted. A queued, skipped, cancelled, missing, or stale-head workflow is not a pass. JSON Schema validates receipt structure; an independent verifier must still recompute hashes and gates before promoting a result. A valid receipt is provenance, not proof that the physical model is correct.

#### Qualification digest protocol and workflow-run evidence

The independent verifier binds its result to a *qualification-core digest*: compact UTF-8 JSON with recursively key-sorted objects after removing `qualification_evidence` and normalizing `qualification_status` to `executed_unqualified`. This lets the verifier inspect a provisional receipt and later attest the same scientific run after promotion. The verifier report lives outside the receipt; its SHA-256 is recorded in `qualification_evidence.verifier_result_sha256`. This avoids a self-referential hash cycle.

Every required CI check must name its exact commit and a `ci_check_evidence` artifact containing the corresponding locally retained workflow-run JSON. The verifier compares run ID, name, head SHA, status, conclusion and URL to the receipt, and also verifies the evidence artifact digest and byte length. Queued, in-progress, skipped, cancelled, missing, failed, neutral, stale-head or other non-success checks cannot qualify. This offline comparison checks consistency and integrity; it does **not** cryptographically authenticate that JSON was returned by GitHub, so the run URL/provider source remains independently auditable.

A verifier report must identify the target qualification status and verifier commit. JSON Schema validity alone never proves scientific validity.
### New calibration research frontier — conditional posterior reliability

The machine-readable model-falsification contract is version 0.1.1 and adds `L6_data_conditioned_calibration`. Traditional simulation-based calibration samples parameters from the prior and checks computational self-consistency across prior-predictive cases. The newer *Posterior SBC* work focuses inference diagnostics around a particular observed-data regime. These answer related but distinct questions, so the benchmark should retain both rather than replace one with the other ([Säilynoja et al., 2026](https://doi.org/10.1007/s11222-026-10825-9)).

For Earth-interior inference, evaluate calibration by predeclared observation strata—spatial coverage, signal/noise, temporal regime, missingness/correlated errors, and simulator-family shift. Include a planted pooled-coverage trap: overall coverage looks acceptable while one subgroup materially under-covers. Score conditional 90%/95% interval coverage, sharpness, proper predictive scores, model-family shift detection and abstention, all with sample counts and uncertainty intervals.

Flow-matching posterior correction under simulator/prior/noise misspecification is a candidate to benchmark, not a default method ([Ruhlmann et al., ICML 2026](https://proceedings.mlr.press/v306/ruhlmann26a.html)). Any correction must improve calibration on held-out known-truth simulations without hiding poor predictive scores behind overly broad intervals. **Passing these synthetic tests is not proof that posterior intervals for the real Earth's hidden core state are calibrated**: real surface geomagnetic/seismic observations do not directly supply the latent core state as ground-truth labels.

Initial target (aspirational, not yet a gate): per-qualified-stratum nominal 90% coverage within a preregistered ±5 percentage-point band, with no material proper-score or sharpness regression. Fix strata, power, thresholds and uncertainty calculation before opening the blinded qualification set.

### Required-CI set completeness

Each benchmark contract now declares `required_ci_checks` (currently `Check` and `Earth Interior receipt verifier`). The receipt verifier compares its listed check names against that policy exactly and rejects missing, duplicate or unexpected check records before inspecting each retained workflow-run JSON artifact. This closes the omission loophole where a syntactically valid receipt could show only one passing check while silently leaving another required check unreported. The policy is bound by the benchmark specification digest and exact code commit. Queued/pending/skipped/cancelled/stale-head evidence remains non-passing.

### Cross-project H0 implementation: information gain and observability

Symtropy's [known-truth H0 draft PR #1804](https://github.com/Luminous-Dynamics/symtropy/pull/1804) now contains four deterministic two-state observation fixtures, a frozen [public fixture manifest](https://github.com/Luminous-Dynamics/symtropy/blob/feat/geophysics-known-truth-h0-20261017/crates/domains/symtropy-geophysics-benchmark/fixtures/h0-public-manifest.json), and a covariance-whitened rank/singular-value oracle. Its exact Gaussian EIG function uses the full observation covariance (R) and prior covariance (S_a):

`EIG = 0.5 * ln(det(I + S_a H^T R^-1 H))`

Under the fixture's standard-normal prior and unit-cost two-channel assumption, complementary sensors have an analytic EIG of 1.6094379124341003 nats; the highly correlated duplicate pair has 0.8067249769620305 nats. Pretending the duplicate errors are independent inflates the score to 1.0986122886681098 nats. The exact references match the analytic fixture in `benchmark-adaptive-observation-design.json`; they are not empirical benchmark results, and no performance gate is yet qualified.

The weak-mode fixture in Symtropy uses (H = mathrm{diag}(1,0.1)) with non-uniform observation covariance, so its singular values/condition number differ from the separate resolution-frontier H0 reference (H = mathrm{diag}(2,0.1)) with identity prior and observation covariance. Keep the matrices, scaling, covariance, and tolerances bound to each fixture ID rather than combining their numbers.

**Status:** 15 Rust tests are authored in Symtropy PR #1804, but no local Rust toolchain is available in this environment and the exact-head workflows remain pending/queued. The manifest remains `benchmark_specification_not_executed`; this cross-project link does not promote it to an executed or qualified benchmark.

### Independent-verifier checkout boundary

Qualification promotion now requires two canonicalized clean checkouts: one at the exact benchmarked code commit, and a separate verifier checkout at a different exact commit with a recorded origin. The verifier rejects identical checkout directories, identical target/verifier commits, dirty verifier checkouts, missing origin, and final receipts whose verifier evidence does not match the separate checkout's current HEAD/origin.

This is stronger than relabelling the tested commit as “independently verified,” but it remains an integrity/provenance control. It does not prove that the verifier was authored or reviewed by a genuinely independent party, and locally retained workflow JSON still needs separate provider-origin review. See the usage contract in [`tools/geo-receipt-verifier/README.md`](../tools/geo-receipt-verifier/README.md) and [GEO-QUAL-001 #128](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/128).

The verifier source change is authored on the feature branch but is not Rust-qualified until the exact-head formatter, tests and Clippy checks complete successfully.

The running verifier binary is also bound to its compiled Cargo manifest location: the canonical repository root derived from `CARGO_MANIFEST_DIR` must equal the declared verifier checkout. This blocks an invocation from passing a second clean path while actually executing a binary compiled from the evaluated checkout. The verifier report records this check. It still does not constitute a cryptographic signature or prove an independent human review.
