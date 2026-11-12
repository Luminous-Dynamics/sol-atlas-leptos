# MUDDI Synthetic Fixture v0.1

**Owner:** Sol-Atlas #755  
**Programme:** Symtropy SUF #4910  
**Status:** non-geographic synthetic interchange fixture only

## Purpose

Provide a finite interchange object for testing the draft Sol-Atlas MUDDI alignment profile. It includes a synthetic utility network with power, fibre, water and stormwater examples, access geometry, observation references and a deliberately conflicting legacy record.

All coordinates use a named abstract local Cartesian frame. The fixture is not a real site, geographic map, surveyed utility record, owner-issued registry, or MUDDI conformance test suite.

## Important identity control

The current candidate power duct and its historical legacy representation intentionally share one ObjectID but have different RecordIDs and SystemIDs. This models two dataset records describing the same real-world object. The records must remain separate, with their respective time, uncertainty and provenance. The historical geometry is explicitly conflicting and is not silently promoted to current.

A duplicate RecordID is a hard failure within the same dataset snapshot. SystemID uniqueness is only meaningful within its provider namespace. Two records with the same ObjectID are not automatically duplicates; the owner identity and reconciliation relation must be preserved.

## Contents

- Examples of all ten MUDDI core classes.
- Synthetic power duct, fibre duct, potable-water main, stormwater interface, start/end nodes, network link and access chamber.
- Three observation examples: synthetic exposed-verification reference, GPR indication, and historical compiled legacy import.
- Four disclosure classes, with exact critical infrastructure geometry classified as restricted.
- A 12-case negative-control manifest.

## Files

- suf-muddi-synthetic-fixture-v0.1.json — finite synthetic model.
- suf-muddi-synthetic-fixture-v0.1.schema.json — draft structural schema.
- muddi-fixture-negative-cases-v0.1.json — expected control outcomes.

## Explicit limits

This fixture uses a simplified profile of MUDDI concepts, not the complete normative MUDDI logical model or abstract tests. The fixture's observations and coordinates are synthetic values solely to test representation. It does not implement host access control, signatures, custody validation, real GIS transformation, owner authentication, survey quality, or engineering truth.


## Initial structural review (2026-11-12)

Fetched the fixture, schema, negative-case list and this note back from the branch. JSON parsing and authored relationship/state checks passed:

- 14 records;
- all ten MUDDI core classes represented;
- three observations with feature-of-interest and event references resolved;
- 12 negative controls;
- one intentional duplicate ObjectID with separate RecordIDs and SystemIDs (the legacy and candidate representations of one object);
- every RecordID unique;
- every internal link and evidence reference resolved;
- availability state distinct from epistemic class and currentness/conflict state.

This is not full JSON Schema 2020-12 validation, MUDDI abstract-test execution, an implemented access-control service, a GIS interoperability round-trip, or field-data validation.
