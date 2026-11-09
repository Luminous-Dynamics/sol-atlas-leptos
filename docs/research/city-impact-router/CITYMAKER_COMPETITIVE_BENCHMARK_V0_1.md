# CityMaker: competitive benchmark and paradigm shift v0.1

**Research date:** 8 November 2026  
**Primary-source verification refreshed:** 9 November 2026  
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
| Bentley iTwin Synchronization / Changed Elements V2 | Connectors compare source models between sync runs and emit Changesets: initial sync includes the full model, subsequent syncs contain incremental differences, and unchanged sources produce no new Changeset. Changed Elements V2 reports added, deleted and modified elements over an explicit Changeset range. | Match initial ingestion cost separately from incremental sync cost; measure delta fidelity, Changeset lineage, connector coverage, stable identity and mapping of non-graphical changes to physical assets. | A single documented claim that all external owner domains, trust/rights changes, qualification evidence and city-service decisions are covered by one complete requalification oracle. |
| Autodesk Construction Cloud / Model Coordination | Aggregates discipline models, automatically detects clashes and supports issue review. Coordination Space V2 performs clash checks based on model content changed since the last check. Revit 2027 describes property-level comparison for changed coordination-model elements. Automatic clash detection requires stable unique element IDs across versions; Autodesk documents a fixed 0.5 mm clash tolerance. | Incremental clash and model-version comparison, property-delta fidelity, issue continuity, stable-ID handling, tolerance matching and behaviour when IDs are unstable or change. | Cross-owner source lifecycle, supply-chain provenance, engineering evidence applicability, bitemporal decision reliance and city outcome assurance as one independently verified chain. |
| Palantir Foundry Ontology | Models objects, properties, links, actions, functions and dynamic security. Native Federation incorporates external datasets without duplicating underlying data. Externally hosted models can have versioning, granular permissions and governed lineage; Ontology actions can write back to operational systems. | Benchmark federation latency/freshness, source-of-truth preservation, access/marking behaviour, model lineage, governed action/writeback and audit history. | A public independently authored built-environment gold oracle for exact cross-owner source/contract/time/coverage routing and engineering requalification. Do not assume proprietary deployments lack such workflows. |
| Siemens Digital Twin Composer | Siemens describes real-time product/plant/factory context, large-scale simulation, virtual commissioning of factories and autonomous-robot training. | Simulation-to-manufacturing workflows, virtual commissioning, industrial-scale model interoperability and robot/process evidence hand-off. | City-wide source-owner inventory, legal/rights lifecycle and cross-jurisdictional decision applicability. |
| ISO 23247-6:2026 | Specifies manufacturing digital-twin composition, including integrated, unified and federated composition, communication, aggregation and interoperation. | Composition taxonomy, interface roles, interoperability assumptions and costs of combining independently developed twins. | A ready-made city/building change-impact engine or proof that an implementation meets engineering acceptance requirements. |
| NIST IR 8536 (9 September 2026) | Final manufacturing supply-chain traceability meta-framework for cryptographically linked, temporally ordered provenance across diverse ecosystems, supporting independent product-history verification and selective disclosure. NIST's 9 September announcement said its Python reference implementation was still undergoing final review. | Map product pedigree, supplier/lot linkage, event lineage and selective disclosure; benchmark the report's model and principles, not an assumed publicly runnable NIST implementation. | Engineering correctness, source-universe completeness, decision applicability or city-service acceptance. |
| IETF RFC 9943 / SCITT | Defines transparency architecture for single-issuer signed statements, verifiable data structures, append-only statement history and registration receipts. | Issuer-statement transparency, receipt validation, auditability and proof that a statement was registered under the service policy. | The factual truth of the statement, completeness of the source universe, legal rights, physical correctness or approval authority. |
| buildingSMART IDS 1.0 / IFC 4.3.2.0 | IDS expresses machine-readable IFC alphanumeric information requirements; IFC 4.3.2.0 is an official IFC release associated with ISO 16739-1:2024. | Exact IFC schema/version binding, alphanumeric delivery requirements, IFC entity/property validation and open exchange fixtures. | IDS alone does not check geometry, dynamic/calculated values, external datasets, discipline-specific advanced relationships or whether a model matches as-built conditions. |

### Primary documentation

- Bentley Synchronization API: https://developer.bentley.com/apis/synchronization/
- Bentley Changed Elements API tutorial: https://developer.bentley.com/tutorials/changed-elements-api/
- Autodesk Model Coordination overview: https://help.autodesk.com/cloudhelp/ENU/Coord-GS/files/About_Model_Coord.html
- Autodesk Coordination Space V2 incremental checks: https://help.autodesk.com/cloudhelp/ENG/Coord-GS/files/coordinate-models/Model_Coord_Coordination_Space_Versions.html
- Autodesk Revit 2027 property-level coordination changes: https://help.autodesk.com/cloudhelp/2027/ENU/Revit-WhatsNew/files/GUID-B9284B64-E145-41D6-9242-9B243F262B2F.htm
- Autodesk stable element-ID requirement and coordination limits: https://help.autodesk.com/cloudhelp/ENG/Coord-GS/files/Model_Coord_Supported_Files.html
- Palantir Foundry Ontology and Native Federation: https://www.palantir.com/explore/platforms/foundry/ontology/
- Palantir Ontology overview and action types: https://www.palantir.com/docs/foundry/ontology/overview
- Siemens Digital Twin Composer (CES 2026): https://www.siemens.com/en-us/company/digital-transformation/industrial-metaverse/introducing-digital-twin-composer/
- ISO 23247-6:2026: https://www.iso.org/standard/87426.html
- NIST IR 8536 final: https://csrc.nist.gov/pubs/ir/8536/final
- NIST announcement and reference-implementation status: https://www.nist.gov/news-events/news/2026/09/finalized-manufacturing-supply-chain-traceability-meta-framework
- RFC 9943 / SCITT: https://www.rfc-editor.org/rfc/rfc9943
- IFC 4.3.2.0 official release: https://technical.buildingsmart.org/standards/ifc/ifc-schema-specifications/
- IDS 1.0 scope and limitations: https://www.buildingsmart.org/standards/bsi-standards/information-delivery-specification-ids/
- IDS documented limitations (geometry, external data, dynamic values, as-built checks): https://github.com/buildingSMART/IDS/blob/development/Documentation/UserManual/specifications.md

**Fair-comparison rule:** Bentley's changesets and Autodesk Model Coordination V2 are already incremental. The benchmark must run each comparator's best documented incremental path, preserve the same source versions and coverage boundary, and include initial tracking/indexing and owner-side processing cost. Autodesk's 0.5 mm clash tolerance and stable-element-ID prerequisite must be matched in the test inputs. If stable IDs cannot be maintained, report the resulting continuity loss rather than silently remapping elements. Do not claim that CityMaker is more incremental merely because it avoids a full recomputation.

**Important IDS boundary:** IDS 1.0 is limited to IFC alphanumeric information and common relationships. Geometry, calculated/dynamic values, external data and as-built state require complementary geometry-aware, survey/as-built or domain-specific evidence. IFC 4.3.2.0 is listed by buildingSMART as official (ISO 16739-1:2024); do not mistake an in-development documentation build for a new published schema.

### Comparator operating assumptions (primary-source check, 9 November 2026)

- **Bentley:** include full initial iModel synchronization in lifecycle cost; subsequent Changesets are incremental. Changed Elements V2 compares a selected Changeset range and reports additions, deletions and modifications. An incremental-only timing that excludes initial ingestion is misleading.
- **Autodesk:** reproduce Coordination Space V2's changed-content clash check; use the documented 0.5 mm tolerance and stable unique element IDs across versions. Run a separate identity-instability stratum because Autodesk documents continuity limitations when IDs are not stable.
- **Palantir:** it already documents native federation, externally hosted model versioning/security/lineage, Ontology object/link/action modelling and operational writeback. Compare governance and action workflows, not just visualization.
- **Siemens:** Digital Twin Composer, unveiled at CES 2026, combines twin data with real-time physical data and supports large-scale factory simulation, virtual commissioning and robot training. It is the factory/workcell comparator for production and commissioning evidence.
- **NIST:** IR 8536 is final; the NIST announcement dated 9 September 2026 said the accompanying Python reference implementation was still in final review. Treat IR 8536 as the meta-framework comparator unless a public, pinned implementation becomes available.

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
