// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Provider-aware discovery and normalization for optical satellite scenes.
//!
//! This module builds STAC search requests and parses returned STAC metadata;
//! it does not fetch imagery or claim that a scene has been radiometrically,
//! geometrically, or scientifically validated. Raster processing belongs in
//! a separate stage that records its own provenance.

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::fmt;

pub const COPERNICUS_STAC_SEARCH_URL: &str = "https://stac.dataspace.copernicus.eu/v1/search";
pub const USGS_LANDSAT_STAC_SEARCH_URL: &str =
    "https://landsatlook.usgs.gov/stac-server/search";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SatelliteMission {
    Sentinel2L2a,
    Landsat8Collection2Level2,
}

impl SatelliteMission {
    pub fn provider(self) -> SatelliteProvider {
        match self {
            Self::Sentinel2L2a => SatelliteProvider::CopernicusDataSpace,
            Self::Landsat8Collection2Level2 => SatelliteProvider::UsgsLandsatLook,
        }
    }

    pub fn collection_id(self) -> &'static str {
        match self {
            Self::Sentinel2L2a => "sentinel-2-l2a",
            Self::Landsat8Collection2Level2 => "landsat-c2l2-sr",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Sentinel2L2a => "Sentinel-2 Level-2A surface reflectance",
            Self::Landsat8Collection2Level2 => {
                "Landsat 8 Collection 2 Level-2 surface reflectance"
            }
        }
    }

    pub fn search_url(self) -> &'static str {
        match self.provider() {
            SatelliteProvider::CopernicusDataSpace => COPERNICUS_STAC_SEARCH_URL,
            SatelliteProvider::UsgsLandsatLook => USGS_LANDSAT_STAC_SEARCH_URL,
        }
    }

    fn matches_item(self, id: &str, platform: Option<&str>) -> bool {
        let id_upper = id.to_ascii_uppercase();
        let platform_lower = platform.unwrap_or_default().to_ascii_lowercase();
        match self {
            Self::Sentinel2L2a => {
                platform_lower.contains("sentinel-2")
                    || id_upper.starts_with("S2A")
                    || id_upper.starts_with("S2B")
                    || id_upper.starts_with("S2C")
            }
            Self::Landsat8Collection2Level2 => {
                platform_lower == "landsat-8"
                    || platform_lower == "landsat_8"
                    || id_upper.starts_with("LC08_")
                    || id_upper.starts_with("LC08-")
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SatelliteProvider {
    CopernicusDataSpace,
    UsgsLandsatLook,
}

/// WGS84 geographic bounding box in longitude/latitude order.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GeographicBbox {
    pub west: f64,
    pub south: f64,
    pub east: f64,
    pub north: f64,
}

impl GeographicBbox {
    pub fn validate(self) -> Result<(), SatelliteCatalogError> {
        let values = [self.west, self.south, self.east, self.north];
        if values.iter().any(|value| !value.is_finite())
            || self.west < -180.0
            || self.west > 180.0
            || self.east < -180.0
            || self.east > 180.0
            || self.south < -90.0
            || self.south > 90.0
            || self.north < -90.0
            || self.north > 90.0
            || self.west >= self.east
            || self.south >= self.north
        {
            return Err(SatelliteCatalogError::InvalidBoundingBox);
        }
        Ok(())
    }

    pub fn as_stac_array(self) -> [f64; 4] {
        [self.west, self.south, self.east, self.north]
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DateRange {
    pub start: String,
    pub end: String,
}

impl DateRange {
    pub fn new(
        start: impl Into<String>,
        end: impl Into<String>,
    ) -> Result<Self, SatelliteCatalogError> {
        let value = Self {
            start: start.into(),
            end: end.into(),
        };
        if !is_valid_date(&value.start) || !is_valid_date(&value.end) || value.start > value.end {
            return Err(SatelliteCatalogError::InvalidDateRange);
        }
        Ok(value)
    }

    pub fn as_stac_interval(&self) -> String {
        format!("{}/{}", self.start, self.end)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SatelliteSearchSpec {
    pub mission: SatelliteMission,
    pub bbox: GeographicBbox,
    pub dates: DateRange,
    /// Client-side filter, because provider query extensions may differ.
    pub maximum_cloud_cover_pct: u8,
    pub limit: u16,
}

impl SatelliteSearchSpec {
    pub fn validate(&self) -> Result<(), SatelliteCatalogError> {
        self.bbox.validate()?;
        DateRange::new(self.dates.start.clone(), self.dates.end.clone())?;
        if self.maximum_cloud_cover_pct > 100 {
            return Err(SatelliteCatalogError::InvalidCloudCoverLimit);
        }
        if !(1..=500).contains(&self.limit) {
            return Err(SatelliteCatalogError::InvalidSearchLimit);
        }
        Ok(())
    }

    /// Build the STAC API Item Search request body. Cloud-cover filtering is
    /// retained as explicit client policy rather than assuming that optional
    /// provider query extensions behave identically.
    pub fn request_body_json(&self) -> Result<String, SatelliteCatalogError> {
        self.validate()?;
        serde_json::to_string(&json!({
            "collections": [self.mission.collection_id()],
            "bbox": self.bbox.as_stac_array(),
            "datetime": self.dates.as_stac_interval(),
            "limit": self.limit,
        }))
        .map_err(|_| SatelliteCatalogError::SerializationFailed)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CloudAssessment {
    WithinLimit,
    AboveLimit,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SatelliteAsset {
    pub href: String,
    pub media_type: Option<String>,
    pub roles: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SatelliteScene {
    pub mission: SatelliteMission,
    pub provider: SatelliteProvider,
    pub collection_id: String,
    pub item_id: String,
    pub item_url: Option<String>,
    /// Preserved from STAC properties; not rewritten to ingestion time.
    pub acquired_at: String,
    pub cloud_cover_pct: Option<f64>,
    pub assets: BTreeMap<String, SatelliteAsset>,
}

impl SatelliteScene {
    pub fn cloud_assessment(&self, maximum_pct: u8) -> CloudAssessment {
        match self.cloud_cover_pct {
            Some(value) if value <= f64::from(maximum_pct) => CloudAssessment::WithinLimit,
            Some(_) => CloudAssessment::AboveLimit,
            None => CloudAssessment::Unknown,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SatelliteCatalogError {
    InvalidBoundingBox,
    InvalidDateRange,
    InvalidCloudCoverLimit,
    InvalidSearchLimit,
    SerializationFailed,
    InvalidJson,
    MissingFeatures,
    InvalidFeature,
    InvalidItemId,
    MissingAcquisitionTime,
    InvalidCloudCover,
}

impl fmt::Display for SatelliteCatalogError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for SatelliteCatalogError {}

/// Parse a STAC ItemCollection, retaining only items from the requested
/// mission. Non-matching platforms (notably Landsat 9 in the Landsat 8/9
/// collection) are ignored, not mislabeled.
pub fn parse_stac_items(
    json_text: &str,
    mission: SatelliteMission,
) -> Result<Vec<SatelliteScene>, SatelliteCatalogError> {
    let root: Value =
        serde_json::from_str(json_text).map_err(|_| SatelliteCatalogError::InvalidJson)?;
    let features = root
        .get("features")
        .and_then(Value::as_array)
        .ok_or(SatelliteCatalogError::MissingFeatures)?;

    let mut scenes = Vec::new();
    for feature in features {
        let id = feature
            .get("id")
            .and_then(Value::as_str)
            .ok_or(SatelliteCatalogError::InvalidItemId)?;
        let properties = feature
            .get("properties")
            .and_then(Value::as_object)
            .ok_or(SatelliteCatalogError::InvalidFeature)?;
        let platform = properties.get("platform").and_then(Value::as_str);
        if !mission.matches_item(id, platform) {
            continue;
        }

        let acquired_at = properties
            .get("datetime")
            .or_else(|| properties.get("start_datetime"))
            .and_then(Value::as_str)
            .filter(|text| !text.trim().is_empty())
            .ok_or(SatelliteCatalogError::MissingAcquisitionTime)?
            .to_owned();

        let cloud_value = properties
            .get("eo:cloud_cover")
            .or_else(|| properties.get("cloud_cover"));
        let cloud_cover_pct = match cloud_value {
            None | Some(Value::Null) => None,
            Some(value) => {
                let cloud_cover = value
                    .as_f64()
                    .ok_or(SatelliteCatalogError::InvalidCloudCover)?;
                if !cloud_cover.is_finite() || !(0.0..=100.0).contains(&cloud_cover) {
                    return Err(SatelliteCatalogError::InvalidCloudCover);
                }
                Some(cloud_cover)
            }
        };

        let collection_id = feature
            .get("collection")
            .and_then(Value::as_str)
            .unwrap_or(mission.collection_id())
            .to_owned();

        let item_url = feature
            .get("links")
            .and_then(Value::as_array)
            .and_then(|links| {
                links.iter().find_map(|link| {
                    (link.get("rel").and_then(Value::as_str) == Some("self"))
                        .then(|| link.get("href").and_then(Value::as_str))
                        .flatten()
                })
            })
            .map(str::to_owned);

        let mut assets = BTreeMap::new();
        if let Some(asset_object) = feature.get("assets").and_then(Value::as_object) {
            for (key, value) in asset_object {
                let Some(href) = value.get("href").and_then(Value::as_str) else {
                    continue;
                };
                if href.trim().is_empty() {
                    continue;
                }
                let roles = value
                    .get("roles")
                    .and_then(Value::as_array)
                    .map(|items| {
                        items
                            .iter()
                            .filter_map(Value::as_str)
                            .map(str::to_owned)
                            .collect()
                    })
                    .unwrap_or_default();
                let media_type = value
                    .get("type")
                    .and_then(Value::as_str)
                    .map(str::to_owned);
                assets.insert(
                    key.clone(),
                    SatelliteAsset {
                        href: href.to_owned(),
                        media_type,
                        roles,
                    },
                );
            }
        }

        scenes.push(SatelliteScene {
            mission,
            provider: mission.provider(),
            collection_id,
            item_id: id.to_owned(),
            item_url,
            acquired_at,
            cloud_cover_pct,
            assets,
        });
    }

    Ok(scenes)
}

fn is_valid_date(text: &str) -> bool {
    let bytes = text.as_bytes();
    if bytes.len() != 10
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || !bytes
            .iter()
            .enumerate()
            .all(|(index, byte)| matches!(index, 4 | 7) || byte.is_ascii_digit())
    {
        return false;
    }

    let year = digits(&bytes[0..4]) as i32;
    let month = digits(&bytes[5..7]);
    let day = digits(&bytes[8..10]);
    if year == 0 || !(1..=12).contains(&month) {
        return false;
    }
    let leap_year = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let days_in_month = match month {
        2 if leap_year => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    };
    (1..=days_in_month).contains(&day)
}

fn digits(bytes: &[u8]) -> u32 {
    bytes
        .iter()
        .fold(0_u32, |value, byte| value * 10 + u32::from(byte - b'0'))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn spec(mission: SatelliteMission) -> SatelliteSearchSpec {
        SatelliteSearchSpec {
            mission,
            bbox: GeographicBbox {
                west: 27.0,
                south: -27.5,
                east: 28.5,
                north: -26.0,
            },
            dates: DateRange::new("2025-01-01", "2025-12-31").unwrap(),
            maximum_cloud_cover_pct: 20,
            limit: 100,
        }
    }

    #[test]
    fn sentinel_search_targets_copernicus_stac_and_correct_collection() {
        let search = spec(SatelliteMission::Sentinel2L2a);
        let body: Value = serde_json::from_str(&search.request_body_json().unwrap()).unwrap();
        assert_eq!(search.search_url(), COPERNICUS_STAC_SEARCH_URL);
        assert_eq!(body["collections"][0], "sentinel-2-l2a");
        assert_eq!(body["datetime"], "2025-01-01/2025-12-31");
        assert_eq!(body["limit"], 100);
    }

    #[test]
    fn landsat_search_targets_usgs_landsatlook_collection() {
        let search = spec(SatelliteMission::Landsat8Collection2Level2);
        let body: Value = serde_json::from_str(&search.request_body_json().unwrap()).unwrap();
        assert_eq!(search.search_url(), USGS_LANDSAT_STAC_SEARCH_URL);
        assert_eq!(body["collections"][0], "landsat-c2l2-sr");
    }

    #[test]
    fn rejects_invalid_bounding_boxes_dates_and_limits() {
        let mut search = spec(SatelliteMission::Sentinel2L2a);
        search.bbox.east = 181.0;
        assert_eq!(
            search.validate(),
            Err(SatelliteCatalogError::InvalidBoundingBox)
        );

        assert!(DateRange::new("2025-02-29", "2025-03-01").is_err());
        assert!(DateRange::new("2025-12-31", "2025-01-01").is_err());

        search = spec(SatelliteMission::Sentinel2L2a);
        search.maximum_cloud_cover_pct = 101;
        assert_eq!(
            search.validate(),
            Err(SatelliteCatalogError::InvalidCloudCoverLimit)
        );
    }

    #[test]
    fn parses_scene_metadata_and_preserves_acquisition_time() {
        let json = r#"{
            "type":"FeatureCollection",
            "features":[{
                "type":"Feature",
                "id":"S2A_MSIL2A_20251012T080001_N0000_R000_T35JPM_20251012T100001",
                "collection":"sentinel-2-l2a",
                "properties":{
                    "platform":"sentinel-2a",
                    "datetime":"2025-10-12T08:00:01Z",
                    "eo:cloud_cover":12.5
                },
                "links":[{"rel":"self","href":"https://catalog.example/items/scene-1"}],
                "assets":{
                    "B03":{
                        "href":"https://data.example/B03.tif",
                        "type":"image/tiff; application=geotiff; profile=cloud-optimized",
                        "roles":["data"]
                    }
                }
            }]
        }"#;
        let scenes = parse_stac_items(json, SatelliteMission::Sentinel2L2a).unwrap();
        assert_eq!(scenes.len(), 1);
        assert_eq!(scenes[0].acquired_at, "2025-10-12T08:00:01Z");
        assert_eq!(scenes[0].cloud_assessment(20), CloudAssessment::WithinLimit);
        assert_eq!(
            scenes[0].assets["B03"].href,
            "https://data.example/B03.tif"
        );
    }

    #[test]
    fn excludes_landsat_9_items_from_a_landsat_8_query_result() {
        let json = r#"{
            "features":[
                {
                    "id":"LC08_L2SP_171081_20251012_20251020_02_T1",
                    "collection":"landsat-c2l2-sr",
                    "properties":{
                        "platform":"landsat-8",
                        "datetime":"2025-10-12T08:00:00Z",
                        "eo:cloud_cover":4.0
                    },
                    "assets":{}
                },
                {
                    "id":"LC09_L2SP_171081_20251012_20251020_02_T1",
                    "collection":"landsat-c2l2-sr",
                    "properties":{
                        "platform":"landsat-9",
                        "datetime":"2025-10-12T08:00:00Z",
                        "eo:cloud_cover":3.0
                    },
                    "assets":{}
                }
            ]
        }"#;
        let scenes =
            parse_stac_items(json, SatelliteMission::Landsat8Collection2Level2).unwrap();
        assert_eq!(scenes.len(), 1);
        assert!(scenes[0].item_id.starts_with("LC08_"));
    }

    #[test]
    fn missing_cloud_metadata_is_unknown_not_zero() {
        let json = r#"{
            "features":[{
                "id":"LC08_L2SP_171081_20251012_20251020_02_T1",
                "collection":"landsat-c2l2-sr",
                "properties":{
                    "platform":"landsat-8",
                    "datetime":"2025-10-12T08:00:00Z"
                },
                "assets":{}
            }]
        }"#;
        let scenes =
            parse_stac_items(json, SatelliteMission::Landsat8Collection2Level2).unwrap();
        assert_eq!(scenes[0].cloud_cover_pct, None);
        assert_eq!(scenes[0].cloud_assessment(20), CloudAssessment::Unknown);
    }

    #[test]
    fn malformed_cloud_cover_is_rejected_not_silently_defaulted() {
        let json = r#"{
            "features":[{
                "id":"LC08_L2SP_171081_20251012_20251020_02_T1",
                "properties":{
                    "platform":"landsat-8",
                    "datetime":"2025-10-12T08:00:00Z",
                    "eo:cloud_cover":"unknown"
                }
            }]
        }"#;
        assert_eq!(
            parse_stac_items(json, SatelliteMission::Landsat8Collection2Level2),
            Err(SatelliteCatalogError::InvalidCloudCover)
        );
    }
}
