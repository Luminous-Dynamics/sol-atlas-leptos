// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Typed consumer for Humanity Atlas v0.1 simulation-run envelopes.
//!
//! This module preserves a run's epistemic classification instead of treating
//! every numeric output as a forecast. Parsing validates the envelope and
//! declared validation tier; it does not independently validate the model,
//! evidence, or scientific truth of a result.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

/// Version currently understood by this consumer.
pub const RUN_SCHEMA_VERSION: &str = "humanity-atlas.run.v0.1";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScenarioRunEnvelope {
    pub schema_version: String,
    pub run_id: String,
    pub scenario_id: String,
    pub inputs: RunInputDigests,
    pub engine: EngineIdentity,
    pub run_status: RunStatus,
    pub output_classification: OutputClassification,
    pub deterministic: Option<bool>,
    pub seed: u64,
    pub replicates: u32,
    pub outputs: Vec<MetricOutput>,
    pub validation: ValidationRecord,
    pub warnings: Vec<String>,
    #[serde(default)]
    pub artifacts: Vec<RunArtifactReference>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunInputDigests {
    pub scenario_sha256: String,
    pub dataset_sha256: Option<String>,
    pub scenario_identity_kind: Option<String>,
    pub resolved_config_sha256: Option<String>,
    pub resolved_params_sha256: Option<String>,
    pub selected_definition_source: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EngineIdentity {
    pub adapter: String,
    pub engine_version: String,
    pub model_sha256: String,
    pub model_identity_kind: Option<String>,
    pub source_revision: Option<String>,
    pub definition_source: Option<String>,
}

/// Reference to a byte-addressed source or domain artifact. The digest binds
/// exact bytes; it does not establish that the URI is retrievable or trusted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunArtifactReference {
    pub artifact_id: String,
    pub role: String,
    pub media_type: String,
    pub artifact_ref: String,
    pub artifact_sha256: String,
    pub json_pointer: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
    Completed,
    Partial,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputClassification {
    SyntheticFixture,
    IllustrativeScenario,
    UnvalidatedSimulation,
    CalibratedSimulation,
    ProspectiveForecast,
}

impl OutputClassification {
    /// Human-readable provenance label; this is not an independent quality score.
    pub fn epistemic_label(self) -> &'static str {
        match self {
            Self::SyntheticFixture => "Synthetic fixture — not empirical",
            Self::IllustrativeScenario => "Illustrative scenario — not a forecast",
            Self::UnvalidatedSimulation => "Unvalidated simulation",
            Self::CalibratedSimulation => "Calibrated simulation — inspect validation record",
            Self::ProspectiveForecast => "Prospective forecast — inspect forecast receipt",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MetricOutput {
    pub metric_id: String,
    pub year: i32,
    pub unit: String,
    pub statistic: OutputStatistic,
    pub value: f64,
    pub sample_count: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputStatistic {
    Value,
    Mean,
    Median,
    Min,
    Max,
    P05,
    P95,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValidationRecord {
    pub status: ValidationStatus,
    pub notes: Vec<String>,
    pub evidence: Vec<ValidationEvidence>,
}

/// Content-addressed pointer to a validation artifact. The digest binds bytes;
/// this structure alone does not authenticate the publisher or the artifact's claims.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValidationEvidence {
    pub evidence_id: String,
    pub kind: ValidationEvidenceKind,
    pub artifact_ref: String,
    pub artifact_sha256: String,
    pub description: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationEvidenceKind {
    Reproducibility,
    RetrospectiveEvaluation,
    ProspectiveForecast,
    IndependentReview,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationStatus {
    NotRun,
    ContractSmoke,
    Retrospective,
    Prospective,
    ExternalReview,
}

impl ScenarioRunEnvelope {
    /// Parse a strict v0.1 run envelope and check cross-field invariants.
    ///
    /// Success means the payload is well-formed and internally consistent
    /// according to the envelope contract. It does not mean the model has
    /// been independently audited or that its outputs describe reality.
    pub fn from_json(raw: &str) -> Result<Self, String> {
        let run: Self = serde_json::from_str(raw)
            .map_err(|error| format!("invalid Humanity Atlas run JSON: {error}"))?;
        run.validate()?;
        Ok(run)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != RUN_SCHEMA_VERSION {
            return Err(format!(
                "unsupported run schema version: {}",
                self.schema_version
            ));
        }
        if self.run_id.trim().is_empty() || self.scenario_id.trim().is_empty() {
            return Err("run_id and scenario_id must be non-empty".into());
        }
        let mut digests = vec![
            ("scenario_sha256", self.inputs.scenario_sha256.as_str()),
            ("model_sha256", self.engine.model_sha256.as_str()),
        ];
        if let Some(digest) = self.inputs.dataset_sha256.as_deref() {
            digests.push(("dataset_sha256", digest));
        }
        if let Some(digest) = self.inputs.resolved_config_sha256.as_deref() {
            digests.push(("resolved_config_sha256", digest));
        }
        if let Some(digest) = self.inputs.resolved_params_sha256.as_deref() {
            digests.push(("resolved_params_sha256", digest));
        }
        for (field, digest) in digests {
            if !is_lowercase_sha256(digest) {
                return Err(format!("{field} must be 64 lowercase hexadecimal characters"));
            }
        }
        if let Some(kind) = self.inputs.scenario_identity_kind.as_deref() {
            if !matches!(kind, "scenario_document_bytes" | "canonical_effective_scenario") {
                return Err("unsupported scenario_identity_kind".into());
            }
        }
        if let Some(source) = self.inputs.selected_definition_source.as_deref() {
            if source.trim().is_empty() {
                return Err("selected_definition_source must be non-empty when present".into());
            }
        }
        if let Some(kind) = self.engine.model_identity_kind.as_deref() {
            if !matches!(kind, "full_model" | "technology_tree_definition" | "unknown") {
                return Err("unsupported model_identity_kind".into());
            }
        }
        if self.engine.source_revision.as_deref().is_some_and(|value| value.trim().is_empty())
            || self.engine.definition_source.as_deref().is_some_and(|value| value.trim().is_empty())
        {
            return Err("source_revision and definition_source must be non-empty when present".into());
        }
        if self.engine.adapter.trim().is_empty() || self.engine.engine_version.trim().is_empty() {
            return Err("engine adapter and version must be non-empty".into());
        }
        if self.replicates == 0 {
            return Err("replicates must be at least one".into());
        }
        if self.run_status == RunStatus::Completed && self.outputs.is_empty() {
            return Err("a completed run must contain at least one output row".into());
        }
        for evidence in &self.validation.evidence {
            if evidence.evidence_id.trim().is_empty()
                || evidence.artifact_ref.trim().is_empty()
                || evidence.description.trim().is_empty()
            {
                return Err("validation evidence ID, artifact reference, and description must be non-empty".into());
            }
            if !is_lowercase_sha256(&evidence.artifact_sha256) {
                return Err("validation evidence artifact_sha256 must be lowercase SHA-256".into());
            }
        }

        let mut artifact_ids = BTreeSet::new();
        for artifact in &self.artifacts {
            if artifact.artifact_id.trim().is_empty()
                || artifact.role.trim().is_empty()
                || artifact.media_type.trim().is_empty()
                || artifact.artifact_ref.trim().is_empty()
            {
                return Err("artifact ID, role, media type, and reference must be non-empty".into());
            }
            if !is_lowercase_sha256(&artifact.artifact_sha256) {
                return Err("artifact artifact_sha256 must be lowercase SHA-256".into());
            }
            if artifact.json_pointer.as_deref().is_some_and(|pointer| !is_valid_json_pointer(pointer)) {
                return Err(format!("invalid JSON Pointer for artifact {}", artifact.artifact_id));
            }
            if !artifact_ids.insert(artifact.artifact_id.as_str()) {
                return Err(format!("duplicate artifact_id: {}", artifact.artifact_id));
            }
        }

        let mut unique_rows = BTreeSet::new();
        for row in &self.outputs {
            if row.metric_id.trim().is_empty() || row.unit.trim().is_empty() {
                return Err("metric_id and unit must be non-empty".into());
            }
            if !row.value.is_finite() {
                return Err(format!("metric {} contains a non-finite value", row.metric_id));
            }
            if row.sample_count == 0 {
                return Err(format!("metric {} has sample_count = 0", row.metric_id));
            }
            let key = (row.metric_id.as_str(), row.year, row.statistic);
            if !unique_rows.insert(key) {
                return Err(format!(
                    "duplicate metric row: {} / {} / {:?}",
                    row.metric_id, row.year, row.statistic
                ));
            }
        }

        if matches!(
            self.output_classification,
            OutputClassification::CalibratedSimulation | OutputClassification::ProspectiveForecast
        ) && self.run_status != RunStatus::Completed
        {
            return Err("calibrated_simulation and prospective_forecast require a completed run".into());
        }

        let evidence_kinds: BTreeSet<ValidationEvidenceKind> =
            self.validation.evidence.iter().map(|item| item.kind).collect();
        match self.output_classification {
            OutputClassification::CalibratedSimulation => {
                if !matches!(
                    self.validation.status,
                    ValidationStatus::Retrospective | ValidationStatus::ExternalReview
                ) {
                    return Err(
                        "calibrated_simulation requires retrospective or external-review validation"
                            .into(),
                    );
                }
                if !evidence_kinds.contains(&ValidationEvidenceKind::RetrospectiveEvaluation) {
                    return Err(
                        "calibrated_simulation requires retrospective evaluation evidence".into(),
                    );
                }
            }
            OutputClassification::ProspectiveForecast => {
                if !matches!(
                    self.validation.status,
                    ValidationStatus::Prospective | ValidationStatus::ExternalReview
                ) {
                    return Err(
                        "prospective_forecast requires prospective or external-review validation"
                            .into(),
                    );
                }
                if !evidence_kinds.contains(&ValidationEvidenceKind::ProspectiveForecast) {
                    return Err(
                        "prospective_forecast requires forecast receipt evidence".into(),
                    );
                }
            }
            _ => {}
        }

        Ok(())
    }
}

fn is_valid_json_pointer(value: &str) -> bool {
    if value.is_empty() {
        return true;
    }
    if !value.starts_with('/') {
        return false;
    }
    let mut chars = value.chars();
    while let Some(character) = chars.next() {
        if character == '~' && !matches!(chars.next(), Some('0' | '1')) {
            return false;
        }
    }
    true
}

fn is_lowercase_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_payload() -> String {
        let digest_a = "a".repeat(64);
        let digest_b = "b".repeat(64);
        let digest_c = "c".repeat(64);
        format!(
            r#"{{
              "schema_version":"humanity-atlas.run.v0.1",
              "run_id":"test-run-001",
              "scenario_id":"test.scenario.v0-1",
              "inputs":{{"scenario_sha256":"{digest_a}","dataset_sha256":"{digest_b}"}},
              "engine":{{"adapter":"fixture","engine_version":"0.1.0","model_sha256":"{digest_c}"}},
              "run_status":"completed",
              "output_classification":"synthetic_fixture",
              "deterministic":true,
              "seed":0,
              "replicates":1,
              "outputs":[{{"metric_id":"storage_gwh","year":2030,"unit":"GWh","statistic":"value","value":12.5,"sample_count":1}}],
              "validation":{{"status":"contract_smoke","notes":["synthetic fixture only"],"evidence":[]}},
              "warnings":["not empirical"]
            }}"#
        )
    }

    #[test]
    fn parses_synthetic_contract_envelope() {
        let run = ScenarioRunEnvelope::from_json(&valid_payload()).expect("valid fixture should parse");
        assert_eq!(run.output_classification.epistemic_label(), "Synthetic fixture — not empirical");
        assert_eq!(run.outputs.len(), 1);
        assert_eq!(run.deterministic, Some(true));
    }

    #[test]
    fn accepts_unknown_dataset_and_determinism_with_a_source_artifact() {
        let raw = valid_payload()
            .replace(
                &format!("\"dataset_sha256\":\"{}\"", "b".repeat(64)),
                "\"dataset_sha256\":null",
            )
            .replace("\"deterministic\":true", "\"deterministic\":null");
        let mut run = ScenarioRunEnvelope::from_json(&raw)
            .expect("null represents unestablished dataset identity/determinism");
        assert_eq!(run.inputs.dataset_sha256, None);
        assert_eq!(run.deterministic, None);
        run.artifacts.push(RunArtifactReference {
            artifact_id: "source-simulator-output".into(),
            role: "source_simulation_output".into(),
            media_type: "application/json".into(),
            artifact_ref: "urn:sha256:source".into(),
            artifact_sha256: "d".repeat(64),
            json_pointer: Some("/technology_tree".into()),
        });
        assert!(run.validate().is_ok());
    }

    #[test]
    fn rejects_duplicate_source_artifact_ids() {
        let mut run = ScenarioRunEnvelope::from_json(&valid_payload()).expect("valid fixture");
        let artifact = RunArtifactReference {
            artifact_id: "same-id".into(),
            role: "source_simulation_output".into(),
            media_type: "application/json".into(),
            artifact_ref: "urn:sha256:artifact".into(),
            artifact_sha256: "d".repeat(64),
            json_pointer: None,
        };
        run.artifacts = vec![artifact.clone(), artifact];
        assert!(run.validate().unwrap_err().contains("duplicate artifact_id"));
    }

    #[test]
    fn rejects_invalid_embedded_artifact_json_pointer() {
        let mut run = ScenarioRunEnvelope::from_json(&valid_payload()).expect("valid fixture");
        run.artifacts.push(RunArtifactReference {
            artifact_id: "bad-pointer".into(),
            role: "source_simulation_output".into(),
            media_type: "application/json".into(),
            artifact_ref: "urn:sha256:artifact".into(),
            artifact_sha256: "d".repeat(64),
            json_pointer: Some("/technology~2tree".into()),
        });
        assert!(run.validate().unwrap_err().contains("invalid JSON Pointer"));
    }

    #[test]
    fn rejects_malformed_source_artifact_digest() {
        let mut run = ScenarioRunEnvelope::from_json(&valid_payload()).expect("valid fixture");
        run.artifacts.push(RunArtifactReference {
            artifact_id: "bad-digest".into(),
            role: "source_simulation_output".into(),
            media_type: "application/json".into(),
            artifact_ref: "urn:sha256:artifact".into(),
            artifact_sha256: "not-a-sha256".into(),
            json_pointer: None,
        });
        assert!(run.validate().unwrap_err().contains("artifact_sha256"));
    }

    #[test]
    fn rejects_unrecognized_schema_version() {
        let bad = valid_payload().replace("humanity-atlas.run.v0.1", "humanity-atlas.run.v9.9");
        assert!(ScenarioRunEnvelope::from_json(&bad)
            .unwrap_err()
            .contains("unsupported run schema version"));
    }

    #[test]
    fn rejects_malformed_digest() {
        let digest = "a".repeat(64);
        let bad = valid_payload().replace(digest.as_str(), "not-a-sha256");
        assert!(ScenarioRunEnvelope::from_json(&bad)
            .unwrap_err()
            .contains("scenario_sha256"));
    }

    #[test]
    fn rejects_forecast_label_without_forecast_validation() {
        let bad = valid_payload().replace("synthetic_fixture", "prospective_forecast");
        assert!(ScenarioRunEnvelope::from_json(&bad)
            .unwrap_err()
            .contains("prospective_forecast requires"));
    }

    #[test]
    fn rejects_calibrated_label_without_evaluation_evidence() {
        let bad = valid_payload()
            .replace("synthetic_fixture", "calibrated_simulation")
            .replace("contract_smoke", "retrospective");
        assert!(ScenarioRunEnvelope::from_json(&bad)
            .unwrap_err()
            .contains("calibrated_simulation requires retrospective evaluation evidence"));
    }

    #[test]
    fn rejects_forecast_label_without_forecast_receipt() {
        let bad = valid_payload()
            .replace("synthetic_fixture", "prospective_forecast")
            .replace("contract_smoke", "prospective");
        assert!(ScenarioRunEnvelope::from_json(&bad)
            .unwrap_err()
            .contains("prospective_forecast requires forecast receipt evidence"));
    }

    #[test]
    fn rejects_calibrated_label_with_independent_review_but_no_evaluation() {
        let mut run = ScenarioRunEnvelope::from_json(&valid_payload()).expect("valid fixture");
        run.output_classification = OutputClassification::CalibratedSimulation;
        run.validation.status = ValidationStatus::ExternalReview;
        run.validation.evidence.push(ValidationEvidence {
            evidence_id: "test.review.v1".into(),
            kind: ValidationEvidenceKind::IndependentReview,
            artifact_ref: "test-fixtures/review.json".into(),
            artifact_sha256: "e".repeat(64),
            description: "Review alone is not a retrospective evaluation".into(),
        });
        assert!(run.validate().unwrap_err().contains(
            "calibrated_simulation requires retrospective evaluation evidence"
        ));
    }

    #[test]
    fn rejects_calibrated_label_for_failed_run() {
        let mut run = ScenarioRunEnvelope::from_json(&valid_payload()).expect("valid fixture");
        run.output_classification = OutputClassification::CalibratedSimulation;
        run.validation.status = ValidationStatus::Retrospective;
        run.validation.evidence.push(ValidationEvidence {
            evidence_id: "test.hindcast.v1".into(),
            kind: ValidationEvidenceKind::RetrospectiveEvaluation,
            artifact_ref: "test-fixtures/hindcast.json".into(),
            artifact_sha256: "f".repeat(64),
            description: "Synthetic receipt shape test only".into(),
        });
        run.run_status = RunStatus::Failed;
        assert!(run.validate().unwrap_err().contains("require a completed run"));
    }

    #[test]
    fn rejects_forecast_label_for_failed_run() {
        let mut run = ScenarioRunEnvelope::from_json(&valid_payload()).expect("valid fixture");
        run.output_classification = OutputClassification::ProspectiveForecast;
        run.validation.status = ValidationStatus::Prospective;
        run.validation.evidence.push(ValidationEvidence {
            evidence_id: "test.forecast.v1".into(),
            kind: ValidationEvidenceKind::ProspectiveForecast,
            artifact_ref: "test-fixtures/forecast.json".into(),
            artifact_sha256: "f".repeat(64),
            description: "Synthetic receipt shape test only".into(),
        });
        run.run_status = RunStatus::Failed;
        assert!(run.validate().unwrap_err().contains("require a completed run"));
    }

    #[test]
    fn accepts_calibrated_label_with_hashed_evaluation_receipt_shape() {
        let mut run = ScenarioRunEnvelope::from_json(&valid_payload()).expect("valid fixture");
        run.output_classification = OutputClassification::CalibratedSimulation;
        run.validation.status = ValidationStatus::Retrospective;
        run.validation.evidence.push(ValidationEvidence {
            evidence_id: "test.hindcast.v1".into(),
            kind: ValidationEvidenceKind::RetrospectiveEvaluation,
            artifact_ref: "test-fixtures/hindcast.json".into(),
            artifact_sha256: "d".repeat(64),
            description: "Synthetic receipt shape test only".into(),
        });
        assert!(run.validate().is_ok());
    }

    #[test]
    fn rejects_duplicate_metric_rows() {
        let mut run = ScenarioRunEnvelope::from_json(&valid_payload()).expect("valid fixture");
        run.outputs.push(run.outputs[0].clone());
        assert!(run.validate().unwrap_err().contains("duplicate metric row"));
    }

    #[test]
    fn rejects_unknown_fields() {
        let bad = valid_payload().replace(
            "\"warnings\":[\"not empirical\"]",
            "\"unknown_claim\":true,\"warnings\":[\"not empirical\"]",
        );
        assert!(ScenarioRunEnvelope::from_json(&bad)
            .unwrap_err()
            .contains("unknown field"));
    }
}
