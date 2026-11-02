// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Typed evidence audit for competing archaeological change-cause hypotheses.
//!
//! This module records support, contradiction, provenance, rights and
//! independence separately. It does not infer cause, adjudicate an event, or
//! determine heritage significance. Repeated interpretations of one source,
//! model-derived outputs and context-only records remain visible but cannot
//! inflate independent corroboration.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use super::heritage_cause::{CHANGE_CAUSES, ChangeCause};

pub const CAUSE_EVIDENCE_AUDIT_SCHEMA_VERSION: &str = "luminous.heritage-cause-evidence-audit.v0.1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CauseEvidencePolarity {
    Supports,
    Contradicts,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CauseEvidenceModality {
    SatelliteOptical,
    SatelliteSar,
    SatelliteMultispectral,
    UavOptical,
    UavThermal,
    AirborneLidar,
    TerrestrialLidar,
    FieldWalkover,
    Magnetometry,
    GroundPenetratingRadar,
    ElectricalResistivity,
    Archival,
    CommunityObservation,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CauseEvidenceTier {
    FieldAssessed,
    IndependentlyExpertReviewed,
    InstrumentObservation,
    SameSourceInterpretation,
    ModelDerived,
    CatalogueReference,
    Unknown,
}

impl CauseEvidenceTier {
    fn can_count_as_independent(self) -> bool {
        matches!(
            self,
            Self::FieldAssessed | Self::IndependentlyExpertReviewed | Self::InstrumentObservation
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CauseEvidenceScope {
    SameEvent,
    ContextOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CauseEvidenceRightsStatus {
    Verified,
    Pending,
    Denied,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CauseEvidenceRecord {
    pub evidence_id: String,
    pub event_id: String,
    pub cause: ChangeCause,
    pub polarity: CauseEvidencePolarity,
    pub modality: CauseEvidenceModality,
    pub evidence_tier: CauseEvidenceTier,
    pub scope: CauseEvidenceScope,
    pub rights_status: CauseEvidenceRightsStatus,
    pub source_ref: String,
    /// SHA-256 of the raw source asset, not a derived chip or model output.
    pub source_asset_sha256: String,
    pub method_id: String,
    /// Declared acquisition/observation independence group. A new image from
    /// the same campaign may share a group and therefore is not independent.
    pub independence_group_id: String,
    pub rights_receipt_ref: Option<String>,
    pub reviewer_ref: Option<String>,
    pub authority_ref: Option<String>,
    /// True when the observation is a non-detection under a declared survey.
    pub is_non_detection: bool,
    /// Required for non-detections; documents target size/depth and method
    /// sensitivity under the actual survey conditions.
    pub detectability_assessment_ref: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CauseHypothesisEvidence {
    pub cause: ChangeCause,
    /// Independent one-vs-rest cause probability; probabilities across causes
    /// are not required to sum to one because the taxonomy is multi-label.
    pub probability: f64,
    pub support_evidence_refs: Vec<String>,
    pub contradiction_evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CauseEvidenceAuditCase {
    pub case_id: String,
    pub event_id: String,
    pub abstained: bool,
    pub cause_threshold: f64,
    pub hypotheses: Vec<CauseHypothesisEvidence>,
    pub evidence_records: Vec<CauseEvidenceRecord>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CauseEvidenceHypothesisSummary {
    pub cause: ChangeCause,
    pub probability: f64,
    pub asserted: bool,
    pub has_event_specific_support: bool,
    pub support_record_count: usize,
    pub contradiction_record_count: usize,
    pub independent_support_count: usize,
    pub independent_contradiction_count: usize,
    pub non_independent_support_count: usize,
    pub non_independent_contradiction_count: usize,
    pub independent_support_modality_count: usize,
    pub independent_contradiction_modality_count: usize,
    pub cross_polarity_conflict_record_count: usize,
    pub contested: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CauseEvidenceAuditReport {
    pub schema_version: String,
    pub case_id: String,
    pub event_id: String,
    pub abstained: bool,
    pub cause_hypothesis_count: usize,
    pub asserted_cause_count: usize,
    pub asserted_causes_with_independent_support: usize,
    pub unsupported_asserted_cause_count: usize,
    pub contested_cause_count: usize,
    pub evidence_record_count: usize,
    pub same_event_evidence_record_count: usize,
    pub context_only_evidence_record_count: usize,
    pub rights_unverified_evidence_record_count: usize,
    pub model_or_catalogue_evidence_record_count: usize,
    pub same_source_interpretation_record_count: usize,
    pub non_detection_record_count: usize,
    pub independent_support_evidence_count: usize,
    pub independent_contradiction_evidence_count: usize,
    pub non_independent_support_evidence_count: usize,
    pub non_independent_contradiction_evidence_count: usize,
    pub cross_polarity_conflict_record_count: usize,
    pub hypothesis_summaries: Vec<CauseEvidenceHypothesisSummary>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CauseEvidenceAuditError {
    EmptyCaseId,
    EmptyEventId,
    InvalidCauseThreshold,
    MissingCauseHypothesis(ChangeCause),
    DuplicateCauseHypothesis(ChangeCause),
    EmptyEvidenceId,
    DuplicateEvidenceId(String),
    EvidenceForDifferentEvent {
        evidence_id: String,
    },
    EmptySourceReference {
        evidence_id: String,
    },
    InvalidSourceDigest {
        evidence_id: String,
    },
    EmptyMethodId {
        evidence_id: String,
    },
    EmptyIndependenceGroupId {
        evidence_id: String,
    },
    EmptyRightsReceipt {
        evidence_id: String,
    },
    EmptyReviewerReference {
        evidence_id: String,
    },
    EmptyAuthorityReference {
        evidence_id: String,
    },
    MissingDetectabilityAssessment {
        evidence_id: String,
    },
    EmptyEvidenceReference {
        cause: ChangeCause,
    },
    DuplicateEvidenceReference {
        cause: ChangeCause,
        evidence_id: String,
    },
    EvidenceReferenceNotFound {
        cause: ChangeCause,
        evidence_id: String,
    },
    EvidenceCauseMismatch {
        cause: ChangeCause,
        evidence_id: String,
    },
    EvidencePolarityMismatch {
        cause: ChangeCause,
        evidence_id: String,
    },
    UnreferencedEvidenceRecord(String),
    InvalidCauseProbability {
        cause: ChangeCause,
    },
}

impl fmt::Display for CauseEvidenceAuditError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "heritage cause evidence audit error: {self:?}")
    }
}

impl std::error::Error for CauseEvidenceAuditError {}

fn is_sha256_hex(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn validate_evidence_record(
    case: &CauseEvidenceAuditCase,
    record: &CauseEvidenceRecord,
) -> Result<(), CauseEvidenceAuditError> {
    if record.evidence_id.trim().is_empty() {
        return Err(CauseEvidenceAuditError::EmptyEvidenceId);
    }
    if record.event_id != case.event_id {
        return Err(CauseEvidenceAuditError::EvidenceForDifferentEvent {
            evidence_id: record.evidence_id.clone(),
        });
    }
    if record.source_ref.trim().is_empty() {
        return Err(CauseEvidenceAuditError::EmptySourceReference {
            evidence_id: record.evidence_id.clone(),
        });
    }
    if !is_sha256_hex(&record.source_asset_sha256) {
        return Err(CauseEvidenceAuditError::InvalidSourceDigest {
            evidence_id: record.evidence_id.clone(),
        });
    }
    if record.method_id.trim().is_empty() {
        return Err(CauseEvidenceAuditError::EmptyMethodId {
            evidence_id: record.evidence_id.clone(),
        });
    }
    if record.independence_group_id.trim().is_empty() {
        return Err(CauseEvidenceAuditError::EmptyIndependenceGroupId {
            evidence_id: record.evidence_id.clone(),
        });
    }
    if record.rights_status == CauseEvidenceRightsStatus::Verified
        && record
            .rights_receipt_ref
            .as_ref()
            .is_none_or(|reference| reference.trim().is_empty())
    {
        return Err(CauseEvidenceAuditError::EmptyRightsReceipt {
            evidence_id: record.evidence_id.clone(),
        });
    }
    if matches!(
        record.evidence_tier,
        CauseEvidenceTier::FieldAssessed | CauseEvidenceTier::IndependentlyExpertReviewed
    ) {
        if record
            .reviewer_ref
            .as_ref()
            .is_none_or(|reference| reference.trim().is_empty())
        {
            return Err(CauseEvidenceAuditError::EmptyReviewerReference {
                evidence_id: record.evidence_id.clone(),
            });
        }
        if record
            .authority_ref
            .as_ref()
            .is_none_or(|reference| reference.trim().is_empty())
        {
            return Err(CauseEvidenceAuditError::EmptyAuthorityReference {
                evidence_id: record.evidence_id.clone(),
            });
        }
    }
    if record.is_non_detection
        && record
            .detectability_assessment_ref
            .as_ref()
            .is_none_or(|reference| reference.trim().is_empty())
    {
        return Err(CauseEvidenceAuditError::MissingDetectabilityAssessment {
            evidence_id: record.evidence_id.clone(),
        });
    }
    if record.is_non_detection && record.polarity != CauseEvidencePolarity::Contradicts {
        return Err(CauseEvidenceAuditError::NonDetectionPolarityMustContradict {
            evidence_id: record.evidence_id.clone(),
        });
    }
    Ok(())
}

fn independent_eligible(record: &CauseEvidenceRecord) -> bool {
    record.scope == CauseEvidenceScope::SameEvent
        && record.rights_status == CauseEvidenceRightsStatus::Verified
        && record.evidence_tier.can_count_as_independent()
        && record
            .rights_receipt_ref
            .as_ref()
            .is_some_and(|reference| !reference.trim().is_empty())
}

fn cross_polarity_conflict_keys(
    support_records: &[&CauseEvidenceRecord],
    contradiction_records: &[&CauseEvidenceRecord],
) -> (BTreeSet<String>, BTreeSet<String>) {
    let mut digest_polarities: BTreeMap<String, (bool, bool)> = BTreeMap::new();
    let mut group_polarities: BTreeMap<String, (bool, bool)> = BTreeMap::new();

    for (records, is_support) in [
        (support_records, true),
        (contradiction_records, false),
    ] {
        for record in records.iter().copied().filter(|record| independent_eligible(record)) {
            let digest = record.source_asset_sha256.to_ascii_lowercase();
            let group = record.independence_group_id.trim().to_owned();
            let entry = digest_polarities.entry(digest).or_default();
            if is_support { entry.0 = true; } else { entry.1 = true; }
            let entry = group_polarities.entry(group).or_default();
            if is_support { entry.0 = true; } else { entry.1 = true; }
        }
    }

    let conflicted_digests = digest_polarities
        .into_iter()
        .filter_map(|(key, (support, contradiction))| (support && contradiction).then_some(key))
        .collect();
    let conflicted_groups = group_polarities
        .into_iter()
        .filter_map(|(key, (support, contradiction))| (support && contradiction).then_some(key))
        .collect();

    (conflicted_digests, conflicted_groups)
}

fn count_independent_records<'a>(
    records: impl Iterator<Item = &'a CauseEvidenceRecord>,
    conflicted_digests: &BTreeSet<String>,
    conflicted_groups: &BTreeSet<String>,
) -> (usize, usize, usize, BTreeSet<CauseEvidenceModality>) {
    let mut seen_source_digests = BTreeSet::new();
    let mut seen_independence_groups = BTreeSet::new();
    let mut independent_modalities = BTreeSet::new();
    let mut independent_count = 0usize;
    let mut non_independent_count = 0usize;
    let mut cross_polarity_conflict_count = 0usize;

    let mut ordered = records.collect::<Vec<_>>();
    ordered.sort_by(|left, right| left.evidence_id.cmp(&right.evidence_id));

    for record in ordered {
        if !independent_eligible(record) {
            non_independent_count += 1;
            continue;
        }
        let digest = record.source_asset_sha256.to_ascii_lowercase();
        let group = record.independence_group_id.trim().to_owned();
        // A source/acquisition group that appears on both polarity sides is
        // contested, not two independent observations. Keep it visible but
        // exclude it from both independent counts.
        if conflicted_digests.contains(&digest) || conflicted_groups.contains(&group) {
            non_independent_count += 1;
            cross_polarity_conflict_count += 1;
            continue;
        }
        // Within one polarity, each raw source and independence group counts once.
        if seen_source_digests.contains(&digest) || seen_independence_groups.contains(&group) {
            non_independent_count += 1;
            continue;
        }
        seen_source_digests.insert(digest);
        seen_independence_groups.insert(group);
        independent_modalities.insert(record.modality);
        independent_count += 1;
    }

    (
        independent_count,
        non_independent_count,
        cross_polarity_conflict_count,
        independent_modalities,
    )
}

/// Audit an evidence bundle for all 11 cause hypotheses. References must
/// resolve exactly to typed, event-specific evidence records. An asserted
/// cause with no independent same-event support is reported as unsupported;
/// it is not upgraded to an adjudicated cause. Contradictory evidence is
/// preserved and surfaced as a contested hypothesis.
pub fn audit_cause_evidence(
    case: &CauseEvidenceAuditCase,
) -> Result<CauseEvidenceAuditReport, CauseEvidenceAuditError> {
    if case.case_id.trim().is_empty() {
        return Err(CauseEvidenceAuditError::EmptyCaseId);
    }
    if case.event_id.trim().is_empty() {
        return Err(CauseEvidenceAuditError::EmptyEventId);
    }
    if !case.cause_threshold.is_finite() || !(0.0..=1.0).contains(&case.cause_threshold) {
        return Err(CauseEvidenceAuditError::InvalidCauseThreshold);
    }

    let mut hypotheses_by_cause = BTreeMap::new();
    for hypothesis in &case.hypotheses {
        if !hypothesis.probability.is_finite() || !(0.0..=1.0).contains(&hypothesis.probability) {
            return Err(CauseEvidenceAuditError::InvalidCauseProbability {
                cause: hypothesis.cause,
            });
        }
        if hypotheses_by_cause
            .insert(hypothesis.cause, hypothesis)
            .is_some()
        {
            return Err(CauseEvidenceAuditError::DuplicateCauseHypothesis(
                hypothesis.cause,
            ));
        }
    }
    for cause in CHANGE_CAUSES {
        if !hypotheses_by_cause.contains_key(&cause) {
            return Err(CauseEvidenceAuditError::MissingCauseHypothesis(cause));
        }
    }

    let mut records_by_id = BTreeMap::new();
    for record in &case.evidence_records {
        validate_evidence_record(case, record)?;
        if records_by_id
            .insert(record.evidence_id.as_str(), record)
            .is_some()
        {
            return Err(CauseEvidenceAuditError::DuplicateEvidenceId(
                record.evidence_id.clone(),
            ));
        }
    }

    let mut referenced_ids = BTreeSet::new();
    let mut summaries = Vec::with_capacity(CHANGE_CAUSES.len());
    let mut asserted_cause_count = 0usize;
    let mut asserted_causes_with_independent_support = 0usize;
    let mut unsupported_asserted_cause_count = 0usize;
    let mut contested_cause_count = 0usize;
    let mut independent_support_evidence_count = 0usize;
    let mut independent_contradiction_evidence_count = 0usize;
    let mut non_independent_support_evidence_count = 0usize;
    let mut non_independent_contradiction_evidence_count = 0usize;
    let mut cross_polarity_conflict_record_count = 0usize;

    for cause in CHANGE_CAUSES {
        let hypothesis = hypotheses_by_cause
            .get(&cause)
            .expect("all cause hypotheses were validated");
        let mut support_records = Vec::new();
        let mut contradiction_records = Vec::new();

        for reference in &hypothesis.support_evidence_refs {
            if reference.trim().is_empty() {
                return Err(CauseEvidenceAuditError::EmptyEvidenceReference { cause });
            }
            if !referenced_ids.insert(reference.as_str()) {
                return Err(CauseEvidenceAuditError::DuplicateEvidenceReference {
                    cause,
                    evidence_id: reference.clone(),
                });
            }
            let record = records_by_id.get(reference.as_str()).ok_or_else(|| {
                CauseEvidenceAuditError::EvidenceReferenceNotFound {
                    cause,
                    evidence_id: reference.clone(),
                }
            })?;
            if record.cause != cause {
                return Err(CauseEvidenceAuditError::EvidenceCauseMismatch {
                    cause,
                    evidence_id: reference.clone(),
                });
            }
            if record.polarity != CauseEvidencePolarity::Supports {
                return Err(CauseEvidenceAuditError::EvidencePolarityMismatch {
                    cause,
                    evidence_id: reference.clone(),
                });
            }
            support_records.push(*record);
        }

        for reference in &hypothesis.contradiction_evidence_refs {
            if reference.trim().is_empty() {
                return Err(CauseEvidenceAuditError::EmptyEvidenceReference { cause });
            }
            if !referenced_ids.insert(reference.as_str()) {
                return Err(CauseEvidenceAuditError::DuplicateEvidenceReference {
                    cause,
                    evidence_id: reference.clone(),
                });
            }
            let record = records_by_id.get(reference.as_str()).ok_or_else(|| {
                CauseEvidenceAuditError::EvidenceReferenceNotFound {
                    cause,
                    evidence_id: reference.clone(),
                }
            })?;
            if record.cause != cause {
                return Err(CauseEvidenceAuditError::EvidenceCauseMismatch {
                    cause,
                    evidence_id: reference.clone(),
                });
            }
            if record.polarity != CauseEvidencePolarity::Contradicts {
                return Err(CauseEvidenceAuditError::EvidencePolarityMismatch {
                    cause,
                    evidence_id: reference.clone(),
                });
            }
            contradiction_records.push(*record);
        }

        let (conflicted_digests, conflicted_groups) =
            cross_polarity_conflict_keys(&support_records, &contradiction_records);
        let (
            independent_support_count,
            non_independent_support_count,
            support_cross_polarity_conflicts,
            support_modalities,
        ) = count_independent_records(
            support_records.iter().copied(),
            &conflicted_digests,
            &conflicted_groups,
        );
        let (
            independent_contradiction_count,
            non_independent_contradiction_count,
            contradiction_cross_polarity_conflicts,
            contradiction_modalities,
        ) = count_independent_records(
            contradiction_records.iter().copied(),
            &conflicted_digests,
            &conflicted_groups,
        );
        let hypothesis_cross_polarity_conflict_count =
            support_cross_polarity_conflicts + contradiction_cross_polarity_conflicts;
        cross_polarity_conflict_record_count += hypothesis_cross_polarity_conflict_count;

        let asserted = !case.abstained && hypothesis.probability >= case.cause_threshold;
        let has_event_specific_support = independent_support_count > 0;
        let contested = !support_records.is_empty() && !contradiction_records.is_empty();

        if asserted {
            asserted_cause_count += 1;
            if has_event_specific_support {
                asserted_causes_with_independent_support += 1;
            } else {
                unsupported_asserted_cause_count += 1;
            }
        }
        if contested {
            contested_cause_count += 1;
        }

        independent_support_evidence_count += independent_support_count;
        independent_contradiction_evidence_count += independent_contradiction_count;
        non_independent_support_evidence_count += non_independent_support_count;
        non_independent_contradiction_evidence_count += non_independent_contradiction_count;

        summaries.push(CauseEvidenceHypothesisSummary {
            cause,
            probability: hypothesis.probability,
            asserted,
            has_event_specific_support,
            support_record_count: support_records.len(),
            contradiction_record_count: contradiction_records.len(),
            independent_support_count,
            independent_contradiction_count,
            non_independent_support_count,
            non_independent_contradiction_count,
            independent_support_modality_count: support_modalities.len(),
            independent_contradiction_modality_count: contradiction_modalities.len(),
            cross_polarity_conflict_record_count: hypothesis_cross_polarity_conflict_count,
            contested,
        });
    }

    for record in &case.evidence_records {
        if !referenced_ids.contains(record.evidence_id.as_str()) {
            return Err(CauseEvidenceAuditError::UnreferencedEvidenceRecord(
                record.evidence_id.clone(),
            ));
        }
    }

    let same_event_evidence_record_count = case
        .evidence_records
        .iter()
        .filter(|record| record.scope == CauseEvidenceScope::SameEvent)
        .count();
    let context_only_evidence_record_count = case
        .evidence_records
        .iter()
        .filter(|record| record.scope == CauseEvidenceScope::ContextOnly)
        .count();
    let rights_unverified_evidence_record_count = case
        .evidence_records
        .iter()
        .filter(|record| record.rights_status != CauseEvidenceRightsStatus::Verified)
        .count();
    let model_or_catalogue_evidence_record_count = case
        .evidence_records
        .iter()
        .filter(|record| {
            matches!(
                record.evidence_tier,
                CauseEvidenceTier::ModelDerived
                    | CauseEvidenceTier::CatalogueReference
                    | CauseEvidenceTier::Unknown
            )
        })
        .count();
    let same_source_interpretation_record_count = case
        .evidence_records
        .iter()
        .filter(|record| record.evidence_tier == CauseEvidenceTier::SameSourceInterpretation)
        .count();
    let non_detection_record_count = case
        .evidence_records
        .iter()
        .filter(|record| record.is_non_detection)
        .count();

    Ok(CauseEvidenceAuditReport {
        schema_version: CAUSE_EVIDENCE_AUDIT_SCHEMA_VERSION.to_owned(),
        case_id: case.case_id.clone(),
        event_id: case.event_id.clone(),
        abstained: case.abstained,
        cause_hypothesis_count: summaries.len(),
        asserted_cause_count,
        asserted_causes_with_independent_support,
        unsupported_asserted_cause_count,
        contested_cause_count,
        evidence_record_count: case.evidence_records.len(),
        same_event_evidence_record_count,
        context_only_evidence_record_count,
        rights_unverified_evidence_record_count,
        model_or_catalogue_evidence_record_count,
        same_source_interpretation_record_count,
        non_detection_record_count,
        independent_support_evidence_count,
        independent_contradiction_evidence_count,
        non_independent_support_evidence_count,
        non_independent_contradiction_evidence_count,
        cross_polarity_conflict_record_count,
        hypothesis_summaries: summaries,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hypotheses() -> Vec<CauseHypothesisEvidence> {
        CHANGE_CAUSES
            .iter()
            .map(|cause| CauseHypothesisEvidence {
                cause: *cause,
                probability: 0.05,
                support_evidence_refs: Vec::new(),
                contradiction_evidence_refs: Vec::new(),
            })
            .collect()
    }

    fn hypothesis_mut<'a>(
        hypotheses: &'a mut [CauseHypothesisEvidence],
        cause: ChangeCause,
    ) -> &'a mut CauseHypothesisEvidence {
        hypotheses
            .iter_mut()
            .find(|item| item.cause == cause)
            .unwrap()
    }

    fn evidence(
        id: &str,
        cause: ChangeCause,
        polarity: CauseEvidencePolarity,
        digest: char,
        group: &str,
    ) -> CauseEvidenceRecord {
        CauseEvidenceRecord {
            evidence_id: id.into(),
            event_id: "event-a".into(),
            cause,
            polarity,
            modality: CauseEvidenceModality::FieldWalkover,
            evidence_tier: CauseEvidenceTier::FieldAssessed,
            scope: CauseEvidenceScope::SameEvent,
            rights_status: CauseEvidenceRightsStatus::Verified,
            source_ref: format!("urn:synthetic:source:{id}"),
            source_asset_sha256: digest.to_string().repeat(64),
            method_id: "field-observation-v1".into(),
            independence_group_id: group.into(),
            rights_receipt_ref: Some(format!("urn:synthetic:rights:{id}")),
            reviewer_ref: Some("urn:synthetic:reviewer".into()),
            authority_ref: Some("urn:synthetic:authority".into()),
            is_non_detection: false,
            detectability_assessment_ref: None,
        }
    }

    fn case(
        hypotheses: Vec<CauseHypothesisEvidence>,
        evidence_records: Vec<CauseEvidenceRecord>,
    ) -> CauseEvidenceAuditCase {
        CauseEvidenceAuditCase {
            case_id: "case-a".into(),
            event_id: "event-a".into(),
            abstained: false,
            cause_threshold: 0.7,
            hypotheses,
            evidence_records,
        }
    }

    #[test]
    fn separate_sources_and_independence_groups_count_as_independent_support() {
        let cause = ChangeCause::ArchaeologicalExcavationLooting;
        let records = vec![
            evidence(
                "support-a",
                cause,
                CauseEvidencePolarity::Supports,
                'a',
                "group-a",
            ),
            evidence(
                "support-b",
                cause,
                CauseEvidencePolarity::Supports,
                'b',
                "group-b",
            ),
        ];
        let mut hypotheses = hypotheses();
        let claim = hypothesis_mut(&mut hypotheses, cause);
        claim.probability = 0.95;
        claim.support_evidence_refs = vec!["support-a".into(), "support-b".into()];

        let report = audit_cause_evidence(&case(hypotheses, records)).unwrap();
        let summary = report
            .hypothesis_summaries
            .iter()
            .find(|item| item.cause == cause)
            .unwrap();
        assert!(summary.asserted);
        assert_eq!(summary.independent_support_count, 2);
        assert_eq!(summary.independent_support_modality_count, 1);
        assert_eq!(report.unsupported_asserted_cause_count, 0);
    }

    #[test]
    fn same_source_or_same_independence_group_never_increases_independent_count() {
        let cause = ChangeCause::ArchaeologicalExcavationLooting;
        let records = vec![
            evidence(
                "support-a",
                cause,
                CauseEvidencePolarity::Supports,
                'a',
                "group-a",
            ),
            evidence(
                "same-source",
                cause,
                CauseEvidencePolarity::Supports,
                'a',
                "group-b",
            ),
            evidence(
                "same-group",
                cause,
                CauseEvidencePolarity::Supports,
                'b',
                "group-a",
            ),
        ];
        let mut hypotheses = hypotheses();
        let claim = hypothesis_mut(&mut hypotheses, cause);
        claim.probability = 0.9;
        claim.support_evidence_refs = vec![
            "support-a".into(),
            "same-source".into(),
            "same-group".into(),
        ];

        let report = audit_cause_evidence(&case(hypotheses, records)).unwrap();
        let summary = report
            .hypothesis_summaries
            .iter()
            .find(|item| item.cause == cause)
            .unwrap();
        assert_eq!(summary.independent_support_count, 1);
        assert_eq!(summary.non_independent_support_count, 2);
    }

    #[test]
    fn same_source_interpretation_is_visible_but_not_independent_support() {
        let cause = ChangeCause::ArchaeologicalExcavationLooting;
        let mut record = evidence(
            "same-source",
            cause,
            CauseEvidencePolarity::Supports,
            'a',
            "group-a",
        );
        record.evidence_tier = CauseEvidenceTier::SameSourceInterpretation;
        let mut hypotheses = hypotheses();
        hypothesis_mut(&mut hypotheses, cause).support_evidence_refs = vec!["same-source".into()];

        let report = audit_cause_evidence(&case(hypotheses, vec![record])).unwrap();
        let summary = report
            .hypothesis_summaries
            .iter()
            .find(|item| item.cause == cause)
            .unwrap();
        assert_eq!(summary.support_record_count, 1);
        assert_eq!(summary.independent_support_count, 0);
        assert_eq!(summary.non_independent_support_count, 1);
    }

    #[test]
    fn context_only_and_rights_pending_evidence_are_not_independent() {
        let cause = ChangeCause::ArchaeologicalExcavationLooting;
        let mut context = evidence(
            "context",
            cause,
            CauseEvidencePolarity::Supports,
            'a',
            "group-a",
        );
        context.scope = CauseEvidenceScope::ContextOnly;
        let mut pending = evidence(
            "pending",
            cause,
            CauseEvidencePolarity::Supports,
            'b',
            "group-b",
        );
        pending.rights_status = CauseEvidenceRightsStatus::Pending;
        pending.rights_receipt_ref = None;
        let mut hypotheses = hypotheses();
        hypothesis_mut(&mut hypotheses, cause).support_evidence_refs =
            vec!["context".into(), "pending".into()];

        let report = audit_cause_evidence(&case(hypotheses, vec![context, pending])).unwrap();
        let summary = report
            .hypothesis_summaries
            .iter()
            .find(|item| item.cause == cause)
            .unwrap();
        assert_eq!(summary.independent_support_count, 0);
        assert_eq!(summary.non_independent_support_count, 2);
        assert_eq!(report.context_only_evidence_record_count, 1);
        assert_eq!(report.rights_unverified_evidence_record_count, 1);
    }

    #[test]
    fn support_and_contradiction_are_both_preserved_as_contested_evidence() {
        let cause = ChangeCause::ArchaeologicalExcavationLooting;
        let records = vec![
            evidence(
                "supports",
                cause,
                CauseEvidencePolarity::Supports,
                'a',
                "group-a",
            ),
            evidence(
                "contradicts",
                cause,
                CauseEvidencePolarity::Contradicts,
                'b',
                "group-b",
            ),
        ];
        let mut hypotheses = hypotheses();
        let claim = hypothesis_mut(&mut hypotheses, cause);
        claim.probability = 0.85;
        claim.support_evidence_refs = vec!["supports".into()];
        claim.contradiction_evidence_refs = vec!["contradicts".into()];

        let report = audit_cause_evidence(&case(hypotheses, records)).unwrap();
        let summary = report
            .hypothesis_summaries
            .iter()
            .find(|item| item.cause == cause)
            .unwrap();
        assert!(summary.contested);
        assert_eq!(summary.independent_support_count, 1);
        assert_eq!(summary.independent_contradiction_count, 1);
        assert_eq!(report.contested_cause_count, 1);
    }

    #[test]
    fn asserted_cause_without_independent_support_is_reported_as_unsupported() {
        let cause = ChangeCause::ArchaeologicalExcavationLooting;
        let mut hypotheses = hypotheses();
        hypothesis_mut(&mut hypotheses, cause).probability = 0.95;
        let report = audit_cause_evidence(&case(hypotheses, Vec::new())).unwrap();
        assert_eq!(report.asserted_cause_count, 1);
        assert_eq!(report.unsupported_asserted_cause_count, 1);
        assert_eq!(report.asserted_causes_with_independent_support, 0);
    }

    #[test]
    fn non_detection_requires_a_detectability_assessment() {
        let cause = ChangeCause::AgricultureCultivation;
        let mut record = evidence(
            "non-detection",
            cause,
            CauseEvidencePolarity::Contradicts,
            'a',
            "group-a",
        );
        record.is_non_detection = true;
        let mut hypotheses = hypotheses();
        hypothesis_mut(&mut hypotheses, cause).contradiction_evidence_refs =
            vec!["non-detection".into()];
        assert!(matches!(
            audit_cause_evidence(&case(hypotheses, vec![record])),
            Err(CauseEvidenceAuditError::MissingDetectabilityAssessment { .. })
        ));
    }

    #[test]
    fn non_detection_with_detectability_is_retained_as_contradictory_evidence() {
        let cause = ChangeCause::AgricultureCultivation;
        let mut record = evidence(
            "non-detection",
            cause,
            CauseEvidencePolarity::Contradicts,
            'a',
            "group-a",
        );
        record.is_non_detection = true;
        record.detectability_assessment_ref = Some("urn:synthetic:detectability".into());
        let mut hypotheses = hypotheses();
        hypothesis_mut(&mut hypotheses, cause).contradiction_evidence_refs =
            vec!["non-detection".into()];
        let report = audit_cause_evidence(&case(hypotheses, vec![record])).unwrap();
        assert_eq!(report.non_detection_record_count, 1);
        let summary = report
            .hypothesis_summaries
            .iter()
            .find(|item| item.cause == cause)
            .unwrap();
        assert_eq!(summary.independent_contradiction_count, 1);
    }

    #[test]
    fn evidence_for_a_different_event_is_rejected() {
        let cause = ChangeCause::ArchaeologicalExcavationLooting;
        let mut record = evidence(
            "other-event",
            cause,
            CauseEvidencePolarity::Supports,
            'a',
            "group-a",
        );
        record.event_id = "event-b".into();
        let mut hypotheses = hypotheses();
        hypothesis_mut(&mut hypotheses, cause).support_evidence_refs = vec!["other-event".into()];
        assert!(matches!(
            audit_cause_evidence(&case(hypotheses, vec![record])),
            Err(CauseEvidenceAuditError::EvidenceForDifferentEvent { .. })
        ));
    }

    #[test]
    fn missing_wrong_polarity_and_duplicate_references_fail_closed() {
        let cause = ChangeCause::ArchaeologicalExcavationLooting;
        let mut missing = hypotheses();
        hypothesis_mut(&mut missing, cause).support_evidence_refs = vec!["absent".into()];
        assert!(matches!(
            audit_cause_evidence(&case(missing, Vec::new())),
            Err(CauseEvidenceAuditError::EvidenceReferenceNotFound { .. })
        ));

        let record = evidence(
            "contradiction",
            cause,
            CauseEvidencePolarity::Contradicts,
            'a',
            "group-a",
        );
        let mut wrong = hypotheses();
        hypothesis_mut(&mut wrong, cause).support_evidence_refs = vec!["contradiction".into()];
        assert!(matches!(
            audit_cause_evidence(&case(wrong, vec![record.clone()])),
            Err(CauseEvidenceAuditError::EvidencePolarityMismatch { .. })
        ));

        let mut duplicate = hypotheses();
        hypothesis_mut(&mut duplicate, cause).support_evidence_refs =
            vec!["contradiction".into(), "contradiction".into()];
        assert!(matches!(
            audit_cause_evidence(&case(duplicate, vec![record])),
            Err(CauseEvidenceAuditError::DuplicateEvidenceReference { .. })
        ));
    }

    #[test]
    fn rights_receipts_and_source_digests_are_required_when_applicable() {
        let cause = ChangeCause::ArchaeologicalExcavationLooting;
        let mut no_receipt = evidence(
            "no-receipt",
            cause,
            CauseEvidencePolarity::Supports,
            'a',
            "group-a",
        );
        no_receipt.rights_receipt_ref = None;
        let mut hypotheses = hypotheses();
        hypothesis_mut(&mut hypotheses, cause).support_evidence_refs = vec!["no-receipt".into()];
        assert!(matches!(
            audit_cause_evidence(&case(hypotheses, vec![no_receipt])),
            Err(CauseEvidenceAuditError::EmptyRightsReceipt { .. })
        ));

        let mut bad_digest = evidence(
            "bad-digest",
            cause,
            CauseEvidencePolarity::Supports,
            'a',
            "group-a",
        );
        bad_digest.source_asset_sha256 = "not-a-sha256".into();
        let mut hypotheses = hypotheses();
        hypothesis_mut(&mut hypotheses, cause).support_evidence_refs = vec!["bad-digest".into()];
        assert!(matches!(
            audit_cause_evidence(&case(hypotheses, vec![bad_digest])),
            Err(CauseEvidenceAuditError::InvalidSourceDigest { .. })
        ));
    }

    #[test]
    fn same_source_used_for_support_and_contradiction_is_not_independent_on_either_side() {
        let cause = ChangeCause::ArchaeologicalExcavationLooting;
        let records = vec![
            evidence("supports", cause, CauseEvidencePolarity::Supports, 'a', "group-a"),
            evidence("contradicts", cause, CauseEvidencePolarity::Contradicts, 'a', "group-b"),
        ];
        let mut hypotheses = hypotheses();
        let claim = hypothesis_mut(&mut hypotheses, cause);
        claim.probability = 0.9;
        claim.support_evidence_refs = vec!["supports".into()];
        claim.contradiction_evidence_refs = vec!["contradicts".into()];

        let report = audit_cause_evidence(&case(hypotheses, records)).unwrap();
        let summary = report.hypothesis_summaries.iter().find(|item| item.cause == cause).unwrap();
        assert!(summary.contested);
        assert_eq!(summary.independent_support_count, 0);
        assert_eq!(summary.independent_contradiction_count, 0);
        assert_eq!(summary.cross_polarity_conflict_record_count, 2);
        assert_eq!(report.cross_polarity_conflict_record_count, 2);
    }

    #[test]
    fn same_independence_group_on_both_sides_is_not_independent_twice() {
        let cause = ChangeCause::ArchaeologicalExcavationLooting;
        let records = vec![
            evidence("supports", cause, CauseEvidencePolarity::Supports, 'a', "group-shared"),
            evidence("contradicts", cause, CauseEvidencePolarity::Contradicts, 'b', "group-shared"),
        ];
        let mut hypotheses = hypotheses();
        let claim = hypothesis_mut(&mut hypotheses, cause);
        claim.support_evidence_refs = vec!["supports".into()];
        claim.contradiction_evidence_refs = vec!["contradicts".into()];

        let report = audit_cause_evidence(&case(hypotheses, records)).unwrap();
        let summary = report.hypothesis_summaries.iter().find(|item| item.cause == cause).unwrap();
        assert_eq!(summary.independent_support_count, 0);
        assert_eq!(summary.independent_contradiction_count, 0);
        assert_eq!(summary.cross_polarity_conflict_record_count, 2);
    }

    #[test]
    fn non_detection_cannot_be_marked_as_supporting_evidence() {
        let cause = ChangeCause::ArchaeologicalExcavationLooting;
        let mut record = evidence("non-detection", cause, CauseEvidencePolarity::Supports, 'a', "group-a");
        record.is_non_detection = true;
        record.detectability_assessment_ref = Some("urn:synthetic:detectability".into());
        let mut hypotheses = hypotheses();
        hypothesis_mut(&mut hypotheses, cause).support_evidence_refs = vec!["non-detection".into()];
        assert!(matches!(
            audit_cause_evidence(&case(hypotheses, vec![record])),
            Err(CauseEvidenceAuditError::NonDetectionPolarityMustContradict { .. })
        ));
    }

    #[test]
    fn field_assessment_requires_reviewer_and_authority_references() {
        let cause = ChangeCause::ArchaeologicalExcavationLooting;
        let mut record = evidence(
            "no-reviewer",
            cause,
            CauseEvidencePolarity::Supports,
            'a',
            "group-a",
        );
        record.reviewer_ref = None;
        let mut hypotheses = hypotheses();
        hypothesis_mut(&mut hypotheses, cause).support_evidence_refs = vec!["no-reviewer".into()];
        assert!(matches!(
            audit_cause_evidence(&case(hypotheses, vec![record])),
            Err(CauseEvidenceAuditError::EmptyReviewerReference { .. })
        ));
    }
}
