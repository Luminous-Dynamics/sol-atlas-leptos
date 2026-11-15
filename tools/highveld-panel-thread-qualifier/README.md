# Highveld Panel Thread Qualifier v0.1

A standalone Rust evaluator for candidate submissions to the Highveld panel-thread composition benchmark.

## Scope

The evaluator grades a complete candidate submission against a supplied corpus. It checks:
- exact case coverage; no omitted or unknown cases;
- duplicate case IDs, reason codes or requalification domains;
- exact disposition agreement;
- reason-code and requalification-domain precision/recall;
- critical false-negative count;
- stale/revoked evidence falsely treated as current;
- deterministic SHA-256 identities for the exact corpus and submission bytes.

It rejects unknown JSON fields and never turns a benchmark result into engineering approval, product release, robot authority, construction authorization or occupancy approval.

## Run

From the repository root:

\`\`\`bash
cargo test -p highveld-panel-thread-qualifier --locked
cargo clippy -p highveld-panel-thread-qualifier --all-targets --locked -- -D warnings
cargo run -p highveld-panel-thread-qualifier --locked -- \\
  --cases docs/research/highveld-keep/HIGHVELD_KEEP_PANEL_THREAD_CORPUS_V0_1.json \\
  --submission candidate-submission.json \\
  --output evaluation-report.json
\`\`\`

The submission must contain exactly one result for every corpus case. See \`HIGHVELD_PANEL_THREAD_SUBMISSION_SCHEMA_V0_1.json\`.

## Important benchmark limitation

The checked-in v0.1 corpus status is \`synthetic_specification_unadjudicated\`. Expected dispositions are visible in the corpus and have not been independently adjudicated. The evaluator therefore **always reports \`PROVISIONAL_ONLY\`** for the present corpus, even when every provisional label matches. This is not a blinded SOTA evaluation and is not evidence that the candidate is independently qualified.

A later version should separate candidate-visible scenario packets from evaluator-only gold labels, independently adjudicate critical cases, preserve an explicit disagreement log, and use a hidden holdout before making benchmark performance claims.

## Exit and authority semantics

Malformed, incomplete, duplicated or unknown input is rejected with exit code 2. A valid evaluation writes a report; report status communicates whether labels are provisional or independently adjudicated. The evaluator has no network access, external process execution, source fetching, authority mutation or physical-control path.

## Claim ceiling

A score is agreement with the supplied synthetic benchmark labels only. It does not verify source artifact bytes, signer authenticity, material identity, real certificates, geometry, physics, robot safety, professional approval or physical performance.
