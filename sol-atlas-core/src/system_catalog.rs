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
use sha2::{Digest, Sha256};

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
    /// Explicit Horizons COMMAND expression, using numeric major-body IDs
    /// or numbered-small-body syntax where applicable. Aggregate populations
    /// intentionally have no single ephemeris target.
    pub ephemeris_target: Option<&'static str>,
    /// Existing local texture path, if one is known to be available.
    /// Missing artwork must fall back to procedural/material rendering.
    pub texture_asset: Option<&'static str>,
}

macro_rules! body {
    (
        $id:literal,
        $name:literal,
        $kind:ident,
        $parent:expr,
        $radius:expr,
        $axis_au:expr,
        $parent_axis:expr,
        $target:expr,
        $texture:expr $(,)?
    ) => {
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
    body!(
        "sun",
        "Sun",
        Star,
        None,
        Some(695_700.0),
        None,
        None,
        Some("10"),
        Some("/assets/globe-textures/sun.jpg")
    ),

    body!(
        "mercury",
        "Mercury",
        Planet,
        Some("sun"),
        Some(2_439.7),
        Some(0.3871),
        None,
        Some("199"),
        None
    ),
    body!(
        "venus",
        "Venus",
        Planet,
        Some("sun"),
        Some(6_051.8),
        Some(0.7233),
        None,
        Some("299"),
        Some("/assets/globe-textures/venus.jpg")
    ),
    body!(
        "earth",
        "Earth",
        Planet,
        Some("sun"),
        Some(6_371.0),
        Some(1.0000),
        None,
        Some("399"),
        Some("/assets/globe-textures/earth-blue-marble.jpg")
    ),
    body!(
        "mars",
        "Mars",
        Planet,
        Some("sun"),
        Some(3_389.5),
        Some(1.5237),
        None,
        Some("499"),
        Some("/assets/globe-textures/mars.jpg")
    ),
    body!(
        "jupiter",
        "Jupiter",
        Planet,
        Some("sun"),
        Some(69_911.0),
        Some(5.2028),
        None,
        Some("599"),
        Some("/assets/globe-textures/jupiter.jpg")
    ),
    body!(
        "saturn",
        "Saturn",
        Planet,
        Some("sun"),
        Some(58_232.0),
        Some(9.5388),
        None,
        Some("699"),
        Some("/assets/globe-textures/saturn.jpg")
    ),
    body!(
        "uranus",
        "Uranus",
        Planet,
        Some("sun"),
        Some(25_362.0),
        Some(19.1914),
        None,
        Some("799"),
        None
    ),
    body!(
        "neptune",
        "Neptune",
        Planet,
        Some("sun"),
        Some(24_622.0),
        Some(30.0611),
        None,
        Some("899"),
        None
    ),

    body!(
        "ceres",
        "Ceres",
        DwarfPlanet,
        Some("sun"),
        Some(473.0),
        Some(2.7675),
        None,
        Some("1;"),
        None
    ),
    body!(
        "pluto",
        "Pluto",
        DwarfPlanet,
        Some("sun"),
        Some(1_188.3),
        Some(39.482),
        None,
        Some("999"),
        None
    ),
    body!(
        "haumea",
        "Haumea",
        DwarfPlanet,
        Some("sun"),
        Some(816.0),
        Some(43.13),
        None,
        Some("136108;"),
        None
    ),
    body!(
        "makemake",
        "Makemake",
        DwarfPlanet,
        Some("sun"),
        Some(715.0),
        Some(45.79),
        None,
        Some("136472;"),
        None
    ),
    body!(
        "eris",
        "Eris",
        DwarfPlanet,
        Some("sun"),
        Some(1_163.0),
        Some(67.78),
        None,
        Some("136199;"),
        None
    ),

    // Representative natural satellites. Satellite orbit distances are
    // parent-centered; they are never interpreted as heliocentric AU values.
    body!(
        "moon",
        "Moon",
        NaturalSatellite,
        Some("earth"),
        Some(1_737.4),
        None,
        Some(384_400.0),
        Some("301"),
        Some("/assets/globe-textures/moon.jpg")
    ),
    body!(
        "phobos",
        "Phobos",
        NaturalSatellite,
        Some("mars"),
        Some(11.3),
        None,
        Some(9_376.0),
        Some("401"),
        None
    ),
    body!(
        "deimos",
        "Deimos",
        NaturalSatellite,
        Some("mars"),
        Some(6.2),
        None,
        Some(23_463.0),
        Some("402"),
        None
    ),

    body!(
        "io",
        "Io",
        NaturalSatellite,
        Some("jupiter"),
        Some(1_821.6),
        None,
        Some(421_700.0),
        Some("501"),
        None
    ),
    body!(
        "europa",
        "Europa",
        NaturalSatellite,
        Some("jupiter"),
        Some(1_560.8),
        None,
        Some(671_100.0),
        Some("502"),
        None
    ),
    body!(
        "ganymede",
        "Ganymede",
        NaturalSatellite,
        Some("jupiter"),
        Some(2_634.1),
        None,
        Some(1_070_400.0),
        Some("503"),
        None
    ),
    body!(
        "callisto",
        "Callisto",
        NaturalSatellite,
        Some("jupiter"),
        Some(2_410.3),
        None,
        Some(1_882_700.0),
        Some("504"),
        None
    ),

    body!(
        "mimas",
        "Mimas",
        NaturalSatellite,
        Some("saturn"),
        Some(198.2),
        None,
        Some(185_539.0),
        Some("601"),
        None
    ),
    body!(
        "enceladus",
        "Enceladus",
        NaturalSatellite,
        Some("saturn"),
        Some(252.1),
        None,
        Some(237_948.0),
        Some("602"),
        None
    ),
    body!(
        "tethys",
        "Tethys",
        NaturalSatellite,
        Some("saturn"),
        Some(531.1),
        None,
        Some(294_619.0),
        Some("603"),
        None
    ),
    body!(
        "dione",
        "Dione",
        NaturalSatellite,
        Some("saturn"),
        Some(561.4),
        None,
        Some(377_396.0),
        Some("604"),
        None
    ),
    body!(
        "rhea",
        "Rhea",
        NaturalSatellite,
        Some("saturn"),
        Some(763.8),
        None,
        Some(527_108.0),
        Some("605"),
        None
    ),
    body!(
        "titan",
        "Titan",
        NaturalSatellite,
        Some("saturn"),
        Some(2_574.7),
        None,
        Some(1_221_870.0),
        Some("606"),
        None
    ),
    body!(
        "iapetus",
        "Iapetus",
        NaturalSatellite,
        Some("saturn"),
        Some(734.5),
        None,
        Some(3_560_820.0),
        Some("608"),
        None
    ),

    body!(
        "miranda",
        "Miranda",
        NaturalSatellite,
        Some("uranus"),
        Some(235.8),
        None,
        Some(129_900.0),
        Some("705"),
        None
    ),
    body!(
        "ariel",
        "Ariel",
        NaturalSatellite,
        Some("uranus"),
        Some(578.9),
        None,
        Some(190_900.0),
        Some("701"),
        None
    ),
    body!(
        "umbriel",
        "Umbriel",
        NaturalSatellite,
        Some("uranus"),
        Some(584.7),
        None,
        Some(266_000.0),
        Some("702"),
        None
    ),
    body!(
        "titania",
        "Titania",
        NaturalSatellite,
        Some("uranus"),
        Some(788.9),
        None,
        Some(435_900.0),
        Some("703"),
        None
    ),
    body!(
        "oberon",
        "Oberon",
        NaturalSatellite,
        Some("uranus"),
        Some(761.4),
        None,
        Some(583_500.0),
        Some("704"),
        None
    ),

    body!(
        "triton",
        "Triton",
        NaturalSatellite,
        Some("neptune"),
        Some(1_353.4),
        None,
        Some(354_759.0),
        Some("801"),
        None
    ),
    body!(
        "charon",
        "Charon",
        NaturalSatellite,
        Some("pluto"),
        Some(606.0),
        None,
        Some(19_596.0),
        Some("901"),
        None
    ),

    // Aggregate layers: never substitute a single representative object for
    // these collections. Their members and positions require separate data.
    body!(
        "asteroid_belt",
        "Main asteroid belt",
        SmallBodyPopulation,
        Some("sun"),
        None,
        None,
        None,
        None,
        None
    ),
    body!(
        "near_earth_objects",
        "Near-Earth objects",
        SmallBodyPopulation,
        Some("sun"),
        None,
        None,
        None,
        None,
        None
    ),
    body!(
        "comets",
        "Comets",
        SmallBodyPopulation,
        Some("sun"),
        None,
        None,
        None,
        None,
        None
    ),
    body!(
        "centaurs",
        "Centaurs",
        SmallBodyPopulation,
        Some("sun"),
        None,
        None,
        None,
        None,
        None
    ),
    body!(
        "kuiper_belt",
        "Kuiper belt",
        SmallBodyPopulation,
        Some("sun"),
        None,
        None,
        None,
        None,
        None
    ),
    body!(
        "scattered_disc",
        "Scattered disc",
        SmallBodyPopulation,
        Some("sun"),
        None,
        None,
        None,
        None,
        None
    ),
    body!(
        "oort_cloud",
        "Oort cloud",
        SmallBodyPopulation,
        Some("sun"),
        None,
        None,
        None,
        None,
        None
    ),
    body!(
        "spacecraft",
        "Spacecraft and missions",
        SpacecraftPopulation,
        None,
        None,
        None,
        None,
        None,
        None
    ),
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

/// Orientation plane for the returned Cartesian coordinates. This is distinct
/// from the inertial reference-system label.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReferencePlane {
    Ecliptic,
    Frame,
    BodyEquator,
}

/// Whether the vector is geometric or includes an astrometric correction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum VectorCorrection {
    Geometric,
    LightTime,
    LightTimeAndStellarAberration,
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

impl EphemerisProvenance {
    /// Compute SHA-256 from the exact canonical URL bytes and raw HTTP body bytes.
    /// This establishes byte-level consistency; it does not independently prove
    /// that the provider or transport was authentic.
    pub fn from_bytes(
        provider: impl Into<String>,
        canonical_url: &str,
        raw_response: &[u8],
        retrieved_at_utc: impl Into<String>,
    ) -> Self {
        Self {
            provider: provider.into(),
            canonical_query_sha256: sha256_hex(canonical_url.as_bytes()),
            raw_response_sha256: sha256_hex(raw_response),
            retrieved_at_utc: retrieved_at_utc.into(),
        }
    }

    /// Verify these digests against the exact request and response bytes.
    pub fn verifies_bytes(&self, canonical_url: &str, raw_response: &[u8]) -> bool {
        self.canonical_query_sha256
            .eq_ignore_ascii_case(&sha256_hex(canonical_url.as_bytes()))
            && self
                .raw_response_sha256
                .eq_ignore_ascii_case(&sha256_hex(raw_response))
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StateVector {
    pub target_id: String,
    pub center_id: String,
    /// Julian date in `time_scale`, not an unlabelled Unix timestamp.
    pub epoch_jd: f64,
    pub time_scale: TimeScale,
    pub reference_system: ReferenceSystem,
    pub reference_plane: ReferencePlane,
    pub vector_correction: VectorCorrection,
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

/// Validate the RFC 3339 UTC timestamp subset used for capture provenance,
/// including actual calendar and clock ranges, without adding a date-time crate.
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

    if bytes.len() > 20
        && (bytes[19] != b'.'
            || bytes.len() <= 21
            || !bytes[20..bytes.len() - 1]
                .iter()
                .all(|byte| byte.is_ascii_digit())
    {
        return false;
    }

    let year = match value[0..4].parse::<u32>() {
        Ok(value) if value > 0 => value,
        _ => return false,
    };
    let month = match value[5..7].parse::<u32>() {
        Ok(value @ 1..=12) => value,
        _ => return false,
    };
    let day = match value[8..10].parse::<u32>() {
        Ok(value) => value,
        _ => return false,
    };
    let hour = match value[11..13].parse::<u32>() {
        Ok(value @ 0..=23) => value,
        _ => return false,
    };
    let minute = match value[14..16].parse::<u32>() {
        Ok(value @ 0..=59) => value,
        _ => return false,
    };
    let second = match value[17..19].parse::<u32>() {
        Ok(value @ 0..=59) => value,
        _ => return false,
    };
    let _ = (hour, minute, second);

    let leap_year = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let days_in_month = match month {
        2 if leap_year => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    };
    day >= 1 && day <= days_in_month
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn provenance_hashes_exact_query_and_response_bytes() {
        let provenance = EphemerisProvenance::from_bytes(
            "test provider",
            "abc",
            b"",
            "2026-10-10T16:00:00Z",
        );
        assert_eq!(
            provenance.canonical_query_sha256,
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(
            provenance.raw_response_sha256,
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        assert!(provenance.verifies_bytes("abc", b""));
        assert!(!provenance.verifies_bytes("abd", b""));
    }

    #[test]
    fn catalogue_contains_all_eight_planets() {
        assert_eq!(objects_of_kind(ObjectKind::Planet).count(), 8);
        for id in ["mercury", "venus", "earth", "mars", "jupiter", "saturn", "uranus", "neptune"] {
            assert_eq!(catalog_object(id).unwrap().kind, ObjectKind::Planet);
        }
    }

    #[test]
    fn catalogue_uses_unambiguous_horizons_target_expressions() {
        for (id, target) in [
            ("sun", "10"),
            ("mercury", "199"),
            ("venus", "299"),
            ("earth", "399"),
            ("mars", "499"),
            ("jupiter", "599"),
            ("saturn", "699"),
            ("uranus", "799"),
            ("neptune", "899"),
            ("ceres", "1;"),
            ("pluto", "999"),
            ("haumea", "136108;"),
            ("makemake", "136472;"),
            ("eris", "136199;"),
            ("moon", "301"),
            ("phobos", "401"),
            ("deimos", "402"),
            ("io", "501"),
            ("europa", "502"),
            ("ganymede", "503"),
            ("callisto", "504"),
            ("triton", "801"),
            ("charon", "901"),
        ] {
            assert_eq!(
                catalog_object(id).unwrap().ephemeris_target,
                Some(target),
                "unexpected Horizons target for {id}"
            );
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
                assert!(
                    catalog_object(parent_id).is_some(),
                    "{} has unknown parent {parent_id}",
                    object.id
                );
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
                    assert!(
                        matches!(
                            object.heliocentric_semi_major_axis_au,
                            Some(axis) if axis.is_finite() && axis > 0.0
                        ),
                        "{}",
                        object.id
                    );
                    assert!(object.parent_orbit_semi_major_axis_km.is_none(), "{}", object.id);
                }
                ObjectKind::NaturalSatellite => {
                    assert!(object.parent_id.is_some(), "{}", object.id);
                    assert!(
                        matches!(
                            object.parent_orbit_semi_major_axis_km,
                            Some(axis) if axis.is_finite() && axis > 0.0
                        ),
                        "{}",
                        object.id
                    );
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
            reference_plane: ReferencePlane::Ecliptic,
            vector_correction: VectorCorrection::Geometric,
            position_km: [1.0, 2.0, 3.0],
            velocity_km_s: [4.0, 5.0, 6.0],
            provenance: EphemerisProvenance::from_bytes(
                "JPL Horizons test sample",
                "canonical test query",
                b"raw test response",
                "2026-10-10T00:00:00Z",
            ),
        }
    }

    #[test]
    fn retrieved_at_timestamp_checks_calendar_and_clock_ranges() {
        assert!(is_rfc3339_utc("2024-02-29T23:59:59Z"));
        assert!(is_rfc3339_utc("2026-10-10T18:30:15.123Z"));
        assert!(!is_rfc3339_utc("2026-02-29T00:00:00Z"));
        assert!(!is_rfc3339_utc("2026-99-99T00:00:00Z"));
        assert!(!is_rfc3339_utc("2026-10-10T25:00:00Z"));
        assert!(!is_rfc3339_utc("2026-10-10T18:61:00Z"));
        assert!(!is_rfc3339_utc("2026-10-10T18:30:61Z"));
        assert!(!is_rfc3339_utc("2026-10-10T18:30:00.Z"));
    }

    #[test]
    fn state_vector_requires_explicit_units_time_frame_and_provenance() {
        assert!(valid_sample().validate().is_ok());

        let mut invalid = valid_sample();
        invalid.position_km[1] = f64::NAN;
        assert_eq!(invalid.validate(), Err("position and velocity components must be finite"));

        let mut invalid = valid_sample();
        invalid.provenance.raw_response_sha256 = "not-a-hash".into();
        assert_eq!(
            invalid.validate(),
            Err("raw_response_sha256 must be 64 hexadecimal characters")
        );

        let mut invalid = valid_sample();
        invalid.center_id.clear();
        assert_eq!(invalid.validate(), Err("target_id and center_id are required"));

        let mut invalid = valid_sample();
        invalid.provenance.retrieved_at_utc = "yesterday".into();
        assert_eq!(
            invalid.validate(),
            Err("retrieved_at_utc must be RFC 3339 UTC text ending in Z")
        );
    }
}
