// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! JSON data loading for Sol Atlas datasets.
//!
//! Consumers provide JSON strings (from `include_str!`, file reads, or network).
//! This module handles deserialization into sol-atlas-core types. Natural-event
//! records pass through the evidence adapter first so classification and source
//! semantics have one implementation.

use crate::evidence::parse_natural_event_records;
use crate::types::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct MaglevNetwork {
    geothermal_nodes: Vec<GeothermalNode>,
    maglev_corridors: Vec<MaglevCorridor>,
}

#[derive(Deserialize)]
struct RawSite {
    id: String,
    name: String,
    lat: f64,
    lon: f64,
    energy_type: EnergyType,
    capacity_mw: f64,
    #[serde(default)]
    status: String,
    #[serde(default)]
    country: String,
}

#[derive(Deserialize)]
struct InfrastructureBundle {
    #[serde(default)]
    emergency_shelters: Vec<EmergencyShelter>,
    #[serde(default)]
    health_facilities: Vec<HealthFacility>,
    #[serde(default)]
    robotics_dispatch: Vec<RoboticsDispatch>,
}

pub fn parse_sites(json: &str) -> Result<Vec<Site>, serde_json::Error> {
    let raw: Vec<RawSite> = serde_json::from_str(json)?;
    Ok(raw
        .into_iter()
        .map(|r| Site {
            id: r.id,
            name: r.name,
            lat: r.lat,
            lon: r.lon,
            energy_type: r.energy_type,
            capacity_mw: r.capacity_mw,
            status: r.status,
            country: r.country,
        })
        .collect())
}

pub fn parse_maglev_network(
    json: &str,
) -> Result<(Vec<GeothermalNode>, Vec<MaglevCorridor>), serde_json::Error> {
    let network: MaglevNetwork = serde_json::from_str(json)?;
    Ok((network.geothermal_nodes, network.maglev_corridors))
}

pub fn parse_vaults(json: &str) -> Result<Vec<ResontiaVault>, serde_json::Error> {
    serde_json::from_str(json)
}

pub fn parse_terra_lumina(json: &str) -> Result<Vec<TerraLuminaSite>, serde_json::Error> {
    serde_json::from_str(json)
}

pub fn parse_regions(json: &str) -> Result<Vec<EarthRegion>, serde_json::Error> {
    serde_json::from_str(json)
}

pub fn parse_supply_routes(json: &str) -> Result<Vec<SupplyRoute>, serde_json::Error> {
    serde_json::from_str(json)
}

pub fn parse_climate_projects(json: &str) -> Result<Vec<ClimateProject>, serde_json::Error> {
    serde_json::from_str(json)
}

pub fn parse_infrastructure(
    json: &str,
) -> Result<
    (
        Vec<EmergencyShelter>,
        Vec<HealthFacility>,
        Vec<RoboticsDispatch>,
    ),
    serde_json::Error,
> {
    let bundle: InfrastructureBundle = serde_json::from_str(json)?;
    Ok((
        bundle.emergency_shelters,
        bundle.health_facilities,
        bundle.robotics_dispatch,
    ))
}

pub fn parse_nuclear_sites(json: &str) -> Result<Vec<NuclearSite>, serde_json::Error> {
    serde_json::from_str(json)
}

pub fn parse_fossil_deposits(json: &str) -> Result<Vec<FossilDeposit>, serde_json::Error> {
    serde_json::from_str(json)
}

fn parse_or_log<T: Default>(dataset: &str, result: Result<T, serde_json::Error>) -> T {
    result.unwrap_or_else(|error| {
        log::warn!("sol-atlas-core: failed to parse dataset '{dataset}': {error}");
        T::default()
    })
}

#[allow(clippy::too_many_arguments)]
pub fn load_all(
    sites_json: &str,
    maglev_json: &str,
    vaults_json: &str,
    terra_lumina_json: &str,
    regions_json: &str,
    supply_routes_json: &str,
    climate_json: &str,
    infrastructure_json: &str,
    fossil_deposits_json: &str,
    nuclear_sites_json: &str,
    earthquakes_json: &str,
    fires_json: &str,
    storms_json: &str,
    volcanoes_json: &str,
    cities_json: &str,
    chokepoints_json: &str,
    critical_infra_json: &str,
) -> LoadedData {
    let sites = parse_or_log("sites-clustered", parse_sites(sites_json));
    let (geothermal_nodes, maglev_corridors) =
        parse_or_log("maglev-network", parse_maglev_network(maglev_json));
    let resontia_vaults = parse_or_log("resontia-vaults", parse_vaults(vaults_json));
    let terra_lumina_sites =
        parse_or_log("terra-lumina-sites", parse_terra_lumina(terra_lumina_json));
    let earth_regions = parse_or_log("earth-regions", parse_regions(regions_json));
    let supply_routes = parse_or_log("supply-routes", parse_supply_routes(supply_routes_json));
    let climate_projects = parse_or_log("climate-projects", parse_climate_projects(climate_json));
    let (emergency_shelters, health_facilities, robotics_dispatch) =
        parse_or_log("infrastructure", parse_infrastructure(infrastructure_json));
    let fossil_deposits = parse_or_log(
        "fossil-deposits",
        parse_fossil_deposits(fossil_deposits_json),
    );
    let nuclear_sites = parse_or_log("nuclear-sites", parse_nuclear_sites(nuclear_sites_json));

    LoadedData {
        sites,
        geothermal_nodes,
        maglev_corridors,
        resontia_vaults,
        terra_lumina_sites,
        earth_regions,
        supply_routes,
        climate_projects,
        emergency_shelters,
        health_facilities,
        robotics_dispatch,
        fossil_deposits,
        nuclear_sites,
        natural_events: parse_natural_events(
            earthquakes_json,
            fires_json,
            storms_json,
            volcanoes_json,
        ),
        major_cities: parse_or_log("major-cities", serde_json::from_str(cities_json)),
        chokepoints: parse_or_log("chokepoints", serde_json::from_str(chokepoints_json)),
        critical_infrastructure: parse_or_log(
            "critical-infrastructure",
            serde_json::from_str(critical_infra_json),
        ),
    }
}

pub fn parse_shipping_lanes(json: &str) -> Vec<Vec<[f64; 2]>> {
    parse_or_log("shipping-lanes", serde_json::from_str(json))
}

/// Preserve the renderer-facing record shape while sourcing it from the
/// evidence-bearing adapter. Consumers that need provenance can call
/// `crate::evidence::parse_natural_event_records` directly; ATLAS-3 will carry
/// those records into selection/inspection state.
fn parse_natural_events(
    earthquakes: &str,
    fires: &str,
    eonet: &str,
    volcanoes: &str,
) -> Vec<NaturalEvent> {
    parse_natural_event_records(earthquakes, fires, eonet, volcanoes)
        .into_iter()
        .map(|record| record.event)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_geojson_earthquakes() {
        let json = r#"{
            "type": "FeatureCollection",
            "features": [
                {
                    "type": "Feature",
                    "properties": {"magnitude": 5.1, "place": "Peru", "source": "USGS", "type": "earthquake"},
                    "geometry": {"type": "Point", "coordinates": [-80.5, -5.2, 10]}
                },
                {
                    "type": "Feature",
                    "properties": {"magnitude": 3.2, "place": "Japan", "source": "USGS"},
                    "geometry": {"type": "Point", "coordinates": [139.7, 35.7, 5]}
                }
            ]
        }"#;
        let events = parse_natural_events(json, "[]", "[]", "[]");
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].event_type, NaturalEventType::Earthquake);
        assert!((events[0].lat - (-5.2)).abs() < 0.01);
        assert!((events[0].magnitude - 5.1).abs() < 0.01);
        assert_eq!(events[0].name, "Peru");
    }

    #[test]
    fn parse_geojson_fires_preserves_renderer_brightness_scale() {
        let json = r#"{
            "type": "FeatureCollection",
            "features": [
                {
                    "type": "Feature",
                    "properties": {"brightness": 387.0, "confidence": 94, "source": "Demo Data", "type": "fire"},
                    "geometry": {"type": "Point", "coordinates": [25.0, -30.0]}
                }
            ]
        }"#;
        let events = parse_natural_events("[]", json, "[]", "[]");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, NaturalEventType::Fire);
        assert!((events[0].magnitude - 3.87).abs() < 0.01);
    }

    #[test]
    fn eonet_wildfire_is_not_misclassified_as_storm() {
        let json = r#"{
            "features": [{
                "properties": {
                    "source": "NASA EONET",
                    "title": "Prescribed Fire",
                    "categories": ["Wildfires"],
                    "event_id": "EONET_1",
                    "magnitude": 1047.0,
                    "magnitude_unit": "acres"
                },
                "geometry": {"coordinates": [-84.0, 30.0]}
            }]
        }"#;
        let events = parse_natural_events("[]", "[]", json, "[]");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, NaturalEventType::Fire);
    }

    #[test]
    fn parse_geojson_empty() {
        let events = parse_natural_events("[]", "[]", "[]", "[]");
        assert!(events.is_empty());
    }

    #[test]
    fn parse_geojson_invalid() {
        let events = parse_natural_events("not json", "{}", "null", "");
        assert!(events.is_empty());
    }

    #[test]
    fn parse_shipping_lanes_roundtrip() {
        let json = "[[[-80.0, 25.0], [-10.0, 50.0]], [[100.0, -5.0], [120.0, 10.0]]]";
        let lanes = parse_shipping_lanes(json);
        assert_eq!(lanes.len(), 2);
        assert_eq!(lanes[0].len(), 2);
        assert!((lanes[0][0][0] - (-80.0)).abs() < 0.01);
    }
}

#[cfg(test)]
mod city_tests {
    use super::*;

    #[test]
    fn parse_major_cities() {
        let json = r#"[
            {"name":"Tokyo","country":"JP","lat":35.68,"lon":139.69,"population":13960000},
            {"name":"Mumbai","country":"IN","lat":19.07,"lon":72.87,"population":12700000}
        ]"#;
        let cities: Vec<MajorCity> = serde_json::from_str(json).unwrap();
        assert_eq!(cities.len(), 2);
        assert_eq!(cities[0].name, "Tokyo");
        assert_eq!(cities[0].population, 13_960_000);
        assert!((cities[1].lat - 19.07).abs() < 0.01);
    }

    #[test]
    fn parse_major_cities_empty() {
        let cities: Vec<MajorCity> = serde_json::from_str("[]").unwrap();
        assert!(cities.is_empty());
    }
}

#[cfg(test)]
mod infra_tests {
    use super::*;

    #[test]
    fn parse_chokepoints() {
        let json = r#"[
            {"name":"Hormuz","lat":26.5,"lon":56.3,"daily_barrels_m":21.0,"type":"oil"},
            {"name":"Malacca","lat":2.5,"lon":101.5,"daily_barrels_m":16.0,"type":"trade"}
        ]"#;
        let chokepoints: Vec<Chokepoint> = serde_json::from_str(json).unwrap();
        assert_eq!(chokepoints.len(), 2);
        assert_eq!(chokepoints[0].name, "Hormuz");
        assert!((chokepoints[0].daily_barrels_m - 21.0).abs() < 0.01);
        assert_eq!(chokepoints[1].chokepoint_type, "trade");
    }

    #[test]
    fn parse_critical_infrastructure() {
        let json = r#"[
            {"name":"TSMC","lat":23.75,"lon":120.32,"type":"semiconductor","global_share":0.54,"risk":"earthquake"}
        ]"#;
        let infra: Vec<CriticalInfrastructure> = serde_json::from_str(json).unwrap();
        assert_eq!(infra.len(), 1);
        assert_eq!(infra[0].infra_type, "semiconductor");
        assert!((infra[0].global_share - 0.54).abs() < 0.01);
        assert_eq!(infra[0].risk, "earthquake");
    }

    #[test]
    fn layer_count() {
        let all = Layer::all();
        assert!(all.len() >= 18, "Expected at least 18 layers, got {}", all.len());
        assert!(all.contains(&Layer::Infrastructure));
        assert!(all.contains(&Layer::Chokepoints));
    }

    #[test]
    fn all_layers_have_labels() {
        for layer in Layer::all() {
            assert!(!layer.label().is_empty(), "Layer {layer:?} has empty label");
            assert!(
                !layer.css_color().is_empty(),
                "Layer {layer:?} has empty css_color"
            );
            let rgb = layer.rgb();
            for component in rgb {
                assert!(
                    (0.0..=1.0).contains(&component),
                    "Layer {layer:?} RGB out of range: {rgb:?}"
                );
            }
        }
    }
}

#[cfg(test)]
mod e2e_tests {
    use super::*;

    #[test]
    fn loaded_data_has_all_fields() {
        let data = load_all(
            "[]",
            r#"{"geothermal_nodes":[],"maglev_corridors":[]}"#,
            "[]",
            "[]",
            "[]",
            "[]",
            "[]",
            r#"{"emergency_shelters":[],"health_facilities":[],"robotics_dispatch":[]}"#,
            "[]",
            "[]",
            "[]",
            "[]",
            "[]",
            "[]",
            "[]",
            "[]",
            "[]",
        );
        assert!(data.sites.is_empty());
        assert!(data.natural_events.is_empty());
        assert!(data.major_cities.is_empty());
        assert!(data.chokepoints.is_empty());
        assert!(data.critical_infrastructure.is_empty());
    }

    #[test]
    fn loaded_data_handles_invalid_json() {
        let data = load_all(
            "not json", "bad", "{}", "null", "[]", "", "[]", "[]", "[]", "[]", "[]", "[]", "[]",
            "[]", "[]", "[]", "[]",
        );
        assert!(data.sites.is_empty());
    }
}
