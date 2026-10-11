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
use sha2::{Digest, Sha256};

pub mod technology_tree;

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
    /// Numeric year coordinate. Interpret only with time_basis; an absent basis
    /// preserves legacy v0.1 behavior and must not be inferred as calendar time.
    pub year: i32,
    pub time_basis: Option<OutputTimeBasis>,
    pub unit: String,
    pub statistic: OutputStatistic,
    pub value: f64,
    pub sample_count: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutputTimeBasis {
    CalendarYear,
    ScenarioRelativeYear,
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
        // These two schema-required fields are nullable. Option<T> alone cannot
        // distinguish an explicit JSON null from a missing key, so check presence
        // before deserializing into the typed contract. Deserialize the raw bytes
        // again below so Serde still rejects duplicate struct fields.
        let value: serde_json::Value = serde_json::from_str(raw)
            .map_err(|error| format!("invalid Humanity Atlas run JSON: {error}"))?;
        let object = value.as_object()
            .ok_or_else(|| "invalid Humanity Atlas run JSON: top level must be an object".to_string())?;
        if !object.contains_key("deterministic") {
            return Err("invalid Humanity Atlas run JSON: missing required field deterministic".into());
        }
        let inputs = object.get("inputs")
            .and_then(serde_json::Value::as_object)
            .ok_or_else(|| "invalid Humanity Atlas run JSON: inputs must be an object".to_string())?;
        if !inputs.contains_key("dataset_sha256") {
            return Err("invalid Humanity Atlas run JSON: missing required field inputs.dataset_sha256".into());
        }

        let run: Self = serde_json::from_str(raw)
            .map_err(|error| format!("invalid Humanity Atlas run JSON: {error}"))?;
        run.validate()?;
        Ok(run)
    }

    /// Verify exact bytes supplied by a caller against a declared artifact digest.
    ///
    /// This does not retrieve `artifact_ref`, authenticate the publisher, establish
    /// scientific validity, or resolve `json_pointer`; callers should perform
    /// semantic parsing only after this integrity check succeeds.
    pub fn verify_artifact_bytes(&self, artifact_id: &str, bytes: &[u8]) -> Result<(), String> {
        self.validate()?;
        let artifact = self
            .artifacts
            .iter()
            .find(|item| item.artifact_id == artifact_id)
            .ok_or_else(|| format!("unknown artifact_id: {artifact_id}"))?;
        let actual = format!("{:x}", Sha256::digest(bytes));
        if actual != artifact.artifact_sha256 {
            return Err(format!("artifact digest mismatch for {artifact_id}"));
        }
        Ok(())
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != RUN_SCHEMA_VERSION {
            return Err(format!(
                "unsupported run schema version: {}",
                self.schema_version
            ));
        }
        let run_id_len = unicode_len(&self.run_id);
        let scenario_id_len = unicode_len(&self.scenario_id);
        if self.run_id.trim().is_empty()
            || self.scenario_id.trim().is_empty()
            || !(3..=200).contains(&run_id_len)
            || !(3..=180).contains(&scenario_id_len)
        {
            return Err("run_id must be non-empty and 3..=200 characters; scenario_id must be non-empty and 3..=180 characters".into());
        }
        if self.seed > 9_007_199_254_740_991 {
            return Err("seed exceeds the exact integer range allowed by the run schema".into());
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
            if source.trim().is_empty() || unicode_len(source) > 200 {
                return Err("selected_definition_source must be 1..=200 characters when present".into());
            }
        }
        if let Some(kind) = self.engine.model_identity_kind.as_deref() {
            if !matches!(kind, "full_model" | "technology_tree_definition" | "unknown") {
                return Err("unsupported model_identity_kind".into());
            }
        }
        if self.engine.model_identity_kind.as_deref() == Some("technology_tree_definition") {
            let scenario_kind = self.inputs.scenario_identity_kind.as_deref();
            let input_source = self.inputs.selected_definition_source.as_deref();
            let engine_source = self.engine.definition_source.as_deref();
            if scenario_kind != Some("canonical_effective_scenario")
                || input_source.is_none()
                || engine_source.is_none()
                || self.inputs.resolved_config_sha256.is_none()
                || self.inputs.resolved_params_sha256.is_none()
            {
                return Err("technology_tree_definition identity requires canonical scenario identity, selected/effective definition source, and resolved config/params digests".into());
            }
            if input_source != engine_source {
                return Err("selected_definition_source disagrees with engine definition_source".into());
            }
        }
        if self.engine.source_revision.as_deref().is_some_and(
            |value| value.trim().is_empty() || unicode_len(value) > 120
        )
            || self.engine.definition_source.as_deref().is_some_and(|value| value.trim().is_empty())
            || self.engine.definition_source.as_deref().is_some_and(|value| unicode_len(value) > 200)
        {
            return Err("source_revision must be non-empty and at most 120 characters; definition_source must be non-empty and at most 200 characters".into());
        }
        if self.engine.adapter.trim().is_empty()
            || unicode_len(&self.engine.adapter) > 120
            || self.engine.engine_version.trim().is_empty()
            || unicode_len(&self.engine.engine_version) > 120
        {
            return Err("engine adapter and version must be non-empty and at most 120 characters".into());
        }
        if self.replicates == 0 || self.replicates > 1_000_000 {
            return Err("replicates must be within the schema range 1..=1000000".into());
        }
        if self.run_status == RunStatus::Completed && self.outputs.is_empty() {
            return Err("a completed run must contain at least one output row".into());
        }
        if self.validation.notes.iter().any(|note| note.trim().is_empty()) {
            return Err("validation notes must be non-empty strings".into());
        }
        if self.warnings.iter().any(|warning| warning.trim().is_empty()) {
            return Err("warnings must be non-empty strings".into());
        }
        for evidence in &self.validation.evidence {
            if evidence.evidence_id.trim().is_empty()
                || unicode_len(&evidence.evidence_id) > 200
                || evidence.artifact_ref.trim().is_empty()
                || unicode_len(&evidence.artifact_ref) > 500
                || evidence.description.trim().is_empty()
                || unicode_len(&evidence.description) > 1200
            {
                return Err("validation evidence fields must be non-empty and within their declared length bounds".into());
            }
            if !is_lowercase_sha256(&evidence.artifact_sha256) {
                return Err("validation evidence artifact_sha256 must be lowercase SHA-256".into());
            }
        }

        let mut artifact_ids = BTreeSet::new();
        for artifact in &self.artifacts {
            if artifact.artifact_id.trim().is_empty()
                || unicode_len(&artifact.artifact_id) > 200
                || artifact.role.trim().is_empty()
                || unicode_len(&artifact.role) > 120
                || artifact.media_type.trim().is_empty()
                || unicode_len(&artifact.media_type) > 120
                || artifact.artifact_ref.trim().is_empty()
                || unicode_len(&artifact.artifact_ref) > 2000
            {
                return Err("artifact ID, role, media type, and reference must be within their declared length limits".into());
            }
            if !is_lowercase_sha256(&artifact.artifact_sha256) {
                return Err("artifact artifact_sha256 must be lowercase SHA-256".into());
            }
            if let Some(reference_digest) = artifact.artifact_ref.strip_prefix("urn:sha256:") {
                if !is_lowercase_sha256(reference_digest) {
                    return Err("urn:sha256 artifact_ref must contain a lowercase SHA-256 digest".into());
                }
                if reference_digest != artifact.artifact_sha256 {
                    return Err("urn:sha256 artifact_ref digest disagrees with artifact_sha256".into());
                }
            }
            if artifact.json_pointer.as_deref().is_some_and(
                |pointer| unicode_len(pointer) > 1000 || !is_valid_json_pointer(pointer)
            ) {
                return Err(format!("invalid JSON Pointer for artifact {}", artifact.artifact_id));
            }
            if !artifact_ids.insert(artifact.artifact_id.as_str()) {
                return Err(format!("duplicate artifact_id: {}", artifact.artifact_id));
            }
        }

        let mut unique_rows = BTreeSet::new();
        for row in &self.outputs {
            let metric_id_len = unicode_len(&row.metric_id);
            let unit_len = unicode_len(&row.unit);
            if row.metric_id.trim().is_empty()
                || row.unit.trim().is_empty()
                || !(1..=200).contains(&metric_id_len)
                || !(1..=80).contains(&unit_len)
            {
                return Err("metric_id must be non-empty and 1..=200 characters; unit must be non-empty and 1..=80 characters".into());
            }
            if !(-100_000..=100_000).contains(&row.year) {
                return Err(format!("metric {} year is outside the schema range", row.metric_id));
            }
            if !row.value.is_finite() {
                return Err(format!("metric {} contains a non-finite value", row.metric_id));
            }
            if row.sample_count == 0 || row.sample_count > 1_000_000 {
                return Err(format!("metric {} sample_count must be within 1..=1000000", row.metric_id));
            }
            let key = (row.metric_id.as_str(), row.year, row.time_basis, row.statistic);
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

fn unicode_len(value: &str) -> usize {
    value.chars().count()
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
            artifact_ref: format!("urn:sha256:{}", "d".repeat(64)),
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
            artifact_ref: format!("urn:sha256:{}", "d".repeat(64)),
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
            artifact_ref: format!("urn:sha256:{}", "d".repeat(64)),
            artifact_sha256: "d".repeat(64),
            json_pointer: Some("/technology~2tree".into()),
        });
        assert!(run.validate().unwrap_err().contains("invalid JSON Pointer"));
    }

    #[test]
    fn verifies_exact_source_artifact_bytes_before_consumption() {
        let bytes = br#"{"technology_tree":{"milestone_count":4}}"#;
        let digest = format!("{:x}", Sha256::digest(bytes));
        let mut run = ScenarioRunEnvelope::from_json(&valid_payload()).expect("valid fixture");
        run.artifacts.push(RunArtifactReference {
            artifact_id: "source-output".into(),
            role: "source_simulation_output".into(),
            media_type: "application/json".into(),
            artifact_ref: format!("urn:sha256:{digest}"),
            artifact_sha256: digest,
            json_pointer: Some("/technology_tree".into()),
        });
        assert!(run.verify_artifact_bytes("source-output", bytes).is_ok());
    }

    #[test]
    fn rejects_tampered_source_artifact_bytes() {
        let bytes = br#"{"technology_tree":{"milestone_count":4}}"#;
        let digest = format!("{:x}", Sha256::digest(bytes));
        let mut run = ScenarioRunEnvelope::from_json(&valid_payload()).expect("valid fixture");
        run.artifacts.push(RunArtifactReference {
            artifact_id: "source-output".into(),
            role: "source_simulation_output".into(),
            media_type: "application/json".into(),
            artifact_ref: format!("urn:sha256:{digest}"),
            artifact_sha256: digest,
            json_pointer: Some("/technology_tree".into()),
        });
        assert!(run
            .verify_artifact_bytes("source-output", br#"{"technology_tree":{"milestone_count":5}}"#)
            .unwrap_err()
            .contains("artifact digest mismatch"));
    }

    #[test]
    fn rejects_content_addressed_reference_digest_mismatch() {
        let mut run = ScenarioRunEnvelope::from_json(&valid_payload()).expect("valid fixture");
        run.artifacts.push(RunArtifactReference {
            artifact_id: "mismatched-reference".into(),
            role: "source_simulation_output".into(),
            media_type: "application/json".into(),
            artifact_ref: format!("urn:sha256:{}", "e".repeat(64)),
            artifact_sha256: "d".repeat(64),
            json_pointer: None,
        });
        assert!(run.validate().unwrap_err().contains("disagrees with artifact_sha256"));
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
    fn preserves_scenario_relative_metric_time_basis() {
        let payload = valid_payload().replace(
            "\"year\":2030",
            "\"year\":150,\"time_basis\":\"scenario_relative_year\"",
        );
        let run = ScenarioRunEnvelope::from_json(&payload).expect("valid relative-time run");
        assert_eq!(run.outputs[0].year, 150);
        assert_eq!(run.outputs[0].time_basis, Some(OutputTimeBasis::ScenarioRelativeYear));
    }

    #[test]
    fn preserves_legacy_metric_without_time_basis_as_unspecified() {
        let run = ScenarioRunEnvelope::from_json(&valid_payload()).expect("legacy envelope");
        assert_eq!(run.outputs[0].time_basis, None);
    }

    #[test]
    fn rejects_tree_scoped_model_without_resolved_input_bindings() {
        let mut run = ScenarioRunEnvelope::from_json(&valid_payload()).expect("valid fixture");
        run.engine.model_identity_kind = Some("technology_tree_definition".into());
        assert!(run.validate().unwrap_err().contains("requires canonical scenario identity"));
    }

    #[test]
    fn rejects_mismatched_selected_and_engine_definition_sources() {
        let mut run = ScenarioRunEnvelope::from_json(&valid_payload()).expect("valid fixture");
        run.engine.model_identity_kind = Some("technology_tree_definition".into());
        run.inputs.scenario_identity_kind = Some("canonical_effective_scenario".into());
        run.inputs.selected_definition_source = Some("in_code_default_tree".into());
        run.inputs.resolved_config_sha256 = Some("a".repeat(64));
        run.inputs.resolved_params_sha256 = Some("b".repeat(64));
        run.engine.definition_source = Some("loaded_external_toml".into());
        assert!(run.validate().unwrap_err().contains("disagrees with engine definition_source"));
    }

    #[test]
    fn rejects_missing_required_nullable_deterministic_field() {
        let bad = valid_payload().replace("\"deterministic\":true,", "");
        assert!(ScenarioRunEnvelope::from_json(&bad).unwrap_err().contains("missing required field deterministic"));
    }

    #[test]
    fn rejects_missing_required_nullable_dataset_digest_field() {
        let digest = "b".repeat(64);
        let bad = valid_payload().replace(&format!(",\"dataset_sha256\":\"{digest}\""), "");
        assert!(ScenarioRunEnvelope::from_json(&bad).unwrap_err().contains("missing required field inputs.dataset_sha256"));
    }

    #[test]
    fn rejects_source_revision_above_schema_limit() {
        let mut run = ScenarioRunEnvelope::from_json(&valid_payload()).expect("valid fixture");
        run.engine.source_revision = Some("r".repeat(121));
        assert!(run.validate().unwrap_err().contains("source_revision must be non-empty and at most 120"));
    }

    #[test]
    fn rejects_seed_above_schema_safe_integer_limit() {
        let bad = valid_payload().replace("\"seed\":0", "\"seed\":9007199254740992");
        assert!(ScenarioRunEnvelope::from_json(&bad).unwrap_err().contains("seed exceeds"));
    }

    #[test]
    fn rejects_replicates_above_schema_limit() {
        let bad = valid_payload().replace("\"replicates\":1", "\"replicates\":1000001");
        assert!(ScenarioRunEnvelope::from_json(&bad).unwrap_err().contains("replicates must be within"));
    }

    #[test]
    fn rejects_metric_year_above_schema_limit() {
        let mut run = ScenarioRunEnvelope::from_json(&valid_payload()).expect("valid fixture");
        run.outputs[0].year = 100001;
        assert!(run.validate().unwrap_err().contains("year is outside"));
    }

    #[test]
    fn rejects_metric_sample_count_above_schema_limit() {
        let mut run = ScenarioRunEnvelope::from_json(&valid_payload()).expect("valid fixture");
        run.outputs[0].sample_count = 1_000_001;
        assert!(run.validate().unwrap_err().contains("sample_count must be within"));
    }

    #[test]
    fn rejects_empty_validation_notes_and_warnings() {
        let mut run = ScenarioRunEnvelope::from_json(&valid_payload()).expect("valid fixture");
        run.validation.notes.push(String::new());
        assert!(run.validate().unwrap_err().contains("validation notes"));
        run.validation.notes.pop();
        run.warnings.push(String::new());
        assert!(run.validate().unwrap_err().contains("warnings"));
    }

    #[test]
    fn rejects_overlong_source_revision() {
        let mut run = ScenarioRunEnvelope::from_json(&valid_payload()).expect("valid fixture");
        run.engine.source_revision = Some("r".repeat(121));
        assert!(run.validate().unwrap_err().contains("source_revision must be non-empty and at most 120"));
    }

    #[test]
    fn rejects_overlong_run_id() {
        let long_id = "r".repeat(201);
        let raw = valid_payload().replace("\"run_id\":\"test-run-001\"", &format!("\"run_id\":\"{long_id}\""));
        assert!(ScenarioRunEnvelope::from_json(&raw).unwrap_err().contains("run_id must be"));
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
    fn allows_same_metric_coordinate_with_distinct_time_basis() {
        let mut run = ScenarioRunEnvelope::from_json(&valid_payload()).expect("valid fixture");
        let mut distinct = run.outputs[0].clone();
        distinct.time_basis = Some(OutputTimeBasis::ScenarioRelativeYear);
        run.outputs.push(distinct);
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
