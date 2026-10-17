# Scientific Evidence Snapshot V1

This module adds renderer-neutral, versioned metadata and provider-record reference contracts for scientific datasets. The record binding is a presentation/interchange adapter, not a second epistemic authority.

## Scope of the first fixture

The frozen fixture records the Open Targets Platform 26.09 release announcement. Open Targets announced that release on 25 September 2026. This is **metadata-only**: it does not contain a downloaded dataset artifact, association rows, drug-target evidence, or a digest of dataset bytes.

The dataset licence metadata is recorded as CC0-1.0 according to the Open Targets Platform licence documentation. The announcement webpage itself has a separate `review_required` licence state because a dataset licence must not be automatically applied to the release page's content.

This fixture is a small source/metadata exercise, not a biomedical evidence set and not proof that a drug-target association is true.

## Contract invariants

- Parse untrusted JSON through `parse_snapshot_v1`; unknown schema fields are rejected and public snapshots are returned only after validation.
- Keep source-record licensing separate from dataset licensing.
- Preserve unknown and review-required licence states rather than silently upgrading them.
- Preserve source publication/release date separately from the date metadata was retrieved.
- A metadata-only record must not carry a dataset artifact digest.
- A recorded digest is syntactically checked SHA-256 hex only. The module does not compute the digest from bytes, authenticate the publisher, or prove integrity.
- Snapshot validation does not qualify a scientific proposition or clinical use.
- `ScientificDatasetRecordRefV1` links a provider-native record to the exact local metadata snapshot and can carry opaque Mycelix artifact/selector refs and a Symthaea analysis ref. It does not resolve or qualify those external references.
- The serialized binding includes its derived `binding_state` and fixed `authority_scope`. Re-import recomputes the expected binding state and rejects mismatch or any authority scope other than `sol_atlas_projection_only`.
- Mycelix artifact-snapshot and selector refs must be supplied together. Without them, the binding remains `LocatorOnly`; a mutable URL is never treated as an exact evidence target.
- The binding's authority scope is fixed to `sol_atlas_projection_only`; adding a Symthaea analysis ref cannot upgrade it.
- `Known` licence status requires an identifier; `Unknown` and `ReviewRequired` prohibit one to prevent contradictory metadata.
- URLs are limited to HTTPS syntax checks; this is not a full URL parser or network verification.

## Run

From the repository root:

    cargo fmt --manifest-path sol-atlas-core/Cargo.toml -- --check
    cargo test --manifest-path sol-atlas-core/Cargo.toml scientific_evidence
    cargo clippy --manifest-path sol-atlas-core/Cargo.toml -- -D warnings

The module currently has 24 focused unit tests covering source metadata, strict parsing, digest semantics, malformed dates, cross-project record bindings, and attempts to upgrade or misstate the serialized authority boundary. CI results, not the test count in this document, determine whether a build or test run passed.

## References

- Release announcement: https://blog.opentargets.org/open-targets-platform-26-09-has-been-released/
- Dataset licensing: https://platform-docs.opentargets.org/licence
