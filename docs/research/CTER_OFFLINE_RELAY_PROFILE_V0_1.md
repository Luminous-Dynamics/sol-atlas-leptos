# CTER Offline Relay Profile and Convergence Benchmark v0.1

**Initiative:** Community Territorial Evidence Relays (CTER)  
**Parent benchmark:** Territorial Integrity Benchmark (TIB), Sol-Atlas issue #937  
**Implementation issue:** [CTER-RELAY-QUAL-001 #945](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/945)  
**Implementation:** [Draft PR #946](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/946)  
**Status:** research contract + synthetic reference model; no production transport or field qualification claimed  
**Date:** 19 November 2026

## 1. Executive thesis

CTER is not differentiated by offline form capture alone. FAO Open Tenure already documents offline mobile collection, encrypted local storage, multilingual forms, participatory validation, dispute support, and synchronization with a community server. Cadasta documents offline-ready geospatial collection, secure storage, role-based sharing, and integration with official land systems.

The research target is harder and narrower:

> Preserve the integrity, agency and contestability of territorial evidence while independent parties submit, carry, copy, delay, replay and reconcile records across disconnected and partially trusted relays.

A relay is a courier and custodian of encrypted packages. It is not an adjudicator, authority issuer, evidence validator or owner of the community's records.

## 2. Paradigm shifts

### 2.1 From offline forms to partition-tolerant evidence networks

Offline capture is an established capability. CTER must specify what happens when claimants, objectors, community groups and couriers create or carry records while disconnected. The network must preserve concurrent events and expose the need for review; it must not silently pick a winner because one device synchronized last.

### 2.2 From last-write-wins to contest-preserving convergence

Use the existing Mycelix-DeSci signed event envelope and append-only stream as the canonical event wrapper. CTER must not create a second signed event format, hash chain, event ledger or global consensus layer. Verification assessments, verifier identities/policy versions, key-status snapshot references, time-basis claims, access annotations and projection hints are detached contextual records; they must never alter the immutable signed-event identity.

Within a Mycelix claim stream, sequence numbers and `previous_hash` establish a single predecessor chain. Two verified events that claim the same `stream_id` and sequence are a fork and must be preserved/quarantined rather than resolved by arrival order. A verified event is also held when a same-slot competitor is awaiting verification or has unresolved revocation status; otherwise an event delivered first could win before the competing signature is checked. A definitively invalid-signature competitor does not poison the valid chain. Separate streams that project onto the same territorial relationship can both be retained and surfaced as a conflict.

The relay's event-set merge is deterministic set union over event identity and signed-event hash. It must be commutative, associative and idempotent. Those algebraic properties make duplicate delivery and reordering safe at the envelope layer; they do not resolve the underlying dispute.

### 2.3 From wall-clock ordering to stream causality

The canonical Mycelix envelope includes `occurred_at`, but that is the signer's claimed time. A server-side `AppendReceipt.received_at` is a separate receipt time. Neither a device wall clock nor relay arrival order may replace `sequence` and `previous_hash` for event-chain causality.

A missing predecessor hash leaves an event pending. A time claim cannot fill the gap. Relay custody times are separate observations and must retain the relay identity, clock source and uncertainty.

### 2.4 From "delivered" to typed receipts

Keep distinct receipts for:

- **Local receipt:** the submitting device durably recorded an envelope.
- **Custody receipt:** a relay accepted responsibility for carrying an identified package.
- **Destination receipt:** a named destination received an identified package.
- **Integrity receipt:** a verifier checked exact bytes/digests using a named algorithm and build.
- **Signature-verification result:** the canonical Mycelix signed-event envelope passed or failed cryptographic verification.
- **Authority decision:** an independent governance/authority service evaluated whether the actor was entitled to perform that specific action.
- **Community attestation:** an authorized participant made a bounded assertion.
- **Institutional determination:** a competent institution recorded a decision under applicable law.

No earlier receipt implies a later one. In particular, transport delivery is not authenticity, truth, community endorsement, consent, actor authority, legal effect or registration.

### 2.5 From public maps to purpose-bound disclosure

The relay must enforce the access policy associated with each payload/artifact, not trust a label supplied by an untrusted courier. A courier may be allowed to carry a sealed object without being allowed to inspect it. An authorization to read a generalized map does not authorize bulk export of household points, testimony, sacred locations or active disputes.

The CTER benchmark uses disclosure classes `public`, `community`, `restricted` and `culturally_protected` as policy annotations. These are not new Mycelix event-envelope fields. They must resolve to the owning artifact/consent/access policy. Mycelix's existing `ArtifactAvailability` states (`Public`, `Controlled`, `Embargoed`, `Unavailable`) remain distinct and must not be silently conflated with community disclosure categories.

### 2.6 From a central online service to community-operable infrastructure

A community should be able to operate a local node without calling a Luminous-hosted endpoint. Devices should support local backup, export and recovery. Interoperability must not require a single central database or token economy.

## 3. Existing canonical event owner and cross-system boundaries

### Mycelix owns the signed event envelope

The public Mycelix-DeSci implementation already defines `ScientificEventEnvelope` and `SignedScientificEvent`:

- `event_id`: UUID-backed event identity;
- `stream_id`: claim-specific stream identity;
- `sequence`: sequence within that stream;
- `previous_hash`: optional hash of the immediately preceding signed event;
- `actor` and optional `acting_organization`;
- `occurred_at`, idempotency key and typed payload;
- versioned canonical codec and Ed25519 signature;
- `event_hash()`: BLAKE3 `ContentHash` over the signed event.

The envelope validator already requires genesis events to have no previous hash and non-genesis events to carry one. The event-log API returns a separate append receipt including stream, sequence, event ID, event hash and server `received_at`.

Signature verification proves integrity of the signed envelope. Mycelix explicitly keeps actor/key authorization as a separate governed append concern. CTER must preserve this distinction.

**Digest firewall:** Mycelix event-chain `ContentHash` is BLAKE3. Sol-Atlas external source/record references use SHA-256 in the territorial profile. Both are 32-byte/64-hex digests but they have different domains and meanings. Never substitute one for the other, infer one from the other, or label an algorithm-less hash string as an interchangeable digest.

### System ownership

| Layer | Owner | Responsibility | Explicit non-responsibility |
|---|---|---|---|
| Territorial semantics | Sol-Atlas | LADM-aligned rights, responsibilities, restrictions, spatial/temporal references and source lineage | Does not adjudicate a claim |
| Signed event identity and authority | Mycelix | Canonical signed envelope, stream chain, actor identity, consent, authority, revocation and attestations | Does not turn DHT consensus into legal registration |
| Relay convergence | CTER profile | Store-carry-forward, custody receipts, replay handling, partition reconciliation and access-policy enforcement | Does not decide legal priority |
| Evidence reasoning | Symthaea | Read-only synthesis, contradictions, uncertainty, source citations and abstention | Does not create or approve a right |
| Environmental observations | Symtropy | Read-only, snapshot-consistent physical/ecological observations and model outputs | A model output is not a measured outcome |
| Geospatial container | OGC GeoPackage | Portable offline feature/raster/tabular packages | A GeoPackage is not an access-control or legal model |
| Transport | BPv7 candidate profile | Store-carry-forward through intermittent and physically mobile links | Transport security is not application authorization |

### Transparency and receipt integration: reuse SCITT, do not build another ledger

IETF published RFC 9942 (COSE Receipts) and RFC 9943 (SCITT architecture) in June 2026. These are now the current standards basis for an optional transparency adapter.

- RFC 9943 defines signed-statement registration through a Transparency Service (TS), explicit auditable registration policy, append-only/non-equivocating/replayable verifiable data structures and COSE receipts.
- RFC 9942 defines COSE-carried VDS proofs, including inclusion, consistency and selective-disclosure proof families.
- Mycelix already owns the relevant receipt, split-view and witness qualification work: LEX-NET-028 (#1215), the SCITT checkpoint anti-equivocation issue (#5942) and model PR #5944. CTER consumes the exact typed Mycelix receipt/checkpoint references; it does not create a second VDS, receipt hierarchy, witness service or global event ledger.

**Claim boundary:** a COSE/SCITT receipt can establish that a particular signed statement was registered under a particular TS/VDS and policy. It does not establish that the statement is true, that its issuer had legal authority, that a community mandate is valid, or that an underlying right exists. A missing receipt is not proof that an event or claim does not exist: an issuer may not have submitted it, a TS may not have accepted it, or the client may not have reached the TS.

**Privacy boundary:** do not send raw claimant identities, testimony, household coordinates, sacred/culturally protected locations or unredacted claim payloads to a public TS. For a transparency commitment, prefer a domain-separated, versioned commitment over exact encrypted/archive bytes using a fresh high-entropy random nonce held with the protected archive. Do not publish unsalted hashes of low-entropy content: they invite dictionary guessing. Even a hash-only TS can reveal issuer, timing, volume and relationship metadata; assess that leakage before registration. The selected SCITT profile must state what bytes and metadata the TS actually receives.

**Split-view boundary:** a TS signature alone does not prove that all clients were shown one consistent latest view. Require the Mycelix checkpoint/witness profile to resolve same-log/same-size divergent roots; if its quorum/consistency evidence is unavailable, report UNKNOWN/INCOMPLETE and preserve both observations. Distinct endpoints do not imply independent operators or failure domains.

### Transport decision

Do not invent a radio protocol or proprietary store-and-forward transport. Evaluate IETF Bundle Protocol v7 (RFC 9171), taking account of its updates RFC 9713 (administrative record type registry) and RFC 9758 (IPN URI scheme). Evaluate BPSec (RFC 9172) and its default security contexts (RFC 9173) where bundle-layer integrity/confidentiality is appropriate. Select an interoperable implementation only after a compatibility and threat-model review. Also benchmark simpler HTTPS synchronization and encrypted removable-media exchange. An RFC is a protocol specification, not proof that a selected implementation is interoperable, secure or suitable for a particular radio.

The v0.1 Rust reference model does not implement BPv7, BPSec, cryptography, key storage or radio communications. It models relay-side convergence of Mycelix-shaped event references after a separate verifier has supplied a signature/key disposition and canonical BLAKE3 event hash.

## 4. Relay benchmark adapter fields

The benchmark adapter reads these canonical event fields from Mycelix. Each delivery must also carry/reference the full serialized SignedScientificEvent bytes under payload.blob_ref and bind their exact bytes with payload.blob_sha256 (SHA-256). This raw-blob digest is not the event identity. Verifier reports and policy annotations are carried separately from the signed envelope:

| Field | Meaning |
|---|---|
| `event_id` | Stable UUID event identity |
| `stream_id` | Claim-specific stream |
| `sequence` | Sequence within the claim stream |
| `previous_hash` | BLAKE3 hash of the prior signed event; absent only for genesis |
| `event_hash` | BLAKE3 hash of the signed canonical event, supplied by the independent Mycelix verifier |
| `payload.blob_ref` + `payload.blob_sha256` | Exact serialized signed-event bytes to carry/restore plus SHA-256 over those bytes; not interchangeable with BLAKE3 event identity |
| `assessment` (detached) | Verifier ID, pinned policy ID, assessment time, key-status snapshot and event-time basis; multiple assessments can coexist |
| `context` (detached) | Projection key and access-policy class; disagreements block projection/disclosure until reviewed |
| `payload.blob_sha256` | Exact-byte commitment for the serialized signed-event payload; divergent payload commitments for one event identity are quarantined |
| `actor` | Canonical event actor identity |
| `verification` | External result: `verified`, `invalid`, `revoked`, or `unavailable` |

The test corpus additionally carries:
- `projection_key`: benchmark-only key describing which derived territorial view an event may affect; it is not an ownership key or canonical envelope field.
- `access_class`: CTER policy annotation resolved against the owning Mycelix artifact/consent policy; it is not a substitute for authorization.

The reference crate checks digest shape, identity collisions, stream sequence and hash-link consistency. It aggregates detached assessments conservatively: policy/status disagreement requires review, while independent verifiers agreeing under the same pinned policy do not create a new event. Context disagreement is surfaced and blocks projection use. It does not recompute BLAKE3, parse Ed25519 signatures, authenticate verifier assessments, verify actor-to-key binding, check authority, resolve consent or enforce confidentiality. A passing structural result is not an authorization.

## 5. Normative convergence behavior for v0.1

1. An exact replay of the same `event_id` and `event_hash` is idempotent.
2. The same event ID paired with different event hashes or different canonical envelope content is equivocation. Keep every variant in quarantine; accept none as the unique event. Detached verification or access-context disagreement is not event equivocation; it produces a separate review state.
3. Reuse of a BLAKE3 event hash under different event IDs is an integrity collision and must be quarantined.
4. The same event identity delivered with different exact serialized-payload SHA-256 digests is a payload-binding conflict; do not accept either delivery for projection until reconciled.
5. Invalid or revoked signatures are never accepted. Unavailable verification remains pending, not accepted.
6. Two verified events claiming the same `stream_id` and `sequence` are a stream fork. Preserve both; do not choose by arrival time. If a same-slot competitor has unavailable verification or unresolved revocation status, hold every candidate in that slot pending verification/review. A definitively invalid signature does not block a verified event.
7. A missing `previous_hash` predecessor leaves the event pending. It may be reconsidered when the predecessor arrives.
8. A child of an invalid, revoked, unavailable, equivocated or forked predecessor is blocked pending review.
9. A predecessor must belong to the same stream and have sequence exactly one less than the child. Cross-stream or wrong-sequence links are blocked.
10. Genesis sequence zero has no previous hash; non-genesis events must have a previous hash.
11. Concurrent accepted events from different streams that affect the same declared projection key are preserved and reported in a compact conflict group. The group contains only the latest accepted event per stream for that projection key; prior same-stream events remain historical evidence.
12. Corrections append new signed events; they do not erase the earlier event or the fact it was received.
13. No deterministic display sort, timestamp or relay count may be interpreted as legal priority.
14. Rebuilding the same projection from the same normalized event set and profile version must produce the same result.
15. Verification status, verifier policy version, key-status snapshot and access annotations are detached from the signed event. Conflicting assessment policies/statuses require review; a later result never rewrites the source event.
16. A key-validity conclusion cannot rely solely on the event's signer-controlled occurred_at; a trusted receipt or external timestamp basis is required.
17. Conflicting access/projection annotations are retained as contextual uncertainty and cannot be resolved by last-write-wins or a less restrictive label.
18. A payload digest mismatch for one immutable event identity is a payload-binding conflict, not a new event and not an acceptable replay.
19. Conflict groups contain only the latest accepted event per stream for the relevant projection key; earlier same-stream events remain in the append-only history.

The reference model's `accepted_events` means only structurally eligible for evidence preservation under the supplied signature/key dispositions and chain links. It does not establish truth, signer authority, community mandate, consent, representativeness or legal effect.

## 6. Threat model and failure injection

| Threat/failure | Required behavior |
|---|---|
| Lost, duplicated, reordered or delayed package | No silent loss; duplicates idempotent; late predecessor unblocks pending events |
| Malicious courier | Cannot read restricted payloads or alter accepted content without detection at the receiving verifier |
| Compromised/stolen device | Revoke/rotate future signing authority; preserve prior receipts and mark time uncertainty |
| Same event ID, different event hash | Quarantine all variants as equivocation |
| Same stream/sequence reused | Preserve and surface fork; do not choose first/last |
| Same event hash under different event IDs | Quarantine as integrity collision |
| Clock skew or rollback | Preserve clock observation and uncertainty; do not use it for chain causality |
| False/unavailable signature verification | Invalid/revoked never accepted; unavailable remains pending |
| Valid signature, unauthorized actor | Integrity may pass; action authority must remain a separate fail-closed check |
| Relay data loss | Recovery from independent copies and verifiable manifests; missing frontier explicit |
| Visibility downgrade | Reject export/delivery to a destination whose authorization does not cover the payload |
| Bulk export or inference | Rate limits, scoped export receipts, minimization and leakage tests |
| Device power loss mid-write | Transactional durability and recovery without partial promotion |
| Conflicting community representations | Preserve competing mandates and decision records; do not use popularity/reputation to settle authority |
| Key rotation/revocation during partition | Preserve historical verification context; new signatures require current policy; ambiguous effective time is quarantined |
| Bundle expiry/TTL | Transport expiry may stop carriage; it does not expire or extinguish the underlying claim |

## 7. Benchmark programme

### TIB-RELAY-A — convergence correctness, signed-payload custody and detached assessment integrity
- Missing signed-envelope payload reference is rejected.
- Payload blob missing/invalid SHA-256 is rejected.
- Distinct local blob references to the same event do not create event equivocation when BLAKE3 event identity and payload bytes independently verify.

- Same signed event delivered with verified then unavailable assessment.
- Same event marked verified using only signer-claimed occurred_at remains pending.
- Same key marked revoked without trusted event-time evidence remains pending rather than being treated as a historical revocation conclusion.
- Same signed event assessed under incompatible policy versions.
- Independent verifiers agree under one pinned policy.
- Access class changes between deliveries; no downgrade is applied.
- Same event identity/hash with changed signed envelope actor/payload.

- Exact duplicate delivery.
- Duplicate plus reordered batches.
- Same event ID / different event hash.
- Same BLAKE3 event hash / different event IDs.
- Same stream/sequence fork.
- Missing predecessor, then late arrival.
- Invalid, revoked and unavailable signature result.
- Child linked to invalid parent.
- Wrong stream or wrong predecessor sequence.
- Concurrent streams projecting onto the same territorial relationship.
- Causally ordered correction in one stream.

### TIB-RELAY-B — hostile transport
- Bit-flipped payload.
- Replayed bundle identifier.
- Replayed valid old event.
- Unknown/revoked key.
- Compromised relay with valid custody identity but no content authority.
- Downgrade of disclosure class.
- Package truncation and interrupted write.
- Malformed manifest and oversized payload.

### TIB-RELAY-C — partition operations
- 24-hour, 7-day and 30-day partitions.
- Multi-hop store-carry-forward with missed contacts.
- Asymmetric links and low bandwidth.
- High loss, duplication and reordering.
- Removable-media round trips.
- Recovery after power loss and node replacement.

### TIB-RELAY-D — privacy and user agency
- Household-coordinate leakage through exports/logs/indexes.
- Culturally protected feature accidentally generalized or exposed.
- Purpose mismatch and revoked publication consent.
- Participant withdrawal while downstream copies exist.
- Dissent submitted without local-leader permission.
- Receipt retrieval without revealing witness identity to a courier.

### Metrics and targets

| Measure | Research target |
|---|---:|
| Silent event loss across frozen partition suite | 0 |
| Silent overwrite / automatic legal-priority selection | 0 |
| Exact replay idempotence | 100% |
| False acceptance of invalid, revoked or unavailable verification | 0 |
| Event-ID equivocation and hash-collision detection | 100% on frozen cases |
| Payload-byte commitment consistency per event identity | 100% of divergent payload digests detected |
| Conflicting verifier assessment withholding | 100% on frozen cases |
| Conflicting access-context downgrade block | 100% on frozen cases |
| Stream/sequence fork detection | 100% on frozen cases |
| Ambiguous same-slot verification hold | 100% of verified candidates withheld until all competing signatures resolve |
| Invalid-signature isolation | 100% of definitively invalid competitors excluded without poisoning a valid event |
| Preservation of active conflict records | 100% |
| Deterministic replay of same normalized event set | 100% |
| Independent reference-oracle agreement | 100% on frozen cases |
| Unauthorized restricted-data disclosures in adversarial corpus | 0 |
| 30-day disconnection with full local submission/receipt workflow | Demonstrate, then publish observed completion and recovery |
| p50/p95/p99 receipt latency, sync convergence time, bytes/event, energy/event, storage growth | Measure on declared devices/network schedules; set targets only after baseline |
| Community task success, comprehension and perceived control | Co-designed field study; never infer from synthetic tests |

Do not collapse the metrics into a single score. Report each failure mode, false positive, abstention, hardware, network trace, payload size, software version and excluded case.

## 8. Fair industry comparison

| Comparator | Documented strength | CTER comparison rule |
|---|---|---|
| FAO Open Tenure / Community Server | Offline mobile capture, encrypted local DB, multilingual support, participatory validation, disputes and sync | Do not claim these are novel; test stream-fork/replay/chain-gap guarantees with matched fixtures |
| Cadasta Platform | Offline-ready GIS collection, secure cloud storage, role-based sharing, integration with national systems and community data protocols | Do not imply lack of offline or sovereignty features; distinguish documented capability from hands-on measurement |
| OGC GeoPackage | Portable SQLite-based vector/raster/tabular geospatial container usable on mobile with limited connectivity | Reuse as a spatial data container; benchmark read/write/copy cost and integrity, not as a land-rights protocol |
| LADM ISO 19152-1:2024 | Shared model for parties, BAUnits, RRRs, spatial units, sources and versioned objects | Map CTER payloads to LADM identities; preserve jurisdictional extensions and unmapped concepts |
| BPv7 / BPSec | Standardized store-carry-forward transport and bundle-level security building blocks | Reuse/implement interoperably where feasible; CTER adds land-evidence relay policy above transport |
| Mycelix-DeSci Scientific Events | Canonical signed event envelope, per-claim stream, BLAKE3 hash chain, idempotency and append receipts | Reuse as the event system; benchmark relay convergence around it, not against a competing ledger |

Hands-on superiority claims require comparable versions and environments. If a product cannot be run or the user population differs, report a documented-capability comparison, not a measured score.

## 9. Implementation sequence

1. Reuse the Mycelix canonical signed envelope and document the exact adapter mapping.
2. Freeze the relay convergence profile and synthetic corpus.
3. Run an independent second-language oracle against the exact corpus.
4. Add exact-commit CI for the standalone reference package; queued/skipped is not pass.
5. Extend the existing TIB report with operational measures and a matched Open Tenure/Cadasta comparator plan.
6. Prototype a local relay using commodity Android/Linux devices and an encrypted local store.
7. Evaluate BPv7/BPSec implementations against HTTPS and removable media on identical network traces.
8. Add Mycelix authority/consent adapters only after their owner contracts pass.
9. Integrate Sol-Atlas GeoPackage/source lineage, then read-only Symthaea review and Symtropy observation context.
10. Begin a community pilot only with a willing partner, legal review, data-governance charter, threat model, consent process and accessible remedy pathway.

## 10. Scale and resource qualification

The reference evaluator now propagates stream acceptance from genesis events with a predecessor work queue and emits compact projection conflict groups. The target complexity is O((N + E) log N) time and O(N + E + C) memory, where N is unique events, E predecessor links and C compact group membership. These are design targets, not measured results. Issue [CTER-SCALE-001 #949](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/949) owns larger 10–100k workloads and low-power device CPU, memory, energy and recovery measurements. The focused tests include a reversed 10,000-event chain and a 1,000-stream high-contention projection; they validate behavior/output shape, not a performance threshold.

## 11. Non-claims

This v0.1 work is not a production relay, deployed mesh, validated radio design, cryptographic implementation, legal register, cadastral survey, field study, ecological certification or proof of SOTA performance. Synthetic fixture agreement is a conformance result only. It does not prove that a community consented, that a right exists, or that a land dispute has been resolved.

## References

- [FAO Open Tenure / Community Tenure Recording](https://www.fao.org/tenure/resources/toolbox/sola/community-tenure-recording/en)
- [Cadasta Platform](https://cadasta.org/technology/)
- [Mycelix-DeSci Scientific Event Kernel](https://github.com/Luminous-Dynamics/mycelix/blob/research/land-rrr-interop-v0-1/mycelix-desci/src/core/src/scientific_events.rs)
- [IETF RFC 9171 — Bundle Protocol v7](https://www.rfc-editor.org/rfc/rfc9171.html)
- [IETF RFC 9172 — Bundle Protocol Security](https://www.rfc-editor.org/rfc/rfc9172.html)
- [IETF RFC 9173 — Default Security Contexts for BPSec](https://www.rfc-editor.org/rfc/rfc9173.html)
- [IETF RFC 9713 — BPv7 Administrative Record Types Registry](https://www.rfc-editor.org/rfc/rfc9713.html)
- [IETF RFC 9758 — Updates to the IPN URI Scheme](https://www.rfc-editor.org/rfc/rfc9758.html)
- [IETF RFC 9942 — COSE Receipts](https://www.rfc-editor.org/rfc/rfc9942.html)
- [IETF RFC 9943 — SCITT Architecture](https://www.rfc-editor.org/rfc/rfc9943.html)
- [OGC GeoPackage](https://www.ogc.org/standards/geopackage/)
- [ISO 19152-1:2024 — LADM Part 1](https://www.iso.org/standard/81263.html)
