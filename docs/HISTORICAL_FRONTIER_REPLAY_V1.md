# Historical Frontier Replay

Sol Atlas V5 replay currently binds an audit to a verified frontier chain. The next additive API should allow selecting an intermediate frontier while validating only the append-only prefix through that frontier.

The invariant is:

selected frontier = verified chain prefix endpoint

not:

selected frontier = arbitrary frontier identifier.

Later frontier records must not participate in the historical replay result. This keeps a historical projection reproducible while preserving the append-only ancestry proof.

The implementation should remain descriptive provenance: selecting a frontier does not alter qualification, rank interpretations, or establish historical truth.
