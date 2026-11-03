# IEAC native node identity v0.2

**Status:** Additive identity-contract proposal and synthetic collision fixture. v0.1 remains unchanged.
**Parent:** [BE-IEAC-BRIDGE-001 / #486](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/486).
**Composition:** [IEAC lifecycle graph #397](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/397), [built-environment graph #138](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/138).

## Problem found

The v0.1 node identifier is `node_type:sha256:artifact_sha256`. That makes payload identity and native-record identity the same key. Two independent records with identical bytes therefore collapse to one node ID even when they represent different sources, observations, revisions or authorities.

This is not a theoretical corner case. A source can legitimately publish two observations whose values/bytes are equal; two tenants can export the same payload; and two artifact revisions can have byte-identical contents while remaining distinct domain records. Equality of content is not equality of identity.

## v0.2 separation

v0.2 separates three values:

- `identity`: the native owner namespace plus named native identity components.
- `identity_sha256`: digest of the canonical identity envelope.
- `artifact_sha256`: digest of the exact payload bytes.

The node ID becomes:

```text
<node_type>:identity-sha256:<identity_sha256>
```

Two nodes may carry the same `artifact_sha256` and still be distinct. Conversely, the same native identity with conflicting content is a duplicate/conflict and must not be silently accepted; callers must supply the actual native revision/observation discriminator as an identity component when the source defines one.

## Identity hashing wire contract

To avoid implicit JSON canonicalization dependencies, v0.2 uses a small domain-separated binary encoding:

1. ASCII domain separator: `LUMINOUS-IEAC-NODE-IDENTITY-V0.2\0`.
2. UTF-8 fields in this order: `node_type`, `identity.identity_namespace`.
3. The `identity.components` array is canonicalized by ascending ASCII `name`; duplicate names reject.
4. For each component, append `name`, then `value`, in UTF-8.
5. Each field is encoded as an unsigned 64-bit big-endian byte length followed by exactly that many UTF-8 bytes.
6. No Unicode normalization, trimming, case folding, URI rewriting or vendor-ID interpretation is performed. Empty names/values reject.
7. SHA-256 over the complete byte sequence is `identity_sha256`.

The schema constrains component names to ASCII identifiers. The runtime independently validates uniqueness and recomputes the digest. Component input order does not affect the identity hash; value bytes do.

The graph snapshot digest remains the existing graph-level canonical JSON digest. This v0.2 node identity encoding does not silently change the v0.1 graph digest contract.

## Collision corpus

`schemas/examples/ieac-node-identity-collision-synthetic-v0.2.json` contains four source-measurement nodes with:

- one identical exact payload digest across all four;
- two observation IDs from one native object and namespace;
- a separate native revision of that object;
- the same native ID in a second source namespace.

Expected result: four unique node IDs, one unique content digest, and no action authority. This fixture is synthetic; it is not a production OPC UA capture.

## Migration rules

- v0.1 fixtures remain valid only under the v0.1 schema and their original identity semantics.
- v0.2 must not rewrite a v0.1 node ID in place.
- A v0.1 digest-keyed node with no recoverable native identity cannot be losslessly promoted. The migration report must be `incomplete_or_indeterminate`, not assign a fabricated native ID.
- Where a v0.1 graph has multiple source records already merged under the same node ID, the source records must be re-extracted from authoritative inputs before migration.
- v0.1 and v0.2 graph digests, event target IDs and report digests are version-bound; no silent cross-version reuse.

## Qualification requirements

P0:
- v0.1 regression corpus remains unchanged and passes.
- 100% identity-digest recomputation agreement on v0.2 fixtures.
- 0 node collapse for equal-payload/different-identity cases.
- 0 acceptance of duplicate native identity with conflicting payload/revision semantics.
- 0 acceptance of identity-digest, node-ID or graph-digest mutations.
- component-order permutations produce identical identity and graph results after graph digest recomputation.

P1:
- Cross-language known-answer vectors for the binary identity encoding.
- Identity-collision corpus spanning OPC UA ExpandedNodeId-like namespaces, AAS global/specific IDs, IFC GlobalId, QIF result IDs and project-local artifact IDs.
- Held-out adapters prove they preserve namespace, native identifier, revision/sequence and observation identity.

## Standards relationship

The identity envelope follows the general principle that source-native identities are scoped by their owning namespace, not inferred from payload bytes. Asset Administration Shell identity guidance uses globally unique identifiers and supports additional specific asset identifiers. OPC UA distinguishes namespace-qualified identifiers from local numeric/string identifiers. These are interoperability anchors; this v0.2 envelope is not a replacement for either source model.

RFC 8785 JCS is useful for canonical JSON hashing, but v0.2 deliberately avoids depending on JSON number/string serialization for node identity. Its fixed binary encoding is easier to reproduce and is separately versioned.

## Nonclaims

This contract does not authenticate source ownership, prove the payload is true, establish that identity components are globally unique outside the declared namespace, or grant any action authority. The adapter must obtain identity components from a source-authorized record or mark projection incomplete.
