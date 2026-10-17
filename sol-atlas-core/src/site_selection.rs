// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Evidence-first, constraint-first site selection.
//!
//! Scores are comparative planning aids, never proof of site safety or legal suitability.
//! Hard constraints are evaluated before weighted preferences. Missing, stale, disputed,
//! or non-verified hard-constraint evidence cannot produce a shortlist-eligible result.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceState {
    Verified,
    Estimated,
    Unknown,
    Disputed,
    Stale,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GateStatus {
    Pass,
    Fail,
    Unknown,
    NotAssessed,
}

/// Project- and jurisdiction-specific constraints. The policy decides which are mandatory.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HardConstraint {
    LandRights,
    PlanningCompatibility,
    ProtectedEcology,
    HazardScreen,
    GeotechnicalFeasibility,
    WaterAndUtilities,
    EmergencyAccess,
    CommunityProcess,
}

/// Preference dimensions are benefit-oriented and normalized to [0, 1], with 1 best.
/// This is not a substitute for the hard constraints above.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SiteCriterion {
    TransitAccess,
    UtilityCapacity,
    ClimateResilience,
    EcologicalFit,
    CommunityBenefit,
    MaterialLogistics,
    LifecycleCarbon,
    CircularityPotential,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConstraintEvidence {
    pub status: GateStatus,
    pub evidence_state: EvidenceState,
    #[serde(default)]
    pub source_id: Option<String>,
    #[serde(default)]
    pub observed_at: Option<String>,
    #[serde(default)]
    pub expires_at: Option<String>,
    #[serde(default)]
    pub note: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct EvidenceScore {
    /// Benefit-oriented score in [0, 1]. None means no usable numeric value.
    pub value: Option<f64>,
    pub evidence_state: EvidenceState,
    #[serde(default)]
    pub source_id: Option<String>,
    #[serde(default)]
    pub observed_at: Option<String>,
    #[serde(default)]
    pub expires_at: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SiteCandidate {
    pub site_id: String,
    #[serde(default)]
    pub hard_constraints: BTreeMap<HardConstraint, ConstraintEvidence>,
    #[serde(default)]
    pub scores: BTreeMap<SiteCriterion, EvidenceScore>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SelectionPolicy {
    /// A project-specific list: never assume one universal checklist fits every jurisdiction.
    pub required_constraints: Vec<HardConstraint>,
    pub weights: BTreeMap<SiteCriterion, f64>,
    /// Weighted evidence coverage required before a site can enter the shortlist.
    pub minimum_evidence_coverage: f64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SuitabilityStatus {
    /// A verified hard constraint fails. Weighted preference scores cannot override this.
    Excluded,
    /// A required gate or enough weighted evidence is missing, stale, disputed, or uncertain.
    NeedsEvidence,
    /// Ready for further professional assessment; NOT selected, permitted, or certified safe.
    ShortlistEligible,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SiteEvaluation {
    pub site_id: String,
    pub status: SuitabilityStatus,
    /// Comparative score from usable verified/estimated metrics, if any.
    pub weighted_score: Option<f64>,
    /// 1.0 means every positively weighted dimension has verified evidence.
    /// Estimated values contribute 0.5; unknown/stale/disputed values contribute 0.
    pub evidence_coverage: f64,
    pub reasons: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SiteSelectionError {
    EmptySiteId,
    EmptyRequiredConstraints,
    DuplicateRequiredConstraint(HardConstraint),
    InvalidMinimumCoverage(f64),
    InvalidWeight { criterion: SiteCriterion, weight: f64 },
    NoPositiveWeights,
    InvalidScore { criterion: SiteCriterion, value: f64 },
}

impl fmt::Display for SiteSelectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptySiteId => write!(f, "site_id must not be empty"),
            Self::EmptyRequiredConstraints => write!(f, "policy must name at least one required hard constraint"),
            Self::DuplicateRequiredConstraint(c) => write!(f, "duplicate required constraint: {c:?}"),
            Self::InvalidMinimumCoverage(v) => write!(f, "minimum evidence coverage must be finite and in [0, 1], got {v}"),
            Self::InvalidWeight { criterion, weight } => write!(f, "weight for {criterion:?} must be finite and non-negative, got {weight}"),
            Self::NoPositiveWeights => write!(f, "at least one criterion weight must be positive"),
            Self::InvalidScore { criterion, value } => write!(f, "score for {criterion:?} must be finite and in [0, 1], got {value}"),
        }
    }
}

impl std::error::Error for SiteSelectionError {}

fn confidence(state: EvidenceState) -> f64 {
    match state {
        EvidenceState::Verified => 1.0,
        EvidenceState::Estimated => 0.5,
        EvidenceState::Unknown | EvidenceState::Disputed | EvidenceState::Stale => 0.0,
    }
}

fn validate_policy(policy: &SelectionPolicy) -> Result<f64, SiteSelectionError> {
    if policy.required_constraints.is_empty() {
        return Err(SiteSelectionError::EmptyRequiredConstraints);
    }
    let mut seen = BTreeSet::new();
    for constraint in &policy.required_constraints {
        if !seen.insert(*constraint) {
            return Err(SiteSelectionError::DuplicateRequiredConstraint(*constraint));
        }
    }
    if !policy.minimum_evidence_coverage.is_finite()
        || !(0.0..=1.0).contains(&policy.minimum_evidence_coverage)
    {
        return Err(SiteSelectionError::InvalidMinimumCoverage(
            policy.minimum_evidence_coverage,
        ));
    }
    for (criterion, weight) in &policy.weights {
        if !weight.is_finite() || *weight < 0.0 {
            return Err(SiteSelectionError::InvalidWeight {
                criterion: *criterion,
                weight: *weight,
            });
        }
    }
    let total_weight: f64 = policy.weights.values().sum();
    if total_weight <= 0.0 || !total_weight.is_finite() {
        return Err(SiteSelectionError::NoPositiveWeights);
    }
    Ok(total_weight)
}

/// Evaluate a site deterministically using a fail-closed hard-gate stage followed by
/// evidence-weighted preference scoring. No source freshness is inferred here: ingestion
/// must mark evidence stale when it exceeds the configured freshness policy.
pub fn evaluate_site(
    candidate: &SiteCandidate,
    policy: &SelectionPolicy,
) -> Result<SiteEvaluation, SiteSelectionError> {
    let total_weight = validate_policy(policy)?;
    if candidate.site_id.trim().is_empty() {
        return Err(SiteSelectionError::EmptySiteId);
    }

    for (criterion, metric) in &candidate.scores {
        if let Some(value) = metric.value {
            if !value.is_finite() || !(0.0..=1.0).contains(&value) {
                return Err(SiteSelectionError::InvalidScore {
                    criterion: *criterion,
                    value,
                });
            }
        }
    }

    let mut reasons = Vec::new();
    let mut hard_failure = false;
    let mut gate_needs_evidence = false;

    for constraint in &policy.required_constraints {
        match candidate.hard_constraints.get(constraint) {
            None => {
                gate_needs_evidence = true;
                reasons.push(format!("required constraint {constraint:?} is missing"));
            }
            Some(evidence)
                if evidence.status == GateStatus::Fail
                    && evidence.evidence_state == EvidenceState::Verified =>
            {
                hard_failure = true;
                reasons.push(format!(
                    "verified hard-constraint failure: {constraint:?}"
                ));
            }
            Some(evidence)
                if evidence.status != GateStatus::Pass
                    || evidence.evidence_state != EvidenceState::Verified =>
            {
                gate_needs_evidence = true;
                reasons.push(format!(
                    "constraint {constraint:?} is not backed by verified passing evidence"
                ));
            }
            Some(_) => {}
        }
    }

    let mut scored_weight = 0.0_f64;
    let mut score_sum = 0.0_f64;
    let mut confidence_weight = 0.0_f64;

    for (criterion, weight) in &policy.weights {
        if *weight == 0.0 {
            continue;
        }
        match candidate.scores.get(criterion) {
            Some(metric) => match (metric.value, metric.evidence_state) {
                (Some(value), EvidenceState::Verified) => {
                    scored_weight += *weight;
                    score_sum += *weight * value;
                    confidence_weight += *weight * confidence(metric.evidence_state);
                }
                (Some(value), EvidenceState::Estimated) => {
                    scored_weight += *weight;
                    score_sum += *weight * value;
                    confidence_weight += *weight * 0.5;
                    reasons.push(format!("{criterion:?} relies on an estimate"));
                }
                (_, EvidenceState::Unknown) => {
                    reasons.push(format!("{criterion:?} evidence is unknown"));
                }
                (_, EvidenceState::Disputed) => {
                    reasons.push(format!("{criterion:?} evidence is disputed"));
                }
                (_, EvidenceState::Stale) => {
                    reasons.push(format!("{criterion:?} evidence is stale"));
                }
                (None, EvidenceState::Verified) => {
                    reasons.push(format!("{criterion:?} is marked verified but has no value"));
                }
                (None, EvidenceState::Estimated) => {
                    reasons.push(format!("{criterion:?} is marked estimated but has no value"));
                }
            },
            None => reasons.push(format!("{criterion:?} has no evidence record")),
        }
    }

    let weighted_score = (scored_weight > 0.0).then(|| score_sum / scored_weight);
    let evidence_coverage = (confidence_weight / total_weight).clamp(0.0, 1.0);

    let status = if hard_failure {
        SuitabilityStatus::Excluded
    } else if gate_needs_evidence
        || evidence_coverage < policy.minimum_evidence_coverage
    {
        SuitabilityStatus::NeedsEvidence
    } else {
        SuitabilityStatus::ShortlistEligible
    };

    Ok(SiteEvaluation {
        site_id: candidate.site_id.clone(),
        status,
        weighted_score,
        evidence_coverage,
        reasons,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    fn policy(minimum_evidence_coverage: f64) -> SelectionPolicy {
        SelectionPolicy {
            required_constraints: vec![
                HardConstraint::LandRights,
                HardConstraint::PlanningCompatibility,
                HardConstraint::ProtectedEcology,
                HardConstraint::HazardScreen,
                HardConstraint::GeotechnicalFeasibility,
                HardConstraint::WaterAndUtilities,
                HardConstraint::EmergencyAccess,
                HardConstraint::CommunityProcess,
            ],
            weights: [
                (SiteCriterion::TransitAccess, 1.0),
                (SiteCriterion::UtilityCapacity, 1.0),
                (SiteCriterion::ClimateResilience, 1.0),
                (SiteCriterion::EcologicalFit, 1.0),
                (SiteCriterion::CommunityBenefit, 1.0),
                (SiteCriterion::MaterialLogistics, 1.0),
                (SiteCriterion::LifecycleCarbon, 1.0),
                (SiteCriterion::CircularityPotential, 1.0),
            ]
            .into_iter()
            .collect(),
            minimum_evidence_coverage,
        }
    }

    fn passing_candidate() -> SiteCandidate {
        let constraints = policy(0.9)
            .required_constraints
            .into_iter()
            .map(|constraint| {
                (
                    constraint,
                    ConstraintEvidence {
                        status: GateStatus::Pass,
                        evidence_state: EvidenceState::Verified,
                        source_id: Some("synthetic:verified-gate".into()),
                        observed_at: Some("2026-10-17".into()),
                        expires_at: None,
                        note: None,
                    },
                )
            })
            .collect();
        let scores = policy(0.9)
            .weights
            .keys()
            .map(|criterion| {
                (
                    *criterion,
                    EvidenceScore {
                        value: Some(0.8),
                        evidence_state: EvidenceState::Verified,
                        source_id: Some("synthetic:verified-metric".into()),
                        observed_at: Some("2026-10-17".into()),
                        expires_at: None,
                    },
                )
            })
            .collect();
        SiteCandidate {
            site_id: "synthetic-site".into(),
            hard_constraints: constraints,
            scores,
        }
    }

    #[test]
    fn verified_gates_and_complete_evidence_are_shortlist_eligible() {
        let result = evaluate_site(&passing_candidate(), &policy(0.9)).unwrap();
        assert_eq!(result.status, SuitabilityStatus::ShortlistEligible);
        assert!((result.weighted_score.unwrap() - 0.8).abs() < 1e-12);
        assert_eq!(result.evidence_coverage, 1.0);
    }

    #[test]
    fn verified_hard_failure_overrides_a_high_preference_score() {
        let mut candidate = passing_candidate();
        candidate
            .hard_constraints
            .get_mut(&HardConstraint::ProtectedEcology)
            .unwrap()
            .status = GateStatus::Fail;
        for metric in candidate.scores.values_mut() {
            metric.value = Some(1.0);
        }
        let result = evaluate_site(&candidate, &policy(0.9)).unwrap();
        assert_eq!(result.status, SuitabilityStatus::Excluded);
        assert!(result.reasons.iter().any(|r| r.contains("verified hard-constraint failure")));
    }

    #[test]
    fn unknown_hard_gate_fails_closed() {
        let mut candidate = passing_candidate();
        let gate = candidate
            .hard_constraints
            .get_mut(&HardConstraint::HazardScreen)
            .unwrap();
        gate.status = GateStatus::Unknown;
        gate.evidence_state = EvidenceState::Unknown;
        let result = evaluate_site(&candidate, &policy(0.9)).unwrap();
        assert_eq!(result.status, SuitabilityStatus::NeedsEvidence);
    }

    #[test]
    fn unverified_failure_does_not_become_an_authoritative_exclusion() {
        let mut candidate = passing_candidate();
        let gate = candidate
            .hard_constraints
            .get_mut(&HardConstraint::LandRights)
            .unwrap();
        gate.status = GateStatus::Fail;
        gate.evidence_state = EvidenceState::Disputed;
        let result = evaluate_site(&candidate, &policy(0.9)).unwrap();
        assert_eq!(result.status, SuitabilityStatus::NeedsEvidence);
    }

    #[test]
    fn stale_metric_is_not_silently_used_or_treated_as_zero() {
        let mut candidate = passing_candidate();
        let metric = candidate.scores.get_mut(&SiteCriterion::EcologicalFit).unwrap();
        metric.value = Some(1.0);
        metric.evidence_state = EvidenceState::Stale;
        let result = evaluate_site(&candidate, &policy(0.9)).unwrap();
        assert_eq!(result.status, SuitabilityStatus::NeedsEvidence);
        assert!((result.weighted_score.unwrap() - 0.8).abs() < 1e-12);
        assert_eq!(result.evidence_coverage, 0.875);
        assert!(result.reasons.iter().any(|r| r.contains("evidence is stale")));
    }

    #[test]
    fn invalid_values_are_rejected() {
        let mut candidate = passing_candidate();
        candidate
            .scores
            .get_mut(&SiteCriterion::TransitAccess)
            .unwrap()
            .value = Some(1.01);
        assert!(matches!(
            evaluate_site(&candidate, &policy(0.9)),
            Err(SiteSelectionError::InvalidScore { .. })
        ));
    }

    #[test]
    fn invalid_negative_weight_is_rejected() {
        let mut p = policy(0.9);
        p.weights.insert(SiteCriterion::TransitAccess, -1.0);
        assert!(matches!(
            evaluate_site(&passing_candidate(), &p),
            Err(SiteSelectionError::InvalidWeight { .. })
        ));
    }

    #[test]
    fn missing_hard_constraint_record_needs_evidence() {
        let mut candidate = passing_candidate();
        candidate.hard_constraints.remove(&HardConstraint::LandRights);
        let result = evaluate_site(&candidate, &policy(0.9)).unwrap();
        assert_eq!(result.status, SuitabilityStatus::NeedsEvidence);
        assert!(result.reasons.iter().any(|r| r.contains("is missing")));
    }

    #[derive(Deserialize)]
    struct BenchmarkCase {
        case_id: String,
        candidate: SiteCandidate,
        policy: SelectionPolicy,
        expected_status: SuitabilityStatus,
        expected_score: f64,
        score_tolerance: f64,
    }

    #[test]
    fn synthetic_benchmark_fixture_matches_expected_dispositions_and_scores() {
        let fixture = include_str!("../../docs/research/fixtures/site-selection-benchmark-v0.1.jsonl");
        let mut records = 0;
        for line in fixture.lines().filter(|line| !line.trim().is_empty()) {
            let case: BenchmarkCase = serde_json::from_str(line).expect("valid benchmark JSONL");
            let result = evaluate_site(&case.candidate, &case.policy)
                .unwrap_or_else(|e| panic!("{}: {e}", case.case_id));
            assert_eq!(result.status, case.expected_status, "{}", case.case_id);
            let score = result.weighted_score.expect("fixture has score");
            assert!(
                (score - case.expected_score).abs() <= case.score_tolerance,
                "{}: expected {}, got {}",
                case.case_id,
                case.expected_score,
                score
            );
            records += 1;
        }
        assert!(records >= 4, "benchmark fixture should include at least four cases");
    }
}
