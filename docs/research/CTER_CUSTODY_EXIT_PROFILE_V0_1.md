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

Use Mycelix's typed receipt algebra (#1215) and long-term validation/cryptographic renewal contract (#1354). Use the existing SCITT anti-equivocation work (#5942) where a transparency receipt/checkpoint is required. Do not create a parallel event log, a second generic receipt hierarchy, or a proprietary archive-timestamp protocol.

## 4. Evidence package contract

The JSON Schema is [cter-community-archive-manifest-v0.1.schema.json](../../schemas/cter-community-archive-manifest-v0.1.schema.json). It defines a strict top-level archive manifest with:

1. Exact archive/profile identity, export scope and exporter build identity.
2. Source frontiers naming each canonical owner, snapshot/record frontier, query or scope, expected count where known, observed count, completeness state and unresolved gaps.
3. Per-object identity, owner, schema/profile version, digest algorithm/value, disclosure class and disposition.
4. Explicit included / withheld / unavailable dispositions with reason codes and optional non-disclosing commitment references.
5. Access and consent references that are kept separate from possession of the archive.
6. Restore capability, key-custody model and actual restore-test state.
7. Cryptographic preservation lineage, predecessor references, timestamp evidence, renewal profile and algorithm status.
8. A fixed claim ceiling: archive structure and restoration do not establish truth, title, consent, authority or legal effect.

The digest algorithm is explicit. Mycelix BLAKE3 event hashes and Sol-Atlas SHA-256 source/record digests must remain distinct. A 64-character hex string without its algorithm and domain is insufficient.

## 5. Completeness and withholding semantics

A global count is not sufficient to establish completeness. Each source must declare the exact source frontier it searched: owner system, snapshot identity, query/scope, time or generation, expected record count if authoritative, observed count, completeness and gaps.

Allowed states:
- **Complete within declared frontier:** all eligible objects in that bounded scope are accounted for.
- **Complete with declared exclusions:** all eligible objects are accounted for, and withheld/unavailable items have explicit reasons and access-policy references.
- **Incomplete:** the source frontier is partial, stale, unavailable, or the export omits an object required by the declared scope.
- **Review required:** schema, license, consent, key-status, preservation or interpretation is unresolved.
- **Invalid:** integrity or package invariants fail.

A withheld record is not absent. A zero-row query is not global absence. An unavailable source cannot be represented as an empty source. Restricted or culturally protected payloads must not be copied into a public archive merely because an exporter has access to the source system.

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

## 7. Transport lifetime is not evidence retention

BPv7's bundle lifetime governs how long a transport bundle is retained/forwarded. Bundle expiry can require the bundle agent to delete the expired bundle. It is not the retention lifetime of the underlying signed event, claim, appeal, or preservation record.

Required rule:

`transport bundle expires` != `evidence event expires`

A relay may delete an expired carrier bundle while retaining the immutable signed event in its local evidence archive under the archive's retention policy. If delivery must be retried, a new bundle can be created from the retained event/package reference. It must not mint a new event or rewrite the historical receipt.

This is particularly important where a community is offline for weeks or where a legal dispute may continue long after a transport attempt times out.

## 8. Long-term cryptographic preservation

Use established preservation mechanisms rather than inventing perpetual signatures. RFC 4998 Evidence Record Syntax defines evidence records, timestamp chains/sequences and renewal mechanisms. Mycelix LEX-NET-038 (#1354) already owns the long-term validation and cryptographic renewal contract.

Preserve separately:
- original signed bytes and original verification outcome/context;
- proof-of-existence/timestamp evidence and its exact scope;
- certificate/key and revocation/status material used at validation time;
- crypto policy and trust-anchor profile;
- later preservation/renewal evidence;
- current cryptographic acceptability for new evidence;
- current local policy for reliance on historical evidence.

A later renewal proves continuity of preserved bytes under its qualified preservation process. It does not prove the described real-world event happened at that time, that the signer had legal authority, or that the right remains current.

## 9. Synthetic qualification corpus

[archive-cases-v0.1.json](../../benchmarks/territorial-integrity/relay/archive-cases-v0.1.json) contains proposed synthetic cases for:
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

The expected dispositions are proposed oracle labels, not executed results. The next implementation increment should add a small Rust evaluator and an independent second implementation before any benchmark pass is reported.

## 10. Metrics and research targets

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

## 11. Acceptance boundary

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
- [RFC 9943 SCITT](https://www.rfc-editor.org/rfc/rfc9943.html)
