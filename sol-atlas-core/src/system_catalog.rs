// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! A renderer-neutral solar-system catalogue and ephemeris state contract.
//!
//! This catalogue is metadata, not an orbital propagator. Its rounded physical
//! values are suitable for labels and rough display sizing only. Positions for
//! a requested epoch must come from a provenance-carrying state vector (for
//! example, JPL Horizons), not from the legacy visual-only orbit animation in
//! `solar_system.rs`.

use serde::{Deserialize, Serialize};

/// High-level object type. Population entries describe queryable layers, not
/// a single physical body and must not be sent to a point-object ephemeris API.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ObjectKind {
    Star,
    Planet,
    DwarfPlanet,
    NaturalSatellite,
    SmallBodyPopulation,
    SpacecraftPopulation,
}

/// Stable, lower-snake-case identifier within Sol Atlas.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CatalogObject {
    pub id: &'static str,
    pub name: &'static str,
    pub kind: ObjectKind,
    /// Parent body's stable catalogue ID. For population layers this is the
    /// orbital context, not a claim that the population is one orbiting object.
    pub parent_id: Option<&'static str>,
    /// Rounded mean/characteristic physical radius. None means not applicable
    /// or intentionally not provided; do not treat it as zero.
    pub mean_radius_km: Option<f64>,
    /// Approximate heliocentric semi-major axis for Sun-orbiting bodies.
    /// None for satellites and population layers.
    pub heliocentric_semi_major_axis_au: Option<f64>,
    /// Approximate semi-major axis relative to `parent_id`, for satellites.
    /// None for heliocentric bodies and aggregate population layers.
    pub parent_orbit_semi_major_axis_km: Option<f64>,
    /// Name/alias to resolve at the external ephemeris provider. This is
    /// deliberately absent for aggregate populations such as "all comets".
    pub ephemeris_target: Option<&'static str>,
    /// Existing local texture path, if one is known to be available.
    /// Missing artwork must fall back to procedural/material rendering.
    pub texture_asset: Option<&'static str>,
}

macro_rules! body {
    ($id:literal, $name:literal, $kind:ident, $parent:expr, $radius:expr, $axis_au:expr, $parent_axis:expr, $target:expr, $texture:expr) => {
        CatalogObject {
            id: $id,
            name: $name,
            kind: ObjectKind::$kind,
            parent_id: $parent,
            mean_radius_km: $radius,
            heliocentric_semi_major_axis_au: $axis_au,
            parent_orbit_semi_major_axis_km: $parent_axis,
            ephemeris_target: $target,
            texture_asset: $texture,
        }
    };
}

/// Curated first-class objects and aggregate layers.
///
/// Distances and radii are rounded reference metadata, not authoritative
/// ephemerides. The catalogue intentionally lists representative major moons,
/// not every known moon, asteroid, comet, or spacecraft. Population layers are
/// extensible query domains and can be backed by JPL SBDB, Horizons, or another
/// explicitly attributed source by a future data adapter.
pub const SOLAR_SYSTEM_CATALOG: &[CatalogObject] = &[
    body!("sun", "Sun", Star, None, Some(695_700.0), None, None, Some("Sun"), Some("/assets/globe-textures/sun.jpg")),

    body!("mercury", "Mercury", Planet, Some("sun"), Some(2_439.7), Some(0.3871), None, Some("Mercury"), None),
    body!("venus", "Venus", Planet, Some("sun"), Some(6_051.8), Some(0.7233), None, Some("Venus"), Some("/assets/globe-textures/venus.jpg")),
    body!("earth", "Earth", Planet, Some("sun"), Some(6_371.0), Some(1.0000), None, Some("Earth"), Some("/assets/globe-textures/earth-blue-marble.jpg")),
    body!("mars", "Mars", Planet, Some("sun"), Some(3_389.5), Some(1.5237), None, Some("Mars"), Some("/assets/globe-textures/mars.jpg")),
    body!("jupiter", "Jupiter", Planet, Some("sun"), Some(69_911.0), Some(5.2028), None, Some("Jupiter"), Some("/assets/globe-textures/jupiter.jpg")),
    body!("saturn", "Saturn", Planet, Some("sun"), Some(58_232.0), Some(9.5388), None, Some("Saturn"), Some("/assets/globe-textures/saturn.jpg")),
    body!("uranus", "Uranus", Planet, Some("sun"), Some(25_362.0), Some(19.1914), None, Some("Uranus"), None),
    body!("neptune", "Neptune", Planet, Some("sun"), Some(24_622.0), Some(30.0611), None, Some("Neptune"), None),

    body!("ceres", "Ceres", DwarfPlanet, Some("sun"), Some(473.0), Some(2.7675), None, Some("Ceres"), None),
    body!("pluto", "Pluto", DwarfPlanet, Some("sun"), Some(1_188.3), Some(39.482), None, Some("Pluto"), None),
    body!("haumea", "Haumea", DwarfPlanet, Some("sun"), Some(816.0), Some(43.13), None, Some("Haumea"), None),
    body!("makemake", "Makemake", DwarfPlanet, Some("sun"), Some(715.0), Some(45.79), None, Some("Makemake"), None),
    body!("eris", "Eris", DwarfPlanet, Some("sun"), Some(1_163.0), Some(67.78), None, Some("Eris"), None),

    // Representative natural satellites. Satellite orbit distances are
    // parent-centered; they are never interpreted as heliocentric AU values.
    body!("moon", "Moon", NaturalSatellite, Some("earth"), Some(1_737.4), None, Some(384_400.0), Some("Moon"), Some("/assets/globe-textures/moon.jpg")),
    body!("phobos", "Phobos", NaturalSatellite, Some("mars"), Some(11.3), None, Some(9_376.0), Some("Phobos"), None),
    body!("deimos", "Deimos", NaturalSatellite, Some("mars"), Some(6.2), None, Some(23_463.0), Some("Deimos"), None),

    body!("io", "Io", NaturalSatellite, Some("jupiter"), Some(1_821.6), None, Some(421_700.0), Some("Io"), None),
    body!("europa", "Europa", NaturalSatellite, Some("jupiter"), Some(1_560.8), None, Some(671_100.0), Some("Europa"), None),
    body!("ganymede", "Ganymede", NaturalSatellite, Some("jupiter"), Some(2_634.1), None, Some(1_070_400.0), Some("Ganymede"), None),
    body!("callisto", "Callisto", NaturalSatellite, Some("jupiter"), Some(2_410.3), None, Some(1_882_700.0), Some("Callisto"), None),

    body!("mimas", "Mimas", NaturalSatellite, Some("saturn"), Some(198.2), None, Some(185_539.0), Some("Mimas"), None),
    body!("enceladus", "Enceladus", NaturalSatellite, Some("saturn"), Some(252.1), None, Some(237_948.0), Some("Enceladus"), None),
    body!("tethys", "Tethys", NaturalSatellite, Some("saturn"), Some(531.1), None, Some(294_619.0), Some("Tethys"), None),
    body!("dione", "Dione", NaturalSatellite, Some("saturn"), Some(561.4), None, Some(377_396.0), Some("Dione"), None),
    body!("rhea", "Rhea", NaturalSatellite, Some("saturn"), Some(763.8), None, Some(527_108.0), Some("Rhea"), None),
    body!("titan", "Titan", NaturalSatellite, Some("saturn"), Some(2_574.7), None, Some(1_221_870.0), Some("Titan"), None),
    body!("iapetus", "Iapetus", NaturalSatellite, Some("saturn"), Some(734.5), None, Some(3_560_820.0), Some("Iapetus"), None),

    body!("miranda", "Miranda", NaturalSatellite, Some("uranus"), Some(235.8), None, Some(129_900.0), Some("Miranda"), None),
    body!("ariel", "Ariel", NaturalSatellite, Some("uranus"), Some(578.9), None, Some(190_900.0), Some("Ariel"), None),
    body!("umbriel", "Umbriel", NaturalSatellite, Some("uranus"), Some(584.7), None, Some(266_000.0), Some("Umbriel"), None),
    body!("titania", "Titania", NaturalSatellite, Some("uranus"), Some(788.9), None, Some(435_900.0), Some("Titania"), None),
    body!("oberon", "Oberon", NaturalSatellite, Some("uranus"), Some(761.4), None, Some(583_500.0), Some("Oberon"), None),

    body!("triton", "Triton", NaturalSatellite, Some("neptune"), Some(1_353.4), None, Some(354_759.0), Some("Triton"), None),
    body!("charon", "Charon", NaturalSatellite, Some("pluto"), Some(606.0), None, Some(19_596.0), Some("Charon"), None),

    // Aggregate layers: never substitute a single representative object for
    // these collections. Their members and positions require separate data.
    body!("asteroid_belt", "Main asteroid belt", SmallBodyPopulation, Some("sun"), None, None, None, None, None),
    body!("near_earth_objects", "Near-Earth objects", SmallBodyPopulation, Some("sun"), None, None, None, None, None),
    body!("comets", "Comets", SmallBodyPopulation, Some("sun"), None, None, None, None, None),
    body!("centaurs", "Centaurs", SmallBodyPopulation, Some("sun"), None, None, None, None, None),
    body!("kuiper_belt", "Kuiper belt", SmallBodyPopulation, Some("sun"), None, None, None, None, None),
    body!("scattered_disc", "Scattered disc", SmallBodyPopulation, Some("sun"), None, None, None, None, None),
    body!("oort_cloud", "Oort cloud", SmallBodyPopulation, Some("sun"), None, None, None, None, None),
    body!("spacecraft", "Spacecraft and missions", SpacecraftPopulation, None, None, None, None, None, None),
];

/// Find a catalogue entry by its stable ID.
pub fn catalog_object(id: &str) -> Option<&'static CatalogObject> {
    SOLAR_SYSTEM_CATALOG.iter().find(|object| object.id == id)
}

/// Iterate catalogue entries by kind, including aggregate layers when requested.
pub fn objects_of_kind(kind: ObjectKind) -> impl Iterator<Item = &'static CatalogObject> {
    SOLAR_SYSTEM_CATALOG.iter().filter(move |object| object.kind == kind)
}

/// Time scale explicitly attached to an ephemeris epoch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimeScale {
    Utc,
    Tt,
    Tdb,
}

/// Reference system for a Cartesian state vector. The vector centre is carried
/// separately: ICRF alone does not imply whether a vector is heliocentric,
/// geocentric, or relative to another body.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReferenceSystem {
    Icrf,
    B1950,
}

/// Hash-bound provenance for the exact query and response that produced a
/// state. Hash shape is validated here; the ingestion adapter must compute the
/// hashes from the canonical query bytes and unmodified response bytes.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EphemerisProvenance {
    pub provider: String,
    pub canonical_query_sha256: String,
    pub raw_response_sha256: String,
    /// Retrieval instant as RFC 3339 UTC text (ends in Z).
    pub retrieved_at_utc: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StateVector {
    pub target_id: String,
    pub center_id: String,
    /// Julian date in `time_scale`, not an unlabelled Unix timestamp.
    pub epoch_jd: f64,
    pub time_scale: TimeScale,
    pub reference_system: ReferenceSystem,
    /// Cartesian position in km, relative to `center_id`.
    pub position_km: [f64; 3],
    /// Cartesian velocity in km/s, relative to `center_id`.
    pub velocity_km_s: [f64; 3],
    pub provenance: EphemerisProvenance,
}

impl StateVector {
    /// Fail closed on missing identity/provenance, malformed hashes, or
    /// non-finite state values before the renderer consumes the sample.
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.target_id.trim().is_empty() || self.center_id.trim().is_empty() {
            return Err("target_id and center_id are required");
        }
        if !self.epoch_jd.is_finite() {
            return Err("epoch_jd must be finite");
        }
        if self.position_km.iter().chain(self.velocity_km_s.iter()).any(|v| !v.is_finite()) {
            return Err("position and velocity components must be finite");
        }
        if self.provenance.provider.trim().is_empty() {
            return Err("ephemeris provider is required");
        }
        if !is_sha256(&self.provenance.canonical_query_sha256) {
            return Err("canonical_query_sha256 must be 64 hexadecimal characters");
        }
        if !is_sha256(&self.provenance.raw_response_sha256) {
            return Err("raw_response_sha256 must be 64 hexadecimal characters");
        }
        if !is_rfc3339_utc(&self.provenance.retrieved_at_utc) {
            return Err("retrieved_at_utc must be RFC 3339 UTC text ending in Z");
        }
        Ok(())
    }
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

/// Check the structural RFC 3339 UTC shape without adding a date-time
/// dependency. The ingestion adapter should use a full parser when available.
fn is_rfc3339_utc(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() < 20
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || bytes[10] != b'T'
        || bytes[13] != b':'
        || bytes[16] != b':'
        || bytes.last() != Some(&b'Z')
    {
        return false;
    }

    for index in [0, 1, 2, 3, 5, 6, 8, 9, 11, 12, 14, 15, 17, 18] {
        if !bytes[index].is_ascii_digit() {
            return false;
        }
    }

    if bytes.len() == 20 {
        return true;
    }

    bytes[19] == b'.'
        && bytes[20..bytes.len() - 1]
            .iter()
            .all(|byte| byte.is_ascii_digit())
        && bytes.len() > 21
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalogue_contains_all_eight_planets() {
        assert_eq!(objects_of_kind(ObjectKind::Planet).count(), 8);
        for id in ["mercury", "venus", "earth", "mars", "jupiter", "saturn", "uranus", "neptune"] {
            assert_eq!(catalog_object(id).unwrap().kind, ObjectKind::Planet);
        }
    }

    #[test]
    fn catalogue_contains_five_recognized_dwarf_planets() {
        assert_eq!(objects_of_kind(ObjectKind::DwarfPlanet).count(), 5);
        for id in ["ceres", "pluto", "haumea", "makemake", "eris"] {
            assert_eq!(catalog_object(id).unwrap().kind, ObjectKind::DwarfPlanet);
        }
    }

    #[test]
    fn each_parent_reference_resolves() {
        for object in SOLAR_SYSTEM_CATALOG {
            if let Some(parent_id) = object.parent_id {
                assert!(catalog_object(parent_id).is_some(), "{} has unknown parent {parent_id}", object.id);
            }
        }
    }

    #[test]
    fn catalogue_ids_are_unique_and_orbit_fields_match_object_kind() {
        use std::collections::HashSet;

        let mut ids = HashSet::new();
        for object in SOLAR_SYSTEM_CATALOG {
            assert!(ids.insert(object.id), "duplicate catalogue ID: {}", object.id);

            match object.kind {
                ObjectKind::Star => {
                    assert_eq!(object.id, "sun");
                    assert!(object.parent_id.is_none());
                    assert!(object.heliocentric_semi_major_axis_au.is_none());
                    assert!(object.parent_orbit_semi_major_axis_km.is_none());
                }
                ObjectKind::Planet | ObjectKind::DwarfPlanet => {
                    assert_eq!(object.parent_id, Some("sun"), "{}", object.id);
                    assert!(matches!(object.heliocentric_semi_major_axis_au, Some(axis) if axis.is_finite() && axis > 0.0), "{}", object.id);
                    assert!(object.parent_orbit_semi_major_axis_km.is_none(), "{}", object.id);
                }
                ObjectKind::NaturalSatellite => {
                    assert!(object.parent_id.is_some(), "{}", object.id);
                    assert!(matches!(object.parent_orbit_semi_major_axis_km, Some(axis) if axis.is_finite() && axis > 0.0), "{}", object.id);
                    assert!(object.heliocentric_semi_major_axis_au.is_none(), "{}", object.id);
                }
                ObjectKind::SmallBodyPopulation | ObjectKind::SpacecraftPopulation => {
                    assert!(object.mean_radius_km.is_none(), "{}", object.id);
                    assert!(object.ephemeris_target.is_none(), "{}", object.id);
                    assert!(object.heliocentric_semi_major_axis_au.is_none(), "{}", object.id);
                    assert!(object.parent_orbit_semi_major_axis_km.is_none(), "{}", object.id);
                }
            }
        }
    }

    #[test]
    fn aggregate_layers_are_not_mistaken_for_ephemeris_targets() {
        for object in objects_of_kind(ObjectKind::SmallBodyPopulation)
            .chain(objects_of_kind(ObjectKind::SpacecraftPopulation))
        {
            assert!(object.ephemeris_target.is_none(), "{} is an aggregate layer", object.id);
            assert!(object.mean_radius_km.is_none(), "{} is not one physical body", object.id);
        }
    }

    #[test]
    fn satellite_orbits_are_parent_centered_not_heliocentric() {
        let moon = catalog_object("moon").unwrap();
        assert_eq!(moon.parent_id, Some("earth"));
        assert_eq!(moon.heliocentric_semi_major_axis_au, None);
        assert_eq!(moon.parent_orbit_semi_major_axis_km, Some(384_400.0));
        assert_eq!(catalog_object("triton").unwrap().parent_id, Some("neptune"));
    }

    fn valid_sample() -> StateVector {
        StateVector {
            target_id: "mars".into(),
            center_id: "sun".into(),
            epoch_jd: 2_460_000.5,
            time_scale: TimeScale::Tdb,
            reference_system: ReferenceSystem::Icrf,
            position_km: [1.0, 2.0, 3.0],
            velocity_km_s: [4.0, 5.0, 6.0],
            provenance: EphemerisProvenance {
                provider: "JPL Horizons".into(),
                canonical_query_sha256: "a".repeat(64),
                raw_response_sha256: "b".repeat(64),
                retrieved_at_utc: "2026-10-10T00:00:00Z".into(),
            },
        }
    }

    #[test]
    fn state_vector_requires_explicit_units_time_frame_and_provenance() {
        assert!(valid_sample().validate().is_ok());

        let mut invalid = valid_sample();
        invalid.position_km[1] = f64::NAN;
        assert_eq!(invalid.validate(), Err("position and velocity components must be finite"));

        let mut invalid = valid_sample();
        invalid.provenance.raw_response_sha256 = "not-a-hash".into();
        assert_eq!(invalid.validate(), Err("raw_response_sha256 must be 64 hexadecimal characters"));

        let mut invalid = valid_sample();
        invalid.center_id.clear();
        assert_eq!(invalid.validate(), Err("target_id and center_id are required"));

        let mut invalid = valid_sample();
        invalid.provenance.retrieved_at_utc = "yesterday".into();
        assert_eq!(invalid.validate(), Err("retrieved_at_utc must be RFC 3339 UTC text ending in Z"));
    }
}
