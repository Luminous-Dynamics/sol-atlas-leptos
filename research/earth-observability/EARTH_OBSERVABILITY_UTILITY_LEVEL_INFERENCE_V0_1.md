# EARTH-OBSERVABILITY-UTILITY-LEVEL-001
## From sensor events to evidence-qualified underground utility networks v0.1

**Parent:** #771 — EARTH-OBSERVABILITY-QUAL-002  
**Related existing owner:** [MUDDI-aligned subterranean utility profile #755](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/755)  
**Related physical programme:** Symtropy [SUBTERRANEAN-UTILITY-FABRIC #4910](https://github.com/Luminous-Dynamics/symtropy/issues/4910)  
**Status:** research protocol and challenge cases; utility-association evaluator NOT_IMPLEMENTED  
**Date:** 2026-11-13  
**Claim ceiling:** no real utility asset, network geometry, locate, survey, depth, material or excavation clearance is produced or qualified by this document.

---

## 1. A more important unit of intelligence: the utility, not the reflection

A GPR detector can correctly identify a hyperbolic reflection and still be wrong about the physical infrastructure.

A single pipe can produce multiple radar events. One radar event may arise from a rock, rubble, a backfill boundary or a processing artefact rather than a utility. A real utility may appear intermittently because of orientation, spacing, moisture, clutter, antenna coupling or depth. A collection of plausible events does not automatically define a continuous pipe/cable route.

A 2026 review of GPR-based urban utility mapping identifies this **event–utility mismatch** and the synthetic-to-field domain gap as central limits to reliable mapping. It describes contemporary methods as often operating at event-level hyperbola detection rather than utility-level inference; the review notes that event-to-asset association and continuous network reconstruction remain difficult. It also reports that field studies cited in the review found 25–40% of algorithmically identified “pipes” corresponded to non-utility objects in particular shallow urban (<1.5 m) survey contexts. That range is a reported result from cited case studies, not a universal false-positive rate and not a baseline to assume for another city or instrument. DOI: https://doi.org/10.3390/s26092708.

This demands a second evaluation layer:

**radar event → target hypothesis → physical asset association → utility segment geometry → network topology → decision-specific confidence**

A system should not be rewarded only for finding more bright hyperbolas. It should be rewarded for producing fewer, better-supported, correctly associated physical asset claims.

## 2. Keep the architecture compositional

Do not create another underground utility data model.

- **Sol-Atlas #755** owns the MUDDI-aligned logical model, persistent identity / record / system identifiers, spatial evidence, disclosure classes and interoperable utility exchange.
- **This benchmark lane** owns event-to-asset reasoning metrics, independent utility truth, continuity/topology tests and claim boundaries.
- **Symtropy #4910** owns physical survey/robotics/locating methods, sensor calibration, route localization and physical qualification.
- **Symthaea** can propose associations, competing network hypotheses and additional investigations, but cannot authoritatively change an owner's utility registry.
- **Mycelix** owns rights, custody, access authorization, provenance and accountable review.
- Canonical utility identity, service state, ownership, access and maintenance stay with the responsible asset owner.

An inferred line is a **candidate utility claim**, not a new authoritative MUDDI NetworkAsset. Keep its evidence lineage, confidence and supersession state explicit. A record-derived alignment is not a verified as-built; a sensor event is not a verified utility; a verified segment does not establish the complete network topology.

## 3. Four distinct layers and claims

### 3.1 Event layer

An event is an observation-level object: radar reflection, EM locator response, seismic anomaly, borehole response or other sensor indication. It has its own acquisition, time, coordinate frame, support, sensor configuration, processing chain and uncertainty.

Measure event detection separately:
- event precision/recall or detection probability against event labels;
- false events per route kilometre / survey area / acquisition time;
- event localization uncertainty;
- detector calibration and abstention;
- environment and configuration strata.

Event-level metrics are diagnostic, not the final utility map score.

### 3.2 Asset-association layer

An asset hypothesis groups one or more events and source records into a proposed physical entity. Association must account for:
- one asset producing multiple events in neighbouring scans or multiple antenna views;
- one event being compatible with multiple asset hypotheses;
- crossing utilities, close parallel conduits and congested service corridors;
- duplicate events caused by multi-channel acquisition, migration or processing;
- non-utility clutter that resembles an asset;
- uncertainty in object type, material, diameter and depth.

Measure:
- event-to-asset association precision/recall;
- asset-instance precision/recall;
- false asset hypotheses per route-km;
- over-merging (distinct physical assets assigned one identity);
- over-splitting (one physical utility assigned multiple identities);
- ambiguity / abstention rate and confidence calibration.

A high event-detection score must not compensate for incorrect asset associations.

### 3.3 Segment-geometry layer

A utility is usually a continuous or piecewise-continuous physical entity. Score inferred segments against independently surveyed as-built/controlled truth:
- centerline lateral and vertical error P50/P95;
- endpoint and junction position error;
- symmetric Hausdorff or average symmetric distance, with geometry scale fixed before evaluation;
- completeness/recall of true utility length;
- false inferred utility length per route-km;
- unsupported inferred-length fraction (length with no admitted supporting evidence);
- interval coverage for position/depth uncertainty;
- gaps, disconnections and confidence along the route.

Do not score a long, smooth hallucinated pipe as successful continuity. Every predicted segment must retain evidence and a declared interpolation / continuity rule.

### 3.4 Network-topology layer

A utility network is more than a set of lines. It contains connectivity, nodes, junctions, crossings, network type and potentially direction/flow constraints. Compare predicted and reference topology:
- node and connection precision/recall;
- edge/segment precision/recall and connectivity F1;
- network component count error;
- wrong cross-connection / missed connection rate;
- topology preservation across crossing/near-parallel cases;
- asset identity and attribute consistency along connected segments.

A visual overlap between two lines must not silently become a network junction. The physical connection itself requires supporting evidence or remains ambiguous.

## 4. Benchmark design: independent truth is the bottleneck

The benchmark must contain event labels, utility asset IDs, segment geometry and network topology, not only radargram boxes. A single physical utility, every view of it and every event associated with it must be kept in one partition.

Minimum split discipline:
- leave-one-asset-out when multiple assets share a site;
- leave-one-site-out;
- leave-one-acquisition campaign / operator / sensor configuration out;
- prospective blind site where lawful access and qualified reference truth are available.

Ground truth candidates include controlled test trenches with surveyed pipes/cables and verified as-built geometry, authorized open excavations, verified utility-owner as-builts reconciled with independent locates, and competent survey/metrology measurements. Record the truth source, exact geometry, coordinate frame, epoch, uncertainty and blind adjudication. A source registry or as-built drawing alone is not infallible truth.

If excavation cannot be conducted safely or lawfully, that target remains unverified. Do not infer physical truth from the same GPR/EM observations used to build the hypothesis.

## 5. Simulation-to-field strategy

Physics-based simulation is useful for controlled variation and known truth. For GPR, finite-difference time-domain models such as gprMax solve Maxwell's equations for a declared geometry and dielectric model. That is an important baseline, but it does not automatically reproduce heterogeneous soils, irregular objects, backfill, clutter, antenna coupling, field motion, instrumentation artefacts or real acquisition variability.

Use a staged hybrid strategy:
1. T1 analytic synthetic cases for graph, association and metric correctness.
2. T2 physics synthetic radargrams with explicitly declared forward assumptions and model discrepancy.
3. T3 permissioned retrospective physical data with independent excavation/as-built truth.
4. T4 prospective campaigns with preregistered blind labels, independent surveys and competent review.

A model trained on synthetic radargrams must not be described as field-capable until it passes held-out physical site/configuration tests. Synthetic data can identify expected sensitivities; it cannot stand in for field validation.

The 2026 review explicitly recommends hybrid simulation and field datasets, physics-guided approaches, multi-sensor evidence and utility-level reasoning rather than treating synthetic data as a direct substitute for measured surveys. DOI: https://doi.org/10.3390/s26092708.

## 6. Additional challenge cases v0.1

The machine-readable case definitions are in `utility-level-inference-cases-v0.1.json`.

| Case | Stress | Expected disposition |
|---|---|---|
| MULTI-EVENT-ONE-ASSET-001 | one physical utility creates repeated events across adjacent lines/views | group as one candidate asset; do not inflate asset count |
| CLUTTER-NO-ASSET-001 | rocks, rubble or migration artefacts mimic a pipe signature | retain event if warranted; do not promote unsupported utility asset |
| CONTINUITY-GAP-001 | real utility has intermittent observations due orientation/attenuation/occlusion | preserve a probabilistic segment or explicit gap; no unsupported certainty |
| CROSSING-IDENTITY-001 | crossing/close parallel utilities produce overlapping signatures | avoid false merge/split and fabricated junction |
| MULTIVIEW-REGISTRATION-001 | B/C/D scan views or GPS/IMU have correlated registration bias | propagate registration uncertainty; no false alignment confidence |
| SYNTHETIC-FIELD-SHIFT-001 | simulator-trained model sees heterogeneous site / different hardware | detect domain shift, stratify or abstain |
| ASBUILT-CONFLICT-001 | legacy utility record conflicts with independent locate or open excavation | preserve claims and conflict; do not overwrite owner record |
| ORIENTATION-POD-001 | utility orientation relative to scan path changes event detectability | conditional POD by orientation; no universal absence claim |

These are planned challenge definitions, not completed measurements or executable sensor simulations.

## 7. Competitive benchmarks and targets

Do not compare a utility-network model only with event-level detectors. Compare the same data pack and reference truth using:

1. manual expert interpretation with documented process and adjudication;
2. conventional hyperbola/feature detector plus manual association;
3. event detector plus geometric clustering (e.g. density clustering / RANSAC trajectory);
4. multi-view event fusion with GPS/IMU and declared registration uncertainty;
5. utility-network inference with source records and physical constraints;
6. physics-guided hybrid or probabilistic asset association;
7. oracle using hidden truth (upper bound only).

Report every layer separately and end-to-end. Required metrics:
- event POD/precision/false alarms per route-km;
- asset instance precision/recall and overmerge/oversplit rates;
- route completeness and false inferred length per km;
- segment P50/P95 lateral/depth error and uncertainty coverage;
- topology edge/node F1 and false connection rate;
- OOD/abstention, unsupported geometry length, source conflict and stale-record handling;
- end-to-end engineering decision loss / regret at an equal investigation budget.

**Proposed promotion gate:** no asset or network geometry claim is admitted without exact event/source lineage, a validated spatial frame, time and uncertainty. Unsupported inferred geometry must be quantified. A correct event detector with a wrong asset map fails the utility-level benchmark. There is no single universal percentage threshold: define acceptable error and completeness per utility type and consequence before blind evaluation.

## 8. Standards and interchange boundary

Use the MUDDI-aligned profile #755 to express the candidate utility/space/network objects, identifiers, observations, evidence relations, time and disclosure class. The new benchmark owns the semantics of what constitutes a detection event, an asset association and a qualified network inference.

Relevant exchange anchors:
- OGC MUDDI Part 1 v1.0: https://docs.ogc.org/is/23-024/23-024.html
- OGC SensorThings API 1.1: https://www.ogc.org/standards/sensorthings/
- OGC API – Connected Systems Parts 1/2 (1.0): https://www.ogc.org/standards/ogc-api-connected-systems/
- OGC SensorML 3.0: https://www.ogc.org/standards/sensorml/

These standards help exchange utility objects, sensor deployments and observations; they do not themselves validate an inferred utility network or authorize disclosure of precise critical-infrastructure geometry.

## 9. Execution order

**P0 — benchmark and fixtures (Sol-Atlas)**
- [x] Define event/asset/segment/network evaluation layers.
- [x] Define eight abstract challenge cases.
- [ ] Extend deterministic fixture with event IDs, physical asset IDs, segment truth and topology labels.
- [ ] Add strict event-to-asset identity and split-leakage guards.
- [ ] Add event-level and asset-level scoring receipts; preserve abstentions and unsupported length.

**P1 — physical model and association (Symtropy + Sol-Atlas)**
- [ ] Build a small synthetic 2D/3D utility scene with multiple views and explicit ground truth.
- [ ] Reproduce event detection / clustering / RANSAC baselines.
- [ ] Add orientation, coupling, clutter and multi-path perturbations.
- [ ] Qualify against permissioned controlled trench / survey data.

**P1 — source reconciliation and decision use (Sol-Atlas + Symthaea + Mycelix)**
- [ ] Bind candidate assets to MUDDI logical objects without minting canonical owner identities.
- [ ] Compare survey-derived geometry to owner-authorized records and preserve conflicts.
- [ ] Choose the next discriminating investigation by existing #133/#134 EVSI path; do not duplicate EVSI.
- [ ] Enforce access, rights and disclosure limits before any recommendation or export.

This is the path from “the radargram contains a plausible reflection” to “this candidate utility segment is supported by independent evidence, its uncertainty is bounded, and the next action is justified.”
