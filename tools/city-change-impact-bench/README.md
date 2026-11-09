# CityMaker Change Impact Projection v0.1

Offline Rust reference adapter for [CITY-CHANGE-IMPACT-001 #643](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/643).

## Purpose

This package projects owner-qualified change-impact findings onto city-level claims. It answers which claims are:
- `applicable_current`;
- `revalidation_required`;
- `invalid_evidence`;
- `insufficient_evidence`;
- `not_affected`.

The adapter **does not calculate the canonical dependency graph, change semantics, trust state, revocation state, impact frontier or decision reliance**. Those remain with Symthaea #10543/#6117, Sol-Atlas #396/#475/#484/#501, and the other named owners. This package validates exact references and projects their qualified results without strengthening them.

## Why this is a different problem from a digital-twin refresh

A new source or design revision should not make every city claim stale. Nor may a claim remain current simply because no one refreshed the district model.

The important distinction is between:
- an authoritative owner change event;
- a complete dependency scope and graph snapshot;
- a qualified owner impact report;
- a claim-specific dependency path or coverage-qualified no-impact proof;
- an exact post-change revalidation receipt;
- the current city decision-reliance projection.

The report is not accepted from a status label alone. The adapter binds the report to the exact event, graph snapshot, scope and claim finding. A missing edge or excluded owner domain cannot produce `not_affected`.

## Input contract

The executable corpus is `CITY_CHANGE_IMPACT_CORPUS_V0_1.json`; its JSON Schema is `CITY_CHANGE_IMPACT_CORPUS_V0_1.schema.json`.

Every case binds:
- owner event identity, change class, source owner/revisions, authority state, effective time, recorded time and trusted receipt time;
- exact evaluation time, optional historical evaluation time, scope and functional unit;
- graph snapshot identity/digest, coverage state, excluded domains and dependency edges;
- impact report identity, event/scope binding, report state, graph/base digest, coverage receipt, query boundary and claim-level findings;
- claim identity, dependency subjects, scope and functional unit;
- optional calibration measurement time, historical/current view or immutable historical-record classification;
- optional revalidation receipts bound to the exact claim, post-change revision, event, report, graph, scope and trusted time.

Timestamps are synthetic UTC Unix seconds. Future-effective changes are permitted to be recorded and trusted before their effective time; record time must not be conflated with effect time.

## Projection rules

1. Unverified, unknown or revoked change authority never becomes a city invalidation pass. It yields `insufficient_evidence` or `invalid_evidence` according to the owner disposition.
2. `not_affected` requires a qualified report, complete coverage, no excluded domains, exact graph/event/scope binding and a non-empty coverage proof ID.
3. An affected claim requires the report's impacted set, a claim-specific dependency path and matching dependency edge.
4. An exact post-change revalidation receipt discharges only the claim it names. It must bind the event, report, graph, scope and new source revision, and be trusted by evaluation time.
5. Revalidation of a receipt does not automatically revalidate a dependent decision.
6. Calibration expiry is evaluated against the measurement's time, not just today's date.
7. A future-effective policy edition is not applied before its effective time.
8. A late correction preserves what was known at an earlier evaluation time while changing current reliance where applicable.
9. Revocation, graph cycles, wrong-base impact deltas and duplicate/out-of-order event sequences cannot be represented as a clean no-impact.
10. Historical records remain replayable; they are not overwritten by current applicability.

No result grants professional approval, permit, release, actuation or operational authority. `action_authority` is always `none`.

## Standards and industry crosswalk

The architecture composes existing standards and systems rather than claiming to invent federated digital twins or traceability.

- **ISO 23247-6:2026** defines integrated, unified and federated digital-twin composition for manufacturing. CityMaker should document which owner boundaries use each composition style and test consequences for consistency, latency and coverage. Applying this manufacturing framework to buildings/cities is an adaptation, not an ISO-conformance claim: https://www.iso.org/standard/87426.html
- **NIST IR 8536 (final, September 2026)** provides an industry-neutral manufacturing traceability meta-framework built around linked, temporally ordered provenance and selective disclosure across supply chains. Use its patterns for owner-native product/lot and supply hand-offs; do not duplicate the meta-framework: https://csrc.nist.gov/pubs/ir/8536/final
- **IETF RFC 9943 / SCITT** provides signed-statement transparency and receipts. A transparency receipt establishes registration/inclusion of a signed statement, not engineering correctness, source completeness, applicability or action authority: https://www.rfc-editor.org/rfc/rfc9943
- **IFC 4.3 / IDS 1.0** provide openBIM exchange and machine-readable alphanumeric information requirements. IDS does not replace geometric/as-built verification; geometry and coordinate-frame changes need their own qualified owners: https://www.buildingsmart.org/standards/bsi-standards/industry-foundation-classes/ and https://www.buildingsmart.org/Standards/Bsi-Standards/Information-Delivery-Specification-Ids/
- **ISO 37123:2019** is a city-resilience outcome-indicator reference. Indicators do not prove that a particular asset, intervention or engineering decision remains applicable: https://www.iso.org/standard/70428.html

Current industry baselines should include:
- full graph recomputation after each change;
- the owner-native incremental impact frontier, where available;
- conventional BIM/PLM/CDE version and change workflows where equivalent dependency data can be exported;
- the CityMaker owner-composed projection.

Compare only on equivalent source exports, graph coverage, change sets, evaluation scope, build versions and evidence rights. Inaccessible relationships are `unassessed`, not absent.

## Metrics and targets

E1 synthetic targets:
- 100% agreement with the 28 frozen known-answer cases;
- zero missed impacted claims on the declared dependency graph;
- zero false `not_affected` on incomplete or excluded coverage;
- zero acceptance of wrong-base deltas, cycles or duplicate/out-of-order events;
- exact post-change receipt discharges only its own claim;
- zero action authority and zero causal claims.

For E2–E5 report separately:
- affected-claim precision, recall and exact-set agreement;
- false-negative impact rate and false-positive invalidation rate;
- path/explanation completeness and missing-edge detection;
- coverage-qualified `not_affected` rate;
- current-reliance false acceptance;
- revalidation obligation creation and exact discharge rates;
- historical replay correctness for late-arriving corrections;
- full recomputation cost versus selective revalidation cost;
- p50/p95/p99 latency, graph construction/validation cost, solver runs, human review and total rework burden;
- Unknown/abstention rate by authority, time, source bytes, coverage, scope and unsupported change class.

Do not optimize a single scalar. Missed revalidation and false no-impact are hard failures. A system that rejects everything is not useful: preserve positive, unaffected and exact-revalidation controls.

## Reproduction

From the repository root:

```sh
cargo fmt --manifest-path tools/city-change-impact-bench/Cargo.toml --all -- --check
cargo test --locked --manifest-path tools/city-change-impact-bench/Cargo.toml
cargo clippy --locked --manifest-path tools/city-change-impact-bench/Cargo.toml --all-targets -- -D warnings
cargo run --locked --quiet --manifest-path tools/city-change-impact-bench/Cargo.toml -- \
  docs/research/city-change-impact/CITY_CHANGE_IMPACT_CORPUS_V0_1.json \
  --source-commit <40-hex-commit>
```

## Claim ceiling

A passing result demonstrates only the adapter's owner-report binding and projection semantics on synthetic fixtures. It does not prove that the real dependency graph is complete, that a real event is authentic, that a correction is true, that an engineering claim is correct, or that a person may approve or execute a change. E1 synthetic results are not a field reliability estimate.
