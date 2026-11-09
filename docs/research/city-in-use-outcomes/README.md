# City in-use outcomes corpus v0.1

30 synthetic known-answer cases for post-occupancy/building-in-use evidence comparability. The evaluator checks observation binding, prediction/model comparability, service thresholds and M&V/causal eligibility. It does not compute energy savings, regulatory compliance, resident wellbeing or causal effect.

## Corpus v0.1 composition

The 30 frozen cases include scope/revision and unit mismatches, missing and low-coverage observations, stale owner revisions, meter/experience/service evidence-class controls, consent denial, static/dynamic prediction distinctions, dynamic-model partition leakage/invalidity, M&V window/receipt eligibility, and causal-claim limits. The six latest additions are OI-025 through OI-030; they are hard negatives and do not add any measured real-building data.

## Distinctions

- Original design prediction vs calibrated dynamic model prediction.
- Static prediction gap vs dynamic calibrated gap.
- Technical meter evidence vs owner service receipts vs consented resident self-report.
- Raw before/after observations vs measurement-and-verification eligibility vs causal attribution.
- Energy outcome vs comfort/service availability and user experience.

## Admission policy

A measurement is comparable only when its owner receipt, scope, design/BOM revision, operating context, observation window, unit, calibration and coverage meet the declared profile. Unknown, stale, insufficient-coverage or privacy-ineligible evidence does not become a favorable observation. A survey cannot satisfy a technical meter requirement.

Dynamic models require calibrated status, independent validation on disjoint data and an applicable envelope. The v0.1 oracle now requires distinct, unique training/validation observation IDs, a declared validation protocol and partition-audit receipt; different digest strings alone are insufficient. This is a metadata-level split check, not independent authentication of the underlying raw observation bytes. For real model claims, the upstream owner must independently acquire and re-hash the raw observations and partition manifest; this synthetic oracle does not prove that a provider's split is genuinely leakage-free. The evaluator reports a gap but does not certify the model.

Coverage fractions are constrained to [0, 1]; values over 100% remain Unknown/Unsupported rather than being accepted as extra evidence. M&V eligibility additionally requires explicit baseline and post windows with the same time unit and no overlap; a valid eligibility state still does not report savings. M&V and causal assessment statuses are eligibility states only. They do not report savings or causal effects; those require the relevant method and independent review. M&V eligibility requires a named method/revision, baseline receipt, raw baseline/post periods, explicit normalization, coverage within [0, 1], baseline-adjustment review, non-overlapping time windows in the same time unit, and a post receipt that identifies the exact admitted meter observation for the declared `post_metric_id` and the evaluated window. A receipt for a different metric, a service event, a resident survey or a different scope cannot qualify the energy M&V review.

## Exact reference-auditor semantics

The known-answer comparator requires exact object-key sets at every level, not merely a subset of expected fields. This prevents a test fixture from silently ceasing to assert a required output field. The CLI emits a raw-corpus SHA-256 and exact source commit in its receipt.

The synthetic training/validation observation IDs and partition receipt are test metadata. A real calibrated-model claim still requires an owner-native, independently verified partition manifest and re-hash of raw source artifacts; this oracle does not establish that data partitions are genuinely independent.

M&V eligibility requires a named method/revision, baseline receipt, explicit normalization variables, coverage in [0, 1], baseline adjustment review and non-overlapping baseline/post observation windows. Eligibility is not a savings result.

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

