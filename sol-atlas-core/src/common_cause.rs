// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Evidence-gated common-cause failure domains for spatial-economic scenarios.
//!
//! Redundant paths are not necessarily independent: multiple assets may share a
//! feeder, control plane, site, supplier, workforce or hazard exposure. This
//! module records and surfaces that overlap. It deliberately does not mark an
//! exposed asset unavailable, infer a failure probability, or prove causation.

use crate::dependency_logic::{DependencyGroupAssessment, DependencyLogic, DependencyMemberState};
use crate::spatial_economic::{EvidenceKind, EvidenceRecord, EvidenceState};
use crate::spatial_graph::is_canonical_utc_timestamp;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const COMMON_CAUSE_SCHEMA_VERSION: &str = "0.1.0";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureMechanism {
    SharedPower,
    SharedControl,
    SharedNetwork,
    SharedSite,
    Flood,
    Heat,
    Fire,
    SharedSupplier,
    SharedWorkforce,
    Cyber,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommonCauseState {
    Active,
    Inactive,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommonCauseBasis {
    Assumed,
    Observed,
}

/// A declared set of entities exposed to one shared failure mechanism.
///
/// Supporting evidence must refer to the domain ID and be asserted/corroborated
/// and temporally valid before the domain can qualify an exposure assessment.
/// The member list is a model boundary; it is not itself proof that the mechanism
/// will cause those entities to fail.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommonCauseDomain {
    pub id: String,
    pub cause_entity_id: String,
    pub mechanism: FailureMechanism,
    pub member_entity_ids: Vec<String>,
    pub supporting_evidence_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_from: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_until: Option<String>,
}

/// An explicit condition for a common-cause mechanism, keyed by domain ID.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommonCauseCondition {
    pub state: CommonCauseState,
    pub basis: CommonCauseBasis,
    pub rationale: String,
    #[serde(default)]
    pub supporting_evidence_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommonCauseScenario {
    pub id: String,
    /// Canonical UTC timestamp: YYYY-MM-DDTHH:MM:SSZ.
    pub as_of: String,
    #[serde(default)]
    pub conditions: BTreeMap<String, CommonCauseCondition>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommonCauseValidationError {
    EmptyScenarioId,
    InvalidAsOfTimestamp,
    DuplicateEvidenceRecordId,
    InvalidEvidenceRecord,
    EmptyDomainId,
    DuplicateDomainId,
    EmptyCauseEntityId,
    EmptyMemberSet,
    EmptyMemberEntityId,
    DuplicateMemberEntityId,
    MissingDomainEvidence,
    DuplicateDomainEvidenceId,
    UnknownEvidenceReference,
    InvalidDomainValidFrom,
    InvalidDomainValidUntil,
    InvalidDomainValidityWindow,
    UnknownScenarioDomain,
    EmptyRationale,
    EvidenceRequiredForObservedCondition,
    EvidenceForbiddenForAssumedCondition,
    DuplicateConditionEvidenceId,
    InadmissibleConditionEvidence,
    ConditionEvidenceWrongKind,
    ConditionEvidenceSubjectMismatch,
    InvalidConditionEvidenceValidity,
    OutsideConditionEvidenceValidity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DomainEvidenceBlockReason {
    ProposedEvidence,
    DisputedEvidence,
    SupersededEvidence,
    WithdrawnEvidence,
    WrongKind,
    WrongSubject,
    InvalidEvidenceValidity,
    OutsideEvidenceValidity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DomainEvidenceBlocker {
    pub evidence_id: String,
    pub reason: DomainEvidenceBlockReason,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommonCauseExposure {
    /// The mechanism is active and the entity is in the declared shared domain.
    /// This is exposure, not a finding that the entity has failed.
    Exposed,
    /// The mechanism is explicitly inactive in this scenario.
    NotTriggered,
    /// The mechanism's state is missing or explicitly unknown.
    Indeterminate,
    /// The domain's evidence is inadmissible or out of scope.
    EvidenceBlocked,
    /// The domain does not apply at the scenario's as-of instant.
    OutsideDomainValidity,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommonCauseEntityExposure {
    pub entity_id: String,
    pub exposure: CommonCauseExposure,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommonCauseAssessment {
    pub domain_id: String,
    pub cause_entity_id: String,
    pub mechanism: FailureMechanism,
    pub as_of: String,
    pub state: CommonCauseState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub basis: Option<CommonCauseBasis>,
    pub domain_evidence_blockers: Vec<DomainEvidenceBlocker>,
    pub condition_evidence_ids: Vec<String>,
    pub entity_exposures: Vec<CommonCauseEntityExposure>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommonCauseAssessmentError {
    InvalidInput(Vec<CommonCauseValidationError>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommonCauseCorrelationError {
    InvalidAsOfTimestamp,
    DuplicateDomainAssessmentId,
    DuplicateDependencyGroupId,
    DuplicateEntityExposure,
    DuplicateDependencyMemberRelationId,
    MismatchedAsOfTimestamp,
}

/// Structural overlap between a declared common-cause domain and one dependency group.
///
/// The warning means at least two alternatives in this group share one qualified
/// failure domain. It does not say those alternatives will fail, nor quantify how
/// likely that is. Active exposure is reported separately from structural overlap.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CommonCauseGroupOverlap {
    pub domain_id: String,
    pub group_id: String,
    pub as_of: String,
    pub domain_state: CommonCauseState,
    pub logic: DependencyLogic,
    pub domain_evidence_admissible: bool,
    pub shared_relation_ids: Vec<String>,
    pub shared_prerequisite_entity_ids: Vec<String>,
    pub currently_exposed_relation_ids: Vec<String>,
    pub independent_redundancy_warning: bool,
}

/// Join current common-cause exposure assessments to qualitative dependency groups.
///
/// Only overlapping pairs are returned, sorted by (domain ID, group ID). Inputs
/// must refer to one as-of instant; combining different snapshots is rejected.
pub fn correlate_common_cause_with_dependency_groups(
    domains: &[CommonCauseAssessment],
    groups: &[DependencyGroupAssessment],
) -> Result<Vec<CommonCauseGroupOverlap>, CommonCauseCorrelationError> {
    let expected_as_of = domains
        .first()
        .map(|value| value.as_of.as_str())
        .or_else(|| groups.first().map(|value| value.as_of.as_str()));
    if let Some(as_of) = expected_as_of {
        if !is_canonical_utc_timestamp(as_of)
            || domains
                .iter()
                .any(|value| !is_canonical_utc_timestamp(&value.as_of))
            || groups
                .iter()
                .any(|value| !is_canonical_utc_timestamp(&value.as_of))
        {
            return Err(CommonCauseCorrelationError::InvalidAsOfTimestamp);
        }
        if domains.iter().any(|value| value.as_of != as_of)
            || groups.iter().any(|value| value.as_of != as_of)
        {
            return Err(CommonCauseCorrelationError::MismatchedAsOfTimestamp);
        }
    }

    for domain in domains {
        let mut entity_ids = BTreeSet::new();
        if domain
            .entity_exposures
            .iter()
            .any(|value| !entity_ids.insert(value.entity_id.as_str()))
        {
            return Err(CommonCauseCorrelationError::DuplicateEntityExposure);
        }
    }
    for group in groups {
        let mut relation_ids = BTreeSet::new();
        if group
            .members
            .iter()
            .any(|value| !relation_ids.insert(value.relation_id.as_str()))
        {
            return Err(CommonCauseCorrelationError::DuplicateDependencyMemberRelationId);
        }
    }

    let mut domain_ids = BTreeSet::new();
    if domains
        .iter()
        .any(|value| !domain_ids.insert(value.domain_id.as_str()))
    {
        return Err(CommonCauseCorrelationError::DuplicateDomainAssessmentId);
    }
    let mut group_ids = BTreeSet::new();
    if groups
        .iter()
        .any(|value| !group_ids.insert(value.group_id.as_str()))
    {
        return Err(CommonCauseCorrelationError::DuplicateDependencyGroupId);
    }

    let mut ordered_domains: Vec<&CommonCauseAssessment> = domains.iter().collect();
    ordered_domains.sort_by(|left, right| left.domain_id.cmp(&right.domain_id));
    let mut ordered_groups: Vec<&DependencyGroupAssessment> = groups.iter().collect();
    ordered_groups.sort_by(|left, right| left.group_id.cmp(&right.group_id));
    let mut overlaps = Vec::new();

    for domain in ordered_domains {
        let exposure_by_entity: BTreeMap<&str, CommonCauseExposure> = domain
            .entity_exposures
            .iter()
            .map(|value| (value.entity_id.as_str(), value.exposure))
            .collect();
        let domain_evidence_admissible = domain.domain_evidence_blockers.is_empty()
            && !domain.entity_exposures.is_empty()
            && domain
                .entity_exposures
                .iter()
                .all(|value| value.exposure != CommonCauseExposure::OutsideDomainValidity);

        for group in &ordered_groups {
            let mut shared_relation_ids = Vec::new();
            let mut shared_entities = BTreeSet::new();
            let mut currently_exposed_relation_ids = Vec::new();

            for member in &group.members {
                if member.state == DependencyMemberState::Inactive
                    || !exposure_by_entity.contains_key(member.prerequisite_entity_id.as_str())
                {
                    continue;
                }
                shared_relation_ids.push(member.relation_id.clone());
                shared_entities.insert(member.prerequisite_entity_id.clone());
                if exposure_by_entity.get(member.prerequisite_entity_id.as_str())
                    == Some(&CommonCauseExposure::Exposed)
                {
                    currently_exposed_relation_ids.push(member.relation_id.clone());
                }
            }

            if shared_relation_ids.is_empty() {
                continue;
            }
            shared_relation_ids.sort();
            currently_exposed_relation_ids.sort();
            let shared_prerequisite_entity_ids: Vec<String> = shared_entities.into_iter().collect();

            overlaps.push(CommonCauseGroupOverlap {
                domain_id: domain.domain_id.clone(),
                group_id: group.group_id.clone(),
                as_of: group.as_of.clone(),
                domain_state: domain.state,
                logic: group.logic,
                domain_evidence_admissible,
                independent_redundancy_warning: domain_evidence_admissible
                    && shared_prerequisite_entity_ids.len() >= 2,
                shared_relation_ids,
                shared_prerequisite_entity_ids,
                currently_exposed_relation_ids,
            });
        }
    }

    Ok(overlaps)
}

impl CommonCauseDomain {
    fn is_valid_at(&self, at: &str) -> bool {
        self.valid_from.as_deref().is_none_or(|start| start <= at)
            && self.valid_until.as_deref().is_none_or(|end| at < end)
    }
}

/// Assess shared-cause exposure without mutating the operational state of assets.
///
/// This fail-closed API validates all referenced records and scenario assumptions,
/// returns domains in stable ID order and member exposures in stable entity-ID
/// order. Missing cause conditions remain indeterminate. A domain with blocked
/// supporting evidence cannot label its members as exposed or safe.
pub fn assess_common_cause_domains(
    domains: &[CommonCauseDomain],
    scenario: &CommonCauseScenario,
    evidence_records: &[EvidenceRecord],
) -> Result<Vec<CommonCauseAssessment>, CommonCauseAssessmentError> {
    let mut errors = Vec::new();

    if scenario.id.trim().is_empty() {
        errors.push(CommonCauseValidationError::EmptyScenarioId);
    }
    if !is_canonical_utc_timestamp(&scenario.as_of) {
        errors.push(CommonCauseValidationError::InvalidAsOfTimestamp);
    }

    let mut evidence_by_id = BTreeMap::new();
    for record in evidence_records {
        if evidence_by_id.insert(record.id.as_str(), record).is_some() {
            errors.push(CommonCauseValidationError::DuplicateEvidenceRecordId);
        }
        if record.validate().is_err() {
            errors.push(CommonCauseValidationError::InvalidEvidenceRecord);
        }
    }

    let mut domain_by_id = BTreeMap::new();
    for domain in domains {
        if domain.id.trim().is_empty() {
            errors.push(CommonCauseValidationError::EmptyDomainId);
        }
        if domain_by_id.insert(domain.id.as_str(), domain).is_some() {
            errors.push(CommonCauseValidationError::DuplicateDomainId);
        }
        if domain.cause_entity_id.trim().is_empty() {
            errors.push(CommonCauseValidationError::EmptyCauseEntityId);
        }
        if domain.member_entity_ids.is_empty() {
            errors.push(CommonCauseValidationError::EmptyMemberSet);
        }
        let mut members = BTreeSet::new();
        for member_id in &domain.member_entity_ids {
            if member_id.trim().is_empty() {
                errors.push(CommonCauseValidationError::EmptyMemberEntityId);
            }
            if !members.insert(member_id.as_str()) {
                errors.push(CommonCauseValidationError::DuplicateMemberEntityId);
            }
        }
        if domain.supporting_evidence_ids.is_empty() {
            errors.push(CommonCauseValidationError::MissingDomainEvidence);
        }
        let mut support_ids = BTreeSet::new();
        for evidence_id in &domain.supporting_evidence_ids {
            if evidence_id.trim().is_empty() || !support_ids.insert(evidence_id.as_str()) {
                errors.push(CommonCauseValidationError::DuplicateDomainEvidenceId);
            }
            if !evidence_by_id.contains_key(evidence_id.as_str()) {
                errors.push(CommonCauseValidationError::UnknownEvidenceReference);
            }
        }
        if let Some(start) = domain.valid_from.as_deref() {
            if !is_canonical_utc_timestamp(start) {
                errors.push(CommonCauseValidationError::InvalidDomainValidFrom);
            }
        }
        if let Some(end) = domain.valid_until.as_deref() {
            if !is_canonical_utc_timestamp(end) {
                errors.push(CommonCauseValidationError::InvalidDomainValidUntil);
            }
        }
        if let (Some(start), Some(end)) =
            (domain.valid_from.as_deref(), domain.valid_until.as_deref())
        {
            if is_canonical_utc_timestamp(start) && is_canonical_utc_timestamp(end) && start >= end
            {
                errors.push(CommonCauseValidationError::InvalidDomainValidityWindow);
            }
        }
    }

    for (domain_id, condition) in &scenario.conditions {
        let Some(domain) = domain_by_id.get(domain_id.as_str()) else {
            errors.push(CommonCauseValidationError::UnknownScenarioDomain);
            continue;
        };
        if condition.rationale.trim().is_empty() {
            errors.push(CommonCauseValidationError::EmptyRationale);
        }
        let mut seen = BTreeSet::new();
        for evidence_id in &condition.supporting_evidence_ids {
            if !seen.insert(evidence_id.as_str()) {
                errors.push(CommonCauseValidationError::DuplicateConditionEvidenceId);
            }
            if !evidence_by_id.contains_key(evidence_id.as_str()) {
                errors.push(CommonCauseValidationError::UnknownEvidenceReference);
            }
        }
        match condition.basis {
            CommonCauseBasis::Assumed => {
                if !condition.supporting_evidence_ids.is_empty() {
                    errors.push(CommonCauseValidationError::EvidenceForbiddenForAssumedCondition);
                }
            }
            CommonCauseBasis::Observed => {
                if condition.supporting_evidence_ids.is_empty() {
                    errors.push(CommonCauseValidationError::EvidenceRequiredForObservedCondition);
                }
                for evidence_id in &condition.supporting_evidence_ids {
                    let Some(record) = evidence_by_id.get(evidence_id.as_str()) else {
                        continue;
                    };
                    if record.kind != EvidenceKind::Observation {
                        errors.push(CommonCauseValidationError::ConditionEvidenceWrongKind);
                    }
                    if record.subject_id != domain.cause_entity_id
                        || record.spatial.entity_id != domain.cause_entity_id
                    {
                        errors.push(CommonCauseValidationError::ConditionEvidenceSubjectMismatch);
                    }
                    if !evidence_is_admissible_at(record, &scenario.as_of) {
                        errors.push(CommonCauseValidationError::InadmissibleConditionEvidence);
                    }
                    let start = record.time.valid_from.as_deref();
                    let end = record.time.valid_until.as_deref();
                    if start.is_some_and(|value| !is_canonical_utc_timestamp(value))
                        || end.is_some_and(|value| !is_canonical_utc_timestamp(value))
                        || matches!((start, end), (Some(first), Some(last)) if {
                            is_canonical_utc_timestamp(first)
                                && is_canonical_utc_timestamp(last)
                                && first >= last
                        })
                    {
                        errors.push(CommonCauseValidationError::InvalidConditionEvidenceValidity);
                    } else if start.is_some_and(|value| value > scenario.as_of.as_str())
                        || end.is_some_and(|value| scenario.as_of.as_str() >= value)
                    {
                        errors.push(CommonCauseValidationError::OutsideConditionEvidenceValidity);
                    }
                }
            }
        }
    }

    if !errors.is_empty() {
        return Err(CommonCauseAssessmentError::InvalidInput(errors));
    }

    let mut ordered_domains: Vec<&CommonCauseDomain> = domains.iter().collect();
    ordered_domains.sort_by(|left, right| left.id.cmp(&right.id));
    let mut assessments = Vec::with_capacity(ordered_domains.len());

    for domain in ordered_domains {
        let mut domain_evidence_blockers = Vec::new();
        for evidence_id in &domain.supporting_evidence_ids {
            let record = evidence_by_id[evidence_id.as_str()];
            domain_evidence_blockers.extend(domain_evidence_blockers_for(
                evidence_id,
                record,
                &domain.id,
                &scenario.as_of,
            ));
        }
        domain_evidence_blockers.sort_by(|left, right| {
            left.evidence_id
                .cmp(&right.evidence_id)
                .then_with(|| (left.reason as u8).cmp(&(right.reason as u8)))
        });

        let condition = scenario.conditions.get(&domain.id);
        let condition_evidence_ids = condition
            .map(|value| value.supporting_evidence_ids.clone())
            .unwrap_or_default();
        let state = condition
            .map(|value| value.state)
            .unwrap_or(CommonCauseState::Unknown);
        let basis = condition.map(|value| value.basis);
        let domain_in_scope = domain.is_valid_at(&scenario.as_of);
        let blocked = !domain_evidence_blockers.is_empty();

        let mut members: Vec<&str> = domain
            .member_entity_ids
            .iter()
            .map(String::as_str)
            .collect();
        members.sort_unstable();
        let entity_exposures = members
            .into_iter()
            .map(|entity_id| {
                let exposure = if !domain_in_scope {
                    CommonCauseExposure::OutsideDomainValidity
                } else if blocked {
                    CommonCauseExposure::EvidenceBlocked
                } else {
                    match state {
                        CommonCauseState::Active => CommonCauseExposure::Exposed,
                        CommonCauseState::Inactive => CommonCauseExposure::NotTriggered,
                        CommonCauseState::Unknown => CommonCauseExposure::Indeterminate,
                    }
                };
                CommonCauseEntityExposure {
                    entity_id: entity_id.to_owned(),
                    exposure,
                }
            })
            .collect();

        assessments.push(CommonCauseAssessment {
            domain_id: domain.id.clone(),
            cause_entity_id: domain.cause_entity_id.clone(),
            mechanism: domain.mechanism,
            as_of: scenario.as_of.clone(),
            state,
            basis,
            domain_evidence_blockers,
            condition_evidence_ids,
            entity_exposures,
        });
    }

    Ok(assessments)
}

fn evidence_is_admissible_at(record: &EvidenceRecord, at: &str) -> bool {
    matches!(
        record.state,
        EvidenceState::Asserted | EvidenceState::Corroborated
    ) && {
        let start = record.time.valid_from.as_deref();
        let end = record.time.valid_until.as_deref();
        !start.is_some_and(|value| !is_canonical_utc_timestamp(value))
            && !end.is_some_and(|value| !is_canonical_utc_timestamp(value))
            && !matches!((start, end), (Some(first), Some(last)) if {
                is_canonical_utc_timestamp(first)
                    && is_canonical_utc_timestamp(last)
                    && first >= last
            })
            && !start.is_some_and(|value| value > at)
            && !end.is_some_and(|value| at >= value)
    }
}

fn domain_evidence_blockers_for(
    evidence_id: &str,
    record: &EvidenceRecord,
    domain_id: &str,
    at: &str,
) -> Vec<DomainEvidenceBlocker> {
    let mut blockers = Vec::new();
    match record.state {
        EvidenceState::Asserted | EvidenceState::Corroborated => {}
        EvidenceState::Proposed => blockers.push(DomainEvidenceBlocker {
            evidence_id: evidence_id.into(),
            reason: DomainEvidenceBlockReason::ProposedEvidence,
        }),
        EvidenceState::Disputed => blockers.push(DomainEvidenceBlocker {
            evidence_id: evidence_id.into(),
            reason: DomainEvidenceBlockReason::DisputedEvidence,
        }),
        EvidenceState::Superseded => blockers.push(DomainEvidenceBlocker {
            evidence_id: evidence_id.into(),
            reason: DomainEvidenceBlockReason::SupersededEvidence,
        }),
        EvidenceState::Withdrawn => blockers.push(DomainEvidenceBlocker {
            evidence_id: evidence_id.into(),
            reason: DomainEvidenceBlockReason::WithdrawnEvidence,
        }),
    }

    if !matches!(
        record.kind,
        EvidenceKind::Observation | EvidenceKind::Scenario
    ) {
        blockers.push(DomainEvidenceBlocker {
            evidence_id: evidence_id.into(),
            reason: DomainEvidenceBlockReason::WrongKind,
        });
    }
    if record.subject_id != domain_id {
        blockers.push(DomainEvidenceBlocker {
            evidence_id: evidence_id.into(),
            reason: DomainEvidenceBlockReason::WrongSubject,
        });
    }
    let start = record.time.valid_from.as_deref();
    let end = record.time.valid_until.as_deref();
    if start.is_some_and(|value| !is_canonical_utc_timestamp(value))
        || end.is_some_and(|value| !is_canonical_utc_timestamp(value))
        || matches!((start, end), (Some(first), Some(last)) if {
            is_canonical_utc_timestamp(first)
                && is_canonical_utc_timestamp(last)
                && first >= last
        })
    {
        blockers.push(DomainEvidenceBlocker {
            evidence_id: evidence_id.into(),
            reason: DomainEvidenceBlockReason::InvalidEvidenceValidity,
        });
    } else if start.is_some_and(|value| value > at) || end.is_some_and(|value| at >= value) {
        blockers.push(DomainEvidenceBlocker {
            evidence_id: evidence_id.into(),
            reason: DomainEvidenceBlockReason::OutsideEvidenceValidity,
        });
    }
    blockers
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::spatial_economic::{
        EVIDENCE_SCHEMA_VERSION, Provenance, SpatialContext, TimeContext,
    };
    use serde::Deserialize;

    #[derive(Deserialize)]
    struct DomainFixture {
        domains: Vec<CommonCauseDomain>,
    }

    #[derive(Deserialize)]
    struct EvidenceFixture {
        records: Vec<EvidenceRecord>,
    }

    #[derive(Debug, Deserialize, PartialEq, Eq)]
    struct OverlapFixture {
        overlaps: Vec<CommonCauseGroupOverlap>,
    }

    const AT: &str = "2026-10-15T10:00:00Z";

    fn evidence(id: &str, subject_id: &str, spatial_entity_id: &str) -> EvidenceRecord {
        EvidenceRecord {
            id: id.into(),
            schema_version: EVIDENCE_SCHEMA_VERSION.into(),
            kind: EvidenceKind::Observation,
            state: EvidenceState::Asserted,
            issuer_id: "did:example:common-cause-tests".into(),
            subject_id: subject_id.into(),
            spatial: SpatialContext {
                entity_id: spatial_entity_id.into(),
                geometry_uri: None,
                crs: None,
            },
            time: TimeContext {
                occurred_at: AT.into(),
                recorded_at: AT.into(),
                valid_from: None,
                valid_until: None,
            },
            statement: "Synthetic common-cause test evidence".into(),
            quantity: None,
            money: None,
            provenance: Provenance {
                method: "synthetic_test".into(),
                source_artifacts: Vec::new(),
                derivation_id: None,
            },
            supersedes: None,
            access_policy: "policy:synthetic-test".into(),
        }
    }

    fn domain() -> CommonCauseDomain {
        CommonCauseDomain {
            id: "domain:shared-feeder".into(),
            cause_entity_id: "feeder:grid-1".into(),
            mechanism: FailureMechanism::SharedPower,
            member_entity_ids: vec!["pump:a".into(), "pump:b".into()],
            supporting_evidence_ids: vec!["ev:domain".into()],
            valid_from: None,
            valid_until: None,
        }
    }

    fn condition(state: CommonCauseState) -> CommonCauseCondition {
        CommonCauseCondition {
            state,
            basis: CommonCauseBasis::Assumed,
            rationale: "Synthetic explicit scenario condition".into(),
            supporting_evidence_ids: Vec::new(),
        }
    }

    fn scenario(condition: Option<CommonCauseCondition>) -> CommonCauseScenario {
        CommonCauseScenario {
            id: "scenario:test".into(),
            as_of: AT.into(),
            conditions: condition
                .map(|value| [("domain:shared-feeder".into(), value)].into())
                .unwrap_or_default(),
        }
    }

    #[test]
    fn shared_cause_overlap_flags_redundancy_assumption_without_rewriting_group_state() {
        use crate::dependency_logic::{DependencyGroupMemberAssessment, DependencyGroupState};

        let mut shared = domain();
        shared.member_entity_ids = vec!["feeder:a".into(), "feeder:b".into()];
        let common = assess_common_cause_domains(
            &[shared],
            &scenario(Some(condition(CommonCauseState::Active))),
            &[evidence(
                "ev:domain",
                "domain:shared-feeder",
                "feeder:grid-1",
            )],
        )
        .unwrap();
        let group = DependencyGroupAssessment {
            group_id: "pump:alternate-feeds".into(),
            dependent_entity_id: "pump:main".into(),
            logic: DependencyLogic::AnyOne,
            as_of: AT.into(),
            state: DependencyGroupState::Satisfied,
            required_count: 1,
            active_count: 2,
            available_count: 2,
            unavailable_count: 0,
            unknown_count: 0,
            members: vec![
                DependencyGroupMemberAssessment {
                    relation_id: "pump-from-a".into(),
                    supporting_evidence_ids: vec!["ev:a".into()],
                    prerequisite_entity_id: "feeder:a".into(),
                    state: DependencyMemberState::Available,
                    evidence_readiness: None,
                    evidence_blockers: Vec::new(),
                    unknown_reason: None,
                },
                DependencyGroupMemberAssessment {
                    relation_id: "pump-from-b".into(),
                    supporting_evidence_ids: vec!["ev:b".into()],
                    prerequisite_entity_id: "feeder:b".into(),
                    state: DependencyMemberState::Available,
                    evidence_readiness: None,
                    evidence_blockers: Vec::new(),
                    unknown_reason: None,
                },
            ],
        };

        let overlaps =
            correlate_common_cause_with_dependency_groups(&common, &[group.clone()]).unwrap();
        assert_eq!(overlaps.len(), 1);
        assert_eq!(
            overlaps[0].shared_relation_ids,
            vec!["pump-from-a", "pump-from-b"]
        );
        assert_eq!(
            overlaps[0].shared_prerequisite_entity_ids,
            vec!["feeder:a", "feeder:b"]
        );
        assert_eq!(overlaps[0].currently_exposed_relation_ids.len(), 2);
        assert!(overlaps[0].independent_redundancy_warning);
        // The existing qualitative group remains unchanged. Exposure is not failure.
        assert_eq!(group.state, DependencyGroupState::Satisfied);
    }

    #[test]
    fn common_cause_correlation_rejects_duplicate_exposure_entities() {
        let domain = CommonCauseAssessment {
            domain_id: "domain:one".into(),
            cause_entity_id: "cause:one".into(),
            mechanism: FailureMechanism::Other,
            as_of: AT.into(),
            state: CommonCauseState::Active,
            basis: Some(CommonCauseBasis::Assumed),
            domain_evidence_blockers: Vec::new(),
            condition_evidence_ids: Vec::new(),
            entity_exposures: vec![
                CommonCauseEntityExposure {
                    entity_id: "asset:one".into(),
                    exposure: CommonCauseExposure::Exposed,
                },
                CommonCauseEntityExposure {
                    entity_id: "asset:one".into(),
                    exposure: CommonCauseExposure::Indeterminate,
                },
            ],
        };
        assert_eq!(
            correlate_common_cause_with_dependency_groups(&[domain], &[]),
            Err(CommonCauseCorrelationError::DuplicateEntityExposure)
        );
    }

    #[test]
    fn common_cause_correlation_rejects_malformed_timestamp() {
        let domain = CommonCauseAssessment {
            domain_id: "domain:one".into(),
            cause_entity_id: "cause:one".into(),
            mechanism: FailureMechanism::Other,
            as_of: "2026-99-99T10:00:00Z".into(),
            state: CommonCauseState::Unknown,
            basis: None,
            domain_evidence_blockers: Vec::new(),
            condition_evidence_ids: Vec::new(),
            entity_exposures: Vec::new(),
        };
        assert_eq!(
            correlate_common_cause_with_dependency_groups(&[domain], &[]),
            Err(CommonCauseCorrelationError::InvalidAsOfTimestamp)
        );
    }

    #[test]
    fn common_cause_correlation_rejects_mixed_time_snapshots() {
        let common = CommonCauseAssessment {
            domain_id: "domain:one".into(),
            cause_entity_id: "cause:one".into(),
            mechanism: FailureMechanism::Other,
            as_of: AT.into(),
            state: CommonCauseState::Unknown,
            basis: None,
            domain_evidence_blockers: Vec::new(),
            condition_evidence_ids: Vec::new(),
            entity_exposures: vec![CommonCauseEntityExposure {
                entity_id: "asset:one".into(),
                exposure: CommonCauseExposure::Indeterminate,
            }],
        };
        let group = DependencyGroupAssessment {
            group_id: "group:one".into(),
            dependent_entity_id: "service:one".into(),
            logic: DependencyLogic::AnyOne,
            as_of: "2026-10-16T10:00:00Z".into(),
            state: crate::dependency_logic::DependencyGroupState::Indeterminate,
            required_count: 1,
            active_count: 1,
            available_count: 0,
            unavailable_count: 0,
            unknown_count: 1,
            members: Vec::new(),
        };
        assert_eq!(
            correlate_common_cause_with_dependency_groups(&[common], &[group]),
            Err(CommonCauseCorrelationError::MismatchedAsOfTimestamp)
        );
    }

    #[test]
    fn shared_feeder_json_fixtures_parse_and_correlate_with_dependency_alternatives() {
        use crate::dependency_logic::{DependencyGroupMemberAssessment, DependencyGroupState};

        let domains: DomainFixture = serde_json::from_str(include_str!(
            "../../schemas/examples/shared-feeder-domain.json"
        ))
        .unwrap();
        let scenario: CommonCauseScenario = serde_json::from_str(include_str!(
            "../../schemas/examples/shared-feeder-scenario.json"
        ))
        .unwrap();
        let evidence: EvidenceFixture = serde_json::from_str(include_str!(
            "../../schemas/examples/shared-feeder-evidence.json"
        ))
        .unwrap();
        let expected: OverlapFixture = serde_json::from_str(include_str!(
            "../../schemas/examples/shared-feeder-overlap.json"
        ))
        .unwrap();

        let assessments =
            assess_common_cause_domains(&domains.domains, &scenario, &evidence.records).unwrap();
        assert_eq!(assessments.len(), 1);
        assert!(
            assessments[0]
                .entity_exposures
                .iter()
                .all(|member| member.exposure == CommonCauseExposure::Exposed)
        );

        let group = DependencyGroupAssessment {
            group_id: "synthetic:pump-alternatives".into(),
            dependent_entity_id: "pump:service".into(),
            logic: DependencyLogic::AnyOne,
            as_of: scenario.as_of.clone(),
            state: DependencyGroupState::Satisfied,
            required_count: 1,
            active_count: 2,
            available_count: 2,
            unavailable_count: 0,
            unknown_count: 0,
            members: vec![
                DependencyGroupMemberAssessment {
                    relation_id: "synthetic:pump-a".into(),
                    supporting_evidence_ids: vec!["ev:a".into()],
                    prerequisite_entity_id: "pump:a".into(),
                    state: DependencyMemberState::Available,
                    evidence_readiness: None,
                    evidence_blockers: Vec::new(),
                    unknown_reason: None,
                },
                DependencyGroupMemberAssessment {
                    relation_id: "synthetic:pump-b".into(),
                    supporting_evidence_ids: vec!["ev:b".into()],
                    prerequisite_entity_id: "pump:b".into(),
                    state: DependencyMemberState::Available,
                    evidence_readiness: None,
                    evidence_blockers: Vec::new(),
                    unknown_reason: None,
                },
            ],
        };
        let overlaps =
            correlate_common_cause_with_dependency_groups(&assessments, &[group]).unwrap();
        assert_eq!(overlaps, expected.overlaps);
        assert_eq!(overlaps.len(), 1);
        assert!(overlaps[0].independent_redundancy_warning);
        assert_eq!(overlaps[0].currently_exposed_relation_ids.len(), 2);
    }

    #[test]
    fn repeated_relations_to_one_entity_do_not_count_as_redundancy() {
        use crate::dependency_logic::{DependencyGroupMemberAssessment, DependencyGroupState};

        let common = assess_common_cause_domains(
            &[domain()],
            &scenario(Some(condition(CommonCauseState::Active))),
            &[evidence(
                "ev:domain",
                "domain:shared-feeder",
                "feeder:grid-1",
            )],
        )
        .unwrap();
        let group = DependencyGroupAssessment {
            group_id: "group:duplicate-route".into(),
            dependent_entity_id: "pump:main".into(),
            logic: DependencyLogic::AnyOne,
            as_of: AT.into(),
            state: DependencyGroupState::Satisfied,
            required_count: 1,
            active_count: 2,
            available_count: 2,
            unavailable_count: 0,
            unknown_count: 0,
            members: vec![
                DependencyGroupMemberAssessment {
                    relation_id: "route:primary".into(),
                    supporting_evidence_ids: Vec::new(),
                    prerequisite_entity_id: "pump:a".into(),
                    state: DependencyMemberState::Available,
                    evidence_readiness: None,
                    evidence_blockers: Vec::new(),
                    unknown_reason: None,
                },
                DependencyGroupMemberAssessment {
                    relation_id: "route:alias".into(),
                    supporting_evidence_ids: Vec::new(),
                    prerequisite_entity_id: "pump:a".into(),
                    state: DependencyMemberState::Available,
                    evidence_readiness: None,
                    evidence_blockers: Vec::new(),
                    unknown_reason: None,
                },
            ],
        };
        let overlaps = correlate_common_cause_with_dependency_groups(&common, &[group]).unwrap();
        assert_eq!(overlaps.len(), 1);
        assert_eq!(overlaps[0].shared_relation_ids.len(), 2);
        assert_eq!(overlaps[0].shared_prerequisite_entity_ids, vec!["pump:a"]);
        assert!(!overlaps[0].independent_redundancy_warning);
    }

    #[test]
    fn blocked_common_cause_evidence_never_emits_exposure_warning() {
        use crate::dependency_logic::{DependencyGroupMemberAssessment, DependencyGroupState};

        let mut shared = domain();
        shared.member_entity_ids = vec!["feeder:a".into(), "feeder:b".into()];
        let mut support = evidence("ev:domain", "domain:shared-feeder", "feeder:grid-1");
        support.state = EvidenceState::Disputed;
        let assessments = assess_common_cause_domains(
            &[shared],
            &scenario(Some(condition(CommonCauseState::Active))),
            &[support],
        )
        .unwrap();
        let group = DependencyGroupAssessment {
            group_id: "group:redundant-pump".into(),
            dependent_entity_id: "pump:main".into(),
            logic: DependencyLogic::AnyOne,
            as_of: AT.into(),
            state: DependencyGroupState::Satisfied,
            required_count: 1,
            active_count: 2,
            available_count: 2,
            unavailable_count: 0,
            unknown_count: 0,
            members: vec![
                DependencyGroupMemberAssessment {
                    relation_id: "relation:a".into(),
                    supporting_evidence_ids: Vec::new(),
                    prerequisite_entity_id: "feeder:a".into(),
                    state: DependencyMemberState::Available,
                    evidence_readiness: None,
                    evidence_blockers: Vec::new(),
                    unknown_reason: None,
                },
                DependencyGroupMemberAssessment {
                    relation_id: "relation:b".into(),
                    supporting_evidence_ids: Vec::new(),
                    prerequisite_entity_id: "feeder:b".into(),
                    state: DependencyMemberState::Available,
                    evidence_readiness: None,
                    evidence_blockers: Vec::new(),
                    unknown_reason: None,
                },
            ],
        };
        let overlaps =
            correlate_common_cause_with_dependency_groups(&assessments, &[group]).unwrap();

        assert_eq!(overlaps.len(), 1);
        assert!(!overlaps[0].domain_evidence_admissible);
        assert!(overlaps[0].currently_exposed_relation_ids.is_empty());
        assert!(!overlaps[0].independent_redundancy_warning);
        assert_eq!(overlaps[0].shared_relation_ids.len(), 2);
    }

    #[test]
    fn active_shared_cause_exposes_all_declared_members_without_marking_them_failed() {
        let evidence = [evidence(
            "ev:domain",
            "domain:shared-feeder",
            "feeder:grid-1",
        )];
        let result = assess_common_cause_domains(
            &[domain()],
            &scenario(Some(condition(CommonCauseState::Active))),
            &evidence,
        )
        .unwrap();

        assert_eq!(result[0].entity_exposures.len(), 2);
        assert_eq!(result[0].entity_exposures[0].entity_id, "pump:a");
        assert_eq!(
            result[0].entity_exposures[0].exposure,
            CommonCauseExposure::Exposed
        );
        assert_eq!(
            result[0].entity_exposures[1].exposure,
            CommonCauseExposure::Exposed
        );
        // There is no operational-state field to mutate: exposure is not failure.
    }

    #[test]
    fn missing_condition_remains_indeterminate() {
        let evidence = [evidence(
            "ev:domain",
            "domain:shared-feeder",
            "feeder:grid-1",
        )];
        let result = assess_common_cause_domains(&[domain()], &scenario(None), &evidence).unwrap();
        assert_eq!(result[0].state, CommonCauseState::Unknown);
        assert!(
            result[0]
                .entity_exposures
                .iter()
                .all(|member| member.exposure == CommonCauseExposure::Indeterminate)
        );
    }

    #[test]
    fn disputed_domain_evidence_blocks_exposure_conclusions() {
        let mut support = evidence("ev:domain", "domain:shared-feeder", "feeder:grid-1");
        support.state = EvidenceState::Disputed;
        let result = assess_common_cause_domains(
            &[domain()],
            &scenario(Some(condition(CommonCauseState::Active))),
            &[support],
        )
        .unwrap();
        assert_eq!(result[0].domain_evidence_blockers.len(), 1);
        assert_eq!(
            result[0].domain_evidence_blockers[0].reason,
            DomainEvidenceBlockReason::DisputedEvidence
        );
        assert!(
            result[0]
                .entity_exposures
                .iter()
                .all(|member| member.exposure == CommonCauseExposure::EvidenceBlocked)
        );
    }

    #[test]
    fn inactive_common_cause_is_not_triggered_not_a_claim_of_independence() {
        let evidence = [evidence(
            "ev:domain",
            "domain:shared-feeder",
            "feeder:grid-1",
        )];
        let result = assess_common_cause_domains(
            &[domain()],
            &scenario(Some(condition(CommonCauseState::Inactive))),
            &evidence,
        )
        .unwrap();
        assert!(
            result[0]
                .entity_exposures
                .iter()
                .all(|member| member.exposure == CommonCauseExposure::NotTriggered)
        );
    }

    #[test]
    fn observed_condition_requires_valid_matching_observation() {
        let mut observed = condition(CommonCauseState::Active);
        observed.basis = CommonCauseBasis::Observed;
        observed.supporting_evidence_ids = vec!["ev:cause".into()];
        let records = [
            evidence("ev:domain", "domain:shared-feeder", "feeder:grid-1"),
            evidence("ev:cause", "feeder:grid-1", "feeder:grid-1"),
        ];
        let result =
            assess_common_cause_domains(&[domain()], &scenario(Some(observed)), &records).unwrap();
        assert_eq!(
            result[0].entity_exposures[0].exposure,
            CommonCauseExposure::Exposed
        );
        assert_eq!(result[0].condition_evidence_ids, vec!["ev:cause"]);
    }

    #[test]
    fn observed_condition_with_wrong_subject_is_rejected() {
        let mut observed = condition(CommonCauseState::Active);
        observed.basis = CommonCauseBasis::Observed;
        observed.supporting_evidence_ids = vec!["ev:cause".into()];
        let records = [
            evidence("ev:domain", "domain:shared-feeder", "feeder:grid-1"),
            evidence("ev:cause", "some-other-entity", "feeder:grid-1"),
        ];
        assert!(matches!(
            assess_common_cause_domains(&[domain()], &scenario(Some(observed)), &records),
            Err(CommonCauseAssessmentError::InvalidInput(errors))
                if errors.contains(&CommonCauseValidationError::ConditionEvidenceSubjectMismatch)
        ));
    }

    #[test]
    fn duplicate_members_and_unknown_domains_fail_closed() {
        let mut bad_domain = domain();
        bad_domain.member_entity_ids.push("pump:a".into());
        assert!(matches!(
            assess_common_cause_domains(
                &[bad_domain],
                &scenario(Some(condition(CommonCauseState::Active))),
                &[evidence("ev:domain", "domain:shared-feeder", "feeder:grid-1")],
            ),
            Err(CommonCauseAssessmentError::InvalidInput(errors))
                if errors.contains(&CommonCauseValidationError::DuplicateMemberEntityId)
        ));

        let mut bad_scenario = scenario(None);
        bad_scenario
            .conditions
            .insert("domain:unknown".into(), condition(CommonCauseState::Active));
        assert!(matches!(
            assess_common_cause_domains(
                &[domain()],
                &bad_scenario,
                &[evidence("ev:domain", "domain:shared-feeder", "feeder:grid-1")],
            ),
            Err(CommonCauseAssessmentError::InvalidInput(errors))
                if errors.contains(&CommonCauseValidationError::UnknownScenarioDomain)
        ));
    }

    #[test]
    fn expired_domain_support_blocks_exposure() {
        let mut support = evidence("ev:domain", "domain:shared-feeder", "feeder:grid-1");
        support.time.valid_until = Some("2026-10-15T09:00:00Z".into());
        let result = assess_common_cause_domains(
            &[domain()],
            &scenario(Some(condition(CommonCauseState::Active))),
            &[support],
        )
        .unwrap();
        assert!(result[0].domain_evidence_blockers.iter().any(|blocker| {
            blocker.reason == DomainEvidenceBlockReason::OutsideEvidenceValidity
        }));
        assert!(
            result[0]
                .entity_exposures
                .iter()
                .all(|member| member.exposure == CommonCauseExposure::EvidenceBlocked)
        );
    }

    #[test]
    fn expired_domain_is_outside_scope_without_relabeling_it_safe() {
        let mut expired = domain();
        expired.valid_until = Some("2026-10-15T09:00:00Z".into());
        let result = assess_common_cause_domains(
            &[expired],
            &scenario(Some(condition(CommonCauseState::Active))),
            &[evidence(
                "ev:domain",
                "domain:shared-feeder",
                "feeder:grid-1",
            )],
        )
        .unwrap();
        assert!(
            result[0]
                .entity_exposures
                .iter()
                .all(|member| member.exposure == CommonCauseExposure::OutsideDomainValidity)
        );
    }

    #[test]
    fn results_and_member_exposures_are_sorted_deterministically() {
        let mut first = domain();
        first.member_entity_ids = vec!["pump:z".into(), "pump:a".into()];
        let mut second = domain();
        second.id = "domain:alternate".into();
        second.supporting_evidence_ids = vec!["ev:alternate".into()];
        let evidence = [
            evidence("ev:domain", "domain:shared-feeder", "feeder:grid-1"),
            evidence("ev:alternate", "domain:alternate", "feeder:grid-2"),
        ];
        let result = assess_common_cause_domains(
            &[first, second],
            &scenario(Some(condition(CommonCauseState::Active))),
            &evidence,
        )
        .unwrap();
        assert_eq!(result[0].domain_id, "domain:alternate");
        assert_eq!(result[1].domain_id, "domain:shared-feeder");
        assert_eq!(result[1].entity_exposures[0].entity_id, "pump:a");
        assert_eq!(result[1].entity_exposures[1].entity_id, "pump:z");
        assert_eq!(result[0].state, CommonCauseState::Unknown);
    }
}
