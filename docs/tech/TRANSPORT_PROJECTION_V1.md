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

Capacity is represented by a quantity unit and a separate basis: concurrent use, per hour, per shift, per day, total or unspecified. Each segment has a dedicated `capacity_evidence_ids` list distinct from general topology/evidence references. A numeric capacity requires at least one explicit capacity-evidence record. Evidence marked conflicted, stale, unknown, unavailable or unqualified cannot support a numeric capacity; scenario-only evidence may support only a proposed/assumed-for-scenario segment. Thus contradictory claims can remain attached to a segment whose selected capacity is null, but cannot silently justify a number. An amount without a meaningful basis remains visibly unspecified and must not be normalized as if it were a throughput rate or an operational reservation. A rated capacity is not the live availability of a road, track, terminal, lock, crane or vessel slot.

A route leg is not declared feasible merely because it is present in a projection. Infrastructure gauge, clearance, axle loads, track possessions, operating rules, waterway draft, cargo compatibility, legal limits, energy, timetable and transfer windows belong in the qualified planner and source evidence. An optimizer result must retain its exact scenario/evidence identity when shown in Sol Atlas.

## Validation implemented

The Rust validator checks:

- schema version and non-empty projection/source identities;
- duplicate or blank IDs per collection;
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
- The 2026 [Draft National Rail Master Plan consultation](https://www.gov.za/news/speeches/minister-barbara-creecy-launch-gauteng-stakeholder-engagement-draft-national-rail) describes a planned Rail Planning Component for freight/passenger flows, capacity, asset condition and rolling stock. The public-comment window closed on 22 July 2026, and the stakeholder platform still labels the document as a draft. Do not infer final approval, a live data feed or a project access agreement from the consultation page.
- The Western Cape's [intermodal freight-network proposal](https://www.westerncape.gov.za/mobility/intermodalism-building-more-connected-freight-network) reports a modelled opportunity of approximately 10.4 million tonnes of road freight that could potentially use intermodal rail terminals. This is a candidate for scenario selection, not guaranteed modal shift or verified current terminal capacity.
- The South African government's [geography overview](https://www.gov.za/about-sa/geography-and-climate) states that none of the country's rivers are commercially navigable. Inland-waterway and canal node/mode types remain valid for other countries or future engineered-waterway proposals, but the South African pilot should focus first on road–rail–port corridors, terminal transfers and water-system resilience rather than inventing existing commercial river freight.

For every prospective input, record the exact publisher, source URI or acquisition record, access/licence decision, source version, geographic and temporal coverage, source time profile, retrieval time, byte count, SHA-256 of exact bytes, parsing/transformation version, missingness and CRS. States such as not acquired, access review required, acquired-unreviewed, qualified for scenario and rejected must remain distinct. Aggregate national throughput must never be presented as route-level usable capacity.

## Staging

- **H0 — contract:** maintain the renderer-neutral schema, validator, synthetic fixture and deterministic-order tests.
- **H1 — read-only explorer:** add layer toggles, selection and node/edge details in the existing Sol Atlas UI; display observed, curated, derived, scenario, unknown, stale and conflicted states distinctly.
- **H2 — verified projection adapters:** consume qualified outputs from Symtropy and Mycelix only after source rights and mapping semantics are reviewed. Keep physical movement and communication-contact graphs separate.
- **H3 — replay and independent verifier:** replay frozen scenarios with missed handoffs, outages, stale capacity and contradictory evidence. A different validation path must check topology, statuses, capacity units and conflicts.
- **H4 — controlled pilot:** compare recommendations in shadow mode with historical and operator-reviewed outcomes before any operational dependency. Any reservation, dispatch or safety-critical act remains inside the responsible operator's authoritative system.

Passing the H0 structural validator establishes only that a frozen projection conforms to this declared contract. It does not establish transport performance, capacity, safety, scientific calibration, affordability or operational readiness.
