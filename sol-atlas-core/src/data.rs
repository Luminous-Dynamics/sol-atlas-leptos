// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! JSON data loading for Sol Atlas datasets.
//!
//! Consumers provide JSON strings (from `include_str!`, file reads, or network).
//! This module handles deserialization into sol-atlas-core types.

use crate::types::*;
use serde::{Deserialize, Serialize};

/// Intermediate type for maglev-network.json (two arrays in one object).
#[derive(Deserialize)]
struct MaglevNetwork {
    geothermal_nodes: Vec<GeothermalNode>,
    maglev_corridors: Vec<MaglevCorridor>,
}

/// Raw site entry from clustered JSON (has extra fields we drop).
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

/// Intermediate type for infrastructure.json (three arrays in one object).
#[derive(Deserialize)]
struct InfrastructureBundle {
    #[serde(default)]
    emergency_shelters: Vec<EmergencyShelter>,
    #[serde(default)]
    health_facilities: Vec<HealthFacility>,
    #[serde(default)]
    robotics_dispatch: Vec<RoboticsDispatch>,
}

/// Parse energy sites from sites-clustered.json.
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

/// Parse maglev-network.json → (geothermal_nodes, maglev_corridors).
pub fn parse_maglev_network(
    json: &str,
) -> Result<(Vec<GeothermalNode>, Vec<MaglevCorridor>), serde_json::Error> {
    let network: MaglevNetwork = serde_json::from_str(json)?;
    Ok((network.geothermal_nodes, network.maglev_corridors))
}

/// Parse resontia-vaults.json.
pub fn parse_vaults(json: &str) -> Result<Vec<ResontiaVault>, serde_json::Error> {
    serde_json::from_str(json)
}

/// Parse terra-lumina-sites.json.
pub fn parse_terra_lumina(json: &str) -> Result<Vec<TerraLuminaSite>, serde_json::Error> {
    serde_json::from_str(json)
}

/// Parse earth-regions.json.
pub fn parse_regions(json: &str) -> Result<Vec<EarthRegion>, serde_json::Error> {
    serde_json::from_str(json)
}

/// Parse supply-routes.json.
pub fn parse_supply_routes(json: &str) -> Result<Vec<SupplyRoute>, serde_json::Error> {
    serde_json::from_str(json)
}

/// Parse climate-projects.json.
pub fn parse_climate_projects(json: &str) -> Result<Vec<ClimateProject>, serde_json::Error> {
    serde_json::from_str(json)
}

/// Parse infrastructure.json → (emergency_shelters, health_facilities, robotics_dispatch).
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

/// Parse nuclear-sites.json.
pub fn parse_nuclear_sites(json: &str) -> Result<Vec<NuclearSite>, serde_json::Error> {
    serde_json::from_str(json)
}

/// Parse fossil-deposits.json.
pub fn parse_fossil_deposits(json: &str) -> Result<Vec<FossilDeposit>, serde_json::Error> {
    serde_json::from_str(json)
}

/// Parse a dataset, logging (never silently swallowing) any failure.
///
/// A malformed dataset still degrades to empty rather than crashing the
/// renderer, but the failure is visible in the console/log instead of the
/// layer just quietly vanishing.
fn parse_or_log<T: Default>(dataset: &str, result: Result<T, serde_json::Error>) -> T {
    result.unwrap_or_else(|e| {
        log::warn!("sol-atlas-core: failed to parse dataset '{dataset}': {e}");
        T::default()
    })
}

/// Machine-readable health of one source dataset.
///
/// ValidEmpty means the source parsed successfully and contained zero records.
/// Malformed means parsing failed; it must not be presented as a valid empty
/// inventory. Unavailable, StaleCache and Rejected are reserved for retrieval,
/// cache and authentication layers that own those decisions.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DatasetLoadState {
    Loaded,
    ValidEmpty,
    Malformed,
    Unavailable,
    StaleCache,
    Rejected,
}

/// Per-dataset parse and freshness diagnostic.
///
/// record_count counts source records represented by the parsed value when
/// known. It is None for malformed input, not zero.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DatasetLoadHealth {
    pub dataset: String,
    pub state: DatasetLoadState,
    pub record_count: Option<usize>,
    pub diagnostic: Option<String>,
}

/// Data plus diagnostics for evidence-aware consumers.
/// The legacy load_all function remains available and returns only data.
#[derive(Debug, Clone, Default)]
pub struct LoadedDataReport {
    pub data: LoadedData,
    pub health: Vec<DatasetLoadHealth>,
}

fn parse_or_log_with_health<T: Default>(
    dataset: &str,
    result: Result<T, serde_json::Error>,
    record_count: impl FnOnce(&T) -> usize,
    health: &mut Vec<DatasetLoadHealth>,
) -> T {
    match result {
        Ok(value) => {
            let count = record_count(&value);
            health.push(DatasetLoadHealth {
                dataset: dataset.to_string(),
                state: if count == 0 {
                    DatasetLoadState::ValidEmpty
                } else {
                    DatasetLoadState::Loaded
                },
                record_count: Some(count),
                diagnostic: None,
            });
            value
        }
        Err(error) => {
            let diagnostic = error.to_string();
            log::warn!("sol-atlas-core: failed to parse dataset '{dataset}': {diagnostic}");
            health.push(DatasetLoadHealth {
                dataset: dataset.to_string(),
                state: DatasetLoadState::Malformed,
                record_count: None,
                diagnostic: Some(diagnostic),
            });
            T::default()
        }
    }
}

/// Load all datasets from their respective JSON strings into a single `LoadedData`.
pub fn load_all_with_health(
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
) -> LoadedDataReport {
    let mut health = Vec::new();
    let sites = parse_or_log_with_health(
        "sites-clustered",
        parse_sites(sites_json),
        |v| v.len(),
        &mut health,
    );
    let (geothermal_nodes, maglev_corridors) = parse_or_log_with_health(
        "maglev-network",
        parse_maglev_network(maglev_json),
        |(nodes, corridors)| nodes.len() + corridors.len(),
        &mut health,
    );
    let resontia_vaults = parse_or_log_with_health(
        "resontia-vaults",
        parse_vaults(vaults_json),
        |v| v.len(),
        &mut health,
    );
    let terra_lumina_sites = parse_or_log_with_health(
        "terra-lumina-sites",
        parse_terra_lumina(terra_lumina_json),
        |v| v.len(),
        &mut health,
    );
    let earth_regions = parse_or_log_with_health(
        "earth-regions",
        parse_regions(regions_json),
        |v| v.len(),
        &mut health,
    );
    let supply_routes = parse_or_log_with_health(
        "supply-routes",
        parse_supply_routes(supply_routes_json),
        |v| v.len(),
        &mut health,
    );
    let climate_projects = parse_or_log_with_health(
        "climate-projects",
        parse_climate_projects(climate_json),
        |v| v.len(),
        &mut health,
    );
    let (emergency_shelters, health_facilities, robotics_dispatch) =
        parse_or_log_with_health(
            "infrastructure",
            parse_infrastructure(infrastructure_json),
            |(shelters, facilities, robotics)| {
                shelters.len() + facilities.len() + robotics.len()
            },
            &mut health,
        );
    let fossil_deposits = parse_or_log_with_health(
        "fossil-deposits",
        parse_fossil_deposits(fossil_deposits_json),
        |v| v.len(),
        &mut health,
    );
    let nuclear_sites = parse_or_log_with_health(
        "nuclear-sites",
        parse_nuclear_sites(nuclear_sites_json),
        |v| v.len(),
        &mut health,
    );
    let (natural_events, natural_event_health) = parse_natural_events_with_health(
        earthquakes_json,
        fires_json,
        storms_json,
        volcanoes_json,
    );
    health.extend(natural_event_health);
    let major_cities = parse_or_log_with_health(
        "major-cities",
        serde_json::from_str(cities_json),
        |v: &Vec<MajorCity>| v.len(),
        &mut health,
    );
    let chokepoints = parse_or_log_with_health(
        "chokepoints",
        serde_json::from_str(chokepoints_json),
        |v: &Vec<Chokepoint>| v.len(),
        &mut health,
    );
    let critical_infrastructure = parse_or_log_with_health(
        "critical-infrastructure",
        serde_json::from_str(critical_infra_json),
        |v: &Vec<CriticalInfrastructure>| v.len(),
        &mut health,
    );

    let data = LoadedData {
        sites, geothermal_nodes, maglev_corridors, resontia_vaults, terra_lumina_sites,
        earth_regions, supply_routes, climate_projects, emergency_shelters, health_facilities,
        robotics_dispatch, fossil_deposits, nuclear_sites, natural_events, major_cities,
        chokepoints, critical_infrastructure,
    };
    LoadedDataReport { data, health }
}

/// Backwards-compatible loader. Use load_all_with_health when callers must
/// distinguish malformed feeds from valid empty inventories.
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
    load_all_with_health(
        sites_json,
        maglev_json,
        vaults_json,
        terra_lumina_json,
        regions_json,
        supply_routes_json,
        climate_json,
        infrastructure_json,
        fossil_deposits_json,
        nuclear_sites_json,
        earthquakes_json,
        fires_json,
        storms_json,
        volcanoes_json,
        cities_json,
        chokepoints_json,
        critical_infra_json,
    ).data
}

/// Parse simplified shipping lane routes (Vec of Vec of [lon, lat]).
pub fn parse_shipping_lanes(json: &str) -> Vec<Vec<[f64; 2]>> {
    parse_or_log("shipping-lanes", serde_json::from_str(json))
}

/// Parse GeoJSON natural events from USGS, NASA EONET, FIRMS, and volcano data.
fn parse_natural_events(
    earthquakes: &str,
    fires: &str,
    storms: &str,
    volcanoes: &str,
) -> Vec<NaturalEvent> {
    parse_natural_events_with_health(earthquakes, fires, storms, volcanoes).0
}

fn parse_natural_events_with_health(
    earthquakes: &str,
    fires: &str,
    storms: &str,
    volcanoes: &str,
) -> (Vec<NaturalEvent>, Vec<DatasetLoadHealth>) {
    let mut events = Vec::new();
    let mut health = Vec::new();

    // Parse GeoJSON FeatureCollections (all share same structure)
    fn parse_geojson(
        name: &str,
        json: &str,
        event_type: NaturalEventType,
        health: &mut Vec<DatasetLoadHealth>,
    ) -> Vec<NaturalEvent> {
        #[derive(Deserialize)]
        struct FeatureCollection {
            features: Vec<Feature>,
        }
        #[derive(Deserialize)]
        struct Feature {
            properties: Properties,
            geometry: Geometry,
        }
        #[derive(Deserialize)]
        struct Properties {
            #[serde(default)]
            magnitude: Option<f64>,
            #[serde(default)]
            brightness: Option<f64>,
            #[serde(default)]
            title: Option<String>,
            #[serde(default)]
            place: Option<String>,
            #[serde(default)]
            name: Option<String>,
            #[serde(rename = "type", default)]
            event_kind: Option<String>,
        }
        #[derive(Deserialize)]
        struct Geometry {
            coordinates: Vec<f64>,
        }

        let fc = match serde_json::from_str::<FeatureCollection>(json) {
            Ok(fc) => fc,
            Err(e) => {
                let diagnostic = e.to_string();
                log::warn!("sol-atlas-core: failed to parse dataset '{name}': {diagnostic}");
                health.push(DatasetLoadHealth {
                    dataset: name.to_string(),
                    state: DatasetLoadState::Malformed,
                    record_count: None,
                    diagnostic: Some(diagnostic),
                });
                return vec![];
            }
        };
        let source_count = fc.features.len();
        let events: Vec<NaturalEvent> = fc.features
            .iter()
            .filter_map(|f| {
                let coords = &f.geometry.coordinates;
                if coords.len() < 2 {
                    return None;
                }
                let lon = coords[0];
                let lat = coords[1];
                if lat.abs() > 90.0 || lon.abs() > 180.0 {
                    return None;
                }
                let magnitude = f
                    .properties
                    .magnitude
                    .or(f.properties.brightness.map(|b| b / 100.0))
                    .unwrap_or(1.0);
                let name = f
                    .properties
                    .title
                    .as_deref()
                    .or(f.properties.place.as_deref())
                    .or(f.properties.name.as_deref())
                    .unwrap_or("Unknown")
                    .to_string();
                Some(NaturalEvent {
                    lat,
                    lon,
                    event_type,
                    magnitude,
                    name,
                })
            })
            .collect();
        health.push(DatasetLoadHealth {
            dataset: name.to_string(),
            state: if source_count == 0 {
                DatasetLoadState::ValidEmpty
            } else {
                DatasetLoadState::Loaded
            },
            record_count: Some(source_count),
            diagnostic: if events.len() < source_count {
                Some(format!(
                    "{} source features were excluded by coordinate validation",
                    source_count - events.len()
                ))
            } else {
                None
            },
        });
        events
    }

    events.extend(parse_geojson(
        "usgs-earthquakes",
        earthquakes,
        NaturalEventType::Earthquake,
        &mut health,
    ));
    events.extend(parse_geojson(
        "nasa-firms",
        fires,
        NaturalEventType::Fire,
        &mut health,
    ));
    events.extend(parse_geojson(
        "nasa-eonet",
        storms,
        NaturalEventType::Storm,
        &mut health,
    ));
    events.extend(parse_geojson(
        "volcanoes",
        volcanoes,
        NaturalEventType::Volcano,
        &mut health,
    ));
    (events, health)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagnosed_loader_distinguishes_valid_empty_from_malformed() {
        let mut health = Vec::new();
        let loaded = parse_or_log_with_health(
            "non-empty",
            Ok(vec![1, 2]),
            |v: &Vec<u8>| v.len(),
            &mut health,
        );
        assert_eq!(loaded, vec![1, 2]);
        let empty = parse_or_log_with_health(
            "empty",
            Ok::<Vec<u8>, serde_json::Error>(vec![]),
            |v| v.len(),
            &mut health,
        );
        assert!(empty.is_empty());
        let malformed = parse_or_log_with_health(
            "broken",
            serde_json::from_str::<Vec<u8>>("{"),
            |v| v.len(),
            &mut health,
        );
        assert!(malformed.is_empty());
        assert_eq!(health[0].state, DatasetLoadState::Loaded);
        assert_eq!(health[0].record_count, Some(2));
        assert_eq!(health[1].state, DatasetLoadState::ValidEmpty);
        assert_eq!(health[1].record_count, Some(0));
        assert_eq!(health[2].state, DatasetLoadState::Malformed);
        assert_eq!(health[2].record_count, None);
        assert!(health[2].diagnostic.as_deref().is_some_and(|d| !d.is_empty()));
    }

    #[test]
    fn load_all_with_health_retains_dataset_failures() {
        let report = load_all_with_health(
            "{",
            r#"{"geothermal_nodes":[],"maglev_corridors":[]}"#,
            "[]", "[]", "[]", "[]", "[]", "{}",
            "[]", "[]",
            "{", // malformed GeoJSON must be visible as malformed too
            r#"{"type":"FeatureCollection","features":[]}"#,
            r#"{"type":"FeatureCollection","features":[]}"#,
            r#"{"type":"FeatureCollection","features":[]}"#,
            "[]", "[]", "[]",
        );
        assert_eq!(report.health.len(), 17);
        assert!(report.data.sites.is_empty());
        let sites = report.health.iter().find(|h| h.dataset == "sites-clustered").unwrap();
        assert_eq!(sites.state, DatasetLoadState::Malformed);
        assert_eq!(sites.record_count, None);
        let earthquakes = report.health.iter().find(|h| h.dataset == "usgs-earthquakes").unwrap();
        assert_eq!(earthquakes.state, DatasetLoadState::Malformed);
        assert_eq!(earthquakes.record_count, None);
        let vaults = report.health.iter().find(|h| h.dataset == "resontia-vaults").unwrap();
        assert_eq!(vaults.state, DatasetLoadState::ValidEmpty);
        assert_eq!(vaults.record_count, Some(0));
        assert!(report.health.iter().all(|h| !h.dataset.is_empty()));
    }

    #[test]
    fn dataset_health_json_preserves_malformed_vs_valid_empty() {
        let malformed = DatasetLoadHealth {
            dataset: "sites-clustered".to_string(),
            state: DatasetLoadState::Malformed,
            record_count: None,
            diagnostic: Some("invalid JSON".to_string()),
        };
        let empty = DatasetLoadHealth {
            dataset: "resontia-vaults".to_string(),
            state: DatasetLoadState::ValidEmpty,
            record_count: Some(0),
            diagnostic: None,
        };

        let malformed_json = serde_json::to_value(malformed).unwrap();
        let empty_json = serde_json::to_value(empty).unwrap();
        assert_eq!(malformed_json["state"], "malformed");
        assert!(malformed_json["record_count"].is_null());
        assert_eq!(empty_json["state"], "valid_empty");
        assert_eq!(empty_json["record_count"], 0);
    }

    #[test]
    fn parse_geojson_earthquakes() {
        let json = r#"{
            "type": "FeatureCollection",
            "features": [
                {
                    "type": "Feature",
                    "properties": {"magnitude": 5.1, "place": "Peru", "type": "earthquake"},
                    "geometry": {"type": "Point", "coordinates": [-80.5, -5.2, 10]}
                },
                {
                    "type": "Feature",
                    "properties": {"magnitude": 3.2, "place": "Japan"},
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
    fn parse_geojson_fires() {
        let json = r#"{
            "type": "FeatureCollection",
            "features": [
                {
                    "type": "Feature",
                    "properties": {"brightness": 387.0, "confidence": 94, "type": "fire"},
                    "geometry": {"type": "Point", "coordinates": [25.0, -30.0]}
                }
            ]
        }"#;
        let events = parse_natural_events("[]", json, "[]", "[]");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event_type, NaturalEventType::Fire);
        assert!((events[0].magnitude - 3.87).abs() < 0.01); // brightness/100
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
        assert_eq!(cities[0].population, 13960000);
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
        assert!(
            all.len() >= 18,
            "Expected at least 18 layers, got {}",
            all.len()
        );
        assert!(all.contains(&Layer::Infrastructure));
        assert!(all.contains(&Layer::Chokepoints));
    }

    #[test]
    fn all_layers_have_labels() {
        for layer in Layer::all() {
            assert!(
                !layer.label().is_empty(),
                "Layer {:?} has empty label",
                layer
            );
            assert!(
                !layer.css_color().is_empty(),
                "Layer {:?} has empty css_color",
                layer
            );
            let rgb = layer.rgb();
            for c in rgb {
                assert!(
                    c >= 0.0 && c <= 1.0,
                    "Layer {:?} RGB out of range: {:?}",
                    layer,
                    rgb
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
        // Test with minimal valid data for each type
        let data = load_all(
            "[]",                                                                         // sites
            r#"{"geothermal_nodes":[],"maglev_corridors":[]}"#,                           // maglev
            "[]",                                                                         // vaults
            "[]", // terra lumina
            "[]", // regions
            "[]", // supply routes
            "[]", // climate
            r#"{"emergency_shelters":[],"health_facilities":[],"robotics_dispatch":[]}"#, // infra
            "[]", // fossil
            "[]", // nuclear
            "[]", // earthquakes
            "[]", // fires
            "[]", // storms
            "[]", // volcanoes
            "[]", // cities
            "[]", // chokepoints
            "[]", // critical infra
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
        // Should not panic, just return empty vecs
        assert!(data.sites.is_empty());
    }
}
