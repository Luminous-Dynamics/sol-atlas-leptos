# SA-WORKFORCE-CALIBRATION-001 — Probabilistic diagnosis and abstention benchmark v0.1

**Tracking issue:** https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/290  
**Composes with:** diagnosis #268, source profile #272, active evidence #279, source robustness #281, chain conformance #283, portfolio #286 and recovery #288.  
**Status:** synthetic-only scoring contract; no field-validity claim.

## Paradigm shift: evaluate the probability and the decision to abstain

A predicted label hides confidence. A system can be confidently wrong, or appear accurate by refusing to answer hard cases. The evaluator must measure probability quality, calibration, decision errors, abstention and coverage separately.

The target event for v0.1 is explicitly binary: **a task-scoped capacity gap is independently confirmed within a predeclared horizon**. It does not mean a whole occupation is scarce; it does not imply that training is the right remedy; it is not a prediction of an individual's employability or worth.

Each frozen case has an independently authored synthetic outcome and predictions from five policies: constant-prevalence baseline, transparent rules, deliberately overconfident negative control, abstaining rules, and an all-abstain negative control. Forecasts include a status and reason so an abstention cannot disappear from denominators.

## Metrics

- **Brier score:** mean squared probability error, lower is better.
- **Logarithmic loss:** mean negative log-likelihood, lower is better; exact 0/1 forecasts use a declared clipping floor only for finite calculation.
- **Reliability bins:** fixed-width 0.2 probability intervals report count, mean probability, empirical event rate and absolute gap; empty bins remain visible with null rates.
- **Action-threshold errors:** false-positive “gap” predictions and missed confirmed gaps at the frozen 0.5 probability threshold; probabilities equal to 0.5 count as positive for this v0.1 fixture.
- **Coverage/abstention:** scored count, abstained count, coverage fraction and minimum-coverage gate. Scores are computed over covered cases only, so a selective score must always be read alongside its coverage and abstention counts; it is not directly comparable to a full-coverage score in isolation.
- **Calibration versus discrimination:** neither Brier alone nor one calibration-gap scalar substitutes for the full score and error table.
- **Perturbation evaluation:** future versions rerun the same frozen cases under source duplication, source loss, stale evidence, taxonomy changes and denominator/scope mismatch.
- **Uncertainty of comparisons:** future empirical comparisons require a time-based held-out split, paired uncertainty intervals and minimum sample disclosure; no superiority claim from this synthetic v0.1.

## Hard invariants

- A forecast is pinned to case scope, model/policy ID and prediction status. Missing or unknown labels are not negative outcomes.
- Each case/model pair appears exactly once; duplicate model IDs or case IDs are rejected.
- Issued probabilities are finite and in [0,1]. Abstention requires null probability and a non-empty reason; issued forecast requires a probability and no abstention reason.
- Every model is scored against the same outcome denominator definition. Scoring excludes abstentions but coverage always uses the full case count.
- The minimum coverage gate prevents an all-abstain policy from “winning” by having no scored errors.
- Candidate-vs-baseline comparisons use a frozen event, horizon, threshold, bins, metrics and case set.
- Fixture, outcomes and digest are synthetic; no status can promote this to observed evidence or causal impact.

## Acceptance gates

- Rust scores agree with frozen reference values for Brier, log loss, counts, false positives/negatives, coverage and each reliability bin.
- 100% of forecasts are accounted for once; no duplicate/missing case-model pair.
- All-abstain negative control fails the minimum-coverage gate.
- Overconfident-wrong control is scored worse than the transparent rules on proper scores.
- Row permutation produces the same canonical model scores and bins.
- Receipt includes fixture/schema SHA-256, metric definitions, full denominator, missing/abstained counts and the synthetic-only claim ceiling.

## Research basis

- Waghmare & Ziegel, *Proper Scoring Rules for Estimation and Forecast Evaluation*, Annual Review of Statistics and Its Application (2026): https://doi.org/10.1146/annurev-statistics-042424-050626
- OECD, *Migration Anticipation and Preparedness*: guidance on calibrating predictive intervals and evaluating probabilistic forecasts with Brier score/CRPS: https://www.oecd.org/en/publications/migration-anticipation-and-preparedness_4161131f-en/full-report/how-can-the-model-be-evaluated_1c893567.html
- NIST, *Expanding the AI Evaluation Toolbox with Statistical Models*, NIST AI 800-3 (2026): https://www.nist.gov/news-events/news/2026/02/new-report-expanding-ai-evaluation-toolbox-statistical-models
- NIST TEVV-Athlon, use-case-specific real-world evaluation framework: https://www.nist.gov/artificial-intelligence/ai-research/tevv-athlon-framework-evaluating-ai-systems
- World Bank, South Africa digital-skills pipeline report (14 Aug 2026): https://www.worldbank.org/en/news/feature/2026/08/14/from-demand-to-delivery-strengthening-south-africas-digital-skills

## Claim ceiling

This v0.1 measures software conformance against synthetic labels only. It does not establish forecast skill on real labour-market data, policy utility, fairness in observed populations or causal impact. Empirical claims require authorized, independently adjudicated, time-split held-out observations and uncertainty intervals.