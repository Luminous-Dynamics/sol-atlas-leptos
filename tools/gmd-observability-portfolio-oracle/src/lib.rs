//! Small, deterministic linear-Gaussian sensor portfolio design oracle.
//!
//! This crate does not provide physical observation operators. It evaluates
//! posterior covariance only after callers supply a qualified prior,
//! observation matrix, and full measurement-noise covariance.
//!
//! For a selected sensor set S:
//! P_post = (P_prior^-1 + H_S^T R_S^-1 H_S)^-1
//!
//! The portfolio objective balances nominal weighted posterior variance with
//! the worst weighted posterior variance after any one selected sensor fails.

use std::error::Error;
use std::fmt;

pub const MAX_CANDIDATE_SENSORS: usize = 12;
pub const MAX_STATE_DIMENSION: usize = 16;
const SYMMETRY_TOLERANCE: f64 = 1.0e-10;
const TIE_TOLERANCE: f64 = 1.0e-12;

/// Complete design problem. Matrix rows/columns use the same declared state
/// and sensor ordering as the adjacent identifiers/weights/costs.
#[derive(Debug, Clone, PartialEq)]
pub struct DesignProblem {
    pub state_ids: Vec<String>,
    pub sensor_ids: Vec<String>,
    pub prior_covariance: Vec<Vec<f64>>,
    /// One row per candidate sensor, one column per state variable.
    pub observation_matrix: Vec<Vec<f64>>,
    /// Full covariance across all candidate sensor errors. Off-diagonal
    /// entries preserve common-mode/correlated measurement errors.
    pub noise_covariance: Vec<Vec<f64>>,
    /// Decision weights; these are not probabilities.
    pub state_weights: Vec<f64>,
    /// Positive, normalized lifecycle-cost units per candidate sensor.
    pub sensor_costs: Vec<u64>,
    pub budget: u64,
    /// Risk-aversion weight for the worst one-sensor-out case (0 = nominal
    /// only; 1 = worst single-sensor outage only).
    pub outage_risk_weight: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PortfolioResult {
    pub selected_sensor_ids: Vec<String>,
    pub total_cost: u64,
    pub prior_weighted_variance: f64,
    pub nominal_weighted_variance: f64,
    pub worst_single_sensor_outage_weighted_variance: f64,
    pub selection_objective_weighted_variance: f64,
    pub nominal_fractional_reduction: f64,
    pub robust_fractional_reduction: f64,
    pub worst_case_outage_sensor_id: Option<String>,
    /// Posterior covariance with all selected sensors available.
    pub nominal_posterior_covariance: Vec<Vec<f64>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DesignError {
    EmptyStateSet,
    TooManyStateDimensions,
    EmptySensorSet,
    TooManyCandidateSensors,
    InvalidIdentifier,
    DuplicateIdentifier,
    InvalidMatrixShape,
    NonFiniteValue,
    MatrixNotSymmetric,
    MatrixNotPositiveDefinite,
    InvalidStateWeight,
    InvalidSensorCost,
    InvalidOutageRiskWeight,
    InvalidSensorIndex,
    NumericalOverflow,
}

impl fmt::Display for DesignError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::EmptyStateSet => "state set must not be empty",
            Self::TooManyStateDimensions => "state dimension exceeds the small-oracle limit",
            Self::EmptySensorSet => "candidate sensor set must not be empty",
            Self::TooManyCandidateSensors => "candidate sensor count exceeds the exhaustive-search limit",
            Self::InvalidIdentifier => "identifiers must not be empty or whitespace-only",
            Self::DuplicateIdentifier => "state and sensor identifiers must be unique within their sets",
            Self::InvalidMatrixShape => "matrix dimensions do not match declared state/sensor dimensions",
            Self::NonFiniteValue => "all matrix and scalar values must be finite",
            Self::MatrixNotSymmetric => "covariance matrix is not symmetric within tolerance",
            Self::MatrixNotPositiveDefinite => "covariance/precision matrix must be positive definite",
            Self::InvalidStateWeight => "state weights must be finite and strictly positive",
            Self::InvalidSensorCost => "sensor lifecycle costs must be positive",
            Self::InvalidOutageRiskWeight => "outage risk weight must be in [0,1]",
            Self::InvalidSensorIndex => "selected sensor index is out of range or duplicated",
            Self::NumericalOverflow => "matrix operation produced a non-finite value",
        };
        formatter.write_str(message)
    }
}

impl Error for DesignError {}

/// Choose a portfolio under the declared lifecycle-cost budget.
///
/// The exhaustive objective is:
///
/// (1 - outage_risk_weight) * nominal_weighted_variance
/// + outage_risk_weight * worst_single_sensor_outage_weighted_variance
///
/// Ties are resolved deterministically by lower cost, then lexicographic
/// selected sensor IDs. The empty portfolio is always a candidate.
pub fn select_portfolio(problem: &DesignProblem) -> Result<PortfolioResult, DesignError> {
    validate_problem(problem)?;

    let sensor_count = problem.sensor_ids.len();
    let prior_risk = weighted_variance(&problem.prior_covariance, &problem.state_weights)?;
    let prior_precision = inverse_spd(&problem.prior_covariance)?;
    let portfolio_count = 1_usize << sensor_count;

    let mut best: Option<PortfolioResult> = None;

    for mask in 0..portfolio_count {
        let mut selected = Vec::new();
        let mut cost = 0_u64;
        let mut over_budget = false;

        for index in 0..sensor_count {
            if mask & (1_usize << index) != 0 {
                cost = cost
                    .checked_add(problem.sensor_costs[index])
                    .ok_or(DesignError::NumericalOverflow)?;
                if cost > problem.budget {
                    over_budget = true;
                    break;
                }
                selected.push(index);
            }
        }
        if over_budget {
            continue;
        }

        let nominal_covariance = posterior_covariance_for_indices(
            problem,
            &selected,
            &prior_precision,
        )?;
        let nominal_risk = weighted_variance(&nominal_covariance, &problem.state_weights)?;

        let (worst_outage_risk, worst_outage_sensor) = if selected.is_empty() {
            (prior_risk, None)
        } else {
            let mut outage_indices = selected.clone();
            outage_indices.sort_by(|left, right| {
                problem.sensor_ids[*left].cmp(&problem.sensor_ids[*right])
            });

            let mut worst_risk = f64::NEG_INFINITY;
            let mut worst_sensor = None;

            for failed_index in outage_indices {
                let available: Vec<usize> = selected
                    .iter()
                    .copied()
                    .filter(|index| *index != failed_index)
                    .collect();
                let outage_covariance =
                    posterior_covariance_for_indices(problem, &available, &prior_precision)?;
                let outage_risk =
                    weighted_variance(&outage_covariance, &problem.state_weights)?;

                if outage_risk > worst_risk + TIE_TOLERANCE {
                    worst_risk = outage_risk;
                    worst_sensor = Some(problem.sensor_ids[failed_index].clone());
                }
            }
            (worst_risk, worst_sensor)
        };

        let alpha = problem.outage_risk_weight;
        let objective = (1.0 - alpha) * nominal_risk + alpha * worst_outage_risk;
        if !objective.is_finite() {
            return Err(DesignError::NumericalOverflow);
        }

        let mut selected_ids: Vec<String> = selected
            .iter()
            .map(|index| problem.sensor_ids[*index].clone())
            .collect();
        selected_ids.sort();

        let result = PortfolioResult {
            selected_sensor_ids: selected_ids,
            total_cost: cost,
            prior_weighted_variance: prior_risk,
            nominal_weighted_variance: nominal_risk,
            worst_single_sensor_outage_weighted_variance: worst_outage_risk,
            selection_objective_weighted_variance: objective,
            nominal_fractional_reduction: fractional_reduction(prior_risk, nominal_risk)?,
            robust_fractional_reduction: fractional_reduction(prior_risk, worst_outage_risk)?,
            worst_case_outage_sensor_id: worst_outage_sensor,
            nominal_posterior_covariance: nominal_covariance,
        };

        let replace = best
            .as_ref()
            .map(|current| is_better(&result, current))
            .unwrap_or(true);
        if replace {
            best = Some(result);
        }
    }

    best.ok_or(DesignError::NumericalOverflow)
}

/// Compute posterior covariance for a caller-selected portfolio. This is
/// useful for independent comparison and scenario evaluation.
pub fn posterior_covariance(
    problem: &DesignProblem,
    selected_sensor_indices: &[usize],
) -> Result<Vec<Vec<f64>>, DesignError> {
    validate_problem(problem)?;
    validate_selected_indices(selected_sensor_indices, problem.sensor_ids.len())?;
    let prior_precision = inverse_spd(&problem.prior_covariance)?;
    posterior_covariance_for_indices(problem, selected_sensor_indices, &prior_precision)
}

fn is_better(candidate: &PortfolioResult, current: &PortfolioResult) -> bool {
    let delta = candidate.selection_objective_weighted_variance
        - current.selection_objective_weighted_variance;

    if delta < -TIE_TOLERANCE {
        return true;
    }
    if delta.abs() > TIE_TOLERANCE {
        return false;
    }
    if candidate.total_cost != current.total_cost {
        return candidate.total_cost < current.total_cost;
    }
    candidate.selected_sensor_ids < current.selected_sensor_ids
}

fn fractional_reduction(prior: f64, posterior: f64) -> Result<f64, DesignError> {
    let result = 1.0 - posterior / prior;
    if result.is_finite() {
        Ok(result)
    } else {
        Err(DesignError::NumericalOverflow)
    }
}

fn validate_problem(problem: &DesignProblem) -> Result<(), DesignError> {
    let state_count = problem.state_ids.len();
    let sensor_count = problem.sensor_ids.len();

    if state_count == 0 {
        return Err(DesignError::EmptyStateSet);
    }
    if state_count > MAX_STATE_DIMENSION {
        return Err(DesignError::TooManyStateDimensions);
    }
    if sensor_count == 0 {
        return Err(DesignError::EmptySensorSet);
    }
    if sensor_count > MAX_CANDIDATE_SENSORS {
        return Err(DesignError::TooManyCandidateSensors);
    }
    if problem.state_ids.iter().any(|id| id.trim().is_empty() || id.trim() != id)
        || problem.sensor_ids.iter().any(|id| id.trim().is_empty() || id.trim() != id)
    {
        return Err(DesignError::InvalidIdentifier);
    }
    if has_duplicates(&problem.state_ids) || has_duplicates(&problem.sensor_ids) {
        return Err(DesignError::DuplicateIdentifier);
    }
    if problem.prior_covariance.len() != state_count
        || problem.observation_matrix.len() != sensor_count
        || problem.noise_covariance.len() != sensor_count
        || problem.state_weights.len() != state_count
        || problem.sensor_costs.len() != sensor_count
        || problem
            .prior_covariance
            .iter()
            .any(|row| row.len() != state_count)
        || problem
            .observation_matrix
            .iter()
            .any(|row| row.len() != state_count)
        || problem
            .noise_covariance
            .iter()
            .any(|row| row.len() != sensor_count)
    {
        return Err(DesignError::InvalidMatrixShape);
    }

    if problem.state_weights.iter().any(|weight| !weight.is_finite()) {
        return Err(DesignError::NonFiniteValue);
    }
    if problem.state_weights.iter().any(|weight| *weight <= 0.0) {
        return Err(DesignError::InvalidStateWeight);
    }
    if problem.sensor_costs.contains(&0) {
        return Err(DesignError::InvalidSensorCost);
    }
    if !problem.outage_risk_weight.is_finite() {
        return Err(DesignError::NonFiniteValue);
    }
    if !(0.0..=1.0).contains(&problem.outage_risk_weight) {
        return Err(DesignError::InvalidOutageRiskWeight);
    }

    ensure_finite_matrix(&problem.prior_covariance)?;
    ensure_finite_matrix(&problem.observation_matrix)?;
    ensure_finite_matrix(&problem.noise_covariance)?;
    cholesky(&problem.prior_covariance)?;
    cholesky(&problem.noise_covariance)?;
    Ok(())
}

fn has_duplicates(values: &[String]) -> bool {
    let mut sorted = values.to_vec();
    sorted.sort();
    sorted.windows(2).any(|pair| pair[0] == pair[1])
}

fn ensure_finite_matrix(matrix: &[Vec<f64>]) -> Result<(), DesignError> {
    if matrix.iter().flatten().any(|value| !value.is_finite()) {
        Err(DesignError::NonFiniteValue)
    } else {
        Ok(())
    }
}

fn validate_selected_indices(indices: &[usize], sensor_count: usize) -> Result<(), DesignError> {
    let mut sorted = indices.to_vec();
    sorted.sort_unstable();
    if sorted.iter().any(|index| *index >= sensor_count)
        || sorted.windows(2).any(|pair| pair[0] == pair[1])
    {
        return Err(DesignError::InvalidSensorIndex);
    }
    Ok(())
}

fn posterior_covariance_for_indices(
    problem: &DesignProblem,
    selected: &[usize],
    prior_precision: &[Vec<f64>],
) -> Result<Vec<Vec<f64>>, DesignError> {
    if selected.is_empty() {
        return Ok(problem.prior_covariance.clone());
    }

    let state_count = problem.state_ids.len();
    let selected_count = selected.len();

    let mut selected_noise = vec![vec![0.0; selected_count]; selected_count];
    let mut selected_observations = vec![vec![0.0; state_count]; selected_count];

    for (row, original_row) in selected.iter().copied().enumerate() {
        selected_observations[row].clone_from(&problem.observation_matrix[original_row]);
        for (column, original_column) in selected.iter().copied().enumerate() {
            selected_noise[row][column] =
                problem.noise_covariance[original_row][original_column];
        }
    }

    let inverse_noise = inverse_spd(&selected_noise)?;
    let weighted_observations = multiply(&inverse_noise, &selected_observations)?;

    let observations_transpose = transpose(&selected_observations);
    let information_increment = multiply(&observations_transpose, &weighted_observations)?;
    let posterior_precision = add_matrices(prior_precision, &information_increment)?;
    inverse_spd(&posterior_precision)
}

fn weighted_variance(
    covariance: &[Vec<f64>],
    weights: &[f64],
) -> Result<f64, DesignError> {
    if covariance.len() != weights.len()
        || covariance.iter().any(|row| row.len() != weights.len())
    {
        return Err(DesignError::InvalidMatrixShape);
    }
    let mut total = 0.0;
    for (index, weight) in weights.iter().enumerate() {
        total += weight * covariance[index][index];
    }
    if total.is_finite() && total > 0.0 {
        Ok(total)
    } else {
        Err(DesignError::NumericalOverflow)
    }
}

// Matrix row/column indices must stay aligned across independent buffers.
#[allow(clippy::needless_range_loop)]
fn cholesky(matrix: &[Vec<f64>]) -> Result<Vec<Vec<f64>>, DesignError> {
    let dimension = matrix.len();
    if dimension == 0 || matrix.iter().any(|row| row.len() != dimension) {
        return Err(DesignError::InvalidMatrixShape);
    }
    ensure_finite_matrix(matrix)?;

    for row in 0..dimension {
        for column in 0..row {
            let left = matrix[row][column];
            let right = matrix[column][row];
            let scale = f64::MIN_POSITIVE.max(left.abs()).max(right.abs());
            if (left - right).abs() > SYMMETRY_TOLERANCE * scale {
                return Err(DesignError::MatrixNotSymmetric);
            }
        }
    }

    let mut lower = vec![vec![0.0; dimension]; dimension];

    for row in 0..dimension {
        for column in 0..=row {
            let mut value = matrix[row][column];
            for prior in 0..column {
                value -= lower[row][prior] * lower[column][prior];
            }
            if row == column {
                if !value.is_finite() {
                    return Err(DesignError::NumericalOverflow);
                }
                if value <= 0.0 {
                    return Err(DesignError::MatrixNotPositiveDefinite);
                }
                lower[row][column] = value.sqrt();
            } else {
                lower[row][column] = value / lower[column][column];
                if !lower[row][column].is_finite() {
                    return Err(DesignError::NumericalOverflow);
                }
            }
        }
    }
    Ok(lower)
}

// Triangular solves use explicit indices to preserve row/column semantics.
#[allow(clippy::needless_range_loop)]
fn inverse_spd(matrix: &[Vec<f64>]) -> Result<Vec<Vec<f64>>, DesignError> {
    let lower = cholesky(matrix)?;
    let dimension = lower.len();
    let mut inverse = vec![vec![0.0; dimension]; dimension];

    for column in 0..dimension {
        let mut forward = vec![0.0; dimension];
        for row in 0..dimension {
            let mut value = if row == column { 1.0 } else { 0.0 };
            for prior in 0..row {
                value -= lower[row][prior] * forward[prior];
            }
            forward[row] = value / lower[row][row];
        }

        let mut solution = vec![0.0; dimension];
        for row in (0..dimension).rev() {
            let mut value = forward[row];
            for later in (row + 1)..dimension {
                value -= lower[later][row] * solution[later];
            }
            solution[row] = value / lower[row][row];
        }

        for row in 0..dimension {
            inverse[row][column] = solution[row];
        }
    }

    for row in 0..dimension {
        for column in 0..dimension {
            if !inverse[row][column].is_finite() {
                return Err(DesignError::NumericalOverflow);
            }
        }
    }

    for row in 0..dimension {
        for column in 0..row {
            let average = 0.5 * (inverse[row][column] + inverse[column][row]);
            inverse[row][column] = average;
            inverse[column][row] = average;
        }
    }
    Ok(inverse)
}

// Indexing is explicit because this is a small fixed-dimension matrix kernel.
#[allow(clippy::needless_range_loop)]
fn transpose(matrix: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let rows = matrix.len();
    let columns = matrix.first().map_or(0, Vec::len);
    let mut result = vec![vec![0.0; rows]; columns];

    for row in 0..rows {
        for column in 0..columns {
            result[column][row] = matrix[row][column];
        }
    }
    result
}

// Indexing is explicit because this is a small fixed-dimension matrix kernel.
#[allow(clippy::needless_range_loop)]
fn multiply(
    left: &[Vec<f64>],
    right: &[Vec<f64>],
) -> Result<Vec<Vec<f64>>, DesignError> {
    let left_rows = left.len();
    let shared = left.first().map_or(0, Vec::len);
    let right_rows = right.len();
    let right_columns = right.first().map_or(0, Vec::len);

    if shared == 0
        || right_rows != shared
        || left.iter().any(|row| row.len() != shared)
        || right.iter().any(|row| row.len() != right_columns)
    {
        return Err(DesignError::InvalidMatrixShape);
    }

    let mut result = vec![vec![0.0; right_columns]; left_rows];
    for row in 0..left_rows {
        for column in 0..right_columns {
            let mut value = 0.0;
            for index in 0..shared {
                value += left[row][index] * right[index][column];
            }
            if !value.is_finite() {
                return Err(DesignError::NumericalOverflow);
            }
            result[row][column] = value;
        }
    }
    Ok(result)
}

// Indexing is explicit because this is a small fixed-dimension matrix kernel.
#[allow(clippy::needless_range_loop)]
fn add_matrices(
    left: &[Vec<f64>],
    right: &[Vec<f64>],
) -> Result<Vec<Vec<f64>>, DesignError> {
    if left.len() != right.len()
        || left.iter().zip(right).any(|(a, b)| a.len() != b.len())
    {
        return Err(DesignError::InvalidMatrixShape);
    }

    let mut result = left.to_vec();
    for row in 0..left.len() {
        for column in 0..left[row].len() {
            result[row][column] += right[row][column];
            if !result[row][column].is_finite() {
                return Err(DesignError::NumericalOverflow);
            }
        }
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(actual: f64, expected: f64, tolerance: f64) {
        assert!(
            (actual - expected).abs() <= tolerance,
            "actual={actual:.16e}, expected={expected:.16e}, tolerance={tolerance:.3e}"
        );
    }

    fn scalar_problem() -> DesignProblem {
        DesignProblem {
            state_ids: vec!["east-west-field".to_owned()],
            sensor_ids: vec!["mag-local".to_owned()],
            prior_covariance: vec![vec![4.0]],
            observation_matrix: vec![vec![1.0]],
            noise_covariance: vec![vec![1.0]],
            state_weights: vec![1.0],
            sensor_costs: vec![1],
            budget: 1,
            outage_risk_weight: 0.0,
        }
    }

    #[test]
    fn single_observation_matches_hand_computable_posterior() {
        let problem = scalar_problem();
        let posterior = posterior_covariance(&problem, &[0]).unwrap();
        close(posterior[0][0], 0.8, 1.0e-12);
    }

    #[test]
    fn empty_selected_set_returns_prior_covariance() {
        let problem = scalar_problem();
        let posterior = posterior_covariance(&problem, &[]).unwrap();
        assert_eq!(posterior, problem.prior_covariance);
    }

    #[test]
    fn correlated_sensor_noise_is_preserved() {
        let problem = DesignProblem {
            state_ids: vec!["local-e-field".to_owned()],
            sensor_ids: vec!["mag-a".to_owned(), "mag-b".to_owned()],
            prior_covariance: vec![vec![1.0]],
            observation_matrix: vec![vec![1.0], vec![1.0]],
            noise_covariance: vec![vec![1.0, 0.5], vec![0.5, 1.0]],
            state_weights: vec![1.0],
            sensor_costs: vec![1, 1],
            budget: 2,
            outage_risk_weight: 0.0,
        };
        let posterior = posterior_covariance(&problem, &[0, 1]).unwrap();
        close(posterior[0][0], 3.0 / 7.0, 1.0e-12);
    }

    #[test]
    fn complementary_sensors_win_under_single_sensor_outage_risk() {
        let problem = DesignProblem {
            state_ids: vec!["state-east".to_owned(), "state-west".to_owned()],
            sensor_ids: vec![
                "mag-east".to_owned(),
                "mag-west".to_owned(),
                "distant-proxy".to_owned(),
            ],
            prior_covariance: vec![vec![1.0, 0.0], vec![0.0, 1.0]],
            observation_matrix: vec![
                vec![1.0, 0.0],
                vec![0.0, 1.0],
                vec![1.0, 1.0],
            ],
            noise_covariance: vec![
                vec![0.1, 0.0, 0.0],
                vec![0.0, 0.1, 0.0],
                vec![0.0, 0.0, 0.1],
            ],
            state_weights: vec![1.0, 1.0],
            sensor_costs: vec![1, 1, 2],
            budget: 2,
            outage_risk_weight: 0.5,
        };

        let result = select_portfolio(&problem).unwrap();
        assert_eq!(result.selected_sensor_ids, vec!["mag-east", "mag-west"]);
        close(result.nominal_weighted_variance, 2.0 / 11.0, 1.0e-12);
        close(
            result.worst_single_sensor_outage_weighted_variance,
            1.0 + 1.0 / 11.0,
            1.0e-12,
        );
        assert!(result.robust_fractional_reduction > 0.0);
    }

    #[test]
    fn budget_excludes_overpriced_sensor() {
        let mut problem = scalar_problem();
        problem.sensor_costs[0] = 2;
        problem.budget = 1;
        let result = select_portfolio(&problem).unwrap();
        assert!(result.selected_sensor_ids.is_empty());
        close(result.nominal_weighted_variance, 4.0, 1.0e-12);
    }

    #[test]
    fn ties_resolve_by_cost_then_lexicographic_sensor_id() {
        let problem = DesignProblem {
            state_ids: vec!["one-state".to_owned()],
            sensor_ids: vec!["sensor-z".to_owned(), "sensor-a".to_owned()],
            prior_covariance: vec![vec![1.0]],
            observation_matrix: vec![vec![1.0], vec![1.0]],
            noise_covariance: vec![vec![1.0, 0.0], vec![0.0, 1.0]],
            state_weights: vec![1.0],
            sensor_costs: vec![1, 1],
            budget: 1,
            outage_risk_weight: 0.0,
        };
        let result = select_portfolio(&problem).unwrap();
        assert_eq!(result.selected_sensor_ids, vec!["sensor-a"]);
    }

    #[test]
    fn invalid_non_positive_definite_covariance_is_rejected() {
        let mut problem = scalar_problem();
        problem.noise_covariance = vec![vec![0.0]];
        assert_eq!(
            select_portfolio(&problem),
            Err(DesignError::MatrixNotPositiveDefinite)
        );
    }

    #[test]
    fn asymmetric_noise_covariance_is_rejected() {
        let mut problem = scalar_problem();
        problem.sensor_ids.push("second-mag".to_owned());
        problem.observation_matrix.push(vec![1.0]);
        problem.noise_covariance = vec![vec![1.0, 0.2], vec![0.1, 1.0]];
        problem.sensor_costs.push(1);
        assert_eq!(
            select_portfolio(&problem),
            Err(DesignError::MatrixNotSymmetric)
        );

        problem.noise_covariance = vec![vec![1.0e-12, 2.0e-13], vec![1.0e-13, 1.0e-12]];
        assert_eq!(
            select_portfolio(&problem),
            Err(DesignError::MatrixNotSymmetric)
        );
    }

    #[test]
    fn duplicated_sensor_ids_are_rejected() {
        let mut problem = scalar_problem();
        problem.sensor_ids.push("mag-local".to_owned());
        problem.observation_matrix.push(vec![1.0]);
        problem.noise_covariance = vec![vec![1.0, 0.0], vec![0.0, 1.0]];
        problem.sensor_costs.push(1);
        assert_eq!(select_portfolio(&problem), Err(DesignError::DuplicateIdentifier));
    }

    #[test]
    fn invalid_state_weights_and_outage_weights_are_rejected() {
        let mut problem = scalar_problem();
        problem.state_weights[0] = 0.0;
        assert_eq!(select_portfolio(&problem), Err(DesignError::InvalidStateWeight));

        let mut problem = scalar_problem();
        problem.outage_risk_weight = 1.5;
        assert_eq!(
            select_portfolio(&problem),
            Err(DesignError::InvalidOutageRiskWeight)
        );
    }

    #[test]
    fn selected_indices_must_be_unique_and_in_range() {
        let problem = scalar_problem();
        assert_eq!(
            posterior_covariance(&problem, &[0, 0]),
            Err(DesignError::InvalidSensorIndex)
        );
        assert_eq!(
            posterior_covariance(&problem, &[1]),
            Err(DesignError::InvalidSensorIndex)
        );
    }

    #[test]
    fn state_dimension_is_bounded_for_the_reference_kernel() {
        let mut problem = scalar_problem();
        problem.state_ids = (0..=MAX_STATE_DIMENSION)
            .map(|index| format!("state-{index}"))
            .collect();
        let dimension = problem.state_ids.len();
        problem.prior_covariance = vec![vec![0.0; dimension]; dimension];
        for (index, row) in problem.prior_covariance.iter_mut().enumerate() {
            row[index] = 1.0;
        }
        problem.observation_matrix = vec![vec![1.0; dimension]];
        assert_eq!(select_portfolio(&problem), Err(DesignError::TooManyStateDimensions));
    }

    #[test]
    fn candidate_count_is_bounded_for_deterministic_exhaustive_search() {
        let mut problem = scalar_problem();
        problem.sensor_ids = (0..=MAX_CANDIDATE_SENSORS)
            .map(|index| format!("sensor-{index}"))
            .collect();
        problem.observation_matrix = vec![vec![1.0]; problem.sensor_ids.len()];
        problem.noise_covariance = vec![
            vec![0.0; problem.sensor_ids.len()];
            problem.sensor_ids.len()
        ];
        for (index, row) in problem.noise_covariance.iter_mut().enumerate() {
            row[index] = 1.0;
        }
        problem.sensor_costs = vec![1; problem.sensor_ids.len()];
        problem.budget = problem.sensor_ids.len() as u64;
        assert_eq!(select_portfolio(&problem), Err(DesignError::TooManyCandidateSensors));
    }
}
