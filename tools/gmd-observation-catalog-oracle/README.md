# GMD observation catalogue rights oracle

A dependency-free Rust reference qualifier for source catalogues, station metadata and specific dataset products.

## Why this exists

The South African GMD pilot should use existing regional scientific infrastructure. SANSA's SANDIMS is a documented repository for geomagnetic, ionospheric and magnetospheric observations. Its metadata supports instrument/site/type/sampling searches and includes GPS location, manufacturer, principal investigator and sampling interval. Data are available through date-filtered bundles; SANSA's published terms distinguish free academic research access subject to registration from commercial use requiring an agreement.

Sources:
- SANDIMS: https://sandims.sansa.org.za/
- SANSA research/data access description: https://www.sansa.org.za/scientific-research-and-global-collaboration/
- SANSA instrument map: https://spaceweather.sansa.org.za/space-weather-information/resources/instrument-map/

The first catalogue record is **repository-level only**. It does not assert current station counts, specific station coordinates, current dataset availability, transformer-neutral GIC measurements, or any permission for commercial ingestion, model training or derivative redistribution.

## Record scopes

- `RepositoryCatalog`: documents a repository and its searchable metadata; cannot be treated as a station or data product.
- `StationInstrument`: a named instrument/site and its measurement metadata; missing fields remain explicitly incomplete.
- `DatasetProduct`: a specific product, processing level, time coverage and optional immutable artefact digest.

The Rust validator enforces scope/payload boundaries. A repository-level catalogue must not contain an invented station or dataset.

## Rights are purpose-specific

Every use purpose has a separate `RightGrant`:
- metadata discovery;
- dataset download;
- academic analysis;
- commercial ingestion;
- model training;
- derivative redistribution.

Every explicit right state other than `Unknown` requires its own source evidence reference. Permission, registration requirement, agreement requirement and prohibition must each be traceable. Validity bounds are assessed against a caller-supplied time, so results are deterministic. Calendar dates are checked for actual month/day validity, not only YYYY-MM-DD formatting. Download permission does not imply permission to train a model or redistribute derivatives. An academic research registration does not become a commercial right.

A repository or station record can support metadata discovery but cannot grant dataset use until a specific `DatasetProduct` record is attached.

## Metadata completeness and claim ceiling

`CompleteForExpertReview` means only that the listed metadata fields are populated. It is not a calibration PASS, physical sensor validation, spatial representativeness claim, GIC ground-truth qualification, or scientific validity verdict.

A dataset without a SHA-256 digest can remain a catalogue discovery record but is marked non-reproducible. A network-level instrument map does not prove a current station count or dataset coverage.

## Files

- Schema: `docs/research/gmd-observation-catalog/GMD_OBSERVATION_CATALOG_RECORD_V0.1.schema.json`
- SANSA repository record: `docs/research/gmd-observation-catalog/SANSA_SANDIMS_REPOSITORY_CATALOGUE_V0.1.json`
- Expected decisions: `docs/research/gmd-observation-catalog/SANSA_SANDIMS_EXPECTED_DISPOSITIONS_V0.1.json`

The schema defines the record shape and scope conditionals. Cross-field semantics, right-evidence requirements, date validity windows and metadata-completeness rules are enforced in Rust. CI's Python-standard-library preflight is a compact fixture check, not a complete JSON Schema Draft 2020-12 validator.

## Run

From the Sol-Atlas repository root:

    cargo fmt --manifest-path tools/gmd-observation-catalog-oracle/Cargo.toml -- --check
    cargo test --manifest-path tools/gmd-observation-catalog-oracle/Cargo.toml --locked --offline
    cargo clippy --manifest-path tools/gmd-observation-catalog-oracle/Cargo.toml --locked --offline --all-targets -- -D warnings

## Non-goals

No dataset downloader, credential storage, scraping of authenticated records, commercial legal determination, sensor calibration, physical model, live ingestion, model training or operational grid recommendation. Source-owner agreements and domain-expert review remain mandatory.
