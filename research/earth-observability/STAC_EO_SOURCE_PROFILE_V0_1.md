# STAC EO Source Profile v0.1

**Issue:** [STAC-EO-PROFILE-001 / #822](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/822)  
**Implementation branch:** `research/stac-eo-profile-v0-1`  
**Owner:** Sol-Atlas STAC adapter  
**Status:** typed offline adapter profile and synthetic conformance tests; Cargo/format/schema-validator outcomes NOT_RUN.

## Purpose

Preserve the source-native identity and processing context of Earth-observation products as they cross the existing STAC importer. The adapter must make sensor/platform/processing changes visible to downstream split and benchmark manifests.

This is a loss-aware profile over the existing STAC Item, **not** a second asset catalogue or sensor registry.

## Source profile

The importer returns a versioned `StacSourceProfile` with:

- top-level collection ID;
- source-declared platform and constellation;
- ordered instrument identifier list;
- optional provider-declared sensor ID;
- a sensor identity record with origin:
  - `provider_declared`;
  - `derived_from_platform_and_instrument`;
  - `unknown`;
- exact source property names and JSON values for the recognized processing context.

The exact original Item JSON is also retained by `StacItemImport::raw_item_json`. Unknown STAC extensions and vendor fields therefore remain available even when the current typed projection does not interpret them.

## Identity resolution

The profile uses the following rule order:

1. If the source declares `properties.sensor` or `properties.sensor_id`, retain the value as provider-declared.
2. Otherwise, if exactly one platform and exactly one instrument are present, derive a deterministic composite identifier from those exact values and mark the origin `derived_from_platform_and_instrument`.
3. If the platform is absent, the instrument list is empty, or more than one instrument is listed, sensor identity remains unresolved. Do not silently select the first instrument.

The current ObservationRecord V1 represents one instrument per observation. Therefore `StacSourceProfile::observation_identity` deliberately returns no mapping for a multi-instrument Item even when a provider sensor label exists. The complete instrument list remains available in the source profile and raw Item for a later explicit multi-instrument observation profile.

A derived ID is not presented as provider-declared. The ObservationRecord stores the identity-origin enum so downstream evaluation can distinguish the two.

## Processing fields

The profile preserves the exact field names and JSON values for the recognized processing properties, including:

- `processing:version`, `processing:level`, `processing:baseline`;
- `s2:processing_baseline`, `s2:product_type`;
- `landsat:processing_level`, `landsat:collection_number`, `landsat:collection_category`.

The helper maps a processing baseline into ObservationRecord only from a recognized baseline/version property. Landsat collection number and processing level remain namespaced fields in the profile; they are not relabelled as a generic processing baseline.

## Per-asset metadata

Each object-valued STAC asset gets a separate `StacAssetProfile`. The importer retains relevant source extension values such as:

- `eo:bands`;
- `raster:bands`;
- `sar:polarizations`, `sar:frequency_band`, `sar:instrument_mode`;
- `gsd`;
- `proj:code`, `proj:epsg`, `proj:wkt2`, `proj:shape`, `proj:transform`.

Asset profiles are sorted deterministically by asset key. Band and polarization metadata are never promoted from one asset to the whole satellite/platform.

## Observation mapping

`StacSourceProfile::observation_identity` returns a typed mapping only when platform and sensor identity are unambiguous.

`StacObservationIdentity::into_observation_record` then binds that identity to:
- canonical AssetId;
- canonical source-registry ID;
- source-asset and processing receipt digests;
- an optional SHA-256 fingerprint of the exact asset-level measurement profile (for example, the frozen `eo:bands`, `raster:bands` or SAR polarization/mode fields);
- exact purpose-bound rights receipt;
- spatial support, acquisition time, modality and EO quality state.

The measurement-profile fingerprint is supplied by the caller/asset-manifest authority; the STAC importer does not invent a cryptographic hash. If it is unavailable—or if collection or processing-baseline identity is missing—the observation records an incomplete measurement configuration and promoted claims require a cross-system comparability receipt or abstention. The importer does not create or authenticate the source/processing/rights receipts. The rights decision remains under the Mycelix/custodian authority.

## Conformance cases

The current fixture/tests cover:
- Sentinel-2A MSI with Sentinel-2 collection, processing baseline and version;
- provider-declared sensor ID overriding the derived composite;
- Landsat collection and processing fields retained with exact namespaces;
- SAR polarization/frequency/mode metadata retained on its own asset;
- source Item with missing platform/instrument;
- multiple instrument ambiguity, including a provider sensor ID;
- duplicate or malformed instrument arrays;
- asset-level metadata remaining asset-scoped;
- deterministic typed profile and asset ordering across JSON key order;
- exact raw STAC Item text and unknown vendor extension preservation;
- typed profile and full import receipt serialization round-trip;
- explicit mapping into ObservationRecord with sensor identity origin preserved.

## Claim boundary

This profile improves identity preservation and evaluation split integrity only. Asset-level measurement-profile fingerprints are part of the downstream observation-system signature; missing fingerprints are not assumed to match. Cross-system comparability and claim-bound receipts are specified in [EO-SYSTEM-COMPARABILITY-001 / #829](https://github.com/Luminous-Dynamics/sol-atlas-leptos/issues/829). It does not prove the source publisher is authentic, verify bytes, grant rights, establish sensor calibration, determine detection probability, validate a physical change, or establish benchmark superiority.