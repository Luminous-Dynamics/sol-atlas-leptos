# City in-use outcomes corpus v0.1

24 synthetic known-answer cases for post-occupancy/building-in-use evidence comparability. The evaluator checks observation binding, prediction/model comparability, service thresholds and M&V/causal eligibility. It does not compute energy savings, regulatory compliance, resident wellbeing or causal effect.

## Distinctions

- Original design prediction vs calibrated dynamic model prediction.
- Static prediction gap vs dynamic calibrated gap.
- Technical meter evidence vs owner service receipts vs consented resident self-report.
- Raw before/after observations vs measurement-and-verification eligibility vs causal attribution.
- Energy outcome vs comfort/service availability and user experience.

## Admission policy

A measurement is comparable only when its owner receipt, scope, design/BOM revision, operating context, observation window, unit, calibration and coverage meet the declared profile. Unknown, stale, insufficient-coverage or privacy-ineligible evidence does not become a favorable observation. A survey cannot satisfy a technical meter requirement.

Dynamic models require calibrated status, independent validation on disjoint data and an applicable envelope. The evaluator reports a gap but does not certify the model.

M&V and causal assessment statuses are eligibility states only. They do not report savings or causal effects; those require the relevant method and independent review.

## Claim ceiling

The corpus qualifies synthetic comparability/evidence semantics only. It is not measured performance of an occupied building, a building-code result, comfort guarantee, energy-savings verification or causal impact estimate.
