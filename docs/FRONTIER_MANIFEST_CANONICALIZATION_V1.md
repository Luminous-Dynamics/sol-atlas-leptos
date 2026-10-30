# Frontier Manifest Canonicalization V1

Sol Atlas frontier manifests use an explicitly versioned canonicalization contract.

## Contract

`EvidenceFrontierManifestCanonicalizationV1` identifies the canonicalization rules used by the existing `EvidenceFrontierV1::manifest_hash` field.

V1 canonicalizes:

- evidence temporal metadata by `evidence_id`;
- source-snapshot temporal metadata by `source_snapshot`.

V1 deliberately preserves the supplied order of `argumentation_metadata`.

That last property is compatibility-sensitive: changing it would alter content-addressed hashes for already-issued frontier manifests even when the underlying argumentation records are otherwise identical.

## Compatibility rule

The V1 version label is an API-level contract identifier and is **not** included as an additional hash payload field. This is intentional: the V1 implementation must reproduce existing manifest hashes byte-for-byte.

A future canonicalization revision MUST use a new explicit versioned contract and MUST NOT silently change `computed_manifest_hash()` semantics for V1 artifacts.

A future V2 may establish argumentation metadata as set-like and canonicalize it by a stable semantic key, but that change requires:

1. an explicit V2 contract;
2. compatibility tests proving V1 hashes remain unchanged;
3. independent reconstruction tests for multiple argumentation records;
4. migration/dual-read semantics if existing manifests need to remain verifiable;
5. documentation of whether argumentation order is semantically meaningful or merely representational.

## Why this boundary exists

W3C PROV treats a provenance instance as a set of statements and defines equivalence independently of statement ordering, while also recognizing that applications such as digital signing may intentionally distinguish syntactically different representations. Sol Atlas therefore separates semantic equivalence questions from content-addressed artifact identity.

The current V1 contract is conservative: it removes incidental ordering from evidence/source metadata while preserving existing argumentation ordering semantics until they are explicitly versioned and tested.

Historical replay must never silently reinterpret an old manifest under a new canonicalization rule.