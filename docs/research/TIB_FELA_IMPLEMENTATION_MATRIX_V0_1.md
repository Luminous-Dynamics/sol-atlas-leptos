# TIB FELA Implementation Matrix v0.1

**Snapshot:** 19 November 2026  
**Framework:** UN-GGIM Framework for Effective Land Administration (FELA)  
**Status:** project-to-framework alignment proposal; not a national FELA assessment or certification

## 1. Why this matrix exists

FELA is not a software architecture or a nine-item feature checklist. It is a socio-technical framework for effective land administration. Its nine pathways span governance/institutions/accountability, policy/legal, finance, data, innovation, standards, partnerships, capacity/education and advocacy/awareness. UN-GGIM groups the pathways under Governance, Technology and People.

The sixth UN-GGIM Expert Meeting on Land Administration and Management took place in Cape Town on 29–30 May 2026 under the theme “From Frameworks to Practice”. The implication for TIB is direct: a geospatially correct exchange is insufficient if institutional authority, resources, participation, access, skills or recourse are missing.

**Primary references**
- [UN-GGIM FELA / Sixth Expert Meeting material (Cape Town, May 2026)](https://www.un.org/globalgeospatial/sites/default/files/2026-06/fela.pdf)
- [UN-GGIM FELA: Nine Pathways](https://ggim.un.org/meetings/GGIM-committee/10th-Session/documents/E-C.20-2020-29-Add_2-Framework-for-Effective-Land-Administration.pdf)
- [Sixth Expert Meeting event page](https://www.un.org/globalgeospatial/en/events/2026/sixth-expert-meeting-expert-group-land-administration-and-management)

## 2. FELA pathway-to-work alignment

| FELA pathway | Territorial assurance responsibility | Relevant Luminous owner / interface | Evidence state today | TIB evaluation measure |
|---|---|---|---|---|
| **1. Governance, institutions and accountability** | Make institutional roles, decision authority, review, appeal and accountability explicit; distinguish who may assert, attest, verify, adjudicate and publish | Mycelix governance / LAND-STEWARD; Property legal-effect boundary #6319; Mycelix evidence and authority references | Reference contracts and draft boundary documents exist; no competent-authority integration or community governance pilot | Role-confusion rate; unsupported authority promotions; appeal/recourse availability; unresolved responsibility count; independent review coverage |
| **2. Policy and legal** | Preserve jurisdictional law/profile references without inventing universal legal hierarchies; make legal-effect boundaries explicit | Sol Atlas TERR-001; Mycelix LAND-RRR; South African retrospective precedent profile | Structural records and retrospective case references only; no live legal rules engine or legal opinion | Citation accuracy; correct issuer jurisdiction/scope; legal status non-promotion; source currentness and revocation tests; reviewer agreement |
| **3. Financial** | Make operational cost, long-term stewardship liabilities, maintenance funding and service continuity visible | Mycelix finance/escrow interfaces; Symtropy cost and resource observations; Property commons | No validated land-administration cost model or community-financing pilot in this workstream | Cost per completed/appealed case; cost of assisted/offline service; funded vs unfunded obligations; lifetime support cost; cost of data exit |
| **4. Data** | Maintain source lineage, versioned records, evidence frontiers, correction/revocation propagation and privacy labels | Sol Atlas territorial graph; Mycelix CIVOS-EVIDENCE/EPI-008; evidence invalidation index | Structural validators and synthetic invalidation tests exist; no connected real source feed or end-to-end delivery | Source-version retention; round-trip fidelity; false absence rate; transitive invalidation completeness; stale projection rate; restricted-data leakage |
| **5. Innovation** | Use automation and AI to identify gaps, conflicts and candidate evidence without substituting predictions for decisions | Symthaea LAND-REVIEW; Symtropy observations and causal analysis; TIB four-outcome envelope | Integration contract / proposed roles only; no qualified real-world model performance in this programme | Reviewer workload reduction without recall loss; calibrated uncertainty; unsupported conclusion rate; out-of-distribution detection; reproducibility |
| **6. Standards** | Interoperate with LADM, STDM, OGC feature/record/asset interfaces and jurisdictional profiles while reporting loss | Sol Atlas LADM crosswalk; JSON-FG/OGC API Features/STAC; Mycelix RRR reference | Crosswalk and reference schema exist; standards conformance has not been independently certified | Mandatory-field round-trip retention; CRS/axis-order/vertical datum preservation; temporal semantics; unmapped concept report; independent conformance result |
| **7. Partnerships** | Establish legitimate relationships with land institutions, surveyors, legal practitioners, communities, NGOs and environmental scientists | Luminous / LADM-SOLA-Open Tenure-Cadasta comparison; South African pilot preparation | Desk research only; no participating community, regulator, surveyor or institutional partner is represented as signed up | Partner diversity; documented mandates; independent reviewer availability; data agreements; issue resolution time; continuity after partner exit |
| **8. Capacity and education** | Support field users, local administrators, translators, reviewers and maintainers; preserve assisted and non-digital routes | Open Tenure interoperability; user-facing Sol Atlas tools; language and accessibility work | No TIB field-user study or training delivery for this workstream | Completion by user group/language; onboarding time; error/correction rate; low-bandwidth/offline recovery; accessibility; training/support cost |
| **9. Advocacy and awareness** | Enable understandable public information, correction channels and transparent explanation without exposing restricted data | Sol Atlas public/controlled/embargoed/restricted projection; TIB reviewer dossier; community feedback | Disclosure classes and design requirements exist; no public-interest deployment or community review of maps | Correction response time; comprehension; accessible appeal route; misinformation/false certainty rate; unauthorized disclosure; public data provenance |

## 3. How TIB should assess FELA without pretending to certify a country

TIB is a product/interoperability benchmark. It must not be marketed as a national FELA compliance assessment. Use the framework to define product evidence and implementation gaps, not to score a country's legal or institutional performance.

Each scored claim must carry one of these evidence labels:

1. **Implemented and executed:** exact code/head, test or independent run evidence exists.
2. **Structurally checked:** static/schema/fixture validation only.
3. **Documented capability:** provider or standards-body documentation describes it.
4. **Proposed target:** a benchmark goal or design proposal, not measured.
5. **Field-unverified:** requires actual community, practitioner, institutional or physical-world validation.
6. **Not evaluable:** necessary permission, data, source or reference ground truth is unavailable.

Queued, skipped or cancelled CI is not a passing result. Desk research is not a product benchmark. A synthetic result is not a field result.

## 4. Institutional interoperability and South African implementation context

The 2026 UN-GGIM presentation summarizes an 85-jurisdiction property-institutions survey. Its Eastern and Southern Sub-Saharan Africa sample has 15 observations and includes South Africa, but it is not a South Africa-specific estimate. That regional table reports 6.7% registry API access, 13.3% cadastre API access, 46.7% registry/cadastre integration and 0% e-conveyance possible. It is a reason to test non-API and assisted reconciliation, not a basis for inferring the status of a specific South African register.

For mining rights, the DMPR official online-systems page currently lists Project Indwe's integrated Mining Licensing System with a map viewer, search/report functions and online lodgement/payment for registered industry users. It does not establish that a public API exists. The BLSA tracker has contradictory older "not yet implemented" entries as well as later entries reporting a Western Cape pilot and 37 registered right-holders; the latest tracker snapshot says outcomes are too early to gauge and that no Q3 2026 progress update was received. TIB should treat the portal as an authoritative service interface to investigate, and the tracker as a time-stamped, mixed-evidence status source.

References:
- [UN-GGIM FELA materials, including the regional table](https://www.un.org/globalgeospatial/sites/default/files/2026-06/fela.pdf)
- [DMPR official online systems](https://www.dmpr.gov.za/Branches/Online-Systems)
- [BLSA Mining Cadastre Reform Tracker](https://tracker.blsa.org.za/economic/?reform=Mining+cadastral+system)

## 5. Proposed FELA-aware TIB acceptance targets

These are initial research targets, not universal legal standards or achieved results.

| Measure | Target |
|---|---:|
| Unsupported promotion of an internal record to legal/registered status | 0 |
| Critical authority-scope or revocation false accepts | 0 |
| Rights/interest omissions on a closed expert-labelled corpus | Report by type; target ≥99% recall |
| Mandatory LADM/STDM/OGC profile fields silently lost in a round trip | 0 |
| Transitive evidence invalidations omitted | 0 |
| Stale derived projections left marked current after a source correction | 0 |
| Unauthorized disclosure of restricted community/heritage/sensitive location data | 0 |
| Offline or concurrent-edit conflicts silently overwritten | 0 |
| Unfunded stewardship obligations represented as completed | 0 |
| Participating-user task completion, correction burden and accessibility | Establish stratified baselines; no synthetic proxy for consent |

No composite “trust score” should collapse legal uncertainty, spatial error, community legitimacy, environmental measurement and software reliability into one number. Report each dimension and its uncertainty separately.

## 6. Competitive posture

Open Tenure already offers community-led, multilingual and offline tenure recording, synchronization and moderation. STDM provides a mature conceptual model for informal, customary and overlapping people-to-land relationships and is a normative annex in LADM Part 2. Cadasta documents community data control, FPIC and CARE-aligned governance. These are benchmark strengths to preserve, not deficiencies to market against.

The proposed Luminous differentiation is narrower and testable: maintain evidence lineage, authority scope, temporal/vertical uncertainty, correction/revocation propagation and safe transaction reconciliation **across** those existing tools and formal land institutions. It is not yet demonstrated as a superiority claim.

## 7. Immediate next work

1. Freeze the FELA pathway evidence labels into TIB result packets.
2. Add low-connectivity, assisted and non-digital process vectors; do not rebuild Open Tenure's mobile capability without a matched unmet need.
3. Add partner and user-study protocols before field claims are made.
4. Build exact data-export/exit and correction workflows against Cadasta/Open Tenure expectations.
5. Connect Symthaea/Symtropy read-only evidence analysis to CIVOS/EPI-008 correction and invalidation events.
6. Treat Property host-level legal-effect gating and community disclosure enforcement as separate runtime acceptance work, not as complete because a reference schema exists.
