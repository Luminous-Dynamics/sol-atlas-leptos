// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Small, dependency-free observability diagnostics for linearized inverse problems.
//!
//! This diagnostic is not a geodynamo solver. It analyzes a supplied local
//! observation Jacobian after explicit parameter and measurement scaling. It
//! cannot establish global identifiability for a nonlinear or misspecified model.

use serde::{Deserialize, Serialize};

/// A linearized map from dimensioned parameters to predicted observations.
///
/// jacobian[i][j] = d(predicted observation i) / d(parameter j).
/// Parameter scales are meaningful physical increments; observation scales are
/// measurement-error or normalization scales. The analyzed dimensionless matrix
/// is J[i][j] * parameter_scale[j] / observation_scale[i]. Poor scale choices
/// can mislead and therefore remain part of the report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinearizedObservationOperator {
    pub observation_ids: Vec<String>,
    pub parameter_ids: Vec<String>,
    pub jacobian: Vec<Vec<f64>>,
    pub observation_scales: Vec<f64>,
    pub parameter_scales: Vec<f64>,
}

/// Numerical description of locally observable and weakly/unobservable directions.
///
/// Each vector in null_space_basis is in parameter_ids order and has unit
/// Euclidean norm in scaled parameter coordinates. Signs are arbitrary. The
/// result applies only to the supplied local Jacobian and declared scaling.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ObservabilityReport {
    pub observation_count: usize,
    pub parameter_ids: Vec<String>,
    pub parameter_scales: Vec<f64>,
    pub observation_scales: Vec<f64>,
    /// Singular values of the dimensionless, error-scaled local Jacobian, descending.
    pub singular_values: Vec<f64>,
    pub numerical_rank: usize,
    pub parameter_dimension: usize,
    pub unobservable_dimension: usize,
    pub rank_tolerance: f64,
    /// Absent unless the operator has full column rank.
    pub condition_number: Option<f64>,
    pub null_space_basis: Vec<Vec<f64>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObservabilityError {
    EmptyObservations,
    EmptyParameters,
    ObservationIdCountMismatch,
    JacobianColumnCountMismatch,
    ScaleCountMismatch,
    EmptyIdentifier(&'static str),
    DuplicateIdentifier(&'static str),
    InvalidScale(&'static str),
    InvalidTolerance,
    NonFiniteInput(&'static str),
    NonFiniteIntermediate,
    EigenSolverNoConvergence,
}

impl LinearizedObservationOperator {
    /// Analyze local identifiability with deterministic Jacobi diagonalization of JᵀJ.
    ///
    /// Singular values are sorted descending. Directions below max(abs_tol,
    /// rel_tol * largest_singular_value) are classed as unobservable. Tolerances,
    /// scaling, Jacobian provenance, and the local model must be kept in the
    /// experiment manifest. This diagnostic is not a substitute for SVD-backed,
    /// domain-validated inverse methods on large or ill-conditioned systems.
    pub fn analyze(
        &self,
        absolute_tolerance: f64,
        relative_tolerance: f64,
    ) -> Result<ObservabilityReport, ObservabilityError> {
        self.validate(absolute_tolerance, relative_tolerance)?;

        let observation_count = self.jacobian.len();
        let parameter_dimension = self.parameter_ids.len();
        let mut normalized = vec![vec![0.0_f64; parameter_dimension]; observation_count];

        for (i, row) in self.jacobian.iter().enumerate() {
            for (j, derivative) in row.iter().enumerate() {
                let value = derivative * self.parameter_scales[j] / self.observation_scales[i];
                if !value.is_finite() {
                    return Err(ObservabilityError::NonFiniteIntermediate);
                }
                normalized[i][j] = value;
            }
        }

        let mut gram = vec![vec![0.0_f64; parameter_dimension]; parameter_dimension];
        for i in 0..parameter_dimension {
            for j in i..parameter_dimension {
                let mut sum = 0.0_f64;
                for row in &normalized {
                    sum += row[i] * row[j];
                    if !sum.is_finite() {
                        return Err(ObservabilityError::NonFiniteIntermediate);
                    }
                }
                gram[i][j] = sum;
                gram[j][i] = sum;
            }
        }

        let (eigenvalues, eigenvectors) = symmetric_eigen_jacobi(gram)?;
        let mut order: Vec<usize> = (0..parameter_dimension).collect();
        order.sort_by(|&left, &right| eigenvalues[right].total_cmp(&eigenvalues[left]));

        let singular_values: Vec<f64> = order
            .iter()
            .map(|&index| eigenvalues[index].max(0.0).sqrt())
            .collect();
        let largest = singular_values.first().copied().unwrap_or(0.0);
        let rank_tolerance = absolute_tolerance.max(relative_tolerance * largest);
        let numerical_rank = singular_values
            .iter()
            .take_while(|&&value| value > rank_tolerance)
            .count();
        let unobservable_dimension = parameter_dimension - numerical_rank;

        let null_space_basis = order
            .iter()
            .enumerate()
            .filter(|(ordered_index, _)| singular_values[*ordered_index] <= rank_tolerance)
            .map(|(_, &eigen_index)| {
                (0..parameter_dimension)
                    .map(|parameter_index| eigenvectors[parameter_index][eigen_index])
                    .collect::<Vec<f64>>()
            })
            .collect::<Vec<_>>();

        let condition_number = if numerical_rank == parameter_dimension {
            let smallest = singular_values[parameter_dimension - 1];
            (smallest > 0.0).then_some(largest / smallest)
        } else {
            None
        };

        Ok(ObservabilityReport {
            observation_count,
            parameter_ids: self.parameter_ids.clone(),
            parameter_scales: self.parameter_scales.clone(),
            observation_scales: self.observation_scales.clone(),
            singular_values,
            numerical_rank,
            parameter_dimension,
            unobservable_dimension,
            rank_tolerance,
            condition_number,
            null_space_basis,
        })
    }

    fn validate(
        &self,
        absolute_tolerance: f64,
        relative_tolerance: f64,
    ) -> Result<(), ObservabilityError> {
        if self.observation_ids.is_empty() {
            return Err(ObservabilityError::EmptyObservations);
        }
        if self.parameter_ids.is_empty() {
            return Err(ObservabilityError::EmptyParameters);
        }
        if self.observation_ids.len() != self.jacobian.len() {
            return Err(ObservabilityError::ObservationIdCountMismatch);
        }
        if self.parameter_ids.len() != self.parameter_scales.len()
            || self.observation_ids.len() != self.observation_scales.len()
        {
            return Err(ObservabilityError::ScaleCountMismatch);
        }
        if !absolute_tolerance.is_finite()
            || !relative_tolerance.is_finite()
            || absolute_tolerance <= 0.0
            || relative_tolerance <= 0.0
        {
            return Err(ObservabilityError::InvalidTolerance);
        }

        validate_identifiers(&self.observation_ids, "observation_ids")?;
        validate_identifiers(&self.parameter_ids, "parameter_ids")?;

        for scale in &self.parameter_scales {
            if !scale.is_finite() || *scale <= 0.0 {
                return Err(ObservabilityError::InvalidScale("parameter_scales"));
            }
        }
        for scale in &self.observation_scales {
            if !scale.is_finite() || *scale <= 0.0 {
                return Err(ObservabilityError::InvalidScale("observation_scales"));
            }
        }
        for row in &self.jacobian {
            if row.len() != self.parameter_ids.len() {
                return Err(ObservabilityError::JacobianColumnCountMismatch);
            }
            if row.iter().any(|value| !value.is_finite()) {
                return Err(ObservabilityError::NonFiniteInput("jacobian"));
            }
        }
        Ok(())
    }
}

fn validate_identifiers(
    identifiers: &[String],
    field: &'static str,
) -> Result<(), ObservabilityError> {
    for (index, identifier) in identifiers.iter().enumerate() {
        if identifier.trim().is_empty() {
            return Err(ObservabilityError::EmptyIdentifier(field));
        }
        if identifiers[..index].contains(identifier) {
            return Err(ObservabilityError::DuplicateIdentifier(field));
        }
    }
    Ok(())
}

/// Jacobi eigensolver for a small real symmetric matrix. Eigenvectors are columns.
/// This O(n³) routine is intended for modest parameter counts, not production
/// geodynamo inversions; large inversions should use a validated SVD backend.
fn symmetric_eigen_jacobi(
    mut matrix: Vec<Vec<f64>>,
) -> Result<(Vec<f64>, Vec<Vec<f64>>), ObservabilityError> {
    let dimension = matrix.len();
    let mut eigenvectors = vec![vec![0.0_f64; dimension]; dimension];
    for i in 0..dimension {
        eigenvectors[i][i] = 1.0;
    }

    if dimension == 1 {
        return Ok((vec![matrix[0][0]], eigenvectors));
    }

    let max_iterations = dimension
        .saturating_mul(dimension)
        .saturating_mul(100)
        .max(1);

    for _ in 0..max_iterations {
        let mut p = 0;
        let mut q = 1;
        let mut largest_off_diagonal = 0.0_f64;
        let mut diagonal_scale = 0.0_f64;
        for i in 0..dimension {
            diagonal_scale = diagonal_scale.max(matrix[i][i].abs());
            for j in (i + 1)..dimension {
                if matrix[i][j].abs() > largest_off_diagonal {
                    largest_off_diagonal = matrix[i][j].abs();
                    p = i;
                    q = j;
                }
            }
        }

        if largest_off_diagonal <= f64::EPSILON * 16.0 * diagonal_scale {
            return Ok(((0..dimension).map(|i| matrix[i][i]).collect(), eigenvectors));
        }
        if matrix[p][q] == 0.0 {
            continue;
        }

        let app = matrix[p][p];
        let aqq = matrix[q][q];
        let apq = matrix[p][q];
        let tau = (aqq - app) / (2.0 * apq);
        let t = if tau >= 0.0 {
            1.0 / (tau + (1.0 + tau * tau).sqrt())
        } else {
            -1.0 / (-tau + (1.0 + tau * tau).sqrt())
        };
        let cosine = 1.0 / (1.0 + t * t).sqrt();
        let sine = t * cosine;

        matrix[p][p] = app - t * apq;
        matrix[q][q] = aqq + t * apq;
        matrix[p][q] = 0.0;
        matrix[q][p] = 0.0;

        for k in 0..dimension {
            if k != p && k != q {
                let akp = matrix[k][p];
                let akq = matrix[k][q];
                let new_kp = cosine * akp - sine * akq;
                let new_kq = sine * akp + cosine * akq;
                matrix[k][p] = new_kp;
                matrix[p][k] = new_kp;
                matrix[k][q] = new_kq;
                matrix[q][k] = new_kq;
            }

            let vkp = eigenvectors[k][p];
            let vkq = eigenvectors[k][q];
            eigenvectors[k][p] = cosine * vkp - sine * vkq;
            eigenvectors[k][q] = sine * vkp + cosine * vkq;
        }
    }

    Err(ObservabilityError::EigenSolverNoConvergence)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn operator(jacobian: Vec<Vec<f64>>) -> LinearizedObservationOperator {
        let observation_ids = (0..jacobian.len())
            .map(|index| format!("obs-{index}"))
            .collect();
        let parameter_count = jacobian.first().map_or(0, Vec::len);
        let parameter_ids = (0..parameter_count)
            .map(|index| format!("parameter-{index}"))
            .collect();
        LinearizedObservationOperator {
            observation_scales: vec![1.0; observation_ids.len()],
            parameter_scales: vec![1.0; parameter_count],
            observation_ids,
            parameter_ids,
            jacobian,
        }
    }

    #[test]
    fn full_rank_operator_reports_finite_condition_number() {
        let report = operator(vec![vec![1.0, 0.0], vec![0.0, 2.0]])
            .analyze(1e-12, 1e-10)
            .unwrap();
        assert_eq!(report.numerical_rank, 2);
        assert_eq!(report.unobservable_dimension, 0);
        assert_eq!(report.null_space_basis.len(), 0);
        assert!(report.condition_number.unwrap().is_finite());
        assert!((report.condition_number.unwrap() - 2.0).abs() < 1e-8);
    }

    #[test]
    fn dependent_observations_expose_an_unobservable_direction() {
        let report = operator(vec![vec![1.0, 1.0], vec![2.0, 2.0]])
            .analyze(1e-10, 1e-9)
            .unwrap();
        assert_eq!(report.numerical_rank, 1);
        assert_eq!(report.unobservable_dimension, 1);
        assert_eq!(report.condition_number, None);
        assert_eq!(report.null_space_basis.len(), 1);
        let direction = &report.null_space_basis[0];
        assert!((direction[0] + direction[1]).abs() < 1e-6);
        assert!((direction[0].powi(2) + direction[1].powi(2) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn explicit_scales_define_dimensionless_observability() {
        let mut op = operator(vec![vec![1.0, 0.0], vec![0.0, 1.0]]);
        op.observation_scales = vec![1.0, 100.0];
        op.parameter_scales = vec![10.0, 10.0];
        let report = op.analyze(1e-12, 1e-10).unwrap();
        assert_eq!(report.numerical_rank, 2);
        assert!((report.condition_number.unwrap() - 10.0).abs() < 1e-8);
        assert_eq!(report.parameter_scales, vec![10.0, 10.0]);
    }

    #[test]
    fn rejects_ragged_jacobians() {
        let report = operator(vec![vec![1.0, 0.0], vec![1.0]])
            .analyze(1e-12, 1e-10);
        assert_eq!(report, Err(ObservabilityError::JacobianColumnCountMismatch));
    }

    #[test]
    fn rejects_non_finite_jacobian_values() {
        let report = operator(vec![vec![f64::NAN]])
            .analyze(1e-12, 1e-10);
        assert_eq!(report, Err(ObservabilityError::NonFiniteInput("jacobian")));
    }

    #[test]
    fn rejects_non_positive_scales_and_invalid_tolerance() {
        let mut op = operator(vec![vec![1.0]]);
        op.parameter_scales[0] = 0.0;
        assert_eq!(
            op.analyze(1e-12, 1e-10),
            Err(ObservabilityError::InvalidScale("parameter_scales"))
        );

        let mut op = operator(vec![vec![1.0]]);
        op.observation_scales[0] = 0.0;
        assert_eq!(
            op.analyze(1e-12, 1e-10),
            Err(ObservabilityError::InvalidScale("observation_scales"))
        );

        let op = operator(vec![vec![1.0]]);
        assert_eq!(
            op.analyze(0.0, 1e-10),
            Err(ObservabilityError::InvalidTolerance)
        );
    }

    #[test]
    fn repeated_analysis_is_deterministic() {
        let op = operator(vec![
            vec![1.0, 2.0, 0.0],
            vec![2.0, 4.0, 1.0],
            vec![3.0, 6.0, 1.0],
        ]);
        let first = op.analyze(1e-10, 1e-8).unwrap();
        let second = op.analyze(1e-10, 1e-8).unwrap();
        assert_eq!(first, second);
        assert_eq!(first.numerical_rank, 2);
    }
}
