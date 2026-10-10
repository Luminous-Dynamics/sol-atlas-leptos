// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Versioned identity and fail-closed validation for energy discovery studies.
//!
//! This module deliberately owns study metadata, not dispatch or control logic.
//! Its digest identifies the declared manifest; it does not authenticate the
//! underlying sources or prove that the declared model produced a result.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashSet;
use std::fmt::Write as _;

pub const ENERGY_STUDY_SCHEMA_VERSION: u16 = 1;
pub const MAX_ENERGY_STUDY_DATASETS: usize = 256;
const MAX_TEXT_BYTES: usize = 512;
const DIGEST_DOMAIN: &[u8] = b"luminous-dynamics.energy-study-manifest.v1\0";
const CASE_DIGEST_DOMAIN: &[u8] = b"luminous-dynamics.energy-study-case.v1\0";

/// Manifest for one fixed set of energy-study inputs and one policy comparison.
///
/// All fields participating in the digest are explicit and integer/string
/// valued. Datasets are ordered by dataset_id in the canonical digest, so
/// equivalent input orderings have the same manifest identity. Duplicate IDs
/// are rejected because they would make source binding ambiguous.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnergyStudyManifestV1 {
    pub schema_version: u16,
    /// Stable study identity, independent of one particular case.
    pub study_id: String,
    /// Stable scenario/case identifier within the study.
    pub case_id: String,
    /// Opaque geography key; avoid publishing household coordinates or meter IDs.
    pub region_id: String,
    /// UTC Unix timestamp of the first interval's start.
    pub start_unix_seconds: i64,
    /// Length of each interval in seconds; V1 uses fixed-duration intervals.
    pub interval_seconds: u32,
    /// Number of intervals. The horizon end is checked for integer overflow.
    pub interval_count: u32,
    /// Content-addressed input datasets used by this case.
    pub datasets: Vec<EnergyDatasetRefV1>,
    /// Digest of the exact physical asset/network registry used by the case.
    pub asset_registry_sha256: String,
    /// Digest of the frozen hard-constraint and metric-threshold configuration.
    pub constraints_sha256: String,
    pub model: EnergyModelRefV1,
    pub policy: EnergyPolicyRefV1,
}

/// A source-bound dataset reference. A digest binds an expected snapshot but
/// does not by itself prove the source is genuine or was read at runtime.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnergyDatasetRefV1 {
    pub dataset_id: String,
    pub source_id: String,
    pub content_sha256: String,
    /// Explicit physical/financial unit, e.g. kWh, kW, W/m2, or USD/kWh.
    pub unit: String,
    pub evidence_class: EnergyEvidenceClass,
}

/// Evidence class for a dataset, separate from the dataset's domain type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnergyEvidenceClass {
    Observed,
    Curated,
    Scenario,
    Modelled,
}

/// Exact model identity and configuration binding.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnergyModelRefV1 {
    pub model_id: String,
    pub repository: String,
    /// Exact 40-character Git SHA-1 or 64-character Git SHA-256 revision.
    pub git_revision: String,
    pub configuration_sha256: String,
}

/// Policy identity is separate from the model implementation identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnergyPolicyRefV1 {
    pub policy_id: String,
    pub repository: String,
    pub git_revision: String,
    pub configuration_sha256: String,
    pub kind: EnergyPolicyKind,
}

/// Policy category used for explicit baseline/candidate reporting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EnergyPolicyKind {
    Baseline,
    RuleBased,
    Optimization,
    Learned,
    ExternalReference,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnergyStudyValidationError {
    UnsupportedSchemaVersion,
    EmptyOrInvalidText(&'static str),
    InvalidSha256(&'static str),
    InvalidGitRevision(&'static str),
    EmptyDatasetList,
    TooManyDatasets,
    DuplicateDatasetId,
    InvalidTimeAxis,
    TimeHorizonOverflow,
    TimeOutsidePortableRange,
}

impl std::fmt::Display for EnergyStudyValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnsupportedSchemaVersion => {
                f.write_str("unsupported energy-study schema version")
            }
            Self::EmptyOrInvalidText(field) => {
                write!(f, "energy-study field {field} is empty or invalid")
            }
            Self::InvalidSha256(field) => {
                write!(f, "energy-study field {field} is not a lowercase SHA-256 digest")
            }
            Self::InvalidGitRevision(field) => {
                write!(f, "energy-study field {field} is not an exact Git revision")
            }
            Self::EmptyDatasetList => f.write_str("energy study must declare at least one dataset"),
            Self::TooManyDatasets => write!(
                f,
                "energy study exceeds the {MAX_ENERGY_STUDY_DATASETS}-dataset limit"
            ),
            Self::DuplicateDatasetId => {
                f.write_str("energy study contains a duplicate dataset_id")
            }
            Self::InvalidTimeAxis => {
                f.write_str("energy-study time interval and count must be positive")
            }
            Self::TimeHorizonOverflow => {
                f.write_str("energy-study horizon overflows the UTC Unix timestamp range")
            }
            Self::TimeOutsidePortableRange => {
                f.write_str("energy-study time values exceed the exact cross-language integer range")
            }
        }
    }
}

impl std::error::Error for EnergyStudyValidationError {}

impl EnergyStudyManifestV1 {
    /// Reject malformed, ambiguous, unpinned, or unbounded manifest metadata.
    pub fn validate(&self) -> Result<(), EnergyStudyValidationError> {
        if self.schema_version != ENERGY_STUDY_SCHEMA_VERSION {
            return Err(EnergyStudyValidationError::UnsupportedSchemaVersion);
        }

        validate_text(&self.study_id, "study_id")?;
        validate_text(&self.case_id, "case_id")?;
        validate_text(&self.region_id, "region_id")?;

        if self.interval_seconds == 0 || self.interval_count == 0 {
            return Err(EnergyStudyValidationError::InvalidTimeAxis);
        }
        // V1 is shared with JavaScript/TypeScript, so all encoded time
        // values must remain exact IEEE-754 safe integers in every runtime.
        const MAX_PORTABLE_INTEGER: i64 = 9_007_199_254_740_991;
        let horizon_seconds = u64::from(self.interval_seconds)
            .checked_mul(u64::from(self.interval_count))
            .ok_or(EnergyStudyValidationError::TimeHorizonOverflow)?;
        if horizon_seconds > MAX_PORTABLE_INTEGER as u64 {
            return Err(EnergyStudyValidationError::TimeOutsidePortableRange);
        }
        let horizon_seconds = i64::try_from(horizon_seconds)
            .map_err(|_| EnergyStudyValidationError::TimeHorizonOverflow)?;
        let horizon_end = self
            .start_unix_seconds
            .checked_add(horizon_seconds)
            .ok_or(EnergyStudyValidationError::TimeHorizonOverflow)?;
        if self.start_unix_seconds > MAX_PORTABLE_INTEGER
            || self.start_unix_seconds < -MAX_PORTABLE_INTEGER
            || horizon_end > MAX_PORTABLE_INTEGER
            || horizon_end < -MAX_PORTABLE_INTEGER
        {
            return Err(EnergyStudyValidationError::TimeOutsidePortableRange);
        }

        if self.datasets.is_empty() {
            return Err(EnergyStudyValidationError::EmptyDatasetList);
        }
        if self.datasets.len() > MAX_ENERGY_STUDY_DATASETS {
            return Err(EnergyStudyValidationError::TooManyDatasets);
        }

        let mut seen_dataset_ids = HashSet::with_capacity(self.datasets.len());
        for dataset in &self.datasets {
            validate_text(&dataset.dataset_id, "datasets.dataset_id")?;
            validate_text(&dataset.source_id, "datasets.source_id")?;
            validate_text(&dataset.unit, "datasets.unit")?;
            if !seen_dataset_ids.insert(dataset.dataset_id.as_str()) {
                return Err(EnergyStudyValidationError::DuplicateDatasetId);
            }
            validate_sha256(&dataset.content_sha256, "datasets.content_sha256")?;
        }

        validate_sha256(&self.asset_registry_sha256, "asset_registry_sha256")?;
        validate_sha256(&self.constraints_sha256, "constraints_sha256")?;

        validate_text(&self.model.model_id, "model.model_id")?;
        validate_text(&self.model.repository, "model.repository")?;
        validate_git_revision(&self.model.git_revision, "model.git_revision")?;
        validate_sha256(
            &self.model.configuration_sha256,
            "model.configuration_sha256",
        )?;

        validate_text(&self.policy.policy_id, "policy.policy_id")?;
        validate_text(&self.policy.repository, "policy.repository")?;
        validate_git_revision(&self.policy.git_revision, "policy.git_revision")?;
        validate_sha256(
            &self.policy.configuration_sha256,
            "policy.configuration_sha256",
        )?;

        Ok(())
    }

    /// Compute the policy/model-independent identity of the frozen study case.
    ///
    /// This digest binds the shared time axis, source datasets, asset registry,
    /// and hard-constraint configuration, but deliberately excludes the model
    /// and policy identities/configurations. Baseline and candidate runs over
    /// the same case therefore share this digest while retaining distinct full
    /// manifest digests. This is an input identity, not proof of source
    /// authenticity or run execution.
    pub fn case_digest_sha256(&self) -> Result<String, EnergyStudyValidationError> {
        self.validate()?;

        let mut hasher = Sha256::new();
        hasher.update(CASE_DIGEST_DOMAIN);
        hasher.update(self.schema_version.to_be_bytes());
        hash_text(&mut hasher, &self.study_id);
        hash_text(&mut hasher, &self.case_id);
        hash_text(&mut hasher, &self.region_id);
        hasher.update(self.start_unix_seconds.to_be_bytes());
        hasher.update(self.interval_seconds.to_be_bytes());
        hasher.update(self.interval_count.to_be_bytes());

        let mut datasets: Vec<&EnergyDatasetRefV1> = self.datasets.iter().collect();
        datasets.sort_by(|left, right| left.dataset_id.cmp(&right.dataset_id));
        hasher.update((datasets.len() as u32).to_be_bytes());
        for dataset in datasets {
            hash_text(&mut hasher, &dataset.dataset_id);
            hash_text(&mut hasher, &dataset.source_id);
            hash_text(&mut hasher, &dataset.content_sha256);
            hash_text(&mut hasher, &dataset.unit);
            hasher.update([dataset.evidence_class.tag()]);
        }

        hash_text(&mut hasher, &self.asset_registry_sha256);
        hash_text(&mut hasher, &self.constraints_sha256);
        Ok(to_lower_hex(&hasher.finalize()))
    }

    /// Compute a stable, domain-separated digest over the validated manifest.
    ///
    /// Encoding contract: fixed field order; unsigned integers are big-endian;
    /// strings are UTF-8 preceded by an unsigned 64-bit big-endian byte length;
    /// dataset references are sorted lexicographically by dataset_id; enums use
    /// the explicit one-byte tags in this function. Changing this format
    /// requires a new schema version and new known-answer fixtures.
    ///
    /// This digest identifies the manifest only. It is not a signature, source
    /// attestation, runtime attestation, or proof that the study actually ran.
    pub fn digest_sha256(&self) -> Result<String, EnergyStudyValidationError> {
        self.validate()?;

        let mut hasher = Sha256::new();
        hasher.update(DIGEST_DOMAIN);
        hasher.update(self.schema_version.to_be_bytes());
        hash_text(&mut hasher, &self.study_id);
        hash_text(&mut hasher, &self.case_id);
        hash_text(&mut hasher, &self.region_id);
        hasher.update(self.start_unix_seconds.to_be_bytes());
        hasher.update(self.interval_seconds.to_be_bytes());
        hasher.update(self.interval_count.to_be_bytes());

        let mut datasets: Vec<&EnergyDatasetRefV1> = self.datasets.iter().collect();
        datasets.sort_by(|left, right| left.dataset_id.cmp(&right.dataset_id));
        hasher.update((datasets.len() as u32).to_be_bytes());
        for dataset in datasets {
            hash_text(&mut hasher, &dataset.dataset_id);
            hash_text(&mut hasher, &dataset.source_id);
            hash_text(&mut hasher, &dataset.content_sha256);
            hash_text(&mut hasher, &dataset.unit);
            hasher.update([dataset.evidence_class.tag()]);
        }

        hash_text(&mut hasher, &self.asset_registry_sha256);
        hash_text(&mut hasher, &self.constraints_sha256);
        hash_text(&mut hasher, &self.model.model_id);
        hash_text(&mut hasher, &self.model.repository);
        hash_text(&mut hasher, &self.model.git_revision);
        hash_text(&mut hasher, &self.model.configuration_sha256);
        hash_text(&mut hasher, &self.policy.policy_id);
        hash_text(&mut hasher, &self.policy.repository);
        hash_text(&mut hasher, &self.policy.git_revision);
        hash_text(&mut hasher, &self.policy.configuration_sha256);
        hasher.update([self.policy.kind.tag()]);

        Ok(to_lower_hex(&hasher.finalize()))
    }
}

impl EnergyEvidenceClass {
    fn tag(self) -> u8 {
        match self {
            Self::Observed => 1,
            Self::Curated => 2,
            Self::Scenario => 3,
            Self::Modelled => 4,
        }
    }
}

impl EnergyPolicyKind {
    fn tag(self) -> u8 {
        match self {
            Self::Baseline => 1,
            Self::RuleBased => 2,
            Self::Optimization => 3,
            Self::Learned => 4,
            Self::ExternalReference => 5,
        }
    }
}

fn validate_text(
    value: &str,
    field: &'static str,
) -> Result<(), EnergyStudyValidationError> {
    if value.is_empty()
        || value.len() > MAX_TEXT_BYTES
        || value.starts_with(' ')
        || value.ends_with(' ')
        || value.chars().any(char::is_control)
    {
        return Err(EnergyStudyValidationError::EmptyOrInvalidText(field));
    }
    Ok(())
}

fn is_lower_hex(value: &str) -> bool {
    value
        .bytes()
        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn validate_sha256(
    value: &str,
    field: &'static str,
) -> Result<(), EnergyStudyValidationError> {
    if value.len() != 64 || !is_lower_hex(value) {
        return Err(EnergyStudyValidationError::InvalidSha256(field));
    }
    Ok(())
}

fn validate_git_revision(
    value: &str,
    field: &'static str,
) -> Result<(), EnergyStudyValidationError> {
    if !matches!(value.len(), 40 | 64) || !is_lower_hex(value) {
        return Err(EnergyStudyValidationError::InvalidGitRevision(field));
    }
    Ok(())
}

fn hash_text(hasher: &mut Sha256, value: &str) {
    hasher.update((value.len() as u64).to_be_bytes());
    hasher.update(value.as_bytes());
}

fn to_lower_hex(bytes: &[u8]) -> String {
    let mut result = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        // Writing to a String is infallible.
        let _ = write!(&mut result, "{byte:02x}");
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digest(byte: char) -> String {
        std::iter::repeat_n(byte, 64).collect()
    }

    fn revision(byte: char) -> String {
        std::iter::repeat_n(byte, 40).collect()
    }

    fn manifest() -> EnergyStudyManifestV1 {
        EnergyStudyManifestV1 {
            schema_version: ENERGY_STUDY_SCHEMA_VERSION,
            study_id: "community-microgrid-v1".into(),
            case_id: "cloud-evening-outage".into(),
            region_id: "region-test-01".into(),
            start_unix_seconds: 1_797_000_000,
            interval_seconds: 900,
            interval_count: 96,
            datasets: vec![
                EnergyDatasetRefV1 {
                    dataset_id: "load-profile".into(),
                    source_id: "fixture/load-v1".into(),
                    content_sha256: digest('a'),
                    unit: "kWh/interval".into(),
                    evidence_class: EnergyEvidenceClass::Scenario,
                },
                EnergyDatasetRefV1 {
                    dataset_id: "weather-profile".into(),
                    source_id: "fixture/weather-v1".into(),
                    content_sha256: digest('b'),
                    unit: "W/m2".into(),
                    evidence_class: EnergyEvidenceClass::Curated,
                },
            ],
            asset_registry_sha256: digest('c'),
            constraints_sha256: digest('d'),
            model: EnergyModelRefV1 {
                model_id: "sol-atlas-microgrid".into(),
                repository: "https://github.com/Luminous-Dynamics/sol-atlas-leptos".into(),
                git_revision: revision('1'),
                configuration_sha256: digest('e'),
            },
            policy: EnergyPolicyRefV1 {
                policy_id: "reserve-aware".into(),
                repository: "https://github.com/Luminous-Dynamics/symthaea".into(),
                git_revision: revision('2'),
                configuration_sha256: digest('f'),
                kind: EnergyPolicyKind::RuleBased,
            },
        }
    }

    #[test]
    fn valid_manifest_has_stable_digest() {
        let study = manifest();
        assert_eq!(study.validate(), Ok(()));
        let first = study.digest_sha256().unwrap();
        let second = study.digest_sha256().unwrap();
        assert_eq!(first, second);
        assert_eq!(first.len(), 64);
        assert!(is_lower_hex(&first));
        assert_eq!(
            first,
            "1aad1f5b88f9d27ef5ae84665ea629a246ae173c7b1695bf42fbca322dbcaaa6"
        );
    }

    #[test]
    fn dataset_order_does_not_change_identity() {
        let mut forward = manifest();
        let mut reversed = manifest();
        reversed.datasets.reverse();
        assert_eq!(forward.digest_sha256().unwrap(), reversed.digest_sha256().unwrap());

        // Reordering is canonicalized, but duplicate identities are not allowed.
        forward.datasets.push(forward.datasets[0].clone());
        assert_eq!(
            forward.validate(),
            Err(EnergyStudyValidationError::DuplicateDatasetId)
        );
    }

    #[test]
    fn changing_any_bound_configuration_changes_identity() {
        let baseline = manifest().digest_sha256().unwrap();
        let baseline_case = manifest().case_digest_sha256().unwrap();

        let mut changed_input = manifest();
        changed_input.datasets[0].content_sha256 = digest('9');
        assert_ne!(baseline, changed_input.digest_sha256().unwrap());
        assert_ne!(baseline_case, changed_input.case_digest_sha256().unwrap());

        let mut changed_model = manifest();
        changed_model.model.configuration_sha256 = digest('8');
        assert_ne!(baseline, changed_model.digest_sha256().unwrap());
        assert_eq!(baseline_case, changed_model.case_digest_sha256().unwrap());

        let mut changed_policy = manifest();
        changed_policy.policy.git_revision = revision('3');
        assert_ne!(baseline, changed_policy.digest_sha256().unwrap());
        assert_eq!(baseline_case, changed_policy.case_digest_sha256().unwrap());

        let mut changed_constraint = manifest();
        changed_constraint.constraints_sha256 = digest('7');
        assert_ne!(baseline, changed_constraint.digest_sha256().unwrap());
        assert_ne!(baseline_case, changed_constraint.case_digest_sha256().unwrap());
    }

    #[test]
    fn rejects_unpinned_revisions_and_malformed_hashes() {
        let mut unpinned = manifest();
        unpinned.model.git_revision = "main".into();
        assert_eq!(
            unpinned.validate(),
            Err(EnergyStudyValidationError::InvalidGitRevision("model.git_revision"))
        );

        let mut malformed = manifest();
        malformed.datasets[0].content_sha256 = "ABC".into();
        assert_eq!(
            malformed.validate(),
            Err(EnergyStudyValidationError::InvalidSha256("datasets.content_sha256"))
        );
    }

    #[test]
    fn rejects_bad_time_axis_and_overflow() {
        let mut zero = manifest();
        zero.interval_seconds = 0;
        assert_eq!(zero.validate(), Err(EnergyStudyValidationError::InvalidTimeAxis));

        let mut overflow = manifest();
        overflow.start_unix_seconds = i64::MAX - 1;
        assert_eq!(
            overflow.validate(),
            Err(EnergyStudyValidationError::TimeHorizonOverflow)
        );

        let mut non_portable = manifest();
        non_portable.start_unix_seconds = 9_007_199_254_740_991;
        assert_eq!(
            non_portable.validate(),
            Err(EnergyStudyValidationError::TimeOutsidePortableRange)
        );
    }

    #[test]
    fn public_fixture_pins_artifacts_and_cross_language_identity() {
        let study_json = include_str!("../../fixtures/energy-study/v1/cloud-evening-outage.json");
        let study: EnergyStudyManifestV1 = serde_json::from_str(study_json).unwrap();
        assert_eq!(study.validate(), Ok(()));
        assert_eq!(
            study.digest_sha256().unwrap(),
            "896b4579a632dac32bafd3b2347f19f5da5a74f7173c14cf818b00a8e5c484da"
        );
        assert_eq!(
            study.case_digest_sha256().unwrap(),
            "1612238432409a5282145479c7c585c53b4ae9a45832de5213722ee75d438f63"
        );

        let dataset_artifacts = [
            (
                "load-profile",
                include_str!("../../fixtures/energy-study/v1/load-profile.csv"),
            ),
            (
                "renewable-generation",
                include_str!("../../fixtures/energy-study/v1/renewable-generation.csv"),
            ),
            (
                "grid-availability",
                include_str!("../../fixtures/energy-study/v1/grid-availability.csv"),
            ),
        ];
        for (dataset_id, contents) in dataset_artifacts {
            let declared = study
                .datasets
                .iter()
                .find(|dataset| dataset.dataset_id == dataset_id)
                .expect("fixture must declare every referenced input artifact");
            let actual = to_lower_hex(&Sha256::digest(contents.as_bytes()));
            assert_eq!(
                declared.content_sha256, actual,
                "input artifact hash mismatch for {dataset_id}"
            );
        }

        assert_eq!(
            study.asset_registry_sha256,
            to_lower_hex(&Sha256::digest(
                include_str!("../../fixtures/energy-study/v1/asset-registry.json").as_bytes()
            ))
        );
        assert_eq!(
            study.constraints_sha256,
            to_lower_hex(&Sha256::digest(
                include_str!("../../fixtures/energy-study/v1/constraints.json").as_bytes()
            ))
        );
        assert_eq!(
            study.model.configuration_sha256,
            to_lower_hex(&Sha256::digest(
                include_str!("../../fixtures/energy-study/v1/model-config.json").as_bytes()
            ))
        );
        assert_eq!(
            study.policy.configuration_sha256,
            to_lower_hex(&Sha256::digest(
                include_str!("../../fixtures/energy-study/v1/policy-config.json").as_bytes()
            ))
        );
        assert_eq!(
            study.model.git_revision,
            "739b51d33d549bf6fd9057eb5fcbd947e76a8266"
        );
        assert_eq!(study.policy.git_revision, study.model.git_revision);
    }

    #[test]
    fn serde_rejects_unknown_fields() {
        let json = serde_json::to_string(&manifest()).unwrap();
        let injected = json.trim_end_matches('}');
        let injected = format!("{injected},\"unreviewed_authority\":true}}");
        assert!(serde_json::from_str::<EnergyStudyManifestV1>(&injected).is_err());
    }
}
