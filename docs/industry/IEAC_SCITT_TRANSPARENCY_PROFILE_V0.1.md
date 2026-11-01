# IEAC SCITT transparency profile v0.1

**Status:** schema and synthetic interoperability fixture; no SCITT client or COSE/VDS verifier is implemented here.  
**Tracks:** [IEAC-TRANSPARENCY-001 / #374](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/374).  
**Stacks on:** [IEAC v0.2 signed-envelope profile PR #363](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/363).

## 1. Purpose and boundary

This profile defines a versioned IEAC event payload that can be carried as a SCITT Signed Statement. It adds a transparency and auditability layer over the existing IEAC event model. It does not replace the IEAC v0.2 signed envelope, the case-level ledger, or domain-specific verification contracts.

The key distinction is:

- **IEAC event payload:** what happened, to which case/subject, under which profile and authority scope, with exact artifact and build-provenance bindings.
- **SCITT Signed Statement:** the COSE-protected, issuer-authenticated statement carrying the exact JCS bytes.
- **SCITT registration receipt:** external evidence that a statement was registered in a transparency service; the relying party independently verifies the receipt and VDS proof.
- **Relying-party decision:** whether the exact event, history state, authority, freshness and source evidence are sufficient for a specific use.

A receipt is not a truth certificate. A valid signature is not proof that the issuer was authorized. A registered statement is not proof that the issuer submitted every relevant event.

## 2. Canonical event payload

The payload uses RFC 8785 JCS UTF-8 bytes and the profile identifier `luminous.ieac-scitt-event.v0.1`. The schema is strict Draft 2020-12 at `schemas/ieac-scitt-event-chain-v0.1.schema.json`. It rejects unknown event, artifact, causal-parent, build-provenance, fixture and header-binding fields. The `domain_payload` object is intentionally opaque: each vertical retains its own semantics rather than being flattened into a second generic IEAC DTO.

Every event binds:

- case ID, event kind, issuer, subject and authority scope;
- issuer-local monotonic sequence and issue time;
- predecessor digest for sequence greater than one, or an explicit initial checkpoint digest for sequence one;
- zero or more causal-parent references, each carrying issuer, issuer-local sequence and event digest;
- IEAC profile digest;
- exact SHA-256 and byte length for every disclosed artifact reference;
- source revision, evaluator digest, verifier digest, builder identity and build-attestation digest;
- evidence class and claim ceiling.

### 2.1 History semantics

Issuer-local sequence is not the SCITT registration order. Multiple issuers may submit events concurrently, and causal dependencies can cross issuer boundaries.

For each issuer and scope:

- sequence 1 requires an explicit externally anchored initial-checkpoint digest;
- sequence greater than 1 requires the exact predecessor event digest;
- same sequence with a different event digest is equivocation;
- lower sequence is rollback/replay;
- missing predecessor, checkpoint or required history range is incomplete/unknown, never an empty valid history;
- causal parents are separate from issuer-local predecessor links and may refer to another issuer.

The synthetic fixture contains three stages from two issuers: an operator request commitment, an auditor schedule-audit statement, and a later operator admission verdict. The admission verdict links to both the operator's preceding event and the auditor's event. The three event digests are recomputed from each event's JCS bytes in CI and every predecessor/causal-parent reference is checked against the corresponding event digest. Initial-checkpoint, artifact, IEAC-profile, build and attestation digests remain placeholders; this does not verify signatures, checkpoint authenticity, source truth or external evidence.

### 2.2 Synthetic JCS digest conformance

The fixture keeps its event JSON inside a deliberately narrow hash-test subset: ASCII keys and strings plus safe integer numbers. CI serializes primitives with JavaScript's ECMAScript JSON serialization, sorts property names by UTF-16 code units, and recomputes SHA-256 for each event. It then verifies both issuer-local predecessor links and cross-issuer causal-parent references against those computed digests. This catches accidental stale references when a fixture event changes.

This small fixture smoke test is not a general cross-language JCS implementation or an adversarial JCS suite. The production profile still requires RFC 8785 conformance vectors and independent Rust/Go verification before any signed statement is accepted.

## 3. COSE and SCITT mapping

The Signed Statement's COSE protected headers must include:

- protected CWT `iss` equal byte-for-byte to the event payload's `issuer`;
- protected CWT `sub` equal byte-for-byte to the event payload's `subject`;
- protected content type `application/vnd.luminous.ieac.scitt-event.v0.1+json`;
- a protected `kid` or certificate identity resolved under independently provisioned relying-party trust policy;
- a fully specified, permitted signing algorithm under the deployed COSE profile.

The existing IEAC v0.2 envelope and its domain-separated external AAD remain a distinct inner/adjacent artifact. Do not silently copy IEAC-specific AAD into the SCITT Signed Statement's COSE processing. The CWT claims and payload fields must match; schema validation alone cannot establish that match.

COSE receipts use the SCITT COSE unprotected-header receipt collection (label 394). Receipt bytes do not belong in the JCS event payload and must not be hashed into a self-referential event. The relying party verifies the receipt against independently provisioned transparency-service keys, checks receipt-to-statement binding and validates the relevant VDS inclusion proof before relying on the event. The fixture only records the expected boundary; it contains no valid COSE signature, receipt or proof.

## 4. Privacy and disclosure

Raw sensor payloads, personal data, credentials, private keys and sensitive evidence remain outside public statements. Events disclose approved metadata and exact commitments or references only. Low-entropy or guessable values must not be exposed as unsalted hashes and treated as private. A digest is an integrity reference, not encryption or access control.

Artifact references are role-labelled and byte-bound. Access-controlled evidence retrieval and encryption happen outside the transparency log under their own authorization contract. A relying party must distinguish public metadata, sensitive external evidence, encrypted external evidence and synthetic fixtures.

## 5. Initial trust profile and future composition

The initial profile assumes one configured SCITT service (D0/D1 descriptive transparency). This is enough to validate the wire profile and receipt verification path, but it does not independently prove global non-equivocation or completeness.

A higher-consequence profile may require two or more independently operated services, separate trust roots, cross-auditing of checkpoints and comparison of service views. Two receipts do not by themselves prove that the logs are consistent. Auditors must validate VDS proofs and compare views for fork/inconsistency evidence.

Registration and receipt verification are mandatory before a downstream relying party treats the specific event as relied-upon evidence. Request-commit and admission-verdict stages remain separate events. This is a transparency precondition, not actuation authority.

## 6. Benchmark and adversarial programme

Compare on an identical frozen event corpus:

1. local hash-only event ledger;
2. signed-only IEAC v0.2 envelope;
3. SCITT statements verified by Microsoft's `scitt-verifier`;
4. a second independently implemented RFC 9942/VDS verifier;
5. one-service versus two-service logging with independent trust roots.

Negative controls include omitted suffixes, missing predecessor/checkpoint, duplicate sequence with same and different digests, rollback, cross-subject and cross-issuer substitution, protected CWT/payload mismatch, wrong receipt-to-statement binding, invalid inclusion proof, inconsistent tree heads/fork, unavailable/stale receipt keys, expired/revoked signer policy, selective submission, partial service outage, replay and private-data leakage.

Measure:
- positive/negative vector detection and false receipt-to-statement bindings;
- time to registration, inclusion proof size and verification latency p50/p95/p99;
- statement/receipt bytes, CPU, memory, storage and egress;
- consistency-audit detection latency and multi-service disagreement;
- missing-sequence and omitted-suffix detection, bounded by independently anchored checkpoints;
- privacy leakage and metadata/linkability.

Targets are synthetic conformance targets, not operational risk bounds: all declared positive vectors verify, every frozen negative control fails closed, and zero rollback/fork or cross-subject receipt bindings are accepted. Do not claim event-history completeness without an independent checkpoint or mandatory receipt-before-use rule.

## 7. Claim ceiling

This profile can establish strict event payload shape and, after an independent SCITT implementation is integrated, can support cryptographic origin/integrity and transparency-registration checks for the exact statement. It does **not** establish source truth, physical-model validity, signer authorization without trusted provisioning, completeness of the issuer's submissions, globally consistent history without independent view comparison, operational safety, or actuation authority.

## 8. Standards and implementations

- IETF RFC 9943, SCITT architecture: https://www.rfc-editor.org/rfc/rfc9943.html
- IETF RFC 9942, COSE Receipts / Verifiable Data Structures: https://www.rfc-editor.org/rfc/rfc9942.html
- IETF RFC 9052, COSE structures and process: https://www.rfc-editor.org/rfc/rfc9052.html
- RFC 8785, JSON Canonicalization Scheme: https://www.rfc-editor.org/rfc/rfc8785.html
- Microsoft SCITT verifier: https://github.com/microsoft/scitt-verifier

**Implementation non-goals for v0.1:** no transparency service, Merkle/VDS generation, COSE receipt parser, production signer, key acquisition, key lifecycle or actuator integration is added. Reuse the standards and independent implementations; do not create a second SCITT client.
