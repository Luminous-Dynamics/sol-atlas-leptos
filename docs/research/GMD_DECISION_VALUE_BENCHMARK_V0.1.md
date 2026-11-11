# GMD Decision Value and Active Observation Benchmark v0.1

**Date:** 2026-11-11  
**Status:** Research benchmark contract; proposed thresholds are not achieved results.  
**Claim ceiling:** E0 design until frozen synthetic cases, independent event/site holdouts and physical evidence gates pass.  
**Tracking:** [GMD-DECISION-VALUE-001 #723](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/723).

## 1. Research objective

Evaluate whether a system can choose *when to act, when to observe, when to wait, and when to stop* so that essential services are protected under uncertain geomagnetic disturbance (GMD) conditions.

The primary research question is not whether an additional sensor reduces posterior variance. It is:

> Does the additional observation arrive in time, change a feasible decision, and reduce the loss of essential services enough to justify its cost and operational burden?

This benchmark composes, but does not replace, the existing:
- [outage-aware observability oracle PR #714](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/714);
- [service continuity oracle PR #717](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/717);
- [operator action-window oracle PR #722](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/722);
- [network mitigation issue #703](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/703);
- [canonical GMD architecture PR #693](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/693).

The benchmark evaluates qualified upstream outputs. It is not a substitute for a geophysical solver, a utility state estimator, an EMT simulator, an operator's judgement or an infrastructure owner's authority.

## 2. Paradigm: information is valuable only when it changes a decision

A sensor portfolio can reduce uncertainty without improving a decision. A highly accurate forecast can arrive too late. A mitigation can reduce GIC at one transformer while transferring stress to other equipment or lowering service elsewhere. An energised substation does not prove that a hospital, water plant or communications network has recovered.

The benchmark therefore distinguishes five quantities:

1. **Information quality:** what an observation says about the physical state.
2. **Decision relevance:** whether the observation changes the preferred feasible option.
3. **Timeliness:** whether the observation and resulting action can complete before the relevant deadline.
4. **Operational admissibility:** whether physics, evidence, rights, resources, scope and human authority permit the action.
5. **Service outcome:** whether owner-defined essential-service floors were maintained or restored.

Information gain, forecast skill, operational readiness and service continuity are separate reported results. None is a proxy for another.

## 3. Formal decision model

### 3.1 State and evidence

At decision time t0, define a finite scenario-state set W with prior probabilities P(w | E0), where E0 is the evidence available at t0.

Each state w may encode:
- geomagnetic forcing and local magnetic-field evolution;
- local Earth conductivity / magnetotelluric response;
- geoelectric-field vector and uncertainty;
- line geometry, network topology and quasi-DC current response;
- transformer type, operating state and response model;
- coupled service dependencies and resource availability.

These are benchmark inputs, not assumed known facts. Each state feature carries its own evidence class, source reference, revision, validity envelope and digest.

### 3.2 Observation option

An observation option o includes:
- measurement operator and quantity/unit;
- candidate station/instrument or data product;
- measurement-noise model, including cross-sensor/common-mode correlation;
- expected acquisition and delivery-latency distribution;
- probability of missing, corrupt, delayed or stale delivery;
- calibration, clock, CRS and quality evidence;
- independent rights for discovery, download, analysis, commercial ingestion, training and redistribution;
- cost and resource requirements.

For each possible observation result z, provide a likelihood P(z | w, o). The posterior may be updated only after the observation's declared arrival time. No future value or unavailable source can be read early.

A rights-restricted or unqualified observation may be useful for academic analysis while remaining ineligible for commercial ingestion or model training. Rights do not transfer between purposes.

### 3.3 Candidate actions and feasibility

Each candidate action a includes:
- physical admissibility by state or qualified state envelope;
- required authority and authorization scope;
- procedure revision and digest;
- minimum/maximum approval, preparation, mobilisation and execution duration;
- resource requirements and availability evidence;
- action reversibility and owner-defined cost;
- service effects and network-wide side effects.

An action is eligible only when its independent gates pass. A posterior probability or confidence score does not grant authority.

The decision set must include non-action options: HOLD, OBSERVE, DEFER and ABORT. The benchmark must not reward action in every scenario.

### 3.4 Service-loss vector

For each service s, retain an owner-defined service-floor loss in that service's native unit. Report at least:
- total unserved quantity;
- cumulative floor-deficit integral;
- duration and number of floor-breach episodes;
- longest continuous breach;
- maximum interval deficit;
- time to floor recovery;
- time to sustained full-service recovery;
- recovery confirmation delay.

Do not sum unlike services into a universal scalar. Publish a service-loss vector and Pareto comparison. A scalar owner utility is allowed only when service normalization, weights, hard floors and priority ordering are explicitly supplied and authorized by the relevant owner.

### 3.5 Value of information

Let pi0 be the best admissible policy using current evidence, and pi(o,z) the admissible policy after observation o returns result z.

For each service s, define the expected decision value:

VOI_s(o) = E[L_s(pi0) | E0] - E_z[E[L_s(pi(o,z)) | E0,z,o]]

where L_s is the predeclared service-loss metric for service s.

Report the VOI vector, observation cost, latency, failure probability and missed-window risk separately. Do not hide these dimensions inside a single arbitrary score.

Where an owner-approved normalization and priority vector exists, a scalar decision utility may be computed in addition to—not instead of—the service vector. A clairvoyant action selected with knowledge of the held-out state is an upper-bound oracle only, never a deployable baseline.

An observation arriving after the latest safe decision instant has zero value for that decision, even if it reduces posterior uncertainty. A measurement can also have negative net value when it consumes scarce resources, delays an action, increases common-mode dependence or creates misleading confidence.

## 4. Policy comparison

Every frozen scenario is replayed with identical initial state, candidate actions, resources, event truth, service dependencies and decision horizon.

Required policies:

| Policy | Behaviour |
|---|---|
| ACT_NOW | Decide from current evidence without requesting another observation. |
| NEAREST_OBSERVATION | Request the nearest eligible measurement, without decision-aware selection. |
| FIXED_PORTFOLIO | Use a fixed sensor portfolio under a declared lifecycle-cost budget. |
| VARIANCE_GREEDY | Select the observation with the largest expected uncertainty reduction. |
| DECISION_AWARE_VOI | Select observations by expected service-loss reduction after latency, reliability and cost. |
| HOLD | Make no new intervention until evidence changes or the deadline is reached. |
| OPERATOR_RULE | Execute the documented operator rule or approved playbook where available. |
| ROBUST_ACTION | Select an admissible action under a declared uncertainty set / chance constraint. |
| ORACLE_UPPER_BOUND | Select the best feasible action with held-out state knowledge; upper bound only. |

Where source material does not support a policy or baseline, mark it unavailable rather than inventing a result.

## 5. Benchmark families

### A. Forecasting and event detection

Keep forecast targets separate by physical quantity:
- solar-wind drivers;
- ground magnetic perturbation dB;
- geoelectric-field vector E;
- line-integrated induced voltage;
- transformer-neutral / effective GIC.

Report quantity-specific MAE, RMSE, normalized error, bias, CRPS, interval coverage and width, Brier score, probability of detection, false-alarm ratio, Heidke skill score, peak magnitude error, peak timing error and lead-time-conditioned performance.

Evaluate both quiet/background conditions and event thresholds. Report missed high-amplitude peaks separately; averages can conceal dangerous peak underprediction. Use complete-event and geographic holdouts, not random adjacent-window splits.

### B. Observability and sensor placement

Compare:
- no additional observation;
- nearest-station selection;
- uniform/geographic placement;
- variance-greedy or D-optimal placement;
- the existing correlated-noise/outage-aware portfolio oracle;
- decision-aware VOI placement.

Hold the lifecycle-cost budget constant. Include sensor outages, correlated noise, shared calibration errors, spatial representativeness, acquisition latency and measurement-quality degradation.

Report weighted posterior uncertainty as a diagnostic, but rank decision-aware policies by service-loss change, missed-window rate and tail performance. A sensor can be statistically informative yet operationally irrelevant.

### C. Physical consequence

Keep the physics chain explicit:

solar-wind / geomagnetic forcing
-> local magnetic perturbation
-> Earth conductivity response
-> geoelectric field
-> line-integrated voltage
-> topology-conditioned quasi-DC GIC
-> transformer response
-> AC network consequence
-> service outcome.

Each handoff needs a versioned input/output contract, unit, coordinate frame, uncertainty, validity domain and independent validation record.

Use analytical known-answer cases before event replay; then validated RMS/EMT models, real-time simulation, controller-HIL, Power-HIL/laboratory work and owner-authorised field exercises. A correct data transformation is not physical validation.

### D. Action-window and operator feasibility

Measure action completion-time distributions, not only point estimates. Include approval latency, preparation, mobilisation, execution, safety buffer, communications delay, resource withdrawal and forecast-arrival uncertainty.

Report:
- false-ready rate;
- missed-window rate;
- completion margin distribution;
- infeasible action recommendation rate;
- authority and scope violations;
- stale-evidence decisions.

Zero false-ready on a frozen known-answer corpus is a software gate, not proof of zero operational risk.

### D.1 Operator decision practice and adoption

Physical optimality is not the same as operational adoption. Include a human-in-the-loop replay lane that records the decision actually available to an operator at the time:

- disposition: ACCEPT, MODIFY, DEFER, REJECT or NO_ACTION;
- selected action and any modified scope/procedure;
- operator role/authority and approval evidence;
- rationale: forecast confidence/locality, remaining lead time, service/revenue exposure, cost, reversibility, staffing, equipment, existing playbook and competing risks;
- evidence and model revisions available at decision time;
- post-event asset and service outcomes, where owner-authorised evidence exists.

Compare the operator's choice with physically feasible, owner-rule, service-floor-constrained and cost-aware counterfactuals. Operator practice is an empirical reference for what was done, not ground truth for optimality or safety. The benchmark must not reward higher adoption by relaxing safety or service constraints.

An important 2026 comparator is *Space Weather Mitigation Decisions by Critical Infrastructure Operators During Solar Cycle 25* (first published 26 September 2026; DOI https://doi.org/10.1029/2026SW005072). The study contacted 313 operators across power, satellite and aviation, received 55 unique responses (18% response rate), interviewed 33 operators and identified 91 mitigation decisions, 74% of which were undertaken. Reported grid actions skewed toward lower-cost situational awareness, staff readiness, simulation and modelling; GIC-blocking activation and protection-setting changes were rare or absent in the surveyed responses. The authors emphasize demand for precise, localized, impact-based forecasts and stronger cost-benefit evidence.

This is a useful design comparator, not a representative census or a universal adoption target. Report sample counts, response rate and sector denominators. Do not train or evaluate an adoption predictor on interviews and then describe those same interviews as independent validation.

### E. Network mitigation and service continuity

Compare no mitigation, documented operator rules, deterministic optimisation, robust/chance-constrained optimisation and receding-horizon policies with targeted observations.

Measure network-wide redistribution: an intervention that lowers current in the target transformer but increases unacceptable risk elsewhere is not a success.

Report each essential service independently. A high average delivered-service percentage must not mask a critical hospital, water-supply, communications or regional floor breach. Include tail and worst-region results, common-cause failures and recovery time.

### F. Offline and federated coordination

Run 72-hour partition exercises with delayed and duplicated messages, clock uncertainty, revoked or expired authority, stale source data, conflicting resource claims, key compromise simulations and reconnect/reconciliation.

Measure:
- unauthorized action count;
- stale-authority acceptance;
- conflict detection and reconciliation time;
- duplicate allocation;
- evidence freshness degradation;
- ability to keep operating within pre-authorized local scope.

Connectivity loss must never expand authority.

## 6. Frozen public and regional scenario material

| Scenario/data family | Benchmark role | Qualification boundary |
|---|---|---|
| NERC TPL-007-4 reference and supplemental waveforms | Reproducible GMD reference forcing and earth-model scaling | Compliance/reference lane; not a South African hazard model or operational forecast. |
| May 2024 TVA/GMU four-site replay | Data ingestion, aggregation, polarity and published-metric reproduction | Do not use the same event to fit and claim independent forecast skill. |
| May 2024 43-site reference study | Broader measured/modelled reference comparison | Retain the dataset's source/licence and measurement semantics; follow existing #706 work. |
| Grassridge October 2003 and Hermanus magnetic data | South African historical event reconstruction | Published GIC records exist; exact raw-data access, topology and calibration must be independently qualified. |
| Botrivier March 2015 differential-magnetometer study | South African line-current proxy and spatial representativeness | Do not equate an under-line magnetometer-derived proxy with transformer-neutral current. |
| Matimba and other South African historical records | Regional transfer and second-site validation | Confirm exact source products, event overlap, owner and access rights before scoring. |
| Texas A&M ACTIVSg2000 | Synthetic GMD network and physical consequence experiments | Synthetic network; it does not represent an actual Texas or South African utility grid. |
| Texas A&M ACTIVSg200 black-start scenarios | Restoration and resource-constrained recovery | Separate restoration case family from the GMD network case; declare coupling assumptions. |
| New Zealand Transpower Gannon 2024 published mitigation | Operational counterfactual case study | Use published results only unless owner-authorised time series and model details are obtained. |

South African source progression:

Repository catalogue -> station/instrument record -> specific dataset product -> calibration/clock/rights qualification -> spatial representativeness -> measurement operator -> physical model input.

Do not claim a SANSA dataset is downloaded, commercially licensed, calibration-qualified or representative of a transformer site until the corresponding evidence exists.

## 7. Industry and SOTA comparison rules

A fair benchmark must respect the scope of competing systems:

- **GeoDGP** provides global probabilistic forecasts of ground magnetic perturbations. It belongs in the magnetic-driver lane, not a direct GIC leaderboard.
- **MM-CNN-MHA (2026)** reports GIC forecasting at a single 220-kV Fujian substation. Compare only with the same site, target definition, lead time, input availability, event split and preprocessing.
- **UK SWIMMR** demonstrates operationalised models for electricity, high-pressure gas and rail impacts. It is a serious operational integration comparator.
- **New Zealand Transpower** provides a real operational mitigation case with published measured and no-mitigation counterfactual results. This should shape the network-wide intervention benchmark.
- **NERC TPL-007-4** supplies a documented planning/reference framework. Compliance is necessary context, not evidence of superior operational resilience.
- **Operator mitigation decisions (2026)** provide an empirical, limited-sample adoption baseline. Use accept/modify/defer/reject/no-action and rationale/outcomes; do not treat the 18% response sample as population prevalence. DOI: https://doi.org/10.1029/2026SW005072.

Do not claim SOTA from one-site RMSE, a synthetic fixture, a single-event replay, a dashboard or a benchmark with easier observation coverage. Report matched baselines, source versions, holdout design, confidence intervals and failed cases.

## 8. South African validation path

1. **Source and rights:** complete the SANSA SANDIMS catalogue workflow and request a dataset-specific inventory, access conditions and calibration/clock metadata. Academic registration is not commercial ingestion permission.
2. **Historical cases:** qualify Grassridge/Hermanus 2003, Botrivier 2015, and Matimba records only as far as raw data, provenance and source rights permit.
3. **Regional physics:** establish local conductivity and MT response evidence; keep 1-D approximations, regional proxies and measured E-field products distinguishable.
4. **Synthetic network:** use the ACTIVSg GMD cases for solver and policy development; never present them as Eskom topology.
5. **Owner collaboration:** obtain owner-approved topology, transformer data, protection limits, service dependencies and mitigation procedures before making real-asset claims.
6. **Field exercise:** only after qualified models, independent review, safety case and explicit human authorization.

## 9. Evidence ladder and acceptance gates

Use E0-E7 as an evidence classification, not as an overall product badge:

| Level | Evidence | What it supports |
|---|---|---|
| E0 | Research design / stated hypothesis | No performance claim. |
| E1 | Schema and structural validation | Input/output contract integrity. |
| E2 | Unit, analytical and negative-control tests | Software logic for declared fixtures. |
| E3 | Independent historical replay | Reproduction of identified event data / published results. |
| E4 | Qualified physical model and independent site/event validation | Physical inference only in the validated domain. |
| E5 | Hardware/controller/Power-HIL or laboratory evidence | Tested hardware interaction in the declared setup. |
| E6 | Owner-authorised field exercise | Exercise-specific operational evidence. |
| E7 | Repeated multi-event operational evidence and independent review | Bounded operational claims within the demonstrated domain. |

Initial software acceptance:
- deterministic replay from a frozen scenario manifest;
- no runtime network access;
- exact digest and schema validation;
- observation latency applied before action selection;
- positive-VOI, zero/negative-VOI and too-late-observation cases;
- stale/correlated sensor failure case where naive VOI performs worse;
- resource unavailable, approval pending/denied and scope-conflict cases;
- service floor cannot be hidden by another service's surplus;
- HOLD/OBSERVE/DEFER/ABORT can be selected;
- explicit tests for no false-ready and no hard-constraint violation on the frozen corpus;
- positive controls prove valid scenarios are accepted;
- exact-head format, compile, tests and Clippy evidence.

For empirical claims, publish numerators, denominators, confidence intervals, event/site coverage and all exclusions. Zero observed failures in a finite corpus is not a universal safety guarantee.

## 10. Immediate implementation plan

1. Keep #714, #717 and #722 independent and complete their own Rust qualification.
2. Build a tiny dependency-light Rust decision-value oracle over frozen scenario states, observation outcomes, action options and per-service losses.
3. Start with six fixtures: positive VOI, zero/negative VOI, observation arriving too late, common-mode sensor error, unavailable resources, and pending/denied human approval.
4. Compose the outputs of the existing three oracles through explicit typed contracts; do not duplicate their logic.
5. Emit a deterministic JSON receipt binding scenario, policy, evidence, observation timing, action decision, service-loss vector, costs, digests and build identity.
6. Add synthetic-network restoration experiments and independent event/site holdouts only after the small oracle is reproducible.

Do not wait for queued workflow runs to design new cases or implement independent code. Do not weaken qualification gates to obtain a green result. Clearly distinguish implemented, locally checked, structurally checked, CI pending, historically replayed and physically validated work.

## 11. Research references

- NERC, TPL-007-4: https://www.nerc.com/globalassets/standards/reliability-standards/tpl/tpl-007-4.pdf
- Chen et al., GeoDGP, Space Weather (2025): https://doi.org/10.1029/2024SW004301
- Zhang et al., MM-CNN-MHA GIC forecasting, Space Weather (2026): https://doi.org/10.1029/2025SW004900
- Beggan et al., UK SWIMMR operational models (2025): https://doi.org/10.1029/2025SW004364
- Mac Manus et al., New Zealand Gannon mitigation (2025): https://doi.org/10.1029/2025SW004388
- Bernhardi, Cilliers & Gaunt (2008), South African Grassridge/Hermanus spatial-interpolation and GIC modelling: https://doi.org/10.1029/2008SW000408
- South African Botrivier differential-magnetometer measurements: https://doi.org/10.1002/2015SW001289
- SANSA SANDIMS: https://sandims.sansa.org.za/
- Texas A&M synthetic electric grid test cases: https://electricgrids.engr.tamu.edu/electric-grid-test-cases/
- Texas A&M ACTIVSg200 restoration scenarios: https://electricgrids.engr.tamu.edu/restoration-data-and-scenarios-for-activsg200/
- IEEE PES, Reliability and Resilience for New Grid Needs (TR148, 1 October 2026): https://resourcecenter.ieee-pes.org/publications/technical-reports/pes_tr_tr148_itslc_102026
- IEEE PES, *Power System Resilience Metrics and Evaluation Methods* (TR143, 8 June 2026): https://doi.org/10.17023/7e16-km93
- Grochowicz, Bloomfield & Victoria, *Resilience metrics to guide back-up investments in the power system during extreme weather* (Nature Sustainability, 14 July 2026): https://doi.org/10.1038/s41893-026-01897-6

## 12. Explicit non-claims

This protocol does not claim:
- that Luminous currently forecasts GIC better than published models;
- that any synthetic test network represents a real utility;
- that a particular SANSA dataset has been obtained or licensed for commercial use;
- that historical proxy measurements are direct transformer-neutral GIC;
- that a passing software oracle certifies transformer safety or grid stability;
- that a recommendation is an executable instruction;
- that essential services are protected until service-level evidence establishes that result.
