# SA-WORKFORCE-DIAG-001 — Workforce Bottleneck Diagnosis Benchmark v0.1

**Status:** research contract / synthetic-only first slice  
**Tracking issue:** https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/267  
**Related:** Sol-Atlas #214 and PR #205; Mycelix #5015; Symthaea HUMAN-TOOL-001 #7983.

## 1. Thesis

Do not infer a worker shortage from a vacancy count, queue, or service backlog. Diagnose the binding constraints that prevent defined work from being completed, then choose an admissible intervention and verify the outcome.

**Unmet work → evidence-bounded diagnosis → intervention preconditions → authorized execution → independently verified outcome**

This is an integration contract and benchmark, not a second portfolio optimizer, job marketplace, ticket system, occupational taxonomy, credential authority, or service-state machine.

## 2. Why this is a useful missing layer

South Africa's Department of Higher Education and Training explicitly states that its 2024 National List of Occupations in High Demand is not a scarce-skills list. It supports enrolment planning and other workforce decisions; some occupations with unusually high registered-unemployed counts warrant investigation. A national occupation label cannot establish a local, task-specific, time-specific shortage.

The World Bank's 14 August 2026 account of South Africa's digital-skills pipeline describes simultaneous unemployment and unfilled digital vacancies, with constraints spanning foundational skills, education alignment, and workplace learning. This supports a multi-cause diagnostic framing; it does not validate this benchmark or prove that an AI system fixes those constraints.

Research anchors:
- DHET, 2024 National List of Occupations in High Demand: https://www.gov.za/documents/notices/2024-national-list-occupations-high-demand-16-apr-2024
- Official notice PDF: https://www.gov.za/sites/default/files/gcis_document/202404/50510gen2414.pdf
- World Bank, From Demand to Delivery (14 Aug 2026): https://www.worldbank.org/en/news/feature/2026/08/14/from-demand-to-delivery-strengthening-south-africas-digital-skills
- O*NET database and task/skill descriptors: https://www.onetcenter.org/database.html
- European Commission ESCO classification: https://esco.ec.europa.eu/en/classification

O*NET and ESCO are useful reference systems, but their coverage and occupational relationships must not be silently treated as South African local truth. Record taxonomy versions and crosswalk provenance. A taxonomy match does not prove a person is practically competent, available, willing to accept the working conditions, or authorized to perform regulated work.

## 3. Causal bottleneck vocabulary

| Cause | Meaning | Typical remedy family |
|---|---|---|
| SKILLS_CAPACITY | Qualified, task-ready capacity is insufficient for funded and feasible demand in the relevant place/time window | Supervised skill development, apprenticeship, qualified redeployment or funded hiring |
| FUNDING_OR_PAYER | No evidenced budget, purchaser, procurement route or payment commitment supports the work | Establish lawful funding/payer/procurement |
| TOOLS_OR_SPARES | Required parts, tools, equipment, energy or consumables are unavailable | Procurement, inventory or qualified substitution |
| AUTHORIZATION_OR_SAFETY_GATE | Licence, permit, inspection, supervision or other mandatory authority is absent or unverified | Obtain qualified review/authorization; block unsafe execution |
| DISPATCH_OR_GEOGRAPHY | Capacity exists but travel, shift overlap, routing, access or scheduling prevents its use | Route/workflow/shift redesign |
| WAGE_OR_RETENTION | Qualified people do not accept or remain in work under current compensation or conditions | Review pay, shifts, working conditions, safety, retention and worker voice |
| DEMAND_DATA_QUALITY | Duplicate, misrouted, stale or poorly specified demand distorts the apparent workload | Deduplicate, validate and re-triage |
| PRACTICAL_COMPETENCE_UNVERIFIED | A credential does not establish present practical ability for this task/profile | Scoped practical assessment and supervised work |
| EVIDENCE_INSUFFICIENT | Missing, stale, contradictory, biased or unrepresentative evidence prevents a defensible diagnosis | Collect/revalidate evidence and abstain from unsupported specificity |
| INTERACTING_BOTTLENECKS | Two or more constraints jointly limit completion | Explicit dependency-aware bundle; verify every precondition |

Causes are not mutually exclusive. Preserve cause sets and an unknown state. Do not force one “root cause” when several constraints bind or evidence cannot distinguish them.

## 4. System responsibilities

- **Sol-Atlas:** represent where/when demand occurs, provenance/coverage, and spatial access/resource constraints. Do not label a real skills shortage from a map or vacancy count alone.
- **Symthaea:** assist diagnostic reasoning and explain missing/conflicting evidence. Advice is not qualification, a licence, approval, or execution authority.
- **Symtropy:** simulate declared queueing, dependency, dispatch and resource scenarios. Simulated throughput remains a model result until validated against appropriate observations.
- **Mycelix:** compose evidence references, scoped credentials, human approvals, paid-work receipts and correction/appeal pathways using existing owners. Credential possession, reputation, employment status and execution authority remain distinct.

No system may manufacture a missing payer, bypass a safety gate, or convert planned work into a completed job. Keep source-of-truth and authorization boundaries with legitimate owners.

## 5. Frozen synthetic corpus

The v0.1 corpus contains 12 hand-authored cases, emphasizing confounding and negative controls:

1. true task-ready capacity shortfall, with funding and tooling available;
2. apparent shortage caused by an absent payer/funding route;
3. worker and funded job blocked by a missing spare;
4. qualified worker blocked by mandatory authorization/inspection;
5. sufficient aggregate staffing but incompatible dispatch/travel windows;
6. qualified workers decline an unviable shift/pay/condition profile;
7. stale or contradictory observations require evidence gathering;
8. novice gets a safe, supervised human + Symthaea diagnostic workbench;
9. skills and spares are jointly binding;
10. duplicate reports inflate apparent demand;
11. credential exists but task-specific practical competence is unverified;
12. multiple constraints and missing critical evidence require a bounded result rather than an invented single cause.

The JSON fixture stores synthetic observations, expected cause sets, admissible and prohibited intervention labels, and a required disposition. Its strict structural schema is [workforce_bottleneck_scenarios_v0_1.schema.json](schemas/workforce_bottleneck_scenarios_v0_1.schema.json). These labels are a tiny oracle, not empirical claims about actual workers or services. Values are artificial; every case is marked SYNTHETIC_TEST_ONLY.

## 6. Baselines and protocol

Use the same frozen input and evaluation split for every comparator:

- **B0 — incumbent practice:** current documented workflow, only where a real service owner supplies a valid baseline.
- **B1 — vacancy/backlog heuristic:** assumes worker shortage; intentionally weak to expose false-shortage errors. The standalone Rust verifier emits a reproducible receipt for this synthetic baseline. Because all 12 frozen cases have a reported backlog, B1 predicts `SKILLS_CAPACITY` as the only cause in all 12 cases. The authored oracle contains a skills-capacity cause in 4 cases, so this deliberately weak baseline generates 8 false shortage diagnoses (8/12, 66.7% of its positive predictions) and exactly matches the complete cause set in only 2/12 cases. These are calculated properties of an artificial fixture, not industry performance or a measured error rate in a real workforce.
- **B2 — taxonomy matching:** fixed version of an occupational/skills reference plus disclosed local crosswalks and limitations.
- **B3 — transparent rule baseline:** reviewable human-readable rules.
- **B4 — independent exact oracle:** exhaustive feasible intervention combinations on the tiny frozen corpus. Keep it separate from candidate selection logic.
- **Candidate — evidence-aware diagnosis:** multi-cause outputs, intervention preconditions, explicit abstention, and machine-readable receipts.

Do not tune on expected labels and report those same cases as held out. V0.1 measures contract correctness, not statistical generalization. A real evaluation needs a sequestered, independently labelled corpus representative of the intended setting.

### Primary metrics

1. **False skills-shortage rate** on negative controls whose blocker is funding, tools/spares, authorization, dispatch, pay/retention, demand quality or stale evidence.
2. **Cause-set exact match and per-class precision/recall**, with macro and micro aggregates.
3. **Intervention admissibility violations:** any recommendation bypassing a hard prerequisite is a critical failure.
4. **Intervention regret** versus B4, using only a predeclared synthetic cost/outcome table; do not invent probabilities or hide protected outcomes inside an opaque scalar.
5. **Abstention quality:** coverage versus error when evidence is insufficient/conflicting.
6. **Robustness:** row-order permutation, missing/stale/conflicting evidence, duplicates and interacting blockers.
7. **Distributional slices:** geography/access, evidence completeness, task family and worker pathway where lawfully supported. Avoid individual risk scores and protected-attribute shortcuts.
8. **Later field endpoints:** verified completion time, first-time fix, repeat work, service restoration, fully accounted cost per verified resolution, worker pay/payment delays, safety, training progression and 6/12-month job retention.

## 7. Acceptance gates

- 100% exact cause-set and intervention-admissibility agreement with the independent oracle on the frozen v0.1 fixture.
- Zero false SKILLS_CAPACITY diagnoses in the specified critical negative controls.
- Zero mandatory authorization/safety/payer/qualification-scope violations.
- Unknown/stale/conflicting evidence yields an explicit bounded/unknown disposition, not fabricated precision.
- Input row permutation leaves canonical output unchanged.
- Every output identifies corpus digest, schema/algorithm version, evidence states, cause set, admissibility dispositions, omitted/unknown reasons and claim ceiling.
- No synthetic job, wage, cost, time or service result is described as observed or causal.
- If any case is unprocessed, mark the report incomplete; never label a partial result complete.

Do not set runtime/scale claims before a repeatable run exists. When a Rust implementation is added, report median and p95 runtime, peak memory, corpus counts, hardware/toolchain and candidate-versus-oracle agreement.

## 8. Industry / SOTA comparison without overclaiming

Commercial incumbents already cover important parts of the scheduling problem:

- **Microsoft Dynamics 365 Field Service:** Universal Resource Scheduling and the separately licensed Resource Scheduling Optimization add-on cover resource/requirement matching, work-order booking, locations, time windows, and configurable optimization goals. Official docs: https://learn.microsoft.com/en-us/dynamics365/field-service/universal-resource-scheduling-for-field-service and https://learn.microsoft.com/en-us/dynamics365/field-service/rso-configuration
- **ServiceNow Field Service Management:** dynamic scheduling can assign work using configured criteria such as skills, travel time, and priority; its broader FSM scope includes schedules, crews, inventory/equipment, work orders, and dispatch. Official docs: https://www.servicenow.com/docs/r/field-service-management/field-service-scheduling/setting-up-scheduling-methods.html and https://www.servicenow.com/docs/r/field-service-management/use-fsm.html
- **Salesforce Field Service:** scheduling policies, work rules, and objectives determine resource/time-slot candidates using such factors as qualifications, proximity, urgency, and travel. Official docs: https://help.salesforce.com/s/articleView?id=service.pfs_customization.htm&language=en_US and https://help.salesforce.com/s/articleView?id=sf.pfs_scheduling.htm&language=en_US&type=5
- **O*NET / ESCO:** occupational/task/skill-reference systems, not field-service execution engines. See https://www.onetcenter.org/database.html and https://esco.ec.europa.eu/en/classification
- **Operations-research solvers:** useful for constrained assignment, routing, scheduling, and exact small-instance oracles where model assumptions match the case.

These pages establish that the products expose the listed features; they do not establish comparative measured performance on our cases. Configure each baseline honestly, name the edition/version/constraints/costs, and compare on identical inputs. If equivalent access or data is unavailable, use a qualitative capability map rather than invented scores.

The intended gap is **pre-scheduling diagnosis plus an evidence-bound bridge to verified work and fair paid-work pathways**: identify whether the blocker is skills, money, parts, authorization, demand data, dispatch, or job quality, then pass only admissible tasks to an existing scheduler. The project should integrate with or complement mature dispatch engines before attempting to replace them. A single blended leaderboard across unlike systems would be misleading.

A fair comparison requires the same frozen tasks, evidence quality, resource limits, hard gates and decision horizon; disclosed product/algorithm/version/configuration and cost; identical scoring and blind cases; predictive quality plus admissibility/worker/service outcomes; explicit handling of missing/stale data; and independent reproduction.

The differentiator to test is not “our model is smarter.” It is whether the integrated stack reduces false shortage diagnoses and unlocks more safely verified work than a vacancy heuristic, taxonomy matcher, transparent rules and a real incumbent workflow. If commercial products cannot be tested on identical inputs, report a qualitative capability map, not a numeric SOTA claim.

## 9. Field-study gate

Do not move from synthetic results to local operational claims until a legitimate service owner confirms data access, scope, permissions, baseline definitions and independent outcome verification. Pre-register a phased or randomized comparison where feasible. Include worker representatives and affected communities; measure worker earnings and conditions, not only employer costs/output. Protect personal data and provide correction/appeal routes.

Candidate interventions must be funded and authorized. Training counts as successful employment only when there is a real payer, fair compensation, evidenced paid work, verified scope/quality and retention follow-up. Planned placements, issued credentials or task recommendations alone do not count as jobs created.

## 10. Claim ceiling

This is a software-contract and synthetic benchmark. It does not establish a real regional skills shortage, predict individual employability, certify practical competence, authorize regulated work, prove causal impact or claim SOTA. The first outcome is structural correctness and honest uncertainty. Real impact remains unmeasured until evaluated with an authorized partner and independent evidence.
