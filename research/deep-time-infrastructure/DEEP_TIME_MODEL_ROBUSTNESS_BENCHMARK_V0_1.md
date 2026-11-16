# Deep-Time Model Robustness Benchmark
## Protocol v0.1

**Status:** 12 synthetic cases; six exact hand-computable references; no geological model or local shoreline validation.  
**Owner:** [DEEP-TIME-MODEL-ROBUSTNESS-001 #896](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/896).  
**Composes:** #886, #890, #894; Symthaea #7636; Symtropy #3894/#5474/#1601; Sol-Atlas #400/#115; Mycelix #5923/#5762.

## 1. Research shift: treat model disagreement as a first-class input

The programme must not search for one average Earth history and optimize against it. Different reconstructions may use different observations, time bins, model families, paleogeography and assumptions. Their disagreement determines which infrastructure conclusions are robust, conditional, model-sensitive or unresolved.

A scenario ensemble is not a probability distribution unless defensible weights, calibration and provenance have been provided. Published curve ranges are not automatically confidence intervals and model papers are not independent votes.

## 2. The model-family contract

Every model output used in a decision must retain:
- scientific question, variable, units, age/interval and statistic type;
- reconstruction family/method and the input data/calibration ancestry;
- source observations, uncertainty and model revision;
- paleogeographic and paleotopographic input versions;
- paleo-coordinate transform and horizontal/vertical references;
- tectonic/basin history, vertical land motion and dynamic topography assumptions;
- eustatic, local relative sea-level and ocean/geoid/ice fingerprints as distinct terms;
- source rights, validity, retrieval and claim ceiling.

A missing datum, uncertain paleo-transform or unknown local process must not be filled by plausible defaults.

## 3. Global and local sea level are different targets

Do not treat a global mean/eustatic curve as a local shoreline. Local relative sea level requires the relevant local vertical-land motion, tectonic/basin context, datum, ocean/geoid/rotational fingerprints and paleogeographic reconstruction. At geological timescales, modern coastlines, terrain, river networks and urban assets cannot simply be shifted vertically and relabelled as the ancient Earth.

The +200 m modern terrain experiment remains a synthetic software stress test. It is not a geological reconstruction or a prediction.

## 4. Five decision outcome classes

Classify each decision/essential-service/ecological-floor claim separately:

- **INVARIANT:** admissible under every qualified model in the declared model set.
- **CONDITIONAL:** admissible only under named models, with exact counterexamples.
- **MODEL_SENSITIVE:** decision ordering or consequence changes materially across model forms.
- **UNRESOLVED:** lineage, chronology, datum, transforms or local process evidence is insufficient.
- **UNIFORMLY INELIGIBLE:** at least one mandatory floor fails under every model in the declared set.

These are not one global confidence or weighted resilience score. An invariant result is only as broad as the declared, qualified model set; it does not prove that the set spans all possible Earth histories.

## 5. Twelve initial test cases

| ID | Case | Qualification target |
|---|---|---|
| DTR-001 | synthetic +200 m offset | Cannot be promoted to a palaeo reconstruction |
| DTR-002 | global eustatic + unknown local VLM | No local shoreline claim |
| DTR-003 | competing model families | Preserve estimates; no forced mean |
| DTR-004 | shared proxy/calibration lineage | No false independent corroboration |
| DTR-005 | chronology mismatch | Block pointwise comparison without an age/statistic crosswalk |
| DTR-006 | synthetic shoreline envelope | Exact invariant-wet, model-sensitive and invariant-dry cells |
| DTR-007 | robust model-set choice | Exact minimax regret without probabilities |
| DTR-008 | mandatory floor | Mean performance cannot conceal one-model hard-floor failure |
| DTR-009 | future supercontinent BC | Cannot be converted into a sea-level offset |
| DTR-010 | changed source digest | New decision record; old record immutable |
| DTR-011 | future model identity | Adaptive policy cannot use hidden future information |
| DTR-012 | incomplete datum/lineage | Classify UNRESOLVED and abstain |

Six cases are initially tagged as exact references in the register (DTR-004, 006, 007, 008, 010 and 012). These expected outcomes are manually recomputable and should be promoted into an independent executable oracle before model integration.

## 6. Exact reference examples

### DTR-006 — synthetic shoreline envelope

Cells have synthetic elevations [0, 2, 5, 8, 12] m. Model A has a synthetic water threshold of 5 m; Model B, 8 m.

- A wet: cells [0,1,2].
- B wet: cells [0,1,2,3].
- Invariant wet: [0,1,2].
- Model-sensitive: [3].
- Invariant dry: [4].

The fixture is only a set-classification oracle. It is not a physical shoreline model.

### DTR-007 — minimax regret without model probabilities

Utility table:

| Strategy | Model A | Model B | Model C | Maximum regret |
|---|---:|---:|---:|---:|
| Coastal fixed | 100 | 30 | 50 | 50 |
| Adaptive dual | 75 | 75 | 70 | **25** |
| Inland rail | 60 | 80 | 90 | 40 |

Best utilities are [100,80,90]. Adaptive dual is the minimax-regret strategy at 25. No model probabilities or average expected utility are implied.

### DTR-008 — hard floors are non-compensatory

A coastal strategy has utility 100 under all three synthetic models but passes a mandatory ecological/water floor under A and C only. Its mean utility is 100, but its result is CONDITIONAL and it cannot be INVARIANT. A high mean does not override a hard-floor failure. The Rust kernel also distinguishes CONDITIONAL from UNIFORMLY_INELIGIBLE when every model fails a mandatory floor.

### DTR-004 — shared model lineage

Model M1 uses observations {PROXY-1, PROXY-2} and calibration CAL-7; M2 uses {PROXY-2, PROXY-3} and the same CAL-7. They share an observation and calibration chain. They can be compared as alternative models, but cannot be counted as two independent corroborations of the same underlying evidence.

## 7. Industry comparison and research opportunity

The 7 November 2026 infrastructure interdependency tool review identifies a heterogeneous field: spatial analytics (Kaleidoscope), participatory causal mapping (CIrcle), probabilistic cascade simulation (REFIT), restoration prioritization (PRIIA), fault propagation (CISIA-Pro), hazard-failure-recovery (InfraRisk) and integrated community-resilience modelling (IN-CORE). This programme should not try to replace those tools' specialist models.

Recent work on adaptive transport under deep uncertainty connects monitoring, contingency planning and adaptive action (Jittrapirom et al., November 2026). The benchmark should compose that adaptive-policy ownership with the model-set envelope here. The central differentiator is to tell decision-makers **which conclusions remain valid across disagreeing model families, which depend on specific assumptions, and where evidence is not sufficient to decide**.

## 7.1 Executable reference kernel (research branch; tests not yet run)

The shared Rust core now has a bounded reference module:
[`sol-atlas-core/src/model_robustness.rs`](https://github.com/Luminous-Dynamics/sol-atlas-leptos/blob/research/deep-time-infrastructure-001/sol-atlas-core/src/model_robustness.rs).

It implements:
- finite unweighted minimax regret with integer utility values, deterministic lexical tie-breaking and overflow rejection;
- mandatory-floor classification: INVARIANT, CONDITIONAL, MODEL_SENSITIVE, UNIFORMLY_INELIGIBLE and UNRESOLVED;
- pairwise ranking stability across model families;
- exact synthetic shoreline cell partitions using integer millimetres;
- shared source-observation and calibration-lineage intersections;
- local-shoreline input gating that rejects synthetic modern terrain offsets and future-supercontinent climate boundary conditions, and requires explicit paleo transform, vertical reference, paleotopography, local vertical motion, geoid/gravitational/rotational fingerprint, ocean dynamics, chronology and lineage/uncertainty declarations.

Eighteen Rust unit tests are written, covering DTR-001/002/004/006/007/008/009, unresolved/failed/incomplete controls, five metamorphic properties and the exhaustive 3x3 differential oracle. The Check workflow now includes an explicit `cargo test -p sol-atlas-core` step; testing the consuming application binary alone does not execute the path-dependency crate's own unit tests. The new core test step has not yet reported an exact-head result. A READY_FOR_PHYSICAL_MODEL_EVALUATION disposition means only that the declared input categories are present; it is not physical validation or site qualification.

## 7.2 Metamorphic qualification panel

A separate five-property panel is frozen in:
- [Metamorphic panel JSON](https://github.com/Luminous-Dynamics/sol-atlas-leptos/blob/research/deep-time-infrastructure-001/research/deep-time-infrastructure/DEEP_TIME_MODEL_ROBUSTNESS_METAMORPHIC_PANEL_V0_1.json)
- [Metamorphic panel schema](https://github.com/Luminous-Dynamics/sol-atlas-leptos/blob/research/deep-time-infrastructure-001/research/deep-time-infrastructure/DEEP_TIME_MODEL_ROBUSTNESS_METAMORPHIC_PANEL_V0_1.schema.json)

The five properties are input-order invariance, per-model additive-utility translation invariance, mandatory-floor order invariance, monotonicity of maximum regret when expanding the model set, and pairwise ranking stability. These properties guard against results depending on iteration order, arbitrary baseline offsets, reordered constraints or a selectively narrow model set.

The Rust module currently contains 17 unit tests, including these five metamorphic controls. Test source is committed; execution status is tracked separately from source presence.

## 8. Required comparisons

- single best-estimate model;
- naïve mean/median ensemble (diagnostic baseline, not privileged);
- non-probabilistic model-family envelope;
- minimax and minimax-regret policies;
- model-weighted expected utility only if weights are calibrated, justified and provenance-reviewed;
- fixed plan vs adaptive pathway using Symthaea #7636;
- exact finite oracle on small synthetic cases.

Keep service, ecological, rights and authority floors separate from weighted objectives.

## 9. Proposed acceptance targets

- 100% model and source lineage for material consumed claims.
- Zero scenario-family or variable-type conflation.
- Zero unsupported global-to-local sea-level transformation.
- 100% exact agreement with independent finite synthetic oracles.
- Zero false-INVARIANT classifications on hard negatives.
- 100% retention of named model counterexamples and disagreement cells.
- 100% historical replay for pinned model/source/configuration digests.
- No probability/expected-loss claim from an unweighted scenario set.

These are targets, not achieved results.

## 10. Claim ceiling

Synthetic passes establish comparison and classification semantics only. They do not validate a Cretaceous coastline, predict future coastlines, establish local relative sea level, validate a canal or port or authorize an infrastructure decision.
