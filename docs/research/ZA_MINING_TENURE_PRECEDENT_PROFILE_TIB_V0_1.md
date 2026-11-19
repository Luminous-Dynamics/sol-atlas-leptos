# South African Mining / Tenure Precedent Profile for TIB v0.1

**Status:** retrospective legal-structure benchmark design; not legal advice or a jurisdictional rules engine  
**Date:** 2026-11-18  
**Programme:** TIB-001 / Territorial Integrity Benchmark  
**Scope:** published South African judgments and current statutory-source metadata

## Purpose

Use published judgments to test whether a territorial evidence system preserves the relationships among:
- surface ownership, occupation and informal/customary land rights;
- mineral authorisations and the State's statutory custodianship framework;
- community governance, representation, consultation and consent;
- statutory dispute/negotiation pathways;
- environmental, water, heritage and stewardship constraints.

These cases are retrospective benchmark references. They do not authorize a field pilot, establish the current boundaries of any land interest, or imply that the parties' underlying rights can be reconstructed from the judgments alone.

## Why these precedents matter

### 1. Baleni and Others v Minister of Mineral Resources and Others

- **Court / date:** High Court of South Africa, Gauteng Division, Pretoria; 22 November 2018.
- **Citation:** Case 73768/2016; commonly cited as *Baleni and Others v Minister of Mineral Resources and Others*.
- **Subject:** proposed mining in land occupied by a customary community whose informal land rights were protected by the Interim Protection of Informal Land Rights Act 31 of 1996 (IPILRA).
- **Reported holding:** the Centre for Environmental Rights' judgment summary records that full and informed consent of the customary community was required under IPILRA for a mining right to be granted over the land it occupied. This was a High Court judgment; a jurisdiction profile must preserve the court, date, legal instruments and procedural context rather than promote the proposition into a global rule.
- **Benchmark question:** can the system distinguish consultation under the MPRDA from consent/deprivation requirements under IPILRA, and can it show which rights-holders, decision process, source evidence and authority were relied upon?
- **Required abstention:** a consultation record, meeting attendance list, traditional-leader letter or general community-resolution reference must not automatically be treated as proof of full and informed consent by all affected informal-right holders. The legal effect depends on the applicable law, custom, process and authoritative findings.

### 2. Maledu and Others v Itereleng Bakgatla Mineral Resources (Pty) Limited and Another

- **Court / date:** Constitutional Court of South Africa; 25 October 2018.
- **Citation:** [2018] ZACC 41; 2019 (1) BCLR 53 (CC); 2019 (2) SA 1 (CC).
- **Subject:** applicants lawfully occupied land under informal rights protected by IPILRA; mining rights over the same land had also been granted.
- **Relevant holdings for system design:** the Constitutional Court held that the existence of a mineral right does not itself extinguish the rights of a landowner or other occupier. The statutory mechanisms under section 54 of the MPRDA could not be bypassed by relying on common-law eviction proceedings in the circumstances before the Court. The legal question is not reduced to a single ownership field or a map overlay.
- **Benchmark question:** can the system represent a mineral right and a continuing surface occupation/use interest at the same time, preserve the dispute, and identify the statutory negotiation/administrative pathway without deciding the dispute itself?
- **Required abstention:** the presence of a mineral authorisation must not silently become an eviction power, extinguishment event, surface-access agreement or proof that occupation is unlawful.

### 3. Current IPILRA application period

The South African Government's document listing records General Notice 4165 in Government Gazette 55438, dated 25 September 2026, as an extension of the application of IPILRA. Contemporary legal update material reports that the extension runs through 31 December 2027 under section 5(2).

For machine-readable temporal qualification:
- instrument identity: IPILRA 31 of 1996, extension notice GeN 4165 of 2026;
- publication: 25 September 2026;
- reported end of extended application: 31 December 2027;
- source references: exact Gazette/notice source, capture timestamp and digest must be pinned by any operational adapter.

The system must not infer that this temporary extension is permanent tenure legislation or that the legal profile will remain unchanged after that date. The source should be rechecked at each relevant decision frontier.

## TIB test design

The machine-readable scenario index is `benchmarks/territorial-integrity/za-mining-tenure-precedents-v0.1.json`.

It defines expected system behaviour as a *review disposition*, not a legal verdict:
- preserve multiple overlapping assertions and their independent bases;
- identify missing or mismatched authority/evidence;
- distinguish consultation, consent, a decision process and legal authorization;
- show currentness/validity periods for the governing instrument;
- preserve an unresolved conflict and its challenge history;
- output `REVIEW_REQUIRED` or `INDETERMINATE` when material evidence is missing;
- never automatically decide title, consent, eviction, compensation, priority or permit validity.

The first synthetic cases should contain no coordinates or real party identifiers. Use abstract spatial references such as `urn:tib:synthetic:surface-unit-01` and `urn:tib:synthetic:subsurface-unit-01`. If later work uses public cadastral or mining-cadastre geometry, its source, date, licence/access terms, CRS, precision and completeness must be verified separately.

## Adversarial cases to add

1. A mining-authorisation polygon overlaps a customary-use assertion; the two interests coexist without priority being inferred.
2. The source provides evidence of consultation, but no evidence of the consent process required by the applicable jurisdiction profile.
3. A community meeting is documented, but notice, participation, representation or affected-right-holder coverage is missing.
4. A mining-right record is present, but no section 54 negotiation/administrative pathway evidence is available for a surface-access dispute.
5. One occupier's challenge is absent from the public index but present in a permissioned source; a partial search must not claim no challenge.
6. A legal source is superseded or its time window expires; historic analyses remain reproducible under their earlier evidence frontier.
7. A model identifies land-cover change near the mine; it cannot infer consent, occupation, illegality, mineral-right priority or community approval.
8. A permit or agreement digest is authentic but the issuer is outside the relevant jurisdiction or scope.
9. A court judgment establishes a legal proposition in its case context; the assistant must cite the exact holding and avoid extending it beyond the profile's scope.

## Claim ceiling

A TIB result can establish that the system:
- represented a specified set of source assertions;
- preserved their source/authority/dispute metadata;
- identified specified missing evidence;
- produced a bounded, reproducible review disposition.

It cannot establish that:
- a specific person's or community's right is legally valid;
- a mining right is valid or invalid;
- consent has legally been given;
- a community representative has a valid mandate;
- a party may be evicted;
- a boundary is legally correct;
- an environmental impact has or has not occurred.

Those matters require competent authorities, qualified practitioners, lawful processes and, where appropriate, affected communities' own decisions.

## References

- [Baleni and Others v Minister of Mineral Resources and Others — Centre for Environmental Rights judgment summary](https://cer.org.za/virtual-library/judgments/high-courts/baleni-and-others-v-minister-of-mineral-resources-and-others)
- [Maledu and Others v Itereleng Bakgatla Mineral Resources — Constitutional Court official case record](https://collections.concourt.org.za/items/440c94e0-f9e4-4c89-8625-b43b3fdb960a)
- [Maledu — SAFLII full judgment](https://www3.saflii.org/za/cases/ZACC/2018/41.html)
- [South African Government — Notice listing, Government Gazette 55438 (25 September 2026)](https://www.gov.za/ss/documents/notices?page=9)
- [IPILRA extension summary, 30 September 2026](https://www.golegal.co.za/protection-informal-land/)

This profile is a benchmark and interoperability aid, not a substitute for legal advice or a determination of any live land/mining dispute.
