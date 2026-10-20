# SA-LIVING-SYSTEMS-BENCHMARK-001
## Outcome-verified, equity-constrained service improvement across the four-project stack

**Status:** Research contract proposal; documentation-only. Numeric impact targets are hypotheses until a local baseline, operational owner, sample design and evaluation protocol are frozen.  
**Initial geography:** Johannesburg / Gauteng as a candidate proving ground; the contracts are designed to be portable across South Africa.  
**Initial vertical:** Essential-service restoration, initially water/wastewater with power, crew, material and transport dependencies.  
**Authority boundary:** Decision support only. This document does not claim a municipal or utility partnership, authorize provider access, or enable automated operational control.

## 1. Purpose

Evaluate whether Symthaea, Mycelix, Sol-Atlas and Symtropy together improve the real delivery of essential services compared with existing practice and credible simpler/domain-specific baselines.

The target is not to build a single national score, an all-knowing AI, or a replacement municipal control plane. The target is a repeatable cycle:

1. Qualify what is known, unknown, stale, disputed or synthetic.
2. Map a service problem to the physical, operational and community context that matters.
3. Compare feasible interventions under identical constraints and scenarios.
4. Record who is authorized to decide and deliver the work.
5. Verify restoration or another declared outcome independently of task closure.
6. Measure cost, reliability, distribution of benefits, worker outcomes and unintended harms.
7. Update only the model claims supported by the new evidence.

A map feature is not a functioning asset; a work order is not a completed repair; a closed ticket is not verified service restoration; a simulation is not an impact evaluation; and a large national average cannot compensate for an essential-service floor being violated in one community.

## 2. Why this benchmark complements existing work

Compose the existing domain work rather than adding competing engines or ledgers:

- Sol-Atlas paired water/power resilience and protected service floors: [issue #147](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/147), with paired intervention comparisons and shared-baseline batching in [issue #150](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/150).
- Sol-Atlas hydraulic / stormwater qualification: [issue #172](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/172) and the synthetic SWMM reference path tracked by [PR #173](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/173).
- Sol-Atlas spatial-economic resilience work: [PR #116](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/116).
- Sol-Atlas compute / site resilience is an adjacent resource-allocation application, not a second allocator: [issue #202](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/202).
- Johannesburg's civic-service reliability and paid-work portfolio: [Mycelix issue #5015](https://github.com/Luminous-Dynamics/mycelix/issues/5015).
- Existing food, access, site-selection and industrial resource work should be added as domain adapters only when their evidence and validation gates are met.

The first task is integration semantics and evaluation. Do not create a new universal ticket core, GIS engine, hydraulic solver, scheduling engine, reputation authority or municipal database merely to make the diagram look unified.

## 3. Paradigm shifts

### 3.1 From asset availability to delivered service

Measure useful service received by people, within declared service limits and time windows. A pump marked available is not proof that a household received water. A mapped clinic is not proof of access, medicine availability, staffing or an appointment. Keep asset status, modeled capacity, measured throughput and user-facing service outcome separate.

### 3.2 From single-scenario optimization to paired robust decisions

Compare baseline and candidate interventions on the same frozen scenario universe: identical time steps, service cells, failure identities, intervention opportunities, community mappings and protected floors. Include correlated events—such as hot weather plus grid constraints plus pump failure—without inventing scenario probabilities. Aggregate stress cases as declared cases unless occurrence probabilities have been empirically calibrated.

### 3.3 From average utility to hard floors plus transparent trade-offs

Essential-service, legal, safety, privacy and rights constraints are non-compensable. A lower cost or better regional mean cannot make a protected-floor regression a success. After hard constraints are applied, report a Pareto frontier for cost, recovery, resource use, emissions, coverage and work opportunities. Do not hide trade-offs inside one opaque weighted score.

### 3.4 From prediction to evidence-to-action-to-verification

The whole loop is evaluated, including data coverage, responsible handoff, the action actually taken, independent outcome confirmation, repeated failures and maintenance burden. Optimize the time and cost of a *valid, independently verified resolution*, not the number of alerts, model calls, generated plans or tickets marked complete.

### 3.5 From central intelligence to contestable civic infrastructure

The tools should make evidence and decisions easier to inspect and challenge—not centralize arbitrary power. Keep source identity, uncertainty, dissenting claims, correction paths, audit history, human responsibility and appeal routes visible. Design for low-bandwidth/manual operation and accessible multilingual explanations. Use the least sensitive data sufficient for the task.

### 3.6 From speculative jobs to funded pathways

A workforce benefit counts only when a real customer, lawful procurement/funding route, paid task, worker protections and verification mechanism exist. Track actual earnings, payment delays, safe working conditions, training/credentials and 6-/12-month retention. Temporary paid work is not automatically a durable job, and unpaid labour or speculative future savings must not be used to finance promised employment.

## 4. Division of responsibility

| Component | Proposed responsibility | Must not claim |
|---|---|---|
| **Sol-Atlas** | Spatial evidence catalogue, source/coverage context, geography/time alignment, scenario and service-cell manifests, comparison envelopes | That a mapped object is current, operational, legally permitted or adequately capable without matching evidence |
| **Symthaea** | Evidence-aware reasoning, hypothesis generation, uncertainty, contradictory evidence, abstention, explanation and falsification prompts | That an LLM/model judgement is ground truth, a causal effect, a certified decision, or proof of consciousness |
| **Symtropy** | Physical/process simulation, resource and capacity constraints, scenario execution, sensitivity and recovery experiments | That a synthetic model is field-calibrated or that a safe simulation grants actuation authority |
| **Mycelix** | Consent/access-aware coordination, commitments, contribution/work evidence, receipts, correction/dispute routes and appropriate provenance | That an internal receipt is external-source truth, a reputation score proves guilt/competence, or ticket closure proves service restoration |
| **Service owner / community / independent evaluator** | Authoritative operational decisions, permissions, work execution, acceptance, field truth and outcome review | That a software output replaces statutory duties, professional judgement or community recourse |

These are target boundaries. Existing component maturity and real integrations must be independently checked before being described as implemented.

## 5. Benchmark ladder and progression gates

Do not skip a layer. Passing a lower layer is necessary but not sufficient for the next.

### L0 — Contract correctness (offline, deterministic)

Freeze JSON/schema versions, units, semantics and canonical ordering. Build small hand-calculable cases, including service floors, capacity limits, resource conservation, missing observations, conflicting source claims and failed interventions.

**Acceptance:**
- 100% of frozen known-answer cases match the independent oracle.
- 100% of seeded critical negative controls reject or return the predeclared indeterminate/failure state.
- Zero missing, stale, unsupported-unit, out-of-domain or unverified input may silently become zero demand, zero risk, unlimited capacity, a safe site, an allowed action or a passing service floor.
- Zero duplicate assignments, missing scenario results or double-counted resources in the contract corpus.
- Identical canonical input bytes, model/build identity and configuration produce a deterministic canonical receipt where determinism is promised.
- Every output reports its denominator, evidence coverage, exclusions, failures and unknowns.

A structurally valid receipt proves only that the declared contract was followed—not that upstream data are authentic or true.

### L1 — Domain-model qualification

For each domain adapter, compare against an accepted reference method or solver on a frozen, task-matched corpus. For water, distinguish hydraulic mass/continuity and solver-adapter checks from empirical leak-localization accuracy. For routing, distinguish estimated route time from observed trips. For energy, distinguish nameplate capacity, modeled deliverable capacity and metered service. For site selection, distinguish a screened candidate from a permitted/engineer-approved site.

**Acceptance:**
- Declare exact model, reference implementation/version, input corpus, units, scenario range and known limitations.
- Use hand calculations or an independent reference for small cases and published/field measurements only where their target and conditions match.
- Preserve absolute error, signed bias, tail error, coverage and no-result rate; never report accuracy only on the easiest covered subset.
- Test spatial, temporal, topology, weather/load and sensor-noise shifts where relevant.
- Do not advance a model to operational decision support on synthetic fixtures alone.

### L2 — Decision quality against credible baselines

Every candidate is evaluated against the same input, service floors, scenario universe, time grid, objective definitions and compute budget.

Required baselines:
- **B0:** documented incumbent/current workflow where trustworthy logs and permissions exist. If it cannot be reconstructed, say so; do not fabricate it.
- **B1:** a transparent simple heuristic appropriate to the task (for example FIFO, fixed schedule, nearest feasible crew or threshold priority).
- **B2:** a validated domain-specific solver/tool, with its exact version and configuration, where one exists.
- **B3:** the integrated candidate pipeline.
- **B4:** exact enumeration/hand-calculated oracle on tiny cases to bound expected correctness and optimization gap.

Report paired deltas per case and by essential service/community, not only a pooled mean. Freeze the test scenario manifest before tuning. Hidden holdout cases must not be used to adjust thresholds and then scored as if untouched.

**Acceptance:** the candidate must first respect all hard constraints. It is a robust improvement only if the predeclared comparison rule is satisfied across the declared suite and no protected service-floor regression is possible under the computed evidence bounds. Inconclusive evidence remains inconclusive.

### L3 — Cross-domain integration and stress resilience

Demonstrate that the stack can coordinate a coupled system without silently breaking domain semantics—for example, water delivery under a pump-power constraint, material/crew shortages, a road disruption and a heat event.

Required adversarial scenarios:
- correlated service and infrastructure failures;
- stale, missing, delayed and conflicting observations;
- sensor outage or bad calibration;
- supplier/crew/resource shortfall;
- intervention failure and rollback;
- source revision after a decision was prepared;
- a candidate that improves aggregate results but degrades one community's floor;
- adversarial or erroneous completion evidence;
- duplicate reports and retries;
- a model that is confidently wrong outside its validated envelope.

Compare an integrated approach to both the best domain-only baseline and a simple composed baseline. Integration complexity is justified only if it improves outcomes, coverage, interpretability, robustness or total cost enough to offset its added operating burden.

### L4 — Operational impact and equity

Operational trials require an identified owner, lawful data access, human fallback, privacy/safety review, pre-registered endpoints and independent verification. Prefer randomized or stepped-wedge rollout when ethical, operational and sample-size constraints permit. Otherwise use a transparent matched comparison or difference-in-differences design where assumptions can be defended. A before/after change without a credible comparison is descriptive, not causal evidence.

**Candidate primary measures:**
- time from valid report to triage, dispatch, verified restoration and stable follow-up;
- service availability / delivered quantity and unmet essential demand;
- repeat-fault rate, backlog age and independently verified resolutions;
- fully accounted cost per verified resolution, including staff, travel, parts, outages and support;
- performance and service-floor compliance by relevant area / population group;
- evidence coverage, stale-data exposure, abstention and correction/dispute outcomes;
- for paid-work pathways: actual earnings, payment latency, safety incidents, credentials, repeat paid work and 6-/12-month retention;
- adverse effects: displacement, privacy exposure, unfair allocation, maintenance burden and unplanned environmental cost.

### L5 — Replication and durable capability

Only promote a capability to a reusable regional/national offering after an independent team can reproduce the benchmark from pinned inputs, understand the limitations, operate the manual fallback and adapt the jurisdiction/source pack without forking core semantics. A second geography is a new validation cohort, not merely another map tile.

## 6. Provisional numeric targets (hypotheses, not SOTA claims)

These targets should guide the first experimental plan. The local owner and evaluator must freeze the final primary endpoint, minimum meaningful effect, sample size and non-inferiority margins **before** outcome data are inspected.

| Dimension | Proposed gate |
|---|---|
| Known-answer contracts | 100% exact agreement on frozen deterministic oracle cases |
| Critical negative controls | 0 false passes across the frozen, versioned safety/unknown-data corpus |
| Resource and service accounting | 0 unexplained balance violations, double allocations or omitted required scenario results |
| Provenance | 100% of decision-critical claims point to a source/version or are explicitly labeled synthetic/inferred/unknown |
| Hard service floors | 0 accepted “improvements” with a possible protected-floor regression |
| Coverage | Report covered/unknown/no-result rates; no coverage shrinkage may be hidden by better scores on a smaller subset |
| Operational impact hypothesis | Test whether the candidate can reduce median time to independently verified resolution by at least 20% versus a credible control, without material worsening of P90 resolution time or any protected service floor |
| Unit-economics hypothesis | Test whether fully accounted cost per independently verified resolution improves by at least 10%, or whether a declared alternative (such as substantially better reliability at an accepted cost premium) is justified in advance |
| Distributional effect | Predeclare material-harm margins per service/community; show intervals and every subgroup for which the data are adequate. Do not rely on absence of statistical significance as proof of no harm |
| Runtime | Use a pinned reference runner; report p50/p95/max, peak memory, input sizes and output sizes against B0–B4 where applicable. Reuse existing task-specific budgets (including issue #150); do not treat them as universal performance standards |
| Replication | A second locality can reproduce the same contract and test process with provenance-tracked source/jurisdiction adapters and no hidden hand edits |

The 20% / 10% figures are proposed decision thresholds to evaluate, not published industry norms, expected outcomes or claims about present software. If they are operationally unrealistic, underpowered or misaligned with the service owner, revise them *before* the trial and document why. If no credible incumbent or ground truth can be obtained, no “better than industry” claim is allowed.

## 7. Industry and SOTA comparison protocol

Use industry standards as comparators for the exact task they cover, not as a single composite leaderboard:

- **Geospatial/digital twin interoperability:** assess open and standard-based schemas/APIs, coordinate reference systems, temporal validity, identifiers, provenance and vendor-independent export. Open Local Digital Twin research emphasizes standards-based components and interoperability; the benchmark must measure actual lossless exchange and reproducibility rather than standard names alone.
- **Well-being:** use multidimensional outcome categories and population gaps aligned with the OECD framework; do not replace these with a proprietary country rank.
- **AI governance:** map risk controls to the NIST AI RMF functions (Govern, Map, Measure, Manage); produce specific artifacts and tests rather than claiming compliance by checklist.
- **Water / hydraulic and leakage:** use a validated hydraulic solver where appropriate; score detection, localization, calibration, coverage and operational restoration as separate tasks. A 2026 study reports high leak-detection accuracy on a large simulated Net3 benchmark but identifies localization as harder; this highlights why task-specific, shift-aware evaluation is required. It is not a universal real-world SOTA figure and is not directly comparable to a Johannesburg field deployment.
- **Civic operations / workforce:** compare to the documented incumbent process, queue/dispatch heuristics, existing maintenance or work-order system and independently observed outcome. Do not compare against a fictional or deliberately weak competitor.

Every published claim of outperformance must provide:
1. exact commit/model/build and configuration;
2. dataset/scenario manifest and content digests;
3. licenses/access basis and held-out split policy;
4. baseline version and matched workload/constraints;
5. metric definitions, denominators, coverage and confidence intervals where statistically appropriate;
6. run receipt, environment, hardware and failures;
7. the strongest known limitations and any contrary result.

Define “beats SOTA” narrowly: on a named task and frozen evaluation, the candidate improves a predeclared metric or Pareto position over a strong, matched reference without violating correctness, coverage, safety or equity gates. No universal outperformance claim follows from passing this benchmark.

## 8. Shared evidence and comparison envelope

The implementation should reuse existing DTOs and contracts where possible. At minimum, a benchmark run should bind:

- schema and benchmark version;
- task / geography / time scope and decision question;
- immutable scenario-set ID and digest;
- data source IDs, versions/digests, rights/terms review, source timestamps, spatial/temporal coverage, units and CRS where relevant;
- evidence class: observed, reported, modeled, inferred or synthetic;
- explicit unknown/conflict/stale states and admissible-use limits;
- candidate and baseline IDs, versions, parameters and relevant hardware/solver budget;
- protected service floors and their source/authority;
- per-case output, resource balances, service outcomes, failure/abstention reason and reference IDs;
- evaluator version, metric definitions, denominators and deterministic ordering;
- outcome verification method and independent reviewer, when applicable.

Do not fabricate upstream authenticity from a locally computed digest. A digest proves equality to supplied bytes, not who published those bytes or whether their contents are true. Separate data custody, source authentication, model correctness and outcome verification.

## 9. Candidate first experimental slice

**Do not wait on the full national data picture or a large infrastructure integration.** Start offline with a compact paired service-restoration scenario using only synthetic, explicitly labeled inputs:

1. Two service areas with different declared essential floors and demand profiles.
2. A water delivery path constrained by a pump/power dependency.
3. One nominal case and a predeclared set of grid, pump, crew, road and sensor failure cases.
4. A fixed/FIFO or threshold baseline and one transparent constraint-aware candidate.
5. A tiny hand-calculated oracle, including a case where aggregate performance improves but one protected floor is breached.
6. Exact conservation/accounting, unknown-input, duplicate-work, retry and stale-source negative controls.
7. A machine-readable manifest and deterministic per-case receipts that compare candidate/baseline on precisely the same cases.

Keep the model intentionally small enough that all results can be independently computed by hand. Reuse the existing paired water/power service-floor path; do not write a second hydraulic solver or new scheduler until a measured gap justifies it.

**Initial gate:** all oracle and negative-control tests pass, all results can be independently reconciled, and the benchmark explicitly abstains where inputs cannot support a conclusion. This qualifies only the synthetic contract. It does not qualify a real water network, prove an intervention works, or authorize operational recommendations.

After that gate, add larger generated stress cases; compare against the existing domain solver and batch comparator; establish reference-machine performance; acquire source data only with verified access/rights; and then seek an explicitly authorized field evaluator. No data acquisition, deployment or public release of sensitive infrastructure information is implied here.

## 10. Stop / revise rules

Pause rather than promote when:
- a hard service floor can be violated but is hidden by average/weighted scoring;
- a source is stale, disputed, unauthenticated or outside its useful spatial/temporal scale;
- the candidate wins only after dropping unknown/no-result cases;
- the model or baseline uses different scenario sets or resource assumptions;
- there is evidence of leakage between tuning and held-out evaluation;
- staffing, parts, permissions, maintenance costs or an actual payer are missing from the operational claim;
- outcomes cannot be independently verified;
- the integration increases privacy, safety or inequity risk without an approved mitigation;
- a queued/missing workflow is treated as a passing exact-head verification.

## 11. Research anchors

- OECD, *Guidelines on Measuring Subjective Well-being (2025 update)* and Well-being Data Monitor: https://www.oecd.org/content/dam/oecd/en/publications/reports/2025/10/oecd-guidelines-on-measuring-subjective-well-being-2025-update_b957f42e/9203632a-en.pdf and https://www.oecd.org/en/data/tools/well-being-data-monitor/current-well-being.html
- NIST AI Risk Management Framework / Playbook: https://www.nist.gov/itl/ai-risk-management-framework/nist-ai-rmf-playbook
- Raes et al., *Public Development: Towards Open Standards and Components for Local Digital Twins* (2025): https://link.springer.com/chapter/10.1007/978-3-031-81451-8_6
- ISPRS, OGC-based energy urban digital twin study (2025): https://isprs-archives.copernicus.org/articles/XLVIII-4-W16-2025/67/2025/isprs-archives-XLVIII-4-W16-2025-67-2025.html
- *Benchmarking Diverse Learning Architectures for Leak Detection and Localization in Water Distribution Systems* (Journal of Water Process Engineering, 2026): https://doi.org/10.1016/j.jwpe.2026.110114
- Statistics South Africa, Quarterly Labour Force Survey, 2026 Q1 landing page for labour-market baseline sourcing: https://www.statssa.gov.za/?PPN=P0211&SCH=74511&page_id=1856
- Auditor-General of South Africa, 2024–25 municipal audit outcomes: https://mfma-2025.agsa.co.za/
- City of Johannesburg public IDP/SDBIP/budget documents, to be version-pinned at pilot intake: https://joburg.org.za/documents_/Pages/Key%20Documents/SDBIP-IDP-and-Budget/SDBIP-IDP-And-Budget.aspx

---

**Claim ceiling:** this is a proposed evaluation contract, not evidence that the integrated stack has already outperformed a baseline or improved any South African service. All impact, runtime and SOTA claims remain unmeasured until a named candidate is evaluated against frozen baselines with reproducible evidence.
