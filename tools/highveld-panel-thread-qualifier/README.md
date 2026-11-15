# Highveld Panel Thread Qualifier v0.1

A standalone Rust evaluator for candidate submissions to the Highveld panel-thread composition benchmark.

## Scope

The evaluator grades a complete candidate submission against a supplied corpus. It checks:
- exact case coverage; no omitted or unknown cases;
- duplicate case IDs, reason codes or requalification domains;
- exact disposition agreement;
- reason-code and requalification-domain precision/recall;
- safety-floor underblocking for critical cases;
- stale/revoked evidence falsely treated as current;
- deterministic SHA-256 identities for the exact corpus and submission bytes.

It rejects unknown JSON fields and never turns a benchmark result into engineering approval, product release, robot authority, construction authorization or occupancy approval.

## Run

From the repository root:

```bash
cargo fmt --manifest-path tools/highveld-panel-thread-qualifier/Cargo.toml -- --check
cargo test --manifest-path tools/highveld-panel-thread-qualifier/Cargo.toml --locked
cargo clippy --manifest-path tools/highveld-panel-thread-qualifier/Cargo.toml --all-targets --locked -- -D warnings
cargo run --manifest-path tools/highveld-panel-thread-qualifier/Cargo.toml --locked -- \
  --cases docs/research/highveld-keep/HIGHVELD_KEEP_PANEL_THREAD_CORPUS_V0_1.json \
  --submission candidate-submission.json \
  --output evaluation-report.json
```

The submission must contain exactly one result for every corpus case. See `HIGHVELD_PANEL_THREAD_SUBMISSION_SCHEMA_V0_1.json`.

## Assurance-to-campaign bridge prototype

The library also exposes a benchmark-only bridge verifier for a normalized envelope. It verifies the relationship between:
- a Mycelix decision episode and its exact unresolved obligation IDs;
- a Mycelix acquisition candidate and its separate non-compensatory authority, rights, safety, schedule, physical-realizability, data-access and source-competence gates;
- a Symtropy option/campaign package and its exact selected action; and
- a bilaterally reviewed Mycelix-to-Symtropy acquisition-to-action mapping.

It is **not** a Mycelix or Symtropy production schema/adapter. All actual owner mappings remain UNBOUND until the owners publish versioned profiles and approve the mapping. The bridge never schedules or optimizes campaigns, creates reservations, recalculates Pareto frontiers, fetches/authenticates source bytes, verifies signatures, or converts a plan into observed or qualified evidence.

Input schema: `HIGHVELD_EVIDENCE_CAMPAIGN_BRIDGE_SCHEMA_V0_1.json`.  
Report schema: `HIGHVELD_EVIDENCE_CAMPAIGN_BRIDGE_REPORT_SCHEMA_V0_1.json`.

A positive synthetic end-to-end CLI example is included at `examples/highveld-bound-synthetic-bridge-example-v0.1.json`. It contains one remote source-review route and one physical observation route connected by a synthetic bilateral mapping. All owner commits, profiles, adapter IDs and review references in this file are **test-only placeholders**, not real Mycelix/Symtropy approvals or source artifacts.

Run it from the repository root:

```bash
cargo run --manifest-path tools/highveld-panel-thread-qualifier/Cargo.toml --locked --bin highveld-bridge-qualifier -- --input tools/highveld-panel-thread-qualifier/examples/highveld-bound-synthetic-bridge-example-v0.1.json --output /tmp/highveld-bridge-report.json
```

The expected semantics are two obligations with a candidate acquisition route, two still-unresolved proof obligations, zero qualified obligations, zero physical observations, no verified signatures/source bytes and no authority effects. The Symtropy campaign result is recorded as an owner-reported status, not an independently verified optimum.

The current-owner negative control is at `examples/highveld-current-owner-bindings-unbound-v0.1.json`; it demonstrates the actual status snapshot in which the real owner projections remain UNBOUND.
Positive results are only `PLANNED_NOT_OBSERVED`. The report keeps three counts separate: obligations with an admissible acquisition route, all unresolved proof obligations (which remain unresolved even when a route is planned), and obligations without an admissible route. A planned route never reduces the unresolved-proof count.

For physical campaigns the bridge records the exact status that Symtropy reports, including whether Symtropy reports its search as optimal. It does **not** independently verify the owner output or its optimality: `owner_outputs_authenticated` remains false. Source-byte verification and signature verification remain false; physical observations and qualified obligations remain zero; authority effects remain none.

## Important benchmark limitation

The checked-in v0.1 corpus status is `synthetic_specification_unadjudicated`. Expected dispositions are visible in the corpus and have not been independently adjudicated. The evaluator accepts only that unadjudicated status and **always reports a provisional status**, even when every provisional label matches. It has no adjudication-signature verifier and cannot issue a benchmark pass/fail qualification. This is not a blinded SOTA evaluation.

A later benchmark version should separate candidate-visible scenario packets from evaluator-only gold labels, independently adjudicate critical cases, preserve an explicit disagreement log, bind adjudication to the exact corpus digest, and use a hidden holdout before publishing performance claims.

## Exit and authority semantics

Malformed, incomplete, duplicated or unknown input is rejected with exit code 2. A valid evaluation writes a report with provisional status and hard-gate results. The evaluator has no network access, external process execution, source fetching, authority mutation or physical-control path.

## Claim ceiling

A score is agreement with the supplied synthetic benchmark labels only. It does not verify source artifact bytes, signer authenticity, material identity, real certificates, geometry, physics, robot safety, professional approval or physical performance.

## Exact-head validation checkpoint

The bridge and evaluator source at code head `d175f4d8bf9828c92f77b6847141a98573139120` passed:
- Focused Highveld qualifier workflow [#43766996384](https://github.com/Luminous-Dynamics/sol-atlas-leptos/actions/runs/43766996384): formatting, 49/49 library tests and strict Clippy.
- Repository Check [#43766996387](https://github.com/Luminous-Dynamics/sol-atlas-leptos/actions/runs/43766996387): formatting, 7/7 application tests, 49/49 qualifier library tests, strict Clippy and WASM check.

The 49 library tests comprise 16 corpus/evaluator tests and 33 bridge tests. The exact current owner-binding/readiness register is `docs/research/highveld-keep/HIGHVELD_OWNER_BINDING_READINESS_REGISTER_V0_1.md`.

A passing software suite validates only the declared synthetic fixtures and code path. It does not promote any real owner profile from UNBOUND, authenticate source artifacts, qualify physical observations, or grant engineering/operational authority.
