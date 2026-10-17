// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Small dense linear-Gaussian resolution diagnostics for inverse problems.
//!
//! The resolution matrix answers a different question than posterior variance:
//! it measures how a perturbation in the true state is mapped into the expected
//! estimate under the declared linear observation operator, covariance, and
//! prior. A strong prior can make the posterior look precise while leaving a
//! weakly observed mode with a diagonal resolution near zero.
//!
//! This module is intentionally dependency-light and intended for small,
//! well-conditioned diagnostics and exact benchmark fixtures. It is not a
//! geodynamo solver, a global-identifiability proof, or a replacement for a
//! reviewed numerical linear algebra backend on large/ill-conditioned systems.

use crate::observability::ObservabilityContext;
use serde::{Deserialize, Serialize};

/// A linear-Gaussian inverse problem in explicitly ordered parameter and
/// observation coordinates.
///
/// The operator has shape observations x parameters. Prior covariance has shape
/// parameters x parameters; observation covariance has shape observations x
/// observations. Covariances may be non-diagonal, so correlated errors and
/// cross-parameter priors are preserved.
///
/// Matrices must use consistent, declared coordinates and units. Record unit
/// scales and all matrix/source identities in the input manifest referenced by
/// context; this module does not authenticate that manifest.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LinearGaussianResolutionProblem {
    pub context: ObservabilityContext,
    pub observation_ids: Vec<String>,
    pub parameter_ids: Vec<String>,
    pub observation_operator: Vec<Vec<f64>>,
    pub prior_covariance: Vec<Vec<f64>>,
    pub observation_covariance: Vec<Vec<f64>>,
}

/// Numerical descriptions of prior-conditioned data resolution.
///
/// resolution_matrix[i][j] describes the expected response of estimated
/// parameter i to a unit perturbation of true parameter j under the declared
/// linear-Gaussian estimator. Diagonal values close to one indicate a mode
/// recovered with little attenuation in these coordinates; values near zero
/// indicate weak data resolution. Off-diagonal values indicate cross-talk.
///
/// These interpretations are model- and coordinate-conditional. Do not turn a
/// single threshold on the diagonal into a universal physical resolution claim.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResolutionReport {
    pub context: ObservabilityContext,
    pub observation_count: usize,
    pub parameter_ids: Vec<String>,
    pub posterior_covariance: Vec<Vec<f64>>,
    pub resolution_matrix: Vec<Vec<f64>>,
    pub diagonal_resolution: Vec<f64>,
    /// L2 norm of off-diagonal values in each resolution-matrix row.
    pub off_diagonal_cross_talk_norm: Vec<f64>,
    /// Trace of the resolution matrix (degrees of freedom for signal).
    pub degrees_of_freedom_for_signal: f64,
    pub symmetry_tolerance: f64,
    pub relative_pivot_tolerance: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResolutionError {
    EmptyObservations,
    EmptyParameters,
    ObservationIdCountMismatch,
    OperatorColumnCountMismatch,
    PriorCovarianceShapeMismatch,
    ObservationCovarianceShapeMismatch,
    EmptyIdentifier(&'static str),
    DuplicateIdentifier(&'static str),
    InvalidTolerance,
    NonFiniteInput(&'static str),
    NonFiniteIntermediate,
    AsymmetricMatrix(&'static str),
    NotPositiveDefinite(&'static str),
    SingularOrIllConditioned(&'static str),
}

/// Calculate the exact linear-Gaussian posterior covariance and data-resolution
/// matrix for a small dense problem.
///
/// Let H be the observation operator, Sa the prior covariance, and Se the
/// measurement covariance. Then:
///
///   Sp = inverse(Hᵀ inverse(Se) H + inverse(Sa))
///   A  = Sp Hᵀ inverse(Se) H
///
/// The implementation uses Cholesky factorizations and triangular solves,
/// whitens parameter coordinates through the prior Cholesky factor, and avoids
/// explicitly constructing inverses of the input covariance matrices. A dense
/// normal-equation information matrix is still formed; for large or ill-conditioned
/// problems use a reviewed numerical backend and compare against this reference
/// only on small qualification fixtures.
impl LinearGaussianResolutionProblem {
    pub fn analyze(
        &self,
        symmetry_tolerance: f64,
        relative_pivot_tolerance: f64,
    ) -> Result<ResolutionReport, ResolutionError> {
        self.validate(symmetry_tolerance, relative_pivot_tolerance)?;

        let m = self.observation_operator.len();
        let n = self.parameter_ids.len();

        let prior_cholesky = cholesky(
            &self.prior_covariance,
            "prior_covariance",
            symmetry_tolerance,
            relative_pivot_tolerance,
        )?;
        let observation_cholesky = cholesky(
            &self.observation_covariance,
            "observation_covariance",
            symmetry_tolerance,
            relative_pivot_tolerance,
        )?;

        // Solve Se * W = H, then form the noise-weighted data information
        // HᵀW. Full Se is used; correlated measurement errors are not discarded.
        let weighted_operator = solve_cholesky(&observation_cholesky, &self.observation_operator)?;
        let information = multiply(&transpose(&self.observation_operator), &weighted_operator)?;

        // Whiten the parameters with Sa = L Lᵀ. In whitened coordinates the
        // posterior precision is I + Lᵀ Hᵀ Se⁻¹ H L. This avoids explicitly
        // constructing the inverse of the prior covariance and is better
        // scaled when the original parameters have different prior variances.
        let information_times_prior = multiply(&information, &prior_cholesky)?;
        let whitened_information = multiply(
            &transpose(&prior_cholesky),
            &information_times_prior,
        )?;
        let mut identity = vec![vec![0.0_f64; n]; n];
        for (i, row) in identity.iter_mut().enumerate() {
            row[i] = 1.0;
        }
        let posterior_whitened_precision = add(&identity, &whitened_information)?;
        let posterior_whitened_cholesky = cholesky(
            &posterior_whitened_precision,
            "posterior_whitened_precision",
            symmetry_tolerance,
            relative_pivot_tolerance,
        )?;
        let posterior_whitened_covariance =
            inverse_from_cholesky(&posterior_whitened_cholesky)?;
        let posterior_covariance = multiply(
            &prior_cholesky,
            &multiply(
                &posterior_whitened_covariance,
                &transpose(&prior_cholesky),
            )?,
        )?;

        // A = Sp * Hᵀ * Se⁻¹ * H. Its off-diagonal elements measure leakage
        // between parameter modes. Store both diagonals and a row-wise leakage
        // norm so a diagonal-only visualization cannot hide cross-talk.
        let resolution_matrix = multiply(&posterior_covariance, &information)?;
        let mut diagonal_resolution = Vec::with_capacity(n);
        let mut off_diagonal_cross_talk_norm = Vec::with_capacity(n);
        let mut degrees_of_freedom_for_signal = 0.0;

        for i in 0..n {
            let diagonal = resolution_matrix[i][i];
            if !diagonal.is_finite() {
                return Err(ResolutionError::NonFiniteIntermediate);
            }
            diagonal_resolution.push(diagonal);
            degrees_of_freedom_for_signal += diagonal;

            let mut squared_cross_talk = 0.0;
            for j in 0..n {
                if i != j {
                    squared_cross_talk += resolution_matrix[i][j] * resolution_matrix[i][j];
                }
            }
            let norm = squared_cross_talk.sqrt();
            if !norm.is_finite() {
                return Err(ResolutionError::NonFiniteIntermediate);
            }
            off_diagonal_cross_talk_norm.push(norm);
        }

        if !degrees_of_freedom_for_signal.is_finite()
            || posterior_covariance.iter().flatten().any(|x| !x.is_finite())
            || resolution_matrix.iter().flatten().any(|x| !x.is_finite())
        {
            return Err(ResolutionError::NonFiniteIntermediate);
        }

        Ok(ResolutionReport {
            context: self.context.clone(),
            observation_count: m,
            parameter_ids: self.parameter_ids.clone(),
            posterior_covariance,
            resolution_matrix,
            diagonal_resolution,
            off_diagonal_cross_talk_norm,
            degrees_of_freedom_for_signal,
            symmetry_tolerance,
            relative_pivot_tolerance,
        })
    }

    fn validate(
        &self,
        symmetry_tolerance: f64,
        relative_pivot_tolerance: f64,
    ) -> Result<(), ResolutionError> {
        if self.observation_ids.is_empty() {
            return Err(ResolutionError::EmptyObservations);
        }
        if self.parameter_ids.is_empty() {
            return Err(ResolutionError::EmptyParameters);
        }
        if self.observation_operator.len() != self.observation_ids.len() {
            return Err(ResolutionError::ObservationIdCountMismatch);
        }
        if !symmetry_tolerance.is_finite()
            || !relative_pivot_tolerance.is_finite()
            || symmetry_tolerance <= 0.0
            || symmetry_tolerance >= 1.0
            || relative_pivot_tolerance <= 0.0
            || relative_pivot_tolerance >= 1.0
        {
            return Err(ResolutionError::InvalidTolerance);
        }

        for (field, value) in [
            ("context.analysis_id", &self.context.analysis_id),
            ("context.model_id", &self.context.model_id),
            ("context.model_version", &self.context.model_version),
            ("context.linearization_id", &self.context.linearization_id),
            ("context.input_manifest_id", &self.context.input_manifest_id),
        ] {
            if value.trim().is_empty() {
                return Err(ResolutionError::EmptyIdentifier(field));
            }
        }
        validate_identifiers(&self.observation_ids, "observation_ids")?;
        validate_identifiers(&self.parameter_ids, "parameter_ids")?;

        let m = self.observation_ids.len();
        let n = self.parameter_ids.len();
        if self.prior_covariance.len() != n
            || self.prior_covariance.iter().any(|row| row.len() != n)
        {
            return Err(ResolutionError::PriorCovarianceShapeMismatch);
        }
        if self.observation_covariance.len() != m
            || self.observation_covariance.iter().any(|row| row.len() != m)
        {
            return Err(ResolutionError::ObservationCovarianceShapeMismatch);
        }
        if self.observation_operator.iter().any(|row| row.len() != n) {
            return Err(ResolutionError::OperatorColumnCountMismatch);
        }

        if self.observation_operator.iter().flatten().any(|x| !x.is_finite()) {
            return Err(ResolutionError::NonFiniteInput("observation_operator"));
        }
        if self.prior_covariance.iter().flatten().any(|x| !x.is_finite()) {
            return Err(ResolutionError::NonFiniteInput("prior_covariance"));
        }
        if self.observation_covariance.iter().flatten().any(|x| !x.is_finite()) {
            return Err(ResolutionError::NonFiniteInput("observation_covariance"));
        }
        Ok(())
    }
}

fn validate_identifiers(
    identifiers: &[String],
    field: &'static str,
) -> Result<(), ResolutionError> {
    for (index, identifier) in identifiers.iter().enumerate() {
        if identifier.trim().is_empty() {
            return Err(ResolutionError::EmptyIdentifier(field));
        }
        if identifiers[..index].contains(identifier) {
            return Err(ResolutionError::DuplicateIdentifier(field));
        }
    }
    Ok(())
}

fn cholesky(
    matrix: &[Vec<f64>],
    label: &'static str,
    symmetry_tolerance: f64,
    relative_pivot_tolerance: f64,
) -> Result<Vec<Vec<f64>>, ResolutionError> {
    let n = matrix.len();
    if n == 0 || matrix.iter().any(|row| row.len() != n) {
        return Err(ResolutionError::SingularOrIllConditioned(label));
    }

    let mut scale = 0.0_f64;
    for i in 0..n {
        scale = scale.max(matrix[i][i].abs());
    }
    if !scale.is_finite() || scale <= 0.0 {
        return Err(ResolutionError::NotPositiveDefinite(label));
    }
    for i in 0..n {
        for j in 0..i {
            let allowed = symmetry_tolerance * scale;
            if (matrix[i][j] - matrix[j][i]).abs() > allowed {
                return Err(ResolutionError::AsymmetricMatrix(label));
            }
        }
    }

    let pivot_floor = relative_pivot_tolerance * scale;
    let mut lower = vec![vec![0.0_f64; n]; n];

    for i in 0..n {
        for j in 0..=i {
            let mut sum = matrix[i][j];
            for k in 0..j {
                sum -= lower[i][k] * lower[j][k];
            }
            if !sum.is_finite() {
                return Err(ResolutionError::NonFiniteIntermediate);
            }
            if i == j {
                if sum <= 0.0 {
                    return Err(ResolutionError::NotPositiveDefinite(label));
                }
                if sum <= pivot_floor {
                    return Err(ResolutionError::SingularOrIllConditioned(label));
                }
                lower[i][j] = sum.sqrt();
            } else {
                let pivot = lower[j][j];
                if !pivot.is_finite() || pivot <= 0.0 {
                    return Err(ResolutionError::SingularOrIllConditioned(label));
                }
                lower[i][j] = sum / pivot;
            }
        }
    }
    Ok(lower)
}

/// Solve L Lᵀ X = B using forward and backward substitution.
fn solve_cholesky(
    lower: &[Vec<f64>],
    rhs: &[Vec<f64>],
) -> Result<Vec<Vec<f64>>, ResolutionError> {
    let n = lower.len();
    if n == 0 || lower.iter().any(|row| row.len() != n) || rhs.len() != n {
        return Err(ResolutionError::SingularOrIllConditioned("linear_solve"));
    }
    let columns = rhs.first().map_or(0, Vec::len);
    if columns == 0 || rhs.iter().any(|row| row.len() != columns) {
        return Err(ResolutionError::SingularOrIllConditioned("linear_solve"));
    }

    let mut intermediate = vec![vec![0.0_f64; columns]; n];
    for i in 0..n {
        for c in 0..columns {
            let mut value = rhs[i][c];
            for k in 0..i {
                value -= lower[i][k] * intermediate[k][c];
            }
            intermediate[i][c] = value / lower[i][i];
            if !intermediate[i][c].is_finite() {
                return Err(ResolutionError::NonFiniteIntermediate);
            }
        }
    }

    let mut solution = vec![vec![0.0_f64; columns]; n];
    for i in (0..n).rev() {
        for c in 0..columns {
            let mut value = intermediate[i][c];
            for k in (i + 1)..n {
                value -= lower[k][i] * solution[k][c];
            }
            solution[i][c] = value / lower[i][i];
            if !solution[i][c].is_finite() {
                return Err(ResolutionError::NonFiniteIntermediate);
            }
        }
    }
    Ok(solution)
}

fn inverse_from_cholesky(lower: &[Vec<f64>]) -> Result<Vec<Vec<f64>>, ResolutionError> {
    let n = lower.len();
    let mut identity = vec![vec![0.0_f64; n]; n];
    for (i, row) in identity.iter_mut().enumerate() {
        row[i] = 1.0;
    }
    solve_cholesky(lower, &identity)
}

fn transpose(matrix: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let rows = matrix.len();
    let columns = matrix.first().map_or(0, Vec::len);
    let mut result = vec![vec![0.0_f64; rows]; columns];
    for i in 0..rows {
        for j in 0..columns {
            result[j][i] = matrix[i][j];
        }
    }
    result
}

fn multiply(left: &[Vec<f64>], right: &[Vec<f64>]) -> Result<Vec<Vec<f64>>, ResolutionError> {
    let left_columns = left.first().map_or(0, Vec::len);
    if left.is_empty()
        || right.is_empty()
        || left_columns != right.len()
        || right.iter().any(|row| row.len() != right[0].len())
    {
        return Err(ResolutionError::SingularOrIllConditioned("matrix_multiply"));
    }
    let right_columns = right[0].len();
    let mut result = vec![vec![0.0_f64; right_columns]; left.len()];
    for i in 0..left.len() {
        for k in 0..left_columns {
            for j in 0..right_columns {
                result[i][j] += left[i][k] * right[k][j];
            }
        }
    }
    if result.iter().flatten().any(|x| !x.is_finite()) {
        return Err(ResolutionError::NonFiniteIntermediate);
    }
    Ok(result)
}

fn add(left: &[Vec<f64>], right: &[Vec<f64>]) -> Result<Vec<Vec<f64>>, ResolutionError> {
    if left.is_empty()
        || left.len() != right.len()
        || left.iter().zip(right).any(|(a, b)| a.len() != b.len())
    {
        return Err(ResolutionError::SingularOrIllConditioned("matrix_add"));
    }
    let result = left
        .iter()
        .zip(right)
        .map(|(a, b)| a.iter().zip(b).map(|(x, y)| x + y).collect::<Vec<_>>())
        .collect::<Vec<_>>();
    if result.iter().flatten().any(|x| !x.is_finite()) {
        return Err(ResolutionError::NonFiniteIntermediate);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context() -> ObservabilityContext {
        ObservabilityContext {
            analysis_id: "resolution-fixture".into(),
            model_id: "linear-gaussian".into(),
            model_version: "1".into(),
            linearization_id: "exact-linear-operator".into(),
            input_manifest_id: "fixture-manifest-v1".into(),
        }
    }

    fn diagonal_problem() -> LinearGaussianResolutionProblem {
        LinearGaussianResolutionProblem {
            context: context(),
            observation_ids: vec!["low".into(), "high".into()],
            parameter_ids: vec!["low_mode".into(), "high_mode".into()],
            observation_operator: vec![vec![2.0, 0.0], vec![0.0, 0.1]],
            prior_covariance: vec![vec![1.0, 0.0], vec![0.0, 1.0]],
            observation_covariance: vec![vec![1.0, 0.0], vec![0.0, 1.0]],
        }
    }

    #[test]
    fn exact_fixture_distinguishes_data_resolution_from_posterior_precision() {
        let report = diagonal_problem().analyze(1e-12, 1e-14).unwrap();
        assert!((report.posterior_covariance[0][0] - 0.2).abs() < 1e-10);
        assert!((report.posterior_covariance[1][1] - 0.9900990099009901).abs() < 1e-10);
        assert!(report.posterior_covariance[0][1].abs() < 1e-10);
        assert!(report.posterior_covariance[1][0].abs() < 1e-10);
        assert!((report.resolution_matrix[0][0] - 0.8).abs() < 1e-10);
        assert!((report.resolution_matrix[1][1] - 0.009900990099009901).abs() < 1e-10);
        assert!(report.resolution_matrix[0][1].abs() < 1e-10);
        assert!(report.resolution_matrix[1][0].abs() < 1e-10);
        assert!((report.degrees_of_freedom_for_signal - 0.8099009900990099).abs() < 1e-10);
        assert!(report.diagonal_resolution[0] > 0.79);
        assert!(report.diagonal_resolution[1] < 0.02);
    }

    #[test]
    fn off_diagonal_information_reports_mode_cross_talk() {
        let mut problem = diagonal_problem();
        problem.observation_operator = vec![vec![1.0, 1.0], vec![0.5, -1.0]];
        let report = problem.analyze(1e-12, 1e-14).unwrap();
        assert!(report.resolution_matrix[0][1].abs() > 1e-4);
        assert!(report.resolution_matrix[1][0].abs() > 1e-4);
        assert!(report.off_diagonal_cross_talk_norm.iter().all(|x| *x > 1e-4));
    }

    #[test]
    fn correlated_observation_errors_are_preserved() {
        let mut problem = diagonal_problem();
        problem.observation_operator = vec![vec![1.0, 0.0], vec![1.0, 1.0]];
        problem.observation_covariance = vec![vec![1.0, 0.8], vec![0.8, 1.0]];
        let report = problem.analyze(1e-12, 1e-14).unwrap();
        assert!(report.posterior_covariance.iter().flatten().all(|x| x.is_finite()));
        assert!(report.resolution_matrix.iter().flatten().all(|x| x.is_finite()));

        // The correlated covariance must change the answer relative to a false
        // independent-noise assumption; checking finiteness alone is insufficient.
        let mut independent = problem.clone();
        independent.observation_covariance = vec![vec![1.0, 0.0], vec![0.0, 1.0]];
        let independent_report = independent.analyze(1e-12, 1e-14).unwrap();
        assert!(
            (report.resolution_matrix[0][0] - independent_report.resolution_matrix[0][0]).abs()
                > 1e-2
        );
    }

    #[test]
    fn correlated_prior_and_observation_covariances_are_both_respected() {
        let mut problem = diagonal_problem();
        problem.observation_operator = vec![vec![1.0, 0.0], vec![1.0, 1.0]];
        problem.prior_covariance = vec![vec![1.0, 0.2], vec![0.2, 2.0]];
        problem.observation_covariance = vec![vec![1.0, 0.8], vec![0.8, 1.0]];

        let report = problem.analyze(1e-12, 1e-14).unwrap();
        assert!((report.posterior_covariance[0][0] - 0.4833333333333333).abs() < 1e-10);
        assert!((report.posterior_covariance[0][1] + 0.0666666666666667).abs() < 1e-10);
        assert!((report.posterior_covariance[1][1] - 0.3133333333333333).abs() < 1e-10);
        assert!((report.resolution_matrix[0][0] - 0.5).abs() < 1e-10);
        assert!((report.resolution_matrix[0][1] - 0.0833333333333333).abs() < 1e-10);
        assert!((report.resolution_matrix[1][0] - 0.1).abs() < 1e-10);
        assert!((report.resolution_matrix[1][1] - 0.8333333333333333).abs() < 1e-10);
        assert!((report.degrees_of_freedom_for_signal - 1.3333333333333333).abs() < 1e-10);
    }

    #[test]
    fn a_tighter_prior_does_not_masquerade_as_more_data_resolution() {
        let baseline = diagonal_problem().analyze(1e-12, 1e-14).unwrap();

        let mut tight_prior = diagonal_problem();
        tight_prior.prior_covariance[1][1] = 0.01;
        let tight = tight_prior.analyze(1e-12, 1e-14).unwrap();

        // The prior makes the high-mode posterior variance much smaller, but
        // lowers the data-resolution response for that mode.
        assert!(
            tight.posterior_covariance[1][1] < baseline.posterior_covariance[1][1] / 50.0
        );
        assert!(
            tight.diagonal_resolution[1] < baseline.diagonal_resolution[1] / 50.0
        );
    }

    #[test]
    fn rejects_malformed_matrix_shapes() {
        let mut problem = diagonal_problem();
        problem.observation_operator[0].pop();
        assert_eq!(
            problem.analyze(1e-12, 1e-14),
            Err(ResolutionError::OperatorColumnCountMismatch)
        );

        let mut problem = diagonal_problem();
        problem.prior_covariance.pop();
        assert_eq!(
            problem.analyze(1e-12, 1e-14),
            Err(ResolutionError::PriorCovarianceShapeMismatch)
        );

        let mut problem = diagonal_problem();
        problem.observation_covariance[0].pop();
        assert_eq!(
            problem.analyze(1e-12, 1e-14),
            Err(ResolutionError::ObservationCovarianceShapeMismatch)
        );
    }

    #[test]
    fn rejects_non_finite_matrices_and_invalid_tolerances() {
        let mut problem = diagonal_problem();
        problem.observation_operator[0][0] = f64::NAN;
        assert_eq!(
            problem.analyze(1e-12, 1e-14),
            Err(ResolutionError::NonFiniteInput("observation_operator"))
        );
        assert_eq!(
            diagonal_problem().analyze(0.0, 1e-14),
            Err(ResolutionError::InvalidTolerance)
        );
        assert_eq!(
            diagonal_problem().analyze(1e-12, 1.0),
            Err(ResolutionError::InvalidTolerance)
        );
    }

    #[test]
    fn rejects_asymmetric_and_non_positive_definite_covariances() {
        let mut problem = diagonal_problem();
        problem.observation_covariance = vec![vec![1.0, 0.1], vec![0.0, 1.0]];
        assert_eq!(
            problem.analyze(1e-12, 1e-14),
            Err(ResolutionError::AsymmetricMatrix("observation_covariance"))
        );

        let mut problem = diagonal_problem();
        problem.prior_covariance = vec![vec![1.0, 2.0], vec![2.0, 1.0]];
        assert_eq!(
            problem.analyze(1e-12, 1e-14),
            Err(ResolutionError::NotPositiveDefinite("prior_covariance"))
        );
    }

    #[test]
    fn exact_zero_sensitivity_mode_remains_unresolved() {
        let mut problem = diagonal_problem();
        problem.observation_operator = vec![vec![1.0, 0.0], vec![0.0, 0.0]];
        let report = problem.analyze(1e-12, 1e-14).unwrap();

        assert!((report.posterior_covariance[0][0] - 0.5).abs() < 1e-10);
        assert!((report.posterior_covariance[1][1] - 1.0).abs() < 1e-10);
        assert!((report.diagonal_resolution[0] - 0.5).abs() < 1e-10);
        assert!(report.diagonal_resolution[1].abs() < 1e-10);
    }

    #[test]
    fn parameter_order_changes_only_the_ordered_report_coordinates() {
        let first = diagonal_problem().analyze(1e-12, 1e-14).unwrap();

        let mut swapped = diagonal_problem();
        swapped.parameter_ids.swap(0, 1);
        for row in &mut swapped.observation_operator {
            row.swap(0, 1);
        }
        swapped.prior_covariance = vec![vec![1.0, 0.0], vec![0.0, 1.0]];
        let second = swapped.analyze(1e-12, 1e-14).unwrap();

        assert_eq!(second.parameter_ids, vec!["high_mode", "low_mode"]);
        assert!((second.diagonal_resolution[0] - first.diagonal_resolution[1]).abs() < 1e-10);
        assert!((second.diagonal_resolution[1] - first.diagonal_resolution[0]).abs() < 1e-10);
    }

    #[test]
    fn duplicate_observation_ids_are_rejected() {
        let mut problem = diagonal_problem();
        problem.observation_ids[1] = "low".into();
        assert_eq!(
            problem.analyze(1e-12, 1e-14),
            Err(ResolutionError::DuplicateIdentifier("observation_ids"))
        );
    }

    #[test]
    fn preserves_experiment_context_and_parameter_order() {
        let report = diagonal_problem().analyze(1e-12, 1e-14).unwrap();
        assert_eq!(report.context.analysis_id, "resolution-fixture");
        assert_eq!(report.parameter_ids, vec!["low_mode", "high_mode"]);
    }
}
