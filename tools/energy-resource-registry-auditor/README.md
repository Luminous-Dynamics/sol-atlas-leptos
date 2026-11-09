# Energy Resource Registry Auditor v0.1

A standalone Rust tool that checks the exact Sol-Atlas Energy Resource Source Registry against the pinned v0.1 schema contract.

## Run

From the repository root:

```sh
cargo fmt --manifest-path tools/energy-resource-registry-auditor/Cargo.toml --all -- --check
cargo test --manifest-path tools/energy-resource-registry-auditor/Cargo.toml --locked
cargo clippy --manifest-path tools/energy-resource-registry-auditor/Cargo.toml --all-targets --locked -- -D warnings

cargo run --release \
  --manifest-path tools/energy-resource-registry-auditor/Cargo.toml \
  --locked -- \
  schemas/energy-resource-source-registry-v0.1.schema.json \
  docs/research/energy/SA_ENERGY_RESOURCE_SOURCE_REGISTER_V0_1.json \
  energy-source-registry-audit.json
```

Exit codes:
- `0`: registry accepted by the v0.1 structural/semantic contract;
- `1`: valid JSON was parsed but one or more contract predicates failed;
- `2`: I/O, strict parsing, typed decoding or receipt construction failed.

The input schema and registry are each limited to 4 MiB. The CLI opens regular files, checks the size on the opened handle, then reads through a limit-plus-one byte cap before parsing.

## What it checks

- Strict JSON parsing rejects duplicate object member names and trailing content before typed decoding.
- Every contract object uses `deny_unknown_fields`.
- Schema draft, top-level / nested property names, required fields, enums, closed-object behavior and discovery-only constants are checked against the Rust contract.
- Source IDs are unique and lowercase kebab-case.
- Locators are HTTPS; source/date/version fields and non-empty requirements are checked.
- Resource-family values are unique.
- Unavailable portal records cannot be relabelled as measured datasets.
- Every v0.1 entry must retain `raw_snapshot_ingested=false` and `snapshot_digest=null`.

The receipt contains exact SHA-256 values for the schema and registry bytes, validator identity/version, source IDs, findings, disposition and a deterministic receipt-subject digest. No timestamps or runtime measurements are included in the digest subject.

## Tests

The 31-test unit suite includes the checked-in 51-entry source register, South African geothermal/marine and mine-shaft/PHS claim-boundary controls, and separate field/laboratory/sector-report/literature-review tests for geothermal, solar, hydro, mine-water, pumped-storage and gravity-storage evidence. Hostile controls cover duplicate keys, trailing bytes, oversize input, unknown fields, duplicate IDs, fabricated snapshot admission, unavailable-source relabelling, HTTP locators, invalid dates, schema enum drift and discovery-status mutation.

## Important boundary

This is a schema-specific conformance auditor, not a general-purpose JSON Schema implementation. It checks the subset of Draft 2020-12 semantics used by this pinned registry schema; the schema is the interchange contract and the Rust checker has explicit drift checks for its property and enum surface.

A pass proves only that the exact bytes satisfy the structural/semantic rules implemented for registry v0.1. It does **not**:
- fetch or authenticate source URLs;
- authenticate source publishers or custodians;
- establish licence adequacy or redistribution permission;
- prove scientific truth or model accuracy;
- admit source data into Sol-Atlas evidence layers;
- establish project feasibility, resource yield, grid deliverability or bankability.

All source entries remain discovery-only until a separate source-ingestion/admission workflow records original bytes, rights, metadata, transformations and independent validation.

The two WASA records are intentionally separate: the full-period 30-year point time-series portal is marked temporarily unavailable, while the WASA3 2020 planning raster is a separate downloadable modelled dataset pending snapshot pinning. A regression test prevents this distinction from collapsing. A second regression test checks that the GWA4 source record preserves the absence of South Africa from the published GWA4 validation-country cohort.


The v0.1 evidence basis includes `literature_review` so review/synthesis papers (such as the 2026 water-distribution hydropower review) cannot be misrepresented as a direct field observation. The source register also separates the WRC South African Hydropower Atlas report, its ArcGIS E-Tool catalogue entry, the DWS sector strategy and the 2026 literature review. All remain discovery/context records, not admitted site measurements.


The evidence basis now includes `laboratory_measurement` in addition to `published_field_observation` and `literature_review`. The South African registry therefore distinguishes:
- measured full-scale PAT/PRV operation in Antalya (published operational observations, one installation and an initial ~5-month period);
- experimental PAT characteristic curves from laboratory/test-rig studies (including two-machine and 16-machine datasets);
- modelled PAT network optimisation and closed-loop control papers;
- technology literature reviews.

A regression suite checks that Antalya stays a published field observation while the 2016/2025 PAT curves and Parra 2018 PAT/PRV tests stay laboratory measurement evidence. The sources are external comparators, not South African validation.


The South African field-observation lane now also includes:
- WRC KV 238/10 (2010): five measured Queenswood PAT point observations. Preserve watts, not kWh, and do not extrapolate five samples into annual yield.
- Tshwane 2014: a separate Pierre van Ryneveld cross-flow conduit-hydro installation (about 15 kW maximum capacity, not annual average); the article's Queenswood discussion is cross-linked to the WRC pilot and is not a second installation.
- Drakenstein/Leliefontein 2021: bidirectional PAT/pump station with SCADA and grid metering described. Its 320 MWh/year and 9.9 GWh values are estimates; a second pair of cumulative meter totals is internally inconsistent and must remain unresolved until operator data confirm the units.

Regression tests protect these distinctions. All records remain discovery/context evidence; no raw local telemetry has been ingested.
