# Sol-Atlas evidence-preserving geospatial exchange profile v0.1

**Status:** research profile / draft; not an OGC Standard, OGC conformance claim, or production API contract.  
**Profile identifier:** `urn:luminous-dynamics:sol-atlas:evidence-profile:0.1`  
**Benchmark:** `ogc-evidence-roundtrip-v0.1`

## 1. Thesis

Sol-Atlas should not invent a replacement for interoperable geospatial transport. It should add a rigorously testable evidence-and-rights layer around established geospatial resources.

The profile composes:
- **OGC API - Features** for discovery and read access to geospatial features;
- **OGC JSON-FG 1.0.0** for feature typing, time, and CRS-aware geometry where appropriate;
- **OGC API - Records** for discovery of separately governed metadata/evidence records;
- **GeoParquet 2.0** as an optional bulk analytical distribution, never as an implicit substitute for the evidence graph or rights policy.

These are separate standards with different scopes. Passing this profile's semantic tests does not establish OGC API conformance. Conversely, an OGC-conformant feature service does not establish that evidence, rights, uncertainty, or provenance survive a GIS round trip.

## 2. Design rules

1. **The feature is not the claim.** Geometry is a spatial assertion; the claim object states what is asserted, by whom, for which temporal/spatial scope, and with what qualification.
2. **Evidence is typed, not a confidence scalar.** Support, contradiction, and context-only associations remain disjoint. Contradictions are retained, not averaged away.
3. **Rights are capabilities, not decorative metadata.** A reference, metadata record, content object, redistribution, and model-training permission are distinct capabilities.
4. **Unknown is a value.** Unknown time, unassessed geometry, missing source, and unresolved rights must not silently become false, zero, absent, or approved.
5. **Provenance is a graph.** Every derived claim/geometry binds to the inputs and processing activity that produced it.
6. **Uncertainty is dimensional.** Preserve metric, units, method, confidence level or interval semantics, reference frame, and sampling context. Do not treat an uncalibrated model score as measurement accuracy.
7. **Plural hypotheses are first-class.** Preserve competing interpretations and their evidence; do not force a single winner as an export side effect.
8. **Public projection is a security boundary.** Restricted identifiers, record counts, stable ordering, geometry details, and association metadata must not leak through error messages or aggregate fields.

## 3. OGC mapping and boundary

| Concern | Standard building block | Sol-Atlas profile responsibility |
|---|---|---|
| Feature discovery and query | OGC API - Features Part 1 Core | Implement only advertised conformance classes; validate endpoints with the official executable test suite |
| Non-default CRS and richer geometry | JSON-FG Core; Features Part 2 where API CRS-by-reference is needed | Preserve CRS URI, axis order, units and coordinate epoch; do not silently reproject |
| Feature type and one selected temporal dimension | JSON-FG `featureType`, `featureSchema`, `time`, `conformsTo` | Define which time dimension is surfaced in `time`; retain other time dimensions explicitly in profile properties |
| Evidence / source metadata discovery | OGC API - Records | Keep records separately governed; expose only metadata permitted by the projection policy |
| Bulk feature distribution | GeoParquet 2.0 | Keep geometry at root-level columns per specification; ship a separately digest-bound evidence manifest when nested semantics cannot be represented losslessly |

**Important JSON-FG details:** a JSON-FG root object declares its `conformsTo` URIs. Its `time` member represents one selected instant or interval; it does not magically encode every valid-time, acquisition-time, and transaction-time dimension. A GeoJSON `geometry` remains in the GeoJSON CRS convention; JSON-FG's `place` member and `coordRefSys` support additional CRS/geometry use cases. The profile must not overload those standard members with private semantics.

The example fixture uses a standard GeoJSON-compatible polygon in the default CRS. A second fixture should exercise non-WGS84 `place` geometry only after it is validated against the official JSON-FG schema and CRS requirements.

## 4. Profile payload

The profile extension lives inside the feature's `properties.solAtlas` object. It does not redefine GeoJSON or JSON-FG members.

### 4.1 Claim

A claim has a stable identifier, a machine-readable proposition or controlled concept, an explicit assessment state, and scope. Suggested assessment states:

- `supported`: currently admitted by the configured evidence policy;
- `contested`: material contradictory evidence is retained;
- `provisional`: insufficient or incomplete evidence;
- `rejected`: rejected by a recorded decision process;
- `unknown`: no qualified assessment exists.

These are workflow states, not universal truth labels. A claim's assessment must link to the decision/provenance record and must not be inferred solely from a confidence score.

### 4.2 Evidence associations

Each element uses exactly three disjoint association sets:

- `supports`
- `contradicts`
- `contextOnly`

Each association contains a stable record identifier, a relation matching its containing set, and a digest where the referenced payload has a stable digest. An evidence record may be associated with multiple features, but for one feature it must not occur in multiple association sets.

A counter-evidence or context-only record never satisfies positive support or direct-measurement admission. A model prediction remains a model prediction even when it contradicts an element. A missing dependency, mismatched relation, overlapping set, or digest mismatch is a hard profile failure.

### 4.3 Rights model

Rights must be expressed as separate dimensions rather than one ambiguous `licensed` Boolean:

- record/identifier visibility;
- metadata visibility;
- content retrieval;
- public disclosure;
- redistribution;
- commercial use;
- model training / tuning.

Each dimension is an explicit policy decision such as `allowed`, `prohibited`, `pending`, or `unknown`. These values are not legal advice and do not authenticate the policy decision.

**No inference rule may upgrade permission.** A public metadata record does not authorize retrieval of the underlying source. A digest is not permission to publish the content. A rights-pending reference is retained only when the projection policy separately authorizes that reference. When the identifier itself is restricted, the public projection must omit it entirely and must not reveal its presence through counts, placeholders, error text, ordering, or timing.

### 4.4 Temporal model

Use JSON-FG `time` for the one primary temporal dimension intentionally selected for interoperable feature-level filtering (for example, the feature's claimed validity interval). Keep independent temporal roles in the profile extension:

- `validTime`: when the claim is asserted to hold in the world;
- `observationTime`: when the source observation occurred;
- `recordedTime`: when the record entered the system;
- `decisionTime`: when an authority or reviewer made a decision.

A role may be `unknown`; do not replace it with an empty interval or a fabricated epoch. Preserve closed/open interval bounds and timezone/precision.

### 4.5 Geometry and metrology

Record the coordinate reference system, axis order, units, coordinate epoch where relevant, transform operation, and uncertainty budget. Distinguish:
- raw observation geometry;
- registered / transformed geometry;
- derived or reconstructed geometry;
- a geometry proposed by a model.

The test comparator has two modes: exact semantic identity when no transformation is allowed, and bounded geometric equivalence only when a declared transform, CRS operation and tolerance justify it. Tolerance must be tied to the source's declared uncertainty, not chosen after seeing results.

### 4.6 Provenance and hypotheses

Every derived result must bind its input digests, processing activity, algorithm/software version, configuration digest, operator class, and output digest. Preserve a hypothesis set as a set of distinct alternatives with their own provenance and evidence relationships. An export/import pipeline must not turn a list of alternatives into a selected winner.

## 5. Round-trip benchmark protocol

### 5.1 Evaluation path

For each case, run a frozen package through:

1. Sol-Atlas canonical input;
2. Sol-Atlas JSON-FG / OGC API representation;
3. independent consumer A export/import;
4. neutral archive / canonical re-encoding;
5. independent consumer B export/import;
6. re-import into the Sol-Atlas reference validator.

Adapters must be named and version-pinned. Preserve the exact package, tool versions, configuration, locale, CRS database version, and digests. Run each path at least three times to test deterministic replay. A skipped adapter is not a pass.

Each adapter hop must publish a **capability manifest** and a **semantic-loss ledger**. For every profile field, the ledger records one of: `preserved`, `transformed`, `omitted`, `blocked_by_policy`, or `unknown`; transformed fields include the mapping, parameters and resulting digest; omitted/blocked fields include a stable reason code. A required field marked omitted or unknown fails qualification. An adapter cannot claim success merely because the output remains syntactically valid GeoJSON.

The ledger is an auditable description, not a permission grant. It must not disclose hidden field names, IDs, values, or counts in public projections when those details are restricted.

### 5.2 Required tracks

| Track | Measurement | v0.1 qualification target |
|---|---|---:|
| OGC feature service | Official OGC API - Features ETS tests for each advertised class | 100% of applicable tests pass; no claim for untested classes |
| JSON-FG syntax | Official 1.0.0 schema / abstract-test-suite checks | 100% fixture validity |
| Identity & lineage | Feature, claim, record and activity references resolve and remain bound | 100% |
| Evidence semantics | Support / contradict / context-only relations preserved | 100%; zero relation promotion |
| Rights non-escalation | No capability changes from pending/unknown/prohibited to allowed | 0 violations |
| Privacy | Restricted identifiers and their existence remain unobservable in public projections | 0 disclosures on registered controls |
| Temporal semantics | Distinct temporal roles retain their values, precision and timezone | 100% |
| CRS and geometry | CRS, axis order, units and declared epoch preserved; geometric error bounded by pre-registered tolerance | 100% CRS identity; zero unexplained axis swaps |
| Uncertainty | Units, metric, method, coverage and bounds preserved | 100% |
| Hypothesis integrity | Alternatives remain separately addressable and unranked unless a decision record says otherwise | 100% |
| Deterministic replay | Canonical semantic digest stable across three identical replays | 100% |
| Semantic-loss disclosure | Every field disposition is declared; required semantics are not silently omitted | 0 silent losses; 100% disposition coverage |
| Privacy non-interference | Public projection is unchanged when only restricted inputs differ | 0 observable differences |
| Permutation invariance | Reordering semantically unordered records does not change canonical semantic output | 100% stable replay |
| Performance | Export/import throughput, p50/p95 latency, peak memory and package size | Report first; set thresholds only after matched baselines |

Targets are qualification gates for this benchmark, not achieved measurements or claims of superiority over GIS vendors.

### 5.3 Negative controls

Every run includes deliberate mutations and must reject or quarantine them:

- move a `contradicts` record into `supports`;
- duplicate one evidence ID across association sets;
- remove an evidence dependency while retaining its association;
- alter a source digest;
- change rights from `pending` / `unknown` / `prohibited` to `allowed`;
- expose a restricted record ID through a side channel;
- swap coordinate axes or remove the CRS identifier;
- drop the valid/observation/recorded temporal distinction;
- remove uncertainty units or method;
- collapse two competing hypotheses into one without a decision record;
- alter a model-derived geometry's origin to `measured`;
- replay the same package with a different hidden or unrecorded configuration;
- strip the complete Sol-Atlas extension while returning a syntactically valid feature response, without reporting loss;
- add or alter only a restricted source record and check whether the public projection changes;
- permute input evidence-record ordering and check whether the canonical semantic result changes.

Any negative control accepted as valid is a benchmark failure, even if visual similarity or geometric overlap remains high. The restricted-input comparison is an information-flow non-interference test; it is not differential privacy and must not be described as such.

### 5.4 Baselines

Compare the *same frozen package* across:

A. a GeoJSON-only round trip;  
B. a JSON-FG-preserving round trip;  
C. the Sol-Atlas profile round trip;  
D. at least two independent, version-pinned GIS/feature-service consumers;  
E. an expert-curated reference dossier for adjudication-sensitive cases.

Record adapter/version support and unsupported fields. Do not score a product as failing when a capability was not claimed; report it as `not supported` and show the resulting information loss. Do not use random tile splits: hold out whole sites / source collections to avoid leakage.

### 5.5 Scoring and anti-gaming

Report each track separately. The profile has hard safety gates; geometry score, speed, or visual quality cannot compensate for evidence loss, rights escalation, privacy leakage, or falsified provenance.

Publish:
- per-case and per-site outcomes;
- confusion matrices for relation classification;
- exact failure counts and reason codes;
- per-stratum worst-case results;
- 95% confidence intervals for statistical metrics;
- all adapter versions and configurations;
- raw and canonical semantic digests;
- unsupported/unknown field inventories.

No single composite leaderboard score is used for qualification.

## 6. What “better than SOTA” should mean here

The defensible target is **semantic survivability across heterogeneous tools**, not beating GIS platforms at map rendering or spatial indexing.

Sol-Atlas can make a meaningful contribution if it demonstrates, on independently held-out packages, that ordinary geospatial round trips lose or misstate evidence semantics while the profile preserves them without rights escalation or privacy leakage. That result must be measured, not assumed.

A credible claim ladder:

1. **Profile implemented:** profile objects and fixture cases exist.
2. **Structural checks pass:** internal invariants and negative controls pass on an exact commit.
3. **Interoperability demonstrated:** two independent adapters complete the same round trip.
4. **OGC conformance demonstrated:** the appropriate official ETS passes on a live endpoint.
5. **Comparative advantage demonstrated:** matched, independently reviewed experiments show lower semantic loss than the named baselines.

The present increment is level 1 only until executable fixtures and adapters are tested. It does not claim OGC certification or a SOTA result.

## 7. Implementation sequence

1. Freeze this schema/fixture vocabulary and add JSON Schema validation plus relation/rights/lineage negative controls.
2. Add a deterministic Rust reference validator and canonical semantic digest; keep the schema and fixture language implementation-neutral.
3. Expose a read-only OGC API - Features endpoint and separate OGC API - Records catalogue, with policy evaluation before projection.
4. Run the official OGC API - Features ETS and JSON-FG schema/abstract tests.
5. Add adapter containers for independent GIS clients and automate multi-hop round trips.
6. Add GeoParquet 2.0 bulk tests with a digest-bound sidecar; verify that format conversion cannot drop evidence or rights metadata unnoticed.

## 8. Non-claims and governance

This draft does not:
- certify Sol-Atlas or a third-party service as OGC conformant;
- authenticate evidence, rights receipts, custodianship or cultural authority;
- resolve contradictory historical interpretations;
- establish that model predictions are observations;
- decide who may access culturally sensitive information;
- claim measured performance or SOTA superiority.

The profile needs review by geospatial interoperability specialists, heritage practitioners, data stewards and affected communities. Public availability is not a substitute for consent, custodianship or appropriate authority.

## 9. Standards and test references

- OGC API - Features Part 1: Core: https://docs.ogc.org/is/17-069r3/17-069r3.html
- OGC API - Features standard and corrigenda: https://www.ogc.org/standards/ogcapi-features/
- OGC API - Features executable conformance test suite: https://cite.opengeospatial.org/teamengine/about/ogcapi-features-1.0/1.0/site/index.html
- OGC JSON-FG 1.0.0: https://docs.ogc.org/is/21-045r1/21-045r1.html
- OGC API - Records: https://docs.ogc.org/is/20-004r1/20-004r1.html
- GeoParquet 2.0.0: https://geoparquet.org/releases/v2.0.0/
- London Charter: https://www.london-charter.org/principles.html
- CARE Principles: https://www.gida-global.org/care
