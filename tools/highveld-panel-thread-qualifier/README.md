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

Schema: `HIGHVELD_EVIDENCE_CAMPAIGN_BRIDGE_SCHEMA_V0_1.json`.

Run the bridge fixture input through the dedicated CLI:

```bash
cargo run --manifest-path tools/highveld-panel-thread-qualifier/Cargo.toml --locked --bin highveld-bridge-qualifier -- --input bridge-input.json --output bridge-report.json
```

Positive results are only `PLANNED_NOT_OBSERVED`; report fields deliberately keep source-byte verification, signature verification, physical observations, qualified obligations and authority effects at false/zero/none.

## Important benchmark limitation

The checked-in v0.1 corpus status is `synthetic_specification_unadjudicated`. Expected dispositions are visible in the corpus and have not been independently adjudicated. The evaluator accepts only that unadjudicated status and **always reports a provisional status**, even when every provisional label matches. It has no adjudication-signature verifier and cannot issue a benchmark pass/fail qualification. This is not a blinded SOTA evaluation.

A later benchmark version should separate candidate-visible scenario packets from evaluator-only gold labels, independently adjudicate critical cases, preserve an explicit disagreement log, bind adjudication to the exact corpus digest, and use a hidden holdout before publishing performance claims.

## Exit and authority semantics

Malformed, incomplete, duplicated or unknown input is rejected with exit code 2. A valid evaluation writes a report with provisional status and hard-gate results. The evaluator has no network access, external process execution, source fetching, authority mutation or physical-control path.

## Claim ceiling

A score is agreement with the supplied synthetic benchmark labels only. It does not verify source artifact bytes, signer authenticity, material identity, real certificates, geometry, physics, robot safety, professional approval or physical performance.
