# CTER Offline Relay Profile and Convergence Benchmark v0.1

**Initiative:** Community Territorial Evidence Relays (CTER)  
**Parent benchmark:** Territorial Integrity Benchmark (TIB), Sol-Atlas issue #937  
**Implementation issue:** [CTER-RELAY-QUAL-001 #945](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/945)  
**Status:** research contract + synthetic reference model; no production transport or field qualification claimed  
**Date:** 19 November 2026

## 1. Executive thesis

CTER is not differentiated by offline form capture alone. FAO Open Tenure already documents offline mobile collection, encrypted local storage, multilingual forms, participatory validation, dispute support, and synchronization with a community server. Cadasta documents offline-ready geospatial collection, secure storage, role-based sharing, and integration with official land systems.

The research target is harder and narrower:

> Preserve the integrity, agency and contestability of territorial evidence while independent parties submit, carry, copy, delay, replay and reconcile records across disconnected and partially trusted relays.

A relay is a courier and custodian of encrypted packages. It is not an adjudicator, authority issuer, evidence validator or owner of the community's records.

## 2. Paradigm shifts

### 2.1 From offline forms to partition-tolerant evidence networks

Ordinary offline workflows usually focus on collecting data locally and synchronizing it later. CTER must specify what happens when two communities, a claimant and an objector, or several couriers independently create records while disconnected.

The network must preserve concurrent events and expose the need for review. It must never silently pick a winner because one device synchronized last.

### 2.2 From last-write-wins to contest-preserving convergence

Use an append-only event set with explicit causal parent references. A deterministic projection may be rebuilt from the set, but competing assertions remain separate. Concurrent events affecting the same declared projection key produce a conflict set, not an automatic merge or legal priority.

The event-set merge operation should be a deterministic set union over event identity and content digest. It should be commutative, associative and idempotent. Those algebraic properties make duplicate delivery and reordering safe at the evidence-envelope layer; they do not resolve the underlying dispute.

### 2.3 From wall-clock ordering to causal evidence

Device clocks can drift, be reset, be deliberately changed or be unavailable. Wall-clock timestamps are useful observations with provenance and uncertainty; they are not a safe ordering primitive.

Causality comes from explicit parent references and origin-scoped sequence evidence. A missing parent leaves the event pending. A timestamp does not fill the causal gap.

### 2.4 From "delivered" to typed receipts

Keep distinct receipts for:

- **Local receipt:** the submitting device durably recorded an envelope.
- **Custody receipt:** a relay accepted responsibility for carrying an identified package.
- **Destination receipt:** a named destination received an identified package.
- **Integrity receipt:** a verifier checked the exact bytes/digest under a named algorithm and build.
- **Signature-verification result:** a named verifier evaluated an issuer signature and key status.
- **Community attestation:** an authorized participant made a bounded assertion.
- **Institutional determination:** a competent institution recorded a decision under applicable law.

No earlier receipt implies a later one. In particular, transport delivery is not authenticity, truth, community endorsement, consent, legal effect or registration.

### 2.5 From public maps to purpose-bound disclosure

Confidentiality classification travels with the envelope and its package. A courier may be allowed to carry a sealed object without being allowed to inspect it. An authorization to read a generalized map does not authorize bulk export of household points, testimony, sacred locations or active disputes.

Public, community-controlled, restricted and culturally protected data must be separated at both the user interface and export boundary. Sensitive coordinates should not leak through logs, map tiles, search indexes, thumbnails, prompts or benchmark artifacts.

### 2.6 From a central online service to community-operable infrastructure

A community should be able to operate a local node without calling a Luminous-hosted endpoint. Devices should support local backup, export and recovery. Interoperability must not require a single central database or a token economy.

## 3. Protocol layers and ownership

| Layer | Owner | Responsibility | Explicit non-responsibility |
|---|---|---|---|
| Territorial semantics | Sol-Atlas | LADM-aligned rights, responsibilities, restrictions, spatial/temporal references and source lineage | Does not adjudicate a claim |
| Evidence identity and authority | Mycelix | Issuer identity, scoped authority, consent, revocation and attestations | Does not turn DHT consensus into legal registration |
| Relay convergence | CTER profile | Envelopes, causal parents, merge, replay handling, custody receipts and offline reconciliation | Does not decide legal priority |
| Evidence reasoning | Symthaea | Read-only synthesis, contradictions, uncertainty, source citations and abstention | Does not create or approve a right |
| Environmental observations | Symtropy | Read-only, snapshot-consistent physical/ecological observations and model outputs | A model output is not a measured outcome |
| Geospatial container | OGC GeoPackage | Portable offline feature/raster/tabular packages | A GeoPackage is not an access-control or legal model |
| Transport | BPv7 candidate profile | Store-carry-forward through intermittent and physically mobile links | Transport security is not application authorization |

### Transport decision

Do not invent a radio protocol or proprietary store-and-forward transport. Evaluate the IETF Bundle Protocol v7 (RFC 9171) as a transport candidate and its Bundle Protocol Security specification (BPSec, RFC 9172) for bundle-level integrity/confidentiality where appropriate. Before implementation, check current RFC updates and select an actual interoperable implementation. Also benchmark simpler HTTPS synchronization and encrypted removable-media exchange.

The v0.1 Rust reference model does not implement BPv7, BPSec, cryptography, key storage or radio communications. It models evidence-event convergence after a separate verifier has supplied a verification disposition.

## 4. Event envelope minimum

The first reference profile uses these fields:

| Field | Meaning |
|---|---|
| `event_id` | Stable identity for one event |
| `origin_id` | Originating device or actor namespace |
| `origin_sequence` | Monotonic sequence within that origin namespace; not a trusted global clock |
| `parents[]` | Explicit causal parent event IDs |
| `projection_key` | Declared derived-view key used to detect concurrent edits; never an ownership key |
| `envelope_sha256` | Digest of the canonical complete envelope |
| `verification` | Result supplied by an independently identified signature/key verifier |
| `visibility` | `public`, `community`, `restricted` or `culturally_protected` |

The canonicalization profile, hash algorithm, signature suites, key lifecycle, trusted timestamp model and authorization policy require separate versioned contracts. The reference crate validates the supplied digest's shape but does not recompute it or authenticate a signature.

Raw evidence blobs are encrypted and stored outside the transport receipt. Public audit records should contain only the minimum references and commitments needed for verification.

## 5. Normative convergence behavior for v0.1

1. An exact replay of the same `event_id` and canonical envelope is idempotent.
2. The same event ID paired with different content digests is equivocation. Keep both variants in quarantine; accept neither as the unique event.
3. Invalid or revoked signatures are never accepted.
4. Unavailable verification is pending, not accepted.
5. A missing parent is pending. It may be reconsidered when the parent later arrives.
6. A child of an invalid, revoked, unavailable, forked or equivocated parent is blocked pending review.
7. Two verified events from the same origin reusing one origin sequence are an origin-sequence fork. Preserve both, accept neither automatically.
8. A causal cycle is quarantined.
9. Verified concurrent events sharing a projection key are both preserved as evidence and reported as a conflict pair.
10. A causally ordered correction is a new event; the prior event remains in history.
11. No deterministic display sort, timestamp or relay count may be interpreted as legal priority.
12. Rebuilding the same projection from the same normalized event set and profile version must produce the same result.

The reference model's `accepted_events` means only structurally eligible for evidence preservation under the supplied verifier dispositions. It does not establish truth, authenticity of the verifier, lawfulness, representativeness, consent or legal effect.

## 6. Threat model and failure injection

| Threat/failure | Required behavior |
|---|---|
| Lost, duplicated, reordered or delayed package | No silent loss; duplicates idempotent; late parents unblock pending events |
| Malicious courier | Cannot read restricted payloads or alter accepted content without detection at the receiving verifier |
| Compromised/stolen device | Revoke/rotate its future signing authority; preserve earlier receipt history and mark the applicable time uncertainty |
| Same event ID, different content | Quarantine all variants as equivocation |
| Origin sequence reused | Preserve and surface fork; do not choose first/last |
| Clock skew or rollback | Preserve clock observation and uncertainty; do not use it for causal ordering |
| False or unavailable signature verification | Invalid/revoked never accepted; unknown/unavailable remains pending |
| Relay data loss | Recovery from independent copies and verifiable manifests; missing frontier is explicit |
| Visibility downgrade | Reject export/delivery to a destination whose authorization does not cover the payload |
| Bulk export or inference | Rate limits, scoped export receipts, minimization and leakage tests |
| Device power loss mid-write | Transactional durability and recovery without partial promotion |
| Conflicting community representations | Preserve competing mandates and decision records; do not use popularity/reputation to settle authority |
| Key rotation and revocation during partition | Existing events retain historical verification context; new signatures require current policy; ambiguous effective time is quarantined |
| Bundle expiry/TTL | Transport expiry may stop carriage; it does not expire or extinguish the underlying claim |

## 7. Benchmark programme

### TIB-RELAY-A — convergence correctness
- Exact duplicate delivery.
- Duplicate plus reordered batches.
- Same event ID / different digest.
- Origin sequence fork.
- Parent missing, then arriving.
- Conflicting concurrent claims.
- Causal correction chain.
- Cycles and blocked descendants.

### TIB-RELAY-B — hostile transport
- Bit-flipped payload.
- Replayed bundle identifier.
- Replayed valid old event.
- Unknown/revoked key.
- Compromised relay with valid custody identity but no content authority.
- Downgrade of visibility classification.
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
- A participant's withdrawal request while downstream copies exist.
- Dissent submitted without the local leader's permission.
- Receipt retrieval without revealing the witness's identity to a courier.

### Metrics and targets

| Measure | Research target |
|---|---:|
| Silent event loss across frozen partition suite | 0 |
| Silent overwrite / automatic legal-priority selection | 0 |
| Exact replay idempotence | 100% |
| False acceptance of invalid, revoked or unavailable verification | 0 |
| Equivocation/fork detection on frozen cases | 100% |
| Preservation of active conflict records | 100% |
| Deterministic replay of same normalized event set | 100% |
| Independent reference-oracle agreement | 100% on frozen cases |
| Unauthorized restricted-data disclosures in adversarial corpus | 0 |
| 30-day disconnection with full local submission/receipt workflow | Demonstrate, then publish observed completion and recovery |
| p50/p95/p99 receipt latency, sync convergence time, bytes/event, energy/event, storage growth | Measure on declared devices/network schedules; no invented target until baseline |
| Community task success, comprehension and perceived control | Co-designed field study; do not infer from synthetic tests |

Do not collapse the metrics into a single score. Report each failure mode, false positive, abstention, hardware, network trace, payload size, software version and excluded case.

## 8. Fair industry comparison

| Comparator | Documented strength | CTER comparison rule |
|---|---|---|
| FAO Open Tenure / Community Server | Offline mobile capture, encrypted local DB, ten-language support, participatory validation, disputes and sync | Do not claim these are novel; test cross-relay merge/fork/replay/causal-gap guarantees with matched fixtures |
| Cadasta Platform | Offline-ready GIS collection, secure cloud storage, role-based sharing, integration with national systems and community data protocols | Do not imply lack of offline or sovereignty features; distinguish product-documentation comparison from hands-on benchmarking |
| OGC GeoPackage | Portable SQLite-based vector/raster/tabular geospatial container usable on mobile with limited connectivity | Reuse as a spatial data container; benchmark read/write/copy cost and integrity, not as a land-rights protocol |
| LADM ISO 19152-1:2024 | Shared model for parties, BAUnits, RRRs, spatial units, sources and versioned objects | Map CTER envelopes to LADM identities; preserve jurisdictional extensions and unmapped concepts |
| BPv7 / BPSec | Standardized store-carry-forward transport and bundle-level security building blocks | Reuse/implement interoperably where feasible; CTER adds land-evidence semantics above transport |

Hands-on superiority claims require access to comparable versions and environments. If a product cannot be run or the user population differs, report a documented-capability comparison, not a measured score.

## 9. Implementation sequence

1. Freeze the event-envelope profile and synthetic corpus.
2. Build the standalone Rust reference convergence model.
3. Run a second-language independent oracle against the exact corpus.
4. Add exact-commit CI for the standalone crate; queued/skipped is not pass.
5. Extend the existing TIB report with operational measures and a matched Open Tenure/Cadasta comparator plan.
6. Prototype a local relay using commodity Android/Linux devices and an encrypted local store.
7. Evaluate BPv7/BPSec implementations against HTTPS and removable media on identical traces.
8. Add Mycelix key/authority/consent adapters only after their owner contracts pass.
9. Integrate Sol-Atlas GeoPackage/source lineage, then read-only Symthaea review and Symtropy observation context.
10. Begin a community pilot only with a willing partner, legal review, data-governance charter, threat model, consent process and accessible remedy pathway.

## 10. Non-claims

This v0.1 work is not a production relay, deployed mesh, validated radio design, cryptographic implementation, legal register, cadastral survey, field study, ecological certification or proof of SOTA performance. Synthetic fixture agreement is a conformance result only. It does not prove that a community has consented, that a right exists, or that a land dispute has been resolved.

## References

- [FAO Open Tenure / Community Tenure Recording](https://www.fao.org/tenure/resources/toolbox/sola/community-tenure-recording/en)
- [Cadasta Platform](https://cadasta.org/technology/)
- [IETF RFC 9171 — Bundle Protocol v7](https://www.rfc-editor.org/rfc/rfc9171.html)
- [IETF RFC 9172 — Bundle Protocol Security](https://www.rfc-editor.org/rfc/rfc9172.html)
- [OGC GeoPackage](https://www.ogc.org/standards/geopackage/)
- [ISO 19152-1:2024 — LADM Part 1](https://www.iso.org/standard/81263.html)
- [W3C Verifiable Credentials Data Model 2.0](https://www.w3.org/TR/vc-data-model-2.0/)
- [UN-GGIM Framework for Effective Land Administration](https://ggim.un.org/UNGGIM-publications/)
