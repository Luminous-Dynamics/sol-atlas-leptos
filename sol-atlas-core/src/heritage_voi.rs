// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Explicit Bayesian expected-information-gain planning for non-invasive heritage surveys.
//! This module consumes supplied priors/likelihoods; it never invents them or
//! authorizes land access, permits, excavation, or physical disturbance.

use crate::heritage::{authorize_rights_and_geometry, AccessContext, AccessTier, HeritageUse, RightsDenial, RightsProfile};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

pub const PROBABILITY_TOLERANCE: f64 = 1e-9;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HypothesisPrior {
    pub hypothesis_id: String,
    pub probability: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SurveyOutcomeModel {
    pub outcome_id: String,
    pub likelihood_by_hypothesis: BTreeMap<String, f64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SurveyDataAuthorization {
    pub source_id: String,
    pub asset_ref: String,
    pub rights_profile: RightsProfile,
    pub geometry_access_tier: AccessTier,
    pub purpose: HeritageUse,
    pub access_context: AccessContext,
    pub exact_geometry: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SurveyAuthorization {
    pub source_data: Vec<SurveyDataAuthorization>,
    pub requires_land_access: bool,
    pub land_access_confirmed: bool,
    pub requires_permit: bool,
    pub permit_confirmed: bool,
    pub requires_custodial_review: bool,
    pub custodial_review_complete: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SurveyAction {
    pub action_id: String,
    pub expected_cost_units: f64,
    /// Declared impact score in [0, 1], not a claim of safety.
    pub impact_score: f64,
    pub authorization: SurveyAuthorization,
    pub outcomes: Vec<SurveyOutcomeModel>,
    /// Source or review reference for the likelihood model; never silently infer it.
    pub likelihood_basis_ref: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SurveyPlanningConstraints {
    pub max_cost_units: f64,
    pub max_impact_score: f64,
    pub probability_tolerance: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SurveyRecommendation {
    pub action_id: String,
    pub expected_information_gain_nats: f64,
    pub expected_posterior_entropy_nats: f64,
    pub expected_cost_units: f64,
    pub information_gain_per_cost: f64,
    pub impact_score: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SurveyRejectionReason {
    SourceRightsDenied(RightsDenial),
    LandAccessNotConfirmed,
    PermitNotConfirmed,
    CustodialReviewIncomplete,
    CostBudgetExceeded,
    ImpactCeilingExceeded,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RejectedSurveyAction {
    pub action_id: String,
    pub reason: SurveyRejectionReason,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SurveyPlan {
    pub prior_entropy_nats: f64,
    pub cost_unit: String,
    pub ranked_actions: Vec<SurveyRecommendation>,
    pub rejected_actions: Vec<RejectedSurveyAction>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SurveyPlannerError {
    EmptyHypotheses,
    EmptyHypothesisId,
    DuplicateHypothesisId(String),
    InvalidPriorProbability { hypothesis_id: String },
    PriorProbabilitiesDoNotSumToOne { sum: f64, tolerance: f64 },
    EmptyActionId,
    EmptySourceDataAuthorizations { action_id: String },
    EmptySourceId { action_id: String },
    EmptyAssetReference { action_id: String },
    DuplicateActionId(String),
    InvalidCost { action_id: String },
    InvalidImpactScore { action_id: String },
    EmptyLikelihoodBasis { action_id: String },
    EmptyOutcomes { action_id: String },
    EmptyOutcomeId { action_id: String },
    DuplicateOutcomeId { action_id: String, outcome_id: String },
    LikelihoodHypothesisIdsMismatch { action_id: String, outcome_id: String },
    InvalidLikelihood { action_id: String, outcome_id: String, hypothesis_id: String },
    LikelihoodsDoNotSumToOne { action_id: String, hypothesis_id: String, sum: f64, tolerance: f64 },
    InvalidMaxCost,
    InvalidMaxImpact,
    InvalidProbabilityTolerance,
    NegativeInformationGain { action_id: String, value: f64 },
}

impl fmt::Display for SurveyPlannerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "heritage survey planner input error: {self:?}")
    }
}

impl std::error::Error for SurveyPlannerError {}

fn entropy_nats(probabilities: impl Iterator<Item = f64>) -> f64 {
    probabilities
        .filter(|probability| *probability > 0.0)
        .map(|probability| -probability * probability.ln())
        .sum()
}

fn validate_priors(
    priors: &[HypothesisPrior],
    tolerance: f64,
) -> Result<BTreeMap<String, f64>, SurveyPlannerError> {
    if priors.is_empty() {
        return Err(SurveyPlannerError::EmptyHypotheses);
    }

    let mut by_id = BTreeMap::new();
    let mut sum = 0.0;
    for prior in priors {
        if prior.hypothesis_id.trim().is_empty() {
            return Err(SurveyPlannerError::EmptyHypothesisId);
        }
        if !prior.probability.is_finite() || !(0.0..=1.0).contains(&prior.probability) {
            return Err(SurveyPlannerError::InvalidPriorProbability {
                hypothesis_id: prior.hypothesis_id.clone(),
            });
        }
        if by_id
            .insert(prior.hypothesis_id.clone(), prior.probability)
            .is_some()
        {
            return Err(SurveyPlannerError::DuplicateHypothesisId(
                prior.hypothesis_id.clone(),
            ));
        }
        sum += prior.probability;
    }

    if (sum - 1.0).abs() > tolerance {
        return Err(SurveyPlannerError::PriorProbabilitiesDoNotSumToOne {
            sum,
            tolerance,
        });
    }
    // Normalize accepted floating-point summation drift before Bayesian updating.
    for probability in by_id.values_mut() {
        *probability /= sum;
    }
    Ok(by_id)
}

fn validate_action(
    action: &SurveyAction,
    priors: &BTreeMap<String, f64>,
    tolerance: f64,
) -> Result<(), SurveyPlannerError> {
    if action.action_id.trim().is_empty() {
        return Err(SurveyPlannerError::EmptyActionId);
    }
    if !action.expected_cost_units.is_finite() || action.expected_cost_units <= 0.0 {
        return Err(SurveyPlannerError::InvalidCost {
            action_id: action.action_id.clone(),
        });
    }
    if !action.impact_score.is_finite() || !(0.0..=1.0).contains(&action.impact_score) {
        return Err(SurveyPlannerError::InvalidImpactScore {
            action_id: action.action_id.clone(),
        });
    }
    if action.authorization.source_data.is_empty() {
        return Err(SurveyPlannerError::EmptySourceDataAuthorizations {
            action_id: action.action_id.clone(),
        });
    }
    for source in &action.authorization.source_data {
        if source.source_id.trim().is_empty() {
            return Err(SurveyPlannerError::EmptySourceId { action_id: action.action_id.clone() });
        }
        if source.asset_ref.trim().is_empty() {
            return Err(SurveyPlannerError::EmptyAssetReference { action_id: action.action_id.clone() });
        }
    }
    if action.likelihood_basis_ref.trim().is_empty() {
        return Err(SurveyPlannerError::EmptyLikelihoodBasis {
            action_id: action.action_id.clone(),
        });
    }
    if action.outcomes.is_empty() {
        return Err(SurveyPlannerError::EmptyOutcomes {
            action_id: action.action_id.clone(),
        });
    }

    let expected_ids: BTreeSet<&String> = priors.keys().collect();
    let mut outcome_ids = BTreeSet::new();
    let mut likelihood_sums: BTreeMap<String, f64> =
        priors.keys().map(|id| (id.clone(), 0.0)).collect();

    for outcome in &action.outcomes {
        if outcome.outcome_id.trim().is_empty() {
            return Err(SurveyPlannerError::EmptyOutcomeId {
                action_id: action.action_id.clone(),
            });
        }
        if !outcome_ids.insert(outcome.outcome_id.as_str()) {
            return Err(SurveyPlannerError::DuplicateOutcomeId {
                action_id: action.action_id.clone(),
                outcome_id: outcome.outcome_id.clone(),
            });
        }

        let actual_ids: BTreeSet<&String> = outcome.likelihood_by_hypothesis.keys().collect();
        if actual_ids != expected_ids {
            return Err(SurveyPlannerError::LikelihoodHypothesisIdsMismatch {
                action_id: action.action_id.clone(),
                outcome_id: outcome.outcome_id.clone(),
            });
        }

        for (hypothesis_id, likelihood) in &outcome.likelihood_by_hypothesis {
            if !likelihood.is_finite() || !(0.0..=1.0).contains(likelihood) {
                return Err(SurveyPlannerError::InvalidLikelihood {
                    action_id: action.action_id.clone(),
                    outcome_id: outcome.outcome_id.clone(),
                    hypothesis_id: hypothesis_id.clone(),
                });
            }
            *likelihood_sums
                .get_mut(hypothesis_id)
                .expect("validated hypothesis key must exist") += likelihood;
        }
    }

    for (hypothesis_id, sum) in likelihood_sums {
        if (sum - 1.0).abs() > tolerance {
            return Err(SurveyPlannerError::LikelihoodsDoNotSumToOne {
                action_id: action.action_id.clone(),
                hypothesis_id,
                sum,
                tolerance,
            });
        }
    }
    Ok(())
}

fn rejection_reason(
    action: &SurveyAction,
    constraints: SurveyPlanningConstraints,
) -> Option<SurveyRejectionReason> {
    for source in &action.authorization.source_data {
        if let Err(reason) = authorize_rights_and_geometry(
            &source.rights_profile,
            source.geometry_access_tier,
            source.purpose,
            source.access_context,
            source.exact_geometry,
        ) {
            return Some(SurveyRejectionReason::SourceRightsDenied(reason));
        }
    }
    if action.authorization.requires_land_access && !action.authorization.land_access_confirmed {
        return Some(SurveyRejectionReason::LandAccessNotConfirmed);
    }
    if action.authorization.requires_permit && !action.authorization.permit_confirmed {
        return Some(SurveyRejectionReason::PermitNotConfirmed);
    }
    if action.authorization.requires_custodial_review
        && !action.authorization.custodial_review_complete
    {
        return Some(SurveyRejectionReason::CustodialReviewIncomplete);
    }
    if action.expected_cost_units > constraints.max_cost_units {
        return Some(SurveyRejectionReason::CostBudgetExceeded);
    }
    if action.impact_score > constraints.max_impact_score {
        return Some(SurveyRejectionReason::ImpactCeilingExceeded);
    }
    None
}

/// Returns (EIG in nats, expected posterior entropy in nats).
pub fn expected_information_gain_nats(
    priors: &[HypothesisPrior],
    action: &SurveyAction,
    probability_tolerance: f64,
) -> Result<(f64, f64), SurveyPlannerError> {
    if !probability_tolerance.is_finite()
        || probability_tolerance <= 0.0
        || probability_tolerance > 1.0
    {
        return Err(SurveyPlannerError::InvalidProbabilityTolerance);
    }
    let prior_by_id = validate_priors(priors, probability_tolerance)?;
    validate_action(action, &prior_by_id, probability_tolerance)?;

    let prior_entropy = entropy_nats(prior_by_id.values().copied());
    let mut expected_posterior_entropy = 0.0;

    for outcome in &action.outcomes {
        let outcome_probability: f64 = prior_by_id
            .iter()
            .map(|(hypothesis_id, prior)| {
                *prior * outcome.likelihood_by_hypothesis[hypothesis_id]
            })
            .sum();

        if outcome_probability <= 0.0 {
            continue;
        }
        let posterior = prior_by_id.iter().map(|(hypothesis_id, prior)| {
            *prior * outcome.likelihood_by_hypothesis[hypothesis_id] / outcome_probability
        });
        expected_posterior_entropy += outcome_probability * entropy_nats(posterior);
    }

    let raw_information_gain = prior_entropy - expected_posterior_entropy;
    if raw_information_gain < -probability_tolerance {
        return Err(SurveyPlannerError::NegativeInformationGain {
            action_id: action.action_id.clone(),
            value: raw_information_gain,
        });
    }
    Ok((raw_information_gain.max(0.0), expected_posterior_entropy))
}

/// Rank eligible, explicitly modelled survey actions. Invalid probability
/// models fail closed. Ineligible actions are retained with a rejection reason.
/// No arbitrary weighted score combines information, cost and impact.
pub fn plan_non_invasive_surveys(
    priors: &[HypothesisPrior],
    actions: &[SurveyAction],
    constraints: SurveyPlanningConstraints,
    cost_unit: impl Into<String>,
) -> Result<SurveyPlan, SurveyPlannerError> {
    if !constraints.max_cost_units.is_finite() || constraints.max_cost_units <= 0.0 {
        return Err(SurveyPlannerError::InvalidMaxCost);
    }
    if !constraints.max_impact_score.is_finite()
        || !(0.0..=1.0).contains(&constraints.max_impact_score)
    {
        return Err(SurveyPlannerError::InvalidMaxImpact);
    }
    if !constraints.probability_tolerance.is_finite()
        || constraints.probability_tolerance <= 0.0
        || constraints.probability_tolerance > 1.0
    {
        return Err(SurveyPlannerError::InvalidProbabilityTolerance);
    }

    let prior_by_id = validate_priors(priors, constraints.probability_tolerance)?;
    let prior_entropy = entropy_nats(prior_by_id.values().copied());
    let mut seen_action_ids = BTreeSet::new();
    let mut ranked_actions = Vec::new();
    let mut rejected_actions = Vec::new();

    for action in actions {
        if action.action_id.trim().is_empty() {
            return Err(SurveyPlannerError::EmptyActionId);
        }
        if !seen_action_ids.insert(action.action_id.as_str()) {
            return Err(SurveyPlannerError::DuplicateActionId(action.action_id.clone()));
        }
        validate_action(action, &prior_by_id, constraints.probability_tolerance)?;

        if let Some(reason) = rejection_reason(action, constraints) {
            rejected_actions.push(RejectedSurveyAction {
                action_id: action.action_id.clone(),
                reason,
            });
            continue;
        }

        let (gain, posterior_entropy) =
            expected_information_gain_nats(priors, action, constraints.probability_tolerance)?;
        ranked_actions.push(SurveyRecommendation {
            action_id: action.action_id.clone(),
            expected_information_gain_nats: gain,
            expected_posterior_entropy_nats: posterior_entropy,
            expected_cost_units: action.expected_cost_units,
            information_gain_per_cost: gain / action.expected_cost_units,
            impact_score: action.impact_score,
        });
    }

    ranked_actions.sort_by(|left, right| {
        right
            .information_gain_per_cost
            .total_cmp(&left.information_gain_per_cost)
            .then_with(|| {
                right
                    .expected_information_gain_nats
                    .total_cmp(&left.expected_information_gain_nats)
            })
            .then_with(|| left.action_id.cmp(&right.action_id))
    });
    rejected_actions.sort_by(|left, right| left.action_id.cmp(&right.action_id));

    Ok(SurveyPlan {
        prior_entropy_nats: prior_entropy,
        cost_unit: cost_unit.into(),
        ranked_actions,
        rejected_actions,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn priors() -> Vec<HypothesisPrior> {
        vec![
            HypothesisPrior { hypothesis_id: "archaeological".into(), probability: 0.5 },
            HypothesisPrior { hypothesis_id: "geological".into(), probability: 0.5 },
        ]
    }

    fn authorization() -> SurveyAuthorization {
        let mut permitted_uses = BTreeSet::new();
        permitted_uses.insert(HeritageUse::Research);
        SurveyAuthorization {
            source_data: vec![SurveyDataAuthorization {
                source_id: "synthetic-source".into(),
                asset_ref: "urn:synthetic:asset:01".into(),
                rights_profile: RightsProfile {
                    state: crate::heritage::RightsState::Verified,
                    permitted_uses,
                    access_tier: AccessTier::ApprovedResearch,
                    exact_geometry_policy: crate::heritage::ExactGeometryPolicy::ApprovedResearchOnly,
                    licence_uri: Some("https://example.invalid/licence".into()),
                    attribution_text: None,
                    consent_or_authority_ref: Some("urn:synthetic:consent:01".into()),
                    jurisdiction: Some("synthetic".into()),
                    review_due_at: None,
                    restrictions: Vec::new(),
                },
                geometry_access_tier: AccessTier::ApprovedResearch,
                purpose: HeritageUse::Research,
                access_context: AccessContext { max_tier: AccessTier::ApprovedResearch, exact_geometry_authorized: true },
                exact_geometry: true,
            }],
            requires_land_access: true,
            land_access_confirmed: true,
            requires_permit: true,
            permit_confirmed: true,
            requires_custodial_review: true,
            custodial_review_complete: true,
        }
    }

    fn action(action_id: &str, cost: f64, impact: f64, likelihoods: [(f64, f64); 2]) -> SurveyAction {
        let mut positive = BTreeMap::new();
        positive.insert("archaeological".into(), likelihoods[0].0);
        positive.insert("geological".into(), likelihoods[0].1);
        let mut negative = BTreeMap::new();
        negative.insert("archaeological".into(), likelihoods[1].0);
        negative.insert("geological".into(), likelihoods[1].1);

        SurveyAction {
            action_id: action_id.into(),
            expected_cost_units: cost,
            impact_score: impact,
            authorization: authorization(),
            outcomes: vec![
                SurveyOutcomeModel { outcome_id: "positive".into(), likelihood_by_hypothesis: positive },
                SurveyOutcomeModel { outcome_id: "negative".into(), likelihood_by_hypothesis: negative },
            ],
            likelihood_basis_ref: "urn:synthetic:likelihoods:v1".into(),
        }
    }

    fn constraints() -> SurveyPlanningConstraints {
        SurveyPlanningConstraints {
            max_cost_units: 10.0,
            max_impact_score: 0.25,
            probability_tolerance: PROBABILITY_TOLERANCE,
        }
    }

    #[test]
    fn perfect_observation_reduces_entropy_to_zero() {
        let action = action("perfect", 1.0, 0.0, [(1.0, 0.0), (0.0, 1.0)]);
        let (gain, posterior) =
            expected_information_gain_nats(&priors(), &action, PROBABILITY_TOLERANCE).unwrap();
        assert!((gain - std::f64::consts::LN_2).abs() < 1e-12);
        assert!(posterior.abs() < 1e-12);
    }

    #[test]
    fn uninformative_observation_has_zero_eig() {
        let action = action("uninformative", 1.0, 0.0, [(0.5, 0.5), (0.5, 0.5)]);
        let (gain, posterior) =
            expected_information_gain_nats(&priors(), &action, PROBABILITY_TOLERANCE).unwrap();
        assert!(gain.abs() < 1e-12);
        assert!((posterior - std::f64::consts::LN_2).abs() < 1e-12);
    }

    #[test]
    fn planner_ranks_information_gain_per_cost_deterministically() {
        let actions = vec![
            action("costly-perfect", 4.0, 0.0, [(1.0, 0.0), (0.0, 1.0)]),
            action("cheap-partial", 1.0, 0.0, [(0.8, 0.2), (0.2, 0.8)]),
            action("zero-information", 1.0, 0.0, [(0.5, 0.5), (0.5, 0.5)]),
        ];
        let first = plan_non_invasive_surveys(&priors(), &actions, constraints(), "analyst_hours")
            .unwrap();
        let second = plan_non_invasive_surveys(&priors(), &actions, constraints(), "analyst_hours")
            .unwrap();
        assert_eq!(first, second);
        assert_eq!(first.ranked_actions[0].action_id, "cheap-partial");
        assert_eq!(first.ranked_actions[1].action_id, "costly-perfect");
    }

    #[test]
    fn authorisation_cost_and_impact_gates_reject_actions() {
        let mut no_rights = action("no-rights", 1.0, 0.0, [(1.0, 0.0), (0.0, 1.0)]);
        no_rights.authorization.source_data[0].rights_profile.state = crate::heritage::RightsState::Unknown;
        let mut no_land = action("no-land", 1.0, 0.0, [(1.0, 0.0), (0.0, 1.0)]);
        no_land.authorization.land_access_confirmed = false;
        let mut no_permit = action("no-permit", 1.0, 0.0, [(1.0, 0.0), (0.0, 1.0)]);
        no_permit.authorization.permit_confirmed = false;
        let mut no_custody = action("no-custody", 1.0, 0.0, [(1.0, 0.0), (0.0, 1.0)]);
        no_custody.authorization.custodial_review_complete = false;
        let costly = action("costly", 11.0, 0.0, [(1.0, 0.0), (0.0, 1.0)]);
        let disturbing = action("disturbing", 1.0, 0.26, [(1.0, 0.0), (0.0, 1.0)]);

        let plan = plan_non_invasive_surveys(
            &priors(),
            &[no_rights, no_land, no_permit, no_custody, costly, disturbing],
            constraints(),
            "survey_cost_units",
        ).unwrap();

        assert!(plan.ranked_actions.is_empty());
        assert_eq!(plan.rejected_actions.len(), 6);
        assert!(plan.rejected_actions.iter().any(|r| matches!(r.reason, SurveyRejectionReason::SourceRightsDenied(RightsDenial::RightsNotVerified(_)))));
        assert!(plan.rejected_actions.iter().any(|r| r.reason == SurveyRejectionReason::LandAccessNotConfirmed));
        assert!(plan.rejected_actions.iter().any(|r| r.reason == SurveyRejectionReason::PermitNotConfirmed));
        assert!(plan.rejected_actions.iter().any(|r| r.reason == SurveyRejectionReason::CustodialReviewIncomplete));
        assert!(plan.rejected_actions.iter().any(|r| r.reason == SurveyRejectionReason::CostBudgetExceeded));
        assert!(plan.rejected_actions.iter().any(|r| r.reason == SurveyRejectionReason::ImpactCeilingExceeded));
    }

    #[test]
    fn survey_actions_use_purpose_specific_rights_and_geometry_tiers() {
        let mut training = action("training-not-permitted", 1.0, 0.0, [(1.0, 0.0), (0.0, 1.0)]);
        training.authorization.source_data[0].purpose = HeritageUse::Training;
        let plan = plan_non_invasive_surveys(
            &priors(),
            &[training],
            constraints(),
            "hours",
        ).unwrap();
        assert!(matches!(
            plan.rejected_actions[0].reason,
            SurveyRejectionReason::SourceRightsDenied(
                RightsDenial::PurposeNotPermitted(HeritageUse::Training)
            )
        ));

        let mut exact_unapproved = action("exact-unapproved", 1.0, 0.0, [(1.0, 0.0), (0.0, 1.0)]);
        exact_unapproved.authorization.source_data[0].access_context.exact_geometry_authorized = false;
        let plan = plan_non_invasive_surveys(
            &priors(),
            &[exact_unapproved],
            constraints(),
            "hours",
        ).unwrap();
        assert!(matches!(
            plan.rejected_actions[0].reason,
            SurveyRejectionReason::SourceRightsDenied(RightsDenial::ExactGeometryNotAuthorized)
        ));

        let mut spatial_restricted = action("geometry-tier-restricted", 1.0, 0.0, [(1.0, 0.0), (0.0, 1.0)]);
        spatial_restricted.authorization.source_data[0].geometry_access_tier = AccessTier::Custodial;
        let plan = plan_non_invasive_surveys(
            &priors(),
            &[spatial_restricted],
            constraints(),
            "hours",
        ).unwrap();
        assert!(matches!(
            plan.rejected_actions[0].reason,
            SurveyRejectionReason::SourceRightsDenied(RightsDenial::AccessTierInsufficient {
                required: AccessTier::Custodial,
                granted: AccessTier::ApprovedResearch
            })
        ));
    }

    #[test]
    fn malformed_prior_and_likelihood_models_fail_closed() {
        let invalid_priors = vec![
            HypothesisPrior { hypothesis_id: "a".into(), probability: 0.7 },
            HypothesisPrior { hypothesis_id: "b".into(), probability: 0.7 },
        ];
        assert!(matches!(
            plan_non_invasive_surveys(&invalid_priors, &[], constraints(), "hours"),
            Err(SurveyPlannerError::PriorProbabilitiesDoNotSumToOne { .. })
        ));

        let mut invalid = action("bad", 1.0, 0.0, [(0.5, 0.5), (0.5, 0.5)]);
        invalid.outcomes[0].likelihood_by_hypothesis.insert("unknown".into(), 0.1);
        assert!(matches!(
            expected_information_gain_nats(&priors(), &invalid, PROBABILITY_TOLERANCE),
            Err(SurveyPlannerError::LikelihoodHypothesisIdsMismatch { .. })
        ));
    }

    #[test]
    fn duplicate_action_and_outcome_ids_are_rejected() {
        let duplicate = action("same", 1.0, 0.0, [(0.5, 0.5), (0.5, 0.5)]);
        assert_eq!(
            plan_non_invasive_surveys(&priors(), &[duplicate.clone(), duplicate], constraints(), "hours"),
            Err(SurveyPlannerError::DuplicateActionId("same".into()))
        );

        let mut duplicate_outcome = action("duplicate-outcome", 1.0, 0.0, [(0.5, 0.5), (0.5, 0.5)]);
        duplicate_outcome.outcomes[1].outcome_id = "positive".into();
        assert!(matches!(
            expected_information_gain_nats(&priors(), &duplicate_outcome, PROBABILITY_TOLERANCE),
            Err(SurveyPlannerError::DuplicateOutcomeId { .. })
        ));
    }
}
