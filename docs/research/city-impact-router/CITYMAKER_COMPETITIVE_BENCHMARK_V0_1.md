# CityMaker: competitive benchmark and paradigm shift v0.1

**Research date:** 8 November 2026  
**Related:** [CITY-IMPACT-ROUTER-001 #649](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/649), [CITY-CHANGE-IMPACT-001 #643](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/643), [IEAC-INVALIDATION-V2-001 #650](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/650)  
**Claim status:** research and benchmark proposal; no competitive superiority has been measured.

## 1. The actual problem

A city or built environment is not one monolithic digital twin. It is an interdependent collection of independently owned evidence and operational systems:

- survey, geospatial, utility and environmental sources;
- BIM, IFC, GIS, CAD and asset registries;
- requirements, design revisions and engineering analyses;
- products, suppliers, material batches, logistics and custody;
- manufacturing, construction, commissioning and inspection;
- operations, maintenance, service continuity and measured outcomes;
- policy, standards, jurisdictional requirements and authorized decisions.

Each specialist system knows different facts and has different authority. A city-wide consumer must not turn federation into central ownership or convert a signed record into truth.

The hard question is:

> Given a change in one owner system, can we identify exactly which downstream engineering, asset, service and decision claims remain applicable, which must be revalidated, which are invalid, and which cannot safely be assessed—without missing consequential dependencies or revalidating unrelated work?

This is a **cross-owner evidence applicability and selective requalification** problem. It is more specific than model federation, model difference, clash detection, digital-thread storage, or a general enterprise ontology.

## 2. Paradigm shifts

### 2.1 From a monolithic twin to an evidence federation

A monolithic twin tries to gather every model and subsystem into one authoritative representation. The alternative is a federation of owner-controlled systems with explicit contracts for identity, payload, scope, time, provenance, dependency and report exchange.

Each owner retains its system of record. CityMaker holds a typed projection and the receipts needed to explain how the city-level view was formed. It must record what it cannot see, what it cannot verify and which owners were not represented.

This aligns with the integrated, unified and federated composition taxonomy in ISO 23247-6:2026. That standard is for manufacturing digital-twin composition; applying its composition principles to city/building evidence is an adaptation to test, not a claim of city-digital-twin certification or a newly invented federation model.  
Source: https://www.iso.org/standard/87426.html

### 2.2 From a single validity score to an evidence state vector

Do not collapse all assurance into one green/red state. Preserve distinct, non-substitutable dimensions:

1. **Identity:** is this the exact owner-native record, revision and subject?
2. **Content integrity:** do the bytes match the declared digest and length?
3. **Authority:** is the source owner or issuer authorized for this assertion and scope?
4. **Rights:** does the asserted use fall within the declared and authenticated rights?
5. **Inventory coverage:** have all relevant source pages, partitions, tenants, revisions and tombstones been accounted for?
6. **Dependency coverage:** are the mandatory relationships and dependency families represented?
7. **Technical applicability:** does the evidence still apply to this exact design, product, process, task and operating envelope?
8. **Decision reliance:** may this exact authorized decision rely on the evidence at the requested evaluation time?
9. **Outcome:** what was actually measured after installation or operation?

These axes are not interchangeable. A byte-verified payload can be revoked. A complete descriptor catalogue can come from an incomplete connector inventory. A graph traversal can be correct over an incomplete graph. A qualified simulation is not a physical inspection. A current engineering result is not an approval.

### 2.3 From change notifications to typed change consequences

A change event is not yet an impact report. The event must be routed to the owner family that can evaluate its consequences:

- artifact/design revision → artifact descendant and revalidation frontier;
- source correction/revocation → bitemporal source lifecycle and evidence projection;
- connector/source-set change → source inventory completeness;
- supplier, lot or custody change → supply-chain owner;
- calibration, robot, fixture or solver change → engineering/process qualification owner;
- policy edition change → standards or policy owner;
- maintenance, installation or operational change → asset/service owner.

A multi-domain change requires multiple owner reports. A single generic report must never be allowed to impersonate all domains.

### 2.4 From periodic whole-project rework to minimal justified requalification

After a change, revalidate the smallest set of claims justified by the authoritative dependency and applicability reports. Preserve unaffected claims and historical knowledge. Record why each affected claim is stale, invalid or unknown, and which exact post-change receipt discharges its revalidation obligation.

A small textual change can have broad physical consequences; a large file change can be irrelevant. Byte-diff magnitude is not a risk model. Domain-owned change semantics and exact dependency edges determine impact.

### 2.5 From "unknown means no issue" to explicit abstention

A missing owner lane, unsupported source, ambiguous coordinate transform, expired calibration, missing dependency edge, or unqualified inventory must remain Unknown/Insufficient. The router should not infer an owner report from absence, and a failed owner integration must not silently fall back to an unrelated graph traversal.

Safe abstention is a capability, not a defect. Improve it by closing real owner-contract gaps, not by changing labels.

## 3. Industry capability baseline

The following products and standards are direct comparators. The point is to compose with them and compare fairly, not to claim they lack established features.

| System / standard | Publicly documented capability | What CityMaker should reuse or benchmark | What is not established by that capability alone |
|---|---|---|---|
| Bentley iTwin Synchronization / Changed Elements | Connectors identify source differences and create incremental changesets; Changed Elements tracks related changed elements and changes over successive versions | Element/property delta fidelity, source synchronization, changeset lineage and connector coverage | Complete downstream engineering requalification across unrelated source owners and city-service decisions |
| Autodesk Construction Cloud / Model Coordination | Aggregates discipline models, detects clashes and supports version comparison; Revit 2027 documentation describes property-level changes for changed coordination elements | Model revision comparison, clash/issue continuity, element and property change precision | Cross-owner source-rights, supplier-lot, engineering-applicability, bitemporal and city-outcome assurance |
| Palantir Foundry Ontology | Represents objects, relations, data, logic and actions; offers native federation of external datasets without copying underlying data; supports model lineage and operational workflows | Federation, lineage, operational decision context, permissions and action audit boundaries | A built-environment-specific gold oracle for typed owner-lane correctness, engineering requalification and physical acceptance |
| Siemens Digital Twin Composer | Describes real-time product/plant/factory visualization, large-scale simulation, virtual commissioning and robot training | Factory and production simulation hand-offs, model identity, simulation-to-commissioning evidence | Construction-site performance or city-wide source/decision applicability |
| ISO 23247-6:2026 | Defines integrated, unified and federated manufacturing twin composition | Composition taxonomy, interface roles, interoperability assumptions and federation costs | A ready-made city/building change-impact engine |
| NIST IR 8536 (September 2026) | Manufacturing supply-chain traceability meta-framework for linking, querying and sharing time-ordered provenance across diverse ecosystems with selective disclosure | Product pedigree, supplier/lot linking, event lineage and selective disclosure | Engineering validity, source-universe completeness or city-service acceptance |
| IETF RFC 9943 / SCITT | Signed-statement transparency, append-only history and registration receipts | Verifiable statement inclusion, issuer attribution and audit trail | Truth, completeness, legal rights, engineering correctness or physical outcome |

### Primary documentation

- Bentley Synchronization API: https://developer.bentley.com/apis/synchronization/
- Bentley Changed Elements API tutorial: https://developer.bentley.com/tutorials/changed-elements-api/
- Autodesk Model Coordination overview: https://help.autodesk.com/cloudhelp/ENG/Coord-About/files/About_Model_Coord.html
- Autodesk Revit 2027 detailed coordination-model changes: https://help.autodesk.com/cloudhelp/2027/ENU/Revit-WhatsNew/files/GUID-B9284B64-E145-41D6-9242-9B243F262B2F.htm
- Palantir Foundry platform overview: https://www.palantir.com/docs/foundry/platform-overview/
- Palantir Ontology and Native Federation: https://www.palantir.com/explore/platforms/foundry/ontology/
- Siemens Digital Twin Composer (CES 2026): https://www.siemens.com/en-us/company/digital-transformation/industrial-metaverse/introducing-digital-twin-composer/
- ISO 23247-6:2026: https://www.iso.org/standard/87426.html
- NIST IR 8536 final: https://csrc.nist.gov/pubs/ir/8536/final
- RFC 9943 / SCITT: https://www.rfc-editor.org/rfc/rfc9943
- IFC 4.3.2.0 official release: https://technical.buildingsmart.org/standards/ifc/ifc-schema-specifications/
- IDS 1.0: https://www.buildingsmart.org/Standards/Bsi-Standards/Information-Delivery-Specification-Ids/

**Important IDS boundary:** IDS 1.0 provides machine-readable IFC alphanumeric information requirements, but buildingSMART says it does not cover geometric aspects. Geometric/as-built change impact therefore needs geometry-aware and survey/as-built evidence, not an IDS-only claim. IFC 4.3.2.0 is listed by buildingSMART as official (ISO 16739-1:2024); do not mistake a development documentation build for a new published schema.  
Sources: https://www.buildingsmart.org/Standards/Bsi-Standards/Information-Delivery-Specification-Ids/ and https://technical.buildingsmart.org/standards/ifc/ifc-schema-specifications/

## 4. Where CityMaker can credibly differentiate

The competitive hypothesis is that CityMaker can provide a **city-level, cross-owner requalification frontier** that:

- preserves specialist systems of record and owner authority;
- binds every event/report to native identity, payload digest, contract/schema, scope and time;
- routes change classes to the correct owner lane;
- composes multiple owner reports without creating a new generic graph/trust/impact engine;
- separately checks source inventory coverage and dependency coverage;
- explains the exact identity-preserving path from source event to engineering receipt, asset and city-service claim;
- identifies exact revalidation obligations and proves their discharge against post-change revisions;
- preserves valid historical knowledge views after late corrections;
- refuses NoImpact when a mandatory lane, source or dependency is missing;
- quantifies the human, solver, procurement and delay cost of safe revalidation.

This is a hypothesis to test. The fact that existing vendors document other capabilities is not evidence that they lack cross-domain features in their proprietary deployments. Compare only what can be observed, exported or independently verified under equivalent inputs.

## 5. Matched benchmark protocol

Use a rights-cleared frozen scenario and identical source snapshots, model revisions, dependency edges, query scope, evaluation time and accepted claim set for each comparator.

### Comparator A — full recomputation baseline

After each mutation, recompute the entire declared dependency/evidence set. This is the correctness baseline, not the expected fastest approach.

Measure total recompute time, human review, solver runs, queue delay, source import cost, decision latency and the exact resulting claim set.

### Comparator B — owner-native incremental workflows

Use Bentley changesets/Changed Elements for model and element changes, Autodesk model compare/clash outcomes for coordination changes, owner-native lifecycle/supply reports where available, and the canonical Sol-Atlas frontier/IEAC reports for their declared scopes.

Measure what each owner reports, its input coverage, export loss, explanation paths and whether it reports only local changes or downstream impact.

### Comparator C — common-data-environment / BIM coordination workflow

Reproduce the same model versions, clash/issue tracking, and change-review process. Do not assume a CDE has no downstream workflow; record actual interfaces, exported artifacts, manual steps and unavailable fields.

### Comparator D — CityMaker typed multi-owner consumer

Consume owner reports, source-inventory manifests and dependency-coverage receipts. It should not recompute specialist facts; it validates report family/identity/scope/time bindings and composes the resulting claim statuses.

### Required workload families

1. Design geometry and coordinate-frame revision.
2. Product/BOM/interface revision.
3. Source correction, rights change, withdrawal and issuer-key compromise.
4. Material-lot correction, recall and custody dispute.
5. Supplier-capacity and logistics disruption.
6. Calibration, firmware, tooling and process qualification change.
7. Standards/policy edition with effective-date boundary.
8. As-built discrepancy, inspection and repair.
9. Commissioning, service continuity and post-occupancy correction.
10. Compound event requiring multiple owner lanes.

### Required metrics

Report each independently:

- **Wrong-lane false acceptance:** target 0.
- **Stale-evidence false acceptance:** target 0 over mandatory declared edges.
- **False NoImpact:** target 0 when source inventory or dependency coverage is incomplete.
- **Affected-claim recall:** target 100% against independently authored gold dependency sets.
- **Unnecessary invalidation:** target 0 on explicit negative controls; report separately from false negatives.
- **Revalidation-frontier exact match:** target 100% on frozen E1 graph/corpus.
- **Path explanation agreement:** exact IDs, edge kinds and revision/digest bindings.
- **Historical replay correctness:** exact "known then" results after late-arriving corrections.
- **Owner report/schema mismatch rejection:** target 100%.
- **Abstention calibration:** separate missing owner, incomplete inventory, missing edge, untrusted authority, rights, stale report, and unsupported lane.
- **Human effort:** review minutes per event and per claim; manual reconciliation actions.
- **Engineering effort:** solver reruns, inspections and work packages triggered.
- **Compute:** full-recompute cost, sparse delta cost, graph import/build cost, validation cost, end-to-end P50/P95/P99 latency.
- **Data movement:** bytes copied, payloads disclosed, cross-owner query count and export loss.
- **Service impact:** time to identify affected assets/service claims and time to verified restoration/closure.

Never compare incremental execution time without including initial graph build, source inventory, coverage audits, report production, validation and human effort.

## 6. E1–E5 qualification ladder and proposed gates

### E1 — deterministic software semantics
Frozen synthetic gold corpus, independent reference implementation, adversarial mutations and exact-head receipts. Required: zero wrong-lane acceptances, zero missed mandatory impacts, zero false NoImpact and zero false invalidations on gold negative controls.

### E2 — upstream owner conformance
Reproduce exact report contracts and version boundaries. Add contract/schema mismatch, wrong-base, stale receipt, duplicate event, missing owner and late correction controls. Do not create duplicate upstream logic.

### E3 — held-out interoperability
Rights-cleared exports from at least two independent source-system families (for example IFC/BIM plus asset/supply-chain data). Independent reviewers compare input loss, native identity preservation, scope/time binding and missing-edge detection.

### E4 — supervised project/workcell trial
Controlled change exercises in a real but supervised project/workcell. Domain professionals verify exact work packages and decisions gated. Measure manual review and solver cost as well as software latency.

### E5 — sustained operating validation
Repeated change/recovery cycles including supplier/lot recalls, calibration, maintenance, delayed corrections, commissioning and service outcomes. Independently replicate on a second project or source-system family.

A synthetic PASS never proves real-world source completeness, engineering correctness, legal compliance, physical safety, service improvement or an authorized decision.

## 7. Scale tests and performance targets

Do not claim current SOTA from a small fixture set. Run a reproducible scale matrix over 10², 10³, 10⁴, 10⁵ and 10⁶ nodes where feasible, with degree distribution, edge family count, event batch size, revision churn and source-owner count reported.

For each size, measure:
- graph import and coverage qualification cost;
- full recomputation vs sparse frontier cost;
- p50/p95/p99 event-to-explanation latency;
- memory peak and serialized report size;
- negative-control and missing-edge detection;
- one owner versus 2, 5, 10, 25 and 50 owners;
- query fan-out and failure isolation when an owner is offline.

**Provisional engineering objective (not a benchmark result):** on a warmed, in-memory 100k-node graph with a pre-qualified exact coverage receipt, the sparse event-to-frontier path should aim for P95 below 1 second for a single small change and at least 10× lower compute than full recomputation, while preserving exact gold output and not counting graph-build/coverage cost as zero. Publish raw hardware, topology, fixtures, build SHA and all denominators; revise the target if the workload shows it is not meaningful.

## 8. Immediate engineering priorities

1. Close [#650](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/650): v0.2 invalidation event/report schemas and runtime for native-identity graph nodes.
2. Turn [#497](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/497) into a versioned source-inventory manifest and runtime receipt. NoImpact cannot rely on a descriptor list alone.
3. Bind [#488](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/488) dependency coverage to the same graph and evaluation scope as each impact report.
4. Add the exact Mycelix lot/supplier/custody receipt references to the lane registry only when owners publish pinned schemas/builds; do not invent them.
5. Add the same typed interface for engineering qualification, asset operation and policy applicability when their owner contracts exist.
6. Expand matched benchmarks to source-to-city service chains and measured repair/availability outcomes, not just graph traversal counts.

## 9. Conclusion

Do not attempt to beat Bentley at model synchronization, Autodesk at clash coordination, Palantir at enterprise ontology/action workflows, or Siemens at factory simulation by rebuilding partial versions of those systems.

Build and prove the narrow capability the combined workflow needs: **a typed, evidence-qualified, explainable and minimal city-level requalification frontier across independently authoritative owners**. The moat, if it exists, will be in zero missed cross-owner dependencies, zero unjustified invalidations, correct temporal replay, transparent abstention and measurable reduction in downstream rework—not in a larger 3D scene or a more generic ontology.
