# Transport Projection Contract V1

Status: draft, renderer-neutral projection contract for SOL-ATLAS-009 / issue #106. The implementation is an offline reference slice. It is not a live freight map, operational integration, transport optimizer, shipment authority, or proof of real-world performance.

## Goal and module boundary

Sol Atlas should display versioned outputs from Symtropy and evidence/coordination systems in Mycelix; it must not become a second canonical logistics database or a competing route optimizer. The contract in `sol-atlas-core/src/transport_projection.rs` is a read-only, strongly typed projection boundary.

The schema deliberately separates five domains:

1. **Physical logistics** — facilities, movement segments, explicit intermodal transfer edges and sourced capacity claims.
2. **Shipment milestones** — planned, forecast, proposed reservation, accepted reservation, dispatched, arrival-reported, delivery-confirmed and cancelled states.
3. **Communication contacts** — endpoints and time-bounded communication opportunities.
4. **Evidence relations** — supports, conflicts, supersedes and derived-from relationships between evidence records.
5. **Infrastructure dependencies** — assets and explicit prerequisite relationships such as handling equipment depending on power.

These collections are separate Rust types and separate serialized fields. A communication contact cannot satisfy a physical-route predicate. Delivery of a message does not establish movement of cargo. An arrival-reported milestone is not automatically a delivery-confirmed milestone.

## Provenance and qualification are separate

Each evidence record separates:

- evidence kind: observed, curated, derived or scenario;
- projection qualification: accepted for projection, scenario-only, unknown, stale, conflicted, unavailable or unqualified;
- source locator, source profile and source version;
- optional source-record SHA-256 commitment;
- source event time, observation time and repository-recorded time;
- optional validity interval and explicit time-profile ID.

These attributes answer different questions. A schema-valid record can still be stale, conflicted, unqualified or only a scenario. Accepted-for-projection is not issuer authentication and does not prove a physical-world claim. The validator preserves conflicts and never resolves them by choosing the newest timestamp or a model confidence score.

Optional hashes are checked for 64-hex-character shape only; this module does not fetch source bytes or recompute digests. The normalized JSON function sorts arrays by stable IDs for repeatable fixture comparison. It is not a cross-language canonical-JSON specification and does not compute a cryptographic digest.

## Spatial, temporal and capacity semantics

Coordinates use integer microdegrees and are range-checked. Missing coordinates remain missing; consumers must not invent geolocation for a schematic or private facility.

Every timestamp in a projection uses an explicit projection time profile. Event time, observation time, forecast time, and repository recording time are not interchangeable. If a source uses another timezone or clock profile, an adapter must perform and document the conversion before producing this projection; the core validator does not silently convert time.

Capacity is represented by a quantity unit and a separate basis: concurrent use, per hour, per shift, per day, total or unspecified. Each segment has a dedicated `capacity_evidence_ids` list distinct from general topology/evidence references. A numeric capacity requires at least one explicit capacity-evidence record. Evidence marked conflicted, stale, unknown, unavailable or unqualified cannot support a numeric capacity; scenario-only evidence may support only a proposed/assumed-for-scenario segment. Thus contradictory claims can remain attached to a segment whose selected capacity is null, but cannot silently justify a number. A numeric capacity on any non-scenario/reported segment must declare a non-`Unspecified` basis; otherwise the projection is invalid. Scenario-only values may remain unspecified if the scenario label is preserved. A rate/quota cannot be normalized into concurrent capacity without a reviewed conversion. A rated capacity is not the live availability of a road, track, terminal, lock, crane or vessel slot.

A route leg is not declared feasible merely because it is present in a projection. Infrastructure gauge, clearance, axle loads, track possessions, operating rules, waterway draft, cargo compatibility, legal limits, energy, timetable and transfer windows belong in the qualified planner and source evidence. An optimizer result must retain its exact scenario/evidence identity when shown in Sol Atlas.

## Validation implemented

The Rust validator checks:

- schema version and non-empty projection/source identities;
- duplicate or blank IDs within collections and ambiguous IDs across projected-object collections;
- every populated evidence `claim_about_id` resolves to an existing projected object;
- projected/reported numeric capacity has an explicit non-`Unspecified` basis, while scenario-only assumptions may remain unspecified;
- optional SHA-256 field shape;
- evidence validity-interval coherence and time-profile compatibility;
- geographic coordinate bounds;
- all evidence references and physical graph endpoints;
- capacity quantities and distinct capacity-basis semantics;
- an explicit evidence list for every numeric capacity and rejection of conflicted/stale/unqualified capacity promotion;
- shipment milestone references and milestone/time-meaning consistency;
- communication endpoints, non-empty contact windows and time-profile compatibility;
- evidence-relation endpoints, without resolving conflicts;
- infrastructure dependency endpoints and direct self-dependencies.

The synthetic fixture has no real coordinates or live source records. It intentionally includes two conflicting rail-capacity claims; the rail leg keeps both evidence references and has no selected numeric capacity. The fixture demonstrates how disagreement should be displayed rather than silently averaged or overwritten.

## South African data strategy

Treat published sources as acquisition leads until exact downloads, access conditions, licensing, schemas, coverage, units and hashes are reviewed.

- The [National Freight Databank](https://freight.transport.gov.za/) describes multimodal freight information across roads, rail, pipelines and facilities such as ports, border posts, weighbridges and intermodal terminals. It is a discovery target. This project has not established that a bulk API is available or that every dataset can be republished.
- A much more actionable rail source is Transnet Rail Infrastructure Manager's [current Network Statement v4 page](https://www.transnet.net/TRIM-Networkstatement), announced 18 September 2026. It links [Annexure 33 — Network Register](https://www.transnet.net/getFile.ashx?id=21073), [Annexure 33a — Point to Point Running Times (XLSX)](https://www.transnet.net/getFile.ashx?id=22633), and [Annexure 33b — Capacity Statement (XLSM)](https://www.transnet.net/getFile.ashx?id=21146). The linked Network Register is dated 26 June 2026, says it applies to the 2026/27 timetable, and describes section-level theoretical, practical, lost, operationally available, allocated and spare train-path capacity at daily and weekly resolutions. These are stronger candidates for a grounded schedule replay than aggregate national freight totals. The underlying spreadsheet bytes and hashes have not been frozen by this project, the schema/units have not been reviewed, and rights remain `ReviewRequired`; no capacity values are promoted from these discovery links.
- The Network Register itself warns that the rail network can be unreliable or unavailable after theft/vandalism. It identifies Leeuhof–Warrenton (via Klerksdorp) and Thabazimbi–Rustenburg as critical sections; it describes rerouting via Kroonstad and Bloemfontein for the former, while it says the latter has no alternative route and requires diesel propulsion. Use these facts to define *qualitative* disruption/re-route test cases, not to invent incident probabilities or numerical delay estimates. The same register explicitly says its scope is infrastructure owned and operated by TRIM and excludes PRASA and other infrastructure managers/concessionaires. Therefore this source cannot be treated as a complete national rail graph, even if a GIS layer uses national-coverage language. The register's displayed ten-route list also repeats the Musina–Pyramid South/Hoedspruit label in items 9 and 10; verify the linked workbook before asserting a distinct-route count.
- The 2026 [Draft National Rail Master Plan consultation](https://www.gov.za/news/speeches/minister-barbara-creecy-launch-gauteng-stakeholder-engagement-draft-national-rail) describes a planned Rail Planning Component for freight/passenger flows, capacity, asset condition and rolling stock. The public-comment window closed on 22 July 2026, and the stakeholder platform still labels the document as a draft. Do not infer final approval, a live data feed or a project access agreement from the consultation page.
- The Western Cape's [intermodal freight-network proposal](https://www.westerncape.gov.za/mobility/intermodalism-building-more-connected-freight-network) reports a modelled opportunity of approximately 10.4 million tonnes of road freight that could potentially use intermodal rail terminals. This is a candidate for scenario selection, not guaranteed modal shift or verified current terminal capacity.
- The South African government's [geography overview](https://www.gov.za/about-sa/geography-and-climate) states that none of the country's rivers are commercially navigable. Inland-waterway and canal node/mode types remain valid for other countries or future engineered-waterway proposals, but the South African pilot should focus first on road–rail–port corridors, terminal transfers and water-system resilience rather than inventing existing commercial river freight.
- For regional topology, the Western Cape GIS Spatial Data Warehouse exposes a [proclaimed provincial road-network layer](https://gis.westerncape.gov.za/server2/rest/services/SpatialDataWarehouse/Transportation/MapServer/1) with road number, road type and district attributes, and an [NGI railway-lines layer](https://gis.westerncape.gov.za/server2/rest/services/SpatialDataWarehouse/Transportation/MapServer/0) with source/capture/vintage metadata. Both report projected WKID 32734 (WGS 84 / UTM Zone 34S), so adapters must record a real CRS transform instead of misreading metre coordinates as latitude/longitude. These are geometry candidates only: the road layer is regional, the rail service's own description is ambiguous about Western Cape versus national extent, and neither layer establishes operational closures, freight eligibility, gauge or capacity. The service/item metadata has no populated licenseInfo; access and redistribution rights remain unreviewed.

For every prospective input, record the exact publisher, source URI or acquisition record, access/licence decision, source version, geographic and temporal coverage, source time profile, retrieval time, byte count, SHA-256 of exact bytes, parsing/transformation version, missingness and CRS. States such as not acquired, access review required, acquired-unreviewed, qualified for scenario and rejected must remain distinct. Aggregate national throughput must never be presented as route-level usable capacity.

## Cross-project adapter boundary

This projection is deliberately a **view contract**, not a replacement for the other projects' canonical semantics:

| Producer / authority | What Sol Atlas may consume | What Sol Atlas must not infer |
|---|---|---|
| Symtropy multimodal planner ([PR #1602](https://github.com/Luminous-Dynamics/symtropy/pull/1602), [schedule-aware capacity PR #1628](https://github.com/Luminous-Dynamics/symtropy/pull/1628)) | Versioned scenario/route outputs, explicitly qualified capacity evidence, objective vectors and rejection reasons | A route in a model is physically feasible; a plan is a reservation; a capacity budget is live availability |
| Mycelix offline logistics twin ([PR #4902](https://github.com/Luminous-Dynamics/mycelix/pull/4902)) | Deterministic replay summaries and independent-verifier findings with frozen manifest lineage | A valid simulation receipt proves a real delivery or authorizes inventory mutation |
| Mycelix logistics standards registry ([PR #5005](https://github.com/Luminous-Dynamics/mycelix/pull/5005)) | Named, versioned source/profile mappings and explicit lossy-mapping notes | Different standards' events can be flattened into one interchangeable event type |
| Geospatial source adapters | Read-only feature snapshots with source schema/profile and coordinate reference commitments | Public landing-page visibility implies an available API, free redistribution or operational freshness |

The adapter should map sources such as GS1 EPCIS while preserving event time separately from repository `recordTime`: GS1 specifies that `recordTime` is repository bookkeeping, not the real-world event time. <https://ref.gs1.org/standards/epcis/2.0.0/> For geographic feature services, prefer a reviewed OGC API – Features profile when the publisher exposes one; the standard is modular, and an endpoint is conformant only if its implemented parts and data contract are established. <https://www.ogc.org/standards/ogcapi-features/>

Every adapter mapping should identify the source standard/release, source schema, internal target field, normalization or lossy transform, unit conversion, time/CRS transform and claim ceiling. Unknown source fields should remain inspectable in the adapter's provenance output; they must not be silently dropped when they could affect routing, capacity, time or custody.

## Source acquisition gate

The companion module `sol-atlas-core/src/transport_source_manifest.rs` provides a typed `TransportSourceManifestV1`. A source record keeps `NotAcquired`, `AccessReviewRequired`, `AcquiredUnreviewed`, `QualifiedForScenario` and `Rejected` states distinct from rights decisions. A source can move to `QualifiedForScenario` only when the manifest has an exact acquired-byte receipt (retrieval timestamp, byte count and SHA-256 shape), a permitted rights state and decision reference, schema and geographic/time profiles as appropriate, coverage interval for non-document data, parser profile, a qualification-review reference and explicit missingness/limitations.

The South African register currently contains eight discovery records:
- National Freight Databank landing page — `AccessReviewRequired`; no dataset bytes, API contract or bulk-download permission are asserted.
- Draft National Rail Master Plan consultation — `NotAcquired` for this manifest; the public document page is not treated as a live freight-data feed.
- Western Cape proposed intermodal terminal network — `NotAcquired`; candidate sites and the reported modelled opportunity are not treated as verified current capacity.
- Transnet Annexure 33 Network Register (June 2026 v0.3) — `AccessReviewRequired`; relevant to the 2026/27 timetable, but its exact bytes are not frozen by this project and its displayed route list needs a duplicate-label check against the workbook.
- Transnet Annexure 33a point-to-point running times — `AccessReviewRequired`; an XLSX download is linked, but its exact bytes, schema, units and rights have not been reviewed here.
- Transnet Annexure 33b Capacity Statement — `AccessReviewRequired`; an XLSM download is linked, but no section-level capacity value is acquired or treated as live availability, and its exact bytes/schema/rights remain unreviewed.
- Western Cape proclaimed Provincial Road Network — `AccessReviewRequired`; queryable polyline metadata has been inspected, but no feature response is frozen, no hash/schema receipt exists and `licenseInfo` is blank. Do not treat it as live traffic, current closures or capacity.
- Western Cape NGI railway lines — `AccessReviewRequired`; polyline/capture metadata has been inspected, but geographic coverage is ambiguous between a Western Cape layer and a dataset described as national. Do not promote it to a national freight graph without extent/completeness review.

These are data-readiness records, not claims that the web pages themselves were never read. The distinction is between researching a source page and acquiring, rights-reviewing, freezing and qualifying the underlying dataset used in a reproducible scenario. The validator checks declared metadata shape; it cannot establish that a legal review is correct or that a dataset is scientifically valid.

## Staging

- **H0 — contract:** maintain the renderer-neutral schema, validator, synthetic fixture and deterministic-order tests.
- **H1 — read-only explorer:** add layer toggles, selection and node/edge details in the existing Sol Atlas UI; display observed, curated, derived, scenario, unknown, stale and conflicted states distinctly.
- **H2 — verified projection adapters:** consume qualified outputs from Symtropy and Mycelix only after source rights and mapping semantics are reviewed. Keep physical movement and communication-contact graphs separate.
- **H3 — replay and independent verifier:** after rights-reviewed Annexure 33a/33b bytes are acquired, replay compatible section-level train paths against the road/terminal graph. Include missed handoffs, stale capacity and contradictory evidence, plus two qualitative source-guided disruption cases: (a) Leeuhof–Warrenton closure/reliability loss with the documented longer reroute via Kroonstad/Bloemfontein; (b) Thabazimbi–Rustenburg disruption with no alternate route and a diesel-propulsion constraint. A separate validation path must check topology, statuses, capacity fields, times/units and conflicts. Until the actual workbook is frozen and reviewed, these are test dimensions only, not quantitative results.
- **H4 — controlled pilot:** compare recommendations in shadow mode with historical and operator-reviewed outcomes before any operational dependency. Any reservation, dispatch or safety-critical act remains inside the responsible operator's authoritative system.

Passing the H0 structural validator establishes only that a frozen projection conforms to this declared contract. It does not establish transport performance, capacity, safety, scientific calibration, affordability or operational readiness.
