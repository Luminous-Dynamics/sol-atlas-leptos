# Highveld Verification Crosswalk v0.1

**Parent:** HIGHVELD-KEEP-001 #607  
**Matched industry benchmark:** HIGHVELD-KEEP-INDUSTRY-001 #617  
**Related composition architecture:** PR #652 / issue #655  
**Related solver owner:** Symtropy EnergyPlus adapter #1900  
**Status:** research and test-protocol artifact; no solver, model, IFC export or physical article is qualified by this document  
**Evidence ceiling:** protocol definition only (E0/E1 planning)

## 1. Purpose

Prevent one successful validation layer from being silently promoted into a different engineering claim.

A building-model workflow has several distinct validation questions:

1. Is the exchange file syntactically and schema-valid?
2. Does it contain the required alphanumeric information?
3. Is its geometry, topology, placement and coordinate frame correct for the declared use?
4. Does a specialist simulation implementation behave within its declared applicability and reference tolerances?
5. Are the chosen model inputs representative of the actual site, climate, occupancy and operating conditions?
6. Does the physical article match the accepted digital definition?
7. Does the occupied asset deliver its declared services over time?

These are not interchangeable checks. A pass at one layer is evidence only for the layer and scope it actually tests.

## 2. Validation ladder and owner boundaries

| Layer | Question | Appropriate reference / owner | What a pass does not establish |
|---|---|---|---|
| IFC syntax and schema | Is the exchange file a valid STEP/IFC document under its declared schema and normative IFC rules? | buildingSMART IFC Validation Service; exact IFC schema edition | Project requirements, national rules, correct geometry for the project, or physical conformance |
| Information requirements | Are declared IFC objects, classifications, properties, quantities, materials and values present and valid? | buildingSMART IDS 1.0; project IDS profile | Clash-free geometry, distances, clearances, dynamic calculations, external references or as-built truth |
| Project/jurisdiction requirements | Does the model meet the exact project and jurisdiction-specific information constraints? | Versioned project profiles and responsible compliance owner #604 | Professional approval or approval by a competent authority |
| Geometry and topology | Are shape, placement, units, coordinate transforms, connectivity, clearances and declared geometric tolerances satisfied? | Exact geometry owner and independent geometry/metrology checks; Symtropy geometry boundary | Material properties, structural adequacy, fire performance, installation quality or real-world safety |
| Simulation software verification | Does the software implementation produce results within the applicable reference-test expectations? | ASHRAE Standard 140-2023 and solver owner #1900, where applicable | Project-model correctness, weather representativeness, professional approval or actual building performance |
| Project model validation | Are geometry, assumptions, schedules, boundary conditions, weather, calibration and applicability correct for this specific decision? | Specialist solver owner and qualified reviewer | Physical test results or occupied service outcomes |
| Fabrication / installation | Does the accepted physical article match its exact design, product, batch, process and installation requirements? | #622, Symtropy panel-cell/metrology owners, Mycelix #5149 | Whole-building commissioning or building-level service |
| Operation / service | Does the asset deliver each declared service to its required level and population over time? | #574, #633/#635, #657 and service owners | Causal benefit beyond the declared evidence and comparison design |

The validation sequence is intentionally layered. An IFC failure may block later checks. An IFC pass merely allows downstream checks to proceed; it does not pre-approve them.

## 3. Standards and reference systems

### 3.1 IFC validation

The buildingSMART IFC Validation Service checks STEP physical-file syntax, the referenced IFC schema and normative IFC specification rules. It also reports certain industry-practice warnings. Its stated boundary excludes project-specific, national-specific and organization-specific rules.

Use it as an exchange-file integrity gate, not as the final project checker.

References:
- https://info.buildingsmart.org/users/services/validation-service/
- https://technical.buildingsmart.org/services/validation-service/
- https://technical.buildingsmart.org/standards/ifc/ifc-schema-specifications/

### 3.2 IDS information validation

IDS 1.0 provides machine-interpretable information requirements for IFC, including object applicability and alphanumeric requirements such as properties, quantities, classifications, materials and values. buildingSMART explicitly states that IDS is limited to alphanumeric information and does not cover geometric aspects.

Use IDS for information-delivery requirements. Pair it with independent geometry/topology checks, project-specific rules and physical acceptance methods.

References:
- https://www.buildingsmart.org/standards/bsi-standards/information-delivery-specification-ids/
- https://github.com/buildingSMART/IDS

### 3.3 Building simulation software verification

ASHRAE Standard 140 specifies test procedures for evaluating the technical capabilities and ranges of applicability of building-performance simulation software. Its purpose includes diagnosing differences caused by algorithmic differences, model limitations, software defects, inadequate documentation or input errors. The 2023 edition adds a test suite for software interpretation of standard-format weather data.

For EnergyPlus 26.2.0 and the Symtropy #1900 adapter, identify which Standard 140 cases are applicable to the exact engine build and declared use. Record unsupported or inapplicable cases explicitly; do not convert them to passes.

A Standard 140 result is software-method verification. It does not establish that a project model is built correctly, that a weather file represents a parcel, that the building is safe, or that an engineer or authority has approved it.

References:
- https://www.ashrae.org/technical-resources/bookstore/standard-140-method-of-test-for-evaluating-building-performance-simulation-software
- https://www.ashrae.org/technical-resources/standards-and-guidelines/titles-purposes-and-scopes
- https://data.ashrae.org/standard140/

### 3.4 Thermal comfort and heat safety

Keep three questions distinct:

- Simulation implementation correctness (for example, Standard 140).
- Occupied-space thermal comfort assessment under the applicable method (for example, ASHRAE Standard 55, where applicable).
- Heat-health and life-safety requirements defined by the responsible local authority and qualified professionals.

A comfort model or comfort pass is not a substitute for a heat-health risk assessment, and neither is a substitute for legal/professional review.

Reference:
- https://www.ashrae.org/technical-resources/bookstore/standard-55-thermal-environmental-conditions-for-human-occupancy

## 4. Required verification manifest

Each validation run should be reproducible from a versioned manifest. At minimum bind:

- run ID, case ID, test family, declared purpose and claim ceiling;
- source model identity, revision and exact byte digest;
- IFC schema and IDS version, where applicable;
- geometry provider, geometry digest, units, coordinate frame, transform chain and tolerance policy;
- solver name/version/build digest, executable/runtime, configuration and adapter revision;
- weather file identity/digest, source, period, station/coordinates/elevation, timezone, units, missing-data checks and reuse rights;
- occupancy, schedules, internal loads, setpoints, boundary conditions and assumptions;
- Standard 140 edition, test ID, reference data, applicability decision and acceptance tolerance, where applicable;
- independent reviewer/verifier identity and build;
- raw output/log digest, normalized result digest and deterministic reason codes;
- result state: PASS, FAIL, UNKNOWN, NOT_COMPARABLE or NOT_APPLICABLE;
- dependencies and invalidation triggers;
- exact evidence tier and prohibited claim extensions.

A manifest records what was tested. It does not authenticate the truth of a source claim unless the source bytes and issuer authority have separately been verified by their responsible owners.

## 5. Differential test cases

The initial crosswalk should include at least these frozen controls:

| ID | Input condition | Required result |
|---|---|---|
| IFC-001 | Valid STEP syntax and declared IFC schema | IFC syntax/schema layer may pass only |
| IFC-002 | Duplicate or malformed STEP entity / invalid attribute cardinality | IFC validation fails; no downstream promotion |
| IDS-001 | All declared alphanumeric requirements satisfied | IDS layer may pass only |
| IDS-002 | Required property absent, wrong type or outside declared value restriction | IDS fails with exact requirement and object path |
| GEO-001 | IDS passes but geometry digest/coordinate frame differs from the qualified geometry input | Geometry is invalid or requires requalification; IDS pass is retained but irrelevant to that claim |
| GEO-002 | Geometry is valid but a required clearance, clash or site-boundary rule fails | Project geometry gate fails; IDS is not used as a substitute |
| GEO-003 | Unit or coordinate transform is missing, ambiguous or changed | Unknown/invalid; no assumed conversion |
| SIM-001 | Applicable Standard 140 case is within the declared tolerance | Software verification case passes for that build/test |
| SIM-002 | Applicable Standard 140 case exceeds the declared tolerance | Software verification fails; preserve raw output and discrepancy classification |
| SIM-003 | Standard 140 case is unsupported by the model type or solver capability | NOT_APPLICABLE / UNSUPPORTED, never PASS |
| WTH-001 | Standard-format weather input is malformed or interpreted with wrong units/time basis | Weather parsing case fails |
| WTH-002 | Weather file parses correctly but is not representative of the declared site/use/period | Software input parsing may pass; project applicability remains UNKNOWN/NOT_COMPARABLE |
| PROJ-001 | Model has stale geometry, BOM, occupancy, schedule or weather dependencies | Only dependent claims are marked for revalidation; no blanket pass from prior receipt |
| PHYS-001 | Accepted design is available but independent as-built metrology is absent | Physical conformance remains UNKNOWN |
| PHYS-002 | Panel metrology passes but building commissioning/service receipt is absent | Panel claim may pass; building-service claim remains UNKNOWN |
| OPS-001 | Commissioning passes but service continuity contains constrained/unknown intervals | Service is not declared stably restored |

Add mutation/property tests for shuffled object order, duplicate IDs, malformed digests, stale model revisions, source revocation, calibration expiry, unsupported schema versions, incomplete inputs and non-deterministic result ordering.

Positive controls are mandatory. A verifier that rejects every case is not a qualified verifier.

## 6. Discrepancy taxonomy

Do not reduce a disagreement to a single solver-agreement percentage. Assign one or more explicit causes:

1. Exchange syntax/schema defect.
2. Information requirement or mapping defect.
3. Geometry/topology/coordinate-frame discrepancy.
4. Unit, conversion or time-basis error.
5. Weather/input-data integrity or representativeness issue.
6. Model-form or algorithmic applicability difference.
7. Solver implementation defect or numerical/convergence issue.
8. Reference-data, test-condition or tolerance mismatch.
9. IFC/IDS transformation or information-loss defect.
10. Physical fabrication, installation or as-built deviation.
11. Operational/control/occupancy mismatch.
12. Insufficient evidence or unresolved authority.

Retain raw values, units, tolerances and applicability decisions so a reviewer can reproduce the classification.

## 7. Highveld / South African application boundary

The current Highveld design brief is synthetic. The candidate Johannesburg weather files are not automatically qualified local inputs.

Before a parcel-specific simulation claim:
- independently inspect and hash the selected weather file;
- establish source provenance, licensing, coordinates/elevation, time zone, station period, data quality and missingness;
- distinguish typical-year design exploration from heatwave, storm, hail, flood and future-climate stress cases;
- use a site-specific surveyed location and declared operating/occupancy assumptions;
- preserve local code, geotechnical/dolomite, fire, structural, accessibility, water and utility requirements under their competent owners.

Standard 140's weather-format tests may show that an engine correctly interprets a standard-format file; they do not show that the chosen file is the correct weather evidence for the project.

No Standard 140, IDS, IFC, or synthetic Highveld result substitutes for South African statutory review, parcel-specific investigation, professional sign-off or approval by a competent authority.

## 8. Benchmark measures

Report each layer independently:

- validation agreement by test family and applicability;
- seeded false-PASS / false-reject counts;
- unsupported/inapplicable/unknown rates;
- numerical deviation from applicable reference results, with units and tolerance source;
- deterministic replay and raw-output digest reproducibility;
- information/geometry round-trip loss;
- dependency invalidation precision and recall against an independent full-scan reference;
- human review time and manual reconciliation effort;
- solver calls, runtime and memory;
- issue discovery time and time to correct/requalify;
- downstream claims correctly blocked after upstream failure.

For any comparative claim, pin exact product edition/build, SKU/feature availability, configuration, licence/region, data inputs and human/compute effort. Compare against a human-led best-of-breed workflow as well as individual tools. An unavailable vendor test is Unassessed, not evidence of vendor incapability.

## 9. Integration plan

1. Keep #652 as the federation architecture and #655 as its strict-input E1 evaluator.
2. Register IFC/IDS and geometry-owner contracts separately; retain explicit projection-loss receipts.
3. Run applicable Standard 140 tests through the EnergyPlus #1900 owner; do not reproduce the solver in Sol-Atlas.
4. Add the verification manifest and discrepancy classes to the #617 matched AEC experiment.
5. Connect accepted physical outputs through #622 and owner-native provenance through Mycelix #5149.
6. Connect commissioning, continuity and in-use outcomes through #574, #633/#635 and #657.
7. Require a common declared scope through #623 before combining results.

## 10. Claim ceiling

This document defines a verification protocol and benchmark crosswalk only. It contains no executed Standard 140 result, no validated EnergyPlus adapter, no validated Highveld weather file, no IFC/IDS export result, no geometry acceptance, no professional approval, and no physical or occupied-habitat qualification. Those claims require their respective owners, exact evidence and independent review.
