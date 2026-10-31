# IEAC v0.2 signed-envelope profile

**Status:** Standards-based design decision record; implementation and cryptographic test vectors are not yet complete.  
**Tracks:** [IEAC-TRUST-001 / #347](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/347).  
**Depends on:** IEAC v0.1 schemas and runtime admission in PRs #345/#346; replayable evidence ledger in PR #362.

## 1. Decision summary

IEAC v0.2 should retain JSON as the human-readable payload format, canonicalize payloads with **RFC 8785 JSON Canonicalization Scheme (JCS)**, and authenticate them with **detached COSE_Sign1**.

The v0.2 signature profile is:

| Concern | Decision |
|---|---|
| Payload format | JSON object governed by a versioned payload schema |
| Canonical bytes | RFC 8785 JCS UTF-8 bytes |
| Payload digest | SHA-256 over the exact JCS UTF-8 byte sequence |
| Signature envelope | COSE_Sign1, detached payload |
| COSE algorithm | **Ed25519, fully specified COSE identifier -19** |
| COSE key | OKP, Ed25519 curve |
| Signature context | COSE protected headers plus the exact domain-separated external AAD defined below |
| Trust bootstrap | Offline/out-of-band root key set; never request-selected |
| Rotation | Root-signed, scoped keyset snapshots with explicit validity and sequence |
| Anti-rollback | Durable monotonic high-water mark per authority source and scope |
| Build identity | SLSA provenance / in-toto attestation for evaluator and verifier artifacts |

**Do not sign ordinary `serde_json::to_vec(Value)` output and call it canonical JSON.** Internal map serialization is not a cross-language canonicalization contract. It may be used for local deterministic logs only when its limitations are explicitly scoped.

## 2. Why this profile

### 2.1 JSON canonicalization

RFC 8785 exists to make JSON suitable for repeatable cryptographic hashing and signing while preserving JSON as the exchanged representation. It constrains input to I-JSON and defines deterministic property sorting and primitive serialization.

This matches the existing IEAC v0.1 JSON schemas and lets external systems inspect payloads without first translating every document into CBOR. Canonicalization must be a named, versioned protocol property rather than an implementation detail.

JCS requirements for v0.2:

- Reject non-I-JSON inputs, duplicate object names, invalid Unicode and numbers that cannot be represented under the JCS numeric model.
- Reject non-finite numeric values. No NaN or infinity extensions.
- Use RFC 8785's exact number serialization and UTF-16 property ordering. Do not normalize Unicode strings or silently rewrite application data.
- Hash the canonical UTF-8 bytes, not the original pretty-printed document.
- Preserve original source bytes and their digest separately when exact submitted-file provenance matters.
- Publish cross-language test vectors and require every producer and verifier implementation to pass the same corpus.

### 2.2 COSE algorithm selection

COSE is a standardized signing envelope. A detached COSE_Sign1 payload avoids duplicating the JSON body inside both the outer envelope and the signature object. The verifier reconstructs the JCS bytes and verifies the COSE signature over those exact bytes.

**Important standards update:** RFC 9864 (October 2025) updates RFC 9053 and deprecates polymorphic COSE EdDSA identifier `-8`. It defines the fully specified Ed25519 algorithm as COSE identifier **`-19`**, marked COSE Recommended. v0.2 therefore uses `-19`; accepting `-8` is a downgrade/legacy negative test, not a compatibility fallback.

The COSE protected header must include:
- `alg = -19` (Ed25519; never infer the curve from a key supplied by the payload);
- `kid` (key identifier used only as a lookup hint into the independently provisioned trust store);
- content type identifying the signed IEAC document profile.

The verifier must confirm that the resolved COSE_Key is OKP / Ed25519, the key usage permits verification, and the key is authorized for the exact signer, registry/source and payload type. A valid signature from an otherwise valid but unauthorized key is not an authorization pass.

### 2.3 Domain separation

The COSE_Sign1 external AAD is exactly the following byte concatenation, with no trailing NUL:

```text
UTF8("LUMINOUS-IEAC") || 0x00 ||
UTF8(payload_type) || 0x00 ||
UTF8("RFC8785-JCS") || 0x00 ||
UTF8(payload_schema_version)
```

The payload type values are:
- `application/vnd.luminous.ieac.profile-pin.v0.2+json`
- `application/vnd.luminous.ieac.revocation-snapshot.v0.2+json`
- `application/vnd.luminous.ieac.trust-keyset.v0.2+json`
- `application/vnd.luminous.ieac.execution-receipt.v0.2+json`

The domain is part of the signed COSE Sig_structure, not a free-form header read from the payload. A signature created for one payload type must fail verification for every other payload type, even if the JSON bytes are identical.

## 3. Signed payload contracts

Each payload carries its own schema version, stable ID, issuer identity, scope, issue time, not-before time, expiry time and relevant object digests.

### 3.1 Profile pin

Required semantic fields:

- `profile_id`, `profile_sha256`, `trusted_registry_id`;
- signer/key identifier and authorization scope;
- `issued_at`, `not_before`, `expires_at`;
- monotonically versioned pin/registry epoch and sequence;
- explicit status and keyset identity.

The pin must be provisioned and signed by an authorized registry signer. A caller-supplied profile, request or recomputed digest cannot establish the trust root.

### 3.2 Authority/revocation snapshot

Required semantic fields:

- `authority_source_id` and exact source scope;
- `epoch`, `sequence`, `issued_at`, `not_before`, `expires_at`;
- identity of the previous accepted snapshot (digest) where the source supports chaining;
- revoked profile IDs, artifact IDs and content digests;
- allowed actions and authorization state;
- signing key ID and keyset identity.

An unavailable or unknown authority state is not equivalent to a current empty revocation list. Denied, unknown, unavailable, expired and revoked states remain distinguishable.

### 3.3 Keyset and delegation

The offline root signs a keyset document that delegates narrowly scoped online keys. Each entry includes:

- stable `key_id`, COSE public key and fully specified algorithm;
- allowed payload types, registry/source IDs and action scopes;
- `not_before` and `expires_at`;
- status, keyset epoch and sequence;
- explicit revocation state.

Routine online-key rotation must not require changing the offline trust root. Root rotation and emergency root recovery are separate operator ceremonies with externally witnessed provisioning and documented rollback protection.

## 4. Anti-replay and anti-rollback state machine

A signature establishes message integrity and key origin. It does **not** establish freshness and does not prevent replay of an older still-valid signed snapshot.

Maintain a durable high-water mark for every `(authority_source_id, scope_id, epoch)` tuple. The minimum transition rules are:

| Incoming snapshot | Required result |
|---|---|
| Sequence greater than stored high-water mark, valid signature and scope | Verify, then atomically advance |
| Same sequence and same digest | Idempotent snapshot reuse; decision-request replay is handled separately |
| Same sequence but different digest | Reject as equivocation |
| Lower sequence | Reject as rollback/replay |
| Future-dated or expired | Reject/abstain per explicit trust policy; never admit |
| Unknown, unavailable or corrupted high-water state | Abstain; never assume sequence zero |
| High-water update cannot be durably committed | Do not issue an admission receipt |

The compare-and-advance transaction must be atomic. The system must define backup/restore and failover semantics so a database restore cannot silently roll the high-water mark backwards. Where local persistence cannot provide rollback resistance, use a protected monotonic service, hardware-backed counter or an independently witnessed transparency mechanism; document the actual assurance boundary.

Authority-snapshot replay and decision-request replay are different threats. A snapshot may be reused for multiple decisions while fresh. A one-shot execution/authorization request must also bind a unique challenge or idempotency key, expiry and consumed state.

## 5. Independent verification and build provenance

The production verifier must not call the candidate's signature verification, canonicalization or admission-decision function.

Required separation:

- Candidate and verifier have separately versioned code and independently identified build artifacts.
- Verifier has its own canonicalization and COSE verification path; shared payload schemas and published test vectors are appropriate.
- The verifier independently checks trust root, keyset, issuer, key usage, payload type, signature, payload digest, time window, authority scope, monotonic sequence and request replay state.
- Every result records verifier ID/version, executable digest, trust-root-set digest, accepted keyset digest and exact payload digest.
- Exact verifier/evaluator executable digests are bound to build provenance. Use SLSA provenance / in-toto statements and verify the expected builder, repository, workflow and source commit; a version string embedded by the binary is not sufficient.

A cryptographically valid signature proves that a key signed bytes. It does not prove the signer was legitimately delegated, the sensor measurement was true, the model was physically credible or the resulting decision improved the physical system.

## 6. Required negative and metamorphic corpus

The frozen preregistration must include at least:

**Signature and envelope:** valid signature; altered payload; altered signature; unknown key; wrong key; key/curve mismatch; wrong protected algorithm; deprecated `-8`; missing protected `kid`; protected-header mutation; malformed COSE; unsupported critical header; payload-type substitution; cross-domain AAD replay.

**Key lifecycle:** expired key; not-yet-valid key; revoked key; wrong scope; wrong registry; unauthorized payload type; stale keyset; keyset sequence rollback; key rotation overlap; old key after rotation; root replacement without out-of-band authorization.

**Authority lifecycle:** stale snapshot; future snapshot; expired snapshot; unknown/unavailable state; duplicate sequence; lower sequence; same sequence with different payload digest; source/scope substitution; revocation removed without a newer authorized sequence; high-water store loss/rollback; failed atomic commit; disaster-recovery restore.

**Canonicalization:** reordered properties; Unicode surrogate-pair ordering; negative zero; integral decimal; exponent boundaries; escaped versus literal Unicode; duplicate object names; malformed Unicode; out-of-domain numbers; different producer/verifier implementations.

**Decision binding:** validly signed pin for wrong profile; validly signed snapshot for wrong asset; authority scope escalation; valid old decision request replay; request nonce reuse; receipt altered after signature; verifier identity mismatch.

Acceptance: 100% of frozen negative controls fail closed; 100% golden positive vectors pass; zero sequence rollback or cross-domain replay admission; and no claim of production trust until the root-provisioning and rollback-resistant state mechanisms are independently reviewed.

## 7. JCS starter vectors

The companion fixture `schemas/examples/ieac-jcs-vectors-v0.2.json` includes small vectors for object-key ordering, numeric normalization, UTF-16 property ordering and nested objects. The expected SHA-256 is over the exact UTF-8 canonical bytes. These are starter regression vectors, not a substitute for the complete RFC 8785 test corpus or validation against at least two independent conforming implementations.

The companion `schemas/examples/ieac-signed-envelope-synthetic-v0.2.json` fixture is **wire-shape-only**. Its COSE_Sign1 structure contains a zero-filled placeholder signature and must fail cryptographic verification. It exists solely to exercise the JSON Schema; it is not a positive signature vector and must never be used as a trust fixture or deployment credential.

## 8. Standards and implementation references

- RFC 8785, JSON Canonicalization Scheme: https://www.rfc-editor.org/rfc/rfc8785.html
- RFC 9052, COSE Structures and Process: https://www.rfc-editor.org/rfc/rfc9052.html
- RFC 9053, Initial COSE Algorithms (updated): https://www.rfc-editor.org/rfc/rfc9053.html
- RFC 9864, Fully-Specified JOSE/COSE Algorithms: https://www.rfc-editor.org/rfc/rfc9864.html
- RFC 8949, CBOR deterministic encoding: https://www.rfc-editor.org/rfc/rfc8949.html
- in-toto Attestation Framework: https://github.com/in-toto/attestation/tree/main/spec
- SLSA Provenance v1: https://slsa.dev/spec/v1.0/provenance
- GitHub Artifact Attestations: https://docs.github.com/en/actions/concepts/security/artifact-attestations
- NISTIR 8620, Digital Twins Workshops Summary Report: https://doi.org/10.6028/NIST.IR.8620
- NIST, Methods of UQ in Computational Modeling and Simulation (2026): https://www.nist.gov/publications/methods-uncertainty-quantification-computational-modeling-and-simulation-harmonizing

## 9. Implementation boundary

This document selects a protocol direction; it does not claim that v0.2 signing, trust roots, replay-resistant storage, key rotation or an independent verifier have been implemented. PR #347 remains open. The next implementation must add schema and golden vectors first, then two separately implemented verifiers, then signed fixtures, then protected key provisioning and persistent anti-rollback state. Do not enable D2/D3 or actuation as a side effect of adding cryptographic signatures.
