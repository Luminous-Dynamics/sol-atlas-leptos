# Projection semantic replay

## Purpose

A temporal projection is not fully reproducible merely because its historical
event time, evidence frontier, and qualification are preserved. The external
semantic vocabulary used to render that projection can also change.

Sol Atlas therefore treats ontology interoperability as a fourth replay
dimension alongside:

1. historical/event time,
2. evidence/source availability,
3. assessment/interpretation availability,
4. semantic vocabulary context.

The new `OntologyMappingContextV1` is content-addressed over the external
standard, pinned version, release status, external term, mapping kind,
mapping relation, and preserved qualification. The mapping relation is
explicitly separate from class/property/individual kind.

## Replay theorem

> A projection is semantically reproducible only when the ontology mapping
> context used to render it is pinned and integrity-checked.

Recording only "CIDOC CRM 7.4" is insufficient: a term, relation, release
status, or mapping interpretation can change while the version string remains
unchanged.

The `ProjectionSemanticEnvelopeV1` binds one or more content-addressed
mapping contexts to:

- projection identity,
- canonical claim reference,
- evidence frontier,
- qualification.

The envelope hash is deterministic. Mapping order is canonicalized for
serialization/replay only; it carries no epistemic priority.

## Compatibility boundary

The external ontology remains external. Sol Atlas does not become an ontology
authority, and the semantic envelope does not upgrade a claim's qualification.

As of October 2026, CIDOC CRM 7.4 was released in August 2026 and is listed by
the official CIDOC CRM registry as **Draft**. Draft status therefore remains
visible in the mapping context rather than being silently treated as an
implementation-stable release:
https://cidoc-crm.org/versions-of-the-cidoc-crm

CRMinf 1.2.1 was released in April 2026 and is listed by the official CRMinf
registry as **Stable**:
https://cidoc-crm.org/crminf/fm_releases

PROV-O remains the provenance interchange boundary. The W3C PROV family defines
validity/consistency constraints and normalization/equivalence for provenance,
while leaving syntax-sensitive cryptographic identity to the application
boundary:
https://www.w3.org/TR/prov-constraints/

## Strict replay boundary

Migration-compatible frontier validation may read legacy manifests whose
content-addressed hash is absent, preserving existing serialized fixtures and
older records. That compatibility mode is not sufficient for reproducible
historical replay.

Strict replay therefore requires all of the following:

- a non-empty self-consistent frontier manifest hash;
- complete temporal metadata for every admitted source snapshot;
- source IDs in metadata to match the admitted source set exactly;
- every evidence metadata record to reference an admitted source;
- every ancestor frontier in the selected chain to satisfy the same strict
  requirements, not merely the selected leaf.

This prevents a valid modern leaf from laundering a legacy or incomplete
ancestor into a reproducible historical chain.

The V1 frontier hash uses an application-defined canonical payload serialized
with Rust's serde_json; it is deterministic within this contract but is not
claimed to be RFC 8785 JSON Canonicalization Scheme (JCS) wire-compatible.
RFC 8785 exists specifically to define an invariant JSON representation for
cross-implementation hashing/signing. A future interoperability-facing
manifest format may adopt a standardized canonical serialization, but changing
the current V1 hash contract would require an explicit versioned migration.

## Invariants

- ontology mapping != canonical claim
- mapping relation != epistemic qualification
- ontology version != truth
- draft/stable/official release status != truth
- mapping order != epistemic ranking
- semantic vocabulary drift must be detectable
- changing a mapping term, relation, version, release status, or qualification
  invalidates the mapping context digest
- the projection envelope preserves the projection's qualification rather than
  recomputing or upgrading it

## Projection identity boundary

A projection identifier is not, by itself, a commitment to every semantic field of the originating projection. Legacy V1 audit/admission schemas intentionally omit some projection fields, so their reciprocal validators can only reconstruct the semantics those schemas represent.

For stronger V2+ provenance, Sol Atlas carries an optional SHA-256 projection-semantic commitment derived from the complete `CulturalProjectionV1` value. Strong constructors populate the commitment, and reciprocal validation recomputes it against the originating projection. Legacy empty values remain readable for migration. The commitment uses an application-defined serde_json payload and is not claimed to be RFC 8785/JCS-compatible.

The current projection semantic canonicalization is intentionally narrow and explicit: primary `evidence_refs` and `source_snapshots` are treated as set-like membership and sorted before hashing, matching the replay receipt contract. Other projection vectors are not reordered by the hash function. Any future change to these equivalence rules should introduce an explicit versioned canonicalization contract rather than silently changing the meaning of existing hashes.

## Construction and validation tiers

The V4 and V5 audit layers now expose two deliberate provenance tiers:

- `from_v2` / `from_v4` remain migration-compatible constructors for already
  serialized or legacy-shaped records.
- `CulturalProjectionAuditV4::from_projection_at(...)` and
  `CulturalProjectionAuditV5::from_projection_at(...)` rebuild the audit chain
  from the originating projection, canonical claim, frontier, and exact
  resolution/argumentation closure. They therefore populate the originating
  projection semantic commitment instead of inheriting an empty legacy field.
- `validate_against_projection(...)` remains migration-compatible, while
  `validate_strong_against_projection(...)` requires both frontier-manifest
  identity and the originating projection semantic commitment to be present
  and independently verified.
- `V5ReplayReceiptV1::from_projection_and_chain(...)` is the strong producer
  path for a current-leaf replay receipt. It refuses to construct a receipt
  unless the audit is already strongly bound to the originating projection.
- `V5HistoricalReplayReceiptV1::from_projection_at(...)` is the corresponding
  strong producer path for an explicitly selected historical prefix. It binds
  the receipt to the selected frontier before constructing or validating the
  content-addressed receipt.
- The compatibility constructors remain available for migration, but new
  provenance-sensitive producers should use the strong constructors rather
  than construct-then-hope-to-validate.

This separation avoids a dangerous compatibility pattern: making legacy data
unreadable just to obtain a stronger security invariant. Older records remain
replayable under the compatibility gate, while new provenance-sensitive call
sites can opt into an explicit strong construction-and-validation path.

The underlying rule is consistent with established provenance practice: a
self-consistent digest establishes content integrity, but provenance validation
also needs the verifier to check that the asserted identity matches the
expected source artifact. W3C PROV-O models qualified derivation by explicitly
citing the source entity and the activity that produced the derived entity;
SLSA likewise treats provenance as verifiable information connecting an output
artifact back to its source. Sol Atlas is not claiming conformance to either
model; these are design analogies for keeping identity and origin verification
distinct.

RFC 8785 similarly requires an invariant representation for cryptographic
operations and deliberately preserves JSON array element order. Sol Atlas
therefore changes only the vectors whose domain semantics explicitly say they
are sets; it does not silently sort arbitrary projection vectors.

## Remaining integration step

The remaining work is call-site migration: new rendering/replay paths should
prefer the strong V4/V5 constructors and strong reciprocal validator, while
legacy deserialization paths should continue using the migration-compatible
APIs until their stored records can be upgraded without changing historical
meaning.
