// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Traceable geometric metrology certificates and covariance-aware area sums.
//!
//! This module computes declared checkpoint residual summaries and a
//! factor-model standard uncertainty for additive, disjoint planar areas.
//! It does not authenticate surveyors, instruments, signatures, data rights,
//! point independence, or geometric disjointness. Those require external
//! custodial evidence and independently controlled workflows.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

pub const HERITAGE_METROLOGY_SCHEMA_VERSION: &str = "luminous.heritage-metrology.v0.1";
pub const MIN_ASPRS_STYLE_CHECKPOINTS: usize = 30;
pub const AREA_AGGREGATION_FORMULA_VERSION: &str = "sum-disjoint-area-factor-model.v0.1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetrologyAssessmentProfile {
    Exploratory,
    ProjectDefined,
    Asprs2024Style,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Checkpoint3d {
    pub checkpoint_id: String,
    pub stratum: String,
    pub reference_xyz_m: [f64; 3],
    pub product_xyz_m: [f64; 3],
    /// Horizontal 1-sigma uncertainty of the independent reference survey.
    pub reference_sigma_h_m: f64,
    /// Vertical 1-sigma uncertainty of the independent reference survey.
    pub reference_sigma_v_m: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetrologyCertificate {
    pub schema_version: String,
    pub calibration_id: String,
    pub site_id: String,
    pub snapshot_id: String,
    pub coordinate_reference_id: String,
    pub reference_survey_id: String,
    pub method_build_sha256: String,
    pub source_capture_ids: Vec<String>,
    /// These points were used to fit/register/calibrate the product.
    pub control_point_ids: Vec<String>,
    /// These points are withheld from fitting and reserved for assessment.
    pub independent_checkpoints: Vec<Checkpoint3d>,
    pub assessment_profile: MetrologyAssessmentProfile,
    pub independence_attestation_ref: String,
    pub rights_receipt_ref: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetrologyAssessmentReport {
    pub schema_version: String,
    pub calibration_id: String,
    pub site_id: String,
    pub snapshot_id: String,
    pub coordinate_reference_id: String,
    pub assessment_profile: MetrologyAssessmentProfile,
    pub checkpoint_count: usize,
    pub stratum_counts: BTreeMap<String, usize>,
    pub rmse_h_m: f64,
    pub rmse_v_m: f64,
    pub rmse_3d_m: f64,
    /// Kept separate from product residuals; no hidden subtraction/correction.
    pub reference_sigma_rmse_h_m: f64,
    pub reference_sigma_rmse_v_m: f64,
    pub method_build_sha256: String,
    pub claim_ceiling: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetrologyError {
    SchemaVersionMismatch,
    EmptyIdentity,
    EmptyCoordinateReference,
    EmptyReferenceSurvey,
    InvalidMethodDigest,
    MissingSourceCapture,
    DuplicateSourceCapture,
    EmptyControlPointId,
    DuplicateControlPointId,
    EmptyCheckpointId,
    DuplicateCheckpointId,
    ControlCheckpointOverlap(String),
    EmptyCheckpointStratum(String),
    InvalidCheckpointCoordinate(String),
    InvalidReferenceUncertainty(String),
    InsufficientAsprsStyleCheckpoints { actual: usize, minimum: usize },
    MissingIndependenceAttestation,
    MissingRightsReceipt,
}

impl fmt::Display for MetrologyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for MetrologyError {}

fn non_empty(value: &str) -> bool {
    !value.trim().is_empty()
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|b| b.is_ascii_hexdigit())
}

fn finite_xyz(value: [f64; 3]) -> bool {
    value.into_iter().all(f64::is_finite)
}

pub fn assess_metrology_certificate(
    certificate: &MetrologyCertificate,
) -> Result<MetrologyAssessmentReport, MetrologyError> {
    if certificate.schema_version != HERITAGE_METROLOGY_SCHEMA_VERSION {
        return Err(MetrologyError::SchemaVersionMismatch);
    }
    if !non_empty(&certificate.calibration_id)
        || !non_empty(&certificate.site_id)
        || !non_empty(&certificate.snapshot_id)
    {
        return Err(MetrologyError::EmptyIdentity);
    }
    if !non_empty(&certificate.coordinate_reference_id) {
        return Err(MetrologyError::EmptyCoordinateReference);
    }
    if !non_empty(&certificate.reference_survey_id) {
        return Err(MetrologyError::EmptyReferenceSurvey);
    }
    if !valid_sha256(&certificate.method_build_sha256) {
        return Err(MetrologyError::InvalidMethodDigest);
    }
    if certificate.source_capture_ids.is_empty() {
        return Err(MetrologyError::MissingSourceCapture);
    }
    let mut source_ids = BTreeSet::new();
    for id in &certificate.source_capture_ids {
        if !non_empty(id) || !source_ids.insert(id.as_str()) {
            return Err(MetrologyError::DuplicateSourceCapture);
        }
    }
    let mut control_ids = BTreeSet::new();
    for id in &certificate.control_point_ids {
        if !non_empty(id) {
            return Err(MetrologyError::EmptyControlPointId);
        }
        if !control_ids.insert(id.as_str()) {
            return Err(MetrologyError::DuplicateControlPointId);
        }
    }
    if certificate.independent_checkpoints.is_empty() {
        return Err(MetrologyError::InsufficientAsprsStyleCheckpoints {
            actual: 0,
            minimum: 1,
        });
    }
    if certificate.assessment_profile == MetrologyAssessmentProfile::Asprs2024Style
        && certificate.independent_checkpoints.len() < MIN_ASPRS_STYLE_CHECKPOINTS
    {
        return Err(MetrologyError::InsufficientAsprsStyleCheckpoints {
            actual: certificate.independent_checkpoints.len(),
            minimum: MIN_ASPRS_STYLE_CHECKPOINTS,
        });
    }
    if !non_empty(&certificate.independence_attestation_ref) {
        return Err(MetrologyError::MissingIndependenceAttestation);
    }
    if !non_empty(&certificate.rights_receipt_ref) {
        return Err(MetrologyError::MissingRightsReceipt);
    }

    let mut checkpoint_ids = BTreeSet::new();
    let mut stratum_counts = BTreeMap::new();
    let mut sum_h_squared = 0.0;
    let mut sum_v_squared = 0.0;
    let mut sum_3d_squared = 0.0;
    let mut sum_reference_h_variance = 0.0;
    let mut sum_reference_v_variance = 0.0;

    for checkpoint in &certificate.independent_checkpoints {
        if !non_empty(&checkpoint.checkpoint_id) {
            return Err(MetrologyError::EmptyCheckpointId);
        }
        if !checkpoint_ids.insert(checkpoint.checkpoint_id.as_str()) {
            return Err(MetrologyError::DuplicateCheckpointId);
        }
        if control_ids.contains(checkpoint.checkpoint_id.as_str()) {
            return Err(MetrologyError::ControlCheckpointOverlap(
                checkpoint.checkpoint_id.clone(),
            ));
        }
        if !non_empty(&checkpoint.stratum) {
            return Err(MetrologyError::EmptyCheckpointStratum(
                checkpoint.checkpoint_id.clone(),
            ));
        }
        if !finite_xyz(checkpoint.reference_xyz_m) || !finite_xyz(checkpoint.product_xyz_m) {
            return Err(MetrologyError::InvalidCheckpointCoordinate(
                checkpoint.checkpoint_id.clone(),
            ));
        }
        if !checkpoint.reference_sigma_h_m.is_finite()
            || checkpoint.reference_sigma_h_m < 0.0
            || !checkpoint.reference_sigma_v_m.is_finite()
            || checkpoint.reference_sigma_v_m < 0.0
        {
            return Err(MetrologyError::InvalidReferenceUncertainty(
                checkpoint.checkpoint_id.clone(),
            ));
        }
        *stratum_counts.entry(checkpoint.stratum.clone()).or_insert(0) += 1;

        let dx = checkpoint.product_xyz_m[0] - checkpoint.reference_xyz_m[0];
        let dy = checkpoint.product_xyz_m[1] - checkpoint.reference_xyz_m[1];
        let dz = checkpoint.product_xyz_m[2] - checkpoint.reference_xyz_m[2];
        let h_squared = dx * dx + dy * dy;
        sum_h_squared += h_squared;
        sum_v_squared += dz * dz;
        sum_3d_squared += h_squared + dz * dz;
        sum_reference_h_variance += checkpoint.reference_sigma_h_m.powi(2);
        sum_reference_v_variance += checkpoint.reference_sigma_v_m.powi(2);
    }

    let n = certificate.independent_checkpoints.len() as f64;
    Ok(MetrologyAssessmentReport {
        schema_version: HERITAGE_METROLOGY_SCHEMA_VERSION.to_owned(),
        calibration_id: certificate.calibration_id.clone(),
        site_id: certificate.site_id.clone(),
        snapshot_id: certificate.snapshot_id.clone(),
        coordinate_reference_id: certificate.coordinate_reference_id.clone(),
        assessment_profile: certificate.assessment_profile,
        checkpoint_count: certificate.independent_checkpoints.len(),
        stratum_counts,
        rmse_h_m: (sum_h_squared / n).sqrt(),
        rmse_v_m: (sum_v_squared / n).sqrt(),
        rmse_3d_m: (sum_3d_squared / n).sqrt(),
        reference_sigma_rmse_h_m: (sum_reference_h_variance / n).sqrt(),
        reference_sigma_rmse_v_m: (sum_reference_v_variance / n).sqrt(),
        method_build_sha256: certificate.method_build_sha256.clone(),
        claim_ceiling: "checkpoint_residual_summary_only".to_owned(),
    })
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AreaComponent {
    pub observation_id: String,
    pub area_m2: f64,
    /// Component-specific residual uncertainty, assumed independent across components.
    pub independent_standard_uncertainty_m2: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComponentFactorSensitivity {
    pub observation_id: String,
    /// Signed derivative of this component's area with respect to the shared factor.
    pub sensitivity: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SharedSystematicFactor {
    pub factor_id: String,
    pub standard_uncertainty: f64,
    pub component_sensitivities: Vec<ComponentFactorSensitivity>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AreaMeasurand {
    DisjointPlanFootprintArea,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdditiveAreaAggregationInput {
    pub schema_version: String,
    pub aggregation_id: String,
    pub measurand: AreaMeasurand,
    pub disjointness_receipt_ref: String,
    pub components: Vec<AreaComponent>,
    pub shared_factors: Vec<SharedSystematicFactor>,
    /// Correlation matrix between shared factors, not between components.
    pub factor_correlation_matrix: Vec<Vec<f64>>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FactorSensitivitySummary {
    pub factor_id: String,
    pub aggregate_signed_sensitivity: f64,
    pub standard_uncertainty: f64,
    pub sigma_scaled_sensitivity: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AdditiveAreaAggregationReport {
    pub schema_version: String,
    pub formula_version: String,
    pub aggregation_id: String,
    pub measurand: AreaMeasurand,
    pub component_count: usize,
    pub area_sum_m2: f64,
    pub independent_variance_m4: f64,
    pub shared_systematic_variance_m4: f64,
    pub combined_standard_uncertainty_m2: f64,
    pub factor_summaries: Vec<FactorSensitivitySummary>,
    pub assumptions: Vec<String>,
    pub claim_ceiling: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AreaAggregationError {
    SchemaVersionMismatch,
    EmptyAggregationId,
    MissingDisjointnessReceipt,
    NoComponents,
    DuplicateObservationId(String),
    InvalidArea(String),
    InvalidIndependentUncertainty(String),
    DuplicateFactorId(String),
    InvalidFactorUncertainty(String),
    MissingOrDuplicateFactorSensitivity(String),
    InvalidFactorSensitivity(String),
    FactorCorrelationShapeMismatch,
    FactorCorrelationNotFiniteOrOutOfRange,
    FactorCorrelationNotSymmetric,
    FactorCorrelationDiagonalNotOne,
    FactorCorrelationNotPositiveSemidefinite,
    NonFiniteAggregate,
}

impl fmt::Display for AreaAggregationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for AreaAggregationError {}

fn correlation_is_psd(matrix: &[Vec<f64>]) -> bool {
    let n = matrix.len();
    let mut lower = vec![vec![0.0; n]; n];
    let scale = matrix
        .iter()
        .flatten()
        .fold(1.0_f64, |acc, value| acc.max(value.abs()));
    let tolerance = 1e-12 * scale;

    for i in 0..n {
        for j in 0..=i {
            let mut residual = matrix[i][j];
            for k in 0..j {
                residual -= lower[i][k] * lower[j][k];
            }
            if i == j {
                if residual < -tolerance {
                    return false;
                }
                lower[i][j] = residual.max(0.0).sqrt();
            } else if lower[j][j] > tolerance.sqrt() {
                lower[i][j] = residual / lower[j][j];
            } else if residual.abs() > tolerance {
                return false;
            }
        }
    }
    true
}

pub fn aggregate_disjoint_plan_areas(
    input: &AdditiveAreaAggregationInput,
) -> Result<AdditiveAreaAggregationReport, AreaAggregationError> {
    if input.schema_version != HERITAGE_METROLOGY_SCHEMA_VERSION {
        return Err(AreaAggregationError::SchemaVersionMismatch);
    }
    if !non_empty(&input.aggregation_id) {
        return Err(AreaAggregationError::EmptyAggregationId);
    }
    if !non_empty(&input.disjointness_receipt_ref) {
        return Err(AreaAggregationError::MissingDisjointnessReceipt);
    }
    if input.components.is_empty() {
        return Err(AreaAggregationError::NoComponents);
    }

    let mut component_ids = BTreeSet::new();
    let mut area_sum = 0.0;
    let mut independent_variance = 0.0;
    for component in &input.components {
        if !non_empty(&component.observation_id)
            || !component_ids.insert(component.observation_id.as_str())
        {
            return Err(AreaAggregationError::DuplicateObservationId(
                component.observation_id.clone(),
            ));
        }
        if !component.area_m2.is_finite() || component.area_m2 <= 0.0 {
            return Err(AreaAggregationError::InvalidArea(
                component.observation_id.clone(),
            ));
        }
        if !component.independent_standard_uncertainty_m2.is_finite()
            || component.independent_standard_uncertainty_m2 < 0.0
        {
            return Err(AreaAggregationError::InvalidIndependentUncertainty(
                component.observation_id.clone(),
            ));
        }
        area_sum += component.area_m2;
        independent_variance += component.independent_standard_uncertainty_m2.powi(2);
    }

    let factor_count = input.shared_factors.len();
    if input.factor_correlation_matrix.len() != factor_count
        || input
            .factor_correlation_matrix
            .iter()
            .any(|row| row.len() != factor_count)
    {
        return Err(AreaAggregationError::FactorCorrelationShapeMismatch);
    }

    let mut factor_ids = BTreeSet::new();
    let mut sigma_scaled_sensitivity = Vec::with_capacity(factor_count);
    let mut factor_summaries = Vec::with_capacity(factor_count);
    for factor in &input.shared_factors {
        if !non_empty(&factor.factor_id) || !factor_ids.insert(factor.factor_id.as_str()) {
            return Err(AreaAggregationError::DuplicateFactorId(
                factor.factor_id.clone(),
            ));
        }
        if !factor.standard_uncertainty.is_finite() || factor.standard_uncertainty < 0.0 {
            return Err(AreaAggregationError::InvalidFactorUncertainty(
                factor.factor_id.clone(),
            ));
        }
        let mut sensitivities = BTreeMap::new();
        for entry in &factor.component_sensitivities {
            if !entry.sensitivity.is_finite()
                || !component_ids.contains(entry.observation_id.as_str())
                || sensitivities
                    .insert(entry.observation_id.as_str(), entry.sensitivity)
                    .is_some()
            {
                return Err(AreaAggregationError::InvalidFactorSensitivity(
                    factor.factor_id.clone(),
                ));
            }
        }
        if sensitivities.len() != component_ids.len()
            || component_ids.iter().any(|id| !sensitivities.contains_key(id))
        {
            return Err(AreaAggregationError::MissingOrDuplicateFactorSensitivity(
                factor.factor_id.clone(),
            ));
        }
        let aggregate_sensitivity: f64 = sensitivities.values().sum();
        let sigma_scaled = aggregate_sensitivity * factor.standard_uncertainty;
        sigma_scaled_sensitivity.push(sigma_scaled);
        factor_summaries.push(FactorSensitivitySummary {
            factor_id: factor.factor_id.clone(),
            aggregate_signed_sensitivity: aggregate_sensitivity,
            standard_uncertainty: factor.standard_uncertainty,
            sigma_scaled_sensitivity: sigma_scaled,
        });
    }

    let matrix = &input.factor_correlation_matrix;
    let symmetry_tolerance = 1e-12;
    for i in 0..factor_count {
        for j in 0..factor_count {
            let value = matrix[i][j];
            if !value.is_finite() || !(-1.0..=1.0).contains(&value) {
                return Err(AreaAggregationError::FactorCorrelationNotFiniteOrOutOfRange);
            }
            if (value - matrix[j][i]).abs() > symmetry_tolerance {
                return Err(AreaAggregationError::FactorCorrelationNotSymmetric);
            }
        }
        if (matrix[i][i] - 1.0).abs() > symmetry_tolerance {
            return Err(AreaAggregationError::FactorCorrelationDiagonalNotOne);
        }
    }
    if !correlation_is_psd(matrix) {
        return Err(AreaAggregationError::FactorCorrelationNotPositiveSemidefinite);
    }

    let mut shared_variance = 0.0;
    for i in 0..factor_count {
        for j in 0..factor_count {
            shared_variance +=
                sigma_scaled_sensitivity[i] * matrix[i][j] * sigma_scaled_sensitivity[j];
        }
    }
    let combined_variance = independent_variance + shared_variance;
    if !area_sum.is_finite()
        || !independent_variance.is_finite()
        || !shared_variance.is_finite()
        || !combined_variance.is_finite()
        || combined_variance < -1e-12
    {
        return Err(AreaAggregationError::NonFiniteAggregate);
    }

    Ok(AdditiveAreaAggregationReport {
        schema_version: HERITAGE_METROLOGY_SCHEMA_VERSION.to_owned(),
        formula_version: AREA_AGGREGATION_FORMULA_VERSION.to_owned(),
        aggregation_id: input.aggregation_id.clone(),
        measurand: input.measurand,
        component_count: input.components.len(),
        area_sum_m2: area_sum,
        independent_variance_m4: independent_variance,
        shared_systematic_variance_m4: shared_variance.max(0.0),
        combined_standard_uncertainty_m2: combined_variance.max(0.0).sqrt(),
        factor_summaries,
        assumptions: vec![
            "Component-specific residual errors are mutually independent.".to_owned(),
            "Shared-factor covariance is represented by the declared correlation matrix.".to_owned(),
            "The disjointness receipt is externally produced and is not authenticated here.".to_owned(),
            "No probability distribution or confidence interval is implied by standard uncertainty.".to_owned(),
            "Only additive, disjoint, calibrated 2D plan-footprint areas are supported.".to_owned(),
        ],
        claim_ceiling: "factor_model_standard_uncertainty_only".to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digest() -> String {
        "a".repeat(64)
    }

    fn checkpoint(index: usize) -> Checkpoint3d {
        let dx = 0.01 * (index as f64 + 1.0);
        let dy = 0.02;
        let dz = 0.03;
        Checkpoint3d {
            checkpoint_id: format!("check-{index}"),
            stratum: if index % 2 == 0 { "open" } else { "occluded" }.to_owned(),
            reference_xyz_m: [100.0 + index as f64, 200.0, 10.0],
            product_xyz_m: [100.0 + index as f64 + dx, 200.0 + dy, 10.0 + dz],
            reference_sigma_h_m: 0.005,
            reference_sigma_v_m: 0.008,
        }
    }

    fn certificate(profile: MetrologyAssessmentProfile, n: usize) -> MetrologyCertificate {
        MetrologyCertificate {
            schema_version: HERITAGE_METROLOGY_SCHEMA_VERSION.to_owned(),
            calibration_id: "cal-1".to_owned(),
            site_id: "site-1".to_owned(),
            snapshot_id: "snapshot-1".to_owned(),
            coordinate_reference_id: "EPSG:32735".to_owned(),
            reference_survey_id: "total-station-campaign-1".to_owned(),
            method_build_sha256: digest(),
            source_capture_ids: vec!["capture-1".to_owned()],
            control_point_ids: vec!["control-1".to_owned()],
            independent_checkpoints: (0..n).map(checkpoint).collect(),
            assessment_profile: profile,
            independence_attestation_ref: "custody://independence/1".to_owned(),
            rights_receipt_ref: "rights://survey/1".to_owned(),
        }
    }

    fn aggregate_input() -> AdditiveAreaAggregationInput {
        AdditiveAreaAggregationInput {
            schema_version: HERITAGE_METROLOGY_SCHEMA_VERSION.to_owned(),
            aggregation_id: "wall-area-1".to_owned(),
            measurand: AreaMeasurand::DisjointPlanFootprintArea,
            disjointness_receipt_ref: "gis-union://receipt/1".to_owned(),
            components: vec![
                AreaComponent {
                    observation_id: "stone-1".to_owned(),
                    area_m2: 1.0,
                    independent_standard_uncertainty_m2: 0.1,
                },
                AreaComponent {
                    observation_id: "stone-2".to_owned(),
                    area_m2: 2.0,
                    independent_standard_uncertainty_m2: 0.2,
                },
            ],
            shared_factors: vec![SharedSystematicFactor {
                factor_id: "ortho-scale".to_owned(),
                standard_uncertainty: 0.5,
                component_sensitivities: vec![
                    ComponentFactorSensitivity {
                        observation_id: "stone-1".to_owned(),
                        sensitivity: 1.0,
                    },
                    ComponentFactorSensitivity {
                        observation_id: "stone-2".to_owned(),
                        sensitivity: 1.0,
                    },
                ],
            }],
            factor_correlation_matrix: vec![vec![1.0]],
        }
    }

    #[test]
    fn exploratory_certificate_reports_independent_rmse_and_reference_error() {
        let report = assess_metrology_certificate(&certificate(
            MetrologyAssessmentProfile::Exploratory,
            2,
        ))
        .expect("exploratory certificate");
        assert_eq!(report.checkpoint_count, 2);
        assert_eq!(report.stratum_counts.get("open"), Some(&1));
        assert!((report.rmse_h_m - (0.000_65_f64 + 0.000_4).sqrt()).abs() > 0.0);
        assert!(report.rmse_h_m > 0.0);
        assert!(report.reference_sigma_rmse_h_m > 0.0);
        assert_eq!(report.claim_ceiling, "checkpoint_residual_summary_only");
    }

    #[test]
    fn asprs_style_profile_requires_thirty_independent_checkpoints() {
        let err = assess_metrology_certificate(&certificate(
            MetrologyAssessmentProfile::Asprs2024Style,
            29,
        ))
        .expect_err("29 is below the declared profile minimum");
        assert_eq!(
            err,
            MetrologyError::InsufficientAsprsStyleCheckpoints {
                actual: 29,
                minimum: MIN_ASPRS_STYLE_CHECKPOINTS
            }
        );
    }

    #[test]
    fn control_point_cannot_be_reused_as_an_independent_checkpoint() {
        let mut input = certificate(MetrologyAssessmentProfile::Exploratory, 1);
        input.independent_checkpoints[0].checkpoint_id = "control-1".to_owned();
        assert!(matches!(
            assess_metrology_certificate(&input),
            Err(MetrologyError::ControlCheckpointOverlap(_))
        ));
    }

    #[test]
    fn duplicate_checkpoint_identity_is_rejected() {
        let mut input = certificate(MetrologyAssessmentProfile::Exploratory, 2);
        input.independent_checkpoints[1].checkpoint_id =
            input.independent_checkpoints[0].checkpoint_id.clone();
        assert_eq!(
            assess_metrology_certificate(&input),
            Err(MetrologyError::DuplicateCheckpointId)
        );
    }

    #[test]
    fn invalid_coordinate_or_survey_sigma_is_rejected() {
        let mut input = certificate(MetrologyAssessmentProfile::Exploratory, 1);
        input.independent_checkpoints[0].product_xyz_m[2] = f64::NAN;
        assert!(matches!(
            assess_metrology_certificate(&input),
            Err(MetrologyError::InvalidCheckpointCoordinate(_))
        ));
        input.independent_checkpoints[0].product_xyz_m[2] = 1.0;
        input.independent_checkpoints[0].reference_sigma_h_m = -0.1;
        assert!(matches!(
            assess_metrology_certificate(&input),
            Err(MetrologyError::InvalidReferenceUncertainty(_))
        ));
    }

    #[test]
    fn factor_model_adds_independent_variance_and_shared_covariance() {
        let report = aggregate_disjoint_plan_areas(&aggregate_input()).expect("valid aggregation");
        assert_eq!(report.area_sum_m2, 3.0);
        assert!((report.independent_variance_m4 - 0.05).abs() < 1e-12);
        assert!((report.shared_systematic_variance_m4 - 1.0).abs() < 1e-12);
        assert!((report.combined_standard_uncertainty_m2 - 1.05_f64.sqrt()).abs() < 1e-12);
    }

    #[test]
    fn correlated_shared_factors_use_the_declared_factor_correlation_matrix() {
        let mut input = aggregate_input();
        input.shared_factors.push(SharedSystematicFactor {
            factor_id: "camera-scale".to_owned(),
            standard_uncertainty: 0.3,
            component_sensitivities: vec![
                ComponentFactorSensitivity {
                    observation_id: "stone-1".to_owned(),
                    sensitivity: 1.0,
                },
                ComponentFactorSensitivity {
                    observation_id: "stone-2".to_owned(),
                    sensitivity: 2.0,
                },
            ],
        });
        input.factor_correlation_matrix = vec![vec![1.0, 0.5], vec![0.5, 1.0]];
        let report = aggregate_disjoint_plan_areas(&input).expect("correlated factors");
        // g = [1.0, 0.9]; g' R g = 1 + 0.81 + 0.9 = 2.71
        assert!((report.shared_systematic_variance_m4 - 2.71).abs() < 1e-12);
    }

    #[test]
    fn non_positive_semidefinite_factor_correlation_is_rejected() {
        let mut input = aggregate_input();
        input.shared_factors.extend([
            SharedSystematicFactor {
                factor_id: "factor-b".to_owned(),
                standard_uncertainty: 0.2,
                component_sensitivities: vec![
                    ComponentFactorSensitivity { observation_id: "stone-1".to_owned(), sensitivity: 1.0 },
                    ComponentFactorSensitivity { observation_id: "stone-2".to_owned(), sensitivity: 1.0 },
                ],
            },
            SharedSystematicFactor {
                factor_id: "factor-c".to_owned(),
                standard_uncertainty: 0.3,
                component_sensitivities: vec![
                    ComponentFactorSensitivity { observation_id: "stone-1".to_owned(), sensitivity: 1.0 },
                    ComponentFactorSensitivity { observation_id: "stone-2".to_owned(), sensitivity: 1.0 },
                ],
            },
        ]);
        input.factor_correlation_matrix = vec![
            vec![1.0, 0.9, -0.9],
            vec![0.9, 1.0, 0.9],
            vec![-0.9, 0.9, 1.0],
        ];
        assert_eq!(
            aggregate_disjoint_plan_areas(&input),
            Err(AreaAggregationError::FactorCorrelationNotPositiveSemidefinite)
        );
    }

    #[test]
    fn missing_factor_sensitivity_is_rejected() {
        let mut input = aggregate_input();
        input.shared_factors[0].component_sensitivities.pop();
        assert!(matches!(
            aggregate_disjoint_plan_areas(&input),
            Err(AreaAggregationError::MissingOrDuplicateFactorSensitivity(_))
        ));
    }

    #[test]
    fn overlapping_or_unproven_geometry_cannot_be_summed_without_receipt() {
        let mut input = aggregate_input();
        input.disjointness_receipt_ref.clear();
        assert_eq!(
            aggregate_disjoint_plan_areas(&input),
            Err(AreaAggregationError::MissingDisjointnessReceipt)
        );
    }

    #[test]
    fn duplicate_components_and_negative_areas_are_rejected() {
        let mut input = aggregate_input();
        input.components[1].observation_id = input.components[0].observation_id.clone();
        assert!(matches!(
            aggregate_disjoint_plan_areas(&input),
            Err(AreaAggregationError::DuplicateObservationId(_))
        ));
        input.components[1].observation_id = "stone-2".to_owned();
        input.components[1].area_m2 = -1.0;
        assert!(matches!(
            aggregate_disjoint_plan_areas(&input),
            Err(AreaAggregationError::InvalidArea(_))
        ));
    }

    #[test]
    fn no_shared_factors_uses_only_component_independent_variance() {
        let mut input = aggregate_input();
        input.shared_factors.clear();
        input.factor_correlation_matrix.clear();
        let report = aggregate_disjoint_plan_areas(&input).expect("independent-only sum");
        assert!((report.combined_standard_uncertainty_m2 - 0.05_f64.sqrt()).abs() < 1e-12);
    }

    #[test]
    fn metrology_schema_and_fixture_are_valid_json() {
        let schema: serde_json::Value = serde_json::from_str(include_str!(
            "../../schemas/heritage-metrology-v0.1.schema.json"
        ))
        .expect("schema JSON");
        let fixture: MetrologyCertificate = serde_json::from_str(include_str!(
            "../../schemas/examples/heritage-metrology-certificate-synthetic-v0.1.json"
        ))
        .expect("certificate fixture");
        assert_eq!(
            schema["$schema"],
            "https://json-schema.org/draft/2020-12/schema"
        );
        assert!(assess_metrology_certificate(&fixture).is_ok());
    }
}
