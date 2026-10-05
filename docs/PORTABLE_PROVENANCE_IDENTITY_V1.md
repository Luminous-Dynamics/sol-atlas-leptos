# Portable provenance identity V1

## Purpose

`V5PortableReplayIdentityV1` is an additive portability envelope around the
existing `V5ReplayReceiptV1`.

It does not replace the V1 receipt hash and does not make a new authority
layer. Its purpose is to make the cryptographic interpretation of a replay
identity explicit to consumers that may not share the Rust implementation.

## Frozen identity contract

The portable identity declares:

- schema: `sol-atlas:portable-provenance-identity:v1`
- payload kind: `v5_replay_receipt`
- digest algorithm: `sha256`
- canonicalization contract: `serde-json-tuple-v1`
- digest domain:
  `sol-atlas:v5-replay-receipt:portable-identity:v1`

The portable content digest is SHA-256 over the serialized tuple:

1. the fixed digest-domain string;
2. the fixed canonicalization identifier;
3. the fixed digest-algorithm identifier;
4. the complete, already-validated `V5ReplayReceiptV1`.

The portable identity also carries the source receipt's existing
`receipt_hash`. This lets a consumer distinguish:

- the legacy receipt's own content-addressed identity;
- the portable envelope's explicitly-described digest.

## Validation tiers

`V5ReplayReceiptV1::validate()` establishes standalone structural and
content-address integrity only.

It does **not** establish that the receipt actually came from the expected
projection, claim, or frontier chain.

That stronger relationship remains the responsibility of:

- `validate_against_audit_and_chain(...)`;
- `validate_against_projection(...)`;
- `validate_strong_against_projection(...)`.

`V5PortableReplayIdentityV1::from_receipt(...)` requires the standalone receipt
integrity gate before calculating the portable digest.

Receipt validation also rejects duplicate frontier identifiers, duplicate
argumentation identities, and duplicate ontology-resolution identities before
the portable digest is calculated.

Therefore construction errors are returned before identity materialization. A
`ProjectionError` variant, its display text, and its machine-readable error
code are not inputs to the portable digest.

## Non-goals

The portable identity does not:

- upgrade qualification;
- establish canonical historical truth;
- replace the external canonical-claim authority;
- claim RFC 8785/JCS wire compatibility;
- claim SLSA or in-toto conformance.

The current canonicalization contract remains application-defined. RFC 8785 is
a future interoperability option because JCS provides an invariant JSON form
for cryptographic operations, including recursive object-property ordering
while preserving array element order.

## External provenance alignment

SLSA v1.2 treats provenance as verifiable information whose consumers must
check that the provenance applies to the intended immutable artifact/revision,
rather than trusting the attestation solely because it is well formed.

Sol Atlas follows that separation as a design analogy:

- integrity: the receipt/identity is internally self-consistent;
- provenance: the receipt is tied back to the expected audit, chain, claim,
  and originating projection;
- authority: canonical historical qualification remains external.
