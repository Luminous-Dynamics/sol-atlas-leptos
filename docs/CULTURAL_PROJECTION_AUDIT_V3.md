# Cultural Projection Audit V3

The semantic replay layer is now connected to the reversible cultural audit boundary through `CulturalProjectionAuditV3`.

## Contract

`CulturalProjectionAuditV3` contains:

- the complete `CulturalProjectionAuditV2`;
- a `ProjectionSemanticEnvelopeV1`;
- the same projection identity;
- the same canonical claim;
- the same evidence frontier;
- the same qualification.

Validation rejects semantic contexts that drift from any of those values.

Each ontology mapping context is itself content-addressed, so the resulting audit can detect vocabulary drift at multiple levels:

`audit -> semantic envelope -> mapping context -> standard/version/release/term/relation`

while the existing evidence path remains:

`audit -> claim -> evidence -> source snapshot -> frontier`

Argumentation remains a separate temporal path within V2 rather than being silently collapsed into the semantic vocabulary layer.

## Strong replay invariant

A rendered cultural projection is fully replayable only when:

1. the historical projection is temporally valid;
2. the evidence/source frontier is valid;
3. any assessment/interpretation argumentation is available at that frontier;
4. the ontology mapping context used for rendering is pinned and integrity checked.

Changing the ontology term, mapping relation, ontology version, release status, mapping qualification, claim, frontier, or projection identity therefore cannot silently alter the meaning of an existing V3 audit.

The V3 layer remains additive. V1 and V2 serialized audit contracts remain available for compatibility.

For provenance verification, the ordinary V3 projection validator checks the audit against its current semantic envelope, while `validate_against_projection_and_semantic_context(...)` additionally requires the caller to supply the exact expected semantic envelope. The additive `validate_strong_against_projection(...)` gate further requires non-empty frontier-manifest and originating projection-semantic commitments, matching the stronger V4/V5 provenance tiers. This mirrors the broader rule that an object's own content hash proves integrity, but external reconstruction/witness data establishes what source artifact was actually used.

## External ontology boundary

This does not make Sol Atlas an ontology authority. External standards remain interoperability vocabularies. In particular, CIDOC CRM 7.4 is currently listed as an August 2026 Draft, and its registry distinguishes Draft releases from Stable and Official releases. Draft status is therefore retained as explicit metadata rather than silently treated as an implementation-stable vocabulary.
