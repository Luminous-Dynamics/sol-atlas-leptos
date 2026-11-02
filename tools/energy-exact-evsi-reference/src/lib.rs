#![forbid(unsafe_code)]
//! Exact, finite-state expected value of sample information (EVSI).
//!
//! All priors and likelihoods are integer parts per million. Expected risks are
//! accumulated as integer numerators over 10^12. No floating-point arithmetic,
//! Monte Carlo estimates, scalarization or posterior rounding is used.

use std::cmp::Ordering;
use std::collections::BTreeSet;
use std::fmt;

pub const PROBABILITY_SCALE: u32 = 1_000_000;
pub const EVSI_DENOMINATOR: u128 = 1_000_000_000_000;
pub const MAX_STATES: usize = 32;
pub const MAX_ACTIONS: usize = 32;
pub const MAX_OBSERVATIONS: usize = 16;
pub const MAX_CANDIDATES: usize = 32;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct WeightedState {
    pub id: String,
    pub prior_probability_ppm: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecisionProblem {
    pub states: Vec<WeightedState>,
    /// Actions are identified by stable IDs; ties are broken by lexical ID.
    pub actions: Vec<String>,
    /// losses[state_index][action_index], in exact integer loss units.
    pub losses: Vec<Vec<u64>>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObservationDesign {
    pub id: String,
    pub observations: Vec<String>,
    /// likelihoods_ppm[state_index][observation_index].
    /// Every state row must sum to exactly PROBABILITY_SCALE.
    pub likelihoods_ppm: Vec<Vec<u32>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum EvsiError {
    EmptyStates,
    EmptyActions,
    EmptyObservations,
    TooManyStates,
    TooManyActions,
    TooManyObservations,
    TooManyCandidates,
    InvalidIdentifier,
    DuplicateIdentifier,
    PriorProbabilitiesDoNotSumToOne,
    LikelihoodProbabilitiesDoNotSumToOne,
    ProbabilityOutOfRange,
    LossMatrixShapeMismatch,
    LikelihoodMatrixShapeMismatch,
    ZeroComputeCost,
    ArithmeticOverflow,
    NegativeEvsi,
    EvsiExceedsPerfectInformation,
}

impl fmt::Display for EvsiError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{self:?}")
    }
}

impl std::error::Error for EvsiError {}

fn valid_id(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"_-.:".contains(&byte))
}

fn validate_unique<'a>(
    values: impl IntoIterator<Item = &'a str>,
) -> Result<(), EvsiError> {
    let mut seen = BTreeSet::new();
    for value in values {
        if !valid_id(value) {
            return Err(EvsiError::InvalidIdentifier);
        }
        if !seen.insert(value) {
            return Err(EvsiError::DuplicateIdentifier);
        }
    }
    Ok(())
}

impl DecisionProblem {
    pub fn validate(&self) -> Result<(), EvsiError> {
        if self.states.is_empty() {
            return Err(EvsiError::EmptyStates);
        }
        if self.actions.is_empty() {
            return Err(EvsiError::EmptyActions);
        }
        if self.states.len() > MAX_STATES {
            return Err(EvsiError::TooManyStates);
        }
        if self.actions.len() > MAX_ACTIONS {
            return Err(EvsiError::TooManyActions);
        }
        validate_unique(self.states.iter().map(|state| state.id.as_str()))?;
        validate_unique(self.actions.iter().map(String::as_str))?;

        let prior_total: u64 = self
            .states
            .iter()
            .map(|state| u64::from(state.prior_probability_ppm))
            .sum();
        if prior_total != u64::from(PROBABILITY_SCALE) {
            return Err(EvsiError::PriorProbabilitiesDoNotSumToOne);
        }
        if self.losses.len() != self.states.len()
            || self
                .losses
                .iter()
                .any(|state_losses| state_losses.len() != self.actions.len())
        {
            return Err(EvsiError::LossMatrixShapeMismatch);
        }
        Ok(())
    }
}

impl ObservationDesign {
    pub fn validate(&self, state_count: usize) -> Result<(), EvsiError> {
        if self.observations.is_empty() {
            return Err(EvsiError::EmptyObservations);
        }
        if self.observations.len() > MAX_OBSERVATIONS {
            return Err(EvsiError::TooManyObservations);
        }
        validate_unique([self.id.as_str()])?;
        validate_unique(self.observations.iter().map(String::as_str))?;
        if self.likelihoods_ppm.len() != state_count
            || self
                .likelihoods_ppm
                .iter()
                .any(|row| row.len() != self.observations.len())
        {
            return Err(EvsiError::LikelihoodMatrixShapeMismatch);
        }
        for row in &self.likelihoods_ppm {
            if row.iter().any(|value| *value > PROBABILITY_SCALE) {
                return Err(EvsiError::ProbabilityOutOfRange);
            }
            let total: u64 = row.iter().map(|value| u64::from(*value)).sum();
            if total != u64::from(PROBABILITY_SCALE) {
                return Err(EvsiError::LikelihoodProbabilitiesDoNotSumToOne);
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObservationDecision {
    pub observation_id: String,
    /// Joint probability mass P(observation), scaled by 10^12.
    pub joint_probability_mass_ppm2: u128,
    pub posterior_best_action_id: Option<String>,
    /// Joint expected loss numerator, with denominator 10^12.
    pub posterior_loss_numerator_ppm2: u128,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExactEvsi {
    pub observation_design_id: String,
    pub prior_best_action_id: String,
    /// Prior Bayes risk numerator over a 10^6 denominator.
    pub prior_bayes_risk_numerator_ppm: u128,
    /// Expected posterior Bayes risk numerator over a 10^12 denominator.
    pub posterior_bayes_risk_numerator_ppm2: u128,
    /// Expected perfect-information risk numerator over a 10^12 denominator.
    pub perfect_information_risk_numerator_ppm2: u128,
    /// EVSI = prior Bayes risk - expected posterior Bayes risk.
    pub evsi_numerator_ppm2: u128,
    /// EVPI = prior Bayes risk - perfect-information risk.
    pub evpi_numerator_ppm2: u128,
    pub observation_cost_numerator_ppm2: u128,
    /// Absolute magnitude of EVSI minus observation cost, still over 10^12.
    pub net_value_magnitude_numerator_ppm2: u128,
    pub net_value_nonnegative: bool,
    pub worthwhile_to_observe: bool,
    pub posterior_decisions: Vec<ObservationDecision>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ObservationCandidate {
    pub id: String,
    pub design: ObservationDesign,
    /// Authored loss lower bound currently used by the mesh heuristic.
    pub heuristic_avoidable_loss_lower_bound_units: u64,
    /// Direct cost of collecting/processing the observation, in loss units.
    pub observation_cost_units: u64,
    /// Marginal compute required to refine/consume this observation.
    pub compute_cost_units: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CandidateEvaluation {
    pub id: String,
    pub heuristic_avoidable_loss_lower_bound_units: u64,
    pub compute_cost_units: u64,
    pub exact_evsi: ExactEvsi,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CandidateRanking {
    /// Ranking induced by the existing lower-bound-per-compute heuristic.
    pub heuristic_order: Vec<String>,
    /// Ranking induced by exact net EVSI per marginal compute unit.
    /// Unprofitable candidates are always placed after worthwhile candidates.
    pub exact_evsi_order: Vec<String>,
    pub evaluations: Vec<CandidateEvaluation>,
}

fn checked_add(total: &mut u128, value: u128) -> Result<(), EvsiError> {
    *total = total.checked_add(value).ok_or(EvsiError::ArithmeticOverflow)?;
    Ok(())
}

fn checked_mul(left: u128, right: u128) -> Result<u128, EvsiError> {
    left.checked_mul(right).ok_or(EvsiError::ArithmeticOverflow)
}

fn weighted_prior_loss(
    problem: &DecisionProblem,
    action_index: usize,
) -> Result<u128, EvsiError> {
    let mut total = 0u128;
    for (state, losses) in problem.states.iter().zip(&problem.losses) {
        checked_add(
            &mut total,
            checked_mul(
                u128::from(state.prior_probability_ppm),
                u128::from(losses[action_index]),
            )?,
        )?;
    }
    Ok(total)
}

fn best_action<'a>(
    problem: &'a DecisionProblem,
    risks: &[u128],
) -> (usize, &'a str) {
    let mut best_index = 0usize;
    for index in 1..problem.actions.len() {
        if risks[index] < risks[best_index]
            || (risks[index] == risks[best_index]
                && problem.actions[index] < problem.actions[best_index])
        {
            best_index = index;
        }
    }
    (best_index, &problem.actions[best_index])
}

fn compare_positive_fractions(
    left_numerator: u128,
    left_denominator: u64,
    right_numerator: u128,
    right_denominator: u64,
) -> Ordering {
    let left_whole = left_numerator / u128::from(left_denominator);
    let right_whole = right_numerator / u128::from(right_denominator);
    match left_whole.cmp(&right_whole) {
        Ordering::Equal => {
            // Remainders are strictly smaller than u64 denominators, so their
            // cross-products fit inside u128 without saturation or rounding.
            let left_remainder = left_numerator % u128::from(left_denominator);
            let right_remainder = right_numerator % u128::from(right_denominator);
            (left_remainder * u128::from(right_denominator))
                .cmp(&(right_remainder * u128::from(left_denominator)))
        }
        other => other,
    }
}

/// Exact EVSI over a finite prior, loss table, and observation likelihood matrix.
///
/// The posterior Bayes risk is the sum over observations of each observation's
/// probability mass multiplied by its minimum posterior expected loss. Joint
/// masses are retained at ppm^2 precision, with no posterior normalization or
/// floating-point rounding.
pub fn exact_evsi(
    problem: &DecisionProblem,
    design: &ObservationDesign,
    observation_cost_units: u64,
) -> Result<ExactEvsi, EvsiError> {
    problem.validate()?;
    design.validate(problem.states.len())?;

    let prior_action_risks: Vec<u128> = (0..problem.actions.len())
        .map(|action_index| weighted_prior_loss(problem, action_index))
        .collect::<Result<_, _>>()?;
    let (prior_best_action_index, prior_best_action_id) =
        best_action(problem, &prior_action_risks);
    let prior_bayes_risk_numerator_ppm = prior_action_risks[prior_best_action_index];

    let mut perfect_information_risk_numerator_ppm2 = 0u128;
    for (state_index, state) in problem.states.iter().enumerate() {
        let best_loss = problem.losses[state_index]
            .iter()
            .copied()
            .min()
            .ok_or(EvsiError::EmptyActions)?;
        let joint_prior_mass = checked_mul(
            u128::from(state.prior_probability_ppm),
            u128::from(PROBABILITY_SCALE),
        )?;
        checked_add(
            &mut perfect_information_risk_numerator_ppm2,
            checked_mul(joint_prior_mass, u128::from(best_loss))?,
        )?;
    }

    let prior_risk_numerator_ppm2 = checked_mul(
        prior_bayes_risk_numerator_ppm,
        u128::from(PROBABILITY_SCALE),
    )?;
    let mut posterior_bayes_risk_numerator_ppm2 = 0u128;
    let mut posterior_decisions = Vec::with_capacity(design.observations.len());

    for observation_index in 0..design.observations.len() {
        let mut joint_state_masses = Vec::with_capacity(problem.states.len());
        let mut joint_probability_mass_ppm2 = 0u128;
        for (state_index, state) in problem.states.iter().enumerate() {
            let mass = checked_mul(
                u128::from(state.prior_probability_ppm),
                u128::from(design.likelihoods_ppm[state_index][observation_index]),
            )?;
            checked_add(&mut joint_probability_mass_ppm2, mass)?;
            joint_state_masses.push(mass);
        }

        if joint_probability_mass_ppm2 == 0 {
            posterior_decisions.push(ObservationDecision {
                observation_id: design.observations[observation_index].clone(),
                joint_probability_mass_ppm2,
                posterior_best_action_id: None,
                posterior_loss_numerator_ppm2: 0,
            });
            continue;
        }

        let mut posterior_action_risks = vec![0u128; problem.actions.len()];
        for (state_index, joint_mass) in joint_state_masses.iter().copied().enumerate() {
            for (action_index, risk) in posterior_action_risks.iter_mut().enumerate() {
                checked_add(
                    risk,
                    checked_mul(
                        joint_mass,
                        u128::from(problem.losses[state_index][action_index]),
                    )?,
                )?;
            }
        }
        let (best_action_index, best_action_id) = best_action(problem, &posterior_action_risks);
        let posterior_loss_numerator_ppm2 = posterior_action_risks[best_action_index];
        checked_add(
            &mut posterior_bayes_risk_numerator_ppm2,
            posterior_loss_numerator_ppm2,
        )?;
        posterior_decisions.push(ObservationDecision {
            observation_id: design.observations[observation_index].clone(),
            joint_probability_mass_ppm2,
            posterior_best_action_id: Some(best_action_id.to_owned()),
            posterior_loss_numerator_ppm2,
        });
    }

    if posterior_bayes_risk_numerator_ppm2 > prior_risk_numerator_ppm2 {
        return Err(EvsiError::NegativeEvsi);
    }
    if posterior_bayes_risk_numerator_ppm2 < perfect_information_risk_numerator_ppm2 {
        return Err(EvsiError::EvsiExceedsPerfectInformation);
    }

    let evsi_numerator_ppm2 =
        prior_risk_numerator_ppm2 - posterior_bayes_risk_numerator_ppm2;
    let evpi_numerator_ppm2 =
        prior_risk_numerator_ppm2 - perfect_information_risk_numerator_ppm2;
    if evsi_numerator_ppm2 > evpi_numerator_ppm2 {
        return Err(EvsiError::EvsiExceedsPerfectInformation);
    }
    let observation_cost_numerator_ppm2 = checked_mul(
        u128::from(observation_cost_units),
        EVSI_DENOMINATOR,
    )?;
    let net_value_nonnegative = evsi_numerator_ppm2 >= observation_cost_numerator_ppm2;
    let net_value_magnitude_numerator_ppm2 = if net_value_nonnegative {
        evsi_numerator_ppm2 - observation_cost_numerator_ppm2
    } else {
        observation_cost_numerator_ppm2 - evsi_numerator_ppm2
    };

    posterior_decisions.sort_by(|left, right| left.observation_id.cmp(&right.observation_id));
    Ok(ExactEvsi {
        observation_design_id: design.id.clone(),
        prior_best_action_id: prior_best_action_id.to_owned(),
        prior_bayes_risk_numerator_ppm,
        posterior_bayes_risk_numerator_ppm2,
        perfect_information_risk_numerator_ppm2,
        evsi_numerator_ppm2,
        evpi_numerator_ppm2,
        observation_cost_numerator_ppm2,
        net_value_magnitude_numerator_ppm2,
        net_value_nonnegative,
        worthwhile_to_observe: net_value_nonnegative,
        posterior_decisions,
    })
}

fn compare_candidate_heuristic(left: &CandidateEvaluation, right: &CandidateEvaluation) -> Ordering {
    compare_positive_fractions(
        u128::from(left.heuristic_avoidable_loss_lower_bound_units),
        left.compute_cost_units,
        u128::from(right.heuristic_avoidable_loss_lower_bound_units),
        right.compute_cost_units,
    )
    .reverse()
    .then_with(|| left.id.cmp(&right.id))
}

fn compare_candidate_exact(left: &CandidateEvaluation, right: &CandidateEvaluation) -> Ordering {
    match (
        left.exact_evsi.net_value_nonnegative,
        right.exact_evsi.net_value_nonnegative,
    ) {
        (true, false) => Ordering::Less,
        (false, true) => Ordering::Greater,
        (true, true) => compare_positive_fractions(
            left.exact_evsi.net_value_magnitude_numerator_ppm2,
            left.compute_cost_units,
            right.exact_evsi.net_value_magnitude_numerator_ppm2,
            right.compute_cost_units,
        )
        .reverse()
        .then_with(|| left.id.cmp(&right.id)),
        (false, false) => compare_positive_fractions(
            left.exact_evsi.net_value_magnitude_numerator_ppm2,
            left.compute_cost_units,
            right.exact_evsi.net_value_magnitude_numerator_ppm2,
            right.compute_cost_units,
        )
        .then_with(|| left.id.cmp(&right.id)),
    }
}

/// Evaluate each observation and return both the current heuristic ranking and
/// the exact posterior-risk reduction ranking. Exact ranks use net EVSI per
/// marginal compute unit; candidates with negative or zero net value follow all
/// candidates with strictly positive net value.
pub fn rank_observation_candidates(
    problem: &DecisionProblem,
    candidates: &[ObservationCandidate],
) -> Result<CandidateRanking, EvsiError> {
    if candidates.len() > MAX_CANDIDATES {
        return Err(EvsiError::TooManyCandidates);
    }
    validate_unique(candidates.iter().map(|candidate| candidate.id.as_str()))?;
    let mut evaluations = Vec::with_capacity(candidates.len());
    for candidate in candidates {
        if candidate.compute_cost_units == 0 {
            return Err(EvsiError::ZeroComputeCost);
        }
        if candidate.id != candidate.design.id {
            return Err(EvsiError::InvalidIdentifier);
        }
        let evsi = exact_evsi(problem, &candidate.design, candidate.observation_cost_units)?;
        evaluations.push(CandidateEvaluation {
            id: candidate.id.clone(),
            heuristic_avoidable_loss_lower_bound_units:
                candidate.heuristic_avoidable_loss_lower_bound_units,
            compute_cost_units: candidate.compute_cost_units,
            exact_evsi: evsi,
        });
    }

    let mut heuristic = evaluations.clone();
    heuristic.sort_by(compare_candidate_heuristic);
    let mut exact = evaluations.clone();
    exact.sort_by(compare_candidate_exact);

    Ok(CandidateRanking {
        heuristic_order: heuristic.iter().map(|candidate| candidate.id.clone()).collect(),
        exact_evsi_order: exact.iter().map(|candidate| candidate.id.clone()).collect(),
        evaluations,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn two_state_problem() -> DecisionProblem {
        DecisionProblem {
            states: vec![
                WeightedState {
                    id: "normal".to_owned(),
                    prior_probability_ppm: 500_000,
                },
                WeightedState {
                    id: "storm".to_owned(),
                    prior_probability_ppm: 500_000,
                },
            ],
            actions: vec!["do-nothing".to_owned(), "prepare".to_owned()],
            losses: vec![vec![0, 40], vec![100, 40]],
        }
    }

    fn design(id: &str, normal: [u32; 2], storm: [u32; 2]) -> ObservationDesign {
        ObservationDesign {
            id: id.to_owned(),
            observations: vec!["signal-normal".to_owned(), "signal-storm".to_owned()],
            likelihoods_ppm: vec![normal.to_vec(), storm.to_vec()],
        }
    }

    fn evsi(problem: &DecisionProblem, design: &ObservationDesign) -> ExactEvsi {
        exact_evsi(problem, design, 0).unwrap()
    }

    #[test]
    fn perfect_observation_equals_value_of_perfect_information() {
        let problem = two_state_problem();
        let perfect = design("perfect", [1_000_000, 0], [0, 1_000_000]);
        let result = evsi(&problem, &perfect);

        assert_eq!(result.prior_best_action_id, "prepare");
        assert_eq!(result.prior_bayes_risk_numerator_ppm, 40_000_000);
        assert_eq!(result.posterior_bayes_risk_numerator_ppm2, 20_000_000_000_000);
        assert_eq!(result.evsi_numerator_ppm2, 20_000_000_000_000);
        assert_eq!(result.evpi_numerator_ppm2, 20_000_000_000_000);
        assert!(result.worthwhile_to_observe);
    }

    #[test]
    fn uninformative_observation_has_exactly_zero_evsi() {
        let problem = two_state_problem();
        let uninformative = design("none", [500_000, 500_000], [500_000, 500_000]);
        let result = evsi(&problem, &uninformative);

        assert_eq!(result.evsi_numerator_ppm2, 0);
        assert_eq!(
            result.posterior_bayes_risk_numerator_ppm2,
            result.prior_bayes_risk_numerator_ppm * u128::from(PROBABILITY_SCALE)
        );
    }

    #[test]
    fn noisy_observation_has_positive_evsi_below_perfect_information() {
        let problem = two_state_problem();
        let noisy = design("noisy", [900_000, 100_000], [100_000, 900_000]);
        let result = evsi(&problem, &noisy);

        assert_eq!(result.evsi_numerator_ppm2, 15_000_000_000_000);
        assert_eq!(result.evpi_numerator_ppm2, 20_000_000_000_000);
        assert!(result.evsi_numerator_ppm2 < result.evpi_numerator_ppm2);
    }

    #[test]
    fn more_informative_symmetric_signal_cannot_reduce_evsi() {
        let problem = two_state_problem();
        let less_informative = design("seventy", [700_000, 300_000], [300_000, 700_000]);
        let more_informative = design("ninety", [900_000, 100_000], [100_000, 900_000]);

        assert!(
            evsi(&problem, &more_informative).evsi_numerator_ppm2
                >= evsi(&problem, &less_informative).evsi_numerator_ppm2
        );
    }

    #[test]
    fn observation_cost_is_compared_without_rounding() {
        let problem = two_state_problem();
        let perfect = design("perfect", [1_000_000, 0], [0, 1_000_000]);
        assert!(exact_evsi(&problem, &perfect, 20).unwrap().worthwhile_to_observe);
        assert!(!exact_evsi(&problem, &perfect, 21).unwrap().worthwhile_to_observe);
    }

    #[test]
    fn zero_probability_observation_is_retained_without_a_posterior_policy() {
        let problem = two_state_problem();
        let design = ObservationDesign {
            id: "zero-mass".to_owned(),
            observations: vec![
                "impossible".to_owned(),
                "ordinary".to_owned(),
                "storm".to_owned(),
            ],
            likelihoods_ppm: vec![
                vec![0, 1_000_000, 0],
                vec![0, 0, 1_000_000],
            ],
        };
        let result = exact_evsi(&problem, &design, 0).unwrap();
        let impossible = result
            .posterior_decisions
            .iter()
            .find(|item| item.observation_id == "impossible")
            .unwrap();
        assert_eq!(impossible.joint_probability_mass_ppm2, 0);
        assert_eq!(impossible.posterior_best_action_id, None);
    }

    #[test]
    fn reordering_states_and_actions_does_not_change_the_policy_or_evsi() {
        let problem = two_state_problem();
        let noisy = design("noisy", [900_000, 100_000], [100_000, 900_000]);
        let first = exact_evsi(&problem, &noisy, 0).unwrap();

        let reordered = DecisionProblem {
            states: vec![problem.states[1].clone(), problem.states[0].clone()],
            actions: vec![problem.actions[1].clone(), problem.actions[0].clone()],
            losses: vec![problem.losses[1].iter().rev().copied().collect(),
                         problem.losses[0].iter().rev().copied().collect()],
        };
        let second_design = design("noisy", [100_000, 900_000], [900_000, 100_000]);
        let second = exact_evsi(&reordered, &second_design, 0).unwrap();
        assert_eq!(first.prior_best_action_id, second.prior_best_action_id);
        assert_eq!(first.evsi_numerator_ppm2, second.evsi_numerator_ppm2);
        assert_eq!(first.evpi_numerator_ppm2, second.evpi_numerator_ppm2);
    }

    #[test]
    fn exact_evsi_overturns_a_high_lower_bound_but_uninformative_mesh_choice() {
        let problem = two_state_problem();
        let candidates = vec![
            ObservationCandidate {
                id: "high-bound-low-information".to_owned(),
                design: design("high-bound-low-information", [500_000, 500_000], [500_000, 500_000]),
                heuristic_avoidable_loss_lower_bound_units: 10_000,
                observation_cost_units: 0,
                compute_cost_units: 1,
            },
            ObservationCandidate {
                id: "lower-bound-informative".to_owned(),
                design: design("lower-bound-informative", [900_000, 100_000], [100_000, 900_000]),
                heuristic_avoidable_loss_lower_bound_units: 100,
                observation_cost_units: 0,
                compute_cost_units: 1,
            },
        ];

        let ranking = rank_observation_candidates(&problem, &candidates).unwrap();
        assert_eq!(
            ranking.heuristic_order,
            vec!["high-bound-low-information", "lower-bound-informative"]
        );
        assert_eq!(
            ranking.exact_evsi_order,
            vec!["lower-bound-informative", "high-bound-low-information"]
        );
    }

    #[test]
    fn unprofitable_information_follows_profitable_information_in_exact_ranking() {
        let problem = two_state_problem();
        let candidates = vec![
            ObservationCandidate {
                id: "expensive-perfect".to_owned(),
                design: design("expensive-perfect", [1_000_000, 0], [0, 1_000_000]),
                heuristic_avoidable_loss_lower_bound_units: 100,
                observation_cost_units: 21,
                compute_cost_units: 1,
            },
            ObservationCandidate {
                id: "free-noisy".to_owned(),
                design: design("free-noisy", [900_000, 100_000], [100_000, 900_000]),
                heuristic_avoidable_loss_lower_bound_units: 10,
                observation_cost_units: 0,
                compute_cost_units: 1,
            },
        ];
        let ranking = rank_observation_candidates(&problem, &candidates).unwrap();
        assert_eq!(ranking.exact_evsi_order[0], "free-noisy");
        assert_eq!(ranking.exact_evsi_order[1], "expensive-perfect");
    }

    #[test]
    fn tied_prior_actions_are_selected_by_lexical_id() {
        let problem = DecisionProblem {
            states: vec![WeightedState {
                id: "single".to_owned(),
                prior_probability_ppm: 1_000_000,
            }],
            actions: vec!["z-action".to_owned(), "a-action".to_owned()],
            losses: vec![vec![10, 10]],
        };
        let uninformative = ObservationDesign {
            id: "signal".to_owned(),
            observations: vec!["only".to_owned()],
            likelihoods_ppm: vec![vec![1_000_000]],
        };
        assert_eq!(
            exact_evsi(&problem, &uninformative, 0)
                .unwrap()
                .prior_best_action_id,
            "a-action"
        );
    }

    #[test]
    fn validates_prior_probability_sum() {
        let mut problem = two_state_problem();
        problem.states[0].prior_probability_ppm -= 1;
        assert_eq!(
            exact_evsi(&problem, &design("d", [1_000_000, 0], [0, 1_000_000]), 0),
            Err(EvsiError::PriorProbabilitiesDoNotSumToOne)
        );
    }

    #[test]
    fn validates_each_likelihood_row_sum_independently() {
        let problem = two_state_problem();
        let invalid = design("bad", [500_000, 400_000], [500_000, 500_000]);
        assert_eq!(
            exact_evsi(&problem, &invalid, 0),
            Err(EvsiError::LikelihoodProbabilitiesDoNotSumToOne)
        );
    }

    #[test]
    fn rejects_probability_values_above_one_million_ppm() {
        let problem = two_state_problem();
        let invalid = design("bad", [1_000_001, 0], [0, 1_000_000]);
        assert_eq!(
            exact_evsi(&problem, &invalid, 0),
            Err(EvsiError::ProbabilityOutOfRange)
        );
    }

    #[test]
    fn rejects_malformed_loss_and_likelihood_matrix_shapes() {
        let mut problem = two_state_problem();
        problem.losses.pop();
        assert_eq!(
            exact_evsi(&problem, &design("ok", [1_000_000, 0], [0, 1_000_000]), 0),
            Err(EvsiError::LossMatrixShapeMismatch)
        );

        let problem = two_state_problem();
        let mut invalid = design("bad", [1_000_000, 0], [0, 1_000_000]);
        invalid.likelihoods_ppm.pop();
        assert_eq!(
            exact_evsi(&problem, &invalid, 0),
            Err(EvsiError::LikelihoodMatrixShapeMismatch)
        );
    }

    #[test]
    fn zero_compute_cost_is_rejected_in_candidate_ranking() {
        let candidates = vec![ObservationCandidate {
            id: "free-compute".to_owned(),
            design: design("free-compute", [1_000_000, 0], [0, 1_000_000]),
            heuristic_avoidable_loss_lower_bound_units: 100,
            observation_cost_units: 0,
            compute_cost_units: 0,
        }];
        assert_eq!(
            rank_observation_candidates(&two_state_problem(), &candidates),
            Err(EvsiError::ZeroComputeCost)
        );
    }

    #[test]
    fn duplicate_state_action_observation_and_candidate_ids_are_rejected() {
        let mut problem = two_state_problem();
        problem.states[1].id = problem.states[0].id.clone();
        assert_eq!(
            problem.validate(),
            Err(EvsiError::DuplicateIdentifier)
        );

        let problem = two_state_problem();
        let mut invalid = design("duplicate", [1_000_000, 0], [0, 1_000_000]);
        invalid.observations[1] = invalid.observations[0].clone();
        assert_eq!(
            exact_evsi(&problem, &invalid, 0),
            Err(EvsiError::DuplicateIdentifier)
        );
    }

    #[test]
    fn duplicate_action_ids_are_rejected() {
        let mut problem = two_state_problem();
        problem.actions[1] = problem.actions[0].clone();
        assert_eq!(
            problem.validate(),
            Err(EvsiError::DuplicateIdentifier)
        );
    }

    #[test]
    fn duplicate_candidate_ids_are_rejected() {
        let design_a = design("same", [900_000, 100_000], [100_000, 900_000]);
        let design_b = design("same", [700_000, 300_000], [300_000, 700_000]);
        let candidates = vec![
            ObservationCandidate {
                id: "same".to_owned(),
                design: design_a,
                heuristic_avoidable_loss_lower_bound_units: 10,
                observation_cost_units: 0,
                compute_cost_units: 1,
            },
            ObservationCandidate {
                id: "same".to_owned(),
                design: design_b,
                heuristic_avoidable_loss_lower_bound_units: 20,
                observation_cost_units: 0,
                compute_cost_units: 1,
            },
        ];
        assert_eq!(
            rank_observation_candidates(&two_state_problem(), &candidates),
            Err(EvsiError::DuplicateIdentifier)
        );
    }

    #[test]
    fn probability_and_loss_precision_is_integer_exact_at_u64_boundaries() {
        let problem = DecisionProblem {
            states: vec![
                WeightedState { id: "a".to_owned(), prior_probability_ppm: 500_000 },
                WeightedState { id: "b".to_owned(), prior_probability_ppm: 500_000 },
            ],
            actions: vec!["left".to_owned(), "right".to_owned()],
            losses: vec![
                vec![u64::MAX, 0],
                vec![0, u64::MAX],
            ],
        };
        let noisy = design("noisy", [900_000, 100_000], [100_000, 900_000]);
        let result = exact_evsi(&problem, &noisy, 0).unwrap();
        assert!(result.evsi_numerator_ppm2 > 0);
        assert!(result.evsi_numerator_ppm2 <= result.evpi_numerator_ppm2);
    }

    #[test]
    fn comparison_of_exact_value_densities_does_not_overflow_u128_cross_products() {
        let left = compare_positive_fractions(u128::MAX - 1, u64::MAX, u128::MAX, u64::MAX - 1);
        assert_eq!(left, Ordering::Less);
    }
}
