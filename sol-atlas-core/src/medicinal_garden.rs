// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Evidence-bounded site-screening semantics for community garden planning.
//!
//! A complete checklist means only that scoped evidence references were supplied
//! for human review. This module does not inspect evidence bytes, interpret lab
//! results, determine legal permissions, authenticate plants, or establish
//! suitability for consumption, therapy, or medicinal manufacture.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const GARDEN_SITE_SCREENING_SCHEMA_VERSION: u16 = 1;

const REQUIRED_DIMENSIONS: [ScreeningDimension; 6] = [
    ScreeningDimension::SiteAndPermission,
    ScreeningDimension::WaterQuantityAndQuality,
    ScreeningDimension::SoilAndContamination,
    ScreeningDimension::ClimateAndGrowingConditions,
    ScreeningDimension::BiodiversityAndKnowledgeRights,
    ScreeningDimension::CommunityOperatingCapacity,
];

/// Declared non-clinical purpose. This enum intentionally has no therapeutic use.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HorticulturalPurpose {
    EducationAndCommunityHorticulture,
    ConservationNursery,
    FoodGarden,
}

/// Independent screening dimensions; they must never be collapsed into a score.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScreeningDimension {
    SiteAndPermission,
    WaterQuantityAndQuality,
    SoilAndContamination,
    ClimateAndGrowingConditions,
    BiodiversityAndKnowledgeRights,
    CommunityOperatingCapacity,
}

/// A per-dimension evidence state. Only Reviewed may satisfy checklist completeness.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScreeningEvidenceState {
    Reviewed,
    Unknown,
    Blocked,
    Stale,
    Conflicting,
    OutOfScope,
}

/// An assessment is bound to one site generation and one declared use.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SiteDimensionAssessmentV1 {
    pub dimension: ScreeningDimension,
    pub state: ScreeningEvidenceState,
    pub site_id: String,
    pub site_generation: String,
    pub purpose: HorticulturalPurpose,
    /// Opaque reference to externally maintained evidence; bytes are not verified here.
    pub evidence_ref: Option<String>,
    /// Opaque accountable reviewer reference, not proof of reviewer identity.
    pub reviewer_ref: Option<String>,
    /// UTC review timestamp in YYYY-MM-DDTHH:MM:SSZ form.
    pub reviewed_at_utc: Option<String>,
    pub note: Option<String>,
}

/// Screening input. Exact/private coordinates and health data are deliberately absent.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GardenSiteScreeningV1 {
    pub schema_version: u16,
    pub site_id: String,
    pub site_generation: String,
    pub agroclimate_profile_ref: String,
    pub geometry_snapshot_ref: String,
    pub intended_purpose: HorticulturalPurpose,
    pub assessments: Vec<SiteDimensionAssessmentV1>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScreeningBlockerCode {
    UnsupportedSchema,
    EmptySiteId,
    EmptySiteGeneration,
    MissingAgroclimateProfile,
    MissingGeometrySnapshot,
    MissingDimensionAssessment,
    DuplicateDimensionAssessment,
    AssessmentSiteMismatch,
    AssessmentGenerationMismatch,
    AssessmentPurposeMismatch,
    DimensionNotReviewed,
    EvidenceReferenceRequired,
    ReviewerReferenceRequired,
    ReviewTimestampRequired,
}

/// Stable, machine-readable blocker. The evaluator orders blockers deterministically.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScreeningBlockerV1 {
    pub dimension: Option<ScreeningDimension>,
    pub code: ScreeningBlockerCode,
    pub detail: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScreeningDisposition {
    /// Evidence references are structurally complete; accountable humans must still review.
    ChecklistCompleteForHumanReview,
    /// At least one required fact, evidence reference, or review gate is unresolved.
    Blocked,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScreeningDecisionV1 {
    pub site_id: String,
    pub site_generation: String,
    pub disposition: ScreeningDisposition,
    pub blockers: Vec<ScreeningBlockerV1>,
}

/// Validate the shape and scope of a screening checklist without asserting site safety.
pub fn evaluate_site_screening(input: &GardenSiteScreeningV1) -> ScreeningDecisionV1 {
    let mut blockers = Vec::new();

    if input.schema_version != GARDEN_SITE_SCREENING_SCHEMA_VERSION {
        blockers.push(blocker(
            None,
            ScreeningBlockerCode::UnsupportedSchema,
            "unsupported screening schema version",
        ));
    }
    if input.site_id.trim().is_empty() {
        blockers.push(blocker(
            None,
            ScreeningBlockerCode::EmptySiteId,
            "site ID is empty",
        ));
    }
    if input.site_generation.trim().is_empty() {
        blockers.push(blocker(
            None,
            ScreeningBlockerCode::EmptySiteGeneration,
            "site generation is empty",
        ));
    }
    if input.agroclimate_profile_ref.trim().is_empty() {
        blockers.push(blocker(
            None,
            ScreeningBlockerCode::MissingAgroclimateProfile,
            "parent agroclimate profile reference is missing",
        ));
    }
    if input.geometry_snapshot_ref.trim().is_empty() {
        blockers.push(blocker(
            None,
            ScreeningBlockerCode::MissingGeometrySnapshot,
            "immutable geometry snapshot reference is missing",
        ));
    }

    let mut seen = BTreeSet::new();
    for dimension in REQUIRED_DIMENSIONS {
        let matches: Vec<&SiteDimensionAssessmentV1> = input
            .assessments
            .iter()
            .filter(|assessment| assessment.dimension == dimension)
            .collect();

        if matches.is_empty() {
            blockers.push(blocker(
                Some(dimension),
                ScreeningBlockerCode::MissingDimensionAssessment,
                "required screening dimension is absent",
            ));
            continue;
        }
        if !seen.insert(dimension) || matches.len() > 1 {
            blockers.push(blocker(
                Some(dimension),
                ScreeningBlockerCode::DuplicateDimensionAssessment,
                "screening dimension must appear exactly once",
            ));
            continue;
        }

        let assessment = matches[0];
        if assessment.site_id != input.site_id {
            blockers.push(blocker(
                Some(dimension),
                ScreeningBlockerCode::AssessmentSiteMismatch,
                "assessment is bound to a different site",
            ));
        }
        if assessment.site_generation != input.site_generation {
            blockers.push(blocker(
                Some(dimension),
                ScreeningBlockerCode::AssessmentGenerationMismatch,
                "assessment is bound to a different site generation",
            ));
        }
        if assessment.purpose != input.intended_purpose {
            blockers.push(blocker(
                Some(dimension),
                ScreeningBlockerCode::AssessmentPurposeMismatch,
                "assessment purpose does not match the screening purpose",
            ));
        }
        if assessment.state != ScreeningEvidenceState::Reviewed {
            blockers.push(blocker(
                Some(dimension),
                ScreeningBlockerCode::DimensionNotReviewed,
                "dimension is not in the reviewed state",
            ));
            continue;
        }
        if !non_empty(assessment.evidence_ref.as_deref()) {
            blockers.push(blocker(
                Some(dimension),
                ScreeningBlockerCode::EvidenceReferenceRequired,
                "reviewed state requires an evidence reference",
            ));
        }
        if !non_empty(assessment.reviewer_ref.as_deref()) {
            blockers.push(blocker(
                Some(dimension),
                ScreeningBlockerCode::ReviewerReferenceRequired,
                "reviewed state requires an accountable reviewer reference",
            ));
        }
        if !valid_utc_timestamp_shape(assessment.reviewed_at_utc.as_deref()) {
            blockers.push(blocker(
                Some(dimension),
                ScreeningBlockerCode::ReviewTimestampRequired,
                "reviewed state requires a UTC timestamp in YYYY-MM-DDTHH:MM:SSZ form",
            ));
        }
    }

    let disposition = if blockers.is_empty() {
        ScreeningDisposition::ChecklistCompleteForHumanReview
    } else {
        ScreeningDisposition::Blocked
    };

    ScreeningDecisionV1 {
        site_id: input.site_id.clone(),
        site_generation: input.site_generation.clone(),
        disposition,
        blockers,
    }
}

fn blocker(
    dimension: Option<ScreeningDimension>,
    code: ScreeningBlockerCode,
    detail: &str,
) -> ScreeningBlockerV1 {
    ScreeningBlockerV1 {
        dimension,
        code,
        detail: detail.to_owned(),
    }
}

fn non_empty(value: Option<&str>) -> bool {
    value.is_some_and(|text| !text.trim().is_empty())
}

/// Shape check only; this does not establish that the timestamp is a real calendar instant.
fn valid_utc_timestamp_shape(value: Option<&str>) -> bool {
    let Some(value) = value else {
        return false;
    };
    let bytes = value.as_bytes();
    bytes.len() == 20
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes[10] == b'T'
        && bytes[13] == b':'
        && bytes[16] == b':'
        && bytes[19] == b'Z'
        && bytes.iter().enumerate().all(|(index, byte)| {
            matches!(index, 4 | 7 | 10 | 13 | 16 | 19) || byte.is_ascii_digit()
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn approved_assessment(dimension: ScreeningDimension) -> SiteDimensionAssessmentV1 {
        SiteDimensionAssessmentV1 {
            dimension,
            state: ScreeningEvidenceState::Reviewed,
            site_id: "SITE-001".to_owned(),
            site_generation: "GEN-1".to_owned(),
            purpose: HorticulturalPurpose::EducationAndCommunityHorticulture,
            evidence_ref: Some(format!("evidence-{:?}", dimension)),
            reviewer_ref: Some("reviewer-001".to_owned()),
            reviewed_at_utc: Some("2026-10-15T07:00:00Z".to_owned()),
            note: None,
        }
    }

    fn valid_input() -> GardenSiteScreeningV1 {
        GardenSiteScreeningV1 {
            schema_version: GARDEN_SITE_SCREENING_SCHEMA_VERSION,
            site_id: "SITE-001".to_owned(),
            site_generation: "GEN-1".to_owned(),
            agroclimate_profile_ref: "agroclimate-johannesburg-highveld-v1".to_owned(),
            geometry_snapshot_ref: "sha256:example-geometry-snapshot".to_owned(),
            intended_purpose: HorticulturalPurpose::EducationAndCommunityHorticulture,
            assessments: REQUIRED_DIMENSIONS
                .into_iter()
                .map(approved_assessment)
                .collect(),
        }
    }

    #[test]
    fn complete_scoped_checklist_is_only_ready_for_human_review() {
        let decision = evaluate_site_screening(&valid_input());
        assert_eq!(
            decision.disposition,
            ScreeningDisposition::ChecklistCompleteForHumanReview
        );
        assert!(decision.blockers.is_empty());
    }

    #[test]
    fn missing_dimension_fails_closed() {
        let mut input = valid_input();
        input.assessments.pop();
        let decision = evaluate_site_screening(&input);
        assert_eq!(decision.disposition, ScreeningDisposition::Blocked);
        assert!(
            decision
                .blockers
                .iter()
                .any(|item| { item.code == ScreeningBlockerCode::MissingDimensionAssessment })
        );
    }

    #[test]
    fn duplicate_dimension_fails_closed() {
        let mut input = valid_input();
        input.assessments.push(input.assessments[0].clone());
        let decision = evaluate_site_screening(&input);
        assert!(
            decision
                .blockers
                .iter()
                .any(|item| { item.code == ScreeningBlockerCode::DuplicateDimensionAssessment })
        );
    }

    #[test]
    fn stale_soil_evidence_cannot_be_offset_by_other_dimensions() {
        let mut input = valid_input();
        let soil = input
            .assessments
            .iter_mut()
            .find(|assessment| assessment.dimension == ScreeningDimension::SoilAndContamination)
            .expect("soil dimension exists");
        soil.state = ScreeningEvidenceState::Stale;
        let decision = evaluate_site_screening(&input);
        assert_eq!(decision.disposition, ScreeningDisposition::Blocked);
        assert!(decision.blockers.iter().any(|item| {
            item.dimension == Some(ScreeningDimension::SoilAndContamination)
                && item.code == ScreeningBlockerCode::DimensionNotReviewed
        }));
    }

    #[test]
    fn evidence_from_another_site_generation_is_rejected() {
        let mut input = valid_input();
        input.assessments[0].site_generation = "GEN-0".to_owned();
        let decision = evaluate_site_screening(&input);
        assert!(
            decision
                .blockers
                .iter()
                .any(|item| { item.code == ScreeningBlockerCode::AssessmentGenerationMismatch })
        );
    }

    #[test]
    fn wrong_intended_purpose_is_rejected() {
        let mut input = valid_input();
        input.assessments[0].purpose = HorticulturalPurpose::ConservationNursery;
        let decision = evaluate_site_screening(&input);
        assert!(
            decision
                .blockers
                .iter()
                .any(|item| { item.code == ScreeningBlockerCode::AssessmentPurposeMismatch })
        );
    }

    #[test]
    fn reviewed_state_requires_reviewer_and_evidence() {
        let mut input = valid_input();
        input.assessments[0].reviewer_ref = None;
        input.assessments[1].evidence_ref = Some(" ".to_owned());
        let decision = evaluate_site_screening(&input);
        assert!(
            decision
                .blockers
                .iter()
                .any(|item| { item.code == ScreeningBlockerCode::ReviewerReferenceRequired })
        );
        assert!(
            decision
                .blockers
                .iter()
                .any(|item| { item.code == ScreeningBlockerCode::EvidenceReferenceRequired })
        );
    }

    #[test]
    fn timestamp_shape_is_checked_without_claiming_calendar_validity() {
        let mut input = valid_input();
        input.assessments[0].reviewed_at_utc = Some("not-a-timestamp".to_owned());
        let decision = evaluate_site_screening(&input);
        assert!(
            decision
                .blockers
                .iter()
                .any(|item| { item.code == ScreeningBlockerCode::ReviewTimestampRequired })
        );
    }

    #[test]
    fn no_medicine_safe_state_exists_in_the_wire_contract() {
        let serialized = serde_json::to_string(&valid_input()).expect("fixture serializes");
        assert!(!serialized.contains("medically_safe"));
        assert!(!serialized.contains("treatment_approved"));
        assert!(serialized.contains("education_and_community_horticulture"));
    }
}
