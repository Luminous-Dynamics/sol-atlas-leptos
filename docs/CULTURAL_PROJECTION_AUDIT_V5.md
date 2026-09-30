# Cultural Projection Audit V5

## Purpose

CulturalProjectionAuditV5 is an additive audit layer over V4. It makes typed
argumentation part of the content-addressed replay identity while preserving
the older V1/V2 argumentation contracts.

The typed vocabulary currently contains:

- InferenceMaking
- BeliefAdoption
- ProvenanceAssessment
- MeaningComprehension

These correspond to the projection-side vocabulary introduced from CRMinf
1.2.1. Sol Atlas does not claim complete CRMinf conformance.

## Invariants

A V5 audit:

1. remains bound to one canonical claim;
2. remains bound to one evidence frontier;
3. validates every argumentation evidence/source closure independently;
4. includes argumentation kind in semantic identity;
5. rejects duplicate (kind, assessment, interpretation) identities;
6. preserves competing argumentations without ranking them;
7. cannot change or upgrade the base qualification;
8. requires a typed record to exactly preserve a legacy V2 argumentation record when one is present;
9. hashes canonical argumentation order so serialization order has no semantic meaning.

The kind field is semantic vocabulary, not a confidence score.

## Replay boundary

The intended replay chain is:

historical state
-> canonical claim
-> evidence/source closure
-> temporal evidence frontier
-> argumentation availability
-> typed argumentation identity
-> ontology/semantic audit
-> reversible projection

A later frontier cannot admit an earlier audit merely because the same claim or
argumentation identifiers exist. Frontier validation remains the authority for
temporal availability.

## Why V5 is additive

V3 and V4 remain usable by callers that do not yet carry typed argumentation.
V5 does not reinterpret existing qualification values or make ontology mappings
into claims. This keeps migration explicit and prevents a semantic vocabulary
upgrade from silently changing historical qualification.

## External alignment

CRMinf 1.2.1 separates argumentation activities such as inference making,
belief adoption, provenance assessment, and meaning comprehension. W3C PROV
similarly treats provenance as a structured record about entities, activities,
and agents rather than as a truth score.

The Sol Atlas contract therefore records what kind of argumentation is being
represented and which evidence closure makes it replayable, without deriving
an epistemic ranking from the argumentation kind itself.
