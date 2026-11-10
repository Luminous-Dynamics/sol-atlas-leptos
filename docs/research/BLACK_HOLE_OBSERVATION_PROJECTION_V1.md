# Black-Hole Observation Projection V1

**Purpose:** define a source-faithful astronomical projection contract for Sol Atlas. This is a visualization/view-model layer, not a scientific inference or truth-authority layer.

## 1. Ownership and authority

Preserve the established ownership boundary:
- Mycelix owns canonical claims, source/data provenance, evidence artifacts, attestations and qualification.
- Symthaea owns scientific reasoning, hypothesis generation, model comparison, uncertainty interpretation and active experiment selection.
- Symtropy owns analytic/numerical physics calculations and validated computational products.
- Sol Atlas owns projection, visual comparison, temporal navigation and human inspection.

Sol Atlas must not upgrade a theory-derived prediction into an observation, infer scientific truth from visual similarity, or collapse counter-evidence into one preferred narrative.

## 2. Typed data layers

Every rendered series/image/curve must carry one of these explicit types:

| Type | Meaning | Example |
|---|---|---|
| DetectorMeasurement | calibrated detector measurement | LIGO strain |
| DataQuality | detector availability/calibration/quality metadata | veto or calibration segment |
| ReconstructedComponent | component recovered under a declared analysis | filtered non-QNM residual |
| InferredParameter | posterior distribution from a declared likelihood/model | remnant mass/spin |
| NumericalSimulation | output of a specific NR code/resolution | SXS waveform |
| AnalyticPrediction | output from a declared physical equation/model | Kerr 2*Omega_H reference |
| SurrogatePrediction | emulated waveform with model version/error estimate | NRSur7dq4v2 |
| ModelResidual | difference under a declared conditioning/model | whitened residual |
| NewsObservable | gravitational-wave News and derived peak-News frequency | peak-News frequency |
| TelescopeMeasurement | interferometric visibility/closure product | EHT VLBI visibility |
| ReconstructedImage | image inferred from telescope measurements | EHT intensity/polarization reconstruction |

No renderer may coerce one layer type into another. A QNM fit is an inference/model, not detector strain. A simulation is not an observation. An EHT image is a reconstruction from interferometric data, not a photograph of the event horizon.

## 3. Projection record contract

Each projected record should carry:
- stable projection ID and schema version;
- source archive/product ID and release version;
- source artifact digest(s);
- canonical Mycelix claim/evidence IDs;
- typed layer category;
- exact model/software name, revision and license where applicable;
- event/source ID (e.g. GW250114 or SXS:BBH:0305);
- observation/simulation epoch and time coordinate;
- detector/source frame and coordinate basis;
- unit-bearing values and axis metadata;
- uncertainty representation (samples, credible interval, covariance, error band);
- data-quality and calibration qualifiers;
- model applicability range;
- reproduction/verification status;
- claim ceiling and unresolved limitations.

A view model is a projection of qualified evidence, never the source of canonical scientific semantics.

## 4. GWTC-5.0 / GW250114 event explorer

Initial source adapters:
- GWOSC / GWTC-5.0 event metadata, strain, posterior samples, skymaps and data-quality segments;
- GWTC-5.0 Tests of GR public release (Zenodo 21454847);
- GW250114 Nature direct-wave analysis package (Zenodo 20017347);
- Suzuki et al. orthonormal-QNM data (Zenodo 19601938);
- SXS waveform catalog with simulation IDs, resolutions and extraction metadata.

Required views:
1. Event summary with event ID, detector set, data-quality, calibration and posterior provenance.
2. Strain view with calibrated/whitened state explicit; show both raw/conditioned data origin and processing configuration.
3. Signal comparison: measured strain, model waveform, reconstructed component and residual in separate togglable layers.
4. Posterior explorer: joint remnant mass/spin samples; preserve sample pairing and show distributions for Kerr horizon observables only as model-derived quantities.
5. QNM spectroscopy: non-orthogonal, orthonormalized, rational-filter and LVK analyses as separate model families, with start time, mode basis, prior and posterior provenance.
6. Direct-wave disagreement view: extracted direct-wave frequency/damping vs horizon and News-peak predictors, with competing BOB/plunge model.
7. Evidence drawer exposing the Mycelix claim, publications, file digests, software commit, workflow, null controls and replication status.

## 5. Direct-wave comparison view

The UI must make the competing interpretations visible rather than visually privileging the horizon hypothesis.

Required predictor layers:
- (2\Omega_H): stationary Kerr reference derived from remnant mass/spin posterior samples;
- ω_News,peak: instantaneous News frequency at the peak News-amplitude time;
- Backwards-One-Body / prompt-plunge model output;
- fitted direct-wave instantaneous frequency from rational-filter residual;
- QNM/overtone/mode-mixing residual and detector/noise alternatives.

Required inputs/metadata:
- SXS case ID and remnant spin;
- NR resolution / extraction / extrapolation;
- waveform mode (initial comparison focuses on (2,2));
- rational filter basis (published counteranalysis: 7 prograde QNMs, 2 retrograde QNMs, plus prograde (3,2,0) mode);
- QNM subtraction set;
- fit interval and start time (in units of M);
- remnant mass/spin posterior;
- News definition and peak-News time;
- frequency/damping units and sign convention;
- model and analysis revision.

Published SXS spin-sweep fixtures:
- SXS:BBH:2089, chi_f=0.4942
- SXS:BBH:2088, chi_f=0.5478
- SXS:BBH:3919, chi_f=0.6215
- SXS:BBH:4434, chi_f=0.6864
- SXS:BBH:0305, chi_f=0.6921
- SXS:BBH:2095, chi_f=0.7447
- SXS:BBH:3927, chi_f=0.8029
- SXS:BBH:2099, chi_f=0.8574
- SXS:BBH:3897, chi_f=0.9075
- SXS:BBH:0178, chi_f=0.9499

The counteranalysis reports an incidental horizon-frequency crossing near chi_f≈0.69 and a high-spin deviation in SXS:BBH:0178. Display those as findings from a specified analysis, not universal truth.

The direct-wave component, its frequency estimate, its interpretation as a horizon measurement, and any area-law inference must be separate claim layers with separate evidence statuses.

## 6. Waveform surrogate benchmark view

Compare models only on matching parameter slices:
- NRSur7dq4;
- NRSur7dq4 NN surrogate;
- NRSur7dq4v2;
- SEOBNRv5PHM_NNSur7dq10;
- AESur3dq8;
- NRSurE_q4NoSpin_22;
- LALSimulation / selected EOB approximants;
- high-resolution SXS references.

Display:
- parameter-domain coverage and OOD status;
- mismatch distribution / phase and amplitude residuals;
- remnant mass/spin recovery bias;
- model uncertainty and held-out calibration;
- latency, memory, hardware, precision and version;
- explicit warning when a point is outside model domain.

A fast model with no calibrated uncertainty must not visually appear more trustworthy than a slower reference model.

## 7. EHT / VLBI observation view

For M87*, Sgr A*, OJ 287 and future sources, separate:
- calibrated complex visibility data;
- closure phases/amplitudes;
- reconstructed intensity image;
- polarization reconstruction;
- synthetic observation from a simulation;
- physical interpretation/model.

Record observing date, frequency band, array configuration, station set, calibration pipeline, reconstruction method, regularization, polarization convention, uncertainty and license. Compare models in measurement space where possible, not only rendered-image pixels.

Sky coordinates, celestial reference frame, observation time and cosmological distance model must be explicit. Do not reuse Earth map coordinates for celestial projection.

## 8. Human-facing evidence design

Every view should answer "Why am I seeing this?" with a trace:
`visual element -> projection record -> Mycelix claim -> evidence artifact -> source/version -> software/workflow -> verification -> uncertainty/claim ceiling`.

Use visible status distinctions:
- measured;
- reconstructed;
- inferred;
- simulated;
- analytic prediction;
- hypothesis / disputed.

Do not encode confidence solely in color, line thickness, or visual prominence. Provide a textual explanation and accessible legend.

## 9. Acceptance tests

1. A simulated waveform cannot be serialized/rendered as DetectorMeasurement.
2. An EHT reconstructed image cannot be labelled as a direct event-horizon photograph.
3. The direct-wave Ω_H and News-peak predictors remain separately labelled and independently toggleable.
4. Posterior sample pairing is preserved for Kerr derived quantities.
5. A missing calibration or source digest visibly lowers evidence qualification.
6. Out-of-domain surrogate results are flagged and cannot be displayed as validated predictions.
7. Reproduction, independent replication, critique, failed and inconclusive results remain visible.
8. Every displayed item resolves to source, model, code/data digest and uncertainty metadata.
9. Historical/stale data versions are not silently replaced with latest data in a reproducibility view.

## 10. Non-goals

- no independent scientific adjudication by Sol Atlas;
- no synthetic observations presented as measured data;
- no image-based claim of event-horizon visibility;
- no unqualified direct-wave/horizon association;
- no hiding dissenting or failed replications;
- no Earth-geospatial coordinate assumptions for astronomy;
- no bespoke waveform inference inside the renderer.
