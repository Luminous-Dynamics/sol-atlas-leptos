// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! JSON data loading for Sol Atlas datasets.
//!
//! Consumers provide JSON strings (from `include_str!`, file reads, or network).
//! This module handles deserialization into sol-atlas-core types.

use crate::types::*;
use serde::Deserialize;

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

/// Load all datasets from their respective JSON strings into a single `LoadedData`.
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

/// Parse simplified shipping lane routes (Vec of Vec of [lon, lat]).
pub fn parse_shipping_lanes(json: &str) -> Vec<Vec<[f64; 2]>> {
    parse_or_log("shipping-lanes", serde_json::from_str(json))
}

/// Parse GeoJSON natural events from USGS, NASA EONET, FIRMS, and volcano data.
fn valid_source_event_url(url: &str, prefix: &str, source_id: Option<&str>) -> bool {
    let Some(source_id) = source_id else {
        return false;
    };
    let Some(path_id) = url.strip_prefix(prefix) else {
        return false;
    };

    !source_id.is_empty()
        && path_id == source_id
        && url.len() <= 2_048
        && !url.chars().any(char::is_control)
        && !url.chars().any(char::is_whitespace)
}

fn parse_natural_events(
    earthquakes: &str,
    fires: &str,
    storms: &str,
    volcanoes: &str,
) -> Vec<NaturalEvent> {
    let mut events = Vec::new();

    // Parse GeoJSON FeatureCollections (all share same structure)
    fn parse_geojson(name: &str, json: &str, event_type: NaturalEventType) -> Vec<NaturalEvent> {
        #[derive(Deserialize)]
        struct FeatureCollection {
            features: Vec<Feature>,
        }
        #[derive(Deserialize)]
        struct Feature {
            #[serde(default)]
            id: Option<String>,
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
            #[serde(default)]
            time: serde_json::Value,
            #[serde(default)]
            updated: serde_json::Value,
            #[serde(default)]
            url: Option<String>,
            #[serde(default)]
            link: Option<String>,
        }
        #[derive(Deserialize)]
        struct Geometry {
            coordinates: Vec<f64>,
        }

        let fc = match serde_json::from_str::<FeatureCollection>(json) {
            Ok(fc) => fc,
            Err(e) => {
                log::warn!("sol-atlas-core: failed to parse dataset '{name}': {e}");
                return vec![];
            }
        };
        fc.features
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
                    source_name: match name {
                        "usgs-earthquakes" => Some("USGS ComCat".to_string()),
                        "nasa-firms" => Some("NASA FIRMS".to_string()),
                        "nasa-eonet" => Some("NASA EONET".to_string()),
                        "volcanoes" => Some("Curated volcano inventory".to_string()),
                        _ => None,
                    },
                    source_id: match name {
                        "usgs-earthquakes" | "nasa-eonet" => f.id.clone(),
                        _ => None,
                    },
                    source_url: match name {
                        "usgs-earthquakes" => f.properties.url.as_deref().filter(|url| {
                            valid_source_event_url(
                                url,
                                crate::usgs::USGS_EVENT_PAGE_PREFIX,
                                f.id.as_deref(),
                            )
                        }),
                        "nasa-eonet" => f.properties.link.as_deref().filter(|url| {
                            valid_source_event_url(
                                url,
                                crate::eonet::EONET_EVENT_API_PREFIX,
                                f.id.as_deref(),
                            )
                        }),
                        _ => None,
                    }.map(str::to_string),
                    occurred_at_ms: f.properties.time.as_i64(),
                    updated_at_ms: f.properties.updated.as_i64(),
                    depth_km: coords.get(2).copied().filter(|depth| depth.is_finite()),
                })
            })
            .collect()
    }

    events.extend(parse_geojson(
        "usgs-earthquakes",
        earthquakes,
        NaturalEventType::Earthquake,
    ));
    events.extend(parse_geojson("nasa-firms", fires, NaturalEventType::Fire));
    events.extend(parse_geojson("nasa-eonet", storms, NaturalEventType::Storm));
    events.extend(parse_geojson(
        "volcanoes",
        volcanoes,
        NaturalEventType::Volcano,
    ));
    events
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


#[cfg(test)]
mod live_timestamp_compat_tests {
    use super::*;

    #[test]
    fn bundled_geojson_with_iso_event_time_still_loads() {
        let earthquake = r#"{
            "type": "FeatureCollection",
            "features": [{
                "type": "Feature",
                "properties": {
                    "magnitude": 5.0,
                    "place": "Test location",
                    "time": "2025-09-16T01:05:59.070000"
                },
                "geometry": {"type": "Point", "coordinates": [1.0, 2.0, 3.0]}
            }]
        }"#;
        let events = parse_natural_events(earthquake, "[]", "[]", "[]");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].name, "Test location");
        assert_eq!(events[0].source_name.as_deref(), Some("USGS ComCat"));
        assert_eq!(events[0].occurred_at_ms, None);
        assert_eq!(events[0].depth_km, Some(3.0));
    }
}
