// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Provenance-aware architectural observation and assembly graph validation.
//!
//! This module validates the structural integrity of stone-to-settlement
//! measurement packages. It does not perform image inference, GIS operations,
//! metrology, archaeological interpretation, or independent source
//! authentication. Pixel measurements are never promoted to physical units.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

pub const HERITAGE_ARCHITECTURE_SCHEMA_VERSION: &str =
    "luminous.heritage-architecture.v0.1";
pub const MAX_ARCHITECTURE_ELEMENTS: usize = 1_000_000;
pub const MAX_ARCHITECTURE_OBSERVATIONS: usize = 5_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArchitectureLevel {
    Stone,
    WallSegment,
    Wall,
    Building,
    Compound,
    Insula,
    Neighbourhood,
    Settlement,
}

impl ArchitectureLevel {
    fn rank(self) -> u8 {
        match self {
            Self::Stone => 0,
            Self::WallSegment => 1,
            Self::Wall => 2,
            Self::Building => 3,
            Self::Compound => 4,
            Self::Insula => 5,
            Self::Neighbourhood => 6,
            Self::Settlement => 7,
        }
    }

    fn valid_parent_level(self, parent: Self) -> bool {
        parent.rank() == self.rank() + 1
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ParentageStatus {
    Assigned,
    Unassigned,
    Ambiguous,
    NotAssessed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Parentage {
    pub status: ParentageStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_element_id: Option<String>,
    #[serde(default)]
    pub candidate_parent_element_ids: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CaptureModality {
    DroneImage,
    Orthomosaic,
    SatelliteImage,
    Photogrammetry,
    PointCloud,
    FieldSurvey,
    TotalStation,
    RtkGnss,
    DerivedModel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RightsState {
    Verified,
    Pending,
    Unknown,
    Restricted,
    Denied,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArchitectureCapture {
    pub capture_id: String,
    pub site_id: String,
    pub snapshot_id: String,
    pub modality: CaptureModality,
    pub source_sha256: String,
    pub coordinate_reference_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub calibration_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ground_sample_distance_m: Option<f64>,
    pub rights_state: RightsState,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rights_receipt_ref: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GeometryOrigin {
    DirectMeasurement,
    HumanDigitization,
    ModelOutput,
    Simulation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MeasurementBasis {
    PixelOnly,
    CalibratedPlanar2d,
    CalibratedSurface3d,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricAreaSemantics {
    PlanFootprint,
    SurfaceArea,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MetricUncertainty {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub area_standard_deviation_m2: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub area_interval_90_m2: Option<[f64; 2]>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArchitectureGeometryObservation {
    pub observation_id: String,
    pub geometry_sha256: String,
    pub origin: GeometryOrigin,
    pub measurement_basis: MeasurementBasis,
    pub source_capture_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spatial_reference_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub calibration_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pixel_area_pixels2: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metric_area_m2: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metric_area_semantics: Option<MetricAreaSemantics>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metric_perimeter_m: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mrr_area_ratio: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub elongation_ratio: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uncertainty: Option<MetricUncertainty>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub method_build_sha256: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ArchitectureElement {
    pub element_id: String,
    pub level: ArchitectureLevel,
    /// Stable identity of the physical feature, not an image/frame identifier.
    /// One element may carry multiple geometry observations from different captures.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub physical_identity_id: Option<String>,
    pub parentage: Parentage,
    pub observations: Vec<ArchitectureGeometryObservation>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HeritageArchitectureInput {
    pub schema_version: String,
    pub project_id: String,
    pub site_id: String,
    pub snapshot_id: String,
    pub coordinate_reference_id: String,
    pub captures: Vec<ArchitectureCapture>,
    pub elements: Vec<ArchitectureElement>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArchitectureValidationStatus {
    StructurallyValid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArchitectureClaimCeiling {
    StructuralIntegrityOnly,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeritageArchitectureValidationReport {
    pub schema_version: String,
    pub status: ArchitectureValidationStatus,
    pub claim_ceiling: ArchitectureClaimCeiling,
    pub element_count: usize,
    pub observation_count: usize,
    pub pixel_only_observation_count: usize,
    pub metric_observation_count: usize,
    pub model_output_observation_count: usize,
    pub simulation_observation_count: usize,
    pub unresolved_parentage_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeritageArchitectureError {
    SchemaVersionMismatch,
    EmptyProjectOrSiteIdentity,
    EmptyCoordinateReference,
    TooManyElements,
    TooManyObservations,
    EmptyCaptureId,
    DuplicateCaptureId(String),
    CaptureScopeMismatch(String),
    InvalidSourceDigest(String),
    EmptyCaptureCoordinateReference(String),
    VerifiedRightsReceiptMissing(String),
    InvalidGroundSampleDistance(String),
    EmptyElementId,
    DuplicateElementId(String),
    MissingPhysicalIdentity(String),
    DuplicatePhysicalIdentity(String),
    ElementHasNoObservations(String),
    EmptyObservationId,
    DuplicateObservationId(String),
    GeometryDigestInvalid(String),
    ObservationHasNoCapture(String),
    CaptureReferenceMissing(String),
    InvalidParentage(String),
    ParentReferenceMissing(String),
    ParentLevelMismatch(String),
    ParentageCandidateInvalid(String),
    PixelMeasurementInvalid(String),
    PixelMeasurementClaimsMetricUnits(String),
    MetricMeasurementIncomplete(String),
    MetricCoordinateReferenceMismatch(String),
    MetricCalibrationMissing(String),
    MetricCalibrationNotInSource(String),
    DirectMeasurementSourceMissing(String),
    MethodBuildDigestMissing(String),
    MethodBuildDigestInvalid(String),
    InvalidMetricValue(String),
    InvalidUncertainty(String),
}

impl fmt::Display for HeritageArchitectureError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for HeritageArchitectureError {}

fn non_empty(value: &str) -> bool {
    !value.trim().is_empty()
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn finite_positive(value: f64) -> bool {
    value.is_finite() && value > 0.0
}

fn validate_parentage(
    element: &ArchitectureElement,
    elements: &BTreeMap<&str, &ArchitectureElement>,
) -> Result<(), HeritageArchitectureError> {
    let parentage = &element.parentage;
    match parentage.status {
        ParentageStatus::Assigned => {
            if !parentage.candidate_parent_element_ids.is_empty() {
                return Err(HeritageArchitectureError::InvalidParentage(
                    element.element_id.clone(),
                ));
            }
            let Some(parent_id) = parentage.parent_element_id.as_deref() else {
                return Err(HeritageArchitectureError::InvalidParentage(
                    element.element_id.clone(),
                ));
            };
            let Some(parent) = elements.get(parent_id) else {
                return Err(HeritageArchitectureError::ParentReferenceMissing(
                    element.element_id.clone(),
                ));
            };
            if !element.level.valid_parent_level(parent.level) {
                return Err(HeritageArchitectureError::ParentLevelMismatch(
                    element.element_id.clone(),
                ));
            }
        }
        ParentageStatus::Ambiguous => {
            if parentage.parent_element_id.is_some()
                || parentage.candidate_parent_element_ids.len() < 2
            {
                return Err(HeritageArchitectureError::InvalidParentage(
                    element.element_id.clone(),
                ));
            }
            let mut candidates = BTreeSet::new();
            for candidate_id in &parentage.candidate_parent_element_ids {
                if !candidates.insert(candidate_id.as_str()) {
                    return Err(HeritageArchitectureError::ParentageCandidateInvalid(
                        element.element_id.clone(),
                    ));
                }
                let Some(candidate) = elements.get(candidate_id.as_str()) else {
                    return Err(HeritageArchitectureError::ParentageCandidateInvalid(
                        element.element_id.clone(),
                    ));
                };
                if !element.level.valid_parent_level(candidate.level) {
                    return Err(HeritageArchitectureError::ParentageCandidateInvalid(
                        element.element_id.clone(),
                    ));
                }
            }
        }
        ParentageStatus::Unassigned | ParentageStatus::NotAssessed => {
            if parentage.parent_element_id.is_some()
                || !parentage.candidate_parent_element_ids.is_empty()
            {
                return Err(HeritageArchitectureError::InvalidParentage(
                    element.element_id.clone(),
                ));
            }
        }
    }
    Ok(())
}

fn validate_observation(
    input: &HeritageArchitectureInput,
    captures: &BTreeMap<&str, &ArchitectureCapture>,
    observation: &ArchitectureGeometryObservation,
) -> Result<(), HeritageArchitectureError> {
    if !non_empty(&observation.observation_id) {
        return Err(HeritageArchitectureError::EmptyObservationId);
    }
    if !valid_sha256(&observation.geometry_sha256) {
        return Err(HeritageArchitectureError::GeometryDigestInvalid(
            observation.observation_id.clone(),
        ));
    }
    if observation.source_capture_ids.is_empty() {
        return Err(HeritageArchitectureError::ObservationHasNoCapture(
            observation.observation_id.clone(),
        ));
    }
    let mut source_captures = Vec::new();
    let mut seen_sources = BTreeSet::new();
    for capture_id in &observation.source_capture_ids {
        if !seen_sources.insert(capture_id.as_str()) {
            return Err(HeritageArchitectureError::CaptureReferenceMissing(
                observation.observation_id.clone(),
            ));
        }
        let Some(capture) = captures.get(capture_id.as_str()) else {
            return Err(HeritageArchitectureError::CaptureReferenceMissing(
                observation.observation_id.clone(),
            ));
        };
        source_captures.push(*capture);
    }

    match observation.origin {
        GeometryOrigin::ModelOutput | GeometryOrigin::Simulation => {
            let Some(digest) = observation.method_build_sha256.as_deref() else {
                return Err(HeritageArchitectureError::MethodBuildDigestMissing(
                    observation.observation_id.clone(),
                ));
            };
            if !valid_sha256(digest) {
                return Err(HeritageArchitectureError::MethodBuildDigestInvalid(
                    observation.observation_id.clone(),
                ));
            }
        }
        GeometryOrigin::DirectMeasurement | GeometryOrigin::HumanDigitization => {}
    }

    match observation.measurement_basis {
        MeasurementBasis::PixelOnly => {
            if observation
                .pixel_area_pixels2
                .is_none_or(|value| !finite_positive(value))
            {
                return Err(HeritageArchitectureError::PixelMeasurementInvalid(
                    observation.observation_id.clone(),
                ));
            }
            if observation.metric_area_m2.is_some()
                || observation.metric_area_semantics.is_some()
                || observation.metric_perimeter_m.is_some()
                || observation.spatial_reference_id.is_some()
                || observation.calibration_id.is_some()
            {
                return Err(
                    HeritageArchitectureError::PixelMeasurementClaimsMetricUnits(
                        observation.observation_id.clone(),
                    ),
                );
            }
        }
        MeasurementBasis::CalibratedPlanar2d | MeasurementBasis::CalibratedSurface3d => {
            if observation
                .metric_area_m2
                .is_none_or(|value| !finite_positive(value))
                || observation.spatial_reference_id.as_deref()
                    != Some(input.coordinate_reference_id.as_str())
            {
                return Err(HeritageArchitectureError::MetricMeasurementIncomplete(
                    observation.observation_id.clone(),
                ));
            }
            let expected_semantics = match observation.measurement_basis {
                MeasurementBasis::CalibratedPlanar2d => MetricAreaSemantics::PlanFootprint,
                MeasurementBasis::CalibratedSurface3d => MetricAreaSemantics::SurfaceArea,
                MeasurementBasis::PixelOnly => unreachable!(),
            };
            if observation.metric_area_semantics != Some(expected_semantics) {
                return Err(HeritageArchitectureError::MetricMeasurementIncomplete(
                    observation.observation_id.clone(),
                ));
            }
            let Some(calibration_id) = observation.calibration_id.as_deref() else {
                return Err(HeritageArchitectureError::MetricCalibrationMissing(
                    observation.observation_id.clone(),
                ));
            };
            if !source_captures.iter().any(|capture| {
                capture.calibration_id.as_deref() == Some(calibration_id)
                    && capture.coordinate_reference_id == input.coordinate_reference_id
            }) {
                return Err(HeritageArchitectureError::MetricCalibrationNotInSource(
                    observation.observation_id.clone(),
                ));
            }
        }
    }

    if observation.metric_perimeter_m.is_some_and(|value| !finite_positive(value)) {
        return Err(HeritageArchitectureError::InvalidMetricValue(
            observation.observation_id.clone(),
        ));
    }
    if observation.mrr_area_ratio.is_some_and(|value| {
        !value.is_finite() || !(0.0..=1.0).contains(&value)
    }) || observation
        .elongation_ratio
        .is_some_and(|value| !value.is_finite() || value < 1.0)
    {
        return Err(HeritageArchitectureError::InvalidMetricValue(
            observation.observation_id.clone(),
        ));
    }
    if let Some(uncertainty) = &observation.uncertainty {
        if uncertainty
            .area_standard_deviation_m2
            .is_some_and(|value| !value.is_finite() || value < 0.0)
        {
            return Err(HeritageArchitectureError::InvalidUncertainty(
                observation.observation_id.clone(),
            ));
        }
        if let Some([lower, upper]) = uncertainty.area_interval_90_m2 {
            let Some(area) = observation.metric_area_m2 else {
                return Err(HeritageArchitectureError::InvalidUncertainty(
                    observation.observation_id.clone(),
                ));
            };
            if !lower.is_finite() || !upper.is_finite() || lower < 0.0 || lower > area || area > upper
            {
                return Err(HeritageArchitectureError::InvalidUncertainty(
                    observation.observation_id.clone(),
                ));
            }
        }
    }

    if observation.origin == GeometryOrigin::DirectMeasurement
        && !source_captures.iter().any(|capture| {
            matches!(
                capture.modality,
                CaptureModality::FieldSurvey
                    | CaptureModality::TotalStation
                    | CaptureModality::RtkGnss
                    | CaptureModality::Photogrammetry
                    | CaptureModality::PointCloud
            )
        })
    {
        return Err(HeritageArchitectureError::DirectMeasurementSourceMissing(
            observation.observation_id.clone(),
        ));
    }
    Ok(())
}

pub fn validate_heritage_architecture(
    input: &HeritageArchitectureInput,
) -> Result<HeritageArchitectureValidationReport, HeritageArchitectureError> {
    if input.schema_version != HERITAGE_ARCHITECTURE_SCHEMA_VERSION {
        return Err(HeritageArchitectureError::SchemaVersionMismatch);
    }
    if !non_empty(&input.project_id) || !non_empty(&input.site_id) || !non_empty(&input.snapshot_id)
    {
        return Err(HeritageArchitectureError::EmptyProjectOrSiteIdentity);
    }
    if !non_empty(&input.coordinate_reference_id) {
        return Err(HeritageArchitectureError::EmptyCoordinateReference);
    }
    if input.elements.len() > MAX_ARCHITECTURE_ELEMENTS {
        return Err(HeritageArchitectureError::TooManyElements);
    }

    let mut captures = BTreeMap::new();
    for capture in &input.captures {
        if !non_empty(&capture.capture_id) {
            return Err(HeritageArchitectureError::EmptyCaptureId);
        }
        if captures.insert(capture.capture_id.as_str(), capture).is_some() {
            return Err(HeritageArchitectureError::DuplicateCaptureId(
                capture.capture_id.clone(),
            ));
        }
        if capture.site_id != input.site_id || capture.snapshot_id != input.snapshot_id {
            return Err(HeritageArchitectureError::CaptureScopeMismatch(
                capture.capture_id.clone(),
            ));
        }
        if !valid_sha256(&capture.source_sha256) {
            return Err(HeritageArchitectureError::InvalidSourceDigest(
                capture.capture_id.clone(),
            ));
        }
        if !non_empty(&capture.coordinate_reference_id) {
            return Err(HeritageArchitectureError::EmptyCaptureCoordinateReference(
                capture.capture_id.clone(),
            ));
        }
        if capture.rights_state == RightsState::Verified
            && capture
                .rights_receipt_ref
                .as_deref()
                .is_none_or(|value| !non_empty(value))
        {
            return Err(HeritageArchitectureError::VerifiedRightsReceiptMissing(
                capture.capture_id.clone(),
            ));
        }
        if capture
            .ground_sample_distance_m
            .is_some_and(|value| !finite_positive(value))
        {
            return Err(HeritageArchitectureError::InvalidGroundSampleDistance(
                capture.capture_id.clone(),
            ));
        }
    }

    let mut elements = BTreeMap::new();
    for element in &input.elements {
        if !non_empty(&element.element_id) {
            return Err(HeritageArchitectureError::EmptyElementId);
        }
        if elements.insert(element.element_id.as_str(), element).is_some() {
            return Err(HeritageArchitectureError::DuplicateElementId(
                element.element_id.clone(),
            ));
        }
        if element.level == ArchitectureLevel::Stone
            && element
                .physical_identity_id
                .as_deref()
                .is_none_or(|value| !non_empty(value))
        {
            return Err(HeritageArchitectureError::MissingPhysicalIdentity(
                element.element_id.clone(),
            ));
        }
        if element.observations.is_empty() {
            return Err(HeritageArchitectureError::ElementHasNoObservations(
                element.element_id.clone(),
            ));
        }
    }

    let mut physical_identities = BTreeSet::new();
    let mut observation_ids = BTreeSet::new();
    let mut observation_count = 0usize;
    let mut pixel_only_observation_count = 0usize;
    let mut metric_observation_count = 0usize;
    let mut model_output_observation_count = 0usize;
    let mut simulation_observation_count = 0usize;
    let mut unresolved_parentage_count = 0usize;

    for element in &input.elements {
        if let Some(identity) = element.physical_identity_id.as_deref() {
            if !physical_identities.insert(identity) {
                return Err(HeritageArchitectureError::DuplicatePhysicalIdentity(
                    identity.to_owned(),
                ));
            }
        }
        validate_parentage(element, &elements)?;
        if matches!(
            element.parentage.status,
            ParentageStatus::Unassigned | ParentageStatus::Ambiguous | ParentageStatus::NotAssessed
        ) {
            unresolved_parentage_count += 1;
        }
        for observation in &element.observations {
            observation_count += 1;
            if observation_count > MAX_ARCHITECTURE_OBSERVATIONS {
                return Err(HeritageArchitectureError::TooManyObservations);
            }
            if !observation_ids.insert(observation.observation_id.as_str()) {
                return Err(HeritageArchitectureError::DuplicateObservationId(
                    observation.observation_id.clone(),
                ));
            }
            validate_observation(input, &captures, observation)?;
            match observation.measurement_basis {
                MeasurementBasis::PixelOnly => pixel_only_observation_count += 1,
                MeasurementBasis::CalibratedPlanar2d | MeasurementBasis::CalibratedSurface3d => {
                    metric_observation_count += 1;
                }
            }
            match observation.origin {
                GeometryOrigin::ModelOutput => model_output_observation_count += 1,
                GeometryOrigin::Simulation => simulation_observation_count += 1,
                GeometryOrigin::DirectMeasurement | GeometryOrigin::HumanDigitization => {}
            }
        }
    }

    Ok(HeritageArchitectureValidationReport {
        schema_version: HERITAGE_ARCHITECTURE_SCHEMA_VERSION.to_owned(),
        status: ArchitectureValidationStatus::StructurallyValid,
        claim_ceiling: ArchitectureClaimCeiling::StructuralIntegrityOnly,
        element_count: input.elements.len(),
        observation_count,
        pixel_only_observation_count,
        metric_observation_count,
        model_output_observation_count,
        simulation_observation_count,
        unresolved_parentage_count,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digest(ch: char) -> String {
        std::iter::repeat_n(ch, 64).collect()
    }

    fn capture(
        id: &str,
        modality: CaptureModality,
        calibration_id: Option<&str>,
    ) -> ArchitectureCapture {
        ArchitectureCapture {
            capture_id: id.to_owned(),
            site_id: "site-1".to_owned(),
            snapshot_id: "snapshot-2026".to_owned(),
            modality,
            source_sha256: digest('a'),
            coordinate_reference_id: "EPSG:32735".to_owned(),
            calibration_id: calibration_id.map(str::to_owned),
            ground_sample_distance_m: Some(0.01),
            rights_state: RightsState::Unknown,
            rights_receipt_ref: None,
        }
    }

    fn observation(
        id: &str,
        capture_id: &str,
        origin: GeometryOrigin,
    ) -> ArchitectureGeometryObservation {
        ArchitectureGeometryObservation {
            observation_id: id.to_owned(),
            geometry_sha256: digest('b'),
            origin,
            measurement_basis: MeasurementBasis::CalibratedPlanar2d,
            source_capture_ids: vec![capture_id.to_owned()],
            spatial_reference_id: Some("EPSG:32735".to_owned()),
            calibration_id: Some("cal-1".to_owned()),
            pixel_area_pixels2: Some(400.0),
            metric_area_m2: Some(0.04),
            metric_area_semantics: Some(MetricAreaSemantics::PlanFootprint),
            metric_perimeter_m: Some(0.8),
            mrr_area_ratio: Some(0.92),
            elongation_ratio: Some(1.3),
            uncertainty: Some(MetricUncertainty {
                area_standard_deviation_m2: Some(0.002),
                area_interval_90_m2: Some([0.036, 0.044]),
            }),
            method_build_sha256: if matches!(
                origin,
                GeometryOrigin::ModelOutput | GeometryOrigin::Simulation
            ) {
                Some(digest('c'))
            } else {
                None
            },
        }
    }

    fn element(
        id: &str,
        level: ArchitectureLevel,
        parent_id: Option<&str>,
        physical_identity_id: Option<&str>,
        observation: ArchitectureGeometryObservation,
    ) -> ArchitectureElement {
        ArchitectureElement {
            element_id: id.to_owned(),
            level,
            physical_identity_id: physical_identity_id.map(str::to_owned),
            parentage: Parentage {
                status: if parent_id.is_some() {
                    ParentageStatus::Assigned
                } else {
                    ParentageStatus::Unassigned
                },
                parent_element_id: parent_id.map(str::to_owned),
                candidate_parent_element_ids: Vec::new(),
            },
            observations: vec![observation],
        }
    }

    fn valid_input() -> HeritageArchitectureInput {
        HeritageArchitectureInput {
            schema_version: HERITAGE_ARCHITECTURE_SCHEMA_VERSION.to_owned(),
            project_id: "project-1".to_owned(),
            site_id: "site-1".to_owned(),
            snapshot_id: "snapshot-2026".to_owned(),
            coordinate_reference_id: "EPSG:32735".to_owned(),
            captures: vec![
                capture("ortho-1", CaptureModality::Orthomosaic, Some("cal-1")),
                capture("survey-1", CaptureModality::TotalStation, Some("cal-1")),
            ],
            elements: vec![
                element(
                    "wall-1",
                    ArchitectureLevel::Wall,
                    None,
                    None,
                    observation("obs-wall", "survey-1", GeometryOrigin::DirectMeasurement),
                ),
                element(
                    "segment-1",
                    ArchitectureLevel::WallSegment,
                    Some("wall-1"),
                    None,
                    observation("obs-segment", "survey-1", GeometryOrigin::DirectMeasurement),
                ),
                element(
                    "stone-1",
                    ArchitectureLevel::Stone,
                    Some("segment-1"),
                    Some("physical-stone-1"),
                    observation("obs-stone", "ortho-1", GeometryOrigin::HumanDigitization),
                ),
            ],
        }
    }

    #[test]
    fn valid_multilevel_architecture_graph_is_qualified_only_structurally() {
        let report = validate_heritage_architecture(&valid_input()).expect("valid graph");
        assert_eq!(report.element_count, 3);
        assert_eq!(report.observation_count, 3);
        assert_eq!(report.metric_observation_count, 3);
        assert_eq!(report.claim_ceiling, ArchitectureClaimCeiling::StructuralIntegrityOnly);
    }

    #[test]
    fn pixel_only_geometry_cannot_claim_physical_units() {
        let mut input = valid_input();
        input.elements[2].observations[0].measurement_basis = MeasurementBasis::PixelOnly;
        input.elements[2].observations[0].spatial_reference_id = None;
        input.elements[2].observations[0].calibration_id = None;
        input.elements[2].observations[0].metric_area_m2 = None;
        input.elements[2].observations[0].metric_area_semantics = None;
        input.elements[2].observations[0].metric_perimeter_m = None;
        input.elements[2].observations[0].uncertainty = None;
        assert!(matches!(validate_heritage_architecture(&input), Ok(_)));
        input.elements[2].observations[0].metric_area_m2 = Some(0.04);
        assert!(matches!(
            validate_heritage_architecture(&input),
            Err(HeritageArchitectureError::PixelMeasurementClaimsMetricUnits(_))
        ));
    }

    #[test]
    fn metric_area_requires_matching_calibration_and_crs() {
        let mut input = valid_input();
        input.elements[2].observations[0].calibration_id = Some("wrong-cal".to_owned());
        assert!(matches!(
            validate_heritage_architecture(&input),
            Err(HeritageArchitectureError::MetricCalibrationNotInSource(_))
        ));
    }

    #[test]
    fn model_output_requires_method_build_digest() {
        let mut input = valid_input();
        let observation = &mut input.elements[2].observations[0];
        observation.origin = GeometryOrigin::ModelOutput;
        observation.method_build_sha256 = None;
        assert!(matches!(
            validate_heritage_architecture(&input),
            Err(HeritageArchitectureError::MethodBuildDigestMissing(_))
        ));
    }

    #[test]
    fn simulated_geometry_is_counted_separately_from_model_output() {
        let mut input = valid_input();
        input.elements[2].observations[0].origin = GeometryOrigin::Simulation;
        input.elements[2].observations[0].method_build_sha256 = Some(digest('c'));
        let report = validate_heritage_architecture(&input).expect("valid simulated provenance");
        assert_eq!(report.simulation_observation_count, 1);
        assert_eq!(report.model_output_observation_count, 0);
    }

    #[test]
    fn assigned_parent_must_be_the_next_higher_architecture_level() {
        let mut input = valid_input();
        input.elements[2].parentage.parent_element_id = Some("wall-1".to_owned());
        assert!(matches!(
            validate_heritage_architecture(&input),
            Err(HeritageArchitectureError::ParentLevelMismatch(_))
        ));
    }

    #[test]
    fn ambiguous_parentage_requires_two_valid_candidates() {
        let mut input = valid_input();
        let mut alternate = input.elements[0].clone();
        alternate.element_id = "wall-2".to_owned();
        alternate.observations[0].observation_id = "obs-wall-2".to_owned();
        input.elements.push(alternate);
        let stone = &mut input.elements[2];
        stone.parentage.status = ParentageStatus::Ambiguous;
        stone.parentage.parent_element_id = None;
        stone.parentage.candidate_parent_element_ids =
            vec!["segment-1".to_owned(), "segment-1".to_owned()];
        assert!(matches!(
            validate_heritage_architecture(&input),
            Err(HeritageArchitectureError::ParentageCandidateInvalid(_))
        ));
    }

    #[test]
    fn unknown_or_pending_rights_are_not_promoted_to_verified() {
        let mut input = valid_input();
        input.captures[0].rights_state = RightsState::Verified;
        input.captures[0].rights_receipt_ref = None;
        assert!(matches!(
            validate_heritage_architecture(&input),
            Err(HeritageArchitectureError::VerifiedRightsReceiptMissing(_))
        ));
    }

    #[test]
    fn physical_stone_identity_is_not_duplicated_in_a_snapshot() {
        let mut input = valid_input();
        let mut duplicate = input.elements[2].clone();
        duplicate.element_id = "stone-duplicate".to_owned();
        duplicate.observations[0].observation_id = "obs-stone-duplicate".to_owned();
        input.elements.push(duplicate);
        assert!(matches!(
            validate_heritage_architecture(&input),
            Err(HeritageArchitectureError::DuplicatePhysicalIdentity(_))
        ));
    }

    #[test]
    fn direct_measurement_requires_a_field_metrology_capture() {
        let mut input = valid_input();
        input.captures[1].modality = CaptureModality::Orthomosaic;
        assert!(matches!(
            validate_heritage_architecture(&input),
            Err(HeritageArchitectureError::DirectMeasurementSourceMissing(_))
        ));
    }

    #[test]
    fn geometry_uncertainty_interval_must_contain_the_measurement() {
        let mut input = valid_input();
        input.elements[2].observations[0].uncertainty.as_mut().unwrap().area_interval_90_m2 =
            Some([0.05, 0.06]);
        assert!(matches!(
            validate_heritage_architecture(&input),
            Err(HeritageArchitectureError::InvalidUncertainty(_))
        ));
    }
}
