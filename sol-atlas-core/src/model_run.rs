// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Reproducibility manifest for a completed or failed model execution.
//!
//! A manifest records what was run, on which versioned inputs, with which
//! configuration and assumptions, and what artifacts resulted. It is an audit
//! record, not proof that a model is valid or that its source data are true.
//! Every artifact digest must be computed over the exact bytes of the artifact.
//! Canonical manifest hashing and external verification are intentionally left
//! to a subsequent layer.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Current schema version accepted by this validator.
pub const MODEL_RUN_SCHEMA_VERSION: u16 = 1;

/// An artifact referenced by a model run using its immutable content digest.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactReference {
    /// Stable identifier for the artifact within its source system.
    pub artifact_id: String,
    /// SHA-256 digest of the exact artifact bytes, prefixed with `sha256:`.
    pub digest: String,
}

/// Explicit spatial, entity and temporal boundaries of a model run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ModelScope {
    /// Human-readable bounded purpose, not an unbounded phrase such as "the world".
    pub description: String,
    /// Stable geography identifiers, if this model is spatially scoped.
    pub geography_ids: Vec<String>,
    /// Stable entity identifiers included in the assessment, if known.
    pub entity_ids: Vec<String>,
    /// Inclusive start of the modeled or assessed period, as Unix seconds.
    pub period_start_unix_s: i64,
    /// Inclusive end of the modeled or assessed period, as Unix seconds.
    pub period_end_unix_s: i64,
}

/// Declared reproducibility contract for a model run.
///
/// This is a declaration that downstream tooling must test; it does not itself
/// guarantee that repeated executions satisfy the declaration.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "mode", rename_all = "snake_case")]
pub enum ReproducibilityDeclaration {
    /// Repeated runs are expected to produce byte-identical canonical outputs.
    BitwiseDeterministic,
    /// Floating point outputs may differ within explicitly declared tolerances.
    NumericallyBounded {
        absolute_tolerance: f64,
        relative_tolerance: f64,
    },
    /// Stochastic behavior is reproducible only when the RNG and seed are recorded.
    SeededStochastic { rng_algorithm: String },
}

/// Lifecycle result of a model execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelRunStatus {
    Completed,
    Failed,
    Cancelled,
}

/// Audit manifest for one model execution.
///
/// Input and output digests bind the record to exact artifacts. `model_artifact_digest`
/// identifies the model artifact, `code_revision` identifies its source revision,
/// and `configuration_digest` identifies the canonical configuration bytes.
/// Assumptions are human-readable and must be retained in the manifest, not only
/// described in a mutable external document.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelRunManifest {
    pub schema_version: u16,
    pub run_id: String,
    pub model_id: String,
    pub model_version: String,
    pub code_revision: String,
    pub model_artifact_digest: String,
    pub configuration_digest: String,
    pub input_artifacts: Vec<ArtifactReference>,
    pub output_artifacts: Vec<ArtifactReference>,
    pub scope: ModelScope,
    pub assumptions: Vec<String>,
    pub reproducibility: ReproducibilityDeclaration,
    pub random_seed: Option<u64>,
    pub status: ModelRunStatus,
    pub started_at_unix_s: i64,
    pub ended_at_unix_s: i64,
    /// Required for failed/cancelled runs; may contain a sanitized diagnostic.
    pub failure_reason: Option<String>,
}

/// Structural manifest validation failures. Truth, causal validity, provenance
/// independence and model performance require separate checks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ModelRunValidationIssue {
    UnsupportedSchemaVersion,
    EmptyRunId,
    EmptyModelId,
    EmptyModelVersion,
    EmptyCodeRevision,
    InvalidModelArtifactDigest,
    InvalidConfigurationDigest,
    EmptyScopeDescription,
    InvalidScopePeriod,
    EmptyGeographyId,
    DuplicateGeographyId,
    EmptyEntityId,
    DuplicateEntityId,
    EmptyInputArtifactId,
    InvalidInputArtifactDigest,
    DuplicateInputArtifactId,
    EmptyOutputArtifactId,
    InvalidOutputArtifactDigest,
    DuplicateOutputArtifactId,
    EmptyAssumption,
    InvalidExecutionTimeRange,
    MissingOutputForCompletedRun,
    UnexpectedFailureReasonForCompletedRun,
    MissingFailureReason,
    EmptyFailureReason,
    InvalidAbsoluteTolerance,
    InvalidRelativeTolerance,
    EmptyRngAlgorithm,
    MissingRandomSeed,
}

impl ModelRunManifest {
    /// Validate all detectable structural invariants in one pass.
    ///
    /// Validation fails closed: any issue returns an error list. Successful
    /// validation does not certify that the model or inputs are scientifically
    /// correct, that a run is reproducible, or that conclusions are causal.
    pub fn validate(&self) -> Result<(), Vec<ModelRunValidationIssue>> {
        let mut issues = Vec::new();

        if self.schema_version != MODEL_RUN_SCHEMA_VERSION {
            issues.push(ModelRunValidationIssue::UnsupportedSchemaVersion);
        }
        if is_blank(&self.run_id) {
            issues.push(ModelRunValidationIssue::EmptyRunId);
        }
        if is_blank(&self.model_id) {
            issues.push(ModelRunValidationIssue::EmptyModelId);
        }
        if is_blank(&self.model_version) {
            issues.push(ModelRunValidationIssue::EmptyModelVersion);
        }
        if is_blank(&self.code_revision) {
            issues.push(ModelRunValidationIssue::EmptyCodeRevision);
        }
        if !is_sha256_digest(&self.model_artifact_digest) {
            issues.push(ModelRunValidationIssue::InvalidModelArtifactDigest);
        }
        if !is_sha256_digest(&self.configuration_digest) {
            issues.push(ModelRunValidationIssue::InvalidConfigurationDigest);
        }

        if is_blank(&self.scope.description) {
            issues.push(ModelRunValidationIssue::EmptyScopeDescription);
        }
        if self.scope.period_start_unix_s > self.scope.period_end_unix_s {
            issues.push(ModelRunValidationIssue::InvalidScopePeriod);
        }
        validate_unique_ids(
            &self.scope.geography_ids,
            ModelRunValidationIssue::EmptyGeographyId,
            ModelRunValidationIssue::DuplicateGeographyId,
            &mut issues,
        );
        validate_unique_ids(
            &self.scope.entity_ids,
            ModelRunValidationIssue::EmptyEntityId,
            ModelRunValidationIssue::DuplicateEntityId,
            &mut issues,
        );

        let mut input_ids = HashSet::new();
        for artifact in &self.input_artifacts {
            if is_blank(&artifact.artifact_id) {
                issues.push(ModelRunValidationIssue::EmptyInputArtifactId);
            } else if !input_ids.insert(artifact.artifact_id.as_str()) {
                issues.push(ModelRunValidationIssue::DuplicateInputArtifactId);
            }
            if !is_sha256_digest(&artifact.digest) {
                issues.push(ModelRunValidationIssue::InvalidInputArtifactDigest);
            }
        }

        let mut output_ids = HashSet::new();
        for artifact in &self.output_artifacts {
            if is_blank(&artifact.artifact_id) {
                issues.push(ModelRunValidationIssue::EmptyOutputArtifactId);
            } else if !output_ids.insert(artifact.artifact_id.as_str()) {
                issues.push(ModelRunValidationIssue::DuplicateOutputArtifactId);
            }
            if !is_sha256_digest(&artifact.digest) {
                issues.push(ModelRunValidationIssue::InvalidOutputArtifactDigest);
            }
        }

        if self.assumptions.iter().any(|assumption| is_blank(assumption)) {
            issues.push(ModelRunValidationIssue::EmptyAssumption);
        }
        if self.started_at_unix_s > self.ended_at_unix_s {
            issues.push(ModelRunValidationIssue::InvalidExecutionTimeRange);
        }

        match self.status {
            ModelRunStatus::Completed => {
                if self.output_artifacts.is_empty() {
                    issues.push(ModelRunValidationIssue::MissingOutputForCompletedRun);
                }
                if self.failure_reason.is_some() {
                    issues.push(ModelRunValidationIssue::UnexpectedFailureReasonForCompletedRun);
                }
            }
            ModelRunStatus::Failed | ModelRunStatus::Cancelled => {
                match self.failure_reason.as_deref() {
                    None => issues.push(ModelRunValidationIssue::MissingFailureReason),
                    Some(reason) if is_blank(reason) => {
                        issues.push(ModelRunValidationIssue::EmptyFailureReason)
                    }
                    Some(_) => {}
                }
            }
        }

        match &self.reproducibility {
            ReproducibilityDeclaration::BitwiseDeterministic => {}
            ReproducibilityDeclaration::NumericallyBounded {
                absolute_tolerance,
                relative_tolerance,
            } => {
                if !absolute_tolerance.is_finite()
                    || *absolute_tolerance < 0.0
                    || (*absolute_tolerance == 0.0 && *relative_tolerance == 0.0)
                {
                    issues.push(ModelRunValidationIssue::InvalidAbsoluteTolerance);
                }
                if !relative_tolerance.is_finite() || *relative_tolerance < 0.0 {
                    issues.push(ModelRunValidationIssue::InvalidRelativeTolerance);
                }
            }
            ReproducibilityDeclaration::SeededStochastic { rng_algorithm } => {
                if is_blank(rng_algorithm) {
                    issues.push(ModelRunValidationIssue::EmptyRngAlgorithm);
                }
                if self.random_seed.is_none() {
                    issues.push(ModelRunValidationIssue::MissingRandomSeed);
                }
            }
        }

        if issues.is_empty() {
            Ok(())
        } else {
            Err(issues)
        }
    }
}

fn validate_unique_ids(
    ids: &[String],
    empty_issue: ModelRunValidationIssue,
    duplicate_issue: ModelRunValidationIssue,
    issues: &mut Vec<ModelRunValidationIssue>,
) {
    let mut seen = HashSet::new();
    for id in ids {
        if is_blank(id) {
            issues.push(empty_issue);
        } else if !seen.insert(id.as_str()) {
            issues.push(duplicate_issue);
        }
    }
}

fn is_blank(value: &str) -> bool {
    value.trim().is_empty()
}

fn is_sha256_digest(value: &str) -> bool {
    let Some(hex) = value.strip_prefix("sha256:") else {
        return false;
    };
    hex.len() == 64 && hex.bytes().all(|byte| byte.is_ascii_hexdigit())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digest(character: char) -> String {
        format!("sha256:{}", character.to_string().repeat(64))
    }

    fn valid_manifest() -> ModelRunManifest {
        ModelRunManifest {
            schema_version: MODEL_RUN_SCHEMA_VERSION,
            run_id: "run-2026-001".into(),
            model_id: "urban-service-resilience".into(),
            model_version: "0.1.0".into(),
            code_revision: "c".repeat(40),
            model_artifact_digest: digest('a'),
            configuration_digest: digest('b'),
            input_artifacts: vec![ArtifactReference {
                artifact_id: "water-electricity-observations".into(),
                digest: digest('c'),
            }],
            output_artifacts: vec![ArtifactReference {
                artifact_id: "scenario-summary".into(),
                digest: digest('d'),
            }],
            scope: ModelScope {
                description: "Example service area and 24-hour stress scenario".into(),
                geography_ids: vec!["za-gp-example".into()],
                entity_ids: vec!["water-plant-001".into(), "feeder-021".into()],
                period_start_unix_s: 1_790_000_000,
                period_end_unix_s: 1_790_086_400,
            },
            assumptions: vec![
                "Plant service depends on the modeled feeder".into(),
                "No backup generation is available in this fixture".into(),
            ],
            reproducibility: ReproducibilityDeclaration::BitwiseDeterministic,
            random_seed: None,
            status: ModelRunStatus::Completed,
            started_at_unix_s: 1_790_000_010,
            ended_at_unix_s: 1_790_000_011,
            failure_reason: None,
        }
    }

    #[test]
    fn accepts_well_formed_completed_manifest() {
        assert_eq!(valid_manifest().validate(), Ok(()));
    }

    #[test]
    fn rejects_unsupported_schema_version() {
        let mut manifest = valid_manifest();
        manifest.schema_version = MODEL_RUN_SCHEMA_VERSION + 1;

        assert!(manifest
            .validate()
            .unwrap_err()
            .contains(&ModelRunValidationIssue::UnsupportedSchemaVersion));
    }

    #[test]
    fn rejects_malformed_model_digest() {
        let mut manifest = valid_manifest();
        manifest.model_artifact_digest = "sha256:short".into();

        assert!(manifest
            .validate()
            .unwrap_err()
            .contains(&ModelRunValidationIssue::InvalidModelArtifactDigest));
    }

    #[test]
    fn rejects_duplicate_input_artifact_ids() {
        let mut manifest = valid_manifest();
        manifest.input_artifacts.push(manifest.input_artifacts[0].clone());

        assert!(manifest
            .validate()
            .unwrap_err()
            .contains(&ModelRunValidationIssue::DuplicateInputArtifactId));
    }

    #[test]
    fn rejects_duplicate_output_artifact_ids() {
        let mut manifest = valid_manifest();
        manifest.output_artifacts.push(manifest.output_artifacts[0].clone());

        assert!(manifest
            .validate()
            .unwrap_err()
            .contains(&ModelRunValidationIssue::DuplicateOutputArtifactId));
    }

    #[test]
    fn completed_run_requires_output_artifacts() {
        let mut manifest = valid_manifest();
        manifest.output_artifacts.clear();

        assert!(manifest
            .validate()
            .unwrap_err()
            .contains(&ModelRunValidationIssue::MissingOutputForCompletedRun));
    }

    #[test]
    fn failed_run_requires_a_reason() {
        let mut manifest = valid_manifest();
        manifest.status = ModelRunStatus::Failed;

        assert!(manifest
            .validate()
            .unwrap_err()
            .contains(&ModelRunValidationIssue::MissingFailureReason));
    }

    #[test]
    fn rejects_reversed_scope_period() {
        let mut manifest = valid_manifest();
        manifest.scope.period_start_unix_s = manifest.scope.period_end_unix_s + 1;

        assert!(manifest
            .validate()
            .unwrap_err()
            .contains(&ModelRunValidationIssue::InvalidScopePeriod));
    }

    #[test]
    fn rejects_reversed_execution_times() {
        let mut manifest = valid_manifest();
        manifest.started_at_unix_s = manifest.ended_at_unix_s + 1;

        assert!(manifest
            .validate()
            .unwrap_err()
            .contains(&ModelRunValidationIssue::InvalidExecutionTimeRange));
    }

    #[test]
    fn rejects_non_finite_numerical_tolerance() {
        let mut manifest = valid_manifest();
        manifest.reproducibility = ReproducibilityDeclaration::NumericallyBounded {
            absolute_tolerance: f64::NAN,
            relative_tolerance: 1e-6,
        };

        assert!(manifest
            .validate()
            .unwrap_err()
            .contains(&ModelRunValidationIssue::InvalidAbsoluteTolerance));
    }

    #[test]
    fn seeded_stochastic_run_requires_seed_and_algorithm() {
        let mut manifest = valid_manifest();
        manifest.reproducibility = ReproducibilityDeclaration::SeededStochastic {
            rng_algorithm: "PCG64".into(),
        };
        manifest.random_seed = None;

        let issues = manifest.validate().unwrap_err();
        assert!(issues.contains(&ModelRunValidationIssue::MissingRandomSeed));
        assert!(!issues.contains(&ModelRunValidationIssue::EmptyRngAlgorithm));
    }

    #[test]
    fn rejects_blank_assumption() {
        let mut manifest = valid_manifest();
        manifest.assumptions.push("  ".into());

        assert!(manifest
            .validate()
            .unwrap_err()
            .contains(&ModelRunValidationIssue::EmptyAssumption));
    }

    #[test]
    fn json_round_trip_preserves_manifest() {
        let manifest = valid_manifest();
        let json = serde_json::to_string(&manifest).expect("serialize manifest");
        let decoded: ModelRunManifest =
            serde_json::from_str(&json).expect("deserialize manifest");
        assert_eq!(decoded, manifest);
        assert_eq!(decoded.validate(), Ok(()));
    }
}
