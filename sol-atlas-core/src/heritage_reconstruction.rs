// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Evidence-bounded historical landscape reconstruction packages.
//!
//! Geometry epistemic state is explicit: observed, inferred, simulated or
//! unknown. The validator checks source/dependency references, chronology
//! intervals, uncertainty, rights and coordinate-publication constraints.
//! It does not authenticate evidence, run a solver, determine historical truth,
//! or decide archaeological significance.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

pub const HERITAGE_RECONSTRUCTION_SCHEMA_VERSION: &str =
    "luminous.heritage-reconstruction.v0.1";
pub const HERITAGE_RECONSTRUCTION_REPORT_SCHEMA_VERSION: &str =
    "luminous.heritage-reconstruction-report.v0.1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EpistemicGeometryClass {
    Observed,
    Inferred,
    Simulated,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceKind {
    RemoteSensingMeasurement,
    PhotogrammetryMeasurement,
    GroundSurveyMeasurement,
    ExcavationMeasurement,
    RemoteSensingSignal,
    GeophysicalSignal,
    SurfaceFieldAssessment,
    ExcavationStratigraphy,
    MaterialAnalysis,
    ChronometricMeasurement,
    HistoricalDocument,
    ExpertInterpretation,
    ModelPrediction,
    Simulation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceOrigin {
    DirectMeasurement,
    HumanInterpretation,
    ModelOutput,
    Simulation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceRelation {
    Supports,
    Contradicts,
    ContextOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RightsStatus {
    Verified,
    Pending,
    Denied,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoordinateAccessTier {
    Public,
    Generalized,
    Controlled,
    Restricted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UncertaintyRepresentation {
    PositionalCovariance,
    BoundingEnvelope,
    Interval,
    VoxelOccupancy,
    ScenarioRange,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GeometryUncertainty {
    pub representation: UncertaintyRepresentation,
    /// Empirical or declared coverage associated with the uncertainty object.
    pub coverage_level: f64,
    /// Declared characteristic spatial scale, in metres.
    pub characteristic_extent_m: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TemporalExtentKind {
    Bounded,
    OpenStart,
    OpenEnd,
    Disputed,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TemporalInterval {
    pub kind: TemporalExtentKind,
    /// Signed CE year; negative values represent BCE. These are interval bounds,
    /// not point estimates, and must not be narrowed without chronology evidence.
    pub lower_year_ce: Option<i32>,
    pub upper_year_ce: Option<i32>,
    pub chronology_evidence_ids: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DependencyKind {
    SourceEvidence,
    MethodBuild,
    Assumption,
    Prior,
    SimulationModel,
    EnvironmentalScenario,
    SimulationManifest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReconstructionDependency {
    pub dependency_id: String,
    pub kind: DependencyKind,
    pub reference_id: Option<String>,
    pub artifact_sha256: Option<String>,
    pub statement: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReconstructionEvidence {
    pub evidence_id: String,
    pub evidence_kind: EvidenceKind,
    pub origin: EvidenceOrigin,
    pub relation: EvidenceRelation,
    pub source_asset_sha256: String,
    pub source_lineage_id: String,
    pub method_family_id: String,
    pub rights_status: RightsStatus,
    pub rights_receipt_ref: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Identifiability {
    Identifiable,
    WeaklyIdentifiable,
    NonIdentifiable,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReconstructionElement {
    pub element_id: String,
    pub epistemic_class: EpistemicGeometryClass,
    pub geometry_asset_sha256: Option<String>,
    pub spatial_reference_id: Option<String>,
    pub geometry_uncertainty: Option<GeometryUncertainty>,
    pub temporal_interval: TemporalInterval,
    pub source_evidence_ids: Vec<String>,
    pub dependencies: Vec<ReconstructionDependency>,
    pub method_build_sha256: Option<String>,
    pub simulation_manifest_sha256: Option<String>,
    pub hypothesis_family_id: Option<String>,
    pub hypothesis_id: Option<String>,
    pub identifiability: Identifiability,
    pub coordinate_access_tier: CoordinateAccessTier,
    pub exact_geometry_publication_allowed: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HeritageReconstructionInput {
    pub schema_version: String,
    pub reconstruction_id: String,
    pub reconstruction_version: String,
    pub spatial_reference_id: String,
    pub coordinate_units: String,
    pub evidence: Vec<ReconstructionEvidence>,
    pub elements: Vec<ReconstructionElement>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReconstructionDisposition {
    StructurallyQualified,
    Indeterminate,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HeritageReconstructionReport {
    pub schema_version: String,
    pub reconstruction_id: String,
    pub reconstruction_version: String,
    pub element_count: usize,
    pub observed_geometry_count: usize,
    pub inferred_geometry_count: usize,
    pub simulated_geometry_count: usize,
    pub unknown_geometry_count: usize,
    pub evidence_record_count: usize,
    pub restricted_exact_geometry_count: usize,
    pub alternative_hypothesis_family_count: usize,
    pub non_identifiable_element_count: usize,
    pub disposition: ReconstructionDisposition,
    pub all_elements_have_explicit_epistemic_class: bool,
    pub all_non_unknown_elements_have_uncertainty: bool,
    pub simulated_geometry_promoted_to_observed: bool,
    pub archaeological_fact_established: bool,
    pub historical_truth_established: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeritageReconstructionError {
    WrongSchemaVersion,
    EmptyReconstructionId,
    EmptyReconstructionVersion,
    EmptySpatialReference,
    EmptyCoordinateUnits,
    NoElements,
    TooManyElements,
    EmptyEvidenceId,
    DuplicateEvidenceId(String),
    InvalidEvidenceDigest { evidence_id: String },
    EmptyEvidenceLineage { evidence_id: String },
    EmptyEvidenceMethod { evidence_id: String },
    VerifiedEvidenceRightsReceiptMissing { evidence_id: String },
    EmptyElementId,
    DuplicateElementId(String),
    GeometryDigestMissing(String),
    InvalidGeometryDigest(String),
    SpatialReferenceMissing(String),
    UncertaintyMissing(String),
    InvalidUncertainty(String),
    UnknownElementCarriesGeometry(String),
    UnknownElementCarriesUncertainty(String),
    ObservedGeometryLacksDirectMeasurement(String),
    InferredGeometryLacksEmpiricalSupport(String),
    SimulatedGeometryLacksSimulationDependency(String),
    SimulatedGeometryMissingManifest(String),
    MethodBuildMissing(String),
    SimulationEvidenceUsedAsEmpiricalSupport(String),
    ModelEvidenceUsedAsEmpiricalSupport(String),
    SourceEvidenceReferenceMissing { element_id: String, evidence_id: String },
    SourceEvidenceDependencyMissing(String),
    DependencyIdEmpty(String),
    DuplicateDependencyId { element_id: String, dependency_id: String },
    DependencyEvidenceReferenceMissing { element_id: String, evidence_id: String },
    DependencyArtifactDigestInvalid { element_id: String },
    DependencyStatementMissing { element_id: String },
    UnknownDependencyTypeMissingReference { element_id: String },
    InvalidTemporalInterval(String),
    TemporalEvidenceReferenceMissing { element_id: String, evidence_id: String },
    TemporalEvidenceNotChronological { element_id: String, evidence_id: String },
    RestrictedCoordinatesPublished(String),
    HypothesisIdentityMissing(String),
    NonIdentifiableHypothesisHasNoAlternative(String),
    EvidenceRightsNotVerified { element_id: String, evidence_id: String },
    EvidenceRightsReceiptMissing { element_id: String, evidence_id: String },
    EvidenceNotSupportingClaim { element_id: String, evidence_id: String },
    SourceEvidenceDependencySetMismatch(String),
    MethodBuildDigestMismatch(String),
    SimulationManifestDigestMismatch(String),
}

impl fmt::Display for HeritageReconstructionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "heritage reconstruction error: {self:?}")
    }
}

impl std::error::Error for HeritageReconstructionError {}

fn non_empty(value: &str) -> bool {
    !value.trim().is_empty()
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn is_direct_measurement(evidence: &ReconstructionEvidence) -> bool {
    evidence.origin == EvidenceOrigin::DirectMeasurement
        && matches!(
            evidence.evidence_kind,
            EvidenceKind::RemoteSensingMeasurement
                | EvidenceKind::PhotogrammetryMeasurement
                | EvidenceKind::GroundSurveyMeasurement
                | EvidenceKind::ExcavationMeasurement
        )
}

fn validate_empirical_support(
    element_id: &str,
    evidence: &ReconstructionEvidence,
) -> Result<(), HeritageReconstructionError> {
    if evidence.relation != EvidenceRelation::Supports {
        return Err(HeritageReconstructionError::EvidenceNotSupportingClaim {
            element_id: element_id.to_owned(),
            evidence_id: evidence.evidence_id.clone(),
        });
    }
    if evidence.rights_status != RightsStatus::Verified {
        return Err(HeritageReconstructionError::EvidenceRightsNotVerified {
            element_id: element_id.to_owned(),
            evidence_id: evidence.evidence_id.clone(),
        });
    }
    if evidence.rights_receipt_ref.as_deref().is_none_or(|value| !non_empty(value)) {
        return Err(HeritageReconstructionError::EvidenceRightsReceiptMissing {
            element_id: element_id.to_owned(),
            evidence_id: evidence.evidence_id.clone(),
        });
    }
    if evidence.origin == EvidenceOrigin::ModelOutput
        || evidence.evidence_kind == EvidenceKind::ModelPrediction
    {
        return Err(HeritageReconstructionError::ModelEvidenceUsedAsEmpiricalSupport(
            element_id.to_owned(),
        ));
    }
    if evidence.origin == EvidenceOrigin::Simulation
        || evidence.evidence_kind == EvidenceKind::Simulation
    {
        return Err(HeritageReconstructionError::SimulationEvidenceUsedAsEmpiricalSupport(
            element_id.to_owned(),
        ));
    }
    Ok(())
}

fn validate_temporal_interval(
    element: &ReconstructionElement,
    evidence_by_id: &BTreeMap<String, &ReconstructionEvidence>,
) -> Result<(), HeritageReconstructionError> {
    let interval = &element.temporal_interval;
    let has_bounds = interval.lower_year_ce.is_some() || interval.upper_year_ce.is_some();
    let has_chronology = !interval.chronology_evidence_ids.is_empty();

    match interval.kind {
        TemporalExtentKind::Bounded => {
            if interval.lower_year_ce.is_none()
                || interval.upper_year_ce.is_none()
                || interval.lower_year_ce > interval.upper_year_ce
                || !has_chronology
            {
                return Err(HeritageReconstructionError::InvalidTemporalInterval(
                    element.element_id.clone(),
                ));
            }
        }
        TemporalExtentKind::OpenStart => {
            if interval.lower_year_ce.is_some() || interval.upper_year_ce.is_none() || !has_chronology {
                return Err(HeritageReconstructionError::InvalidTemporalInterval(
                    element.element_id.clone(),
                ));
            }
        }
        TemporalExtentKind::OpenEnd => {
            if interval.lower_year_ce.is_none() || interval.upper_year_ce.is_some() || !has_chronology {
                return Err(HeritageReconstructionError::InvalidTemporalInterval(
                    element.element_id.clone(),
                ));
            }
        }
        TemporalExtentKind::Disputed => {
            if !has_bounds || !has_chronology {
                return Err(HeritageReconstructionError::InvalidTemporalInterval(
                    element.element_id.clone(),
                ));
            }
        }
        TemporalExtentKind::Unknown => {
            if has_bounds || has_chronology {
                return Err(HeritageReconstructionError::InvalidTemporalInterval(
                    element.element_id.clone(),
                ));
            }
        }
    }

    let mut seen = BTreeSet::new();
    for evidence_id in &interval.chronology_evidence_ids {
        if !seen.insert(evidence_id) {
            return Err(HeritageReconstructionError::InvalidTemporalInterval(
                element.element_id.clone(),
            ));
        }
        let Some(evidence) = evidence_by_id.get(evidence_id) else {
            return Err(
                HeritageReconstructionError::TemporalEvidenceReferenceMissing {
                    element_id: element.element_id.clone(),
                    evidence_id: evidence_id.clone(),
                },
            );
        };
        if !matches!(
            evidence.evidence_kind,
            EvidenceKind::ChronometricMeasurement | EvidenceKind::HistoricalDocument
        ) {
            return Err(HeritageReconstructionError::TemporalEvidenceNotChronological {
                element_id: element.element_id.clone(),
                evidence_id: evidence_id.clone(),
            });
        }
        validate_empirical_support(&element.element_id, evidence)?;
    }
    Ok(())
}

fn validate_element(
    element: &ReconstructionElement,
    evidence_by_id: &BTreeMap<String, &ReconstructionEvidence>,
) -> Result<(), HeritageReconstructionError> {
    if !non_empty(&element.element_id) {
        return Err(HeritageReconstructionError::EmptyElementId);
    }
    if !non_empty(&element.spatial_reference_id)
        && element.epistemic_class != EpistemicGeometryClass::Unknown
    {
        return Err(HeritageReconstructionError::SpatialReferenceMissing(
            element.element_id.clone(),
        ));
    }

    let mut source_evidence = Vec::new();
    let mut source_seen = BTreeSet::new();
    for evidence_id in &element.source_evidence_ids {
        if !source_seen.insert(evidence_id.clone()) {
            return Err(HeritageReconstructionError::SourceEvidenceReferenceMissing {
                element_id: element.element_id.clone(),
                evidence_id: evidence_id.clone(),
            });
        }
        let Some(evidence) = evidence_by_id.get(evidence_id) else {
            return Err(HeritageReconstructionError::SourceEvidenceReferenceMissing {
                element_id: element.element_id.clone(),
                evidence_id: evidence_id.clone(),
            });
        };
        validate_empirical_support(&element.element_id, evidence)?;
        source_evidence.push(*evidence);
    }

    let mut dependency_ids = BTreeSet::new();
    let mut source_dependency_ids = BTreeSet::new();
    let mut method_dependency_digests = BTreeSet::new();
    let mut simulation_dependency_digests = BTreeSet::new();
    let mut simulation_manifest_digests = BTreeSet::new();
    for dependency in &element.dependencies {
        if !non_empty(&dependency.dependency_id) {
            return Err(HeritageReconstructionError::DependencyIdEmpty(
                element.element_id.clone(),
            ));
        }
        if !dependency_ids.insert(&dependency.dependency_id) {
            return Err(HeritageReconstructionError::DuplicateDependencyId {
                element_id: element.element_id.clone(),
                dependency_id: dependency.dependency_id.clone(),
            });
        }
        if !non_empty(&dependency.statement) {
            return Err(HeritageReconstructionError::DependencyStatementMissing {
                element_id: element.element_id.clone(),
            });
        }
        match dependency.kind {
            DependencyKind::SourceEvidence => {
                let Some(reference_id) = dependency.reference_id.as_deref() else {
                    return Err(HeritageReconstructionError::UnknownDependencyTypeMissingReference {
                        element_id: element.element_id.clone(),
                    });
                };
                if !evidence_by_id.contains_key(reference_id) {
                    return Err(HeritageReconstructionError::DependencyEvidenceReferenceMissing {
                        element_id: element.element_id.clone(),
                        evidence_id: reference_id.to_owned(),
                    });
                }
                source_dependency_ids.insert(reference_id.to_owned());
            }
            DependencyKind::MethodBuild => {
                let Some(digest) = dependency.artifact_sha256.as_deref() else {
                    return Err(HeritageReconstructionError::DependencyArtifactDigestInvalid {
                        element_id: element.element_id.clone(),
                    });
                };
                if !valid_sha256(digest) {
                    return Err(HeritageReconstructionError::DependencyArtifactDigestInvalid {
                        element_id: element.element_id.clone(),
                    });
                }
                method_dependency_digests.insert(digest.to_owned());
            }
            DependencyKind::SimulationModel | DependencyKind::EnvironmentalScenario => {
                let Some(digest) = dependency.artifact_sha256.as_deref() else {
                    return Err(HeritageReconstructionError::DependencyArtifactDigestInvalid {
                        element_id: element.element_id.clone(),
                    });
                };
                if !valid_sha256(digest) {
                    return Err(HeritageReconstructionError::DependencyArtifactDigestInvalid {
                        element_id: element.element_id.clone(),
                    });
                }
                simulation_dependency_digests.insert(digest.to_owned());
            }
            DependencyKind::SimulationManifest => {
                let Some(digest) = dependency.artifact_sha256.as_deref() else {
                    return Err(HeritageReconstructionError::DependencyArtifactDigestInvalid {
                        element_id: element.element_id.clone(),
                    });
                };
                if !valid_sha256(digest) {
                    return Err(HeritageReconstructionError::DependencyArtifactDigestInvalid {
                        element_id: element.element_id.clone(),
                    });
                }
                simulation_manifest_digests.insert(digest.to_owned());
            }
            DependencyKind::Assumption | DependencyKind::Prior => {
                if dependency.reference_id.as_deref().is_none_or(|value| !non_empty(value)) {
                    return Err(HeritageReconstructionError::UnknownDependencyTypeMissingReference {
                        element_id: element.element_id.clone(),
                    });
                }
            }
        }
    }
    if source_dependency_ids != source_seen {
        return Err(HeritageReconstructionError::SourceEvidenceDependencySetMismatch(
            element.element_id.clone(),
        ));
    }

    match element.epistemic_class {
        EpistemicGeometryClass::Observed => {
            if element.geometry_asset_sha256.as_deref().is_none_or(|value| !valid_sha256(value)) {
                return Err(HeritageReconstructionError::GeometryDigestMissing(
                    element.element_id.clone(),
                ));
            }
            if element.geometry_uncertainty.is_none() {
                return Err(HeritageReconstructionError::UncertaintyMissing(
                    element.element_id.clone(),
                ));
            }
            if !source_evidence.iter().any(|item| is_direct_measurement(item)) {
                return Err(HeritageReconstructionError::ObservedGeometryLacksDirectMeasurement(
                    element.element_id.clone(),
                ));
            }
            let Some(method_digest) = element.method_build_sha256.as_deref() else {
                return Err(HeritageReconstructionError::MethodBuildMissing(
                    element.element_id.clone(),
                ));
            };
            if !valid_sha256(method_digest) {
                return Err(HeritageReconstructionError::MethodBuildMissing(
                    element.element_id.clone(),
                ));
            }
            if !method_dependency_digests.contains(method_digest) {
                return Err(HeritageReconstructionError::MethodBuildDigestMismatch(
                    element.element_id.clone(),
                ));
            }
        }
        EpistemicGeometryClass::Inferred => {
            if element.geometry_asset_sha256.as_deref().is_none_or(|value| !valid_sha256(value)) {
                return Err(HeritageReconstructionError::GeometryDigestMissing(
                    element.element_id.clone(),
                ));
            }
            if element.geometry_uncertainty.is_none() {
                return Err(HeritageReconstructionError::UncertaintyMissing(
                    element.element_id.clone(),
                ));
            }
            if source_evidence.is_empty() {
                return Err(HeritageReconstructionError::InferredGeometryLacksEmpiricalSupport(
                    element.element_id.clone(),
                ));
            }
            let Some(method_digest) = element.method_build_sha256.as_deref() else {
                return Err(HeritageReconstructionError::MethodBuildMissing(
                    element.element_id.clone(),
                ));
            };
            if !valid_sha256(method_digest) {
                return Err(HeritageReconstructionError::MethodBuildMissing(
                    element.element_id.clone(),
                ));
            }
            if !method_dependency_digests.contains(method_digest) {
                return Err(HeritageReconstructionError::MethodBuildDigestMismatch(
                    element.element_id.clone(),
                ));
            }
            if source_evidence.iter().any(|item| {
                item.origin == EvidenceOrigin::Simulation
                    || item.evidence_kind == EvidenceKind::Simulation
            }) {
                return Err(HeritageReconstructionError::SimulationEvidenceUsedAsEmpiricalSupport(
                    element.element_id.clone(),
                ));
            }
            if source_evidence.iter().any(|item| {
                item.origin == EvidenceOrigin::ModelOutput
                    || item.evidence_kind == EvidenceKind::ModelPrediction
            }) {
                return Err(HeritageReconstructionError::ModelEvidenceUsedAsEmpiricalSupport(
                    element.element_id.clone(),
                ));
            }
            let Some(method_digest) = element.method_build_sha256.as_deref() else {
                return Err(HeritageReconstructionError::MethodBuildMissing(
                    element.element_id.clone(),
                ));
            };
            if !valid_sha256(method_digest) {
                return Err(HeritageReconstructionError::MethodBuildMissing(
                    element.element_id.clone(),
                ));
            }
            if !method_dependency_digests.contains(method_digest) {
                return Err(HeritageReconstructionError::MethodBuildDigestMismatch(
                    element.element_id.clone(),
                ));
            }
        }
        EpistemicGeometryClass::Simulated => {
            if element.geometry_asset_sha256.as_deref().is_none_or(|value| !valid_sha256(value)) {
                return Err(HeritageReconstructionError::GeometryDigestMissing(
                    element.element_id.clone(),
                ));
            }
            if element.geometry_uncertainty.is_none() {
                return Err(HeritageReconstructionError::UncertaintyMissing(
                    element.element_id.clone(),
                ));
            }
            let Some(method_digest) = element.method_build_sha256.as_deref() else {
                return Err(HeritageReconstructionError::MethodBuildMissing(
                    element.element_id.clone(),
                ));
            };
            if !valid_sha256(method_digest) {
                return Err(HeritageReconstructionError::MethodBuildMissing(
                    element.element_id.clone(),
                ));
            }
            if !method_dependency_digests.contains(method_digest) {
                return Err(HeritageReconstructionError::MethodBuildDigestMismatch(
                    element.element_id.clone(),
                ));
            }
            let Some(simulation_digest) = element.simulation_manifest_sha256.as_deref() else {
                return Err(HeritageReconstructionError::SimulatedGeometryMissingManifest(
                    element.element_id.clone(),
                ));
            };
            if !valid_sha256(simulation_digest)
                || !simulation_manifest_digests.contains(simulation_digest)
                || simulation_dependency_digests.is_empty()
            {
                return Err(HeritageReconstructionError::SimulationManifestDigestMismatch(
                    element.element_id.clone(),
                ));
            }
            if !source_evidence.iter().all(|item| {
                matches!(
                    item.origin,
                    EvidenceOrigin::DirectMeasurement | EvidenceOrigin::HumanInterpretation
                )
            }) {
                return Err(HeritageReconstructionError::SimulationEvidenceUsedAsEmpiricalSupport(
                    element.element_id.clone(),
                ));
            }
        }
        EpistemicGeometryClass::Unknown => {
            if element.geometry_asset_sha256.is_some() || element.spatial_reference_id.is_some() {
                return Err(HeritageReconstructionError::UnknownElementCarriesGeometry(
                    element.element_id.clone(),
                ));
            }
            if element.geometry_uncertainty.is_some() {
                return Err(HeritageReconstructionError::UnknownElementCarriesUncertainty(
                    element.element_id.clone(),
                ));
            }
        }
    }

    if let Some(uncertainty) = element.geometry_uncertainty {
        if !uncertainty.coverage_level.is_finite()
            || uncertainty.coverage_level <= 0.0
            || uncertainty.coverage_level >= 1.0
            || !uncertainty.characteristic_extent_m.is_finite()
            || uncertainty.characteristic_extent_m <= 0.0
        {
            return Err(HeritageReconstructionError::InvalidUncertainty(
                element.element_id.clone(),
            ));
        }
        if element.epistemic_class == EpistemicGeometryClass::Simulated
            && uncertainty.representation != UncertaintyRepresentation::ScenarioRange
        {
            return Err(HeritageReconstructionError::InvalidUncertainty(
                element.element_id.clone(),
            ));
        }
    }

    validate_temporal_interval(element, evidence_by_id)?;

    if element.coordinate_access_tier != CoordinateAccessTier::Public
        && element.exact_geometry_publication_allowed
    {
        return Err(HeritageReconstructionError::RestrictedCoordinatesPublished(
            element.element_id.clone(),
        ));
    }
    if element.identifiability != Identifiability::Identifiable {
        if element.hypothesis_family_id.as_deref().is_none_or(|value| !non_empty(value))
            || element.hypothesis_id.as_deref().is_none_or(|value| !non_empty(value))
        {
            return Err(HeritageReconstructionError::HypothesisIdentityMissing(
                element.element_id.clone(),
            ));
        }
    }
    Ok(())
}

pub fn validate_heritage_reconstruction(
    input: &HeritageReconstructionInput,
) -> Result<HeritageReconstructionReport, HeritageReconstructionError> {
    if input.schema_version != HERITAGE_RECONSTRUCTION_SCHEMA_VERSION {
        return Err(HeritageReconstructionError::WrongSchemaVersion);
    }
    if !non_empty(&input.reconstruction_id) {
        return Err(HeritageReconstructionError::EmptyReconstructionId);
    }
    if !non_empty(&input.reconstruction_version) {
        return Err(HeritageReconstructionError::EmptyReconstructionVersion);
    }
    if !non_empty(&input.spatial_reference_id) {
        return Err(HeritageReconstructionError::EmptySpatialReference);
    }
    if !non_empty(&input.coordinate_units) {
        return Err(HeritageReconstructionError::EmptyCoordinateUnits);
    }
    if input.elements.is_empty() {
        return Err(HeritageReconstructionError::NoElements);
    }
    if input.elements.len() > 100_000 {
        return Err(HeritageReconstructionError::TooManyElements);
    }

    let mut evidence_by_id = BTreeMap::new();
    for evidence in &input.evidence {
        if !non_empty(&evidence.evidence_id) {
            return Err(HeritageReconstructionError::EmptyEvidenceId);
        }
        if evidence_by_id.insert(evidence.evidence_id.clone(), evidence).is_some() {
            return Err(HeritageReconstructionError::DuplicateEvidenceId(
                evidence.evidence_id.clone(),
            ));
        }
        if !valid_sha256(&evidence.source_asset_sha256) {
            return Err(HeritageReconstructionError::InvalidEvidenceDigest {
                evidence_id: evidence.evidence_id.clone(),
            });
        }
        if !non_empty(&evidence.source_lineage_id) {
            return Err(HeritageReconstructionError::EmptyEvidenceLineage {
                evidence_id: evidence.evidence_id.clone(),
            });
        }
        if !non_empty(&evidence.method_family_id) {
            return Err(HeritageReconstructionError::EmptyEvidenceMethod {
                evidence_id: evidence.evidence_id.clone(),
            });
        }
        if evidence.rights_status == RightsStatus::Verified
            && evidence.rights_receipt_ref.as_deref().is_none_or(|value| !non_empty(value))
        {
            return Err(HeritageReconstructionError::VerifiedEvidenceRightsReceiptMissing {
                evidence_id: evidence.evidence_id.clone(),
            });
        }
    }

    let mut element_ids = BTreeSet::new();
    let mut hypothesis_families: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut observed_count = 0;
    let mut inferred_count = 0;
    let mut simulated_count = 0;
    let mut unknown_count = 0;
    let mut restricted_exact_count = 0;
    let mut non_identifiable_count = 0;

    for element in &input.elements {
        if !element_ids.insert(element.element_id.clone()) {
            return Err(HeritageReconstructionError::DuplicateElementId(
                element.element_id.clone(),
            ));
        }
        validate_element(element, &evidence_by_id)?;

        match element.epistemic_class {
            EpistemicGeometryClass::Observed => observed_count += 1,
            EpistemicGeometryClass::Inferred => inferred_count += 1,
            EpistemicGeometryClass::Simulated => simulated_count += 1,
            EpistemicGeometryClass::Unknown => unknown_count += 1,
        }
        if element.coordinate_access_tier != CoordinateAccessTier::Public {
            restricted_exact_count += 1;
        }
        if element.identifiability != Identifiability::Identifiable {
            non_identifiable_count += 1;
            if let (Some(family), Some(hypothesis)) =
                (&element.hypothesis_family_id, &element.hypothesis_id)
            {
                hypothesis_families
                    .entry(family.clone())
                    .or_default()
                    .insert(hypothesis.clone());
            }
        }
    }

    for element in &input.elements {
        if element.identifiability == Identifiability::NonIdentifiable {
            let family = element
                .hypothesis_family_id
                .as_deref()
                .unwrap_or_default();
            if hypothesis_families.get(family).is_none_or(|items| items.len() < 2) {
                return Err(
                    HeritageReconstructionError::NonIdentifiableHypothesisHasNoAlternative(
                        element.element_id.clone(),
                    ),
                );
            }
        }
    }

    let alternative_family_count = hypothesis_families
        .values()
        .filter(|hypotheses| hypotheses.len() >= 2)
        .count();

    Ok(HeritageReconstructionReport {
        schema_version: HERITAGE_RECONSTRUCTION_REPORT_SCHEMA_VERSION.to_owned(),
        reconstruction_id: input.reconstruction_id.clone(),
        reconstruction_version: input.reconstruction_version.clone(),
        element_count: input.elements.len(),
        observed_geometry_count: observed_count,
        inferred_geometry_count: inferred_count,
        simulated_geometry_count: simulated_count,
        unknown_geometry_count: unknown_count,
        evidence_record_count: input.evidence.len(),
        restricted_exact_geometry_count: restricted_exact_count,
        alternative_hypothesis_family_count: alternative_family_count,
        non_identifiable_element_count: non_identifiable_count,
        disposition: ReconstructionDisposition::StructurallyQualified,
        all_elements_have_explicit_epistemic_class: true,
        all_non_unknown_elements_have_uncertainty: true,
        simulated_geometry_promoted_to_observed: false,
        archaeological_fact_established: false,
        historical_truth_established: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn evidence(
        id: &str,
        kind: EvidenceKind,
        origin: EvidenceOrigin,
        digest_char: char,
    ) -> ReconstructionEvidence {
        ReconstructionEvidence {
            evidence_id: id.into(),
            evidence_kind: kind,
            origin,
            relation: EvidenceRelation::Supports,
            source_asset_sha256: digest_char.to_string().repeat(64),
            source_lineage_id: format!("lineage-{id}"),
            method_family_id: format!("method-{id}"),
            rights_status: RightsStatus::Verified,
            rights_receipt_ref: Some(format!("urn:synthetic:rights:{id}")),
        }
    }

    fn uncertainty(kind: UncertaintyRepresentation) -> GeometryUncertainty {
        GeometryUncertainty {
            representation: kind,
            coverage_level: 0.90,
            characteristic_extent_m: 0.25,
        }
    }

    fn observed_element() -> ReconstructionElement {
        ReconstructionElement {
            element_id: "surface-dem".into(),
            epistemic_class: EpistemicGeometryClass::Observed,
            geometry_asset_sha256: Some("1".repeat(64)),
            spatial_reference_id: Some("EPSG:32736".into()),
            geometry_uncertainty: Some(uncertainty(
                UncertaintyRepresentation::PositionalCovariance,
            )),
            temporal_interval: TemporalInterval {
                kind: TemporalExtentKind::Unknown,
                lower_year_ce: None,
                upper_year_ce: None,
                chronology_evidence_ids: vec![],
            },
            source_evidence_ids: vec!["lidar-measurement".into()],
            dependencies: vec![
                ReconstructionDependency {
                    dependency_id: "source-lidar".into(),
                    kind: DependencyKind::SourceEvidence,
                    reference_id: Some("lidar-measurement".into()),
                    artifact_sha256: None,
                    statement: "Direct LiDAR-derived surface measurement".into(),
                },
                ReconstructionDependency {
                    dependency_id: "dem-build".into(),
                    kind: DependencyKind::MethodBuild,
                    reference_id: Some("dem-builder-v1".into()),
                    artifact_sha256: Some("2".repeat(64)),
                    statement: "Declared terrain reconstruction pipeline".into(),
                },
            ],
            method_build_sha256: Some("2".repeat(64)),
            simulation_manifest_sha256: None,
            hypothesis_family_id: None,
            hypothesis_id: None,
            identifiability: Identifiability::Identifiable,
            coordinate_access_tier: CoordinateAccessTier::Public,
            exact_geometry_publication_allowed: true,
        }
    }

    fn input() -> HeritageReconstructionInput {
        HeritageReconstructionInput {
            schema_version: HERITAGE_RECONSTRUCTION_SCHEMA_VERSION.into(),
            reconstruction_id: "landscape-001".into(),
            reconstruction_version: "v0.1".into(),
            spatial_reference_id: "EPSG:32736".into(),
            coordinate_units: "metres".into(),
            evidence: vec![
                evidence(
                    "lidar-measurement",
                    EvidenceKind::RemoteSensingMeasurement,
                    EvidenceOrigin::DirectMeasurement,
                    'a',
                ),
                evidence(
                    "field-assessment",
                    EvidenceKind::SurfaceFieldAssessment,
                    EvidenceOrigin::HumanInterpretation,
                    'b',
                ),
                evidence(
                    "chronology",
                    EvidenceKind::ChronometricMeasurement,
                    EvidenceOrigin::DirectMeasurement,
                    'c',
                ),
                evidence(
                    "simulation",
                    EvidenceKind::Simulation,
                    EvidenceOrigin::Simulation,
                    'd',
                ),
            ],
            elements: vec![observed_element()],
        }
    }

    #[test]
    fn observed_geometry_requires_and_preserves_direct_measurement() {
        let report = validate_heritage_reconstruction(&input()).unwrap();
        assert_eq!(report.observed_geometry_count, 1);
        assert_eq!(report.inferred_geometry_count, 0);
        assert!(!report.archaeological_fact_established);
        assert!(!report.historical_truth_established);
    }

    #[test]
    fn inferred_geometry_requires_empirical_support_and_method_lineage() {
        let mut value = input();
        let mut element = observed_element();
        element.element_id = "inferred-wall".into();
        element.epistemic_class = EpistemicGeometryClass::Inferred;
        element.source_evidence_ids = vec!["field-assessment".into()];
        element.dependencies = vec![
            ReconstructionDependency {
                dependency_id: "field-source".into(),
                kind: DependencyKind::SourceEvidence,
                reference_id: Some("field-assessment".into()),
                artifact_sha256: None,
                statement: "Independent surface survey interpretation".into(),
            },
            ReconstructionDependency {
                dependency_id: "inference-method".into(),
                kind: DependencyKind::MethodBuild,
                reference_id: Some("wall-model-v1".into()),
                artifact_sha256: Some("3".repeat(64)),
                statement: "Declared wall geometry inference".into(),
            },
        ];
        element.method_build_sha256 = Some("3".repeat(64));
        value.elements = vec![element];
        let report = validate_heritage_reconstruction(&value).unwrap();
        assert_eq!(report.inferred_geometry_count, 1);
    }

    #[test]
    fn simulation_is_explicit_and_never_counted_as_observed() {
        let mut value = input();
        let mut element = observed_element();
        element.element_id = "climate-scenario".into();
        element.epistemic_class = EpistemicGeometryClass::Simulated;
        element.source_evidence_ids = vec!["lidar-measurement".into()];
        element.geometry_uncertainty = Some(uncertainty(UncertaintyRepresentation::ScenarioRange));
        element.dependencies = vec![
            ReconstructionDependency {
                dependency_id: "source-lidar".into(),
                kind: DependencyKind::SourceEvidence,
                reference_id: Some("lidar-measurement".into()),
                artifact_sha256: None,
                statement: "Initial measured terrain conditions".into(),
            },
            ReconstructionDependency {
                dependency_id: "model-build".into(),
                kind: DependencyKind::MethodBuild,
                reference_id: Some("symtropy-climate-v1".into()),
                artifact_sha256: Some("4".repeat(64)),
                statement: "Declared forward model".into(),
            },
            ReconstructionDependency {
                dependency_id: "simulation-model".into(),
                kind: DependencyKind::SimulationModel,
                reference_id: Some("symtropy-climate-v1".into()),
                artifact_sha256: Some("4".repeat(64)),
                statement: "Simulation model identity".into(),
            },
            ReconstructionDependency {
                dependency_id: "simulation-manifest".into(),
                kind: DependencyKind::SimulationManifest,
                reference_id: Some("simulation-run-001".into()),
                artifact_sha256: Some("5".repeat(64)),
                statement: "Exact simulation run manifest".into(),
            },
        ];
        element.method_build_sha256 = Some("4".repeat(64));
        element.simulation_manifest_sha256 = Some("5".repeat(64));
        value.elements = vec![element];
        let report = validate_heritage_reconstruction(&value).unwrap();
        assert_eq!(report.simulated_geometry_count, 1);
        assert_eq!(report.observed_geometry_count, 0);
        assert!(!report.simulated_geometry_promoted_to_observed);
    }

    #[test]
    fn unknown_geometry_has_no_asset_or_uncertainty() {
        let mut value = input();
        value.elements = vec![ReconstructionElement {
            element_id: "unresolved-area".into(),
            epistemic_class: EpistemicGeometryClass::Unknown,
            geometry_asset_sha256: None,
            spatial_reference_id: None,
            geometry_uncertainty: None,
            temporal_interval: TemporalInterval {
                kind: TemporalExtentKind::Unknown,
                lower_year_ce: None,
                upper_year_ce: None,
                chronology_evidence_ids: vec![],
            },
            source_evidence_ids: vec![],
            dependencies: vec![],
            method_build_sha256: None,
            simulation_manifest_sha256: None,
            hypothesis_family_id: None,
            hypothesis_id: None,
            identifiability: Identifiability::NonIdentifiable,
            coordinate_access_tier: CoordinateAccessTier::Restricted,
            exact_geometry_publication_allowed: false,
        }];
        // Unknown with non-identifiable status needs explicit alternatives.
        value.elements[0].identifiability = Identifiability::Identifiable;
        let report = validate_heritage_reconstruction(&value).unwrap();
        assert_eq!(report.unknown_geometry_count, 1);
    }

    #[test]
    fn simulation_cannot_be_serialized_as_observed_geometry() {
        let mut value = input();
        value.evidence[0].origin = EvidenceOrigin::Simulation;
        value.evidence[0].evidence_kind = EvidenceKind::Simulation;
        assert!(matches!(
            validate_heritage_reconstruction(&value),
            Err(HeritageReconstructionError::SimulationEvidenceUsedAsEmpiricalSupport(_))
        ));
    }

    #[test]
    fn unknown_geometry_cannot_carry_geometry_asset() {
        let mut value = input();
        let mut element = observed_element();
        element.epistemic_class = EpistemicGeometryClass::Unknown;
        value.elements = vec![element];
        assert!(matches!(
            validate_heritage_reconstruction(&value),
            Err(HeritageReconstructionError::UnknownElementCarriesGeometry(_))
        ));
    }

    #[test]
    fn inferred_geometry_cannot_use_model_only_evidence_as_empirical_support() {
        let mut value = input();
        value.evidence.push(evidence(
            "model-prediction",
            EvidenceKind::ModelPrediction,
            EvidenceOrigin::ModelOutput,
            'e',
        ));
        let mut element = observed_element();
        element.epistemic_class = EpistemicGeometryClass::Inferred;
        element.source_evidence_ids = vec!["model-prediction".into()];
        element.dependencies[0].reference_id = Some("model-prediction".into());
        value.elements = vec![element];
        assert!(matches!(
            validate_heritage_reconstruction(&value),
            Err(HeritageReconstructionError::ModelEvidenceUsedAsEmpiricalSupport(_))
        ));
    }

    #[test]
    fn restricted_coordinates_cannot_be_published_exactly() {
        let mut value = input();
        value.elements[0].coordinate_access_tier = CoordinateAccessTier::Restricted;
        assert!(matches!(
            validate_heritage_reconstruction(&value),
            Err(HeritageReconstructionError::RestrictedCoordinatesPublished(_))
        ));
    }

    #[test]
    fn bounded_chronology_requires_chronology_evidence() {
        let mut value = input();
        value.elements[0].temporal_interval = TemporalInterval {
            kind: TemporalExtentKind::Bounded,
            lower_year_ce: Some(-300),
            upper_year_ce: Some(-100),
            chronology_evidence_ids: vec![],
        };
        assert!(matches!(
            validate_heritage_reconstruction(&value),
            Err(HeritageReconstructionError::InvalidTemporalInterval(_))
        ));
    }

    #[test]
    fn exact_year_is_not_invented_from_unknown_chronology() {
        let mut value = input();
        value.elements[0].temporal_interval.lower_year_ce = Some(250);
        assert!(matches!(
            validate_heritage_reconstruction(&value),
            Err(HeritageReconstructionError::InvalidTemporalInterval(_))
        ));
    }

    #[test]
    fn bounded_interval_must_reference_chronology_evidence() {
        let mut value = input();
        value.elements[0].temporal_interval = TemporalInterval {
            kind: TemporalExtentKind::Bounded,
            lower_year_ce: Some(-300),
            upper_year_ce: Some(-100),
            chronology_evidence_ids: vec!["chronology".into()],
        };
        let report = validate_heritage_reconstruction(&value).unwrap();
        assert_eq!(report.element_count, 1);
    }

    #[test]
    fn open_temporal_interval_preserves_one_unknown_bound() {
        let mut value = input();
        value.elements[0].temporal_interval = TemporalInterval {
            kind: TemporalExtentKind::OpenEnd,
            lower_year_ce: Some(350),
            upper_year_ce: None,
            chronology_evidence_ids: vec!["chronology".into()],
        };
        assert!(validate_heritage_reconstruction(&value).is_ok());
    }

    #[test]
    fn non_identifiable_hypotheses_require_multiple_alternatives() {
        let mut value = input();
        value.elements[0].identifiability = Identifiability::NonIdentifiable;
        value.elements[0].hypothesis_family_id = Some("settlement-layout".into());
        value.elements[0].hypothesis_id = Some("layout-a".into());
        assert!(matches!(
            validate_heritage_reconstruction(&value),
            Err(HeritageReconstructionError::NonIdentifiableHypothesisHasNoAlternative(_))
        ));
        let mut alternative = value.elements[0].clone();
        alternative.element_id = "layout-alternative".into();
        alternative.hypothesis_id = Some("layout-b".into());
        value.elements.push(alternative);
        let report = validate_heritage_reconstruction(&value).unwrap();
        assert_eq!(report.alternative_hypothesis_family_count, 1);
    }

    #[test]
    fn missing_evidence_dependency_fails_closed() {
        let mut value = input();
        value.elements[0].dependencies[0].reference_id = Some("missing-evidence".into());
        assert!(matches!(
            validate_heritage_reconstruction(&value),
            Err(HeritageReconstructionError::DependencyEvidenceReferenceMissing { .. })
        ));
    }

    #[test]
    fn simulated_geometry_requires_scenario_uncertainty() {
        let mut value = input();
        let mut element = observed_element();
        element.epistemic_class = EpistemicGeometryClass::Simulated;
        element.geometry_uncertainty = Some(uncertainty(
            UncertaintyRepresentation::BoundingEnvelope,
        ));
        element.method_build_sha256 = Some("4".repeat(64));
        element.simulation_manifest_sha256 = Some("5".repeat(64));
        element.dependencies = vec![
            ReconstructionDependency {
                dependency_id: "source-lidar".into(),
                kind: DependencyKind::SourceEvidence,
                reference_id: Some("lidar-measurement".into()),
                artifact_sha256: None,
                statement: "Initial measured terrain conditions".into(),
            },
            ReconstructionDependency {
                dependency_id: "method".into(),
                kind: DependencyKind::MethodBuild,
                reference_id: Some("sim-v1".into()),
                artifact_sha256: Some("4".repeat(64)),
                statement: "simulation method".into(),
            },
            ReconstructionDependency {
                dependency_id: "simulation".into(),
                kind: DependencyKind::SimulationModel,
                reference_id: Some("sim-v1".into()),
                artifact_sha256: Some("4".repeat(64)),
                statement: "simulation model".into(),
            },
            ReconstructionDependency {
                dependency_id: "simulation-manifest".into(),
                kind: DependencyKind::SimulationManifest,
                reference_id: Some("simulation-run-001".into()),
                artifact_sha256: Some("5".repeat(64)),
                statement: "Exact simulation run manifest".into(),
            },
        ];
        value.elements = vec![element];
        assert!(matches!(
            validate_heritage_reconstruction(&value),
            Err(HeritageReconstructionError::InvalidUncertainty(_))
        ));
    }

    #[test]
    fn synthetic_observed_fixture_and_report_round_trip() {
        let input: HeritageReconstructionInput = serde_json::from_str(include_str!(
            "../../schemas/examples/heritage-reconstruction-input-synthetic-v0.1.json"
        ))
        .unwrap();
        let actual = validate_heritage_reconstruction(&input).unwrap();
        let expected: HeritageReconstructionReport = serde_json::from_str(include_str!(
            "../../schemas/examples/heritage-reconstruction-report-synthetic-v0.1.json"
        ))
        .unwrap();
        assert_eq!(actual, expected);
    }
}
