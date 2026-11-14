// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Constrained investigation-policy tournament scoring.
//!
//! This module scores submitted investigation plans against a frozen action
//! catalogue, common budget and adjudicated decision-loss table. It does not
//! calculate EVSI, generate survey actions, authenticate permits, establish
//! truth, or authorize field work. An external planner may submit its action
//! choices and predicted information-gain estimates; the evaluator scores
//! realized decision loss and separately reports the submitted information
//! gain so entropy is not mistaken for decision value.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DisturbanceClass {
    Low,
    Moderate,
    High,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ActionAuthority {
    pub land_access_authorized: bool,
    pub method_authorized: bool,
    pub required_permits_satisfied: bool,
    pub rights_satisfied: bool,
    pub safety_review_satisfied: bool,
    pub custodial_review_required: bool,
    pub custodial_review_completed: bool,
}

impl ActionAuthority {
    pub fn is_eligible(self) -> bool {
        self.land_access_authorized
            && self.method_authorized
            && self.required_permits_satisfied
            && self.rights_satisfied
            && self.safety_review_satisfied
            && (!self.custodial_review_required || self.custodial_review_completed)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InvestigationBudget {
    pub currency: String,
    pub max_cost: f64,
    pub max_crew_hours: f64,
    pub maximum_disturbance: DisturbanceClass,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InvestigationAction {
    pub action_id: String,
    pub support_id: String,
    pub modality: String,
    pub currency: String,
    pub direct_cost: f64,
    pub crew_hours: f64,
    pub disturbance: DisturbanceClass,
    /// Planner-supplied normalized estimate in [0,1]. Diagnostic only:
    /// this is not an EVSI value and is not used to rank policy outcomes.
    pub predicted_information_gain: Option<f64>,
    pub authority: ActionAuthority,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InvestigationPolicyKind {
    Expert,
    FixedGrid,
    RandomEligible,
    CostOnly,
    EntropyGreedy,
    ExpectedValueOfInformation,
    Learned,
    OracleUpperBound,
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InvestigationPolicyPlan {
    pub policy_id: String,
    pub policy_kind: InvestigationPolicyKind,
    /// Oracle plans are diagnostic upper bounds only and must never be
    /// described as deployable policies.
    pub is_oracle_upper_bound: bool,
    pub selected_action_ids: Vec<String>,
    pub chosen_decision_id: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionLossEntry {
    pub truth_state_id: String,
    pub decision_id: String,
    pub loss: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InvestigationTournamentCase {
    pub evaluation_id: String,
    pub dataset_snapshot_id: String,
    /// Hidden from the policy during plan generation; read by the evaluator
    /// only after submissions are frozen in a real blind tournament.
    pub adjudicated_truth_state_id: String,
    pub budget: InvestigationBudget,
    pub action_catalogue: Vec<InvestigationAction>,
    pub decision_losses: Vec<DecisionLossEntry>,
    pub policy_plans: Vec<InvestigationPolicyPlan>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InvestigationTournamentError {
    EmptyIdentifier(&'static str),
    InvalidBudget,
    InvalidActionCost(String),
    InvalidCrewHours(String),
    InvalidInformationGain(String),
    CurrencyMismatch(String),
    DuplicateAction(String),
    DuplicatePolicy(String),
    DuplicateDecisionLoss(String),
    MissingDecisionLoss(String),
    InvalidDecisionLoss(String),
    UnknownAction(String, String),
    DuplicateSelectedAction(String, String),
    IneligibleActionSelected(String, String),
    DisturbanceLimitExceeded(String, String),
    CostBudgetExceeded(String),
    CrewBudgetExceeded(String),
    OracleFlagMismatch(String),
    EmptyActionCatalogue,
    EmptyPolicySet,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InvestigationPolicyMetrics {
    pub policy_id: String,
    pub policy_kind: InvestigationPolicyKind,
    pub is_oracle_upper_bound: bool,
    pub selected_action_count: u64,
    pub total_cost: f64,
    pub total_crew_hours: f64,
    pub highest_disturbance: Option<DisturbanceClass>,
    /// Sum of planner-supplied normalized information-gain estimates.
    /// It is a diagnostic only and is not a decision-value metric.
    pub submitted_information_gain_sum: Option<f64>,
    pub chosen_decision_id: String,
    pub realized_decision_loss: f64,
    pub oracle_minimum_decision_loss: f64,
    pub realized_decision_regret: f64,
    pub budget_cost_utilization: f64,
    pub budget_crew_utilization: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InvestigationTournamentReceipt {
    pub benchmark_id: String,
    pub evaluation_id: String,
    pub dataset_snapshot_id: String,
    pub status: String,
    pub common_budget: InvestigationBudget,
    pub policy_metrics: Vec<InvestigationPolicyMetrics>,
    pub claim_boundary: String,
}

fn non_empty(value: &str) -> bool {
    !value.trim().is_empty()
}

fn valid_non_negative(value: f64) -> bool {
    value.is_finite() && value >= 0.0
}

fn action_authority_errors(
    action: &InvestigationAction,
    policy_id: &str,
    errors: &mut Vec<InvestigationTournamentError>,
) {
    if !action.authority.is_eligible() {
        errors.push(InvestigationTournamentError::IneligibleActionSelected(
            policy_id.to_string(),
            action.action_id.clone(),
        ));
    }
}

/// Validate one frozen case and all policy submissions against one shared
/// action catalogue and one shared cost/crew/disturbance budget.
pub fn validate_investigation_tournament(
    case: &InvestigationTournamentCase,
) -> Vec<InvestigationTournamentError> {
    let mut errors = Vec::new();

    for (field, value) in [
        ("evaluation_id", case.evaluation_id.as_str()),
        ("dataset_snapshot_id", case.dataset_snapshot_id.as_str()),
        (
            "adjudicated_truth_state_id",
            case.adjudicated_truth_state_id.as_str(),
        ),
        ("budget_currency", case.budget.currency.as_str()),
    ] {
        if !non_empty(value) {
            errors.push(InvestigationTournamentError::EmptyIdentifier(field));
        }
    }

    if !valid_non_negative(case.budget.max_cost) || !valid_non_negative(case.budget.max_crew_hours)
    {
        errors.push(InvestigationTournamentError::InvalidBudget);
    }

    if case.action_catalogue.is_empty() {
        errors.push(InvestigationTournamentError::EmptyActionCatalogue);
    }
    if case.policy_plans.is_empty() {
        errors.push(InvestigationTournamentError::EmptyPolicySet);
    }

    let mut actions = BTreeMap::new();
    for action in &case.action_catalogue {
        if !non_empty(&action.action_id)
            || !non_empty(&action.support_id)
            || !non_empty(&action.modality)
        {
            errors.push(InvestigationTournamentError::EmptyIdentifier(
                "action_identity_or_support",
            ));
        }
        if actions.insert(action.action_id.as_str(), action).is_some() {
            errors.push(InvestigationTournamentError::DuplicateAction(
                action.action_id.clone(),
            ));
        }
        if !valid_non_negative(action.direct_cost) {
            errors.push(InvestigationTournamentError::InvalidActionCost(
                action.action_id.clone(),
            ));
        }
        if !valid_non_negative(action.crew_hours) {
            errors.push(InvestigationTournamentError::InvalidCrewHours(
                action.action_id.clone(),
            ));
        }
        if action.currency != case.budget.currency {
            errors.push(InvestigationTournamentError::CurrencyMismatch(
                action.action_id.clone(),
            ));
        }
        if let Some(gain) = action.predicted_information_gain {
            if !gain.is_finite() || !(0.0..=1.0).contains(&gain) {
                errors.push(InvestigationTournamentError::InvalidInformationGain(
                    action.action_id.clone(),
                ));
            }
        }
    }

    let mut decision_losses = BTreeMap::new();
    for loss in &case.decision_losses {
        if !non_empty(&loss.truth_state_id) || !non_empty(&loss.decision_id) {
            errors.push(InvestigationTournamentError::EmptyIdentifier(
                "decision_loss_identity",
            ));
        }
        if !valid_non_negative(loss.loss) {
            errors.push(InvestigationTournamentError::InvalidDecisionLoss(
                loss.decision_id.clone(),
            ));
        }
        let key = (loss.truth_state_id.as_str(), loss.decision_id.as_str());
        if decision_losses.insert(key, loss.loss).is_some() {
            errors.push(InvestigationTournamentError::DuplicateDecisionLoss(
                loss.decision_id.clone(),
            ));
        }
    }

    let truth_has_decision = case
        .decision_losses
        .iter()
        .any(|entry| entry.truth_state_id == case.adjudicated_truth_state_id);
    if !truth_has_decision {
        errors.push(InvestigationTournamentError::MissingDecisionLoss(
            case.adjudicated_truth_state_id.clone(),
        ));
    }

    let mut policy_ids = BTreeSet::new();
    for plan in &case.policy_plans {
        if !non_empty(&plan.policy_id) || !non_empty(&plan.chosen_decision_id) {
            errors.push(InvestigationTournamentError::EmptyIdentifier(
                "policy_or_decision_id",
            ));
        }
        if !policy_ids.insert(plan.policy_id.as_str()) {
            errors.push(InvestigationTournamentError::DuplicatePolicy(
                plan.policy_id.clone(),
            ));
        }
        if (plan.policy_kind == InvestigationPolicyKind::OracleUpperBound)
            != plan.is_oracle_upper_bound
        {
            errors.push(InvestigationTournamentError::OracleFlagMismatch(
                plan.policy_id.clone(),
            ));
        }

        if !decision_losses.contains_key((
            case.adjudicated_truth_state_id.as_str(),
            plan.chosen_decision_id.as_str(),
        )) {
            errors.push(InvestigationTournamentError::MissingDecisionLoss(
                plan.chosen_decision_id.clone(),
            ));
        }

        let mut selected = BTreeSet::new();
        let mut total_cost = 0.0;
        let mut total_crew_hours = 0.0;
        let mut highest_disturbance = None;

        for action_id in &plan.selected_action_ids {
            if !selected.insert(action_id.as_str()) {
                errors.push(InvestigationTournamentError::DuplicateSelectedAction(
                    plan.policy_id.clone(),
                    action_id.clone(),
                ));
                continue;
            }
            let Some(action) = actions.get(action_id.as_str()) else {
                errors.push(InvestigationTournamentError::UnknownAction(
                    plan.policy_id.clone(),
                    action_id.clone(),
                ));
                continue;
            };

            action_authority_errors(action, &plan.policy_id, &mut errors);
            if action.disturbance > case.budget.maximum_disturbance {
                errors.push(InvestigationTournamentError::DisturbanceLimitExceeded(
                    plan.policy_id.clone(),
                    action.action_id.clone(),
                ));
            }
            total_cost += action.direct_cost;
            total_crew_hours += action.crew_hours;
            highest_disturbance = Some(
                highest_disturbance.map_or(action.disturbance, |current: DisturbanceClass| {
                    current.max(action.disturbance)
                }),
            );
        }

        if !total_cost.is_finite() || total_cost > case.budget.max_cost + f64::EPSILON {
            errors.push(InvestigationTournamentError::CostBudgetExceeded(
                plan.policy_id.clone(),
            ));
        }
        if !total_crew_hours.is_finite()
            || total_crew_hours > case.budget.max_crew_hours + f64::EPSILON
        {
            errors.push(InvestigationTournamentError::CrewBudgetExceeded(
                plan.policy_id.clone(),
            ));
        }

        let _ = highest_disturbance;
    }

    errors
}

/// Score policy submissions after the shared case/budget has been validated.
/// Lower realized loss/regret is better. Information gain is reported only as
/// a diagnostic so a high-entropy action cannot stand in for decision value.
pub fn score_investigation_tournament(
    case: &InvestigationTournamentCase,
) -> Result<InvestigationTournamentReceipt, Vec<InvestigationTournamentError>> {
    let errors = validate_investigation_tournament(case);
    if !errors.is_empty() {
        return Err(errors);
    }

    let oracle_minimum_decision_loss = case
        .decision_losses
        .iter()
        .filter(|entry| entry.truth_state_id == case.adjudicated_truth_state_id)
        .map(|entry| entry.loss)
        .fold(f64::INFINITY, f64::min);

    let actions: BTreeMap<&str, &InvestigationAction> = case
        .action_catalogue
        .iter()
        .map(|action| (action.action_id.as_str(), action))
        .collect();

    let mut policy_metrics = Vec::with_capacity(case.policy_plans.len());
    for plan in &case.policy_plans {
        let mut total_cost = 0.0;
        let mut total_crew_hours = 0.0;
        let mut highest_disturbance = None;
        let mut information_gain_sum = 0.0;
        let mut information_gain_complete = true;

        for action_id in &plan.selected_action_ids {
            let action = actions[action_id.as_str()];
            total_cost += action.direct_cost;
            total_crew_hours += action.crew_hours;
            highest_disturbance = Some(
                highest_disturbance.map_or(action.disturbance, |current: DisturbanceClass| {
                    current.max(action.disturbance)
                }),
            );
            match action.predicted_information_gain {
                Some(value) => information_gain_sum += value,
                None => information_gain_complete = false,
            }
        }

        let realized_decision_loss = case
            .decision_losses
            .iter()
            .find(|entry| {
                entry.truth_state_id == case.adjudicated_truth_state_id
                    && entry.decision_id == plan.chosen_decision_id
            })
            .expect("validated decision loss exists")
            .loss;

        policy_metrics.push(InvestigationPolicyMetrics {
            policy_id: plan.policy_id.clone(),
            policy_kind: plan.policy_kind,
            is_oracle_upper_bound: plan.is_oracle_upper_bound,
            selected_action_count: plan.selected_action_ids.len() as u64,
            total_cost,
            total_crew_hours,
            highest_disturbance,
            submitted_information_gain_sum: information_gain_complete
                .then_some(information_gain_sum),
            chosen_decision_id: plan.chosen_decision_id.clone(),
            realized_decision_loss,
            oracle_minimum_decision_loss,
            realized_decision_regret: (realized_decision_loss - oracle_minimum_decision_loss)
                .max(0.0),
            budget_cost_utilization: if case.budget.max_cost > 0.0 {
                total_cost / case.budget.max_cost
            } else {
                0.0
            },
            budget_crew_utilization: if case.budget.max_crew_hours > 0.0 {
                total_crew_hours / case.budget.max_crew_hours
            } else {
                0.0
            },
        });
    }

    Ok(InvestigationTournamentReceipt {
        benchmark_id: "EARTH-OBSERVABILITY-INVESTIGATION-TOURNAMENT-001".to_string(),
        evaluation_id: case.evaluation_id.clone(),
        dataset_snapshot_id: case.dataset_snapshot_id.clone(),
        status: "synthetic_policy_scoring_not_evsi_or_physical_qualification".to_string(),
        common_budget: case.budget.clone(),
        policy_metrics,
        claim_boundary: "Scores frozen policy submissions against a synthetic adjudicated decision-loss table and common action catalogue/budget. It does not calculate EVSI, prove policy blindness, authenticate access/permits, validate the truth, or authorize field work.".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = include_str!(
        "../../research/earth-observability/fixtures/investigation-tournament-synthetic-v0.1.json"
    );

    fn fixture() -> InvestigationTournamentCase {
        serde_json::from_str(FIXTURE).expect("synthetic tournament fixture parses")
    }

    #[test]
    fn high_information_gain_does_not_automatically_win_on_decision_loss() {
        let receipt = score_investigation_tournament(&fixture()).unwrap();
        let entropy = receipt
            .policy_metrics
            .iter()
            .find(|metric| metric.policy_id == "entropy-greedy")
            .unwrap();
        let decision = receipt
            .policy_metrics
            .iter()
            .find(|metric| metric.policy_id == "decision-aware")
            .unwrap();
        assert!(
            entropy.submitted_information_gain_sum.unwrap()
                > decision.submitted_information_gain_sum.unwrap()
        );
        assert!(entropy.realized_decision_loss > decision.realized_decision_loss);
        assert!(entropy.realized_decision_regret > decision.realized_decision_regret);
    }

    #[test]
    fn action_authority_is_a_hard_constraint_not_a_soft_penalty() {
        let mut case = fixture();
        case.policy_plans[0].selected_action_ids = vec!["prohibited-trial-hole".to_string()];
        assert!(validate_investigation_tournament(&case).contains(
            &InvestigationTournamentError::IneligibleActionSelected(
                "entropy-greedy".to_string(),
                "prohibited-trial-hole".to_string()
            )
        ));
    }

    #[test]
    fn common_budget_rejects_overspending() {
        let mut case = fixture();
        case.policy_plans[0].selected_action_ids = vec![
            "wide-area-gpr".to_string(),
            "targeted-crossline-em".to_string(),
        ];
        assert!(validate_investigation_tournament(&case).contains(
            &InvestigationTournamentError::CostBudgetExceeded("entropy-greedy".to_string())
        ));
    }

    #[test]
    fn disturbance_limit_is_not_traded_for_information_gain() {
        let mut case = fixture();
        case.budget.maximum_disturbance = DisturbanceClass::Moderate;
        case.action_catalogue
            .iter_mut()
            .find(|action| action.action_id == "targeted-crossline-em")
            .unwrap()
            .disturbance = DisturbanceClass::High;
        case.policy_plans[1].selected_action_ids = vec!["targeted-crossline-em".to_string()];
        assert!(validate_investigation_tournament(&case).contains(
            &InvestigationTournamentError::DisturbanceLimitExceeded(
                "decision-aware".to_string(),
                "targeted-crossline-em".to_string()
            )
        ));
    }

    #[test]
    fn oracle_is_reported_as_upper_bound_not_a_deployable_policy() {
        let receipt = score_investigation_tournament(&fixture()).unwrap();
        let oracle = receipt
            .policy_metrics
            .iter()
            .find(|metric| metric.policy_id == "hidden-truth-oracle")
            .unwrap();
        assert!(oracle.is_oracle_upper_bound);
        assert_eq!(oracle.realized_decision_regret, 0.0);
    }

    #[test]
    fn unknown_actions_and_missing_decision_loss_are_rejected() {
        let mut case = fixture();
        case.policy_plans[0].selected_action_ids = vec!["not-in-catalogue".to_string()];
        case.policy_plans[0].chosen_decision_id = "not-in-loss-table".to_string();
        let errors = validate_investigation_tournament(&case);
        assert!(
            errors.contains(&InvestigationTournamentError::UnknownAction(
                "entropy-greedy".to_string(),
                "not-in-catalogue".to_string()
            ))
        );
        assert!(
            errors.contains(&InvestigationTournamentError::MissingDecisionLoss(
                "not-in-loss-table".to_string()
            ))
        );
    }
}
