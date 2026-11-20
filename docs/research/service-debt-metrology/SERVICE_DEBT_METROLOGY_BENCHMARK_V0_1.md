# Service-debt metrology benchmark and reference oracle v0.1

**Tracking issue:** [SERVICE-DEBT-METROLOGY-001 #965](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/965)  
**Composition:** [service-opportunity resilience #959](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/959)  
**Maturity:** hand-authored synthetic golden corpus and executable evaluator; real data and owner conformance not established  
**Claim ceiling:** no live resident demand, delivered service, service capacity, unexpressed need, welfare, equity, or policy-effect claim

## 1. Objective

A binary service-opportunity result cannot answer how much capacity is missing, how much accepted service was not delivered, or how much need may remain unexpressed.

This benchmark defines four separate evidence ledgers. It is intentionally not a universal "service debt score", demand forecast, allocation policy, queueing model, capacity solver, booking system, welfare metric, or optimizer.

The measurement unit is:

**service × functional unit × resident cohort × scope × time basis × observation interval**

A value without that identity is not safely comparable or additive.

## 2. Paradigm shift: capability accounting, not facility counting

The target evidence chain is:

\`resident-relevant need \rightarrow reachable route \rightarrow complete task window
\rightarrow qualified deliverable capacity \rightarrow authorized allocation
\rightarrow accepted delivery \rightarrow unmet / unresolved residual\`

Each transition has an owner and evidentiary burden. A downstream state cannot be inferred from an upstream proxy:

- a facility existing does not prove it is operating;
- a route existing does not prove the resident can complete the task;
- nominal capacity does not prove that capacity can be delivered in the relevant interval;
- an offer or booking does not prove a completed delivery;
- zero recorded requests does not prove zero need;
- UNKNOWN, STALE, PARTIAL, or NOT_COMPARABLE evidence is not a measured zero.

This is the central design choice: **preserve the causal and epistemic boundaries between need, demand, capacity, allocation, and delivery.**

## 3. Four independent quantities

### 3.1 Capacity feasibility gap

For one fully matched observation interval:

\`capacity\_shortfall = max(0, qualified\_demand - qualified\_deliverable\_capacity)\`

\`capacity\_surplus = max(0, qualified\_deliverable\_capacity - qualified\_demand)\`

Both inputs must be current, complete, and bound to identical:
- service and functional unit;
- quantity unit;
- cohort and population frame;
- period and interval;
- time basis and scope digest.

The quantities must already refer to the same interval and service semantics. This v0.1 benchmark does not invent a rate, prorate a daily figure, interpolate a gap, or turn nameplate capacity into deliverable capacity.

This is a **feasibility gap**, not proof that a resident was denied service. It can be positive while every recorded accepted request is delivered.

### 3.2 Accepted-but-undelivered service

For each unique accepted request:

\`undelivered_i = accepted_units_i - completed_delivery_units_i\`

The aggregate is the sum over an owner-qualified, complete request register for the exact scope. Cancelled and declined records are not accepted debt. Delivery greater than accepted quantity is invalid, not negative debt. Duplicate request identities are rejected rather than counted twice.

A partial or stale register cannot prove zero undelivered service. A reservation, offer, queue entry, or accepted booking is not a completed delivery receipt.

This is an **expressed-and-accepted demand residual**, not total unmet need.

### 3.3 Resident-measured unexpressed need

Unexpressed need is recorded only from independent resident needs evidence, such as a declared survey/assessment method with:
- an identified target cohort and population frame;
- sample size and method;
- an estimated quantity and lower/upper uncertainty bounds;
- the service and functional unit;
- source identity, revision, time basis, and observation period.

The evaluator preserves the externally produced estimate and its interval. It does not infer this quantity from non-use, no bookings, low utilization, empty queues, route traces, or a missing request stream.

"Not collected" is a real result. It is not a numeric zero.

### 3.4 Unresolved evidence exposure

The evaluator reports whether a measurement dimension remains unresolved and the duration of the common observation interval to which the unresolved state applies. This duration is not converted into service units or combined with another metric.

Reasons are explicit and typed:
- incomplete, stale, or unknown capacity/demand evidence;
- incomplete, stale, or unknown accepted-request/delivery evidence;
- resident need not assessed, uncertain, or scope-incompatible.

Unknowns are not imputed, silently interpolated, or assigned zero. A future multi-interval implementation must additionally account for explicit interval gaps and preserve owner-native temporal semantics; this v0.1 fixture uses one bounded window per case.

## 4. Normative interpretation and state rules

| State | Meaning | Permitted interpretation |
|---|---|---|
| KNOWN | Both required source records are current, complete, and exactly scope-matched | A unit-consistent difference may be computed |
| UNKNOWN | Evidence is stale, absent, incomplete, or explicitly unresolved | No numeric zero or point deficit is asserted |
| NOT_COMPARABLE | Population, service, functional unit, unit, period, time basis, or scope differs | No subtraction or cross-source aggregation |
| NOT_COLLECTED | A resident-needs assessment was not performed | No unexpressed-need value is asserted |
| MEASURED | An owner-qualified resident assessment supplies an estimate and uncertainty interval | Preserve the estimate and bounds; do not synthesize a point from missing requests |

The evaluator validates:
- positive, explicit observation interval;
- exact common scope for arithmetic;
- freshness and completeness before asserting a known result;
- unique case and request identities;
- non-negative quantities;
- delivered quantity no greater than accepted quantity;
- uncertainty interval containing its point estimate;
- no cross-metric scalar or implied welfare ranking.

## 5. Frozen synthetic corpus

The machine-readable reference is:

- \`tools/shared-amenity-spatial-eval/fixtures/SERVICE_DEBT_METROLOGY_FIXTURES_V0.1.json\`
- \`tools/shared-amenity-spatial-eval/fixtures/SERVICE_DEBT_METROLOGY_FIXTURES_V0.1.schema.json\`

It contains **19 hand-authored cases**. The evaluator's expected values are embedded in the fixture and are not generated from the evaluator algorithm.

The cases cover:
1. exact demand/capacity match and fully delivered requests;
2. capacity shortfall with zero accepted-but-undelivered requests;
3. capacity surplus with partially delivered accepted requests;
4. unknown demand;
5. stale capacity;
6. partial demand coverage;
7. capacity unit mismatch;
8. a partially delivered accepted request;
9. multiple partial deliveries;
10. cancelled and declined requests;
11. resident-survey measured unexpressed need;
12. no requests and no resident-needs assessment;
13. capacity shortfall distinct from completed accepted requests;
14. accepted delivery debt despite capacity surplus;
15. known delivery debt while capacity remains unknown;
16. stale resident-needs assessment;
17. request-scope mismatch;
18. incomplete request-register coverage;
19. stale demand evidence.

The Rust evaluator is \`service-debt-metrology\`. Its output includes the fixture SHA-256, every case's scope, separate metric states and quantities, and the count of exact golden matches.

## 6. State-of-the-art comparison: compare equivalent layers

The work should not claim novelty for time-dependent spatial accessibility or capacity-aware facility accessibility:

- **STH-G2SFCA (2024)** already extends accessibility toward service access, incorporates space-time heterogeneity, and considers service capacity and supply–demand relations. The benchmark therefore does not claim to invent temporal service accessibility or capacity-aware accessibility. [Chen et al., *Cities* 153 (2024), 105314](https://doi.org/10.1016/j.cities.2024.105314).
- **System / system-facilitated transit accessibility (October 2026)** combines fine-grained stop access with network travel to destinations. It is a strong transit-side comparator for the journey layer, not a completed service-delivery ledger. [Yadav et al., *Transportation Research Part A* 212 (2026), 105158](https://doi.org/10.1016/j.tra.2026.105158).
- **Adjusted Transit Accessibility Index (May 2026)** normalizes public-transport accessibility against a proximity-independent benchmark. It addresses a different question—isolating transit system performance from land-use/geography—so it belongs in the mobility baseline, not as a direct service-delivery comparator. [*Journal of Transport Geography* 133 (2026), 104616](https://doi.org/10.1016/j.jtrangeo.2026.104616).
- **Capacitated facility location with service-level constraints (April 2026)** optimizes facility location under demand uncertainty with independent service-level controls. It is an operations-research optimization comparator. This evaluator does not compete with or replace its optimizer; it audits realized evidence and residuals. [Zhang & Kalcsics, *European Journal of Operational Research* 330 (2026), 47–71](https://doi.org/10.1016/j.ejor.2025.08.056).
- **Distributionally robust facility location (2024)** models uncertainty in provider capacity and customer demand. That is a design/optimization problem. Sol-Atlas should consume owner-qualified demand/capacity outputs rather than rebuild stochastic optimization. [Cheng et al., *Omega* 122 (2024), 102959](https://doi.org/10.1016/j.omega.2023.102959).
- **Directional latent demand index (June 2026)** uses movement data and statistical models to infer potential unmet transit demand, while explicitly discussing false discoveries and uncertainty. That reinforces the need to label mobility-derived inference separately from resident-declared need; neither is interchangeable with a completed request or direct survey estimate. [Lim et al., *Computational Urban Science* 6 (2026), article 41](https://doi.org/10.1007/s43762-026-00276-3).
- **NEED needs-assessment framework (2025)** distinguishes patient and societal need from supply-driven innovation and proposes structured indicators for needs-driven policy. It supports the measurement boundary between need and service supply. [*Health Policy* 155 (2025), 105263](https://doi.org/10.1016/j.healthpol.2025.105263).
- Public-health needs-assessment guidance distinguishes need, expressed demand, and supply; OECD analysis documents how survey definitions and populations change reported unmet-need results. These distinctions make cross-metric blending especially risky. [Public Health Wales needs assessment guidance](https://phwwhocc.co.uk/wp-content/uploads/2024/04/Needs-Assessment.pdf); [OECD, *Unmet needs for health care* (2020)](https://www.oecd.org/content/dam/oecd/en/publications/reports/2020/01/unmet-needs-for-health-care_1e29e6c7/9c2be79c-en.pdf).
- **NIST AMS 100-85 (October 2026)** recommends risk-proportionate verification, validation, and uncertainty quantification, scaled by model influence and decision consequence. Essential-service floors and capital decisions should receive stronger evidence requirements than low-consequence descriptive views. [Srinivasan, NIST AMS 100-85 (2026)](https://doi.org/10.6028/NIST.AMS.100-85).

### Industry capability map

| Layer | Representative industry/research capability | What Habitat should consume or add |
|---|---|---|
| Routing and travel-time distributions | Conveyal R5, OpenTripPlanner, GTFS-based routing | Consume route outputs, access classes, and departure-window distributions; do not claim a better route engine |
| Potential spatial accessibility | Cumulative opportunity, gravity, 2SFCA, STH-G2SFCA, PTAL | Compare on identical network, population, opportunity, and time inputs |
| Facility/capacity planning | Capacitated facility-location and service-level optimization | Consume demand/capacity/decision records; do not impersonate the optimizer |
| Scheduling, booking, and service operations | Provider calendars, reservations, dispatch and service-management systems | Consume owner-verified accepted requests and completed delivery receipts |
| Needs assessment | Resident surveys, participatory assessment, structured public-health needs frameworks | Preserve direct needs estimates, sampling design, confidence bounds, and cohort scope |
| Evidence-qualified composition | This Habitat increment | Join the ledgers without losing units, provenance, uncertainty, delivery completion, or unknown states |

**What can outperform current practice:** not a new travel-time index or a claim of better optimization. The opportunity is a reproducible, independently checkable audit trail that prevents false zeros and distinguishes projected capacity from actual accepted service delivery and measured resident need.

## 7. Benchmark targets

| Benchmark | v0.1 target | Evidence needed |
|---|---:|---|
| Golden corpus agreement | 19 / 19 exact | Exact-head CI receipt |
| False-zero promotion from UNKNOWN, STALE, or PARTIAL inputs | 0 | Mutation and adversarial tests |
| False comparability under scope/unit/cohort mismatch | 0 | Negative controls |
| Duplicate request double counting | 0 | Duplicate-ID rejection |
| Negative undelivered quantity under over-delivery | 0 | Over-delivery rejection |
| Accepted request conservation | Exact arithmetic | Request-level hand calculations |
| Need inferred from no requests / no utilization | 0 | Negative control with absent survey |
| Measured need without method, sample, or uncertainty bounds | 0 | Schema and evaluator rejection |
| Cross-metric scalar emitted | 0 | Output-schema inspection |
| Field claims | None from synthetic corpus | Independent owner sign-off and resident/operational validation |

For empirical testing, report per-service-unit and per-cohort error, calibration of survey uncertainty where applicable, missingness and stale-evidence rates, duplicate/reconciliation errors, and decision consequences. Do not collapse these to a single score.

## 8. Matched comparison protocol

Use three comparison levels:

- **B0 — incumbent practice:** calculate the accessibility, service-level, and capacity measures already used by the municipality/provider on a frozen dataset.
- **B1 — owner-composed evidence:** retain the incumbent outputs but add complete-task, scope/revision, request-to-delivery, and resident-needs evidence where independently available.
- **B2 — independent audit oracle:** hand-calculated, finite known-answer cases plus adversarial mutations, including the 19 fixtures in this release.

All levels must use the same resident cohort, service, functional unit, period, time basis, network/route source, and source revisions. If the incumbent cannot express an output, label it unsupported; do not manufacture a baseline result. Measure any claimed improvement in false-zero rate, traceability, decision reproducibility, and detection of unmet accepted service. No routing, solver, reliability, causal-effect, or equity outperformance claim is justified by v0.1.

## 9. Distributive justice and capability-floor research boundary

A follow-on question is whether a service opportunity meets a resident-authorized minimum capability floor. This is deliberately separate from the four v0.1 metrology outputs and is tracked in [CAPABILITY-FLOOR-ASSURANCE-001 #969](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/969).

Current work already covers much of the fairness and threshold design space:

- The **MAP framework** (published online 3 June 2026) operationalizes equality, utilitarian, and Rawlsian distributive spatial-justice lenses, and applies them to networks in the Netherlands, Mexico and South Africa. Habitat should not claim a new theory or general fairness index. Its narrower future contribution is exact-scope evidence qualification and non-concealment for service-task and delivery floors. [*Ethically Informed Urban Planning: Measuring Distributive Spatial Justice for Neighborhood Accessibility* (2026)](https://doi.org/10.1080/24694452.2026.2671995).
- A **bi-level multi-objective facility-location framework** (July 2026) already balances accessibility, equity and sustainability and produces Pareto-frontier options. Habitat must consume/compare such optimizer outputs, not replace them with a binary score. [*Balancing accessibility, equity, and sustainability in public facility location-allocation* (2026)](https://doi.org/10.1016/j.apgeog.2026.104027).
- **Perceived Minimum Accessibility** and **Perceived Sufficient Accessibility** thresholds (July 2026) are derived from a Canadian Mobility Survey sample of 15,960, with region-specific thresholds and differentiated population coverage. This shows that thresholds should be service-, geography-, mode-, population- and perception-bound; no Canadian threshold may be transplanted to Gauteng by default. [*Operationalizing accessibility thresholds: Identifying perceived minimum and sufficient accessibility in Canada* (2026)](https://doi.org/10.1016/j.jtrangeo.2026.104758).
- An **equitable facility-location** method using Kolm–Pollak equally-distributed-equivalent measures has been benchmarked against p-median, p-center and p-centdian alternatives across 428 US cities. This is an optimization comparator, not an evidence-ledger competitor. [*A scalable optimization approach for equitable facility location* (2025)](https://doi.org/10.1016/j.trb.2025.103319).
- A September 2026 study developed a Gauteng City Region housing accessibility index using spatial analysis and AHP–MCDA and reported service-access misalignment for subsidized housing locations. This is directly relevant South African context, but it is a potential-accessibility assessment; it does not establish actual task completion, provider delivery or resident-reported unmet need. [*Developing a Decision Support Housing Accessibility Index (DSHAI) in the Gauteng city region, South Africa* (2026)](https://doi.org/10.1016/j.sciaf.2026.e03511).
- A 2026 systematic review of fifteen-minute-city accessibility assessment argues that potential/objective indicators should be supplemented by realized, behavioral or subjective evidence when effective access or equity is the principal concern. This supports keeping access, service delivery and resident experience as separate evidence families. [*Revisiting the 15-min city through accessibility assessment* (2026)](https://www.sciencedirect.com/science/article/pii/S0264275126007481).

**Implication:** do not propose a new universal fairness score. Let residents and accountable authorities declare service-specific floors; preserve competing justice lenses as separately reported policy views; evaluate each cohort against the exact approved floor; show unresolved coverage explicitly; only then expose admissible efficiency/sustainability trade-offs. A city average, Pareto improvement, or optimizer objective may not erase a protected-cohort floor failure.

## 10. Research frontiers after v0.1

1. **Capability-floor accounting.** Compare each essential service and cohort against a declared minimum service floor. Use lexicographic safeguards for essential floors before cost efficiency; avoid a single weighted welfare score.
2. **Common-cause-aware service debt.** Join the service-debt ledger to #959 resilience only after upstream owners qualify dependency maps and fault/recovery evidence.
3. **Distributional service guarantees.** Report the full cohort distribution and worst-served tails; do not allow a citywide mean to hide cohort failure.
4. **Value-of-information prioritization.** Rank missing evidence by expected decision consequence and reversibility, not by data volume.
5. **Intervention evaluation.** Test whether added staff, extended hours, route changes, or modular capacity actually reduce the same unit-scoped shortfall and improve completed delivery. Use preregistered comparisons and appropriate causal designs; a before/after map alone is not causal evidence.
6. **Temporal metrology.** Integrate #966's source precision and conversion rules before joining observations from unlike clock resolutions.

## 11. Field validation gate

Before any real city/resident claim:
- service owner approves the functional unit and what counts as completed delivery;
- resident representatives approve cohort/suitability definitions and the needs-assessment approach;
- the operator supplies revisioned, period-complete capacity and request/delivery evidence;
- route/time evidence is bound to the same interval and cohort;
- privacy, consent, retention, and re-identification risks are independently reviewed;
- a held-out period and a negative-control cohort are declared before evaluation;
- the results are reconciled against a manual sample by an independent reviewer.

This fixture set proves neither data availability nor operational validity. Its job is to make those missing requirements impossible to hide behind an attractive map or aggregate number.
