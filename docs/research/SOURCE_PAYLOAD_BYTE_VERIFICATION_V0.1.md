# Source payload and schema byte verification v0.1

**Status:** Rust in-memory verifier, receipt schema, and synthetic receipt fixture implemented; exact-head formatting, strict Clippy, unit tests and schema CI are pending.
**Parent:** [#525 — independent payload and schema byte verification](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/525).
**Composes with:** [descriptor sidecar PR #496](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/496), [projector PR #491](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/491), [IEAC identity PR #487](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/487), [coverage runtime PR #490](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/490), and trust/canonicalization issue #347.

## The missing proof

The source descriptor catalogue distinguishes four digest bases:
- locally retrieved and hashed;
- source-owner-attested;
- external reference;
- unavailable.

That vocabulary is useful only if the claimed byte digest can be checked against actual bytes. A descriptor can say that content was retrieved and hashed; the verifier must not accept that sentence as its own evidence.

This module hashes caller-supplied byte buffers. It has no URL fetcher and does not use a display filename as an object key.

## API

The Rust entry point is:

\`verify_catalog_payload_bytes(case, catalog, payload_inputs, schema_inputs)\`

- \`case\`: exact typed BuiltEnvironmentCase;
- \`catalog\`: exact descriptor catalogue, validated against the case before byte processing;
- \`payload_inputs\`: BTreeMap keyed by typed case record key: \`evidence:<id>\`, \`artifact:<id>\`, \`engineering_receipt:<id>\`, or \`decision:<id>\`;
- \`schema_inputs\`: BTreeMap keyed by \`(schema_id, schema_version)\`.

For every provided buffer, the verifier computes SHA-256 from the exact byte slice. Payloads with a declared byte length are checked independently against that length. Schema buffers are hashed separately against the declared schema digest.

The input manifests bind the sorted input keys, byte lengths, and computed byte digests. The receipt then binds the exact case representation, descriptor catalogue digest, payload manifest, schema manifest, per-record result, all findings, and the explicit nonclaims. The receipt self-digest uses the canonical serde_json struct serialization with the receipt_sha256 field empty; the digest basis is stated in the receipt itself.

## Outcomes

### Payload outcomes

- \`verified_supplied_bytes_match_local_digest_claim\`: caller-supplied bytes match the locally claimed source digest and declared length.
- \`supplied_bytes_match_owner_attested_digest\`: supplied bytes corroborate the owner's digest, but do not authenticate the owner or signature.
- \`verified_supplied_bytes_match_external_digest_claim\`: supplied bytes match the digest attached to an external content reference.
- \`missing_bytes\`: a digest exists but no corresponding byte buffer was supplied.
- \`observed_bytes_without_source_digest_claim\`: bytes were hashed, but no source digest was claimed; the verifier does not promote its computed digest to source-native identity.
- \`source_digest_unavailable\`: neither a source digest nor a byte buffer is available.
- \`digest_mismatch\`, \`length_mismatch\`, or \`invalid_expected_digest\`: the claimed byte binding fails and the disposition is invalid.

Digest and length mismatches are evaluated independently. One bad buffer can therefore produce both findings instead of hiding one behind the other.

### Schema outcomes

- \`schema_bytes_match_declared_digest\`;
- \`schema_bytes_missing\`;
- \`schema_digest_unavailable\`;
- \`schema_identity_unavailable\`;
- \`schema_digest_mismatch\`.

A schema byte match proves which exact schema bytes were supplied. It does not prove that a parser implements the schema correctly, that an IFC/IDS implementation conforms, or that the schema semantics are adequate for an engineering decision.

## Disposition

- \`complete_for_supplied_claims\`: every supplied digest claim and schema digest in scope was matched to the exact supplied bytes, with no unreferenced inputs or descriptor incompleteness.
- \`incomplete_or_indeterminate\`: bytes or digests are unavailable, descriptor validation is incomplete, an input is unreferenced, or a schema binding cannot be established.
- \`invalid_input\`: a payload digest, declared length, schema digest, or descriptor catalogue binding is contradictory.

The disposition is only about the descriptor and byte inputs supplied for this run. It does not mean the source inventory is complete.

## Nonclaims

The verifier explicitly returns:
- source owner authentication: \`not_performed\`;
- source attestation authentication: \`not_performed\`;
- rights authentication: \`not_performed\`;
- source-universe completeness: \`not_performed\`;
- engineering correctness: \`not_performed\`;
- action authority: \`none\`.

Even a fully matching byte receipt does not prove where the bytes came from, whether a named source owner controls that namespace, whether an owner signature is valid, whether rights permit the requested use, whether all required sources were enumerated, or whether an engineering result is correct.

Those remain separate obligations owned by the Mycelix trust/custody layer (#347), the source coverage verifier (#489/#490), the temporal invalidation engine (#397), and the independent impact oracle (#484).

## v0.1 implementation boundary

This implementation accepts in-memory byte slices. Memory use is proportional to all supplied payload and schema buffers. It does not stream 100 MiB/1 GiB artefacts, download from network object stores, or authenticate retrieval receipts. Large engineering payloads should move to a future streaming Read interface and an explicit content-addressed object-store adapter.

The verifier is intentionally separate from the descriptor validator and projector. It does not rewrite the descriptor catalogue or silently replace a missing source digest with its own computed value.

## Qualification targets

P0:
- zero incorrect payload digest accepted;
- zero incorrect byte length accepted;
- zero schema digest mismatch accepted;
- zero unverified buffer labelled verified;
- zero unavailable source digest promoted to a native source claim;
- exact replay of input manifest and receipt digests;
- every unreferenced payload/schema input reported;
- no byte-verification result can promote source authenticity, rights, completeness, engineering correctness or action authority.

P1:
- streaming profiles at 1 KiB, 1 MiB, 100 MiB, and 1 GiB;
- report MB/s, p50/p95/p99, peak RSS, allocations, CPU time, and bytes read;
- compare SHA-256 throughput against platform-optimized backends on identical byte snapshots;
- require exact digest agreement across all implementations;
- profile one large schema reused across many records and many small schemas across a large project;
- measure full pipeline cost, including object retrieval and validation, not only hashing.

P2:
- rights-cleared IFC, QIF, CAD/BIM and material-certificate payloads from independently governed source systems;
- independent source-owner confirmation of descriptor IDs and allowed retrieval;
- cross-owner selective disclosure under #347;
- held-out project/source-system tests.

No benchmark target above is an achieved result.
