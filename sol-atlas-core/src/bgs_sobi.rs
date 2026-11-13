// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! BGS Single Onshore Borehole Index (SOBI) OGC API Features adapter profile.
//!
//! This parses and qualifies the source-index envelope only. A SOBI feature is
//! a borehole-record index entry, not a digital log, lithology interval,
//! calibrated measurement, or statement of subsurface absence. Related scan
//! and AGS links are discovered but never fetched by this module.
//!
//! Source profile reviewed 2026-11-13:
//! https://ogcapi.bgs.ac.uk/collections/onshoreboreholeindex?f=html

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BgsSobiFeatureCollection {
    #[serde(rename = "type")]
    pub collection_type: String,
    pub features: Vec<BgsSobiFeature>,
    #[serde(rename = "numberReturned")]
    pub number_returned: Option<usize>,
    #[serde(rename = "numberMatched")]
    pub number_matched: Option<usize>,
    #[serde(rename = "timeStamp")]
    pub timestamp: Option<String>,
    #[serde(default)]
    pub links: Vec<BgsSobiLink>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BgsSobiFeature {
    #[serde(rename = "type")]
    pub feature_type: String,
    pub id: Option<Value>,
    pub properties: BgsSobiProperties,
    pub geometry: Option<BgsSobiPointGeometry>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BgsSobiProperties {
    pub reference: Option<String>,
    pub name: Option<String>,
    pub grid_ref: Option<String>,
    pub easting: Option<f64>,
    pub northing: Option<f64>,
    /// Preserve the provider label verbatim; this is not an independently
    /// verified positional standard deviation or survey accuracy statement.
    pub precision: Option<String>,
    /// Preserve the raw index field. Non-positive values remain uninterpreted
    /// until the source's value dictionary is qualified.
    pub length: Option<f64>,
    pub year_known: Option<String>,
    pub sitereport: Option<String>,
    pub held_at: Option<String>,
    pub id: Option<f64>,
    pub length_scan_cat: Option<String>,
    pub bgs_id: Option<f64>,
    /// Source-record metadata update time, not drilling/observation time.
    pub date_updated: Option<String>,
    pub scan_url: Option<String>,
    pub ags_log_url: Option<String>,
    pub water_well_ref: Option<String>,
    pub scan_quality: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct BgsSobiPointGeometry {
    #[serde(rename = "type")]
    pub geometry_type: String,
    /// BGS GeoJSON responses use CRS84 longitude, latitude ordering.
    pub coordinates: Vec<f64>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BgsSobiLink {
    pub rel: Option<String>,
    pub href: String,
    #[serde(rename = "type")]
    pub media_type: Option<String>,
    pub title: Option<String>,
    #[serde(flatten)]
    pub extra: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BgsSobiLengthStatus {
    Missing,
    NonPositiveCodeUninterpreted,
    PositiveIndexValue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BgsSobiRelatedArtifactKind {
    Scan,
    AgsLog,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BgsSobiArtifactRightsStatus {
    Unqualified,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BgsSobiRelatedArtifact {
    pub kind: BgsSobiRelatedArtifactKind,
    pub uri: String,
    /// A discovery link does not establish a right to fetch, process, derive,
    /// retain, or redistribute the linked artifact.
    pub rights_status: BgsSobiArtifactRightsStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BgsSobiValidationError {
    WrongCollectionType,
    ReturnedCountMismatch,
    MatchedCountLessThanReturned,
    WrongFeatureType,
    MissingStableRecordIdentity,
    MissingGeometry,
    WrongGeometryType,
    UnsupportedCoordinateDimension,
    NonFiniteCoordinate,
    LongitudeOutOfRange,
    LatitudeOutOfRange,
}

impl BgsSobiProperties {
    /// The index length is not automatically a qualified depth. Positive
    /// provider values remain as reported; non-positive codes stay uninterpreted.
    pub fn length_status(&self) -> BgsSobiLengthStatus {
        match self.length {
            None => BgsSobiLengthStatus::Missing,
            Some(value) if value <= 0.0 => BgsSobiLengthStatus::NonPositiveCodeUninterpreted,
            Some(_) => BgsSobiLengthStatus::PositiveIndexValue,
        }
    }
}

impl BgsSobiFeature {
    /// Prefer the BGS-specific ID; do not assume the reference label is unique.
    pub fn stable_record_key(&self) -> Option<String> {
        self.properties
            .bgs_id
            .map(|value| format!("bgs-id:{value:.0}"))
            .or_else(|| self.properties.id.map(|value| format!("sobi-id:{value:.0}")))
            .or_else(|| {
                self.id.as_ref().and_then(|value| match value {
                    Value::String(value) if !value.trim().is_empty() => {
                        Some(format!("feature-id:{value}"))
                    }
                    Value::Number(value) => Some(format!("feature-id:{value}")),
                    _ => None,
                })
            })
    }

    /// Related artifacts are emitted as unqualified links. No HTTP requests
    /// are made and no source/index licence is propagated to these artifacts.
    pub fn related_artifacts(&self) -> Vec<BgsSobiRelatedArtifact> {
        let mut artifacts = Vec::new();
        for (kind, uri) in [
            (BgsSobiRelatedArtifactKind::Scan, self.properties.scan_url.as_ref()),
            (BgsSobiRelatedArtifactKind::AgsLog, self.properties.ags_log_url.as_ref()),
        ] {
            if let Some(uri) = uri.filter(|uri| !uri.trim().is_empty()) {
                artifacts.push(BgsSobiRelatedArtifact {
                    kind,
                    uri: uri.clone(),
                    rights_status: BgsSobiArtifactRightsStatus::Unqualified,
                });
            }
        }
        artifacts
    }
}

impl BgsSobiFeatureCollection {
    /// Parse a BGS SOBI GeoJSON page and validate structural invariants.
    /// Unknown source properties are retained rather than silently discarded.
    pub fn parse_and_validate(bytes: &[u8]) -> Result<Self, BgsSobiParseError> {
        let collection: Self = serde_json::from_slice(bytes)
            .map_err(|error| BgsSobiParseError::Json(error.to_string()))?;
        let errors = collection.validate();
        if errors.is_empty() {
            Ok(collection)
        } else {
            Err(BgsSobiParseError::Validation(errors))
        }
    }

    pub fn validate(&self) -> Vec<BgsSobiValidationError> {
        let mut errors = Vec::new();
        if self.collection_type != "FeatureCollection" {
            errors.push(BgsSobiValidationError::WrongCollectionType);
        }
        if self.number_returned.is_some_and(|count| count != self.features.len()) {
            errors.push(BgsSobiValidationError::ReturnedCountMismatch);
        }
        if self
            .number_returned
            .zip(self.number_matched)
            .is_some_and(|(returned, matched)| matched < returned)
        {
            errors.push(BgsSobiValidationError::MatchedCountLessThanReturned);
        }

        for feature in &self.features {
            if feature.properties.scan_url.as_ref().is_some_and(|uri| uri.trim().is_empty())
                || feature.properties.ags_log_url.as_ref().is_some_and(|uri| uri.trim().is_empty())
            {
                errors.push(BgsSobiValidationError::EmptyRelatedArtifactUri);
            }
            if feature.feature_type != "Feature" {
                errors.push(BgsSobiValidationError::WrongFeatureType);
            }
            if feature.stable_record_key().is_none() {
                errors.push(BgsSobiValidationError::MissingStableRecordIdentity);
            }

            match feature.geometry.as_ref() {
                None => errors.push(BgsSobiValidationError::MissingGeometry),
                Some(geometry) => {
                    if geometry.geometry_type != "Point" {
                        errors.push(BgsSobiValidationError::WrongGeometryType);
                    }
                    if geometry.coordinates.len() != 2 {
                        errors.push(BgsSobiValidationError::UnsupportedCoordinateDimension);
                        continue;
                    }
                    let longitude = geometry.coordinates[0];
                    let latitude = geometry.coordinates[1];
                    if !longitude.is_finite() || !latitude.is_finite() {
                        errors.push(BgsSobiValidationError::NonFiniteCoordinate);
                        continue;
                    }
                    if !(-180.0..=180.0).contains(&longitude) {
                        errors.push(BgsSobiValidationError::LongitudeOutOfRange);
                    }
                    if !(-90.0..=90.0).contains(&latitude) {
                        errors.push(BgsSobiValidationError::LatitudeOutOfRange);
                    }
                }
            }
        }
        errors
    }

    /// A matched-result count is a page/query statistic, not a completeness
    /// assertion for SOBI or for the national borehole inventory.
    pub fn matched_count_snapshot(&self) -> Option<usize> {
        self.number_matched
    }

    /// Follow only the exact next URL returned by the source. Callers must
    /// still apply BGS fair-use/rate limits and keep queries bounded.
    pub fn next_page_url(&self) -> Option<&str> {
        self.links
            .iter()
            .find(|link| link.rel.as_deref() == Some("next"))
            .map(|link| link.href.as_str())
    }

    /// Server timestamp is response time, not the borehole observation date.
    pub fn source_self_url(&self) -> Option<&str> {
        self.links
            .iter()
            .find(|link| link.rel.as_deref() == Some("self"))
            .map(|link| link.href.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SYNTHETIC_PAGE: &str = include_str!(
        "../../research/earth-observability/fixtures/bgs-sobi-ogc-api-synthetic-v0.1.json"
    );

    #[test]
    fn synthetic_collection_parses_and_preserves_page_metadata() {
        let page = BgsSobiFeatureCollection::parse_and_validate(SYNTHETIC_PAGE.as_bytes())
            .expect("synthetic BGS page parses");
        assert_eq!(page.collection_type, "FeatureCollection");
        assert_eq!(page.number_returned, Some(2));
        assert_eq!(page.matched_count_snapshot(), Some(250));
        assert!(page.next_page_url().unwrap().contains("offset=2"));
        assert_eq!(page.features.len(), 2);
    }

    #[test]
    fn geojson_coordinates_are_interpreted_as_longitude_then_latitude() {
        let page = BgsSobiFeatureCollection::parse_and_validate(SYNTHETIC_PAGE.as_bytes())
            .expect("synthetic BGS page parses");
        let point = page.features[0].geometry.as_ref().unwrap();
        assert_eq!(point.coordinates, vec![0.125, 0.25]);
    }

    #[test]
    fn source_precision_label_is_preserved_not_recast_as_sigma() {
        let page = BgsSobiFeatureCollection::parse_and_validate(SYNTHETIC_PAGE.as_bytes())
            .expect("synthetic BGS page parses");
        assert_eq!(page.features[0].properties.precision.as_deref(), Some("± METRE"));
    }

    #[test]
    fn non_positive_length_code_is_not_zero_filled_or_called_depth() {
        let page = BgsSobiFeatureCollection::parse_and_validate(SYNTHETIC_PAGE.as_bytes())
            .expect("synthetic BGS page parses");
        let record = &page.features[1];
        assert_eq!(record.properties.length, Some(-2.0));
        assert_eq!(
            record.properties.length_status(),
            BgsSobiLengthStatus::NonPositiveCodeUninterpreted
        );
        assert_eq!(record.properties.year_known, None);
    }

    #[test]
    fn scan_and_ags_links_remain_separate_unqualified_artifacts() {
        let page = BgsSobiFeatureCollection::parse_and_validate(SYNTHETIC_PAGE.as_bytes())
            .expect("synthetic BGS page parses");
        let artifacts = page.features[0].related_artifacts();
        assert_eq!(artifacts.len(), 1);
        assert_eq!(artifacts[0].kind, BgsSobiRelatedArtifactKind::Scan);
        assert_eq!(
            artifacts[0].rights_status,
            BgsSobiArtifactRightsStatus::Unqualified
        );
    }

    #[test]
    fn unknown_future_fields_are_preserved() {
        let page = BgsSobiFeatureCollection::parse_and_validate(SYNTHETIC_PAGE.as_bytes())
            .expect("synthetic BGS page parses");
        assert!(page.extra.contains_key("synthetic_fixture_marker"));
        assert!(page.features[0].properties.extra.contains_key("future_api_field"));
    }

    #[test]
    fn invalid_coordinate_or_missing_identity_is_rejected() {
        let mut page: BgsSobiFeatureCollection =
            serde_json::from_str(SYNTHETIC_PAGE).expect("synthetic fixture parses");
        page.features[0].geometry.as_mut().unwrap().coordinates[0] = 181.0;
        assert!(page.validate().contains(&BgsSobiValidationError::LongitudeOutOfRange));

        page.features[0].geometry.as_mut().unwrap().coordinates[0] = 0.125;
        page.features[0].properties.bgs_id = None;
        page.features[0].properties.id = None;
        page.features[0].id = None;
        assert!(
            page.validate()
                .contains(&BgsSobiValidationError::MissingStableRecordIdentity)
        );
    }

    #[test]
    fn declared_return_count_must_match_page_features() {
        let mut page: BgsSobiFeatureCollection =
            serde_json::from_str(SYNTHETIC_PAGE).expect("synthetic fixture parses");
        page.number_returned = Some(99);
        assert!(page.validate().contains(&BgsSobiValidationError::ReturnedCountMismatch));
    }
}