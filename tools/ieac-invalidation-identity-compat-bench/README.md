# IEAC Invalidation Identity Compatibility Reference v0.2

Standalone Rust reference checker for the additive graph/event/report compatibility boundary described in issue #650.

## Purpose

IEAC graph v0.2 separates a source-native node identity from the exact current artifact content. This adapter prevents the older v0.1 invalidation-event and impact-report contracts from being silently applied to a v0.2 graph.

Every v0.2 target/result must bind all three of:
- `node_id` using `type:identity-sha256:<digest>`;
- `identity_sha256`, recomputed from the native identity namespace and canonical components;
- `artifact_sha256`, the exact current artifact payload digest.

The identity digest uses the same v0.2 domain separator and length-prefixed field encoding as the existing graph projector. Identity component order is canonicalized by ASCII component name. Component values remain exact UTF-8 strings. The artifact digest is never substituted for identity.

## Compatibility matrix

| Graph | Invalidation event | Impact report | Policy |
|---|---|---|---|
| v0.1 | v0.1 | v0.1 | Preserve legacy behavior without coercion |
| v0.2 | v0.2 | v0.2 | Require native identity and content binding |
| v0.2 | v0.1 | any | Reject |
| v0.1 | v0.2 | any | Reject |
| any | any | mixed/unsupported | Reject |

v0.1 schemas and historical fixtures are not modified. The v0.2 files are additive.

## Checks performed

- Graph snapshot digest is recomputed from canonical JSON with its own digest member excluded.
- Native identity digest is recomputed from the exact v0.2 domain-separated length-prefixed encoding.
- Duplicate/invalid identity components and unknown node types are rejected.
- Event digest is recomputed with `event_sha256` excluded.
- The event's graph snapshot and the report's graph/event digests must bind to the exact evaluated inputs.
- Each event target is resolved to exactly one graph node and must match type, native identity and artifact digest.
- Each impact result must resolve to a graph node and must match the same three fields; every invalidation target must have a report result.
- Retroactive invalidation requires verified effective time and approved independent review. A pending review is not promoted by attaching a SCITT receipt.
- `action_authority` must remain `none`.

## Frozen synthetic corpus

The 25 known-answer cases cover positive `source_evidence`, `engineering_artifact` and `engineering_receipt` node types; legacy v0.1 preservation; version-matrix rejection; wrong identity, content digest or node type; absent targets; graph/event/report digest mismatch; duplicate node identities; unknown node types; retroactive time uncertainty; pending authority review; component-order invariance; and identical payload digests for distinct native identities.

All fixture identities, content and timestamps are synthetic. The evaluator does not authenticate real issuer signatures, SCITT transparency receipts, owner rights or the authority of a revoker.

## Run

From repository root:

~~~sh
cargo fmt --manifest-path tools/ieac-invalidation-identity-compat-bench/Cargo.toml --all -- --check
cargo test --locked --manifest-path tools/ieac-invalidation-identity-compat-bench/Cargo.toml
cargo clippy --locked --manifest-path tools/ieac-invalidation-identity-compat-bench/Cargo.toml --all-targets -- -D warnings
cargo run --locked --quiet --manifest-path tools/ieac-invalidation-identity-compat-bench/Cargo.toml -- \
  docs/research/ieac-identity-compat/IEAC_INVALIDATION_IDENTITY_COMPAT_CORPUS_V0_1.json \
  docs/research/ieac-identity-compat/IEAC_INVALIDATION_IDENTITY_COMPAT_CORPUS_V0_1.schema.json \
  schemas/industry-assurance-dependency-graph-v0.2.schema.json \
  schemas/industry-assurance-invalidation-event-v0.2.schema.json \
  schemas/industry-assurance-impact-report-v0.2.schema.json \
  --source-commit "$(git rev-parse HEAD)" > ieac-invalidation-identity-compat-receipt.json
~~~

The CLI validates the pinned schema identifiers/editions, corpus structure, and exact known-answer outputs. It does not claim to be a general JSON Schema 2020-12 engine or the production IEAC invalidation propagation engine.

## Claim ceiling

A passing result establishes only synthetic v0.1/v0.2 compatibility and native-identity/content binding mechanics. It does not authenticate the issuer/revoker, validate a SCITT receipt, prove source completeness or rights, establish engineering correctness, certify a habitat or authorize any action.
