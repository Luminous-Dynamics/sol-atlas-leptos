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

## Next integration step

The semantic envelope is intentionally additive. The next integration can add
a semantic-context reference to `CulturalProjectionAuditV2` via a new audit
version, preserving old serialized records while making "why shown?" audits
replayable against the exact ontology vocabulary used at render time.
