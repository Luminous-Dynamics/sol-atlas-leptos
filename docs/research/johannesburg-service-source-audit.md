# Johannesburg Service-Discovery Source Audit

**Observed:** 2026-10-17  
**Status:** preliminary public-source inventory; no bulk data import or redistribution is authorized by this note.  
**Companion:** [CITY-CONNECTION-001](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/123)  
**Parent design:** [City Belonging Navigator](city-belonging-navigator.md)

## Executive finding

There are credible public starting points for a Johannesburg service-discovery pilot, but this first audit does **not** establish a complete or currently accurate service directory.

Three important source classes appeared during research:

1. The City of Johannesburg's public library page describes libraries and associated activities, and links to library contact lists by the City's seven regions. The page states that libraries host literacy classes, children's storytelling, gatherings for older residents, lectures, exhibitions and talks. Its footer displays © 2018 City of Johannesburg; the inspected page does not establish when each record or schedule was last checked.
2. The City's Joburg Cares voluntary-organisations page describes support categories including community-policing forums, addiction-related groups, animal welfare and organisations supporting vulnerable children. It explicitly says more categories will be added and the footer displays © 2018 City of Johannesburg; this is a lead, not a current completeness claim.
3. The National Development Agency's Gauteng CSO Footprint page currently displays DISTRICT: CITY OF JOHANNESBURG (578 CSO's) and lists organisations by sector. The inspected page does not show a visible last-updated timestamp, and the research has not established a reusable bulk export, API, licence or record-level operation status.

Therefore, the starting problem is **source qualification and provider verification**, not map rendering. Do not mass-import these pages as live services or infer that a listed organisation is open, reachable, accepting new participants or safe merely because a record appears online.

## Candidate source registry

| Source ID | Public source | Potential value | Known caveat from this audit | Admission status |
|---|---|---|---|---|
| COJ-LIBRARIES | https://joburg.org.za/departments_/Communitydevelopment/Pages/Libraries%20And%20Information%20Services/libraries-and-information-services.aspx | Library network, activities, regional contact pages; libraries may serve as inclusive connection points beyond borrowing books. | Footer says © 2018; record-level freshness, reuse permission and accessibility/schedule currentness are not yet established. | Lead only; verify individual records and terms. |
| COJ-VOLUNTARY-ORGS | https://joburg.org.za/services_/JoburgCares/Pages/voluntary-organisations.aspx | Initial lead list for some social support, community safety, animal welfare and volunteering categories. | Footer says © 2018 and page says more categories will be added; clearly not sufficient evidence of full city coverage. | Lead only; provider and schedule revalidation required. |
| NDA-GAUTENG-CSO-FOOTPRINT | https://www.nda.org.za/services/CSO-footprint/gauteng-province | Broad organisation discovery by sector; page displays 578 CSOs for City of Johannesburg. | No visible update timestamp found in inspected page; a footprint listing is not proof of current operation, location precision, availability, safety or directory reuse rights. | Lead only; check terms, export method and each record's currentness before use. |
| OPEN-REFERRAL-HSDS | https://docs.openreferral.org/en/3.1/hsds/overview.html | Interchange model for organisations, services, locations, schedules, eligibility-related fields, languages, accessibility, service areas and metadata. | A data format is not a source of local listings or a guarantee of record quality. Pin and test a specific schema/API version. | Recommended interchange target, subject to version pinning and mapping tests. |
| OPENSTREETMAP | https://www.openstreetmap.org/ | Geographic context, roads, mapped places/amenities, candidate proximity and navigation inputs. | A mapped feature does not confirm that a social service exists, is open, accessible, safe, or accepting participants. Attribution, licence and provider/rate requirements must be followed. | Spatial context only; never service-truth authority. |

**Source observation rule:** observed date refers to when this research inspected the source, not to when the source data was updated. The timestamp must not be copied into a listing's last_verified_at field.

### References checked

- City of Johannesburg, Libraries and Information Services: https://joburg.org.za/departments_/Communitydevelopment/Pages/Libraries%20And%20Information%20Services/libraries-and-information-services.aspx
- City of Johannesburg, Joburg Cares — Voluntary Organisations: https://joburg.org.za/services_/JoburgCares/Pages/voluntary-organisations.aspx
- National Development Agency, Gauteng CSO Footprint: https://www.nda.org.za/services/CSO-footprint/gauteng-province
- Open Referral HSDS overview: https://docs.openreferral.org/en/3.1/hsds/overview.html
- HSDS 3.1 announcement, including capacity and URL modelling: https://openreferral.org/upgrading-our-standards-introducing-version-3-1-of-the-human-service-data-specifications/
- City libraries and community services may also be discovered through the City region pages linked from the main library page; verify current content and terms before copying any records.

## Data rights and ethical admission gates

No record should be bulk-imported into a public application until the source's licence/terms and the intended use have been reviewed. This audit did not establish an open-data licence for the City pages or the NDA footprint page. Public readability does not automatically grant bulk-reuse rights.

For each prospective record, retain:
- source ID and canonical URI;
- source organisation and applicable licence/permission disposition;
- upstream record identifier where available;
- observation/retrieval timestamp, separate from the provider's last-confirmed timestamp;
- source type (provider confirmed, official directory, CSO registry, open geographic data, community report, or unverified user suggestion);
- distinct operation, contactability, schedule, accessibility, language, cost, eligibility and capacity states;
- verifier identity/role and verification method where permitted;
- correction/supersession lineage and reasons for marking stale;
- public location precision and safety/consent considerations.

Do not publish vulnerable clients, attendance, search history, private addresses or inferred need. A low-confidence or stale record should be clearly marked unknown/stale, not silently upgraded by AI-generated enrichment.

## Pilot design

### 0. Choose a bounded area and users

Select a small, contiguous pilot catchment only after source availability, transport patterns and community feedback are reviewed. Do not assume the City supports the project or that any organisation has agreed to participate. Invite libraries/community organisations and people who use services to review taxonomy, access details and what “welcoming to newcomers” means. Offer compensation where possible.

Include options across distinct purposes, rather than pretending all services are interchangeable:
- community connection and recurring activities;
- practical social support;
- public/community spaces with legitimate connection opportunities;
- human navigation/help-to-contact pathways, but only where a real operator accepts responsibility.

Do not fill a quota with invented records. An empty category is a coverage gap to report.

### 1. Use a two-layer directory

- **Layer A: candidate discovery records.** May come from public directories or community nominations; not displayed as verified live services until minimum fields and source permission pass admission.
- **Layer B: user-facing verified records.** Provider-confirmed or otherwise reviewed under an explicit policy; expose evidence class and freshness. Community confirmation is useful but should not be falsely attributed to a provider.

A directory-only record and a successfully completed connection are different states. Record each transition separately where the person explicitly opts in.

### 2. Suggested freshness policy for discussion

Proposed starting defaults, to review with local operators before any launch:
- One-off event/session: verify date, time and registration status from the organiser for that occurrence; mark stale once the event has passed or its details change.
- Recurring activity: request provider reconfirmation at least every 30 days during pilot; mark stale if overdue.
- Core organisation contact and ongoing-service status: reconfirm at least every 90 days during pilot.
- Urgent, capacity-limited or higher-risk services: use the provider's own currentness/response guarantee; do not display live availability unless there is an authorised live source.
- Any user report of closure, unsafe access, incorrect contact details or material harm: immediately downgrade the affected claim to disputed/unknown until reviewed.

These are proposed operational gates, not established universal service-directory standards. Report observed freshness distributions and set stricter or more permissive policies only after evidence and operator review.

## Evaluation plan

### Gate A — source readiness

Before an application shows records as verified:
- 100% of displayed records have reuse-permission disposition, source lineage, retrieval time, last verification state and a contact/correction route where available.
- 100% of fields such as cost, eligibility, language, accessibility, schedule, capacity and operational status use explicit known, unknown, stale, disputed, or not-applicable semantics; no missing value silently becomes a positive claim.
- Report per-source record coverage and missingness rather than one aggregate “data quality score.”
- Maintain a public list of known noncoverage and source limitations.

### Gate B — bounded live pilot

Do not prescribe a target count until the source audit determines what can be lawfully verified. An initial planning target of 30–50 verified services across multiple categories may be useful to exercise the workflow, but it must not be described as representative coverage of Johannesburg. If only 12 records meet the gate, report 12 and focus on improving verified access.

For each task, measure:
- whether the person found a plausible option meeting stated constraints;
- correct comprehension of price, eligibility, access requirements and schedule;
- stale or incorrect listing rate;
- time to a verified next action;
- no-match rate and which unmet constraints caused it;
- wanted first contact or completed connection only with explicit opt-in follow-up;
- whether the person prefers human navigation.

Benchmark against normal keyword search and available local channels; compare results on the same source set. Use blinded task grading where feasible. Establish baseline before precommitting improvements.

### Gate C — safety, accessibility and sovereignty

- Zero sensitive disclosure required for basic browsing.
- Zero sharing of referral information without consent to the exact payload and recipient.
- No attendance or search-history publication, no inferred loneliness/religion/developmental profiles, and no penalty for declining community or spiritual options.
- Include screen-reader, keyboard, low-bandwidth and language/access needs in task review; report performance by relevant conditions while protecting participant privacy.
- Human handoff must have a named real receiving organisation, role/ownership, current contact route, response expectations and escalation/safeguarding procedure. Without that, provide self-directed contact information only.

### Gate D — outcomes

Do not claim the navigator reduces loneliness simply because users click listings, attend an event, or say the interface is useful. Any claim about improved social connection needs a separately reviewed evaluation, suitable outcome measures, follow-up, and an appropriate comparison design. The WHO Commission on Social Connection treats measurement and evidence for interventions as major priorities; a new app should contribute to that evidence rather than assume impact.

## Execution checklist

1. Review reuse terms and export options for all candidate sources; record the result rather than assuming rights.
2. Ask which organisation can confirm local listings, especially hours, eligibility, schedule and accessibility.
3. Select one bounded catchment with community input.
4. Create a small synthetic HSDS conformance fixture before ingesting live records.
5. Create a manual verification workflow and change log.
6. Only display provider-confirmed or explicitly labelled candidate records under a reviewed publication policy.
7. Freeze a task benchmark against baseline search before usability testing.
8. Publish data-coverage gaps and failed referrals, not just success stories.

## Claim ceiling

This audit identifies plausible public sources and a verification strategy. It does not establish a current Johannesburg service inventory, reuse permissions, a City partnership, a service provider partnership, a working integration, or any reduction in loneliness. The next milestone is verified source readiness and a small testable pilot, not a citywide coverage claim.
