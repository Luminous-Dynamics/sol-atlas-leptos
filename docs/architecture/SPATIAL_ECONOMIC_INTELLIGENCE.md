# Spatial-Economic Intelligence: architecture RFC

**Status:** Proposed design; not operationally validated  
**Version:** 0.1.0  
**Scope:** Sol-Atlas spatial reasoning and its interfaces to Mycelix coordination and economic execution  
**Implementation status:** initial Rust evidence contract, time-bounded dependency graph, qualitative dependency groups, intervention comparison, common-cause overlap, uncertainty-bounded service capacity, directed network flow, network counterfactuals, sampled recovery metrics, and exhaustive bounded link-failure/common-cause stress; no live data integration

## Executive thesis

Spatial software usually answers *where is it?* and *what is near it?* Business software usually answers *what was booked, ordered, invoiced, or paid?* A useful next-generation system must connect those questions without treating the records as interchangeable.

**Spatial-Economic Intelligence (SEI)** models places, assets, resources, observations, dependencies, commitments, events, costs, and outcomes as a time-aware evidence graph. It lets people ask:

- What do we know about this place during this time window, and where did each claim come from?
- Which essential services or supply routes depend on the place or asset?
- What commitments are at risk, and what evidence supports that assessment?
- Which interventions are feasible under explicit budget, capacity, environmental, rights, and fairness constraints?
- Did the intervention produce the intended outcome relative to a documented baseline?

This RFC is a direction and contract proposal—not a claim that Sol-Atlas already answers these questions reliably.

## Paradigm shifts

### 1. From map layers to a living, time-aware dependency graph

A raster, point, boundary, work order, contract, delivery and payment are not just overlays. They are distinct records linked to stable entities through typed relations and event time. Store event time separately from record time so late-arriving evidence and retrospective corrections are visible. Model dependencies explicitly: a pump serves an irrigation block; a road links a warehouse to a market; a power feeder supplies the pump; a contract commits a named party to supply a defined quantity by a deadline.

The map becomes one view over this model, not the canonical database and not a universal score.

### 2. From data aggregation to evidence-bounded knowledge

Keep each claim attached to its issuer, method, source captures, spatial/temporal scope, uncertainty, access policy and correction history. Represent missing, conflicting and stale evidence explicitly. A SHA-256 digest establishes byte identity, not truth. A signature establishes who signed a statement under a key, not whether the statement is accurate. A transparency receipt establishes registration, not physical delivery or causal impact.

Do not use one opaque confidence number as a substitute for source-specific limitations or independent corroboration.

### 3. From money-first accounting to resource-and-obligation accounting

Financial events are only one part of an economic process. Represent the distinct chain:

**need → proposal → commitment → dispatch → delivery → acceptance → invoice → settlement → measured outcome**

Not every workflow uses every step, and exception paths must remain explicit. None of the states above implies the next one automatically. A bank transfer does not prove that goods arrived; a delivery receipt does not prove quality; a paid invoice does not prove the public benefit happened.

Support at least three reconcilable views:
- **Physical ledger:** quantities, units, lots, capacities, losses and transfers.
- **Obligation ledger:** parties, promises, deadlines, conditions, acceptance and disputes.
- **Financial ledger:** currency, amounts, taxes, accounts, settlement and reversals.

Money must use an explicit currency and decimal-safe arithmetic. A tonne of maize, a kWh, a maintenance hour, a grant, a non-transferable reputation signal and a ZAR invoice are different kinds of value. They may coordinate in one system without pretending they are interchangeable or that one scalar captures social value.

### 4. From descriptive dashboards to auditable interventions

The system should compare a proposed intervention with a named baseline and record assumptions, model versions, resource constraints, uncertainty ranges, affected populations and side effects. Before making causal claims, specify a credible evaluation design and negative controls. Separate observed facts, forecasts, model scenarios, commitments/actions and measured outcomes.

A forecast must never silently become an observation. A simulation result must not be reported as an achieved outcome. The first intervention-comparison API reports qualitative before/after deltas only; it is not causal attribution, cost-benefit optimisation, or proof that an action was executed.

### 5. From centralised supermap to federated sovereignty

Allow different communities, agencies, operators and firms to retain authority over their data. Exchange signed, scoped statements and references under explicit policies; disclose the minimum information required. Public transparency logs must never receive secrets or personal data by default. Provide offline and low-connectivity workflows, revocation/key-rotation procedures, and a human appeal path.

Reputation can inform review but must not become an automatic eligibility, deprivation, credit or resource-allocation decision. There must be no automatic denial of essential services based solely on a model score.

### 6. From one “economy” score to plural, inspectable value models

Measure outcomes in terms relevant to the use case—reliability of water delivery, food loss, service access, time to restore, total cost, emissions, distribution of benefits, and harms—alongside conventional monetary totals. Publish the aggregation choices. Do not hide trade-offs by combining incomparable units into an unexplained composite score.


### 7. From nominal redundancy to explicit shared-cause domains

Two routes do not provide meaningful redundancy if they rely on the same upstream feeder, control plane, network, site, supplier, workforce or hazard. The first common-cause increment represents those shared domains separately from dependency edges. Each domain has an explicit member set, a mechanism label, a time window and evidence references; a scenario says whether the cause is assumed active/inactive/unknown or observed with admissible observation evidence.

The output is **exposure**, not a predicted outage. `Exposed` means the mechanism is active and the entity is listed in that domain; it does not mutate the entity's operational state. Missing cause conditions stay indeterminate. Disputed, stale, mismatched or wrong-kind domain evidence blocks the exposure claim. Expired domains are out of scope—not evidence that a system is safe.

The current overlap analyzer correlates declared shared domains with `any_one` and `at_least(k)` dependency groups to report alternatives that share an evidence-admissible failure mode. It still does not quantify reliability, assume independence, prove causation, or automatically decide an allocation. Probability models must wait for explicit dependency assumptions, failure-mode data, calibration and independent validation.

### 8. From binary availability to measured functional adequacy

A water service delivering 70% of its required flow is not equivalent to a fully functional service or a complete outage. `service_function.rs` introduces a deliberately bounded comparison between an available-capacity quantity and a required-capacity quantity, each carrying an explicit unit, uncertainty interval, basis and supporting evidence. It reports conservative/optimistic coverage bounds and distinguishes `adequate`, `degraded`, `unavailable` and `indeterminate` outcomes.

An `adequate` conclusion requires the conservative capacity lower bound to meet the requirement upper bound. A `degraded` conclusion requires the entire non-zero capacity interval to remain below the requirement interval; zero capacity with a positive lower-bound requirement is `unavailable`. When uncertainty crosses a classification boundary, the result remains `indeterminate`. `uncertainty_abs` is treated as an absolute bound only when the source's semantics support that interpretation; it is not automatically a statistical confidence interval. Unit strings must match exactly; conversions belong in an explicit, versioned adapter. Observed measurements must match their supporting observation records, including entity, unit, value, uncertainty, state and validity; this query profile requires canonical UTC-second event, recording and validity timestamps no later than `as_of`. Batch evaluation indexes evidence once and returns stable scenario-ID ordering.

This is functional adequacy, not probabilistic reliability or causal inference. It does not yet compose multiple asset capacities, model hydraulics/load flow, estimate temporal recovery curves, or connect to live sensors. Those extensions need domain-specific conservation laws, calibrated inputs and independent evaluation before operational use. Recent interdependent-infrastructure research identifies limitations in binary component models and the need to represent progressive functionality and coupled dependencies; the local model begins with one narrow, auditable capacity comparison rather than claiming to solve those broader systems.

### 9. From nominal capacity sums to constrained end-to-end service flow

The capacity of individual components cannot simply be added to estimate what a service receives. A shared trunk may cap several parallel routes; a network may contain alternative paths that genuinely add capacity; and an unverified link must not silently count as either working or failed. The new service_network.rs applies directed maximum flow to an explicitly declared topology and reports a conservative flow using active edges and capacity lower bounds, plus an optimistic flow that also includes unknown-availability edges and capacity upper bounds. The conservative minimum cut returns the modeled links constraining that lower-bound flow.

Topology completeness is explicit. When the topology is partial or unknown, the state is forced to indeterminate; a missing edge list can no longer produce a false claim of outage or adequacy. The result is only as valid as the declared network, exact unit, edge capacities, availability states and evidence. In v0.1 this is a pure flow abstraction: it does not model pressure loss, voltage constraints, congestion, travel time, shared-cause correlation, or temporal routing. It must not be presented as a hydraulic, electrical or traffic simulator.


### 10. From identifying bottlenecks to comparing recovery interventions

An explanation of the current bottleneck is useful, but resilience planning needs to compare candidate actions: restore a route, increase a shared trunk, or leave a system unchanged. `service_network_intervention.rs` applies explicit hypothetical link overrides to a named baseline, re-evaluates conservative and optimistic max-flow bounds, exposes the before/after min cuts, and reports the change in modeled capacity bounds. Its batch API validates and indexes one shared evidence corpus once and orders results by (scenario ID, intervention ID).

The intervention output is a counterfactual comparison, not an execution receipt. A positive delta proves only that the stated assumptions improved flow in the declared model. If topology is incomplete, the result is `topology_incomplete`; it must not claim a service-wide improvement. The next planning layer should evaluate portfolios under explicit budgets, crews, repair times, access constraints and distributional safeguards. Do not rank actions by flow gain alone: essential-service equity, repair feasibility, affordability and risks must remain inspectable dimensions, not hidden weights in a single score. The current portfolio evaluator therefore retains its deterministic display rank but now also emits Pareto non-dominated layers over four explicit operational objectives: worst-case conservative capacity gain, minimum post-intervention conservative capacity, scenario count improved, and scenario count worsened. A Pareto front exposes trade-offs; it is not a social-welfare verdict. Cost, population served, distributional equity, crew-hours, repair time and ecological constraints are not yet modeled as objectives, so the current frontier must not be used as a capital-budget recommendation.

### 11. From point-in-time status to sampled service recovery

Resilience is not just the number of components that survived. It also concerns how long essential functions remain degraded and how much service is lost before restoration. NIST's community-resilience work calls for metrics that connect infrastructure performance to the social and economic functions served, including time to recovery and the effects of interdependencies. This aligns SEI with service-centred measurement rather than an asset-only score.

The new service_recovery.rs consumes individually evidence-validated capacity/demand snapshots for one service. It reports the first sampled robustly-adequate observation, the preceding sample and elapsed-time bracket, whether later samples remain adequate through the end of the observation window, maximum observed sampling gap, and lower/upper bounds on the time-integral of normalized service deficit. The integrated metric uses trapezoidal interpolation between the sampled deficit fractions and is expressed in equivalent full-service-deficit seconds; it is not a physical volume, exact unobserved recovery timestamp, probability, or causal estimate.

The trajectory requires the first snapshot to coincide with the declared disruption start, a definitely degraded or unavailable initial state, stable service/capacity identity and units, strictly increasing timestamps, and an explicit maximum sampling gap. Larger gaps fail closed rather than being bridged silently. A recovery claim is limited to the observed window: adequacy maintained through the last sample does not prove that service stayed adequate afterward. Explicit assumed snapshots remain scenario inputs, not observations; observed capacity and demand must pass the service-function evidence checks.

This is a discrete trajectory primitive, not a stochastic restoration model or a domain-specific physical simulator. Future time-to-recovery estimates must be evaluated against independently recorded disruptions and restoration times, with sampling gaps and uncertain records retained in the report.

### 10. From nominal resilience estimates to exhaustive bounded failure experiments

A baseline max-flow value answers what the declared network can deliver under one scenario. It does not answer how fragile that service is when multiple links fail together. The new service_network_stress.rs enumerates all combinations of one, two or three currently active link failures, subject to explicit limits of at most 10,000 failure cases and 2,000,000 total scenario-link references. It also accepts explicitly declared common-cause or hazard bundles (for example, one shared feeder fault disabling two alternative routes), labeled separately from the enumerated link combinations. It reports the exact candidate/evaluated case counts, case provenance, worst-case conservative and optimistic capacity, flow loss and all unique failed-link sets tied for the worst result.

This is deterministic stress testing, not probabilistic risk estimation. The enumerated cases are not event likelihoods; an N-2 result does not imply that two failures are likely or independent. Only active links are candidates. Each forced failure is an explicit assumed override that clears availability evidence for the overridden state but preserves the capacity observations. The analysis requires complete declared topology; a partial network must not support a service-wide resilience claim.

The budget is a safety boundary. If the exhaustive set exceeds the configured limit, the algorithm fails closed rather than silently truncating or sampling failure combinations. A successful response is exhaustive for the chosen order and declared graph. It still does not cover omitted hazards, shared-cause relationships that are not translated into the scenario, repair-time dependencies, or physical effects such as pressure and voltage. Those need separate, explicit models. This direction complements NIST's system-of-systems resilience methods, which emphasize service functionality, interdependencies, disruptive scenarios and recovery. The implementation intentionally keeps its narrower graph assumptions visible.

## Contract boundary (v0.1)

The machine-readable contracts are grouped by purpose:

- **Evidence and dependencies:** [EvidenceRecord](../../schemas/spatial-economic-evidence-v0.1.schema.json), [SpatialRelation](../../schemas/spatial-economic-relation-v0.1.schema.json), [DependencyGraph](../../schemas/spatial-economic-dependency-graph-v0.1.schema.json), [DependencyGroups](../../schemas/spatial-economic-dependency-groups-v0.1.schema.json), [AvailabilityScenario](../../schemas/spatial-economic-availability-scenario-v0.1.schema.json), [InterventionPlan](../../schemas/spatial-economic-intervention-plan-v0.1.schema.json), and [InterventionComparison](../../schemas/spatial-economic-intervention-comparison-v0.1.schema.json).
- **Functional capacity:** [Service-function input](../../schemas/spatial-economic-service-function-v0.1.schema.json) and [assessment output](../../schemas/spatial-economic-service-function-assessment-v0.1.schema.json).
- **Network flow:** [Service-capacity network input](../../schemas/spatial-economic-service-capacity-network-v0.1.schema.json) and [assessment output](../../schemas/spatial-economic-service-capacity-network-assessment-v0.1.schema.json).
- **Network stress:** [Exhaustive N-k stress request](../../schemas/spatial-economic-service-network-stress-v0.1.schema.json) and [stress assessment output](../../schemas/spatial-economic-service-network-stress-assessment-v0.1.schema.json).
- **Service recovery trajectories:** [Trajectory input](../../schemas/spatial-economic-service-recovery-trajectory-v0.1.schema.json) and [assessment output](../../schemas/spatial-economic-service-recovery-trajectory-assessment-v0.1.schema.json).
- **Network counterfactuals:** [Intervention plan](../../schemas/spatial-economic-service-network-intervention-v0.1.schema.json) and [intervention comparison](../../schemas/spatial-economic-service-network-intervention-comparison-v0.1.schema.json).
- **Common-cause domains:** [Domain](../../schemas/spatial-economic-common-cause-domain-v0.1.schema.json), [scenario](../../schemas/spatial-economic-common-cause-scenario-v0.1.schema.json), [assessment output](../../schemas/spatial-economic-common-cause-assessment-v0.1.schema.json), and [dependency-group overlap output](../../schemas/spatial-economic-common-cause-overlap-v0.1.schema.json).

The synthetic water-system network, dependency groups, baseline, scenario and intervention inputs are [water-system-dependencies.json](../../schemas/examples/water-system-dependencies.json), [water-system-dependency-groups.json](../../schemas/examples/water-system-dependency-groups.json), [water-system-availability-scenario.json](../../schemas/examples/water-system-availability-scenario.json), [water-system-outage-baseline.json](../../schemas/examples/water-system-outage-baseline.json), and [water-system-intervention-plan.json](../../schemas/examples/water-system-intervention-plan.json). Common-cause fixtures are [shared-feeder-domain.json](../../schemas/examples/shared-feeder-domain.json), [shared-feeder-scenario.json](../../schemas/examples/shared-feeder-scenario.json), [shared-feeder-evidence.json](../../schemas/examples/shared-feeder-evidence.json), and golden [shared-feeder-overlap.json](../../schemas/examples/shared-feeder-overlap.json). Functional-capacity fixtures are [service-function-scenario.json](../../schemas/examples/service-function-scenario.json) and golden [service-function-assessment.json](../../schemas/examples/service-function-assessment.json). Network-flow fixtures are [service-capacity-network-scenario.json](../../schemas/examples/service-capacity-network-scenario.json) and golden [service-capacity-network-assessment.json](../../schemas/examples/service-capacity-network-assessment.json). The exhaustive N-1 stress fixture is [service-network-stress-request.json](../../schemas/examples/service-network-stress-request.json); a separate [common-cause stress fixture](../../schemas/examples/service-network-stress-common-cause-request.json) disables two alternate routes in one declared event. Network-intervention fixtures are [service-network-intervention-plan.json](../../schemas/examples/service-network-intervention-plan.json) and golden [service-network-intervention-comparison.json](../../schemas/examples/service-network-intervention-comparison.json). Recovery-trajectory fixtures are [service-recovery-trajectory.json](../../schemas/examples/service-recovery-trajectory.json) and golden [service-recovery-trajectory-assessment.json](../../schemas/examples/service-recovery-trajectory-assessment.json).

Rust implementation is in [spatial_economic.rs](../../sol-atlas-core/src/spatial_economic.rs), [spatial_graph.rs](../../sol-atlas-core/src/spatial_graph.rs), [dependency_logic.rs](../../sol-atlas-core/src/dependency_logic.rs), [intervention.rs](../../sol-atlas-core/src/intervention.rs), [common_cause.rs](../../sol-atlas-core/src/common_cause.rs), [service_function.rs](../../sol-atlas-core/src/service_function.rs), [service_network.rs](../../sol-atlas-core/src/service_network.rs), [service_network_intervention.rs](../../sol-atlas-core/src/service_network_intervention.rs), [service_network_portfolio.rs](../../sol-atlas-core/src/service_network_portfolio.rs), [service_network_stress.rs](../../sol-atlas-core/src/service_network_stress.rs) for exhaustive bounded N-k stress, and [service_recovery.rs](../../sol-atlas-core/src/service_recovery.rs) for sample-bounded recovery analysis.

Each EvidenceRecord has:
- **Identity:** immutable statement ID, schema version, issuer and subject IDs.
- **Claim type:** observation, forecast, scenario, commitment, delivery, acceptance, invoice, settlement or impact measurement.
- **Epistemic state:** proposed, asserted, corroborated, disputed, superseded or withdrawn. This is not the lifecycle state of an order.
- **Spatial context:** canonical entity reference plus optional geometry URI and its coordinate reference system.
- **Time context:** occurred-at and recorded-at timestamps, with optional validity interval.
- **Typed values:** optional quantity/unit and/or decimal-string money plus explicit currency.
- **Provenance:** method, source artifact references, optional digest of exact captured bytes, and optional derivation ID.
- **Correction and policy:** superseded statement reference and an access-policy reference.

This contract validates structure only. It cannot verify issuer authority, signature validity, whether a source is independent, whether a physical event occurred, whether a currency code is current, or whether an intervention caused an impact. Those are separate, policy-aware verifiers and domain checks.

For decisions, use `DependencyGraph::assess_dependents_at`: it validates graph/evidence references, checks each supporting record's epistemic state and validity window, reports blocked edges with reasons, and returns the evidence readiness along each traversed path. Proposed, disputed, superseded, withdrawn, invalidly-timed, and out-of-window evidence cannot silently drive propagation. An asserted-only path is distinguished from a path containing a record marked corroborated. Neither label is a truth score or substitute for independent-source review. The lower-level `affected_dependents_at` is explicitly structural and should only be used where its evidence limitations are handled by the caller.

A correction appends a new statement that references the old one; it does not rewrite history. The core type deliberately does not encode a global reputation score or declare any source universally trustworthy.

## Reference architecture

1. **Spatial substrate:** stable entity IDs, geometry/CRS metadata, spatial-temporal queries, and support for offline snapshots. Prefer interoperable standards and adapters over a bespoke geospatial wire protocol.
2. **Evidence layer:** immutable/correctable claims, source manifests, issuer identity, explicit states and provenance.
3. **Dependency graph and logic:** typed, time-bounded links among places, assets, facilities, goods, counterparties, services, contracts and resources. `spatial_graph.rs` supplies deterministic evidence-gated reachability; `dependency_logic.rs` evaluates explicit `all_required`, `any_one` and `at_least(k)` groups against scenario conditions. Results are `satisfied`, `disrupted` or `indeterminate`; missing states and evidence-blocked edges remain unknown, not failures.
3a. **Common-cause exposure:** `common_cause.rs` records shared failure domains independently of dependency edges, returns evidence-gated exposure for each declared member, and correlates those exposures with dependency-group alternatives. It flags when two or more alternatives share an evidence-admissible domain and lists currently exposed relations, without changing the group outcome. Admissibility is not truth, source independence, or causal proof. No independence or probability claim is valid merely because a group is labelled redundant.
3b. **Functional adequacy:** `service_function.rs` compares one service's capacity and requirement using explicit units and uncertainty bounds, validates observations against cited evidence, and provides a batch API that indexes a shared evidence corpus once. It distinguishes adequate, degraded, unavailable and indeterminate outcomes. There is no implicit unit conversion, capacity aggregation, or reliability probability.
3c. **Network flow:** `service_network.rs` composes directed link capacities with conservative/optimistic max-flow bounds and exposes a deterministic lower-bound minimum cut. Unknown link availability widens the bounds. Incomplete topology forces indeterminate, even if the declared subgraph has zero flow. The algorithm is not a domain-specific physical simulator.
3d. **Exhaustive stress:** `service_network_stress.rs` evaluates every single-, double- or triple-active-link failure set within a caller-declared budget, returns worst-case flow-loss summaries and exact failed-link combinations, and refuses over-budget partial outputs. Scenario counts are not probabilities. This tests declared-link robustness; it does not prove robustness to unmodeled hazards or unknown topology.
3d. **Service recovery trajectory:** `service_recovery.rs` evaluates chronologically ordered service-capacity snapshots, brackets the first observed robust-adequacy sample, integrates normalized deficit bounds under an explicit trapezoidal interpolation policy, and rejects large sampling gaps. These metrics remain sample-bounded and non-causal.

The qualitative outcome rule is:

| Logic | Satisfied | Disrupted | Indeterminate |
|---|---|---|---|
| `all_required` | All currently active members are available | At least one active member is known unavailable | No active members, or unknown members prevent a conclusion |
| `any_one` | At least one active member is available | Every active member is known unavailable | No active members, or no available member and at least one unknown |
| `at_least(k)` | At least `k` active members are available | Even treating unknown members as available cannot reach `k` | Unknown members could change whether `k` is reached |

A missing scenario condition defaults to Unknown. Scenario inputs labelled `assumed` are explicit what-if inputs, not observations. Inputs labelled `observed` must cite structurally valid observation records whose state is asserted/corroborated, whose subject or spatial entity matches the component, and whose validity covers the query instant. These qualitative rules do not imply reliability probabilities or establish causal outcomes.
4. **Economic execution:** adapters for double-entry accounting, procurement, inventory, work orders, dispatch, invoices and payment systems. External systems remain authoritative for their own transactions.
5. **Scenario/evaluation engine:** explicit constraints, versioned assumptions, reproducible runs and baseline/negative-control reports.
6. **Human interface:** map, dependency explorer, claim timeline, commitment board, scenario compare view and evidence inspector. Every consequential recommendation should show its inputs and limitations.
7. **Federation/security:** identity, consent, access policy, signed statements, optional transparency receipts and independently verifiable exports.

### Standards reuse strategy

- **OGC API – Features:** discover and query interoperable spatial feature collections. Do not fork basic geospatial serving semantics.
- **OGC API – Moving Features:** consider for trajectories and moving entities when the use case requires it.
- **OGC SensorThings API:** consider for sensor/device/datastream/observation exchange; it does not replace the evidence contract or economic lifecycle model.
- **ETSI NGSI-LD:** consider for context-entity exchange and temporal relationship/property adapters; retain explicit local semantics and validation when mapping external records.
- **W3C PROV-O:** map provenance and derivation concepts where useful rather than inventing isolated names for all provenance relationships.
- **Open Contracting Data Standard (OCDS):** adapt procurement/contract publication where appropriate; it is not an all-purpose physical-delivery or settlement model.
- **IETF RFC 9943 (SCITT):** consider for signed statement transparency and receipts in supply-chain paths. Transparency is accountability machinery, not an oracle for truth.

These standards solve different layers. SEI must compose them, preserve their scope and document mapping losses rather than claim that any one standard supplies the whole system.

## First proving ground: water-to-food resilience

Use one bounded irrigation/food system as a design pilot, aligned with the current Baixo Limpopo source-readiness work. Start with source availability, rights, vintage, resolution, units, geography, independence and completeness—not a dashboard.

The minimal model might connect: water source → pump/energy asset → irrigation block → crop/season → storage/road → delivery point → purchase/contract → acceptance → invoice/settlement. Each link needs its own evidence; avoid inferring field-level productivity from province-level context or claiming causality from correlated satellite products.

Initial outputs are a reviewable evidence graph and a deterministic analysis plan, not an automated allocation engine.

### Evaluation gates

1. **Schema and semantics:** invalid records are rejected; claim types cannot be confused; state transitions are tested; decimal money and unit-bearing quantities remain explicit.
2. **Source qualification:** exact source bytes/hashes where captured, product/version, license, geography, temporal coverage, resolution and limitations are reviewed.
3. **Reproducibility:** identical inputs and model versions reproduce the same query/scenario outputs; manifests include code, parameters and source IDs.
4. **Economic reconciliation:** obligations, physical quantities and financial entries reconcile independently. Disputes, partial deliveries, losses, refunds and reversals remain representable.
5. **Outcome validity:** compare against a declared baseline with appropriate independent observations, uncertainty reporting, negative controls and explicit limits on causal interpretation.
6. **Human and rights review:** access control, data minimisation, appeal, distribution of benefits, and risks of exclusion are evaluated before any operational use.

Never treat “no data” as zero, “not captured” as unavailable, “queued” as passed, or a successful API response as scientific validation.

## Near-term implementation sequence

- **P0 — Contract:** ship the Rust contract primitives, JSON Schema and unit tests (this initial increment).
- **P1 — Adversarial fixtures:** add schema-validation tests and fixtures for missing CRS, invalid digest, conflicting claims, supersession, uncertain measurements, bad units and attempted state confusion.
- **P2 — Evidence adapters:** wrap current CHIRPS/WaPOR and supply-chain capture manifests without downloading or asserting data that has not been qualified.
- **P3 — Query proof:** evidence-gated traversal, qualitative dependency groups, common-cause overlap, functional-capacity intervals, network max-flow/min-cut, network intervention comparisons, bounded network portfolios, exhaustive N-k stress, and sampled service-recovery metrics now exist as separate bounded primitives. N-k stress enumerates all single-, double- or triple-active-link combinations and evaluates explicitly declared common-cause/hazard bundles. Each case preserves its origin, rationale and failed-link IDs; event bundles are not inferred causal facts. The combined case and graph-materialization budgets fail closed before materializing cases rather than silently truncating them. Tests compare 64 small deterministic networks against an independent exhaustive min-cut oracle and exercise a 10,000-link chain. Next, validate stress results against independently recorded outage combinations and observed service-restoration histories. Keep quantitative reliability probabilities disabled until dependence assumptions and calibrated event data exist.
- **P3 benchmark gate:** run `cargo bench --bench spatial_economic_intervention` and record exact commit, rustc version, target, CPU/RAM, warm-up/sample counts, p50/p95/max, and correctness counts in [the benchmark protocol](../benchmarks/SPATIAL_ECONOMIC_INTELLIGENCE.md). The harness is observational; published thresholds are proposed targets until measured on a named reference runner.
- **P4 — Economic lifecycle:** simulate one end-to-end obligation with partial delivery, acceptance rejection, invoice, settlement and reversal; test physical and financial reconciliation separately.
- **P5 — Evaluated pilot:** only after source qualification and human review, compare a bounded intervention to baseline.

## Non-goals and stop conditions

This RFC does not propose replacing OGC, accounting systems, banks, ERPs or government registries; tokenising all economic activity; centralising all data; inferring individual behaviour; or turning model outputs into automatic entitlement decisions. No operational claim is authorised by a valid schema, a matching digest, a signature, or a green unit test alone.

Stop or roll back any integration that silently drops provenance, changes units/currency, merges forecast and observation, cannot represent correction/dispute, leaks restricted data, or reports causal impact without a defensible evaluation design.

## References

- OGC API – Features: https://www.ogc.org/standards/ogcapi-features/
- OGC API – Moving Features: https://www.ogc.org/standards/ogc-api-moving-features/
- W3C PROV-O: https://www.w3.org/TR/prov-o/
- OGC SensorThings API: https://www.ogc.org/standards/sensorthings/
- ETSI NGSI-LD context information management: https://cim.etsi.org/NGSI-LD/official/clause-4.html
- W3C OWL-Time (Candidate Recommendation Draft): https://www.w3.org/TR/owl-time/
- IEC 61025 fault tree analysis: https://webstore.iec.ch/en/publication/4311
- IEC 61078 reliability block diagrams: https://tc56.iec.ch/dependability-standards/
- NIST/SEMATECH reliability handbook, parallel/redundant model: https://www.itl.nist.gov/div898/handbook/apr/section1/apr183.htm
- NIST Community Resilience Program: https://www.nist.gov/programs-projects/community-resilience-program
- NetworkX flow algorithms (independent reference for max-flow/min-cut semantics): https://networkx.org/documentation/stable/reference/algorithms/flow.html
- NIST Community Resilience Program, function-centred resilience and recovery metrics: https://www.nist.gov/programs-projects/community-resilience-program
- NIST Design Methods for Resilient Community Systems: https://www.nist.gov/programs-projects/design-methods-resilient-community-systems
- Liu et al. (2026), *Vulnerability analyses of power-water interdependent infrastructure networks by using load-functional dependency coupled cascading failure model*, Reliability Engineering & System Safety, DOI: https://doi.org/10.1016/j.ress.2026.113092. The work motivates representing progressive function and cross-network coupling; the Sol-Atlas capacity evaluator is a much narrower, non-probabilistic primitive.
- NIST Community Resilience Systems Model: https://www.nist.gov/programs-projects/development-first-generation-community-resilience-systems-model
- CISA Infrastructure Dependency Primer: https://www.cisa.gov/topics/critical-infrastructure-security-and-resilience/resilience-services/infrastructure-dependency-primer/learn

- Open Contracting Data Standard: https://standard.open-contracting.org/latest/en/
- IETF RFC 9943, *An Architecture for Trustworthy and Transparent Digital Supply Chains*: https://datatracker.ietf.org/doc/rfc9943/
