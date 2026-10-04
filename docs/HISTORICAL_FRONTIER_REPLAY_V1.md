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

Before selection, the supplied sequence's frontier IDs are required to be globally unique.
Later frontier contents are still outside the validation scope: malformed content on a distinct
later ID therefore cannot invalidate an otherwise valid earlier replay. Duplicate identity is
different because it would make the historical selector itself ambiguous.

Conversely, a later audit cannot masquerade as an earlier replay: its frontier identity and
closure must match the selected historical endpoint.

## Historical receipts

`cultural_projection_historical_receipt::V5HistoricalReplayReceiptV1` is the
content-addressed artifact for a selected historical replay.

The receipt:

1. records the selected frontier explicitly;
2. builds the existing V5 replay receipt against only the verified prefix through that frontier;
3. binds its own hash to both the selected frontier and the underlying replay receipt;
4. preserves the existing `leaf_frontier` semantics instead of redefining them.

The adversarial fixture now models three epochs — `frontier:1950`,
`frontier:1951`, and `frontier:1952` — with independently introduced evidence.
The tests assert that:

- the three manifests are distinct and the full chain validates;
- a root receipt contains only its one-frontier lineage;
- corruption in later frontiers cannot invalidate that historical receipt;
- the same corruption does invalidate strict validation of the descendant chain;
- rewriting an inherited root manifest invalidates descendants;
- a receipt cannot be transplanted to a later structurally similar frontier;
- qualification remains the canonical claim's qualification.

The leaf-oriented V5 replay receipt applies the same identity rule: projection-bound
validation requires globally unique frontier identifiers before selecting the receipt's leaf
prefix. Historical selection changes the validation boundary, not the meaning of `leaf_frontier`.

## Epistemic boundary

Historical replay is descriptive provenance, not historical adjudication. Selecting a frontier
does not upgrade qualification, rank interpretations, or establish historical truth.

This boundary is consistent with PROV's distinction between provenance records and the
assessments users may make from them, and with CRMinf's treatment of argumentation and
provenance assessment as explicit activities rather than an implicit truth oracle.
