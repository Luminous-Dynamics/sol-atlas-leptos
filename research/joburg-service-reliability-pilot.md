# Johannesburg Essential-Service Reliability Pilot

**Research snapshot:** 2026-10-13  
**Status:** Design/research only. No live integration, field validation, or service partnership is claimed.  
**Proposed first domain:** water interruptions and last-mile response coordination.  
**Primary owner candidate:** Sol Atlas for evidence presentation; integrations remain separately owned by their canonical projects.

## Executive finding

Johannesburg is a credible test location for a service-reliability workflow because water-supply interruptions are publicly reported, have a documented infrastructure dependency on electricity, and now have an independent, structured historical record. The opportunity is not to make another generic outage map. Watersearch already offers a suburb-by-suburb explorer and a free, read-only API for the status Johannesburg Water reports about reservoirs and towers.

The proposed missing layer is the last mile between **infrastructure-level status → locally observed impact → verified help/resource availability → accountable response → evidence of resolution**. Each arrow is a separate claim that must be supported by its own evidence. An infrastructure status is not a measurement at a household tap; a resident report is not automatically corroborated; a map marker is not proof that a water point has water; a queued request is not a completed response.

The first deliverable should be a read-only, replayable research/pilot slice. Do not dispatch tankers, publish unverified water-point availability, make health-risk classifications, or claim reduced outage durations.

## Why water first, rather than the other urgent domains

| Domain | Evidence of need | Software contribution | Main constraint | Pilot priority |
|---|---|---|---|---|
| Water supply | Johannesburg Water publishes repeated supply notices and status boards; government convened water and electricity stakeholders on 8 September 2026 to address interruptions arising from power trips. | Preserve a source-attributed history; relate supply assets to their published suburb links; add a separate, carefully qualified layer for locally observed impact and verified assistance. | Reservoir/tower reports are not tap readings; repairs, pumping, storage, and physical distribution require operators and capital. | **1 — best sourceable first pilot** |
| Waste collection / recycling | Reporting on 9 October 2026 described Pikitup contractor-payment pressures, fleet maintenance problems, and collection disruption. | Match confirmed missed collections with available workers, vehicles, and recyclers; measure task completion and material recovery. | A coordination interface cannot pay contractors, repair trucks, or remedy cash-flow failures. The current research found no comparably ready public historical API. | 2 — good community-cooperative pilot |
| Clinic administration | Public-clinic queueing and records workflows are plausible high-burden areas. | Queue measurement, appointment logistics, referral coordination, and administrative bottleneck detection using minimal data. | Patient data, clinical safety, and public-health approvals raise the bar; do not start with patient records or automated clinical triage. | 3 — pursue with a health partner |
| Local energy resilience | The city’s water/electricity dependency is visible in the September 2026 joint statement. | Compare bounded backup-power scenarios and record whether named critical loads are served in the model. | An aggregate energy balance does not prove that a specific pump, clinic, communications site, or other critical load received power. | Parallel technical prerequisite, not the first field pilot |

A global deployment pattern could later reuse the same service-reliability contract for electricity, sanitation, health logistics, emergency response, waste, and food distribution. That should be a family of domain adapters—not one universal metric that conflates unrelated types of evidence.

## Johannesburg evidence checked

### Water data already available

Watersearch (watersearch.co.za) says it has read Johannesburg Water’s status boards and notices since March 2025. Its API exposes suburb-to-supply links, daily history, reported levels, notices, outage intervals, supply systems, regions, and dated releases. Each response declares a data release and caveats. No API key is required, and the site states CC BY 4.0 for its compilation and readings, with attribution to Johannesburg Water as the underlying source. The API page still describes that licence as provisional until the site is public; even though the site is publicly accessible, the exact attribution/redistribution terms should be checked before we mirror bulk data or redistribute stored source snapshots.

The latest indexed release observed during this research was **2026-10-09-85db5414**, with a source-board timestamp of 9 October 2026 at 16:55 SAST. At that snapshot the explorer reported:

- 77 of 93 listed reservoirs, towers, and pump stations as normal;
- Brixton 1 Tower as short of water;
- Hursthill 1 Reservoir, Hursthill 2 Reservoir, and Robertville Pump Station under warning;
- six assets with weak pressure or a low level, and six intentionally closed overnight in the preceding 48-hour reporting window;
- Hursthill 1 and Hursthill 2 Reservoir each reported short of water on 106 of 197 days in the year-to-date window used by that release.

These are **historical reports about named supply assets**, not live observations at taps. They are an example of why the interface must preserve the reporting timestamp, release identifier, level semantics, gaps, and caveat rather than render a simple “water available / unavailable” claim for every household.

### Evidence of systemic coupling

On 10 September 2026, the South African Government published a joint statement saying national/provincial authorities, the City of Johannesburg, Rand Water, Johannesburg Water, City Power, Eskom representatives, and others had met to discuss reliable electricity for water-supply systems and to reduce disruptions arising from intermittent power trips. This is a strong reason to connect water-impact reporting to a separately evidenced energy-resilience view, but it is not evidence that any single outage was caused by a specific power event.

Johannesburg Water’s official customer-notice page and dated notice posts are the authoritative route for current source notices and escalation. The public fault-reporting channel should remain the official route for water incidents. A Sol Atlas view should link back to the exact source notice rather than ask residents to treat a third-party dashboard as the authority.

### Other candidate: waste

On 9 October 2026, EWN reported Pikitup financial and fleet difficulties, including amounts owed to contractors, maintenance pressures, and disruptions in refuse collection. This is a credible next opportunity, but it is important not to imply that better route planning alone resolves an underlying financing or fleet-capacity shortfall. Waste becomes the stronger first field pilot if a recycling cooperative or community organization can provide a bounded service area, vehicle/work availability, and permission to collect outcome data.

## Proposed bounded pilot

### Phase 0 — partner and source review

1. Read the Watersearch API and method documentation and contact its author before building a competing public product. Ask about preferred attribution, release pinning, source corrections, bulk redistribution, and whether a small service-coordination extension would be useful.
2. Confirm that the intended reuse is within the source’s stated terms. Keep source facts, Watersearch’s derived/compiled data, and our own derived metrics distinct.
3. Select a pilot suburb only after reviewing asset-to-suburb links, source coverage, a relevant local partner, and the existence of a trustworthy way to verify the status of any proposed water collection point. The Hursthill assets are candidates for retrospective study, not a preselected deployment site.
4. Define a community data-governance and safeguarding protocol before accepting reports about households or vulnerable facilities.

**Exit condition:** a reviewed source-use decision and one participating local organization, or remain an offline-only prototype.

### Phase 1 — deterministic historical replay

Build an adapter that consumes a pinned Watersearch release or a frozen, attributed fixture. Keep network access out of ordinary CI; live retrieval should be a separate explicitly invoked source-acquisition step.

Display only what the source supports:
- named supply asset and the suburb links published by Johannesburg Water;
- exact reported status and original source link;
- reporting time, date/release identifier, and retrieval time as separate fields;
- gaps and source staleness explicitly;
- a history view that does not convert missing boards into normal service;
- clear separation between reported source facts, curated associations, derived summaries, and hypothetical scenarios.

This slice should be useful for retrospective analysis even if the external source is temporarily unavailable. A future live adapter should preserve the last accepted snapshot on refresh failure while visibly marking that snapshot as stale or unavailable.

### Phase 2 — local observations, deliberately separate from infrastructure reports

Add an optional, consent-aware observation pathway only after a local partner approves the protocol. A report should record the observation time, coarse location, observation category (for example: tap dry, low pressure, suspected leak, or a personally observed water point), optional expiry, provenance, and validation state.

Use explicit states such as **unverified**, **corroborated**, **contradicted**, **expired**, and **withdrawn**. These states describe the evidence for a report; they do not measure the trustworthiness or worth of the person submitting it. Multiple reports from one network or household must not automatically count as independent corroboration.

Keep precise household locations and personal identifiers private. Public views should use an appropriately coarse geography; retention, access, correction, deletion, and safeguarding rules need to be agreed before collecting data. Reports about water quality must never be displayed as a potability/safety certification.

### Phase 3 — verified resource and response coordination

If a participating organization supplies a verified registry, add water collection points or delivery offers with a clear owner, last-checked time, opening window, stock/capacity if known, accessibility details, eligibility/conditions, and a contact route. Do not invent public water points or infer availability from a map marker.

Represent a request, an accepted task, a dispatched resource, and a completed service as different states. Completion should carry an evidence receipt and a named verification method; where only a participant’s unverified statement exists, label it “reported complete,” not “verified complete.” Actual dispatch remains under the authority of the participating operator.

## Cross-project architecture

### Sol Atlas — geographic and evidence interface

Sol Atlas should own the rendering, source/provenance presentation, and navigation from supply asset → linked suburbs → local observations → verified resources. The semantic model must remain renderer-independent. Map visibility must never imply resource availability, qualification, safety, or authorization.

The current public main branch inspected for this work contains static global data layers but no Johannesburg/Watersearch adapter. Some live-feed and dataset-health functionality exists in open pull requests, not yet in the inspected main branch; integration must be based on the exact branch/commit actually merged, not on PR descriptions alone.

Relevant existing work:
- [SOL-ATLAS-001 / issue #17 — visualization for the Bootstrap Atlas](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/17)
- [SOL-ATLAS-002 / issue #19 — deterministic capability/dependency slice](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/19)
- [Issue #84 — independently model and verify critical-load service](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/84)
- [Earth Systems Intelligence work / PR #101](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/101), a related broad domain catalogue, currently a separate proposal.

### Symthaea — advisory analysis, only after a valid baseline exists

Potential later roles: duplicate candidate detection, event clustering, short-horizon workload estimates, and comparison of response options with uncertainty. Each output must cite its inputs and expose uncertainty and abstention. Start by comparing analysis against a deterministic baseline and a reviewed test set. No model output should establish whether a home has water, declare cause, prioritize a person based on a reputation score, or dispatch a resource.

### Mycelix — consent, requests, and auditable coordination, later

Mycelix Commons already has documented water-flow, water-purity, water-capture, stewardship, mutual-aid request/resource, food distribution, and transport domains. However, the public README labels Commons pre-alpha and says its UI is not deployed. Treat these as candidate integration points, not a live municipal service. First specify minimal signed/attributable records and consent boundaries; introduce a Holochain integration only after the required multi-agent tests, identity/authorization review, operational support, and recovery behavior are demonstrated.

Public project maturity reference: [Mycelix README and maturity matrix](https://github.com/Luminous-Dynamics/mycelix).

### Symtropy — offline scenarios, not operational dispatch

A later model could compare bounded delivery/collection plans using explicit road-network, vehicle-capacity, opening-time, travel-time, and demand inputs. The simulation must state what is synthetic, observed, and assumed. It cannot claim a route is feasible because a visualization looks plausible. Use Symtropy only once the modeled mechanics match the actual question; it is not a hydraulic network model and does not validate water quality.

### Cross-cutting dependency: critical-load verification

Water infrastructure and clinics may depend on electricity. The existing Sol Atlas issue #84 correctly identifies that a model with only aggregate load cannot prove delivery to a declared critical load. Before using an energy result to support a real water/health service plan, the model must separately record and independently verify critical-load demand served/unserved, priority rules, and adverse outages. Energy availability must not be inferred from overall generation balance or low aggregate unserved energy.

## Data and evidence contract

Every observation or task should preserve, at minimum:

- stable record identity and schema version;
- source owner, exact source URL, original source timestamp, fetch timestamp, source release, and the precise scope of the assertion;
- source/fixture digest only when the corresponding exact bytes were actually acquired and hashed;
- status semantics, units, time window, location granularity, provenance class, and evidence/verification state;
- explicit missingness and refresh health;
- transformations, assumptions, and claim ceiling for derived outputs;
- any human correction, contradiction, supersession, or withdrawal without silently erasing the history.

Required fail-closed rules:

1. no_report is not normal service; stale data is not live data; an empty valid dataset is not a failed/malformed feed.
2. A source-reported reservoir state is never copied into a claim about a household tap.
3. A reported outage is not automatically linked causally to a grid incident.
4. A resource’s geographic presence is not evidence it is operating, stocked, accessible, safe, or authorized.
5. A submitted/queued/accepted task is not a completed or successful task.
6. Conflicting sources remain visible and unresolved rather than being silently averaged or overwritten.
7. Model-derived priority is advisory and contestable; no automated denial of essential services or opaque person-level ranking.
8. Privacy-sensitive reports are not written into a public DHT by default. Public records should contain only data approved for disclosure, with personal data handled under an explicit lawful basis and retention policy.

## Evaluation plan

Begin with a 30-day retrospective baseline over a reviewed, frozen source release and then use a small consented pilot area. Establish thresholds from observed baseline and operator needs rather than inventing success targets before the data is known.

**Data/integrity measures**
- share of records parsed with all mandatory source, time, release, and caveat fields intact;
- number of malformed/partial/stale refresh scenarios caught by deterministic tests;
- proof that missing-board days remain in the correct denominator and are never treated as normal;
- corrections and conflicts preserved with links to the original source.

**Service outcome measures**
- time from a source update to a visible, attributed update;
- time from a local report to acknowledgement and corroboration;
- time needed to find a currently verified resource, where the pilot actually has such resources;
- accepted requests, time to a response, verified completed tasks, and resolution rate;
- outcomes by neighborhood and channel, including gaps for people without reliable internet access;
- operating cost per verified resolution and the number of reports requiring human adjudication.

Do not claim the pilot reduces physical outage duration unless an operator relationship and suitable comparison design actually measure outage duration. Do not claim lives saved, health improvements, or causal effects from dashboard engagement metrics.

## Decision gates

- **Gate A — source terms and attribution:** reviewed; otherwise no mirroring/republication of source records.
- **Gate B — reproducibility:** frozen release/fixture, exact digest, schema, tests, and release/caveat display.
- **Gate C — safe local reporting:** partner-approved consent, location precision, moderation, correction, expiry, and escalation rules.
- **Gate D — resource registry:** verified owner and a method to confirm availability; otherwise show no live resource-availability layer.
- **Gate E — service coordination:** operator authorization, clear incident/task ownership, human fallback, and verifiable receipts.
- **Gate F — simulation:** exact inputs, explicit assumptions, adverse cases, independent verification, and critical-load service proof.
- **Gate G — outcome claim:** defined baseline and comparison method; no claim stronger than the measured outcome.

Failure to satisfy a gate leaves the relevant feature in research/demo status rather than allowing another subsystem to upgrade its claim.

## Open questions before any public deployment

1. Will Watersearch’s author approve the intended integration and confirm the preferred attribution and release-citation format?
2. Which neighborhood has a willing local organization and a trustworthy way to confirm water-point or delivery availability?
3. Can the pilot operate without household identity, precise address, health data, or an account requirement?
4. Who is responsible for corrections, moderation, escalation, and false reports when a volunteer observation conflicts with an official board?
5. What operational data can a participating provider lawfully share, at what refresh cadence, and with what reliability?
6. Who funds actual delivery/repair capacity, rather than just software and coordination?

## Sources

- Watersearch explorer: https://watersearch.co.za/
- Watersearch API contract and stated licence: https://watersearch.co.za/api
- Watersearch method, limitations, and provenance: https://watersearch.co.za/method
- Watersearch OpenAPI document: https://watersearch.co.za/api/v1/openapi.json
- Johannesburg Water official customer notices: https://www.johannesburgwater.co.za/media/media-statement/customer-notices/
- Johannesburg Water customer notice, 11 October 2026: https://www.johannesburgwater.co.za/blog/2026/10/11/customer-notice-11-october-2026/
- Government of South Africa, 10 September 2026 joint statement on water-supply disruptions resulting from power trips: https://www.gov.za/news/media-statements/government-measures-mitigate-water-supply-disruptions-resulting-power-trips
- EWN, 9 October 2026, Pikitup financial and fleet troubles: https://www.ewn.co.za/2026/10/09/pikitup-financial-and-fleet-troubles-hit-joburg-waste-delivery-ahead-of-elections
- WHO, global WASH report, 5 October 2026: https://www.who.int/publications/i/item/B09880

## Research limits

This artifact records source-document and repository inspection on 13 October 2026. The Watersearch API documentation and OpenAPI description were inspected through the browser, but no source dataset was downloaded, hashed, parsed end-to-end, or independently audited in this research. No data owner or community organization was contacted. No field service, outage, causal relationship, integration, local pilot, or reduction in burden has been verified. The proposed implementation remains design-only until the gates above are passed.
