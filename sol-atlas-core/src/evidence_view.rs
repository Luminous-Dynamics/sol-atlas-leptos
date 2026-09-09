// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
//! Presentation-safe summaries of canonical evidence records.
//!
//! Renderer crates should not need to understand or duplicate the underlying
//! Mycelix evidence enums merely to display provenance.

use crate::evidence::EvidenceBearingNaturalEvent;
use chrono::DateTime;
use mycelix_core_types::{EvidenceClass, TemporalExtent, Uncertainty};

#[derive(Debug, Clone, PartialEq)]
pub struct EventEvidenceSummary {
    pub observation_id: String,
    pub class: &'static str,
    pub source: String,
    pub resource: String,
    pub source_time: String,
    pub measurement_value: Option<String>,
    pub measurement_unit: Option<String>,
    pub uncertainty: String,
}

pub fn summarize_event_evidence(record: &EvidenceBearingNaturalEvent) -> EventEvidenceSummary {
    let observation = &record.observation;
    let source_ref = observation.evidence.first();

    EventEvidenceSummary {
        observation_id: observation.id.clone(),
        class: class_label(observation.class),
        source: source_ref
            .map(|reference| reference.source_system.clone())
            .unwrap_or_else(|| "unknown source".to_string()),
        resource: source_ref
            .map(|reference| reference.resource_id.clone())
            .unwrap_or_else(|| "unknown resource".to_string()),
        source_time: temporal_label(observation.temporal),
        measurement_value: observation
            .measurement
            .as_ref()
            .map(|measurement| format_measurement_value(measurement.value)),
        measurement_unit: observation
            .measurement
            .as_ref()
            .map(|measurement| measurement.unit.clone()),
        uncertainty: uncertainty_label(&observation.uncertainty),
    }
}

fn class_label(class: EvidenceClass) -> &'static str {
    match class {
        EvidenceClass::Reported => "reported",
        EvidenceClass::Observed => "observed",
        EvidenceClass::Derived => "derived",
        EvidenceClass::Inferred => "inferred",
        EvidenceClass::Forecast => "forecast",
        EvidenceClass::Scenario => "scenario",
    }
}

fn temporal_label(temporal: TemporalExtent) -> String {
    match temporal {
        TemporalExtent::Unspecified => "source time not supplied".to_string(),
        TemporalExtent::Instant(timestamp) => format_unix(timestamp),
        TemporalExtent::Interval { start, end } => {
            format!("{} to {}", format_unix(start), format_unix(end))
        }
    }
}

fn format_unix(timestamp: i64) -> String {
    DateTime::from_timestamp(timestamp, 0)
        .map(|time| time.format("%Y-%m-%d %H:%M:%S UTC").to_string())
        .unwrap_or_else(|| format!("Unix {timestamp}"))
}

fn format_measurement_value(value: f64) -> String {
    if value.abs() >= 100.0 {
        format!("{value:.0}")
    } else if value.abs() >= 10.0 {
        format!("{value:.1}")
    } else {
        format!("{value:.2}")
    }
}

fn uncertainty_label(uncertainty: &Uncertainty) -> String {
    match uncertainty {
        Uncertainty::Unspecified => "not supplied".to_string(),
        Uncertainty::Interval {
            lower,
            upper,
            confidence,
        } => match confidence {
            Some(confidence) => format!(
                "{lower:.3}–{upper:.3} ({:.1}% coverage)",
                confidence * 100.0
            ),
            None => format!("{lower:.3}–{upper:.3}"),
        },
        Uncertainty::StandardDeviation(sigma) => format!("σ = {sigma:.3}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{NaturalEvent, NaturalEventType};
    use mycelix_core_types::{
        EnvironmentalObservation, ExternalEvidenceRef, GeoPoint, Measurement, SpatialExtent,
    };

    fn record(class: EvidenceClass, measurement: Option<Measurement>) -> EvidenceBearingNaturalEvent {
        EvidenceBearingNaturalEvent {
            event: NaturalEvent {
                lat: -5.2,
                lon: -80.5,
                event_type: NaturalEventType::Earthquake,
                magnitude: 5.1,
                name: "Peru".into(),
            },
            observation: EnvironmentalObservation::new(
                "atlas:test",
                "earthquake",
                class,
                measurement,
                SpatialExtent::Point(GeoPoint::new(-5.2, -80.5).unwrap()),
                TemporalExtent::instant(1_758_591_159),
                Uncertainty::Unspecified,
                vec![ExternalEvidenceRef {
                    source_system: "USGS".into(),
                    resource_id: "event:test".into(),
                    content_digest: None,
                    retrieved_at: None,
                    license: None,
                }],
            )
            .unwrap(),
        }
    }

    #[test]
    fn summary_surfaces_source_class_time_and_unit() {
        let record = record(
            EvidenceClass::Observed,
            Some(Measurement::new(5.1, "Mw").unwrap()),
        );
        let summary = summarize_event_evidence(&record);
        assert_eq!(summary.class, "observed");
        assert_eq!(summary.source, "USGS");
        assert_eq!(summary.measurement_unit.as_deref(), Some("Mw"));
        assert!(summary.source_time.contains("UTC"));
    }

    #[test]
    fn absent_measurement_remains_absent() {
        let summary = summarize_event_evidence(&record(EvidenceClass::Reported, None));
        assert!(summary.measurement_value.is_none());
        assert!(summary.measurement_unit.is_none());
    }
}
