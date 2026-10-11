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
use crate::microgrid::{BatteryConfig, MicrogridSimulation};
use crate::microgrid_verifier::{
    verify_microgrid_simulation, MicrogridVerificationError, MicrogridVerificationReport,
};
use serde::Deserialize;
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
    InvalidConstraintConfiguration,
    InvalidCriticalLoadProfile,
    CriticalLoadInputMismatch { interval: usize },
    CriticalLoadAccountingOverflow,
    InvalidAssetRegistry,
    InvalidRenewableGenerationProfile,
    InvalidGridAvailabilityProfile,
    ConstraintAssetMismatch,
    IntervalOutputSerializationFailure,
    InvalidMicrogridSimulation(MicrogridVerificationError),
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
            Self::InvalidConstraintConfiguration => {
                f.write_str("constraint configuration is malformed or outside its supported domain")
            }
            Self::InvalidCriticalLoadProfile => {
                f.write_str("critical-load profile is malformed or outside its physical domain")
            }
            Self::CriticalLoadInputMismatch { interval } => {
                write!(f, "interval {interval} load does not match the hashed load-profile artifact")
            }
            Self::CriticalLoadAccountingOverflow => {
                f.write_str("critical-load accounting produced a non-finite aggregate")
            }
            Self::InvalidAssetRegistry => {
                f.write_str("asset registry is malformed or outside its supported domain")
            }
            Self::InvalidRenewableGenerationProfile => {
                f.write_str("renewable-generation profile is malformed or exceeds the asset rating")
            }
            Self::InvalidGridAvailabilityProfile => {
                f.write_str("grid-availability profile is malformed")
            }
            Self::ConstraintAssetMismatch => {
                f.write_str("battery minimum state of charge differs between constraints and asset registry")
            }
            Self::IntervalOutputSerializationFailure => {
                f.write_str("could not canonically serialize a microgrid interval")
            }
            Self::InvalidMicrogridSimulation(error) => {
                write!(f, "independent microgrid verification failed: {error}")
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
            !artifacts.datasets.iter().any(|(actual_id, _)| *actual_id == expected.dataset_id.as_str())
        })
    {
        return Err(EnergyStudyQualificationError::ArtifactSetMismatch);
    }

    let mut entries: Vec<(String, String)> = Vec::with_capacity(manifest.datasets.len() + 4);
    for dataset in &manifest.datasets {
        let (_, bytes) = artifacts.datasets.iter()
            .find(|(dataset_id, _)| *dataset_id == dataset.dataset_id.as_str())
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

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct EnergyStudyConstraintSetV1 {
    schema_version: u16,
    critical_unserved_energy_max_kwh: f64,
    battery_minimum_soc_kwh: f64,
    allow_physical_actuation: bool,
    require_power_balance_residual_max_kwh: f64,
}

/// Conservative outcome for a critical-load constraint.
///
/// The current simulator reports aggregate unserved load, not an explicit
/// critical/non-critical allocation. The bound therefore assumes that as much
/// of the unserved energy as physically possible could be critical. Passing
/// proves the threshold from aggregate outputs; failing means the claim is not
/// proven (and may be a real violation). It never invents a critical-service
/// allocation from aggregate totals.
#[derive(Debug, Clone, PartialEq)]
pub struct CriticalLoadConstraintAssessmentV1 {
    pub critical_unserved_energy_upper_bound_kwh: f64,
    pub maximum_allowed_kwh: f64,
    pub interval_count: u32,
    pub proven_within_limit: bool,
}

/// Check the manifest-bound critical-load threshold against hashed inputs.
///
/// The exact constraints bytes and load-profile CSV bytes must match the
/// manifest before their values are used. The load-profile CSV schema is the
/// frozen V1 fixture contract: step,load_kwh,critical_load_kwh. A false
/// proven_within_limit result is a fail-closed qualification outcome, not
/// proof that a particular critical load was actually shed.
pub fn assess_critical_load_constraint_v1(
    manifest: &EnergyStudyManifestV1,
    constraints_bytes: &[u8],
    load_profile_csv_bytes: &[u8],
    simulation: &MicrogridSimulation,
) -> Result<CriticalLoadConstraintAssessmentV1, EnergyStudyQualificationError> {
    manifest.validate()?;
    if sha256_hex(constraints_bytes) != manifest.constraints_sha256 {
        return Err(EnergyStudyQualificationError::ArtifactHashMismatch("constraints"));
    }
    let load_dataset = manifest.datasets.iter()
        .find(|dataset| dataset.dataset_id == "load-profile")
        .ok_or(EnergyStudyQualificationError::ArtifactSetMismatch)?;
    if sha256_hex(load_profile_csv_bytes) != load_dataset.content_sha256 {
        return Err(EnergyStudyQualificationError::ArtifactHashMismatch("load-profile"));
    }

    let constraints: EnergyStudyConstraintSetV1 = serde_json::from_slice(constraints_bytes)
        .map_err(|_| EnergyStudyQualificationError::InvalidConstraintConfiguration)?;
    if constraints.schema_version != 1
        || !constraints.critical_unserved_energy_max_kwh.is_finite()
        || constraints.critical_unserved_energy_max_kwh < 0.0
        || !constraints.battery_minimum_soc_kwh.is_finite()
        || constraints.battery_minimum_soc_kwh < 0.0
        || !constraints.require_power_balance_residual_max_kwh.is_finite()
        || constraints.require_power_balance_residual_max_kwh < 0.0
        || constraints.allow_physical_actuation
    {
        return Err(EnergyStudyQualificationError::InvalidConstraintConfiguration);
    }

    let (loads, critical_loads) = parse_critical_load_profile_v1(load_profile_csv_bytes)?;
    if loads.len() != manifest.interval_count as usize
        || loads.len() != simulation.intervals.len()
    {
        return Err(EnergyStudyQualificationError::IntervalCountMismatch);
    }

    let mut upper_bound_kwh = 0.0;
    for (interval_index, ((expected_load, critical_load), interval)) in loads.iter()
        .zip(critical_loads.iter())
        .zip(simulation.intervals.iter())
        .enumerate()
    {
        if interval.load_kwh != *expected_load
            || !interval.unserved_load_kwh.is_finite()
            || interval.unserved_load_kwh < 0.0
            || interval.unserved_load_kwh > interval.load_kwh + 1e-9
        {
            return Err(EnergyStudyQualificationError::CriticalLoadInputMismatch {
                interval: interval_index,
            });
        }
        // Worst-case overlap of aggregate unserved energy with critical demand.
        upper_bound_kwh += (*critical_load).min(interval.unserved_load_kwh);
        if !upper_bound_kwh.is_finite() {
            return Err(EnergyStudyQualificationError::CriticalLoadAccountingOverflow);
        }
    }

    Ok(CriticalLoadConstraintAssessmentV1 {
        critical_unserved_energy_upper_bound_kwh: upper_bound_kwh,
        maximum_allowed_kwh: constraints.critical_unserved_energy_max_kwh,
        interval_count: manifest.interval_count,
        proven_within_limit: upper_bound_kwh <= constraints.critical_unserved_energy_max_kwh,
    })
}

fn parse_critical_load_profile_v1(
    bytes: &[u8],
) -> Result<(Vec<f64>, Vec<f64>), EnergyStudyQualificationError> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| EnergyStudyQualificationError::InvalidCriticalLoadProfile)?;
    let mut lines = text.lines();
    if lines.next() != Some("step,load_kwh,critical_load_kwh") {
        return Err(EnergyStudyQualificationError::InvalidCriticalLoadProfile);
    }

    let mut loads = Vec::new();
    let mut critical_loads = Vec::new();
    for line in lines {
        if line.is_empty() {
            return Err(EnergyStudyQualificationError::InvalidCriticalLoadProfile);
        }
        let fields: Vec<&str> = line.split(',').collect();
        if fields.len() != 3 {
            return Err(EnergyStudyQualificationError::InvalidCriticalLoadProfile);
        }
        let index = fields[0].parse::<usize>()
            .map_err(|_| EnergyStudyQualificationError::InvalidCriticalLoadProfile)?;
        let load = fields[1].parse::<f64>()
            .map_err(|_| EnergyStudyQualificationError::InvalidCriticalLoadProfile)?;
        let critical = fields[2].parse::<f64>()
            .map_err(|_| EnergyStudyQualificationError::InvalidCriticalLoadProfile)?;
        if index != loads.len()
            || !load.is_finite()
            || !critical.is_finite()
            || load < 0.0
            || critical < 0.0
            || critical > load
        {
            return Err(EnergyStudyQualificationError::InvalidCriticalLoadProfile);
        }
        loads.push(load);
        critical_loads.push(critical);
    }
    if loads.is_empty() {
        return Err(EnergyStudyQualificationError::InvalidCriticalLoadProfile);
    }
    Ok((loads, critical_loads))
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct EnergyStudyAssetRegistryV1 {
    schema_version: u16,
    site_id: String,
    battery: EnergyStudyBatteryV1,
    renewable_asset: EnergyStudyRenewableAssetV1,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct EnergyStudyBatteryV1 {
    capacity_kwh: f64,
    initial_soc_kwh: f64,
    minimum_soc_kwh: f64,
    max_charge_power_kw: f64,
    max_discharge_power_kw: f64,
    charge_efficiency: f64,
    discharge_efficiency: f64,
}

impl From<EnergyStudyBatteryV1> for BatteryConfig {
    fn from(value: EnergyStudyBatteryV1) -> Self {
        Self {
            capacity_kwh: value.capacity_kwh,
            initial_soc_kwh: value.initial_soc_kwh,
            minimum_soc_kwh: value.minimum_soc_kwh,
            max_charge_power_kw: value.max_charge_power_kw,
            max_discharge_power_kw: value.max_discharge_power_kw,
            charge_efficiency: value.charge_efficiency,
            discharge_efficiency: value.discharge_efficiency,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct EnergyStudyRenewableAssetV1 {
    rated_power_kw: f64,
}

/// End-to-end qualification report for this V1 single-site microgrid case.
///
/// A false qualification flag is a real failed gate in the report, not a
/// successful simulation status. Critical-load satisfaction is conservative:
/// aggregate unserved energy is insufficient to prove which loads were shed.
#[derive(Debug, Clone, PartialEq)]
pub struct EnergyStudyMicrogridQualificationReportV1 {
    pub artifacts: EnergyStudyArtifactVerificationReceiptV1,
    pub run: EnergyStudyRunVerificationReceiptV1,
    pub physical_accounting: MicrogridVerificationReport,
    pub critical_load: CriticalLoadConstraintAssessmentV1,
    pub power_balance_within_limit: bool,
    pub qualification_passed: bool,
}

/// Qualify a frozen V1 fixture using only its content-addressed artifacts.
///
/// This binds the exact CSV/JSON inputs, checks the simulation against the
/// parsed source profiles and asset limits, verifies receipt bytes against the
/// actual serialized interval rows, and applies the hashed critical-load and
/// power-balance thresholds. It is a software-model qualification, not live
/// control authorization or proof that input sources are authentic.
pub fn qualify_microgrid_study_v1(
    manifest: &EnergyStudyManifestV1,
    artifacts: EnergyStudyArtifactBytesV1<'_>,
    run_receipt: &EnergyStudyRunReceiptV1,
    simulation: &MicrogridSimulation,
) -> Result<EnergyStudyMicrogridQualificationReportV1, EnergyStudyQualificationError> {
    let artifact_receipt = verify_energy_study_artifacts_v1(manifest, artifacts)?;

    let load_bytes = dataset_bytes(artifacts.datasets, "load-profile")?;
    let generation_bytes = dataset_bytes(artifacts.datasets, "renewable-generation")?;
    let grid_bytes = dataset_bytes(artifacts.datasets, "grid-availability")?;

    let (loads, _) = parse_critical_load_profile_v1(load_bytes)?;
    let generation = parse_renewable_generation_profile_v1(generation_bytes)?;
    let grid_available = parse_grid_availability_profile_v1(grid_bytes)?;
    if generation.len() != manifest.interval_count as usize
        || grid_available.len() != manifest.interval_count as usize
    {
        return Err(EnergyStudyQualificationError::IntervalCountMismatch);
    }

    let asset_registry: EnergyStudyAssetRegistryV1 =
        serde_json::from_slice(artifacts.asset_registry)
            .map_err(|_| EnergyStudyQualificationError::InvalidAssetRegistry)?;
    if asset_registry.schema_version != 1
        || asset_registry.site_id.is_empty()
        || !asset_registry.renewable_asset.rated_power_kw.is_finite()
        || asset_registry.renewable_asset.rated_power_kw < 0.0
    {
        return Err(EnergyStudyQualificationError::InvalidAssetRegistry);
    }
    let battery: BatteryConfig = asset_registry.battery.into();

    let constraint_set: EnergyStudyConstraintSetV1 = serde_json::from_slice(artifacts.constraints)
        .map_err(|_| EnergyStudyQualificationError::InvalidConstraintConfiguration)?;
    if battery.minimum_soc_kwh != constraint_set.battery_minimum_soc_kwh {
        return Err(EnergyStudyQualificationError::ConstraintAssetMismatch);
    }

    let timestep_hours = f64::from(manifest.interval_seconds) / 3600.0;
    let rated_interval_generation = asset_registry.renewable_asset.rated_power_kw * timestep_hours;
    if generation.iter().any(|value| *value > rated_interval_generation + 1e-9) {
        return Err(EnergyStudyQualificationError::InvalidRenewableGenerationProfile);
    }

    let physical_accounting = verify_microgrid_simulation(
        &loads,
        &generation,
        &grid_available,
        timestep_hours,
        battery,
        simulation,
    )
    .map_err(EnergyStudyQualificationError::InvalidMicrogridSimulation)?;

    let serialized_intervals: Vec<Vec<u8>> = simulation.intervals.iter()
        .map(|interval| serde_json::to_vec(interval)
            .map_err(|_| EnergyStudyQualificationError::IntervalOutputSerializationFailure))
        .collect::<Result<_, _>>()?;
    let serialized_slices: Vec<&[u8]> = serialized_intervals.iter()
        .map(Vec::as_slice)
        .collect();
    let run_verification = verify_energy_study_run_receipt_v1(
        manifest,
        run_receipt,
        &serialized_slices,
    )?;

    let critical_load = assess_critical_load_constraint_v1(
        manifest,
        artifacts.constraints,
        load_bytes,
        simulation,
    )?;
    let maximum_residual = physical_accounting.max_load_balance_residual_kwh
        .max(physical_accounting.max_generation_balance_residual_kwh)
        .max(physical_accounting.max_battery_transition_residual_kwh);
    let power_balance_within_limit =
        maximum_residual <= constraint_set.require_power_balance_residual_max_kwh;
    let qualification_passed =
        power_balance_within_limit && critical_load.proven_within_limit;

    Ok(EnergyStudyMicrogridQualificationReportV1 {
        artifacts: artifact_receipt,
        run: run_verification,
        physical_accounting,
        critical_load,
        power_balance_within_limit,
        qualification_passed,
    })
}

fn dataset_bytes<'a>(
    datasets: &'a [(&'a str, &'a [u8])],
    required_id: &str,
) -> Result<&'a [u8], EnergyStudyQualificationError> {
    datasets.iter()
        .find(|(dataset_id, _)| *dataset_id == required_id)
        .map(|(_, bytes)| *bytes)
        .ok_or(EnergyStudyQualificationError::ArtifactSetMismatch)
}

fn parse_renewable_generation_profile_v1(
    bytes: &[u8],
) -> Result<Vec<f64>, EnergyStudyQualificationError> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| EnergyStudyQualificationError::InvalidRenewableGenerationProfile)?;
    let mut lines = text.lines();
    if lines.next() != Some("step,renewable_generation_kwh") {
        return Err(EnergyStudyQualificationError::InvalidRenewableGenerationProfile);
    }
    let mut generation = Vec::new();
    for line in lines {
        let fields: Vec<&str> = line.split(',').collect();
        if fields.len() != 2 {
            return Err(EnergyStudyQualificationError::InvalidRenewableGenerationProfile);
        }
        let index = fields[0].parse::<usize>()
            .map_err(|_| EnergyStudyQualificationError::InvalidRenewableGenerationProfile)?;
        let value = fields[1].parse::<f64>()
            .map_err(|_| EnergyStudyQualificationError::InvalidRenewableGenerationProfile)?;
        if index != generation.len() || !value.is_finite() || value < 0.0 {
            return Err(EnergyStudyQualificationError::InvalidRenewableGenerationProfile);
        }
        generation.push(value);
    }
    if generation.is_empty() {
        return Err(EnergyStudyQualificationError::InvalidRenewableGenerationProfile);
    }
    Ok(generation)
}

fn parse_grid_availability_profile_v1(
    bytes: &[u8],
) -> Result<Vec<bool>, EnergyStudyQualificationError> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| EnergyStudyQualificationError::InvalidGridAvailabilityProfile)?;
    let mut lines = text.lines();
    if lines.next() != Some("step,grid_available") {
        return Err(EnergyStudyQualificationError::InvalidGridAvailabilityProfile);
    }
    let mut availability = Vec::new();
    for line in lines {
        let fields: Vec<&str> = line.split(',').collect();
        if fields.len() != 2 {
            return Err(EnergyStudyQualificationError::InvalidGridAvailabilityProfile);
        }
        let index = fields[0].parse::<usize>()
            .map_err(|_| EnergyStudyQualificationError::InvalidGridAvailabilityProfile)?;
        let value = match fields[1] {
            "true" => true,
            "false" => false,
            _ => return Err(EnergyStudyQualificationError::InvalidGridAvailabilityProfile),
        };
        if index != availability.len() {
            return Err(EnergyStudyQualificationError::InvalidGridAvailabilityProfile);
        }
        availability.push(value);
    }
    if availability.is_empty() {
        return Err(EnergyStudyQualificationError::InvalidGridAvailabilityProfile);
    }
    Ok(availability)
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
        // Golden vector independently pinned by Mycelix SDK TypeScript tests.
        assert_eq!(verified.artifact_set_sha256, "74a2bb57ea351ab2f0d68502443d27a971db33293fa4c11e12a45d0afa3b30e2");
        assert_eq!(verified.verified_dataset_count, 3);

        let all = input_artifacts();
        let reversed_datasets = [all.datasets[2], all.datasets[1], all.datasets[0]];
        let reversed = EnergyStudyArtifactBytesV1 { datasets: &reversed_datasets, ..all };
        let reversed_receipt = verify_energy_study_artifacts_v1(&manifest, reversed).unwrap();
        assert_eq!(reversed_receipt.artifact_set_sha256, verified.artifact_set_sha256);
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
    fn end_to_end_fixture_is_physically_checked_and_critical_gate_is_not_hidden() {
        use crate::microgrid::simulate_microgrid;

        let manifest = manifest();
        let artifacts = input_artifacts();
        let load_bytes = include_bytes!("../../fixtures/energy-study/v1/load-profile.csv");
        let (loads, _) = parse_critical_load_profile_v1(load_bytes).unwrap();
        let generation = parse_renewable_generation_profile_v1(
            include_bytes!("../../fixtures/energy-study/v1/renewable-generation.csv")
        ).unwrap();
        let grid = parse_grid_availability_profile_v1(
            include_bytes!("../../fixtures/energy-study/v1/grid-availability.csv")
        ).unwrap();
        let battery = BatteryConfig {
            capacity_kwh: 6.0,
            initial_soc_kwh: 3.0,
            minimum_soc_kwh: 1.0,
            max_charge_power_kw: 2.0,
            max_discharge_power_kw: 2.0,
            charge_efficiency: 0.92,
            discharge_efficiency: 0.92,
        };
        let simulation = simulate_microgrid(&loads, &generation, &grid, 0.25, battery).unwrap();
        let serialized: Vec<Vec<u8>> = simulation.intervals.iter()
            .map(|interval| serde_json::to_vec(interval).unwrap()).collect();
        let mut receipt = run_receipt(&manifest, &serialized);
        receipt.intervals.iter_mut().zip(serialized.iter()).for_each(|(row, bytes)| {
            row.output_sha256 = sha256_hex(bytes);
        });

        let report = qualify_microgrid_study_v1(&manifest, artifacts, &receipt, &simulation).unwrap();
        assert_eq!(report.physical_accounting.interval_count, 8);
        assert_eq!(report.run.verified_interval_count, 8);
        assert!(report.power_balance_within_limit);
        assert!(report.critical_load.critical_unserved_energy_upper_bound_kwh > 0.0);
        assert!(!report.critical_load.proven_within_limit);
        assert!(!report.qualification_passed);
    }

    #[test]
    fn critical_load_limit_fails_closed_when_aggregate_outputs_cannot_prove_service() {
        use crate::microgrid::simulate_microgrid;

        let manifest = manifest();
        let load_bytes = include_bytes!("../../fixtures/energy-study/v1/load-profile.csv");
        let constraint_bytes = include_bytes!("../../fixtures/energy-study/v1/constraints.json");
        let (loads, _) = parse_critical_load_profile_v1(load_bytes).unwrap();
        let generation = [0.0; 8];
        let grid = [false; 8];
        let battery = BatteryConfig {
            capacity_kwh: 0.0,
            initial_soc_kwh: 0.0,
            minimum_soc_kwh: 0.0,
            max_charge_power_kw: 0.0,
            max_discharge_power_kw: 0.0,
            charge_efficiency: 1.0,
            discharge_efficiency: 1.0,
        };
        let simulation = simulate_microgrid(&loads, &generation, &grid, 0.25, battery).unwrap();
        let assessment = assess_critical_load_constraint_v1(
            &manifest,
            constraint_bytes,
            load_bytes,
            &simulation,
        ).unwrap();
        assert!(assessment.critical_unserved_energy_upper_bound_kwh > 0.0);
        assert_eq!(assessment.maximum_allowed_kwh, 0.0);
        assert!(!assessment.proven_within_limit);

        assert_eq!(
            assess_critical_load_constraint_v1(&manifest, b"tampered", load_bytes, &simulation),
            Err(EnergyStudyQualificationError::ArtifactHashMismatch("constraints"))
        );
        assert_eq!(
            assess_critical_load_constraint_v1(&manifest, constraint_bytes, b"tampered", &simulation),
            Err(EnergyStudyQualificationError::ArtifactHashMismatch("load-profile"))
        );
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
