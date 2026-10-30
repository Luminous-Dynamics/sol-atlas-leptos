# Industry Engineering Assurance Contract (IEAC) v0.1

**Status:** Research/RFC and schema contract; no Luminous industry capability is qualified by this document.  
**Issue:** [IND-ASSURANCE-001](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/327)  
**Date:** 2026-10-30  
**Primary purpose:** qualify a specific operational claim for a specific decision, instance and validity envelope.

## 1. Executive decision

The Luminous projects should share an Industry Engineering Framework, but its assurance mechanism must not be a single maturity ladder.

A capability can be:
- implemented but not numerically verified;
- numerically verified but not physically validated;
- physically validated on one asset but not another;
- well calibrated but trained on unauthorized or unrepresentative data;
- scientifically credible but not secure or operationally integrated;
- operationally useful in advisory mode but not authorized for autonomous control.

These are different states. A scalar E0–E6 score hides them.

**IEAC v0.1 introduces a multi-dimensional evidence vector, an executable validity envelope, and decision-specific authority tiers.** Qualification applies to the claim and operating context, not globally to a software repository or model family.

The framework composes existing project-specific owners. It does not add another capability ontology, physics engine, provenance graph, evidence-independence algorithm or universal benchmark score.

## 2. Existing assets to compose

The current engineering programme already contains substantial foundations:

| Existing work | Reuse boundary |
|---|---|
| [Sol-Atlas Capability Atlas PR #18](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/18) | Reuse its separation of capability definition, instance, geographic presence, operational evidence and qualification. Do not create a competing ontology. |
| [Sol-Atlas artifact-chain conformance PR #273](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/273) | Reuse exact cross-project artifact pins, link validation, authorization semantics and synthetic claim ceilings. |
| [Mycelix evidence-independence profile PR #5526](https://github.com/Luminous-Dynamics/mycelix/pull/5526) | Reuse profile-specific assessments for reviewer diversity, organization independence, empirical replication and computational reproduction. Do not equate different actor IDs with independent evidence. |
| [Symtropy maintenance Tier-A reference PR #3015](https://github.com/Luminous-Dynamics/symtropy/pull/3015) | Reuse exact event/repair reference cases and the distinction between scripted event correctness and a production maintenance predictor. |
| [Symtropy model-uncertainty work](https://github.com/Luminous-Dynamics/symtropy/issues/2563) | Reuse explicit model-form and parameter uncertainty analysis for decision robustness. |
| [Symthaea CropFoundry PR #8046](https://github.com/Luminous-Dynamics/symthaea/pull/8046) | Reuse source permission inventory, grouped leakage controls, trial-power preflight, bounded irrigation policy evaluation, postharvest accounting and nutrient accounting. |
| [Sol-Atlas spatial-economic resilience PR #116](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/116) | Reuse capacity-aware service networks, common-cause stress, service floors and evidence-aware intervention comparisons. |

IEAC should be a thin qualification and claim-ceiling layer over those contracts.

## 3. Paradigm shift A — qualification is a vector, not a ladder

Every claim carries a versioned evidence vector. At minimum, assess these dimensions independently:

1. **Software/interface conformance** — exact source, dependency, schema and interface behavior.
2. **Numerical verification** — known answers, invariants, conservation, convergence, solver checks and independent implementations.
3. **Physical/empirical validation** — comparison to observations in a declared validity domain.
4. **Data integrity and rights** — provenance, permission, quality, freshness, measurement traceability and license status.
5. **Uncertainty credibility** — calibration, interval coverage, proper scores, sensitivity and model-form uncertainty.
6. **Generalization** — held-out machines, sites, topologies, seasons, users, process families and operating regimes.
7. **Robustness** — missing, biased, stale, corrupted, shifted and adversarial inputs; failure-mode coverage.
8. **Operational assurance** — integration, monitoring, recovery, fallback, maintainability and human factors.
9. **Safety/security/authority** — hazard controls, cybersecurity, privacy, applicable regulation, responsible approvals and permitted actions.
10. **Prospective outcome/value** — independently reconciled service, quality, throughput, cost, resource use and user outcomes.

Each dimension uses one of five states:

| State | Meaning |
|---|---|
| established | The profile's specified evidence and acceptance rule have been met within the stated scope. |
| not_established | Evidence exists but does not meet the requirement, or the required test has failed. |
| unknown | Evidence is absent, insufficient, inaccessible or not yet assessed. |
| conflicted | Admissible evidence disagrees or provenance/semantics conflict. |
| not_applicable | The dimension is genuinely irrelevant to this profile; requires an explicit rationale and reviewer. |

Unknown, conflicted and not_established never become passes through averaging. A mandatory dimension cannot be waived by high scores elsewhere. A not_applicable decision is itself reviewable evidence.

There is no universal weighted confidence score. A summary display may show the vector, but it must preserve every dimension and its underlying evidence.

## 4. Paradigm shift B — qualify a decision in an operating envelope

The same model can be useful for retrospective reporting, acceptable for a human-reviewed recommendation, and unacceptable for direct actuation.

Each qualification profile binds:
- the exact claim, intended decision and intended users;
- capability and deployed-instance identity;
- model, evaluator, source and configuration artifacts;
- asset/site/topology/process identity;
- units, ranges, time support, geography, operating regime and population;
- calibration and validation data represented by the evidence;
- excluded domains and known counterexamples;
- output tolerance and decision thresholds;
- failure modes, fallbacks and required human approvals;
- jurisdiction, data rights, privacy, safety and security constraints;
- evidence validity dates, re-review triggers and revocation conditions.

Qualification must not transfer automatically to a different asset, site, crop, topology, chemistry, device, population, jurisdiction or model version.

### Executable validity envelope

Every profile needs a machine-readable applicability envelope. An evaluator must classify an input as:
- in_domain;
- marginal;
- out_of_domain;
- indeterminate.

The classification must be based on declared tests and thresholds, not on a model's self-reported confidence alone. Missing/stale sensors, expired calibration, unknown topology, unsupported units, configuration drift, unseen process regimes or source-permission loss may require abstention.

An out-of-domain input must never silently receive an in-domain positive qualification. The system should abstain, request a measurement, or route to an independently validated fallback.

## 5. Paradigm shift C — scale assurance with decision risk

Assurance effort should be allocated according to the consequence of a wrong decision and the influence of the model—not equally across every model and not according to model size.

NIST AMS 100-85, published 5 October 2026, describes a risk-based credibility approach that scales VVUQ according to model influence and decision consequence. This is an important external grounding for IEAC's assurance-budget principle.

Each profile should record a reviewed risk assessment across separate dimensions:

| Risk dimension | Questions |
|---|---|
| Consequence | What is the severity of a wrong decision for people, assets, environment, service or finances? |
| Model influence | Does the model inform, constrain, select or directly execute the decision? |
| Exposure | How many assets/users and what time/economic/environmental scope are affected? |
| Reversibility | Can the decision be reversed, and at what cost or delay? |
| Detectability | How quickly and reliably would an erroneous output or adverse outcome be detected? |
| Fallback | Is an independent safe fallback available, tested and operational? |

Do not collapse these dimensions into an arbitrary universal numeric risk score. A reviewed risk class selects mandatory tests, reference models, holdouts, human approvals, monitoring and recovery requirements. The class and its rationale are versioned and auditable.

### Decision authority tiers

These are action-authority tiers, not maturity grades:

| Tier | Permitted use | Additional assurance expectation |
|---|---|---|
| D0 — descriptive | Retrospective reporting and visualization; no recommendation | Data integrity, numerical correctness for reported quantities, scope and uncertainty disclosures |
| D1 — advisory | Generate/rank recommendations for an accountable human decision; no external side effects | Use-case validation, validity envelope, uncertainty/calibration, OOD behavior, explanation, human review |
| D2 — delegated planning | Schedule or allocate within a bounded, pre-approved envelope | D1 plus independent constraint verifier, risk-specific stress tests, action limits, audit trail, rollback and operational monitoring |
| D3 — autonomous control | Real-time external actuation | Separate safety case and hazard analysis, HIL/field qualification, independent safety assessment, secure control boundary, tested fail-safe and recovery, continuous assurance and explicit operational authorization |

D3 cannot be earned by accumulating documentation, unit tests or synthetic benchmark wins. It requires domain-specific safety engineering and the applicable operational/regulatory authority.

## 5.1 Minimum mandatory evidence by decision tier

The profile author cannot choose the minimum assurance bar by marking difficult dimensions optional. The validator and JSON Schema enforce this baseline for a profile declared `qualified_for_profile`:

| Dimension | D0 descriptive | D1 advisory | D2 delegated planning | D3 autonomous control |
|---|---|---|---|---|
| Software conformance | Mandatory | Mandatory | Mandatory | Mandatory |
| Numerical verification | Mandatory | Mandatory | Mandatory | Mandatory |
| Physical validation | Profile-specific | Mandatory | Mandatory | Mandatory |
| Data integrity and rights | Mandatory | Mandatory | Mandatory | Mandatory |
| Uncertainty credibility | Profile-specific | Mandatory | Mandatory | Mandatory |
| Generalization | Profile-specific | Mandatory | Mandatory | Mandatory |
| Robustness | Profile-specific | Mandatory | Mandatory | Mandatory |
| Operational assurance | Profile-specific | Mandatory | Mandatory | Mandatory |
| Safety/security/authority | Profile-specific | Mandatory | Mandatory | Mandatory |
| Prospective outcomes | Profile-specific | Profile-specific | Mandatory | Mandatory |

“Profile-specific” means the dimension may be optional for that decision tier, not that an unknown result is treated as established. A mandatory dimension must be `established` and point to at least one evidence record. If a mandatory dimension is genuinely not applicable, the profile must be redesigned or the tier reduced; it cannot be silently waived.

These are Luminous internal minimums. Domain-specific safety standards, legislation, operator authorization and risk review may require stricter evidence.

## 6. Evidence provenance is not physical truth

Keep these claims distinct:
- a source was identified;
- the source is accessible and permissioned;
- a file's bytes match a digest;
- a transformation is reproducible;
- an evaluator is independently implemented;
- evidence sources are independent for a particular scientific/operational profile;
- a model is valid for a particular physical domain;
- a decision was authorized;
- an intervention actually occurred;
- the intended outcome was measured and reconciled.

A hash proves byte identity, not source truth. A signature proves who signed, not that work happened. A second software run proves computational reproducibility, not independent empirical replication. A scenario simulation is not a field observation.

Mycelix's evidence-independence profiles should supply profile-specific lineage analysis where the use case requires it. The IEAC record must preserve unknown and conflicted results rather than converting them into an independence count.

### Artifact identity and data-rights binding

Every artifact uses one of two explicit identity forms:

- **Git artifact:** repository owner/name, exact commit SHA, repository-relative path and Git blob SHA-1. An optional content SHA-256 may also be recorded.
- **External artifact:** source URI plus SHA-256 of the acquired bytes. A version, acquisition time and rights statement should be included in the profile evidence record or its referenced acquisition manifest.

A Git blob SHA-1 is a Git object identifier; it is not a SHA-256 content digest. Never place one in the other's field. A digest binds bytes only after the bytes have actually been acquired and hashed. The current local validator checks digest syntax and references but does not fetch or recompute remote artifact bytes.

A profile declared qualified requires every bound artifact to have a verified rights state. Unknown, restricted or unresolved rights cannot support a qualified decision. A downloadable file is not automatically authorized for training or operational use.

## 7. Required profile structure

The machine-readable qualification profile should include:

1. Schema/profile ID and version.
2. Claim ID, description, intended user and decision.
3. Capability ID and deployed instance ID.
4. Requested D0–D3 decision tier.
5. Operating envelope, exclusions and known counterexamples.
6. Reviewed risk dimensions and rationale.
7. Ten-dimension evidence vector with state, requiredness, evidence references, scope and validity.
8. Exact source/model/evaluator/configuration artifact bindings.
9. Data provenance and license/permission state.
10. Evidence-independence profile where relevant.
11. Holdout split identity, group boundaries and leakage controls.
12. Uncertainty/calibration outputs and tolerances.
13. Required human approvals, fallback and prohibited actions.
14. Qualification disposition: qualified_for_profile, not_qualified, indeterminate or revoked.
15. Valid-from, review-by, expiry and revocation triggers.
16. Evaluator identity, exact input/report digests and reviewer identities.

A report must include counterexamples, limitations, missing evidence and reasons for abstention—not only a positive result.

## 8. Benchmark programme

Benchmarks should be separated into task-specific lanes. Do not create a single cross-industry leaderboard.

### A. Claim-ceiling and semantic integrity
Seed false promotions:
- synthetic fixture labelled observed;
- simulation labelled field validation;
- computational reproduction labelled empirical replication;
- signed record labelled physically verified;
- digest match labelled source authenticity;
- capability definition labelled deployed instance;
- recommendation labelled authorized action.

Acceptance: every invalid promotion is rejected or marked indeterminate.

### B. Risk and authority correctness
Test D0→D3 escalation, stale approvals, changed risk classes, missing fallback, self-review, and mandatory-dimension waiver attempts.

Acceptance: a lower-tier profile cannot authorize a higher-tier action; no mandatory dimension can be averaged away; any changed claim/model/context invalidates the previous qualification until re-evaluated.

### C. Validity-domain and OOD behavior
Use held-out asset/site/topology/season/process-family splits, not only random row splits. Inject sensor drift, unsupported units, range excursions, calibration expiry, missing fields, topology changes and population shift.

Measure:
- selective risk versus coverage;
- false-confident qualification rate;
- abstention precision/recall;
- calibration and interval coverage by subgroup/regime;
- value of information from requested measurements.

### D. VVUQ
Require analytical references, invariants, convergence, matched independent solvers and physical comparison where available. Report parameter and model-form uncertainty separately. Use appropriate metrics such as normalized errors, bias, multivariate validation distances, interval coverage and proper interval scores only where the data/task supports them.

### E. Decision and operational value
Compare the proposed system with named incumbent policies and bounded small-instance oracles. Measure operational regret, useful output/service, quality/safety floors, downtime, repair/crew burden, energy/water/material use, whole-life cost, human review time, overrides and recovery. Reconcile predictions to realized results independently.

### F. Reproducibility and resource budget
Bind exact source, data, model, evaluator, environment and configuration. Retain seeds for stochastic runs. Equalize information, data, tuning and compute/energy budgets when comparing methods. Report computing energy separately from physical process energy. A candidate selector and its independent verifier must not share the same decision logic.

### Reporting rule

There is no universal weighted score. Report:
- per-lane metrics and denominators;
- risk and claim scope;
- uncertainty and subgroup performance;
- hard constraint violations;
- operational outcomes;
- Pareto frontiers;
- runtime, memory and computational energy;
- reproducibility and verifier identity.

## 9. Starter application profiles

The first three applications should demonstrate different assurance needs:

| Application | Starting decision tier | Core external reference | Main validity challenge |
|---|---|---|---|
| Telecom site/backhaul resilience | D1 advisory | Declared topology and capacity references; later ITU-R P.1812-8 and 5G-LENA in separate RF/network lanes | Site inventory completeness, physical route diversity, battery/load evidence, aggregate service denominators |
| Water–food–energy resilience | D1 advisory | EPANET, AquaCrop and FAO WaPOR in separate hydraulic, crop-response and remote-sensing lanes | Sensor/source comparability, soil/crop/site transfer, seasonal holdouts, water-energy coupling |
| Industrial condition-based maintenance | D1 advisory | NIST SimPROCESD/Simantha and NASA/PHM datasets in task-matched lanes | Machine-wise/grouped holdouts, sensor degradation, failure lead time, maintenance crew capacity and production impact |

A decision tier is only a proposed starting authority. Each actual profile must derive its mandatory requirements from consequence, influence, exposure, reversibility, detectability and fallback.

## 10. External standards and research

- **NIST AMS 100-85**, 5 October 2026: *On Methods of Uncertainty Quantification in Computational Modeling and Simulation: Harmonizing Modeling and Metrology Practices*. Risk-based credibility, ASME/JCGM UQ, uncertainty propagation, sensitivity and digital-twin integration. https://www.nist.gov/publications/methods-uncertainty-quantification-computational-modeling-and-simulation-harmonizing
- **NISTIR 8620**, 21 July 2026: *Digital Twins Workshops Summary Report*. Identifies interoperability, VVUQ, cybersecurity and workforce readiness as current digital-twin research and standards priorities. https://www.nist.gov/publications/digital-twins-workshops-summary-report
- **ISO/AWI 23247-8**: an approved new work item registered on 23 October 2026 and under development. It scopes VVUQ across the manufacturing digital-twin lifecycle. It is not yet a published standard. https://committee.iso.org/standard/94858.html
- **ASME VVUQ portfolio**: terminology, lifecycle and domain-specific verification and validation references. https://www.asme.org/codes-standards/vvuq-standards
- **W3C PROV-O**: reuse established provenance semantics rather than inventing a generic provenance ontology. https://www.w3.org/TR/prov-o/
- **OPC UA and Asset Administration Shell**: candidate interoperability adapters where relevant; neither is mandatory for every domain.

IEAC is an internal engineering contract and is not an ISO, ASME, NIST or regulatory certification.

## 11. Acceptance criteria for v0.1

- A profile cannot qualify when any required evidence dimension is unknown, conflicted or not_established.
- A higher decision tier cannot inherit qualification from a lower tier without satisfying additional requirements.
- Out-of-domain, stale or unsupported inputs cannot yield an in-domain positive qualification.
- Qualification is bound to exact capability, instance, model, data, evaluator and configuration identities.
- Missing/invalid source permissions and required provenance fail closed.
- not_applicable cannot waive a required dimension without rationale and reviewer identity.
- Changing a required artifact digest, validity envelope, risk class, decision tier or evidence status invalidates the prior report.
- Synthetic fixtures cover every declared false-promotion and authority-escalation path; 100% of negative controls must fail closed or return indeterminate.
- Every qualification report is reproducible and independently checked.
- No industry, regulatory, safety or production qualification is claimed from schema conformance or synthetic fixtures alone.

## 12. Phased implementation

- **P0 — RFC and schema:** freeze terms, decision tiers, evidence states and qualification-profile schema.
- **P1 — Rust validation:** implement strict parsing, semantic checks and an independent report checker; test all seeded negative controls.
- **P2 — artifact composition:** consume exact Sol-Atlas/Mycelix/Symtropy/Symthaea artifacts using existing project-specific validators.
- **P3 — vertical profiles:** publish telecom, water–food–energy and maintenance profiles with their own validity envelopes and domain references.
- **P4 — shadow qualification:** compare model outputs with independently measured/authorized operational observations; no actuation.
- **P5 — bounded operational authorization:** only after domain safety, cybersecurity, human authority, rollback and continuing assurance requirements are met.

## 13. Non-goals

- No new generic capability ontology, evidence-independence engine, provenance graph or physics solver.
- No universal numerical risk score or scalar confidence score.
- No one leaderboard spanning incomparable scientific and operational tasks.
- No declaration that any existing Luminous vertical is already qualified, certified or production-assured.
- No production actuation or automatic authorization in v0.1.
- No SOTA claim before matched, independent and preferably prospective evaluation.

## 14. Decision

Adopt IEAC as the shared assurance contract for the first industrial pilots and future verticals. Reuse the existing project owners, qualify each claim against its actual decision and operating envelope, scale the assurance budget to decision risk, and keep every unsupported dimension visible.
