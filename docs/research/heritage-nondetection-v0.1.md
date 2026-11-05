# HDB-22: Survival- and Detectability-Aware Non-Detection Evidence v0.1

## Scientific objective

A non-detection is not equivalent to absence. Archaeological evidence may have been destroyed, buried, unexposed, inaccessible, outside the actual survey footprint, unobservable under the sensor/season/vegetation conditions, or simply missed. HDB-22 expresses those stages explicitly and updates a prior only for a valid searched-no-detection event.

For one event:

\[
P(D\mid P)=P(S)\,P(E\mid S)\,P(O\mid S,E)\,P(C\mid S,E,O)\,P(D\mid S,E,O,C)
\]

Here \(P\) denotes originally present, and the factors are explicitly conditional. Their product is the chain rule; it is not a claim that the latent events are mutually independent.

For a searched miss:

\[
P(\neg D\mid P)=1-P(D\mid P)
\]
\[
P(\neg D\mid \neg P)=1-P(FP\mid\neg P)
\]
\[
P(P\mid\neg D)=
\frac{\pi P(\neg D\mid P)}
{\pi P(\neg D\mid P)+(1-\pi)P(\neg D\mid\neg P)}
\]

The false-positive probability is separately calibrated on task-relevant hard negatives. It cannot be inferred from positive-only seeding trials.

## Observation states

Only \`searched_no_detection\` can update the prior. \`not_searched\`, \`not_observable_or_inaccessible\`, \`survey_incomplete_or_inconclusive\`, and \`detected\` return an explicit no-negative-update disposition. A detected candidate is not automatically an archaeological identification.

## Probability and provenance contract

Each prior/stage has:
- lower and upper probability bounds in [0,1];
- target-class and context identity;
- profile, method-build, source-data, calibration-data and independent-validation-split digests;
- a validation receipt reference and digest;
- analytical-use rights decision and rights receipt;
- access tier.

The required stage conditions are fixed:
- survival given originally present;
- exposure given survival;
- observability given survival and exposure;
- coverage given survival, exposure and observability;
- detection given survival, exposure, observability and coverage;
- false positive given originally absent.

All six records must refer to the same target class and declared context. Unknown/denied analytical rights fail closed. Digests are syntax-checked; their bytes and issuing authorities are not authenticated by this module.

## Coverage is geometry, not planned effort

A planned transect length or nominal search-unit field cannot establish that the target location was searched. The coverage stage requires an actual spatial-union/geometry receipt. It also requires an explicit target-location probability model and digest.

Two permitted models are represented:
- uniform area-weighted target location within a declared survey unit;
- a target-specific spatial distribution.

A covered-area fraction becomes a target-location probability only under the first model and only for the declared survey unit. For a known candidate coordinate, use an appropriate target-specific distribution or geometric intersection receipt; do not substitute a unit-wide area percentage.

## Probability-band propagation

Inputs are rectangular lower/upper bounds. The evaluator computes conservative detection and miss bounds, then propagates posterior endpoint bounds using the monotonicity of Bayes' rule in the prior and likelihood terms. These are parameter bounds, not credible intervals or confidence intervals. No distributional interpretation is added.

If the observed event has zero likelihood under the supplied model at a required endpoint, the evaluator does not invent a posterior.

## Integration

The PR is stacked on HDB-15 field-sweep (#523), which itself builds on HDB-11 context-qualified observability (#514). The first slice defines the single-event evidence contract and consumes receipt-shaped stage evidence. A future adapter should bind the HDB-11 Wilson detectability interval and HDB-15 spatial coverage receipt, while retaining separate survival/exposure calibration inputs. The HDB-15 unit-wide coverage fraction must not be converted into a candidate-level coverage probability without an explicit target-location model.

HDB-18 supplies measurement uncertainty; HDB-19 supplies independent spatial splits; the rights/provenance ledger (#539) supplies fixity and purpose-specific rights; HDB-21 preserves annotation disagreement. HDB-22 does not replace any of those systems.

## Synthetic benchmark and adversarial suite

The synthetic fixture uses known stage probabilities and checks exact Bayes arithmetic. Additional tests cover interval propagation, missing/duplicate stages, conditioning relabels, target/context mismatch, denied/unknown rights, malformed digests, false-positive bounds, no-search/no-observability/no-detection states, planned-only coverage, missing geometry receipts, and zero-likelihood outcomes.

P0 acceptance targets:
- 100% no-update for not searched, not observable/inaccessible, incomplete/inconclusive and detected states;
- 100% rejection of missing/duplicate/misconditioned stage evidence;
- 100% rejection of planned-only coverage;
- 100% rejection of denied/undetermined analytical rights;
- 100% no posterior fabrication on zero-likelihood outcomes;
- zero assumption of independence across repeated survey events.

## Real-world evaluation

Compare naive absence updates, detectability-only updates, survival × exposure × observability × coverage × detection, and calibrated false-positive-aware Bayesian updates. Use preregistered known-presence units, hard negatives, and independent site/region splits. Report Brier score, log score, reliability, posterior interval coverage/sharpness, false-absence rate, worst-context performance, verified finds per crew-hour and false-lead burden.

Do not treat an unrecorded location as a negative label. Do not claim a calibrated posterior from a single site or from expert priors without validation. Excavation, stratigraphy, geophysics, chronometry and independently adjudicated field references remain distinct evidence tiers.

## Claim boundary

This implementation validates a supplied single-event probability chain and computes a conditional update. It does not estimate survival, exposure, observability, coverage or detection from raw data; authenticate source receipts; prove that the calibration applies to the target; establish archaeological absence/presence; or authorise field action.
