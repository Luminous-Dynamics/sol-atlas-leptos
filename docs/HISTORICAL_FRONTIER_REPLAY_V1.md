# Historical Frontier Replay

Sol Atlas V5 now exposes a selected-frontier replay boundary through
`cultural_projection_historical_replay::validate_v5_at`.

The invariant is:

selected frontier = verified chain prefix endpoint

not:

selected frontier = arbitrary frontier identifier.

## Validation boundary

The API:

1. Locates the requested frontier by stable ID.
2. Constructs only the ordered chain prefix ending at that frontier.
3. Strictly validates that prefix as an append-only lineage.
4. Requires the audit and canonical claim to name that same frontier.
5. Checks the canonical claim, audit, typed argumentation, and ontology closure against that selected frontier.

Later frontier records are intentionally outside the validation scope. A later malformed
frontier therefore cannot invalidate an otherwise valid earlier replay.

Conversely, a later audit cannot masquerade as an earlier replay: its frontier identity and
closure must match the selected historical endpoint.

## Test obligations

The implementation includes coverage for:

- replay at a verified root prefix;
- later-frontier corruption not contaminating an earlier replay;
- a child-bound audit being rejected at the parent frontier;
- unknown frontier identifiers being rejected.

This is deliberately additive to the existing leaf-oriented V5 replay receipt. A future
historical receipt should record the selected prefix lineage rather than silently redefining
the existing `leaf_frontier` contract.

## Epistemic boundary

Historical replay is descriptive provenance, not historical adjudication. Selecting a frontier
does not upgrade qualification, rank interpretations, or establish historical truth.

This boundary is consistent with PROV's distinction between provenance records and the
assessments users may make from them, and with CRMinf's treatment of argumentation and
provenance assessment as explicit activities rather than an implicit truth oracle.
