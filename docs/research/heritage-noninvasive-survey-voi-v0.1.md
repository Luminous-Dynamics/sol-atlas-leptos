# Heritage Non-Invasive Survey Value of Information (HVOI) v0.1

**Scope:** one-step Bayesian decision support for selecting the next non-invasive observation of a candidate. This is not a fieldwork authorization, automated survey order or archaeological truth classifier.

## Why it matters

Most remote-sensing systems rank places. A stronger operational system ranks the next observation that can resolve uncertainty and makes costs, time, physical impact and permissions visible.

Mern & Caers (2023) formulate sequential geoscientific data acquisition as a POMDP and demonstrate adaptive planning on a synthetic mineral-exploration problem. This is adjacent-domain evidence for sequential survey design, not an archaeology benchmark: https://doi.org/10.5194/gmd-16-289-2023.

A 2026 review of UAV thermal/multispectral archaeology highlights the scarcity of independently verified ground truth and negative examples, and the gap between integrating complementary datasets and using them to distinguish archaeological from agronomic/pedological explanations: https://doi.org/10.3390/heritage9100429.

## Bayesian calculation

For prior hypotheses H with probabilities p(h), prior entropy is H(H) = -sum_h p(h) ln p(h). For each possible survey outcome o, supplied p(o|h) gives p(o) = sum_h p(h)p(o|h), then Bayes' rule gives p(h|o) = p(h)p(o|h)/p(o).

Expected information gain is EIG(action) = H(H) - sum_o p(o)H(H|o). The Rust implementation reports nats because the core uses natural logarithms. This differs from information gain in bits reported elsewhere; values must not be compared without conversion.

Likelihoods are supplied, not inferred by the planner. Each action includes a source reference and SHA-256 digest for its likelihood model. For real planning, likelihoods must be calibrated on held-out modality-specific observations with target size/depth and survey sensitivity stated. Synthetic likelihoods validate the mathematics only.

## Permissions and scope

- Every source asset is checked with the shared `RightsProfile`, `AccessContext`, use purpose, geometry access tier and exact-geometry policy.
- Land access, permit and custodial-review gates are separate from data rights.
- This v0.1 planner is limited to non-invasive actions. An action declared invasive is excluded even if permit flags are complete; excavation/disturbance requires separate expert, custodial and regulator-led processes.
- Configurable hard limits apply to cost, duration and declared impact.
- Impact score is caller-declared and not a universal physical/cultural harm metric.
- Ineligible actions remain visible with all rejection reasons and receive no EIG score.

## Ranking

The planner preserves information-gain-per-cost ranking for convenience, but no longer treats it as the sole decision. It reports a Pareto frontier across higher EIG, lower cost, shorter duration and lower declared impact. An action is dominated only if another eligible action is no worse on all four dimensions and strictly better on at least one.

## What it is not yet

This is one-step/myopic EIG, not a full sequential POMDP planner. The next stage should update beliefs after an observed outcome and select a second action, with a finite horizon and bounded action set. Evaluate against random, uniform-grid, expert-designed, highest-anomaly-first and cost-only strategies under matched budgets.

## Benchmarks

- Calibration: predicted EIG vs realised entropy reduction on held-out survey outcomes.
- Decision efficiency: validated entropy reduction per cost unit and field-hour.
- Sequential efficiency: cumulative cost/time and number of actions to reach a preregistered decision threshold.
- Regret: gap to the best hindsight action sequence on held-out cases.
- Safety: 100% of invasive actions excluded; 100% of eligible actions pass rights/access/permit/custodial gates.
- Matched-impact utility: compare strategies at equal maximum impact and equal budget.
- Coverage: report unknown hypotheses, uncalibrated likelihoods and abstentions.

## Provisional research target

On independent candidates with calibrated, adjudicated likelihood models, target at least 20% lower cumulative cost or survey time to reach the same preregistered uncertainty threshold than random/grid baselines, while maintaining or improving validated-candidate recall and not increasing disturbance. This is an aspiration, not a Sol-Atlas result.

## Limits

Expected information gain is only as reliable as its prior and likelihood model. A mathematically exact EIG computed from poorly calibrated probabilities is still a poor decision aid. The planner does not validate source truth or survey sensitivity and does not authorize any real-world activity. Human archaeologists, custodians, communities and regulators retain decision authority.