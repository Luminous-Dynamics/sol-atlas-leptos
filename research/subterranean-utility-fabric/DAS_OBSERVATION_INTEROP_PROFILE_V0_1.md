# DAS Observation Interoperability Profile v0.1

**Owner:** Sol-Atlas #755  
**Programme:** Symtropy SUF #4910 / DAS qualification #4972  
**Status:** proposed local application profile; not an OGC conformance claim  
**Date:** 2026-11-13

## 1. Purpose and design decision

Define a vendor-neutral way to describe DAS observations and link them to underground spatial evidence while reusing existing standards.

Use:
- OGC MUDDI Part 1 v1.0 for underground feature/network concepts and owner identifiers;
- OGC SensorThings API Part 1 v2.0 for interoperable sensor/observation access;
- OGC/ISO Observations, Measurements and Samples and W3C SOSA/SSN semantics for observation meaning;
- W3C PROV-O or equivalent for derivation and transformation lineage;
- immutable vendor-native, HDF5 or Zarr waveform artifacts for high-volume payloads.

This document is a local profile proposal. It does not claim that the MUDDI logical model is conformant, that any vendor exports a common raw DAS representation, or that SensorThings API itself validates a physical measurement.

## 2. SensorThings conceptual mapping

Use these mappings as profile defaults; each adapter must declare deviations and source limitations. In SensorThings, a Thing is a physical or virtual identifiable host/platform; a Sensor is a hardware, human or software observer; and an Observation estimates an ObservedProperty of a FeatureOfInterest, which may be proximate or ultimate. Do not collapse these roles.

| SensorThings concept | SUF/DAS profile meaning | Important constraint |
|---|---|---|
| Thing | The identifiable physical/virtual monitoring host or platform in the SensorThings sense—for example an interrogator installation or monitoring station that hosts sensors/datastreams | It is not the observed fibre route or the canonical owner utility asset; represent those as FeatureOfInterest / external MUDDI asset references as appropriate |
| Sensor | Exact interrogator measurement source or a declared processing sensor | Keep vendor model, serial, firmware and processing revision where available |
| Datastream | Measurements with the same sensor, observed property, unit/result type and declared spatial/temporal scope | Do not mix DAS strain, DAS strain-rate, DTS temperature and CAMOS electrical-current measurements in one stream |
| ObservedProperty | Explicit phenomenon such as axial strain, strain-rate, temperature or electrical current | Quantity, definition and unit are mandatory profile semantics; CAMOS current is not DAS |
| Observation | One event, aggregate result, or a bounded reference to a measurement artifact | Include phenomenonTime, resultTime and a precise result/resultType representation |
| FeatureOfInterest | The proximate observed fibre span/route segment, or an explicit unresolved reference; the ultimate feature may be an owner utility asset only where the relationship is evidenced | Do not equate the monitoring Thing with the observed feature or infer asset attribution from proximity |
| MultiDatastream | Only where measured properties and result structure genuinely form a jointly defined multivariate record | Do not use as a generic bag for vendor channels with incompatible units/configuration |

Where suitable, SensorThings API 2.0 resultType can describe structured values using the OGC SWE Common Data Model JSON encoding (OGC 24-014). The adapter must preserve the vendor-native artifact and record the exact mapping and conversion method.

## 3. Observation result envelope

The following is the proposed local semantic minimum, not a final JSON Schema:

- observation_id and source_artifact_id;
- source_owner and source_system_namespace;
- thing_id, sensor_id, datastream_id and exact revisions;
- observed_property URI/identifier, definition, quantity kind and unit;
- phenomenon_time, result_time, time_zone/clock basis and synchronization quality;
- result_type, encoding, spatial/channel indexing and sampling description;
- fibre_id and fibre_revision, when known;
- interrogator_model, firmware/configuration, optical_path and direction;
- gauge_length, channel_pitch, pulse_rate, sample_rate, wavelength/mode and calibration references where provided by the source;
- processing_method and software revision, including any filtering, resampling, normalization or windowing;
- feature_of_interest reference, or a typed unresolved state;
- fibre-relative range/chainage and uncertainty when available;
- route/geographic transform reference only when separately qualified;
- raw waveform URI, immutable object version, byte length, media type and SHA-256 digest;
- data rights, access classification, custody, retention and permitted purpose;
- epistemic state, data-quality state, observability state and independent review status.

If a source does not supply a field, represent it as unavailable/unknown with a reason. Do not infer gauge length, units, clock quality or geographic coordinates from a vendor model name.

## 4. Large waveform payloads

DAS may generate high-volume data. Do not force each sample, channel or high-rate time window into an independent SensorThings Observation resource.

Instead:

1. Preserve the original vendor-native object or authorized raw recording unchanged.
2. Store the payload as a versioned immutable object in a supported container format such as HDF5 or Zarr where the data provider and rights permit.
3. Expose a SensorThings Observation describing the relevant measured property, time interval, sensor, feature of interest and result metadata.
4. Reference the waveform object using a stable URI/version and bind its exact bytes with length, media type and digest.
5. Record the mapping between DAS channels/ranges and fibre path as a separately versioned calibration object.
6. Create normalized or windowed derivatives as new artifacts with full parent references and transformation receipts.
7. Retain a route-segment/time index for efficient retrieval without mutating the source waveform.

A digest establishes byte identity for an acquired artifact; it does not authenticate publisher identity, establish licence rights or prove measurement truth.

## 5. Coordinate and identity separation

At least five different references must remain distinct:

1. interrogator optical range;
2. physical fibre chainage on a named fibre revision and endpoint direction;
3. route station on a versioned fibre/utility-route geometry;
4. geographic position with CRS, axis order, vertical datum and coordinate epoch;
5. owner-issued asset identity and attribution.

A DAS observation with optical range or fibre chainage is valid as a fibre-relative observation even when geographic mapping is unavailable. The system must not fabricate a geographic FeatureOfInterest to satisfy an API shape.

For SensorThings representation, a provisional or unresolved feature may use a separate feature resource with a declared unresolved status and no invented point geometry, or the observation may remain outside geographic serving until the profile defines a conformant encoding. An API consumer must not mistake a fibre chainage for an Earth coordinate.

Route calibration and transformation evidence must bind:
- exact source observation and fibre identity/revision;
- exact optical configuration and endpoint/direction;
- physical path/topology assumptions;
- exact route geometry revision;
- CRS, axis order, vertical datum and coordinate epoch;
- control-point identities and independent truth source;
- residuals, uncertainty and validity interval;
- transformation software/method and digest;
- approval/adjudication status.

A fibre repair, splice change, reroute, endpoint swap, interrogator change or reference-frame change may invalidate dependent transforms. Preserve old receipts for replay and mark them superseded; do not silently rewrite them.

## 6. Epistemic, availability and quality states

Keep these orthogonal rather than collapsing all states into one status field:

- **epistemic:** observed, compiled_secondary, model_inference, scenario_assumption, speculative_hypothesis, unknown;
- **availability:** available, unobserved, unavailable, withheld, stale;
- **quality:** valid, degraded, invalid, conflicting, unsupported;
- **review:** unreviewed, accepted, rejected, indeterminate.

A no-result interval is not necessarily a negative event observation. It may be blind, unobserved, unavailable, decoupled or invalid. No API response should turn a missing waveform into a clean bill of health.

## 7. Security and operational boundary

SensorThings API 2.0 includes an authentication conformance class, but its security considerations explicitly leave the authorization mechanism to the implementation and do not define integrity/confidentiality controls for stored data. The standard uses HTTP 401 for authentication-required and 403 for forbidden cases; that is not a complete critical-infrastructure access policy.

For SUF, the serving implementation must additionally satisfy:

- **Deny-by-default authorization:** apply owner IAM and purpose/audience/route scope at the service/data layer, preferably with database-level row/resource policies. Do not rely only on a reverse proxy that checks the visible URL path.
- **OData relationship closure:** authorization must cover nested relationships and equivalent access paths through `$filter`, `$expand`, navigation properties and related entity collections. A restricted fibre or asset must not leak through an otherwise permitted query.
- **Write protection:** public and partner-facing observation endpoints are read-only by default. Only explicitly authorized acquisition/ingestion actors may publish, and they cannot mutate canonical utility asset state.
- **MQTT protection:** anonymous publishing is disabled; broker authentication, per-client publish/subscribe authorization, topic restrictions, message-size/rate quotas and abuse monitoring are mandatory. High-volume observation topics must not become a denial-of-service path.
- **Query/resource control:** cap time range, spatial extent, page size, aggregation cost, concurrent subscriptions and request rate. Apply bounded streaming and explicit timeouts.
- **Web boundary:** CORS allowlists are explicit; client applications treat descriptions/results as untrusted data and escape/sanitize before rendering.
- **Integrity and confidentiality:** protect transport and storage using the utility owner's approved controls. Observation digests do not replace authenticated source identity, access policy, encryption or audit.
- **Operational separation:** DAS data intake and serving remain read-only relative to SCADA and electrical protection.

Security conformance tests must exercise relation traversal and query-equivalent paths, not merely direct GETs to a single entity URL.

## 8. Interoperability and conformance boundaries

SensorThings API standardizes observation access but does not prescribe the utility owner's authorization model. Enforce authorization through utility-owner IAM and a narrowly scoped service policy. Apply minimum disclosure and least privilege to:
- exact fibre paths and splice locations;
- cable and utility asset topology;
- sensor/interrogator locations;
- event detections that expose critical-asset security posture;
- work/maintenance evidence and operator identity.

Public output should normally be aggregated or generalized. Exact critical-asset geometry is restricted by default.

Operational DAS intake must be read-only relative to SCADA and electrical protection. The profile does not authorize control writes, switching, isolation, dispatch, excavation or repair.

## 8. Interoperability and conformance boundaries

- MUDDI A.1–A.4 remain a separate Sol-Atlas logical-model conformance effort. Passing a local JSON Schema or SensorThings API example does not establish MUDDI conformance.
- SensorThings API conformance requires the applicable normative abstract tests and a real service implementation; this profile is not one.
- OMS/SOSA semantics and local vocabulary mappings must be versioned and machine-readable.
- JSON Schema validates structure; SHACL or independent graph checks may validate relationships where a graph representation is used; neither alone proves normative MUDDI conformance.
- Each transformation has a receipt and explicit error budget.
- Preserve original source terms and any commercial confidentiality restrictions.

## 9. Proposed interoperability tests

Positive tests:
- one DAS interrogator, one strain-rate datastream and a waveform object reference;
- one fibre chainage event with no geographic route yet;
- one route-calibrated event with a valid coordinate frame and uncertainty;
- one multimodal installation with distinct DAS and current-monitoring datastreams;
- one source-native result and one normalized derivative with exact lineage.

Negative controls:
- strain-rate presented as strain or temperature;
- channel pitch asserted as spatial resolution;
- missing unit, gauge length or sample-rate metadata silently filled;
- wrong fibre identity with numerically plausible chainage;
- endpoint reversal or stale splice revision;
- route transform applied after the route revision has changed;
- missing CRS or axis order;
- a fibre-relative chainage emitted as geographic coordinates;
- waveform bytes changed without digest update;
- a derived result presented as raw source data;
- an unobserved interval represented as a negative event observation;
- unauthorized consumer requests exact restricted geometry;
- SCADA write or protection action attempted through the observation interface.

## 10. References

- OGC SensorThings API Part 1 v2.0 (OGC 23-019), including Annex IV Security Considerations: https://docs.ogc.org/is/23-019/23-019.html
- OGC SensorThings API v2.0 publication (2026-11-04): https://www.ogc.org/announcement/ogc-publishes-sensorthings-api-version-2-0/
- OGC MUDDI Part 1 v1.0 (OGC 23-024): https://docs.ogc.org/is/23-024/23-024.html
- OGC SWE Common Data Model (OGC 24-014): https://docs.ogc.org/is/24-014/24-014.html
- OGC/ISO Observations, Measurements and Samples: https://www.ogc.org/standard/oms/
- W3C SOSA/SSN: https://www.w3.org/TR/vocab-ssn/
- W3C PROV-O: https://www.w3.org/TR/prov-o/
- SUF Symtropy DAS workstream #4972: https://github.com/Luminous-Dynamics/symtropy/issues/4972

## 11. Claim ceiling

This is a local interoperability profile proposal. It is not a deployed API, a SensorThings or MUDDI conformance result, an interrogator adapter, an asset registry, a security approval or a measured DAS result.
