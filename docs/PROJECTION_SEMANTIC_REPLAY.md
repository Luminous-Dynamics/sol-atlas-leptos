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

For example, CIDOC CRM 7.4 was released in August 2026 but is currently marked
Draft by the CIDOC CRM version registry. Draft status therefore remains visible
in the mapping context rather than being silently treated as an implementation
stable release. citeturn0search1turn0search2

CRMinf 1.2.1 is the natural argumentation interoperability target for the
existing assessment/interpretation layer, while PROV-O remains the provenance
interchange boundary. PROV-O is a W3C Recommendation and explicitly supports
provenance interoperability, versioning, reproducibility, and derivation. citeturn0search9turn0search0

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

## Next integration step

The semantic envelope is intentionally additive. The next integration can add
a semantic-context reference to `CulturalProjectionAuditV2` via a new audit
version, preserving old serialized records while making "why shown?" audits
replayable against the exact ontology vocabulary used at render time.
