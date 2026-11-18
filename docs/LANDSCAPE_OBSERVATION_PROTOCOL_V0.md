# Landscape Observation Protocol v0

**Status:** schema + strict Rust batch validator + synthetic fixture. No real site measurements are included.  
**Purpose:** connect a landscape design and its evaluation plan to later observations without confusing authored intent, model output, and measurements.

## Paradigm shift

A landscape design should be treated as a falsifiable intervention rather than a static drawing. The repeatable chain is:

`design snapshot → preregistered evaluation plan → model result → observation → comparison → revised decision`

This contract covers the portable observation envelope only. It is not a sensor API, database, causal-analysis engine, consent verifier, or proof that an intervention worked.

## Relationship to standards

- **OGC SensorThings API Part 1: Sensing 1.1** provides the interoperable sensing model for Observation, Datastream, Sensor, ObservedProperty and FeatureOfInterest, with phenomenon and result times: https://docs.ogc.org/is/18-088/18-088.html
- **OGC STAplus 1.0** extends SensorThings for user-owned/citizen-science observations and licensing: https://www.ogc.org/standards/sensor-things-api-extension/
- **OGC JSON-FG 1.0** is a candidate format for rich geospatial feature exchange (including projected CRS and temporal/spatial feature properties), not a reason to rewrite SensorThings: https://www.ogc.org/standards/json-fg/

The native record is an **offline application envelope**, not an attempt to clone SensorThings. A future read-only adapter should map standard SensorThings fields into this envelope and preserve provider IDs, URLs, version and raw-source digests. Do not implement SensorThings Tasking, control endpoints or actuator authority here.

## Files and validation

- Record schema: `schemas/landscape-observation-v0.schema.json`
- Offline batch schema: `schemas/landscape-observation-batch-v0.schema.json`
- Synthetic fixture: `fixtures/landscape/observation-batch-v0.json`
- Rust validator/tests: `src/landscape_observation.rs`

The canonical fixture covers all three location representations, all result kinds, and uncertainty states; its two-step transformation chain is checked. The batch validator enforces exact keys, stable opaque IDs, project/site references, observation/result time ordering, property/result unit agreement, explicit evidence class and source kind, digest syntax, coordinate bounds and CRS labels, quality/calibration state, uncertainty consistency, privacy policy, transformation-chain continuity and a 10,000-record batch limit.

### Idempotency rule

- Same observation ID + identical record content is a recognized, reported duplicate; `normalize_batch` returns the first copy in original order and the unique count remains unchanged.
- Same observation ID + different content is a hard conflict. There is no last-write-wins behavior.
- Stable source event ID and source-payload digest let later adapters trace a record back to its upstream event.
- This v0 validator does not itself compute a digest over canonical JSON or verify a claimed source payload digest against retained source bytes. It checks digest syntax and lineage relationships only.

## Evidence classes and boundaries

- `field-observation`: a field procedure recorded a result.
- `sensor-observation`: a sensor/instrument produced a result; an accepted record requires declared current calibration.
- `remote-sensing-classification`: a derived classification with source/method provenance, not a direct ground truth.
- `lab-analysis`: an analytical result with source and method.
- `community-report`: a participant-originated report.
- `imported-observation`: a record mapped by a declared external adapter.
- `synthetic-test`: test data that must remain explicitly synthetic.

A valid envelope says the fields are internally consistent; it does not prove the measurement is accurate or unbiased. An accepted quality state is a declaration from the source workflow, not a scientific certificate. Stale/unavailable calibration cannot silently become an accepted sensor result.

Time is represented as Unix milliseconds in the native contract. Adapters translating to SensorThings ISO 8601 timestamps must declare time basis and preserve source timestamps and clock uncertainty; unknown time must not be replaced by a fabricated precise time.

## Spatial and privacy rules

Location is one of:
- `geographic-wgs84`: explicit `OGC:CRS84` longitude/latitude order, bounds and precision in metres;
- `project-local`: explicit frame ID, units, XY order and precision;
- `opaque-site`: an opaque site-reference digest plus a declared precision class, without raw coordinates.

This validates representation, not a datum transformation or the truth of location accuracy. Local drawing units are never automatically treated as metres or geographic coordinates.

The privacy envelope references a consent/authority record under the Mycelix boundary. **The Rust validator does not verify legal consent or grant publication rights.** It blocks raw-public approval when the record is not classified public, the consent reference is absent, or the location is not marked public. Sensitive coordinates must be redacted/generalized by a separate audited publication step before they can be released. Presence of a `publication_policy` is not itself permission to publish.

Observer references, instrument references and consent references are opaque SHA-256 identifiers, not public personal identities. Author/observer identity is distinct from sensor/source identity.

## Acceptance gates

- Schema constants and the canonical synthetic fixture are tested in Rust.
- Negative tests cover duplicate/conflicting IDs, unit mismatch, reversed time, invalid CRS/coordinate bounds, stale sensor calibration, synthetic-to-field relabeling, unsafe publication, malformed uncertainty, broken transformation digest chains, unknown fields and incomplete project references.
- Identical duplicate import is idempotent and reported; conflicting content fails closed.
- All third-party adapters use mocked/offline fixtures in CI. No CI test depends on a live sensing server.
- The evaluation plan stays separate: a plan declares how an observation will be interpreted; the observation carries what was recorded. Neither one proves impact or causality.


## Read-only SensorThings adapter profile

The initial adapter is a **pure JSON mapper**, not an HTTP client. Its fixture is `fixtures/landscape/sensorthings-observation-v1-1.json`, and implementation/tests live in `src/sensorthings_adapter.rs`.

Profile v0 accepts an expanded Observation containing:
- an OData Observation ID;
- RFC3339 phenomenon time as an instant or one start/end interval;
- RFC3339 result time with explicit UTC/offset and at most millisecond fractional precision;
- finite numeric result;
- expanded Datastream, Sensor, ObservedProperty and unit-of-measurement symbol;
- caller-pinned source URI/version and SHA-256 of the exact raw bytes.

Other result types, absent unit/sensor/observed-property metadata, unsupported timestamp precision, missing payload digest, and invalid site/project references fail closed. No HTTP requests, SensorThings Tasking/commands, or actuator paths are implemented.

The adapter deliberately does not copy FeatureOfInterest coordinates. It emits an opaque site reference, private-only publication policy, unreviewed quality state, unknown calibration state, unknown measurement uncertainty, and a calibration-not-verified flag. This is a conservative ingest state, not a claim that the sensor reading is accepted or accurate.

The mapping is recorded with profile ID `urn:luminous:adapter:ogc-sensorthings-sensing-1.1-landscape-v0`. In this v0, its output digest is SHA-256 of the mapped JSON value with an empty transformation list, serialized by the pinned `serde_json` Value serializer; it is not an RFC 8785/JCS digest. The raw input digest is supplied by the caller. A later canonical payload-digest contract should replace or formally pin this profile before cross-language digest equality is promised.

Tests prove the sample FeatureOfInterest geometry is not copied into the adapted output, offset timestamps normalize consistently, sub-millisecond precision fails rather than being truncated, numeric-only result mapping fails closed for string results, and malformed source context is rejected.

## Cross-repository boundaries

- **Sol-Atlas:** portable schema, batch import, validation, display, source and design snapshot references.
- **Mycelix:** observer authorization, consent, publication permissions, stewardship commitments and epistemic evidence graph. Reuse Observation V2 concepts; do not duplicate the DHT graph.
- **Symtropy:** model qualification/comparison against observations and held-out validation.
- **Symthaea:** explain disagreements and alternate explanations; don't convert association to causation.


## Executable ingest benchmark

The CLI is in `src/bin/landscape_observation_bench.rs`, behind Cargo feature `landscape-observation-bench`. It is excluded from the default application and WebAssembly target. CI tests it together with the main native app target.

Example invocation on a pinned native host:

```sh
cargo run --release --features landscape-observation-bench --bin landscape-observation-bench -- \
  --commit-sha "$(git rev-parse HEAD)" \
  --machine-label "reference-host-01" \
  --cpu-model "fill-in-exact-cpu-model" \
  --ram-mib 32768 \
  --records 10000 --iterations 25 --warmup 3 --profile unique \
  > landscape-observation-bench.json
```

Run the same frozen host/toolchain across `unique`, `exact-1pct`, `exact-10pct`, `exact-100pct`, `conflict`, and `over-limit`. The CLI records raw per-iteration timings, p50/p95/p99, input/output digests, duplicate accounting, correctness mismatch count, OS/architecture/Rust version, and user-supplied host metadata. It reports a JSON status of `correctness-pass` only when observed validity matches the expected profile on all samples.

The CLI keeps input in memory; it does **not** establish operating-system cold-cache behavior, peak RSS, sensor-provider latency, or field-data quality. Those remain explicit gaps in the v0 acceptance protocol. Do not advertise the proposed 500 ms/1 s or 128 MiB targets as achieved until the report is captured on a named reference machine and independently checked.


## Performance metric ↔ observation crosswalk

Files:
- `metrics/landscape/observation-map-v0.json`
- `schemas/landscape-metric-observation-map-v0.schema.json`

The crosswalk maps the eight initial performance metric IDs to admissible or merely supporting observation properties, expected source-unit symbols, needed feature/project references and mandatory non-observation inputs. It explicitly marks several capabilities as unavailable in v0: canopy-cover area requires spatial geometry; event runoff requires hydrologic time-series aggregation and a qualified catchment/model; maintenance-assignment coverage requires a Mycelix governance artifact. Other measurements can supply evidence without themselves computing a metric.

The mapping currently declares `cross-pr-dependency-not-yet-merged-or-validated-against-registry`: its metric IDs refer to the landscape registry proposed in Sol-Atlas draft PR #191, which remains a separate branch. The Rust test pins eight unique IDs and checks that every mapping includes prohibited-inference text; it does **not** claim schema-instance validation against an unmerged registry. After the registry lands, the IDs and units should be checked against its exact schema/version before enabling automated calculations.

This is deliberate: a point observation does not become an area metric, a single discharge reading does not become event volume, and a sensor cannot attest that a maintenance obligation was accepted.
