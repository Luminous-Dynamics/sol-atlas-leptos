# IEAC runtime-admission benchmark v0.1

**Status:** Executable benchmark-driver tranche; no benchmark has been run or passed by this change.  
**Depends on:** [IBAP run-manifest contract, PR #339](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/339), [runtime admission, PR #346](https://github.com/Luminous-Dynamics/sol-atlas-leptos/pull/346), [benchmark protocol, issue #348](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/348).

## 1. What this measures

The driver evaluates the runtime admission gate over deterministic, seeded mutation families derived from the synthetic profile/request fixtures. It stratifies by the single vertical declared by the supplied frozen IBAP run manifest and reports each failure family independently. Run one frozen manifest per vertical; do not pool different vertical estimands into one score.

The baseline ladder is deliberately narrow:

- **Static-only profile baseline:** checks the static profile contract but does not inspect request freshness, current authority, snapshot identity or action scope. It is an intentionally weaker capability baseline, not a production competitor.
- **Runtime candidate:** evaluate_admission.
- **Reconstruction checker:** independent_check, evaluated separately against the candidate report.

The candidate and checker currently share some low-level parsing, schema and digest primitives. Agreement is implementation-path evidence, not proof of fully independent source truth or a cryptographically independent verifier. The checker-independence limitation remains explicit.

## 2. Run protocol

First validate the supplied manifest against the IBAP Draft 2020-12 schema using the pinned jsonschema-cli version from the IBAP workflow. It must be a pre-registered synthetic_fixture manifest with one candidate and at least one baseline. The driver hashes the exact manifest bytes and binds that digest into its report. The manifest must be frozen before tuning or execution; edits require a new manifest and new run.

Example, from the repository root after the IBAP schema/example branch is available:

~~~bash
cargo run --release \
  --manifest-path tools/industry-runtime-admission/Cargo.toml \
  --locked --bin industry-admission-benchmark -- \
  --manifest schemas/examples/ieac-runtime-admission-benchmark-manifest-synthetic-v0.1.json \
  --trials-per-family 100 \
  --seed 0x20261031 \
  --output "$RUNNER_TEMP/ieac-admission-benchmark-report.json"
~~~

The output path is create-new: the driver refuses to overwrite a prior receipt. Keep the frozen manifest, generated report and exact executable/build identity together. This first driver reports the exact manifest-byte digest, method IDs, evaluator/checker versions and seed; it does not yet validate a binary/source digest against a signed build attestation.

The default 100 trials per family is a smoke/evaluation-development setting, not adequate for a tight statistical claim. For zero observed false admissions, the exact one-sided 95% binomial upper bound is:

p_U = 1 - 0.05^(1/n)

At 30,000 negative trials per failure family, the separate one-sided 95% upper bound is approximately 10^-4. The vertical-level equal-weight family macro-average uses Bonferroni-adjusted simultaneous exact bounds; it is reported separately because the family-stratified trials are not one IID pooled sample. These are bounds for the declared synthetic generation process and its sampling assumptions, not estimates of industrial or field risk. If failures are observed, the driver computes the one-sided exact binomial upper bound for the observed count. Publish each stratum's denominator and bound; never hide a weak group in an aggregate.

## 3. Frozen families and expected outcomes

| Family | Expected gate result | Protected reason code |
|---|---|---|
| Positive control | admit_descriptive | none |
| Stale measurement | abstain | MEASUREMENT_STALE |
| Future measurement | reject | FUTURE_DATED_MEASUREMENT |
| Missing measurement | abstain | CONDITION_MEASUREMENT_MISSING |
| Duplicate measurement | reject | DUPLICATE_CONDITION_MEASUREMENT |
| Uncertainty interval crosses envelope | abstain | MEASUREMENT_INTERVAL_CROSSES_ENVELOPE |
| Out-of-range value | reject | VALUE_OUTSIDE_OPERATING_ENVELOPE |
| Unit mismatch | reject | UNIT_MISMATCH |
| Model identity drift | reject | RUNTIME_IDENTITY_BINDING_INVALID |
| Profile expiry | reject | PROFILE_OUTSIDE_VALIDITY_WINDOW |
| Revoked profile pin | reject | PROFILE_PIN_REVOKED |
| Revoked measurement source | reject | ARTIFACT_REVOKED |
| Unavailable authority source | abstain | REVOCATION_STATE_UNAVAILABLE |
| Decision-tier escalation | reject | DECISION_TIER_NOT_ADMITTED |

The driver also mutates every generated report and checks that the reconstruction checker detects the changed disposition. Three repeated evaluations per family are compared byte-for-byte for deterministic replay. Those replay checks are smoke checks, not exhaustive proof of determinism.

The four supported synthetic strata are telecommunications, water_food_energy, industrial_maintenance and buildings. They are dimensionally distinct contract fixtures, not cross-vertical physical benchmarks. The manifest vertical determines which one is evaluated in a run.

## 4. Metrics emitted

Per vertical and failure family: sample count, expected disposition agreement, false admissions, false rejections on positive controls, one-sided exact 95% upper bound, reason-code hit count, candidate/checker disagreements, report-tamper detection, and p50/p95/p99 evaluator-plus-checker latency.

Overall: elapsed wall time, throughput, false-admission total, checker disagreement total, deterministic replay mismatches, and report-tamper results. Peak memory and CPU energy are explicitly not_measured in v0.1; no fabricated resource metrics are emitted. A subsequent tranche should add OS-specific resource collectors with calibration and failure semantics, not silently substitute process wall time for CPU energy.

## 5. Acceptance gates

- Zero false admissions across the frozen negative corpus.
- Zero candidate/reconstruction-checker disagreements.
- Every report mutation is detected.
- Zero deterministic replay mismatches in the declared replay sample.
- 100% positive controls produce the frozen expected admission.
- Every failure family reports its own denominator, disposition, reason-code coverage and latency distribution.
- Separate one-sided exact confidence bounds are reported per family; a Bonferroni simultaneous upper bound is reported for the equal-weight family macro-average within each vertical.
- No aggregate score may compensate for a false admission, an incorrect reason code, a failed positive control or a failed checker.
- A manifest or corpus change creates a new pre-registration and new run receipt.

A run that fails any acceptance gate remains a recorded failure. Do not rerun with different inputs under the same manifest and replace its receipt.

## 6. Limitations and next steps

The first driver is synthetic and uses the current v0.1 admission contract. It does not authenticate profile pins or revocation authority snapshots; that remains [IEAC-TRUST-001 / #347](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/347). It does not validate measurements, model credibility, physical constraints, real operations or production safety. It does not demonstrate superiority to Siemens, AWS, Palantir, Bentley, specialist simulators or any SOTA benchmark.

Next benchmark increments:
1. Integrate signed pin/revocation and anti-replay families after #347, while keeping trust failures separate from measurement failures.
2. Add source-level independent checker implementation and third-party golden vectors.
3. Add adversarial combinatorial interactions (e.g. stale plus revoked, identity drift plus validly refreshed snapshot) rather than relying only on single-fault mutants.
4. Add resource collectors, calibrated energy measurement and pinned hardware profiles.
5. Add prospective shadow-pilot outcomes only after synthetic software admission tests are stable and source/authority rights are established.

A benchmark result advances only the evidence dimension it directly measures.
