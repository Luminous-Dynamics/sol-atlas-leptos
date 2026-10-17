// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Bounded, provenance-preserving parser for the public USGS ComCat GeoJSON feed.
//! This module parses only; callers own transport, refresh scheduling, and UI.

use serde::Deserialize;
use std::fmt;

pub const USGS_M1_DAY_FEED_URL: &str =
    "https://earthquake.usgs.gov/earthquakes/feed/v1.0/summary/1.0_day.geojson";
pub const USGS_EVENT_PAGE_PREFIX: &str = "https://earthquake.usgs.gov/earthquakes/eventpage/";
pub const MAX_FEED_BYTES: usize = 8 * 1024 * 1024;
pub const MAX_FEATURES: usize = 20_000;
pub const MAX_FEED_AGE_MS: i64 = 10 * 60 * 1_000;
pub const MAX_FUTURE_SKEW_MS: i64 = 5 * 60 * 1_000;
/// The selected 1.0_day feed should contain events from roughly the previous day.
pub const MAX_EVENT_AGE_MS: i64 = 26 * 60 * 60 * 1_000;

#[derive(Debug, Clone, PartialEq)]
pub struct UsgsEarthquake {
    pub id: String,
    pub latitude: f64,
    pub longitude: f64,
    pub depth_km: Option<f64>,
    pub magnitude: f64,
    pub place: String,
    pub occurred_at_ms: i64,
    pub updated_at_ms: Option<i64>,
    /// Populated only for the official USGS event-page path.
    pub event_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct UsgsFeedSnapshot {
    /// Feed-side generation time, not the local download time.
    pub generated_at_ms: i64,
    pub fetched_at_ms: i64,
    pub skipped_features: usize,
    pub earthquakes: Vec<UsgsEarthquake>,
}

impl UsgsFeedSnapshot {
    pub fn is_fresh_at(&self, now_ms: i64) -> bool {
        self.generated_at_ms >= now_ms.saturating_sub(MAX_FEED_AGE_MS)
            && self.generated_at_ms <= now_ms.saturating_add(MAX_FUTURE_SKEW_MS)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UsgsFeedError {
    EmptyFeed,
    BodyTooLarge { actual: usize, maximum: usize },
    InvalidJson(String),
    InvalidRootType,
    MissingMetadata,
    UpstreamStatus(u16),
    MissingGeneratedTimestamp,
    InvalidFetchTimestamp,
    FutureDatedFeed,
    StaleFeed,
    TooManyFeatures { actual: usize, maximum: usize },
    NoValidFeatures,
}

impl fmt::Display for UsgsFeedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyFeed => {
                write!(f, "USGS live feed unexpectedly contained no features")
            }
            Self::BodyTooLarge { actual, maximum } => {
                write!(f, "USGS payload is {actual} bytes; limit is {maximum}")
            }
            Self::InvalidJson(error) => write!(f, "invalid USGS GeoJSON: {error}"),
            Self::InvalidRootType => {
                write!(f, "USGS payload is not a GeoJSON FeatureCollection")
            }
            Self::MissingMetadata => write!(f, "USGS payload has no feed metadata"),
            Self::UpstreamStatus(status) => write!(f, "USGS feed reported status {status}"),
            Self::MissingGeneratedTimestamp => {
                write!(f, "USGS feed has no generation timestamp")
            }
            Self::InvalidFetchTimestamp => write!(f, "local fetch timestamp is invalid"),
            Self::FutureDatedFeed => {
                write!(f, "USGS feed timestamp is too far in the future")
            }
            Self::StaleFeed => {
                write!(f, "USGS feed timestamp is outside the freshness window")
            }
            Self::TooManyFeatures { actual, maximum } => {
                write!(f, "USGS feed has {actual} features; limit is {maximum}")
            }
            Self::NoValidFeatures => {
                write!(f, "USGS feed contained features but none passed validation")
            }
        }
    }
}

impl std::error::Error for UsgsFeedError {}

#[derive(Deserialize)]
struct FeedWire {
    #[serde(rename = "type")]
    kind: String,
    metadata: Option<FeedMetadata>,
    features: Vec<FeatureWire>,
}

#[derive(Deserialize)]
struct FeedMetadata {
    generated: Option<i64>,
    status: Option<u16>,
}

#[derive(Deserialize)]
struct FeatureWire {
    #[serde(rename = "type")]
    kind: Option<String>,
    id: Option<String>,
    properties: Option<PropertiesWire>,
    geometry: Option<GeometryWire>,
}

#[derive(Deserialize)]
struct PropertiesWire {
    #[serde(default, alias = "magnitude")]
    mag: Option<f64>,
    #[serde(default)]
    place: Option<String>,
    #[serde(default)]
    time: Option<i64>,
    #[serde(default)]
    updated: Option<i64>,
    #[serde(default)]
    url: Option<String>,
    #[serde(rename = "type", default)]
    event_type: Option<String>,
}

#[derive(Deserialize)]
struct GeometryWire {
    #[serde(rename = "type")]
    kind: String,
    coordinates: Vec<f64>,
}

/// Parse a USGS summary feed. Bad features are skipped individually, while a
/// nonempty feed that yields no valid observations is rejected as schema drift.
pub fn parse_feed(json: &str, fetched_at_ms: i64) -> Result<UsgsFeedSnapshot, UsgsFeedError> {
    if json.len() > MAX_FEED_BYTES {
        return Err(UsgsFeedError::BodyTooLarge {
            actual: json.len(),
            maximum: MAX_FEED_BYTES,
        });
    }
    if fetched_at_ms <= 0 {
        return Err(UsgsFeedError::InvalidFetchTimestamp);
    }

    let wire: FeedWire = serde_json::from_str(json)
        .map_err(|error| UsgsFeedError::InvalidJson(error.to_string()))?;
    if wire.kind != "FeatureCollection" {
        return Err(UsgsFeedError::InvalidRootType);
    }

    let metadata = wire.metadata.ok_or(UsgsFeedError::MissingMetadata)?;
    if let Some(status) = metadata.status {
        if status != 200 {
            return Err(UsgsFeedError::UpstreamStatus(status));
        }
    }

    let generated_at_ms = metadata
        .generated
        .filter(|value| *value > 0)
        .ok_or(UsgsFeedError::MissingGeneratedTimestamp)?;

    if generated_at_ms > fetched_at_ms.saturating_add(MAX_FUTURE_SKEW_MS) {
        return Err(UsgsFeedError::FutureDatedFeed);
    }
    if generated_at_ms < fetched_at_ms.saturating_sub(MAX_FEED_AGE_MS) {
        return Err(UsgsFeedError::StaleFeed);
    }
    if wire.features.len() > MAX_FEATURES {
        return Err(UsgsFeedError::TooManyFeatures {
            actual: wire.features.len(),
            maximum: MAX_FEATURES,
        });
    }

    let feature_count = wire.features.len();
    if feature_count == 0 {
        return Err(UsgsFeedError::EmptyFeed);
    }

    let mut earthquakes = Vec::with_capacity(feature_count.min(2_000));
    let mut skipped_features = 0usize;

    for feature in wire.features {
        let Some(id) = feature.id.filter(|value| valid_identifier(value)) else {
            skipped_features += 1;
            continue;
        };

        if feature
            .kind
            .as_deref()
            .is_some_and(|value| value != "Feature")
        {
            skipped_features += 1;
            continue;
        }

        let (Some(properties), Some(geometry)) = (feature.properties, feature.geometry) else {
            skipped_features += 1;
            continue;
        };

        if geometry.kind != "Point"
            || properties
                .event_type
                .as_deref()
                .is_some_and(|value| value != "earthquake")
        {
            skipped_features += 1;
            continue;
        }

        let Some(magnitude) = properties
            .mag
            .filter(|value| value.is_finite() && (-3.0..=10.0).contains(value))
        else {
            skipped_features += 1;
            continue;
        };

        let Some(occurred_at_ms) = properties.time.filter(|value| *value > 0) else {
            skipped_features += 1;
            continue;
        };

        if occurred_at_ms < generated_at_ms.saturating_sub(MAX_EVENT_AGE_MS)
            || occurred_at_ms > generated_at_ms.saturating_add(MAX_FUTURE_SKEW_MS)
        {
            skipped_features += 1;
            continue;
        }

        let coordinates = &geometry.coordinates;
        if coordinates.len() < 2 {
            skipped_features += 1;
            continue;
        }

        let (longitude, latitude) = (coordinates[0], coordinates[1]);
        if !latitude.is_finite()
            || !longitude.is_finite()
            || !(-90.0..=90.0).contains(&latitude)
            || !(-180.0..=180.0).contains(&longitude)
        {
            skipped_features += 1;
            continue;
        }

        let place = properties
            .place
            .unwrap_or_else(|| "Unknown location".to_string());
        if place.trim().is_empty() || place.len() > 512 {
            skipped_features += 1;
            continue;
        }

        let depth_km = coordinates
            .get(2)
            .copied()
            .filter(|value| value.is_finite());
        let updated_at_ms = properties.updated.filter(|value| {
            *value >= occurred_at_ms && *value <= generated_at_ms.saturating_add(MAX_FUTURE_SKEW_MS)
        });
        let event_url = properties.url.filter(|value| valid_event_url(value, &id));

        earthquakes.push(UsgsEarthquake {
            id,
            latitude,
            longitude,
            depth_km,
            magnitude,
            place,
            occurred_at_ms,
            updated_at_ms,
            event_url,
        });
    }

    if feature_count > 0 && earthquakes.is_empty() {
        return Err(UsgsFeedError::NoValidFeatures);
    }

    Ok(UsgsFeedSnapshot {
        generated_at_ms,
        fetched_at_ms,
        skipped_features,
        earthquakes,
    })
}

fn valid_identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 200
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}

fn valid_event_url(value: &str, event_id: &str) -> bool {
    let Some(link_id) = value.strip_prefix(USGS_EVENT_PAGE_PREFIX) else {
        return false;
    };

    !link_id.is_empty()
        && link_id == event_id
        && value.len() <= 2_048
        && !value.chars().any(char::is_control)
        && !value.chars().any(char::is_whitespace)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeedHealth {
    Loading,
    Live,
    Degraded,
    Stale,
    Unavailable,
}

impl FeedHealth {
    pub fn label(self) -> &'static str {
        match self {
            Self::Loading => "USGS · CONNECTING",
            Self::Live => "USGS · LIVE",
            Self::Degraded => "USGS · LAST KNOWN",
            Self::Stale => "USGS · STALE",
            Self::Unavailable => "USGS · OFFLINE",
        }
    }

    pub fn css_class(self) -> &'static str {
        match self {
            Self::Loading => "loading",
            Self::Live => "live",
            Self::Degraded => "degraded",
            Self::Stale => "stale",
            Self::Unavailable => "unavailable",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsgsFeedStatus {
    pub health: FeedHealth,
    pub last_fetched_at_ms: Option<i64>,
    pub generated_at_ms: Option<i64>,
    pub event_count: usize,
    pub skipped_features: usize,
}

impl Default for UsgsFeedStatus {
    fn default() -> Self {
        Self {
            health: FeedHealth::Loading,
            last_fetched_at_ms: None,
            generated_at_ms: None,
            event_count: 0,
            skipped_features: 0,
        }
    }
}

impl UsgsFeedStatus {
    pub fn description(&self) -> String {
        let source_time = match (self.generated_at_ms, self.last_fetched_at_ms) {
            (Some(generated), Some(fetched)) => {
                format!("Feed generated at {generated} ms UTC; fetched at {fetched} ms UTC.")
            }
            _ => "No validated live snapshot has been accepted yet.".to_string(),
        };

        format!(
            "{}. {} Valid events: {}; rejected features: {}. Failed refreshes preserve the last accepted snapshot.",
            self.health.label(),
            source_time,
            self.event_count,
            self.skipped_features
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOW: i64 = 1_700_000_000_000;

    fn fixture(generated: i64, features: &str) -> String {
        format!(
            r#"{{
                "type": "FeatureCollection",
                "metadata": {{"generated": {generated}, "status": 200, "count": 1}},
                "features": [{features}]
            }}"#
        )
    }

    fn quake(id: &str, lon: &str, lat: &str, mag: &str) -> String {
        format!(
            r#"{{
                "type": "Feature",
                "id": "{id}",
                "properties": {{
                    "mag": {mag},
                    "place": "Test location",
                    "time": 1699999990000,
                    "updated": 1699999995000,
                    "url": "https://earthquake.usgs.gov/earthquakes/eventpage/{id}",
                    "type": "earthquake"
                }},
                "geometry": {{
                    "type": "Point",
                    "coordinates": [{lon}, {lat}, 12.5]
                }}
            }}"#
        )
    }

    #[test]
    fn parses_native_usgs_fields_and_trusted_event_link() {
        let json = fixture(NOW, &quake("us123", "-80.5", "-5.2", "4.7"));
        let feed = parse_feed(&json, NOW).unwrap();
        let event = &feed.earthquakes[0];

        assert_eq!(event.id, "us123");
        assert_eq!(event.latitude, -5.2);
        assert_eq!(event.longitude, -80.5);
        assert_eq!(event.depth_km, Some(12.5));
        assert_eq!(event.magnitude, 4.7);
        assert_eq!(event.occurred_at_ms, 1_699_999_990_000);
        assert_eq!(
            event.event_url.as_deref(),
            Some("https://earthquake.usgs.gov/earthquakes/eventpage/us123")
        );
        assert!(feed.is_fresh_at(NOW));
    }

    #[test]
    fn skips_invalid_features_but_keeps_valid_events() {
        let valid = quake("us123", "-80.5", "-5.2", "4.7");
        let invalid = quake("us999", "0", "91", "5.0");
        let json = fixture(NOW, &format!("{valid},{invalid}"));
        let feed = parse_feed(&json, NOW).unwrap();

        assert_eq!(feed.earthquakes.len(), 1);
        assert_eq!(feed.skipped_features, 1);
    }

    #[test]
    fn rejects_stale_and_future_dated_feeds() {
        let feature = quake("us123", "10", "20", "3.0");
        let stale = fixture(NOW - MAX_FEED_AGE_MS - 1, &feature);
        let future = fixture(NOW + MAX_FUTURE_SKEW_MS + 1, &feature);

        assert_eq!(parse_feed(&stale, NOW), Err(UsgsFeedError::StaleFeed));
        assert_eq!(
            parse_feed(&future, NOW),
            Err(UsgsFeedError::FutureDatedFeed)
        );
    }

    #[test]
    fn skips_events_outside_the_feed_time_window() {
        let valid = quake("us123", "10", "20", "3.0");
        let old = quake("us124", "11", "21", "3.1").replace("1699999990000", "1600000000000");
        let future = quake("us125", "12", "22", "3.2")
            .replace("1699999990000", &(NOW + MAX_FUTURE_SKEW_MS + 1).to_string());
        let json = fixture(NOW, &format!("{valid},{old},{future}"));
        let feed = parse_feed(&json, NOW).unwrap();

        assert_eq!(feed.earthquakes.len(), 1);
        assert_eq!(feed.skipped_features, 2);
    }

    #[test]
    fn rejects_oversized_and_non_success_feeds() {
        let oversized = " ".repeat(MAX_FEED_BYTES + 1);
        let payload = format!(
            r#"{{"type":"FeatureCollection","metadata":{{"generated":{NOW},"status":503}},"features":[]}}"#
        );

        assert!(matches!(
            parse_feed(&oversized, NOW),
            Err(UsgsFeedError::BodyTooLarge { .. })
        ));
        assert_eq!(
            parse_feed(&payload, NOW),
            Err(UsgsFeedError::UpstreamStatus(503))
        );
    }

    #[test]
    fn rejects_empty_feed_instead_of_clearing_last_good_data() {
        let payload = format!(
            r#"{{"type":"FeatureCollection","metadata":{{"generated":{NOW},"status":200,"count":0}},"features":[]}}"#
        );

        assert_eq!(parse_feed(&payload, NOW), Err(UsgsFeedError::EmptyFeed));
    }

    #[test]
    fn rejects_nonempty_feed_if_all_features_are_invalid() {
        let bad_id = quake("bad/id", "10", "20", "3.0");
        let json = fixture(NOW, &bad_id);

        assert_eq!(parse_feed(&json, NOW), Err(UsgsFeedError::NoValidFeatures));
    }

    #[test]
    fn untrusted_event_urls_are_discarded() {
        let payload = fixture(NOW, &quake("us123", "10", "20", "3.0")).replace(
            "https://earthquake.usgs.gov/earthquakes/eventpage/us123",
            "javascript:alert(1)",
        );
        let feed = parse_feed(&payload, NOW).unwrap();

        assert_eq!(feed.earthquakes[0].event_url, None);
    }

    #[test]
    fn event_url_must_match_the_feature_id() {
        let payload = fixture(NOW, &quake("us123", "10", "20", "3.0")).replace(
            "/earthquakes/eventpage/us123",
            "/earthquakes/eventpage/us999",
        );
        let feed = parse_feed(&payload, NOW).unwrap();

        assert_eq!(feed.earthquakes[0].event_url, None);
    }
}
