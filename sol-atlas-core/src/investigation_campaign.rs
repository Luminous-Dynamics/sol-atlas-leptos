// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Project-level aggregation for constrained investigation tournaments.
//!
//! This preserves per-project outcomes and reports macro-average and
//! worst-project loss/regret. It refuses comparisons with different budgets,
//! dataset snapshots, or policy sets. It does not treat project folds as
//! statistically independent replications of a universal population.

use crate::investigation_tournament::{
    InvestigationBudget, InvestigationPolicyKind, InvestigationTournamentCase,
    InvestigationTournamentError, score_investigation_tournament,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectInvestigationCase {
    pub project_id: String,
    pub case: InvestigationTournamentCase,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InvestigationTournamentCampaign {
    pub campaign_id: String,
    pub dataset_snapshot_id: String,
    pub common_budget: InvestigationBudget,
    pub project_cases: Vec<ProjectInvestigationCase>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InvestigationCampaignError {
    EmptyIdentifier(&'static str),
    TooFewProjectCases,
    DuplicateProject(String),
    DuplicateEvaluation(String),
    DatasetSnapshotMismatch(String),
    BudgetMismatch(String),
    PolicySetMismatch(String),
    PolicyKindMismatch(String),
    InvalidProjectCase(String, Vec<InvestigationTournamentError>),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectPolicyResult {
    pub project_id: String,
    pub policy_id: String,
    pub realized_decision_loss: f64,
    pub realized_decision_regret: f64,
    pub total_cost: f64,
    pub total_crew_hours: f64,
    pub submitted_information_gain_sum: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CampaignPolicyMetrics {
    pub policy_id: String,
    pub policy_kind: InvestigationPolicyKind,
    pub is_oracle_upper_bound: bool,
    pub project_count: u64,
    /// Unweighted project macro-average; never radargram/activity weighted.
    pub macro_mean_decision_loss: f64,
    pub macro_mean_decision_regret: f64,
    pub worst_project_decision_loss: f64,
    pub worst_project_loss_id: String,
    pub worst_project_decision_regret: f64,
    pub worst_project_regret_id: String,
    pub macro_mean_cost: f64,
    pub maximum_project_cost: f64,
    pub macro_mean_crew_hours: f64,
    pub maximum_project_crew_hours: f64,
    pub macro_mean_information_gain: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InvestigationCampaignReceipt {
    pub benchmark_id: String,
    pub campaign_id: String,
    pub dataset_snapshot_id: String,
    pub project_count: u64,
    pub status: String,
    pub common_budget: InvestigationBudget,
    pub policy_metrics: Vec<CampaignPolicyMetrics>,
    /// Per-project results remain in the receipt so macro averages cannot
    /// conceal one failed geography or acquisition domain.
    pub project_results: Vec<ProjectPolicyResult>,
    pub claim_boundary: String,
}

fn non_empty(value: &str) -> bool {
    !value.trim().is_empty()
}

fn same_budget(left: &InvestigationBudget, right: &InvestigationBudget) -> bool {
    left.currency == right.currency
        && left.max_cost == right.max_cost
        && left.max_crew_hours == right.max_crew_hours
        && left.maximum_disturbance == right.maximum_disturbance
}

/// Validate project identities, snapshot/budget equality and identical policy
/// coverage before any macro-average is emitted.
pub fn validate_investigation_campaign(
    campaign: &InvestigationTournamentCampaign,
) -> Result<(), Vec<InvestigationCampaignError>> {
    let mut errors = Vec::new();

    if !non_empty(&campaign.campaign_id) {
        errors.push(InvestigationCampaignError::EmptyIdentifier("campaign_id"));
    }
    if !non_empty(&campaign.dataset_snapshot_id) {
        errors.push(InvestigationCampaignError::EmptyIdentifier(
            "dataset_snapshot_id",
        ));
    }
    if campaign.project_cases.len() < 2 {
        errors.push(InvestigationCampaignError::TooFewProjectCases);
    }

    let mut project_ids = BTreeSet::new();
    let mut evaluation_ids = BTreeSet::new();
    let mut reference_policy_ids: Option<BTreeSet<String>> = None;
    let mut reference_policy_kinds: BTreeMap<String, (InvestigationPolicyKind, bool)> =
        BTreeMap::new();

    for project_case in &campaign.project_cases {
        if !non_empty(&project_case.project_id) {
            errors.push(InvestigationCampaignError::EmptyIdentifier("project_id"));
        }
        if !project_ids.insert(project_case.project_id.clone()) {
            errors.push(InvestigationCampaignError::DuplicateProject(
                project_case.project_id.clone(),
            ));
        }
        if !evaluation_ids.insert(project_case.case.evaluation_id.clone()) {
            errors.push(InvestigationCampaignError::DuplicateEvaluation(
                project_case.case.evaluation_id.clone(),
            ));
        }
        if project_case.case.dataset_snapshot_id != campaign.dataset_snapshot_id {
            errors.push(InvestigationCampaignError::DatasetSnapshotMismatch(
                project_case.project_id.clone(),
            ));
        }
        if !same_budget(&project_case.case.budget, &campaign.common_budget) {
            errors.push(InvestigationCampaignError::BudgetMismatch(
                project_case.project_id.clone(),
            ));
        }

        if let Err(case_errors) =
            crate::investigation_tournament::validate_investigation_tournament(&project_case.case)
        {
            errors.push(InvestigationCampaignError::InvalidProjectCase(
                project_case.project_id.clone(),
                case_errors,
            ));
        }

        let current_ids: BTreeSet<String> = project_case
            .case
            .policy_plans
            .iter()
            .map(|plan| plan.policy_id.clone())
            .collect();
        match &reference_policy_ids {
            None => {
                reference_policy_ids = Some(current_ids.clone());
                for plan in &project_case.case.policy_plans {
                    reference_policy_kinds.insert(
                        plan.policy_id.clone(),
                        (plan.policy_kind, plan.is_oracle_upper_bound),
                    );
                }
            }
            Some(expected) if expected != &current_ids => {
                errors.push(InvestigationCampaignError::PolicySetMismatch(
                    project_case.project_id.clone(),
                ));
            }
            Some(_) => {
                for plan in &project_case.case.policy_plans {
                    if reference_policy_kinds.get(&plan.policy_id)
                        != Some(&(plan.policy_kind, plan.is_oracle_upper_bound))
                    {
                        errors.push(InvestigationCampaignError::PolicyKindMismatch(
                            plan.policy_id.clone(),
                        ));
                    }
                }
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Score all projects under identical policy/budget conditions. Each project
/// receives equal weight irrespective of number of traces, radargrams,
/// activities or events. Worst-project metrics are mandatory outputs.
pub fn score_investigation_campaign(
    campaign: &InvestigationTournamentCampaign,
) -> Result<InvestigationCampaignReceipt, Vec<InvestigationCampaignError>> {
    validate_investigation_campaign(campaign)?;

    let mut project_results = Vec::new();
    let mut per_policy: BTreeMap<
        String,
        Vec<(InvestigationPolicyKind, bool, ProjectPolicyResult)>,
    > = BTreeMap::new();

    for project_case in &campaign.project_cases {
        let case_receipt =
            score_investigation_tournament(&project_case.case).map_err(|errors| {
                vec![InvestigationCampaignError::InvalidProjectCase(
                    project_case.project_id.clone(),
                    errors,
                )]
            })?;

        for metrics in case_receipt.policy_metrics {
            let result = ProjectPolicyResult {
                project_id: project_case.project_id.clone(),
                policy_id: metrics.policy_id.clone(),
                realized_decision_loss: metrics.realized_decision_loss,
                realized_decision_regret: metrics.realized_decision_regret,
                total_cost: metrics.total_cost,
                total_crew_hours: metrics.total_crew_hours,
                submitted_information_gain_sum: metrics.submitted_information_gain_sum,
            };
            project_results.push(result.clone());
            per_policy.entry(metrics.policy_id).or_default().push((
                metrics.policy_kind,
                metrics.is_oracle_upper_bound,
                result,
            ));
        }
    }

    project_results.sort_by(|left, right| {
        left.policy_id
            .cmp(&right.policy_id)
            .then(left.project_id.cmp(&right.project_id))
    });

    let mut policy_metrics = Vec::new();
    for (policy_id, rows) in per_policy {
        let project_count = rows.len() as f64;
        let (policy_kind, is_oracle_upper_bound, _) = &rows[0];

        let macro_mean_decision_loss = rows
            .iter()
            .map(|(_, _, row)| row.realized_decision_loss)
            .sum::<f64>()
            / project_count;
        let macro_mean_decision_regret = rows
            .iter()
            .map(|(_, _, row)| row.realized_decision_regret)
            .sum::<f64>()
            / project_count;
        let macro_mean_cost =
            rows.iter().map(|(_, _, row)| row.total_cost).sum::<f64>() / project_count;
        let macro_mean_crew_hours = rows
            .iter()
            .map(|(_, _, row)| row.total_crew_hours)
            .sum::<f64>()
            / project_count;

        let (worst_loss_id, worst_loss) = rows
            .iter()
            .map(|(_, _, row)| (row.project_id.clone(), row.realized_decision_loss))
            .max_by(|left, right| {
                left.1
                    .total_cmp(&right.1)
                    .then_with(|| right.0.cmp(&left.0))
            })
            .expect("validated policy has project rows");
        let (worst_regret_id, worst_regret) = rows
            .iter()
            .map(|(_, _, row)| (row.project_id.clone(), row.realized_decision_regret))
            .max_by(|left, right| {
                left.1
                    .total_cmp(&right.1)
                    .then_with(|| right.0.cmp(&left.0))
            })
            .expect("validated policy has project rows");

        let information_gain_values: Vec<f64> = rows
            .iter()
            .filter_map(|(_, _, row)| row.submitted_information_gain_sum)
            .collect();
        let macro_mean_information_gain = (information_gain_values.len() == rows.len())
            .then(|| information_gain_values.iter().sum::<f64>() / project_count);

        policy_metrics.push(CampaignPolicyMetrics {
            policy_id,
            policy_kind: *policy_kind,
            is_oracle_upper_bound: *is_oracle_upper_bound,
            project_count: rows.len() as u64,
            macro_mean_decision_loss,
            macro_mean_decision_regret,
            worst_project_decision_loss: worst_loss,
            worst_project_loss_id: worst_loss_id,
            worst_project_decision_regret: worst_regret,
            worst_project_regret_id: worst_regret_id,
            macro_mean_cost,
            maximum_project_cost: rows
                .iter()
                .map(|(_, _, row)| row.total_cost)
                .fold(0.0_f64, f64::max),
            macro_mean_crew_hours,
            maximum_project_crew_hours: rows
                .iter()
                .map(|(_, _, row)| row.total_crew_hours)
                .fold(0.0_f64, f64::max),
            macro_mean_information_gain,
        });
    }

    Ok(InvestigationCampaignReceipt {
        benchmark_id: "EARTH-OBSERVABILITY-INVESTIGATION-CAMPAIGN-001".to_string(),
        campaign_id: campaign.campaign_id.clone(),
        dataset_snapshot_id: campaign.dataset_snapshot_id.clone(),
        project_count: campaign.project_cases.len() as u64,
        status: "synthetic_project_macro_aggregation_not_physical_qualification".to_string(),
        common_budget: campaign.common_budget.clone(),
        policy_metrics,
        project_results,
        claim_boundary: "Project-macro aggregation over frozen synthetic or admitted project cases with identical policy sets and budgets. Per-project results are retained and worst-project outcomes reported. It does not establish statistical independence, physical truth, policy blindness, EVSI calibration or field performance.".to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = include_str!(
        "../../research/earth-observability/fixtures/investigation-campaign-synthetic-v0.1.json"
    );

    fn fixture() -> InvestigationTournamentCampaign {
        serde_json::from_str(FIXTURE).expect("campaign fixture parses")
    }

    #[test]
    fn project_campaign_preserves_every_project_and_worst_case() {
        let receipt = score_investigation_campaign(&fixture()).unwrap();
        assert_eq!(receipt.project_count, 3);
        assert_eq!(receipt.project_results.len(), 15);
        let entropy = receipt
            .policy_metrics
            .iter()
            .find(|metric| metric.policy_id == "entropy-greedy")
            .unwrap();
        assert_eq!(entropy.project_count, 3);
        assert!((entropy.macro_mean_decision_loss - 80.0).abs() < 1e-12);
        assert_eq!(entropy.worst_project_decision_loss, 100.0);
        assert_eq!(entropy.worst_project_loss_id, "project-a");
    }

    #[test]
    fn project_macro_average_weights_projects_not_event_counts() {
        let mut campaign = fixture();
        let truth_state_id = campaign.project_cases[1]
            .case
            .adjudicated_truth_state_id
            .clone();
        campaign.project_cases[1]
            .case
            .decision_losses
            .iter_mut()
            .find(|entry| {
                entry.truth_state_id == truth_state_id && entry.decision_id == "declare-junction"
            })
            .unwrap()
            .loss = 10.0;
        let receipt = score_investigation_campaign(&campaign).unwrap();
        let entropy = receipt
            .policy_metrics
            .iter()
            .find(|metric| metric.policy_id == "entropy-greedy")
            .unwrap();
        assert!((entropy.macro_mean_decision_loss - (170.0 / 3.0)).abs() < 1e-12);
    }

    #[test]
    fn campaigns_reject_different_budgets() {
        let mut campaign = fixture();
        campaign.project_cases[0].case.budget.max_cost = 151.0;
        assert!(
            validate_investigation_campaign(&campaign)
                .unwrap_err()
                .contains(&InvestigationCampaignError::BudgetMismatch(
                    "project-a".to_string()
                ))
        );
    }

    #[test]
    fn campaigns_reject_policy_set_drift() {
        let mut campaign = fixture();
        campaign.project_cases[0].case.policy_plans.pop();
        assert!(
            validate_investigation_campaign(&campaign)
                .unwrap_err()
                .contains(&InvestigationCampaignError::PolicySetMismatch(
                    "project-a".to_string()
                ))
        );
    }

    #[test]
    fn campaign_requires_distinct_project_and_evaluation_identities() {
        let mut campaign = fixture();
        campaign.project_cases[1].project_id = "project-a".to_string();
        assert!(
            validate_investigation_campaign(&campaign)
                .unwrap_err()
                .contains(&InvestigationCampaignError::DuplicateProject(
                    "project-a".to_string()
                ))
        );
    }
}
