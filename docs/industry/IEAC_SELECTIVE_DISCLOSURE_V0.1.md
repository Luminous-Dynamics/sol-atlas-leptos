# IEAC selective-disclosure profile v0.1

**Status:** Research and request-contract tranche. No SD-JWT, BBS or zero-knowledge proof implementation is included.  
**Tracks:** [IEAC-PRIVACY-001 / #383](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/383).  
**Composes with:** ITIP federated evidence (#366), IEAC trust (#347), SCITT transparency (#374/#375), and event-history auditing (#376/#382).

## 1. Decision

Treat privacy as a property of the whole evidence path, not as a hash field or encryption checkbox.

- Keep raw source records and high-sensitivity measurements in an access-controlled evidence store.
- Put only approved identifiers, purpose-bound metadata and high-entropy commitments in SCITT statements.
- Let a holder disclose only the claims required for one decision case and one relying party.
- Preserve the meaning of the disclosed evidence: source measurement, issuer-signed derived assertion, and mathematical proof over hidden inputs are different proof classes.
- Keep source rights, provenance, uncertainty, freshness and context-of-use qualification bound to the disclosed claim.

SCITT RFC 9943 permits encrypted Signed Statements and discusses VDSs that retain cryptographic metadata rather than full statements. It also warns that timing and artifact relationships can leak through metadata even when the full statement is not retained. A digest-only log is therefore a data-minimization tool, not a complete privacy solution.

## 2. Standards maturity and intended use

### 2.1 RFC 9901 SD-JWT — first standards-track baseline

RFC 9901 defines selective disclosure of individual JSON claims through salted hashes in a signed JWT. The issuer keeps a selectively disclosable claim's cleartext out of the signed payload and gives a Disclosure to the holder; the holder presents only selected Disclosures to a verifier.

Important operational rules:
- use a cryptographically secure random salt with at least a 128-bit random component;
- use a distinct salt per claim;
- do not disclose unrevealed salts;
- verify each disclosed claim against the original signed SD-JWT;
- bind the presentation to the relying party, decision case, purpose, nonce and expiry using an explicit holder/key-binding design where the profile requires it.

A deterministic SHA-256 digest of a low-entropy value is not adequate privacy. An attacker can enumerate likely values. A high-entropy, independent salt prevents the straightforward dictionary attack described in RFC 9901, provided the salt remains secret.

SD-JWT discloses claims; it does not by itself prove a computation over undisclosed source data.

### 2.2 W3C BBS Data Integrity — experimental until mature

The W3C Data Integrity BBS Cryptosuites v1.0 document is a Candidate Recommendation Draft, latest publication 2026-09-10. It defines selective disclosure and unlinkable derived proofs using BBS signatures. The W3C publication history still labels it Candidate Recommendation Draft, and its advancement criteria require at least two independent implementations for each feature.

Treat BBS as an optional experimental profile:
- don't call it a final W3C Recommendation;
- don't assume an implementation's unlinkability without cross-implementation testing;
- don't use BBS disclosure as an arbitrary hidden range/aggregate proof;
- don't require BBS to interpret a basic SD-JWT claim.

The first production-oriented baseline remains RFC 9901 SD-JWT with a pinned verifier and exact profile policy. BBS can graduate after independent implementation and security review.

### 2.3 Three distinct proof semantics

Every requested claim must declare one of:

| Proof semantics | Meaning | What it does not prove |
|---|---|---|
| disclosed_signed_claim | The holder reveals a claim whose digest is covered by the issuer's signed SD-JWT/credential | That the source measurement is physically true |
| issuer_signed_predicate | The issuer signs an assertion such as “the reading was within the approved envelope” | That the issuer computed it correctly or that it is a zero-knowledge proof |
| zero_knowledge_predicate | A cryptographic proof establishes a defined predicate over hidden inputs without disclosing those inputs | Source truth, source calibration, or correctness of the trusted circuit/oracle |

A zero_knowledge_predicate must identify an actual supported proof system, exact circuit/program digest, public inputs, private-input commitment, verifier identity and conformance corpus. The request schema reserves an explicit external_predicate_proof_profile_pending value to make that non-availability visible. It does not pretend SD-JWT or BBS implements arbitrary predicate proofs.

## 3. Case-bound disclosure request

The strict Draft 2020-12 request contract is schemas/industry-evidence-disclosure-request-v0.1.schema.json; its synthetic fixture is schemas/examples/ieac-evidence-disclosure-request-synthetic-v0.1.json.

Each request binds:
- stable request and decision-case identifiers;
- a case-scoped subject pseudonym, issuer and relying-party identity;
- explicit purpose and IEAC profile digest;
- issuance time, expiry and a base64url nonce with at least 128 bits of required runtime randomness;
- a bounded list of requested claim IDs, each with semantic class, proof semantics, presentation profile, required/optional status and maximum disclosure bytes;
- non-negotiable privacy constraints: no raw source payload, case-scoped pseudonym only, salt holder-only and digest-only transparency payload;
- action_authority = none.

JSON Schema checks only syntax and bounded shape. Runtime must additionally:
- verify expires_at > issued_at and enforce a short profile-owned TTL;
- check nonce uniqueness, secure randomness and one-time consumption;
- reject duplicate claim_id values;
- confirm the relying party and issuer are authorized for the case, purpose and profile;
- ensure each claim is actually bound to its source artifact and current rights state;
- ensure disclosed data and uncertainty metadata are consistent with the signed source statement;
- reject unsupported presentation profiles rather than downgrading;
- bind the resulting presentation digest to the SCITT event while keeping values and salts out of the transparency payload.

The sample nonce is synthetic and predictable. It is a schema fixture, not a credential, random nonce or security example.

## 4. Privacy and trust threat model

### Claim and commitment leakage
- Dictionary-attack controls for low-entropy values and identifiers.
- Salt entropy, uniqueness, reuse, disclosure and exfiltration tests.
- Prohibit stable global subject identifiers in presentations where a case-scoped pseudonym suffices.
- Measure information leakage from proof sizes, claim paths, timestamps, issuer identities and event sequencing.

### Presentation binding
- Wrong case, subject, purpose, relying party, profile, nonce and expiry.
- Replay of a previously valid presentation in a new case.
- Duplicate or substituted claims, path confusion and holder over-disclosure.
- Expired, revoked or unauthorized signer credentials.

### Trust laundering
- A valid SD-JWT or BBS proof is not source truth.
- A signed predicate is not an independently verified computation.
- A zero-knowledge proof only establishes the statement encoded by its public inputs and verified circuit/program; it does not authenticate the truth of external sensors or the circuit's assumptions.
- Rights/consent, uncertainty and provenance must remain visible in the permitted disclosure or remain explicit blockers.

### Transparency metadata
- No raw measurement, source payload, personally identifying data, credentials or unrevealed salts in public SCITT statements.
- Treat digests of guessable values as public leakage unless protected by high-entropy independent salts or encryption.
- Consider timing/linkability across statements and transparency services.
- Keep the disclosure request and presentation privately retrievable; log only approved event metadata and digests.

## 5. Benchmark plan

Compare the same synthetic claims and decision cases under:
1. full cleartext signed evidence;
2. digest-only commitment with private evidence retrieval;
3. RFC 9901 SD-JWT selective disclosure;
4. W3C BBS selective disclosure/unlinkable presentation (experimental);
5. a separate predicate proof for threshold/range/aggregate cases.

Measure:
- positive claim verification and wrong-case/purpose/subject/nonce/expiry rejection;
- undisclosed-claim leakage, low-entropy dictionary-attack success, and salt-reuse detection;
- repeated-presentation linkability across different verifiers and cases;
- disclosure payload bytes and proof sizes;
- proof generation/verification p50/p95/p99, CPU and memory;
- transparency metadata exposure and cross-subject correlation;
- decision utility: fraction of IEAC cases evaluable from the minimum allowed claims;
- uncertainty, missingness, rights and provenance preservation after disclosure;
- cross-implementation parity for every supported cryptographic profile.

Initial conformance targets:
- 100% of golden positive SD-JWT vectors pass in two independent implementations;
- 100% of frozen case/purpose/subject/nonce/expiry substitutions fail closed;
- zero salt reuse in the corpus and zero dictionary recovery for 128-bit-random salts under bounded tests;
- zero raw claim values or salts in the transparency event payload;
- no presentation with unsupported proof semantics may receive a pass.

These are frozen test-corpus targets, not field privacy guarantees. Report exact corpus digests, cryptosuite/version, verifier executable digest, issuer keyset, policy and hardware with every result.

## 6. Claim ceiling and next stages

This tranche defines the request language and privacy constraints only. It does not:
- generate or verify SD-JWT disclosures;
- generate or verify BBS proofs;
- implement an arbitrary zero-knowledge predicate circuit;
- prove source measurement truth or derived-assertion correctness;
- grant admission, authorization or actuation.

The next implementation should add an SD-JWT issuer/holder/verifier test fixture using RFC 9901 vectors, then a distinct policy evaluator that binds the presentation to the IEAC decision case and SCITT event. BBS remains optional until two independent implementations and the current W3C conformance requirements are satisfied.
