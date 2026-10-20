# AIRSPACE-ATLAS-001 — Airspace source validity and spatial evidence contract

**Status:** research contract / v0.1  
**Owner:** Sol-Atlas  
**Consumer:** Symtropy synthetic/offline scenario engine  
**Evidence reasoning:** Symthaea  
**Provenance/access-control composition:** Mycelix AeroCommons  
**Claim ceiling:** read-only spatial context; never an operational clearance or proof that a route is allowed.

## 1. Design shift

Treat the airspace atlas as a **versioned evidence catalogue**, not a map layer with a single `current` boolean. A feature may be geometrically present but stale, incomplete, outside the source's authority, not yet effective, expired, in conflict with another record, or unsuitable for a particular operation. These conditions must remain distinct in both the data model and the UI.

A map should answer separately:

1. **What does this source publish?**
2. **Where and when does the claim apply?**
3. **What territory, altitude, time, or attribute coverage is actually known?**
4. **Is the source fresh enough for this declared use?**
5. **Which transformations and derived claims were applied?**
6. **What remains unknown, conflicted, out-of-scope, or requires a human/authority check?**

The system must not turn a missing feature into evidence that there is no restriction. Nor may a mapped airfield, route, vertiport, or corridor be presented as operational, available, or permitted solely because geometry exists.

## 2. Record model

Each source-backed claim should be an immutable, revisioned record. The initial structural shape is in [`schemas/aviation/airspace-source-record.schema.json`](../../../schemas/aviation/airspace-source-record.schema.json). An isolated Rust checker at [`tools/airspace-source-contract-check`](../../../tools/airspace-source-contract-check/Cargo.toml) validates the schema against the Draft 2020-12 meta-schema and checks cross-field invariants on the fixture corpus. Neither structural nor semantic validation authenticates a publisher or establishes operational/legal authority.

| Group | Required fields | Why it matters |
|---|---|---|
| Identity | `record_id`, stable `feature_id`, `claim_kind`, schema version, record revision | Keep different claims about one feature separate and make revisions addressable |
| Publisher and product | publisher/authority name, product family, publication/document ID, edition or revision, canonical source locator | Distinguish source identity from a downloaded file or display name |
| Acquisition | `retrieved_at` UTC, source publication/issue time, source update token, response metadata where lawfully retained | Retrieval time is not publication time, and neither alone proves validity |
| Validity | `effective_from`, `effective_until`, source-specific freshness policy ID, expiry/withdrawal state | Support future-effective, expired, withdrawn and historically valid records |
| Integrity and authenticity | source-byte SHA-256, byte length, signature/authentication evidence where the publisher provides it, verification method/version | A hash can establish equality with bytes that are available; it does not authenticate a publisher or prove the content is true |
| Rights | acquisition permission, storage permission, redistribution/publication permission, model-use permission, license/terms reference, review state | Publicly reachable data is not automatically licensed for every use |
| Spatial semantics | horizontal CRS or named local frame, axis order, units, coordinate epoch if relevant, vertical datum, vertical units, transformation operation/version | Prevent silent mixing of coordinates, altitudes and datums |
| Scope | jurisdiction/authority scope, geographic coverage, altitude range, time range, aircraft/operation category when the source defines one | A source can be suitable for discovery but not for an operational constraint claim |
| Quality and coverage | geometry validity result, topology result, source coverage footprint, attribute coverage, known gaps, parser/adapter revision | An empty query is not evidence of complete coverage |
| Lineage | parent/source record IDs, `supersedes`, derived-from links, manifest digest, processing command/tool version | Enable deterministic replay, change impact, and audit history |
| Evidence class | authority-published, measured observation, third-party derived, crowd-sourced, synthetic, or human-reviewed; confidence scoped to a named field/use | Do not collapse different kinds of evidence into one confidence score |

Use UTC instants for validity while retaining the source's original notation and timezone as evidence. Do not infer an effective interval from a crawl timestamp. Missing required times, unknown datum, unlicensed use, invalid geometry or unverified transformations must produce an explicit unresolved state, not a guessed default.

## 3. State machine and decision semantics

State is multi-dimensional; do not compress the following into one “green” status.

- **Present:** this record exists in the retrieved source snapshot.
- **CoverageKnown:** the source declares coverage for the relevant geographic/time/attribute domain and that coverage has passed its declared checks.
- **FreshForUse:** the source revision and freshness policy meet the requirements of the *specific declared use* at the assessment time.
- **NotYetEffective / Expired / Withdrawn:** validity is known and the requested time falls outside it or the publisher withdrew the record.
- **Stale:** the relevant refresh contract or expected source update has not been satisfied. Preserve the historical value but do not let it satisfy a current-use gate.
- **Conflicted:** multiple applicable records disagree and the resolution policy does not establish a supported winner.
- **OutOfScope:** the source or record does not apply to the requested jurisdiction, altitude, time, aircraft category, or operation.
- **Unknown:** the evidence is insufficient to decide. Record a reason such as missing product, incomplete coverage, invalid transformation, inaccessible source, or unresolved source conflict.
- **AbsentWithinVerifiedCoverage:** an explicit, testable negative claim limited to a stated source revision and verified coverage domain. This is not the same as a missing record or a claim that the airspace is unrestricted.

A consumer asking a *current-use* question must evaluate applicability, coverage and freshness for the requested location, altitude and time. If a required constraint source is unknown, stale, out-of-scope or conflicted, return an unresolved/not-assessable result. The consumer may offer a historical or discovery view, labelled accordingly, but must not silently relax the constraint.

Source precedence must be an explicit, versioned policy scoped to a particular field and purpose. “Last write wins,” popularity, vote count, or an unscoped trust score is not a sound conflict-resolution rule for authoritative aviation information. Preserve conflicting inputs and the reason a resolution was or was not accepted.

## 4. South African source handling

For South African work, include the SACAA's official aeronautical information entry points in the source catalogue: AIP and amendments, AIP supplements, charts, NOTAM summaries, aerodrome/helistop directories, registered/approved aerodromes, AIRAC dates and publication checklists. Store source revision and effective interval per product instead of assigning one global “SACAA current” timestamp.

Source discovery must not be treated as operational approval. Before a real operation, the applicable current source, relevant NOTAMs/supplements, operator procedures and authority rules must be independently checked by competent personnel. The research prototype must not fetch, redistribute or publish material without recorded rights review, and must not expose restricted operational data or private flight plans.

The checker uses the Rust `jsonschema` Draft 2020-12 implementation ([documentation](https://docs.rs/jsonschema/0.59.0/jsonschema/draft202012/index.html)) for standard schema/meta-schema validation, plus explicit project-specific cross-field rules. The validator is an engineering test harness only.

Useful official source pages:
- SACAA, [Aeronautical Information / AIP amendments](https://www.caa.co.za/industry-information/aip-amendments/)
- SACAA, [AIP Supplements](https://www.caa.co.za/industry-information/aip-sup/)
- SACAA, [Aeronautical Charts and Information](https://www.caa.co.za/industry-information/aeronautical-information-aeronautical-charts/)
- SACAA, [Airworthiness](https://www.caa.co.za/industry-information/airworthiness/)
- EUROCONTROL, [Specification for Trajectory Prediction, Edition 2.0](https://www.eurocontrol.int/publication/eurocontrol-specification-trajectory-prediction)

This is a source-ingestion and evidence-boundary contract, not an interpretation of the legal effect of any individual publication.

## 5. Spatial and temporal query contract

A query result should return the feature/claim, source revision, effective interval, assessment time, queried geographic and altitude domain, applicable policy revision, evidence/coverage state, transformation provenance, data-rights state, unresolved dependencies, and reasons for any non-assessable result.

Required semantics:

- Exact source snapshots and manifests must be replayable from recorded source revision and adapter/tool versions, subject to source availability and use rights.
- A query's answer must declare the **time of assessment** and the **time the source claim is valid for**. These are different axes.
- Separate horizontal extent, vertical extent, and temporal validity. Unknown vertical datum cannot be treated as an unlimited or matching altitude range.
- Transformations must name input/output reference systems, axis ordering, units, operation/pipeline, implementation/version, and any grid or datum resources used.
- Geometry validation, source freshness, legal applicability and operational status are distinct checks with distinct failure reasons.
- When source updates supersede or retract a record, preserve the old revision and record the state change; never silently overwrite history.
- Derived intersections or route-context queries must cite their exact input records and transformation configuration. A derived geometry does not inherit more authority than its inputs.

## 6. Benchmark and falsification plan

Freeze a synthetic fixture corpus first. Do not make a conformance claim against real aviation data until reuse rights, expected source revisions, field-level reference answers and independent reviewers are in place.

| Benchmark | Acceptance target |
|---|---|
| Invalid CRS, axis order, units or vertical datum | 100% rejected or returned as not-assessable; no implicit fallback |
| Future-effective, expired, withdrawn and overlapping-validity records | 100% correct state transition on frozen fixtures |
| Stale cached source with a newer expected publication | Never accepted as fresh for a current-use query |
| Missing restriction source vs verified coverage with no matching feature | Never conflated |
| Conflicting applicable records | Preserved as conflict unless an explicit scoped policy resolves them |
| Tampered source bytes or wrong digest | 100% integrity failure |
| Changed parser, transform, policy or source revision | Dependents invalidated or explicitly marked for re-review |
| Repeat export of same frozen sources/configuration | Byte-identical canonical manifest and stable record ordering |
| Incomplete geographic/attribute/temporal coverage | Coverage gaps exposed in result; no inference of unrestricted area |
| Data-rights failure or unknown licensing state | No prohibited redistribution/publication |

For larger test sets, measure ingestion throughput, update-detection delay, snapshot-build p50/p95, query p50/p95, memory, stale-record detection recall, false freshness acceptance, coverage completeness and lineage resolution. Establish hardware-specific baselines before setting latency targets. Correctness, rights and freshness gates may not be traded against speed.

## 7. Architecture boundary

- **Sol-Atlas:** source adapter, immutable spatial/context records, coverage, validity, transformation provenance, change detection and read-only query output.
- **Symtropy:** evaluates synthetic/offline trajectories and scenarios against supplied, explicitly scoped context; owns its separate geometry/vehicle-feasibility logic.
- **Symthaea:** checks evidence consistency, missing assumptions and model applicability; abstains when evidence is insufficient.
- **Mycelix:** retains access-controlled evidence lineage, signer/role attestations where available, review receipts and change-impact relationships.

No subsystem converts this catalogue into clearance, flight release, operational separation assurance, or an airworthiness decision.

## 8. Immediate implementation sequence

1. Review and freeze the record/status vocabulary and source-rights semantics.
2. The Rust checker now performs Draft 2020-12 meta-schema validation, format-enabled instance validation, and semantic cross-field checks for time ordering, freshness, coverage completeness, source conflicts, withdrawn claims, and authentication-state consistency. Negative controls test malformed CRS identity, digest syntax, stale-to-current promotion, unknown-to-complete coverage promotion, conflict suppression, reversed validity time, and failed-authentication promotion.
3. The `airspace-source-contract-check` job is wired into `.github/workflows/check.yml` to run isolated-workspace rustfmt, clippy, and tests. Its exact current-head CI result must be checked before calling this work green; local tests have not been run in the authoring environment.
4. Implement a read-only synthetic fixture adapter and canonical manifest serializer; do not begin with live NOTAM ingestion.
4. Add independent golden fixtures for CRS/datum rejection, interval boundaries, explicit coverage gaps and source revision changes.
5. Emit structured reasons and dependencies so Symtropy can distinguish “known clear in this fictional scenario” from “not assessable.”
6. Only after those gates pass, assess whether an actual source adapter is permitted and maintainable; require human review of source authority, applicability and updates.
