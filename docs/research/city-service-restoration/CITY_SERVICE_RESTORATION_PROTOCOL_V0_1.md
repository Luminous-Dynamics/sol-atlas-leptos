# CITY-SERVICE-RESTORATION-001 — Protocol v0.1

**Status:** research protocol; synthetic-first; not a deployed planner or operational approval  
**Owner:** Sol-Atlas composition layer  
**Parent:** CITY-MAKER-001 #549; CITY-REFERENCE-DISTRICT-001 #615  
**Issue:** https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/639  
**Branch:** research/city-service-restoration-v0.1

## 1. Research question

Given a service interruption and a proposed repair/restoration action sequence, can the sequence be executed with the exact parts, lots, people, competencies, tools, access, transport, work windows, isolation prerequisites, inspections and commissioning evidence available at the required times?

Existing CityMaker owners already measure service-state timelines, asset/repair workflows, production/supply facts and delivery feasibility. This work composes their authoritative receipts to validate restoration feasibility. It must not create a second continuity engine, repair planner, inventory ledger, scheduler, dispatch system, robot controller, asset registry, safety authority or commissioning authority.

## 2. Design hypothesis

A restoration claim becomes materially more trustworthy when the city model checks a complete, time-consistent chain:

`observed condition → permitted intervention → qualified resources → feasible delivery/access → work execution → independent inspection → commissioning → owner-confirmed service`

This is a hypothesis to test, not an assertion that software can guarantee repair success.

The key shift is from *predicting a recovery timestamp* to *checking the evidence and physical prerequisites that make a proposed recovery sequence possible*. Recovery time is not inferred by this evaluator.

## 3. Owner boundaries

| Evidence or capability | Authoritative owner | This protocol's role |
|---|---|---|
| Adequate/constrained/inaccessible/unknown service timeline and stable recovery | Sol-Atlas #574 | Consume; never recompute |
| District, asset, scenario, revision and functional-unit envelope | Sol-Atlas #623 / PR #624; #615 consumer | Require exact binding |
| Source payload bytes and observed digest | Sol-Atlas #500; adapter #637 | Require report when the contract demands byte verification |
| Delivery route, receiving, unloading and acceptance | Sol-Atlas #573 | Consume owner receipts |
| Access and affected-group constraints | Sol-Atlas #555 | Consume owner receipts |
| Service cassette test, isolation, replacement and recommissioning | Symtropy #4181 | Consume product/work evidence |
| Robotic inspection, repair and re-verification | Symtropy #4043 | Consume asset workflow evidence |
| Accepted work output, labour, rework and task accounting | Symtropy #3605 | Consume accepted-output evidence |
| Supplier, inventory, substitution, capacity, skills and lead times | Mycelix #4814 | Consume authoritative resource facts |
| Lot identity, custody, inspection, correction and revocation | Mycelix #5149 | Consume provenance facts |

No owner fact is copied into a second ledger. A missing owner adapter is an explicit Unknown, not an inferred fact.

## 4. Input envelope

Every restoration proposal must bind:

- District, building, asset and service identifiers.
- Exact design, as-built, configuration, product/BOM and source revisions.
- Functional unit, scenario identity, scope and validity interval.
- Fault/condition observation and owner-native service-state timeline.
- Ordered work packages, precedence constraints, permitted repair envelope, hold points and isolation requirements.
- Exact part and lot identity, quantities, custody, receiving, current condition and owner-native reservation/availability receipts.
- Crew/worker identity, skills and task authorization for the required interval.
- Tool/equipment identity, capability, availability and maintenance state.
- Instrument identity, calibration status and valid measurement interval.
- Site access, work-zone constraints, lifting/handling envelope, transport, delivery, unloading and laydown receipts.
- Inspection, functional test, commissioning and stable-service predicates.
- Evidence owner, schema/revision, object reference, digest basis, byte report where applicable, evidence class, freshness, uncertainty, rights and permitted claim ceiling.

### Time semantics

All event times must use a declared time basis and unit. Intervals are explicit and typed (valid time, observation time, planned time, actual time, receipt time). The evaluator must not silently compare naive local timestamps, UTC timestamps and elapsed durations. Incompatible time bases or missing required offsets are Unknown unless a proven contradiction makes the proposal Infeasible.

### Resource semantics

A catalogue record is not stock. A stock quantity is not an allocation. A nearby supplier is not delivery capacity. A person profile is not task authorization. A robot capability is not available equipment. A part is not deployable without exact revision, lot/custody, acceptance and compatible interface evidence. Resource ownership, reservation and allocation remain with their source systems.

## 5. Verdicts and state transitions

### Plan readiness

- **Feasible:** every declared hard precondition is supported by current, scope-matched owner evidence, and no resource, sequence, time, interface or hard-gate conflict exists.
- **Infeasible:** authoritative evidence proves a hard contradiction: wrong product revision, revoked lot, exclusive double allocation, impossible time window, denied access, failed required isolation prerequisite, insufficient committed quantity or an out-of-envelope intervention.
- **Unknown:** required evidence is absent, stale, ambiguous, inaccessible, unsupported, not byte-verified when byte verification is required, or not authoritative enough for the proposed claim.

A proven hard contradiction wins over unrelated positive facts. Missing evidence does not become Infeasible merely because it is absent; it remains Unknown. Unknown is never treated as Feasible.

### Four independent lifecycle axes

1. **Plan readiness:** Feasible / Infeasible / Unknown.
2. **Work execution:** NotStarted / InProgress / Failed / Completed / Unknown.
3. **Inspection:** NotPerformed / Pass / Fail / Unknown.
4. **Commissioning:** NotPerformed / Pass / Fail / Unknown.
5. **Service state:** owner-native Adequate / Constrained / Inaccessible / Unknown.

Do not overload these dimensions into one enum. For example, a proposal may be plan-Feasible, work-NotStarted, inspection-NotPerformed, commissioning-NotPerformed and service-Unknown. It is not a restored service.

Only the continuity owner can establish service adequacy and stable restoration for the exact service, scenario, scope and time interval. This evaluator must not create a recovery timestamp from plan completion, a robot's completion event, a visual inspection or a single adequate sample.

## 6. E1 synthetic corpus

The companion file `CITY_SERVICE_RESTORATION_CORPUS_V0_1.json` declares 32 deterministic synthetic fixtures. It includes positive controls and adversarial cases for parts, revisions, lots, custody, substitution, suppliers, route closures, delivery windows, unloading, skills, workforce capacity, concurrent demand, tools, calibration, sequencing, isolation, protected services, access, dimensional fit, repair limits, rework, inspection, commissioning, stable service and payload provenance.

The fixture expectations are software known answers only. They are not engineering values, workforce standards, real inventory, local supplier evidence, hazard probabilities or performance predictions.

### Corpus construction rules

- Freeze every input fixture and its schema version before evaluation.
- Author expected results independently of the implementation.
- Include positive controls for each state and negative controls for each hard gate.
- Mutate one causal condition at a time where possible; add pairwise interactions in a separate layer.
- Include permutation/replay controls to detect order-dependent results.
- Preserve Unknown and Infeasible as distinct outcomes.
- Record all blockers, source identities and reason codes; never emit only a Boolean.
- Keep a held-out corpus for E2/E3; do not tune against it.

## 7. Metrics

Report each metric separately:

- Known-answer agreement and exact-head reproducibility.
- False-Feasible rate (critical; zero target for all hard-negative fixtures).
- False-Infeasible rate.
- Unknown/abstention rate, classified by missing, stale, ambiguous, revoked, unsupported and inaccessible evidence.
- Blocker localization completeness and reason-code agreement.
- Owner receipt and payload lineage completeness.
- Temporal consistency and interval-boundary correctness.
- Duplicate exclusive allocation and resource contention detection.
- False promotion between readiness, work, inspection, commissioning and service states.
- Once physically measured by source owners: complete-workflow labour, elapsed time, first-pass/final acceptance, rework, service deficit-hours, time to stable restoration, unaffected-service fraction, worker exposure and lifecycle material/energy/cost.

Do not introduce a weighted recovery score. A faster plan cannot compensate for unsafe isolation, inaccessible work, unqualified parts, missing inspection or inequitable loss of essential service.

## 8. Benchmark ladder

| Level | Evidence | Exit criterion |
|---|---|---|
| E1 | Frozen synthetic fixtures | 100% independently authored known-answer agreement; zero false-Feasible on hard negatives; deterministic receipt |
| E2 | Method reproduction | Map applicable ISO 37123 city-resilience indicator definitions and NIST task-specific robotics measurement principles without misrepresenting either as a repair planner |
| E3 | Instrumented, guarded mock-up | Calibrated measurements and independently reviewed work/inspection/commissioning sequence on non-hazardous articles |
| E4 | Supervised physical deployment | Professional design, site risk assessment, competent people, statutory approvals, safe work controls and accountable commissioning |
| E5 | Sustained operation | Repeated, measured maintenance/recovery cycles and independent replication across more than one context |

Do not set productivity improvement targets before a matched conventional baseline exists. Use total workflow labour-hours and independently accepted functional output from Symtropy #3605; count design, coordination, kitting, logistics, setup, supervision, inspection, failed attempts, rework and recovery.

## 9. Research and industry references

- ISO 37123:2019, *Indicators for resilient cities*. Defines comparable and verifiable city resilience indicators; reviewed and confirmed in 2025. This is an outcome-measurement reference, not a work-order scheduler. https://www.iso.org/standard/70428.html
- NIST, *Performance Assessment Framework for Robotic Systems*. Composes task-specific perception, mobility, dexterity and safety measurements; supports subsystem-to-task measurement rather than capability-by-marketing-claim. https://www.nist.gov/programs-projects/performance-assessment-framework-robotic-systems
- NIOSH, *Safety in Construction Robotics*, 7 October 2026. Highlights changing workspaces, sensor degradation, communication/software faults and unsafe fault recovery. https://www.cdc.gov/niosh/bulletin/2026/safety-in-construction-robotics.html
- Xu et al., *Construction robotics: a systematic review of robot types, applications and human robot collaborations*, Results in Engineering, June 2026. PRISMA review of 375 studies; reports concentration in structural tasks and underexplored planning, maintenance and demolition. https://doi.org/10.1016/j.rineng.2026.110406
- Valkonen et al., *Tolerance management in prefabrication of mechanical, electrical, and plumbing installations*, published online 11 February 2026. Five project cases and seven principles; emphasizes communication, detailed design, quality control and buffer assembly. https://doi.org/10.1080/01446193.2026.2626735
- buildingSMART, *Information Delivery Specification (IDS) 1.0*. Useful machine-readable IFC information-requirements crosswalk. IDS does not itself validate geometry, dynamic values, external data or whether a model matches as-built state; those checks need dedicated owners. https://www.buildingsmart.org/Standards/Bsi-Standards/Information-Delivery-Specification-Ids/

South African physical pilots remain subject to the National Building Regulations and Building Standards Act 103 of 1977, applicable SANS requirements, competent professional design and local authority approvals. This software protocol grants no statutory approval. https://www.gov.za/documents/national-building-regulations-and-building-standards-act-16-apr-2015-1302

## 10. Claim boundary

An E1 Feasible result proves only that a frozen synthetic fixture satisfies this evaluator's declared receipt and temporal preconditions. It does not prove a real part exists, a person is qualified, a worksite is safe, a repair will succeed or a city service has been restored. Physical claims require measured, owner-qualified evidence at the appropriate E-level.
