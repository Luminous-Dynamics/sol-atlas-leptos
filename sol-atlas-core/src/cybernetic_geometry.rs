// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Semantic validation and transform resolution for cybernetic interface geometry.
//!
//! Complements JSON Schema by checking graph-level invariants: unique root,
//! parent existence, connectivity, acyclicity, coordinate conventions,
//! quaternion normalization, interface references, and transform round trips.
//! A valid synthetic graph is still synthetic; this does not establish physical
//! metrology, anatomical truth, fit, pressure/shear safety, or clinical suitability.

use serde_json::Value;
use std::collections::{HashMap, HashSet};

pub const SCHEMA_ID: &str = "luminous.cybernetic-interface-geometry";
pub const QUATERNION_NORM_TOLERANCE: f64 = 1.0e-6;
pub const TRANSLATION_CLOSURE_TOLERANCE_M: f64 = 1.0e-9;
pub const ROTATION_CLOSURE_TOLERANCE: f64 = 1.0e-12;

/// Stable category for a semantic geometry validation failure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeometryErrorCode {
    UnsupportedSchema,
    MissingOrInvalidField,
    EmptyFrameGraph,
    EmptyFrameId,
    DuplicateFrameId,
    RootCountMismatch,
    DeclaredRootMismatch,
    MissingParentFrame,
    DisconnectedFrame,
    CycleDetected,
    RootTransformMustBeNull,
    ChildTransformMissing,
    InvalidTransformShape,
    NonFiniteTransform,
    QuaternionNotNormalized,
    CoordinateConventionMismatch,
    DuplicateInterfaceId,
    UnknownInterfaceFrame,
    DuplicateObservationId,
    UnknownObservationInterface,
    TransformClosureFailure,
}

/// One error and its JSON-style source path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeometryValidationError {
    pub path: String,
    pub code: GeometryErrorCode,
}

/// Transform from a child frame into its parent frame.
///
/// Quaternion convention is x,y,z,w; translation is metres. Composition
/// follows parent-from-child semantics.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RigidTransform3 {
    pub translation_m: [f64; 3],
    pub quaternion_xyzw: [f64; 4],
}

impl RigidTransform3 {
    pub const fn identity() -> Self {
        Self {
            translation_m: [0.0, 0.0, 0.0],
            quaternion_xyzw: [0.0, 0.0, 0.0, 1.0],
        }
    }

    /// Compose A-from-B with B-from-C to obtain A-from-C.
    pub fn compose(self, b_from_c: Self) -> Self {
        let translated = rotate(self.quaternion_xyzw, b_from_c.translation_m);
        let mut quaternion = qmul(self.quaternion_xyzw, b_from_c.quaternion_xyzw);
        let quaternion_norm = dot4(quaternion, quaternion).sqrt();
        if quaternion_norm.is_finite() && quaternion_norm > 1.0e-15 {
            for component in &mut quaternion {
                *component /= quaternion_norm;
            }
        }
        Self {
            translation_m: add3(self.translation_m, translated),
            quaternion_xyzw: quaternion,
        }
    }

    pub fn inverse(self) -> Self {
        let q = self.quaternion_xyzw;
        let inverse_q = [-q[0], -q[1], -q[2], q[3]];
        Self {
            translation_m: rotate(inverse_q, scale3(self.translation_m, -1.0)),
            quaternion_xyzw: inverse_q,
        }
    }

    pub fn residual(self, other: Self) -> (f64, f64) {
        let delta = [
            self.translation_m[0] - other.translation_m[0],
            self.translation_m[1] - other.translation_m[1],
            self.translation_m[2] - other.translation_m[2],
        ];
        let translation_error = dot3(delta, delta).sqrt();
        let rotation_error =
            (1.0 - dot4(self.quaternion_xyzw, other.quaternion_xyzw).abs().min(1.0)).abs();
        (translation_error, rotation_error)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ResolvedFrame {
    pub frame_id: String,
    pub root_from_frame: RigidTransform3,
}

/// A valid tree with deterministic root-relative and pairwise transforms.
///
/// The source schema represents a tree, not redundant measured loop edges.
/// Closure checks here establish arithmetic consistency of the computed
/// transforms, not independent physical loop-closure metrology.
#[derive(Debug, Clone, PartialEq)]
pub struct ValidatedFrameGraph {
    pub root_frame_id: String,
    pub frames: Vec<ResolvedFrame>,
    pub closure_pairs_checked: usize,
    pub max_translation_closure_error_m: f64,
    pub max_rotation_closure_error: f64,
}

impl ValidatedFrameGraph {
    pub fn root_from_frame(&self, frame_id: &str) -> Option<RigidTransform3> {
        self.frames
            .iter()
            .find(|frame| frame.frame_id == frame_id)
            .map(|frame| frame.root_from_frame)
    }

    /// Return from-frame-from-to-frame.
    pub fn frame_from_frame(&self, from: &str, to: &str) -> Option<RigidTransform3> {
        Some(
            self.root_from_frame(from)?
                .inverse()
                .compose(self.root_from_frame(to)?),
        )
    }
}

#[derive(Debug, Clone)]
struct ParsedFrame {
    id: String,
    parent: Option<String>,
    parent_from_child: Option<RigidTransform3>,
}

/// Validate graph and interface references after applying the declared JSON Schema.
///
/// This function is not a complete JSON Schema implementation. Call the
/// schema validator separately, then use this function for semantic graph rules.
pub fn validate_cybernetic_interface_geometry(
    document: &Value,
) -> Result<ValidatedFrameGraph, Vec<GeometryValidationError>> {
    let mut errors = Vec::new();

    if document.get("schema").and_then(Value::as_str) != Some(SCHEMA_ID)
        || document.get("schema_version").and_then(Value::as_u64) != Some(1)
    {
        return Err(vec![err("$", GeometryErrorCode::UnsupportedSchema)]);
    }

    let Some(graph) = document.get("frame_graph").and_then(Value::as_object) else {
        return Err(vec![err("frame_graph", GeometryErrorCode::MissingOrInvalidField)]);
    };
    let root_id = graph
        .get("root_frame_id")
        .and_then(Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_default()
        .to_owned();
    if root_id.is_empty() {
        errors.push(err("frame_graph.root_frame_id", GeometryErrorCode::MissingOrInvalidField));
    }
    if graph.get("transform_direction").and_then(Value::as_str)
        != Some("transform_parent_from_child")
    {
        errors.push(err(
            "frame_graph.transform_direction",
            GeometryErrorCode::CoordinateConventionMismatch,
        ));
    }

    let Some(raw_frames) = graph.get("frames").and_then(Value::as_array) else {
        errors.push(err("frame_graph.frames", GeometryErrorCode::MissingOrInvalidField));
        return Err(errors);
    };
    if raw_frames.is_empty() {
        return Err(vec![err("frame_graph.frames", GeometryErrorCode::EmptyFrameGraph)]);
    }

    let mut frames = Vec::with_capacity(raw_frames.len());
    let mut indices = HashMap::new();
    for (i, raw) in raw_frames.iter().enumerate() {
        let path = format!("frame_graph.frames[{i}]");
        let Some(frame) = raw.as_object() else {
            errors.push(err(&path, GeometryErrorCode::MissingOrInvalidField));
            continue;
        };
        let id = frame
            .get("frame_id")
            .and_then(Value::as_str)
            .filter(|value| !value.trim().is_empty())
            .map(str::to_owned);
        let Some(id) = id else {
            errors.push(err(&format!("{path}.frame_id"), GeometryErrorCode::EmptyFrameId));
            continue;
        };
        if indices.insert(id.clone(), frames.len()).is_some() {
            errors.push(err(&format!("{path}.frame_id"), GeometryErrorCode::DuplicateFrameId));
        }

        let parent = match frame.get("parent_frame_id") {
            Some(Value::Null) => None,
            Some(Value::String(value)) if !value.trim().is_empty() => Some(value.clone()),
            _ => {
                errors.push(err(
                    &format!("{path}.parent_frame_id"),
                    GeometryErrorCode::MissingOrInvalidField,
                ));
                None
            }
        };

        if frame.get("handedness").and_then(Value::as_str) != Some("right_handed")
            || frame.get("linear_unit").and_then(Value::as_str) != Some("m")
            || frame.get("angular_unit").and_then(Value::as_str) != Some("rad")
            || !valid_axis_labels(frame.get("axis_labels"))
        {
            errors.push(err(&path, GeometryErrorCode::CoordinateConventionMismatch));
        }

        let transform = if parent.is_none() {
            if !matches!(frame.get("transform_parent_from_child"), Some(Value::Null)) {
                errors.push(err(
                    &format!("{path}.transform_parent_from_child"),
                    GeometryErrorCode::RootTransformMustBeNull,
                ));
            }
            None
        } else {
            match frame.get("transform_parent_from_child") {
                Some(value) => parse_transform(value, &format!("{path}.transform_parent_from_child"), &mut errors),
                None => {
                    errors.push(err(
                        &format!("{path}.transform_parent_from_child"),
                        GeometryErrorCode::ChildTransformMissing,
                    ));
                    None
                }
            }
        };
        frames.push(ParsedFrame { id, parent, parent_from_child: transform });
    }

    if frames.is_empty() {
        errors.push(err("frame_graph.frames", GeometryErrorCode::EmptyFrameGraph));
        return Err(errors);
    }

    let roots: Vec<usize> = frames.iter().enumerate()
        .filter_map(|(i, frame)| frame.parent.is_none().then_some(i))
        .collect();
    if roots.len() != 1 {
        errors.push(err("frame_graph.frames", GeometryErrorCode::RootCountMismatch));
    } else if frames[roots[0]].id != root_id {
        errors.push(err("frame_graph.root_frame_id", GeometryErrorCode::DeclaredRootMismatch));
    }

    for (i, frame) in frames.iter().enumerate() {
        if let Some(parent) = &frame.parent {
            if !indices.contains_key(parent) {
                errors.push(err(
                    &format!("frame_graph.frames[{i}].parent_frame_id"),
                    GeometryErrorCode::MissingParentFrame,
                ));
            }
        }
    }

    validate_interfaces(document, &indices, &mut errors);
    if !errors.is_empty() {
        return Err(errors);
    }

    let mut states = vec![0u8; frames.len()];
    let mut transforms = vec![None; frames.len()];
    for i in 0..frames.len() {
        let _ = resolve(i, &frames, &indices, &mut states, &mut transforms, &mut errors);
    }
    for (i, transform) in transforms.iter().enumerate() {
        if transform.is_none()
            && !errors.iter().any(|item| {
                item.path == format!("frame_graph.frames[{i}]")
                    && item.code == GeometryErrorCode::CycleDetected
            })
        {
            errors.push(err(
                &format!("frame_graph.frames[{i}]"),
                GeometryErrorCode::DisconnectedFrame,
            ));
        }
    }
    if !errors.is_empty() {
        return Err(errors);
    }

    let resolved: Vec<ResolvedFrame> = frames.iter().enumerate()
        .map(|(i, frame)| ResolvedFrame {
            frame_id: frame.id.clone(),
            root_from_frame: transforms[i].unwrap_or_else(RigidTransform3::identity),
        })
        .collect();

    let mut pairs = 0usize;
    let mut max_translation: f64 = 0.0;
    let mut max_rotation: f64 = 0.0;
    for from in &resolved {
        for to in &resolved {
            let from_from_to = from.root_from_frame.inverse().compose(to.root_from_frame);
            let round_trip = from.root_from_frame.compose(from_from_to);
            let (translation_error, rotation_error) = round_trip.residual(to.root_from_frame);
            pairs += 1;
            max_translation = max_translation.max(translation_error);
            max_rotation = max_rotation.max(rotation_error);
            if !translation_error.is_finite()
                || !rotation_error.is_finite()
                || translation_error > TRANSLATION_CLOSURE_TOLERANCE_M
                || rotation_error > ROTATION_CLOSURE_TOLERANCE
            {
                errors.push(err("frame_graph", GeometryErrorCode::TransformClosureFailure));
                break;
            }
        }
        if !errors.is_empty() { break; }
    }
    if !errors.is_empty() {
        return Err(errors);
    }

    Ok(ValidatedFrameGraph {
        root_frame_id: root_id,
        frames: resolved,
        closure_pairs_checked: pairs,
        max_translation_closure_error_m: max_translation,
        max_rotation_closure_error: max_rotation,
    })
}

fn valid_axis_labels(value: Option<&Value>) -> bool {
    value.and_then(Value::as_array).is_some_and(|labels| {
        labels.len() == 3
            && labels.iter().all(|label| {
                label.as_str().is_some_and(|value| !value.trim().is_empty())
            })
    })
}

fn parse_transform(
    value: &Value,
    path: &str,
    errors: &mut Vec<GeometryValidationError>,
) -> Option<RigidTransform3> {
    let Some(object) = value.as_object() else {
        errors.push(err(path, GeometryErrorCode::MissingOrInvalidField));
        return None;
    };
    let translation = parse_array::<3>(object.get("translation_m"), &format!("{path}.translation_m"), errors)?;
    let quaternion = parse_array::<4>(object.get("quaternion_xyzw"), &format!("{path}.quaternion_xyzw"), errors)?;
    let norm = dot4(quaternion, quaternion).sqrt();
    if !norm.is_finite() || (norm - 1.0).abs() > QUATERNION_NORM_TOLERANCE {
        errors.push(err(
            &format!("{path}.quaternion_xyzw"),
            GeometryErrorCode::QuaternionNotNormalized,
        ));
        return None;
    }
    Some(RigidTransform3 { translation_m: translation, quaternion_xyzw: quaternion })
}

fn parse_array<const N: usize>(
    value: Option<&Value>,
    path: &str,
    errors: &mut Vec<GeometryValidationError>,
) -> Option<[f64; N]> {
    let Some(values) = value.and_then(Value::as_array) else {
        errors.push(err(path, GeometryErrorCode::InvalidTransformShape));
        return None;
    };
    if values.len() != N {
        errors.push(err(path, GeometryErrorCode::InvalidTransformShape));
        return None;
    }
    let mut result = [0.0; N];
    for (i, item) in values.iter().enumerate() {
        let Some(number) = item.as_f64().filter(|number| number.is_finite()) else {
            errors.push(err(&format!("{path}[{i}]"), GeometryErrorCode::NonFiniteTransform));
            return None;
        };
        result[i] = number;
    }
    Some(result)
}

fn resolve(
    index: usize,
    frames: &[ParsedFrame],
    indices: &HashMap<String, usize>,
    states: &mut [u8],
    transforms: &mut [Option<RigidTransform3>],
    errors: &mut Vec<GeometryValidationError>,
) -> Option<RigidTransform3> {
    if states[index] == 2 { return transforms[index]; }
    if states[index] == 1 {
        errors.push(err(
            &format!("frame_graph.frames[{index}]"),
            GeometryErrorCode::CycleDetected,
        ));
        return None;
    }
    states[index] = 1;
    let result = match (&frames[index].parent, frames[index].parent_from_child) {
        (None, _) => Some(RigidTransform3::identity()),
        (Some(parent), Some(parent_from_child)) => {
            let parent_index = *indices.get(parent)?;
            resolve(parent_index, frames, indices, states, transforms, errors)
                .and_then(|root_from_parent| {
                    let composed = root_from_parent.compose(parent_from_child);
                    let finite_translation = composed.translation_m.iter().all(|value| value.is_finite());
                    let finite_quaternion = composed.quaternion_xyzw.iter().all(|value| value.is_finite());
                    if !finite_translation || !finite_quaternion {
                        errors.push(err(
                            &format!("frame_graph.frames[{index}].transform_parent_from_child"),
                            GeometryErrorCode::NonFiniteTransform,
                        ));
                        None
                    } else {
                        Some(composed)
                    }
                })
        }
        (Some(_), None) => None,
    };
    states[index] = 2;
    transforms[index] = result;
    result
}

fn validate_interfaces(
    document: &Value,
    frame_indices: &HashMap<String, usize>,
    errors: &mut Vec<GeometryValidationError>,
) {
    let Some(envelopes) = document.get("interface_envelopes").and_then(Value::as_array) else {
        errors.push(err("interface_envelopes", GeometryErrorCode::MissingOrInvalidField));
        return;
    };
    let mut interfaces = HashSet::new();
    for (i, envelope) in envelopes.iter().enumerate() {
        let path = format!("interface_envelopes[{i}]");
        let Some(object) = envelope.as_object() else {
            errors.push(err(&path, GeometryErrorCode::MissingOrInvalidField));
            continue;
        };
        let id = object.get("interface_id").and_then(Value::as_str)
            .filter(|value| !value.trim().is_empty());
        let Some(id) = id else {
            errors.push(err(&format!("{path}.interface_id"), GeometryErrorCode::MissingOrInvalidField));
            continue;
        };
        if !interfaces.insert(id.to_owned()) {
            errors.push(err(&format!("{path}.interface_id"), GeometryErrorCode::DuplicateInterfaceId));
        }
        for field in ["body_frame_ref", "device_frame_ref"] {
            if object.get(field).and_then(Value::as_str)
                .is_none_or(|frame| !frame_indices.contains_key(frame))
            {
                errors.push(err(&format!("{path}.{field}"), GeometryErrorCode::UnknownInterfaceFrame));
            }
        }
    }

    let Some(observations) = document.get("interface_observations").and_then(Value::as_array) else {
        errors.push(err("interface_observations", GeometryErrorCode::MissingOrInvalidField));
        return;
    };
    let mut observation_ids = HashSet::new();
    for (i, observation) in observations.iter().enumerate() {
        let path = format!("interface_observations[{i}]");
        let Some(object) = observation.as_object() else {
            errors.push(err(&path, GeometryErrorCode::MissingOrInvalidField));
            continue;
        };
        let id = object.get("observation_id").and_then(Value::as_str);
        if id.is_none_or(|value| value.trim().is_empty()) {
            errors.push(err(&format!("{path}.observation_id"), GeometryErrorCode::MissingOrInvalidField));
        } else if !observation_ids.insert(id.unwrap_or_default().to_owned()) {
            errors.push(err(&format!("{path}.observation_id"), GeometryErrorCode::DuplicateObservationId));
        }
        if object.get("interface_id").and_then(Value::as_str)
            .is_none_or(|id| !interfaces.contains(id))
        {
            errors.push(err(&format!("{path}.interface_id"), GeometryErrorCode::UnknownObservationInterface));
        }
    }
}

fn err(path: &str, code: GeometryErrorCode) -> GeometryValidationError {
    GeometryValidationError { path: path.to_owned(), code }
}

fn qmul(a: [f64; 4], b: [f64; 4]) -> [f64; 4] {
    let [x1, y1, z1, w1] = a;
    let [x2, y2, z2, w2] = b;
    [
        w1 * x2 + x1 * w2 + y1 * z2 - z1 * y2,
        w1 * y2 - x1 * z2 + y1 * w2 + z1 * x2,
        w1 * z2 + x1 * y2 - y1 * x2 + z1 * w2,
        w1 * w2 - x1 * x2 - y1 * y2 - z1 * z2,
    ]
}

fn rotate(q: [f64; 4], v: [f64; 3]) -> [f64; 3] {
    let [x, y, z, w] = q;
    let qv = [x, y, z];
    let t = scale3(cross3(qv, v), 2.0);
    add3(add3(v, scale3(t, w)), cross3(qv, t))
}
fn dot3(a: [f64; 3], b: [f64; 3]) -> f64 { a[0]*b[0] + a[1]*b[1] + a[2]*b[2] }
fn dot4(a: [f64; 4], b: [f64; 4]) -> f64 { a[0]*b[0] + a[1]*b[1] + a[2]*b[2] + a[3]*b[3] }
fn add3(a: [f64; 3], b: [f64; 3]) -> [f64; 3] { [a[0]+b[0], a[1]+b[1], a[2]+b[2]] }
fn scale3(a: [f64; 3], s: f64) -> [f64; 3] { [a[0]*s, a[1]*s, a[2]*s] }
fn cross3(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[1]*b[2]-a[2]*b[1], a[2]*b[0]-a[0]*b[2], a[0]*b[1]-a[1]*b[0]]
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn fixture() -> Value {
        json!({
            "schema": SCHEMA_ID,
            "schema_version": 1,
            "frame_graph": {
                "root_frame_id": "world",
                "transform_direction": "transform_parent_from_child",
                "frames": [
                    {"frame_id":"world","parent_frame_id":null,"transform_parent_from_child":null,
                     "handedness":"right_handed","axis_labels":["X","Y","Z"],"linear_unit":"m","angular_unit":"rad"},
                    {"frame_id":"body","parent_frame_id":"world",
                     "transform_parent_from_child":{"translation_m":[1.0,0.0,0.0],"quaternion_xyzw":[0.0,0.0,0.0,1.0]},
                     "handedness":"right_handed","axis_labels":["X","Y","Z"],"linear_unit":"m","angular_unit":"rad"},
                    {"frame_id":"device","parent_frame_id":"body",
                     "transform_parent_from_child":{"translation_m":[0.0,2.0,0.0],"quaternion_xyzw":[0.0,0.0,0.0,1.0]},
                     "handedness":"right_handed","axis_labels":["X","Y","Z"],"linear_unit":"m","angular_unit":"rad"}
                ]
            },
            "interface_envelopes":[{"interface_id":"cuff","body_frame_ref":"body","device_frame_ref":"device"}],
            "interface_observations":[{"observation_id":"pressure-1","interface_id":"cuff"}]
        })
    }

    fn contains(errors: &[GeometryValidationError], code: GeometryErrorCode) -> bool {
        errors.iter().any(|item| item.code == code)
    }

    #[test]
    fn resolves_root_and_pairwise_transforms() {
        let graph = validate_cybernetic_interface_geometry(&fixture()).unwrap();
        assert_eq!(graph.root_frame_id, "world");
        assert_eq!(graph.closure_pairs_checked, 9);
        let device = graph.root_from_frame("device").unwrap();
        assert!((device.translation_m[0] - 1.0).abs() < 1e-12);
        assert!((device.translation_m[1] - 2.0).abs() < 1e-12);
        let body_from_device = graph.frame_from_frame("body", "device").unwrap();
        assert!(body_from_device.translation_m[0].abs() < 1e-12);
        assert!((body_from_device.translation_m[1] - 2.0).abs() < 1e-12);
    }

    #[test]
    fn composes_child_translation_through_rotated_parent() {
        let mut doc = fixture();
        let half = std::f64::consts::FRAC_1_SQRT_2;
        doc["frame_graph"]["frames"][1]["transform_parent_from_child"]["quaternion_xyzw"] =
            json!([0.0, 0.0, half, half]);

        let graph = validate_cybernetic_interface_geometry(&doc).unwrap();
        let root_from_device = graph.root_from_frame("device").unwrap();
        // Body is translated +1m in root X and rotated +90° about Z.
        // The device's +2m local Y translation therefore becomes -2m root X.
        assert!((root_from_device.translation_m[0] - (-1.0)).abs() < 1e-12);
        assert!(root_from_device.translation_m[1].abs() < 1e-12);

        let world_from_device = graph.frame_from_frame("world", "device").unwrap();
        let round_trip = world_from_device.inverse().compose(world_from_device);
        let (translation_error, rotation_error) =
            round_trip.residual(RigidTransform3::identity());
        assert!(translation_error < 1e-12);
        assert!(rotation_error < 1e-12);
    }

    #[test]
    fn rejects_duplicate_ids_and_multiple_roots() {
        let mut doc = fixture();
        let duplicate = doc["frame_graph"]["frames"][1].clone();
        doc["frame_graph"]["frames"].as_array_mut().unwrap().push(duplicate);
        let errors = validate_cybernetic_interface_geometry(&doc).unwrap_err();
        assert!(contains(&errors, GeometryErrorCode::DuplicateFrameId));

        let mut doc = fixture();
        doc["frame_graph"]["frames"][2]["parent_frame_id"] = Value::Null;
        doc["frame_graph"]["frames"][2]["transform_parent_from_child"] = Value::Null;
        let errors = validate_cybernetic_interface_geometry(&doc).unwrap_err();
        assert!(contains(&errors, GeometryErrorCode::RootCountMismatch));
    }

    #[test]
    fn rejects_missing_parent_and_cycle() {
        let mut doc = fixture();
        doc["frame_graph"]["frames"][2]["parent_frame_id"] = json!("missing");
        let errors = validate_cybernetic_interface_geometry(&doc).unwrap_err();
        assert!(contains(&errors, GeometryErrorCode::MissingParentFrame));

        let mut doc = fixture();
        doc["frame_graph"]["frames"][1]["parent_frame_id"] = json!("device");
        let errors = validate_cybernetic_interface_geometry(&doc).unwrap_err();
        assert!(contains(&errors, GeometryErrorCode::CycleDetected));
    }

    #[test]
    fn rejects_invalid_quaternion_nonfinite_and_wrong_units() {
        let mut doc = fixture();
        doc["frame_graph"]["frames"][1]["transform_parent_from_child"]["quaternion_xyzw"] =
            json!([0.0, 0.0, 0.0, 2.0]);
        let errors = validate_cybernetic_interface_geometry(&doc).unwrap_err();
        assert!(contains(&errors, GeometryErrorCode::QuaternionNotNormalized));

        let mut doc = fixture();
        doc["frame_graph"]["frames"][1]["transform_parent_from_child"]["translation_m"] =
            json!([1.0, "bad", 0.0]);
        let errors = validate_cybernetic_interface_geometry(&doc).unwrap_err();
        assert!(contains(&errors, GeometryErrorCode::NonFiniteTransform));

        let mut doc = fixture();
        doc["frame_graph"]["frames"][1]["linear_unit"] = json!("mm");
        let errors = validate_cybernetic_interface_geometry(&doc).unwrap_err();
        assert!(contains(&errors, GeometryErrorCode::CoordinateConventionMismatch));
    }

    #[test]
    fn rejects_finite_inputs_that_overflow_during_transform_composition() {
        let mut doc = fixture();
        doc["frame_graph"]["frames"][1]["transform_parent_from_child"]["translation_m"] =
            json!([1.0e308, 0.0, 0.0]);
        doc["frame_graph"]["frames"][2]["transform_parent_from_child"]["translation_m"] =
            json!([1.0e308, 0.0, 0.0]);
        let errors = validate_cybernetic_interface_geometry(&doc).unwrap_err();
        assert!(contains(&errors, GeometryErrorCode::NonFiniteTransform));
    }

    #[test]
    fn rejects_unknown_interface_frame_and_observation_reference() {
        let mut doc = fixture();
        doc["interface_envelopes"][0]["device_frame_ref"] = json!("unknown");
        let errors = validate_cybernetic_interface_geometry(&doc).unwrap_err();
        assert!(contains(&errors, GeometryErrorCode::UnknownInterfaceFrame));

        let mut doc = fixture();
        doc["interface_observations"][0]["interface_id"] = json!("unknown");
        let errors = validate_cybernetic_interface_geometry(&doc).unwrap_err();
        assert!(contains(&errors, GeometryErrorCode::UnknownObservationInterface));
    }

    #[test]
    fn rejects_unsupported_schema_version() {
        let mut doc = fixture();
        doc["schema_version"] = json!(2);
        let errors = validate_cybernetic_interface_geometry(&doc).unwrap_err();
        assert!(contains(&errors, GeometryErrorCode::UnsupportedSchema));
    }
}
