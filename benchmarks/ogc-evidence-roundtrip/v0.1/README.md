# OGC evidence round-trip benchmark v0.1

This directory contains a synthetic, implementation-neutral fixture corpus for the draft Sol-Atlas evidence-preserving exchange profile.

- `fixtures/feature-collection.json`: JSON-FG 1.0.0 root FeatureCollection with one synthetic feature, typed claim, three disjoint evidence associations, separate temporal roles, uncertainty, provenance and competing hypotheses.
- `fixtures/evidence-records.json`: three synthetic evidence records. The contradictory record exposes only a public reference and metadata; content retrieval, disclosure, redistribution and model training are prohibited or pending.
- `schemas/feature-extension.schema.json`: JSON Schema for the per-feature Sol-Atlas extension.
- `schemas/evidence-records.schema.json`: JSON Schema for evidence metadata and rights dimensions; prohibited content is not embedded.
- `cases.json`: expected baseline invariants and 14 registered negative controls.

## Scope

These are benchmark fixtures, not a deployed OGC API endpoint, not a certified OGC conformance suite, and not proof that any GIS application preserves the extension fields. The fixture is intended to make information loss testable before adapters are implemented.

## Rust reference validator

A dependency-light Rust validator is provided in `reference/`. Run `cargo test --manifest-path benchmarks/ogc-evidence-roundtrip/v0.1/reference/Cargo.toml` and `cargo run --manifest-path benchmarks/ogc-evidence-roundtrip/v0.1/reference/Cargo.toml -- fixtures/feature-collection.json fixtures/evidence-records.json` from this directory. It checks cross-file associations, relation agreement, rights/content consistency, temporal-role presence, provenance-to-association binding and retained hypotheses. It emits a canonical semantic projection; it does not yet compute a cryptographic digest or replace JSON Schema / official OGC conformance validation.

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
