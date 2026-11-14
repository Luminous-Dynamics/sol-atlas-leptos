# AeroWeft South African Aviation Source Admission V0.1

**Tracking issue:** [AEROWEFT-SA-DATA-001 #825](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/825)  
**Owner:** Sol-Atlas  
**Consumers:** Symtropy AeroWeft #5163 / #5181 / #5186; Mycelix #5023; Symthaea #6249  
**Status:** source-inventory research; no live ingestion or operational use  
**Checked:** 14 November 2026 (Africa/Johannesburg)

## 1. Why this matters

South African aviation source data is not one uniform feed. AIP publications, temporary NOTAMs, source access terms, weather products, spatial reference systems and operational approval all have different owners, validity periods and reuse conditions.

AeroWeft must distinguish:
- source is publicly visible;
- source is current for a particular stated purpose;
- source is authoritative for a particular claim;
- source is geometrically/structurally parseable;
- source may legally be acquired, stored, transformed, redistributed or used commercially;
- source is suitable for an operational workflow.

One condition never implies the others.

## 2. Source inventory checked

### 2.1 SACAA AIP

SACAA's [AIP Amendments page](https://www.caa.co.za/industry-information/aip-amendments/) currently lists:
- AIP General — 15 October 2026;
- AIP Enroute — 15 October 2026;
- AIP Aerodromes — 15 October 2026;
- amendment artefacts, including a control sheet, ENR material, aerodrome data and charts.

**Admission:** CANDIDATE. The public page establishes the visible product list and the displayed edition date. It does not by itself establish:
- automated-acquisition rights;
- archival/retention rights;
- extraction/derivative geometry rights;
- public redistribution rights;
- commercial model-use rights;
- service-level/freshness guarantees for any particular derived field.

Those permissions require exact terms and source-owner confirmation. Do not assume a publicly downloadable PDF is a commercially reusable spatial dataset.

### 2.2 SACAA NOTAM summaries

SACAA's [NOTAM Summary page](https://www.caa.co.za/industry-information/aeronautical-information-notam-summaries/) displayed a latest update of 12 November 2026 when checked.

The page explicitly says:
- the posted summary is valid only at the time it is created;
- it must not be used for flight preparation;
- the latest NOTAM in force are available through the Centralised Aeronautical Information Management Unit;
- pilots are directed to the ATNS authenticated File2Fly self-briefing service for route-, aerodrome- and zone-based briefings.

**Admission:** SUMMARY_CONTEXT_ONLY / RIGHTS_AND_ACCESS_PENDING.

This is an especially important negative control. A public NOTAM summary can support historical research about a snapshot, but it is not the complete current NOTAM set and is not a flight-preparation service. Do not scrape protected services, bypass authentication or turn a public snapshot into operational authority.

### 2.3 Civil Aviation Regulations

The South African Government lists [Government Gazette 55226](https://www.gov.za/documents/latest?o=3&page=11) dated 21 August 2026 and [Notice 7844 / the Thirty-Third Amendment of the Civil Aviation Regulations 2026](https://www.gov.za/sites/default/files/gcis_document/202608/55226gon7844.pdf).

The amendment includes changes to Part 101 regulation 101.01.2's private-operation provision. The narrow private-operation carve-out refers to R-VLOS with Class 1A or Class 1B RPA and prohibits dangerous goods aboard a UA.

**Admission:** SOURCE_VERIFIED / LEGAL_APPLICABILITY_REVIEW_PENDING.

This is not the entire Part 101 regime, not blanket commercial or BVLOS permission, and not a determination that a specific medical payload is permitted. Exact consolidated text, commencement and operation/payload classification require SACAA / aviation-counsel review.

AeroWeft should store regulation references, amendment date, effective date, jurisdiction, affected provision and review state as versioned legal-context records. It should not attempt to render a generic "allowed" result from a regulation title or one excerpt.

### 2.4 Visual line-of-sight versus radio line-of-sight

A checked SACAA UAS information page lists R-VLOS, VLOS, EVLOS and BVLOS as operating modes under its **Radio line-of-sight (RLOS)** heading. It describes RLOS as a direct electronic point-to-point link and says that BRLOS (beyond radio line-of-sight) is reserved for future use.

This must be read alongside, not substituted for, the regulation text:

- The official Government Gazette 48228 / Notice R.3170 (17 March 2023), which substitutes Part 101 regulation 101.05.11, states that a UA must not be operated BVLOS unless otherwise approved by the Director in a UASOC's operations manual; the Director may approve B-VLOS subject to SA-CATS 101; approved B-VLOS is limited to VMC and below 400 ft above surface level unless otherwise approved. See the official Gazette PDF, printed page 194 / PDF page 141: https://www.gov.za/sites/default/files/gcis_document/202303/48228rg11556gon3170.pdf
- The Thirty-Third Amendment (Gazette 55226 / Notice 7844, 21 August 2026) separately substitutes regulation 101.01.2: **private operations** are limited to R-VLOS with a Class 1A or 1B RPA, and dangerous goods may not be carried aboard a UA. This private-operation carve-out must not be generalized to all commercial/corporate/non-profit operations, nor treated as a general BVLOS authorization.
- SACAA's current UAS information page lists approval requirements by operation type, including UASOC and Air Service Licence requirements for commercial operations. The page is a summary; applicability, exact current consolidated text, OpSpecs, approvals and payload classification still require provision-level review and SACAA/legal confirmation.

**Critical semantic distinction: BVLOS is not synonymous with BRLOS.** A flight can be beyond the remote pilot's visual line of sight while remaining within the declared radio-link line-of-sight conditions. Conversely, a network relay or beyond-radio-link concept cannot be inferred to be permitted merely because a BVLOS approval path exists.

**Admission state:** SOURCE_VERIFIED / OPERATION_SCOPE_REVIEW_REQUIRED. Preserve separate fields for:
- visual operation rule: R-VLOS / VLOS / EVLOS / BVLOS;
- radio-link scope: RLOS / BRLOS / unknown;
- operator category: commercial / corporate / non-profit / private;
- aircraft class and applicable Part 101 scope;
- UASOC and OpSpec identity/status;
- Director approval and operations-manual scope;
- SA-CATS 101 conditions;
- applicable weather, altitude, airspace and payload restrictions;
- source edition/effective date and legal review result.

Do not return a single boolean "BVLOS allowed". Do not convert a missing radio-link classification or approval record into permission. The private-operation R-VLOS/Class 1A/1B/dangerous-goods restrictions must remain scoped to private operations under the 2026 amendment.

Sources checked 14 November 2026:
- SACAA UAS information: https://www.caa.co.za/industry-information/uas/
- Official Government Gazette 48228 / 26th Amendment (Part 101.05.11 B-VLOS approval path): https://www.gov.za/sites/default/files/gcis_document/202303/48228rg11556gon3170.pdf
- Official Government Gazette 48228 / Notice R.3170 / 26th Amendment (Part 101.05.11): https://www.gov.za/sites/default/files/gcis_document/202303/48228rg11556gon3170.pdf
- Official Government Gazette 55226 / Notice 7844 / 33rd Amendment (private-operation rule): https://www.gov.za/sites/default/files/gcis_document/202608/55226gon7844.pdf

## 3. Candidate sources not yet admitted

| Source family | Candidate owner | Needed review | Current state |
|---|---|---|---|
| ATNS AIP/NOTAM briefing services | ATNS / AIMU | Access agreement, API, authentication, use rights, updates, service guarantees | RIGHTS_AND_ACCESS_PENDING |
| Aviation weather | South African Weather Service | Product/API access, observation/forecast semantics, coverage, update intervals, commercial use | CANDIDATE |
| Aerodrome/runway/heliport details | SACAA / aerodrome operators | Exact authority, edition, data rights, operational status and capacity semantics | CANDIDATE |
| Terrain/obstacle/geospatial data | Relevant survey/geospatial publishers | CRS, vertical datum, accuracy, update process, licence and transformation rights | CANDIDATE |
| Ground transport/access | Road/rail/public-service data owners | Geographic coverage, observation period, licence, service vs geometry distinction | CANDIDATE |
| Health facility/medical logistics context | Health-service owners / public datasets | Privacy, exact location sensitivity, access rights, service availability and custody rules | CANDIDATE |

No live feed, protected account or personal flight data is required for the first synthetic demonstrator.

## 4. Source-record semantics

Reuse the existing Sol-Atlas [AIRSPACE-ATLAS-001 #189](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/189) source-validity design and [PR #196](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/196) schema/contract checker.

Every source record should preserve:
- publisher, canonical product identity, edition/revision and locator;
- source-issued, source-updated, retrieved and valid-from/valid-until times;
- exact raw-byte digest and byte length;
- authentication/signature state (not merely digest syntax);
- CRS, coordinate epoch if relevant, axis order, vertical datum, units and transformation lineage;
- geographic/time/altitude coverage and known omissions;
- acquisition, storage, transformation, redistribution, publication and model-use permissions separately;
- claim-specific authority and applicability scope;
- parser/schema/toolchain identity;
- status, supersession, revocation, conflict and correction history;
- exact downstream derived artefacts and the claims they support.

Suggested states:
CANDIDATE, RIGHTS_PENDING, RIGHTS_CLEARED, DEVELOPMENT_ONLY,
CONFIRMATORY_ONLY, OPERATIONAL_INTEGRATION_APPROVED, REJECTED.

These are source-use/evidence states, not flight authorizations or certification decisions.

## 5. Initial synthetic fixtures

Add no live operational data in V0.1. Use fictional records for:
1. an AIP issue whose edition date is known but reuse rights are pending;
2. a previous AIP issue superseded by a later edition;
3. a NOTAM summary with a snapshot timestamp and explicit not-for-flight-preparation limitation;
4. a protected briefing service with access and permission unknown;
5. a geometry record with an unknown vertical datum;
6. a mapped aerodrome with unknown operational status/capacity;
7. a regulation excerpt with a known provision but unresolved applicability to a proposed commercial cargo operation;
8. a visually-BVLOS operation whose radio-link state is RLOS, with separate private/commercial operator scopes and approval status;
9. a BRLOS request incorrectly inferred from a BVLOS approval, which must remain unsupported/unqualified.

Expected outcomes should be frozen and independently checked.

## 6. Acceptance

- No public summary is promoted to a complete live operational feed.
- No stale/expired/superseded publication silently becomes current.
- No rights-pending source is used for commercial model training, redistribution or production.
- No missing restriction is interpreted as an unrestricted volume.
- No geometry implies operational status, permission or capacity.
- No regulation excerpt becomes a legal permission without provision-level applicability review.
- BVLOS and BRLOS are stored as distinct visual-rule and radio-link dimensions.
- A BVLOS approval is never interpreted as a BRLOS authorization.
- The 2026 private-operation R-VLOS/Class 1A or 1B/dangerous-goods restrictions are not generalized to every operator category.
- All outputs reproduce from exact source bytes, parser revision and manifest.
- Each downstream field retains the source/claim scope and unresolved dependencies.

## 7. Integration boundaries

- Sol-Atlas owns source admission, read-only spatial context, freshness, geometry semantics, source rights and lineage.
- Symtropy consumes explicit snapshots and unknown states for synthetic planning/reachability benchmarks; it does not infer operational permission.
- Symthaea owns aircraft physics/profile applicability, not regulatory interpretation.
- Mycelix may store identities, access decisions and provenance receipts; consensus does not establish a source's physical or legal truth.

No live flight-plan ingestion, aircraft tracking, protected NOTAM scraping, pilot/crew personal data, operational route recommendations, flight clearance or aircraft control is added.

## 8. Reference links

- SACAA AIP amendments: https://www.caa.co.za/industry-information/aip-amendments/
- SACAA NOTAM summaries: https://www.caa.co.za/industry-information/aeronautical-information-notam-summaries/
- SACAA UAS information: https://www.caa.co.za/industry-information/uas/
- Government Gazette 55226 / Notice 7844: https://www.gov.za/sites/default/files/gcis_document/202608/55226gon7844.pdf
- Sol-Atlas AIRSPACE-ATLAS-001: https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/189
