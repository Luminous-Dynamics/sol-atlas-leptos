// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Offline import of the supported subset of a STAC Item.
//!
//! This is an adapter, not a full STAC conformance validator. It reads a
//! supplied JSON string only; it performs no network requests and does not
//! verify source identity, media bytes, licences, or scientific claims.
//! Unsupported display assets are returned as explicit skip records instead
//! of disappearing silently.

use crate::geospatial_assets::{
    AssetId, AssetKind, AssetRecord, AssetSource, GeoFootprint, LicenceMetadata, LicenceReview,
    ObservationTime, ObservationTimeKind, SpatialResolution,
};
use serde_json::{Map, Value};

/// Result of importing one STAC Item.
#[derive(Debug, Clone, PartialEq)]
pub struct StacItemImport {
    item_id: String,
    imported_assets: Vec<AssetRecord>,
    skipped_assets: Vec<SkippedStacAsset>,
}

impl StacItemImport {
    pub fn item_id(&self) -> &str {
        &self.item_id
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
    let root: Value = serde_json::from_str(input)
        .map_err(|error| format!("invalid STAC Item JSON: {error}"))?;
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

    // A BTreeMap is not guaranteed by serde_json's configuration, so sort keys
    // explicitly to make the report stable regardless of input object ordering.
    let mut keys: Vec<&String> = assets.keys().collect();
    keys.sort();

    let mut imported_assets = Vec::new();
    let mut skipped_assets = Vec::new();

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

        let Some(href) = optional_nonempty_text(asset.get("href"), "asset href")? else {
            skipped_assets.push(SkippedStacAsset {
                key: key.clone(),
                reason: "asset has no usable href".into(),
            });
            continue;
        };

        let Some(media_type) = optional_nonempty_text(asset.get("type"), "asset media type")? else {
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
        let resolution = parse_resolution(asset)?;
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

    Ok(StacItemImport {
        item_id,
        imported_assets,
        skipped_assets,
    })
}

fn required_nonempty_text(value: Option<&Value>, label: &str) -> Result<String, String> {
    optional_nonempty_text(value, label)?
        .ok_or_else(|| format!("missing {label}"))
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
    let datetime = optional_nonempty_text(properties.get("datetime"), "properties.datetime")?;
    let start = optional_nonempty_text(
        properties.get("start_datetime"),
        "properties.start_datetime",
    )?;
    let end =
        optional_nonempty_text(properties.get("end_datetime"), "properties.end_datetime")?;

    match (start, end) {
        (Some(start), Some(end)) => ObservationTime::new(
            ObservationTimeKind::Interval,
            Some(start),
            Some(end),
        ),
        (Some(_), None) | (None, Some(_)) => {
            Err("STAC start_datetime and end_datetime must occur together".into())
        }
        (None, None) => match datetime {
            Some(value) => ObservationTime::new(
                ObservationTimeKind::Instant,
                Some(value),
                None,
            ),
            None => Ok(ObservationTime::unknown()),
        },
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

fn parse_resolution(asset: &Map<String, Value>) -> Result<Option<SpatialResolution>, String> {
    // STAC Common Metadata defines asset-level gsd in meters. Prefer it when
    // present; otherwise summarize Raster Extension per-band spatial resolution.
    if let Some(value) = asset.get("gsd").filter(|value| !value.is_null()) {
        let value = value
            .as_f64()
            .ok_or_else(|| "asset gsd must be numeric".to_string())?;
        return SpatialResolution::new(value, crate::geospatial_assets::ResolutionUnit::Meter)
            .map(Some)
            .map_err(|error| format!("invalid asset gsd: {error}"));
    }

    let Some(bands) = asset.get("raster:bands") else {
        return Ok(None);
    };
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
            let value = value.as_f64().ok_or_else(|| {
                "raster:spatial_resolution must be numeric".to_string()
            })?;
            resolutions.push(value);
        }
    }

    if resolutions.is_empty() {
        return Ok(None);
    }
    if resolutions.iter().any(|value| !value.is_finite() || *value <= 0.0) {
        return Err(
            "raster:spatial_resolution values must be finite and greater than zero".into(),
        );
    }

    // Bands can legitimately have different native resolutions. Preserve the
    // coarsest stated resolution as one conservative summary; the full per-band
    // values are not represented by this simplified internal field.
    let max_resolution = resolutions.into_iter().fold(0.0_f64, f64::max);
    SpatialResolution::new(max_resolution, crate::geospatial_assets::ResolutionUnit::Meter)
        .map(Some)
        .map_err(|error| format!("invalid raster resolution: {error}"))
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
    } else if media_type.contains("3dtiles") || media_type == "application/vnd.cesium.3dtiles+json" {
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
    const ALPHABET: &[u8; 64] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";
    let mut output = String::with_capacity((input.len() * 4).div_ceil(3));
    for chunk in input.chunks(3) {
        let first = chunk[0];
        let second = chunk.get(1).copied();
        let third = chunk.get(2).copied();

        output.push(ALPHABET[(first >> 2) as usize] as char);
        output.push(ALPHABET[(((first & 0x03) << 4) | (second.unwrap_or(0) >> 4)) as usize] as char);

        if let Some(second) = second {
            output.push(ALPHABET[(((second & 0x0f) << 2) | (third.unwrap_or(0) >> 6)) as usize] as char);
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

        let raster = result.imported_assets()
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
        assert_eq!(raster.observation_time().kind(), ObservationTimeKind::Interval);
    }

    #[test]
    fn six_coordinate_bbox_uses_geojson_axis_positions() {
        let item = r#"{
          "type":"Feature",
          "id":"three-dimensional",
          "bbox":[10,20,1,30,40,9],
          "geometry":{
            "type":"Polygon",
            "coordinates":[[[10,20],[30,20],[30,40],[10,40],[10,20]]]
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
    fn unknown_dates_and_licences_stay_unknown() {
        let item = r#"{
          "type":"Feature",
          "id":"unknown",
          "properties":{},
          "assets":{"visual":{"href":"fixture:visual","type":"image/jpeg"}}
        }"#;
        let result = import_item_json(item).unwrap();
        assert_eq!(result.imported_assets()[0].observation_time().kind(), ObservationTimeKind::Unknown);
        assert_eq!(result.imported_assets()[0].licence().review(), LicenceReview::Unknown);
        assert!(result.imported_assets()[0].licence().identifier().is_none());
    }

    #[test]
    fn malformed_item_level_metadata_fails_closed() {
        let bad_type = ITEM.replace(r#""type": "Feature""#, r#""type": "Catalog""#);
        assert!(import_item_json(&bad_type).is_err());

        let bad_bbox = ITEM.replace(r#""bbox": [170, -10, -170, 10]"#, r#""bbox": [1, 2, 3]"#);
        assert!(import_item_json(&bad_bbox).is_err());

        let unpaired_date = ITEM.replace(
            r#""end_datetime": "2025-01-31T00:00:00Z","#,
            r#""end_datetime": null,"#
        );
        assert!(import_item_json(&unpaired_date).is_err());
    }

    #[test]
    fn asset_identity_is_deterministic_and_safe_for_non_token_stac_ids() {
        let a = import_item_json(ITEM).unwrap();
        let b = import_item_json(ITEM).unwrap();
        let ids_a: Vec<_> = a.imported_assets().iter().map(|asset| asset.asset_id().as_str()).collect();
        let ids_b: Vec<_> = b.imported_assets().iter().map(|asset| asset.asset_id().as_str()).collect();
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
