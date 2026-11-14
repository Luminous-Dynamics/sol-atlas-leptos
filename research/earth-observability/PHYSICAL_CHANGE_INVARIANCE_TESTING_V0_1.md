# Physical Change Invariance Testing (PCIT) v0.1

**Status:** research protocol and synthetic qualification corpus; not a validated physical-change detector.  
**Issue:** [#845](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/845)  
**Scope:** target-conditional qualification of Earth-observation change claims.

## 1. Research proposition

An Earth-observation product is a measurement of a target, not the target itself. A difference between two maps can arise from a change in the physical world, a change in the observation process, a change in the interpretation process, or an interaction among them.

PCIT asks:

> Given an apparent change between observations, which part is identifiable as physical change, which part is attributable to the measurement system, and when must the system abstain because the available observations cannot distinguish the explanations?

The programme is deliberately different from another change detector or GIS agent. It evaluates whether the *claim* survives the measurement conditions and reference design under which it is being made.

## 2. Measurement model and estimand

Let the latent target state be (X_t), the measurement profile be (M_t), the interpretation profile be (C_t), and the observed product be:

`Y_t = g(X_t, M_t, C_t, Z_t) + \\epsilon_t`

where (Z_t) contains nuisance or context variables such as atmospheric state, acquisition geometry, phenology, illumination, cloud contamination and terrain. The function (g) is not assumed linear or additive.

The apparent difference:

`\\Delta Y = Y_{t_2} - Y_{t_1}`

is therefore not, by itself, an estimate of physical change:

`\\Delta X = X_{t_2} - X_{t_1}`.

It is only interpretable after the target estimand and the relevant measurement profiles have been specified.

Every test must freeze:

- **Target quantity:** e.g. built-up area fraction, persistent vegetation cover, class-transition area, boundary displacement or an event-time interval.
- **Spatial support:** pixel, object, local window, transect, subcatchment or catchment.
- **Temporal support:** instant, acquisition window, seasonal composite or interval.
- **Minimum mapping unit / minimum detectable effect:** below this boundary, a negative result cannot be interpreted as absence.
- **Class ontology:** versioned legend, mapping rules and unmapped/ambiguous classes.
- **Measurement profile:** platform, instrument, band response, native and delivered resolution, acquisition geometry, atmospheric correction, QA masks, georegistration, processing baseline, resampling and product lineage.
- **Interpretation profile:** classifier/model version, parameters, thresholds, training data and reference-label lineage.

### 2.1 Conditional, not universal, invariance

A transformation is admissible only when it preserves the stated estimand within a preregistered tolerance. Area-weighted aggregation may preserve a catchment-level area fraction under suitable support and sampling conditions. It does not preserve a pixel-level boundary, small-object recall or a feature whose size is below the coarser sensor's effective support.

PCIT therefore separates:

1. **Admissible transformations:** transformations expected to preserve the target estimand, within a stated tolerance.
2. **Stress transformations:** transformations that probe robustness but may change support or observability.
3. **Negative controls:** perturbations known not to represent target-state change.
4. **Out-of-envelope cases:** inputs for which the declared measurement model is not qualified.

Do not count a support-changing transformation as a failed invariance test. Recompute the estimand at the new support, quantify the observability loss, or abstain.

### 2.2 Identifiability

A physical-change effect is not identifiable merely because a model returns a confident answer. It is identifiable only when the admitted design contains enough information to distinguish the target-state explanation from the relevant measurement/interpretation alternatives.

Examples of useful design leverage include same-sensor temporal pairs, near-contemporaneous cross-sensor pairs, stable reference targets, independently sampled reference labels, repeated seasonal windows, independent sensor modalities, and controlled processing re-runs. Where the alternatives remain observationally equivalent, the correct result is `UNRESOLVED_CONFOUNDING` or `INSUFFICIENT_OBSERVABILITY`.

## 3. Experimental design

Use a crossed, preregistered design rather than one-factor-at-a-time anecdotes whenever feasible.

| Factor | Controlled levels / perturbations | Qualification question |
|---|---|---|
| Sensor | same instrument; different instrument; different generations | Does the claim survive a qualified sensor comparison? |
| Spectral response | native; calibrated/bandpass-adjusted; excluded bands | Is the inferred class transition tied to sensor response? |
| Processing | pinned baseline; changed baseline; alternative atmospheric/BRDF correction | Does processing alone create or remove the apparent change? |
| Spatial support | native grid; common grid; conservative aggregation; alternate resampling | Does the conclusion change because of mixed pixels or support? |
| Geometry | nominal alignment; measured shift; controlled half/one-pixel shifts | Is the change an edge or registration artifact? |
| Season / time | season-matched; shifted phenological window; interval composite | Is the signal seasonal state or persistent land-cover change? |
| Interpretation | fixed classifier; classifier update; class crosswalk variants | Does the conclusion depend on the model or legend? |
| Reference | independent probability sample; derivative label; leaked training label; absent | Is the validation independent and sufficient? |
| Evidence lineage | independent acquisition; same-acquisition derivatives; reused labels/model | Is corroboration genuinely independent? |

Record interactions. Sensor × class, season × vegetation class, resolution × minimum mapping unit, and classifier × label ontology can be more important than individual main effects. Do not force a variance decomposition into additive terms when the design does not support it.

### 3.1 Required paired comparisons

At minimum, preserve separate strata for:

- same sensor / same processing family over time;
- different sensors with no harmonization;
- different sensors with a qualified harmonization profile;
- same acquisition processed with different baselines;
- same physical scene with alternate spatial grids/supports;
- season-matched and season-mismatched pairs.

Never merge these into a single “multi-sensor F1” figure.

### 3.2 Reference design

A reference is independent only when its provenance and sampling design are independent of the evaluated product/model in the ways relevant to the claim. A second land-cover map derived from the same imagery, labels or model family is not an independent reference merely because it has a different publisher.

For area estimates, follow a probability-sampling design appropriate to the target population and report design-weighted estimates and uncertainty. Publish the sampling frame, strata, inclusion probabilities, reference-label protocol, adjudication, temporal alignment, interpreter agreement and nonresponse treatment where rights permit. Do not substitute pixel-wise algorithm variance for map accuracy assessment.

## 4. PCIT evaluation record

Each case result must bind:

- corpus version and case ID;
- manifest digest and exact code/model/policy digests;
- target estimand, support, interval and minimum detectable effect;
- all source-native observation/product IDs;
- measurement and interpretation profiles;
- transformation family, admissibility and reason;
- reference design, label independence and rights status;
- claim-level evidence dependencies and common-cause groups;
- per-transformation estimates, residuals and uncertainty;
- negative-control verdicts;
- disposition, claim ceiling, reason codes and reviewer status.

A machine-readable case contract is in [`pcit-case-manifest-v0.1.schema.json`](../../schemas/pcit-case-manifest-v0.1.schema.json); the locked development cases are in [`PCIT_SYNTHETIC_CORPUS_V0_1.json`](./PCIT_SYNTHETIC_CORPUS_V0_1.json).

## 5. Terminal dispositions

| Disposition | Meaning | What it does not mean |
|---|---|---|
| `PHYSICAL_CHANGE_SUPPORTED_WITHIN_ENVELOPE` | Independent evidence supports the target change under the declared measurement envelope | Absolute certainty or causal attribution |
| `NO_CHANGE_DETECTED_WITHIN_ENVELOPE` | The qualified test did not detect change above its stated minimum detectable effect | Proof that nothing changed |
| `MEASUREMENT_SHIFT_EXPLAINS_APPARENT_CHANGE` | The controlled measurement/interpretation perturbation explains the apparent difference for this estimand | Proof that the landscape never changed |
| `INSUFFICIENT_OBSERVABILITY` | Coverage, resolution, reference, rights or signal-to-noise is inadequate | A negative physical finding |
| `UNRESOLVED_CONFOUNDING` | Competing physical and measurement explanations remain indistinguishable | A basis for promoting a consequential claim |

A disposition is always scoped to a target, AOI, interval, support, sensor family, processing envelope and reference design.

## 6. Metrics

Report each metric by class, sensor pair, geography, support, season and difficulty. Include uncertainty intervals and the denominator.

### Scientific performance

- Change precision, recall and F1 by class and transition; PR-AUC as a secondary summary.
- False-change rate on *null physical-change* and *measurement-only* negative controls, reported separately.
- Omission rate and minimum detectable effect curves (effect magnitude × prevalence × observability).
- Area-adjusted estimates and design-based standard errors when a probability sample supports them.
- Spatial boundary metrics and object-level detection only where the estimand requires them.

### Uncertainty and abstention

- Brier score and log score for probabilistic claims.
- Empirical coverage and width of 50%, 80%, 90% and 95% intervals.
- Reliability diagrams by class and measurement profile; ECE only as a secondary diagnostic.
- Risk–coverage and selective-risk curves: accuracy among promoted claims versus coverage.
- Abstention precision: how often an abstention corresponds to a genuinely out-of-envelope or non-identifiable case.

### Invariance and dependency

- Maximum and distributional change in the estimand under admissible transformations, normalized by a preregistered tolerance.
- Sensitivity under support-changing stress transformations.
- False promotion rate when all apparent corroborators share an upstream acquisition, processor, model or label set.
- Effective independent evidence sets / admissible corroboration, with explicit dependency graph and common-cause assumptions.
- Reference-label leakage detection and rejection rate.

### Engineering and operational cost

- Exact replay agreement and independent-verifier attestation validity.
- Workflow repair precision/recall: affected descendants recalculated; unaffected descendants preserved.
- Provenance and rights completeness among promoted claims.
- Data volume, compute, latency, energy and cost per *qualified* finding, not per generated answer.

Do not make a single global accuracy number the headline result. A lower false-promotion rate with calibrated abstention can be preferable to higher recall obtained by silently converting unobservable cases into confident labels.

## 7. Baselines and comparison protocol

Use identical frozen inputs, AOIs, temporal windows, reference samples, class ontology, target support, compute budgets and output schema.

1. Naïve map-to-map differencing / thresholding.
2. Same-sensor, same-product temporal baseline.
3. Cross-sensor comparison without harmonization.
4. Sensor-specific calibrated or harmonized comparison, only where the product actually supports the sensor pair.
5. Conventional pixel/object change-detection methods and a strong learned baseline.
6. Agent workflow without PCIT qualification.
7. Agent workflow plus PCIT, dependency-aware corroboration and abstention.

For agent comparisons, reuse relevant task design lessons from Earth-Agent / Earth-Agent-Pro, TerraBench, ESA-EO-Bench and GISAgentBench, but do not compare their raw scores with PCIT scores: task distributions, oracle access and scoring contracts differ.

Use spatially blocked and temporally blocked splits. Hold out entire AOIs, acquisition dates/tiles and reference-label sources where possible. Prevent pixels from the same scene, duplicate mosaics, derivative products or reused labels from crossing train/test boundaries. Report multi-seed variation for stochastic agents and a deterministic replay score for deterministic components.

### 7.1 Current benchmark landscape (retrieved 14 November 2026)

| Benchmark | Published scope / result | What it is useful for | What PCIT must add |
|---|---|---|---|
| Earth-Agent (2025) | 248 expert-curated tasks; 13,729 images; RGB, spectral and product modalities | Multimodal EO tool use and trajectory + outcome evaluation | Explicit null physical-change controls and measurement-profile perturbations |
| Earth-Agent-Pro / Earth-Bench-Pro (2026) | 248 task cores / 744 questions; 248 open-world execution questions. Reported 66.13% LLM-as-Judge accuracy for the shared GPT-5 setup | Open-world acquisition-to-answer workflows; repair of affected workflow suffixes | Independent measurement-effect identification, source-independence scoring and calibrated abstention |
| TerraBench (2026) | 403 tasks; 24,500 verified execution steps; three tracks and eight application domains | Heterogeneous Earth-system, GIS, simulator and document-grounded workflows with tolerance-aware numeric scoring | Target-conditional physical-change truth and explicit uncertainty about the observation operator |
| ESA-EO-Bench (2026) | 294 tasks; 10 independently authenticated MCP servers; 70 tools | Live multi-agent workflows and balanced imagery + policy/text reasoning | Evidence dependency independence and scientific correctness under sensor/processing shifts |
| GISAgentBench (2026) | 349 practitioner-sourced multi-step GIS tasks with executable references and exact outputs; best reported strict completion 32.7% | Exact, deterministic spatial output evaluation beyond LLM judging | Distinguish an exact output from an identifiable physical-world conclusion |
| GeoNatureAgent (2026) | 103 tasks (93 main + 10 comparison expansion), 18 categories, 16 tools; Claude Sonnet 4 reports 61.7% on all 103 tasks; close-value comparisons report 0% across evaluated models | Fixed API/tool evaluation, error handling, cost reporting and hard comparison tasks | Cross-sensor measurement qualification and independent reference/label provenance |

These numbers come from different task sets, model versions, scoring contracts and execution environments. They are **not a common leaderboard** and must not be ranked against one another. In particular, an LLM-as-judge score, strict file equality/tolerance score, task pass rate and tool-orchestration score are not interchangeable.

The decisive opening for Sol-Atlas is not task-count inflation. The cited agent benchmarks primarily establish whether an agent can operate tools and produce plausible/correct task outputs. PCIT asks whether the physical interpretation is identifiable under measurement-system alternatives, whether corroboration is genuinely independent, and whether the system correctly abstains when it is not. This should become an orthogonal qualification track that can consume outputs from those benchmark styles without conflating their scores.

**Matched SOTA comparison rule:** report agent-execution performance and scientific-qualification performance separately. Freeze the data, target estimand, spatial/temporal support, reference sampling, class ontology, tool layer, compute/cost budget and scoring policy. Compare both (a) the same underlying detector with and without the PCIT qualification layer and (b) strong conventional / learned change-detection baselines on the same admitted data. The relevant win is lower false physical-change promotion and better calibration at a comparable detection/coverage level—not a higher score on a different dataset.

## 8. Initial research targets (not achieved results)

| Measure | Qualification target | Long-term ambition |
|---|---:|---:|
| Hard-gate violations on locked controls | 0 | 0 |
| Physical-change promotions on null / measurement-only controls | 0 on the locked corpus; report a confidence bound on any field estimate | Maintain a low, independently measured field false-promotion rate |
| Promoted claims with complete source, method, dependency and rights lineage | 100% | 100% |
| Valid independent replay attestations for promoted computational results | 100% | 100% |
| Interval coverage | Empirical coverage statistically compatible with nominal coverage on held-out cases | Stable calibration across regions and sensor generations |
| Selective risk | Establish matched baseline at fixed coverage | At least 2× lower error at the same coverage, if supported by matched evaluation |
| Detection above preregistered MDE | Establish per-class baseline and power curve | Beat matched strong baselines without degrading false-promotion or calibration |
| Cost per qualified finding | Establish full-stack baseline | Reduce while holding the five scorecards constant |

Zero failures on a finite synthetic suite is a corpus acceptance criterion, not proof of zero field error. The target of 2× lower selective risk is an aspiration to test, not a claimed performance result. Set class- and region-specific MDEs and tolerances before observing test outcomes.

## 9. Jukskei qualification implications

The Jukskei study by Mawasha (2022) reports a 1987–2001–2015 Landsat time series and four broad classes. The article abstract describes Landsat-5 MSS for 1987, Landsat-5 TM for 2001 and Landsat-8 OLI for 2015. This sequence is not a single homogeneous sensor series: MSS and TM differ substantially in bands and spatial sampling, while OLI is another instrument generation. Confirm the exact scene/product IDs, acquisition dates and processing levels from source-native metadata before reproducing any result.

NASA's Harmonized Landsat Sentinel-2 (HLS) process harmonizes Landsat 8/9 OLI and Sentinel-2 MSI through atmospheric correction, cloud/shadow masking, BRDF normalization, spectral bandpass adjustment and common gridding. It is **not** a drop-in harmonization for the Jukskei 1987 MSS → 2001 TM → 2015 OLI series. The older pairings need their own qualification or a more conservative comparison.

The 2025 CEOS WGCV Land Cover and Change Map Accuracy Assessment and Area Estimation Good Practices Protocol is the methodological reference for reference sampling and area estimation. Use the study's published accuracies as literature comparators only until the reference sample, design, independence and reuse rights have been inspected. Preserve #842's separate rights gate: catalogue availability is not a licence grant.

The first Jukskei output may be "insufficient evidence." This is a successful scientific qualification if the dossier makes the limiting observation or reference gap explicit.

## 10. Relationship to the evidence system

PCIT is a qualification layer, not another evidence graph:

- Sol-Atlas retains source-native observation identities, measurement profiles, claim lineage, dependency links and qualification outcomes.
- Symthaea proposes competing explanations and ranks discriminating observations; those proposals are not evidence.
- Symtropy predicts physical outcomes with an explicit model-discrepancy envelope; simulation is not observation.
- The independent replay verifier reproduces the declared computation; attestation is not scientific truth.
- Mycelix governs rights, provenance custody and legitimate coordination; it does not turn a technically valid claim into permission for consequential action.

When a source, model, label set, processing baseline or rights record changes, invalidate affected descendants only. Preserve the old result and its exact evidence snapshot; publish a new claim version with an explicit supersession edge.

## 11. Known limitations and non-claims

- The v0.1 corpus is synthetic and tests protocol logic, not real sensor behaviour.
- A harmonized product may reduce a known measurement difference but cannot establish complete equivalence or remove all domain shift.
- Uncertainty decomposition is model- and design-dependent; components may be confounded or correlated.
- A reference sample can itself be uncertain, temporally misaligned or dependent.
- No causal claim about flooding, pollution, restoration impact or policy effectiveness follows from land-cover change alone.
- No SOTA superiority claim is permitted from this single regional case or from synthetic controls.
- No data is admitted for redistribution or model training until licence and source terms are confirmed.

## 12. References

- Mawasha, T. (2022). [Detecting land use and land cover change for the past 28-years using multi-temporal Landsat satellite images in the Jukskei River catchment](https://sajg.org.za/index.php/sajg/article/view/823), *South African Journal of Geomatics*, 11(1).
- Tyukavina et al. (2025). [CEOS WGCV Land Cover and Change Map Accuracy Assessment and Area Estimation Good Practices Protocol, v1.1](https://doi.org/10.5067/doc/ceoswgcv/lpv/lc.001).
- NASA. [Harmonized Landsat Sentinel-2 algorithms](https://hls.gsfc.nasa.gov/algorithms/).
- USGS. [Landsat 5 instrument characteristics](https://www.usgs.gov/landsat-missions/landsat-5).
- Lv et al. (2026). [Earth-Agent-Pro](https://arxiv.org/abs/2609.12533).
- Nguyen et al. (2026). [TerraBench](https://arxiv.org/abs/2606.13148).
- Pothuri et al. (2026). [GISAgentBench](https://arxiv.org/abs/2608.01645).
- Diaz-Ireland et al. (2026). [GeoNatureAgent (GNA)](https://arxiv.org/abs/2610.09112).
- ESA Computer and Information Network. [Agentic AI for Earth Observation: Evaluation and Benchmarking of LLMs](https://cin.philab.esa.int/databases/projects/agentic-ai-for-earth-observation-evaluation-and-benchmarking-of-llms).
