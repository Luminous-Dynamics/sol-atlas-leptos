# Ontology Mapping Evidence Resolution

## Purpose

Semantic replay must not become an evidence-free side channel.

OntologyMappingContextV1 records the external vocabulary used to render a projection: standard, pinned version, release status, external term, mapping kind, semantic relation, qualification, and a content hash.

OntologyMappingResolutionV1 adds the missing evidentiary closure:

projection -> mapping context -> canonical claim -> evidence -> source snapshots -> temporal frontier

The resolution is projection metadata, not a second claim authority.

## Boundary

A resolution can only be constructed when the underlying OntologyMappingV2 is already frontier-safe against the canonical claim and selected frontier. The resolution then copies the exact claim/evidence/source closure and binds it to the semantic mapping context with a deterministic SHA-256 digest.

Replay therefore has two independent integrity checks:

1. semantic vocabulary integrity — the mapping context hash;
2. evidentiary closure integrity — the resolution hash plus frontier admission.

Changing the ontology term, relation, claim, evidence set, source set, or frontier makes the resolution fail validation or frontier safety.

## Why this matters

A renderer may legitimately use an external ontology to label or structure a view, but that vocabulary must not silently become an assertion that is better qualified than the underlying evidence. The resolution boundary preserves the direction of authority:

canonical claim -> evidence closure -> temporal frontier -> semantic projection

rather than:

ontology mapping -> implied fact

## Version status

External release status remains explicit. CIDOC CRM currently lists 7.4 (August 2026) as Draft, while CRMinf 1.2.1 was announced in April 2026. Draft terminology can be retained for research/replay, but its draft status must remain visible rather than being silently treated as a stable implementation target.

## Non-goals

This layer does not:

- adjudicate historical or cultural truth;
- infer transmission from ontology similarity;
- upgrade qualification;
- replace Mycelix canonical claim authority;
- decide contested interpretations;
- make a Draft ontology release equivalent to a Stable or Official release.
