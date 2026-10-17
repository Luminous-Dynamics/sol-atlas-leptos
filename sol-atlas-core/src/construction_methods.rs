// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Evidence-backed construction-method screening and robust trade-off analysis.
//!
//! This module compares declared construction alternatives over a shared functional
//! basis and study period. Non-compensatory gates are evaluated before metrics; a
//! confirmed current hard-gate failure excludes an option regardless of cost, carbon,
//! speed or circularity. Missing or uncertain mandatory evidence remains unresolved.
//!
//! Metric intervals are compared using conservative interval dominance, never collapsed
//! into a single weighted score. This module validates record structure and declared
//! evidence metadata; it does not independently authenticate source bytes, certify code
//! compliance, run structural/fire/moisture calculations, or approve construction.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConstructionFamily {
    ConventionalCastInPlace,
    PrecastConcrete,
    LightGaugeSteel,
    MassTimber,
    PanelizedTimber,
    VolumetricModular,
    EarthBased,
    AdditiveManufactured,
    Hybrid,
    Other,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConstructionGate {
    RegulatoryApprovalPath,
    StructuralVerification,
    FireLifeSafety,
    MoistureDurability,
    GroundAndFoundationInterface,
    WorkerSafetyPlan,
    QualityAssurancePlan,
    MaintainabilityAndServiceAccess,
    SupplyChainAndRepairCapacity,
    EndOfLifeRecoveryPlan,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GateOutcome {
    Pass,
    Fail,
    Unresolved,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConstructionEvidenceState {
    Verified,
    Estimated,
    Unknown,
    Disputed,
    Stale,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConstructionMetric {
    UpfrontCarbonA0A5,
    EmbodiedCarbonAtoC,
    OperationalCarbonB6B7,
    WholeLifeCostStudyPeriod,
    OnSiteDurationDays,
    OnSiteLaborHoursPerSquareMeter,
    MaintenanceRepairHoursPerSquareMeter,
    RecoverableMassFraction,
    AdaptabilityScore,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObjectiveDirection {
    Minimize,
    Maximize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GateFinding {
    ConfirmedPass,
    ConfirmedFail,
    NeedsEvidence,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MethodScreeningStatus {
    Eligible,
    Excluded,
    NeedsEvidence,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConstructionGateReceipt {
    pub gate: ConstructionGate,
    pub outcome: GateOutcome,
    pub evidence_state: ConstructionEvidenceState,
    pub evidence_id: String,
    pub evidence_sha256: String,
    pub reviewer_id: String,
    pub project_scope_id: String,
    pub project_scope_sha256: String,
    pub design_revision_sha256: String,
    pub assessed_at_unix_seconds: i64,
    pub valid_until_unix_seconds: i64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConstructionMetricReceipt {
    pub metric: ConstructionMetric,
    pub lower: f64,
    pub upper: f64,
    pub unit: String,
    pub evidence_state: ConstructionEvidenceState,
    pub source_id: String,
    pub source_sha256: String,
    pub project_scope_id: String,
    pub project_scope_sha256: String,
    pub design_revision_sha256: String,
    pub functional_basis_id: String,
    pub study_period_years: u16,
    pub assessed_at_unix_seconds: i64,
    pub valid_until_unix_seconds: i64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConstructionMethodCandidate {
    pub method_id: String,
    pub family: ConstructionFamily,
    pub project_scope_id: String,
    pub project_scope_sha256: String,
    pub design_revision_sha256: String,
    /// Exact shared basis, e.g. one m² of usable floor area delivering the same service.
    pub functional_basis_id: String,
    pub study_period_years: u16,
    pub gate_receipts: Vec<ConstructionGateReceipt>,
    pub metric_receipts: Vec<ConstructionMetricReceipt>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConstructionMetricObjective {
    pub metric: ConstructionMetric,
    pub direction: ObjectiveDirection,
    /// Must use a common unit and normalization for all alternatives in this policy.
    pub unit: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConstructionMethodPolicy {
    pub evaluation_time_unix_seconds: i64,
    pub project_scope_id: String,
    pub project_scope_sha256: String,
    pub functional_basis_id: String,
    pub study_period_years: u16,
    pub maximum_gate_age_seconds: i64,
    pub maximum_metric_age_seconds: i64,
    /// These are non-compensatory. A score may never waive a required gate.
    pub required_hard_gates: BTreeSet<ConstructionGate>,
    pub objectives: Vec<ConstructionMetricObjective>,
    /// Expected to contain Verified and optionally Estimated, never unsafe/unknown states.
    pub accepted_metric_states: BTreeSet<ConstructionEvidenceState>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MetricInterval {
    pub lower: f64,
    pub upper: f64,
    pub unit: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConstructionMethodAssessment {
    pub method_id: String,
    pub family: ConstructionFamily,
    pub screening_status: MethodScreeningStatus,
    pub gate_findings: BTreeMap<ConstructionGate, GateFinding>,
    pub metrics: BTreeMap<ConstructionMetric, MetricInterval>,
    pub robust_frontier: bool,
    pub reasons: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConstructionMethodReport {
    pub assessments: BTreeMap<String, ConstructionMethodAssessment>,
    pub eligible_count: usize,
    pub excluded_count: usize,
    pub needs_evidence_count: usize,
    pub robust_frontier_method_ids: Vec<String>,
    /// True means an option with unresolved evidence could change the frontier later.
    pub frontier_conditional_on_missing_evidence: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub enum ConstructionMethodError {
    EmptyCandidates,
    EmptyProjectScope,
    InvalidScopeDigest,
    EmptyFunctionalBasis,
    InvalidStudyPeriod,
    InvalidEvaluationTime,
    InvalidMaximumAge,
    EmptyRequiredGates,
    EmptyObjectives,
    EmptyMetricUnit(ConstructionMetric),
    DuplicateObjective(ConstructionMetric),
    InvalidAcceptedMetricState,
    EmptyMethodId,
    DuplicateMethodId(String),
    ScopeMismatch(String),
    ScopeDigestMismatch(String),
    DesignDigestMismatch(String),
    FunctionalBasisMismatch(String),
    StudyPeriodMismatch(String),
    DuplicateGateReceipt { method_id: String, gate: ConstructionGate },
    DuplicateMetricReceipt { method_id: String, metric: ConstructionMetric },
    UnexpectedGateReceipt { method_id: String, gate: ConstructionGate },
    UnexpectedMetricReceipt { method_id: String, metric: ConstructionMetric },
    MissingReceiptMetadata { method_id: String, field: String },
    InvalidDigest { method_id: String, field: String },
    InvalidReceiptTime { method_id: String },
    InvalidMetricInterval { method_id: String, metric: ConstructionMetric },
    MetricUnitMismatch { method_id: String, metric: ConstructionMetric },
}

impl fmt::Display for ConstructionMethodError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for ConstructionMethodError {}

fn nonempty(value: &str) -> bool {
    !value.trim().is_empty()
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn receipt_is_current(
    assessed_at: i64,
    valid_until: i64,
    evaluation_time: i64,
    maximum_age: i64,
) -> bool {
    assessed_at >= 0
        && assessed_at <= evaluation_time
        && valid_until >= evaluation_time
        && evaluation_time.saturating_sub(assessed_at) <= maximum_age
}

fn validate_policy(policy: &ConstructionMethodPolicy) -> Result<(), ConstructionMethodError> {
    if !nonempty(&policy.project_scope_id) {
        return Err(ConstructionMethodError::EmptyProjectScope);
    }
    if !valid_sha256(&policy.project_scope_sha256) {
        return Err(ConstructionMethodError::InvalidScopeDigest);
    }
    if !nonempty(&policy.functional_basis_id) {
        return Err(ConstructionMethodError::EmptyFunctionalBasis);
    }
    if policy.study_period_years == 0 {
        return Err(ConstructionMethodError::InvalidStudyPeriod);
    }
    if policy.evaluation_time_unix_seconds < 0 {
        return Err(ConstructionMethodError::InvalidEvaluationTime);
    }
    if policy.maximum_gate_age_seconds < 0 || policy.maximum_metric_age_seconds < 0 {
        return Err(ConstructionMethodError::InvalidMaximumAge);
    }
    if policy.required_hard_gates.is_empty() {
        return Err(ConstructionMethodError::EmptyRequiredGates);
    }
    if policy.objectives.is_empty() {
        return Err(ConstructionMethodError::EmptyObjectives);
    }

    let mut seen = BTreeSet::new();
    for objective in &policy.objectives {
        if !nonempty(&objective.unit) {
            return Err(ConstructionMethodError::EmptyMetricUnit(objective.metric));
        }
        if !seen.insert(objective.metric) {
            return Err(ConstructionMethodError::DuplicateObjective(objective.metric));
        }
    }

    if policy.accepted_metric_states.is_empty()
        || policy.accepted_metric_states.iter().any(|state| {
            matches!(
                state,
                ConstructionEvidenceState::Unknown
                    | ConstructionEvidenceState::Disputed
                    | ConstructionEvidenceState::Stale
            )
        })
    {
        return Err(ConstructionMethodError::InvalidAcceptedMetricState);
    }
    Ok(())
}

fn validate_candidate_identity(
    candidate: &ConstructionMethodCandidate,
    policy: &ConstructionMethodPolicy,
) -> Result<(), ConstructionMethodError> {
    if !nonempty(&candidate.method_id) {
        return Err(ConstructionMethodError::EmptyMethodId);
    }
    if candidate.project_scope_id != policy.project_scope_id {
        return Err(ConstructionMethodError::ScopeMismatch(
            candidate.method_id.clone(),
        ));
    }
    if !valid_sha256(&candidate.project_scope_sha256)
        || candidate.project_scope_sha256 != policy.project_scope_sha256
    {
        return Err(ConstructionMethodError::ScopeDigestMismatch(
            candidate.method_id.clone(),
        ));
    }
    if !valid_sha256(&candidate.design_revision_sha256) {
        return Err(ConstructionMethodError::DesignDigestMismatch(
            candidate.method_id.clone(),
        ));
    }
    if candidate.functional_basis_id != policy.functional_basis_id {
        return Err(ConstructionMethodError::FunctionalBasisMismatch(
            candidate.method_id.clone(),
        ));
    }
    if candidate.study_period_years != policy.study_period_years {
        return Err(ConstructionMethodError::StudyPeriodMismatch(
            candidate.method_id.clone(),
        ));
    }
    Ok(())
}

fn validate_gate_receipt(
    candidate: &ConstructionMethodCandidate,
    receipt: &ConstructionGateReceipt,
    policy: &ConstructionMethodPolicy,
) -> Result<(), ConstructionMethodError> {
    for (field, value) in [
        ("evidence_id", receipt.evidence_id.as_str()),
        ("reviewer_id", receipt.reviewer_id.as_str()),
        ("project_scope_id", receipt.project_scope_id.as_str()),
    ] {
        if !nonempty(value) {
            return Err(ConstructionMethodError::MissingReceiptMetadata {
                method_id: candidate.method_id.clone(),
                field: field.into(),
            });
        }
    }
    if !valid_sha256(&receipt.evidence_sha256) {
        return Err(ConstructionMethodError::InvalidDigest {
            method_id: candidate.method_id.clone(),
            field: "gate evidence_sha256".into(),
        });
    }
    if !valid_sha256(&receipt.project_scope_sha256)
        || receipt.project_scope_sha256 != policy.project_scope_sha256
    {
        return Err(ConstructionMethodError::ScopeDigestMismatch(
            candidate.method_id.clone(),
        ));
    }
    if !valid_sha256(&receipt.design_revision_sha256)
        || receipt.design_revision_sha256 != candidate.design_revision_sha256
        || receipt.project_scope_id != policy.project_scope_id
    {
        return Err(ConstructionMethodError::DesignDigestMismatch(
            candidate.method_id.clone(),
        ));
    }
    if receipt.assessed_at_unix_seconds < 0
        || receipt.valid_until_unix_seconds < receipt.assessed_at_unix_seconds
        || receipt.assessed_at_unix_seconds > policy.evaluation_time_unix_seconds
    {
        return Err(ConstructionMethodError::InvalidReceiptTime {
            method_id: candidate.method_id.clone(),
        });
    }
    if !policy.required_hard_gates.contains(&receipt.gate) {
        return Err(ConstructionMethodError::UnexpectedGateReceipt {
            method_id: candidate.method_id.clone(),
            gate: receipt.gate,
        });
    }
    Ok(())
}

fn validate_metric_receipt(
    candidate: &ConstructionMethodCandidate,
    receipt: &ConstructionMetricReceipt,
    policy: &ConstructionMethodPolicy,
    objective: &ConstructionMetricObjective,
) -> Result<(), ConstructionMethodError> {
    for (field, value) in [
        ("source_id", receipt.source_id.as_str()),
        ("project_scope_id", receipt.project_scope_id.as_str()),
        ("functional_basis_id", receipt.functional_basis_id.as_str()),
    ] {
        if !nonempty(value) {
            return Err(ConstructionMethodError::MissingReceiptMetadata {
                method_id: candidate.method_id.clone(),
                field: field.into(),
            });
        }
    }
    if !valid_sha256(&receipt.source_sha256) {
        return Err(ConstructionMethodError::InvalidDigest {
            method_id: candidate.method_id.clone(),
            field: "metric source_sha256".into(),
        });
    }
    if !valid_sha256(&receipt.project_scope_sha256)
        || receipt.project_scope_sha256 != policy.project_scope_sha256
    {
        return Err(ConstructionMethodError::ScopeDigestMismatch(
            candidate.method_id.clone(),
        ));
    }
    if !valid_sha256(&receipt.design_revision_sha256)
        || receipt.design_revision_sha256 != candidate.design_revision_sha256
        || receipt.project_scope_id != policy.project_scope_id
    {
        return Err(ConstructionMethodError::DesignDigestMismatch(
            candidate.method_id.clone(),
        ));
    }
    if receipt.functional_basis_id != policy.functional_basis_id {
        return Err(ConstructionMethodError::FunctionalBasisMismatch(
            candidate.method_id.clone(),
        ));
    }
    if receipt.study_period_years != policy.study_period_years {
        return Err(ConstructionMethodError::StudyPeriodMismatch(
            candidate.method_id.clone(),
        ));
    }
    if !receipt.lower.is_finite()
        || !receipt.upper.is_finite()
        || receipt.lower < 0.0
        || receipt.lower > receipt.upper
        || (matches!(
            receipt.metric,
            ConstructionMetric::RecoverableMassFraction | ConstructionMetric::AdaptabilityScore
        ) && receipt.upper > 1.0)
        || receipt.assessed_at_unix_seconds < 0
        || receipt.valid_until_unix_seconds < receipt.assessed_at_unix_seconds
        || receipt.assessed_at_unix_seconds > policy.evaluation_time_unix_seconds
    {
        return Err(ConstructionMethodError::InvalidMetricInterval {
            method_id: candidate.method_id.clone(),
            metric: receipt.metric,
        });
    }
    if receipt.unit != objective.unit {
        return Err(ConstructionMethodError::MetricUnitMismatch {
            method_id: candidate.method_id.clone(),
            metric: receipt.metric,
        });
    }
    Ok(())
}

fn evaluate_candidate(
    candidate: &ConstructionMethodCandidate,
    policy: &ConstructionMethodPolicy,
) -> Result<ConstructionMethodAssessment, ConstructionMethodError> {
    validate_candidate_identity(candidate, policy)?;

    let mut gate_receipts = BTreeMap::new();
    for receipt in &candidate.gate_receipts {
        validate_gate_receipt(candidate, receipt, policy)?;
        if gate_receipts.insert(receipt.gate, receipt).is_some() {
            return Err(ConstructionMethodError::DuplicateGateReceipt {
                method_id: candidate.method_id.clone(),
                gate: receipt.gate,
            });
        }
    }

    let mut metric_receipts = BTreeMap::new();
    let objective_by_metric: BTreeMap<_, _> = policy
        .objectives
        .iter()
        .map(|objective| (objective.metric, objective))
        .collect();
    for receipt in &candidate.metric_receipts {
        let objective = objective_by_metric.get(&receipt.metric).ok_or_else(|| {
            ConstructionMethodError::UnexpectedMetricReceipt {
                method_id: candidate.method_id.clone(),
                metric: receipt.metric,
            }
        })?;
        validate_metric_receipt(candidate, receipt, policy, objective)?;
        if metric_receipts.insert(receipt.metric, receipt).is_some() {
            return Err(ConstructionMethodError::DuplicateMetricReceipt {
                method_id: candidate.method_id.clone(),
                metric: receipt.metric,
            });
        }
    }

    let mut gate_findings = BTreeMap::new();
    let mut reasons = Vec::new();
    let mut confirmed_failure = false;
    let mut unresolved_gate = false;
    for gate in &policy.required_hard_gates {
        let finding = match gate_receipts.get(gate) {
            None => {
                reasons.push(format!("required hard gate {gate:?} has no receipt"));
                GateFinding::NeedsEvidence
            }
            Some(receipt) => {
                let current = receipt_is_current(
                    receipt.assessed_at_unix_seconds,
                    receipt.valid_until_unix_seconds,
                    policy.evaluation_time_unix_seconds,
                    policy.maximum_gate_age_seconds,
                );
                if receipt.evidence_state != ConstructionEvidenceState::Verified || !current {
                    reasons.push(format!(
                        "hard gate {gate:?} is not backed by current verified evidence"
                    ));
                    GateFinding::NeedsEvidence
                } else {
                    match receipt.outcome {
                        GateOutcome::Pass => GateFinding::ConfirmedPass,
                        GateOutcome::Fail => {
                            reasons.push(format!("hard gate {gate:?} has a confirmed failure"));
                            confirmed_failure = true;
                            GateFinding::ConfirmedFail
                        }
                        GateOutcome::Unresolved => {
                            reasons.push(format!("hard gate {gate:?} remains unresolved"));
                            GateFinding::NeedsEvidence
                        }
                    }
                }
            }
        };
        if finding == GateFinding::NeedsEvidence {
            unresolved_gate = true;
        }
        gate_findings.insert(*gate, finding);
    }

    let mut metrics = BTreeMap::new();
    let mut unresolved_metric = false;
    for objective in &policy.objectives {
        match metric_receipts.get(&objective.metric) {
            None => {
                reasons.push(format!("objective metric {:?} is missing", objective.metric));
                unresolved_metric = true;
            }
            Some(receipt) => {
                let current = receipt_is_current(
                    receipt.assessed_at_unix_seconds,
                    receipt.valid_until_unix_seconds,
                    policy.evaluation_time_unix_seconds,
                    policy.maximum_metric_age_seconds,
                );
                if !policy.accepted_metric_states.contains(&receipt.evidence_state) || !current {
                    reasons.push(format!(
                        "objective metric {:?} is stale or its evidence state is not accepted",
                        objective.metric
                    ));
                    unresolved_metric = true;
                } else {
                    metrics.insert(
                        objective.metric,
                        MetricInterval {
                            lower: receipt.lower,
                            upper: receipt.upper,
                            unit: receipt.unit.clone(),
                        },
                    );
                }
            }
        }
    }

    let screening_status = if confirmed_failure {
        MethodScreeningStatus::Excluded
    } else if unresolved_gate || unresolved_metric {
        MethodScreeningStatus::NeedsEvidence
    } else {
        MethodScreeningStatus::Eligible
    };

    Ok(ConstructionMethodAssessment {
        method_id: candidate.method_id.clone(),
        family: candidate.family,
        screening_status,
        gate_findings,
        metrics,
        robust_frontier: false,
        reasons,
    })
}

fn robustly_dominates(
    left: &ConstructionMethodAssessment,
    right: &ConstructionMethodAssessment,
    objectives: &[ConstructionMetricObjective],
) -> bool {
    let mut strictly_better = false;
    for objective in objectives {
        let Some(a) = left.metrics.get(&objective.metric) else {
            return false;
        };
        let Some(b) = right.metrics.get(&objective.metric) else {
            return false;
        };
        match objective.direction {
            ObjectiveDirection::Minimize => {
                if a.upper > b.lower {
                    return false;
                }
                if a.upper < b.lower {
                    strictly_better = true;
                }
            }
            ObjectiveDirection::Maximize => {
                if a.lower < b.upper {
                    return false;
                }
                if a.lower > b.upper {
                    strictly_better = true;
                }
            }
        }
    }
    strictly_better
}

/// Evaluates mandatory gates first, then returns the robust Pareto frontier for methods
/// whose mandatory gates and all configured metric objectives have current accepted evidence.
///
/// Robust interval dominance is deliberately strict: for every minimizing objective A.upper
/// must be <= B.lower; for every maximizing objective A.lower must be >= B.upper; at least
/// one objective must be strictly better. Overlapping intervals remain non-dominated.
pub fn evaluate_construction_methods(
    candidates: &[ConstructionMethodCandidate],
    policy: &ConstructionMethodPolicy,
) -> Result<ConstructionMethodReport, ConstructionMethodError> {
    validate_policy(policy)?;
    if candidates.is_empty() {
        return Err(ConstructionMethodError::EmptyCandidates);
    }

    let mut assessments = BTreeMap::new();
    for candidate in candidates {
        let assessment = evaluate_candidate(candidate, policy)?;
        if assessments
            .insert(assessment.method_id.clone(), assessment)
            .is_some()
        {
            return Err(ConstructionMethodError::DuplicateMethodId(
                candidate.method_id.clone(),
            ));
        }
    }

    let eligible_ids: Vec<String> = assessments
        .iter()
        .filter(|(_, assessment)| assessment.screening_status == MethodScreeningStatus::Eligible)
        .map(|(id, _)| id.clone())
        .collect();

    let mut frontier = Vec::new();
    for method_id in &eligible_ids {
        let candidate = &assessments[method_id];
        let dominated = eligible_ids.iter().any(|other_id| {
            other_id != method_id
                && robustly_dominates(
                    &assessments[other_id],
                    candidate,
                    &policy.objectives,
                )
        });
        if !dominated {
            frontier.push(method_id.clone());
            assessments
                .get_mut(method_id)
                .expect("method exists")
                .robust_frontier = true;
        }
    }

    let eligible_count = assessments
        .values()
        .filter(|assessment| assessment.screening_status == MethodScreeningStatus::Eligible)
        .count();
    let excluded_count = assessments
        .values()
        .filter(|assessment| assessment.screening_status == MethodScreeningStatus::Excluded)
        .count();
    let needs_evidence_count = assessments
        .values()
        .filter(|assessment| assessment.screening_status == MethodScreeningStatus::NeedsEvidence)
        .count();

    Ok(ConstructionMethodReport {
        assessments,
        eligible_count,
        excluded_count,
        needs_evidence_count,
        robust_frontier_method_ids: frontier,
        frontier_conditional_on_missing_evidence: needs_evidence_count > 0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digest(byte: char) -> String {
        std::iter::repeat_n(byte, 64).collect()
    }

    fn policy() -> ConstructionMethodPolicy {
        ConstructionMethodPolicy {
            evaluation_time_unix_seconds: 1_000_000,
            project_scope_id: "project-synthetic".into(),
            project_scope_sha256: digest('a'),
            functional_basis_id: "one-square-meter-usable-floor-60y-v1".into(),
            study_period_years: 60,
            maximum_gate_age_seconds: 10_000,
            maximum_metric_age_seconds: 10_000,
            required_hard_gates: [ConstructionGate::StructuralVerification]
                .into_iter()
                .collect(),
            objectives: vec![
                ConstructionMetricObjective {
                    metric: ConstructionMetric::EmbodiedCarbonAtoC,
                    direction: ObjectiveDirection::Minimize,
                    unit: "kgCO2e/m2/60y".into(),
                },
                ConstructionMetricObjective {
                    metric: ConstructionMetric::WholeLifeCostStudyPeriod,
                    direction: ObjectiveDirection::Minimize,
                    unit: "cost-unit/m2/60y".into(),
                },
                ConstructionMetricObjective {
                    metric: ConstructionMetric::RecoverableMassFraction,
                    direction: ObjectiveDirection::Maximize,
                    unit: "fraction".into(),
                },
            ],
            accepted_metric_states: [
                ConstructionEvidenceState::Verified,
                ConstructionEvidenceState::Estimated,
            ]
            .into_iter()
            .collect(),
        }
    }

    fn candidate(method_id: &str, carbon: (f64, f64), cost: (f64, f64), recovery: (f64, f64)) -> ConstructionMethodCandidate {
        let p = policy();
        ConstructionMethodCandidate {
            method_id: method_id.into(),
            family: ConstructionFamily::Hybrid,
            project_scope_id: p.project_scope_id.clone(),
            project_scope_sha256: p.project_scope_sha256.clone(),
            design_revision_sha256: digest('b'),
            functional_basis_id: p.functional_basis_id.clone(),
            study_period_years: p.study_period_years,
            gate_receipts: vec![ConstructionGateReceipt {
                gate: ConstructionGate::StructuralVerification,
                outcome: GateOutcome::Pass,
                evidence_state: ConstructionEvidenceState::Verified,
                evidence_id: format!("{method_id}-structural-review"),
                evidence_sha256: digest('c'),
                reviewer_id: "qualified-reviewer-role".into(),
                project_scope_id: p.project_scope_id.clone(),
                project_scope_sha256: p.project_scope_sha256.clone(),
                design_revision_sha256: digest('b'),
                assessed_at_unix_seconds: 999_000,
                valid_until_unix_seconds: 1_010_000,
            }],
            metric_receipts: vec![
                metric_receipt(
                    ConstructionMetric::EmbodiedCarbonAtoC,
                    carbon,
                    "kgCO2e/m2/60y",
                    method_id,
                ),
                metric_receipt(
                    ConstructionMetric::WholeLifeCostStudyPeriod,
                    cost,
                    "cost-unit/m2/60y",
                    method_id,
                ),
                metric_receipt(
                    ConstructionMetric::RecoverableMassFraction,
                    recovery,
                    "fraction",
                    method_id,
                ),
            ],
        }
    }

    fn metric_receipt(
        metric: ConstructionMetric,
        interval: (f64, f64),
        unit: &str,
        method_id: &str,
    ) -> ConstructionMetricReceipt {
        let p = policy();
        ConstructionMetricReceipt {
            metric,
            lower: interval.0,
            upper: interval.1,
            unit: unit.into(),
            evidence_state: ConstructionEvidenceState::Verified,
            source_id: format!("{method_id}-{metric:?}-estimate"),
            source_sha256: digest('d'),
            project_scope_id: p.project_scope_id,
            project_scope_sha256: p.project_scope_sha256,
            design_revision_sha256: digest('b'),
            functional_basis_id: "one-square-meter-usable-floor-60y-v1".into(),
            study_period_years: 60,
            assessed_at_unix_seconds: 999_000,
            valid_until_unix_seconds: 1_010_000,
        }
    }

    fn one_gate_policy() -> ConstructionMethodPolicy {
        let mut p = policy();
        p.objectives = vec![ConstructionMetricObjective {
            metric: ConstructionMetric::EmbodiedCarbonAtoC,
            direction: ObjectiveDirection::Minimize,
            unit: "kgCO2e/m2/60y".into(),
        }];
        p
    }

    fn one_gate_candidate(
        method_id: &str,
        outcome: GateOutcome,
        state: ConstructionEvidenceState,
        age_seconds: i64,
    ) -> ConstructionMethodCandidate {
        let p = one_gate_policy();
        let mut candidate = candidate(method_id, (10.0, 20.0), (100.0, 120.0), (0.2, 0.3));
        candidate.metric_receipts.retain(|m| m.metric == ConstructionMetric::EmbodiedCarbonAtoC);
        candidate.gate_receipts[0].outcome = outcome;
        candidate.gate_receipts[0].evidence_state = state;
        candidate.gate_receipts[0].assessed_at_unix_seconds =
            p.evaluation_time_unix_seconds - age_seconds;
        candidate.gate_receipts[0].valid_until_unix_seconds =
            p.evaluation_time_unix_seconds + 10_000;
        candidate.metric_receipts[0].assessed_at_unix_seconds =
            p.evaluation_time_unix_seconds - 1_000;
        candidate.metric_receipts[0].valid_until_unix_seconds =
            p.evaluation_time_unix_seconds + 10_000;
        candidate
    }

    #[derive(Deserialize)]
    struct GateBenchmarkCase {
        case_id: String,
        family: ConstructionFamily,
        outcome: GateOutcome,
        evidence_state: ConstructionEvidenceState,
        age_seconds: i64,
        metric_state: ConstructionEvidenceState,
        expected: MethodScreeningStatus,
    }

    #[test]
    fn pinned_gate_benchmark_cases_match_expected_screening() {
        let p = one_gate_policy();
        let mut count = 0;
        for line in include_str!("../../docs/research/fixtures/construction-method-benchmark-v0.1.jsonl")
            .lines()
            .filter(|line| !line.trim().is_empty())
        {
            let case: GateBenchmarkCase = serde_json::from_str(line).expect("valid gate fixture");
            let mut option = one_gate_candidate(
                &case.case_id,
                case.outcome,
                case.evidence_state,
                case.age_seconds,
            );
            option.family = case.family;
            option.metric_receipts[0].evidence_state = case.metric_state;
            let report = evaluate_construction_methods(&[option], &p)
                .unwrap_or_else(|error| panic!("{}: {error}", case.case_id));
            assert_eq!(
                report.assessments[&case.case_id].screening_status,
                case.expected,
                "fixture {}",
                case.case_id
            );
            assert_eq!(
                report.frontier_conditional_on_missing_evidence,
                case.expected == MethodScreeningStatus::NeedsEvidence,
                "conditional frontier flag in {}",
                case.case_id
            );
            count += 1;
        }
        assert_eq!(count, 30, "pinned construction-method benchmark size");
    }

    #[test]
    fn confirmed_failure_excludes_even_with_other_missing_gates() {
        let mut p = policy();
        p.required_hard_gates.insert(ConstructionGate::FireLifeSafety);
        let incomplete = candidate("incomplete", (10.0, 20.0), (100.0, 120.0), (0.2, 0.3));
        let report = evaluate_construction_methods(&[incomplete], &p).unwrap();
        assert_eq!(
            report.assessments["incomplete"].screening_status,
            MethodScreeningStatus::NeedsEvidence
        );
        let mut failed = one_gate_candidate(
            "verified-fail",
            GateOutcome::Fail,
            ConstructionEvidenceState::Verified,
            100,
        );
        let report = evaluate_construction_methods(&[failed], &one_gate_policy()).unwrap();
        assert_eq!(
            report.assessments["verified-fail"].screening_status,
            MethodScreeningStatus::Excluded
        );
    }

    #[test]
    fn stale_or_estimated_hard_gate_never_clears_gate() {
        for (state, age) in [
            (ConstructionEvidenceState::Estimated, 100),
            (ConstructionEvidenceState::Verified, 20_000),
            (ConstructionEvidenceState::Stale, 100),
        ] {
            let option = one_gate_candidate("uncertain-gate", GateOutcome::Pass, state, age);
            let report = evaluate_construction_methods(&[option], &one_gate_policy()).unwrap();
            assert_eq!(
                report.assessments["uncertain-gate"].screening_status,
                MethodScreeningStatus::NeedsEvidence
            );
        }
    }

    #[test]
    fn verified_failure_excludes_even_when_metric_evidence_is_missing() {
        let mut option = one_gate_candidate(
            "fail-with-missing-metrics",
            GateOutcome::Fail,
            ConstructionEvidenceState::Verified,
            100,
        );
        option.metric_receipts.clear();
        let report = evaluate_construction_methods(&[option], &one_gate_policy()).unwrap();
        assert_eq!(
            report.assessments["fail-with-missing-metrics"].screening_status,
            MethodScreeningStatus::Excluded
        );
    }

    #[test]
    fn overlapping_intervals_remain_on_robust_frontier() {
        let a = candidate("method-a", (100.0, 150.0), (100.0, 150.0), (0.3, 0.5));
        let b = candidate("method-b", (140.0, 170.0), (90.0, 120.0), (0.4, 0.6));
        let report = evaluate_construction_methods(&[a, b], &policy()).unwrap();
        assert_eq!(report.robust_frontier_method_ids, vec!["method-a", "method-b"]);
    }

    #[test]
    fn robustly_dominated_method_is_removed_from_frontier() {
        let dominating = candidate("better", (50.0, 80.0), (60.0, 75.0), (0.6, 0.8));
        let dominated = candidate("worse", (100.0, 130.0), (100.0, 140.0), (0.2, 0.4));
        let report = evaluate_construction_methods(&[dominated, dominating], &policy()).unwrap();
        assert_eq!(report.robust_frontier_method_ids, vec!["better"]);
        assert!(!report.assessments["worse"].robust_frontier);
    }

    #[test]
    fn missing_method_evidence_makes_frontier_conditional() {
        let mut unresolved = candidate("unknown", (100.0, 130.0), (100.0, 140.0), (0.2, 0.4));
        unresolved.gate_receipts.clear();
        let eligible = candidate("eligible", (120.0, 140.0), (120.0, 150.0), (0.2, 0.4));
        let report = evaluate_construction_methods(&[eligible, unresolved], &policy()).unwrap();
        assert!(report.frontier_conditional_on_missing_evidence);
        assert_eq!(report.robust_frontier_method_ids, vec!["eligible"]);
    }

    #[test]
    fn functional_basis_mismatch_is_rejected() {
        let mut option = candidate("wrong-basis", (100.0, 130.0), (100.0, 140.0), (0.2, 0.4));
        option.functional_basis_id = "un-normalized-per-wall".into();
        assert!(matches!(
            evaluate_construction_methods(&[option], &policy()),
            Err(ConstructionMethodError::FunctionalBasisMismatch(_))
        ));
    }

    #[test]
    fn invalid_metric_interval_is_rejected() {
        let mut option = candidate("bad-interval", (100.0, 130.0), (100.0, 140.0), (0.2, 0.4));
        option.metric_receipts[0].lower = f64::NAN;
        assert!(matches!(
            evaluate_construction_methods(&[option], &policy()),
            Err(ConstructionMethodError::InvalidMetricInterval { .. })
        ));
    }


    #[test]
    fn changed_project_scope_digest_invalidates_gate_receipt() {
        let mut option = candidate("scope-digest-drift", (100.0, 130.0), (100.0, 140.0), (0.2, 0.4));
        option.gate_receipts[0].project_scope_sha256 = digest('e');
        assert!(matches!(
            evaluate_construction_methods(&[option], &policy()),
            Err(ConstructionMethodError::ScopeDigestMismatch(_))
        ));
    }

    #[test]
    fn incompatible_metric_unit_is_rejected() {
        let mut option = candidate("wrong-unit", (100.0, 130.0), (100.0, 140.0), (0.2, 0.4));
        option.metric_receipts[0].unit = "kgCO2e/whole-building".into();
        assert!(matches!(
            evaluate_construction_methods(&[option], &policy()),
            Err(ConstructionMethodError::MetricUnitMismatch { .. })
        ));
    }

    #[test]
    fn stale_or_unaccepted_metrics_never_become_rankable() {
        let mut option = candidate("stale-metric", (100.0, 130.0), (100.0, 140.0), (0.2, 0.4));
        option.metric_receipts[0].evidence_state = ConstructionEvidenceState::Stale;
        let report = evaluate_construction_methods(&[option], &policy()).unwrap();
        assert_eq!(
            report.assessments["stale-metric"].screening_status,
            MethodScreeningStatus::NeedsEvidence
        );
    }

    #[test]
    fn duplicate_metric_receipt_is_rejected() {
        let mut option = candidate("duplicate-metric", (100.0, 130.0), (100.0, 140.0), (0.2, 0.4));
        option.metric_receipts.push(option.metric_receipts[0].clone());
        assert!(matches!(
            evaluate_construction_methods(&[option], &policy()),
            Err(ConstructionMethodError::DuplicateMetricReceipt { .. })
        ));
    }

    #[test]
    fn shuffled_candidate_order_preserves_report_and_frontier() {
        let a = candidate("method-a", (100.0, 150.0), (100.0, 150.0), (0.3, 0.5));
        let b = candidate("method-b", (140.0, 170.0), (90.0, 120.0), (0.4, 0.6));
        let c = candidate("method-c", (50.0, 80.0), (60.0, 75.0), (0.6, 0.8));
        let left = evaluate_construction_methods(&[a.clone(), b.clone(), c.clone()], &policy()).unwrap();
        let right = evaluate_construction_methods(&[c, b, a], &policy()).unwrap();
        assert_eq!(left, right);
    }
}
