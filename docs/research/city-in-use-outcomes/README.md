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


## Evidence-class and owner binding

Each requirement declares its expected owner repository, contract and source revision. The fixture does not infer a source contract from a generic field name.

- `meter`: a meter measurement with an identified instrument, current calibration, coverage and observation window.
- `experience`: a consented survey/user-reported observation; meter calibration does not apply, but consent and purpose/rights do.
- `service`: an owner-native operational service receipt; instrument calibration does not apply unless the source contract explicitly requires it.

All classes still bind to scope, design/BOM revision, period, operating context, subject identity and source receipt. The owner contract/profile remains authoritative for the meaning of a service disposition.

## Owner composition

- Symthaea #5949: service capability, operational reachability, continuity, maintenance and replacement dependencies.
- Symthaea #6037: commissioning, integrated system testing, deficiencies, retest and recommissioning evidence.
- Symthaea #6040: design/as-built/configuration digital thread and discrepancy/change lineage.
- Symtropy #1900: version-pinned EnergyPlus simulation adapter and model validation, not POE authority.
- Symtropy #1766: resident-led requirements and habitability feedback.
- Mycelix #5035: declared energy/water/material flows and study boundaries; #5149: asset/custody/inspection/maintenance provenance.
- Mycelix #5015: civic asset reliability and authorized maintenance/work coordination, not an invented municipal ticket API.
- Sol-Atlas #555/#563/#623: access, continuity and common-scope binding.

