# IEAC SCITT transparency profile v0.1

**Status:** Proposed application profile and synthetic conformance fixture. No COSE implementation, transparency-service deployment, or production trust is included.  
**Tracks:** [IEAC-TRANSPARENCY-001 / #374](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/374).  
**Depends on:** [IEAC trust profile v0.2 / PR #363](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/363), [replayable case ledger / PR #362](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/362), [pairwise fault panel / PR #364](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/364), and [federated evidence references / PR #369](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/369).

## 1. Decision

IEAC should **reuse SCITT** for signed-statement transparency and receipt verification. It should not implement a proprietary append-only log, a new Merkle tree, or another receipt format.

The IEAC v0.2 signed envelope and the outer SCITT Signed Statement are different objects:

- The inner IEAC envelope authenticates the exact IEAC profile, authority, evidence, request, and/or evaluation artifact under IEAC's JCS + detached COSE profile and IEAC-specific domain separation.
- The outer SCITT statement is a SCITT-compatible COSE_Sign1 statement. Its detached JCS payload commits to the exact SHA-256 digest of the inner envelope and the related evidence/report artifacts. It follows the SCITT COSE conventions rather than inheriting IEAC's external AAD.
- A SCITT Receipt proves that a Transparency Service registered that exact Signed Statement in its Verifiable Data Structure. The Receipt is not a decision verdict.

This deliberately composes two cryptographic scopes instead of changing the inner IEAC signature contract and risking incompatibility with existing IEAC consumers.

## 2. Standards mapping

This profile targets the IETF SCITT architecture in [RFC 9943](https://www.rfc-editor.org/rfc/rfc9943.html) and COSE Receipts / Verifiable Data Structures in [RFC 9942](https://www.rfc-editor.org/rfc/rfc9942.html).

### 2.1 SCITT Signed Statement

The outer object is a tagged COSE_Sign1 Signed Statement using the media type `application/scitt-statement+cose`. The v0.1 application profile selects:

| COSE element | IEAC profile rule |
|---|---|
| COSE tag | Tag 18 / COSE_Sign1 |
| Payload | Detached RFC 8785 JCS UTF-8 bytes for the event payload |
| Protected CWT Claims header | Label 15 MUST contain `iss` (label 1) and `sub` (label 2) |
| `iss` | MUST equal the payload's `issuer_id` |
| `sub` | MUST equal the payload's `subject_id` |
| `alg` | Ed25519 COSE identifier `-19` for this v0.1 profile |
| `kid` | Protected key identifier resolved through the independently provisioned IEAC/issuer trust policy; certificate identity may be used only under a separately pinned certificate profile |
| `content_type` | `application/vnd.luminous.ieac.scitt-decision-statement.v0.1+json` |
| External AAD | Empty byte string for this profile; do not copy IEAC-specific domain-separated AAD into the SCITT statement |
| Receipt media type | `application/scitt-receipt+cose` |

The RFC 9943 architecture requires the protected CWT Issuer and Subject claims, and supports detached payloads. It also defines a Transparent Statement by adding one or more COSE Receipts to the unprotected header using label 394. A profile verifier MUST compare the protected `iss` and `sub` values with the JCS payload; JSON Schema alone cannot do this.

**Receipt placement:** Receipts belong in the SCITT COSE unprotected-header receipt collection (label 394). Do not put a receipt copy inside the JCS event payload. Keep the statement and each receipt as separately retrievable, independently verifiable objects.

### 2.2 Why an outer statement is necessary

The IEAC v0.2 envelope uses an IEAC-specific external AAD to separate profile pins, keysets, revocation snapshots and execution receipts. SCITT statement registration has its own COSE conventions and protected CWT claims. Reusing the inner envelope bytes as though they were already a complete SCITT statement would collapse these different protocol contexts.

Instead, the SCITT issuer signs a payload that binds the exact inner envelope bytes by SHA-256. The relying party verifies both layers:

1. Verify the SCITT Signed Statement signature and its `iss` / `sub` / content-type bindings.
2. Verify the SCITT Receipt inclusion proof against an independently trusted Transparency Service key.
3. Verify the digest binding to the exact IEAC envelope and other referenced artifacts.
4. Verify the inner IEAC signature, profile pin, authority snapshot, freshness and replay state.
5. Run the independent IEAC checker and relying-party policy.
6. Reconcile subsequent source outcomes through separately signed and registered outcome events.

A valid outer signature or Receipt does not skip any IEAC checks.

## 3. Event payload contract

The strict Draft 2020-12 schema is `schemas/industry-scitt-decision-statement-v0.1.schema.json`. It defines the JCS payload only; it is not a schema for COSE, a Receipt, a VDS proof or a transparency service.

The payload contains:

- stable `event_id`, `decision_case_id`, `issuer_id`, `subject_id`, and issuer-local `issuer_stream_id`;
- `event_kind` such as `request_committed`, `admission_verdict`, `authorization_record`, `execution_receipt`, `outcome_reconciliation`, `supersession`, or `revocation_checkpoint`;
- issuer-local positive `issuer_sequence` and `previous_event_payload_sha256`;
- cross-issuer `causal_parent_payload_sha256` links;
- the signed profile identity and digest;
- exact digest/media-type bindings for the inner IEAC envelope, evidence ledger, candidate report, independent verifier report, source snapshot, authorization record, build provenance or outcome record;
- an explicit `claim_ceiling = record_only` and `action_authority = none`.

The current schema makes the first issuer event's predecessor null and requires a SHA-256 predecessor for subsequent events. Only `admission_verdict` events carry a disposition. This is structural validation; the runtime auditor must still verify graph continuity, uniqueness, predecessor matching, issuer/subject scope, and artifact bytes.

### 3.1 Digest and ordering semantics

Define the event payload digest as:

`event_payload_sha256 = SHA-256(UTF8(RFC8785_JCS(event_payload)))`

The `previous_event_payload_sha256` chain is scoped to the tuple:

`(issuer_id, subject_id, issuer_stream_id)`

The same issuer-local sequence with the same digest is an idempotent duplicate. The same sequence with a different digest is an equivocation/conflict and MUST be rejected. A sequence gap or missing predecessor makes the stream incomplete/unknown; it is not an empty predecessor.

`causal_parent_payload_sha256` creates a directed acyclic event graph across issuers, such as:

`source gateway request → IEAC verdict → independent outcome record → later IEAC reassessment`

The causal graph is not the same as the global SCITT registration order. RFC 9943 explicitly warns that a relying party cannot assume a Transparency Service's registration order matches statement issuance order unless that service's Registration Policy advertises this property. Use the signed issuer-local sequence and causal links as the application semantics; use the Receipt's registration metadata as a separate log fact.

## 4. Completeness, omissions and equivocation

A valid Receipt establishes inclusion of one Signed Statement. It does **not** prove that the issuer submitted every statement it created. RFC 9943 explicitly allows an issuer to refuse registration or selectively submit statements. Therefore:

- Never infer a complete history from one valid Receipt.
- For every decision event that a relying party will consume, require the configured SCITT Receipt(s) before that event is used.
- A future control-plane integration should commit the request before evaluation and commit the verdict before a relying party accepts it. This makes logging a precondition to reliance; it does not grant execution authority.
- Publish signed, independently anchored stream-head checkpoints at a declared cadence, and audit sequence/predecessor continuity against them. A checkpoint is only as trustworthy as its issuer and independent anchoring process.
- Compare VDS consistency proofs / signed tree heads between independent auditors to detect log equivocation. Do not treat a second Receipt as a substitute for consistency checks.
- For any future higher-consequence profile, consider two or more independently operated Transparency Services with distinct operators, trust roots and operational custody. The D0/D1 descriptive profile starts with one configured service; D2/D3 remain outside the current admission boundary.

A missing Receipt, unavailable keyset, unanchored checkpoint, inconsistent tree head or missing predecessor is `unknown / incomplete` and cannot become a pass.

## 5. Trust and privacy boundaries

### 5.1 Distinct trust domains

Issuer signing keys and Transparency Service receipt-signing keys are distinct authorities. They require separate identities, independently provisioned trust roots, rotation/revocation policy, validity periods, and freshness controls. The IEAC root/keyset work in #347 does not automatically establish trust in a SCITT service's receipt key.

A relying-party policy must also distinguish the two issuer namespaces in the SCITT COSE objects:

- The protected CWT `iss` claim identifies the **Signed Statement issuer** (the IEAC producer).
- The `iss` claim carried by a **COSE Receipt** identifies the Transparency Service that issued the receipt.

These are not interchangeable. Pin both independently, constrain the protected CWT `sub` to the expected asset or decision subject, and require the configured receipt count / service set. In the Microsoft verifier's policy vocabulary, `issuer` constrains receipt issuers, while `statementIssuer` constrains the statement's CWT issuer; a policy that pins only `issuer` does not authenticate the IEAC statement signer. Receipt presence alone is not enough: the receipt must cryptographically bind to the exact statement and verify under the independently pinned service trust material.

SCITT registration is not a substitute for signer authorization. A signature proves that a key signed bytes; the relying party still decides whether that key is authorized for that event, subject, scope, and decision profile.

### 5.2 Privacy

Do not send raw measurements, confidential source payloads, personal data, credentials, or operational secrets to a public transparency service. The payload should contain only the minimum approved metadata and cryptographic commitments; encrypted evidence remains in a separately access-controlled repository.

A plain hash is not a privacy mechanism for a low-entropy or guessable value. Where a commitment could be dictionary-attacked, use a high-entropy secret salt/nonce whose preimage and access policy remain protected, or commit to an encrypted object. Subject identifiers should be scope-limited pseudonyms where appropriate; stable global subject IDs can create cross-system linkability.

The schema's `digest_only` and `encrypted_external_reference` values state disclosure intent, not proof that the referenced bytes are encrypted, private, licensed, or authentic.

## 6. Failure and recovery semantics

| Condition | Required result |
|---|---|
| SCITT statement signature invalid or unsupported | Reject / cannot evaluate |
| Receipt absent when policy requires it | Abstain; do not treat as transparent |
| Receipt inclusion proof invalid | Reject |
| Receipt valid but for a different statement | Reject |
| Receipt keyset untrusted, stale, revoked or unavailable | Unknown / cannot evaluate; no pass |
| Protected `iss` / `sub` differs from payload | Reject |
| Inner IEAC digest does not match exact artifact bytes | Reject |
| IEAC signature, authority, freshness, replay or independent-check fails | Follow IEAC fail-closed policy |
| Sequence rolled back, same sequence has a different digest, or predecessor is missing | Reject / incomplete |
| Log fork or inconsistent consistency proof | Reject the affected log view and escalate to independent audit |
| One configured service unavailable | No receipt-dependent pass; explicit degraded status only |
| Two configured services disagree | Abstain/reject according to the relying-party policy; preserve both proofs |

A log's availability, registration delay, and key-rotation policy are operational parameters to measure, not assumptions.

## 7. Industry comparison and integration boundary

### SCITT and existing verifier implementations

The IETF architecture already standardizes the core Signed Statement, Transparency Service and Receipt vocabulary. Microsoft publishes `scitt-verifier`, an offline-by-default Rust CLI that verifies statement signatures, Receipt inclusion, binding to the statement, and a separately supplied relying-party policy. Its documented receipt support currently covers `CCF_LEDGER_SHA256`; it refuses `RFC9162_SHA256`, and its limitations explicitly include no revocation checking.

IEAC should use this as an external baseline / interoperability target, not fork the basic COSE receipt verifier. The differentiator is the industrial layer on top: source-native evidence, profile/authority qualification, issuer-local causal history, independent admission checking, selective abstention, and measured outcome reconciliation. A second independently implemented verifier is still required for cross-implementation assurance.

### Physical-site engagement receipts

The September 2026 Physical-Site Engagement Receipt Internet-Draft is a work in progress and a useful adjacent example of applying SCITT to real-world operations. It explicitly avoids claiming that a recorded engagement was safe/correct or that site conditions were as recorded. It also explains that a locally maintained chain or TEE does not prove that the history is complete; external SCITT registration is required by that draft's profile.

IEAC is complementary, not a replacement: it links the source evidence, evaluated decision, independent verifier, authorization boundary and later outcome. The IEAC profile must preserve the same narrow-claim discipline and add deterministic decision replay; it should not copy the draft's site/TEE vocabulary.

### TUF and anti-rollback

The Update Framework (TUF) remains a relevant design reference for trust-metadata distribution: threshold roots, role separation, version monotonicity, expiry and freeze/rollback checks. TUF protects software update metadata; SCITT provides statement transparency. IEAC's signed keysets and high-water state should borrow applicable anti-rollback discipline without conflating the two protocols.

## 8. Benchmark plan

The first fixture is deliberately small and hand-checkable: four synthetic events from three issuers, three causal edges, and one issuer stream that advances from sequence 1 to 2. The second IEAC verdict abstains, illustrating that a later event can supersede an earlier descriptive admission without erasing the earlier history.

Compare identical statements and corpora across:

1. SHA-256-only local case ledger (#362).
2. Signed-only IEAC envelope (#363).
3. SCITT statement and Receipt verification using Microsoft's `scitt-verifier`.
4. A second independent RFC 9942 VDS/Receipt implementation.
5. One versus two independently operated Transparency Services.

Frozen positive and negative controls must include:
- valid statement/Receipt, correct Receipt-to-statement binding, and valid detached payload digest;
- missing Receipt, wrong-statement Receipt, invalid inclusion proof, unsupported VDS proof;
- protected CWT `iss` / `sub` mismatch, `kid` mismatch, wrong content type, invalid COSE signature;
- missing predecessor, missing suffix, duplicate sequence/same digest, duplicate sequence/different digest, rollback, causal cycle, cross-case/subject substitution;
- inconsistent tree heads / fork, stale or unavailable receipt keyset, service outage, selective submission, and key rotation/revocation;
- public-field mutation, salted-commitment mutation, dictionary-attack controls, and accidental raw-evidence disclosure.

### Promotion targets

| Measure | Initial target |
|---|---|
| Golden positive vectors | 100% verified by both verifier implementations |
| Frozen negative controls | 100% fail closed; no unsupported proof format may pass |
| Receipt-to-statement binding | Zero false bindings |
| Issuer sequence / predecessor controls | Zero accepted rollback, same-sequence equivocation, or missing-predecessor cases |
| Log consistency | 100% detection of planted fork/inconsistent-proof cases in the bounded corpus |
| Completeness claims | Only claim complete over an independently anchored checkpoint or mandatory receipt-before-use boundary |
| Cross-implementation parity | Identical result classifications on the shared supported corpus; unsupported cases remain explicit |
| Performance | Record p50/p95/p99 registration, receipt verification, event-graph verification; proof/statement size; CPU/memory; storage/egress; and outage behavior. Freeze numerical SLOs only after pinning service, VDS, proof size, and hardware. |

The 100% targets apply only to frozen conformance vectors. They are not measured field-risk bounds. Report the corpus digest, verifier versions/build digests, TS identity, receipt-keyset digest, trust policy, proof format, run seed, and exact source revisions with every result.

## 9. Claim ceiling and non-goals

This profile does not:
- prove that a measurement is physically true or that a source is honest;
- establish model validity, calibration, VVUQ or engineering fitness-for-purpose;
- prove that an issuer registered every statement it created;
- establish a complete history without a trusted checkpoint or a receipt-before-use enforcement boundary;
- make statement issuance time equal transparency registration time;
- authenticate a signer or receipt key without independent trust provisioning;
- implement a Transparency Service, Merkle tree, COSE Receipt verifier, trust-key acquisition or live connector;
- grant action or actuation authority.

A transparent record is more auditable, not automatically more correct. D0/D1 remain descriptive; D2/D3 and actuation remain rejected until their separate evidence and authority gates are satisfied.

## 10. References

- RFC 9943, SCITT architecture: https://www.rfc-editor.org/rfc/rfc9943.html
- RFC 9942, COSE Receipts and Verifiable Data Structures: https://www.rfc-editor.org/rfc/rfc9942.html
- RFC 8785, JSON Canonicalization Scheme: https://www.rfc-editor.org/rfc/rfc8785.html
- RFC 9864, fully specified COSE algorithms: https://www.rfc-editor.org/rfc/rfc9864.html
- Microsoft SCITT verifier: https://github.com/microsoft/scitt-verifier
- SCITT Physical-Site Engagement Receipt Internet-Draft (work in progress): https://datatracker.ietf.org/doc/draft-wilder-scitt-physical-site-engage-receipt/
- The Update Framework specification: https://theupdateframework.github.io/specification/v1.0.36/
