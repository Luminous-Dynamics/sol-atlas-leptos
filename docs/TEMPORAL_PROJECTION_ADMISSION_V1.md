# Temporal Projection Admission V1

## Purpose

`ProjectionAdmissionV1` is the reversible receipt for a temporal projection candidate
that crossed an `EvidenceFrontierV1`.

It is provenance, not authority. An admission says why a snapshot or transition was
eligible for the requested projection; it does not establish the historical claim as
true, uncontested, or authoritative.

## Receipt identity

A receipt records:

- the projected object (`Snapshot` or `Transition`);
- the selected evidence frontier ID;
- the selected frontier's content hash when one exists;
- the evidence references carried by the projection;
- the source snapshots carried by the projection.

For newly hashed frontiers, `frontier_manifest_hash` binds the receipt to the exact
content-addressed frontier manifest. Reciprocal validation also verifies the frontier's
current contents before trusting that hash.

The invariant is therefore:

`receipt.frontier_manifest_hash == frontier.manifest_hash`

plus independent verification of the populated frontier manifest.

A frontier with the same ID but different content cannot replace the bound frontier when
the receipt carries a populated manifest hash.

## Reciprocal validation

`validate_against_snapshot` and `validate_against_transition` perform the reverse
direction of the construction functions.

They require:

1. the receipt itself to be structurally valid;
2. the candidate to be structurally valid;
3. the frontier to be temporally/structurally valid when it carries a populated manifest;
4. the receipt and candidate frontier IDs to match the supplied frontier;
5. the candidate to pass the canonical frontier admission predicate;
6. the receipt to equal the deterministic receipt reconstructed from the candidate.

This prevents editing the receipt's evidence, sources, projection identity, frontier
identity, or other admission fields after construction.

## Projection-set validation

`TemporalProjectionSetV1::validate()` uses the reciprocal admission validators rather
than relying only on shape checks or local field equality.

The result-level invariant is:

`recorded admission == deterministic admission reconstructed from the included candidate and selected frontier`

This keeps the projection set auditable in either direction:

`candidate + frontier -> admission`

and

`admission + candidate + frontier -> exact reconstruction`

## Legacy migration

Older serialized receipts may not contain `frontier_manifest_hash`. The field therefore
uses a serde default for deserialization.

An empty manifest hash remains valid for migration-compatible ID-only frontiers.

A legacy empty-hash receipt must not be treated as content-bound to a newly populated
content-addressed frontier. New/reproducible replay should use strict frontier validation,
which requires a non-empty self-consistent manifest hash.

## Boundary

Admission is an eligibility/provenance boundary. It does not:

- upgrade `QualificationStatus`;
- infer historical truth from graph structure;
- turn replication or signing into authority;
- resolve contested interpretations;
- replace the canonical claim/evidence/source closure.

Temporal availability remains governed by `available_by <= known_by_year`, and frontier
lineage remains a separate append-only verification concern.

## Compatibility rule

Future changes to receipt identity must be explicitly versioned. Existing V1 manifest hashes
and legacy migration behavior must not be silently reinterpreted.
