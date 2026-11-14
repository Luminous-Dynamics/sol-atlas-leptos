// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Offline import of the supported subset of a STAC Item.
//!
//! This is an adapter, not a full STAC conformance validator. It reads a
//! supplied JSON string only; it performs no network requests and does not
//! verify source identity, media bytes, licences, or scientific claims.
//! Unsupported display assets are returned as explicit skip records instead
//! of disappearing silently.

use crate::earth_observation::{
    ObservationModality, ObservationQuality, ObservationRecord, RightsReceiptRef,
    SensorIdentityOrigin,
};
use crate::geospatial_assets::{
    AssetId, AssetKind, AssetRecord, AssetSource, GeoFootprint, LicenceMetadata, LicenceReview,
    ObservationTime, ObservationTimeKind, SpatialResolution,
};
use serde_json::{Map, Value};
use std::collections::BTreeMap;

pub const STAC_EO_SOURCE_PROFILE_VERSION: u32 = 1;

/// STAC Item-level, source-native platform/instrument context. The original
/// source JSON is retained separately so unknown extensions are not discarded.
#[derive(Debug, Clone, PartialEq)]
pub struct StacSourceProfile {
    profile_version: u32,
    collection_id: Option<String>,
    platform: Option<String>,
    constellation: Option<String>,
    instruments: Vec<String>,
    provider_sensor_id: Option<String>,
    sensor_identity: StacSensorIdentity,
    /// Exact property names and JSON values for the recognized processing
    /// fields. Values are not normalized into an invented common scalar.
    processing_fields: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StacSensorIdentityOrigin {
    ProviderDeclared,
    DerivedFromPlatformAndInstrument,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StacSensorIdentity {
    value: Option<String>,
    origin: StacSensorIdentityOrigin,
}

impl StacSensorIdentity {
    pub fn value(&self) -> Option<&str> { self.value.as_deref() }
    pub fn origin(&self) -> StacSensorIdentityOrigin { self.origin }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StacObservationIdentity {
    pub platform_id: String,
    pub sensor_id: String,
    pub sensor_identity_origin: SensorIdentityOrigin,
    pub instrument_id: Option<String>,
    pub collection_id: Option<String>,
    pub processing_baseline: Option<String>,
}

impl StacObservationIdentity {
    /// Bind the source-native STAC identity into the satellite observation
    /// contract. Asset identity, source registry ID, integrity receipts and
    /// purpose-bound rights decision are supplied by their authoritative
    /// adapters; this method does not invent or authenticate them.
    #[allow(clippy::too_many_arguments)]
    pub fn into_observation_record(
        self,
        observation_id: impl Into<String>,
        asset_id: AssetId,
        source_id: impl Into<String>,
        source_asset_receipt_sha256: impl Into<String>,
        processing_receipt_sha256: impl Into<String>,
        rights: RightsReceiptRef,
        support_id: impl Into<String>,
        acquisition: ObservationTime,
        modality: ObservationModality,
        quality: ObservationQuality,
    ) -> Result<ObservationRecord, String> {
        ObservationRecord::new(
            observation_id,
            asset_id,
            source_id,
            self.platform_id,
            self.sensor_id,
            self.sensor_identity_origin,
            self.instrument_id,
            self.collection_id,
            self.processing_baseline,
            source_asset_receipt_sha256,
            processing_receipt_sha256,
            rights,
            support_id,
            acquisition,
            modality,
            quality,
        )
    }
}

impl StacSourceProfile {
    pub fn profile_version(&self) -> u32 { self.profile_version }
    pub fn collection_id(&self) -> Option<&str> { self.collection_id.as_deref() }
    pub fn platform(&self) -> Option<&str> { self.platform.as_deref() }
    pub fn constellation(&self) -> Option<&str> { self.constellation.as_deref() }
    pub fn instruments(&self) -> &[String] { &self.instruments }
    pub fn provider_sensor_id(&self) -> Option<&str> { self.provider_sensor_id.as_deref() }
    pub fn sensor_identity(&self) -> &StacSensorIdentity { &self.sensor_identity }
    pub fn processing_fields(&self) -> &BTreeMap<String, Value> { &self.processing_fields }

    /// Build the observation identity only when platform and sensor identity
    /// are unambiguous. Unknown/multiple-instrument cases remain unresolved.
    pub fn observation_identity(&self) -> Option<StacObservationIdentity> {
        // ObservationRecord V1 is a single-instrument profile. Preserve
        // multi-instrument STAC source metadata, but do not silently select one.
        if self.instruments.len() > 1 {
            return None;
        }
        let platform_id = self.platform.clone()?;
        let sensor_id = self.sensor_identity.value.clone()?;
        let sensor_identity_origin = match self.sensor_identity.origin {
            StacSensorIdentityOrigin::ProviderDeclared => SensorIdentityOrigin::ProviderDeclared,
            StacSensorIdentityOrigin::DerivedFromPlatformAndInstrument => {
                SensorIdentityOrigin::DerivedFromPlatformAndInstrument
            }
            StacSensorIdentityOrigin::Unknown => return None,
        };
        let instrument_id = (self.instruments.len() == 1)
            .then(|| self.instruments[0].clone());
        let processing_baseline = [
            "s2:processing_baseline",
            "processing:version",
            "processing:baseline",
        ]
        .iter()
        .find_map(|key| match self.processing_fields.get(*key) {
            Some(Value::String(text)) if !text.trim().is_empty() => Some(text.clone()),
            Some(Value::Number(number)) => Some(number.to_string()),
            _ => None,
        });
        Some(StacObservationIdentity {
            platform_id,
            sensor_id,
            sensor_identity_origin,
            instrument_id,
            collection_id: self.collection_id.clone(),
            processing_baseline,
        })
    }
}

/// Source-native asset metadata retained separately for each STAC asset.
#[derive(Debug, Clone, PartialEq)]
pub struct StacAssetProfile {
    asset_key: String,
    extension_fields: BTreeMap<String, Value>,
}

impl StacAssetProfile {
    pub fn asset_key(&self) -> &str { &self.asset_key }
    pub fn extension_fields(&self) -> &BTreeMap<String, Value> { &self.extension_fields }
}


/// Result of importing one STAC Item.
#[derive(Debug, Clone, PartialEq)]
pub struct StacItemImport {
    item_id: String,
    /// Exact UTF-8 JSON text supplied to the importer, preserving unknown
    /// STAC extensions and their source-native representation.
    raw_item_json: String,
    source_profile: StacSourceProfile,
    asset_profiles: Vec<StacAssetProfile>,
    imported_assets: Vec<AssetRecord>,
    skipped_assets: Vec<SkippedStacAsset>,
}

impl StacItemImport {
    pub fn item_id(&self) -> &str {
        &self.item_id
    }

    pub fn raw_item_json(&self) -> &str {
        &self.raw_item_json
    }

    pub fn source_profile(&self) -> &StacSourceProfile {
        &self.source_profile
    }

    pub fn asset_profiles(&self) -> &[StacAssetProfile] {
        &self.asset_profiles
    }

    pub fn imported_assets(&self) -> &[AssetRecord] {
        &self.imported_assets
    }

    pub fn skipped_assets(&self) -> &[SkippedStacAsset] {
        &self.skipped_assets
    }
}

/// A STAC asset excluded from the current model, with an explicit reason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SkippedStacAsset {
    key: String,
    reason: String,
}

impl SkippedStacAsset {
    pub fn key(&self) -> &str {
        &self.key
    }

    pub fn reason(&self) -> &str {
        &self.reason
    }
}

/// Import a single STAC Item JSON record into Sol Atlas asset metadata.
///
/// Supported inputs include raster imagery/scientific rasters, GeoJSON and
/// selected vector formats, point clouds with recognized media types,
/// 3D Tiles, and PMTiles. Unsupported or metadata-only assets are reported
/// in `skipped_assets`. Malformed item-level metadata returns an error.
///
/// A STAC Item's WGS84 bbox is shared by its assets because the common STAC
/// Item model does not require a separate footprint for each asset. Asset-level
/// `proj:epsg` or `proj:code` is preferred over the item-level value.
/// A six-number GeoJSON bbox is interpreted as [west, south, min-z, east,
/// north, max-z], not as a two-dimensional bbox.
///
/// The importer deliberately does not infer currentness, capture bytes, resolve
/// relative hrefs, or mark licences reusable just because a STAC item declares
/// a licence identifier.
pub fn import_item_json(input: &str) -> Result<StacItemImport, String> {
    let root: Value =
        serde_json::from_str(input).map_err(|error| format!("invalid STAC Item JSON: {error}"))?;
    let object = root
        .as_object()
        .ok_or_else(|| "STAC Item root must be an object".to_string())?;

    if object.get("type").and_then(Value::as_str) != Some("Feature") {
        return Err("STAC Item must declare type 'Feature'".into());
    }

    let item_id = required_nonempty_text(object.get("id"), "item id")?;
    let properties = object
        .get("properties")
        .and_then(Value::as_object)
        .ok_or_else(|| "STAC Item properties must be an object".to_string())?;
    let assets = object
        .get("assets")
        .and_then(Value::as_object)
        .ok_or_else(|| "STAC Item assets must be an object".to_string())?;

    let footprint = parse_item_footprint(object.get("bbox"))?;
    let observation_time = parse_observation_time(properties)?;
    let licence = parse_licence(object)?;
    let source_profile = parse_source_profile(object, properties, &item_id)?;


    // A BTreeMap is not guaranteed by serde_json's configuration, so sort keys
    // explicitly to make the report stable regardless of input object ordering.
    let mut keys: Vec<&String> = assets.keys().collect();
    keys.sort();

    let mut imported_assets = Vec::new();
    let mut skipped_assets = Vec::new();
    let mut asset_profiles = Vec::new();

    for key in keys {
        if key.trim().is_empty() {
            return Err("STAC asset key must not be empty".into());
        }
        let value = &assets[key];
        let Some(asset) = value.as_object() else {
            skipped_assets.push(SkippedStacAsset {
                key: key.clone(),
                reason: "asset value is not a JSON object".into(),
            });
            continue;
        };

        asset_profiles.push(parse_asset_profile(key, asset));
        let Some(href) = optional_nonempty_text(asset.get("href"), "asset href")? else {
            skipped_assets.push(SkippedStacAsset {
                key: key.clone(),
                reason: "asset has no usable href".into(),
            });
            continue;
        };

        let Some(media_type) = optional_nonempty_text(asset.get("type"), "asset media type")?
        else {
            skipped_assets.push(SkippedStacAsset {
                key: key.clone(),
                reason: "asset has no declared media type".into(),
            });
            continue;
        };

        let kind = match classify_asset(asset, &media_type) {
            Some(kind) => kind,
            None => {
                skipped_assets.push(SkippedStacAsset {
                    key: key.clone(),
                    reason: format!("unsupported media type or asset semantics: {media_type}"),
                });
                continue;
            }
        };

        let asset_title = optional_nonempty_text(asset.get("title"), "asset title")?
            .unwrap_or_else(|| key.clone());
        let source_crs = source_crs(asset, properties)?;
        let id = AssetId::new(format!(
            "stac:{}:{}",
            base64url_no_pad(item_id.as_bytes()),
            base64url_no_pad(key.as_bytes())
        ))?;
        let source = AssetSource::new(None, href)?;
        let resolution = parse_resolution(asset, properties)?;
        let record = AssetRecord::new(
            id,
            asset_title,
            kind,
            source,
            Some(media_type),
            source_crs,
            footprint.clone(),
            observation_time.clone(),
            licence.clone(),
            resolution,
            None,
            None,
        )
        .map_err(|error| format!("invalid STAC asset '{key}': {error}"))?;

        imported_assets.push(record);
    }

    imported_assets.sort_by(|left, right| left.asset_id().cmp(right.asset_id()));
    asset_profiles.sort_by(|left, right| left.asset_key.cmp(&right.asset_key));

    Ok(StacItemImport {
        item_id,
        raw_item_json: input.to_string(),
        source_profile,
        asset_profiles,
        imported_assets,
        skipped_assets,
    })
}

fn parse_source_profile(
    object: &Map<String, Value>,
    properties: &Map<String, Value>,
    item_id: &str,
) -> Result<StacSourceProfile, String> {
    let collection_id = optional_nonempty_text(object.get("collection"), "collection")?;
    let platform = optional_nonempty_text(properties.get("platform"), "properties.platform")?;
    let constellation =
        optional_nonempty_text(properties.get("constellation"), "properties.constellation")?;
    let instruments = parse_string_array(properties.get("instruments"), "properties.instruments")?;
    let provider_sensor_id = optional_nonempty_text(
        properties.get("sensor").or_else(|| properties.get("sensor_id")),
        "properties.sensor",
    )?;

    let sensor_identity = if let Some(value) = provider_sensor_id.clone() {
        StacSensorIdentity {
            value: Some(value),
            origin: StacSensorIdentityOrigin::ProviderDeclared,
        }
    } else if let (Some(platform), [instrument]) = (platform.as_deref(), instruments.as_slice()) {
        StacSensorIdentity {
            value: Some(format!(
                "derived:{}:{}",
                base64url_no_pad(platform.as_bytes()),
                base64url_no_pad(instrument.as_bytes())
            )),
            origin: StacSensorIdentityOrigin::DerivedFromPlatformAndInstrument,
        }
    } else {
        // Multiple instruments are intentionally ambiguous without an explicit
        // provider sensor ID; never pick the first instrument silently.
        let _ = item_id;
        StacSensorIdentity {
            value: None,
            origin: StacSensorIdentityOrigin::Unknown,
        }
    };

    let processing_keys = [
        "processing:version",
        "processing:level",
        "processing:baseline",
        "s2:processing_baseline",
        "s2:product_type",
        "landsat:processing_level",
        "landsat:collection_number",
        "landsat:collection_category",
    ];
    let mut processing_fields = BTreeMap::new();
    for key in processing_keys {
        if let Some(value) = properties.get(key) {
            processing_fields.insert(key.to_string(), value.clone());
        }
    }

    Ok(StacSourceProfile {
        profile_version: STAC_EO_SOURCE_PROFILE_VERSION,
        collection_id,
        platform,
        constellation,
        instruments,
        provider_sensor_id,
        sensor_identity,
        processing_fields,
    })
}

fn parse_string_array(value: Option<&Value>, label: &str) -> Result<Vec<String>, String> {
    let Some(value) = value.filter(|value| !value.is_null()) else {
        return Ok(Vec::new());
    };
    let array = value
        .as_array()
        .ok_or_else(|| format!("{label} must be an array of strings or null"))?;
    let mut seen = std::collections::BTreeSet::new();
    let mut output = Vec::with_capacity(array.len());
    for (index, item) in array.iter().enumerate() {
        let text = item
            .as_str()
            .filter(|text| !text.trim().is_empty())
            .ok_or_else(|| format!("{label}[{index}] must be a non-empty string"))?;
        if !seen.insert(text.to_string()) {
            return Err(format!("{label} contains duplicate value '{text}'"));
        }
        // Preserve source ordering; the set is used only to reject duplicates.
        output.push(text.to_string());
    }
    Ok(output)
}

fn parse_asset_profile(
    asset_key: &str,
    asset: &Map<String, Value>,
) -> StacAssetProfile {
    let source_native_keys = [
        "eo:bands",
        "raster:bands",
        "sar:polarizations",
        "sar:frequency_band",
        "sar:instrument_mode",
        "gsd",
        "proj:code",
        "proj:epsg",
        "proj:wkt2",
        "proj:shape",
        "proj:transform",
    ];
    let mut extension_fields = BTreeMap::new();
    for key in source_native_keys {
        if let Some(value) = asset.get(key) {
            extension_fields.insert(key.to_string(), value.clone());
        }
    }
    StacAssetProfile {
        asset_key: asset_key.to_string(),
        extension_fields,
    }
}

fn required_nonempty_text(value: Option<&Value>, label: &str) -> Result<String, String> {
    optional_nonempty_text(value, label)?.ok_or_else(|| format!("missing {label}"))
}

fn optional_nonempty_text(value: Option<&Value>, label: &str) -> Result<Option<String>, String> {
    match value {
        None | Some(Value::Null) => Ok(None),
        Some(Value::String(text)) if !text.trim().is_empty() => Ok(Some(text.clone())),
        Some(Value::String(_)) => Err(format!("{label} must not be empty")),
        Some(_) => Err(format!("{label} must be a string or null")),
    }
}

fn parse_item_footprint(value: Option<&Value>) -> Result<Option<GeoFootprint>, String> {
    let Some(value) = value.filter(|value| !value.is_null()) else {
        return Ok(None);
    };
    let bbox = value
        .as_array()
        .ok_or_else(|| "STAC Item bbox must be an array or null".to_string())?;
    let number = |index: usize| -> Result<f64, String> {
        bbox.get(index)
            .and_then(Value::as_f64)
            .ok_or_else(|| format!("STAC Item bbox coordinate {index} must be numeric"))
    };

    let (west, south, east, north) = match bbox.len() {
        4 => (number(0)?, number(1)?, number(2)?, number(3)?),
        6 => (number(0)?, number(1)?, number(3)?, number(4)?),
        length => {
            return Err(format!(
                "STAC Item bbox must have 4 or 6 coordinates, got {length}"
            ));
        }
    };

    GeoFootprint::new(west, south, east, north)
        .map(Some)
        .map_err(|error| format!("invalid STAC Item bbox: {error}"))
}

fn parse_observation_time(properties: &Map<String, Value>) -> Result<ObservationTime, String> {
    // STAC Item core requires datetime. A null datetime is valid only when
    // both interval bounds are supplied; missing timestamps are not a valid
    // STAC Item and must not be silently accepted as a conforming unknown.
    let datetime_value = properties
        .get("datetime")
        .ok_or_else(|| "STAC Item properties.datetime is required".to_string())?;
    let datetime = optional_nonempty_text(Some(datetime_value), "properties.datetime")?;
    let start = optional_nonempty_text(
        properties.get("start_datetime"),
        "properties.start_datetime",
    )?;
    let end = optional_nonempty_text(properties.get("end_datetime"), "properties.end_datetime")?;

    match (datetime, start, end) {
        (Some(datetime), Some(start), Some(end)) => ObservationTime::new_with_datetime(
            ObservationTimeKind::Interval,
            Some(datetime),
            Some(start),
            Some(end),
        ),
        (Some(datetime), None, None) => {
            ObservationTime::new(ObservationTimeKind::Instant, Some(datetime), None)
        }
        (Some(_), Some(_), None) | (Some(_), None, Some(_)) => {
            Err("STAC start_datetime and end_datetime must occur together".into())
        }
        (None, Some(start), Some(end)) => ObservationTime::new_with_datetime(
            ObservationTimeKind::Interval,
            None,
            Some(start),
            Some(end),
        ),
        (None, Some(_), None) | (None, None, Some(_)) => {
            Err("STAC start_datetime and end_datetime must occur together".into())
        }
        (None, None, None) => Err(
            "STAC datetime may be null only when start_datetime and end_datetime are supplied"
                .into(),
        ),
    }
}

fn parse_licence(object: &Map<String, Value>) -> Result<LicenceMetadata, String> {
    let identifier = optional_nonempty_text(object.get("license"), "license")?;
    let terms_locator = object
        .get("links")
        .and_then(Value::as_array)
        .and_then(|links| {
            links.iter().find_map(|link| {
                let link = link.as_object()?;
                (link.get("rel").and_then(Value::as_str) == Some("license"))
                    .then(|| link.get("href").and_then(Value::as_str))
                    .flatten()
            })
        })
        .map(str::to_string);

    // Do not convert a declaration into legal clearance. A missing or
    // provider-specific "other" declaration remains explicitly unknown.
    let review = match identifier.as_deref() {
        None | Some("other") => LicenceReview::Unknown,
        Some(_) => LicenceReview::NotReviewed,
    };

    LicenceMetadata::new(identifier, terms_locator, None, review)
}

fn source_crs(
    asset: &Map<String, Value>,
    properties: &Map<String, Value>,
) -> Result<Option<String>, String> {
    // Asset-level projection metadata overrides item-level metadata.
    // Prefer current proj:code; accept deprecated proj:epsg for older catalogs.
    // WKT2 and PROJJSON remain explicit, opaque declarations when no code exists.
    for object in [asset, properties] {
        if let Some(value) = object.get("proj:code").filter(|value| !value.is_null()) {
            return optional_nonempty_text(Some(value), "proj:code");
        }
        if let Some(value) = object.get("proj:epsg").filter(|value| !value.is_null()) {
            if let Some(code) = value.as_i64() {
                if code <= 0 {
                    return Err("proj:epsg must be a positive integer".into());
                }
                return Ok(Some(format!("EPSG:{code}")));
            }
            return Err("proj:epsg must be a positive integer or null".into());
        }
        if let Some(value) = object.get("proj:wkt2").filter(|value| !value.is_null()) {
            let wkt = required_nonempty_text(Some(value), "proj:wkt2")?;
            return Ok(Some(format!("WKT2:{wkt}")));
        }
        if let Some(value) = object.get("proj:projjson").filter(|value| !value.is_null()) {
            if !value.is_object() {
                return Err("proj:projjson must be an object or null".into());
            }
            let serialized = serde_json::to_string(value)
                .map_err(|error| format!("invalid proj:projjson: {error}"))?;
            return Ok(Some(format!("PROJJSON:{serialized}")));
        }
    }
    Ok(None)
}

fn parse_resolution(
    asset: &Map<String, Value>,
    properties: &Map<String, Value>,
) -> Result<Option<SpatialResolution>, String> {
    // STAC Common Metadata permits gsd on the Item or an individual asset.
    // Asset-level gsd overrides the Item value. Band-specific raster extension
    // metadata is a useful fallback when no asset-level gsd is supplied.
    if let Some(value) = asset.get("gsd").filter(|value| !value.is_null()) {
        return parse_gsd(value, "asset gsd");
    }

    if let Some(bands) = asset.get("raster:bands") {
        let bands = bands
            .as_array()
            .ok_or_else(|| "raster:bands must be an array".to_string())?;
        let mut resolutions = Vec::new();

        for band in bands {
            let Some(band) = band.as_object() else {
                return Err("each raster:bands entry must be an object".into());
            };
            if let Some(value) = band
                .get("raster:spatial_resolution")
                .filter(|value| !value.is_null())
            {
                let value = value
                    .as_f64()
                    .ok_or_else(|| "raster:spatial_resolution must be numeric".to_string())?;
                resolutions.push(value);
            }
        }

        if !resolutions.is_empty() {
            if resolutions
                .iter()
                .any(|value| !value.is_finite() || *value <= 0.0)
            {
                return Err(
                    "raster:spatial_resolution values must be finite and greater than zero".into(),
                );
            }

            // Bands can legitimately have different native resolutions.
            // Preserve the coarsest stated value as a conservative summary.
            let max_resolution = resolutions.into_iter().fold(0.0_f64, f64::max);
            return SpatialResolution::new(
                max_resolution,
                crate::geospatial_assets::ResolutionUnit::Meter,
            )
            .map(Some)
            .map_err(|error| format!("invalid raster resolution: {error}"));
        }
    }

    // Item-level gsd is inherited only after any explicit asset or band-level
    // resolution is considered.
    if let Some(value) = properties.get("gsd").filter(|value| !value.is_null()) {
        return parse_gsd(value, "properties.gsd");
    }

    Ok(None)
}

fn parse_gsd(value: &Value, label: &str) -> Result<Option<SpatialResolution>, String> {
    let value = value
        .as_f64()
        .ok_or_else(|| format!("{label} must be numeric"))?;
    SpatialResolution::new(value, crate::geospatial_assets::ResolutionUnit::Meter)
        .map(Some)
        .map_err(|error| format!("invalid {label}: {error}"))
}

fn classify_asset(asset: &Map<String, Value>, media_type: &str) -> Option<AssetKind> {
    let media_type = media_type
        .split(';')
        .next()
        .unwrap_or(media_type)
        .trim()
        .to_ascii_lowercase();

    if asset.get("vector:layers").is_some() {
        return Some(AssetKind::VectorFeatures);
    }
    if asset.get("pc:count").is_some() || asset.get("pc:encoding").is_some() {
        return Some(AssetKind::PointCloud);
    }
    if asset.get("raster:bands").is_some() {
        return Some(AssetKind::ScientificRaster);
    }

    if media_type.starts_with("image/") {
        if media_type.contains("tiff") || media_type.contains("geotiff") {
            Some(AssetKind::ScientificRaster)
        } else {
            Some(AssetKind::Imagery)
        }
    } else if media_type.contains("3dtiles") || media_type == "application/vnd.cesium.3dtiles+json"
    {
        Some(AssetKind::ThreeDTiles)
    } else if matches!(
        media_type.as_str(),
        "application/geo+json"
            | "application/vnd.geo+json"
            | "application/geopackage+sqlite3"
            | "application/vnd.flatgeobuf"
            | "application/x-flatgeobuf"
    ) {
        Some(AssetKind::VectorFeatures)
    } else if matches!(
        media_type.as_str(),
        "application/vnd.las"
            | "application/x-las"
            | "application/vnd.laszip"
            | "application/x-laz"
    ) {
        Some(AssetKind::PointCloud)
    } else if media_type == "application/vnd.pmtiles" || media_type == "application/x-pmtiles" {
        Some(AssetKind::TilePackage)
    } else {
        None
    }
}

/// Deterministic URL-safe base64 encoding without padding for opaque ID parts.
fn base64url_no_pad(input: &[u8]) -> String {
    const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut output = String::with_capacity((input.len() * 4).div_ceil(3));
    for chunk in input.chunks(3) {
        let first = chunk[0];
        let second = chunk.get(1).copied();
        let third = chunk.get(2).copied();

        output.push(ALPHABET[(first >> 2) as usize] as char);
        output
            .push(ALPHABET[(((first & 0x03) << 4) | (second.unwrap_or(0) >> 4)) as usize] as char);

        if let Some(second) = second {
            output.push(
                ALPHABET[(((second & 0x0f) << 2) | (third.unwrap_or(0) >> 6)) as usize] as char,
            );
        }
        if let Some(third) = third {
            output.push(ALPHABET[(third & 0x3f) as usize] as char);
        }
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;

    const ITEM: &str = include_str!("../tests/fixtures/stac/sentinel-item.json");

    #[test]
    fn imports_supported_assets_and_reports_unsupported_assets() {
        let result = import_item_json(ITEM).unwrap();
        assert_eq!(result.item_id(), "sentinel/item 01");
        assert_eq!(result.imported_assets().len(), 2);
        assert_eq!(result.skipped_assets().len(), 1);
        assert_eq!(result.skipped_assets()[0].key(), "metadata");
        assert!(result.skipped_assets()[0].reason().contains("unsupported"));

        let raster = result
            .imported_assets()
            .iter()
            .find(|record| record.source().locator().ends_with("scene.tif"))
            .unwrap();
        assert_eq!(raster.kind(), AssetKind::ScientificRaster);
        assert_eq!(raster.media_type(), Some("image/tiff; application=geotiff"));
        assert_eq!(raster.source_crs(), Some("EPSG:32633"));
        assert_eq!(raster.spatial_resolution().unwrap().value(), 20.0);
        assert_eq!(raster.licence().review(), LicenceReview::NotReviewed);
        assert_eq!(
            raster.licence().terms_locator(),
            Some("https://example.test/licence")
        );
        assert!(raster.footprint().unwrap().crosses_antimeridian());
        assert_eq!(
            raster.observation_time().kind(),
            ObservationTimeKind::Interval
        );
    }

    #[test]
    fn six_coordinate_bbox_uses_geojson_axis_positions() {
        let item = r#"{
          "type":"Feature",
          "id":"three-dimensional",
          "bbox":[10,20,1,30,40,9],
          "geometry":{
            "type":"Polygon",
            "coordinates":[[[10,20,1],[30,20,1],[30,40,9],[10,40,9],[10,20,1]]]
          },
          "properties":{"datetime":"2025-01-01T00:00:00Z"},
          "assets":{"visual":{"href":"fixture:visual","type":"image/jpeg"}}
        }"#;
        let result = import_item_json(item).unwrap();
        let bbox = result.imported_assets()[0].footprint().unwrap();
        assert_eq!(bbox.west(), 10.0);
        assert_eq!(bbox.south(), 20.0);
        assert_eq!(bbox.east(), 30.0);
        assert_eq!(bbox.north(), 40.0);
    }

    #[test]
    fn missing_or_inconsistent_stac_datetime_fails_closed() {
        let missing = r#"{
          "type":"Feature",
          "id":"missing-date",
          "properties":{},
          "assets":{"visual":{"href":"fixture:visual","type":"image/jpeg"}}
        }"#;
        assert!(
            import_item_json(missing)
                .unwrap_err()
                .contains("datetime is required")
        );

        let null_without_range = r#"{
          "type":"Feature",
          "id":"unknown-date",
          "properties":{"datetime":null},
          "assets":{"visual":{"href":"fixture:visual","type":"image/jpeg"}}
        }"#;
        assert!(
            import_item_json(null_without_range)
                .unwrap_err()
                .contains("may be null only")
        );

        let partial = r#"{
          "type":"Feature",
          "id":"partial-date",
          "properties":{
            "datetime":"2025-01-01T00:00:00Z",
            "start_datetime":"2025-01-01T00:00:00Z"
          },
          "assets":{"visual":{"href":"fixture:visual","type":"image/jpeg"}}
        }"#;
        assert!(
            import_item_json(partial)
                .unwrap_err()
                .contains("must occur together")
        );

        let mixed = r#"{
          "type":"Feature",
          "id":"mixed-date",
          "properties":{
            "datetime":"2025-01-01T00:00:00Z",
            "start_datetime":"2025-01-01T00:00:00Z",
            "end_datetime":"2025-01-02T00:00:00Z"
          },
          "assets":{"visual":{"href":"fixture:visual","type":"image/jpeg"}}
        }"#;
        let result = import_item_json(mixed).unwrap();
        let temporal = result.imported_assets()[0].observation_time();
        assert_eq!(temporal.kind(), ObservationTimeKind::Interval);
        assert_eq!(temporal.datetime(), Some("2025-01-01T00:00:00Z"));
        assert_eq!(temporal.start(), Some("2025-01-01T00:00:00Z"));
        assert_eq!(temporal.end(), Some("2025-01-02T00:00:00Z"));

        let unknown_licence = r#"{
          "type":"Feature",
          "id":"unknown-licence",
          "properties":{"datetime":"2025-01-01T00:00:00Z"},
          "assets":{"visual":{"href":"fixture:visual","type":"image/jpeg"}}
        }"#;
        let result = import_item_json(unknown_licence).unwrap();
        assert_eq!(
            result.imported_assets()[0].licence().review(),
            LicenceReview::Unknown
        );
        assert!(result.imported_assets()[0].licence().identifier().is_none());
    }

    #[test]
    fn item_gsd_fallback_is_used_when_asset_and_bands_lack_resolution() {
        let item = r#"{
          "type":"Feature",
          "id":"item-gsd",
          "properties":{"datetime":"2025-01-01T00:00:00Z","gsd":30},
          "assets":{"visual":{"href":"fixture:visual","type":"image/jpeg"}}
        }"#;
        let result = import_item_json(item).unwrap();
        assert_eq!(
            result.imported_assets()[0]
                .spatial_resolution()
                .unwrap()
                .value(),
            30.0
        );

        let asset_override = r#"{
          "type":"Feature",
          "id":"asset-gsd",
          "properties":{"datetime":"2025-01-01T00:00:00Z","gsd":30},
          "assets":{"visual":{"href":"fixture:visual","type":"image/jpeg","gsd":5}}
        }"#;
        let result = import_item_json(asset_override).unwrap();
        assert_eq!(
            result.imported_assets()[0]
                .spatial_resolution()
                .unwrap()
                .value(),
            5.0
        );
    }

    #[test]
    fn malformed_item_level_metadata_fails_closed() {
        let bad_type = ITEM.replace(r#""type": "Feature""#, r#""type": "Catalog""#);
        assert!(import_item_json(&bad_type).is_err());

        let bad_bbox = ITEM.replace(r#""bbox": [170, -10, -170, 10]"#, r#""bbox": [1, 2, 3]"#);
        assert!(import_item_json(&bad_bbox).is_err());

        let unpaired_date = ITEM.replace(
            r#""end_datetime": "2025-01-31T00:00:00Z","#,
            r#""end_datetime": null,"#,
        );
        assert!(import_item_json(&unpaired_date).is_err());
    }

    #[test]
    fn asset_identity_is_deterministic_and_safe_for_non_token_stac_ids() {
        let a = import_item_json(ITEM).unwrap();
        let b = import_item_json(ITEM).unwrap();
        let ids_a: Vec<_> = a
            .imported_assets()
            .iter()
            .map(|asset| asset.asset_id().as_str())
            .collect();
        let ids_b: Vec<_> = b
            .imported_assets()
            .iter()
            .map(|asset| asset.asset_id().as_str())
            .collect();
        assert_eq!(ids_a, ids_b);
        assert!(ids_a.iter().all(|id| id.starts_with("stac:")));
        assert!(ids_a.iter().all(|id| id.len() <= 128));
    }

    #[test]
    fn legacy_epsg_and_wkt2_crs_declarations_are_preserved() {
        let legacy = r#"{
          "type":"Feature",
          "id":"legacy-epsg",
          "properties":{"datetime":"2025-01-01T00:00:00Z","proj:epsg":3857},
          "assets":{"visual":{"href":"fixture:visual","type":"image/jpeg"}}
        }"#;
        let legacy_result = import_item_json(legacy).unwrap();
        assert_eq!(
            legacy_result.imported_assets()[0].source_crs(),
            Some("EPSG:3857")
        );

        let wkt = r#"{
          "type":"Feature",
          "id":"wkt-crs",
          "properties":{"datetime":"2025-01-01T00:00:00Z","proj:code":null,"proj:wkt2":"GEOGCRS[fixture]"},
          "assets":{"visual":{"href":"fixture:visual","type":"image/jpeg"}}
        }"#;
        let wkt_result = import_item_json(wkt).unwrap();
        assert_eq!(
            wkt_result.imported_assets()[0].source_crs(),
            Some("WKT2:GEOGCRS[fixture]")
        );
    }

    #[test]
    fn preserves_sentinel_platform_instrument_collection_and_processing_baseline() {
        let item = r#"{
          "type":"Feature",
          "stac_version":"1.0.0",
          "id":"s2a-scene-001",
          "collection":"sentinel-2-l2a",
          "stac_extensions":[
            "https://stac-extensions.github.io/eo/v2.0.0/schema.json",
            "https://stac-extensions.github.io/instrument/v1.0.0/schema.json",
            "https://stac-extensions.github.io/sar/v1.0.0/schema.json"
          ],
          "properties":{
            "datetime":"2025-01-01T00:00:00Z",
            "platform":"sentinel-2a",
            "constellation":"sentinel-2",
            "instruments":["MSI"],
            "s2:processing_baseline":"04.00",
            "processing:version":"Sen2Cor-2.11",
            "vendor:opaque_extension":{"preserve":true}
          },
          "assets":{
            "B04":{
              "href":"fixture:b04.tif",
              "type":"image/tiff",
              "eo:bands":[{"name":"B04","common_name":"red","center_wavelength":0.665}],
              "raster:bands":[{"raster:spatial_resolution":10}]
            },
            "VH":{
              "href":"fixture:vh.tif",
              "type":"image/tiff",
              "sar:polarizations":["VV","VH"],
              "sar:frequency_band":"C",
              "sar:instrument_mode":"IW"
            }
          }
        }"#;
        let result = import_item_json(item).unwrap();
        let profile = result.source_profile();
        assert_eq!(profile.profile_version(), STAC_EO_SOURCE_PROFILE_VERSION);
        assert_eq!(profile.collection_id(), Some("sentinel-2-l2a"));
        assert_eq!(profile.platform(), Some("sentinel-2a"));
        assert_eq!(profile.constellation(), Some("sentinel-2"));
        assert_eq!(profile.instruments(), &["MSI".to_string()]);
        assert_eq!(
            profile.sensor_identity().origin,
            StacSensorIdentityOrigin::DerivedFromPlatformAndInstrument
        );
        assert_eq!(
            profile.sensor_identity().value.as_deref(),
            Some("derived:c2VudGluZWwtMmE:TVNJ")
        );
        assert_eq!(
            profile.processing_fields().get("s2:processing_baseline"),
            Some(&Value::String("04.00".into()))
        );
        assert_eq!(
            profile.processing_fields().get("processing:version"),
            Some(&Value::String("Sen2Cor-2.11".into()))
        );

        let identity = profile.observation_identity().unwrap();
        assert_eq!(identity.platform_id, "sentinel-2a");
        assert_eq!(identity.sensor_id, "derived:c2VudGluZWwtMmE:TVNJ");
        assert_eq!(
            identity.sensor_identity_origin,
            SensorIdentityOrigin::DerivedFromPlatformAndInstrument
        );
        assert_eq!(identity.instrument_id.as_deref(), Some("MSI"));
        assert_eq!(identity.collection_id.as_deref(), Some("sentinel-2-l2a"));
        assert_eq!(identity.processing_baseline.as_deref(), Some("04.00"));

        let b04 = result.asset_profiles().iter().find(|a| a.asset_key() == "B04").unwrap();
        assert!(b04.extension_fields().contains_key("eo:bands"));
        assert!(b04.extension_fields().contains_key("raster:bands"));
        let vh = result.asset_profiles().iter().find(|a| a.asset_key() == "VH").unwrap();
        assert_eq!(
            vh.extension_fields().get("sar:polarizations"),
            Some(&serde_json::json!(["VV", "VH"]))
        );
        assert!(!b04.extension_fields().contains_key("sar:polarizations"));
        assert!(result.raw_item_json().contains("vendor:opaque_extension"));
    }

    #[test]
    fn frozen_sentinel_source_profile_fixture_preserves_namespaced_metadata() {
        let fixture = include_str!("../tests/fixtures/stac/sentinel-eo-source-profile-item.json");
        let result = import_item_json(fixture).unwrap();
        let profile = result.source_profile();
        assert_eq!(profile.collection_id(), Some("sentinel-2-l2a"));
        assert_eq!(profile.platform(), Some("sentinel-2a"));
        assert_eq!(profile.constellation(), Some("sentinel-2"));
        assert_eq!(profile.instruments(), &["MSI".to_string()]);
        assert_eq!(
            profile.sensor_identity().origin(),
            StacSensorIdentityOrigin::DerivedFromPlatformAndInstrument
        );
        assert_eq!(
            profile.sensor_identity().value(),
            Some("derived:c2VudGluZWwtMmE:TVNJ")
        );
        assert!(profile.processing_fields().contains_key("s2:processing_baseline"));
        assert!(result.raw_item_json().contains("vendor:unmodeled_field"));
        assert_eq!(result.asset_profiles().len(), 2);
        let b04 = result.asset_profiles().iter().find(|asset| asset.asset_key() == "B04").unwrap();
        let vh = result.asset_profiles().iter().find(|asset| asset.asset_key() == "VH").unwrap();
        assert!(b04.extension_fields().contains_key("eo:bands"));
        assert!(!b04.extension_fields().contains_key("sar:polarizations"));
        assert!(vh.extension_fields().contains_key("sar:polarizations"));
        assert!(!vh.extension_fields().contains_key("eo:bands"));
        let identity = profile.observation_identity().unwrap();
        assert_eq!(
            identity.sensor_identity_origin,
            SensorIdentityOrigin::DerivedFromPlatformAndInstrument
        );
        let mapped = identity.into_observation_record(
            "mapped-observation",
            AssetId::new("stac:fixture:asset").unwrap(),
            "fixture-source-registry",
            "b".repeat(64),
            "c".repeat(64),
            RightsReceiptRef::new(
                "fixture-subject",
                "internal-evaluation",
                "fixture-source-registry",
                "stac:fixture:asset",
                "a".repeat(64),
            ).unwrap(),
            "support-sentinel-a",
            ObservationTime::new(
                ObservationTimeKind::Instant,
                Some("2025-01-01T00:00:00Z".into()),
                None,
            ).unwrap(),
            ObservationModality::Optical,
            ObservationQuality::QualifiedUsable,
        ).unwrap();
        assert_eq!(mapped.platform_id(), "sentinel-2a");
        assert_eq!(mapped.sensor_id(), "derived:c2VudGluZWwtMmE:TVNJ");
        assert_eq!(
            mapped.sensor_identity_origin(),
            SensorIdentityOrigin::DerivedFromPlatformAndInstrument
        );
        assert_eq!(mapped.instrument_id(), Some("MSI"));
        assert_eq!(mapped.processing_baseline(), Some("04.00"));
    }

    #[test]
    fn provider_sensor_identity_overrides_composite_derivation() {
        let item = r#"{
          "type":"Feature",
          "id":"provider-sensor",
          "collection":"provider-collection",
          "properties":{
            "datetime":"2025-01-01T00:00:00Z",
            "platform":"platform-a",
            "instruments":["instrument-a"],
            "sensor":"provider-sensor-id"
          },
          "assets":{"visual":{"href":"fixture:visual","type":"image/jpeg"}}
        }"#;
        let result = import_item_json(item).unwrap();
        let identity = result.source_profile().observation_identity().unwrap();
        assert_eq!(identity.sensor_id, "provider-sensor-id");
        assert_eq!(identity.sensor_identity_origin, SensorIdentityOrigin::ProviderDeclared);
    }

    #[test]
    fn ambiguous_multiple_instruments_and_missing_platform_remain_unresolved() {
        let multiple = r#"{
          "type":"Feature",
          "id":"multiple-instruments",
          "properties":{
            "datetime":"2025-01-01T00:00:00Z",
            "platform":"platform-a",
            "instruments":["instrument-a","instrument-b"]
          },
          "assets":{"visual":{"href":"fixture:visual","type":"image/jpeg"}}
        }"#;
        let result = import_item_json(multiple).unwrap();
        assert_eq!(
            result.source_profile().sensor_identity().origin,
            StacSensorIdentityOrigin::Unknown
        );
        assert!(result.source_profile().observation_identity().is_none());

        let multiple_with_provider_sensor = r#"{
          "type":"Feature",
          "id":"multiple-instruments-provider-sensor",
          "properties":{
            "datetime":"2025-01-01T00:00:00Z",
            "platform":"platform-a",
            "instruments":["instrument-a","instrument-b"],
            "sensor":"provider-sensor-id"
          },
          "assets":{"visual":{"href":"fixture:visual","type":"image/jpeg"}}
        }"#;
        let result = import_item_json(multiple_with_provider_sensor).unwrap();
        assert_eq!(
            result.source_profile().sensor_identity().origin,
            StacSensorIdentityOrigin::ProviderDeclared
        );
        assert!(result.source_profile().observation_identity().is_none());

        let no_platform = r#"{
          "type":"Feature",
          "id":"no-platform",
          "properties":{
            "datetime":"2025-01-01T00:00:00Z",
            "instruments":["instrument-a"]
          },
          "assets":{"visual":{"href":"fixture:visual","type":"image/jpeg"}}
        }"#;
        let result = import_item_json(no_platform).unwrap();
        assert!(result.source_profile().observation_identity().is_none());
    }

    #[test]
    fn malformed_and_duplicate_instrument_values_fail_closed() {
        let duplicate = r#"{
          "type":"Feature",
          "id":"duplicate-instruments",
          "properties":{
            "datetime":"2025-01-01T00:00:00Z",
            "platform":"platform-a",
            "instruments":["MSI","MSI"]
          },
          "assets":{"visual":{"href":"fixture:visual","type":"image/jpeg"}}
        }"#;
        assert!(import_item_json(duplicate).unwrap_err().contains("duplicate"));

        let malformed = r#"{
          "type":"Feature",
          "id":"malformed-instruments",
          "properties":{
            "datetime":"2025-01-01T00:00:00Z",
            "platform":"platform-a",
            "instruments":"MSI"
          },
          "assets":{"visual":{"href":"fixture:visual","type":"image/jpeg"}}
        }"#;
        assert!(import_item_json(malformed).unwrap_err().contains("array of strings"));
    }

    #[test]
    fn processing_and_asset_metadata_keep_source_namespaces() {
        let item = r#"{
          "type":"Feature",
          "id":"landsat-and-sar",
          "collection":"landsat-c2-l2",
          "properties":{
            "datetime":"2025-01-01T00:00:00Z",
            "platform":"landsat-9",
            "constellation":"landsat",
            "instruments":["OLI-2"],
            "landsat:collection_number":2,
            "landsat:processing_level":"L2SP"
          },
          "assets":{
            "analytic":{
              "href":"fixture:analytic.tif",
              "type":"image/tiff",
              "eo:bands":[{"name":"SR_B4","common_name":"red"}]
            }
          }
        }"#;
        let result = import_item_json(item).unwrap();
        assert_eq!(
            result.source_profile().processing_fields().get("landsat:collection_number"),
            Some(&serde_json::json!(2))
        );
        assert_eq!(
            result.source_profile().processing_fields().get("landsat:processing_level"),
            Some(&Value::String("L2SP".into()))
        );
        assert_eq!(result.asset_profiles()[0].asset_key(), "analytic");
        assert!(result.asset_profiles()[0].extension_fields().contains_key("eo:bands"));
    }

    #[test]
    fn item_and_asset_profiles_are_deterministic_but_raw_json_is_preserved() {
        let a = r#"{"type":"Feature","id":"ordered","collection":"c","properties":{"datetime":"2025-01-01","platform":"p","instruments":["i"]},"assets":{"z":{"href":"z.tif","type":"image/tiff","sar:polarizations":["VV"]},"a":{"href":"a.tif","type":"image/tiff","eo:bands":[{"name":"B1"}]}}}"#;
        let b = r#"{"assets":{"a":{"eo:bands":[{"name":"B1"}],"type":"image/tiff","href":"a.tif"},"z":{"sar:polarizations":["VV"],"type":"image/tiff","href":"z.tif"}},"properties":{"instruments":["i"],"platform":"p","datetime":"2025-01-01"},"collection":"c","id":"ordered","type":"Feature"}"#;
        let left = import_item_json(a).unwrap();
        let right = import_item_json(b).unwrap();
        assert_eq!(left.source_profile(), right.source_profile());
        assert_eq!(
            left.asset_profiles().iter().map(|a| a.asset_key()).collect::<Vec<_>>(),
            right.asset_profiles().iter().map(|a| a.asset_key()).collect::<Vec<_>>()
        );
        assert_ne!(left.raw_item_json(), right.raw_item_json());
    }

    #[test]
    fn importer_does_not_resolve_relative_hrefs_or_claim_verification() {
        let item = r#"{
          "type":"Feature",
          "id":"relative",
          "properties":{"datetime":"2025-01-01"},
          "assets":{"visual":{"href":"./local.tif","type":"image/tiff"}}
        }"#;
        let result = import_item_json(item).unwrap();
        let asset = &result.imported_assets()[0];
        assert_eq!(asset.source().locator(), "./local.tif");
        assert_eq!(asset.source().provider_id(), None);
        assert_eq!(asset.source_snapshot_id(), None);
    }
}
