# Methane Action Contract

A small, dependency-free Rust contract for **benchmark input admission**, not a methane detector.

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

## Complete-census evaluator

The exported `census` module scores a submitted detector against the complete predeclared opportunity set, separately from input admission. Every expected case must have exactly one response row. Missing, extra, or duplicate rows suppress headline metrics. A positive event that was not tasked or fell on an invalid scene remains a miss for end-to-end recall; it is not silently omitted. No-coverage and reference-unavailable cases are reported separately and are not true negatives.

The evaluator reports end-to-end recall, recall conditional on coverage, false-alert rate on covered zero-release cases, coverage rate, abstention rate, per-stratum counts, and positive-detection log-rate error. It rejects detected-rate records with non-finite or non-positive values. This is a deterministic scoring reference; it does not authenticate the submitting system, run a detector, establish hidden/blind evaluation, or validate the ground-truth labels.

## Run locally

    cargo test --manifest-path tools/methane-action-contract/Cargo.toml
    cargo fmt --manifest-path tools/methane-action-contract/Cargo.toml -- --check
    cargo clippy --manifest-path tools/methane-action-contract/Cargo.toml --all-targets -- -D warnings

The only external crate dependency is `sha2 = 0.10.9`, with a standalone lockfile pinned to the same versions already present in the repository lockfile.

## Trust and scope boundaries

The validator recomputes canonical lowercase SHA-256 for exact source, terms, and (when provided or required) calibration bytes supplied as `ArtifactSnapshot` records and checks those results against the corresponding declared digests. It rejects missing, duplicated, empty, or mismatched snapshots. In-situ observations must be linked to an instrument and a calibration window covering the observation. The validator does not:
- authenticate the upstream provider, prove that the supplied bytes were retrieved from the URI, or authenticate a named reviewer;
- interpret the actual legal scope of a license or determine whether a review is legally sufficient;
- verify instrument calibration, infer unseen events, establish source attribution, or validate fluid/atmospheric physics;
- establish detector precision, recall, emissions reduction, operational safety, regulatory compliance, or state-of-the-art performance.

A source/terms review is a recorded prerequisite, not legal advice or proof of reviewer identity. Adapters must fetch and preserve the actual bytes and relevant access metadata; the validator then recomputes their digests. Synthetic fixtures prove only contract behavior, not field truth.

The tests use invented fixtures only; they are not real observations or measurement-validation evidence.

## Source adapters

No source data are downloaded or redistributed by this crate. Candidate providers include UNEP's MARS plume/source exports and Carbon Mapper's data API. Check current access terms before use: UNEP's plume/source export is CC BY-NC-SA 4.0 and non-commercial, while Carbon Mapper's API warns about usage restrictions and possible uncalibrated Tanager first-light data.

- [UNEP MARS plume/source data dictionary](https://methanedata.unep.org/dict-mars-plumes)
- [Carbon Mapper API documentation](https://api.carbonmapper.org/api/v1/docs)
- [IEA Global Methane Tracker 2026](https://www.iea.org/reports/global-methane-tracker-2026/key-findings)
