// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Manifest-bound evidence checks for reproducible energy-study runs.
//!
//! This module binds exact input bytes and interval-output bytes to a declared
//! study. It does not prove that a process actually executed those bytes, that
//! source data is genuine, or that a model is scientifically accurate. Callers
//! must still run the independent physical verifier.

use crate::energy_study::{
    EnergyEvidenceClass, EnergyStudyManifestV1, EnergyStudyValidationError,
};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::fmt::Write as _;

pub const ENERGY_STUDY_RECEIPT_SCHEMA_VERSION: u16 = 1;
const ARTIFACT_DOMAIN: &[u8] = b"luminous-dynamics.energy-study-artifacts.v1\0";

/// Exact bytes of every input declared by the manifest.
/// Dataset keys are manifest dataset_id values.
#[derive(Debug, Clone, Copy)]
pub struct EnergyStudyArtifactBytesV1<'a> {
    pub datasets: &'a [(&'a str, &'a [u8])],
    pub asset_registry: &'a [u8],
    pub constraints: &'a [u8],
    pub model_configuration: &'a [u8],
    pub policy_configuration: &'a [u8],
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnergyStudyArtifactVerificationReceiptV1 {
    pub manifest_sha256: String,
    pub case_sha256: String,
    pub artifact_set_sha256: String,
    pub verified_dataset_count: u32,
    pub verified_configuration_count: u8,
    /// Declared classifications, not independent proof of source authenticity.
    pub declared_evidence_classes: Vec<(String, EnergyEvidenceClass)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnergyStudyIntervalReceiptV1 {
    pub interval_index: u32,
    /// UTC Unix timestamp for the interval start.
    pub interval_start_unix_seconds: i64,
    /// SHA-256 over the exact serialized bytes for this interval's output.
    pub output_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnergyStudyRunReceiptV1 {
    pub schema_version: u16,
    pub manifest_sha256: String,
    pub case_sha256: String,
    pub intervals: Vec<EnergyStudyIntervalReceiptV1>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnergyStudyRunVerificationReceiptV1 {
    pub manifest_sha256: String,
    pub case_sha256: String,
    pub verified_interval_count: u32,
    pub interval_outputs_sha256: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnergyStudyQualificationError {
    InvalidManifest(EnergyStudyValidationError),
    UnsupportedReceiptSchemaVersion,
    InvalidSha256(&'static str),
    ArtifactSetMismatch,
    DuplicateArtifactId,
    ArtifactHashMismatch(&'static str),
    IntervalCountMismatch,
    IntervalIndexMismatch { position: usize },
    IntervalTimestampMismatch { position: usize },
    IntervalOutputHashMismatch { position: usize },
    TimeArithmeticOverflow { position: usize },
}

impl std::fmt::Display for EnergyStudyQualificationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidManifest(error) => write!(f, "invalid study manifest: {error}"),
            Self::UnsupportedReceiptSchemaVersion => {
                f.write_str("unsupported energy-study receipt schema version")
            }
            Self::InvalidSha256(field) => write!(f, "{field} must be a lowercase SHA-256 digest"),
            Self::ArtifactSetMismatch => {
                f.write_str("provided artifact IDs do not exactly match the manifest")
            }
            Self::DuplicateArtifactId => f.write_str("duplicate input artifact ID"),
            Self::ArtifactHashMismatch(field) => {
                write!(f, "input artifact bytes do not match declared digest: {field}")
            }
            Self::IntervalCountMismatch => {
                f.write_str("interval receipt and output counts must equal manifest interval_count")
            }
            Self::IntervalIndexMismatch { position } => {
                write!(f, "interval receipt at position {position} is missing, duplicated, or reordered")
            }
            Self::IntervalTimestampMismatch { position } => {
                write!(f, "interval receipt at position {position} is outside the declared time axis")
            }
            Self::IntervalOutputHashMismatch { position } => {
                write!(f, "interval output bytes do not match receipt at position {position}")
            }
            Self::TimeArithmeticOverflow { position } => {
                write!(f, "timestamp arithmetic overflow at interval {position}")
            }
        }
    }
}

impl std::error::Error for EnergyStudyQualificationError {}

impl From<EnergyStudyValidationError> for EnergyStudyQualificationError {
    fn from(value: EnergyStudyValidationError) -> Self {
        Self::InvalidManifest(value)
    }
}

/// Verify all raw input bytes and produce a deterministic artifact-set digest.
///
/// Missing, extra, duplicate, or mutated dataset artifacts fail closed. The
/// canonical artifact-set digest is byte-compatible with the Mycelix SDK.
pub fn verify_energy_study_artifacts_v1(
    manifest: &EnergyStudyManifestV1,
    artifacts: EnergyStudyArtifactBytesV1<'_>,
) -> Result<EnergyStudyArtifactVerificationReceiptV1, EnergyStudyQualificationError> {
    manifest.validate()?;
    let manifest_sha256 = manifest.digest_sha256()?;
    let case_sha256 = manifest.case_digest_sha256()?;

    let mut seen = HashSet::with_capacity(artifacts.datasets.len());
    for (dataset_id, _) in artifacts.datasets {
        if !seen.insert(*dataset_id) {
            return Err(EnergyStudyQualificationError::DuplicateArtifactId);
        }
    }
    if artifacts.datasets.len() != manifest.datasets.len()
        || manifest.datasets.iter().any(|expected| {
            !artifacts.datasets.iter().any(|(actual_id, _)| *actual_id == expected.dataset_id)
        })
    {
        return Err(EnergyStudyQualificationError::ArtifactSetMismatch);
    }

    let mut entries: Vec<(String, String)> = Vec::with_capacity(manifest.datasets.len() + 4);
    for dataset in &manifest.datasets {
        let (_, bytes) = artifacts.datasets.iter()
            .find(|(dataset_id, _)| *dataset_id == dataset.dataset_id)
            .ok_or(EnergyStudyQualificationError::ArtifactSetMismatch)?;
        let actual_hash = sha256_hex(bytes);
        if actual_hash != dataset.content_sha256 {
            return Err(EnergyStudyQualificationError::ArtifactHashMismatch("dataset"));
        }
        entries.push((format!("dataset:{}", dataset.dataset_id), actual_hash));
    }

    let configs = [
        ("asset_registry", artifacts.asset_registry, manifest.asset_registry_sha256.as_str()),
        ("constraints", artifacts.constraints, manifest.constraints_sha256.as_str()),
        ("model_configuration", artifacts.model_configuration, manifest.model.configuration_sha256.as_str()),
        ("policy_configuration", artifacts.policy_configuration, manifest.policy.configuration_sha256.as_str()),
    ];
    for (artifact_id, bytes, expected_hash) in configs {
        let actual_hash = sha256_hex(bytes);
        if actual_hash != expected_hash {
            return Err(EnergyStudyQualificationError::ArtifactHashMismatch(artifact_id));
        }
        entries.push((artifact_id.to_owned(), actual_hash));
    }

    entries.sort_by(|left, right| left.0.as_bytes().cmp(right.0.as_bytes()));
    let mut canonical = Vec::new();
    canonical.extend_from_slice(ARTIFACT_DOMAIN);
    canonical.extend_from_slice(&(entries.len() as u32).to_be_bytes());
    for (artifact_id, digest) in &entries {
        write_text(&mut canonical, artifact_id);
        write_text(&mut canonical, digest);
    }

    let declared_evidence_classes = manifest.datasets.iter()
        .map(|dataset| (dataset.dataset_id.clone(), dataset.evidence_class))
        .collect();

    Ok(EnergyStudyArtifactVerificationReceiptV1 {
        manifest_sha256,
        case_sha256,
        artifact_set_sha256: sha256_hex(&canonical),
        verified_dataset_count: manifest.datasets.len() as u32,
        verified_configuration_count: 4,
        declared_evidence_classes,
    })
}

/// Verify that ordered interval-output bytes match a manifest-bound receipt.
///
/// This checks identity, byte hashes, sequence completeness, and timestamps;
/// it does not parse payloads, test physical conservation, or attest execution.
pub fn verify_energy_study_run_receipt_v1(
    manifest: &EnergyStudyManifestV1,
    receipt: &EnergyStudyRunReceiptV1,
    interval_output_bytes: &[&[u8]],
) -> Result<EnergyStudyRunVerificationReceiptV1, EnergyStudyQualificationError> {
    manifest.validate()?;
    if receipt.schema_version != ENERGY_STUDY_RECEIPT_SCHEMA_VERSION {
        return Err(EnergyStudyQualificationError::UnsupportedReceiptSchemaVersion);
    }

    let manifest_sha256 = manifest.digest_sha256()?;
    let case_sha256 = manifest.case_digest_sha256()?;
    if !is_sha256(&receipt.manifest_sha256) {
        return Err(EnergyStudyQualificationError::InvalidSha256("receipt.manifest_sha256"));
    }
    if !is_sha256(&receipt.case_sha256) {
        return Err(EnergyStudyQualificationError::InvalidSha256("receipt.case_sha256"));
    }
    if receipt.manifest_sha256 != manifest_sha256 || receipt.case_sha256 != case_sha256 {
        return Err(EnergyStudyQualificationError::ArtifactSetMismatch);
    }

    let expected_count = usize::try_from(manifest.interval_count)
        .map_err(|_| EnergyStudyQualificationError::IntervalCountMismatch)?;
    if receipt.intervals.len() != expected_count || interval_output_bytes.len() != expected_count {
        return Err(EnergyStudyQualificationError::IntervalCountMismatch);
    }

    let mut canonical = Vec::with_capacity(expected_count.saturating_mul(44) + 64);
    canonical.extend_from_slice(b"luminous-dynamics.energy-study-interval-outputs.v1\0");
    canonical.extend_from_slice(&manifest.interval_count.to_be_bytes());

    for (position, (interval, output_bytes)) in receipt.intervals.iter()
        .zip(interval_output_bytes.iter()).enumerate()
    {
        let expected_index = u32::try_from(position)
            .map_err(|_| EnergyStudyQualificationError::IntervalCountMismatch)?;
        if interval.interval_index != expected_index {
            return Err(EnergyStudyQualificationError::IntervalIndexMismatch { position });
        }
        let offset = i64::from(manifest.interval_seconds)
            .checked_mul(i64::from(expected_index))
            .ok_or(EnergyStudyQualificationError::TimeArithmeticOverflow { position })?;
        let expected_timestamp = manifest.start_unix_seconds.checked_add(offset)
            .ok_or(EnergyStudyQualificationError::TimeArithmeticOverflow { position })?;
        if interval.interval_start_unix_seconds != expected_timestamp {
            return Err(EnergyStudyQualificationError::IntervalTimestampMismatch { position });
        }
        if !is_sha256(&interval.output_sha256) {
            return Err(EnergyStudyQualificationError::InvalidSha256("interval.output_sha256"));
        }
        let actual_hash = sha256_hex(output_bytes);
        if actual_hash != interval.output_sha256 {
            return Err(EnergyStudyQualificationError::IntervalOutputHashMismatch { position });
        }
        canonical.extend_from_slice(&expected_index.to_be_bytes());
        canonical.extend_from_slice(&expected_timestamp.to_be_bytes());
        canonical.extend_from_slice(&hex_to_bytes(&actual_hash).expect("validated SHA-256 hex"));
    }

    Ok(EnergyStudyRunVerificationReceiptV1 {
        manifest_sha256,
        case_sha256,
        verified_interval_count: manifest.interval_count,
        interval_outputs_sha256: sha256_hex(&canonical),
    })
}

/// Combined report from the artifact and interval-receipt checks.
///
/// A successful result means all declared input bytes and interval output bytes
/// match the manifest-bound receipt. It still does not prove execution or
/// physical correctness; apply the independent microgrid verifier separately.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnergyStudyQualificationReportV1 {
    pub artifacts: EnergyStudyArtifactVerificationReceiptV1,
    pub run: EnergyStudyRunVerificationReceiptV1,
}

/// Run both identity checks as one fail-closed operation.
pub fn qualify_energy_study_run_v1(
    manifest: &EnergyStudyManifestV1,
    artifacts: EnergyStudyArtifactBytesV1<'_>,
    run_receipt: &EnergyStudyRunReceiptV1,
    interval_output_bytes: &[&[u8]],
) -> Result<EnergyStudyQualificationReportV1, EnergyStudyQualificationError> {
    let artifact_receipt = verify_energy_study_artifacts_v1(manifest, artifacts)?;
    let run_verification =
        verify_energy_study_run_receipt_v1(manifest, run_receipt, interval_output_bytes)?;
    if artifact_receipt.manifest_sha256 != run_verification.manifest_sha256
        || artifact_receipt.case_sha256 != run_verification.case_sha256
    {
        return Err(EnergyStudyQualificationError::ArtifactSetMismatch);
    }
    Ok(EnergyStudyQualificationReportV1 {
        artifacts: artifact_receipt,
        run: run_verification,
    })
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(64);
    for byte in digest {
        let _ = write!(&mut output, "{byte:02x}");
    }
    output
}

fn write_text(output: &mut Vec<u8>, value: &str) {
    output.extend_from_slice(&(value.len() as u64).to_be_bytes());
    output.extend_from_slice(value.as_bytes());
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn hex_to_bytes(value: &str) -> Option<Vec<u8>> {
    if !is_sha256(value) {
        return None;
    }
    (0..value.len()).step_by(2)
        .map(|index| u8::from_str_radix(&value[index..index + 2], 16).ok())
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const STUDY_JSON: &str =
        include_str!("../../fixtures/energy-study/v1/cloud-evening-outage.json");

    fn manifest() -> EnergyStudyManifestV1 {
        serde_json::from_str(STUDY_JSON).expect("frozen study fixture must parse")
    }

    fn input_artifacts() -> EnergyStudyArtifactBytesV1<'static> {
        EnergyStudyArtifactBytesV1 {
            datasets: &[
                ("grid-availability", include_bytes!("../../fixtures/energy-study/v1/grid-availability.csv")),
                ("load-profile", include_bytes!("../../fixtures/energy-study/v1/load-profile.csv")),
                ("renewable-generation", include_bytes!("../../fixtures/energy-study/v1/renewable-generation.csv")),
            ],
            asset_registry: include_bytes!("../../fixtures/energy-study/v1/asset-registry.json"),
            constraints: include_bytes!("../../fixtures/energy-study/v1/constraints.json"),
            model_configuration: include_bytes!("../../fixtures/energy-study/v1/model-config.json"),
            policy_configuration: include_bytes!("../../fixtures/energy-study/v1/policy-config.json"),
        }
    }

    fn run_receipt(manifest: &EnergyStudyManifestV1, outputs: &[Vec<u8>]) -> EnergyStudyRunReceiptV1 {
        EnergyStudyRunReceiptV1 {
            schema_version: ENERGY_STUDY_RECEIPT_SCHEMA_VERSION,
            manifest_sha256: manifest.digest_sha256().unwrap(),
            case_sha256: manifest.case_digest_sha256().unwrap(),
            intervals: outputs.iter().enumerate().map(|(position, bytes)| {
                EnergyStudyIntervalReceiptV1 {
                    interval_index: position as u32,
                    interval_start_unix_seconds: manifest.start_unix_seconds
                        + i64::from(manifest.interval_seconds) * position as i64,
                    output_sha256: sha256_hex(bytes),
                }
            }).collect(),
        }
    }

    #[test]
    fn verifies_exact_fixture_artifact_set_and_preserves_declared_evidence_class() {
        let manifest = manifest();
        let verified = verify_energy_study_artifacts_v1(&manifest, input_artifacts()).unwrap();
        assert_eq!(verified.manifest_sha256, "896b4579a632dac32bafd3b2347f19f5da5a74f7173c14cf818b00a8e5c484da");
        assert_eq!(verified.case_sha256, "1612238432409a5282145479c7c585c53b4ae9a45832de5213722ee75d438f63");
        assert_eq!(verified.verified_dataset_count, 3);
        assert_eq!(verified.verified_configuration_count, 4);
        assert!(verified.declared_evidence_classes.iter().all(|(_, class)| *class == EnergyEvidenceClass::Scenario));
    }

    #[test]
    fn rejects_missing_extra_duplicate_and_mutated_input_artifacts() {
        let manifest = manifest();
        let all = input_artifacts();
        let missing = EnergyStudyArtifactBytesV1 { datasets: &all.datasets[..2], ..all };
        assert_eq!(verify_energy_study_artifacts_v1(&manifest, missing), Err(EnergyStudyQualificationError::ArtifactSetMismatch));

        let extra_datasets = [all.datasets[0], all.datasets[1], all.datasets[2], ("unlisted", b"extra".as_slice())];
        let extra = EnergyStudyArtifactBytesV1 { datasets: &extra_datasets, ..all };
        assert_eq!(verify_energy_study_artifacts_v1(&manifest, extra), Err(EnergyStudyQualificationError::ArtifactSetMismatch));

        let duplicate_datasets = [all.datasets[0], all.datasets[0], all.datasets[2]];
        let duplicate = EnergyStudyArtifactBytesV1 { datasets: &duplicate_datasets, ..all };
        assert_eq!(verify_energy_study_artifacts_v1(&manifest, duplicate), Err(EnergyStudyQualificationError::DuplicateArtifactId));

        let mutated_datasets = [all.datasets[0], all.datasets[1], ("renewable-generation", b"tampered".as_slice())];
        let mutated = EnergyStudyArtifactBytesV1 { datasets: &mutated_datasets, ..all };
        assert_eq!(verify_energy_study_artifacts_v1(&manifest, mutated), Err(EnergyStudyQualificationError::ArtifactHashMismatch("dataset")));
    }

    #[test]
    fn binds_ordered_timestamped_output_bytes_to_manifest() {
        let manifest = manifest();
        let outputs: Vec<Vec<u8>> = (0..manifest.interval_count)
            .map(|index| format!("fixture-output-{index}").into_bytes()).collect();
        let receipt = run_receipt(&manifest, &outputs);
        let output_slices: Vec<&[u8]> = outputs.iter().map(Vec::as_slice).collect();
        let verified = verify_energy_study_run_receipt_v1(&manifest, &receipt, &output_slices).unwrap();
        assert_eq!(verified.verified_interval_count, manifest.interval_count);
        assert_eq!(verified.manifest_sha256, manifest.digest_sha256().unwrap());
    }

    #[test]
    fn rejects_manifest_substitution_missing_rows_reordering_and_timestamp_changes() {
        let manifest = manifest();
        let outputs: Vec<Vec<u8>> = (0..manifest.interval_count)
            .map(|index| format!("fixture-output-{index}").into_bytes()).collect();
        let output_slices: Vec<&[u8]> = outputs.iter().map(Vec::as_slice).collect();
        let good = run_receipt(&manifest, &outputs);

        let mut substituted = good.clone();
        substituted.manifest_sha256 = "0".repeat(64);
        assert_eq!(verify_energy_study_run_receipt_v1(&manifest, &substituted, &output_slices), Err(EnergyStudyQualificationError::ArtifactSetMismatch));

        let missing = EnergyStudyRunReceiptV1 { intervals: good.intervals[..7].to_vec(), ..good.clone() };
        assert_eq!(verify_energy_study_run_receipt_v1(&manifest, &missing, &output_slices), Err(EnergyStudyQualificationError::IntervalCountMismatch));

        let mut reordered = good.clone();
        reordered.intervals.swap(0, 1);
        assert_eq!(verify_energy_study_run_receipt_v1(&manifest, &reordered, &output_slices), Err(EnergyStudyQualificationError::IntervalIndexMismatch { position: 0 }));

        let mut wrong_time = good;
        wrong_time.intervals[3].interval_start_unix_seconds += 1;
        assert_eq!(verify_energy_study_run_receipt_v1(&manifest, &wrong_time, &output_slices), Err(EnergyStudyQualificationError::IntervalTimestampMismatch { position: 3 }));
    }

    #[test]
    fn combined_qualification_binds_inputs_and_outputs_to_same_study() {
        let manifest = manifest();
        let outputs: Vec<Vec<u8>> = (0..manifest.interval_count)
            .map(|index| format!("fixture-output-{index}").into_bytes()).collect();
        let receipt = run_receipt(&manifest, &outputs);
        let output_slices: Vec<&[u8]> = outputs.iter().map(Vec::as_slice).collect();
        let report = qualify_energy_study_run_v1(
            &manifest,
            input_artifacts(),
            &receipt,
            &output_slices,
        ).unwrap();
        assert_eq!(report.artifacts.manifest_sha256, report.run.manifest_sha256);
        assert_eq!(report.artifacts.case_sha256, report.run.case_sha256);
        assert_eq!(report.run.verified_interval_count, manifest.interval_count);
    }

    #[test]
    fn rejects_changed_output_bytes_even_when_other_rows_are_intact() {
        let manifest = manifest();
        let outputs: Vec<Vec<u8>> = (0..manifest.interval_count)
            .map(|index| format!("fixture-output-{index}").into_bytes()).collect();
        let receipt = run_receipt(&manifest, &outputs);
        let mut changed_outputs = outputs.clone();
        changed_outputs[4] = b"tampered-output".to_vec();
        let changed_slices: Vec<&[u8]> = changed_outputs.iter().map(Vec::as_slice).collect();
        assert_eq!(verify_energy_study_run_receipt_v1(&manifest, &receipt, &changed_slices), Err(EnergyStudyQualificationError::IntervalOutputHashMismatch { position: 4 }));
    }
}
