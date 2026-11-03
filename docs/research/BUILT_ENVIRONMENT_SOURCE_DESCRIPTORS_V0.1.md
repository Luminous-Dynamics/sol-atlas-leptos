# Built-environment source descriptor catalogue v0.1

**Status:** additive sidecar schema and Rust consistency verifier implemented; exact-head formatting, Clippy, unit tests, and schema workflow remain pending.
**Parent:** [#492 — owner-native source descriptors](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/492).
**Composes with:** [#486 projector / PR #491](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/491), [#487 IEAC identity v0.2](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/487), [#489/#490 coverage](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/490), [#347 trust/canonicalization](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/347).

## Why a sidecar instead of changing BuiltEnvironmentCase

The current typed case has important source IDs, revisions, content digests, evidence state, rights, and references. It does not carry all owner-native identity, media type, schema, retrieval, and attestation metadata for each record.

Adding mandatory fields directly to `BuiltEnvironmentCase` would break existing struct construction and would tempt adapters to fill missing values with guessed defaults. The sidecar is additive and case-bound:

`BuiltEnvironmentCase + SourceDescriptorCatalog -> descriptor validation -> projection inputs`

The catalogue is bound to the exact typed case representation digest. Legacy cases remain readable; missing descriptors are reported as incomplete rather than backfilled from filenames, display labels, presumed vendors, or inferred IDs. The case ID binds the catalogue and graph scope but is deliberately excluded from owner-native identity hashing: the same owner namespace and native record/revision tuple must replay to the same node identity when referenced by multiple cases. Case membership remains in the graph scope and case-specific graph digest.

## Descriptor model

Each descriptor is keyed by `(record_kind, case_record_id)` and preserves. The record key locates the typed case entry; it is not substituted for the source-owner identity. Identity hashes bind node type, owner namespace and sorted native identity components, never the case ID:

| Dimension | Required information | Why it is separate |
| --- | --- | --- |
| Owner identity | owner namespace + named native identity components | Prevent equal content from collapsing distinct source records |
| Payload identity | SHA-256, digest basis, byte-retrieval state, length, object reference | Distinguish source bytes from case/projector representations |
| Representation | media type, schema ID/version, schema digest | A payload digest does not describe how to parse or interpret bytes |
| Source snapshot | source ID, snapshot ID/digest, capture/retrieval time | Prevent mixing records from different source states |
| Observation time | observed time and recorded time | Preserve valid-time/system-time distinctions |
| Rights | state, permitted uses, rights evidence reference | Do not infer permission from availability |
| Attestation | statement reference and key identity | Preserve the claim without claiming that it has been authenticated |

The descriptor supports four explicit source payload digest bases:

- `locally_retrieved_and_hashed`: declares that bytes were retrieved, a byte length and content-addressed object reference exist, and a SHA-256 is recorded. The descriptor verifier still does not fetch or re-hash those bytes.
- `source_owner_attested`: a source owner provides a digest, but bytes have not been locally retrieved. A statement reference is expected; the source attestation is not authenticated here.
- `external_reference`: an exact digest is bound to a content-addressed object reference, but the referenced bytes have not been retrieved by this verifier.
- `unavailable`: no digest, bytes, byte length, or content reference is claimed. This remains incomplete.

A content-addressed object reference must bind the exact digest as `cas:sha256:<lowercase-sha256>`. A reference cannot silently point to a different digest.

## Rust verifier semantics

`sol-atlas-core::built_environment_source_descriptor::validate_source_descriptor_catalog` validates the sidecar against one exact `BuiltEnvironmentCase`.

It checks:

- exact case ID and typed-case representation digest;
- a descriptor for every typed record and no descriptor for a nonexistent record;
- duplicate case record keys and duplicate descriptor keys;
- owner namespace, native component names, uniqueness, and required native ID/discriminator;
- v0.2 identity recomputation using the same domain-separated, u64 length-prefixed encoding as #487;
- duplicate native identity and conflicting payload digests;
- descriptor payload digest agreement with any source digest already present in the case;
- payload digest-basis consistency and exact content-addressed object-reference binding;
- schema/media metadata missingness;
- rights-state/permitted-use consistency and supported purpose vocabulary;
- source snapshot digest and capture/retrieval/observation chronology;
- attestation state cannot be promoted to authenticated by this layer.

The result reports the descriptor count, missing/extra descriptor count, identity digests, owner namespace map, deterministic findings, and explicit verification boundaries.

## Three distinct outcomes

- `internally_consistent_for_declared_case`: each typed case record has a coherent sidecar descriptor under this schema and its bindings reconcile.
- `incomplete_or_indeterminate`: at least one descriptor, payload, media/schema field, rights proof, or temporal claim is missing or not yet independently established.
- `invalid_input`: contradictory case/catalog bindings, invalid digest or identity encoding, duplicate native identity, invalid rights escalation, or inconsistent digest basis.

Even an internally consistent catalogue does **not** prove that an owner namespace is authentic, that the source bytes match the declared digest, that the asserted rights are valid, or that the source inventory is complete. The report explicitly returns:

- `payload_byte_verification = not_performed`;
- `rights_authentication = not_performed`;
- `source_attestation_authentication = not_performed`;
- `action_authority = none`.

A source attestation reference and key ID are descriptors, not signatures verified by this module. #347 owns trust-root, signature, sequence/anti-replay, and revocation evaluation. #489/#490 owns source inventory and coverage. #484 owns the independent impact oracle. #397 owns temporal invalidation and current-reliance logic.

## Projection composition

The catalogue's `native_identity_namespaces` output uses the same record keys as the #491 projector:

- `evidence:<id>`
- `artifact:<id>`
- `engineering_receipt:<id>`
- `decision:<id>`

This allows a caller to supply the validated namespace map to the projector without deriving namespaces from display metadata. The map is caller/source-provided and remains unauthenticated until #347 evaluation.

The descriptor catalogue also carries payload media/schema details absent from today's typed case. A follow-on projector integration should bind the catalogue digest and consume those metadata fields only when the catalogue is valid for the exact case representation. If the descriptor is absent or incomplete, the projector must retain its current explicit-loss behavior. It must not silently upgrade the claim ceiling.

## Migration

- Existing cases remain valid inputs.
- Cases without descriptors use adapter-scoped identity only as an explicit fallback and stay incomplete.
- No source owner namespace, schema, media type, license, rights, or attestation is inferred.
- A changed case representation invalidates the catalogue binding; it must be regenerated or independently revalidated.
- Two independent adapters should emit the same IEAC identity envelope when they receive the same owner namespace and native identity components, even if payload representations are different.

## Qualification targets

P0:
- 100% of typed case records either have exactly one descriptor or an explicit missing-descriptor finding;
- zero duplicate native identity accepted;
- zero conflicting digest for the same native identity accepted;
- exact identity digest agreement with an independent implementation;
- zero source/case digest disagreement accepted;
- zero unavailable payloads promoted to digest-present;
- zero unknown-rights records with permitted uses;
- exact case-representation and source-snapshot binding;
- no descriptor validation result grants trust, rights, approval, or action authority.

P1:
- generated catalogues at 10, 100, 1,000, 10,000, and 100,000 descriptors;
- p50/p95/p99 validation latency, peak RSS, allocations, and serialized bytes;
- mutation sweeps for namespace swap, duplicate component, revision substitution, digest rebinding, partial snapshot, expired observation, rights escalation, missing attestation, and changed case representation;
- cross-adapter identity parity on a frozen corpus.

P2:
- rights-cleared, held-out records from at least two source-system families;
- independent source-owner and reviewer validation;
- source adapters for IFC/IDS, QIF/metrology, CAD/BIM, permits, material certificates, and solver receipts only when the relevant source semantics can be represented without coercion.

The objective is not to make every descriptor complete. It is to ensure every unknown remains visible and every identity can be independently reconstructed.
