// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
//! Evidence-bearing adapters for physical-world Sol Atlas records.
//!
//! This module is intentionally renderer-independent. It maps source records
//! into the canonical Mycelix planetary-evidence contract while retaining the
//! existing lightweight `NaturalEvent` used by renderers.

use crate::types::{DataKind, DataProvenance, Layer, NaturalEvent, NaturalEventType};
use chrono::{DateTime, NaiveDateTime};
use mycelix_core_types::{
    EnvironmentalObservation, EvidenceClass, ExternalEvidenceRef, GeoPoint, Measurement,
    SpatialExtent, TemporalExtent, Uncertainty,
};
use serde::Deserialize;

/// Existing renderer record paired with its source evidence.
#[derive(Debug, Clone)]
pub struct EvidenceBearingNaturalEvent {
    pub event: NaturalEvent,
    pub observation: EnvironmentalObservation,
}

#[derive(Debug, Deserialize)]
struct FeatureCollection {
    features: Vec<Feature>,
}

#[derive(Debug, Deserialize)]
struct Feature {
    #[serde(default)]
    id: Option<String>,
    properties: Properties,
    geometry: Geometry,
}

#[derive(Debug, Deserialize)]
struct Geometry {
    coordinates: Vec<f64>,
}

#[derive(Debug, Default, Deserialize)]
struct Properties {
    #[serde(default)]
    magnitude: Option<f64>,
    #[serde(default)]
    magnitude_unit: Option<String>,
    #[serde(default)]
    brightness: Option<f64>,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    place: Option<String>,
    #[serde(default)]
    name: Option<String>,
    #[serde(default)]
    source: Option<String>,
    #[serde(default)]
    time: Option<String>,
    #[serde(default)]
    detection_time: Option<String>,
    #[serde(default)]
    date: Option<String>,
    #[serde(default)]
    event_id: Option<String>,
    #[serde(default)]
    link: Option<String>,
    #[serde(default)]
    categories: Vec<String>,
    #[serde(default)]
    data_lineage: Vec<String>,
}

/// Parse all four checked-in natural-event datasets into records that carry
/// canonical evidence provenance.
///
/// Parse/validation failures are logged and excluded rather than converted to
/// fabricated placeholder observations.
pub fn parse_natural_event_records(
    earthquakes: &str,
    fires: &str,
    eonet: &str,
    volcanoes: &str,
) -> Vec<EvidenceBearingNaturalEvent> {
    let mut records = Vec::new();
    records.extend(parse_dataset(
        "usgs-earthquakes",
        earthquakes,
        NaturalEventType::Earthquake,
    ));
    records.extend(parse_dataset(
        "nasa-firms-demo",
        fires,
        NaturalEventType::Fire,
    ));
    records.extend(parse_dataset(
        "nasa-eonet",
        eonet,
        NaturalEventType::Storm,
    ));
    records.extend(parse_dataset(
        "smithsonian-gvp",
        volcanoes,
        NaturalEventType::Volcano,
    ));
    records
}

fn parse_dataset(
    dataset: &'static str,
    json: &str,
    fallback_type: NaturalEventType,
) -> Vec<EvidenceBearingNaturalEvent> {
    let collection = match serde_json::from_str::<FeatureCollection>(json) {
        Ok(collection) => collection,
        Err(error) => {
            log::warn!("sol-atlas-core: failed to parse evidence dataset '{dataset}': {error}");
            return Vec::new();
        }
    };

    collection
        .features
        .into_iter()
        .enumerate()
        .filter_map(|(index, feature)| build_record(dataset, index, feature, fallback_type))
        .collect()
}

fn build_record(
    dataset: &'static str,
    index: usize,
    feature: Feature,
    fallback_type: NaturalEventType,
) -> Option<EvidenceBearingNaturalEvent> {
    let coordinates = &feature.geometry.coordinates;
    if coordinates.len() < 2 {
        log::warn!("sol-atlas-core: {dataset}[{index}] has no point coordinates");
        return None;
    }

    let longitude = coordinates[0];
    let latitude = coordinates[1];
    let point = match GeoPoint::new(latitude, longitude) {
        Ok(point) => point,
        Err(error) => {
            log::warn!("sol-atlas-core: invalid coordinates in {dataset}[{index}]: {error}");
            return None;
        }
    };

    let event_type = classify_event_type(&feature.properties, fallback_type);
    let evidence_class = classify_evidence(&feature.properties, event_type);
    let source_time = source_time(&feature.properties);
    let compact_id = compact_record_id(dataset, index, &feature, source_time);
    let resource_id = feature
        .properties
        .link
        .clone()
        .unwrap_or_else(|| compact_id.clone());
    let source_system = feature
        .properties
        .source
        .clone()
        .unwrap_or_else(|| dataset.to_string());

    let measurement = measurement_for(&feature.properties, event_type);
    let display_magnitude = feature
        .properties
        .magnitude
        .or_else(|| feature.properties.brightness.map(|brightness| brightness / 100.0))
        .unwrap_or(1.0);

    if !display_magnitude.is_finite() {
        log::warn!("sol-atlas-core: non-finite display magnitude in {dataset}[{index}]");
        return None;
    }

    let name = feature
        .properties
        .title
        .as_deref()
        .or(feature.properties.place.as_deref())
        .or(feature.properties.name.as_deref())
        .unwrap_or("Unknown")
        .to_string();

    let temporal = source_time
        .and_then(parse_source_time)
        .map(TemporalExtent::instant)
        .unwrap_or(TemporalExtent::Unspecified);

    let observation = match EnvironmentalObservation::new(
        format!("atlas:{compact_id}"),
        phenomenon_for(event_type),
        evidence_class,
        measurement,
        SpatialExtent::Point(point),
        temporal,
        Uncertainty::Unspecified,
        vec![ExternalEvidenceRef {
            source_system,
            resource_id,
            content_digest: None,
            retrieved_at: None,
            license: None,
        }],
    ) {
        Ok(observation) => observation,
        Err(error) => {
            log::warn!("sol-atlas-core: invalid evidence in {dataset}[{index}]: {error}");
            return None;
        }
    };

    Some(EvidenceBearingNaturalEvent {
        event: NaturalEvent {
            lat: latitude,
            lon: longitude,
            event_type,
            magnitude: display_magnitude,
            name,
        },
        observation,
    })
}

fn compact_record_id(
    dataset: &str,
    index: usize,
    feature: &Feature,
    source_time: Option<&str>,
) -> String {
    if let Some(event_id) = feature.properties.event_id.as_deref() {
        return format!("{dataset}:{event_id}");
    }
    if let Some(feature_id) = feature.id.as_deref() {
        return format!("{dataset}:{feature_id}");
    }
    if let Some(source_time) = source_time {
        return format!("{dataset}:{source_time}:{index}");
    }
    format!("{dataset}:{index}")
}

fn source_time(properties: &Properties) -> Option<&str> {
    properties
        .time
        .as_deref()
        .or(properties.detection_time.as_deref())
        .or(properties.date.as_deref())
}

fn parse_source_time(value: &str) -> Option<i64> {
    if let Ok(timestamp) = DateTime::parse_from_rfc3339(value) {
        return Some(timestamp.timestamp());
    }
    NaiveDateTime::parse_from_str(value, "%Y-%m-%dT%H:%M:%S%.f")
        .ok()
        .map(|timestamp| timestamp.and_utc().timestamp())
}

fn classify_event_type(
    properties: &Properties,
    fallback: NaturalEventType,
) -> NaturalEventType {
    for category in &properties.categories {
        let category = category.to_ascii_lowercase();
        if category.contains("wildfire") || category.contains("fire") {
            return NaturalEventType::Fire;
        }
        if category.contains("volcano") {
            return NaturalEventType::Volcano;
        }
        if category.contains("earthquake") || category.contains("seismic") {
            return NaturalEventType::Earthquake;
        }
        if category.contains("storm") || category.contains("cyclone") {
            return NaturalEventType::Storm;
        }
    }
    fallback
}

fn classify_evidence(properties: &Properties, event_type: NaturalEventType) -> EvidenceClass {
    let source = properties
        .source
        .as_deref()
        .unwrap_or_default()
        .to_ascii_lowercase();
    let lineage = properties.data_lineage.join(" ").to_ascii_lowercase();

    if source.contains("demo")
        || lineage.contains("synthetic")
        || lineage.contains("generated")
    {
        EvidenceClass::Scenario
    } else if event_type == NaturalEventType::Volcano || lineage.contains("expert review") {
        EvidenceClass::Reported
    } else {
        EvidenceClass::Observed
    }
}

fn measurement_for(
    properties: &Properties,
    event_type: NaturalEventType,
) -> Option<Measurement> {
    if event_type == NaturalEventType::Earthquake {
        return properties
            .magnitude
            .and_then(|value| Measurement::new(value, "Mw").ok());
    }

    if let Some(brightness) = properties.brightness {
        return Measurement::new(brightness, "K").ok();
    }

    match (properties.magnitude, properties.magnitude_unit.as_deref()) {
        (Some(value), Some(unit)) => Measurement::new(value, unit).ok(),
        _ => None,
    }
}

fn phenomenon_for(event_type: NaturalEventType) -> &'static str {
    match event_type {
        NaturalEventType::Earthquake => "earthquake",
        NaturalEventType::Fire => "fire",
        NaturalEventType::Storm => "severe_storm",
        NaturalEventType::Volcano => "volcanic_activity",
    }
}

/// Audited layer-level fallback used until every renderer consumes per-record
/// evidence directly. Record evidence is authoritative when available.
pub fn audited_layer_provenance(layer: Layer) -> DataProvenance {
    match layer {
        Layer::Earthquakes => DataProvenance {
            source: "USGS earthquake snapshot",
            snapshot_date: "2025-09",
            kind: DataKind::Observed,
        },
        Layer::Fires => DataProvenance {
            source: "mixed: synthetic FIRMS demo + observed EONET wildfire records",
            snapshot_date: "2025-09",
            // DataKind v1 has no Mixed variant. Curated is the conservative
            // layer-level fallback; per-record EvidenceClass carries truth.
            kind: DataKind::Curated,
        },
        Layer::Storms => DataProvenance {
            source: "NASA EONET mixed natural-event snapshot",
            snapshot_date: "2025-09",
            kind: DataKind::Observed,
        },
        Layer::Volcanoes => DataProvenance {
            source: "Smithsonian GVP expert-reviewed records",
            snapshot_date: "",
            kind: DataKind::Curated,
        },
        _ => layer.provenance(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn usgs_earthquake_becomes_observed_mw_evidence() {
        let json = r#"{
          "features": [{
            "properties": {
              "source": "USGS",
              "magnitude": 5.1,
              "place": "Peru",
              "time": "2025-09-16T01:05:59.070000",
              "data_lineage": ["USGS", "Seismic Network", "Real-time"]
            },
            "geometry": {"coordinates": [-80.5, -5.2, 10]}
          }]
        }"#;
        let records = parse_dataset("usgs-earthquakes", json, NaturalEventType::Earthquake);
        assert_eq!(records.len(), 1);
        let record = &records[0];
        assert_eq!(record.event.event_type, NaturalEventType::Earthquake);
        assert_eq!(record.observation.class, EvidenceClass::Observed);
        let measurement = record.observation.measurement.as_ref().unwrap();
        assert_eq!(measurement.unit, "Mw");
        assert!((measurement.value - 5.1).abs() < 0.001);
        assert!(matches!(record.observation.temporal, TemporalExtent::Instant(_)));
    }

    #[test]
    fn synthetic_firms_record_is_scenario_not_observed() {
        let json = r#"{
          "features": [{
            "properties": {
              "source": "Demo Data",
              "brightness": 387.6,
              "detection_time": "2025-09-16T02:39:42.850887",
              "data_lineage": ["Demo", "Synthetic", "Generated"]
            },
            "geometry": {"coordinates": [-117.3, 33.3]}
          }]
        }"#;
        let records = parse_dataset("nasa-firms-demo", json, NaturalEventType::Fire);
        assert_eq!(records[0].observation.class, EvidenceClass::Scenario);
        assert_eq!(
            records[0].observation.measurement.as_ref().unwrap().unit,
            "K"
        );
    }

    #[test]
    fn eonet_category_overrides_storm_fallback() {
        let json = r#"{
          "features": [{
            "properties": {
              "source": "NASA EONET",
              "title": "Prescribed Fire",
              "categories": ["Wildfires"],
              "event_id": "EONET_1",
              "magnitude": 1047.0,
              "magnitude_unit": "acres",
              "date": "2025-09-11T09:06:00Z",
              "data_lineage": ["NASA", "EONET", "Satellite Observation"]
            },
            "geometry": {"coordinates": [-84.0, 30.0]}
          }]
        }"#;
        let records = parse_dataset("nasa-eonet", json, NaturalEventType::Storm);
        assert_eq!(records[0].event.event_type, NaturalEventType::Fire);
        assert_eq!(records[0].observation.class, EvidenceClass::Observed);
        assert_eq!(
            records[0].observation.measurement.as_ref().unwrap().unit,
            "acres"
        );
    }

    #[test]
    fn expert_reviewed_volcano_needs_no_fake_scalar_or_time() {
        let json = r#"{
          "features": [{
            "properties": {
              "source": "Smithsonian GVP",
              "name": "Kilauea",
              "data_lineage": ["Smithsonian", "Global Volcanism Program", "Expert Review"]
            },
            "geometry": {"coordinates": [-155.287, 19.421]}
          }]
        }"#;
        let records = parse_dataset("smithsonian-gvp", json, NaturalEventType::Volcano);
        assert_eq!(records[0].observation.class, EvidenceClass::Reported);
        assert!(records[0].observation.measurement.is_none());
        assert_eq!(records[0].observation.temporal, TemporalExtent::Unspecified);
    }

    #[test]
    fn invalid_coordinates_are_rejected_not_repaired() {
        let json = r#"{
          "features": [{
            "properties": {"source": "USGS", "magnitude": 4.0},
            "geometry": {"coordinates": [0.0, 91.0]}
          }]
        }"#;
        assert!(parse_dataset("usgs-earthquakes", json, NaturalEventType::Earthquake).is_empty());
    }

    #[test]
    fn audited_fire_layer_no_longer_claims_pure_observed_firms() {
        let provenance = audited_layer_provenance(Layer::Fires);
        assert_eq!(provenance.kind, DataKind::Curated);
        assert!(provenance.source.contains("synthetic"));
        assert_eq!(provenance.snapshot_date, "2025-09");
    }
}
