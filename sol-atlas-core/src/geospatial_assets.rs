// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Versioned, renderer-neutral metadata for geospatial assets.
//!
//! This module describes data that Sol Atlas may display; it does not fetch
//! assets, verify publishers, judge scientific truth, or grant reuse rights.
//! Source locators are references, not integrity or authenticity proofs.
//! Constructors and deserialization share the same validation boundaries.

use serde::{Deserialize, Deserializer, Serialize};
use std::collections::BTreeSet;

/// Current wire schema version for AssetRecord and AssetCatalogV1.
pub const ASSET_SCHEMA_VERSION: u32 = 1;

/// Stable semantic identity for a geospatial asset, independent of its URL.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
#[serde(transparent)]
pub struct AssetId(String);

impl AssetId {
    pub fn new(value: impl Into<String>) -> Result<Self, String> {
        let value = value.into();
        if value.is_empty() || value.trim() != value {
            return Err("asset ID must be non-empty and have no surrounding whitespace".into());
        }
        if value.len() > 128
            || !value
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b'.' | b':'))
        {
            return Err("asset ID must be <=128 ASCII letters, digits, '-', '_', '.', or ':'".into());
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str { &self.0 }
}

impl<'de> Deserialize<'de> for AssetId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de>,
    {
        let value = String::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}

/// Broad data family. This is descriptive metadata, not a renderer instruction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AssetKind {
    Imagery,
    Elevation,
    VectorFeatures,
    ScientificRaster,
    ThreeDTiles,
    PointCloud,
    TilePackage,
}

/// WGS84 longitude/latitude bounding box.
///
/// west > east represents a footprint crossing the antimeridian. The full
/// world can be represented by [-180, 180]; [180, -180] is a degenerate box.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct GeoFootprint {
    west: f64,
    south: f64,
    east: f64,
    north: f64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GeoFootprintWire { west: f64, south: f64, east: f64, north: f64 }

impl GeoFootprint {
    pub fn new(west: f64, south: f64, east: f64, north: f64) -> Result<Self, String> {
        if [west, south, east, north].iter().any(|v| !v.is_finite()) {
            return Err("footprint coordinates must be finite".into());
        }
        if !(-180.0..=180.0).contains(&west) || !(-180.0..=180.0).contains(&east) {
            return Err("footprint longitude must be within [-180, 180]".into());
        }
        if !(-90.0..=90.0).contains(&south) || !(-90.0..=90.0).contains(&north) {
            return Err("footprint latitude must be within [-90, 90]".into());
        }
        if south >= north {
            return Err("footprint must have south < north".into());
        }
        if west == east || (west == 180.0 && east == -180.0) {
            return Err("footprint must have non-zero longitudinal width".into());
        }
        Ok(Self { west, south, east, north })
    }

    pub fn west(&self) -> f64 { self.west }
    pub fn south(&self) -> f64 { self.south }
    pub fn east(&self) -> f64 { self.east }
    pub fn north(&self) -> f64 { self.north }
    pub fn crosses_antimeridian(&self) -> bool { self.west > self.east }

    /// Bounding-box longitude span in degrees, including antimeridian crossing.
    pub fn longitude_span_degrees(&self) -> f64 {
        if self.crosses_antimeridian() {
            (180.0 - self.west) + (self.east + 180.0)
        } else {
            self.east - self.west
        }
    }
}

impl<'de> Deserialize<'de> for GeoFootprint {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de>,
    {
        let wire = GeoFootprintWire::deserialize(deserializer)?;
        Self::new(wire.west, wire.south, wire.east, wire.north)
            .map_err(serde::de::Error::custom)
    }
}

/// How observation-time metadata should be interpreted.
///
/// Values are preserved as supplied by the source; this type does not claim
/// to parse or authenticate provider timestamps. Chronological ordering is not
/// inferred from strings with unknown timezones or precisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObservationTimeKind { Unknown, Instant, Interval, Approximate }

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ObservationTime {
    kind: ObservationTimeKind,
    start: Option<String>,
    end: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ObservationTimeWire {
    kind: ObservationTimeKind,
    start: Option<String>,
    end: Option<String>,
}

impl ObservationTime {
    pub fn unknown() -> Self {
        Self { kind: ObservationTimeKind::Unknown, start: None, end: None }
    }

    pub fn new(
        kind: ObservationTimeKind,
        start: Option<String>,
        end: Option<String>,
    ) -> Result<Self, String> {
        for value in [start.as_deref(), end.as_deref()].into_iter().flatten() {
            if value.trim().is_empty() {
                return Err("observation time values must not be empty".into());
            }
        }
        let valid_shape = match kind {
            ObservationTimeKind::Unknown => start.is_none() && end.is_none(),
            ObservationTimeKind::Instant => start.is_some() && end.is_none(),
            ObservationTimeKind::Interval => start.is_some() && end.is_some() && start != end,
            ObservationTimeKind::Approximate => start.is_some() || end.is_some(),
        };
        if !valid_shape {
            return Err("observation time values do not match declared kind".into());
        }
        Ok(Self { kind, start, end })
    }

    pub fn kind(&self) -> ObservationTimeKind { self.kind }
    pub fn start(&self) -> Option<&str> { self.start.as_deref() }
    pub fn end(&self) -> Option<&str> { self.end.as_deref() }
}

impl<'de> Deserialize<'de> for ObservationTime {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de>,
    {
        let wire = ObservationTimeWire::deserialize(deserializer)?;
        Self::new(wire.kind, wire.start, wire.end).map_err(serde::de::Error::custom)
    }
}

/// Whether licence terms were reviewed for the intended use.
/// This is metadata, not legal advice or a claim of source accuracy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LicenceReview { Unknown, NotReviewed, ReusableForDeclaredUse, Restricted }

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LicenceMetadata {
    identifier: Option<String>,
    terms_locator: Option<String>,
    attribution: Option<String>,
    review: LicenceReview,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LicenceMetadataWire {
    identifier: Option<String>,
    terms_locator: Option<String>,
    attribution: Option<String>,
    review: LicenceReview,
}

impl LicenceMetadata {
    pub fn new(
        identifier: Option<String>,
        terms_locator: Option<String>,
        attribution: Option<String>,
        review: LicenceReview,
    ) -> Result<Self, String> {
        for (label, value) in [
            ("licence identifier", identifier.as_deref()),
            ("licence terms locator", terms_locator.as_deref()),
            ("attribution", attribution.as_deref()),
        ] {
            if value.is_some_and(|s| s.trim().is_empty()) {
                return Err(format!("{label} must be absent or non-empty"));
            }
        }
        Ok(Self { identifier, terms_locator, attribution, review })
    }

    pub fn unknown() -> Self {
        Self { identifier: None, terms_locator: None, attribution: None, review: LicenceReview::Unknown }
    }

    pub fn identifier(&self) -> Option<&str> { self.identifier.as_deref() }
    pub fn terms_locator(&self) -> Option<&str> { self.terms_locator.as_deref() }
    pub fn attribution(&self) -> Option<&str> { self.attribution.as_deref() }
    pub fn review(&self) -> LicenceReview { self.review }
}

impl<'de> Deserialize<'de> for LicenceMetadata {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de>,
    {
        let wire = LicenceMetadataWire::deserialize(deserializer)?;
        Self::new(wire.identifier, wire.terms_locator, wire.attribution, wire.review)
            .map_err(serde::de::Error::custom)
    }
}

/// Units explicitly declared by the source for spatial resolution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResolutionUnit { Meter, Centimeter, Foot, Degree, ArcSecond }

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SpatialResolution { value: f64, unit: ResolutionUnit }

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SpatialResolutionWire { value: f64, unit: ResolutionUnit }

impl SpatialResolution {
    pub fn new(value: f64, unit: ResolutionUnit) -> Result<Self, String> {
        if !value.is_finite() || value <= 0.0 {
            return Err("spatial resolution must be finite and greater than zero".into());
        }
        Ok(Self { value, unit })
    }
    pub fn value(&self) -> f64 { self.value }
    pub fn unit(&self) -> ResolutionUnit { self.unit }
}

impl<'de> Deserialize<'de> for SpatialResolution {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de>,
    {
        let wire = SpatialResolutionWire::deserialize(deserializer)?;
        Self::new(wire.value, wire.unit).map_err(serde::de::Error::custom)
    }
}

/// A provider locator and optional provider identity.
///
/// The locator is intentionally opaque: it may be a URL or provider-defined ID.
/// Constructing one does not cause network access or verify the locator.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AssetSource { provider_id: Option<String>, locator: String }

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AssetSourceWire { provider_id: Option<String>, locator: String }

impl AssetSource {
    pub fn new(provider_id: Option<String>, locator: impl Into<String>) -> Result<Self, String> {
        let locator = locator.into();
        if locator.trim().is_empty() {
            return Err("source locator must not be empty".into());
        }
        if provider_id.as_deref().is_some_and(|s| s.trim().is_empty()) {
            return Err("provider ID must be absent or non-empty".into());
        }
        Ok(Self { provider_id, locator })
    }
    pub fn provider_id(&self) -> Option<&str> { self.provider_id.as_deref() }
    pub fn locator(&self) -> &str { &self.locator }
}

impl<'de> Deserialize<'de> for AssetSource {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de>,
    {
        let wire = AssetSourceWire::deserialize(deserializer)?;
        Self::new(wire.provider_id, wire.locator).map_err(serde::de::Error::custom)
    }
}

/// A single catalog entry. Private fields ensure every construction path validates.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AssetRecord {
    schema_version: u32,
    asset_id: AssetId,
    title: String,
    kind: AssetKind,
    source: AssetSource,
    media_type: Option<String>,
    source_crs: Option<String>,
    footprint: Option<GeoFootprint>,
    observation_time: ObservationTime,
    licence: LicenceMetadata,
    spatial_resolution: Option<SpatialResolution>,
    published_at: Option<String>,
    /// Provider/source snapshot ID, not proof that bytes were captured.
    source_snapshot_id: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AssetRecordWire {
    schema_version: u32,
    asset_id: AssetId,
    title: String,
    kind: AssetKind,
    source: AssetSource,
    media_type: Option<String>,
    source_crs: Option<String>,
    footprint: Option<GeoFootprint>,
    observation_time: ObservationTime,
    licence: LicenceMetadata,
    spatial_resolution: Option<SpatialResolution>,
    published_at: Option<String>,
    source_snapshot_id: Option<String>,
}

impl AssetRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        asset_id: AssetId,
        title: impl Into<String>,
        kind: AssetKind,
        source: AssetSource,
        media_type: Option<String>,
        source_crs: Option<String>,
        footprint: Option<GeoFootprint>,
        observation_time: ObservationTime,
        licence: LicenceMetadata,
        spatial_resolution: Option<SpatialResolution>,
        published_at: Option<String>,
        source_snapshot_id: Option<String>,
    ) -> Result<Self, String> {
        let title = title.into();
        if title.trim().is_empty() {
            return Err("asset title must not be empty".into());
        }
        for (label, value) in [
            ("media type", media_type.as_deref()),
            ("source CRS", source_crs.as_deref()),
            ("publication time", published_at.as_deref()),
            ("source snapshot ID", source_snapshot_id.as_deref()),
        ] {
            if value.is_some_and(|s| s.trim().is_empty()) {
                return Err(format!("{label} must be absent or non-empty"));
            }
        }
        Ok(Self {
            schema_version: ASSET_SCHEMA_VERSION,
            asset_id, title, kind, source, media_type, source_crs, footprint, observation_time, licence,
            spatial_resolution, published_at, source_snapshot_id,
        })
    }

    pub fn schema_version(&self) -> u32 { self.schema_version }
    pub fn asset_id(&self) -> &AssetId { &self.asset_id }
    pub fn title(&self) -> &str { &self.title }
    pub fn kind(&self) -> AssetKind { self.kind }
    pub fn source(&self) -> &AssetSource { &self.source }
    pub fn media_type(&self) -> Option<&str> { self.media_type.as_deref() }
    pub fn source_crs(&self) -> Option<&str> { self.source_crs.as_deref() }
    pub fn footprint(&self) -> Option<&GeoFootprint> { self.footprint.as_ref() }
    pub fn observation_time(&self) -> &ObservationTime { &self.observation_time }
    pub fn licence(&self) -> &LicenceMetadata { &self.licence }
    pub fn spatial_resolution(&self) -> Option<&SpatialResolution> { self.spatial_resolution.as_ref() }
    pub fn published_at(&self) -> Option<&str> { self.published_at.as_deref() }
    pub fn source_snapshot_id(&self) -> Option<&str> { self.source_snapshot_id.as_deref() }
}

impl<'de> Deserialize<'de> for AssetRecord {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de>,
    {
        let wire = AssetRecordWire::deserialize(deserializer)?;
        if wire.schema_version != ASSET_SCHEMA_VERSION {
            return Err(serde::de::Error::custom(format!(
                "unsupported asset schema version {}; expected {}",
                wire.schema_version, ASSET_SCHEMA_VERSION
            )));
        }
        Self::new(
            wire.asset_id, wire.title, wire.kind, wire.source, wire.media_type, wire.source_crs,
            wire.footprint, wire.observation_time, wire.licence, wire.spatial_resolution,
            wire.published_at, wire.source_snapshot_id,
        ).map_err(serde::de::Error::custom)
    }
}

/// Deterministically normalized catalogue. Duplicate semantic asset IDs fail closed.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AssetCatalogV1 { schema_version: u32, assets: Vec<AssetRecord> }

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AssetCatalogWire { schema_version: u32, assets: Vec<AssetRecord> }

impl AssetCatalogV1 {
    pub fn new(mut assets: Vec<AssetRecord>) -> Result<Self, String> {
        let mut seen = BTreeSet::new();
        for asset in &assets {
            if !seen.insert(asset.asset_id().clone()) {
                return Err(format!("duplicate asset ID '{}'", asset.asset_id().as_str()));
            }
        }
        assets.sort_by(|a, b| a.asset_id().cmp(b.asset_id()));
        Ok(Self { schema_version: ASSET_SCHEMA_VERSION, assets })
    }

    pub fn schema_version(&self) -> u32 { self.schema_version }
    pub fn assets(&self) -> &[AssetRecord] { &self.assets }
    pub fn len(&self) -> usize { self.assets.len() }
    pub fn is_empty(&self) -> bool { self.assets.is_empty() }
    pub fn get(&self, id: &AssetId) -> Option<&AssetRecord> {
        self.assets.binary_search_by(|record| record.asset_id().cmp(id))
            .ok().map(|index| &self.assets[index])
    }
}

impl<'de> Deserialize<'de> for AssetCatalogV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where D: Deserializer<'de>,
    {
        let wire = AssetCatalogWire::deserialize(deserializer)?;
        if wire.schema_version != ASSET_SCHEMA_VERSION {
            return Err(serde::de::Error::custom(format!(
                "unsupported catalogue schema version {}; expected {}",
                wire.schema_version, ASSET_SCHEMA_VERSION
            )));
        }
        Self::new(wire.assets).map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_asset(id: &str) -> AssetRecord {
        AssetRecord::new(
            AssetId::new(id).unwrap(),
            format!("Fixture {id}"),
            AssetKind::Imagery,
            AssetSource::new(Some("fixture-provider".into()), "fixture:scene-1").unwrap(),
            Some("image/tiff; application=geotiff".into()),
            Some("EPSG:4326".into()),
            Some(GeoFootprint::new(16.0, -35.0, 19.0, -33.0).unwrap()),
            ObservationTime::new(
                ObservationTimeKind::Interval,
                Some("2025-01-01".into()),
                Some("2025-01-31".into()),
            ).unwrap(),
            LicenceMetadata::new(
                Some("CC-BY-4.0".into()),
                Some("https://creativecommons.org/licenses/by/4.0/".into()),
                Some("Fixture Provider".into()),
                LicenceReview::ReusableForDeclaredUse,
            ).unwrap(),
            Some(SpatialResolution::new(10.0, ResolutionUnit::Meter).unwrap()),
            Some("2025-02-01".into()),
            Some(format!("snapshot-{id}")),
        ).unwrap()
    }

    #[test]
    fn asset_ids_are_validated_during_deserialization() {
        assert!(AssetId::new(" ").is_err());
        assert!(AssetId::new("../secret").is_err());
        assert!(serde_json::from_str::<AssetId>("").is_err());
        assert!(serde_json::from_str::<AssetId>("\"bad/id\"").is_err());
        assert_eq!(
            serde_json::from_str::<AssetId>("\"imagery:coast-1\"").unwrap().as_str(),
            "imagery:coast-1"
        );
    }

    #[test]
    fn footprint_accepts_antimeridian_crossing_and_computes_span() {
        let bbox = GeoFootprint::new(170.0, -10.0, -170.0, 10.0).unwrap();
        assert!(bbox.crosses_antimeridian());
        assert!((bbox.longitude_span_degrees() - 20.0).abs() < f64::EPSILON);
        assert!(GeoFootprint::new(-181.0, -10.0, 170.0, 10.0).is_err());
        assert!(GeoFootprint::new(10.0, 5.0, 10.0, 8.0).is_err());
        assert!(GeoFootprint::new(180.0, -10.0, -180.0, 10.0).is_err());
        assert!(GeoFootprint::new(0.0, 90.0, 20.0, 91.0).is_err());
        assert!(serde_json::from_str::<GeoFootprint>(
            r#"{"west":0,"south":91,"east":1,"north":92}"#
        ).is_err());
    }

    #[test]
    fn observation_time_preserves_unknown_and_range_semantics() {
        assert_eq!(ObservationTime::unknown().kind(), ObservationTimeKind::Unknown);
        assert!(ObservationTime::new(
            ObservationTimeKind::Unknown, Some("2025".into()), None
        ).is_err());
        assert!(ObservationTime::new(
            ObservationTimeKind::Interval, Some("2025".into()), Some("2025".into())
        ).is_err());
        let time = ObservationTime::new(
            ObservationTimeKind::Approximate, Some("circa 2020".into()), None
        ).unwrap();
        assert_eq!(time.start(), Some("circa 2020"));
    }

    #[test]
    fn catalogue_is_sorted_and_rejects_duplicate_semantic_ids() {
        let catalog = AssetCatalogV1::new(vec![fixture_asset("z"), fixture_asset("a")]).unwrap();
        assert_eq!(catalog.assets()[0].asset_id().as_str(), "a");
        assert_eq!(catalog.assets()[1].asset_id().as_str(), "z");
        assert!(AssetCatalogV1::new(vec![fixture_asset("same"), fixture_asset("same")]).is_err());
        assert_eq!(catalog.get(&AssetId::new("z").unwrap()).unwrap().title(), "Fixture z");
    }

    #[test]
    fn same_source_locator_can_have_distinct_asset_ids_and_snapshots() {
        let mut first = fixture_asset("snapshot-a");
        let mut second = fixture_asset("snapshot-b");
        assert_eq!(first.source().locator(), second.source().locator());
        assert_ne!(first.asset_id(), second.asset_id());
        first.source_snapshot_id = Some("capture-a".into());
        second.source_snapshot_id = Some("capture-b".into());
        let catalog = AssetCatalogV1::new(vec![first, second]).unwrap();
        assert_eq!(catalog.len(), 2);
    }

    #[test]
    fn catalogue_round_trip_is_deterministic() {
        let a = AssetCatalogV1::new(vec![fixture_asset("b"), fixture_asset("a")]).unwrap();
        let b = AssetCatalogV1::new(vec![fixture_asset("a"), fixture_asset("b")]).unwrap();
        let json_a = serde_json::to_string(&a).unwrap();
        let json_b = serde_json::to_string(&b).unwrap();
        assert_eq!(json_a, json_b);
        let decoded: AssetCatalogV1 = serde_json::from_str(&json_a).unwrap();
        assert_eq!(decoded, a);
    }

    #[test]
    fn asset_record_deserialization_rejects_invalid_version_and_blank_title() {
        let valid = serde_json::to_value(fixture_asset("valid")).unwrap();
        let mut wrong_version = valid.clone();
        wrong_version["schema_version"] = serde_json::json!(99);
        assert!(serde_json::from_value::<AssetRecord>(wrong_version).is_err());
        let mut blank_title = valid;
        blank_title["title"] = serde_json::json!("   ");
        assert!(serde_json::from_value::<AssetRecord>(blank_title).is_err());
    }

    #[test]
    fn unknown_licence_and_resolution_are_not_fabricated() {
        let licence = LicenceMetadata::unknown();
        assert_eq!(licence.review(), LicenceReview::Unknown);
        assert!(licence.identifier().is_none());
        assert!(SpatialResolution::new(f64::NAN, ResolutionUnit::Meter).is_err());
        assert!(SpatialResolution::new(0.0, ResolutionUnit::Meter).is_err());
        assert!(serde_json::from_str::<SpatialResolution>(
            r#"{"value":-1.0,"unit":"meter"}"#
        ).is_err());
    }
}
