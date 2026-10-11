// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Geographic proximity context between a hazard observation and mapped
//! infrastructure. This is not a damage, exposure, causality, or risk model.

use crate::geo::haversine_km;
use crate::types::{CriticalInfrastructure, NaturalEvent};

#[derive(Debug, Clone)]
pub struct NearbyInfrastructure<'a> {
    pub asset: &'a CriticalInfrastructure,
    pub distance_km: f64,
}

/// Return mapped infrastructure within a caller-selected radius, nearest first.
///
/// Invalid coordinates, a non-finite/non-positive radius, or a zero result
/// limit fail closed with an empty result. Invalid asset coordinates are
/// ignored. Distance is spherical great-circle distance only; callers must
/// not interpret proximity as evidence that an event affected an asset.
pub fn nearby_infrastructure<'a>(
    event: &NaturalEvent,
    infrastructure: &'a [CriticalInfrastructure],
    radius_km: f64,
    limit: usize,
) -> Vec<NearbyInfrastructure<'a>> {
    if !valid_coordinate(event.lat, event.lon)
        || !radius_km.is_finite()
        || radius_km <= 0.0
        || limit == 0
    {
        return Vec::new();
    }

    let mut nearby: Vec<_> = infrastructure
        .iter()
        .filter(|asset| valid_coordinate(asset.lat, asset.lon))
        .filter_map(|asset| {
            let distance_km = haversine_km(event.lat, event.lon, asset.lat, asset.lon);
            (distance_km.is_finite() && distance_km <= radius_km).then_some(
                NearbyInfrastructure {
                    asset,
                    distance_km,
                },
            )
        })
        .collect();

    nearby.sort_by(|left, right| {
        left.distance_km
            .total_cmp(&right.distance_km)
            .then_with(|| left.asset.name.cmp(&right.asset.name))
            .then_with(|| left.asset.infra_type.cmp(&right.asset.infra_type))
    });
    nearby.truncate(limit);
    nearby
}

fn valid_coordinate(latitude: f64, longitude: f64) -> bool {
    latitude.is_finite()
        && longitude.is_finite()
        && (-90.0..=90.0).contains(&latitude)
        && (-180.0..=180.0).contains(&longitude)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::NaturalEventType;

    fn event(latitude: f64, longitude: f64) -> NaturalEvent {
        NaturalEvent {
            lat: latitude,
            lon: longitude,
            event_type: NaturalEventType::Earthquake,
            magnitude: 4.0,
            name: "Test event".to_string(),
            source_name: None,
            source_id: None,
            source_url: None,
            occurred_at_ms: None,
            updated_at_ms: None,
            depth_km: None,
            geometry_note: None,
        }
    }

    fn asset(name: &str, latitude: f64, longitude: f64) -> CriticalInfrastructure {
        CriticalInfrastructure {
            name: name.to_string(),
            lat: latitude,
            lon: longitude,
            infra_type: "test asset".to_string(),
            global_share: 0.0,
            risk: "not assessed".to_string(),
        }
    }

    #[test]
    fn returns_nearest_assets_first_and_respects_radius_and_limit() {
        let infrastructure = vec![
            asset("far", 3.0, 0.0),
            asset("co-located", 0.0, 0.0),
            asset("near", 1.0, 0.0),
            asset("outside", 10.0, 0.0),
        ];

        let nearby = nearby_infrastructure(&event(0.0, 0.0), &infrastructure, 250.0, 2);

        assert_eq!(nearby.len(), 2);
        assert_eq!(nearby[0].asset.name, "co-located");
        assert_eq!(nearby[0].distance_km, 0.0);
        assert_eq!(nearby[1].asset.name, "near");
        assert!((nearby[1].distance_km - 111.2).abs() < 1.0);
    }

    #[test]
    fn handles_the_antimeridian_with_great_circle_distance() {
        let infrastructure = vec![asset("across date line", 0.0, -179.5)];

        let nearby = nearby_infrastructure(&event(0.0, 179.5), &infrastructure, 120.0, 4);

        assert_eq!(nearby.len(), 1);
        assert!(nearby[0].distance_km < 120.0);
        assert!(nearby[0].distance_km > 100.0);
    }

    #[test]
    fn fails_closed_for_invalid_radius_or_coordinates() {
        let infrastructure = vec![asset("valid", 0.0, 0.0)];

        assert!(nearby_infrastructure(&event(0.0, 0.0), &infrastructure, 0.0, 4).is_empty());
        assert!(nearby_infrastructure(&event(0.0, 0.0), &infrastructure, f64::NAN, 4).is_empty());
        assert!(nearby_infrastructure(&event(0.0, 0.0), &infrastructure, 10.0, 0).is_empty());
        assert!(nearby_infrastructure(&event(91.0, 0.0), &infrastructure, 10.0, 4).is_empty());
    }

    #[test]
    fn skips_infrastructure_with_invalid_coordinates() {
        let infrastructure = vec![
            asset("invalid latitude", 91.0, 0.0),
            asset("invalid longitude", 0.0, f64::INFINITY),
        ];

        assert!(nearby_infrastructure(&event(0.0, 0.0), &infrastructure, 100.0, 4).is_empty());
    }
}
