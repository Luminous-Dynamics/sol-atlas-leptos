# Methane Action Contract

A small Rust benchmark contract, census scorer, and paired comparison toolkit—not a methane detector or physical model.

## Why this exists

One satellite overpass may produce a plume observation; multiple plume observations may map to one source; and a source may or may not be attributable to a specific facility. The contract keeps these identities distinct and represents source/facility matches as separate, reviewable associations.

The data model distinguishes:
- explicit gas-species and evidence-kind gates so methane and CO2 rates, observed plumes, inventory estimates, model inferences, and synthetic scenarios cannot be silently mixed;
- sensor/platform and processing-method identities are mandatory for remote-sensing and in-situ measurements, enabling stratified comparison;
- quantified emission-rate estimates, including uncertainty and a detection limit;
- non-detections, which are **not zero-emission observations**;
- explicitly unquantified observations;
- remote-sensing plume records, in-situ measurements, inventory estimates, model inferences, and synthetic scenarios;
- source associations and optional facility associations;
- terms approval bound to the exact source URI/version/digest and exact terms URI/digest;
- source, terms, and calibration digests recomputed from caller-supplied exact bytes during admission, with missing, duplicate, empty, and mismatched snapshots rejected;
- in-situ measurements require an identified instrument, method/version, and calibration snapshot whose validity interval contains the observation timestamp and whose instrument ID matches;
- empty observation batches are rejected rather than vacuously passing.

The whole batch must be rejected if any validation violation is returned. The validator does not silently drop or repair records.

## JSON census CLI

The `methane-census` binary reads one strict JSON request from stdin and writes one JSON receipt to stdout. Malformed JSON/schema input exits nonzero without a receipt; structurally invalid benchmark submissions produce a receipt with `evaluation_status: "invalid"` and exit code 2; valid submissions exit 0.

Example using a **synthetic smoke fixture only**:

```sh
cargo run --manifest-path tools/methane-action-contract/Cargo.toml --locked --bin methane-census \
  < tools/methane-action-contract/examples/synthetic-census.json > receipt.json
```

The receipt includes SHA-256 digests for the exact input JSON bytes and running evaluator executable, plus submitted system/configuration identifiers. These hashes identify the inputs and executable image; they do not authenticate the input source, prove who built the binary, authenticate the upstream provider or reviewer, or establish reference truth. The bundled two-case fixture is intentionally synthetic and is only a CLI/conformance smoke test, not evidence of methane-detector performance.

Input schema version 1 uses explicit tagged states for reference truth, coverage, and system disposition. Unknown object fields are rejected rather than silently ignored.

## Leakage-resistant split manifest

The `split` module validates a separately versioned `SplitManifest` against the **exact corpus JSON bytes**. It recomputes the corpus digest, requires a complete one-to-one case assignment, and rejects cross-partition leakage through repeated cluster IDs, source-group IDs, or identical case-artifact digests. Train, validation, and test partitions must all be populated; a blind test partition is optional.

The `methane-split-check` binary accepts two files, writes a JSON validation receipt, and exits 2 for a structurally invalid split. Example using a **synthetic-only corpus**:

```sh
cargo run --manifest-path tools/methane-action-contract/Cargo.toml --locked --bin methane-split-check -- \
  tools/methane-action-contract/examples/split-corpus.json \
  tools/methane-action-contract/examples/split-manifest.json
```

The corpus contains five made-up cases across four named partitions. Its manifest is pinned to the exact bytes of `split-corpus.json`. The receipt includes corpus, split-manifest, and evaluator-executable digests. This checks internal consistency; it does not establish that clusters are independent, labels are accurate, or a holdout represents future deployment conditions.

## Paired baseline comparison

The `comparison` module compares a candidate and baseline on the **same complete case universe**. Each case now carries a required `cluster_id` (for example, site × overpass/time window) in addition to its analysis `stratum_id`. Do not use a stratum label as a substitute for a sampling cluster unless that is the true independence unit.

The comparator reports candidate-minus-baseline differences for end-to-end recall and covered negative-case false-alert rate, discordant event counts, and deterministic cluster-bootstrap intervals. It resamples whole clusters instead of individual rows to preserve within-cluster dependence. For this implementation, intervals are withheld unless there are at least 10 distinct clusters and 1,000 requested replicates. These are conservative implementation gates, not universal guarantees of statistical adequacy; study design, cluster definition, and independent holdout selection remain decisive. If clusters share sites, overpasses, or time windows, define the cluster at the highest relevant dependence level.

Use the `methane-compare` binary to generate a paired comparison receipt from a strict JSON request. The input includes `schema_version`, `cases`, `baseline`, `candidate`, `bootstrap_replicates`, and `seed`. Example using **synthetic test data only**:

```sh
cargo run --manifest-path tools/methane-action-contract/Cargo.toml --locked --bin methane-compare \
  < tools/methane-action-contract/examples/synthetic-paired-comparison.json > comparison-receipt.json
```

The receipt binds the exact input JSON bytes and running evaluator executable by SHA-256, and includes cluster counts, point estimates, discordant pairs, and bootstrap intervals. Binary hashing identifies the executable bytes but does not prove who built them, establish reference truth, or show that the candidate generalizes beyond the tested corpus.

## Complete-census evaluator

The exported `census` module scores a submitted detector against the complete predeclared opportunity set, separately from input admission. Every expected case must have exactly one response row. Missing, extra, or duplicate rows suppress headline metrics. A positive event that was not tasked or fell on an invalid scene remains a miss for end-to-end recall; it is not silently omitted. No-coverage and reference-unavailable cases are reported separately and are not true negatives.

The evaluator reports end-to-end recall, recall conditional on coverage, false-alert rate on covered zero-release cases, coverage rate, abstention rate, per-stratum counts, and positive-detection log-rate error. It also exposes two-sided 95% Wilson score intervals for rates, returning no interval when a denominator is zero. These are per-system descriptive intervals; fair comparisons on shared sites/scenes require paired or cluster-aware analysis. It rejects detected-rate records with non-finite or non-positive values. This is a deterministic scoring reference; it does not authenticate the submitting system, run a detector, establish hidden/blind evaluation, or validate the ground-truth labels.

## Run locally

    cargo test --manifest-path tools/methane-action-contract/Cargo.toml
    cargo fmt --manifest-path tools/methane-action-contract/Cargo.toml -- --check
    cargo clippy --manifest-path tools/methane-action-contract/Cargo.toml --all-targets -- -D warnings

External crates are `sha2 = 0.10.9`, `serde`, and `serde_json`, with a standalone lockfile pinned to versions already present in the repository lockfile.

## Trust and scope boundaries

The validator recomputes canonical lowercase SHA-256 for exact source, terms, and (when provided or required) calibration bytes supplied as `ArtifactSnapshot` records and checks those results against the corresponding declared digests. It rejects missing, duplicated, empty, or mismatched snapshots. In-situ observations must be linked to an instrument and a calibration window covering the observation. The validator does not:
- authenticate the upstream provider, prove that the supplied bytes were retrieved from the URI, or authenticate a named reviewer;
- interpret the actual legal scope of a license or determine whether a review is legally sufficient;
- verify instrument calibration, infer unseen events, establish source attribution, or validate fluid/atmospheric physics;
- establish detector precision, recall, emissions reduction, operational safety, regulatory compliance, or state-of-the-art performance.

A source/terms review is a recorded prerequisite, not legal advice or proof of reviewer identity. Adapters must fetch and preserve the actual bytes and relevant access metadata; the validator then recomputes their digests. Synthetic fixtures prove only contract behavior, not field truth.

All 66 Rust tests use invented fixtures only; they are not real observations, detector-performance results, calibration evidence, or field measurement validation.

## Post-intervention mitigation assessment

The `mitigation` module is a separate evidence gate for before/after methane-rate comparisons. It requires a submitter-declared verified intervention, exact-byte work-order and verification-report snapshots, reviewed associations from the same source, one baseline reading before completion, and repeated in-situ methane follow-ups after completion. By default, all readings must use the same instrument and processing method; calibration must be valid at each reading time. Follow-ups must meet the policy's minimum count and time span.

When all post-intervention readings are quantified, the report gives the median follow-up rate and a baseline-minus-median rate difference. It also reports a conservative interval built from the declared symmetric **absolute uncertainty bounds**. Those bounds are not assumed to be Gaussian or 1-sigma. A non-detect follow-up keeps its detection limit as an upper bound and is never coerced to zero; in that case, the point reduction estimate is omitted. The report marks reduction supported by the sampled measurements only when the conservative lower bound exceeds zero across all follow-ups.

This does **not** calculate total tonnes avoided, establish causal attribution, authenticate a field verifier, or prove that a repair remains effective outside the sampled windows. Do not annualize a rate reduction without a separate duration/activity model, temporal variability, and uncertainty propagation. The 10 module tests use entirely synthetic source, instrument, work-order and verification fixtures.

## Source adapters

No source data are downloaded or redistributed by this crate. Candidate providers include UNEP's MARS plume/source exports and Carbon Mapper's data API. The current UNEP MARS dictionary says Sentinel-2, Landsat-8/9, EMIT, PRISMA and EnMAP plumes were requantified after 20 February 2026, and the export is CC BY-NC-SA 4.0 / non-commercial. Carbon Mapper warns that its API may contain uncalibrated Tanager first-light data. Pin retrieval/product versions, keep pre/post-requantification data distinct, preserve provider quality/calibration state, and verify current terms before use. Do not use MARS-derived assets in a commercial product unless separately established rights permit it.

- [UNEP MARS plume/source data dictionary](https://methanedata.unep.org/dict-mars-plumes)
- [Carbon Mapper API documentation](https://api.carbonmapper.org/api/v1/docs)
- [IEA Global Methane Tracker 2026](https://www.iea.org/reports/global-methane-tracker-2026/key-findings)
