// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Evidence-qualified metadata for 2D/3D territorial geometry references.
//!
//! This module does not parse or validate raw geometry, geodetic transformations,
//! survey measurements, legal instruments or external registry records. It
//! checks whether the metadata and referenced qualification evidence are
//! complete enough for the next review stage. Every outcome has the fixed
//! reference_only_no_legal_effect ceiling.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub const SPATIAL_QUALIFICATION_SCHEMA: &str =
    "luminous.territorial-spatial-qualification.v0";
pub const SPATIAL_QUALIFICATION_SCHEMA_URI: &str =
    "https://luminousdynamics.io/schemas/territorial-spatial-qualification-v0.schema.json";
pub const SPATIAL_QUALIFICATION_CLAIM_CEILING: &str = "reference_only_no_legal_effect";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpatialDimension { TwoD, ThreeD }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpatialGeometryKind {
    Point, Curve, Polygon, MultiPolygon, PolyhedralSurface, Prism, Solid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GeometryEncoding { GeoJson, JsonFg, Gml, Ifc, Other }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpatialScope {
    SurfaceOnly, SubsurfaceVolume, MixedSurfaceSubsurface, MunicipalDevelopmentEnvelope,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpatialLayerKind {
    ExternalRegisteredGeometryReference,
    MunicipalRegulatoryEnvelope,
    SurveyEvidenceOnly,
    PhysicalAssetModel,
    IndicativeDigitalTwin,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoordinateAxisOrder { Xy, Yx, LonLat, LatLon, EastNorth, NorthEast }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SurveyMethod {
    Lidar, Gnss, TotalStation, Photogrammetry, DigitalElevationModel, AsBuiltSurvey, Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TopologyState {
    NotAssessed, ValidatedWatertight, OpenSurface, NonManifold, SelfIntersecting, Invalid,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OverlapState { NotAssessed, Disjoint, Overlapping, Ambiguous }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SpatialQualificationOutcome {
    StructuralAccept, StructuralReject, ReviewRequired, NotEvaluable,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HorizontalReference {
    pub source_crs_uri: String,
    pub target_crs_uri: String,
    pub realization: String,
    pub axis_order: CoordinateAxisOrder,
    pub axis_order_profile_ref: String,
    #[serde(default)]
    pub coordinate_epoch: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VerticalReference {
    pub source_crs_uri: String,
    pub target_crs_uri: String,
    pub datum_name: String,
    pub realization: String,
    pub unit: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CoordinateOperation {
    pub source_crs_uri: String,
    pub target_crs_uri: String,
    pub operation_ref: String,
    pub operation_sha256: String,
    pub reported_accuracy_m: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SurveyEvidence {
    pub source_reference: String,
    pub source_sha256: String,
    pub captured_at: String,
    pub survey_method: SurveyMethod,
    pub surveyor_reference: String,
    pub horizontal_uncertainty_m: f64,
    #[serde(default)]
    pub vertical_uncertainty_m: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TopologyAssessment {
    pub state: TopologyState,
    #[serde(default)]
    pub validation_report_reference: Option<String>,
    #[serde(default)]
    pub validation_report_sha256: Option<String>,
    #[serde(default)]
    pub assessor_reference: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OverlapAssessment {
    pub state: OverlapState,
    pub compared_feature_references: Vec<String>,
    #[serde(default)]
    pub analysis_reference: Option<String>,
    #[serde(default)]
    pub analysis_sha256: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SpatialQualificationRecordV0 {
    pub schema: String,
    pub schema_uri: String,
    pub record_id: String,
    /// Opaque feature reference; raw coordinates/meshes are not copied here.
    pub feature_reference: String,
    pub geometry_sha256: String,
    pub source_record_reference: String,
    pub source_record_sha256: String,
    pub captured_at: String,
    pub geometry_encoding: GeometryEncoding,
    pub geometry_kind: SpatialGeometryKind,
    pub dimension: SpatialDimension,
    pub spatial_scope: SpatialScope,
    pub layer_kind: SpatialLayerKind,
    pub horizontal_reference: HorizontalReference,
    #[serde(default)]
    pub vertical_reference: Option<VerticalReference>,
    #[serde(default)]
    pub coordinate_operations: Vec<CoordinateOperation>,
    #[serde(default)]
    pub survey_evidence: Option<SurveyEvidence>,
    pub topology_assessment: TopologyAssessment,
    #[serde(default)]
    pub overlap_assessment: Option<OverlapAssessment>,
    #[serde(default)]
    pub governing_instrument_reference: Option<String>,
    #[serde(default)]
    pub external_register_record_reference: Option<String>,
    pub claim_ceiling: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpatialQualificationAssessment {
    pub outcome: SpatialQualificationOutcome,
    pub reasons: Vec<String>,
}

impl SpatialQualificationRecordV0 {
    /// Validate metadata structure and reference consistency only.
    pub fn validate_structural(&self) -> Result<(), Vec<String>> {
        let mut issues = Vec::new();
        for (field, value) in [
            ("record_id", self.record_id.as_str()),
            ("feature_reference", self.feature_reference.as_str()),
            ("source_record_reference", self.source_record_reference.as_str()),
            ("captured_at", self.captured_at.as_str()),
            ("horizontal_reference.source_crs_uri", self.horizontal_reference.source_crs_uri.as_str()),
            ("horizontal_reference.target_crs_uri", self.horizontal_reference.target_crs_uri.as_str()),
            ("horizontal_reference.realization", self.horizontal_reference.realization.as_str()),
            ("horizontal_reference.axis_order_profile_ref", self.horizontal_reference.axis_order_profile_ref.as_str()),
        ] {
            require(field, value, &mut issues);
        }

        if self.schema != SPATIAL_QUALIFICATION_SCHEMA {
            issues.push("unsupported spatial qualification schema".into());
        }
        if self.schema_uri != SPATIAL_QUALIFICATION_SCHEMA_URI {
            issues.push("unexpected spatial qualification schema URI".into());
        }
        if self.claim_ceiling != SPATIAL_QUALIFICATION_CLAIM_CEILING {
            issues.push("claim ceiling must remain reference_only_no_legal_effect".into());
        }
        if !is_sha256(&self.geometry_sha256) || !is_sha256(&self.source_record_sha256) {
            issues.push("geometry and source record digests must be lowercase SHA-256".into());
        }
        if self.horizontal_reference.coordinate_epoch.is_some_and(|epoch| !epoch.is_finite()) {
            issues.push("coordinate epoch must be finite".into());
        }

        let horizontal_changed = self.horizontal_reference.source_crs_uri
            != self.horizontal_reference.target_crs_uri;
        let vertical_changed = self.vertical_reference.as_ref().is_some_and(|vertical| {
            vertical.source_crs_uri != vertical.target_crs_uri
        });
        let mut operation_pairs = HashSet::new();
        for operation in &self.coordinate_operations {
            for (field, value) in [
                ("coordinate_operation.source_crs_uri", operation.source_crs_uri.as_str()),
                ("coordinate_operation.target_crs_uri", operation.target_crs_uri.as_str()),
                ("coordinate_operation.operation_ref", operation.operation_ref.as_str()),
            ] {
                require(field, value, &mut issues);
            }
            if !is_sha256(&operation.operation_sha256)
                || !operation.reported_accuracy_m.is_finite()
                || operation.reported_accuracy_m < 0.0
            {
                issues.push("coordinate operation requires a SHA-256 and finite non-negative accuracy".into());
            }
            if !operation_pairs.insert((
                operation.source_crs_uri.as_str(),
                operation.target_crs_uri.as_str(),
            )) {
                issues.push("duplicate coordinate operation source/target pair".into());
            }
        }
        if horizontal_changed
            && !operation_pairs.contains(&(
                self.horizontal_reference.source_crs_uri.as_str(),
                self.horizontal_reference.target_crs_uri.as_str(),
            ))
        {
            issues.push("horizontal CRS change lacks an exact versioned coordinate operation".into());
        }
        if let Some(vertical) = &self.vertical_reference {
            for (field, value) in [
                ("vertical_reference.source_crs_uri", vertical.source_crs_uri.as_str()),
                ("vertical_reference.target_crs_uri", vertical.target_crs_uri.as_str()),
                ("vertical_reference.datum_name", vertical.datum_name.as_str()),
                ("vertical_reference.realization", vertical.realization.as_str()),
                ("vertical_reference.unit", vertical.unit.as_str()),
            ] {
                require(field, value, &mut issues);
            }
            if vertical_changed
                && !operation_pairs.contains(&(
                    vertical.source_crs_uri.as_str(),
                    vertical.target_crs_uri.as_str(),
                ))
            {
                issues.push("vertical reference change lacks an exact versioned coordinate operation".into());
            }
        }

        if self.dimension == SpatialDimension::TwoD
            && matches!(
                self.geometry_kind,
                SpatialGeometryKind::PolyhedralSurface | SpatialGeometryKind::Prism | SpatialGeometryKind::Solid
            )
        {
            issues.push("3D geometry kind cannot be declared as 2D".into());
        }
        if self.dimension == SpatialDimension::ThreeD
            && matches!(
                self.spatial_scope,
                SpatialScope::SubsurfaceVolume | SpatialScope::MixedSurfaceSubsurface | SpatialScope::MunicipalDevelopmentEnvelope
            )
            && self.geometry_kind == SpatialGeometryKind::Polygon
        {
            issues.push("3D volumetric scope cannot be represented by a 2D Polygon".into());
        }
        if self.layer_kind == SpatialLayerKind::MunicipalRegulatoryEnvelope {
            if self.spatial_scope != SpatialScope::MunicipalDevelopmentEnvelope {
                issues.push("municipal regulatory envelope must declare municipal_development_envelope scope".into());
            }
            if self.governing_instrument_reference.is_none() {
                issues.push("municipal regulatory envelope requires its governing instrument reference".into());
            }
            if self.external_register_record_reference.is_some() {
                issues.push("municipal regulatory envelope cannot be relabelled as an external registered record".into());
            }
        }
        if self.layer_kind == SpatialLayerKind::ExternalRegisteredGeometryReference
            && self.external_register_record_reference.is_none()
        {
            issues.push("external registered geometry reference requires an opaque external record reference".into());
        }

        if let Some(survey) = &self.survey_evidence {
            if survey.source_reference.trim().is_empty()
                || !is_sha256(&survey.source_sha256)
                || survey.captured_at.trim().is_empty()
                || survey.surveyor_reference.trim().is_empty()
                || !survey.horizontal_uncertainty_m.is_finite()
                || survey.horizontal_uncertainty_m < 0.0
                || survey.vertical_uncertainty_m.is_some_and(|value| !value.is_finite() || value < 0.0)
            {
                issues.push("survey evidence has invalid identity, digest, time, surveyor or uncertainty".into());
            }
        }

        if self.topology_assessment.state == TopologyState::ValidatedWatertight {
            if self.topology_assessment.validation_report_reference.as_deref().is_none_or(str::is_empty)
                || !self.topology_assessment.validation_report_sha256.as_deref().is_some_and(is_sha256)
                || self.topology_assessment.assessor_reference.as_deref().is_none_or(str::is_empty)
            {
                issues.push("validated watertight topology requires report digest and assessor reference".into());
            }
        }
        if let Some(overlap) = &self.overlap_assessment {
            let mut refs = HashSet::new();
            for reference in &overlap.compared_feature_references {
                require("overlap_assessment.compared_feature_references[]", reference, &mut issues);
                if !refs.insert(reference.as_str()) {
                    issues.push("overlap assessment contains duplicate compared feature references".into());
                }
            }
            if matches!(overlap.state, OverlapState::Overlapping | OverlapState::Ambiguous) {
                if overlap.compared_feature_references.len() < 2
                    || overlap.analysis_reference.as_deref().is_none_or(str::is_empty)
                    || !overlap.analysis_sha256.as_deref().is_some_and(is_sha256)
                {
                    issues.push("overlap/ambiguous assessment requires two feature refs and an analysis digest".into());
                }
            }
        }

        if issues.is_empty() {
            Ok(())
        } else {
            issues.sort();
            Err(issues)
        }
    }

    /// Classify metadata readiness. Structural acceptance never means legal title,
    /// survey authenticity, or registration; the claim ceiling is fixed above.
    pub fn assess(&self) -> SpatialQualificationAssessment {
        if let Err(reasons) = self.validate_structural() {
            return SpatialQualificationAssessment {
                outcome: SpatialQualificationOutcome::StructuralReject,
                reasons,
            };
        }

        let mut missing = Vec::new();
        if matches!(
            self.spatial_scope,
            SpatialScope::SubsurfaceVolume | SpatialScope::MixedSurfaceSubsurface | SpatialScope::MunicipalDevelopmentEnvelope
        ) && self.dimension != SpatialDimension::ThreeD
        {
            missing.push("spatial scope requires a true 3D representation".to_string());
        }
        if self.dimension == SpatialDimension::ThreeD {
            if self.vertical_reference.is_none() {
                missing.push("3D geometry has no vertical CRS/datum/realization".into());
            }
            if self.survey_evidence.is_none() {
                missing.push("3D geometry has no source survey/measurement evidence".into());
            }
            if self.survey_evidence.as_ref().is_some_and(|survey| survey.vertical_uncertainty_m.is_none()) {
                missing.push("3D survey evidence lacks vertical uncertainty".into());
            }
            if self.topology_assessment.state == TopologyState::NotAssessed {
                missing.push("3D topology has not been independently assessed".into());
            }
        }
        if !missing.is_empty() {
            return SpatialQualificationAssessment {
                outcome: SpatialQualificationOutcome::NotEvaluable,
                reasons: missing,
            };
        }

        match self.topology_assessment.state {
            TopologyState::OpenSurface | TopologyState::NonManifold | TopologyState::SelfIntersecting | TopologyState::Invalid => {
                return SpatialQualificationAssessment {
                    outcome: SpatialQualificationOutcome::StructuralReject,
                    reasons: vec!["3D topology assessment reports an invalid or non-watertight volume".into()],
                };
            }
            TopologyState::NotAssessed | TopologyState::ValidatedWatertight => {}
        }

        if let Some(overlap) = &self.overlap_assessment {
            if matches!(overlap.state, OverlapState::Overlapping | OverlapState::Ambiguous) {
                return SpatialQualificationAssessment {
                    outcome: SpatialQualificationOutcome::ReviewRequired,
                    reasons: vec!["overlapping or ambiguous spatial interests must remain separate pending competent review; no priority is inferred".into()],
                };
            }
        }

        if self.layer_kind != SpatialLayerKind::ExternalRegisteredGeometryReference {
            return SpatialQualificationAssessment {
                outcome: SpatialQualificationOutcome::ReviewRequired,
                reasons: vec!["geometry is a municipal, survey-only, physical or indicative layer; it is not an external registered legal geometry".into()],
            };
        }

        SpatialQualificationAssessment {
            outcome: SpatialQualificationOutcome::StructuralAccept,
            reasons: vec!["spatial metadata and referenced qualification evidence satisfy the v0 preflight; this is not a legal, survey-authenticity or registry determination".into()],
        }
    }
}

#[derive(Debug, Deserialize)]
struct SpatialCorpus {
    suite_id: String,
    suite_version: String,
    scenarios: Vec<SpatialScenario>,
}

#[derive(Debug, Deserialize)]
struct SpatialScenario {
    scenario_id: String,
    record: SpatialQualificationRecordV0,
    expected_outcome: SpatialQualificationOutcome,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn spatial_qualification_schema_and_corpus_are_machine_readable() {
        let schema: serde_json::Value = serde_json::from_str(include_str!(
            "../../benchmarks/territorial-integrity/tib-3d-spatial-qualification-v0.1.schema.json"
        )).expect("3D spatial schema must parse");
        let corpus: SpatialCorpus = serde_json::from_str(include_str!(
            "../../benchmarks/territorial-integrity/tib-3d-spatial-qualification-v0.1.json"
        )).expect("3D spatial corpus must deserialize");
        assert_eq!(schema["$schema"], "https://json-schema.org/draft/2020-12/schema");
        assert_eq!(schema["$id"], SPATIAL_QUALIFICATION_SCHEMA_URI);
        assert_eq!(corpus.suite_id, "TIB-3D-SPATIAL-QUALIFICATION-001");
        assert_eq!(corpus.suite_version, "0.1.0");
        assert_eq!(corpus.scenarios.len(), 9);
        for scenario in corpus.scenarios {
            assert_eq!(
                scenario.record.assess().outcome,
                scenario.expected_outcome,
                "unexpected spatial qualification outcome for {}",
                scenario.scenario_id
            );
            assert_eq!(scenario.record.claim_ceiling, SPATIAL_QUALIFICATION_CLAIM_CEILING);
        }
    }

    #[test]
    fn capetown_dms_example_remains_a_municipal_not_registered_geometry() {
        let example: serde_json::Value = serde_json::from_str(include_str!(
            "../examples/capetown-dms-regulatory-envelope-synthetic-v0.json"
        ))
        .expect("Cape Town example must parse");
        let record: SpatialQualificationRecordV0 = serde_json::from_value(
            example["record"].clone(),
        )
        .expect("Cape Town spatial record must deserialize");
        assert_eq!(
            record.layer_kind,
            SpatialLayerKind::MunicipalRegulatoryEnvelope
        );
        assert_eq!(
            record.assess().outcome,
            SpatialQualificationOutcome::ReviewRequired
        );
        assert_eq!(
            record.claim_ceiling,
            SPATIAL_QUALIFICATION_CLAIM_CEILING
        );
    }

    #[test]
    fn missing_coordinate_operation_cannot_hide_a_crs_transformation() {
        let mut record = valid_registered_3d();
        record.horizontal_reference.target_crs_uri = "urn:crs:target:v2".into();
        assert!(record.validate_structural().is_err());
    }

    #[test]
    fn municipal_regulatory_envelope_is_never_promoted_to_registered_geometry() {
        let mut record = valid_registered_3d();
        record.layer_kind = SpatialLayerKind::MunicipalRegulatoryEnvelope;
        record.spatial_scope = SpatialScope::MunicipalDevelopmentEnvelope;
        record.external_register_record_reference = None;
        record.governing_instrument_reference = Some("urn:bylaw:municipal-planning-2025".into());
        assert_eq!(record.assess().outcome, SpatialQualificationOutcome::ReviewRequired);
        assert_eq!(record.claim_ceiling, SPATIAL_QUALIFICATION_CLAIM_CEILING);
    }

    #[test]
    fn invalid_topology_is_rejected_even_when_geodetic_metadata_is_complete() {
        let mut record = valid_registered_3d();
        record.topology_assessment.state = TopologyState::NonManifold;
        assert_eq!(record.assess().outcome, SpatialQualificationOutcome::StructuralReject);
    }

    #[test]
    fn overlapping_volumes_remain_separate_and_require_review() {
        let mut record = valid_registered_3d();
        record.overlap_assessment = Some(OverlapAssessment {
            state: OverlapState::Overlapping,
            compared_feature_references: vec!["urn:feature:volume:a".into(), "urn:feature:volume:b".into()],
            analysis_reference: Some("urn:analysis:overlap:001".into()),
            analysis_sha256: Some("d".repeat(64)),
        });
        let assessment = record.assess();
        assert_eq!(assessment.outcome, SpatialQualificationOutcome::ReviewRequired);
        assert!(assessment.reasons[0].contains("no priority is inferred"));
        assert!(!serde_json::to_string(&record).unwrap().contains("winning_owner"));
    }

    fn valid_registered_3d() -> SpatialQualificationRecordV0 {
        SpatialQualificationRecordV0 {
            schema: SPATIAL_QUALIFICATION_SCHEMA.into(),
            schema_uri: SPATIAL_QUALIFICATION_SCHEMA_URI.into(),
            record_id: "synthetic-3d-record-001".into(),
            feature_reference: "urn:ogc:feature:volume-001".into(),
            geometry_sha256: "a".repeat(64),
            source_record_reference: "urn:source:survey-plan:001".into(),
            source_record_sha256: "b".repeat(64),
            captured_at: "2026-11-19T06:00:00Z".into(),
            geometry_encoding: GeometryEncoding::JsonFg,
            geometry_kind: SpatialGeometryKind::Prism,
            dimension: SpatialDimension::ThreeD,
            spatial_scope: SpatialScope::SubsurfaceVolume,
            layer_kind: SpatialLayerKind::ExternalRegisteredGeometryReference,
            horizontal_reference: HorizontalReference {
                source_crs_uri: "urn:crs:itrf:2020".into(),
                target_crs_uri: "urn:crs:itrf:2020".into(),
                realization: "ITRF2020".into(),
                axis_order: CoordinateAxisOrder::EastNorth,
                axis_order_profile_ref: "urn:profile:axis-order:en".into(),
                coordinate_epoch: Some(2026.0),
            },
            vertical_reference: Some(VerticalReference {
                source_crs_uri: "urn:crs:ihrf:geoid:v0".into(),
                target_crs_uri: "urn:crs:ihrf:geoid:v0".into(),
                datum_name: "Geoid-based vertical reference".into(),
                realization: "IHRF-aligned profile reference".into(),
                unit: "metre".into(),
            }),
            coordinate_operations: Vec::new(),
            survey_evidence: Some(SurveyEvidence {
                source_reference: "urn:survey:evidence:001".into(),
                source_sha256: "c".repeat(64),
                captured_at: "2026-11-19T05:00:00Z".into(),
                survey_method: SurveyMethod::Gnss,
                surveyor_reference: "urn:professional:surveyor:001".into(),
                horizontal_uncertainty_m: 0.02,
                vertical_uncertainty_m: Some(0.03),
            }),
            topology_assessment: TopologyAssessment {
                state: TopologyState::ValidatedWatertight,
                validation_report_reference: Some("urn:topology:report:001".into()),
                validation_report_sha256: Some("d".repeat(64)),
                assessor_reference: Some("urn:assessor:001".into()),
            },
            overlap_assessment: None,
            governing_instrument_reference: Some("urn:instrument:external:001".into()),
            external_register_record_reference: Some("urn:registry:record:001".into()),
            claim_ceiling: SPATIAL_QUALIFICATION_CLAIM_CEILING.into(),
        }
    }
}
