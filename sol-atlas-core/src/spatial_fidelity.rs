// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Task-weighted spatial fidelity metrics for calibrated point clouds.
//!
//! This module complements global geometric metrics with reference-authored
//! semantic importance weights. It is not an implementation of SHREC 2026
//! FACD: FACD derives criticality from local geometry, while this metric lets
//! qualified reviewers declare why a region matters independently of its
//! curvature.
//!
//! Inputs must already be in the same calibrated coordinate frame and in
//! metres. This module performs no registration, meshing, filtering, normal
//! estimation, sensor calibration, rights evaluation or physical validation.
//! The O(n*m) nearest-neighbour implementation is a deterministic reference
//! oracle for modest evaluation fixtures, not a large-cloud production index.

use std::fmt;

/// A reference point with an externally authored semantic-importance weight.
///
/// Weights must be finite and strictly positive. Their absolute scale does not
/// affect the weighted metrics; only their relative values matter.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ReferenceSample {
    pub position_m: [f64; 3],
    pub importance_weight: f64,
}

impl ReferenceSample {
    pub const fn new(position_m: [f64; 3], importance_weight: f64) -> Self {
        Self {
            position_m,
            importance_weight,
        }
    }
}

/// A reconstructed point in the same calibrated metric coordinate frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ReconstructedSample {
    pub position_m: [f64; 3],
}

impl ReconstructedSample {
    pub const fn new(position_m: [f64; 3]) -> Self {
        Self { position_m }
    }
}

/// Explicit input-validation failures. Invalid data is never silently skipped.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpatialFidelityError {
    EmptyReference,
    EmptyReconstruction,
    InvalidTolerance,
    InvalidReferenceCoordinate { point: usize, axis: usize },
    InvalidReconstructionCoordinate { point: usize, axis: usize },
    InvalidImportanceWeight { point: usize },
}

impl fmt::Display for SpatialFidelityError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyReference => write!(f, "reference point cloud is empty"),
            Self::EmptyReconstruction => write!(f, "reconstruction point cloud is empty"),
            Self::InvalidTolerance => write!(f, "tolerance must be finite and greater than zero"),
            Self::InvalidReferenceCoordinate { point, axis } => {
                write!(f, "reference point {point} has a non-finite coordinate on axis {axis}")
            }
            Self::InvalidReconstructionCoordinate { point, axis } => {
                write!(
                    f,
                    "reconstruction point {point} has a non-finite coordinate on axis {axis}"
                )
            }
            Self::InvalidImportanceWeight { point } => write!(
                f,
                "reference point {point} has a non-finite or non-positive importance weight"
            ),
        }
    }
}

impl std::error::Error for SpatialFidelityError {}

/// Global and feature-weighted point-cloud fidelity at one declared tolerance.
///
/// Chamfer values are in metres. Precision/recall/F1 values are in [0, 1].
/// Reference recall asks how much of the reference surface was reconstructed;
/// reconstruction precision asks how much of the candidate surface is
/// supported by the reference. The feature-weighted variants use only the
/// frozen reference importance weights, including when assigning weights to
/// candidate points through their nearest reference point.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpatialFidelityMetrics {
    pub reference_to_reconstruction_mean_m: f64,
    pub reconstruction_to_reference_mean_m: f64,
    pub symmetric_chamfer_m: f64,
    pub feature_reference_to_reconstruction_mean_m: f64,
    pub feature_reconstruction_to_reference_mean_m: f64,
    pub feature_weighted_symmetric_chamfer_m: f64,
    pub global_precision_at_tolerance: f64,
    pub global_recall_at_tolerance: f64,
    pub global_f1_at_tolerance: f64,
    pub feature_precision_at_tolerance: f64,
    pub feature_recall_at_tolerance: f64,
    pub feature_f1_at_tolerance: f64,
    pub tolerance_m: f64,
    pub reference_point_count: usize,
    pub reconstruction_point_count: usize,
}

fn squared_distance(a: [f64; 3], b: [f64; 3]) -> f64 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    dx * dx + dy * dy + dz * dz
}

fn nearest_reconstruction_distance(
    query: [f64; 3],
    candidates: &[ReconstructedSample],
) -> f64 {
    candidates
        .iter()
        .map(|candidate| squared_distance(query, candidate.position_m))
        .fold(f64::INFINITY, f64::min)
        .sqrt()
}

fn nearest_reference(
    query: [f64; 3],
    candidates: &[ReferenceSample],
) -> (f64, usize) {
    let mut best_distance_squared = f64::INFINITY;
    let mut best_index = 0usize;

    for (index, candidate) in candidates.iter().enumerate() {
        let distance_squared = squared_distance(query, candidate.position_m);
        if distance_squared < best_distance_squared {
            best_distance_squared = distance_squared;
            best_index = index;
        }
    }

    (best_distance_squared.sqrt(), best_index)
}

fn f1(precision: f64, recall: f64) -> f64 {
    let denominator = precision + recall;
    if denominator == 0.0 {
        0.0
    } else {
        2.0 * precision * recall / denominator
    }
}

fn validate_inputs(
    reference: &[ReferenceSample],
    reconstruction: &[ReconstructedSample],
    tolerance_m: f64,
) -> Result<(), SpatialFidelityError> {
    if reference.is_empty() {
        return Err(SpatialFidelityError::EmptyReference);
    }
    if reconstruction.is_empty() {
        return Err(SpatialFidelityError::EmptyReconstruction);
    }
    if !tolerance_m.is_finite() || tolerance_m <= 0.0 {
        return Err(SpatialFidelityError::InvalidTolerance);
    }

    for (point_index, point) in reference.iter().enumerate() {
        for (axis, coordinate) in point.position_m.iter().enumerate() {
            if !coordinate.is_finite() {
                return Err(SpatialFidelityError::InvalidReferenceCoordinate {
                    point: point_index,
                    axis,
                });
            }
        }
        if !point.importance_weight.is_finite() || point.importance_weight <= 0.0 {
            return Err(SpatialFidelityError::InvalidImportanceWeight { point: point_index });
        }
    }

    for (point_index, point) in reconstruction.iter().enumerate() {
        for (axis, coordinate) in point.position_m.iter().enumerate() {
            if !coordinate.is_finite() {
                return Err(SpatialFidelityError::InvalidReconstructionCoordinate {
                    point: point_index,
                    axis,
                });
            }
        }
    }

    Ok(())
}

/// Evaluate global and semantic feature-weighted spatial fidelity.
///
/// The reference set supplies semantic importance weights before candidate
/// evaluation. Each candidate point inherits the importance weight of its
/// nearest reference point for the reconstruction-to-reference direction.
/// This prevents a candidate from making its own result look better by
/// assigning itself low weights.
///
/// The returned metric is a complementary research measure, not a single
/// universal measure of 3D quality. Always report the directional errors and
/// global metrics alongside the feature-weighted scores.
pub fn evaluate_spatial_fidelity(
    reference: &[ReferenceSample],
    reconstruction: &[ReconstructedSample],
    tolerance_m: f64,
) -> Result<SpatialFidelityMetrics, SpatialFidelityError> {
    validate_inputs(reference, reconstruction, tolerance_m)?;

    let mut reference_to_reconstruction_sum = 0.0;
    let mut feature_reference_to_reconstruction_sum = 0.0;
    let mut reference_weight_sum = 0.0;
    let mut reference_hits = 0usize;
    let mut feature_reference_hits = 0.0;

    for point in reference {
        let distance = nearest_reconstruction_distance(point.position_m, reconstruction);
        reference_to_reconstruction_sum += distance;
        feature_reference_to_reconstruction_sum += distance * point.importance_weight;
        reference_weight_sum += point.importance_weight;

        if distance <= tolerance_m {
            reference_hits += 1;
            feature_reference_hits += point.importance_weight;
        }
    }

    let mut reconstruction_to_reference_sum = 0.0;
    let mut feature_reconstruction_to_reference_sum = 0.0;
    let mut candidate_weight_sum = 0.0;
    let mut candidate_hits = 0usize;
    let mut feature_candidate_hits = 0.0;

    for point in reconstruction {
        let (distance, nearest_reference_index) =
            nearest_reference(point.position_m, reference);
        let importance_weight = reference[nearest_reference_index].importance_weight;

        reconstruction_to_reference_sum += distance;
        feature_reconstruction_to_reference_sum += distance * importance_weight;
        candidate_weight_sum += importance_weight;

        if distance <= tolerance_m {
            candidate_hits += 1;
            feature_candidate_hits += importance_weight;
        }
    }

    let reference_count = reference.len() as f64;
    let reconstruction_count = reconstruction.len() as f64;

    let reference_to_reconstruction_mean_m =
        reference_to_reconstruction_sum / reference_count;
    let reconstruction_to_reference_mean_m =
        reconstruction_to_reference_sum / reconstruction_count;
    let feature_reference_to_reconstruction_mean_m =
        feature_reference_to_reconstruction_sum / reference_weight_sum;
    let feature_reconstruction_to_reference_mean_m =
        feature_reconstruction_to_reference_sum / candidate_weight_sum;

    let global_precision = candidate_hits as f64 / reconstruction_count;
    let global_recall = reference_hits as f64 / reference_count;
    let feature_precision = feature_candidate_hits / candidate_weight_sum;
    let feature_recall = feature_reference_hits / reference_weight_sum;

    Ok(SpatialFidelityMetrics {
        reference_to_reconstruction_mean_m,
        reconstruction_to_reference_mean_m,
        symmetric_chamfer_m: (reference_to_reconstruction_mean_m
            + reconstruction_to_reference_mean_m)
            / 2.0,
        feature_reference_to_reconstruction_mean_m,
        feature_reconstruction_to_reference_mean_m,
        feature_weighted_symmetric_chamfer_m: (feature_reference_to_reconstruction_mean_m
            + feature_reconstruction_to_reference_mean_m)
            / 2.0,
        global_precision_at_tolerance: global_precision,
        global_recall_at_tolerance: global_recall,
        global_f1_at_tolerance: f1(global_precision, global_recall),
        feature_precision_at_tolerance: feature_precision,
        feature_recall_at_tolerance: feature_recall,
        feature_f1_at_tolerance: f1(feature_precision, feature_recall),
        tolerance_m,
        reference_point_count: reference.len(),
        reconstruction_point_count: reconstruction.len(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_clouds_have_zero_error_and_perfect_scores() {
        let reference = [
            ReferenceSample::new([0.0, 0.0, 0.0], 1.0),
            ReferenceSample::new([1.0, 0.0, 0.0], 5.0),
            ReferenceSample::new([1.0, 1.0, 0.0], 2.0),
        ];
        let reconstruction = [
            ReconstructedSample::new([0.0, 0.0, 0.0]),
            ReconstructedSample::new([1.0, 0.0, 0.0]),
            ReconstructedSample::new([1.0, 1.0, 0.0]),
        ];

        let metrics =
            evaluate_spatial_fidelity(&reference, &reconstruction, 0.001).expect("valid clouds");

        assert_eq!(metrics.symmetric_chamfer_m, 0.0);
        assert_eq!(metrics.feature_weighted_symmetric_chamfer_m, 0.0);
        assert_eq!(metrics.global_f1_at_tolerance, 1.0);
        assert_eq!(metrics.feature_f1_at_tolerance, 1.0);
    }

    #[test]
    fn important_inscription_loss_is_not_hidden_by_broad_surface_accuracy() {
        let mut reference = Vec::new();
        let mut reconstruction = Vec::new();

        for index in 0..99 {
            let point = [index as f64, 0.0, 0.0];
            reference.push(ReferenceSample::new(point, 1.0));
            reconstruction.push(ReconstructedSample::new(point));
        }

        // A semantically critical inscription point is displaced by one metre.
        reference.push(ReferenceSample::new([1_000.0, 0.0, 0.0], 100.0));
        reconstruction.push(ReconstructedSample::new([1_000.0, 0.0, 1.0]));

        let metrics =
            evaluate_spatial_fidelity(&reference, &reconstruction, 0.1).expect("valid clouds");

        assert!(metrics.symmetric_chamfer_m < 0.02);
        assert!(metrics.feature_weighted_symmetric_chamfer_m > 0.5);
        assert!(metrics.feature_weighted_symmetric_chamfer_m > metrics.symmetric_chamfer_m * 20.0);
        assert!(metrics.global_f1_at_tolerance > 0.98);
        assert!(metrics.feature_f1_at_tolerance < 0.51);
    }

    #[test]
    fn extra_reconstructed_surface_reduces_precision_without_reducing_reference_recall() {
        let reference = [
            ReferenceSample::new([0.0, 0.0, 0.0], 1.0),
            ReferenceSample::new([1.0, 0.0, 0.0], 1.0),
        ];
        let reconstruction = [
            ReconstructedSample::new([0.0, 0.0, 0.0]),
            ReconstructedSample::new([1.0, 0.0, 0.0]),
            ReconstructedSample::new([4.0, 0.0, 0.0]),
        ];

        let metrics =
            evaluate_spatial_fidelity(&reference, &reconstruction, 0.01).expect("valid clouds");

        assert_eq!(metrics.global_recall_at_tolerance, 1.0);
        assert!((metrics.global_precision_at_tolerance - 2.0 / 3.0).abs() < 1e-12);
        assert!((metrics.feature_precision_at_tolerance - 2.0 / 3.0).abs() < 1e-12);
    }

    #[test]
    fn results_are_deterministic_for_frozen_inputs() {
        let reference = [
            ReferenceSample::new([0.0, 0.0, 0.0], 2.0),
            ReferenceSample::new([0.0, 1.0, 0.0], 3.0),
        ];
        let reconstruction = [
            ReconstructedSample::new([0.01, 0.0, 0.0]),
            ReconstructedSample::new([0.0, 1.02, 0.0]),
        ];

        let first =
            evaluate_spatial_fidelity(&reference, &reconstruction, 0.05).expect("valid clouds");
        let second =
            evaluate_spatial_fidelity(&reference, &reconstruction, 0.05).expect("valid clouds");

        assert_eq!(first, second);
    }

    #[test]
    fn rejects_empty_clouds_and_invalid_tolerances() {
        let reference = [ReferenceSample::new([0.0, 0.0, 0.0], 1.0)];
        let reconstruction = [ReconstructedSample::new([0.0, 0.0, 0.0])];

        assert_eq!(
            evaluate_spatial_fidelity(&[], &reconstruction, 0.1),
            Err(SpatialFidelityError::EmptyReference)
        );
        assert_eq!(
            evaluate_spatial_fidelity(&reference, &[], 0.1),
            Err(SpatialFidelityError::EmptyReconstruction)
        );
        assert_eq!(
            evaluate_spatial_fidelity(&reference, &reconstruction, 0.0),
            Err(SpatialFidelityError::InvalidTolerance)
        );
        assert_eq!(
            evaluate_spatial_fidelity(&reference, &reconstruction, f64::INFINITY),
            Err(SpatialFidelityError::InvalidTolerance)
        );
    }

    #[test]
    fn rejects_non_finite_coordinates_and_non_positive_feature_weights() {
        let reconstruction = [ReconstructedSample::new([0.0, 0.0, 0.0])];

        assert_eq!(
            evaluate_spatial_fidelity(
                &[ReferenceSample::new([f64::NAN, 0.0, 0.0], 1.0)],
                &reconstruction,
                0.1
            ),
            Err(SpatialFidelityError::InvalidReferenceCoordinate {
                point: 0,
                axis: 0
            })
        );
        assert_eq!(
            evaluate_spatial_fidelity(
                &[ReferenceSample::new([0.0, 0.0, 0.0], 0.0)],
                &reconstruction,
                0.1
            ),
            Err(SpatialFidelityError::InvalidImportanceWeight { point: 0 })
        );
        assert_eq!(
            evaluate_spatial_fidelity(
                &[ReferenceSample::new([0.0, 0.0, 0.0], f64::NAN)],
                &reconstruction,
                0.1
            ),
            Err(SpatialFidelityError::InvalidImportanceWeight { point: 0 })
        );
        assert_eq!(
            evaluate_spatial_fidelity(
                &[ReferenceSample::new([0.0, 0.0, 0.0], 1.0)],
                &[ReconstructedSample::new([0.0, f64::INFINITY, 0.0])],
                0.1
            ),
            Err(SpatialFidelityError::InvalidReconstructionCoordinate {
                point: 0,
                axis: 1
            })
        );
    }
}
