# City Belonging Navigator: service discovery as a real-world access workflow

**Status:** proposal / design artifact; no deployment, local inventory, provider partnership, or service coverage is claimed.  
**Tracking issue:** [CITY-CONNECTION-001](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/123)  
**Cross-project context:** Mycelix [#2878](https://github.com/Luminous-Dynamics/mycelix/issues/2878), [#5015](https://github.com/Luminous-Dynamics/mycelix/issues/5015), and research [PR #5097](https://github.com/Luminous-Dynamics/mycelix/pull/5097).

## 1. Product thesis

A map pin answers where. A useful navigator helps a person answer:

> Given what I want, what I can realistically access, and how much human help I want, what are my next few good options—and what will happen when I try one?

The system should make both human connection and practical help easier to discover, with minimal friction and no demand to disclose a sensitive personal story.

The initial audiences include people who have moved to a city, lost a social network, work irregular hours, are new to a faith or secular community, need peer support, want to volunteer, have accessibility needs, or simply want to find something worthwhile to do with other people. Do not infer any of these conditions about an individual.

## 2. What it should and should not be

It should be:
- a federated discovery interface over community service and activity data;
- useful in list-first, map-enhanced, low-bandwidth, accessible, and print-friendly modes;
- transparent about evidence source, freshness, cost, eligibility, and accessibility;
- capable of a user-approved introduction or referral when a real partner and workflow exist;
- open to faith-based, secular, cultural, recreational, practical, peer-support, and mutual-aid options on equal footing;
- designed with community organisations and people who actually use the services.

It must not:
- replace 211, local social prescribing, community navigators, statutory services, clinicians, or the organisations themselves;
- imply that a listing means the service is currently open, safe, accepting new people, or endorsed by government;
- make users create accounts or disclose loneliness, religion, trauma, health information, exact GPS, or an inferred developmental level before basic search;
- optimise engagement at the expense of successful connection or autonomy;
- create maps of people who searched for, attended, or need a service.

## 3. User journeys

### Journey A — “I want to meet people, but I don't know where to start”

1. Select “Find connection” or search in the user's own words.
2. Optionally narrow by general interests, approximate area, schedule, cost, language, accessibility, drop-in vs registration, or online vs in-person.
3. Review a small number of relevant options with plain-language descriptions, next session, travel/access details, cost, eligibility, newcomer information, and listing confidence.
4. Choose “What to expect on a first visit,” “Open directions,” “Call/message the organiser,” or “Help me make the first contact” where supported.
5. Optionally save/share the listing or ask for a warm introduction. Raw search history is not retained by default.

The system never diagnoses loneliness or decides that the user needs a spiritual group. The user selects the kind of connection.

### Journey B — “I need help with something practical”

1. Select “Find practical support.”
2. Browse relevant categories such as food, housing, training/work, transport, disability access, debt/financial advice, health/social support, and local mutual aid.
3. Show verified eligibility, needed documentation, schedule, location/virtual options, language, cost, and best next step.
4. Separate general information from urgent/emergency resources; emergency information must be jurisdiction-specific and checked by a qualified local steward.
5. Where an authorised, real referral partner exists, offer explicit consent for sharing a limited, previewed payload.

### Journey C — “I need a person, not another directory”

Offer a human-navigation route only when there is a real, currently responsible organisation, clear contact channel, response expectation, and safeguarding procedure. Show whether the route is staffed, when it is staffed, and what information it receives. Never make up a link worker or imply that an unstaffed form is monitored.

## 4. Standards and data sources

### Use Open Referral HSDS 3.1+ as the interoperability baseline

Open Referral's Human Services Data Specification defines objects and relationships for organisations, services, locations, service areas, schedules, languages, accessibility, contact details, and metadata. Version 3.1 also improves URL representation and supports modelling service capacity. Reuse the maintained standard rather than create a proprietary global service directory.

References:
- Specification overview: https://docs.openreferral.org/en/latest/
- HSDS schema reference: https://docs.openreferral.org/en/v3.3/hsds/schema_reference.html
- HSDS 3.1 changes: https://openreferral.org/upgrading-our-standards-introducing-version-3-1-of-the-human-service-data-specifications/

Build a local projection/adapter, not a new competing source of truth. Keep the exact upstream version and field mapping.

### Geographic layer

Reuse Sol Atlas' layer/timeline patterns and Mycelix #2878's OSM/civic-geographic boundary. OpenStreetMap can help with roads, points, building/place context and spatial proximity. OSM geometry or a tag is not authoritative proof of an organisation, active operation, safe entry, government approval, or a correct service category. Honor attribution, licensing, and provider usage/rate limits.

### Source registry

Before real data import, freeze a source registry containing:
- provider/dataset ID and description;
- source owner/contact and update path;
- licence/terms and permitted reuse;
- geographical scope and known noncoverage;
- retrieval method, retrieval time, upstream version/record ID;
- freshness policy for each class of fact;
- verification state and correction process;
- whether the source is open data, provider supplied, community reported, or human reviewed.

Do not presume a local source or API exists until it is found and its reuse conditions verified. A Johannesburg pilot is an option, not evidence of city partnership or complete local coverage.

## 5. Record provenance, freshness, and uncertainty

A user-facing record needs at least these separate concepts:

- **Provider identity:** who operates this service and who may confirm its information.
- **Service identity:** what activity/help is being offered and to whom.
- **Location:** physical, postal, virtual, or service-area location; precise public coordinates need a safety review where relevant.
- **Operational status:** active, paused, closed, unknown, or unverified, as supported by evidence.
- **Schedule:** advertised hours/next session and the source/last-checked time.
- **Eligibility and required documents:** only when sourced; absence means unknown.
- **Cost:** free, donation-based, sliding-scale or priced only where confirmed; no cost data must never be silently rendered as free.
- **Accessibility/language:** distinct, sourced claims; absence means unknown, not “accessible to all.”
- **Capacity:** advertised or confirmed availability with source and timestamp; do not infer present capacity from a general programme description.
- **Provenance and corrections:** upstream IDs/version, imported-at time, last verified-at time, correction history, verification status.
- **Next action:** call, email, website, visit, RSVP, or an opt-in referral, depending on what the service actually supports.

Never flatten all of this into a green/red “trust score.” Display the most important limitations directly, e.g. “Not recently confirmed—please call before travelling.”

## 6. Consent and privacy

- Anonymous search and map browsing by default; login is optional.
- Keep area, language, cost, schedule, accessibility and interest filters local where practical.
- Let the user enter a neighborhood or use approximate location. Exact GPS is optional and is not retained as history by default.
- Do not infer or store a profile of loneliness, religion, mental-health status, vulnerability or developmental stage.
- Do not track attendance, publish who visited a service, or use service choices as reputation signals.
- Before any referral/intro, show the exact information, recipient, purpose, retention, and revocation limits; require a final consent action.
- Withdrawal prevents future controlled processing/access but cannot guarantee recall of already exported, cached, or replicated copies. State that accurately.
- Avoid exposing small cohorts or home addresses in aggregate maps. Community demand data must be aggregated with a declared minimum cohort threshold and reviewed for re-identification risk.

## 7. Accessibility, trust and “first visit” friction

The discovery interface should not make map fluency, high-speed internet, or confidence with strangers a prerequisite.

Minimum presentation:
- list-first results; map is an optional spatial aid;
- keyboard/screen-reader accessible controls and useful textual directions;
- fast, low-bandwidth page; no forced heavy map download;
- print/download option with a generation date and stale-data warning;
- provider-confirmed language, accessibility, cost, hours, eligibility and contact details;
- “what happens when I arrive?”, entrance/meeting point, what to bring, whether walk-ins are welcome and what to do if full—only when provided/confirmed;
- neutral and culturally respectful language; no assumptions that faith, therapy, volunteering or group activities suit everyone;
- a feedback action for wrong, inaccessible, closed, unaffordable or no-longer-available records;
- a way for small organisations to correct and maintain listings without buying software or surrendering their data.

Do not label a group “beginner-safe” or “welcoming” unless the organiser has explicitly supplied the information or the description makes the claim clear. A community rating is not a substitute for safeguarding or provider verification.

## 8. Benchmarks

### Data quality (before launch)
- 100% of displayable listings carry a source reference, reuse-permission status, retrieval timestamp, provenance class and explicit freshness state.
- Missing eligibility, cost, accessibility, language or capacity is always shown as unknown.
- Frozen fixtures cover duplicate sources, stale schedules, closed organisations, invalid contact details, ambiguous coordinates, conflicting provider reports and incorrect categories.
- Report the fraction of records independently confirmed by providers for each critical field; never present the directory's completeness as better than the evidence supports.

### Search and task success
Construct a preregistered or otherwise versioned task set with varied needs, languages, schedules, budgets and accessibility constraints. Compare:
1. ordinary keyword search over the same source corpus;
2. filtered directory search;
3. map-enhanced discovery;
4. human navigation, where a real service is available and consented.

Measure:
- successful identification of at least one suitable option under the stated constraints;
- precision of the first five results, with “suitable” judged against an independently specified task rubric;
- user comprehension of cost, eligibility, accessibility, schedule and uncertainty;
- correct next-action selection and time-to-verified-contact;
- stale/false-positive listing rate and proportion of tasks with no suitable option found;
- abandonment and whether the person preferred human assistance.

Do not set arbitrary efficacy thresholds before measuring source coverage and task difficulty. Freeze baseline results first and precommit a meaningful improvement target.

### Real access, belonging and equity
- With explicit opt-in follow-up, measure wanted contact, successful first attendance/connection, and whether the user felt the opportunity matched what they asked for.
- A click, call attempt, RSVP, attendance, or short-term satisfaction are different outcomes; report them separately.
- Where safe and appropriately reviewed, use validated social-connection or wellbeing measures and follow-up. Do not equate these with spiritual growth or infer causality without a suitable study design.
- Report subgroup task success and no-match rates by relevant access conditions without publishing identifiable personal profiles.
- Include people with disabilities, people using low bandwidth, community groups, local navigators, faith/secular participants, and residents who opt not to disclose personal stories in usability/review.

### Autonomy and privacy
- Zero sensitive disclosure required before basic search.
- Zero sharing of referral payloads without exact-payload consent.
- Zero ranking of users by loneliness, religion, spirituality, or perceived developmental maturity.
- Publish privacy/capability limitations and test against search-history leakage and small-cohort reidentification.
- Measure whether users can decline, switch framing, browse anonymously, or leave without penalty.

## 9. Integration boundaries

| Component | Owns | Must not claim |
|---|---|---|
| Sol Atlas | Spatial/list discovery, accessibility, data-source display, route/context visualisation. | That geometry proves service legitimacy or availability. |
| Mycelix | Source/provenance contracts, contributor/provider governance, consented handoff records, correction lineage, community stewardship. | That reputation proves truth or that replicated data can always be erased. |
| Symthaea | Optional natural-language search help, disambiguation, user-controlled preference clarification, alternative explanations. | Diagnosis, faith inference, spiritual ranking or unverified service claims. |
| Symtropy | Synthetic scenarios for resource-access gaps, coverage trade-offs and travel/capacity questions where assumptions are explicit. | Human-efficacy proof from simulation. |
| Human/community operators | Confirm listings, maintain relationships, deliver services/referrals, safeguarding and care. | An unstaffed digital pathway should never be represented as a staffed service. |

## 10. Rollout

1. **Source audit:** identify what local data is legitimately available, its coverage, licences, freshness and who can update it. If there is no trustworthy source, say so and start community/provider onboarding rather than fabricating listings.
2. **Synthetic fixture:** build data adapter tests before any live deployment.
3. **Discovery-only slice:** list-first search, filters, source/freshness status, and map overlay. No personal profile or referral tracking.
4. **Community maintenance:** provider updates and corrections with provenance.
5. **Human handoff:** only after confirming a real receiving organisation, consent semantics, staffing, response targets and safeguarding route.
6. **Pilot evaluation:** preregister tasks and outcome measures, review with community participants, report both coverage gaps and failures.
7. **Replication:** port the architecture to other cities by replacing source registries and language/category profiles, not by copying assumptions about local institutions.

The differentiator should be **the path from discoverable option to a realistic, wanted human connection**, not the number of map pins. This is a research proposal until a bounded data inventory, implementation, and independent evaluation exist.
