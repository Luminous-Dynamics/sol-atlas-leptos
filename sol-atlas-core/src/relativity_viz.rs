// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Evidence-aware black-hole research view models for Sol Atlas.
//!
//! This module is intentionally separate from Earth-geospatial layers.
//! Astronomical frames, detector time series, reconstructed products, posterior
//! inference, simulations, and hypotheses must not be rendered as latitude /
//! longitude pins or as interchangeable observations.
//!
//! The core stores references and metadata, not large strain / HDF5 / VLBI
//! arrays. Payloads remain in their authoritative archives or content-addressed
//! storage. A digest establishes artifact identity, not scientific correctness.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// How a displayed scientific series relates to the underlying physical system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ScientificDataRole {
    /// Direct instrument or detector measurement.
    Measured,
    /// Output produced by an imaging / reconstruction algorithm.
    Reconstructed,
    /// Parameter, state, or quantity inferred from data under a declared model.
    Inferred,
    /// Output of a physical or numerical simulation.
    Simulated,
    /// A model-derived value not itself measured in the cited data.
    Predicted,
    /// A proposed explanation or not-yet-tested model.
    Hypothesized,
    /// A deliberately null / noise / negative-control output.
    NullControl,
}

impl ScientificDataRole {
    pub const fn label(self) -> &'static str {
        match self {
            Self::Measured => "Measured",
            Self::Reconstructed => "Reconstructed",
            Self::Inferred => "Inferred",
            Self::Simulated => "Simulated",
            Self::Predicted => "Predicted",
            Self::Hypothesized => "Hypothesized",
            Self::NullControl => "Null control",
        }
    }

    /// True only for raw instrument-level observations.
    pub const fn is_direct_measurement(self) -> bool {
        matches!(self, Self::Measured)
    }

    /// True when a role must never be presented as raw observed strain or image data.
    pub const fn is_model_derived(self) -> bool {
        matches!(
            self,
            Self::Inferred
                | Self::Simulated
                | Self::Predicted
                | Self::Hypothesized
                | Self::NullControl
        )
    }
}

/// Kind of referenced artifact; separate from the epistemic role of a series.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResearchArtifactKind {
    Publication,
    Dataset,
    Software,
    Workflow,
    Protocol,
    Configuration,
    Measurement,
    Reconstruction,
    Posterior,
    SimulationOutput,
    NullControl,
    AnalysisReport,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactAvailability {
    Public,
    Controlled,
    Embargoed,
    Unavailable,
}

/// A source or output artifact reference. Large payloads are not embedded.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResearchArtifactRef {
    pub id: String,
    pub kind: ResearchArtifactKind,
    pub title: String,
    pub uri: String,
    /// Optional BLAKE3 hex digest of the exact referenced bytes.
    pub blake3_digest: Option<String>,
    pub media_type: Option<String>,
    pub license: Option<String>,
    pub availability: ArtifactAvailability,
    pub source_release: Option<String>,
}

impl ResearchArtifactRef {
    pub fn validate(&self) -> Result<(), ValidationError> {
        require_text(&self.id, "artifact.id")?;
        require_text(&self.title, "artifact.title")?;
        require_text(&self.uri, "artifact.uri")?;
        if let Some(digest) = &self.blake3_digest {
            validate_blake3_hex(digest)?;
        }
        Ok(())
    }
}

/// Astronomical coordinate / time context. It is not a terrestrial location.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CelestialReferenceFrame {
    /// e.g. ICRS, source frame, detector frame, or a named simulation chart.
    pub frame: String,
    /// e.g. UTC, GPS, TDB, source-frame time, or an affine parameter convention.
    pub time_standard: Option<String>,
    /// Optional coordinate chart (e.g. Kerr Boyer-Lindquist).
    pub coordinate_chart: Option<String>,
    /// Explicit unit convention for coordinates / plotted quantities.
    pub unit_convention: String,
}

impl CelestialReferenceFrame {
    pub fn validate(&self) -> Result<(), ValidationError> {
        require_text(&self.frame, "frame.frame")?;
        require_text(&self.unit_convention, "frame.unit_convention")?;
        Ok(())
    }
}

/// A chartable observable or model series. The series payload is in a referenced
/// artifact so Sol Atlas can stream / downsample it without bloating DHT records.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScientificSeriesRef {
    pub id: String,
    pub label: String,
    pub role: ScientificDataRole,
    pub quantity: String,
    pub unit: String,
    pub frame: CelestialReferenceFrame,
    pub source_artifact_id: String,
    pub model_run_id: Option<String>,
    /// Optional series-level note (e.g. "whitened strain", "90% credible band").
    pub description: Option<String>,
}

impl ScientificSeriesRef {
    pub fn validate(&self) -> Result<(), ValidationError> {
        require_text(&self.id, "series.id")?;
        require_text(&self.label, "series.label")?;
        require_text(&self.quantity, "series.quantity")?;
        require_text(&self.unit, "series.unit")?;
        require_text(&self.source_artifact_id, "series.source_artifact_id")?;
        self.frame.validate()?;
        Ok(())
    }
}

/// Black-hole observation / event metadata. No inferred parameter is stored
/// as an exact measurement; model-derived quantities live in separate series.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlackHoleEventRef {
    pub event_id: String,
    pub catalog: String,
    pub catalog_release: String,
    pub observation_epoch: Option<String>,
    pub detectors_or_observatories: Vec<String>,
    pub source_artifact_ids: Vec<String>,
    pub data_quality_artifact_ids: Vec<String>,
}

impl BlackHoleEventRef {
    pub fn validate(&self) -> Result<(), ValidationError> {
        require_text(&self.event_id, "event.event_id")?;
        require_text(&self.catalog, "event.catalog")?;
        require_text(&self.catalog_release, "event.catalog_release")?;
        if self.detectors_or_observatories.is_empty() {
            return Err(ValidationError::EmptyField("event.detectors_or_observatories"));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClaimTopic {
    ResidualExtraction,
    ResidualCharacterization,
    HorizonAssociation,
    SourceRegionAttribution,
    AreaLawImplication,
    RingdownMode,
    AstrophysicalClassification,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClaimDisposition {
    Open,
    Supported,
    Contested,
    Superseded,
    Retracted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceRelation {
    Supports,
    Refutes,
    Inconclusive,
    Context,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ClaimEvidenceLink {
    pub artifact_id: String,
    pub relation: EvidenceRelation,
    pub note: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlackHoleScientificClaim {
    pub claim_id: String,
    pub topic: ClaimTopic,
    pub statement: String,
    pub disposition: ClaimDisposition,
    /// Mirrors the Mycelix evidence maturity vocabulary without deriving a score.
    pub evidence_maturity: String,
    pub evidence: Vec<ClaimEvidenceLink>,
    pub supersedes_claim_id: Option<String>,
    pub caveat: Option<String>,
}

impl BlackHoleScientificClaim {
    pub fn validate(&self) -> Result<(), ValidationError> {
        require_text(&self.claim_id, "claim.claim_id")?;
        require_text(&self.statement, "claim.statement")?;
        require_text(&self.evidence_maturity, "claim.evidence_maturity")?;
        if self.disposition == ClaimDisposition::Contested {
            let has_support = self.evidence.iter().any(|e| e.relation == EvidenceRelation::Supports);
            let has_refutation = self.evidence.iter().any(|e| e.relation == EvidenceRelation::Refutes);
            if !(has_support && has_refutation) {
                return Err(ValidationError::ContestedClaimNeedsBothSides(self.claim_id.clone()));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelRunRef {
    pub model_run_id: String,
    pub model_name: String,
    pub model_version: String,
    pub formulation: String,
    pub code_artifact_id: String,
    pub configuration_artifact_id: String,
    pub output_artifact_ids: Vec<String>,
    pub applicability_domain: String,
}

impl ModelRunRef {
    pub fn validate(&self) -> Result<(), ValidationError> {
        require_text(&self.model_run_id, "model_run.model_run_id")?;
        require_text(&self.model_name, "model_run.model_name")?;
        require_text(&self.model_version, "model_run.model_version")?;
        require_text(&self.formulation, "model_run.formulation")?;
        require_text(&self.code_artifact_id, "model_run.code_artifact_id")?;
        require_text(&self.configuration_artifact_id, "model_run.configuration_artifact_id")?;
        require_text(&self.applicability_domain, "model_run.applicability_domain")?;
        if self.output_artifact_ids.is_empty() {
            return Err(ValidationError::EmptyField("model_run.output_artifact_ids"));
        }
        Ok(())
    }
}

/// One portable view bundle. It contains only references and claims, not raw data.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BlackHoleResearchBundle {
    pub schema_version: u16,
    pub events: Vec<BlackHoleEventRef>,
    pub artifacts: Vec<ResearchArtifactRef>,
    pub series: Vec<ScientificSeriesRef>,
    pub model_runs: Vec<ModelRunRef>,
    pub claims: Vec<BlackHoleScientificClaim>,
}

impl BlackHoleResearchBundle {
    pub fn validate(&self) -> Result<(), ValidationError> {
        if self.schema_version != 1 {
            return Err(ValidationError::UnsupportedSchemaVersion(self.schema_version));
        }

        let mut artifact_ids = BTreeSet::new();
        for artifact in &self.artifacts {
            artifact.validate()?;
            if !artifact_ids.insert(artifact.id.as_str()) {
                return Err(ValidationError::DuplicateIdentifier(artifact.id.clone()));
            }
        }

        let mut event_ids = BTreeSet::new();
        for event in &self.events {
            event.validate()?;
            if !event_ids.insert(event.event_id.as_str()) {
                return Err(ValidationError::DuplicateIdentifier(event.event_id.clone()));
            }
            for artifact_id in event.source_artifact_ids.iter().chain(event.data_quality_artifact_ids.iter()) {
                if !artifact_ids.contains(artifact_id.as_str()) {
                    return Err(ValidationError::UnresolvedArtifact(artifact_id.clone()));
                }
            }
        }

        let mut run_ids = BTreeSet::new();
        for run in &self.model_runs {
            run.validate()?;
            if !run_ids.insert(run.model_run_id.as_str()) {
                return Err(ValidationError::DuplicateIdentifier(run.model_run_id.clone()));
            }
            for id in std::iter::once(&run.code_artifact_id)
                .chain(std::iter::once(&run.configuration_artifact_id))
                .chain(run.output_artifact_ids.iter())
            {
                if !artifact_ids.contains(id.as_str()) {
                    return Err(ValidationError::UnresolvedArtifact(id.clone()));
                }
            }
        }

        let mut series_ids = BTreeSet::new();
        for series in &self.series {
            series.validate()?;
            if !series_ids.insert(series.id.as_str()) {
                return Err(ValidationError::DuplicateIdentifier(series.id.clone()));
            }
            if !artifact_ids.contains(series.source_artifact_id.as_str()) {
                return Err(ValidationError::UnresolvedArtifact(series.source_artifact_id.clone()));
            }
            if let Some(run_id) = &series.model_run_id {
                if !run_ids.contains(run_id.as_str()) {
                    return Err(ValidationError::UnresolvedModelRun(run_id.clone()));
                }
                if series.role == ScientificDataRole::Measured {
                    return Err(ValidationError::ModelOutputCannotBeMeasured(series.id.clone()));
                }
            }
        }

        let mut claim_ids = BTreeSet::new();
        for claim in &self.claims {
            claim.validate()?;
            if !claim_ids.insert(claim.claim_id.as_str()) {
                return Err(ValidationError::DuplicateIdentifier(claim.claim_id.clone()));
            }
            for evidence in &claim.evidence {
                if !artifact_ids.contains(evidence.artifact_id.as_str()) {
                    return Err(ValidationError::UnresolvedArtifact(evidence.artifact_id.clone()));
                }
            }
        }

        Ok(())
    }

    pub fn artifact_index(&self) -> BTreeMap<&str, &ResearchArtifactRef> {
        self.artifacts.iter().map(|a| (a.id.as_str(), a)).collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    UnsupportedSchemaVersion(u16),
    EmptyField(&'static str),
    InvalidDigest(String),
    DuplicateIdentifier(String),
    UnresolvedArtifact(String),
    UnresolvedModelRun(String),
    ModelOutputCannotBeMeasured(String),
    ContestedClaimNeedsBothSides(String),
}

fn require_text(value: &str, field: &'static str) -> Result<(), ValidationError> {
    if value.trim().is_empty() {
        Err(ValidationError::EmptyField(field))
    } else {
        Ok(())
    }
}

fn validate_blake3_hex(value: &str) -> Result<(), ValidationError> {
    if value.len() != 64 || !value.bytes().all(|b| b.is_ascii_hexdigit()) {
        return Err(ValidationError::InvalidDigest(value.to_string()));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn artifact(id: &str, kind: ResearchArtifactKind) -> ResearchArtifactRef {
        ResearchArtifactRef {
            id: id.to_string(),
            kind,
            title: format!("artifact {id}"),
            uri: format!("https://example.org/{id}"),
            blake3_digest: Some("a".repeat(64)),
            media_type: Some("application/json".into()),
            license: Some("CC-BY-4.0".into()),
            availability: ArtifactAvailability::Public,
            source_release: Some("v1".into()),
        }
    }

    #[test]
    fn rejects_invalid_digest() {
        let mut a = artifact("a", ResearchArtifactKind::Dataset);
        a.blake3_digest = Some("not-a-digest".into());
        assert!(matches!(a.validate(), Err(ValidationError::InvalidDigest(_))));
    }

    #[test]
    fn validates_minimal_bundle_and_links() {
        let mut b = BlackHoleResearchBundle {
            schema_version: 1,
            artifacts: vec![
                artifact("dataset", ResearchArtifactKind::Dataset),
                artifact("code", ResearchArtifactKind::Software),
                artifact("config", ResearchArtifactKind::Configuration),
                artifact("output", ResearchArtifactKind::SimulationOutput),
            ],
            model_runs: vec![ModelRunRef {
                model_run_id: "run-1".into(),
                model_name: "Kerr test model".into(),
                model_version: "0.1".into(),
                formulation: "analytic reference".into(),
                code_artifact_id: "code".into(),
                configuration_artifact_id: "config".into(),
                output_artifact_ids: vec!["output".into()],
                applicability_domain: "stationary Kerr exterior".into(),
            }],
            series: vec![ScientificSeriesRef {
                id: "series-1".into(),
                label: "model reference".into(),
                role: ScientificDataRole::Predicted,
                quantity: "frequency".into(),
                unit: "Hz".into(),
                frame: CelestialReferenceFrame {
                    frame: "source frame".into(),
                    time_standard: Some("source coordinate time".into()),
                    coordinate_chart: Some("Kerr Boyer-Lindquist".into()),
                    unit_convention: "SI".into(),
                },
                source_artifact_id: "output".into(),
                model_run_id: Some("run-1".into()),
                description: None,
            }],
            ..Default::default()
        };
        assert!(b.validate().is_ok());
        b.series[0].role = ScientificDataRole::Measured;
        assert!(matches!(
            b.validate(),
            Err(ValidationError::ModelOutputCannotBeMeasured(_))
        ));
    }

    #[test]
    fn contested_claim_requires_support_and_refutation_links() {
        let c = BlackHoleScientificClaim {
            claim_id: "horizon-association".into(),
            topic: ClaimTopic::HorizonAssociation,
            statement: "A residual frequency tracks Kerr horizon frequency".into(),
            disposition: ClaimDisposition::Contested,
            evidence_maturity: "contested".into(),
            evidence: vec![ClaimEvidenceLink {
                artifact_id: "paper-a".into(),
                relation: EvidenceRelation::Supports,
                note: None,
            }],
            supersedes_claim_id: None,
            caveat: None,
        };
        assert!(matches!(
            c.validate(),
            Err(ValidationError::ContestedClaimNeedsBothSides(_))
        ));
    }

    #[test]
    fn model_outputs_cannot_be_laundered_into_measurements() {
        let mut b = BlackHoleResearchBundle {
            schema_version: 1,
            artifacts: vec![
                artifact("code", ResearchArtifactKind::Software),
                artifact("cfg", ResearchArtifactKind::Configuration),
                artifact("out", ResearchArtifactKind::SimulationOutput),
            ],
            model_runs: vec![ModelRunRef {
                model_run_id: "run".into(),
                model_name: "test".into(),
                model_version: "1".into(),
                formulation: "test".into(),
                code_artifact_id: "code".into(),
                configuration_artifact_id: "cfg".into(),
                output_artifact_ids: vec!["out".into()],
                applicability_domain: "fixture".into(),
            }],
            series: vec![ScientificSeriesRef {
                id: "s".into(),
                label: "sim output".into(),
                role: ScientificDataRole::Measured,
                quantity: "strain".into(),
                unit: "dimensionless".into(),
                frame: CelestialReferenceFrame {
                    frame: "detector frame".into(),
                    time_standard: Some("GPS".into()),
                    coordinate_chart: None,
                    unit_convention: "SI".into(),
                },
                source_artifact_id: "out".into(),
                model_run_id: Some("run".into()),
                description: None,
            }],
            ..Default::default()
        };
        assert!(matches!(
            b.validate(),
            Err(ValidationError::ModelOutputCannotBeMeasured(_))
        ));
    }
}
