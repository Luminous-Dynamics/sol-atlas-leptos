// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Confluence: an honest, legible signal for where real-world systems
//! co-locate — the thing a satellite photo structurally cannot show because
//! it has no notion of semantic layers to correlate.
//!
//! # Design constraints
//!
//! 1. **Never Scenario data.** Only layers known to be real may contribute.
//! 2. **No weighting, no hidden coefficients.** Cells expose the distinct
//!    categories present and a raw entity count, not a severity score.
//! 3. **Co-location is not danger.** The UI must not imply that overlapping
//!    systems are inherently risky.
//! 4. **Points only.** Region centroids and line routes are excluded.
//! 5. **Mixed-source layers fail closed.** The current Fires layer combines
//!    synthetic FIRMS demo records with observed EONET wildfire records. Until
//!    per-record evidence is carried into `LoadedData`, fires are excluded from
//!    Confluence rather than allowing synthetic records into a real-data signal.

use crate::evidence::audited_layer_provenance;
use crate::types::{DataKind, Layer, LoadedData, NaturalEventType};
use h3o::{CellIndex, LatLng, Resolution};
use std::collections::HashMap;

pub const CONFLUENCE_RESOLUTION: Resolution = Resolution::Two;

#[derive(Debug, Clone, PartialEq)]
pub struct ConfluenceCell {
    pub cell: CellIndex,
    pub lat: f64,
    pub lon: f64,
    pub layers: Vec<Layer>,
    pub entity_count: u32,
}

impl ConfluenceCell {
    pub fn summary(&self) -> String {
        let names: Vec<&str> = self.layers.iter().map(|layer| layer.label()).collect();
        format!(
            "{} real system{}: {}",
            self.layers.len(),
            if self.layers.len() == 1 { "" } else { "s" },
            names.join(", ")
        )
    }
}

/// Point-like layers whose current loaded representation can be treated as
/// wholly real. Fires are deliberately absent because that layer is mixed.
fn eligible_real_layers() -> [Layer; 9] {
    [
        Layer::Energy,
        Layer::Nuclear,
        Layer::FossilDeposits,
        Layer::Earthquakes,
        Layer::Storms,
        Layer::Volcanoes,
        Layer::MajorCities,
        Layer::Chokepoints,
        Layer::Infrastructure,
    ]
}

pub fn compute(data: &LoadedData, min_layers: usize) -> Vec<ConfluenceCell> {
    let mut cells: HashMap<CellIndex, (Vec<Layer>, u32)> = HashMap::new();

    let mut bin = |lat: f64, lon: f64, layer: Layer| {
        debug_assert_ne!(
            audited_layer_provenance(layer).kind,
            DataKind::Scenario,
            "confluence must never bin scenario-kind layer {layer:?}"
        );
        let Ok(lat_lon) = LatLng::new(lat, lon) else {
            return;
        };
        let cell = lat_lon.to_cell(CONFLUENCE_RESOLUTION);
        let entry = cells.entry(cell).or_default();
        if !entry.0.contains(&layer) {
            entry.0.push(layer);
        }
        entry.1 += 1;
    };

    for site in &data.sites {
        bin(site.lat, site.lon, Layer::Energy);
    }
    for site in &data.nuclear_sites {
        bin(site.lat, site.lon, Layer::Nuclear);
    }
    for deposit in &data.fossil_deposits {
        bin(deposit.lat, deposit.lon, Layer::FossilDeposits);
    }
    for event in &data.natural_events {
        let layer = match event.event_type {
            NaturalEventType::Earthquake => Layer::Earthquakes,
            // Fail closed until the renderer/state path preserves each fire's
            // EvidenceClass. `Layer::Fires` currently mixes Scenario and
            // Observed records, so layer-level inclusion would be dishonest.
            NaturalEventType::Fire => continue,
            NaturalEventType::Storm => Layer::Storms,
            NaturalEventType::Volcano => Layer::Volcanoes,
        };
        bin(event.lat, event.lon, layer);
    }
    for city in &data.major_cities {
        bin(city.lat, city.lon, Layer::MajorCities);
    }
    for chokepoint in &data.chokepoints {
        bin(chokepoint.lat, chokepoint.lon, Layer::Chokepoints);
    }
    for infrastructure in &data.critical_infrastructure {
        bin(
            infrastructure.lat,
            infrastructure.lon,
            Layer::Infrastructure,
        );
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

    cells
        .into_iter()
        .filter(|(_, (layers, _))| layers.len() >= min_layers)
        .map(|(cell, (mut layers, entity_count))| {
            layers.sort_by_key(|layer| layer.label());
            let lat_lon = LatLng::from(cell);
            ConfluenceCell {
                cell,
                lat: lat_lon.lat(),
                lon: lat_lon.lng(),
                layers,
                entity_count,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{
        Chokepoint, CriticalInfrastructure, EnergyType, NaturalEvent, NuclearSite, ReactorType,
        Site,
    };

    #[test]
    fn eligible_layers_are_never_scenario() {
        for layer in eligible_real_layers() {
            assert_ne!(
                audited_layer_provenance(layer).kind,
                DataKind::Scenario,
                "{layer:?} is Scenario — must never be confluence-eligible"
            );
        }
        for layer in [Layer::Climate, Layer::Emergency, Layer::Health] {
            assert_ne!(audited_layer_provenance(layer).kind, DataKind::Scenario);
        }
    }

    fn site(lat: f64, lon: f64) -> Site {
        Site {
            id: "s".into(),
            name: "Test Site".into(),
            lat,
            lon,
            energy_type: EnergyType::Solar,
            capacity_mw: 10.0,
            status: "active".into(),
            country: "US".into(),
        }
    }

    fn nuclear(lat: f64, lon: f64) -> NuclearSite {
        NuclearSite {
            name: "Test Reactor".into(),
            lat,
            lon,
            reactor_type: ReactorType::PWR,
            capacity_mw: 1000.0,
            status: "operating".into(),
            operator: "Utility".into(),
            country: "US".into(),
            commission_year: 2000,
        }
    }

    fn chokepoint(lat: f64, lon: f64) -> Chokepoint {
        Chokepoint {
            name: "Test Strait".into(),
            lat,
            lon,
            daily_barrels_m: 5.0,
            chokepoint_type: "oil".into(),
        }
    }

    fn infra(lat: f64, lon: f64) -> CriticalInfrastructure {
        CriticalInfrastructure {
            name: "Test Fab".into(),
            lat,
            lon,
            infra_type: "semiconductor".into(),
            global_share: 0.3,
            risk: "earthquake".into(),
        }
    }

    fn event(lat: f64, lon: f64, event_type: NaturalEventType) -> NaturalEvent {
        NaturalEvent {
            lat,
            lon,
            event_type,
            magnitude: 5.0,
            name: "Test Event".into(),
        }
    }

    #[test]
    fn no_confluence_below_min_layers() {
        let data = LoadedData {
            sites: vec![site(10.0, 10.0), site(10.001, 10.001)],
            ..Default::default()
        };
        assert!(compute(&data, 2).is_empty());
    }

    #[test]
    fn three_distinct_real_layers_co_locate() {
        let data = LoadedData {
            nuclear_sites: vec![nuclear(35.0, 139.0)],
            natural_events: vec![event(35.0001, 139.0001, NaturalEventType::Earthquake)],
            chokepoints: vec![chokepoint(35.0002, 139.0002)],
            ..Default::default()
        };
        let result = compute(&data, 2);
        assert_eq!(result.len(), 1);
        let cell = &result[0];
        assert_eq!(cell.layers.len(), 3);
        assert!(cell.layers.contains(&Layer::Nuclear));
        assert!(cell.layers.contains(&Layer::Earthquakes));
        assert!(cell.layers.contains(&Layer::Chokepoints));
        assert_eq!(cell.entity_count, 3);
        assert!(cell.summary().contains("3 real systems"));
    }

    #[test]
    fn mixed_fire_layer_cannot_influence_real_confluence() {
        let data = LoadedData {
            nuclear_sites: vec![nuclear(35.0, 139.0)],
            natural_events: vec![event(35.0001, 139.0001, NaturalEventType::Fire)],
            ..Default::default()
        };
        assert!(compute(&data, 2).is_empty());
    }

    #[test]
    fn scenario_data_never_appears_even_at_identical_coordinates() {
        let data = LoadedData {
            nuclear_sites: vec![nuclear(9.145, 40.49)],
            chokepoints: vec![chokepoint(9.1451, 40.4901)],
            ..Default::default()
        };
        let result = compute(&data, 2);
        assert_eq!(result.len(), 1);
        for layer in &result[0].layers {
            assert_ne!(audited_layer_provenance(*layer).kind, DataKind::Scenario);
        }
    }

    #[test]
    fn far_apart_entities_do_not_merge() {
        let data = LoadedData {
            nuclear_sites: vec![nuclear(35.0, 139.0)],
            chokepoints: vec![chokepoint(-33.0, 151.0)],
            ..Default::default()
        };
        assert!(compute(&data, 2).is_empty());
    }

    #[test]
    fn cell_center_is_a_valid_coordinate() {
        let data = LoadedData {
            nuclear_sites: vec![nuclear(35.0, 139.0)],
            critical_infrastructure: vec![infra(35.0, 139.0)],
            ..Default::default()
        };
        let result = compute(&data, 2);
        assert_eq!(result.len(), 1);
        assert!(result[0].lat.abs() <= 90.0);
        assert!(result[0].lon.abs() <= 180.0);
    }
}
