// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Bounded, provenance-preserving parser for NASA EONET v3 active natural events.
//!
//! EONET is curated event metadata, not a complete sensor feed. Point
//! geometries remain points; polygon geometries become approximate spherical
//! vertex-centers for globe markers, not area-weighted centroids.

use serde::Deserialize;
use serde_json::Value;
use std::fmt;

pub const EONET_OPEN_EVENTS_URL: &str =
    "https://eonet.gsfc.nasa.gov/api/v3/events?category=severeStorms,wildfires,volcanoes&status=open&days=7&limit=500";
pub const EONET_EVENT_API_PREFIX: &str = "https://eonet.gsfc.nasa.gov/api/v3/events/";
pub const MAX_EONET_BODY_BYTES: usize = 4 * 1024 * 1024;
pub const MAX_EONET_EVENTS: usize = 1_000;
pub const MAX_GEOMETRIES_PER_EVENT: usize = 256;
pub const MAX_POLYGON_VERTICES: usize = 10_000;
pub const MAX_EONET_EVENT_AGE_MS: i64 = 8 * 24 * 60 * 60 * 1_000;
pub const EONET_FUTURE_SKEW_MS: i64 = 10 * 60 * 1_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EonetEventType {
    Fire,
    Storm,
    Volcano,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EonetGeometryKind {
    Point,
    PolygonCenter,
}

impl EonetGeometryKind {
    pub fn label(self) -> &'static str {
        match self {
            Self::Point => "source point",
            Self::PolygonCenter => "approximate polygon center",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EonetEvent {
    pub id: String,
    pub title: String,
    pub event_type: EonetEventType,
    pub latitude: f64,
    pub longitude: f64,
    pub occurred_at_ms: i64,
    pub event_url: Option<String>,
    /// Optional source-provided value; units are not standardized across categories.
    pub magnitude_value: Option<f64>,
    pub magnitude_unit: Option<String>,
    pub magnitude_description: Option<String>,
    pub geometry_kind: EonetGeometryKind,
}

#[derive(Debug, Clone, PartialEq)]
pub struct EonetFeedSnapshot {
    /// Local retrieval time; the events collection lacks an authoritative feed-generation time.
    pub fetched_at_ms: i64,
    pub skipped_events: usize,
    pub events: Vec<EonetEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EonetFeedError {
    BodyTooLarge { actual: usize, maximum: usize },
    InvalidJson(String),
    InvalidFetchTimestamp,
    TooManyEvents { actual: usize, maximum: usize },
    TooManyGeometries { event_id: String, actual: usize, maximum: usize },
    NoValidEvents,
}

impl fmt::Display for EonetFeedError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::BodyTooLarge { actual, maximum } => {
                write!(f, "EONET payload is {actual} bytes; limit is {maximum}")
            }
            Self::InvalidJson(error) => write!(f, "invalid NASA EONET response: {error}"),
            Self::InvalidFetchTimestamp => write!(f, "local EONET fetch timestamp is invalid"),
            Self::TooManyEvents { actual, maximum } => {
                write!(f, "EONET returned {actual} events; limit is {maximum}")
            }
            Self::TooManyGeometries { event_id, actual, maximum } => {
                write!(f, "EONET event {event_id} has {actual} geometries; limit is {maximum}")
            }
            Self::NoValidEvents => {
                write!(f, "EONET returned events but none passed category, time, and geometry validation")
            }
        }
    }
}
impl std::error::Error for EonetFeedError {}

#[derive(Deserialize)]
struct EonetResponse {
    events: Vec<EventWire>,
}

#[derive(Deserialize)]
struct EventWire {
    id: String,
    title: String,
    #[serde(default)]
    link: Option<String>,
    #[serde(default)]
    closed: Option<String>,
    #[serde(default)]
    categories: Vec<CategoryWire>,
    #[serde(default)]
    geometry: Vec<GeometryWire>,
    #[serde(rename = "magnitudeValue", default)]
    magnitude_value: Option<f64>,
    #[serde(rename = "magnitudeUnit", default)]
    magnitude_unit: Option<String>,
    #[serde(rename = "magnitudeDescription", default)]
    magnitude_description: Option<String>,
}

#[derive(Deserialize)]
struct CategoryWire {
    id: String,
}

#[derive(Deserialize)]
struct GeometryWire {
    date: String,
    #[serde(rename = "type")]
    kind: String,
    coordinates: Value,
}

/// A well-formed empty open-event catalog is valid and clears source-owned
/// records. A nonempty response whose events all fail validation is rejected
/// so schema drift cannot clear state.
pub fn parse_feed(json: &str, fetched_at_ms: i64) -> Result<EonetFeedSnapshot, EonetFeedError> {
    if json.len() > MAX_EONET_BODY_BYTES {
        return Err(EonetFeedError::BodyTooLarge {
            actual: json.len(),
            maximum: MAX_EONET_BODY_BYTES,
        });
    }
    if fetched_at_ms <= 0 {
        return Err(EonetFeedError::InvalidFetchTimestamp);
    }
    let response: EonetResponse = serde_json::from_str(json)
        .map_err(|error| EonetFeedError::InvalidJson(error.to_string()))?;
    if response.events.len() > MAX_EONET_EVENTS {
        return Err(EonetFeedError::TooManyEvents {
            actual: response.events.len(),
            maximum: MAX_EONET_EVENTS,
        });
    }

    let original_count = response.events.len();
    let mut events = Vec::with_capacity(original_count.min(500));
    let mut skipped_events = 0usize;
    for event in response.events {
        if !valid_identifier(&event.id)
            || event.title.trim().is_empty()
            || event.title.len() > 512
            || event.closed.is_some()
        {
            skipped_events += 1;
            continue;
        }
        let Some(event_type) = classify_categories(&event.categories) else {
            skipped_events += 1;
            continue;
        };
        if event.geometry.len() > MAX_GEOMETRIES_PER_EVENT {
            return Err(EonetFeedError::TooManyGeometries {
                event_id: event.id,
                actual: event.geometry.len(),
                maximum: MAX_GEOMETRIES_PER_EVENT,
            });
        }

        let mut latest: Option<(i64, f64, f64, EonetGeometryKind)> = None;
        for geometry in &event.geometry {
            let Some(timestamp_ms) = parse_iso8601_ms(&geometry.date) else {
                continue;
            };
            if timestamp_ms < fetched_at_ms.saturating_sub(MAX_EONET_EVENT_AGE_MS)
                || timestamp_ms > fetched_at_ms.saturating_add(EONET_FUTURE_SKEW_MS)
            {
                continue;
            }
            let Some((latitude, longitude, geometry_kind)) =
                representative_point(&geometry.kind, &geometry.coordinates)
            else {
                continue;
            };
            if latest.as_ref().is_none_or(|current| timestamp_ms > current.0) {
                latest = Some((timestamp_ms, latitude, longitude, geometry_kind));
            }
        }

        let Some((occurred_at_ms, latitude, longitude, geometry_kind)) = latest else {
            skipped_events += 1;
            continue;
        };
        let event_url = event.link.as_deref()
            .filter(|url| valid_event_url(url, &event.id))
            .map(str::to_string);
        let magnitude_value = event.magnitude_value
            .filter(|value| value.is_finite() && (0.0..=1_000_000.0).contains(value));
        let magnitude_unit = event.magnitude_unit.filter(|unit| {
            !unit.trim().is_empty() && unit.len() <= 64 && !unit.chars().any(char::is_control)
        });
        let magnitude_description = event.magnitude_description.filter(|description| {
            !description.trim().is_empty()
                && description.len() <= 256
                && !description.chars().any(char::is_control)
        });

        events.push(EonetEvent {
            id: event.id,
            title: event.title,
            event_type,
            latitude,
            longitude,
            occurred_at_ms,
            event_url,
            magnitude_value,
            magnitude_unit,
            magnitude_description,
            geometry_kind,
        });
    }

    if original_count > 0 && events.is_empty() {
        return Err(EonetFeedError::NoValidEvents);
    }
    events.sort_by(|left, right| {
        right.occurred_at_ms.cmp(&left.occurred_at_ms)
            .then_with(|| left.id.cmp(&right.id))
    });
    Ok(EonetFeedSnapshot { fetched_at_ms, skipped_events, events })
}

fn classify_categories(categories: &[CategoryWire]) -> Option<EonetEventType> {
    let has = |needle: &str| categories.iter().any(|category| category.id == needle);
    if has("wildfires") {
        Some(EonetEventType::Fire)
    } else if has("severeStorms") {
        Some(EonetEventType::Storm)
    } else if has("volcanoes") {
        Some(EonetEventType::Volcano)
    } else {
        None
    }
}

fn valid_identifier(value: &str) -> bool {
    !value.is_empty() && value.len() <= 160
        && value.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}

fn valid_event_url(value: &str, event_id: &str) -> bool {
    value.strip_prefix(EONET_EVENT_API_PREFIX) == Some(event_id)
        && value.len() <= 2_048
        && !value.chars().any(char::is_control)
        && !value.chars().any(char::is_whitespace)
}

fn valid_coordinate(latitude: f64, longitude: f64) -> bool {
    latitude.is_finite() && longitude.is_finite()
        && (-90.0..=90.0).contains(&latitude)
        && (-180.0..=180.0).contains(&longitude)
}

fn representative_point(
    kind: &str,
    coordinates: &Value,
) -> Option<(f64, f64, EonetGeometryKind)> {
    match kind {
        "Point" => {
            let pair = coordinates.as_array()?;
            let longitude = pair.first()?.as_f64()?;
            let latitude = pair.get(1)?.as_f64()?;
            valid_coordinate(latitude, longitude)
                .then_some((latitude, longitude, EonetGeometryKind::Point))
        }
        "Polygon" => {
            let rings = coordinates.as_array()?;
            let exterior = rings.first()?.as_array()?;
            if exterior.is_empty() || exterior.len() > MAX_POLYGON_VERTICES {
                return None;
            }

            // Unit-vector averaging avoids a false Greenwich center for rings
            // crossing +/-180 degrees. This is not area-weighted.
            let (mut x, mut y, mut z) = (0.0, 0.0, 0.0);
            for point in exterior {
                let pair = point.as_array()?;
                let longitude = pair.first()?.as_f64()?;
                let latitude = pair.get(1)?.as_f64()?;
                if !valid_coordinate(latitude, longitude) {
                    return None;
                }
                let lat = latitude.to_radians();
                let lon = longitude.to_radians();
                x += lat.cos() * lon.cos();
                y += lat.cos() * lon.sin();
                z += lat.sin();
            }
            if (x * x + y * y + z * z) < 1e-16 {
                return None;
            }
            let longitude = y.atan2(x).to_degrees();
            let latitude = z.atan2((x * x + y * y).sqrt()).to_degrees();
            valid_coordinate(latitude, longitude).then_some((
                latitude,
                longitude,
                EonetGeometryKind::PolygonCenter,
            ))
        }
        _ => None,
    }
}

/// Parse a UTC date or RFC3339 timestamp with Z / numeric UTC offset.
fn parse_iso8601_ms(value: &str) -> Option<i64> {
    let bytes = value.as_bytes();
    if bytes.len() < 10 {
        return None;
    }
    let year = parse_digits(bytes, 0, 4)? as i64;
    let month = parse_digits(bytes, 5, 2)?;
    let day = parse_digits(bytes, 8, 2)?;
    if bytes.get(4) != Some(&b'-') || bytes.get(7) != Some(&b'-')
        || !(1..=12).contains(&month)
    {
        return None;
    }
    let days_in_month = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if is_leap_year(year) => 29,
        2 => 28,
        _ => return None,
    };
    if day == 0 || day > days_in_month {
        return None;
    }

    let (mut hour, mut minute, mut second, mut fraction_ms, mut offset_seconds) =
        (0i64, 0i64, 0i64, 0i64, 0i64);
    if bytes.len() > 10 {
        if bytes.get(10) != Some(&b'T') || bytes.len() < 20
            || bytes.get(13) != Some(&b':') || bytes.get(16) != Some(&b':')
        {
            return None;
        }
        hour = parse_digits(bytes, 11, 2)? as i64;
        minute = parse_digits(bytes, 14, 2)? as i64;
        second = parse_digits(bytes, 17, 2)? as i64;
        if hour > 23 || minute > 59 || second > 59 {
            return None;
        }

        let mut suffix_start = 19usize;
        if bytes.get(suffix_start) == Some(&b'.') {
            suffix_start += 1;
            let start = suffix_start;
            while bytes.get(suffix_start).is_some_and(u8::is_ascii_digit) {
                suffix_start += 1;
            }
            if suffix_start == start {
                return None;
            }
            let fraction = &bytes[start..suffix_start];
            let first_three: String = fraction.iter().take(3).map(|byte| *byte as char).collect();
            fraction_ms = format!("{first_three:0<3}").parse::<i64>().ok()?;
        }

        match bytes.get(suffix_start).copied()? {
            b'Z' if suffix_start + 1 == bytes.len() => {}
            sign @ (b'+' | b'-') if suffix_start + 6 == bytes.len()
                && bytes.get(suffix_start + 3) == Some(&b':') =>
            {
                let offset_hour = parse_digits(bytes, suffix_start + 1, 2)? as i64;
                let offset_minute = parse_digits(bytes, suffix_start + 4, 2)? as i64;
                if offset_hour > 23 || offset_minute > 59 {
                    return None;
                }
                let multiplier = if sign == b'+' { 1 } else { -1 };
                offset_seconds = multiplier * (offset_hour * 3600 + offset_minute * 60);
            }
            _ => return None,
        }
    }

    let days = days_from_civil(year, month, day);
    days.checked_mul(86_400)?
        .checked_add(hour * 3600 + minute * 60 + second)?
        .checked_sub(offset_seconds)?
        .checked_mul(1_000)?
        .checked_add(fraction_ms)
}

fn parse_digits(bytes: &[u8], start: usize, len: usize) -> Option<u32> {
    let slice = bytes.get(start..start + len)?;
    if !slice.iter().all(u8::is_ascii_digit) {
        return None;
    }
    std::str::from_utf8(slice).ok()?.parse().ok()
}

fn is_leap_year(year: i64) -> bool {
    year % 4 == 0 && (year % 100 != 0 || year % 400 == 0)
}

fn days_from_civil(year: i64, month: u32, day: u32) -> i64 {
    let year = year - if month <= 2 { 1 } else { 0 };
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = year - era * 400;
    let adjusted_month = month as i64 + if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * adjusted_month + 2) / 5 + day as i64 - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EonetFeedStatus {
    pub health: super::usgs::FeedHealth,
    pub last_fetched_at_ms: Option<i64>,
    pub event_count: usize,
    pub skipped_events: usize,
}

impl Default for EonetFeedStatus {
    fn default() -> Self {
        Self {
            health: super::usgs::FeedHealth::Loading,
            last_fetched_at_ms: None,
            event_count: 0,
            skipped_events: 0,
        }
    }
}

impl EonetFeedStatus {
    pub fn label(&self) -> &'static str {
        use super::usgs::FeedHealth;
        match self.health {
            FeedHealth::Loading => "EONET · CONNECTING",
            FeedHealth::Live => "EONET · LIVE",
            FeedHealth::Degraded => "EONET · LAST KNOWN",
            FeedHealth::Stale => "EONET · STALE",
            FeedHealth::Unavailable => "EONET · OFFLINE",
        }
    }

    pub fn css_class(&self) -> &'static str {
        self.health.css_class()
    }

    pub fn description(&self) -> String {
        match self.last_fetched_at_ms {
            Some(fetched) => format!(
                "{}. Last validated EONET fetch: {fetched} ms UTC. Active mapped events: {}; rejected events: {}. This is curated metadata, not an exhaustive sensor feed.",
                self.label(), self.event_count, self.skipped_events
            ),
            None => format!(
                "{}. No validated NASA EONET snapshot has been accepted.",
                self.label()
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOW: i64 = 1_777_766_400_000; // 2026-05-03T00:00:00Z

    fn fixture(events: &str) -> String {
        format!(r#"{{"title":"EONET Events","events":[{events}]}}"#)
    }

    fn event(id: &str, category: &str, closed: &str, geometry: &str, link_id: &str) -> String {
        format!(
            r#"{{"id":"{id}","title":"Test event","link":"{EONET_EVENT_API_PREFIX}{link_id}","closed":{closed},"categories":[{{"id":"{category}","title":"Test"}}],"geometry":[{geometry}]}}"#
        )
    }

    fn point_geometry(date: &str, longitude: f64, latitude: f64) -> String {
        format!(r#"{{"date":"{date}","type":"Point","coordinates":[{longitude},{latitude}]}}"#)
    }

    #[test]
    fn parses_point_and_polygon_events_with_time_and_trusted_links() {
        let point = event(
            "EONET_1", "severeStorms", "null",
            &point_geometry("2026-05-01T12:00:00Z", -80.0, 12.0), "EONET_1",
        );
        let polygon = event(
            "EONET_2", "wildfires", "null",
            r#"{"date":"2026-05-01T13:00:00Z","type":"Polygon","coordinates":[[[-179.8,10.0],[179.8,10.0],[179.8,11.0],[-179.8,11.0],[-179.8,10.0]]]}"#,
            "EONET_2",
        );
        let feed = parse_feed(&fixture(&format!("{point},{polygon}")), NOW).unwrap();

        assert_eq!(feed.events.len(), 2);
        assert_eq!(feed.events[0].id, "EONET_2");
        assert_eq!(feed.events[0].event_type, EonetEventType::Fire);
        assert_eq!(feed.events[0].geometry_kind, EonetGeometryKind::PolygonCenter);
        assert!(feed.events[0].longitude.abs() > 170.0);
        assert_eq!(feed.events[1].event_type, EonetEventType::Storm);
        assert_eq!(
            feed.events[1].event_url.as_deref(),
            Some("https://eonet.gsfc.nasa.gov/api/v3/events/EONET_1")
        );
    }

    #[test]
    fn accepts_a_well_formed_empty_open_event_catalog() {
        let feed = parse_feed(r#"{"events":[]}"#, NOW).unwrap();
        assert!(feed.events.is_empty());
        assert_eq!(feed.fetched_at_ms, NOW);
    }

    #[test]
    fn rejects_nonempty_catalog_when_all_events_are_invalid() {
        let bad = event(
            "EONET_1", "wildfires", "null",
            &point_geometry("2026-05-01T12:00:00Z", 400.0, 10.0), "EONET_1",
        );
        assert_eq!(parse_feed(&fixture(&bad), NOW), Err(EonetFeedError::NoValidEvents));
    }

    #[test]
    fn rejects_url_id_mismatch_and_skips_closed_events() {
        let mismatch = event(
            "EONET_1", "wildfires", "null",
            &point_geometry("2026-05-01T12:00:00Z", 10.0, 20.0), "EONET_9",
        );
        let closed = event(
            "EONET_2", "volcanoes", r#""2026-05-02T00:00:00Z""#,
            &point_geometry("2026-05-01T12:00:00Z", 11.0, 21.0), "EONET_2",
        );
        let feed = parse_feed(&fixture(&format!("{mismatch},{closed}")), NOW).unwrap();
        assert_eq!(feed.events.len(), 1);
        assert!(feed.events[0].event_url.is_none());
    }

    #[test]
    fn parses_iso_timestamps_and_offsets() {
        assert_eq!(parse_iso8601_ms("1970-01-01"), Some(0));
        assert_eq!(parse_iso8601_ms("1970-01-01T00:00:00Z"), Some(0));
        assert_eq!(parse_iso8601_ms("1970-01-01T01:00:00+01:00"), Some(0));
        assert_eq!(parse_iso8601_ms("1970-01-01T01:00:00.1234+01:00"), Some(123));
        assert_eq!(parse_iso8601_ms("2025-02-30T00:00:00Z"), None);
        assert_eq!(parse_iso8601_ms("not-a-date"), None);
    }
}
