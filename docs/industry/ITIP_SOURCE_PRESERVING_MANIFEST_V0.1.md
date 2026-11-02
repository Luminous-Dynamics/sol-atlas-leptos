# ITIP source-preserving interface manifest v0.1

**Status:** Synthetic contract and semantic-validator implementation tranche. This is not an ISO/IEC/OPC conformity assessment and does not establish production interoperability.  
**Parent:** [IND-INTEROP-001 / #341](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/341)  
**Implementation issue:** [ITIP-CONFORMANCE-001 / #354](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/354)  
**Architecture RFC:** [ITIP v0.1 / PR #342](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/342)

## 1. Purpose

This manifest is the evidence envelope at the boundary between an independently owned industrial twin and a Sol-Atlas composition. It does not replace a source protocol, adapter, historian, capability ontology, enterprise graph or IEAC qualification profile.

The manifest records enough information to answer:

- Which system, twin, owner, model, native schema and document kind produced this input?
- Which immutable source snapshot was retrieved, under what rights basis and at what time?
- What did the source mean by observation time, ingestion time, clock domain and sequence?
- Which participant and explicit relationship connect each composition member?
- Which transformations were applied, by whom reviewed, and what semantic loss was declared?
- Which principal has source read, write and actuation scopes?
- Which bounded D0/D1 decision is being considered, and which IEAC profile should evaluate it?

A source locator is not source authenticity. A digest establishes byte identity, not publisher identity, data rights, measurement truth or physical validity.

## 2. Composition modes

The root composition mode is explicit and is never inferred:

- integrated: a declared shared schema is used for the bounded composition. The mapping remains explicit and loss-aware.
- unified: participants retain their own schemas and native models while a typed cross-system view references them.
- federated: participants retain their own source ownership and authority; membership and relations are recorded without promoting one member into a global source of truth.

The validator requires the declared member set to close exactly over the participant IDs. Federated fixtures require at least two distinct owner IDs. This is a local profile invariant, not a claim that an implementation conforms to ISO 23247-6.

## 3. Evidence-preservation invariants

### Identity and source snapshots

Every participant has its own participant, system, twin, owner and native-model IDs. Native model and schema artifacts carry SHA-256 digests. Every source snapshot binds a URI, immutable revision, source-byte digest, retrieval timestamp, rights state and license/basis statement.

The exact payload may be retained elsewhere under the source system's data-rights policy; this manifest binds its locator and digest rather than copying or rewriting it. A production adapter must preserve raw payloads or provide a legally and technically stable immutable reference.

### Time and sequence

event_time_field and ingestion_time_field are distinct fields. clock_domain and timezone are explicit. If a source declares monotonic sequencing, the sequence field is mandatory. timestamp_precision=unknown is indeterminate for this v0.1 admitted path.

Ingestion time must never overwrite source event time. Clock correction, time conversion and late-event handling are transformations, not silent parsing conveniences.

### Relationships and mappings

A relationship is only exact_native_id or typed_reference; fuzzy joins are not accepted in a qualified path. Each edge identifies both participating fields and an identity namespace.

Every derived mapping binds a source participant/field, target concept, source and target units, transform ID/version/digest, loss class and human-review state. A changed unit requires an explicit non-identity transform. Lossy mappings require human review. Opaque mappings require a written loss description.

Unknown source extensions must remain in the original payload or be explicitly represented as loss. The interface contract does not permit silent field dropping.

### Rights and action authority

verified and not_applicable (for repository-owned synthetic fixtures) are the only rights states accepted in this v0.1 synthetic path. Restricted or unknown rights are not promoted into qualified use.

Read, write and actuation scopes are distinct typed strings (:read, :write, :actuate) with no wildcard scopes. A data-read permission does not imply write or actuation permission. This contract only accepts D0 descriptive and D1 advisory decision tiers; D0 is limited to view/annotate, D1 requires human approval, and actuation remains disabled. The actual source authorization system remains authoritative.

The qualification profile reference is a pointer for subsequent IEAC evaluation, not an attestation or delegated permission.

## 4. Files and execution

- Schema: schemas/industry-twin-interface-manifest-v0.1.schema.json
- Positive unified fixture: schemas/examples/industry-twin-interface-unified-synthetic-v0.1.json
- Positive federated fixture: schemas/examples/industry-twin-interface-federated-synthetic-v0.1.json
- Frozen negative fixtures: schemas/examples/industry-twin-interface-negative-*.json
- Rust validator: tools/industry-twin-interface-validator/

Run the semantic validator from the repository root:

~~~bash
cargo run --release \
  --manifest-path tools/industry-twin-interface-validator/Cargo.toml \
  --locked -- \
  schemas/examples/industry-twin-interface-unified-synthetic-v0.1.json \
  /tmp/itip-validation-report.json
~~~

The report binds the exact manifest bytes, exact schema bytes and validator version. Its status is INTERFACE_CONTRACT_VALID_NOT_STANDARD_CONFORMANT for a locally valid manifest. A semantic failure has stable issue codes and a non-zero exit code. The optional report path uses create-new semantics and refuses to overwrite an existing report.

The Rust validator checks cross-object relationships that JSON Schema cannot conveniently establish by itself: participant/member closure, duplicate identities, federation owner distribution, relationship endpoint references, mapping source references, unit-transform consistency, source-time field separation, sequence semantics, rights state and action-scope separation. CI runs the pinned Draft 2020-12 schema validator and the Rust validator over the same positive and negative fixture set.

## 5. Frozen v0.1 negative families

| Family | Schema result | Semantic result expected |
|---|---|---|
| Unknown root field | Reject | Reject |
| Missing source digest | Reject | Reject |
| Fuzzy join enum | Reject | Reject |
| Dangling relationship endpoint | Valid shape | Reject |
| Event-time/ingestion-time alias | Valid shape | Reject |
| Unit change with identity transform | Valid shape | Reject |
| Duplicate participant ID | Valid shape | Reject |
| Unknown/restricted source rights | Valid shape | Reject |
| Missing sequence field for ordered stream | Schema rejects required field or semantic validator rejects |
| D0 advisory action | Schema-valid enum combination | Reject |

Schema validity and semantic validity are separate results. A shape-valid manifest may still be semantically inadmissible. The runtime IEAC gate remains a separate per-decision evaluation.

## 6. Standards boundary

This profile is architecturally informed by the published scopes of ISO/TS 25271:2026 (industrial digital-twin interface), ISO 23247-5:2026 (digital thread), ISO 23247-6:2026 (composition), IEC 63278-1/-4 (AAS) and OPC UA/UAFX. The public descriptions do not provide the full normative clauses needed for conformity assessment.

The profile does not claim standards conformity, interoperability with a named vendor, correct physical twin behavior, safety, data authenticity, regulatory acceptance, operational authorization or SOTA. Full normative-text review, clause mapping and independently sourced conformance fixtures are prerequisites to stronger claims.

## 7. Next experiments

1. Add source-specific fixtures from independently maintained public specifications or authorized vendor systems.
2. Add deterministic event-stream mutation tests for duplicate, delayed, missing, reordered and replayed events.
3. Add clock drift, clock reset, timezone ambiguity and uncertain clock mapping.
4. Add schema evolution and extension-preservation round trips.
5. Add separate source credential, authorization-revocation and federated-member-unavailable cases.
6. Add cross-language golden vectors and a separately implemented verifier.
7. Only then conduct a matched cross-vendor pilot, measuring source-to-admission latency, semantic-loss rate, mapping-maintenance time, resource use and operator review effort.

The intended claim is narrow: this v0.1 contract makes source identity, temporal semantics, mapping loss, rights state and action-scope boundaries explicit enough to test. It does not prove that any real industrial source satisfies them.
