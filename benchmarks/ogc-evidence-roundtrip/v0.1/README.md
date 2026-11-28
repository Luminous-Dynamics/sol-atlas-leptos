# OGC evidence round-trip benchmark v0.1

This directory contains a synthetic, implementation-neutral fixture corpus for the draft Sol-Atlas evidence-preserving exchange profile.

- `fixtures/feature-collection.json`: JSON-FG 1.0.0 root FeatureCollection with one synthetic feature, typed claim, three disjoint evidence associations, separate temporal roles, uncertainty, provenance and competing hypotheses.
- `fixtures/evidence-records.json`: three public/synthetic evidence records plus one unreferenced restricted record. The contradictory record exposes only a public reference and metadata; content retrieval, disclosure, redistribution and model training are prohibited or pending.
- `fixtures/field-disposition-ledger.json`: explicit registration-only ledger for the expected adapter semantics. Its `not_executed` state and nine `not_assessed` entries are deliberate; this is not evidence that an adapter round-trip has run.
- `schemas/feature-extension.schema.json`: JSON Schema for the per-feature Sol-Atlas extension.
- `schemas/evidence-records.schema.json`: JSON Schema for evidence metadata and rights dimensions; content is embedded only when both retrieval and public disclosure are explicitly allowed; pending or unknown rights fail closed.
- `schemas/field-disposition-ledger.schema.json`: machine-readable ledger schema with conditional rules that prevent a route being marked completed while required evidence is absent or fields remain unassessed.\n- `properties.solAtlas.fieldDependencies`: required source-record lineage for each public semantic field. The Rust validator withholds any derived field whose dependencies include records that are not explicitly visible.
- `cases.json`: expected baseline invariants and 21 registered negative controls.
- `adapter-readiness.json`: reviewed candidate/validator matrix; every adapter route is explicitly marked not executed until real adapter tests run.

See also [`SOL-ATLAS-INDUSTRY-SOTA-AND-EVALUATION-UPDATE-2026-11.md`](../../../docs/research/SOL-ATLAS-INDUSTRY-SOTA-AND-EVALUATION-UPDATE-2026-11.md) for current standards, industry baselines, benchmark tracks and falsifiable research hypotheses.

## Scope

These are benchmark fixtures, not a deployed OGC API endpoint, not a certified OGC conformance suite, and not proof that any GIS application preserves the extension fields. The fixture is intended to make information loss testable before adapters are implemented.

## Rust reference validator

A dependency-light Rust validator is provided in `reference/`. Run `cargo test --manifest-path benchmarks/ogc-evidence-roundtrip/v0.1/reference/Cargo.toml` and `cargo run --manifest-path benchmarks/ogc-evidence-roundtrip/v0.1/reference/Cargo.toml -- fixtures/feature-collection.json fixtures/evidence-records.json` from this directory. It checks cross-file associations, relation agreement, rights/content consistency, temporal-role presence, provenance-to-association binding, retained hypotheses, deterministic ordering and restricted-record non-interference. Its public provenance projection is allowlist-based and deliberately omits the private output digest. It does not yet compute source-payload cryptographic digests or replace official OGC conformance validation.

## Machine-readable field disposition

Every adapter path must account for each field as preserved, transformed, omitted, blocked or not assessed. A route with `executionStatus: not_executed` is only a planned/registered route. A route cannot be marked completed until its source and target artifact digests are present and every field has a terminal disposition. The schema check includes negative controls for false completion, missing reasons, missing/duplicate required field paths, and payload embedding while retrieval or disclosure permission is pending/unknown.

## Initial validation

A complete adapter qualification must:

1. validate JSON syntax and the three profile schemas;
2. run the official JSON-FG 1.0.0 normative schema and relevant Annex A abstract tests;
3. resolve every association to exactly one record;
4. check relation labels, set disjointness, digest binding and rights non-escalation;
5. verify source and provenance digests against frozen bytes;
6. enforce restricted-content omission and restricted linked-metadata and derived-field non-interference;
7. canonicalize the public semantic projection and compare it after each adapter hop;
8. account for every field in the disposition ledger;
9. apply all negative controls and assert rejection / quarantine with a reason code.

Do not treat JSON parse success as profile conformance. Do not compare only rendered maps, geometry overlap or feature counts.

## Independent JSON-FG validation

CI invokes the Geonovum OGC Checker CLI at the pinned package version `@geonovum/ogc-checker@1.3.2` against the synthetic JSON-FG 1.0.0 feature collection. A negative control removes the required Core conformance declaration and requires the independent checker to reject it. This is a document-level conformance check, not full certification. The checker documentation marks `/req/core/valid-geometry` untested, `/req/core/axis-order` not testable, and `/req/types-schemas/feature-schemas` untested because referenced schemas are not dereferenced. These gaps remain explicit in `adapter-readiness.json`.

## Official JSON-FG conformance reference

The fixture declares JSON-FG Core and Feature Types and Schemas conformance. The normative JSON-FG 1.0.0 specification was published on 2026-04-30. Qualification must use the relevant abstract tests in Annex A and normative schemas in Annex B; checking only the `conformsTo` string is not conformance.

- Standard: https://docs.ogc.org/is/21-045r1/21-045r1.html
- OGC JSON-FG standard page and official model/schema resources: https://www.ogc.org/standards/json-fg/
- JSON-FG Core conformance URI: `http://www.opengis.net/spec/json-fg-1/1.0/conf/core`
- Feature Types and Schemas conformance URI: `http://www.opengis.net/spec/json-fg-1/1.0/conf/types-schemas`
