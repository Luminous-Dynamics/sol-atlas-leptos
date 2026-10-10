// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Renderer-neutral composition of hash-bound Horizons vectors.
//!
//! This module resolves centre-relative vectors into a common solar-system
//! barycentric coordinate system without changing their recorded source state.
//! It never propagates or interpolates ephemerides. A scene represents one
//! requested epoch and requires compatible time/frame/correction metadata.

use std::collections::{BTreeMap, HashSet};

use crate::horizons::HashBoundStateVector;
use crate::system_catalog::{
    catalog_object, ObjectKind, ReferencePlane, ReferenceSystem, StateVector, TimeScale,
    VectorCorrection,
};

/// Maximum epoch difference accepted when grouping independently queried states
/// into one scene. This is 0.864 milliseconds, not a propagation allowance.
const EPOCH_COMPATIBILITY_TOLERANCE_DAYS: f64 = 1.0e-8;

/// Problems that prevent a deterministic scene from being composed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SceneBuildError {
    NoSamples,
    InvalidStateVector(&'static str),
    UnknownTarget(String),
    AggregateTarget(String),
    UnknownCenter(String),
    AggregateCenter(String),
    DuplicateTarget(String),
    SelfCenteredTarget(String),
    MissingCenterSample(String),
    ReferenceCycle(String),
    EpochMismatch(String),
    IncompatibleMetadata(&'static str),
    UnsupportedVectorCorrection,
    NonFiniteResolvedState(String),
    InvalidDisplayScale,
    DisplayCoordinateOutOfRange,
}

impl std::fmt::Display for SceneBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoSamples => write!(f, "no hash-bound ephemeris samples supplied"),
            Self::InvalidStateVector(reason) => write!(f, "invalid state vector: {reason}"),
            Self::UnknownTarget(id) => write!(f, "target is not in the Sol Atlas catalogue: {id}"),
            Self::AggregateTarget(id) => {
                write!(f, "aggregate catalogue layer cannot be plotted as a point: {id}")
            }
            Self::UnknownCenter(id) => write!(f, "centre is not in the Sol Atlas catalogue: {id}"),
            Self::AggregateCenter(id) => {
                write!(f, "aggregate catalogue layer cannot be a point centre: {id}")
            }
            Self::DuplicateTarget(id) => {
                write!(f, "multiple samples for target at one epoch: {id}")
            }
            Self::SelfCenteredTarget(id) => write!(f, "target is its own vector centre: {id}"),
            Self::MissingCenterSample(id) => {
                write!(f, "no state sample supplied to resolve centre: {id}")
            }
            Self::ReferenceCycle(id) => {
                write!(f, "cyclic centre references prevent absolute position resolution: {id}")
            }
            Self::EpochMismatch(id) => {
                write!(f, "ephemeris sample epoch is incompatible with the scene: {id}")
            }
            Self::IncompatibleMetadata(field) => {
                write!(f, "ephemeris samples disagree on {field}")
            }
            Self::UnsupportedVectorCorrection => write!(
                f,
                "barycentric centre-chain composition requires geometric vectors"
            ),
            Self::NonFiniteResolvedState(id) => {
                write!(f, "resolved state for {id} contains a non-finite component")
            }
            Self::InvalidDisplayScale => {
                write!(f, "display_units_per_km must be finite and greater than zero")
            }
            Self::DisplayCoordinateOutOfRange => {
                write!(f, "camera-relative display coordinate is outside the finite f32 range")
            }
        }
    }
}

impl std::error::Error for SceneBuildError {}

/// Receipt of a source state that contributes to a composed body's absolute
/// position/velocity. The original query and response hashes remain intact.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SceneSourceReceipt {
    pub target_id: String,
    pub canonical_request_sha256: String,
    pub raw_response_sha256: String,
}

/// A barycentric state derived from one body sample plus its centre chain.
#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedBodyState {
    pub target_id: String,
    pub epoch_jd: f64,
    pub time_scale: TimeScale,
    pub reference_system: ReferenceSystem,
    pub reference_plane: ReferencePlane,
    pub vector_correction: VectorCorrection,
    /// Position in km relative to the solar-system barycentre.
    pub position_km: [f64; 3],
    /// Velocity in km/s relative to the solar-system barycentre.
    pub velocity_km_s: [f64; 3],
    /// Exact input-state receipts used to derive the composed vector.
    pub sources: Vec<SceneSourceReceipt>,
}

/// A collection of compatible barycentric state vectors for one epoch.
#[derive(Debug, Clone, PartialEq)]
pub struct EphemerisScene {
    pub epoch_jd: f64,
    pub time_scale: TimeScale,
    pub reference_system: ReferenceSystem,
    pub reference_plane: ReferencePlane,
    pub vector_correction: VectorCorrection,
    bodies: BTreeMap<String, ResolvedBodyState>,
}

impl EphemerisScene {
    /// Compose hash-bound samples into a barycentric scene. All samples must
    /// refer to one epoch (within the fixed comparison tolerance), time scale,
    /// reference system, reference plane, and vector-correction mode.
    ///
    /// A sample centered at ssb is already barycentric. A sample centered at
    /// another catalogued body is composed recursively with that body's
    /// resolved state. Every centre in the chain must have a sample in this
    /// scene; the function does not silently assume a missing centre is at zero.
    pub fn build(samples: &[HashBoundStateVector]) -> Result<Self, SceneBuildError> {
        let Some(first) = samples.first() else {
            return Err(SceneBuildError::NoSamples);
        };
        let anchor = first.state();

        let mut states = BTreeMap::<String, &StateVector>::new();
        for sample in samples {
            let state = sample.state();
            state
                .validate()
                .map_err(SceneBuildError::InvalidStateVector)?;

            let object = catalog_object(&state.target_id)
                .ok_or_else(|| SceneBuildError::UnknownTarget(state.target_id.clone()))?;
            if !is_physical_object(object.kind) {
                return Err(SceneBuildError::AggregateTarget(state.target_id.clone()));
            }
            if state.center_id == state.target_id {
                return Err(SceneBuildError::SelfCenteredTarget(state.target_id.clone()));
            }
            if state.vector_correction != VectorCorrection::Geometric {
                return Err(SceneBuildError::UnsupportedVectorCorrection);
            }

            if (state.epoch_jd - anchor.epoch_jd).abs()
                > EPOCH_COMPATIBILITY_TOLERANCE_DAYS
            {
                return Err(SceneBuildError::EpochMismatch(state.target_id.clone()));
            }
            if state.time_scale != anchor.time_scale {
                return Err(SceneBuildError::IncompatibleMetadata("time scale"));
            }
            if state.reference_system != anchor.reference_system {
                return Err(SceneBuildError::IncompatibleMetadata("reference system"));
            }
            if state.reference_plane != anchor.reference_plane {
                return Err(SceneBuildError::IncompatibleMetadata("reference plane"));
            }
            if state.vector_correction != anchor.vector_correction {
                return Err(SceneBuildError::IncompatibleMetadata("vector correction"));
            }

            if state.center_id != "ssb" {
                let center = catalog_object(&state.center_id)
                    .ok_or_else(|| SceneBuildError::UnknownCenter(state.center_id.clone()))?;
                if !is_physical_object(center.kind) {
                    return Err(SceneBuildError::AggregateCenter(state.center_id.clone()));
                }
            }

            if states.insert(state.target_id.clone(), state).is_some() {
                return Err(SceneBuildError::DuplicateTarget(state.target_id.clone()));
            }
        }

        let mut resolved = BTreeMap::new();
        for target_id in states.keys() {
            resolve_state(
                target_id,
                &states,
                &mut HashSet::new(),
                &mut resolved,
            )?;
        }

        Ok(Self {
            epoch_jd: anchor.epoch_jd,
            time_scale: anchor.time_scale,
            reference_system: anchor.reference_system,
            reference_plane: anchor.reference_plane,
            vector_correction: anchor.vector_correction,
            bodies: resolved,
        })
    }

    pub fn body(&self, target_id: &str) -> Option<&ResolvedBodyState> {
        self.bodies.get(target_id)
    }

    pub fn bodies(&self) -> impl Iterator<Item = (&str, &ResolvedBodyState)> {
        self.bodies
            .iter()
            .map(|(target_id, state)| (target_id.as_str(), state))
    }

    pub fn len(&self) -> usize {
        self.bodies.len()
    }

    pub fn is_empty(&self) -> bool {
        self.bodies.is_empty()
    }
}

fn is_physical_object(kind: ObjectKind) -> bool {
    matches!(
        kind,
        ObjectKind::Star
            | ObjectKind::Planet
            | ObjectKind::DwarfPlanet
            | ObjectKind::NaturalSatellite
    )
}

fn resolve_state(
    target_id: &str,
    states: &BTreeMap<String, &StateVector>,
    visiting: &mut HashSet<String>,
    resolved: &mut BTreeMap<String, ResolvedBodyState>,
) -> Result<ResolvedBodyState, SceneBuildError> {
    if let Some(cached) = resolved.get(target_id) {
        return Ok(cached.clone());
    }
    if !visiting.insert(target_id.to_owned()) {
        return Err(SceneBuildError::ReferenceCycle(target_id.to_owned()));
    }

    let result = (|| {
        let state = states
            .get(target_id)
            .copied()
            .ok_or_else(|| SceneBuildError::MissingCenterSample(target_id.to_owned()))?;

        let (position_km, velocity_km_s, mut sources) = if state.center_id == "ssb" {
            (
                state.position_km,
                state.velocity_km_s,
                vec![source_receipt(state)],
            )
        } else {
            let center = resolve_state(&state.center_id, states, visiting, resolved)?;
            let position_km = add_vectors(center.position_km, state.position_km);
            let velocity_km_s = add_vectors(center.velocity_km_s, state.velocity_km_s);
            let mut sources = center.sources;
            sources.push(source_receipt(state));
            sources.sort();
            sources.dedup();
            (position_km, velocity_km_s, sources)
        };

        if position_km.iter().chain(velocity_km_s.iter()).any(|x| !x.is_finite()) {
            return Err(SceneBuildError::NonFiniteResolvedState(target_id.to_owned()));
        }
        sources.sort();
        sources.dedup();

        Ok(ResolvedBodyState {
            target_id: target_id.to_owned(),
            epoch_jd: state.epoch_jd,
            time_scale: state.time_scale,
            reference_system: state.reference_system,
            reference_plane: state.reference_plane,
            vector_correction: state.vector_correction,
            position_km,
            velocity_km_s,
            sources,
        })
    })();

    visiting.remove(target_id);
    let result = result?;
    resolved.insert(target_id.to_owned(), result.clone());
    Ok(result)
}

fn source_receipt(state: &StateVector) -> SceneSourceReceipt {
    SceneSourceReceipt {
        target_id: state.target_id.clone(),
        canonical_request_sha256: state.provenance.canonical_request_sha256.clone(),
        raw_response_sha256: state.provenance.raw_response_sha256.clone(),
    }
}

fn add_vectors(left: [f64; 3], right: [f64; 3]) -> [f64; 3] {
    [
        left[0] + right[0],
        left[1] + right[1],
        left[2] + right[2],
    ]
}

/// Convert to renderer precision only after subtracting the camera origin.
/// display_units_per_km is an explicit visualization scale and never modifies
/// the scientific coordinates stored in the scene.
pub fn camera_relative_display_position(
    position_km: [f64; 3],
    camera_origin_km: [f64; 3],
    display_units_per_km: f64,
) -> Result<[f32; 3], SceneBuildError> {
    if !display_units_per_km.is_finite() || display_units_per_km <= 0.0 {
        return Err(SceneBuildError::InvalidDisplayScale);
    }

    let mut output = [0.0_f32; 3];
    for index in 0..3 {
        let delta_km = position_km[index] - camera_origin_km[index];
        let display_coordinate = delta_km * display_units_per_km;
        if !display_coordinate.is_finite()
            || display_coordinate.abs() > f32::MAX as f64
        {
            return Err(SceneBuildError::DisplayCoordinateOutOfRange);
        }
        output[index] = display_coordinate as f32;
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::horizons::{parse_horizons_vectors_json, HorizonsVectorRequest};
    use crate::system_catalog::{
        catalog_object, EphemerisProvenance, ReferencePlane, ReferenceSystem, TimeScale,
        VectorCorrection,
    };

    const EPOCH_JD: f64 = 2_461_323.5;

    fn parsed_sample(
        target_id: &str,
        center_id: &str,
        provider_center: &str,
        expected_center_name: &str,
        epoch_jd: f64,
        reference_plane: ReferencePlane,
        vector_correction: VectorCorrection,
        position_km: [f64; 3],
        velocity_km_s: [f64; 3],
    ) -> HashBoundStateVector {
        let target = catalog_object(target_id).unwrap();
        let request = HorizonsVectorRequest::for_catalog_object(
            target,
            center_id,
            provider_center,
            expected_center_name,
            vec![epoch_jd],
            TimeScale::Tdb,
            ReferenceSystem::Icrf,
            reference_plane,
            vector_correction,
        )
        .unwrap();

        let plane_label = match reference_plane {
            ReferencePlane::Ecliptic => "ECLIPTIC",
            ReferencePlane::Frame => "FRAME",
            ReferencePlane::BodyEquator => "BODY EQUATOR",
        };
        let correction_label = match vector_correction {
            VectorCorrection::Geometric => "NONE",
            VectorCorrection::LightTime => "LT",
            VectorCorrection::LightTimeAndStellarAberration => "LT+S",
        };
        let result = format!(
            "*******************************************************************************\n\
Target body name: {} ({})\n\
Center body name: {} ({})\n\
Output units    : KM-S\n\
Reference frame : ICRF\n\
Reference plane : {}\n\
Aberration corrections : {}\n\
JDTDB, Calendar Date (TDB), X, Y, Z, VX, VY, VZ\n\
$SOE\n\
{epoch_jd:.9}, A.D. 2026-Oct-10 00:00:00.0000 TDB, {:.9E}, {:.9E}, {:.9E}, {:.9E}, {:.9E}, {:.9E}\n\
$EOE\n\
*******************************************************************************",
            target.name,
            request.provider_target,
            expected_center_name,
            provider_center,
            plane_label,
            correction_label,
            position_km[0],
            position_km[1],
            position_km[2],
            velocity_km_s[0],
            velocity_km_s[1],
            velocity_km_s[2]
        )
        // Keep the Horizons protocol markers exact; the parser intentionally
        // does not accept a single-dollar near-match.
        .replace("$SOE", concat!("$", "$", "SOE"))
        .replace("$EOE", concat!("$", "$", "EOE"));
        let payload = serde_json::json!({
            "signature": { "source": "NASA/JPL Horizons API", "version": "1.3" },
            "result": result
        })
        .to_string();
        let identity = request.canonical_request_identity().unwrap();
        let provenance = EphemerisProvenance::from_bytes(
            "synthetic scene fixture",
            &identity,
            payload.as_bytes(),
            "2026-10-10T18:00:00Z",
        );
        parse_horizons_vectors_json(&payload, &request, &provenance)
            .unwrap()
            .remove(0)
    }

    fn sun_sample() -> HashBoundStateVector {
        parsed_sample(
            "sun",
            "ssb",
            "@0",
            "Solar System Barycenter",
            EPOCH_JD,
            ReferencePlane::Frame,
            VectorCorrection::Geometric,
            [100.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
        )
    }

    fn earth_sample() -> HashBoundStateVector {
        parsed_sample(
            "earth",
            "sun",
            "@10",
            "Sun",
            EPOCH_JD,
            ReferencePlane::Frame,
            VectorCorrection::Geometric,
            [1_000.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
        )
    }

    fn moon_sample() -> HashBoundStateVector {
        parsed_sample(
            "moon",
            "earth",
            "@399",
            "Earth",
            EPOCH_JD,
            ReferencePlane::Frame,
            VectorCorrection::Geometric,
            [10.0, 0.0, 0.0],
            [3.0, 0.0, 0.0],
        )
    }

    #[test]
    fn empty_sample_set_fails_closed() {
        assert_eq!(EphemerisScene::build(&[]), Err(SceneBuildError::NoSamples));
    }

    #[test]
    fn composes_barycentric_parent_chain_and_tracks_every_source() {
        let sun = sun_sample();
        let earth = earth_sample();
        let moon = moon_sample();
        let scene = EphemerisScene::build(&[sun.clone(), earth.clone(), moon.clone()]).unwrap();

        assert_eq!(scene.len(), 3);
        assert_eq!(scene.body("sun").unwrap().position_km, [100.0, 0.0, 0.0]);
        assert_eq!(scene.body("earth").unwrap().position_km, [1_100.0, 0.0, 0.0]);
        assert_eq!(scene.body("moon").unwrap().position_km, [1_110.0, 0.0, 0.0]);
        assert_eq!(scene.body("sun").unwrap().velocity_km_s, [1.0, 0.0, 0.0]);
        assert_eq!(scene.body("earth").unwrap().velocity_km_s, [3.0, 0.0, 0.0]);
        assert_eq!(scene.body("moon").unwrap().velocity_km_s, [6.0, 0.0, 0.0]);
        assert_eq!(scene.body("sun").unwrap().sources.len(), 1);
        assert_eq!(scene.body("earth").unwrap().sources.len(), 2);
        assert_eq!(scene.body("moon").unwrap().sources.len(), 3);
        assert!(scene
            .body("moon")
            .unwrap()
            .sources
            .iter()
            .any(|receipt| receipt.target_id == "earth"));
    }

    #[test]
    fn non_geometric_vectors_are_not_summed_into_barycentric_scene() {
        let light_time_corrected_sun = parsed_sample(
            "sun",
            "ssb",
            "@0",
            "Solar System Barycenter",
            EPOCH_JD,
            ReferencePlane::Frame,
            VectorCorrection::LightTime,
            [100.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
        );
        assert_eq!(
            EphemerisScene::build(&[light_time_corrected_sun]),
            Err(SceneBuildError::UnsupportedVectorCorrection)
        );
    }

    #[test]
    fn barycentric_sample_needs_no_synthetic_zero_origin() {
        let mars = parsed_sample(
            "mars",
            "ssb",
            "@0",
            "Solar System Barycenter",
            EPOCH_JD,
            ReferencePlane::Frame,
            VectorCorrection::Geometric,
            [2.0e8, -7.0e7, 3.0e7],
            [12.0, 20.0, 4.0],
        );
        let scene = EphemerisScene::build(&[mars]).unwrap();
        assert_eq!(scene.body("mars").unwrap().position_km, [2.0e8, -7.0e7, 3.0e7]);
        assert_eq!(scene.body("mars").unwrap().sources.len(), 1);
    }

    #[test]
    fn missing_center_sample_fails_closed() {
        let earth = earth_sample();
        assert!(matches!(
            EphemerisScene::build(&[earth]),
            Err(SceneBuildError::MissingCenterSample(id)) if id == "sun"
        ));
    }

    #[test]
    fn duplicate_target_sample_fails_closed() {
        let sun = sun_sample();
        assert!(matches!(
            EphemerisScene::build(&[sun.clone(), sun]),
            Err(SceneBuildError::DuplicateTarget(id)) if id == "sun"
        ));
    }

    #[test]
    fn epoch_or_coordinate_metadata_mismatch_fails_closed() {
        let sun = sun_sample();
        let later_earth = parsed_sample(
            "earth",
            "sun",
            "@10",
            "Sun",
            EPOCH_JD + 1.0,
            ReferencePlane::Frame,
            VectorCorrection::Geometric,
            [1_000.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
        );
        assert!(matches!(
            EphemerisScene::build(&[sun.clone(), later_earth]),
            Err(SceneBuildError::EpochMismatch(id)) if id == "earth"
        ));

        let ecliptic_earth = parsed_sample(
            "earth",
            "sun",
            "@10",
            "Sun",
            EPOCH_JD,
            ReferencePlane::Ecliptic,
            VectorCorrection::Geometric,
            [1_000.0, 0.0, 0.0],
            [2.0, 0.0, 0.0],
        );
        assert_eq!(
            EphemerisScene::build(&[sun, ecliptic_earth]),
            Err(SceneBuildError::IncompatibleMetadata("reference plane"))
        );
    }

    #[test]
    fn self_center_and_reference_cycle_fail_closed() {
        let self_centered = parsed_sample(
            "mars",
            "mars",
            "@499",
            "Mars",
            EPOCH_JD,
            ReferencePlane::Frame,
            VectorCorrection::Geometric,
            [0.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
        );
        assert!(matches!(
            EphemerisScene::build(&[self_centered]),
            Err(SceneBuildError::SelfCenteredTarget(id)) if id == "mars"
        ));

        let earth_centered_on_mars = parsed_sample(
            "earth",
            "mars",
            "@499",
            "Mars",
            EPOCH_JD,
            ReferencePlane::Frame,
            VectorCorrection::Geometric,
            [10.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
        );
        let mars_centered_on_earth = parsed_sample(
            "mars",
            "earth",
            "@399",
            "Earth",
            EPOCH_JD,
            ReferencePlane::Frame,
            VectorCorrection::Geometric,
            [-10.0, 0.0, 0.0],
            [0.0, 0.0, 0.0],
        );
        assert!(matches!(
            EphemerisScene::build(&[earth_centered_on_mars, mars_centered_on_earth]),
            Err(SceneBuildError::ReferenceCycle(_))
        ));
    }

    #[test]
    fn camera_relative_transform_subtracts_large_origin_before_f32_cast() {
        let position = [100_000_000_001.0, -50_000_000_003.0, 8.0];
        let origin = [100_000_000_000.0, -50_000_000_000.0, 0.0];
        assert_eq!(
            camera_relative_display_position(position, origin, 0.5).unwrap(),
            [0.5, -1.5, 4.0]
        );
        assert_eq!(
            camera_relative_display_position(position, origin, 0.0),
            Err(SceneBuildError::InvalidDisplayScale)
        );
    }
}
