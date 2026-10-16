// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Small, dependency-free observability diagnostics for linearized inverse problems.
//!
//! This diagnostic is not a geodynamo solver. It analyzes a supplied local
//! observation Jacobian after explicit parameter and measurement scaling. It
//! cannot establish global identifiability for a nonlinear or misspecified model.

use serde::{Deserialize, Serialize};

/// Stable scientific identity for one local-linearization analysis. The manifest
/// must resolve these references to immutable, versioned inputs; this module does
/// not authenticate external identifiers or content digests.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ObservabilityContext {
    pub analysis_id: String,
    pub model_id: String,
    pub model_version: String,
    pub linearization_id: String,
    pub input_manifest_id: String,
}

/// A linearized map from dimensioned parameters to predicted observations.
///
/// jacobian[i][j] = d(predicted observation i) / d(parameter j).
/// Parameter scales are meaningful physical increments; observation scales are
/// measurement-error or normalization scales. The analyzed dimensionless matrix
/// is J[i][j] * parameter_scale[j] / observation_scale[i]. Poor scale choices
/// can mislead and therefore remain part of the report.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinearizedObservationOperator {
    pub context: ObservabilityContext,
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
    pub context: ObservabilityContext,
    pub absolute_tolerance: f64,
    pub relative_tolerance: f64,
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
    SvdNoConvergence,
}

impl LinearizedObservationOperator {
    /// Analyze local identifiability with a one-sided Jacobi SVD.
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

        // One-sided Jacobi works on the scaled Jacobian directly instead of
        // forming JᵀJ, avoiding unnecessary loss of precision from squaring
        // the condition number in this low-dimensional diagnostic.
        let (singular_values_unsorted, right_vectors) = one_sided_jacobi_svd(normalized)?;
        let mut order: Vec<usize> = (0..parameter_dimension).collect();
        order.sort_by(|&left, &right| {
            singular_values_unsorted[right].total_cmp(&singular_values_unsorted[left])
        });

        let singular_values: Vec<f64> = order
            .iter()
            .map(|&index| singular_values_unsorted[index])
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
            .map(|(_, &right_vector_index)| {
                (0..parameter_dimension)
                    .map(|parameter_index| right_vectors[parameter_index][right_vector_index])
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
            context: self.context.clone(),
            absolute_tolerance,
            relative_tolerance,
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
            || relative_tolerance >= 1.0
        {
            return Err(ObservabilityError::InvalidTolerance);
        }

        validate_context(&self.context)?;
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

fn validate_context(context: &ObservabilityContext) -> Result<(), ObservabilityError> {
    for (field, value) in [
        ("context.analysis_id", &context.analysis_id),
        ("context.model_id", &context.model_id),
        ("context.model_version", &context.model_version),
        ("context.linearization_id", &context.linearization_id),
        ("context.input_manifest_id", &context.input_manifest_id),
    ] {
        if value.trim().is_empty() {
            return Err(ObservabilityError::EmptyIdentifier(field));
        }
    }
    Ok(())
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

/// One-sided Jacobi SVD for small rectangular matrices.
/// The input matrix is rotated by columns until they are mutually orthogonal;
/// the resulting column norms are singular values and the accumulated rotation
/// matrix contains the right singular vectors. This avoids explicitly forming
/// JᵀJ, which squares the condition number. Large inversions still need a
/// domain-reviewed numerical linear algebra backend.
fn one_sided_jacobi_svd(
    mut matrix: Vec<Vec<f64>>,
) -> Result<(Vec<f64>, Vec<Vec<f64>>), ObservabilityError> {
    let row_count = matrix.len();
    let column_count = matrix.first().map_or(0, Vec::len);
    if row_count == 0 || column_count == 0 {
        return Err(ObservabilityError::EmptyParameters);
    }
    let mut right_vectors = vec![vec![0.0_f64; column_count]; column_count];
    for i in 0..column_count {
        right_vectors[i][i] = 1.0;
    }

    let max_sweeps = column_count.saturating_mul(100).max(1);
    for _ in 0..max_sweeps {
        let mut rotated_in_sweep = false;

        for p in 0..column_count {
            for q in (p + 1)..column_count {
                let mut alpha = 0.0_f64;
                let mut beta = 0.0_f64;
                let mut gamma = 0.0_f64;
                for row in &matrix {
                    alpha += row[p] * row[p];
                    beta += row[q] * row[q];
                    gamma += row[p] * row[q];
                }
                if !alpha.is_finite() || !beta.is_finite() || !gamma.is_finite() {
                    return Err(ObservabilityError::NonFiniteIntermediate);
                }
                if alpha == 0.0 || beta == 0.0 {
                    continue;
                }
                let correlation_bound = f64::EPSILON * 16.0 * alpha.sqrt() * beta.sqrt();
                if !correlation_bound.is_finite() {
                    return Err(ObservabilityError::NonFiniteIntermediate);
                }
                if gamma.abs() <= correlation_bound {
                    continue;
                }

                // Stable form of the Jacobi tangent; avoids tau*tau overflow.
                let delta = (beta - alpha) * 0.5;
                let tangent = if delta == 0.0 {
                    1.0
                } else {
                    let denominator = delta.abs() + delta.hypot(gamma);
                    if !denominator.is_finite() || denominator == 0.0 {
                        return Err(ObservabilityError::NonFiniteIntermediate);
                    }
                    delta.signum() * gamma.signum() * (gamma.abs() / denominator)
                };
                let cosine = 1.0 / (1.0 + tangent * tangent).sqrt();
                let sine = tangent * cosine;

                for row in &mut matrix {
                    let old_p = row[p];
                    let old_q = row[q];
                    row[p] = cosine * old_p - sine * old_q;
                    row[q] = sine * old_p + cosine * old_q;
                    if !row[p].is_finite() || !row[q].is_finite() {
                        return Err(ObservabilityError::NonFiniteIntermediate);
                    }
                }
                for row in &mut right_vectors {
                    let old_p = row[p];
                    let old_q = row[q];
                    row[p] = cosine * old_p - sine * old_q;
                    row[q] = sine * old_p + cosine * old_q;
                }
                rotated_in_sweep = true;
            }
        }

        if !rotated_in_sweep {
            let mut singular_values = Vec::with_capacity(column_count);
            for column in 0..column_count {
                let mut squared_norm = 0.0_f64;
                for row in &matrix {
                    squared_norm += row[column] * row[column];
                }
                if !squared_norm.is_finite() {
                    return Err(ObservabilityError::NonFiniteIntermediate);
                }
                singular_values.push(squared_norm.sqrt());
            }
            return Ok((singular_values, right_vectors));
        }
    }

    Err(ObservabilityError::SvdNoConvergence)
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
            context: ObservabilityContext {
                analysis_id: "analysis-test-001".into(),
                model_id: "synthetic-linear-model".into(),
                model_version: "1.0.0".into(),
                linearization_id: "jacobian-fixture-001".into(),
                input_manifest_id: "fixture-manifest-001".into(),
            },
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
        assert!((report.condition_number.unwrap() - 100.0).abs() < 1e-8);
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
        assert_eq!(
            op.analyze(1e-12, 1.0),
            Err(ObservabilityError::InvalidTolerance)
        );
    }

    #[test]
    fn report_preserves_context_and_tolerances() {
        let mut op = operator(vec![vec![1.0]]);
        op.context.analysis_id = "survey-window-a".into();
        op.context.input_manifest_id = "manifest-sha256-abc123".into();
        let report = op.analyze(1e-9, 1e-7).unwrap();
        assert_eq!(report.context, op.context);
        assert_eq!(report.absolute_tolerance, 1e-9);
        assert_eq!(report.relative_tolerance, 1e-7);
    }

    #[test]
    fn rejects_missing_experiment_identity() {
        let mut op = operator(vec![vec![1.0]]);
        op.context.linearization_id = "  ".into();
        assert_eq!(
            op.analyze(1e-12, 1e-10),
            Err(ObservabilityError::EmptyIdentifier("context.linearization_id"))
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
