# CTER Authority-Consumption and Contestation Profile v0.1

**Status:** synthetic consumer contract and reference evaluator; not an authority service or production authorization mechanism.  
**Owner:** Sol-Atlas CTER consumer boundary.  
**Tracked by:** [CTER-AUTH-CONSUMPTION-001 #951](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/951).

## 1. Do not duplicate the owners

The new profile deliberately composes existing work:

- [Mycelix LAND-RRR-INTEROP-001 #6315](https://github.com/Luminous-Dynamics/mycelix/issues/6315) owns LADM-aligned rights/authority references, jurisdiction mapping, multiple RRRs and the external legal-effect firewall.
- [Mycelix LAND-STEWARD-001 #5277](https://github.com/Luminous-Dynamics/mycelix/issues/5277) and [draft PR #5298](https://github.com/Luminous-Dynamics/mycelix/pull/5298) own stewardship records, separate measurement consent, site access and publication consent, and their structural safety gates.
- [Mycelix LEX-NET-028 #1215](https://github.com/Luminous-Dynamics/mycelix/issues/1215) owns the typed receipt algebra and anti-receipt-laundering contract.
- [Mycelix EPI-008 #2631](https://github.com/Luminous-Dynamics/mycelix/issues/2631) owns typed time/currentness semantics.
- [Mycelix CIVOS-EVIDENCE-001 #5709](https://github.com/Luminous-Dynamics/mycelix/issues/5709) owns rights-aware source/claim/derivation/dispute lineage.

CTER does not mint authority, issue a Mycelix AuthorizationReceipt, adjudicate a claim, create a community vote procedure, or maintain a parallel consent/identity/rights ledger.

## 2. Consumer theorem

For one exact request, the adapter checks whether the host has supplied typed, current, correctly scoped upstream decision outputs for:

- actor and subject/participant;
- exact operation;
- resource, project snapshot and site;
- purpose and named recipient;
- jurisdiction profile;
- explicit data-class set;
- policy and source-receipt lineage;
- signature/issuer-authority/revocation/currentness result;
- validity interval and bounded offline lease.

Only an upstream typed `AuthorizationReceipt` can satisfy an action prerequisite. A `TransportReceipt`, `RecognitionReceipt`, `TranslationReceipt`, `CapabilityLeaseReceipt` or `ExecutionReceipt` cannot be relabelled into action authorization.

The Rust evaluator and the synthetic fixtures consume host-resolved fields; they do not validate cryptographic signatures or authenticate the resolver. A production host must first verify source bytes, receipt signatures, issuer authority, consent, applicability, revocation and currentness through Mycelix's owners.

## 3. Operation-specific prerequisites

| Requested operation | Required decision domains |
|---|---|
| collect_measurement | participant_consent + site_access; add community_mandate for community/culturally protected data |
| publish_observation | publication_consent; add community_disclosure for community/culturally protected data |
| export_evidence | archive_export; add community_disclosure for community/culturally protected data |
| assert_legal_title | always DENY at CTER |

Measurement consent does not imply site access. Either does not imply publication consent. Individual consent is not a collective mandate. A community disclosure decision is not statutory registration.

The CTER check returns `eligible`, `deny`, `review`, `unknown` or `incomplete`. **Eligible means only that the consumer contract may proceed to the next separately enforced stage.** It is not a legal decision or permission by itself.

## 4. Community representation, dissent and contestation

For community/culturally protected information, the input must reference the applicable community governance profile, decision procedure, representation assessment and appeal route.

- Unresolved representation, known coverage gaps, material unresolved dissent or an open material challenge produce REVIEW.
- Unknown representation/currentness remains UNKNOWN.
- Recorded dissent is retained with a reference; it is not automatically treated as a veto.
- No leader, majority percentage, reputation, token holding or DID possession is hard-coded as a universal representation rule. The relevant community's own documented procedure and competent resolver own that decision.

CARE is Indigenous-specific data-governance guidance and must be applied as relevant peoples direct, not as a generic certificate. FAO VGGT is voluntary, non-binding guidance for tenure governance and does not cover water or mineral rights. Jurisdiction-specific law, customary protocols and competent authorities remain external inputs. [FAO VGGT](https://www.fao.org/land-water/land/land-governance-and-planning/land-resources-planning-toolbox/detail/voluntary-guidelines-on-responsible-governance-of-tenure-of-land-fisheries-and-forests-in-the-context-of-national-food-security-%28vggt%29/en) [CARE background](https://www.data.cam.ac.uk/policies-ethics-legal/care-principles).

## 5. Offline behavior

Every decision has a host-resolved currentness/revocation state, a validity interval and (for offline use) a bounded offline lease. Expired lease, stale status snapshot, unavailable resolver or unknown revocation is not a positive decision. The consumer returns UNKNOWN/INCOMPLETE and blocks new disclosure or mutation pending reconciliation. A relay cannot renew its own authority lease.

A missing receipt is not evidence that no underlying right, consent or decision exists.

## 6. Frozen corpus and claim ceiling

The synthetic suite at `benchmarks/territorial-integrity/relay/authority-cases-v0.1.json` exercises operation/type/scope laundering, class overreach, separate consent domains, contested representation, dissent, challenges, revocation, stale status, offline lease expiry, missing decisions and prohibited legal-effect assertions.

Target: 100% agreement between the Rust reference and an independent second-language oracle on frozen cases, 0 negative-control decisions marked eligible, and deterministic reason codes. These are qualification targets, not achieved product results.

No personal, claimant, sacred-site or real community information belongs in the public fixtures. The current suite is synthetic only and does not qualify any real community's representation or legal authority.

## 7. Principal and decision-maker separation

The request's `principal_ref` identifies the actor/system seeking to perform the operation. The receipt's `decision_maker_ref` identifies the person or institution whose decision is recorded, while `decision_authority_ref` identifies the upstream authority basis. These are distinct roles: a researcher does not become the consent-giver simply by being the collector; nor does the institution authorizing site access become the requester. The source resolver remains responsible for authenticating and validating these references and any proxy/guardian/community procedure.

An identical replay of the same `receipt_ref` and contents is idempotent. The same receipt reference with changed contents is an identity conflict and is denied. Different decision references for one required decision domain are reviewable conflicts, not a last-write-wins situation.

The Rust consumer additionally checks SHA-256-shaped project snapshot commitments and a strict UTC timestamp shape with Gregorian date bounds. JSON Schema validation remains mandatory; the standalone validator is defense in depth, not a schema replacement.

## 8. Three separate trust questions

The consumer projection distinguishes three facts which cannot substitute for each other:

1. `signature_status`: does the receipt's cryptographic signature verify over the exact signed receipt bytes?
2. `issuer_trust_status`: is the receipt issuer trusted under the relevant trust policy and current trust-anchor state?
3. `decision_authority_status`: did the referenced decision maker have authority for this exact scope, jurisdiction, purpose and operation?

A valid signature from an untrusted issuer is denied; an unavailable issuer-trust resolver is UNKNOWN; and a validly signed receipt whose decision-maker authority remains unresolved requires REVIEW. The upstream verifier/resolver must authenticate each status; CTER only consumes the normalized result.
