# Highveld Keep — Federated Habitat Twin Composition v0.1

**Parent:** HIGHVELD-KEEP-001 #607  
**Industry comparison:** HIGHVELD-KEEP-INDUSTRY-001 #617  
**Status:** research architecture and synthetic benchmark protocol; not an implemented federation service  
**Research cut-off:** 8 November 2026  
**Claim ceiling:** architecture contract and synthetic interoperability mechanics only

## 1. Architectural decision

Do not build one universal Luminous digital twin. Build a **federated habitat evidence system** that composes owner-native twins, models and records without transferring authority to the composer.

Use three composition modes deliberately:

- **Federated by default** for owner-native source data, trust, rights, custody, operational systems and jurisdictional authority.
- **Unified only at the query/contract boundary** where common identity, units, time, scope and result semantics are required.
- **Integrated only inside a bounded, explicitly authorized application or simulation** where relevant inputs may lawfully and safely be brought together.

A project may use all three modes, but each composition must declare its mode and authority boundary.

The differentiated claim to test is not that federated digital twins are new. They are not. It is that Luminous can compose independently owned built-environment evidence into an independently replayable, time-qualified decision while preserving source authority, rights, uncertainty and selective disclosure across design, manufacture, construction, operation and maintenance.

No advantage has yet been demonstrated.

## 2. Standards update and scope

### ISO 23247-6:2026

Published in July 2026, ISO 23247-6 defines integrated, unified and federated digital-twin composition in manufacturing, including communication, aggregation and interoperability among twins developed by different vendors or parties.

**Highveld use:** adopt its composition taxonomy and implementation discipline as an architectural reference for manufacturing and building-product evidence.

**Limit:** it belongs to the manufacturing digital-twin series. This mapping is not a claim that Highveld or an urban twin conforms to the standard, nor that this standard alone covers permitting, professional engineering, BIM, GIS, occupant consent or city governance.

Official source: https://www.iso.org/standard/87426.html

### NIST IR 8536 — supply-chain traceability

NIST IR 8536, Supply Chain Traceability: Manufacturing Meta-Framework, was finalized on 9 September 2026. It describes an industry-neutral approach to linking and querying traceability records across supply-chain ecosystems, including cryptographically verifiable links and selective disclosure.

**Highveld use:** apply the principles to product/BOM revision, supplier declarations, material-lot identity, custody, inspection, correction, revocation and accepted panel output. Compose Mycelix #5149 and #4814 rather than implementing a competing provenance or supplier ledger.

**Limit:** cryptographic linkage establishes properties of records; it does not make an underlying claim true, qualify a material, approve an installation or replace a building code.

Official sources:
- https://csrc.nist.gov/pubs/ir/8536/final
- https://www.nist.gov/news-events/news/2026/09/finalized-manufacturing-supply-chain-traceability-meta-framework

### IFC 4.3.2.0 and IDS 1.0

Use official IFC 4.3.2.0 (IFC 4.3 ADD2 / ISO 16739-1:2024) as the open exchange baseline for the first Highveld package. Pin the exact schema and exchange profile for each run.

Use IDS 1.0 for machine-readable alphanumeric information requirements and property checks. IDS success is not proof of correct geometry, topology, coordinate frames, clearances, physical condition or as-built equivalence. Geometry and reality comparisons require independent methods and declared tolerances.

Official sources:
- https://technical.buildingsmart.org/standards/ifc/ifc-schema-specifications/
- https://www.buildingsmart.org/standards/bsi-standards/information-delivery-specification-ids/

### EnergyPlus 26.2.0

EnergyPlus 26.2.0 was released on 30 September 2026. Pin this exact release through Symtropy #1900 when the adapter's qualification evidence is available. Do not silently substitute a development build or a different release into matched comparisons.

Release record: https://energyplus.net/epupdate.htm

### Robotics performance composition

NIST's Performance Assessment Framework for Robotic Systems composes task-level performance from perception, mobility, dexterity and safety measurements rather than assuming component specifications prove end-to-end task performance.

Reuse this philosophy with Symtropy's capability-bound work packages and accepted-output records. Measure setup, calibration, localization, fixture changes, human intervention, safe aborts, downtime, inspection, rework and accepted output—not only nominal robot cycle time.

Official source: https://www.nist.gov/programs-projects/performance-assessment-framework-robotic-systems

A 2026 systematic review of 375 construction-robotics studies and a separate scoping review of 129 on-site studies identify deployment barriers, site variability, integration effort and safety assurance; operation and end-of-life applications remain comparatively sparse. This supports a guarded, non-load-bearing panel workcell with full task accounting—not an autonomous whole-building robot.

Research:
- https://doi.org/10.1016/j.rineng.2026.110406
- https://www.sciencedirect.com/science/article/pii/S0926580526005339

## 3. Federation contract

Every owner-native object remains authoritative at its source. A composed view may reference, compare, summarize or selectively disclose it, but cannot silently rewrite it or elevate the consumer's authority.

A portable owner reference binds, where applicable:

| Field | Purpose |
|---|---|
| Owner and contract identity | Who owns the assertion and which contract defines it |
| Native object, subject and revision | Exact source identity; no name/geometry fuzzy join |
| Schema/profile version | Exact interpretation of the source record |
| Payload reference and digest basis | Exact object/bytes, not consumer-reserialized JSON |
| Common scope and functional unit | What the record is about and whether it can be compared |
| Time basis and intervals | Valid, observed, recorded, registered and ingested time |
| Evidence state and freshness policy | Current, stale, delayed, disputed, revoked, unknown or unsupported |
| Authority and permitted purpose | Who may issue, verify, disclose and rely on the record |
| Rights/disclosure profile | What can be shared, with whom, for how long and for what purpose |
| Verifier identity/build and receipt | Which independent implementation checked which claim |
| Claim ceiling and defeaters | What the result does not establish |

A reference is not a copy of source truth. Its digest is not proof of issuer authority. A signature is not proof of engineering correctness. An IDS pass is not a geometry pass. A common scope digest is not evidence completeness. A successful simulation is not a permit. A completed robot task is not accepted work.

### Authority map

| Domain | Source of authority | Luminous composition role |
|---|---|---|
| Resident intent and consent | Residents / authorized representatives | Preserve purpose, access and dissent |
| Site and spatial context | Qualified survey, public authority, source-native GIS | Bind context and uncertainty |
| Design intent and alternatives | Versioned design contract and accountable designers | Explore alternatives and retain Pareto trade-offs |
| Geometry/BIM | Exact geometry provider and BIM authoring owner | Validate identity, units, revisions and transformation loss |
| Thermal/structural/fire/civil claims | Qualified specialist solvers and responsible professionals | Bind exact inputs, assumptions, convergence and limits |
| Products and material lots | Supplier/manufacturer and approval/provenance owners | Preserve revision, lot, custody and approval references |
| Fabrication and inspection | Producer/workcell and independent metrology/inspection owners | Compose accepted-output and as-built receipts |
| Construction and approvals | Responsible professionals, contractor and competent authority | Preserve approvals and hold points |
| Operations and maintenance | Asset/utility/service owners | Compose current state, calibration and repair records |
| Decision reliance | Authorized relying party and its policy | Evaluate evidence for a declared purpose, never authorize by default |

### Composition rules

1. Federate by default; do not duplicate confidential owner databases merely to make a dashboard easier.
2. Unify semantics only where a cross-owner query requires common identity, units, time and scope.
3. Integrate data only for a declared application and purpose, with the complete input manifest preserved.
4. Never fuzzy-merge identities from similar names, geometry or location.
5. Do not propagate authority transitively: byte verification does not imply approval; product approval does not approve installation; commissioning does not prove sustained service.
6. Missing, inaccessible, stale, unsupported or rights-restricted data remains Unknown, Unavailable or NotComparable.
7. Use selective disclosure; share only claims required for the relying party and purpose.
8. Require deterministic replay for identical source snapshots, policies, verifier builds and evaluation times.

## 4. Platform allocation

- **Symthaea — intent and discovery:** compiles resident/project intent into explicit requirements, candidate hypotheses, conflicts and value-of-information questions. Recommendations are not geometry, solver results or approvals.
- **Symtropy — physical/process evaluation:** supplies exact geometry, qualified solver adapters, manufacturing/process models, accepted task outputs and metrology interfaces within their declared scope.
- **Sol-Atlas — spatial and lifecycle composition:** binds site/district context, common scope, source identity, temporal state, continuity and restoration evidence; it does not become the source owner.
- **Mycelix — provenance, trust and governance:** preserves issuer/holder/verifier identity, consent, authority, custody, lot provenance, correction, dispute and revocation. A signed claim is not thereby technically true.
- **External AEC/engineering systems:** Revit, Forma, Rhino/Grasshopper, IFC/IDS tools, EnergyPlus, structural solvers, BMS/CMMS/ERP and manufacturer systems remain valid participants. Bridge where Luminous adds evidence continuity; do not replace mature tools without a matched reason.

Reuse existing Sol-Atlas decision reliance/change-impact/source coverage work (#501/#509/#644, #488/#500/#606/#637), Mycelix trust/provenance (#347/#5149), Symtropy specialist evidence, and the state synchronization/restoration slices (#638/#646). Do not create another trust registry, ledger, graph database or universal schema.

## 5. Minimal demonstrator: one non-load-bearing panel

1. Freeze one panel's function, acceptance criteria, repairability and inspection requirements.
2. Bind exact panel geometry revision and IFC/IDS projection; preserve identity, unit, coordinate-frame and round-trip loss reports.
3. Bind only qualified specialist checks with exact inputs; unqualified thermal, structural, fire or weather claims stay Unknown.
4. Bind manufacturer/product/BOM revision, product approval scope, lot identity and custody/correction/revocation history.
5. Bind workcell, robot/tool capability, fixture, calibration, work window, human interventions and accepted functional output.
6. Bind independent metrology, tolerance, defect and rework receipts.
7. Bind the exact installed component and design/as-built revision.
8. Bind in-use inspection, maintenance and service state to the same declared scope.
9. Assess proposed repair feasibility only when authoritative resource, access, isolation, delivery and crew receipts exist.
10. Inject a changed BOM, revoked lot, geometry revision or calibration expiry; verify selective invalidation without missed obligations.

A single panel with a complete evidence thread is a stronger first demonstrator than an unqualified whole-building twin.

## 6. E1 federated-composition benchmark

Build hand-checkable synthetic cases before live connectors:

1. All required owner references are exact-scope and current.
2. One source owner is unavailable; projection becomes Unknown, not partial success.
3. Similar asset labels but distinct native IDs; no fuzzy merge.
4. Explicit owner-authorized identity succession preserves old and new IDs.
5. Scope digest matches but functional unit differs; NotComparable.
6. Payload digest matches but issuer authority is missing; insufficient evidence.
7. Byte verification passes but permitted-purpose rights are revoked; no reliance.
8. IFC/IDS property check passes while geometry identity/coordinate frame differs; geometry remains independently unresolved.
9. Owner A updates a BOM while owner B retains a stale panel link; dependent claims go stale.
10. Late-recorded correction has valid time before a historical decision; preserve both as-known historical and current corrected views.
11. Correction is valid only after the decision; do not rewrite the historical view.
12. Late ingestion changes current knowledge but must not appear in an earlier ingestion-time replay.
13. Schema/profile change makes an adapter unsupported until requalified.
14. Source inventory is incomplete; no NoImpact claim.
15. Query requests restricted data outside its purpose/role; deny disclosure.
16. Owner revocation/dispute arrives; do not restore the old assertion from a cache.
17. Solver output is authentic but exact input geometry changed; revalidation required.
18. Physical panel is accepted but building commissioning/service evidence is absent; no building-service claim.

P0 targets:
- 100% known-answer agreement.
- Zero false native-identity merges.
- Zero false current/reliable admissions with required source coverage, rights, trust or scope missing.
- Zero silent semantic, geometry, revision or units loss in declared critical fields.
- Zero missed mandatory invalidations against an independent full-scan oracle.
- Zero restricted-field disclosures under seeded purpose/authority negatives.
- 100% deterministic replay for identical pinned inputs.
- Preserve positive controls; rejecting everything is not success.

P1 strata: 10 / 100 / 1,000 / 10,000 / 100,000 owner objects and 1 / 5 / 10 / 25 owners. Report p50/p95/p99 latency, memory, network bytes, source-owner calls, false reliance/refusal, unknown rate, stale-detection latency, invalidation precision/recall, review time and proof size separately. Performance comparisons start only after P0 correctness gates pass.

### Machine-readable gold corpus

The 18 cases are frozen in:
- `HIGHVELD_KEEP_FEDERATED_TWIN_COMPOSITION_CORPUS_V0_1.json`
- `HIGHVELD_KEEP_FEDERATED_TWIN_COMPOSITION_CORPUS_V0_1.schema.json`

Each case carries an as-known-at-decision cut and an as-known-now cut. The expected result separates three views:
- **Historical reliance:** evidence actually available/admitted at the original decision cutoff.
- **Retrospective historical assessment:** whether later-discovered evidence indicates that a fact was already incorrect at the original decision time (for example, a correction discovered later but valid earlier).
- **Current reliance:** whether the decision can still be relied upon now.

This prevents two opposite errors: leaking future information into the historical replay, and pretending that a later-discovered backdated correction has no effect on present-day assessment of the old decision. The expected dispositions map to the decision-reliance vocabulary used by #501/#509. This is a proposed cross-owner fixture for those owners to consume; this addendum does not implement another decision-reliance evaluator.

## 7. Matched industry comparison

Compare toolchains at workflow level, not one product against an entire industry.

| Baseline | Strength to respect | Matched Highveld question |
|---|---|---|
| Autodesk Forma + Revit | Connected early design, schematic modelling, analysis and detailed BIM handoff | Do requirements, source context and evidence status survive into downstream checks? |
| Rhino/Grasshopper + specialist solvers | Parametric exploration and custom analysis | Are hard gates, diversity, uncertainty and equal high-fidelity budgets preserved? |
| Bentley iTwin | Engineering/reality-data integration and lifecycle change visibility | Can cross-owner decision reliance be independently replayed? |
| IFC/IDS toolchains | Open exchange and machine-readable requirements | Are information and geometry/coordinate changes independently validated? |
| CMMS/EAM + ERP/supply systems | Asset, maintenance, parts, procurement and work-order management | Is a repair proposal checked against time-valid stock, custody, crew, tool, access and accepted-work evidence? |
| Human-led best-of-breed workflow | Professional judgement and mature domain methods | Does federation reduce evidence reconciliation without hiding human review? |
| Luminous federation | Proposed cross-owner evidence continuity | Does it reduce false assurance and revalidation cost while preserving interoperability, rights and source sovereignty? |

Freeze product/version, date, regional feature availability, licence, connectors, export scope, hardware, inputs, human time and exclusions. Inaccessible vendor data is Unassessed, not absent. Non-comparable capabilities are declared, not scored as a competitor failure. Marketing statements are not measured performance.

### 2026 product capability check — 8 November 2026

The matched baseline must account for capabilities already announced or released:

- **Forma Building Design (4 November 2026):** Indoor Daylight is in public beta and reports daylight access during occupied hours over a typical year; parking layout includes accessible/EV markings and access aisles; furniture is available in plan/3D; DXF import and DWG export are supported, with Revit handoff in the connected workflow. This is a real early-design baseline, not evidence that the underlying weather is locally representative or that a design meets all professional daylight/energy criteria.
- **Forma Site Design / Street Design:** Autodesk's 14 October 2026 update describes stronger site-to-detailed-design connectivity and Street Design as still in development. Do not count a pre-release capability as a generally available production feature.
- **TestFit Solve Filters (17 September 2026 documentation):** filters include units, parking ratio, NRSF, efficiency, FAR, height, dwelling units/acre, stalls, critical power, parcels, cut/fill and earthwork cost. Zoning values can be populated where local zoning data is available through its provider. Availability and completeness of local South African zoning data must be separately assessed; do not infer it from the US-centric product example.
- **Bentley iTwin:** its platform already combines engineering data from diverse design tools with reality data and supports change visibility and infrastructure lifecycle workflows. Luminous should test cross-owner decision reliance and exact evidence requalification, not claim to invent engineering/reality integration.

The comparison target is therefore not "more generated designs" or "first daylight analysis." It is whether a federated Luminous workflow can preserve the decision's exact source/geometry/solver/product/lot/fabrication/as-built/maintenance evidence and correctly determine what must be revalidated after change. Compare with the same user brief, local data availability, licences, feature access, output contract, human time and specialist analysis budget.

References:
- Forma Building Design update, 4 November 2026: https://blogs.autodesk.com/forma/2026/11/04/whats-new-in-forma-building-design-indoor-daylight-parking-furniture-and-autocad-workflows/
- Autodesk Forma AU 2026 update, 14 October 2026: https://blogs.autodesk.com/forma/2026/10/14/what-autodesk-forma-announced-au-2026/
- TestFit Solve Filters, 17 September 2026: https://support.testfit.io/knowledge/generative-design-solve-filters
- Bentley iTwin platform: https://developer.bentley.com/itwinplatform/

## 8. Build / bridge / buy

- **Build:** owner-neutral composition, exact scope/time binding, source continuity, independent replay, typed Unknown/NotComparable/Unsupported and selective invalidation.
- **Bridge:** IFC/IDS, supplier product/BOM, SCITT/VC-style proofs, EnergyPlus and specialist solvers, reality capture, CMMS/EAM/ERP and robot accepted-output records.
- **Buy/use:** mature geometry authoring, energy/structural/fire solvers, BIM authoring, metrology instruments and robotics hardware where established tools outperform a replacement.
- **Do not build:** duplicate CAD/BIM, physics solvers, national building code, ERP/CMMS, global identity authority or a central copy of every owner's data.

The goal is not to maximize integration count. It is to compose the smallest sufficient evidence set for a declared decision and preserve what remains unknown.

## 9. Next work

1. Review the federation contract with owners #501/#509/#606/#623/#637 and Mycelix #5149.
2. Add the 18-case corpus to existing source/scope/decision-reliance infrastructure, not a new trust engine.
3. Complete one exact panel lifecycle demonstration under #622.
4. Use #646 only to assess proposed repair-plan feasibility against owner receipts; never infer a repair date.
5. Bind qualified local weather and EnergyPlus 26.2.0 outputs only after source/adapter qualification; preserve TMYx rights and representativeness as Unknown until resolved.
6. Run the matched workflow pilot under #617 and publish inaccessible, non-comparable and failed cases.
7. Promote beyond E1 only with rights-cleared real source data, independent technical review, accountable professionals and a prospective physical demonstrator.

## 10. References

- ISO 23247-6:2026, July 2026: https://www.iso.org/standard/87426.html
- NIST IR 8536, final 9 September 2026: https://csrc.nist.gov/pubs/ir/8536/final
- NIST robotic-systems performance assessment: https://www.nist.gov/programs-projects/performance-assessment-framework-robotic-systems
- buildingSMART IFC register: https://technical.buildingsmart.org/standards/ifc/ifc-schema-specifications/
- buildingSMART IDS: https://www.buildingsmart.org/standards/bsi-standards/information-delivery-specification-ids/
- EnergyPlus 26.2.0 release record, 30 September 2026: https://energyplus.net/epupdate.htm
- Xu et al. (2026), 375-study construction-robotics review: https://doi.org/10.1016/j.rineng.2026.110406
- 2026 review of 129 on-site construction-robotics studies: https://www.sciencedirect.com/science/article/pii/S0926580526005339

**Final claim boundary:** This addendum defines a federation architecture and synthetic benchmark proposal only. It does not establish ISO 23247-6 conformance, product superiority, correct building geometry, thermal/structural adequacy, legal approval, real material provenance, safe robot operation, repair feasibility or restored service.
