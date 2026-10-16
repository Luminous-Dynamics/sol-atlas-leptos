# Spatial-Economic Intelligence: architecture RFC

**Status:** Proposed design; not operationally validated  
**Version:** 0.1.0  
**Scope:** Sol-Atlas spatial reasoning and its interfaces to Mycelix coordination and economic execution  
**Implementation status:** initial Rust evidence contract, time-bounded dependency graph, qualitative dependency-group evaluator, baseline-versus-intervention comparator and JSON Schemas; no live economic or spatial integration

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

## Contract boundary (v0.1)

The machine-readable contracts are the [EvidenceRecord JSON Schema](../../schemas/spatial-economic-evidence-v0.1.schema.json), [SpatialRelation JSON Schema](../../schemas/spatial-economic-relation-v0.1.schema.json), [DependencyGraph JSON Schema](../../schemas/spatial-economic-dependency-graph-v0.1.schema.json), [DependencyGroup JSON Schema](../../schemas/spatial-economic-dependency-groups-v0.1.schema.json), [AvailabilityScenario JSON Schema](../../schemas/spatial-economic-availability-scenario-v0.1.schema.json), and [InterventionPlan JSON Schema](../../schemas/spatial-economic-intervention-plan-v0.1.schema.json). The synthetic network, redundancy groups, baseline, scenario and intervention inputs are in [water-system-dependencies.json](../../schemas/examples/water-system-dependencies.json), [water-system-dependency-groups.json](../../schemas/examples/water-system-dependency-groups.json), [water-system-availability-scenario.json](../../schemas/examples/water-system-availability-scenario.json), [water-system-outage-baseline.json](../../schemas/examples/water-system-outage-baseline.json) and [water-system-intervention-plan.json](../../schemas/examples/water-system-intervention-plan.json). Rust primitives are in [spatial_economic.rs](../../sol-atlas-core/src/spatial_economic.rs), [spatial_graph.rs](../../sol-atlas-core/src/spatial_graph.rs), [dependency_logic.rs](../../sol-atlas-core/src/dependency_logic.rs) and [intervention.rs](../../sol-atlas-core/src/intervention.rs).

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
- **P3 — Query proof:** evidence-gated as-of traversal, qualitative dependency groups, and a baseline-versus-intervention comparison now exist. The first comparator uses explicit assumed-state overrides and reports recovered/degraded/resolved/indeterminate changes without making causal claims. Next, model shared-cause/common-mode failure and alternate-route semantics; keep quantitative reliability probabilities disabled until dependence assumptions and calibrated component data exist.
- **P3 benchmark gate:** run `cargo bench --manifest-path sol-atlas-core/Cargo.toml --bench spatial_economic_intervention` and record exact commit, rustc version, target, CPU/RAM, warm-up/sample counts, p50/p95/max, and correctness counts in [the benchmark protocol](../benchmarks/SPATIAL_ECONOMIC_INTELLIGENCE.md). The harness is observational; published thresholds are proposed targets until measured on a named reference runner.
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

- Open Contracting Data Standard: https://standard.open-contracting.org/latest/en/
- IETF RFC 9943, *An Architecture for Trustworthy and Transparent Digital Supply Chains*: https://datatracker.ietf.org/doc/rfc9943/
