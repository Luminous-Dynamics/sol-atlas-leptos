# OGC evidence round-trip benchmark v0.1

This directory contains a synthetic, implementation-neutral fixture corpus for the draft Sol-Atlas evidence-preserving exchange profile.

- `fixtures/feature-collection.json`: JSON-FG 1.0.0 root FeatureCollection with one synthetic feature, typed claim, three disjoint evidence associations, separate temporal roles, uncertainty, provenance and competing hypotheses.
- `fixtures/evidence-records.json`: three synthetic evidence records. The contradictory record exposes only a public reference and metadata; content retrieval, disclosure, redistribution and model training are prohibited or pending.
- `cases.json`: expected baseline invariants and 14 registered negative controls.

## Scope

These are benchmark fixtures, not a deployed OGC API endpoint, not a certified OGC conformance suite, and not proof that any GIS application preserves the extension fields. The fixture is intended to make information loss testable before adapters are implemented.

## Initial validation

A future executable runner must:
1. parse and validate both JSON documents;
2. check JSON-FG root conformance declarations against the official JSON-FG 1.0.0 schema / abstract tests;
3. resolve every association to exactly one record;
4. check that relation labels match the association bucket and the sets are disjoint;
5. verify source and provenance digests against frozen bytes;
6. enforce rights non-escalation and restricted-content omission;
7. canonicalize a semantic projection and compare it after each adapter hop;
8. apply all negative controls and assert rejection / quarantine with a reason code.

Do not treat JSON parse success as profile conformance. Do not compare only rendered maps, geometry overlap or feature counts.
