# Industrial Twin Interoperability Profile (ITIP) v0.1

**Status:** Architecture/RFC draft. No standard conformity, vendor interoperability or production integration is claimed.  
**Issue:** [IND-INTEROP-001 / #341](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/341)  
**Parent assurance framework:** [IEAC v0.1](INDUSTRY_ENGINEERING_ASSURANCE_CONTRACT_V0.1.md)  
**Benchmark execution contract:** [IBAP run manifest v0.1](INDUSTRIAL_BENCHMARK_RUN_MANIFEST_V0.1.md)

## 1. Executive decision

Luminous should not build a competing PLM, MES, SCADA, historian, enterprise ontology, CAD/BIM/CAE suite, 3D scene editor or universal industrial protocol.

The product opportunity is an **assurance and decision layer over heterogeneous industrial twins**. It should connect existing systems, preserve their source semantics and identity, qualify how their data/models may be used for a specific decision, and independently verify outcomes.

The initial composition preference is:
- **Unified** when systems retain native models/schemas while exposing an explicit cross-system view.
- **Federated** when peers coordinate while retaining local ownership and authority.
- **Integrated** only where a shared schema and complete, reviewed mapping are appropriate and loss-aware.

The terms align with the published ISO 23247-6:2026 composition modes. The exact implementation profile and applicable normative clauses still require a standards gap analysis.

## 2. Current standards landscape (31 October 2026)

| Reference | Verified status | Relevance to ITIP |
|---|---|---|
| ISO/TS 25271:2026 | Published, August 2026 | Interface architecture among digital twin, physical twin and their interface |
| ISO 23247-5:2026 | Published, June 2026 | Digital-thread links across manufacturing lifecycle stages |
| ISO 23247-6:2026 | Published, July 2026 | Integrated, unified and federated twin composition |
| ISO/AWI 23247-7 | Approved work item; under development | Commissioning of manufacturing digital twins |
| ISO/AWI 23247-8 | Approved work item; under development | Lifecycle verification, validation and uncertainty quantification |
| IEC 63278-1:2023 | Published | Asset Administration Shell structure |
| IEC 63278-4:2026 | Published, 17 September 2026 | Application of the Asset Administration Shell |
| OPC UA / UAFX | Published protocol family and evolving roadmap | Industrial information exchange, discovery, device onboarding and secure communication |
| MTConnect / QIF | Version- and document-kind-specific | Machine observations, quality plans and inspection results |

Official references:
- ISO/TS 25271:2026 — https://www.iso.org/standard/89689.html
- ISO 23247-5:2026 — https://committee.iso.org/standard/87425.html
- ISO 23247-6:2026 — https://www.iso.org/standard/87426.html
- ISO/AWI 23247-7 — https://www.iso.org/standard/94859.html
- ISO/AWI 23247-8 — https://committee.iso.org/standard/94858.html
- IEC 63278-1:2023 — https://webstore.iec.ch/en/publication/65628
- IEC 63278-4:2026 — https://webstore.iec.ch/en/publication/83593
- OPC UA roadmap — https://opcfoundation.org/about/opc-technologies/opc-ua/opcua-roadmap/

**Important boundary:** the published abstracts establish scope and status, not conformance. ITIP must not claim ISO/IEC conformity until the full normative texts have been reviewed, applicable clauses mapped, and independent conformance tests performed.

## 3. Competitive platform boundary

Published product material shows that incumbent platforms already have strong capabilities in infrastructure integration, visualization, twin modeling and workflow orchestration:

| Platform | Publicly described strengths | Luminous posture |
|---|---|---|
| Siemens Xcelerator / Digital Twin Composer | Industrial twin, simulation/Omniverse and live engineering data composition | Consume engineering/twin outputs and attach assurance evidence; do not reproduce high-fidelity scene authoring |
| AWS IoT TwinMaker | Knowledge graph, connectors, sensor/camera/enterprise integration and scene composition | Use as one operational twin provider/connector surface |
| Bentley iTwin | Infrastructure engineering/reality/IoT data, APIs and lifecycle applications | Consume infrastructure asset and observation references; preserve geospatial semantics |
| Palantir Foundry | Enterprise objects, relations, logic, actions, security, model objectives and process orchestration | Integrate with existing ontology/actions; specialize in physical validity, evidence ceilings and independent assurance |
| Dassault 3DEXPERIENCE | Enterprise knowledge, virtual-twin experiences and lifecycle process orchestration | Consume design/process identities and model versions |
| ThingWorx under Velotic | Industrial connectivity, applications, analytics, operations and quality workflows | Integrate with existing industrial telemetry and operational systems |

References:
- Siemens — https://press.siemens.com/global/en/pressrelease/siemens-unveils-technologies-accelerate-industrial-ai-revolution-ces-2026
- AWS IoT TwinMaker — https://docs.aws.amazon.com/iot-twinmaker/latest/guide/what-is-twinmaker.html
- Bentley iTwin — https://developer.bentley.com/itwinplatform/
- Palantir Foundry — https://www.palantir.com/docs/foundry/platform-overview/
- Dassault 3DEXPERIENCE — https://www.3ds.com/products/3dexperience
- ThingWorx — https://www.velotic.com/products/thingworx

These are public product descriptions, not a controlled technical bake-off. They do not prove that these vendors lack VVUQ, provenance or safety controls. Luminous must benchmark actual supported workflows before making comparative superiority claims.

## 4. Assurance plane over heterogeneous twins

The interface boundary must preserve two things independently:

1. The external system's own source object, schema, measurements, model and operational authority.
2. Luminous' decision-specific interpretation, evidence, qualification and claim ceiling.

An adapter must never overwrite the source record with a normalized value. Normalization creates a derived value alongside the original value and binds the transformation version.

Every admitted interface snapshot must bind:
- external system, twin, asset/object and source-document IDs;
- native schema/version, document format and document kind;
- parser/validator and adapter identity/version;
- source locator, exact bytes digest or immutable upstream revision;
- source, observation, ingestion and validity times as separate fields;
- stream/event sequence and source clock domain, where applicable;
- original unit/value and any derived normalized value;
- transformation ID/version, mapping loss report and output digest;
- rights, retention, access scope and authentication principal;
- declared composition mode and applicable interface/claim profile;
- IEAC profile ID, model/evaluator/configuration identities and decision tier.

A source URL is only a locator. A hash proves byte identity, not publisher authenticity, data rights, measurement truth or physical validity.

## 5. Composition contract

### 5.1 Integrated

Participants share a common schema for the declared bounded domain. Admission requires reviewed, loss-aware mappings, source identity preservation and round-trip fixtures. Unknown extensions must be retained or explicitly reported; they cannot disappear silently.

### 5.2 Unified

Each participant keeps its own schema and native model. A cross-system view exposes typed references, mappings and lineage. This is the preferred mode when a common schema would erase domain semantics.

### 5.3 Federated

Participants coordinate through explicit requests and replies while keeping local source ownership and authority. A federation result records the member set, each member's profile/version and the exact evidence snapshots that contributed.

A combined result must retain missing and conflicted participant state. A federated aggregate must not promote a weak member's evidence to a strong global claim.

### 5.4 Default rule

Choose the least-transformative composition mode that can answer the declared decision. Do not integrate schemas for convenience when unified/federated access better preserves source semantics.

## 6. Adapter responsibilities

The adapter pipeline is separated into layers:

1. **Acquisition / reference:** connect to an authorized source or resolve an immutable external artifact.
2. **Format admission:** check document-kind/version/schema and validator identity.
3. **Parsing:** retain native IDs, source locators, timestamps, units, sequences and extensions.
4. **Mapping:** produce typed derived values with explicit transforms and unsupported-field diagnostics.
5. **Composition:** connect models/objects through explicit IDs and typed relations; no fuzzy qualified joins.
6. **Evidence binding:** attach raw digests, rights, observation time, calibration and lineage references where provided.
7. **Runtime admission:** check current snapshot against the exact IEAC profile, identity, validity envelope, freshness and authority.
8. **Decision / execution:** route through the existing human and source-system authorization path; a successful data write is not proof of physical execution.
9. **Reconciliation:** link inspection/telemetry/outcomes to the authorized request and update evidence/qualification as a separate decision.

### Existing owners to reuse

| Concern | Existing owner |
|---|---|
| Capability and instance semantics | Sol-Atlas Capability Atlas PR #18 |
| Cross-repository artifact links | Sol-Atlas artifact-chain PR #273 |
| Asset/geometry and geospatial source semantics | Sol-Atlas ATLAS-GEO-001 #65 and current asset-catalog work |
| Manufacturing MTConnect/QIF mapping | Symtropy MFG-INTEROP-001 #3025 / PR #3030 |
| MTConnect/QIF schema-source admission | Symtropy MFG-INTEROP-001B #3033 |
| Evidence independence | Mycelix evidence-independence profile PR #5526 |
| Qualification vector and decision tiers | IEAC PR #331 |
| Static schema/validator parity | Sol-Atlas #335 |
| Current operating-envelope admission | Sol-Atlas #336 |
| Frozen benchmark comparison | IBAP run manifest PR #339 |

ITIP does not add another capability ontology, process registry, sensor registry, provenance graph, evidence-independence algorithm or domain physics solver.

## 7. Required interface failure semantics

The following states must remain distinct:
- `valid_and_admitted`
- `unsupported_version`
- `unsupported_document_kind`
- `invalid_source`
- `missing_or_unverified_rights`
- `digest_mismatch`
- `unknown_or_conflicting_mapping`
- `stale_or_future_observation`
- `sequence_gap_or_replay`
- `clock_mapping_indeterminate`
- `unit_mapping_unsupported`
- `source_revoked`
- `access_denied`
- `temporarily_unavailable`

A failure in one source must not be silently replaced with an assumed or stale value. For a mandatory condition, the composite decision must abstain/reject or use a separately validated fallback with explicit provenance.

## 8. Benchmark programme

### Standards and format admission
- Pin source release, standard edition, schema URI, document kind, format, imports closure, validator identity and exact schema-bundle digest.
- Positive and negative examples must originate independently of the adapter author where possible.
- Report conformance by version and document kind; an XML result cannot qualify JSON, and a Streams schema cannot qualify Assets.
- Unsupported formats and versions fail closed.

### Semantic preservation
- 100% preservation of native identifiers, source locators, source timestamps/sequences, schema versions and original units for admitted fields.
- Zero silent loss of unknown extensions or unsupported fields.
- Zero incorrect asset/part/lot/operation/characteristic joins in the frozen negative corpus.
- Every transform binds source digest, mapping version and derived output digest.

### Temporal and stream integrity
- Exercise duplicate, missing, out-of-order, delayed, replayed and late-arriving events.
- Exercise clock offsets, drift, timezone ambiguity, resets and uncertain source mappings.
- Preserve source event-time separately from ingestion/processing time.
- Never manufacture missing samples or regularize irregular telemetry without an explicit, versioned transform.

### Security and authority
- Test read and write permission separately; read access never implies actuation authority.
- Exercise expired/revoked credentials, source authorization drift, access denials and partial connectivity.
- Preserve external authorization scope and Mycelix action identity; ITIP cannot mint new source permissions.
- An untrusted or mismatched action receipt cannot qualify an intervention as physically complete.

### Performance and operations
- End-to-end source-to-admission latency, not just connector throughput.
- p50/p95/p99 ingest, parse, map, query, admission and reconciliation latency.
- CPU, memory, storage, bandwidth and compute energy under matched conditions.
- Backlog, reconnect, duplicate suppression and event-gap recovery.
- Human review and mapping-maintenance time per source/model/version.

## 9. Proposed internal acceptance gates

These are project targets, not industry standards or achieved results:
- 100% exact identity and digest preservation on admitted fixtures.
- Zero silent schema/document-kind fallback.
- Zero false qualified joins in the declared negative corpus.
- 100% unknown/unsupported mapping loss is recorded.
- 100% stale, revoked, conflicted or unauthorized mandatory conditions abstain/reject as specified.
- Deterministic replay of frozen source snapshots and mapping versions where deterministic behavior is expected.
- All composition results declare integrated, unified or federated mode.
- 100% of qualified decisions bind the exact interface snapshot, IEAC profile and requested action.
- No real-world standards, safety, SLA, physical-validity or production claim from synthetic adapter fixtures alone.

## 10. Implementation sequence

**P0 — standards gap and owner map**
- Review full normative texts for ISO/TS 25271, ISO 23247-5/-6, IEC 63278-1/-4 and selected OPC UA profiles.
- Map each applicable clause to an existing owner or an explicit gap; do not implement based on public abstracts alone.

**P1 — source-preserving interface fixtures**
- One manufacturing route uses Symtropy's MTConnect/QIF adapter.
- One infrastructure route uses a Sol-Atlas geospatial/service asset plus a documented external source snapshot.
- Both preserve exact native identifiers and source digests.

**P2 — composition and admission**
- Produce one unified composition example and one federated composition example.
- Bind both to an IEAC profile and exercise the runtime admission gate.

**P3 — independent verification**
- Independent source-preservation/semantic checker.
- Temporal, rights, schema drift, unit and authority negative controls.
- Exact-head reproducibility and a detached source/composition manifest digest.

**P4 — cross-vendor pilot**
- Use an authorized customer/vendor twin system or NIST testbed artifacts.
- Retain independent test fixtures and do not claim vendor compatibility from self-authored mock data.

## 11. Claim ceiling

ITIP v0.1 is an architecture profile. It does not implement a connector, establish compliance with ISO/IEC standards, validate a physical model, prove source authenticity, qualify a sensor, grant operational authority or certify safe execution.

The intended positioning is a standards-aware assurance and decision layer that composes existing industrial twins. Luminous should compete on decision admissibility, traceable evidence, independent verification and measured outcomes—not by attempting to replace every incumbent twin, engineering or enterprise-operations platform.
