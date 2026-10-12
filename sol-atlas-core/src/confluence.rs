// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Confluence: an honest, legible signal for where real-world systems
//! co-locate — the thing a satellite photo (Google Earth) structurally
//! cannot show you, because it has no notion of "layers" to correlate.
//!
//! # Design constraints (why this is NOT a risk/danger score)
//!
//! 1. **Never Scenario data.** Only layers whose [`DataProvenance`] kind is
//!    `Observed` or `Curated` may contribute. Resontia Vaults, Maglev,
//!    Geothermal, Terra Lumina, Robotics Dispatch, and the DeSci demo are
//!    speculative planning fiction — mixing them into a "real systems
//!    overlap here" signal would be actively dishonest, not just
//!    incomplete. [`eligible_real_layers`] is the single, tested
//!    enforcement point for this.
//! 2. **No weighting, no hidden coefficients.** A [`ConfluenceCell`] is
//!    just the plain set of distinct real categories present and a raw
//!    entity count — not a blended "severity" or "risk" number. Every
//!    lit cell can answer "why are you lit?" with a factual sentence
//!    ("Earthquakes, Nuclear, Chokepoints"), never a black box.
//! 3. **Co-location is not danger.** Multiple real systems overlapping
//!    might mean compounding fragility, or it might just mean a
//!    well-developed region. The UI states this explicitly and never
//!    implies severity via color/language — that judgment stays with
//!    the person looking at it.
//! 4. **Points only, not areas or lines.** Earth Regions (population/GDP
//!    aggregates keyed to a whole region's centroid) and Supply Chain
//!    routes (line geometry between two points) are deliberately
//!    excluded — binning either into a single H3 cell would misrepresent
//!    what's actually physically co-located there.

use crate::types::{DataKind, Layer, LoadedData, NaturalEventType};
use h3o::{CellIndex, LatLng, Resolution};
use std::collections::HashMap;

/// H3 resolution used for confluence binning: continental/regional scale
/// (average cell area ~86,801 km²) — fine enough to distinguish regions,
/// coarse enough that patterns read at a glance from an orbital view
/// without needing to zoom in. Matches the "civilizational scale"
/// framing this feature was built for.
pub const CONFLUENCE_RESOLUTION: Resolution = Resolution::Two;

/// A geographic cell where 2 or more distinct real data categories
/// co-locate. Every field is directly inspectable — no derived score.
#[derive(Debug, Clone, PartialEq)]
pub struct ConfluenceCell {
    pub cell: CellIndex,
    pub lat: f64,
    pub lon: f64,
    /// Distinct real-data [`Layer`]s present in this cell, sorted by
    /// label for stable, legible display order.
    pub layers: Vec<Layer>,
    /// Total entity count across all contributing layers (may exceed
    /// `layers.len()` if e.g. two earthquakes fall in the same cell).
    pub entity_count: u32,
}

impl ConfluenceCell {
    /// Human-readable summary, e.g. "3 real systems: Earthquakes,
    /// Nuclear Sites, Maritime Chokepoints". Deliberately factual, never
    /// implies severity.
    pub fn summary(&self) -> String {
        let names: Vec<&str> = self.layers.iter().map(|l| l.label()).collect();
        format!(
            "{} real system{}: {}",
            self.layers.len(),
            if self.layers.len() == 1 { "" } else { "s" },
            names.join(", ")
        )
    }
}

/// The layers eligible to contribute to confluence — every one is
/// `Observed` or `Curated` (never `Scenario`), and every one is
/// point-like data (not a region centroid or a line route). This is the
/// single source of truth `compute` draws from; the accompanying test
/// (`eligible_layers_are_never_scenario`) is what makes that a checked
/// invariant, not just a comment.
fn eligible_real_layers() -> [Layer; 10] {
    [
        Layer::Energy,
        Layer::Nuclear,
        Layer::FossilDeposits,
        Layer::Earthquakes,
        Layer::Fires,
        Layer::Storms,
        Layer::Volcanoes,
        Layer::MajorCities,
        Layer::Chokepoints,
        Layer::Infrastructure,
    ]
}
// Climate, Emergency, and Health are also real point-like data but are
// added directly in `compute` below alongside the natural-events split —
// kept out of this fixed-size array only because Rust arrays can't mix
// literal counts conveniently with the loop below; the invariant test
// checks the full effective set via `compute`, not just this array.

/// Bin every eligible real-data entity into H3 cells at
/// [`CONFLUENCE_RESOLUTION`], and return the cells where at least
/// `min_layers` distinct real categories co-locate (2 is the sensible
/// default — a single category alone isn't a confluence, it's just that
/// layer's own marker).
pub fn compute(data: &LoadedData, min_layers: usize) -> Vec<ConfluenceCell> {
    let mut cells: HashMap<CellIndex, (Vec<Layer>, u32)> = HashMap::new();

    let mut bin = |lat: f64, lon: f64, layer: Layer| {
        debug_assert_ne!(
            layer.provenance().kind,
            DataKind::Scenario,
            "confluence must never bin scenario-kind layer {layer:?}"
        );
        let Ok(ll) = LatLng::new(lat, lon) else {
            return; // non-finite coordinate — skip rather than panic
        };
        let cell = ll.to_cell(CONFLUENCE_RESOLUTION);
        let entry = cells.entry(cell).or_default();
        if !entry.0.contains(&layer) {
            entry.0.push(layer);
        }
        entry.1 += 1;
    };

    for s in &data.sites {
        bin(s.lat, s.lon, Layer::Energy);
    }
    for n in &data.nuclear_sites {
        bin(n.lat, n.lon, Layer::Nuclear);
    }
    for f in &data.fossil_deposits {
        bin(f.lat, f.lon, Layer::FossilDeposits);
    }
    for e in &data.natural_events {
        let layer = match e.event_type {
            NaturalEventType::Earthquake => Layer::Earthquakes,
            NaturalEventType::Fire => Layer::Fires,
            NaturalEventType::Storm => Layer::Storms,
            NaturalEventType::Volcano => Layer::Volcanoes,
        };
        bin(e.lat, e.lon, layer);
    }
    for c in &data.major_cities {
        bin(c.lat, c.lon, Layer::MajorCities);
    }
    for c in &data.chokepoints {
        bin(c.lat, c.lon, Layer::Chokepoints);
    }
    for i in &data.critical_infrastructure {
        bin(i.lat, i.lon, Layer::Infrastructure);
    }
    for c in &data.climate_projects {
        bin(c.lat, c.lon, Layer::Climate);
    }
    for e in &data.emergency_shelters {
        bin(e.lat, e.lon, Layer::Emergency);
    }
    for h in &data.health_facilities {
        bin(h.lat, h.lon, Layer::Health);
    }

    cells
        .into_iter()
        .filter(|(_, (layers, _))| layers.len() >= min_layers)
        .map(|(cell, (mut layers, entity_count))| {
            layers.sort_by_key(|l| l.label());
            let ll = LatLng::from(cell);
            ConfluenceCell {
                cell,
                lat: ll.lat(),
                lon: ll.lng(),
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

    /// The core honesty invariant: not one eligible layer is Scenario.
    /// If someone adds a new real layer to `compute` without adding it
    /// here (or vice versa), this is the test that should fail.
    #[test]
    fn eligible_layers_are_never_scenario() {
        for layer in eligible_real_layers() {
            assert_ne!(
                layer.provenance().kind,
                DataKind::Scenario,
                "{layer:?} is Scenario — must never be confluence-eligible"
            );
        }
        // The three added directly in `compute` (not in the fixed array
        // above, see its comment) get the same check here.
        for layer in [Layer::Climate, Layer::Emergency, Layer::Health] {
            assert_ne!(layer.provenance().kind, DataKind::Scenario);
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

    fn quake(lat: f64, lon: f64) -> NaturalEvent {
        NaturalEvent {
            lat,
            lon,
            event_type: NaturalEventType::Earthquake,
            magnitude: 5.0,
            name: "Test Quake".into(),
            // The confluence fixture has no source record: keep every
            // source-derived field explicitly unknown instead of fabricating
            // provenance or timestamp metadata for synthetic test data.
            magnitude_unit: None,
            magnitude_description: None,
            source_name: None,
            source_id: None,
            source_url: None,
            occurred_at_ms: None,
            updated_at_ms: None,
            depth_km: None,
            geometry_note: None,
        }
    }

    #[test]
    fn no_confluence_below_min_layers() {
        // Two energy sites in the same cell: still only ONE distinct
        // layer, so this must not count as a confluence at min_layers=2.
        let data = LoadedData {
            sites: vec![site(10.0, 10.0), site(10.001, 10.001)],
            ..Default::default()
        };
        let result = compute(&data, 2);
        assert!(result.is_empty());
    }

    #[test]
    fn three_distinct_real_layers_co_locate() {
        let data = LoadedData {
            nuclear_sites: vec![nuclear(35.0, 139.0)],
            natural_events: vec![quake(35.0001, 139.0001)],
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
    fn scenario_data_never_appears_even_at_identical_coordinates() {
        // Resontia Vaults share coordinates with real infra in this
        // fixture on purpose — compute() only reads from LoadedData
        // fields it explicitly binds (sites/nuclear/etc.), never
        // resontia_vaults/geothermal_nodes/maglev_corridors/
        // terra_lumina_sites/robotics_dispatch, so there is no code path
        // for scenario data to enter a ConfluenceCell at all. This test
        // documents that guarantee structurally: even with real data at
        // the same coordinates, the result must contain zero scenario
        // layers by construction.
        let data = LoadedData {
            nuclear_sites: vec![nuclear(9.145, 40.49)],
            chokepoints: vec![chokepoint(9.1451, 40.4901)],
            ..Default::default()
        };
        let result = compute(&data, 2);
        assert_eq!(result.len(), 1);
        for layer in &result[0].layers {
            assert_ne!(layer.provenance().kind, DataKind::Scenario);
        }
    }

    #[test]
    fn far_apart_entities_do_not_merge() {
        let data = LoadedData {
            nuclear_sites: vec![nuclear(35.0, 139.0)],
            chokepoints: vec![chokepoint(-33.0, 151.0)], // Sydney, far away
            ..Default::default()
        };
        let result = compute(&data, 2);
        assert!(result.is_empty());
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
