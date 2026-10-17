// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Coverage-aware interpretation of spatial-constraint overlay receipts.
//!
//! Geometry operations are performed by a GIS adapter. This module validates the
//! operation's declared inputs/metadata and interprets its result conservatively:
//! confirmed intersection can establish presence, but non-intersection establishes
//! absence only when coverage and accuracy are adequate. It does not compute polygon
//! intersections, transform CRSs, or certify legal/environmental facts.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConstraintCriticality {
    Hard,
    Soft,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceEvidenceState {
    Verified,
    Estimated,
    Unknown,
    Disputed,
    Stale,
    Restricted,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourcePolicyDisposition {
    Accepted,
    NeedsEvidence,
    Rejected,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GeometryValidity {
    Valid,
    Invalid,
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoverageStatus {
    Complete,
    Partial,
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpatialRelation {
    Intersects,
    Disjoint,
    Indeterminate,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpatialAccuracy {
    Adequate,
    Insufficient,
    Unknown,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConstraintFinding {
    Present,
    Absent,
    NeedsEvidence,
    Invalid,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CandidateGateStatus {
    Eligible,
    Excluded,
    NeedsEvidence,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SpatialOverlayReceipt {
    pub candidate_id: String,
    pub candidate_geometry_sha256: String,
    pub candidate_crs: String,
    pub constraint_id: String,
    pub source_id: String,
    pub source_payload_sha256: String,
    pub layer_geometry_sha256: String,
    pub layer_crs: String,
    pub source_state: SourceEvidenceState,
    /// Result of the policy-specific source catalog check (rights, authority, freshness).
    pub source_policy: SourcePolicyDisposition,
    /// Output of the chosen geometry engine's validity check.
    pub geometry_validity: GeometryValidity,
    /// Whether the source demonstrably covers the entire candidate area.
    pub coverage: CoverageStatus,
    /// Output of the geometry adapter for candidate-vs-constraint relation.
    pub relation: SpatialRelation,
    /// Whether source scale/resolution/positional accuracy supports this inference.
    pub accuracy: SpatialAccuracy,
    pub overlay_engine_id: String,
    pub overlay_engine_version: String,
    pub computed_at_unix_seconds: i64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SpatialOverlayPolicy {
    pub evaluation_time_unix_seconds: i64,
    pub candidate_ids: BTreeSet<String>,
    /// Required constraints and whether each is a hard gate or soft consideration.
    pub required_constraints: BTreeMap<String, ConstraintCriticality>,
    pub accepted_source_states: BTreeSet<SourceEvidenceState>,
    pub maximum_overlay_age_seconds: Option<i64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ConstraintAssessment {
    pub criticality: ConstraintCriticality,
    pub finding: ConstraintFinding,
    pub reasons: Vec<String>,
    pub source_id: Option<String>,
    pub source_payload_sha256: Option<String>,
    pub candidate_geometry_sha256: Option<String>,
    pub layer_geometry_sha256: Option<String>,
    pub overlay_engine_id: Option<String>,
    pub overlay_engine_version: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CandidateSpatialAssessment {
    pub candidate_id: String,
    pub gate_status: CandidateGateStatus,
    pub constraints: BTreeMap<String, ConstraintAssessment>,
    pub reasons: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SpatialOverlayReport {
    pub candidates: BTreeMap<String, CandidateSpatialAssessment>,
    /// Stable count metrics for independent review and dashboards.
    pub eligible_count: usize,
    pub excluded_count: usize,
    pub needs_evidence_count: usize,
    pub confirmed_present_count: usize,
    pub confirmed_absent_count: usize,
    pub unresolved_constraint_count: usize,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SpatialOverlayError {
    EmptyCandidates,
    EmptyConstraints,
    EmptyAcceptedSourceStates,
    InvalidEvaluationTime(i64),
    InvalidMaximumAge(i64),
    EmptyCandidateId,
    EmptyConstraintId,
    InvalidAcceptedSourceState,
    NoHardConstraints,
    UnknownCandidate(String),
    UnknownConstraint(String),
    DuplicateCandidateConstraint {
        candidate_id: String,
        constraint_id: String,
    },
    CandidateGeometryMismatch(String),
    CandidateCrsMismatch(String),
    InvalidDigest {
        item_id: String,
        field: String,
    },
    EmptyMetadata {
        item_id: String,
        field: String,
    },
    InvalidOverlayTimestamp {
        item_id: String,
        timestamp: i64,
    },
}

impl fmt::Display for SpatialOverlayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyCandidates => write!(f, "policy must declare at least one candidate"),
            Self::EmptyConstraints => write!(f, "policy must declare at least one required constraint"),
            Self::EmptyAcceptedSourceStates => write!(f, "policy must accept at least one evidence state"),
            Self::InvalidEvaluationTime(value) => write!(f, "evaluation time must be non-negative, got {value}"),
            Self::InvalidMaximumAge(value) => write!(f, "maximum overlay age must be non-negative, got {value}"),
            Self::EmptyCandidateId => write!(f, "candidate IDs must not be empty"),
            Self::EmptyConstraintId => write!(f, "constraint IDs must not be empty"),
            Self::InvalidAcceptedSourceState => write!(f, "unknown, disputed, stale and restricted evidence states cannot be accepted for spatial gate inference"),
            Self::NoHardConstraints => write!(f, "policy must define at least one hard constraint for candidate eligibility"),
            Self::UnknownCandidate(id) => write!(f, "overlay receipt refers to undeclared candidate {id}"),
            Self::UnknownConstraint(id) => write!(f, "overlay receipt refers to undeclared constraint {id}"),
            Self::DuplicateCandidateConstraint { candidate_id, constraint_id } => write!(
                f, "duplicate overlay receipt for candidate {candidate_id}, constraint {constraint_id}"
            ),
            Self::CandidateGeometryMismatch(id) => write!(
                f, "candidate {id} has overlay receipts pinned to different geometry digests"
            ),
            Self::CandidateCrsMismatch(id) => write!(
                f, "candidate {id} has overlay receipts using different candidate CRSs"
            ),
            Self::InvalidDigest { item_id, field } => write!(f, "{item_id} has an invalid SHA-256 in {field}"),
            Self::EmptyMetadata { item_id, field } => write!(f, "{item_id} has empty required metadata in {field}"),
            Self::InvalidOverlayTimestamp { item_id, timestamp } => write!(
                f, "overlay receipt {item_id} has invalid/future computation time {timestamp}"
            ),
        }
    }
}
impl std::error::Error for SpatialOverlayError {}

fn nonempty(value: &str) -> bool {
    !value.trim().is_empty()
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn validate_policy(policy: &SpatialOverlayPolicy) -> Result<(), SpatialOverlayError> {
    if policy.candidate_ids.is_empty() {
        return Err(SpatialOverlayError::EmptyCandidates);
    }
    if policy.required_constraints.is_empty() {
        return Err(SpatialOverlayError::EmptyConstraints);
    }
    if policy.accepted_source_states.is_empty() {
        return Err(SpatialOverlayError::EmptyAcceptedSourceStates);
    }
    if policy.evaluation_time_unix_seconds < 0 {
        return Err(SpatialOverlayError::InvalidEvaluationTime(
            policy.evaluation_time_unix_seconds,
        ));
    }
    if let Some(age) = policy.maximum_overlay_age_seconds {
        if age < 0 {
            return Err(SpatialOverlayError::InvalidMaximumAge(age));
        }
    }
    if policy.candidate_ids.iter().any(|id| !nonempty(id)) {
        return Err(SpatialOverlayError::EmptyCandidateId);
    }
    if policy.required_constraints.keys().any(|id| !nonempty(id)) {
        return Err(SpatialOverlayError::EmptyConstraintId);
    }
    if !policy
        .required_constraints
        .values()
        .any(|criticality| *criticality == ConstraintCriticality::Hard)
    {
        return Err(SpatialOverlayError::NoHardConstraints);
    }
    if policy.accepted_source_states.iter().any(|state| {
        matches!(state, SourceEvidenceState::Unknown | SourceEvidenceState::Disputed | SourceEvidenceState::Stale | SourceEvidenceState::Restricted)
    }) {
        // These states should never be accepted as a basis for asserting presence/absence.
        return Err(SpatialOverlayError::InvalidAcceptedSourceState);
    }
    Ok(())
}

fn validate_receipt(
    receipt: &SpatialOverlayReceipt,
    policy: &SpatialOverlayPolicy,
) -> Result<(), SpatialOverlayError> {
    if !policy.candidate_ids.contains(&receipt.candidate_id) {
        return Err(SpatialOverlayError::UnknownCandidate(receipt.candidate_id.clone()));
    }
    if !policy.required_constraints.contains_key(&receipt.constraint_id) {
        return Err(SpatialOverlayError::UnknownConstraint(receipt.constraint_id.clone()));
    }
    for (field, value) in [
        ("candidate_id", receipt.candidate_id.as_str()),
        ("candidate_crs", receipt.candidate_crs.as_str()),
        ("constraint_id", receipt.constraint_id.as_str()),
        ("source_id", receipt.source_id.as_str()),
        ("layer_crs", receipt.layer_crs.as_str()),
        ("overlay_engine_id", receipt.overlay_engine_id.as_str()),
        ("overlay_engine_version", receipt.overlay_engine_version.as_str()),
    ] {
        if !nonempty(value) {
            return Err(SpatialOverlayError::EmptyMetadata {
                item_id: receipt.candidate_id.clone(),
                field: field.into(),
            });
        }
    }
    for (field, digest) in [
        ("candidate_geometry_sha256", receipt.candidate_geometry_sha256.as_str()),
        ("source_payload_sha256", receipt.source_payload_sha256.as_str()),
        ("layer_geometry_sha256", receipt.layer_geometry_sha256.as_str()),
    ] {
        if !valid_sha256(digest) {
            return Err(SpatialOverlayError::InvalidDigest {
                item_id: receipt.candidate_id.clone(),
                field: field.into(),
            });
        }
    }
    if receipt.computed_at_unix_seconds < 0
        || receipt.computed_at_unix_seconds > policy.evaluation_time_unix_seconds
    {
        return Err(SpatialOverlayError::InvalidOverlayTimestamp {
            item_id: format!("{}:{}", receipt.candidate_id, receipt.constraint_id),
            timestamp: receipt.computed_at_unix_seconds,
        });
    }
    Ok(())
}

fn assess_receipt(
    receipt: Option<&SpatialOverlayReceipt>,
    criticality: ConstraintCriticality,
    policy: &SpatialOverlayPolicy,
) -> ConstraintAssessment {
    let Some(receipt) = receipt else {
        return ConstraintAssessment {
            criticality,
            finding: ConstraintFinding::NeedsEvidence,
            reasons: vec!["required spatial overlay receipt is missing".into()],
            source_id: None,
            source_payload_sha256: None,
            candidate_geometry_sha256: None,
            layer_geometry_sha256: None,
            overlay_engine_id: None,
            overlay_engine_version: None,
        };
    };

    let mut reasons = Vec::new();
    let mut finding = ConstraintFinding::NeedsEvidence;
    let age = policy
        .evaluation_time_unix_seconds
        .saturating_sub(receipt.computed_at_unix_seconds);
    let current_enough = policy
        .maximum_overlay_age_seconds
        .is_none_or(|maximum| age <= maximum);

    if receipt.geometry_validity == GeometryValidity::Invalid {
        finding = ConstraintFinding::Invalid;
        reasons.push("candidate or constraint geometry is invalid".into());
    } else if receipt.geometry_validity == GeometryValidity::Unknown {
        reasons.push("geometry validity has not been established".into());
    } else if receipt.candidate_crs != receipt.layer_crs {
        finding = ConstraintFinding::Invalid;
        reasons.push("candidate and constraint CRSs do not match; no implicit transformation is applied".into());
    } else if receipt.source_policy != SourcePolicyDisposition::Accepted {
        reasons.push(match receipt.source_policy {
            SourcePolicyDisposition::NeedsEvidence => "source policy has not accepted this layer".into(),
            SourcePolicyDisposition::Rejected => "source was rejected by the source policy and cannot establish presence or absence".into(),
            SourcePolicyDisposition::Accepted => unreachable!(),
        });
    } else if !policy.accepted_source_states.contains(&receipt.source_state) {
        reasons.push(format!("source evidence state {:?} is not accepted by this overlay policy", receipt.source_state));
    } else if !current_enough {
        reasons.push(format!("overlay age {age} seconds exceeds the policy maximum"));
    } else if receipt.accuracy != SpatialAccuracy::Adequate {
        reasons.push(match receipt.accuracy {
            SpatialAccuracy::Insufficient => "source scale/positional accuracy is insufficient for this inference".into(),
            SpatialAccuracy::Unknown => "source accuracy is unknown".into(),
            SpatialAccuracy::Adequate => unreachable!(),
        });
    } else {
        match receipt.relation {
            SpatialRelation::Intersects => {
                finding = ConstraintFinding::Present;
                reasons.push("GIS adapter reports an intersection under valid, aligned, policy-accepted inputs".into());
            }
            SpatialRelation::Disjoint if receipt.coverage == CoverageStatus::Complete => {
                finding = ConstraintFinding::Absent;
                reasons.push("GIS adapter reports disjoint geometry and complete candidate-area coverage".into());
            }
            SpatialRelation::Disjoint => {
                reasons.push(match receipt.coverage {
                    CoverageStatus::Partial => "non-intersection cannot establish absence because coverage is partial".into(),
                    CoverageStatus::Unknown => "non-intersection cannot establish absence because coverage is unknown".into(),
                    CoverageStatus::Complete => unreachable!(),
                });
            }
            SpatialRelation::Indeterminate => {
                reasons.push("GIS adapter could not establish intersection or disjointness".into());
            }
        }
    }

    ConstraintAssessment {
        criticality,
        finding,
        reasons,
        source_id: Some(receipt.source_id.clone()),
        source_payload_sha256: Some(receipt.source_payload_sha256.clone()),
        candidate_geometry_sha256: Some(receipt.candidate_geometry_sha256.clone()),
        layer_geometry_sha256: Some(receipt.layer_geometry_sha256.clone()),
        overlay_engine_id: Some(receipt.overlay_engine_id.clone()),
        overlay_engine_version: Some(receipt.overlay_engine_version.clone()),
    }
}

/// Interpret precomputed overlay receipts and produce conservative candidate gates.
///
/// This function does not perform GIS. The adapter is responsible for geometry
/// operations and must supply the exact input digests, CRS, coverage and accuracy
/// metadata. A candidate is excluded when any verified hard constraint is present;
/// it is eligible only when every required hard constraint is confirmed absent.
pub fn analyze_spatial_overlays(
    receipts: &[SpatialOverlayReceipt],
    policy: &SpatialOverlayPolicy,
) -> Result<SpatialOverlayReport, SpatialOverlayError> {
    validate_policy(policy)?;

    let mut seen = BTreeSet::new();
    let mut geometry_by_candidate: BTreeMap<String, (String, String)> = BTreeMap::new();
    let mut receipt_by_pair: BTreeMap<(String, String), &SpatialOverlayReceipt> = BTreeMap::new();

    for receipt in receipts {
        validate_receipt(receipt, policy)?;
        let pair = (receipt.candidate_id.clone(), receipt.constraint_id.clone());
        if !seen.insert(pair.clone()) {
            return Err(SpatialOverlayError::DuplicateCandidateConstraint {
                candidate_id: pair.0,
                constraint_id: pair.1,
            });
        }
        match geometry_by_candidate.get(&receipt.candidate_id) {
            None => {
                geometry_by_candidate.insert(
                    receipt.candidate_id.clone(),
                    (receipt.candidate_geometry_sha256.clone(), receipt.candidate_crs.clone()),
                );
            }
            Some((digest, crs)) if digest != &receipt.candidate_geometry_sha256 => {
                return Err(SpatialOverlayError::CandidateGeometryMismatch(receipt.candidate_id.clone()));
            }
            Some((_, crs)) if crs != &receipt.candidate_crs => {
                return Err(SpatialOverlayError::CandidateCrsMismatch(receipt.candidate_id.clone()));
            }
            Some(_) => {}
        }
        receipt_by_pair.insert(pair, receipt);
    }

    let mut candidates = BTreeMap::new();
    let mut eligible_count = 0;
    let mut excluded_count = 0;
    let mut needs_evidence_count = 0;
    let mut confirmed_present_count = 0;
    let mut confirmed_absent_count = 0;
    let mut unresolved_constraint_count = 0;

    for candidate_id in &policy.candidate_ids {
        let mut constraints = BTreeMap::new();
        let mut reasons = Vec::new();
        let mut hard_present = false;
        let mut hard_unresolved = false;

        for (constraint_id, criticality) in &policy.required_constraints {
            let receipt = receipt_by_pair
                .get(&(candidate_id.clone(), constraint_id.clone()))
                .copied();
            let assessment = assess_receipt(receipt, *criticality, policy);
            match assessment.finding {
                ConstraintFinding::Present => confirmed_present_count += 1,
                ConstraintFinding::Absent => confirmed_absent_count += 1,
                ConstraintFinding::NeedsEvidence | ConstraintFinding::Invalid => {
                    unresolved_constraint_count += 1;
                }
            }
            if *criticality == ConstraintCriticality::Hard {
                match assessment.finding {
                    ConstraintFinding::Present => hard_present = true,
                    ConstraintFinding::Absent => {}
                    ConstraintFinding::NeedsEvidence | ConstraintFinding::Invalid => {
                        hard_unresolved = true;
                    }
                }
            }
            if assessment.finding != ConstraintFinding::Absent {
                reasons.push(format!(
                    "constraint {constraint_id} ({criticality:?}) is {:?}: {}",
                    assessment.finding,
                    assessment.reasons.join("; ")
                ));
            }
            constraints.insert(constraint_id.clone(), assessment);
        }

        let gate_status = if hard_present {
            CandidateGateStatus::Excluded
        } else if hard_unresolved {
            CandidateGateStatus::NeedsEvidence
        } else {
            CandidateGateStatus::Eligible
        };
        match gate_status {
            CandidateGateStatus::Eligible => eligible_count += 1,
            CandidateGateStatus::Excluded => excluded_count += 1,
            CandidateGateStatus::NeedsEvidence => needs_evidence_count += 1,
        }
        reasons.sort();

        candidates.insert(
            candidate_id.clone(),
            CandidateSpatialAssessment {
                candidate_id: candidate_id.clone(),
                gate_status,
                constraints,
                reasons,
            },
        );
    }

    Ok(SpatialOverlayReport {
        candidates,
        eligible_count,
        excluded_count,
        needs_evidence_count,
        confirmed_present_count,
        confirmed_absent_count,
        unresolved_constraint_count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    const EVALUATION_TIME: i64 = 1_792_195_200;

    fn policy() -> SpatialOverlayPolicy {
        SpatialOverlayPolicy {
            evaluation_time_unix_seconds: EVALUATION_TIME,
            candidate_ids: ["site-A".into()].into_iter().collect(),
            required_constraints: [("protected-area".into(), ConstraintCriticality::Hard)]
                .into_iter()
                .collect(),
            accepted_source_states: [SourceEvidenceState::Verified].into_iter().collect(),
            maximum_overlay_age_seconds: Some(86_400),
        }
    }

    fn receipt(relation: SpatialRelation, coverage: CoverageStatus) -> SpatialOverlayReceipt {
        SpatialOverlayReceipt {
            candidate_id: "site-A".into(),
            candidate_geometry_sha256: "a".repeat(64),
            candidate_crs: "EPSG:4326".into(),
            constraint_id: "protected-area".into(),
            source_id: "synthetic-authority".into(),
            source_payload_sha256: "b".repeat(64),
            layer_geometry_sha256: "c".repeat(64),
            layer_crs: "EPSG:4326".into(),
            source_state: SourceEvidenceState::Verified,
            source_policy: SourcePolicyDisposition::Accepted,
            geometry_validity: GeometryValidity::Valid,
            coverage,
            relation,
            accuracy: SpatialAccuracy::Adequate,
            overlay_engine_id: "synthetic-overlay-engine".into(),
            overlay_engine_version: "1.0.0".into(),
            computed_at_unix_seconds: EVALUATION_TIME,
        }
    }

    #[test]
    fn confirmed_intersection_excludes_hard_constraint_candidate() {
        let result = analyze_spatial_overlays(
            &[receipt(SpatialRelation::Intersects, CoverageStatus::Partial)],
            &policy(),
        )
        .unwrap();
        assert_eq!(result.candidates["site-A"].gate_status, CandidateGateStatus::Excluded);
        assert_eq!(
            result.candidates["site-A"].constraints["protected-area"].finding,
            ConstraintFinding::Present
        );
    }

    #[test]
    fn disjoint_complete_coverage_confirms_absence() {
        let result = analyze_spatial_overlays(
            &[receipt(SpatialRelation::Disjoint, CoverageStatus::Complete)],
            &policy(),
        )
        .unwrap();
        assert_eq!(result.candidates["site-A"].gate_status, CandidateGateStatus::Eligible);
        assert_eq!(
            result.candidates["site-A"].constraints["protected-area"].finding,
            ConstraintFinding::Absent
        );
    }

    #[test]
    fn partial_coverage_never_turns_disjointness_into_absence() {
        let result = analyze_spatial_overlays(
            &[receipt(SpatialRelation::Disjoint, CoverageStatus::Partial)],
            &policy(),
        )
        .unwrap();
        assert_eq!(result.candidates["site-A"].gate_status, CandidateGateStatus::NeedsEvidence);
        assert_eq!(
            result.candidates["site-A"].constraints["protected-area"].finding,
            ConstraintFinding::NeedsEvidence
        );
    }

    #[test]
    fn unknown_coverage_never_turns_disjointness_into_absence() {
        let result = analyze_spatial_overlays(
            &[receipt(SpatialRelation::Disjoint, CoverageStatus::Unknown)],
            &policy(),
        )
        .unwrap();
        assert_eq!(result.candidates["site-A"].gate_status, CandidateGateStatus::NeedsEvidence);
    }

    #[test]
    fn missing_required_overlay_is_not_vacuous_eligibility() {
        let result = analyze_spatial_overlays(&[], &policy()).unwrap();
        assert_eq!(result.candidates["site-A"].gate_status, CandidateGateStatus::NeedsEvidence);
        assert_eq!(
            result.candidates["site-A"].constraints["protected-area"].finding,
            ConstraintFinding::NeedsEvidence
        );
    }

    #[test]
    fn crs_mismatch_is_invalid_not_absent() {
        let mut item = receipt(SpatialRelation::Disjoint, CoverageStatus::Complete);
        item.layer_crs = "EPSG:3857".into();
        let result = analyze_spatial_overlays(&[item], &policy()).unwrap();
        assert_eq!(
            result.candidates["site-A"].constraints["protected-area"].finding,
            ConstraintFinding::Invalid
        );
        assert_eq!(result.candidates["site-A"].gate_status, CandidateGateStatus::NeedsEvidence);
    }

    #[test]
    fn invalid_geometry_is_not_absence() {
        let mut item = receipt(SpatialRelation::Disjoint, CoverageStatus::Complete);
        item.geometry_validity = GeometryValidity::Invalid;
        let result = analyze_spatial_overlays(&[item], &policy()).unwrap();
        assert_eq!(result.candidates["site-A"].gate_status, CandidateGateStatus::NeedsEvidence);
        assert_eq!(
            result.candidates["site-A"].constraints["protected-area"].finding,
            ConstraintFinding::Invalid
        );
    }

    #[test]
    fn stale_or_rejected_source_cannot_establish_absence() {
        let mut item = receipt(SpatialRelation::Disjoint, CoverageStatus::Complete);
        item.source_state = SourceEvidenceState::Stale;
        let result = analyze_spatial_overlays(&[item.clone()], &policy()).unwrap();
        assert_eq!(result.candidates["site-A"].gate_status, CandidateGateStatus::NeedsEvidence);

        item.source_state = SourceEvidenceState::Verified;
        item.source_policy = SourcePolicyDisposition::Rejected;
        let result = analyze_spatial_overlays(&[item], &policy()).unwrap();
        assert_eq!(result.candidates["site-A"].gate_status, CandidateGateStatus::NeedsEvidence);
    }

    #[test]
    fn soft_constraint_does_not_override_a_clear_hard_gate() {
        let mut p = policy();
        p.candidate_ids.insert("site-B".into());
        p.required_constraints.insert("solar-potential".into(), ConstraintCriticality::Soft);
        let mut a = receipt(SpatialRelation::Disjoint, CoverageStatus::Complete);
        let mut b = a.clone();
        b.candidate_id = "site-B".into();
        let mut soft = a.clone();
        soft.constraint_id = "solar-potential".into();
        soft.relation = SpatialRelation::Intersects;
        let mut soft_b = soft.clone();
        soft_b.candidate_id = "site-B".into();
        a.constraint_id = "protected-area".into();
        b.constraint_id = "protected-area".into();
        let result = analyze_spatial_overlays(&[a, b, soft, soft_b], &p).unwrap();
        assert_eq!(result.candidates["site-A"].gate_status, CandidateGateStatus::Eligible);
        assert_eq!(result.candidates["site-B"].gate_status, CandidateGateStatus::Eligible);
        assert_eq!(
            result.candidates["site-A"].constraints["solar-potential"].finding,
            ConstraintFinding::Present
        );
    }

    #[test]
    fn candidate_geometry_digest_must_be_stable_across_layers() {
        let first = receipt(SpatialRelation::Disjoint, CoverageStatus::Complete);
        let mut second = first.clone();
        second.constraint_id = "flood-zone".into();
        second.candidate_geometry_sha256 = "d".repeat(64);
        let mut p = policy();
        p.required_constraints.insert("flood-zone".into(), ConstraintCriticality::Hard);
        assert!(matches!(
            analyze_spatial_overlays(&[first, second], &p),
            Err(SpatialOverlayError::CandidateGeometryMismatch(_))
        ));
    }

    #[test]
    fn policy_with_only_soft_constraints_cannot_claim_candidate_eligibility() {
        let mut p = policy();
        p.required_constraints = [("solar-potential".into(), ConstraintCriticality::Soft)]
            .into_iter()
            .collect();
        assert!(matches!(
            analyze_spatial_overlays(&[], &p),
            Err(SpatialOverlayError::NoHardConstraints)
        ));
    }

    #[test]
    fn unsafe_evidence_states_cannot_be_added_to_accepted_state_policy() {
        let mut p = policy();
        p.accepted_source_states.insert(SourceEvidenceState::Stale);
        assert!(matches!(
            analyze_spatial_overlays(&[], &p),
            Err(SpatialOverlayError::InvalidAcceptedSourceState)
        ));
    }

    #[test]
    fn duplicate_overlay_receipts_are_rejected() {
        let item = receipt(SpatialRelation::Disjoint, CoverageStatus::Complete);
        assert!(matches!(
            analyze_spatial_overlays(&[item.clone(), item], &policy()),
            Err(SpatialOverlayError::DuplicateCandidateConstraint { .. })
        ));
    }

    #[test]
    fn stale_overlay_result_is_held_for_evidence() {
        let mut item = receipt(SpatialRelation::Disjoint, CoverageStatus::Complete);
        item.computed_at_unix_seconds -= 90 * 86_400;
        let result = analyze_spatial_overlays(&[item], &policy()).unwrap();
        assert_eq!(result.candidates["site-A"].gate_status, CandidateGateStatus::NeedsEvidence);
    }

    #[derive(Deserialize)]
    struct BenchmarkCase {
        case_id: String,
        policy: SpatialOverlayPolicy,
        receipts: Vec<SpatialOverlayReceipt>,
        expected_gate: BTreeMap<String, CandidateGateStatus>,
        expected_findings: BTreeMap<String, ConstraintFinding>,
    }

    #[test]
    fn synthetic_spatial_overlay_benchmark_matches_expected_results() {
        let fixture =
            include_str!("../../docs/research/fixtures/spatial-overlay-benchmark-v0.1.jsonl");
        let mut count = 0;
        for line in fixture.lines().filter(|line| !line.trim().is_empty()) {
            let case: BenchmarkCase = serde_json::from_str(line).expect("valid spatial overlay JSONL");
            let result = analyze_spatial_overlays(&case.receipts, &case.policy)
                .unwrap_or_else(|error| panic!("{}: {error}", case.case_id));
            let gates: BTreeMap<String, CandidateGateStatus> = result
                .candidates
                .iter()
                .map(|(id, assessment)| (id.clone(), assessment.gate_status))
                .collect();
            let findings: BTreeMap<String, ConstraintFinding> = result
                .candidates
                .iter()
                .flat_map(|(candidate_id, assessment)| {
                    assessment.constraints.iter().map(move |(constraint_id, constraint)| {
                        (format!("{candidate_id}:{constraint_id}"), constraint.finding)
                    })
                })
                .collect();
            assert_eq!(gates, case.expected_gate, "{}", case.case_id);
            assert_eq!(findings, case.expected_findings, "{}", case.case_id);
            count += 1;
        }
        assert!(count >= 12, "fixture should include at least 12 negative/positive controls");
    }
}
