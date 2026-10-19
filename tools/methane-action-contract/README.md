# Methane Action Contract

A small, dependency-free Rust contract for **benchmark input admission**, not a methane detector.

## Why this exists

One satellite overpass may produce a plume observation; multiple plume observations may map to one source; and a source may or may not be attributable to a specific facility. The contract keeps these identities distinct and represents source/facility matches as separate, reviewable associations.

The data model distinguishes:
- an explicit gas-species field and required-species gate so methane and CO2 rates cannot be mixed;
- quantified emission-rate estimates, including uncertainty and a detection limit;
- non-detections, which are **not zero-emission observations**;
- explicitly unquantified observations;
- remote-sensing plume records, in-situ measurements, inventory estimates, model inferences, and synthetic scenarios;
- source associations and optional facility associations;
- declared source/terms provenance and intended data-use review.

The whole batch must be rejected if any validation violation is returned. The validator does not silently drop or repair records.

## Run locally

    cargo test --manifest-path tools/methane-action-contract/Cargo.toml
    cargo fmt --manifest-path tools/methane-action-contract/Cargo.toml -- --check
    cargo clippy --manifest-path tools/methane-action-contract/Cargo.toml --all-targets -- -D warnings

No third-party crate dependencies are introduced.

## Trust and scope boundaries

This crate checks record structure and declarations only. In particular, it does not:
- compute SHA-256 or prove a digest matches the referenced bytes;
- authenticate a named reviewer or determine the actual legal scope of a license;
- verify measurement calibration, infer unseen events, estimate source attribution quality, or validate fluid/atmospheric physics;
- establish detector precision, recall, emissions reduction, operational safety, regulatory compliance, or state-of-the-art performance.

A source/terms review is a recorded prerequisite, not legal advice or proof of reviewer identity. Data adapters must retrieve and pin actual bytes, independently compute digests, inspect terms, and ensure the approved purpose covers the planned use.

The tests use invented fixtures only; they are not real observations or measurement-validation evidence.

## Source adapters

No source data are downloaded or redistributed by this crate. Candidate providers include UNEP's MARS plume/source exports and Carbon Mapper's data API. Check current access terms before use: UNEP's plume/source export is CC BY-NC-SA 4.0 and non-commercial, while Carbon Mapper's API warns about usage restrictions and possible uncalibrated Tanager first-light data.

- [UNEP MARS plume/source data dictionary](https://methanedata.unep.org/dict-mars-plumes)
- [Carbon Mapper API documentation](https://api.carbonmapper.org/api/v1/docs)
- [IEA Global Methane Tracker 2026](https://www.iea.org/reports/global-methane-tracker-2026/key-findings)
