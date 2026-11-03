# Built-environment to IEAC v0.2 projector

**Status:** implementation, receipt schema, and integration qualification lane are committed on `research/built-environment-ieac-projector-v0-1`; no Rust compiler, Clippy, integration-test, or schema-validation pass is claimed until the exact-head workflow reports it.

**Composition:** [#486 projector request](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/486), [#487 native identity v0.2](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/487), [#489 coverage manifest](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/489), [#490 coverage runtime verifier](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/490).

## Contract boundary

The projector is a read-only adapter from the typed `BuiltEnvironmentCase` model into an IEAC v0.2 graph. It is not a second source graph, coverage engine, impact traversal, trust verifier, engineering solver, approval workflow, or action-authority issuer.

**Known input-model limitation:** `BuiltEnvironmentCase` has source IDs but no distinct owner-native namespace, media type, or source schema fields. The projector therefore accepts a caller-supplied per-record `native_identity_namespaces` map in `ProjectionContext`. These namespace values are included in identity hashing and bound by a deterministic map digest in the receipt, but the projector cannot authenticate who supplied them. Missing entries use an adapter-scoped fallback and emit `NATIVE_IDENTITY_NAMESPACE_NOT_SUPPLIED`; blank, padded, or overlong values are invalid. Even when namespaces are supplied, ordinary evidence/artifact cases remain `projected_with_known_loss` because their source media types and schema identities are not represented by the current case model. [#492](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/492) proposes a durable descriptor sidecar.

Its disposition vocabulary is deliberately distinct from coverage:

- `projected_without_known_loss`: every record and relationship represented by the typed case was mapped under this profile, without a detected projection loss.
- `projected_with_known_loss`: one or more records/relationships were unavailable, unresolved, unsupported, or not representable.
- `invalid_input`: the input or projected graph contains a contradiction or structural violation.

Even `projected_without_known_loss` is only an assertion about the provided typed case representation. It does **not** assert that the case contains all relevant source systems, that the source records are authentic, or that the selected profile is adequate for an engineering decision. It never authorizes `NoImpact`, release, construction, or actuation.

## Mapping table

| BuiltEnvironmentCase source | IEAC v0.2 node | Payload digest basis | Relationships |
| --- | --- | --- | --- |
| `EvidenceRecord` | `source_evidence` | Source `source_payload_sha256`, preserved byte-for-byte | Evidence-to-artifact, evidence-to-receipt, authority-to-decision |
| `ArtifactRecord` | `engineering_artifact` | Source `content_sha256` | Parent-to-child `derived_from`; evidence-to-artifact; artifact-to-receipt `verified_by`; artifact-to-decision |
| `EngineeringReceipt` | `engineering_receipt` | SHA-256 of the exact typed Rust struct serialization; explicitly not a source-native payload digest | Artifact-to-receipt `verified_by`; assumption evidence-to-receipt |
| `DecisionRecord` | `decision` | SHA-256 of the exact typed Rust struct serialization; explicitly not a source-native payload digest | Artifact target, receipt, authority evidence, and superseded-decision relationships |

Native identity uses the same v0.2 domain separator and length-prefixed, sorted identity components as #487. Identity and content digests remain distinct. Every identity contains the case and source identifiers plus a record/revision discriminator. The identity namespace comes from the caller-supplied per-record map when present; missing keys use the adapter-scoped fallback. The receipt binds the complete map digest and records `source_owner_namespace_authenticated=false`. This preserves the owner-provided namespace value without claiming to authenticate it.

For evidence records, the identity includes source ID, evidence ID, subject, field path and jurisdiction. Artifact identities include design ID, artifact ID, revision, family and jurisdiction. Receipt and decision identities include design ID and their native receipt/event ID.

## Loss and uncertainty accounting

The projection receipt is itself specified by [`built-environment-ieac-projection-receipt-v0.1.schema.json`](../../schemas/built-environment-ieac-projection-receipt-v0.1.schema.json). The exact-head workflow validates both the generated IEAC graph and this receipt schema. The receipt's self-digest is computed over the complete receipt body with only its own digest field omitted; the raw typed case digest and namespace-map digest are separately retained.

The projection receipt preserves:

- exact typed case representation digest and serialization profile;
- node identity mappings and their native record keys;
- edge mappings and native relationship keys;
- all typed evidence-record representations, including records that cannot become graph nodes;
- source evidence state, data-rights state, licence, subject, field path, jurisdiction, and observation time;
- source payload digests separately from typed case-record representation digests;
- exact typed receipt and decision serialization bytes where the source model has no source-payload digest;
- every unresolved or digest-mismatched relationship as an explicit unmapped item;
- projector version, caller-supplied build digest, mapping-profile digest, graph digest and deterministic findings.

A missing evidence source digest does not get replaced with a digest of a display object or projected JSON. The evidence record is preserved in the receipt, but no source-payload-addressed graph node is created; all relationships depending on that node remain unmapped.

The original source payload bytes are not present in the current `BuiltEnvironmentCase` model. The model lacks first-class owner namespace, media type, and source schema fields. Caller-provided namespaces partially close the identity gap; unauthenticated provenance is explicit, and media type/schema absence is reported as `SOURCE_EVIDENCE_MEDIA_TYPE_SCHEMA_NOT_MODELED` or `ENGINEERING_ARTIFACT_MEDIA_TYPE_SCHEMA_NOT_MODELED`. A digest is therefore a reference to source bytes, not proof that the projector fetched or independently hashed those bytes. Likewise, artifact media types and source payload schema identifiers are not present in the model and remain explicitly unknown.

## Structural invariants

- Node IDs use IEAC v0.2 native-identity IDs, not payload-derived IDs.
- Graph snapshot digest is computed over the serialized graph core with `graph_snapshot_sha256` excluded, matching the #487 runtime rule.
- Nodes, edges, and mapping receipts are sorted by deterministic keys.
- The projection receipt binds the exact typed input serialization. Reordering source vectors may leave the graph digest unchanged but changes the input and receipt digests; the two claims are intentionally distinct.
- Parent, input artifact, target artifact, evidence, assumption, authority and supersession relationships are mapped only when native IDs and required content-digest bindings resolve.
- Duplicate graph endpoint pairs are rejected rather than coalesced into one edge.
- Directed dependency cycles are rejected rather than repaired or silently ignored.
- A case with no source-addressable nodes returns a receipt with no graph; it does not emit an empty graph that violates the IEAC schema.
- All projected nodes retain `verification.result=unknown`, `trusted_at=null`, and the graph has `action_authority=none`.

## Qualification tests

The Rust core tests and root integration tests cover:

- identical payload hashes with distinct native evidence identities;
- exact preservation of evidence status, rights, licence and observation metadata;
- evidence with missing source payload digest remains in receipt but is not projected as an authenticated evidence node;
- parent, solver-input, target, evidence, receipt, authority and supersession mapping;
- mismatched native reference digests remain unmapped;
- duplicate endpoint pair rejection;
- dependency cycle rejection;
- missing decision-authority evidence is explicit and incomplete;
- empty/no-node case does not produce an empty graph;
- deterministic graph output under vector reordering while exact-input receipt digests differ;
- exact Unix timestamp conversion at the epoch and leap-day boundary.

The exact-head workflow compiles and lints the root package with locked dependencies, runs public API integration tests, emits an actual graph from a synthetic case, and validates that emitted graph against the IEAC v0.2 Draft 2020-12 schema. No test result is claimed until that job reports against the exact source SHA.

## Remaining gates

The projector cannot authenticate:

1. source owner identity, source signatures, source snapshot completeness, or source payload bytes;
2. reviewer identity, reviewer competence, or independence;
3. whether the supplied coverage profile is adequate for the actual engineering question;
4. whether dependencies absent from the typed case should have existed;
5. the validity of engineering solver conclusions, professional approvals, or legal authority.

These stay in their owning tracks: #347 trust roots and signed payloads, #489/#490 source inventory and coverage, #484 gold inventory and impact oracle, and #397 bitemporal lifecycle invalidation. A future composed assurance result must bind each receipt's exact input digest, scope, verifier build, trust bundle and freshness. It must not infer an overall pass from the projector alone.

## Next benchmark tranche

Use paired generated cases over 10, 100, 1,000, 10,000 and 100,000 records. Mutations include identity-component substitution, equal-content identity collision, omitted source digest, missing/duplicate native references, edge digest rebinding, stale observation, jurisdiction mismatch, cycles, duplicate endpoint pairs, reordered vectors, and source-schema/media-type absence.

Report separate denominators for:
- identity preservation and false identity merges;
- source-payload digest preservation;
- record/relationship mapping recall and precision against a frozen gold mapping;
- correct loss classification and false completeness;
- graph schema validity and DAG validity;
- replay equality and input/graph/receipt digest agreement;
- latency, peak memory and output size.

The key P0 target is **zero false claims of lossless projection**. A system that reports loss when it could have preserved a record is inefficient; a system that fails to report a missing or altered engineering relationship is unsafe. Keep those errors separate rather than trading them against speed in a single score.
