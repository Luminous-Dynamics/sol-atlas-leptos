# CTER Community Custody, Portability and Exit Profile v0.1

**Tracking issue:** [CTER-CUSTODY-EXIT-001 #947](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/947)  
**Parent benchmark:** [TIB-001 #937](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/937)  
**Relay workstream:** [CTER-RELAY-QUAL-001 #945](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/945)  
**Status:** proposed package contract and synthetic design corpus; no archive exporter/restorer or field qualification claimed  
**Date:** 19 November 2026

## 1. Thesis: portability must preserve evidence, not merely data

An archive that contains a current map but omits its source assertions, signed event chain, objections, corrections, consent scopes, restrictions, legal basis, source licenses or verification gaps is not a complete territorial archive.

CTER therefore separates **evidence portability** from ordinary file export. The goal is that an independently operated implementation can reconstruct the same bounded evidence view without contacting the original SaaS provider, donor, NGO, Luminous endpoint or author of the exporter.

This profile does not imply that every object can or should be disclosed. A complete export may legitimately include explicit withheld/unavailable entries, commitments and reason codes while keeping protected payloads inaccessible.

## 2. Industry baseline

Do not claim that offline mapping, community data ownership, role-based sharing, ongoing consent or export are novel. Cadasta documents those capabilities, including data-governance support, export/transition options and a post-project continuity pathway. FAO Open Tenure documents offline mobile capture, local encrypted storage, local/cloud synchronization, participatory validation and Community Server deployment options.

CTER's research hypothesis is narrower: qualify evidence-complete, machine-verifiable, provider-independent restoration across multiple owner systems, with an explicit frontier for every omitted, restricted, unavailable or unresolved object.

The benchmark should evaluate against documented Cadasta/Open Tenure capabilities fairly. Hands-on comparison requires equivalent access, versions, tasks, data rights, device/network conditions and partner roles.

## 3. Ownership: do not create another ledger

- **Mycelix** remains the canonical owner of signed scientific event envelopes, claim-stream sequence/previous-hash chain, BLAKE3 event hashes, identity/authority, consent, typed receipts and cryptographic preservation interfaces.
- **Sol-Atlas** remains the owner of territorial/spatial records, LADM profile mappings, source-layer licenses, geometry references and spatial qualification metadata.
- **Symthaea** owns read-only source-grounded review artifacts, model/policy version, citation and abstention records.
- **Symtropy** owns environmental observations and simulation outputs with source, method, uncertainty and measured/modelled evidence classes.
- **CTER** defines package manifest, relay custody, export-frontier and independent restore qualification.
- **OGC GeoPackage** remains the geospatial container. The package can reference other controlled artefacts without embedding them.

Use Mycelix's typed receipt algebra (#1215) and long-term validation/cryptographic renewal contract (#1354). For transparency, align with IETF RFC 9943 (SCITT) and RFC 9942 (COSE Receipts), and consume the existing Mycelix split-view/witness work (#5942; model PR #5944). Mycelix owns SCITT receipt/checkpoint semantics. Do not create a parallel event log, second generic receipt hierarchy, VDS, witness service or proprietary archive-timestamp protocol.

## 4. Evidence package contract

The JSON Schema is [cter-community-archive-manifest-v0.1.schema.json](../../schemas/cter-community-archive-manifest-v0.1.schema.json). Its export context names an opaque destination and an explicit set of authorized disclosure classes. It deliberately does not assume that `restricted` automatically includes `community`, or that communities share one universal sensitivity ranking. The schema defines a strict top-level archive manifest with:

1. Exact archive/profile identity, export scope and exporter build identity.
2. Source frontiers naming each canonical owner, snapshot/record frontier, query or scope, expected count where known, observed count, completeness state and unresolved gaps.
3. Per-object identity, owner, schema/profile version, digest algorithm/value, disclosure class and disposition.
4. Explicit included / withheld / unavailable dispositions with reason codes and optional non-disclosing commitment references.
5. Access and consent references that are kept separate from possession of the archive.
6. Opaque export destination identity, exact authorized disclosure-class set, purpose and authorization reference; classes are not a universal rank hierarchy and the authorization must be resolved by an independent policy/authority service before disclosure.
7. Restore capability, key-custody model and actual restore-test state.
8. Cryptographic preservation lineage, predecessor references, timestamp evidence, renewal profile and algorithm status.
9. A fixed claim ceiling: archive structure and restoration do not establish truth, title, consent, authority or legal effect.

The digest algorithm is explicit. Mycelix BLAKE3 event hashes and Sol-Atlas SHA-256 source/record digests must remain distinct. A 64-character hex string without its algorithm and domain is insufficient.

## 5. Completeness and withholding semantics

A global count is not sufficient to establish completeness. Each source must declare the exact source frontier it searched: owner system, snapshot identity, query/scope, time or generation, expected record count if authoritative, observed count, completeness and gaps.

Allowed states:
- **Complete within declared frontier:** all eligible objects in that bounded scope are accounted for.
- **Complete with declared exclusions:** all eligible objects are accounted for, and withheld/unavailable items have explicit reasons and access-policy references.
- **Incomplete:** the source frontier is partial, stale, unavailable, or the export omits an object required by the declared scope.
- **Review required:** schema, license, consent, key-status, preservation or interpretation is unresolved.
- **Invalid:** integrity or package invariants fail.

A withheld record is not absent. A zero-row query is not global absence. An unavailable source cannot be represented as an empty source. Restricted or culturally protected payloads must not be copied unless the exact class is explicitly included in the destination's independently resolved authorization. Access classes are labels, not a universal ordered ladder; one class does not implicitly include another.

## 6. Independent restore and key custody

At least one restore path must work without the original provider's API, cloud tenant, software license or administrator account.

The archive must identify the key-custody arrangement:
- community-controlled threshold recovery;
- participant-controlled key;
- independent trustee arrangement;
- provider-only custody;
- unknown or unresolved.

Only the first three can qualify for a positive provider-independent restore claim, subject to the actual recovery policy and a tested ceremony. A document saying "community-owned" does not prove that the community holds the decryption key.

Recovery must not create a single-person capture path. A recovery procedure should state its threshold, participants, revocation behavior, audit evidence and process for leadership/authority disputes. A new key must not silently rewrite old signed events.

## 7. Community governance, consent and contestation are first-class evidence

Technical custody is not community authority. A cryptographically valid signature, an archive export, a majority tally or an administrator account cannot by itself establish who may speak for a community, who may disclose a particular dataset, or whose consent is required.

### Separate decisions that are too often collapsed

The owning governance adapter must distinguish at least:
- authority to collect a particular data class in a defined area;
- individual participation and consent for personal testimony or identifiable data;
- collective/community authorization for shared territory, customary knowledge or community-held records;
- authority to disclose to a named recipient for a stated purpose;
- authority to publish or generalize a map or dataset;
- authority to transfer custody, appoint/revoke a steward, or initiate key recovery;
- retention, preservation, withdrawal, correction and deletion decisions;
- any competent public/legal authority decision, which remains external to CTER.

These permissions have different subjects, scopes, procedures, validity periods and appeal paths. They must not be encoded as one `community_approved=true` flag.

### A bounded governance mandate

A decision record should bind: the recognized decision-making body under the community's own rules; the people/groups represented and any known gaps; the defined territory/data categories; purpose and recipients; allowed operations; effective and expiry times; decision procedure; quorum or consensus rule if the community itself defines one; dissent/minority positions; appeal route; language/accessibility arrangements; and the exact evidence/receipts that support the decision.

Do not assume that one elected leader, one elder, one landholder, one NGO, or one simple majority represents every rights-holder. Preserve competing mandates and objections as evidence. Unresolved representation or scope means REVIEW/INDETERMINATE, not majority-wins-by-default. Mycelix's authority/consent owner (#6315/#5277) must provide the actual policy decision; the archive validator only checks that relevant references and outcomes are accounted for.

### Consent withdrawal is not magical remote erasure

A withdrawal can change future permitted collection, processing, disclosure or reuse. It does not prove that a recipient has deleted an earlier export, that a public copy can be recalled, or that every offline relay has received the update. Preserve distinct states for: withdrawal requested; authenticated and evaluated; future egress blocked; known recipients notified; recipient deletion/retention response received; residual copies unknown; legal/preservation hold; and dispute/appeal. Never issue a universal "deleted everywhere" receipt without independently verifiable acknowledgements from every declared copy/recipient.

A stale offline node must not treat old consent or authority as current indefinitely. Where currentness cannot be established under a bounded policy/lease, fail closed for new disclosure and preserve the pending update. Existing evidence and retention obligations require a separately governed treatment; do not silently erase an objection or legal hold to satisfy a blanket deletion request.

### Governance frameworks are context, not a replacement for law

For Indigenous data, the CARE Principles (Collective Benefit, Authority to Control, Responsibility, Ethics) are a people- and purpose-oriented complement to FAIR data principles. Apply them in the way relevant Indigenous peoples and their governance bodies direct; do not treat CARE as a generic certification checkbox for every community. The FAO VGGT provide important voluntary guidance for land/fisheries/forest tenure, but they are not legally binding and do not cover water or mineral rights. National and local law, customary protocols and jurisdiction-specific authority must be separately mapped. Neither CARE nor VGGT is a legal adjudication engine.

### Governance adversarial benchmark

Add synthetic cases for: a valid signature from a person outside the decision mandate; a mandate outside its geographic/time scope; consent withdrawn while a relay is partitioned; a public-export request over a restricted item; a dissenting group whose objection is omitted; a donor/NGO administrator attempting to block community export; a key-recovery quorum captured by one controller; a recipient that has not acknowledged deletion; and a legal hold conflicting with deletion. Required fail-safe outcomes are typed DENY, REVIEW, INCOMPLETE or UNKNOWN—not a green aggregate score.

## 8. Transport lifetime is not evidence retention

BPv7's bundle lifetime governs how long a transport bundle is retained/forwarded. Bundle expiry can require the bundle agent to delete the expired bundle. It is not the retention lifetime of the underlying signed event, claim, appeal, or preservation record.

Required rule:

`transport bundle expires` != `evidence event expires`

A relay may delete an expired carrier bundle while retaining the immutable signed event in its local evidence archive under the archive's retention policy. If delivery must be retried, a new bundle can be created from the retained event/package reference. It must not mint a new event or rewrite the historical receipt.

This is particularly important where a community is offline for weeks or where a legal dispute may continue long after a transport attempt times out.

## 9. Long-term cryptographic preservation

Use established preservation mechanisms rather than inventing perpetual signatures. RFC 4998 Evidence Record Syntax defines evidence records, timestamp chains/sequences and renewal mechanisms. Mycelix LEX-NET-038 (#1354) already owns the long-term validation and cryptographic renewal contract. Add cryptographic agility and post-quantum migration planning in line with NIST CSWP 39upd1 (June 2026) and NIST IR 8547 (final 27 October 2026); preserve exact historical validation contexts and never silently re-sign old evidence as if it were newly issued.

Preserve separately:
- original signed bytes and original verification outcome/context;
- proof-of-existence/timestamp evidence and its exact scope;
- certificate/key and revocation/status material used at validation time;
- crypto policy and trust-anchor profile;
- later preservation/renewal evidence;
- current cryptographic acceptability for new evidence;
- current local policy for reliance on historical evidence.

A later renewal proves continuity of preserved bytes under its qualified preservation process. It does not prove the described real-world event happened at that time, that the signer had legal authority, or that the right remains current.

## 10. Synthetic qualification corpus

[archive-cases-v0.1.json](../../benchmarks/territorial-integrity/relay/archive-cases-v0.1.json) contains 12 synthetic cases for:
- complete independent restore;
- complete archive with explicitly withheld culturally protected item;
- omitted active objection;
- unknown/mislabelled digest algorithm;
- provider-only encryption key;
- restricted coordinates leaked into a public export;
- expired BPv7 carrier while evidence archive remains intact;
- preserved event with predecessor-bound cryptographic renewal;
- compromised/deprecated signature algorithm without adequate renewal;
- unsupported schema with declared mapping loss;
- unresolved license/consent;
- partial source frontier misrepresented as complete.

The expected dispositions are frozen synthetic oracle labels. The Rust evaluator is committed but still requires exact-head Cargo/CI execution and independent second-implementation agreement.

## 11. Metrics and research targets

| Measure | Target |
|---|---:|
| Export-eligible objects accounted for within declared frontier | 100% |
| Silent omission of events, objections, corrections or appeals | 0 |
| Unauthorized disclosure in frozen adversarial corpus | 0 |
| Exact event-chain integrity after independent restore | 100% of eligible events |
| Round-trip loss in declared supported concepts | 0; all loss/extension explicit |
| Provider-independent restore after provider termination | Demonstrated by cold-start restore |
| Restore from a 30-day isolated archive | 100% of eligible records and receipts |
| Withheld/unavailable records silently represented as absent | 0 |
| Preservation renewal predecessor binding | 100% |
| Archive validation disposition | VALID / FAILED / INDETERMINATE; never an unqualified Boolean |
| Export/restore time, bytes, energy, operator effort and cost | Measured by package size, device and network profile |
| Consent/withdrawal understanding and perceived control | Co-designed community study |

Do not collapse these into a single sovereignty score. Report each dimension, exclusions, uncertainty and source version separately.

## 12. Acceptance boundary

This is a protocol and benchmark proposal. It does not claim that an archive exporter, cold-start restore, key-recovery ceremony, preservation service, user study or community pilot has been implemented or qualified. The synthetic corpus does not establish legal sufficiency or community legitimacy.

## References

- [Cadasta Platform](https://cadasta.org/technology/)
- [Cadasta Data Sovereignty & Governance](https://cadasta.org/data-sovereignty-governance/)
- [FAO Community Tenure Recording / Open Tenure](https://www.fao.org/tenure/resources/toolbox/sola/community-tenure-recording/en)
- [OGC GeoPackage](https://www.ogc.org/standards/geopackage/)
- [ISO 19152-1:2024 LADM](https://www.iso.org/standard/81263.html)
- [RFC 9171 Bundle Protocol v7](https://www.rfc-editor.org/rfc/rfc9171.html)
- [RFC 9172 BPSec](https://www.rfc-editor.org/rfc/rfc9172.html)
- [RFC 4998 Evidence Record Syntax](https://www.rfc-editor.org/rfc/rfc4998.html)
- [RFC 9942 COSE Receipts](https://www.rfc-editor.org/rfc/rfc9942.html)
- [RFC 9943 SCITT Architecture](https://www.rfc-editor.org/rfc/rfc9943.html)
- [NIST CSWP 39upd1 — Considerations for Achieving Crypto Agility](https://csrc.nist.gov/pubs/cswp/39/upd1/final)
- [NIST IR 8547 — Transition to Post-Quantum Cryptography Standards](https://csrc.nist.gov/pubs/ir/8547/final)
- [Global Indigenous Data Alliance — CARE Principles](https://www.gida-global.org/careprinciples)
- [FAO — Voluntary Guidelines on the Responsible Governance of Tenure](https://www.fao.org/land-water/land/land-governance-and-planning/land-resources-planning-toolbox/detail/voluntary-guidelines-on-responsible-governance-of-tenure-of-land-fisheries-and-forests-in-the-context-of-national-food-security-%28vggt%29/en)
