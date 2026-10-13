# Scientific Evidence Snapshot V1

This module adds a renderer-neutral, versioned metadata contract for scientific datasets and source records.

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
- `Known` licence status requires an identifier; `Unknown` and `ReviewRequired` prohibit one to prevent contradictory metadata.
- URLs are limited to HTTPS syntax checks; this is not a full URL parser or network verification.

## Run

From the repository root:

    cargo fmt --manifest-path sol-atlas-core/Cargo.toml -- --check
    cargo test --manifest-path sol-atlas-core/Cargo.toml scientific_evidence
    cargo clippy --manifest-path sol-atlas-core/Cargo.toml -- -D warnings

The schema currently has 14 focused unit tests, including both malformed and syntactically valid recorded-digest cases. CI results, not the test count in this document, determine whether a build or test run passed.

## References

- Release announcement: https://blog.opentargets.org/open-targets-platform-26-09-has-been-released/
- Dataset licensing: https://platform-docs.opentargets.org/licence
