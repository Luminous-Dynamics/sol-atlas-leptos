# RFC-TERR-001 — Territorial Rights, Mineral Interests & Stewardship Evidence

**Status:** Draft research RFC / v0.1 data-contract implementation  
**Date:** 2026-11-18  
**Repository:** `Luminous-Dynamics/sol-atlas-leptos`  
**Scope:** Sol Atlas spatial/evidence interchange boundary; cross-project integration with Mycelix, Symthaea and Symtropy  
**Legal effect:** None. This software does not register, create, transfer, adjudicate or extinguish rights.

## 1. Executive proposal

Build a federated **Territorial Rights & Stewardship Evidence Fabric** that represents land and natural-resource relationships as time-bounded, spatially explicit, evidence-linked assertions.

The fabric is not another title registry, cadastral authority, tokenization layer or legal AI. It is an interoperability and assurance layer connecting:

- spatial units and their geometries;
- parties, communities and institutions;
- rights, responsibilities and restrictions (RRRs);
- the evidence and authority supporting each assertion;
- overlapping, contested, expired and superseded claims;
- environmental observations and uncertainty;
- stewardship obligations, independent verification and remedies.

The first engineering increment is a deliberately bounded Rust data contract in `sol-atlas-core::territorial`. It records assertions without silently promoting them to legally established rights. It keeps dispute state independent of the basis on which an assertion was recorded and requires provenance links for administrative, judicial, community-attested and contractual bases.

The next increment is a synthetic adversarial corpus and property-based validation suite. Public-data ingestion and live community pilots come later, with legal, surveying, environmental and community governance partners.

## 2. Why this is worth doing

Land, minerals, water, biodiversity, infrastructure, heritage and community livelihood are often managed in separate systems. A map can show a parcel but not necessarily explain the legal or customary relationships affecting it. A title database may be authoritative for a particular class of registered interest but is not a complete account of occupation, informal tenure, customary use, cultural obligations or environmental constraints. Satellite observations can show a land-cover change but cannot establish who owns, occupies or is authorised to use the affected land.

The international need is substantial. FAO's 2026 land-tenure material reports that around 1.1 billion adults feel insecure about their land rights and that only 35% of land worldwide is formally documented. These figures are global context, not a forecast of the market or an estimate of the addressable South African population.

South Africa is a useful first jurisdiction because surface rights, informal and customary tenure, mineral authorisations, water-use permissions, heritage and biodiversity interests interact. The Mineral and Petroleum Resources Development Act establishes the State's custodianship role for the nation's mineral and petroleum resources. That legal structure is not equivalent to surface ownership. Any South African implementation must also preserve the legal distinctions and procedural protections applicable to informal and customary rights, and must be reviewed by qualified local practitioners.

South Africa's Biodiversity Stewardship programme is a strong institutional starting point rather than a system to displace. The World Bank's September 2026 assessment reports approximately 2.8 million hectares across 469 sites and identifies cost and partnership advantages. This creates a credible route to test environmental stewardship records alongside rights and obligations.

## 3. Paradigm shifts

### 3.1 From parcel registry to a layered territorial graph

A territory is not a single polygon with one owner field. It is a set of spatial and temporal relationships that can overlap and have different authorities.

Represent at least:

- surface parcels and occupation areas;
- underground mineral or geological spaces;
- aquifers, catchments and surface-water systems;
- servitudes, transport and utility corridors;
- customary territories, communal use areas and cultural landscapes;
- protected areas, biodiversity features and restoration sites.

Each interest has its own spatial reference, vertical extent where applicable, temporal validity, evidence and dispute state. A surface parcel is not a substitute for the geometry of a mining right, aquifer or underground infrastructure.

### 3.2 From "the record" to evidence-qualified assertions

A record is not necessarily true merely because it is digital, signed, replicated or immutable.

Keep distinct:
- what a party asserts;
- what a community process attests;
- what an administrative register records;
- what a court or tribunal determines;
- what a contract states;
- what a statute or regulation provides;
- what a sensor or survey observes.

These bases are not one universal confidence ladder. A registered interest can be challenged. A community-attested interest may be legally significant without being registered. A remote-sensing observation can corroborate land use but cannot establish tenure. The model must preserve distinctions rather than blend them into a confidence score.

### 3.3 From static maps to four-dimensional legal and physical space

Treat geography, depth, time and authority as separate dimensions. Boundaries can change; rights can commence, expire or be suspended; mines and infrastructure occupy subsurface volumes; aquifers and ecosystems do not respect cadastral parcel edges.

The first schema references geometry externally and records the CRS, optional vertical extent and uncertainty. It does not yet implement 3D topology or a geodetic transformation engine. Those are separate qualification tasks.

### 3.4 From document storage to proof-carrying change

A consequential territorial change should be represented as a proposal with a complete, reviewable evidence package:

1. proposed activity and geographic/time scope;
2. initiating party and claimed authority;
3. affected rights, restrictions and duties;
4. authoritative instruments and source evidence;
5. consultation, notice and consent/approval steps where legally required;
6. unresolved objections and challenges;
7. environmental baseline and impact assumptions;
8. mitigation, rehabilitation and financial obligations;
9. accountable parties, independent verification and appeal/remedy route.

The system can test whether required evidence is missing, stale, inconsistent or outside scope. It must not decide that a legally required consent or authorisation exists merely because a workflow checkbox is complete.

### 3.5 From self-reported restoration to independently evidenced stewardship

A rehabilitation promise is not a rehabilitation outcome. Obligations should remain open until physical progress and appropriate independent evidence are both recorded. Evidence must identify the protocol, observation time, responsible party and verifier.

Symthaea's subterranean stewardship work already uses a useful principle: physical progress alone does not complete an obligation; externally attributable evidence is also required. This RFC carries that evidence boundary into the territorial interchange model. It does not claim that a reference or hash proves the measurement is scientifically sound.

### 3.6 From "open data by default" to FAIR + CARE

Interoperability is important, but publishing every boundary, testimony, sacred-site location, personal record or customary knowledge item can cause harm.

Use FAIR concepts for discoverability and interoperability where appropriate, and CARE principles for community data: Collective Benefit, Authority to Control, Responsibility and Ethics. Data access must be purpose-bound and controlled by the legitimate data stewards. The v0.1 disclosure enum defaults to `Restricted` on evidence, authority, challenge, assertion and obligation records; this is a metadata default, not an access-control implementation.

### 3.7 From one central platform to polycentric federation

Do not attempt to replace national registries, survey authorities, courts, customary governance institutions, conservation agencies or community-controlled records.

Each source remains authoritative only within its actual mandate. Sol Atlas supplies spatial context and interoperable views; Mycelix coordinates identity, governance and attestations; Symthaea performs evidence and rule analysis; Symtropy models physical and ecological consequences. The platform preserves source references and disagreement rather than manufacturing a universal source of truth.

## 4. Existing standards and industry baseline

This programme should reuse mature standards rather than invent a proprietary ontology.

| Area | Existing baseline | What Luminous should do |
|---|---|---|
| Land-administration semantics | ISO 19152-1:2024 LADM generic conceptual model | Map parties, spatial units, sources and RRRs; document jurisdiction profiles |
| Land-registration semantics | ISO 19152-2:2025 LADM land registration, including above- and below-surface spatial elements | Keep registration evidence and spatial representations distinct |
| Geospatial feature access | OGC API Features | Use feature/collection identifiers and standard query semantics |
| Geospatial asset catalogues | STAC 1.1 / STAC API 1.0 | Reference imagery, DEM, LiDAR, SAR and derived data with temporal metadata |
| Geospatial metadata discovery | OGC API Records | Expose searchable evidence and dataset metadata |
| Responsible tenure governance | FAO VGGT | Translate principles into auditable process controls, without misrepresenting voluntary guidance as national law |
| Community data governance | CARE Principles | Give communities meaningful control over collection, disclosure and secondary use |
| Extractive-sector transparency | EITI Standard | Integrate license, contract-allocation and beneficial-ownership disclosures where available |
| Responsible mine assurance | IRMA Standard | Map relevant human-rights, engagement, grievance, environmental and closure evidence |
| Tailings management | Global Industry Standard on Tailings Management | Support lifecycle evidence, affected-community engagement and disclosure |
| Earth observation | Digital Earth Africa, Landsat/Sentinel-derived products and other trusted providers | Track observable change with source-specific uncertainty and field validation |

### What existing offerings already do well

- **GIS and geospatial platforms** provide mature geometry, visualization, spatial analysis, imagery and operational mapping. We should build on their standards and data products, not compete by rewriting a GIS.
- **National cadastral and deeds systems** are authoritative for specific records under their legal mandates. We must interoperate with them and must not imply that our mirror is the legal register.
- **Land-administration frameworks such as LADM and fit-for-purpose approaches** provide the established semantic and implementation foundations for formal, informal, customary and collective tenure. We should extend interoperability and evidence assurance, not claim to have invented inclusive tenure mapping.
- **Earth-observation platforms** can identify observable land-cover, water and infrastructure changes at scale. They cannot infer legal title, consent or representative legitimacy.
- **Mining assurance frameworks** already define important company-level social, environmental, governance, tailings and closure expectations. Our opportunity is to link commitments to spatial scope, source evidence, current observations, open challenges and remediation.
- **Blockchain or distributed registries** can preserve ordering and tamper-evident histories. They do not establish that the original assertion was true, that the issuer had authority, that a community representative was legitimate, or that disclosure was ethical.

The proposed advantage is the integration of these disciplines into a verifiable, contestable, privacy-aware evidence lifecycle.

## 5. South African profile and data strategy

### 5.1 Legal boundary

The first country profile should be South Africa, but its legal rules must be curated and reviewed separately from the generic data model.

The profile must distinguish at minimum:
- registered surface interests and cadastral/survey records;
- lawful occupation, informal land rights and customary tenure;
- mineral and petroleum permissions/rights;
- water-use authorisations and resource constraints;
- environmental authorisations and financial provisioning;
- heritage and protected-area restrictions;
- community decisions, mandates, consultation records and disputes;
- court orders, administrative decisions and appeal status.

The system must preserve the date, version and jurisdiction of the applicable legislation and instrument. Legal conclusions require qualified review. An LLM-generated explanation is not a legal opinion or an authoritative rights determination.

### 5.2 Public-first, consent-aware data

Start with published legal instruments, open spatial layers and public environmental observations. Do not scrape or republish personal data, private title documents, sensitive community testimony, sacred locations or restricted customary knowledge.

Candidate data classes:
- public law, regulations, gazettes and judgments;
- public cadastral and mining-cadastre information, subject to source terms;
- public protected-area, biodiversity and heritage layers;
- public water, catchment, geology and land-cover products;
- synthetic records for disputed and overlapping rights;
- field observations only after method, consent, retention and disclosure rules are agreed.

All ingestion adapters must retain source URI, retrieval time, issue/effective time when known, licence/terms, coordinate reference system, version, transformation history and content digest. A digest establishes byte-level identity only; it does not establish legal authenticity or scientific validity.

### 5.3 Earth-observation caveat

Digital Earth Africa products are valuable for time-series analysis. However, its published Fractional Cover documentation explicitly warns that the service has not been quantitatively evaluated in Africa because of a lack of suitable field data and is provisional. Therefore it can support screening and prioritisation, but cannot by itself certify habitat restoration, soil condition or compliance. Field-calibrated and independently reviewed evidence is required for consequential decisions.

## 6. v0.1 data contract

The initial Rust module is `sol-atlas-core::territorial`.

It defines:
- `TerritorialRecord`: versioned container for territory assertions, evidence, authority references, challenges and obligations;
- `TerritorialAssertion`: one party's right/responsibility/restriction relationship to a spatial and temporal extent;
- `ClaimBasis`: asserted, community-attested, administratively recorded, judicially determined, contractual or statutory source basis;
- `DisputeState`: orthogonal to the basis so registered records can remain challenged;
- `EvidenceReference`: source URI, evidence kind, capture time, optional SHA-256 and disclosure class;
- `AuthorityReference`: issuer, jurisdiction, scope, source evidence and validity interval;
- `ChallengeReference`: challenger reference, filing time, state and supporting evidence;
- `StewardshipObligation`: responsible party, location, originating evidence, due date, verification protocol, independent verifier and completion evidence.

### Structural invariants

The v0.1 validator rejects:
- unsupported schema versions, empty/duplicate IDs and missing required references;
- missing source evidence for authority records;
- malformed SHA-256 strings;
- negative/non-finite spatial uncertainty or invalid vertical extents;
- missing evidence for assertions;
- administrative-record bases without registry/regulator authority and registry/permit evidence;
- court-decision bases without court/tribunal authority and court-order evidence;
- community-attested bases without a recorded community-decision evidence item (testimony alone is not enough);
- contractual/statutory bases without corresponding instrument evidence;
- challenged/reviewed/resolved states without challenge references;
- verified-complete obligations without a separate verifier and completion evidence.

### Explicit v0.1 non-claims and gaps

This is a structural Rust contract only. It does not yet:
- authenticate signatures or authority credentials;
- verify the source URI, digest, issuer or legal instrument;
- prove that a customary institution or community representative is legitimate;
- enforce consent, notice, consultation or data-access policy;
- parse or compare RFC 3339/date intervals;
- validate GeoJSON geometry, topology, survey quality or coordinate transformations;
- detect spatial intersections or resolve boundary conflicts;
- provide cryptographic signatures, encryption or key management;
- establish environmental measurement quality, laboratory accreditation or restoration success;
- connect to Project Indwe, Deeds Registry, Surveyor-General, Mycelix conductor or Symtropy runtime;
- provide a user interface or production deployment.

The validator is a fail-closed structural preflight, not a legal or scientific certification engine.

## 7. Cross-project responsibility boundaries

| Project | Responsibility | Must not do |
|---|---|---|
| Sol Atlas | Spatial catalogue, time/depth-aware visualization, source provenance, territorial query and evidence views | Become a national register or decide title |
| Mycelix | Party/organisation credentials, governance process, attestation, challenge routing, consent and agreement workflows | Treat distributed consensus as proof of legal truth or auto-transfer disputed rights |
| Symthaea | Source-grounded legal/policy retrieval, conflict detection, missing-evidence analysis, counterfactual comparison and explanation | Grant permits, decide community representation, infer consent or silently resolve disputes |
| Symtropy | Physical/ecological digital twin, baselines, impact scenarios, sensor/field observation and rehabilitation monitoring | Treat simulation as measured reality or certify outcomes without independent evidence |

The inter-project interface should be versioned, language-neutral and evidence-carrying. The Sol Atlas core types are the starting interchange contract, not a mandate for every runtime to adopt the same internal storage engine.

## 8. Territorial Integrity Benchmark (TIB)

There is no single public benchmark that combines tenure semantics, competing claims, geospatial overlap, legal authority, community data sovereignty, environmental monitoring and restoration verification. TIB should fill that evaluation gap, not compare unrelated headline metrics.

Build a versioned corpus with:
1. synthetic adversarial cases;
2. de-identified, permissioned case studies reviewed by domain experts;
3. public-source cases with stable source snapshots and known limitations;
4. held-out jurisdictions to test whether the core model generalises without assuming South African legal rules apply elsewhere.

Do not include real personal data, sacred-site coordinates or confidential community records in the public corpus.

### Benchmark tracks and proposed targets

Targets below are research goals, not current results.

| Track | Evaluation | Initial target |
|---|---|---|
| Provenance integrity | All material assertions resolve to source evidence; missing/duplicate/replayed links | 100% invariant enforcement in deterministic tests |
| Claim-status integrity | No automated promotion from asserted/attested to registered/adjudicated | Zero silent promotions |
| Dispute preservation | Retain a challenge independently of the registry/contract basis | 100% preservation across import/export/replay |
| Authority matching | Correctly require registry evidence for administrative-record claims and court evidence for judicial claims | 100% on typed contract cases |
| Spatial conflict detection | Precision/recall for 2D and 3D overlap against surveyed synthetic ground truth | Target ≥99% recall on in-scope synthetic cases; report error by geometry class |
| Boundary accuracy | Difference between supplied and reconstructed geometry | Within source survey tolerance; report CRS and uncertainty, never one universal metre threshold |
| Stakeholder omission | Recall of affected right-holders identified by expert-reviewed case files | Target ≥99% on a closed, labelled corpus, with harm-weighted false-negative reporting |
| Evidence retrieval | Source-level recall@k, exact citation/paragraph accuracy and version correctness | Target ≥99% recall for required controlling sources; no unsupported legal answer |
| Temporal correctness | Expiry, supersession, revocation, appeal and retroactive correction handling | 100% of deterministic temporal edge cases |
| Stewardship completion | Obligation closure without valid evidence or independent verifier | Zero invalid closures |
| Environmental monitoring | Error and calibration against field observations by sensor/product/site | Set per measured variable after baseline collection; no synthetic universal threshold |
| Community data governance | Access-control and secondary-use policy tests, disclosure leakage and revocation | Zero critical unauthorised disclosure in adversarial test suite |
| Operational inclusion | Offline capture, low-bandwidth sync, multilingual workflow completion | Measured task completion and error rates with participating users; no launch threshold before user research |

Use precision, recall, F1, calibration error, Brier score where probabilistic outputs are justified, geometric error, temporal consistency, latency, cost and user task outcomes. Never compress legal status, rights security and ecological condition into one composite "trust score".

### Mandatory adversarial suites

- forged or altered deed, permit, court order or survey;
- real issuer but wrong jurisdiction or expired delegation;
- overlapping claims from two communities;
- registered record with a pending challenge;
- a challenge that is withdrawn, rejected or adjudicated;
- inaccurate geometry and CRS mismatch;
- surface parcel incorrectly reused as a subsurface volume;
- missing affected stakeholder or excluded household;
- compromised community representative or duplicate voting identity;
- evidence replay, stale credential, clock skew and source revocation;
- satellite change alert incorrectly presented as proof of illegal occupation;
- missing/biased environmental baseline;
- self-reported rehabilitation presented as independent verification;
- sensitive cultural information leaked through maps, exports, logs or model prompts;
- malicious actor attempts to lower a confidence threshold or rewrite provenance.

For safety- and rights-critical cases, false negatives require more scrutiny than a dashboard's overall accuracy. Every incorrect omission should be severity-labelled and reproducible.

## 9. What would make this better than current practice

Do not claim "better than SOTA" until matched tests exist. The proposed route to superiority is:

1. **Cross-layer coverage:** join spatial, legal, customary, environmental and operational evidence in one temporal graph.
2. **No authority laundering:** technical attestations never become legal authority by accumulation or consensus.
3. **Contestability by design:** every material assertion has a challenge path and a way to preserve dissent.
4. **Uncertainty first:** store the source's measurement accuracy, coverage gaps and model limitations rather than presenting a crisp map as certainty.
5. **Rights-holder control:** community-governed information has purpose-bound access and consent-aware reuse.
6. **Physical outcome loop:** obligations are continuously compared with observations and independent field verification.
7. **Federated interoperability:** connect to authoritative government and professional systems instead of replacing them.
8. **Reproducible decisions:** export the inputs, rules, evidence versions, reviewers, dissent and outputs needed for an independent replay.
9. **Cost and inclusion:** support incremental, fit-for-purpose adoption, low-bandwidth/offline work and non-digital participation.

The advantage is not a faster map or a more impressive AI answer. It is fewer unsupported assertions, fewer overlooked rights-holders, clearer accountability, earlier conflict discovery and independently evidenced environmental outcomes.

## 10. Delivery plan

### M0 — contract foundation (current PR)
- Add the Rust territorial interchange types and structural validator.
- Add tests for asserted claims, administrative-record basis, court decisions, contested registered records, digest/geometry metadata and independent obligation verification.
- Publish this RFC with explicit boundaries and benchmark plan.

### M1 — reproducible synthetic corpus
- Define machine-readable fixtures for parcel, communal-use, mineral-right, water, conservation and heritage overlap.
- Add adversarial fixtures for conflicting claims, authority expiry, appeals, vertical extents, stale data and sensitive disclosure.
- Add deterministic validation and replay tests.
- Record exact source and schema versions in fixture manifests.

### M2 — spatial proof layer
- Add geometry ingestion and validation against OGC API Features-compatible resources.
- Support feature references, CRS transformation metadata, topology checks, vertical extents and source uncertainty.
- Produce a map view that visibly separates assertion basis, dispute state, source freshness and confidence/uncertainty.
- Never display an unverified claim with the same visual treatment as an authoritative registered record.

### M3 — Mycelix and Symthaea adapters
- Map governance attestations and challenges to Mycelix credentials and process receipts.
- Add a Symthaea evidence-analysis adapter that returns source-linked conflicts and missing prerequisites rather than a legal verdict.
- Add authority-scope, credential expiry and revocation tests.

### M4 — Symtropy stewardship loop
- Bind a physical baseline, observation protocol, sensor/source metadata and field-verification record to a spatial unit.
- Compare predicted impacts to observations with uncertainty bounds.
- Keep obligations open until the declared acceptance protocol is met by independent evidence.
- Establish site-specific scientific review before any real-world assurance claim.

### M5 — supervised South African pilot
- Select a landscape only with willing community and institutional partners.
- Start with public/synthetic records; do not ingest sensitive information until governance, consent, access and retention are agreed.
- Have South African land, mining, water, environmental, customary-law, surveying and community practitioners review the profile.
- Evaluate against a manually curated baseline and publish the errors, omissions and unresolved cases, not just successes.

## 11. Release gates

A prototype can be released only as a prototype with the limitations above visible.

A pilot that informs real-world decisions additionally requires:
- legal review by appropriately qualified South African practitioners;
- professional survey/geospatial review;
- environmental-science and measurement review;
- documented community governance and data-use agreements;
- independently reviewable challenge and appeal processes;
- cryptographic provenance and key-management review;
- privacy, access-control, security and data-retention assessment;
- field validation with explicit uncertainty reporting;
- a rollback/reconciliation path for bad, stale or retracted source data.

No CI result, benchmark result or simulation result can substitute for these external gates.

## 12. Research references

- [ISO 19152-1:2024 — LADM generic conceptual model](https://www.iso.org/standard/81263.html)
- [ISO 19152-2:2025 — LADM land registration](https://www.iso.org/standard/81264.html)
- [OGC API Features](https://www.ogc.org/standards/ogcapi-features/)
- [OGC API Records](https://ogcapi.ogc.org/records/)
- [STAC 1.1 / STAC API 1.0](https://www.ogc.org/standards/stac/)
- [FAO Voluntary Guidelines on the Responsible Governance of Tenure (VGGT)](https://www.fao.org/tenure/voluntary-guidelines/en/)
- [Global Indigenous Data Alliance — CARE Principles](https://www.gida-global.org/careprinciples)
- [EITI Standard and guidance](https://eiti.org/guide-implementing-eiti-standard)
- [IRMA Standard](https://responsiblemining.net/what-we-do/standard/irma-mining-standard/)
- [Global Industry Standard on Tailings Management](https://globaltailingsreview.org/global-industry-standard/)
- [World Bank — 20 years of South African Biodiversity Stewardship, 8 September 2026](https://www.worldbank.org/en/brief/2026/09/08/analysis-of-20-years-of-implementing-south-africa-s-biodiversity-stewardship-progr)
- [Digital Earth Africa — Fractional Cover and its evaluation caveat](https://docs.digitalearthafrica.org/en/latest/data_specs/Fractional_Cover_specs.html)
- [South African Mineral and Petroleum Resources Development Act](https://www.gov.za/ve/documents/mineral-and-petroleum-resources-development-act)
- [Constitutional Court — Tongoane and Others](https://collections.concourt.org.za/items/e7459d6e-99bf-4f4d-af49-ca7e30b69475/full)

---

**Current implementation status:** the Rust v0.1 type module and unit tests have been added in this branch; this RFC records intended scope and known limitations. No local build, test run, CI result, external legal review, survey validation or field trial is claimed by this document.
