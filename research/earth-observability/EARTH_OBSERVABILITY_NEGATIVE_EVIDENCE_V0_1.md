# EARTH-OBSERVABILITY-NEGATIVE-EVIDENCE-001
## Non-detection, unobservability and qualified absence reasoning v0.1

**Parent:** Sol-Atlas #771  
**Implementation prerequisite:** #779 joint-network branch, `sensor_evsi_bridge.rs`  
**Status:** likelihood method and negative-evidence protocol drafted; physical absence qualification NOT_RUN  
**Date:** 2026-11-14

---

## 1. The central distinction

A non-detection is an observation about a sensor and an acquisition, not a direct observation of absence.

The same absent detector response has at least four explanations:

1. The target is absent.
2. The target exists but the sensor did not detect it.
3. The target was outside the acquisition footprint or observable domain.
4. The acquisition or evidence is invalid, stale, incomplete or otherwise unqualified.

Sol-Atlas must preserve these as different epistemic states. `unobserved`, `out_of_domain`, `below_detection_limit`, `unqualified` and `absence_evidence_eligible` are not interchangeable.

A missing target event is not evidence of absence unless coverage, configuration, calibration, target class, size/depth/contrast envelope and environmental domain are qualified for the relevant spatial support.

## 2. Posterior after a non-detection

For a precisely defined binary search opportunity:

- (POD=P(	ext{hit}mid T))
- (PFA=P(	ext{hit}mid 
eg T))
- (pi=P(T)), an explicitly supplied target prevalence.

Then:

[
P(Tmid	ext{no hit}) =
rac{(1-POD)pi}
{(1-POD)pi + (1-PFA)(1-pi)}
]

This is the posterior probability of target presence after a non-detection. It is not a declaration that the target is absent.

The existing typed detector-to-EVSI bridge now exposes `posterior_target_probability_given_no_hit(prevalence)` alongside the hit posterior. It:
- requires an explicit prevalence; it does not guess prevalence;
- uses both (P(	ext{no hit}mid T)) and (P(	ext{no hit}mid
eg T));
- rejects invalid prevalence and impossible zero-probability observations;
- retains the sensor evidence status and opportunity support;
- does not upgrade synthetic or research point estimates into qualified physical evidence.

The posterior is decision support. The separate Sol-Atlas evidence/authority layer still governs whether a non-detection is eligible to support an absence-related claim.

## 3. Why prevalence matters

Consider a detector with POD 0.95 and PFA 0.02.

At prior prevalence 0.50, the posterior probability of a target after no hit is approximately 4.85%.

At prior prevalence 0.01, it is approximately 0.052%.

The likelihoods are identical. The posterior differs because the prior population of possible targets differs. This is why a detector score alone cannot establish the probability that a particular route segment contains no utility.

The prevalence source, population definition, time validity and spatial support must be declared and independently reviewed.

## 4. Route false-alarm rates are not per-opportunity probabilities

A route-level false-alarm rate cannot be substituted for (PFA) in this equation unless the opportunity model is separately qualified.

The existing bridge only converts a homogeneous Poisson route rate when the exact searched route length, sensor configuration, domain, support and independently qualified process-model receipt are supplied.

Repeated trace windows and multimodal observations cannot be multiplied as independent likelihoods merely because they came from separate files or sensors. Correlated route registration, antenna orientation, soil response, processing and common acquisition conditions require an explicit dependence model.

## 5. Claim ladder for negative evidence

| State | Minimum conditions | Permitted statement |
|---|---|---|
| Unobserved | Acquisition footprint does not cover the claim support | No observation was made for this support |
| Out of domain | Target/environment/configuration outside qualified envelope | Sensor non-detection is not interpretable for this target |
| Below detection limit | Valid acquisition but target-specific POD below preregistered minimum | Non-detection cannot support absence reasoning |
| Unqualified | Missing calibration, identity, source, support or valid likelihood | No qualified inference |
| Absence-evidence eligible | Coverage, calibration, target envelope, environmental domain and preregistered POD are all satisfied | Non-detection may contribute likelihood evidence; it is not proof of absence |
| Absent (adjudicated) | Separate evidence adjudication and applicable authority/verification process | The scoped absence claim may be published with its evidence and validity interval |

The last row is not emitted by the detector or this likelihood function.

## 6. Benchmarks required

### Probability calibration
- Proper log score and Brier score for both hit and no-hit outcomes.
- Reliability diagrams stratified by target class, size, depth, orientation, material/contrast, moisture, geology, instrument and acquisition mode.
- Calibration under leave-project/site/instrument/geology-out splits.
- Prevalence sensitivity curves, not one convenient prior.
- Expected calibration error only as a supplementary diagnostic; it is not a proper scoring rule and must not replace proper scores.

### Negative-evidence risk
- False-absence rate against independent physical truth, only in domains with adequate reference coverage.
- Fraction of no-hit cases classified as unobserved, out-of-domain, below-limit or unqualified.
- Selective risk versus coverage: how errors change as the system abstains more.
- Spatial false-absence area/length and consequence-weighted miss cost.
- Revalidation after calibration, sensor configuration, coordinate transform, target-envelope or environmental-model changes.

### Multi-sensor dependence
- Compare naive likelihood multiplication against declared common-cause/dependence models on matched correlated-observation cases.
- Include repeated acquisitions with shared route/attitude drift, same soil domain, same antenna, common processing and different modalities.
- Require the system to preserve residual probability and abstain when the dependence model is not qualified.

### Decision value
- Compare fixed-grid, expert-selected, random eligible, entropy-greedy and EVSI/expected-net-benefit actions.
- Score at equal permitted investigation budgets.
- Include high-information actions that cannot change the engineering decision; information gain alone is not decision value.

## 7. Industry and standards relationship

This protocol supplements, rather than replaces:
- PAS 128:2022 survey type and quality-level requirements;
- ASCE/UESI/CI 38-22 utility investigation and documentation practice;
- ASCE/UESI/CI 75-22 utility infrastructure data recording/exchange;
- OGC MUDDI underground data integration.

A posterior probability is not a PAS quality level, a professional surveyor's certification, an owner acceptance, a permit or a clearance to excavate.

## 8. Nonclaims

The synthetic likelihood calculation does not validate POD or PFA, estimate prevalence, establish independence, qualify a sensor, establish physical absence or authorize an excavation.

No field data were used in the illustrative calculations. Physical false-absence risk remains NOT_RUN until an independently adjudicated and blinded field benchmark exists.
