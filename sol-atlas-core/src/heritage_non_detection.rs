// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Survival-, exposure-, observability-, coverage- and detectability-aware
//! interpretation of one archaeological non-detection.
//!
//! A non-detection is evidence only under an explicit observation state and
//! context-matched probability chain. This module supports one survey event;
//! it never multiplies repeated visits as if they were independent.
//!
//! The evaluator validates supplied probability bands and provenance receipts.
//! It does not fit calibration models, authenticate receipts or source bytes,
//! perform GIS coverage overlays, establish archaeological presence/absence,
//! or authorise excavation or other field actions.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fmt;

pub const HERITAGE_NONDETECTION_SCHEMA_VERSION: &str =
    "luminous.heritage-nondetection.v0.1";
pub const MAX_STAGE_EVIDENCE: usize = 6;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProbabilityBand {
    pub lower: f64,
    pub upper: f64,
}

impl ProbabilityBand {
    fn validate(self, label: &str) -> Result<(), HeritageNonDetectionError> {
        if !self.lower.is_finite()
            || !self.upper.is_finite()
            || self.lower < 0.0
            || self.upper > 1.0
            || self.lower > self.upper
        {
            return Err(HeritageNonDetectionError::InvalidProbabilityBand(
                label.into(),
            ));
        }
        Ok(())
    }

    fn complement(self) -> Self {
        Self {
            lower: 1.0 - self.upper,
            upper: 1.0 - self.lower,
        }
    }

    fn multiply(self, other: Self) -> Self {
        Self {
            lower: self.lower * other.lower,
            upper: self.upper * other.upper,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObservationState {
    Detected,
    SearchedNoDetection,
    NotSearched,
    NotObservableOrInaccessible,
    SurveyIncompleteOrInconclusive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceStage {
    Survival,
    Exposure,
    Observability,
    FootprintCoverage,
    ConditionalDetection,
    FalsePositive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Conditioning {
    OriginallyPresent,
    SurvivedGivenOriginallyPresent,
    ExposedGivenSurvival,
    ObservableGivenSurvivalAndExposure,
    CoveredGivenSurvivalExposureAndObservability,
    DetectedGivenSurvivalExposureObservabilityAndCoverage,
    FalsePositiveGivenOriginallyAbsent,
}

impl EvidenceStage {
    fn required_conditioning(self) -> Conditioning {
        match self {
            Self::Survival => Conditioning::OriginallyPresent,
            Self::Exposure => Conditioning::SurvivedGivenOriginallyPresent,
            Self::Observability => Conditioning::ObservableGivenSurvivalAndExposure,
            Self::FootprintCoverage => {
                Conditioning::CoveredGivenSurvivalExposureAndObservability
            }
            Self::ConditionalDetection => {
                Conditioning::DetectedGivenSurvivalExposureObservabilityAndCoverage
            }
            Self::FalsePositive => Conditioning::FalsePositiveGivenOriginallyAbsent,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnalyticalRights {
    Allowed,
    Denied,
    Undetermined,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccessTier {
    Public,
    Controlled,
    Restricted,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoverageEvidenceBasis {
    ActualSpatialUnionReceipt,
    PlannedSurveyMetadata,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PriorEvidence {
    pub target_class_id: String,
    pub context_id: String,
    pub probability: ProbabilityBand,
    pub prior_profile_sha256: String,
    pub source_data_sha256: String,
    pub validation_split_manifest_sha256: String,
    pub validation_receipt_ref: String,
    pub validation_receipt_sha256: String,
    pub rights_decision_id: String,
    pub analytical_rights: AnalyticalRights,
    pub rights_receipt_ref: String,
    pub access_tier: AccessTier,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StageEvidence {
    pub stage: EvidenceStage,
    pub probability: ProbabilityBand,
    pub target_class_id: String,
    pub context_id: String,
    pub conditioning: Conditioning,
    pub method_id: String,
    pub profile_sha256: String,
    pub method_build_sha256: String,
    pub calibration_data_sha256: String,
    pub validation_split_manifest_sha256: String,
    pub validation_receipt_ref: String,
    pub validation_receipt_sha256: String,
    pub source_data_sha256: String,
    pub analytical_rights: AnalyticalRights,
    pub rights_receipt_ref: String,
    pub access_tier: AccessTier,
    /// Present only for footprint-coverage evidence.
    pub coverage_evidence_basis: Option<CoverageEvidenceBasis>,
    /// Digest of actual survey-unit / transect-union geometry evidence.
    pub coverage_geometry_sha256: Option<String>,
    /// Digest of the GIS-side spatial-union and clipping receipt.
    pub spatial_coverage_receipt_sha256: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HeritageNonDetectionInput {
    pub schema_version: String,
    pub event_id: String,
    pub target_class_id: String,
    pub context_id: String,
    pub observation_state: ObservationState,
    pub prior: PriorEvidence,
    /// Exactly one receipt per stage is required only for a searched miss.
    /// Other observation states do not trigger a negative-evidence update.
    pub stage_evidence: Vec<StageEvidence>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum NonDetectionDisposition {
    QualifiedNegativeEvidenceUpdate,
    DetectedNoNegativeUpdate,
    NotSearchedNoUpdate,
    NotObservableNoUpdate,
    SurveyIncompleteNoUpdate,
    PosteriorUndefinedZeroLikelihood,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StageEvidenceReference {
    pub stage: EvidenceStage,
    pub profile_sha256: String,
    pub validation_receipt_ref: String,
    pub validation_receipt_sha256: String,
    pub rights_receipt_ref: String,
    pub access_tier: AccessTier,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HeritageNonDetectionReport {
    pub schema_version: String,
    pub event_id: String,
    pub target_class_id: String,
    pub context_id: String,
    pub observation_state: ObservationState,
    pub disposition: NonDetectionDisposition,
    pub prior_probability: ProbabilityBand,
    pub probability_of_detection_if_originally_present: Option<ProbabilityBand>,
    pub probability_of_no_detection_if_originally_present: Option<ProbabilityBand>,
    pub false_positive_probability_if_originally_absent: Option<ProbabilityBand>,
    pub probability_of_no_detection_if_originally_absent: Option<ProbabilityBand>,
    /// Conservative rectangular-bound posterior. None means no negative
    /// update applied or the observed outcome has zero model likelihood.
    pub posterior_probability_originally_present: Option<ProbabilityBand>,
    pub stage_evidence_references: Vec<StageEvidenceReference>,
    pub stage_count: usize,
    /// This evaluator handles one survey event only.
    pub repeated_visit_independence_assumed: bool,
    /// A posterior is conditional evidence, not an archaeological verdict.
    pub archaeological_absence_claimed: bool,
    /// A non-detection update never claims that an archaeological target exists.
    pub archaeological_identification_claimed: bool,
    /// No construction, excavation, publication or field-action authority.
    pub action_authority: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeritageNonDetectionError {
    WrongSchemaVersion,
    EmptyEventId,
    EmptyTargetClassId,
    EmptyContextId,
    PriorTargetClassMismatch,
    PriorContextMismatch,
    InvalidProbabilityBand(String),
    InvalidDigest(String),
    EmptyEvidenceReference(String),
    AnalyticalRightsNotAllowed(String),
    RightsReceiptMissing(String),
    TooManyStageEvidence,
    MissingStage(EvidenceStage),
    DuplicateStage(EvidenceStage),
    StageTargetClassMismatch(EvidenceStage),
    StageContextMismatch(EvidenceStage),
    ConditioningMismatch(EvidenceStage),
    EmptyMethodId(EvidenceStage),
    CoverageBasisNotActual,
    CoverageGeometryDigestMissing,
    SpatialCoverageReceiptDigestMissing,
    CoverageFieldsUnexpected(EvidenceStage),
}

impl fmt::Display for HeritageNonDetectionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "heritage non-detection error: {self:?}")
    }
}

impl std::error::Error for HeritageNonDetectionError {}

fn non_empty(value: &str) -> bool {
    !value.trim().is_empty()
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn validate_digest(value: &str, label: &str) -> Result<(), HeritageNonDetectionError> {
    if !valid_sha256(value) {
        return Err(HeritageNonDetectionError::InvalidDigest(label.into()));
    }
    Ok(())
}

fn validate_prior(
    input: &HeritageNonDetectionInput,
) -> Result<(), HeritageNonDetectionError> {
    if !non_empty(&input.event_id) {
        return Err(HeritageNonDetectionError::EmptyEventId);
    }
    if !non_empty(&input.target_class_id) {
        return Err(HeritageNonDetectionError::EmptyTargetClassId);
    }
    if !non_empty(&input.context_id) {
        return Err(HeritageNonDetectionError::EmptyContextId);
    }
    if input.prior.target_class_id != input.target_class_id {
        return Err(HeritageNonDetectionError::PriorTargetClassMismatch);
    }
    if input.prior.context_id != input.context_id {
        return Err(HeritageNonDetectionError::PriorContextMismatch);
    }
    input.prior.probability.validate("prior_probability")?;
    for (value, label) in [
        (&input.prior.prior_profile_sha256, "prior_profile_sha256"),
        (&input.prior.source_data_sha256, "prior_source_data_sha256"),
        (
            &input.prior.validation_split_manifest_sha256,
            "prior_validation_split_manifest_sha256",
        ),
        (
            &input.prior.validation_receipt_sha256,
            "prior_validation_receipt_sha256",
        ),
    ] {
        validate_digest(value, label)?;
    }
    if !non_empty(&input.prior.validation_receipt_ref)
        || !non_empty(&input.prior.rights_decision_id)
    {
        return Err(HeritageNonDetectionError::EmptyEvidenceReference(
            "prior_evidence".into(),
        ));
    }
    if input.prior.analytical_rights != AnalyticalRights::Allowed {
        return Err(HeritageNonDetectionError::AnalyticalRightsNotAllowed(
            "prior_evidence".into(),
        ));
    }
    if !non_empty(&input.prior.rights_receipt_ref) {
        return Err(HeritageNonDetectionError::RightsReceiptMissing(
            "prior_evidence".into(),
        ));
    }
    Ok(())
}

fn validate_stage(
    stage: &StageEvidence,
    target_class_id: &str,
    context_id: &str,
) -> Result<(), HeritageNonDetectionError> {
    stage
        .probability
        .validate(&format!("{:?}", stage.stage))?;
    if stage.target_class_id != target_class_id {
        return Err(HeritageNonDetectionError::StageTargetClassMismatch(
            stage.stage,
        ));
    }
    if stage.context_id != context_id {
        return Err(HeritageNonDetectionError::StageContextMismatch(
            stage.stage,
        ));
    }
    if stage.conditioning != stage.stage.required_conditioning() {
        return Err(HeritageNonDetectionError::ConditioningMismatch(
            stage.stage,
        ));
    }
    if !non_empty(&stage.method_id) {
        return Err(HeritageNonDetectionError::EmptyMethodId(stage.stage));
    }
    for (value, label) in [
        (&stage.profile_sha256, "profile_sha256"),
        (&stage.method_build_sha256, "method_build_sha256"),
        (&stage.calibration_data_sha256, "calibration_data_sha256"),
        (
            &stage.validation_split_manifest_sha256,
            "validation_split_manifest_sha256",
        ),
        (
            &stage.validation_receipt_sha256,
            "validation_receipt_sha256",
        ),
        (&stage.source_data_sha256, "source_data_sha256"),
    ] {
        validate_digest(value, label)?;
    }
    if !non_empty(&stage.validation_receipt_ref) {
        return Err(HeritageNonDetectionError::EmptyEvidenceReference(
            format!("{:?}", stage.stage),
        ));
    }
    if stage.analytical_rights != AnalyticalRights::Allowed {
        return Err(HeritageNonDetectionError::AnalyticalRightsNotAllowed(
            format!("{:?}", stage.stage),
        ));
    }
    if !non_empty(&stage.rights_receipt_ref) {
        return Err(HeritageNonDetectionError::RightsReceiptMissing(
            format!("{:?}", stage.stage),
        ));
    }

    match stage.stage {
        EvidenceStage::FootprintCoverage => {
            if stage.coverage_evidence_basis
                != Some(CoverageEvidenceBasis::ActualSpatialUnionReceipt)
            {
                return Err(HeritageNonDetectionError::CoverageBasisNotActual);
            }
            let geometry_digest = stage
                .coverage_geometry_sha256
                .as_deref()
                .ok_or(HeritageNonDetectionError::CoverageGeometryDigestMissing)?;
            validate_digest(geometry_digest, "coverage_geometry_sha256")?;
            let receipt_digest = stage
                .spatial_coverage_receipt_sha256
                .as_deref()
                .ok_or(HeritageNonDetectionError::SpatialCoverageReceiptDigestMissing)?;
            validate_digest(receipt_digest, "spatial_coverage_receipt_sha256")?;
        }
        _ => {
            if stage.coverage_evidence_basis.is_some()
                || stage.coverage_geometry_sha256.is_some()
                || stage.spatial_coverage_receipt_sha256.is_some()
            {
                return Err(HeritageNonDetectionError::CoverageFieldsUnexpected(
                    stage.stage,
                ));
            }
        }
    }
    Ok(())
}

fn validate_stages(
    input: &HeritageNonDetectionInput,
) -> Result<BTreeMap<EvidenceStage, StageEvidence>, HeritageNonDetectionError> {
    if input.stage_evidence.len() > MAX_STAGE_EVIDENCE {
        return Err(HeritageNonDetectionError::TooManyStageEvidence);
    }
    let mut stages = BTreeMap::new();
    for stage in &input.stage_evidence {
        validate_stage(stage, &input.target_class_id, &input.context_id)?;
        if stages.insert(stage.stage, stage.clone()).is_some() {
            return Err(HeritageNonDetectionError::DuplicateStage(stage.stage));
        }
    }
    for stage in [
        EvidenceStage::Survival,
        EvidenceStage::Exposure,
        EvidenceStage::Observability,
        EvidenceStage::FootprintCoverage,
        EvidenceStage::ConditionalDetection,
        EvidenceStage::FalsePositive,
    ] {
        if !stages.contains_key(&stage) {
            return Err(HeritageNonDetectionError::MissingStage(stage));
        }
    }
    Ok(stages)
}

fn posterior_probability(
    prior: f64,
    no_detection_if_present: f64,
    no_detection_if_absent: f64,
) -> Option<f64> {
    let present_likelihood = prior * no_detection_if_present;
    let absent_likelihood = (1.0 - prior) * no_detection_if_absent;
    let denominator = present_likelihood + absent_likelihood;
    if !denominator.is_finite() || denominator <= 0.0 {
        None
    } else {
        Some((present_likelihood / denominator).clamp(0.0, 1.0))
    }
}

fn make_report(
    input: &HeritageNonDetectionInput,
    disposition: NonDetectionDisposition,
    detection_if_present: Option<ProbabilityBand>,
    no_detection_if_present: Option<ProbabilityBand>,
    false_positive_if_absent: Option<ProbabilityBand>,
    no_detection_if_absent: Option<ProbabilityBand>,
    posterior: Option<ProbabilityBand>,
    references: Vec<StageEvidenceReference>,
) -> HeritageNonDetectionReport {
    HeritageNonDetectionReport {
        schema_version: HERITAGE_NONDETECTION_SCHEMA_VERSION.into(),
        event_id: input.event_id.clone(),
        target_class_id: input.target_class_id.clone(),
        context_id: input.context_id.clone(),
        observation_state: input.observation_state,
        disposition,
        prior_probability: input.prior.probability,
        probability_of_detection_if_originally_present: detection_if_present,
        probability_of_no_detection_if_originally_present: no_detection_if_present,
        false_positive_probability_if_originally_absent: false_positive_if_absent,
        probability_of_no_detection_if_originally_absent: no_detection_if_absent,
        posterior_probability_originally_present: posterior,
        stage_count: references.len(),
        stage_evidence_references: references,
        repeated_visit_independence_assumed: false,
        archaeological_absence_claimed: false,
        archaeological_identification_claimed: false,
        action_authority: "none".into(),
    }
}

/// Evaluates one observation. Only SearchedNoDetection can produce a negative
/// evidence posterior update. Other observation states preserve the prior and
/// never pretend that unsearched or unobservable territory is archaeological
/// absence.
pub fn evaluate_non_detection(
    input: &HeritageNonDetectionInput,
) -> Result<HeritageNonDetectionReport, HeritageNonDetectionError> {
    if input.schema_version != HERITAGE_NONDETECTION_SCHEMA_VERSION {
        return Err(HeritageNonDetectionError::WrongSchemaVersion);
    }
    validate_prior(input)?;

    let no_update = |disposition| {
        make_report(
            input,
            disposition,
            None,
            None,
            None,
            None,
            None,
            Vec::new(),
        )
    };

    match input.observation_state {
        ObservationState::Detected => {
            return Ok(no_update(NonDetectionDisposition::DetectedNoNegativeUpdate));
        }
        ObservationState::NotSearched => {
            return Ok(no_update(NonDetectionDisposition::NotSearchedNoUpdate));
        }
        ObservationState::NotObservableOrInaccessible => {
            return Ok(no_update(NonDetectionDisposition::NotObservableNoUpdate));
        }
        ObservationState::SurveyIncompleteOrInconclusive => {
            return Ok(no_update(NonDetectionDisposition::SurveyIncompleteNoUpdate));
        }
        ObservationState::SearchedNoDetection => {}
    }

    let stages = validate_stages(input)?;
    let survival = stages[&EvidenceStage::Survival].probability;
    let exposure = stages[&EvidenceStage::Exposure].probability;
    let observability = stages[&EvidenceStage::Observability].probability;
    let coverage = stages[&EvidenceStage::FootprintCoverage].probability;
    let detection = stages[&EvidenceStage::ConditionalDetection].probability;
    let false_positive = stages[&EvidenceStage::FalsePositive].probability;

    // The conditional probability chain uses the chain rule; it does not
    // assert mutual independence between the latent events.
    let detection_if_present = survival
        .multiply(exposure)
        .multiply(observability)
        .multiply(coverage)
        .multiply(detection);
    let no_detection_if_present = detection_if_present.complement();
    let no_detection_if_absent = false_positive.complement();

    // The posterior is monotone increasing in prior and miss-if-present,
    // and decreasing in miss-if-absent. Rectangular endpoint propagation
    // yields conservative bounds under the declared input ranges.
    let posterior_lower = posterior_probability(
        input.prior.probability.lower,
        no_detection_if_present.lower,
        no_detection_if_absent.upper,
    );
    let posterior_upper = posterior_probability(
        input.prior.probability.upper,
        no_detection_if_present.upper,
        no_detection_if_absent.lower,
    );

    let references: Vec<StageEvidenceReference> = stages
        .values()
        .map(|stage| StageEvidenceReference {
            stage: stage.stage,
            profile_sha256: stage.profile_sha256.clone(),
            validation_receipt_ref: stage.validation_receipt_ref.clone(),
            validation_receipt_sha256: stage.validation_receipt_sha256.clone(),
            rights_receipt_ref: stage.rights_receipt_ref.clone(),
            access_tier: stage.access_tier,
        })
        .collect();

    match (posterior_lower, posterior_upper) {
        (Some(lower), Some(upper)) if lower <= upper => Ok(make_report(
            input,
            NonDetectionDisposition::QualifiedNegativeEvidenceUpdate,
            Some(detection_if_present),
            Some(no_detection_if_present),
            Some(false_positive),
            Some(no_detection_if_absent),
            Some(ProbabilityBand { lower, upper }),
            references,
        )),
        _ => Ok(make_report(
            input,
            NonDetectionDisposition::PosteriorUndefinedZeroLikelihood,
            Some(detection_if_present),
            Some(no_detection_if_present),
            Some(false_positive),
            Some(no_detection_if_absent),
            None,
            references,
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digest(byte: char) -> String {
        std::iter::repeat(byte).take(64).collect()
    }

    fn prior() -> PriorEvidence {
        PriorEvidence {
            target_class_id: "dry_stone_wall".into(),
            context_id: "site-context-1".into(),
            probability: ProbabilityBand {
                lower: 0.6,
                upper: 0.6,
            },
            prior_profile_sha256: digest('a'),
            source_data_sha256: digest('b'),
            validation_split_manifest_sha256: digest('c'),
            validation_receipt_ref: "receipt:prior".into(),
            validation_receipt_sha256: digest('d'),
            rights_decision_id: "rights:prior".into(),
            analytical_rights: AnalyticalRights::Allowed,
            rights_receipt_ref: "receipt:rights-prior".into(),
            access_tier: AccessTier::Controlled,
        }
    }

    fn stage(stage: EvidenceStage, probability: ProbabilityBand) -> StageEvidence {
        let conditioning = stage.required_conditioning();
        let is_coverage = stage == EvidenceStage::FootprintCoverage;
        StageEvidence {
            stage,
            probability,
            target_class_id: "dry_stone_wall".into(),
            context_id: "site-context-1".into(),
            conditioning,
            method_id: format!("method:{stage:?}"),
            profile_sha256: digest('e'),
            method_build_sha256: digest('f'),
            calibration_data_sha256: digest('1'),
            validation_split_manifest_sha256: digest('2'),
            validation_receipt_ref: format!("receipt:{stage:?}"),
            validation_receipt_sha256: digest('3'),
            source_data_sha256: digest('4'),
            analytical_rights: AnalyticalRights::Allowed,
            rights_receipt_ref: format!("rights:{stage:?}"),
            access_tier: AccessTier::Controlled,
            coverage_evidence_basis: is_coverage
                .then_some(CoverageEvidenceBasis::ActualSpatialUnionReceipt),
            coverage_geometry_sha256: is_coverage.then(|| digest('5')),
            spatial_coverage_receipt_sha256: is_coverage.then(|| digest('6')),
        }
    }

    fn valid_input() -> HeritageNonDetectionInput {
        HeritageNonDetectionInput {
            schema_version: HERITAGE_NONDETECTION_SCHEMA_VERSION.into(),
            event_id: "survey-event-1".into(),
            target_class_id: "dry_stone_wall".into(),
            context_id: "site-context-1".into(),
            observation_state: ObservationState::SearchedNoDetection,
            prior: prior(),
            stage_evidence: vec![
                stage(
                    EvidenceStage::Survival,
                    ProbabilityBand {
                        lower: 0.9,
                        upper: 0.9,
                    },
                ),
                stage(
                    EvidenceStage::Exposure,
                    ProbabilityBand {
                        lower: 0.8,
                        upper: 0.8,
                    },
                ),
                stage(
                    EvidenceStage::Observability,
                    ProbabilityBand {
                        lower: 0.7,
                        upper: 0.7,
                    },
                ),
                stage(
                    EvidenceStage::FootprintCoverage,
                    ProbabilityBand {
                        lower: 0.5,
                        upper: 0.5,
                    },
                ),
                stage(
                    EvidenceStage::ConditionalDetection,
                    ProbabilityBand {
                        lower: 0.6,
                        upper: 0.6,
                    },
                ),
                stage(
                    EvidenceStage::FalsePositive,
                    ProbabilityBand {
                        lower: 0.1,
                        upper: 0.1,
                    },
                ),
            ],
        }
    }

    #[test]
    fn one_event_chain_rule_and_bayes_update_match_reference_arithmetic() {
        let report = evaluate_non_detection(&valid_input()).unwrap();
        assert_eq!(
            report.disposition,
            NonDetectionDisposition::QualifiedNegativeEvidenceUpdate
        );
        let detection = 0.9_f64 * 0.8 * 0.7 * 0.5 * 0.6;
        let miss_present = 1.0 - detection;
        let miss_absent = 0.9;
        let expected =
            (0.6 * miss_present) / (0.6 * miss_present + 0.4 * miss_absent);
        let posterior = report.posterior_probability_originally_present.unwrap();
        assert!((posterior.lower - expected).abs() < 1e-12);
        assert!((posterior.upper - expected).abs() < 1e-12);
        assert!(!report.repeated_visit_independence_assumed);
        assert!(!report.archaeological_absence_claimed);
        assert!(!report.archaeological_identification_claimed);
        assert_eq!(report.action_authority, "none");
    }

    #[test]
    fn lower_and_upper_probability_bands_propagate_conservatively() {
        let mut input = valid_input();
        input.prior.probability = ProbabilityBand {
            lower: 0.5,
            upper: 0.7,
        };
        input.stage_evidence[0].probability = ProbabilityBand {
            lower: 0.8,
            upper: 0.95,
        };
        input.stage_evidence[5].probability = ProbabilityBand {
            lower: 0.02,
            upper: 0.15,
        };
        let report = evaluate_non_detection(&input).unwrap();
        let posterior = report.posterior_probability_originally_present.unwrap();
        assert!(posterior.lower <= posterior.upper);
        assert!(posterior.lower >= 0.0);
        assert!(posterior.upper <= 1.0);
    }

    #[test]
    fn planned_transect_metadata_cannot_stand_in_for_actual_coverage() {
        let mut input = valid_input();
        input.stage_evidence[3].coverage_evidence_basis =
            Some(CoverageEvidenceBasis::PlannedSurveyMetadata);
        assert_eq!(
            evaluate_non_detection(&input).unwrap_err(),
            HeritageNonDetectionError::CoverageBasisNotActual
        );
    }

    #[test]
    fn missing_coverage_geometry_receipt_fails_closed() {
        let mut input = valid_input();
        input.stage_evidence[3].spatial_coverage_receipt_sha256 = None;
        assert_eq!(
            evaluate_non_detection(&input).unwrap_err(),
            HeritageNonDetectionError::SpatialCoverageReceiptDigestMissing
        );
    }

    #[test]
    fn not_searched_preserves_prior_without_negative_update() {
        let mut input = valid_input();
        input.observation_state = ObservationState::NotSearched;
        input.stage_evidence.clear();
        let report = evaluate_non_detection(&input).unwrap();
        assert_eq!(report.disposition, NonDetectionDisposition::NotSearchedNoUpdate);
        assert_eq!(report.posterior_probability_originally_present, None);
        assert_eq!(report.prior_probability, input.prior.probability);
        assert!(report.stage_evidence_references.is_empty());
    }

    #[test]
    fn not_observable_preserves_prior_without_negative_update() {
        let mut input = valid_input();
        input.observation_state = ObservationState::NotObservableOrInaccessible;
        input.stage_evidence.clear();
        let report = evaluate_non_detection(&input).unwrap();
        assert_eq!(report.disposition, NonDetectionDisposition::NotObservableNoUpdate);
        assert_eq!(report.posterior_probability_originally_present, None);
    }

    #[test]
    fn incomplete_survey_preserves_prior_without_negative_update() {
        let mut input = valid_input();
        input.observation_state = ObservationState::SurveyIncompleteOrInconclusive;
        input.stage_evidence.clear();
        let report = evaluate_non_detection(&input).unwrap();
        assert_eq!(report.disposition, NonDetectionDisposition::SurveyIncompleteNoUpdate);
        assert_eq!(report.posterior_probability_originally_present, None);
    }

    #[test]
    fn a_detection_is_not_misused_as_negative_evidence_or_archaeological_identification() {
        let mut input = valid_input();
        input.observation_state = ObservationState::Detected;
        input.stage_evidence.clear();
        let report = evaluate_non_detection(&input).unwrap();
        assert_eq!(report.disposition, NonDetectionDisposition::DetectedNoNegativeUpdate);
        assert_eq!(report.posterior_probability_originally_present, None);
        assert!(!report.archaeological_identification_claimed);
    }

    #[test]
    fn missing_required_stage_fails_closed() {
        let mut input = valid_input();
        input
            .stage_evidence
            .retain(|stage| stage.stage != EvidenceStage::Exposure);
        assert_eq!(
            evaluate_non_detection(&input).unwrap_err(),
            HeritageNonDetectionError::MissingStage(EvidenceStage::Exposure)
        );
    }

    #[test]
    fn duplicate_stage_fails_closed() {
        let mut input = valid_input();
        input.stage_evidence.push(input.stage_evidence[0].clone());
        assert_eq!(
            evaluate_non_detection(&input).unwrap_err(),
            HeritageNonDetectionError::DuplicateStage(EvidenceStage::Survival)
        );
    }

    #[test]
    fn conditional_stage_semantics_cannot_be_relabelled() {
        let mut input = valid_input();
        input.stage_evidence[1].conditioning = Conditioning::OriginallyPresent;
        assert_eq!(
            evaluate_non_detection(&input).unwrap_err(),
            HeritageNonDetectionError::ConditioningMismatch(EvidenceStage::Exposure)
        );
    }

    #[test]
    fn stage_context_mismatch_fails_closed() {
        let mut input = valid_input();
        input.stage_evidence[2].context_id = "different-season".into();
        assert_eq!(
            evaluate_non_detection(&input).unwrap_err(),
            HeritageNonDetectionError::StageContextMismatch(EvidenceStage::Observability)
        );
    }

    #[test]
    fn rights_unknown_or_denied_fail_closed() {
        let mut input = valid_input();
        input.stage_evidence[0].analytical_rights = AnalyticalRights::Undetermined;
        assert!(matches!(
            evaluate_non_detection(&input),
            Err(HeritageNonDetectionError::AnalyticalRightsNotAllowed(_))
        ));
        input = valid_input();
        input.prior.analytical_rights = AnalyticalRights::Denied;
        assert!(matches!(
            evaluate_non_detection(&input),
            Err(HeritageNonDetectionError::AnalyticalRightsNotAllowed(_))
        ));
    }

    #[test]
    fn invalid_digest_fails_closed() {
        let mut input = valid_input();
        input.stage_evidence[0].profile_sha256 = "not-a-digest".into();
        assert!(matches!(
            evaluate_non_detection(&input),
            Err(HeritageNonDetectionError::InvalidDigest(_))
        ));
    }

    #[test]
    fn stage_probability_outside_unit_interval_fails_closed() {
        let mut input = valid_input();
        input.stage_evidence[0].probability = ProbabilityBand {
            lower: 0.9,
            upper: 1.1,
        };
        assert!(matches!(
            evaluate_non_detection(&input),
            Err(HeritageNonDetectionError::InvalidProbabilityBand(_))
        ));
    }

    #[test]
    fn prior_and_stage_target_class_must_match() {
        let mut input = valid_input();
        input.target_class_id = "masonry_house".into();
        assert_eq!(
            evaluate_non_detection(&input).unwrap_err(),
            HeritageNonDetectionError::PriorTargetClassMismatch
        );
    }

    #[test]
    fn zero_likelihood_outcome_does_not_emit_a_fabricated_posterior() {
        let mut input = valid_input();
        input.prior.probability = ProbabilityBand {
            lower: 1.0,
            upper: 1.0,
        };
        for stage in &mut input.stage_evidence {
            stage.probability = ProbabilityBand {
                lower: 1.0,
                upper: 1.0,
            };
        }
        let report = evaluate_non_detection(&input).unwrap();
        assert_eq!(
            report.disposition,
            NonDetectionDisposition::PosteriorUndefinedZeroLikelihood
        );
        assert_eq!(report.posterior_probability_originally_present, None);
    }

    #[test]
    fn stage_order_does_not_change_deterministic_report() {
        let mut input = valid_input();
        let first = evaluate_non_detection(&input).unwrap();
        input.stage_evidence.reverse();
        let second = evaluate_non_detection(&input).unwrap();
        assert_eq!(first, second);
    }

    #[test]
    fn report_schema_version_mismatch_fails_closed() {
        let mut input = valid_input();
        input.schema_version = "luminous.heritage-nondetection.v9".into();
        assert_eq!(
            evaluate_non_detection(&input).unwrap_err(),
            HeritageNonDetectionError::WrongSchemaVersion
        );
    }
}
