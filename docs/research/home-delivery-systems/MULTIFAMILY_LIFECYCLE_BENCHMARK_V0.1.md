# Multi-family Lifecycle and Adaptation Benchmark v0.1

**Status:** synthetic identity / revision / event / claim-lineage benchmark. No real building lifecycle event, measurement, occupancy, repair or adaptation has been executed.
**Parent:** [MULTIFAMILY-LIFECYCLE-ADAPTATION-001 #849](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/849).
**Building:** [SYN-MF-COURTYARD-004](multifamily-building-synthetic-v0.1.json).
**Resident programme:** [SYN-MF-RESIDENT-PROGRAMME-001](multifamily-resident-programme-synthetic-v0.1.json).
**Unit layout:** [SYN-MF-UNIT-LAYOUT-004](multifamily-unit-layout-synthetic-v0.1.json).
**Shared services:** [SYN-MF-SERVICE-DISTRIBUTION-001](multifamily-service-distribution-synthetic-v0.1.json).
**Lifecycle adapter:** [multifamily-lifecycle-synthetic-v0.1.json](multifamily-lifecycle-synthetic-v0.1.json).

## 1. Research question

Can an apartment/building workflow preserve exact unit and asset identity from design through fabrication, installation, as-built capture, commissioning, occupation, maintenance, repair and adaptation—while ensuring each claim remains bound to the correct configuration revision and becomes invalid when its dependencies change?

The target chain is:

resident intent -> design revision -> assembly release -> installation/as-built -> commissioning -> occupied operation -> maintenance/repair -> adaptation -> reinspection/recommissioning

A home is not finished at practical completion. A BIM model, installation record, commissioning receipt, maintenance work order, measured service result and resident report are distinct evidence classes. One cannot silently substitute for another.

## 2. Paradigm shift: from handover to longitudinal evidence continuity

Treat a home as a versioned physical and service system, not a static design file or a single "digital twin" object.

The benchmark separates three clocks/identities:
- **Event time:** when a physical/configuration event occurred.
- **Record time:** when the event/evidence became known to the system.
- **Configuration revision:** the exact design/asset state to which the event and evidence apply.

This is bitemporal provenance. Late-arriving records, corrections, revoked measurements and superseded claims must be preserved without rewriting the history of what was previously known. Missing event/record time is UNKNOWN; it is never filled with an invented date or inferred from commit time.

Keep at least these state transitions separate:
- design issued / design changed;
- fabrication batch released;
- component installed / as-built surveyed;
- commissioning and acceptance;
- occupancy / in-use observation;
- maintenance / defect report;
- repair / replacement;
- reinspection / recommissioning;
- adaptation / retrofit / decommissioning.

An as-built scan does not prove a hidden fire/acoustic assembly. A work order does not prove a repair succeeded. A recovery label does not prove essential service returned. A post-occupancy reading from the previous revision does not automatically apply after an adaptation.

## 3. What current standards and research reveal

### OpenBIM: necessary exchange, not lifecycle truth

buildingSMART lists IFC 4.3.2.0 (IFC 4.3 ADD2; ISO 16739-1:2024) as the latest official IFC release. IFC 4.4 is still under development/draft:
- https://www.buildingsmart.org/standards/bsi-standards/industry-foundation--classes/
- https://standards.buildingsmart.org/

IDS 1.0 is the official machine-readable information-requirements standard. Its current documented scope is alphanumeric IFC information and common relationships; geometry, calculated/dynamic values, external references and specialist domain rules require other checking tools:
- https://www.buildingsmart.org/standards/bsi-standards/information-delivery-specification-ids/
- https://github.com/buildingSMART/IDS/blob/development/Documentation/UserManual/specifications.md

The 2026 buildingSMART IDS survey reports that the same IDS file can yield different checking results in different tools. It records strong demand for checking/authoring certification: 131 of 186 respondents (70%) wanted some form of IDS software certification. These are survey responses, not a measured defect rate across all BIM projects:
- https://www.buildingsmart.org/ids-survey-results-2026/

**Benchmark implication:** pin IFC/IDS editions and checker versions; compare rule-by-rule outcomes across independent implementations; keep geometry and physical-state validation as separate oracles; record implementation parity and disagreements. Luminous should not invent a competing BIM ontology or claim that IDS validates geometry, fire, accessibility, structure, thermal or hydraulics.

### Residential modular construction: missing lifecycle observations

A systematic review and meta-analysis published 15 November 2026 identifies three major modular-residential performance themes—GHG emissions, energy efficiency and indoor environmental quality—and highlights limited evidence for in-use/operational and end-of-life stages. Its reviewed embodied-carbon values range from 73 to 900 kgCO2e/m². The range reflects heterogeneous buildings, boundaries and methods; it is not a universal modular-housing factor or a result for a Luminous design:
- Perera, Rajagopalan & Woo, *An integrated performance assessment of modular residential buildings: Systematic review and meta-analysis*, Energy and Buildings 371 (2026), article 118174, https://doi.org/10.1016/j.enbuild.2026.118174

A separate 15 September 2026 residential-stock study proposes integrating functional and technical adaptability into performance-based renovation, emphasizing the value of preserving existing embodied investment rather than assuming demolition/reconstruction:
- Martin-Goñi & Paris-Viviana, *Adaptability assessment framework for the renovation of the residential building stock*, Energy and Buildings 367 (2026), article 117654, https://doi.org/10.1016/j.enbuild.2026.117654
- https://www.sciencedirect.com/science/article/pii/S0378778826007140

**Benchmark implication:** do not stop at factory output, first-pass installation, annual energy or practical completion. Measure the whole installed and occupied trajectory: defects, repairs, service downtime, replacement lead time, adaptation disruption, rework, material recovery and in-use outcomes. Publish the exact functional unit and system boundary.

### South African service context

Johannesburg Water's 29 October 2026 statement reported constrained reservoirs/towers, low pressure, intermittent supply and prolonged outages across multiple systems, with higher-lying areas particularly affected as levels declined. This is a dated municipal-system observation, not a forecast for a specific apartment:
- https://www.johannesburgwater.co.za/blog/2026/10/29/johannesburg-water-implements-urgent-interventions-to-stabilise-severely-constrained-systems/
- https://joburg.org.za/media_/Documents/2026-Media-Statements/JW-to-implement-urgent-interventions-to-stabilise-constrained-systems.pdf

**Benchmark implication:** lifecycle evidence should preserve service impact by unit/floor/riser and tie it to the asset and configuration revision. A building-average water-availability result must not hide an upper-floor or single-unit outage. Service outcomes remain with the owner-native measurement and resilience tools; the lifecycle adapter binds their receipts.

## 4. Existing owners and integration boundaries

The lifecycle adapter is deliberately thin:

| Concern | Owner | Lifecycle adapter responsibility |
|---|---|---|
| Design changes and evidence invalidation | Sol-Atlas #832 | Bind the exact change-impact receipt; reject stale claim state |
| In-use outcomes / POE / M&V / consent | Sol-Atlas #633 | Bind unit, revision, observation window, rights and owner-native receipt |
| Manufacturing / installation / batch / as-built | Sol-Atlas #814 | Preserve exact assembly/batch/installed-location identity |
| Site and foundation evidence | Sol-Atlas #836 | Bind investigation/design/hold-point/as-built lineage |
| Apartment geometry and resident programme | Sol-Atlas #833 / #840 | Preserve exact unit and intent identity |
| Shared service and outage impact | Sol-Atlas #844; Symtropy #1848; Symthaea service owners | Bind per-unit service boundary and trajectory receipts |
| Multi-epoch scan identity and physical-state continuity | Symtropy #4846 | Consume scan/state identity evidence, do not rebuild scan-to-BIM |
| Metrology / repeatability | Symtropy #4795 / #4507; Mycelix building metrology | Bind method, instrument, calibration, session and uncertainty |
| Asset custody, approval, maintenance and revocation | Mycelix asset/construction provenance | Preserve issuer and custody; never self-authorize |

This programme does not create a BMS, IoT platform, CMMS/CAFM, asset registry, survey system, sensor gateway, IFC replacement, POE survey tool, generic revision engine or building-physics solver.

## 5. P0 synthetic artifacts and contract

- multifamily-lifecycle.schema.json
- multifamily-lifecycle-synthetic-v0.1.json
- src/bin/home-multifamily-lifecycle-qualifier.rs
- .github/workflows/home-multifamily-qualifier.yml

The current synthetic candidate contains:
- 4 exact apartment unit identities;
- 13 unit-scoped configuration revisions;
- 2 shared-asset bindings (F0 water riser, shared water pump) linked to service-topology node identities;
- 24 authored lifecycle event records across design, adaptation, as-built, commissioning, occupancy, post-occupancy, maintenance, repair, reinspection and shared-asset maintenance;
- 4 unit timelines;
- 7 claim records, including two baseline U-001 claims explicitly invalidated by the adaptation event.

All event/record times are null and marked NOT_SUPPLIED. All events are AUTHORED_NOT_EXECUTED. All actual artifact digests, inspection/scan data, product lots, meters, resident feedback, repairs, costs, service downtime, calibration, approvals and commissioning evidence remain absent/unknown. No actual construction, occupation or repair is claimed.

The Rust qualifier checks:
- exact building, resident-programme, unit-layout and service-topology identity bindings;
- 4/4 unit-set equality across building, programme, layout and lifecycle;
- unique revision, asset, event, timeline and claim identities;
- revision parent existence, same-unit scope and cycle rejection;
- asset-to-service-node identity and served-unit membership;
- typed event kinds, exact unit/asset scope, monotonic logical event order and explicit unknown event/record time;
- revision transition ancestry and exact event/timeline coverage;
- reciprocal invalidation linkage between an adaptation event and stale claims;
- rejection of stale PASS/FAIL and stale UNKNOWN claims that were not invalidated/superseded/revoked;
- refusal to promote synthetic resident data, timestamps, evidence or physical execution.

It checks only identity, revision and claim-state consistency. It does not calculate the general dependency closure owned by #832. It does not prove any event happened or any building state is true.

## 6. Benchmark case families

The first synthetic positive control and negative controls should include:
1. Complete unit/asset/revision/event lineage with no physical claim.
2. Duplicate unit, asset, revision, event, claim or timeline identity.
3. Unknown unit, floor, service asset or served-unit mapping.
4. Broken revision parent, cross-unit parent or ancestry cycle.
5. Adaptation where a stale PASS survives.
6. Adaptation where an affected claim is absent from the invalidation receipt.
7. Unaffected neighbour unnecessarily requalified despite no dependency impact.
8. Unknown dependency incorrectly treated as proof of unaffected status.
9. As-built observation promoted into design intent or specialist certification.
10. Commissioning report bound to the wrong installed product/configuration revision.
11. Maintenance ticket promoted to successful repair.
12. Repair event promoted to service restoration without reinspection/recommissioning.
13. Recurrent fault hidden by a new PASS or changed status label.
14. Post-occupancy measurement applied to a different unit, meter boundary or revision.
15. Expired calibration or changed instrument profile used as current evidence.
16. Event time confused with record time when a correction arrives late.
17. Missing event time silently synthesized from file/commit timestamp.
18. Resident survey/feedback reused without purpose, consent/legal basis, retention and access state.
19. Unit-level service or disruption hidden by building average.
20. Physical/operational event falsely claimed from an authored synthetic scenario.

The adapter should validate the event/claim envelope; owner-native tools should execute and adjudicate their own physical or operational cases.

## 7. Fair industry comparison

Do not compare a synthetic lifecycle JSON document against a production digital-twin platform and declare victory. Compare equivalent tasks and outputs:

| Baseline | Equivalent task | Comparison outputs |
|---|---|---|
| Revit / OpenBIM CDE handover | Model and asset identity across design/as-built handover | IFC identity/relationship round-trip, revision mismatch, exchange loss |
| CMMS / CAFM workflow | Asset work order, maintenance, repair and recommissioning | asset-to-unit mapping, closure evidence, repeat-fault handling, repair history |
| Scan-to-BIM workflow | Multi-epoch building/component state continuity | identity switches, correspondence completeness, as-built discrepancy and uncertainty |
| POE / M&V workflow | Design-to-in-use result and resident feedback | revision/window alignment, calibration, consent, M&V comparability and missingness |
| Professional-led integrated AEC handover | Human coordination of design, construction, commissioning and operations | manual review effort, lost identity, unresolved scope and re-review time |
| Luminous lifecycle adapter | Evidence continuity over owner-native receipts | stale-PASS count, invalidation recall, unit-level impact, time to identify responsible owner and exact affected scope |

Freeze tool/version/license, IFC/IDS edition, source and target model, asset/unit scope, event corpus, observation window, export path, human effort and output package. A feature unavailable in a region/account is not counted as available. Compare each tool only in its declared capability envelope.

## 8. Metrics

### Hard gates
- 100% exact unit/asset/configuration identity continuity across the declared lifecycle sequence;
- zero stale PASS after a relevant design/physical revision;
- 100% invalidation recall against an independently adjudicated dependency corpus owned by #832;
- zero false re-review of unaffected claims without an explicit reason;
- zero repair/restoration claim without a post-repair test/recommissioning receipt;
- zero measurement claim without exact unit boundary, source, method, instrument, calibration, window, uncertainty and configuration revision;
- zero real date/time or physical execution claimed from synthetic logical ordering;
- zero use of resident personal data without purpose/rights/consent;
- zero building-average result that hides a unit-specific service or habitability failure.

### Outcome distributions
- event/asset identity continuity and scan identity switches;
- as-designed/as-built mismatch detection and adjudication time;
- stale claim count; invalidation precision/recall; time to requalify;
- repair and recommissioning duration; repeat fault rate;
- unit service-loss and resident-disruption time;
- replacement part availability/lead time, factory/site rework, waste and lifecycle cost;
- resident-reported disruption and acceptance, kept as a self-report evidence class;
- missingness, uncertainty, disputed evidence and unresolved owner dependencies.

### Research hypotheses (not achieved results)
- At least 25% lower time to identify affected units and claims than a matched manual handover/maintenance review, with no reduction in invalidation recall.
- At least 20% lower repair-to-requalified-service time for a matched replaceable assembly, with the same safety, commissioning and specialist review gates.
- Zero stale PASS on all known-answer and sealed confirmation cases before any superiority claim.
- No universal scalar score. Report per-unit, per-floor, per-building and lifecycle vectors; retain distributional differences.

## 9. Physical pilot gates

A future pilot should begin in an unoccupied, instrumented test cell such as the Symtropy #4531 integration bench, not by experimenting on an occupied apartment. A real-building pilot requires owner/operator permission, competent engineering and inspection, resident notice/consent where relevant, data rights/privacy review, calibrated instruments, incident/stop conditions and independent adjudication.

A measured lifecycle study must separately capture:
- exact installed component/lot and as-built identity;
- commissioning and calibrated meter/instrument references;
- unit-boundary service and indoor-environment outcomes;
- repair/maintenance work and post-repair test;
- resident-reported experience with consent/retention rights;
- design/configuration revision and exact impact/invalidation receipt.

Never create an outage or safety defect to produce a benchmark trace in an occupied home.

## 10. Claim ceiling

The current schema, fixture and Rust qualifier establish only synthetic identity, revision lineage, logical event ordering, unit/asset scope and non-contradictory claim state. They do not prove as-built condition, actual construction, commissioning, occupation, resident comfort, service continuity, repair, compliance, safety, cost savings, adaptability performance or buildability. No physical or post-occupancy result is claimed.
