// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root
//! Evidence-native Confluence computation.
//!
//! The original Confluence path only knew a layer-level `DataKind`. That is too
//! coarse for mixed datasets: the checked-in Fires layer contains synthetic demo
//! FIRMS records while NASA EONET can contain observed wildfire records. Treating
//! the whole layer as either real or fictional loses information in both
//! directions.
//!
//! This module keeps non-natural point layers on the existing audited layer
//! contract, but admits natural events record-by-record from canonical planetary
//! evidence. Scenario/forecast/inferred/derived event products never contribute
//! to the factual "real systems co-locate" signal in this v1 policy.

use crate::confluence::{ConfluenceCell, CONFLUENCE_RESOLUTION};
use crate::evidence::EvidenceBearingNaturalEvent;
use crate::types::{Layer, LoadedData, NaturalEventType};
use h3o::{CellIndex, LatLng};
use mycelix_core_types::EvidenceClass;
use std::collections::HashMap;

/// Whether human/expert reports may contribute to Confluence alongside direct
/// observations. Both modes always reject scenario, forecast, inferred, and
/// derived event products.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventAdmissionPolicy {
    ObservedOnly,
    ObservedOrReported,
}

impl Default for EventAdmissionPolicy {
    fn default() -> Self {
        Self::ObservedOrReported
    }
}

/// Explainable admission/exclusion accounting for natural-event evidence.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct EvidenceConfluenceAudit {
    pub admitted_observed: u32,
    pub admitted_reported: u32,
    pub excluded_reported_by_policy: u32,
    pub excluded_scenario: u32,
    pub excluded_forecast: u32,
    pub excluded_inferred: u32,
    pub excluded_derived: u32,
}

impl EvidenceConfluenceAudit {
    pub fn admitted_total(&self) -> u32 {
        self.admitted_observed + self.admitted_reported
    }

    pub fn excluded_total(&self) -> u32 {
        self.excluded_reported_by_policy
            + self.excluded_scenario
            + self.excluded_forecast
            + self.excluded_inferred
            + self.excluded_derived
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EvidenceConfluenceResult {
    pub cells: Vec<ConfluenceCell>,
    pub audit: EvidenceConfluenceAudit,
}

/// Compute Confluence while evaluating every natural event by its own evidence
/// class instead of inheriting the layer's coarse provenance label.
///
/// Non-natural inputs retain the same point-only scope as the original
/// Confluence implementation: regions and routes are excluded because a region
/// centroid or route endpoint would misrepresent physical co-location.
pub fn compute_evidence_aware(
    data: &LoadedData,
    event_records: &[EvidenceBearingNaturalEvent],
    min_layers: usize,
    policy: EventAdmissionPolicy,
) -> EvidenceConfluenceResult {
    let mut cells: HashMap<CellIndex, (Vec<Layer>, u32)> = HashMap::new();
    let mut audit = EvidenceConfluenceAudit::default();

    let mut bin = |lat: f64, lon: f64, layer: Layer| {
        let Ok(location) = LatLng::new(lat, lon) else {
            return;
        };
        let cell = location.to_cell(CONFLUENCE_RESOLUTION);
        let entry = cells.entry(cell).or_default();
        if !entry.0.contains(&layer) {
            entry.0.push(layer);
        }
        entry.1 += 1;
    };

    // Existing non-natural real/curated point-like layers.
    for site in &data.sites {
        bin(site.lat, site.lon, Layer::Energy);
    }
    for site in &data.nuclear_sites {
        bin(site.lat, site.lon, Layer::Nuclear);
    }
    for deposit in &data.fossil_deposits {
        bin(deposit.lat, deposit.lon, Layer::FossilDeposits);
    }
    for city in &data.major_cities {
        bin(city.lat, city.lon, Layer::MajorCities);
    }
    for point in &data.chokepoints {
        bin(point.lat, point.lon, Layer::Chokepoints);
    }
    for site in &data.critical_infrastructure {
        bin(site.lat, site.lon, Layer::Infrastructure);
    }
    for project in &data.climate_projects {
        bin(project.lat, project.lon, Layer::Climate);
    }
    for shelter in &data.emergency_shelters {
        bin(shelter.lat, shelter.lon, Layer::Emergency);
    }
    for facility in &data.health_facilities {
        bin(facility.lat, facility.lon, Layer::Health);
    }

    // Natural events are evidence-native. The legacy `data.natural_events`
    // vector is deliberately ignored here so one physical event cannot be
    // counted once as a lossy marker and again as canonical evidence.
    for record in event_records {
        let admitted = match record.observation.class {
            EvidenceClass::Observed => {
                audit.admitted_observed += 1;
                true
            }
            EvidenceClass::Reported if policy == EventAdmissionPolicy::ObservedOrReported => {
                audit.admitted_reported += 1;
                true
            }
            EvidenceClass::Reported => {
                audit.excluded_reported_by_policy += 1;
                false
            }
            EvidenceClass::Scenario => {
                audit.excluded_scenario += 1;
                false
            }
            EvidenceClass::Forecast => {
                audit.excluded_forecast += 1;
                false
            }
            EvidenceClass::Inferred => {
                audit.excluded_inferred += 1;
                false
            }
            EvidenceClass::Derived => {
                audit.excluded_derived += 1;
                false
            }
        };
        if !admitted {
            continue;
        }

        let layer = match record.event.event_type {
            NaturalEventType::Earthquake => Layer::Earthquakes,
            NaturalEventType::Fire => Layer::Fires,
            NaturalEventType::Storm => Layer::Storms,
            NaturalEventType::Volcano => Layer::Volcanoes,
        };
        bin(record.event.lat, record.event.lon, layer);
    }

    let cells = cells
        .into_iter()
        .filter(|(_, (layers, _))| layers.len() >= min_layers)
        .map(|(cell, (mut layers, entity_count))| {
            layers.sort_by_key(|layer| layer.label());
            let center = LatLng::from(cell);
            ConfluenceCell {
                cell,
                lat: center.lat(),
                lon: center.lng(),
                layers,
                entity_count,
            }
        })
        .collect();

    EvidenceConfluenceResult { cells, audit }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Chokepoint, NaturalEvent, NuclearSite, ReactorType};
    use mycelix_core_types::{
        EnvironmentalObservation, ExternalEvidenceRef, GeoPoint, SpatialExtent, TemporalExtent,
        Uncertainty,
    };

    fn event(
        event_type: NaturalEventType,
        class: EvidenceClass,
        lat: f64,
        lon: f64,
        id: &str,
    ) -> EvidenceBearingNaturalEvent {
        EvidenceBearingNaturalEvent {
            event: NaturalEvent {
                lat,
                lon,
                event_type,
                magnitude: 1.0,
                name: id.into(),
            },
            observation: EnvironmentalObservation::new(
                format!("atlas:{id}"),
                "natural_event",
                class,
                None,
                SpatialExtent::Point(GeoPoint::new(lat, lon).unwrap()),
                TemporalExtent::Unspecified,
                Uncertainty::Unspecified,
                vec![ExternalEvidenceRef {
                    source_system: "test".into(),
                    resource_id: id.into(),
                    content_digest: Some(format!("sha256:{id}")),
                    retrieved_at: None,
                    license: None,
                }],
            )
            .unwrap(),
        }
    }

    fn nuclear(lat: f64, lon: f64) -> NuclearSite {
        NuclearSite {
            name: "Reactor".into(),
            lat,
            lon,
            reactor_type: ReactorType::PWR,
            capacity_mw: 1000.0,
            status: "operating".into(),
            operator: "Utility".into(),
            country: "ZA".into(),
            commission_year: 2000,
        }
    }

    fn chokepoint(lat: f64, lon: f64) -> Chokepoint {
        Chokepoint {
            name: "Port".into(),
            lat,
            lon,
            daily_barrels_m: 1.0,
            chokepoint_type: "trade".into(),
        }
    }

    #[test]
    fn synthetic_fire_never_enters_real_confluence() {
        let data = LoadedData {
            nuclear_sites: vec![nuclear(-26.2, 28.0)],
            ..Default::default()
        };
        let records = vec![event(
            NaturalEventType::Fire,
            EvidenceClass::Scenario,
            -26.2001,
            28.0001,
            "synthetic-fire",
        )];

        let result = compute_evidence_aware(
            &data,
            &records,
            2,
            EventAdmissionPolicy::ObservedOrReported,
        );
        assert!(result.cells.is_empty());
        assert_eq!(result.audit.excluded_scenario, 1);
        assert_eq!(result.audit.admitted_total(), 0);
    }

    #[test]
    fn observed_wildfire_can_participate_even_when_same_layer_has_synthetic_records() {
        let data = LoadedData {
            nuclear_sites: vec![nuclear(-26.2, 28.0)],
            ..Default::default()
        };
        let records = vec![
            event(
                NaturalEventType::Fire,
                EvidenceClass::Scenario,
                -26.2001,
                28.0001,
                "demo-fire",
            ),
            event(
                NaturalEventType::Fire,
                EvidenceClass::Observed,
                -26.2002,
                28.0002,
                "eonet-fire",
            ),
        ];

        let result = compute_evidence_aware(
            &data,
            &records,
            2,
            EventAdmissionPolicy::ObservedOrReported,
        );
        assert_eq!(result.cells.len(), 1);
        assert!(result.cells[0].layers.contains(&Layer::Nuclear));
        assert!(result.cells[0].layers.contains(&Layer::Fires));
        assert_eq!(result.audit.admitted_observed, 1);
        assert_eq!(result.audit.excluded_scenario, 1);
    }

    #[test]
    fn reported_events_are_policy_controlled() {
        let data = LoadedData {
            chokepoints: vec![chokepoint(19.42, -155.28)],
            ..Default::default()
        };
        let records = vec![event(
            NaturalEventType::Volcano,
            EvidenceClass::Reported,
            19.421,
            -155.287,
            "gvp-kilauea",
        )];

        let strict = compute_evidence_aware(
            &data,
            &records,
            2,
            EventAdmissionPolicy::ObservedOnly,
        );
        assert!(strict.cells.is_empty());
        assert_eq!(strict.audit.excluded_reported_by_policy, 1);

        let inclusive = compute_evidence_aware(
            &data,
            &records,
            2,
            EventAdmissionPolicy::ObservedOrReported,
        );
        assert_eq!(inclusive.cells.len(), 1);
        assert_eq!(inclusive.audit.admitted_reported, 1);
        assert!(inclusive.cells[0].layers.contains(&Layer::Volcanoes));
    }

    #[test]
    fn derived_forecast_and_inferred_products_are_not_double_counted_as_physical_events() {
        let records = vec![
            event(
                NaturalEventType::Storm,
                EvidenceClass::Derived,
                0.0,
                0.0,
                "derived",
            ),
            event(
                NaturalEventType::Storm,
                EvidenceClass::Forecast,
                0.0,
                0.0,
                "forecast",
            ),
            event(
                NaturalEventType::Storm,
                EvidenceClass::Inferred,
                0.0,
                0.0,
                "inferred",
            ),
        ];
        let result = compute_evidence_aware(
            &LoadedData::default(),
            &records,
            1,
            EventAdmissionPolicy::ObservedOrReported,
        );
        assert!(result.cells.is_empty());
        assert_eq!(result.audit.excluded_derived, 1);
        assert_eq!(result.audit.excluded_forecast, 1);
        assert_eq!(result.audit.excluded_inferred, 1);
    }
}
