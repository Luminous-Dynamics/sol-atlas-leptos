// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Qualitative dependency-group logic for intervention and outage scenarios.
//!
//! This evaluates one explicitly configured group at a time. It does not infer
//! probabilities, solve graph cycles, establish causation, or prove operational
//! state. Scenario conditions must be labelled as assumptions or evidence-backed
//! observations; missing information remains Unknown.

use crate::spatial_economic::{EvidenceKind, EvidenceRecord, EvidenceState};
use crate::spatial_graph::{
    AssessedDependencyReport, DependencyAssessmentError, DependencyGraph, EvidenceBlocker,
    EvidenceReadiness, GraphValidationError, RelationEvidenceAssessment,
    is_canonical_utc_timestamp,
};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum DependencyLogic {
    /// Every currently active dependency member must be available.
    AllRequired,
    /// At least one currently active dependency member must be available.
    AnyOne,
    /// At least minimum currently active dependency members must be available.
    AtLeast { minimum: usize },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DependencyGroup {
    pub id: String,
    pub dependent_entity_id: String,
    pub logic: DependencyLogic,
    /// Relationship IDs participating in this required/redundant group.
    pub relation_ids: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DependencyGroupValidationError {
    EmptyGroupId,
    EmptyDependentEntity,
    EmptyRelations,
    DuplicateRelationId,
    UnknownRelationId,
    NonDependencyRelation,
    WrongDependentEntity,
    InvalidMinimum,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityOperationalState {
    Available,
    Unavailable,
    Unknown,
}

/// Input basis. An assumed state is a scenario input, never a fact; an observed
/// state must cite active evidence whose state is Asserted or Corroborated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScenarioBasis {
    Assumed,
    Observed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScenarioEntityCondition {
    pub state: EntityOperationalState,
    pub basis: ScenarioBasis,
    pub rationale: String,
    #[serde(default)]
    pub supporting_evidence_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AvailabilityScenario {
    pub id: String,
    /// Canonical UTC timestamp: YYYY-MM-DDTHH:MM:SSZ.
    pub as_of: String,
    /// Missing entities default to Unknown, never Available.
    #[serde(default)]
    pub entity_conditions: BTreeMap<String, ScenarioEntityCondition>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScenarioValidationError {
    EmptyScenarioId,
    EmptyEntityId,
    EmptyRationale,
    MissingEvidenceForObservedState,
    DuplicateSupportingEvidence,
    UnknownSupportingEvidence,
    InadmissibleSupportingEvidence,
    EvidenceNotObservation,
    EvidenceSubjectMismatch,
    InvalidSupportingEvidenceValidity,
    OutsideSupportingEvidenceValidity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DependencyGroupState {
    Satisfied,
    Disrupted,
    Indeterminate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DependencyMemberState {
    Available,
    Unavailable,
    Unknown,
    Inactive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UnknownMemberReason {
    EvidenceBlocked,
    MissingScenarioCondition,
    ExplicitlyUnknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DependencyGroupMemberAssessment {
    pub relation_id: String,
    #[serde(default)]
    pub supporting_evidence_ids: Vec<String>,
    pub prerequisite_entity_id: String,
    pub state: DependencyMemberState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub evidence_readiness: Option<EvidenceReadiness>,
    #[serde(default)]
    pub evidence_blockers: Vec<EvidenceBlocker>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub unknown_reason: Option<UnknownMemberReason>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DependencyGroupAssessment {
    pub group_id: String,
    pub dependent_entity_id: String,
    pub logic: DependencyLogic,
    pub as_of: String,
    pub state: DependencyGroupState,
    pub required_count: usize,
    pub active_count: usize,
    pub available_count: usize,
    pub unavailable_count: usize,
    pub unknown_count: usize,
    pub members: Vec<DependencyGroupMemberAssessment>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScenarioEvaluationError {
    InvalidAsOfTimestamp,
    InvalidGraph(Vec<GraphValidationError>),
    InvalidGroup(Vec<DependencyGroupValidationError>),
    InvalidScenario(Vec<ScenarioValidationError>),
    EmptyGroupSet,
    DuplicateGroupId,
    EvidenceAssessmentFailed(DependencyAssessmentError),
    MissingAssessmentForActiveRelation,
}

impl DependencyGraph {
    /// Validate that a group contains unique dependency relations for one
    /// dependent entity. Thresholds are structural configuration, not estimates.
    pub fn validate_dependency_group(
        &self,
        group: &DependencyGroup,
    ) -> Result<(), Vec<DependencyGroupValidationError>> {
        let relations_by_id: BTreeMap<&str, _> = self
            .relations
            .iter()
            .map(|relation| (relation.id.as_str(), relation))
            .collect();
        let mut errors = Vec::new();

        if group.id.trim().is_empty() {
            errors.push(DependencyGroupValidationError::EmptyGroupId);
        }
        if group.dependent_entity_id.trim().is_empty() {
            errors.push(DependencyGroupValidationError::EmptyDependentEntity);
        }
        if group.relation_ids.is_empty() {
            errors.push(DependencyGroupValidationError::EmptyRelations);
        }
        if let DependencyLogic::AtLeast { minimum } = group.logic {
            if minimum == 0 || minimum > group.relation_ids.len() {
                errors.push(DependencyGroupValidationError::InvalidMinimum);
            }
        }

        let mut seen = BTreeSet::new();
        for relation_id in &group.relation_ids {
            if !seen.insert(relation_id.as_str()) {
                errors.push(DependencyGroupValidationError::DuplicateRelationId);
            }
            match relations_by_id.get(relation_id.as_str()) {
                None => errors.push(DependencyGroupValidationError::UnknownRelationId),
                Some(relation) => {
                    if !relation.kind.is_dependency_edge() {
                        errors.push(DependencyGroupValidationError::NonDependencyRelation);
                    }
                    if relation.subject_entity_id != group.dependent_entity_id {
                        errors.push(DependencyGroupValidationError::WrongDependentEntity);
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

    /// Evaluate multiple groups against the same scenario while validating and
    /// constructing evidence/graph indexes once. Results are sorted by group ID.
    ///
    /// This is the preferred batch API for baseline/intervention comparisons.
    /// It remains qualitative and does not recursively compose groups.
    pub fn evaluate_dependency_groups_at(
        &self,
        groups: &[DependencyGroup],
        scenario: &AvailabilityScenario,
        evidence_records: &[EvidenceRecord],
    ) -> Result<Vec<DependencyGroupAssessment>, ScenarioEvaluationError> {
        if groups.is_empty() {
            return Err(ScenarioEvaluationError::EmptyGroupSet);
        }
        if !is_canonical_utc_timestamp(&scenario.as_of) {
            return Err(ScenarioEvaluationError::InvalidAsOfTimestamp);
        }
        self.validate(evidence_records)
            .map_err(ScenarioEvaluationError::InvalidGraph)?;
        validate_scenario(scenario, evidence_records)
            .map_err(ScenarioEvaluationError::InvalidScenario)?;

        let relations: BTreeMap<&str, _> = self
            .relations
            .iter()
            .map(|relation| (relation.id.as_str(), relation))
            .collect();
        let mut group_ids = BTreeSet::new();
        let mut ordered_groups: Vec<&DependencyGroup> = groups.iter().collect();
        ordered_groups.sort_by(|left, right| left.id.cmp(&right.id));

        for group in &ordered_groups {
            if !group_ids.insert(group.id.as_str()) {
                return Err(ScenarioEvaluationError::DuplicateGroupId);
            }
            let mut errors = Vec::new();
            if group.id.trim().is_empty() {
                errors.push(DependencyGroupValidationError::EmptyGroupId);
            }
            if group.dependent_entity_id.trim().is_empty() {
                errors.push(DependencyGroupValidationError::EmptyDependentEntity);
            }
            if group.relation_ids.is_empty() {
                errors.push(DependencyGroupValidationError::EmptyRelations);
            }
            if let DependencyLogic::AtLeast { minimum } = group.logic {
                if minimum == 0 || minimum > group.relation_ids.len() {
                    errors.push(DependencyGroupValidationError::InvalidMinimum);
                }
            }

            let mut seen = BTreeSet::new();
            for relation_id in &group.relation_ids {
                if !seen.insert(relation_id.as_str()) {
                    errors.push(DependencyGroupValidationError::DuplicateRelationId);
                }
                match relations.get(relation_id.as_str()) {
                    None => errors.push(DependencyGroupValidationError::UnknownRelationId),
                    Some(relation) => {
                        if !relation.kind.is_dependency_edge() {
                            errors.push(DependencyGroupValidationError::NonDependencyRelation);
                        }
                        if relation.subject_entity_id != group.dependent_entity_id {
                            errors.push(DependencyGroupValidationError::WrongDependentEntity);
                        }
                    }
                }
            }
            if !errors.is_empty() {
                return Err(ScenarioEvaluationError::InvalidGroup(errors));
            }
        }

        // The graph API attaches readiness/blockers for every active dependency edge
        // to this report. The root only selects reachability impacts, not that edge list.
        let report = self
            .assess_dependents_at(
                &ordered_groups[0].dependent_entity_id,
                &scenario.as_of,
                evidence_records,
            )
            .map_err(ScenarioEvaluationError::EvidenceAssessmentFailed)?;
        let assessments = relation_assessment_index(&report);
        let mut output = Vec::with_capacity(ordered_groups.len());

        for group in ordered_groups {
            let mut relation_ids = group.relation_ids.clone();
            relation_ids.sort();
            let mut members = Vec::with_capacity(relation_ids.len());
            let mut active_count = 0;
            let mut available_count = 0;
            let mut unavailable_count = 0;
            let mut unknown_count = 0;

            for relation_id in relation_ids {
                let relation = relations[relation_id.as_str()];
                if !relation_is_active(relation, &scenario.as_of) {
                    members.push(DependencyGroupMemberAssessment {
                        relation_id,
                        supporting_evidence_ids: relation.supporting_evidence_ids.clone(),
                        prerequisite_entity_id: relation.object_entity_id.clone(),
                        state: DependencyMemberState::Inactive,
                        evidence_readiness: None,
                        evidence_blockers: Vec::new(),
                        unknown_reason: None,
                    });
                    continue;
                }

                active_count += 1;
                let evidence_assessment = assessments
                    .get(relation_id.as_str())
                    .ok_or(ScenarioEvaluationError::MissingAssessmentForActiveRelation)?;

                let member = if evidence_assessment.readiness == EvidenceReadiness::Blocked {
                    unknown_count += 1;
                    DependencyGroupMemberAssessment {
                        relation_id,
                        supporting_evidence_ids: relation.supporting_evidence_ids.clone(),
                        prerequisite_entity_id: relation.object_entity_id.clone(),
                        state: DependencyMemberState::Unknown,
                        evidence_readiness: Some(evidence_assessment.readiness),
                        evidence_blockers: evidence_assessment.blockers.clone(),
                        unknown_reason: Some(UnknownMemberReason::EvidenceBlocked),
                    }
                } else {
                    let condition = scenario.entity_conditions.get(&relation.object_entity_id);
                    let state = match condition.map(|value| value.state) {
                        Some(EntityOperationalState::Available) => {
                            available_count += 1;
                            DependencyMemberState::Available
                        }
                        Some(EntityOperationalState::Unavailable) => {
                            unavailable_count += 1;
                            DependencyMemberState::Unavailable
                        }
                        Some(EntityOperationalState::Unknown) | None => {
                            unknown_count += 1;
                            DependencyMemberState::Unknown
                        }
                    };
                    let reason = match (state, condition) {
                        (DependencyMemberState::Unknown, None) => {
                            Some(UnknownMemberReason::MissingScenarioCondition)
                        }
                        (DependencyMemberState::Unknown, Some(_)) => {
                            Some(UnknownMemberReason::ExplicitlyUnknown)
                        }
                        _ => None,
                    };
                    DependencyGroupMemberAssessment {
                        relation_id,
                        supporting_evidence_ids: relation.supporting_evidence_ids.clone(),
                        prerequisite_entity_id: relation.object_entity_id.clone(),
                        state,
                        evidence_readiness: Some(evidence_assessment.readiness),
                        evidence_blockers: Vec::new(),
                        unknown_reason: reason,
                    }
                };
                members.push(member);
            }

            let required_count = match group.logic {
                DependencyLogic::AllRequired => active_count,
                DependencyLogic::AnyOne => 1,
                DependencyLogic::AtLeast { minimum } => minimum,
            };
            let state = if active_count == 0 {
                DependencyGroupState::Indeterminate
            } else if available_count >= required_count {
                DependencyGroupState::Satisfied
            } else if available_count + unknown_count < required_count {
                DependencyGroupState::Disrupted
            } else {
                DependencyGroupState::Indeterminate
            };

            output.push(DependencyGroupAssessment {
                group_id: group.id.clone(),
                dependent_entity_id: group.dependent_entity_id.clone(),
                logic: group.logic,
                as_of: scenario.as_of.clone(),
                state,
                required_count,
                active_count,
                available_count,
                unavailable_count,
                unknown_count,
                members,
            });
        }

        Ok(output)
    }

    /// Evaluate one required/redundant dependency group for a scenario at an
    /// instant. This is qualitative logic, not a reliability probability.
    ///
    /// For AnyOne, one available member is sufficient. For AtLeast(k), at least
    /// k available members are needed. Evidence-blocked and unstated operational
    /// conditions count as Unknown, never as failed or available. If no member
    /// is active at the requested time, the group is Indeterminate.
    ///
    /// The evaluator does not recursively compose groups or resolve graph cycles.
    /// Multiple groups attached to one entity require an explicit higher-level
    /// composition rule and are deliberately not silently combined here.
    pub fn evaluate_dependency_group_at(
        &self,
        group: &DependencyGroup,
        scenario: &AvailabilityScenario,
        evidence_records: &[EvidenceRecord],
    ) -> Result<DependencyGroupAssessment, ScenarioEvaluationError> {
        if !is_canonical_utc_timestamp(&scenario.as_of) {
            return Err(ScenarioEvaluationError::InvalidAsOfTimestamp);
        }
        self.validate(evidence_records)
            .map_err(ScenarioEvaluationError::InvalidGraph)?;
        self.validate_dependency_group(group)
            .map_err(ScenarioEvaluationError::InvalidGroup)?;
        validate_scenario(scenario, evidence_records)
            .map_err(ScenarioEvaluationError::InvalidScenario)?;

        let report = self
            .assess_dependents_at(
                &group.dependent_entity_id,
                &scenario.as_of,
                evidence_records,
            )
            .map_err(ScenarioEvaluationError::EvidenceAssessmentFailed)?;
        let assessments = relation_assessment_index(&report);
        let relations: BTreeMap<&str, _> = self
            .relations
            .iter()
            .map(|relation| (relation.id.as_str(), relation))
            .collect();

        // Stable relation ordering makes serialized scenario output reproducible.
        let mut relation_ids = group.relation_ids.clone();
        relation_ids.sort();

        let mut members = Vec::with_capacity(relation_ids.len());
        let mut active_count = 0;
        let mut available_count = 0;
        let mut unavailable_count = 0;
        let mut unknown_count = 0;

        for relation_id in relation_ids {
            let relation = relations[relation_id.as_str()];
            if !relation_is_active(relation, &scenario.as_of) {
                members.push(DependencyGroupMemberAssessment {
                    relation_id,
                    supporting_evidence_ids: relation.supporting_evidence_ids.clone(),
                    prerequisite_entity_id: relation.object_entity_id.clone(),
                    state: DependencyMemberState::Inactive,
                    evidence_readiness: None,
                    evidence_blockers: Vec::new(),
                    unknown_reason: None,
                });
                continue;
            }

            active_count += 1;
            let evidence_assessment = assessments
                .get(relation_id.as_str())
                .ok_or(ScenarioEvaluationError::MissingAssessmentForActiveRelation)?;

            let member = if evidence_assessment.readiness == EvidenceReadiness::Blocked {
                unknown_count += 1;
                DependencyGroupMemberAssessment {
                    relation_id,
                    supporting_evidence_ids: relation.supporting_evidence_ids.clone(),
                    prerequisite_entity_id: relation.object_entity_id.clone(),
                    state: DependencyMemberState::Unknown,
                    evidence_readiness: Some(evidence_assessment.readiness),
                    evidence_blockers: evidence_assessment.blockers.clone(),
                    unknown_reason: Some(UnknownMemberReason::EvidenceBlocked),
                }
            } else {
                let condition = scenario.entity_conditions.get(&relation.object_entity_id);
                let state = match condition.map(|value| value.state) {
                    Some(EntityOperationalState::Available) => {
                        available_count += 1;
                        DependencyMemberState::Available
                    }
                    Some(EntityOperationalState::Unavailable) => {
                        unavailable_count += 1;
                        DependencyMemberState::Unavailable
                    }
                    Some(EntityOperationalState::Unknown) | None => {
                        unknown_count += 1;
                        DependencyMemberState::Unknown
                    }
                };
                let reason = match (state, condition) {
                    (DependencyMemberState::Unknown, None) => {
                        Some(UnknownMemberReason::MissingScenarioCondition)
                    }
                    (DependencyMemberState::Unknown, Some(_)) => {
                        Some(UnknownMemberReason::ExplicitlyUnknown)
                    }
                    _ => None,
                };
                DependencyGroupMemberAssessment {
                    relation_id,
                    supporting_evidence_ids: relation.supporting_evidence_ids.clone(),
                    prerequisite_entity_id: relation.object_entity_id.clone(),
                    state,
                    evidence_readiness: Some(evidence_assessment.readiness),
                    evidence_blockers: Vec::new(),
                    unknown_reason: reason,
                }
            };
            members.push(member);
        }

        let required_count = match group.logic {
            DependencyLogic::AllRequired => active_count,
            DependencyLogic::AnyOne => 1,
            DependencyLogic::AtLeast { minimum } => minimum,
        };
        let state = if active_count == 0 {
            DependencyGroupState::Indeterminate
        } else if available_count >= required_count {
            DependencyGroupState::Satisfied
        } else if available_count + unknown_count < required_count {
            DependencyGroupState::Disrupted
        } else {
            DependencyGroupState::Indeterminate
        };

        Ok(DependencyGroupAssessment {
            group_id: group.id.clone(),
            dependent_entity_id: group.dependent_entity_id.clone(),
            logic: group.logic,
            as_of: scenario.as_of.clone(),
            state,
            required_count,
            active_count,
            available_count,
            unavailable_count,
            unknown_count,
            members,
        })
    }
}

fn relation_is_active(relation: &crate::spatial_graph::SpatialRelation, as_of: &str) -> bool {
    relation
        .valid_from
        .as_deref()
        .is_none_or(|start| start <= as_of)
        && relation
            .valid_until
            .as_deref()
            .is_none_or(|end| as_of < end)
}

fn relation_assessment_index(
    report: &AssessedDependencyReport,
) -> BTreeMap<&str, &RelationEvidenceAssessment> {
    report
        .relation_assessments
        .iter()
        .map(|assessment| (assessment.relation_id.as_str(), assessment))
        .collect()
}

fn validate_scenario(
    scenario: &AvailabilityScenario,
    evidence_records: &[EvidenceRecord],
) -> Result<(), Vec<ScenarioValidationError>> {
    let mut errors = Vec::new();
    if scenario.id.trim().is_empty() {
        errors.push(ScenarioValidationError::EmptyScenarioId);
    }

    let evidence_by_id: BTreeMap<&str, &EvidenceRecord> = evidence_records
        .iter()
        .map(|record| (record.id.as_str(), record))
        .collect();

    for (entity_id, condition) in &scenario.entity_conditions {
        if entity_id.trim().is_empty() {
            errors.push(ScenarioValidationError::EmptyEntityId);
        }
        if condition.rationale.trim().is_empty() {
            errors.push(ScenarioValidationError::EmptyRationale);
        }
        if condition.basis == ScenarioBasis::Observed
            && condition.supporting_evidence_ids.is_empty()
        {
            errors.push(ScenarioValidationError::MissingEvidenceForObservedState);
        }

        let mut seen = BTreeSet::new();
        for evidence_id in &condition.supporting_evidence_ids {
            if !seen.insert(evidence_id.as_str()) {
                errors.push(ScenarioValidationError::DuplicateSupportingEvidence);
            }
            let record = match evidence_by_id.get(evidence_id.as_str()) {
                Some(record) => record,
                None => {
                    errors.push(ScenarioValidationError::UnknownSupportingEvidence);
                    continue;
                }
            };

            if condition.basis == ScenarioBasis::Observed
                && !matches!(
                    record.state,
                    EvidenceState::Asserted | EvidenceState::Corroborated
                )
            {
                errors.push(ScenarioValidationError::InadmissibleSupportingEvidence);
            }

            if condition.basis == ScenarioBasis::Observed {
                if record.kind != EvidenceKind::Observation {
                    errors.push(ScenarioValidationError::EvidenceNotObservation);
                }
                if record.subject_id.as_str() != entity_id.as_str()
                    && record.spatial.entity_id.as_str() != entity_id.as_str()
                {
                    errors.push(ScenarioValidationError::EvidenceSubjectMismatch);
                }
            }

            let start = record.time.valid_from.as_deref();
            let end = record.time.valid_until.as_deref();
            let invalid_window = start.is_some_and(|value| !is_canonical_utc_timestamp(value))
                || end.is_some_and(|value| !is_canonical_utc_timestamp(value))
                || matches!((start, end), (Some(first), Some(last)) if {
                    is_canonical_utc_timestamp(first)
                        && is_canonical_utc_timestamp(last)
                        && first >= last
                });
            if invalid_window {
                errors.push(ScenarioValidationError::InvalidSupportingEvidenceValidity);
            } else if condition.basis == ScenarioBasis::Observed
                && (start.is_some_and(|value| value > scenario.as_of.as_str())
                    || end.is_some_and(|value| scenario.as_of.as_str() >= value))
            {
                errors.push(ScenarioValidationError::OutsideSupportingEvidenceValidity);
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spatial_economic::{
        EVIDENCE_SCHEMA_VERSION, EvidenceKind, EvidenceRecord, EvidenceState, Money, Provenance,
        Quantity, SourceArtifact, SpatialContext, TimeContext,
    };
    use crate::spatial_graph::{RelationKind, SpatialRelation};

    fn evidence(id: &str) -> EvidenceRecord {
        EvidenceRecord {
            id: id.into(),
            schema_version: EVIDENCE_SCHEMA_VERSION.into(),
            kind: EvidenceKind::Observation,
            state: EvidenceState::Asserted,
            issuer_id: "did:example:scenario-tests".into(),
            subject_id: "urn:example:component".into(),
            spatial: SpatialContext {
                entity_id: "urn:example:component".into(),
                geometry_uri: None,
                crs: None,
            },
            time: TimeContext {
                occurred_at: "2026-10-15T10:00:00Z".into(),
                recorded_at: "2026-10-15T10:01:00Z".into(),
                valid_from: None,
                valid_until: None,
            },
            statement: "Synthetic scenario test evidence".into(),
            quantity: Some(Quantity {
                value: 1.0,
                unit: "item".into(),
                uncertainty_abs: None,
            }),
            money: Some(Money {
                amount_decimal: "1.00".into(),
                currency: "ZAR".into(),
            }),
            provenance: Provenance {
                method: "synthetic_test".into(),
                source_artifacts: vec![SourceArtifact {
                    id: format!("{id}:artifact"),
                    uri: format!("https://example.invalid/{id}"),
                    role: "synthetic_fixture".into(),
                    sha256: None,
                }],
                derivation_id: None,
            },
            supersedes: None,
            access_policy: "policy:synthetic-test".into(),
        }
    }

    fn relation(
        id: &str,
        dependent: &str,
        prerequisite: &str,
        evidence_id: &str,
    ) -> SpatialRelation {
        SpatialRelation {
            id: id.into(),
            subject_entity_id: dependent.into(),
            kind: RelationKind::SuppliedBy,
            object_entity_id: prerequisite.into(),
            valid_from: None,
            valid_until: None,
            supporting_evidence_ids: vec![evidence_id.into()],
        }
    }

    fn assumed(state: EntityOperationalState) -> ScenarioEntityCondition {
        ScenarioEntityCondition {
            state,
            basis: ScenarioBasis::Assumed,
            rationale: "Explicit synthetic test assumption".into(),
            supporting_evidence_ids: Vec::new(),
        }
    }

    fn scenario(
        id: &str,
        conditions: impl IntoIterator<Item = (String, ScenarioEntityCondition)>,
    ) -> AvailabilityScenario {
        AvailabilityScenario {
            id: id.into(),
            as_of: "2026-10-15T10:00:00Z".into(),
            entity_conditions: conditions.into_iter().collect(),
        }
    }

    fn group(logic: DependencyLogic, relation_ids: &[&str]) -> DependencyGroup {
        DependencyGroup {
            id: "synthetic-group".into(),
            dependent_entity_id: "pump".into(),
            logic,
            relation_ids: relation_ids.iter().map(|id| (*id).to_owned()).collect(),
        }
    }

    #[test]
    fn any_one_redundancy_survives_one_unavailable_input() {
        let graph = DependencyGraph {
            relations: vec![
                relation("pump-primary", "pump", "feeder-a", "ev-primary"),
                relation("pump-backup", "pump", "feeder-b", "ev-backup"),
            ],
        };
        let states = scenario(
            "any-one",
            [
                (
                    "feeder-a".into(),
                    assumed(EntityOperationalState::Unavailable),
                ),
                (
                    "feeder-b".into(),
                    assumed(EntityOperationalState::Available),
                ),
            ],
        );
        let result = graph
            .evaluate_dependency_group_at(
                &group(DependencyLogic::AnyOne, &["pump-primary", "pump-backup"]),
                &states,
                &[evidence("ev-primary"), evidence("ev-backup")],
            )
            .unwrap();
        assert_eq!(result.state, DependencyGroupState::Satisfied);
        assert_eq!(result.available_count, 1);
        assert_eq!(result.unavailable_count, 1);
        assert_eq!(result.required_count, 1);
    }

    #[test]
    fn any_one_redundancy_is_disrupted_when_all_members_are_unavailable() {
        let graph = DependencyGraph {
            relations: vec![
                relation("pump-primary", "pump", "feeder-a", "ev-primary"),
                relation("pump-backup", "pump", "feeder-b", "ev-backup"),
            ],
        };
        let states = scenario(
            "both-failed",
            [
                (
                    "feeder-a".into(),
                    assumed(EntityOperationalState::Unavailable),
                ),
                (
                    "feeder-b".into(),
                    assumed(EntityOperationalState::Unavailable),
                ),
            ],
        );
        let result = graph
            .evaluate_dependency_group_at(
                &group(DependencyLogic::AnyOne, &["pump-primary", "pump-backup"]),
                &states,
                &[evidence("ev-primary"), evidence("ev-backup")],
            )
            .unwrap();
        assert_eq!(result.state, DependencyGroupState::Disrupted);
        assert_eq!(result.available_count, 0);
        assert_eq!(result.unknown_count, 0);
    }

    #[test]
    fn two_of_three_distinguishes_satisfied_disrupted_and_indeterminate() {
        let graph = DependencyGraph {
            relations: vec![
                relation("pump-a", "pump", "feeder-a", "ev-a"),
                relation("pump-b", "pump", "feeder-b", "ev-b"),
                relation("pump-c", "pump", "feeder-c", "ev-c"),
            ],
        };
        let logic = DependencyLogic::AtLeast { minimum: 2 };
        let dep_group = group(logic, &["pump-a", "pump-b", "pump-c"]);
        let evidence = [evidence("ev-a"), evidence("ev-b"), evidence("ev-c")];

        let satisfied = scenario(
            "satisfied",
            [
                (
                    "feeder-a".into(),
                    assumed(EntityOperationalState::Available),
                ),
                (
                    "feeder-b".into(),
                    assumed(EntityOperationalState::Available),
                ),
                (
                    "feeder-c".into(),
                    assumed(EntityOperationalState::Unavailable),
                ),
            ],
        );
        assert_eq!(
            graph
                .evaluate_dependency_group_at(&dep_group, &satisfied, &evidence)
                .unwrap()
                .state,
            DependencyGroupState::Satisfied
        );

        let disrupted = scenario(
            "disrupted",
            [
                (
                    "feeder-a".into(),
                    assumed(EntityOperationalState::Available),
                ),
                (
                    "feeder-b".into(),
                    assumed(EntityOperationalState::Unavailable),
                ),
                (
                    "feeder-c".into(),
                    assumed(EntityOperationalState::Unavailable),
                ),
            ],
        );
        assert_eq!(
            graph
                .evaluate_dependency_group_at(&dep_group, &disrupted, &evidence)
                .unwrap()
                .state,
            DependencyGroupState::Disrupted
        );

        let uncertain = scenario(
            "uncertain",
            [
                (
                    "feeder-a".into(),
                    assumed(EntityOperationalState::Available),
                ),
                (
                    "feeder-b".into(),
                    assumed(EntityOperationalState::Unavailable),
                ),
            ],
        );
        let result = graph
            .evaluate_dependency_group_at(&dep_group, &uncertain, &evidence)
            .unwrap();
        assert_eq!(result.state, DependencyGroupState::Indeterminate);
        assert_eq!(result.unknown_count, 1);
    }

    #[test]
    fn all_required_group_is_disrupted_when_one_required_input_is_unavailable() {
        let graph = DependencyGraph {
            relations: vec![
                relation("pump-power", "pump", "feeder", "ev-power"),
                relation("pump-water", "pump", "water", "ev-water"),
            ],
        };
        let states = scenario(
            "series-dependency",
            [
                ("feeder".into(), assumed(EntityOperationalState::Available)),
                ("water".into(), assumed(EntityOperationalState::Unavailable)),
            ],
        );
        let result = graph
            .evaluate_dependency_group_at(
                &group(DependencyLogic::AllRequired, &["pump-power", "pump-water"]),
                &states,
                &[evidence("ev-power"), evidence("ev-water")],
            )
            .unwrap();
        assert_eq!(result.state, DependencyGroupState::Disrupted);
        assert_eq!(result.required_count, 2);
    }

    #[test]
    fn disputed_backup_is_unknown_not_failed_or_available() {
        let graph = DependencyGraph {
            relations: vec![
                relation("pump-primary", "pump", "feeder-a", "ev-primary"),
                relation("pump-backup", "pump", "feeder-b", "ev-backup"),
            ],
        };
        let states = scenario(
            "unverified-backup",
            [
                (
                    "feeder-a".into(),
                    assumed(EntityOperationalState::Unavailable),
                ),
                (
                    "feeder-b".into(),
                    assumed(EntityOperationalState::Available),
                ),
            ],
        );
        let mut backup = evidence("ev-backup");
        backup.state = EvidenceState::Disputed;
        let result = graph
            .evaluate_dependency_group_at(
                &group(DependencyLogic::AnyOne, &["pump-primary", "pump-backup"]),
                &states,
                &[evidence("ev-primary"), backup],
            )
            .unwrap();
        assert_eq!(result.state, DependencyGroupState::Indeterminate);
        assert_eq!(result.unknown_count, 1);
        assert!(result.members.iter().any(|member| {
            member.relation_id == "pump-backup"
                && member.state == DependencyMemberState::Unknown
                && member.unknown_reason == Some(UnknownMemberReason::EvidenceBlocked)
        }));
    }

    #[test]
    fn missing_condition_defaults_to_unknown_and_no_active_group_is_indeterminate() {
        let graph = DependencyGraph {
            relations: vec![relation("pump-source", "pump", "feeder", "ev-source")],
        };
        let no_condition = scenario("missing-condition", []);
        let result = graph
            .evaluate_dependency_group_at(
                &group(DependencyLogic::AnyOne, &["pump-source"]),
                &no_condition,
                &[evidence("ev-source")],
            )
            .unwrap();
        assert_eq!(result.state, DependencyGroupState::Indeterminate);
        assert_eq!(result.unknown_count, 1);
        assert_eq!(
            result.members[0].unknown_reason,
            Some(UnknownMemberReason::MissingScenarioCondition)
        );

        let mut inactive_relation = relation("pump-source", "pump", "feeder", "ev-source");
        inactive_relation.valid_until = Some("2026-10-15T09:00:00Z".into());
        let inactive_graph = DependencyGraph {
            relations: vec![inactive_relation],
        };
        let result = inactive_graph
            .evaluate_dependency_group_at(
                &group(DependencyLogic::AnyOne, &["pump-source"]),
                &scenario("no-active-members", []),
                &[evidence("ev-source")],
            )
            .unwrap();
        assert_eq!(result.state, DependencyGroupState::Indeterminate);
        assert_eq!(result.active_count, 0);
        assert_eq!(result.members[0].state, DependencyMemberState::Inactive);
    }

    #[test]
    fn observed_conditions_require_admissible_supporting_evidence() {
        let graph = DependencyGraph {
            relations: vec![relation("pump-source", "pump", "feeder", "ev-source")],
        };
        let mut condition = assumed(EntityOperationalState::Available);
        condition.basis = ScenarioBasis::Observed;
        condition.supporting_evidence_ids = vec!["ev-proposed".into()];
        let states = scenario("invalid-observation", [("feeder".into(), condition)]);
        let mut proposed = evidence("ev-proposed");
        proposed.state = EvidenceState::Proposed;
        let result = graph
            .evaluate_dependency_group_at(
                &group(DependencyLogic::AnyOne, &["pump-source"]),
                &states,
                &[evidence("ev-source"), proposed],
            )
            .unwrap_err();
        assert!(matches!(
            result,
            ScenarioEvaluationError::InvalidScenario(errors)
                if errors.contains(&ScenarioValidationError::InadmissibleSupportingEvidence)
        ));
    }

    #[test]
    fn observed_condition_rejects_forecasts_and_expired_observations() {
        let graph = DependencyGraph {
            relations: vec![relation("pump-source", "pump", "feeder", "ev-link")],
        };
        let mut state_record = evidence("ev-state");
        state_record.subject_id = "feeder".into();
        state_record.spatial.entity_id = "feeder".into();
        let condition = ScenarioEntityCondition {
            state: EntityOperationalState::Available,
            basis: ScenarioBasis::Observed,
            rationale: "Synthetic observed condition".into(),
            supporting_evidence_ids: vec!["ev-state".into()],
        };
        let states = scenario("observed-condition", [("feeder".into(), condition)]);
        let dep_group = group(DependencyLogic::AnyOne, &["pump-source"]);

        state_record.kind = EvidenceKind::Forecast;
        let wrong_kind = graph
            .evaluate_dependency_group_at(
                &dep_group,
                &states,
                &[evidence("ev-link"), state_record.clone()],
            )
            .unwrap_err();
        assert!(matches!(
            wrong_kind,
            ScenarioEvaluationError::InvalidScenario(errors)
                if errors.contains(&ScenarioValidationError::EvidenceNotObservation)
        ));

        state_record.kind = EvidenceKind::Observation;
        state_record.time.valid_until = Some("2026-10-10T00:00:00Z".into());
        let expired = graph
            .evaluate_dependency_group_at(&dep_group, &states, &[evidence("ev-link"), state_record])
            .unwrap_err();
        assert!(matches!(
            expired,
            ScenarioEvaluationError::InvalidScenario(errors)
                if errors.contains(&ScenarioValidationError::OutsideSupportingEvidenceValidity)
        ));
    }

    #[test]
    fn dependency_group_validation_rejects_bad_members_and_thresholds() {
        let graph = DependencyGraph {
            relations: vec![
                relation("pump-source", "pump", "feeder", "ev-source"),
                SpatialRelation {
                    id: "pump-location".into(),
                    subject_entity_id: "pump".into(),
                    kind: RelationKind::LocatedAt,
                    object_entity_id: "site".into(),
                    valid_from: None,
                    valid_until: None,
                    supporting_evidence_ids: vec!["ev-location".into()],
                },
                relation("field-pump", "field", "pump", "ev-field"),
            ],
        };
        assert!(
            graph
                .validate_dependency_group(&group(DependencyLogic::AnyOne, &["missing"]))
                .unwrap_err()
                .contains(&DependencyGroupValidationError::UnknownRelationId)
        );
        assert!(
            graph
                .validate_dependency_group(&group(DependencyLogic::AnyOne, &["pump-location"]))
                .unwrap_err()
                .contains(&DependencyGroupValidationError::NonDependencyRelation)
        );
        assert!(
            graph
                .validate_dependency_group(&group(DependencyLogic::AnyOne, &["field-pump"]))
                .unwrap_err()
                .contains(&DependencyGroupValidationError::WrongDependentEntity)
        );
        assert!(
            graph
                .validate_dependency_group(&group(
                    DependencyLogic::AnyOne,
                    &["pump-source", "pump-source"],
                ))
                .unwrap_err()
                .contains(&DependencyGroupValidationError::DuplicateRelationId)
        );
        assert!(
            graph
                .validate_dependency_group(&group(
                    DependencyLogic::AtLeast { minimum: 0 },
                    &["pump-source"],
                ))
                .unwrap_err()
                .contains(&DependencyGroupValidationError::InvalidMinimum)
        );
    }

    #[test]
    fn synthetic_redundancy_fixture_evaluates_against_graph_and_schema() {
        #[derive(Deserialize)]
        struct Fixture {
            groups: Vec<DependencyGroup>,
        }

        let graph: DependencyGraph = serde_json::from_str(include_str!(
            "../../schemas/examples/water-system-dependencies.json"
        ))
        .unwrap();
        let scenario: AvailabilityScenario = serde_json::from_str(include_str!(
            "../../schemas/examples/water-system-availability-scenario.json"
        ))
        .unwrap();
        let fixture: Fixture = serde_json::from_str(include_str!(
            "../../schemas/examples/water-system-dependency-groups.json"
        ))
        .unwrap();
        let schema: serde_json::Value = serde_json::from_str(include_str!(
            "../../schemas/spatial-economic-dependency-groups-v0.1.schema.json"
        ))
        .unwrap();
        let scenario_schema: serde_json::Value = serde_json::from_str(include_str!(
            "../../schemas/spatial-economic-availability-scenario-v0.1.schema.json"
        ))
        .unwrap();
        assert_eq!(scenario_schema["properties"]["as_of"]["type"], "string");
        assert!(
            scenario_schema["properties"]["entity_conditions"]["additionalProperties"]
                ["properties"]["basis"]["enum"]
                .as_array()
                .unwrap()
                .contains(&serde_json::json!("assumed"))
        );

        let choices = schema["$defs"]["logic"]["oneOf"].as_array().unwrap();
        assert!(
            choices
                .iter()
                .any(|item| item["properties"]["mode"]["const"] == "all_required")
        );
        assert!(
            choices
                .iter()
                .any(|item| item["properties"]["mode"]["const"] == "any_one")
        );
        assert!(
            choices
                .iter()
                .any(|item| item["properties"]["mode"]["const"] == "at_least")
        );
        assert_eq!(
            serde_json::to_value(DependencyLogic::AtLeast { minimum: 2 }).unwrap(),
            serde_json::json!({"mode":"at_least","minimum":2})
        );

        let evidence_records = [
            evidence("synthetic-evidence-pump-power"),
            evidence("synthetic-evidence-pump-backup-power"),
            evidence("synthetic-evidence-field-pump"),
            evidence("synthetic-evidence-warehouse-location"),
        ];
        graph.validate(&evidence_records).unwrap();
        let assessment = graph
            .evaluate_dependency_group_at(&fixture.groups[0], &scenario, &evidence_records)
            .unwrap();
        assert_eq!(assessment.state, DependencyGroupState::Satisfied);
        assert_eq!(assessment.available_count, 1);
        assert_eq!(assessment.unavailable_count, 1);
    }

    #[test]
    fn observed_condition_accepts_matching_asserted_observation() {
        let graph = DependencyGraph {
            relations: vec![relation("pump-source", "pump", "feeder", "ev-link")],
        };
        let mut state_record = evidence("ev-state");
        state_record.subject_id = "feeder".into();
        state_record.spatial.entity_id = "feeder".into();
        let condition = ScenarioEntityCondition {
            state: EntityOperationalState::Available,
            basis: ScenarioBasis::Observed,
            rationale: "Synthetic observation fixture".into(),
            supporting_evidence_ids: vec!["ev-state".into()],
        };
        let states = scenario("observed-state", [("feeder".into(), condition)]);
        let result = graph
            .evaluate_dependency_group_at(
                &group(DependencyLogic::AnyOne, &["pump-source"]),
                &states,
                &[evidence("ev-link"), state_record],
            )
            .unwrap();
        assert_eq!(result.state, DependencyGroupState::Satisfied);
        assert_eq!(result.members[0].state, DependencyMemberState::Available);
    }
}
