// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Audit whether an evidence record actually corroborates or tests a particular
//! heritage candidate. Merely recording that a project acquired geophysics does
//! not count as independent validation of every aerial anomaly in that project.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fmt;

pub const EVIDENCE_AUDIT_SCHEMA_VERSION: &str =
    "luminous.heritage-evidence-audit.v0.1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceModality {
    FieldSurvey,
    Magnetometry,
    GroundPenetratingRadar,
    ElectricalResistivity,
    Excavation,
    ArchivalResearch,
    CommunityCustodialKnowledge,
    AirborneLidar,
    UavOptical,
    SatelliteOptical,
    ThermalInfrared,
    SyntheticSimulation,
    Other,
}

impl EvidenceModality {
    /// Methods that measure or directly inspect the ground/archaeological
    /// feature independently of the remote-sensing proxy being evaluated.
    pub fn is_ground_verification(self) -> bool {
        matches!(
            self,
            Self::FieldSurvey
                | Self::Magnetometry
                | Self::GroundPenetratingRadar
                | Self::ElectricalResistivity
                | Self::Excavation
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceRelationship {
    /// The measurement/interview/record actually covers the same candidate
    /// geometry or feature instance, as confirmed by the assessor.
    SameCandidate,
    NearbyContext,
    RegionalContext,
    Unrelated,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceOutcome {
    Supports,
    DoesNotSupport,
    Inconclusive,
    NotTested,
    Unavailable,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceIndependence {
    /// A distinct measurement/source and a distinct method from the
    /// remote-sensing input. Ground-verification eligibility is still checked
    /// separately using the modality and candidate relationship.
    IndependentMeasurement,
    /// A different qualified interpretation of the same underlying source.
    /// Useful review evidence, but not independent physical corroboration.
    IndependentInterpretationSameSource,
    /// The evidence reuses the same source or method without an independent
    /// measurement/interpretation claim.
    SharedSource,
    /// No corroboration activity was performed or available.
    NotAssessed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceRightsState {
    Verified,
    Pending,
    Denied,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HeritageEvidenceRecord {
    pub evidence_id: String,
    pub candidate_id: String,
    /// Asset digest used by the remote-sensing detector.
    pub detection_source_sha256: String,
    pub detection_method_id: String,
    pub corroboration_source_sha256: Option<String>,
    pub corroboration_method_id: Option<String>,
    pub modality: Option<EvidenceModality>,
    pub relationship: EvidenceRelationship,
    pub independence: EvidenceIndependence,
    pub outcome: EvidenceOutcome,
    /// Fraction of the candidate geometry actually covered by the ground
    /// measurement/inspection, in [0,1]. Not the project's total surveyed area.
    pub candidate_coverage_fraction: Option<f64>,
    /// Required to interpret a negative result: target size/depth/sensitivity
    /// assumptions for the method and this survey.
    pub detectability_assessment_ref: Option<String>,
    pub evidence_assessment_ref: Option<String>,
    pub reviewer_ref: Option<String>,
    pub authority_ref: Option<String>,
    pub rights_state: EvidenceRightsState,
    pub rights_receipt_ref: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct EvidenceAuditConfig {
    /// Minimum candidate-area coverage for a same-candidate ground result to
    /// count as supporting or eligible negative evidence.
    pub minimum_candidate_coverage_fraction: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvidenceRecordAssessment {
    pub evidence_id: String,
    pub candidate_id: String,
    pub outcome: EvidenceOutcome,
    pub modality: Option<EvidenceModality>,
    pub relationship: EvidenceRelationship,
    pub independence: EvidenceIndependence,
    pub rights_state: EvidenceRightsState,
    pub same_candidate_ground_measurement: bool,
    pub eligible_independent_corroboration: bool,
    pub eligible_negative_control: bool,
    pub adequate_candidate_coverage: bool,
    pub explanatory_status: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HeritageEvidenceAuditReport {
    pub schema_version: String,
    pub candidate_count: usize,
    pub evidence_record_count: usize,
    pub candidates_with_any_evidence: usize,
    pub same_candidate_ground_measurement_count: usize,
    pub independently_corroborated_candidate_count: usize,
    pub independent_corroboration_record_count: usize,
    pub eligible_negative_control_candidate_count: usize,
    pub eligible_negative_control_record_count: usize,
    pub independent_interpretation_same_source_count: usize,
    pub context_only_record_count: usize,
    pub untested_record_count: usize,
    pub unavailable_record_count: usize,
    pub inconclusive_record_count: usize,
    pub rights_not_verified_record_count: usize,
    pub minimum_candidate_coverage_fraction: f64,
    pub assessments: Vec<EvidenceRecordAssessment>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvidenceAuditError {
    InvalidConfig,
    EmptyCandidateId,
    DuplicateCandidateId(String),
    EmptyEvidenceId,
    DuplicateEvidenceId(String),
    EvidenceCandidateNotInManifest { evidence_id: String, candidate_id: String },
    InvalidDetectionSourceDigest { evidence_id: String },
    InvalidDetectionMethodId { evidence_id: String },
    EmptyOptionalReference { evidence_id: String, field: &'static str },
    MissingCorroborationSource { evidence_id: String },
    MissingCorroborationMethod { evidence_id: String },
    MissingModality { evidence_id: String },
    MissingAssessmentReference { evidence_id: String },
    MissingReviewerReference { evidence_id: String },
    MissingAuthorityReference { evidence_id: String },
    MissingRightsReceipt { evidence_id: String },
    MissingCandidateCoverage { evidence_id: String },
    InvalidCandidateCoverage { evidence_id: String },
    MissingDetectabilityAssessment { evidence_id: String },
    IndependentMeasurementReusesDetectionSource { evidence_id: String },
    IndependentMeasurementReusesDetectionMethod { evidence_id: String },
    IndependentInterpretationUsesDifferentSource { evidence_id: String },
    IndependentInterpretationReusesDetectionMethod { evidence_id: String },
    SharedSourceUsesDifferentDigest { evidence_id: String },
    NotAssessedHasCorroborationData { evidence_id: String },
    CorroborationOutcomeMissingData { evidence_id: String },
    InvalidNotTestedRecord { evidence_id: String },
}

impl fmt::Display for EvidenceAuditError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "heritage evidence audit error: {self:?}")
    }
}

impl std::error::Error for EvidenceAuditError {}

fn is_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn validate_optional_reference(
    value: &Option<String>,
    evidence_id: &str,
    field: &'static str,
) -> Result<(), EvidenceAuditError> {
    if value.as_ref().is_some_and(|value| value.trim().is_empty()) {
        return Err(EvidenceAuditError::EmptyOptionalReference {
            evidence_id: evidence_id.to_owned(),
            field,
        });
    }
    Ok(())
}

fn validate_record(record: &HeritageEvidenceRecord) -> Result<(), EvidenceAuditError> {
    if record.evidence_id.trim().is_empty() {
        return Err(EvidenceAuditError::EmptyEvidenceId);
    }
    if record.candidate_id.trim().is_empty() {
        return Err(EvidenceAuditError::EmptyCandidateId);
    }
    if !is_sha256(&record.detection_source_sha256) {
        return Err(EvidenceAuditError::InvalidDetectionSourceDigest {
            evidence_id: record.evidence_id.clone(),
        });
    }
    if record.detection_method_id.trim().is_empty() {
        return Err(EvidenceAuditError::InvalidDetectionMethodId {
            evidence_id: record.evidence_id.clone(),
        });
    }

    for (value, field) in [
        (&record.corroboration_source_sha256, "corroboration_source_sha256"),
        (&record.corroboration_method_id, "corroboration_method_id"),
        (&record.detectability_assessment_ref, "detectability_assessment_ref"),
        (&record.evidence_assessment_ref, "evidence_assessment_ref"),
        (&record.reviewer_ref, "reviewer_ref"),
        (&record.authority_ref, "authority_ref"),
        (&record.rights_receipt_ref, "rights_receipt_ref"),
    ] {
        validate_optional_reference(value, &record.evidence_id, field)?;
    }

    match record.outcome {
        EvidenceOutcome::NotTested | EvidenceOutcome::Unavailable => {
            if record.independence != EvidenceIndependence::NotAssessed
                || record.corroboration_source_sha256.is_some()
                || record.corroboration_method_id.is_some()
                || record.modality.is_some()
                || record.candidate_coverage_fraction.is_some()
                || record.detectability_assessment_ref.is_some()
                || record.evidence_assessment_ref.is_some()
                || record.reviewer_ref.is_some()
                || record.authority_ref.is_some()
            {
                return Err(EvidenceAuditError::InvalidNotTestedRecord {
                    evidence_id: record.evidence_id.clone(),
                });
            }
        }
        EvidenceOutcome::Supports
        | EvidenceOutcome::DoesNotSupport
        | EvidenceOutcome::Inconclusive => {
            if record.corroboration_source_sha256.is_none() {
                return Err(EvidenceAuditError::MissingCorroborationSource {
                    evidence_id: record.evidence_id.clone(),
                });
            }
            if !record
                .corroboration_source_sha256
                .as_deref()
                .is_some_and(is_sha256)
            {
                return Err(EvidenceAuditError::MissingCorroborationSource {
                    evidence_id: record.evidence_id.clone(),
                });
            }
            if record.corroboration_method_id.is_none() {
                return Err(EvidenceAuditError::MissingCorroborationMethod {
                    evidence_id: record.evidence_id.clone(),
                });
            }
            if record.modality.is_none() {
                return Err(EvidenceAuditError::MissingModality {
                    evidence_id: record.evidence_id.clone(),
                });
            }
            if record.evidence_assessment_ref.is_none() {
                return Err(EvidenceAuditError::MissingAssessmentReference {
                    evidence_id: record.evidence_id.clone(),
                });
            }
            if record.reviewer_ref.is_none() {
                return Err(EvidenceAuditError::MissingReviewerReference {
                    evidence_id: record.evidence_id.clone(),
                });
            }
            if record.authority_ref.is_none() {
                return Err(EvidenceAuditError::MissingAuthorityReference {
                    evidence_id: record.evidence_id.clone(),
                });
            }
            if record.rights_state == EvidenceRightsState::Verified
                && record.rights_receipt_ref.is_none()
            {
                return Err(EvidenceAuditError::MissingRightsReceipt {
                    evidence_id: record.evidence_id.clone(),
                });
            }
            if let Some(coverage) = record.candidate_coverage_fraction {
                if !coverage.is_finite() || !(0.0..=1.0).contains(&coverage) {
                    return Err(EvidenceAuditError::InvalidCandidateCoverage {
                        evidence_id: record.evidence_id.clone(),
                    });
                }
            }
        }
    }

    match record.independence {
        EvidenceIndependence::IndependentMeasurement => {
            if record.outcome == EvidenceOutcome::NotTested
                || record.outcome == EvidenceOutcome::Unavailable
            {
                return Err(EvidenceAuditError::NotAssessedHasCorroborationData {
                    evidence_id: record.evidence_id.clone(),
                });
            }
            if record.corroboration_source_sha256.as_deref()
                == Some(record.detection_source_sha256.as_str())
            {
                return Err(EvidenceAuditError::IndependentMeasurementReusesDetectionSource {
                    evidence_id: record.evidence_id.clone(),
                });
            }
            if record.corroboration_method_id.as_deref()
                == Some(record.detection_method_id.as_str())
            {
                return Err(EvidenceAuditError::IndependentMeasurementReusesDetectionMethod {
                    evidence_id: record.evidence_id.clone(),
                });
            }
        }
        EvidenceIndependence::IndependentInterpretationSameSource => {
            if record.corroboration_source_sha256.as_deref()
                != Some(record.detection_source_sha256.as_str())
            {
                return Err(EvidenceAuditError::IndependentInterpretationUsesDifferentSource {
                    evidence_id: record.evidence_id.clone(),
                });
            }
            if record.corroboration_method_id.as_deref()
                == Some(record.detection_method_id.as_str())
            {
                return Err(EvidenceAuditError::IndependentInterpretationReusesDetectionMethod {
                    evidence_id: record.evidence_id.clone(),
                });
            }
        }
        EvidenceIndependence::SharedSource => {
            if record.corroboration_source_sha256.as_deref()
                != Some(record.detection_source_sha256.as_str())
            {
                return Err(EvidenceAuditError::SharedSourceUsesDifferentDigest {
                    evidence_id: record.evidence_id.clone(),
                });
            }
        }
        EvidenceIndependence::NotAssessed => {
            if !matches!(
                record.outcome,
                EvidenceOutcome::NotTested | EvidenceOutcome::Unavailable
            ) {
                return Err(EvidenceAuditError::NotAssessedHasCorroborationData {
                    evidence_id: record.evidence_id.clone(),
                });
            }
        }
    }

    Ok(())
}

/// Audit evidence records against an explicit candidate manifest.
///
/// This does not establish the truth of a reviewer or custodial declaration.
/// It only counts independent corroboration when a distinct ground-measurement
/// source and method were actually applied to the same candidate, coverage is
/// adequate, rights are marked verified, and the evidence outcome is explicit.
///
/// A negative/does-not-support result is eligible as a benchmark negative
/// control only when a detectability assessment is also provided. A method
/// that could not detect the target at its expected size/depth cannot create a
/// valid negative example.
pub fn audit_heritage_evidence(
    candidate_ids: &[String],
    records: &[HeritageEvidenceRecord],
    config: EvidenceAuditConfig,
) -> Result<HeritageEvidenceAuditReport, EvidenceAuditError> {
    if !config.minimum_candidate_coverage_fraction.is_finite()
        || !(0.0..=1.0).contains(&config.minimum_candidate_coverage_fraction)
        || config.minimum_candidate_coverage_fraction == 0.0
    {
        return Err(EvidenceAuditError::InvalidConfig);
    }

    let mut candidates = BTreeSet::new();
    for candidate_id in candidate_ids {
        if candidate_id.trim().is_empty() {
            return Err(EvidenceAuditError::EmptyCandidateId);
        }
        if !candidates.insert(candidate_id.as_str()) {
            return Err(EvidenceAuditError::DuplicateCandidateId(candidate_id.clone()));
        }
    }

    let mut evidence_ids = BTreeSet::new();
    let mut candidates_with_any_evidence = BTreeSet::new();
    let mut same_candidate_ground_measurement_count = 0;
    let mut independently_corroborated_candidates = BTreeSet::new();
    let mut independent_corroboration_record_count = 0;
    let mut eligible_negative_candidates = BTreeSet::new();
    let mut eligible_negative_control_record_count = 0;
    let mut independent_interpretation_same_source_count = 0;
    let mut context_only_record_count = 0;
    let mut untested_record_count = 0;
    let mut unavailable_record_count = 0;
    let mut inconclusive_record_count = 0;
    let mut rights_not_verified_record_count = 0;
    let mut assessments = Vec::with_capacity(records.len());

    for record in records {
        if !evidence_ids.insert(record.evidence_id.as_str()) {
            return Err(EvidenceAuditError::DuplicateEvidenceId(
                record.evidence_id.clone(),
            ));
        }
        if !candidates.contains(record.candidate_id.as_str()) {
            return Err(EvidenceAuditError::EvidenceCandidateNotInManifest {
                evidence_id: record.evidence_id.clone(),
                candidate_id: record.candidate_id.clone(),
            });
        }
        validate_record(record)?;

        if record.outcome == EvidenceOutcome::NotTested {
            untested_record_count += 1;
        } else if record.outcome == EvidenceOutcome::Unavailable {
            unavailable_record_count += 1;
        } else if record.outcome == EvidenceOutcome::Inconclusive {
            inconclusive_record_count += 1;
        }
        if record.rights_state != EvidenceRightsState::Verified {
            rights_not_verified_record_count += 1;
        }

        let same_candidate = record.relationship == EvidenceRelationship::SameCandidate;
        if same_candidate && record.outcome != EvidenceOutcome::NotTested
            && record.outcome != EvidenceOutcome::Unavailable
        {
            candidates_with_any_evidence.insert(record.candidate_id.as_str());
        }
        if !same_candidate {
            context_only_record_count += 1;
        }

        let ground_modality = record.modality.is_some_and(|modality| {
            modality.is_ground_verification()
        });
        let independent_measurement =
            record.independence == EvidenceIndependence::IndependentMeasurement;
        let adequate_coverage = record
            .candidate_coverage_fraction
            .is_some_and(|coverage| {
                coverage >= config.minimum_candidate_coverage_fraction
            });
        let same_candidate_ground_measurement =
            same_candidate && ground_modality && independent_measurement;

        if same_candidate_ground_measurement {
            same_candidate_ground_measurement_count += 1;
        }

        let rights_verified = record.rights_state == EvidenceRightsState::Verified;
        let eligible_independent_corroboration = same_candidate_ground_measurement
            && adequate_coverage
            && rights_verified
            && record.outcome == EvidenceOutcome::Supports;

        if eligible_independent_corroboration {
            independently_corroborated_candidates.insert(record.candidate_id.as_str());
            independent_corroboration_record_count += 1;
        }

        let eligible_negative_control = same_candidate_ground_measurement
            && adequate_coverage
            && rights_verified
            && record.outcome == EvidenceOutcome::DoesNotSupport
            && record.detectability_assessment_ref.is_some();

        if eligible_negative_control {
            eligible_negative_candidates.insert(record.candidate_id.as_str());
            eligible_negative_control_record_count += 1;
        }

        if record.independence == EvidenceIndependence::IndependentInterpretationSameSource {
            independent_interpretation_same_source_count += 1;
        }

        let explanatory_status = if record.outcome == EvidenceOutcome::NotTested {
            "not_tested"
        } else if record.outcome == EvidenceOutcome::Unavailable {
            "evidence_unavailable"
        } else if record.rights_state != EvidenceRightsState::Verified {
            "rights_not_verified_excluded_from_corroboration"
        } else if !same_candidate {
            "context_only_does_not_test_candidate"
        } else if !independent_measurement {
            "not_an_independent_measurement"
        } else if !ground_modality {
            "independent_source_but_not_ground_verification"
        } else if !adequate_coverage {
            "candidate_coverage_below_threshold_or_missing"
        } else if record.outcome == EvidenceOutcome::Inconclusive {
            "inconclusive"
        } else if eligible_independent_corroboration {
            "independent_ground_corroboration"
        } else if eligible_negative_control {
            "eligible_independent_negative_control"
        } else {
            "not_eligible_for_independent_corroboration"
        };

        assessments.push(EvidenceRecordAssessment {
            evidence_id: record.evidence_id.clone(),
            candidate_id: record.candidate_id.clone(),
            outcome: record.outcome,
            modality: record.modality,
            relationship: record.relationship,
            independence: record.independence,
            rights_state: record.rights_state,
            same_candidate_ground_measurement,
            eligible_independent_corroboration,
            eligible_negative_control,
            adequate_candidate_coverage: adequate_coverage,
            explanatory_status: explanatory_status.to_owned(),
        });
    }

    Ok(HeritageEvidenceAuditReport {
        schema_version: EVIDENCE_AUDIT_SCHEMA_VERSION.to_owned(),
        candidate_count: candidate_ids.len(),
        evidence_record_count: records.len(),
        candidates_with_any_evidence: candidates_with_any_evidence.len(),
        same_candidate_ground_measurement_count,
        independently_corroborated_candidate_count: independently_corroborated_candidates.len(),
        independent_corroboration_record_count,
        eligible_negative_control_candidate_count: eligible_negative_candidates.len(),
        eligible_negative_control_record_count,
        independent_interpretation_same_source_count,
        context_only_record_count,
        untested_record_count,
        unavailable_record_count,
        inconclusive_record_count,
        rights_not_verified_record_count,
        minimum_candidate_coverage_fraction: config.minimum_candidate_coverage_fraction,
        assessments,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config() -> EvidenceAuditConfig {
        EvidenceAuditConfig { minimum_candidate_coverage_fraction: 0.8 }
    }

    fn record(
        evidence_id: &str,
        candidate_id: &str,
        outcome: EvidenceOutcome,
        modality: EvidenceModality,
        relation: EvidenceRelationship,
        independence: EvidenceIndependence,
        coverage: f64,
    ) -> HeritageEvidenceRecord {
        HeritageEvidenceRecord {
            evidence_id: evidence_id.into(),
            candidate_id: candidate_id.into(),
            detection_source_sha256: "a".repeat(64),
            detection_method_id: "satellite_optical_model_v1".into(),
            corroboration_source_sha256: Some("b".repeat(64)),
            corroboration_method_id: Some(format!("{modality:?}_survey_v1")),
            modality: Some(modality),
            relationship: relation,
            independence,
            outcome,
            candidate_coverage_fraction: Some(coverage),
            detectability_assessment_ref: if outcome == EvidenceOutcome::DoesNotSupport {
                Some("urn:synthetic:detectability:1".into())
            } else {
                None
            },
            evidence_assessment_ref: Some(format!("urn:synthetic:assessment:{evidence_id}")),
            reviewer_ref: Some("urn:synthetic:reviewer:1".into()),
            authority_ref: Some("urn:synthetic:custodian:approval:1".into()),
            rights_state: EvidenceRightsState::Verified,
            rights_receipt_ref: Some("urn:synthetic:rights-receipt:1".into()),
        }
    }

    #[test]
    fn same_candidate_gpr_support_counts_as_independent_corroboration() {
        let candidate_ids = vec!["candidate-a".to_owned()];
        let evidence = record(
            "evidence-a",
            "candidate-a",
            EvidenceOutcome::Supports,
            EvidenceModality::GroundPenetratingRadar,
            EvidenceRelationship::SameCandidate,
            EvidenceIndependence::IndependentMeasurement,
            0.95,
        );
        let report = audit_heritage_evidence(&candidate_ids, &[evidence], config()).unwrap();
        assert_eq!(report.independently_corroborated_candidate_count, 1);
        assert_eq!(report.independent_corroboration_record_count, 1);
    }

    #[test]
    fn geophysics_elsewhere_in_project_is_context_not_candidate_verification() {
        let candidate_ids = vec!["candidate-a".to_owned()];
        let evidence = record(
            "evidence-context",
            "candidate-a",
            EvidenceOutcome::Supports,
            EvidenceModality::Magnetometry,
            EvidenceRelationship::RegionalContext,
            EvidenceIndependence::IndependentMeasurement,
            1.0,
        );
        let report = audit_heritage_evidence(&candidate_ids, &[evidence], config()).unwrap();
        assert_eq!(report.candidates_with_any_evidence, 0);
        assert_eq!(report.context_only_record_count, 1);
        assert_eq!(report.independently_corroborated_candidate_count, 0);
    }

    #[test]
    fn inadequate_coverage_cannot_count_as_independent_confirmation() {
        let candidate_ids = vec!["candidate-a".to_owned()];
        let evidence = record(
            "evidence-low-coverage",
            "candidate-a",
            EvidenceOutcome::Supports,
            EvidenceModality::ElectricalResistivity,
            EvidenceRelationship::SameCandidate,
            EvidenceIndependence::IndependentMeasurement,
            0.2,
        );
        let report = audit_heritage_evidence(&candidate_ids, &[evidence], config()).unwrap();
        assert_eq!(report.same_candidate_ground_measurement_count, 1);
        assert_eq!(report.independently_corroborated_candidate_count, 0);
        assert_eq!(
            report.assessments[0].explanatory_status,
            "candidate_coverage_below_threshold_or_missing"
        );
    }

    #[test]
    fn negative_control_requires_detectability_and_adequate_coverage() {
        let candidate_ids = vec!["candidate-a".to_owned()];
        let mut evidence = record(
            "evidence-negative",
            "candidate-a",
            EvidenceOutcome::DoesNotSupport,
            EvidenceModality::GroundPenetratingRadar,
            EvidenceRelationship::SameCandidate,
            EvidenceIndependence::IndependentMeasurement,
            0.95,
        );
        evidence.detectability_assessment_ref = None;
        let report = audit_heritage_evidence(&candidate_ids, &[evidence.clone()], config()).unwrap();
        assert_eq!(report.eligible_negative_control_record_count, 0);

        evidence.detectability_assessment_ref = Some("urn:synthetic:detectability:depth-and-size".into());
        let report = audit_heritage_evidence(&candidate_ids, &[evidence], config()).unwrap();
        assert_eq!(report.eligible_negative_control_record_count, 1);
        assert_eq!(report.eligible_negative_control_candidate_count, 1);
    }

    #[test]
    fn independent_interpretation_of_same_satellite_source_is_not_ground_measurement() {
        let candidate_ids = vec!["candidate-a".to_owned()];
        let mut evidence = record(
            "evidence-review",
            "candidate-a",
            EvidenceOutcome::Supports,
            EvidenceModality::SatelliteOptical,
            EvidenceRelationship::SameCandidate,
            EvidenceIndependence::IndependentInterpretationSameSource,
            1.0,
        );
        evidence.corroboration_source_sha256 =
            Some(evidence.detection_source_sha256.clone());
        evidence.corroboration_method_id = Some("independent_archaeologist_review".into());

        let report = audit_heritage_evidence(&candidate_ids, &[evidence], config()).unwrap();
        assert_eq!(report.independent_interpretation_same_source_count, 1);
        assert_eq!(report.same_candidate_ground_measurement_count, 0);
        assert_eq!(report.independently_corroborated_candidate_count, 0);
    }

    #[test]
    fn shared_source_geophysics_does_not_claim_independence() {
        let candidate_ids = vec!["candidate-a".to_owned()];
        let mut evidence = record(
            "evidence-shared",
            "candidate-a",
            EvidenceOutcome::Supports,
            EvidenceModality::GroundPenetratingRadar,
            EvidenceRelationship::SameCandidate,
            EvidenceIndependence::SharedSource,
            1.0,
        );
        evidence.corroboration_source_sha256 =
            Some(evidence.detection_source_sha256.clone());
        let report = audit_heritage_evidence(&candidate_ids, &[evidence], config()).unwrap();
        assert_eq!(report.independently_corroborated_candidate_count, 0);
        assert_eq!(report.assessments[0].explanatory_status, "not_an_independent_measurement");
    }

    #[test]
    fn unverified_rights_exclude_evidence_from_corroboration() {
        let candidate_ids = vec!["candidate-a".to_owned()];
        let mut evidence = record(
            "evidence-rights",
            "candidate-a",
            EvidenceOutcome::Supports,
            EvidenceModality::Excavation,
            EvidenceRelationship::SameCandidate,
            EvidenceIndependence::IndependentMeasurement,
            1.0,
        );
        evidence.rights_state = EvidenceRightsState::Pending;
        evidence.rights_receipt_ref = None;
        let report = audit_heritage_evidence(&candidate_ids, &[evidence], config()).unwrap();
        assert_eq!(report.rights_not_verified_record_count, 1);
        assert_eq!(report.independently_corroborated_candidate_count, 0);
    }

    #[test]
    fn untested_and_unavailable_are_not_negative_controls() {
        let candidate_ids = vec!["candidate-a".to_owned(), "candidate-b".to_owned()];
        let untested = HeritageEvidenceRecord {
            evidence_id: "untested".into(),
            candidate_id: "candidate-a".into(),
            detection_source_sha256: "a".repeat(64),
            detection_method_id: "satellite_model_v1".into(),
            corroboration_source_sha256: None,
            corroboration_method_id: None,
            modality: None,
            relationship: EvidenceRelationship::SameCandidate,
            independence: EvidenceIndependence::NotAssessed,
            outcome: EvidenceOutcome::NotTested,
            candidate_coverage_fraction: None,
            detectability_assessment_ref: None,
            evidence_assessment_ref: None,
            reviewer_ref: None,
            authority_ref: None,
            rights_state: EvidenceRightsState::Unknown,
            rights_receipt_ref: None,
        };
        let mut unavailable = untested.clone();
        unavailable.evidence_id = "unavailable".into();
        unavailable.candidate_id = "candidate-b".into();
        unavailable.outcome = EvidenceOutcome::Unavailable;

        let report = audit_heritage_evidence(&candidate_ids, &[untested, unavailable], config()).unwrap();
        assert_eq!(report.untested_record_count, 1);
        assert_eq!(report.unavailable_record_count, 1);
        assert_eq!(report.eligible_negative_control_record_count, 0);
    }

    #[test]
    fn shared_source_label_requires_same_source_digest() {
        let candidate_ids = vec!["candidate-a".to_owned()];
        let mut evidence = record(
            "shared-mismatch",
            "candidate-a",
            EvidenceOutcome::Supports,
            EvidenceModality::GroundPenetratingRadar,
            EvidenceRelationship::SameCandidate,
            EvidenceIndependence::SharedSource,
            1.0,
        );
        assert!(matches!(
            audit_heritage_evidence(&candidate_ids, &[evidence.clone()], config()),
            Err(EvidenceAuditError::SharedSourceUsesDifferentDigest { .. })
        ));
        evidence.corroboration_source_sha256 =
            Some(evidence.detection_source_sha256.clone());
        assert!(audit_heritage_evidence(&candidate_ids, &[evidence], config()).is_ok());
    }

    #[test]
    fn independent_measurement_must_use_different_source_and_method() {
        let candidate_ids = vec!["candidate-a".to_owned()];
        let mut evidence = record(
            "not-independent",
            "candidate-a",
            EvidenceOutcome::Supports,
            EvidenceModality::Magnetometry,
            EvidenceRelationship::SameCandidate,
            EvidenceIndependence::IndependentMeasurement,
            1.0,
        );
        evidence.corroboration_source_sha256 =
            Some(evidence.detection_source_sha256.clone());
        assert!(matches!(
            audit_heritage_evidence(&candidate_ids, &[evidence], config()),
            Err(EvidenceAuditError::IndependentMeasurementReusesDetectionSource { .. })
        ));
    }

    #[test]
    fn invalid_candidate_coverage_duplicate_ids_and_bad_digests_fail_closed() {
        let candidate_ids = vec!["candidate-a".to_owned()];
        let mut evidence = record(
            "bad-coverage",
            "candidate-a",
            EvidenceOutcome::Supports,
            EvidenceModality::FieldSurvey,
            EvidenceRelationship::SameCandidate,
            EvidenceIndependence::IndependentMeasurement,
            1.0,
        );
        evidence.candidate_coverage_fraction = Some(1.1);
        assert!(matches!(
            audit_heritage_evidence(&candidate_ids, &[evidence.clone()], config()),
            Err(EvidenceAuditError::InvalidCandidateCoverage { .. })
        ));

        evidence.candidate_coverage_fraction = Some(1.0);
        assert!(matches!(
            audit_heritage_evidence(&candidate_ids, &[evidence.clone(), evidence.clone()], config()),
            Err(EvidenceAuditError::DuplicateEvidenceId(_))
        ));

        evidence.detection_source_sha256 = "bad-digest".into();
        assert!(matches!(
            audit_heritage_evidence(&candidate_ids, &[evidence], config()),
            Err(EvidenceAuditError::InvalidDetectionSourceDigest { .. })
        ));
    }
}
