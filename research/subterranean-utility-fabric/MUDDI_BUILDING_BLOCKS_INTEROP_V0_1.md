# MUDDI / OGC Building-Block Interoperability Profile v0.1

**Owner:** Sol-Atlas #755  
**Programme:** Symtropy SUF #4910  
**Status:** design proposal based on published OGC conceptual standards and a 2026 non-normative technology paper

## 1. Why one large schema is not enough

OGC MUDDI Part 1 v1.0 is a conceptual model. A conformant logical model must specialize its core classes, associations, attributes and abstract value types, including concrete geometry types. Its core conformance class has four requirement/test groups: A.1 concepts/classes; A.2 associations (source, target, direction, roles and maximum multiplicity); A.3 attributes (names, definitions, types and maximum multiplicity); A.4 concrete value types replacing AbstractValueType.

A list of ten MUDDI class names is not a full MUDDI implementation. A JSON schema that validates a local fixture is not proof that the logical model implements the UML model. The normative UML model has precedence where the document text and model differ.

OGC Document 26-050 (12 August 2026), Building Blocks for Interoperable, AI-Ready Geospatial Data, proposes a useful non-normative architecture for AI-ready data spaces: keep data structure and semantics separate, assemble reusable building blocks, and bind each dataset to exact schemas, vocabularies, transformation receipts and provenance. It is a discussion/technology paper, not an OGC standard or normative position.

The central design lesson for SUF is: **MUDDI conceptual conformance, local profile semantics, concrete data encoding, provenance and disclosure are related but distinct contracts.** A monolithic JSON object should not be expected to solve all of them.

## 2. Proposed building-block decomposition

Each block should be independently versioned, testable and reusable. Dependencies are declared rather than copied into every record.

| Block | Contract | Candidate standard/technology |
|---|---|---|
| B1 — MUDDI logical core | Feature classes, inheritance, associations, mandatory attributes and logical names | OGC MUDDI Part 1 v1.0; ISO 19107 geometry |
| B2 — Concrete spatial values | Geometry types, CRS, vertical datum, coordinate epoch, axis order, units, dimensionality and uncertainty | ISO 19107 / OGC Simple Features; GeoJSON for 2D-compatible exchange; JSON-FG / other explicit CRS encoding for extended geometries |
| B3 — Observation semantics | Sensor, observed property, feature of interest, procedure, result, result time and sampling context | OGC SensorThings API 2.0 (approved 2026-09-14; published 2026-11-04); OGC/ISO OMS; W3C SOSA/SSN; MUDDI Observation extension |
| B4 — Provenance and derivation | Source snapshot, agent, activity, transformation, exact input/output digest, method and rights | W3C PROV-O, DCAT 3, OGC API Records/STAC catalogue profiles |
| B5 — Vocabulary and quantities | Meaning of utility type, commodity, units, event classes, methods and epistemic terms | SKOS vocabularies, QUDT quantities/units; stable owner/domain vocabularies |
| B6 — Validation | Structural, semantic, graph and policy constraints with positive/negative examples | JSON Schema 2020-12 for structure; SHACL for graph constraints; executable independent checks |
| B7 — Identity and federation | ObjectID, RecordID, SystemID, owner/system namespace and resolution policy | MUDDI identifier concepts + URI/namespace policy |
| B8 — Security and disclosure | Purpose, audience, sensitivity class, precision generalization, authorized scope and audit | Utility-owner IAM policy; Mycelix/host authorization reference; no public geometry by default |
| B9 — Transformation receipts | Source-to-canonical mapping, CRS transforms, semantic uplift, versioned executable steps and errors | PROV-O activities; exact transformation/configuration digests; reproducible toolchain |

The format and semantic layers must be independently reusable. Two datasets can have the same structure but different meanings, or the same meaning encoded in different structures. A schema alone cannot resolve that distinction.

## 3. SUF implementation anatomy

A candidate building-block package should contain:

1. Identity and dependency declaration.
2. JSON Schema for structural validation.
3. JSON-LD context mapping local terms to shared semantic identifiers.
4. SHACL shapes for graph-level associations and conditions where RDF/linked-data representations are used.
5. Transformation specification for source CRS/format/semantic mappings.
6. Positive examples and negative fixtures.
7. Human-readable documentation and change history.
8. Conformance matrix binding each test to a requirement and exact block revision.

Not every block needs every artifact. The package should declare omissions and dependencies. JSON Schema is not a substitute for the MUDDI UML association tests; SHACL does not automatically prove standards conformance either.

## 4. Human-curated semantic uplift

Automation may detect missing source fields or unknown vocabulary terms, suggest candidate mappings, propose transformations, detect incompatible CRS/unit/time metadata, and identify outliers or contradictory evidence.

Automation must not promote a model inference to observed, choose a canonical ObjectID merely because geometry matches, resolve conflicting legacy records without an owner decision, infer asset absence from blank/unavailable layers, or decide that a disclosure/permit/access condition is harmless.

A human data curator or authorized asset owner retains the final say on semantic mappings and whether a data gap materially affects permitted use. Suggested/inferred values remain explicitly labelled and traceable.

## 5. MUDDI core conformance traceability

A logical profile claiming MUDDI Core conformance must demonstrate all four normative groups:

| MUDDI requirement | Evidence needed | Current SUF position |
|---|---|---|
| /req/core/classes-concepts / Abstract test A.1 | Every Core UML class and referenced concept represented | Draft crosswalk lists ten concepts; needs UML-level review |
| /req/core/classes-associations / Abstract test A.2 | Exact source/target/direction/roles/max multiplicity from UML | Not yet mapped or tested exhaustively |
| /req/core/classes-attributes / Abstract test A.3 | Every attribute name/definition/type/max multiplicity | Partial class/attribute work in fixture only |
| /req/core/concrete-value-types / Abstract test A.4 | Every AbstractValueType replaced by concrete implementation values, especially geometry | Not yet complete; CRS/geometry profile unresolved |

The MUDDI Core conceptual model also includes a LogicalName mapping-table requirement for changed names. The local profile must preserve a machine-readable mapping from each local concrete type and attribute back to the MUDDI concept. This is essential when using local jurisdiction/utility vocabularies.

MUDDI extended elements such as Observation, Sensor, Container and Access can be adopted for SUF use cases, but they must be declared as extended-profile choices rather than being confused with the ten mandatory Core classes.

## 6. Observation/evidence design for DAS and robots

A DAS event, robot inspection or survey result is an observation, not a canonical asset mutation.

The observation block should carry feature of interest or explicit unresolved target; observed property and result type; source instrument/fibre/interrogator/robot identity; procedure and method revision; calibration and clock/timebase; result time and valid time; chainage/geometry reference frame and uncertainty; raw artifact reference, digest and rights; route/asset revision used to interpret the result; derivation and transformation receipts; currentness, availability, epistemic and disclosure states. For interoperable observation access, use a declared OGC SensorThings API 2.0 serving profile and OMS/SOSA semantics rather than inventing a competing sensor API. SensorThings is an observation access model, not the canonical asset registry, physical-truth authority or security policy.

For fibre sensing, preserve two different results:
- fibre_chainage_event: a position along a particular fibre revision;
- geographic_route_location: a mapped geographic point/geometry after route calibration.

A chainage result cannot be silently cast into a geographic location. The second claim must bind the first result, the exact route model, coordinate transform, calibration, uncertainty and independent validation.

## 7. Proposed machine-checkable acceptance gates

- 100% of mandatory MUDDI Core concepts and referenced classes mapped.
- 100% of Core associations mapped with exact source/target/direction/roles/multiplicity.
- 100% of Core attributes mapped with definitions/types/multiplicities.
- 100% of mandatory AbstractValueType fields assigned concrete types.
- Complete LogicalName mapping for renamed/localized elements.
- Positive and negative examples for every mandatory relationship, value type and ID rule.
- Round-trip preservation of object, record and system identifiers; provenance; missingness; conflicts; uncertainty; disclosure; and source rights.
- No high-confidence geography without a valid route/coordinate transform.
- A conformance report tied to exact schema, context, shapes, transformation and fixture revisions.
- Independent reviewer sign-off before any claim of MUDDI Core conformance.

## 8. References

- OGC MUDDI Part 1 v1.0, especially §§2, 5, 8.5–8.6, 9.2 and Annex A: https://docs.ogc.org/is/23-024/23-024.html
- Official OGC MUDDI schema/model directory and v1.0 UML XMI: https://schemas.opengis.net/muddi/ and https://schemas.opengis.net/muddi/1.0/
- OGC Document 26-050 (12 August 2026), Building Blocks for Interoperable, AI-Ready Geospatial Data: https://docs.ogc.org/techpaper/26-050/26-050.html
- OGC SensorThings API Part 1 v2.0 (OGC 23-019): https://docs.ogc.org/is/23-019/23-019.html
- OGC Observations, Measurements and Samples (OMS): https://www.ogc.org/standard/oms/
- OGC SensorThings API security and authorization requirements: authorization policy is not prescribed by the standard; use utility-owner IAM and least-privilege serving boundaries.
- DAS observation profile: `DAS_OBSERVATION_INTEROP_PROFILE_V0_1.md`
- SOSA/SSN: https://www.w3.org/TR/vocab-ssn/
- PROV-O: https://www.w3.org/TR/prov-o/
- JSON-LD 1.1: https://www.w3.org/TR/json-ld11/
- SHACL: https://www.w3.org/TR/shacl/
- Sol-Atlas SUF MUDDI issue #755: https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/755