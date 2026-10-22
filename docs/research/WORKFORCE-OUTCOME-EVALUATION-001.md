# SA-WORKFORCE-OUTCOME-001 — Diagnosis-to-outcome evaluation contract v0.1

**Tracking issue:** https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/276  
**Composes with:** [diagnosis benchmark #268](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/268), [evidence profile #272](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/272), [Symthaea HUMAN-TOOL-001 #7983](https://github.com/Luminous-Dynamics/symthaea/issues/7983), and [Mycelix EMB-SKILL-001 #3031](https://github.com/Luminous-Dynamics/mycelix/issues/3031).  
**Status:** evaluation contract and entirely synthetic fixture; no field-effectiveness or job-creation result.

## The paradigm shift

Most benchmarks stop at the prediction. This one tests the entire causal chain:

**evidence available at decision time → diagnosis → feasible intervention → human authorization → action completion → independently verified result → paid-work/job-quality follow-up.**

These are separate gates and metrics. Accurate labels do not prove a recommendation was feasible. Recommendation adoption does not prove execution. A closed work order does not prove service restoration. A placement does not prove the worker was paid, the conditions were viable, or the placement persisted. Keep each denominator visible.

The system is decision support, not an employer, credential authority, scheduler, payer, or safety approver. It must abstain when evidence cannot distinguish explanations. It must not score individual employability, rank workers for adverse decisions, infer sensitive traits, or turn source missingness into a negative person-level inference.

## Why this evaluation layer matters

South Africa's national unemployment rate and local occupational demand describe different populations and concepts. The QLFS Q2 2026 release reports an official unemployment rate of 33.6%; DHET's 2024 National List contains 350 occupations and explicitly is not a scarce-skills list; the World Bank describes simultaneous unemployment and unfilled digital vacancies with bottlenecks across foundational skills, education alignment and workplace learning. None of these contextual facts validates this benchmark or proves a particular local shortage.

- Stats SA, QLFS Q2 2026: https://www.statssa.gov.za/?PPN=P0211&SCH=74512&page_id=1856
- DHET, 2024 National List of Occupations in High Demand: https://www.gov.za/sites/default/files/gcis_document/202404/50510gen2414.pdf
- World Bank, *From Demand to Delivery* (14 Aug 2026): https://www.worldbank.org/en/news/feature/2026/08/14/from-demand-to-delivery-strengthening-south-africas-digital-skills
- NIST, *TEVV-Athlon Framework for Evaluating AI Systems* (2026 draft): https://www.nist.gov/artificial-intelligence/ai-research/tevv-athlon-framework-evaluating-ai-systems
- OECD, *Digital Government Outlook 2026*, impact measurement and user feedback: https://www.oecd.org/en/publications/digital-government-outlook_0496b2bc-en/full-report/adopting-and-governing-ai-in-government_7ef312a9.html
- OECD, Jobs Strategy evaluation literature on deadweight, displacement and substitution effects in labour-market programmes: https://www.oecd.org/content/dam/oecd/en/about/programmes/jobs-strategy/JobsStudy1941679.pdf
- UNU-WIDER, *The effect of wage subsidies on job retention: Evidence from South Africa during the COVID-19 pandemic* (quasi-experimental difference-in-differences evaluation): https://www.wider.unu.edu/publication/effect-wage-subsidies-job-retention

Commercial field-service products such as Dynamics 365 Resource Scheduling Optimization, Salesforce Field Service and ServiceNow Dynamic Scheduling provide relevant operational comparators for skills/availability/travel/priority-constrained scheduling. They are not direct evidence of workforce-shortage diagnosis or causal employment impact. Compare like for like on shared tasks and budgets; do not claim to outperform a vendor based on a different task or an unverified fixture.
- https://learn.microsoft.com/en-us/dynamics365/field-service/rso-overview
- https://help.salesforce.com/s/articleView?id=sfdo.fs_optimization.htm&type=5
- https://www.servicenow.com/docs/bundle/yokohama-customer-service-management/page/product/field-service-management/concept/c_DynamicScheduling.html

## Evaluation ladder

1. **Contract-only / synthetic.** Verify schemas, accounting identities, explicit abstention, precondition and safety gates, permutation invariance, and negative controls. Synthetic counts prove only the test contract behaves as designed.
2. **Authorized retrospective replay.** Use time-stamped snapshots and the evidence actually available at each decision time. Freeze policy versions before scoring a later time window. Keep near-duplicate work orders, vacancy copies and linked cases in the same partition.
3. **Prospective pilot.** Preregister policy, primary outcome, denominator, exclusions, data retention, sample-size rationale, stopping rules, uncertainty estimates and harm review. Use randomized assignment where feasible and ethical; otherwise select a credible stepped-wedge, matched, interrupted-time-series or difference-in-differences design with its assumptions and diagnostics written down.
4. **Independent replication.** Freeze the metric contract and repeat at a separate site/cohort or later time period before claiming generalization.

A before/after increase alone does not establish causality. Record seasonality, demand changes, staffing/budget changes, contamination, selection, attrition and concurrent policy changes. If credible attribution is unavailable, report descriptive association, not causal effect.

## Baselines and fair comparisons

- **B0 — incumbent process:** documented, measured existing process; omit if no comparable records exist.
- **B1 — proxy baseline:** backlog or public-ad count, a deliberately weak negative-control comparator.
- **B2 — transparent evidence rules:** explicit thresholds, source-quality checks and abstention reasons.
- **B3 — blinded independent expert panel:** multiple-cause adjudication on frozen evidence, before candidate output is shown.
- **Candidate:** source-aware multi-cause diagnosis plus precondition-checked intervention bundles and explicit unknown/abstain state.
- **Operational comparator:** existing scheduler/field-service product only for tasks it actually performs (assignment/routing/dispatch) under matched jobs, constraints, resource budgets and evaluation windows.

Freeze case definitions and input snapshots. Keep candidate outputs hidden from adjudicators until their labels and disagreements are frozen. Compare candidates at matched coverage and report both quality and abstention; never force a system to guess merely to raise coverage.

## Metrics — never compress into one score

**Diagnostic quality:** multi-label precision/recall/F1; exact cause-set match; false-shortage and false-surplus rate; confusion by cause; abstention rate, selective risk and coverage-risk curve; calibration only if probabilities are emitted and sample/population assumptions permit it.

**Evidence efficiency:** time/staff effort needed to resolve an indeterminate case; source freshness and provenance completeness; fraction of conclusions supported by independent, comparable evidence; rate of stale/contradictory evidence and taxonomy-crosswalk ambiguity.

**Action quality:** proportion of proposed actions whose preconditions hold at decision time; unauthorized/unsafe proposal rate; budget/resource violations; human approval, refusal and override; execution completion; time to completion. Report infeasible or rejected actions rather than excluding them.

**Verified service outcomes:** proportion of eligible tasks reaching a defined functional and, where relevant, safety-verified restoration; median/P90 request-to-verified-restoration time; rework/repeat-failure rate; non-completion, referral and missing follow-up; fully accounted cost per verified resolution. Failed and unobserved cases remain in the denominator under a preregistered missing-data policy.

**Paid-work outcomes:** only count a paid-work event when the evidence contract verifies the payer/contractual route, paid hours or earnings, payment status, period and worker consent/permission. Report time-to-payment, acceptance/decline, hours, job terms/conditions and 3/6/12-month retention when legitimately observable. Training completion and credential issuance are intermediate outcomes, not job outcomes.

**Additionality, displacement and job quality:** gross paid-work events are not net additional employment. A placement can represent deadweight (the hire would have happened anyway), substitution (one worker replaces another), displacement (activity is diverted from another employer/service), or a temporary subsidized job that ends when the support ends. A causal job-creation claim needs an explicit counterfactual and must report gross work events separately from the estimated incremental paid hours/jobs, uncertainty interval, displacement, substitution, deadweight and relevant positive/negative spillovers. Unknown effects stay unknown; zero observed displacement is not proof that displacement is absent. A synthetic receipt must leave net additional hours unestimated and mark these effects not assessed. The additionality record also names the policy whose gross paid-work events it counts; those events must reconcile to that policy's verified paid-work count.

**Harm and service guardrails:** unsafe recommendations, essential-service floor regressions, false exclusion, accessibility failures, worker-reported conditions, appeal/correction path, privacy incidents and unreasonable work intensification. Subgroup analysis requires lawful and justified data, sufficient sample sizes, privacy protection and predeclared slices.

**Cost and operational load:** staff minutes, elapsed time, data-acquisition expense, compute/tool expense and incremental implementation burden. Do not call a model better if its accuracy gain requires hidden labour, longer decision time or unaffordable data collection.

## Leakage and adversarial controls

The first frozen corpus should include at least these cases: (1) raw advert with no proven payer; (2) duplicate/stale advert; (3) a high-demand occupation marker with no local funded demand; (4) funded job but no task-ready available worker; (5) capable worker blocked by missing tools/spares; (6) unknown rather than absent authorization; (7) worker availability outside the shift/geography; (8) nonviable pay/conditions and offer decline; (9) training without a supervisor/workplace/funded role; (10) action completed but no verified restoration; (11) work labelled completed but no payment evidence; (12) contradictory/stale data that should trigger abstention.

Additional rules:
- Use as-of timestamps; future work outcomes and later source revisions must not be model inputs.
- Group related and near-duplicate records in the same split. Prefer temporal and site/employer/cohort holdouts where data permits.
- Preserve unknown, not-applicable, not-collected and observed-zero as different states.
- Publish the denominator, missingness, exclusions and all preregistered slices.
- Include an independent adjudication process, conflict resolution and disagreement rates.
- Count outages, unsuccessful action and harm reports; do not drop them as “incomplete successes.”
- Any metric, exclusion or outcome-definition change after unblinding gets a new protocol version and is identified as exploratory.

## Acceptance gates for the contract

- 100% of fixture records parse against the versioned schema; canonical report is deterministic under input-record permutation.
- No synthetic result is represented as observed evidence, calibrated probability, employment impact or causal effect.
- No confirmed-shortage conclusion may arise solely from a high-demand marker, raw advert count or service backlog.
- Unknown funding/authorization never becomes confirmed absence/failure; mandatory safety/authorization preconditions fail closed.
- No event is counted as paid work without declared payment evidence and valid observation period.
- Gross paid-work events are never called net additional jobs/hours without a declared counterfactual; causal additionality requires preregistration, uncertainty and explicit displacement/substitution/deadweight status.
- Every report includes data/metric/policy/code digests, observation window, denominator, missingness, decision coverage and claim ceiling.
- No superiority claim unless a preregistered primary outcome, matched baseline, uncertainty, cost/resource measures and safety guardrails jointly support it.
- No generalization claim from one synthetic corpus or one site.

## Implemented evaluation slice

The Rust crate `tools/workforce-outcome-verifier/` now validates the receipt schema and cross-field accounting contract, checks unique policy IDs, reconciles gross paid-work events to the named policy's verified paid-work count, rejects impossible aggregate counts, and enforces the synthetic/additionality claim ceiling. It emits a deterministic machine-readable receipt with fixture/schema SHA-256 digests. The dedicated workflow makes formatting, tests and receipt generation hard gates.

The synthetic fixture deliberately leaves net additional paid hours unestimated. The evaluator therefore cannot convert four gross verified paid-work events into four additional jobs. An empirical additionality claim requires a non-synthetic evidence class, a preregistered credible comparison, explicit counterfactual basis, estimate/uncertainty interval and assessed displacement, substitution, deadweight and spillover states.

Next steps remain a sequestered, independently adjudicated case manifest and an authorized retrospective/prospective design. Compose Mycelix's scoped practical skill evidence and Symthaea's repair-workbench outcome protocol rather than duplicating registries, work-order state, payment records or authority.

## Claim ceiling

This benchmark contract and synthetic fixture are not evidence that any system creates jobs, reduces unemployment, resolves a real shortage, improves productivity, or beats state of the art. Those claims require authorized observations, matched baselines and a credible predeclared evaluation.