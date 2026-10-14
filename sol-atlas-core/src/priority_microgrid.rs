// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Manifest-bound critical/noncritical service accounting.
//!
//! This opt-in extension preserves the existing aggregate simulator API.
//! V1 deterministically allocates energy actually served to critical demand
//! first, then noncritical demand. These are modeled allocations, not physical
//! measurements of individual circuits or loads.

use crate::energy_study::{EnergyStudyManifestV1, EnergyStudyValidationError};
use crate::energy_study_qualification::{
    EnergyStudyArtifactBytesV1, EnergyStudyArtifactVerificationReceiptV1,
    EnergyStudyIntervalReceiptV1, EnergyStudyQualificationError, EnergyStudyRunReceiptV1,
    verify_energy_study_artifacts_v1, verify_energy_study_run_receipt_v1,
};
use crate::microgrid::{BatteryConfig, MicrogridError, MicrogridSimulation, simulate_microgrid};
use crate::microgrid_verifier::{
    MicrogridVerificationError, MicrogridVerificationReport, verify_microgrid_simulation,
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const PRIORITY_MICROGRID_SCHEMA_VERSION_V1: u16 = 1;
const SERVICE_ROWS_DOMAIN: &[u8] = b"luminous-dynamics.priority-service-rows.v1\0";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PriorityServiceClassV1 {
    Critical,
    Noncritical,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PriorityServiceAllocationV1 {
    CriticalFirst,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PriorityServicePolicyV1 {
    pub schema_version: u16,
    pub policy_id: String,
    pub allocation: PriorityServiceAllocationV1,
    pub service_order: Vec<PriorityServiceClassV1>,
}

impl PriorityServicePolicyV1 {
    fn is_supported_for(&self, manifest: &EnergyStudyManifestV1) -> bool {
        self.schema_version == PRIORITY_MICROGRID_SCHEMA_VERSION_V1
            && self.policy_id == manifest.policy.policy_id
            && self.allocation == PriorityServiceAllocationV1::CriticalFirst
            && self.service_order.len() == 2
            && self.service_order[0] == PriorityServiceClassV1::Critical
            && self.service_order[1] == PriorityServiceClassV1::Noncritical
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PriorityServiceIntervalV1 {
    pub interval_index: u32,
    pub critical_load_kwh: f64,
    pub critical_served_kwh: f64,
    pub critical_unserved_kwh: f64,
    pub noncritical_load_kwh: f64,
    pub noncritical_served_kwh: f64,
    pub noncritical_unserved_kwh: f64,
    pub service_balance_residual_kwh: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PriorityServiceTotalsV1 {
    pub critical_load_kwh: f64,
    pub critical_served_kwh: f64,
    pub critical_unserved_kwh: f64,
    pub noncritical_load_kwh: f64,
    pub noncritical_served_kwh: f64,
    pub noncritical_unserved_kwh: f64,
    pub max_service_balance_residual_kwh: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PriorityMicrogridSimulationV1 {
    pub schema_version: u16,
    pub manifest_sha256: String,
    pub case_sha256: String,
    pub artifact_set_sha256: String,
    pub policy_configuration_sha256: String,
    pub load_profile_sha256: String,
    pub aggregate: MicrogridSimulation,
    pub run_receipt: EnergyStudyRunReceiptV1,
    pub service_intervals: Vec<PriorityServiceIntervalV1>,
    pub service_totals: PriorityServiceTotalsV1,
    pub service_rows_sha256: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PriorityMicrogridQualificationReportV1 {
    pub manifest_sha256: String,
    pub case_sha256: String,
    pub artifact_set_sha256: String,
    pub interval_outputs_sha256: String,
    pub service_rows_sha256: String,
    pub interval_count: u32,
    pub physical_accounting: MicrogridVerificationReport,
    pub critical_unserved_energy_kwh: f64,
    pub critical_unserved_energy_max_kwh: f64,
    pub noncritical_unserved_energy_kwh: f64,
    pub max_service_balance_residual_kwh: f64,
    pub power_balance_within_limit: bool,
    pub critical_load_within_limit: bool,
    pub qualification_passed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PriorityMicrogridError {
    InvalidManifest(EnergyStudyValidationError),
    ArtifactVerification(EnergyStudyQualificationError),
    Microgrid(MicrogridError),
    PhysicalVerification(MicrogridVerificationError),
    InvalidPriorityPolicy,
    InvalidModelConfiguration,
    InvalidAssetRegistry,
    InvalidConstraintConfiguration,
    InvalidLoadProfile,
    InvalidRenewableGenerationProfile,
    InvalidGridAvailabilityProfile,
    InvalidCriticalLoadProfile,
    ConstraintAssetMismatch,
    RenewableRatingViolation { interval: usize },
    IntervalCountMismatch,
    IntervalInputMismatch { interval: usize },
    StudyIdentityMismatch,
    UnsupportedReceiptSchemaVersion,
    ServiceAllocationMismatch { interval: usize },
    ServiceTotalsMismatch,
    ServiceRowsDigestMismatch,
    ArithmeticOverflow,
    SerializationFailure,
}

impl std::fmt::Display for PriorityMicrogridError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidManifest(error) => write!(f, "invalid study manifest: {error}"),
            Self::ArtifactVerification(error) => write!(f, "artifact verification failed: {error}"),
            Self::Microgrid(error) => write!(f, "microgrid simulation failed: {error}"),
            Self::PhysicalVerification(error) => write!(f, "physical accounting failed: {error}"),
            Self::InvalidPriorityPolicy => {
                f.write_str("unsupported or unbound priority service policy")
            }
            Self::InvalidModelConfiguration => {
                f.write_str("model configuration does not match the manifest time axis")
            }
            Self::InvalidAssetRegistry => {
                f.write_str("asset registry is malformed or outside its supported domain")
            }
            Self::InvalidConstraintConfiguration => {
                f.write_str("constraint configuration is malformed or outside its supported domain")
            }
            Self::InvalidLoadProfile => f.write_str("load profile is malformed"),
            Self::InvalidRenewableGenerationProfile => {
                f.write_str("renewable-generation profile is malformed")
            }
            Self::InvalidGridAvailabilityProfile => {
                f.write_str("grid-availability profile is malformed")
            }
            Self::InvalidCriticalLoadProfile => {
                f.write_str("critical load must be finite and satisfy 0 <= critical <= total load")
            }
            Self::ConstraintAssetMismatch => f.write_str(
                "minimum state of charge differs between constraints and asset registry",
            ),
            Self::RenewableRatingViolation { interval } => write!(
                f,
                "renewable generation exceeds its rated interval energy at interval {interval}"
            ),
            Self::IntervalCountMismatch => {
                f.write_str("profile, simulation, receipt, and manifest interval counts must match")
            }
            Self::IntervalInputMismatch { interval } => write!(
                f,
                "aggregate interval {interval} does not match the bound load profile"
            ),
            Self::StudyIdentityMismatch => {
                f.write_str("candidate output identity differs from the verified study")
            }
            Self::UnsupportedReceiptSchemaVersion => {
                f.write_str("unsupported receipt schema version")
            }
            Self::ServiceAllocationMismatch { interval } => write!(
                f,
                "critical-first service allocation mismatch at interval {interval}"
            ),
            Self::ServiceTotalsMismatch => {
                f.write_str("reported service totals differ from independently recomputed totals")
            }
            Self::ServiceRowsDigestMismatch => {
                f.write_str("service-row digest does not match serialized service rows")
            }
            Self::ArithmeticOverflow => {
                f.write_str("service accounting produced a non-finite or overflowing result")
            }
            Self::SerializationFailure => {
                f.write_str("could not serialize interval or service rows deterministically")
            }
        }
    }
}

impl std::error::Error for PriorityMicrogridError {}

impl From<EnergyStudyValidationError> for PriorityMicrogridError {
    fn from(value: EnergyStudyValidationError) -> Self {
        Self::InvalidManifest(value)
    }
}
impl From<EnergyStudyQualificationError> for PriorityMicrogridError {
    fn from(value: EnergyStudyQualificationError) -> Self {
        Self::ArtifactVerification(value)
    }
}
impl From<MicrogridError> for PriorityMicrogridError {
    fn from(value: MicrogridError) -> Self {
        Self::Microgrid(value)
    }
}
impl From<MicrogridVerificationError> for PriorityMicrogridError {
    fn from(value: MicrogridVerificationError) -> Self {
        Self::PhysicalVerification(value)
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct AssetRegistryV1 {
    schema_version: u16,
    site_id: String,
    battery: AssetBatteryV1,
    renewable_asset: RenewableAssetV1,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct AssetBatteryV1 {
    capacity_kwh: f64,
    initial_soc_kwh: f64,
    minimum_soc_kwh: f64,
    max_charge_power_kw: f64,
    max_discharge_power_kw: f64,
    charge_efficiency: f64,
    discharge_efficiency: f64,
}
impl From<AssetBatteryV1> for BatteryConfig {
    fn from(value: AssetBatteryV1) -> Self {
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
struct RenewableAssetV1 {
    rated_power_kw: f64,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ConstraintsV1 {
    schema_version: u16,
    critical_unserved_energy_max_kwh: f64,
    battery_minimum_soc_kwh: f64,
    allow_physical_actuation: bool,
    require_power_balance_residual_max_kwh: f64,
}
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ModelConfigurationV1 {
    model: String,
    model_version: String,
    timestep_seconds: u32,
    accounting: String,
    dispatch_order: String,
    loss_model: String,
}

struct VerifiedInputsV1 {
    artifact_receipt: EnergyStudyArtifactVerificationReceiptV1,
    load_kwh: Vec<f64>,
    critical_load_kwh: Vec<f64>,
    renewable_generation_kwh: Vec<f64>,
    grid_available: Vec<bool>,
    battery: BatteryConfig,
    timestep_hours: f64,
    critical_unserved_energy_max_kwh: f64,
    max_power_balance_residual_kwh: f64,
    load_profile_sha256: String,
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut encoded = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write as _;
        let _ = write!(&mut encoded, "{byte:02x}");
    }
    encoded
}

fn near(left: f64, right: f64) -> bool {
    left.is_finite()
        && right.is_finite()
        && (left - right).abs() <= 1e-9 + 1e-9 * left.abs().max(right.abs())
}

fn dataset_bytes<'a>(
    artifacts: EnergyStudyArtifactBytesV1<'a>,
    dataset_id: &str,
) -> Result<&'a [u8], PriorityMicrogridError> {
    artifacts
        .datasets
        .iter()
        .find(|(id, _)| *id == dataset_id)
        .map(|(_, bytes)| *bytes)
        .ok_or(PriorityMicrogridError::ArtifactVerification(
            EnergyStudyQualificationError::ArtifactSetMismatch,
        ))
}

fn parse_load_profile_v1(bytes: &[u8]) -> Result<(Vec<f64>, Vec<f64>), PriorityMicrogridError> {
    let text =
        std::str::from_utf8(bytes).map_err(|_| PriorityMicrogridError::InvalidLoadProfile)?;
    let mut lines = text.lines();
    if lines.next() != Some("step,load_kwh,critical_load_kwh") {
        return Err(PriorityMicrogridError::InvalidLoadProfile);
    }
    let mut loads = Vec::new();
    let mut critical = Vec::new();
    for (index, line) in lines.enumerate() {
        let fields: Vec<&str> = line.split(',').collect();
        if fields.len() != 3 || fields[0].parse::<usize>().ok() != Some(index) {
            return Err(PriorityMicrogridError::InvalidLoadProfile);
        }
        let load = fields[1]
            .parse::<f64>()
            .map_err(|_| PriorityMicrogridError::InvalidLoadProfile)?;
        let critical_load = fields[2]
            .parse::<f64>()
            .map_err(|_| PriorityMicrogridError::InvalidCriticalLoadProfile)?;
        if !load.is_finite() || load < 0.0 {
            return Err(PriorityMicrogridError::InvalidLoadProfile);
        }
        if !critical_load.is_finite() || critical_load < 0.0 || critical_load > load {
            return Err(PriorityMicrogridError::InvalidCriticalLoadProfile);
        }
        loads.push(load);
        critical.push(critical_load);
    }
    if loads.is_empty() {
        return Err(PriorityMicrogridError::InvalidLoadProfile);
    }
    Ok((loads, critical))
}

fn parse_generation_profile_v1(bytes: &[u8]) -> Result<Vec<f64>, PriorityMicrogridError> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| PriorityMicrogridError::InvalidRenewableGenerationProfile)?;
    let mut lines = text.lines();
    if lines.next() != Some("step,renewable_generation_kwh") {
        return Err(PriorityMicrogridError::InvalidRenewableGenerationProfile);
    }
    let mut values = Vec::new();
    for (index, line) in lines.enumerate() {
        let fields: Vec<&str> = line.split(',').collect();
        if fields.len() != 2 || fields[0].parse::<usize>().ok() != Some(index) {
            return Err(PriorityMicrogridError::InvalidRenewableGenerationProfile);
        }
        let value = fields[1]
            .parse::<f64>()
            .map_err(|_| PriorityMicrogridError::InvalidRenewableGenerationProfile)?;
        if !value.is_finite() || value < 0.0 {
            return Err(PriorityMicrogridError::InvalidRenewableGenerationProfile);
        }
        values.push(value);
    }
    if values.is_empty() {
        return Err(PriorityMicrogridError::InvalidRenewableGenerationProfile);
    }
    Ok(values)
}

fn parse_grid_profile_v1(bytes: &[u8]) -> Result<Vec<bool>, PriorityMicrogridError> {
    let text = std::str::from_utf8(bytes)
        .map_err(|_| PriorityMicrogridError::InvalidGridAvailabilityProfile)?;
    let mut lines = text.lines();
    if lines.next() != Some("step,grid_available") {
        return Err(PriorityMicrogridError::InvalidGridAvailabilityProfile);
    }
    let mut values = Vec::new();
    for (index, line) in lines.enumerate() {
        let fields: Vec<&str> = line.split(',').collect();
        if fields.len() != 2 || fields[0].parse::<usize>().ok() != Some(index) {
            return Err(PriorityMicrogridError::InvalidGridAvailabilityProfile);
        }
        values.push(match fields[1] {
            "true" => true,
            "false" => false,
            _ => return Err(PriorityMicrogridError::InvalidGridAvailabilityProfile),
        });
    }
    if values.is_empty() {
        return Err(PriorityMicrogridError::InvalidGridAvailabilityProfile);
    }
    Ok(values)
}

fn load_verified_inputs_v1(
    manifest: &EnergyStudyManifestV1,
    artifacts: EnergyStudyArtifactBytesV1<'_>,
) -> Result<VerifiedInputsV1, PriorityMicrogridError> {
    let artifact_receipt = verify_energy_study_artifacts_v1(manifest, artifacts)?;
    let policy: PriorityServicePolicyV1 = serde_json::from_slice(artifacts.policy_configuration)
        .map_err(|_| PriorityMicrogridError::InvalidPriorityPolicy)?;
    if !policy.is_supported_for(manifest) {
        return Err(PriorityMicrogridError::InvalidPriorityPolicy);
    }

    let model: ModelConfigurationV1 = serde_json::from_slice(artifacts.model_configuration)
        .map_err(|_| PriorityMicrogridError::InvalidModelConfiguration)?;
    if model.model != manifest.model.model_id
        || model.model_version.is_empty()
        || model.timestep_seconds != manifest.interval_seconds
        || model.accounting != "interval_energy_kwh"
        || model.dispatch_order != "renewable_load_battery_grid_export"
        || model.loss_model != "fixed_charge_and_discharge_efficiency"
    {
        return Err(PriorityMicrogridError::InvalidModelConfiguration);
    }

    let asset: AssetRegistryV1 = serde_json::from_slice(artifacts.asset_registry)
        .map_err(|_| PriorityMicrogridError::InvalidAssetRegistry)?;
    if asset.schema_version != 1 || asset.site_id.is_empty() {
        return Err(PriorityMicrogridError::InvalidAssetRegistry);
    }
    let battery = BatteryConfig::from(asset.battery);

    let constraints: ConstraintsV1 = serde_json::from_slice(artifacts.constraints)
        .map_err(|_| PriorityMicrogridError::InvalidConstraintConfiguration)?;
    if constraints.schema_version != 1
        || !constraints.critical_unserved_energy_max_kwh.is_finite()
        || constraints.critical_unserved_energy_max_kwh < 0.0
        || !constraints.battery_minimum_soc_kwh.is_finite()
        || constraints.battery_minimum_soc_kwh < 0.0
        || !constraints
            .require_power_balance_residual_max_kwh
            .is_finite()
        || constraints.require_power_balance_residual_max_kwh < 0.0
        || constraints.allow_physical_actuation
    {
        return Err(PriorityMicrogridError::InvalidConstraintConfiguration);
    }
    if !near(constraints.battery_minimum_soc_kwh, battery.minimum_soc_kwh) {
        return Err(PriorityMicrogridError::ConstraintAssetMismatch);
    }

    let load_bytes = dataset_bytes(artifacts, "load-profile")?;
    let generation_bytes = dataset_bytes(artifacts, "renewable-generation")?;
    let grid_bytes = dataset_bytes(artifacts, "grid-availability")?;
    let (load_kwh, critical_load_kwh) = parse_load_profile_v1(load_bytes)?;
    let renewable_generation_kwh = parse_generation_profile_v1(generation_bytes)?;
    let grid_available = parse_grid_profile_v1(grid_bytes)?;
    let count = usize::try_from(manifest.interval_count)
        .map_err(|_| PriorityMicrogridError::IntervalCountMismatch)?;
    if load_kwh.len() != count
        || critical_load_kwh.len() != count
        || renewable_generation_kwh.len() != count
        || grid_available.len() != count
    {
        return Err(PriorityMicrogridError::IntervalCountMismatch);
    }
    let timestep_hours = f64::from(manifest.interval_seconds) / 3600.0;
    if !asset.renewable_asset.rated_power_kw.is_finite()
        || asset.renewable_asset.rated_power_kw < 0.0
    {
        return Err(PriorityMicrogridError::InvalidAssetRegistry);
    }
    for (interval, generation) in renewable_generation_kwh.iter().enumerate() {
        if *generation > asset.renewable_asset.rated_power_kw * timestep_hours + 1e-9 {
            return Err(PriorityMicrogridError::RenewableRatingViolation { interval });
        }
    }

    Ok(VerifiedInputsV1 {
        artifact_receipt,
        load_kwh,
        critical_load_kwh,
        renewable_generation_kwh,
        grid_available,
        battery,
        timestep_hours,
        critical_unserved_energy_max_kwh: constraints.critical_unserved_energy_max_kwh,
        max_power_balance_residual_kwh: constraints.require_power_balance_residual_max_kwh,
        load_profile_sha256: sha256_hex(load_bytes),
    })
}

fn output_receipt_v1(
    manifest: &EnergyStudyManifestV1,
    aggregate: &MicrogridSimulation,
) -> Result<EnergyStudyRunReceiptV1, PriorityMicrogridError> {
    if aggregate.intervals.len() != manifest.interval_count as usize {
        return Err(PriorityMicrogridError::IntervalCountMismatch);
    }
    let manifest_sha256 = manifest.digest_sha256()?;
    let case_sha256 = manifest.case_digest_sha256()?;
    let mut intervals = Vec::with_capacity(aggregate.intervals.len());
    for (index, interval) in aggregate.intervals.iter().enumerate() {
        let interval_index =
            u32::try_from(index).map_err(|_| PriorityMicrogridError::ArithmeticOverflow)?;
        let offset = i64::from(manifest.interval_seconds)
            .checked_mul(i64::from(interval_index))
            .ok_or(PriorityMicrogridError::ArithmeticOverflow)?;
        let timestamp = manifest
            .start_unix_seconds
            .checked_add(offset)
            .ok_or(PriorityMicrogridError::ArithmeticOverflow)?;
        let bytes = serde_json::to_vec(interval)
            .map_err(|_| PriorityMicrogridError::SerializationFailure)?;
        intervals.push(EnergyStudyIntervalReceiptV1 {
            interval_index,
            interval_start_unix_seconds: timestamp,
            output_sha256: sha256_hex(&bytes),
        });
    }
    Ok(EnergyStudyRunReceiptV1 {
        schema_version: 1,
        manifest_sha256,
        case_sha256,
        intervals,
    })
}

fn build_service_row_v1(
    index: usize,
    load_kwh: f64,
    critical_kwh: f64,
    unserved_kwh: f64,
) -> Result<PriorityServiceIntervalV1, PriorityMicrogridError> {
    if !load_kwh.is_finite()
        || load_kwh < 0.0
        || !critical_kwh.is_finite()
        || critical_kwh < 0.0
        || critical_kwh > load_kwh
        || !unserved_kwh.is_finite()
        || unserved_kwh < 0.0
        || unserved_kwh > load_kwh
    {
        return Err(PriorityMicrogridError::InvalidCriticalLoadProfile);
    }
    let served = (load_kwh - unserved_kwh).max(0.0);
    let critical_served = critical_kwh.min(served);
    let critical_unserved = critical_kwh - critical_served;
    let noncritical_load = load_kwh - critical_kwh;
    let noncritical_served = noncritical_load.min((served - critical_served).max(0.0));
    let noncritical_unserved = noncritical_load - noncritical_served;
    let residual =
        load_kwh - critical_served - critical_unserved - noncritical_served - noncritical_unserved;
    if !residual.is_finite() {
        return Err(PriorityMicrogridError::ArithmeticOverflow);
    }
    Ok(PriorityServiceIntervalV1 {
        interval_index: u32::try_from(index)
            .map_err(|_| PriorityMicrogridError::ArithmeticOverflow)?,
        critical_load_kwh: critical_kwh,
        critical_served_kwh: critical_served,
        critical_unserved_kwh: critical_unserved,
        noncritical_load_kwh: noncritical_load,
        noncritical_served_kwh: noncritical_served,
        noncritical_unserved_kwh: noncritical_unserved,
        service_balance_residual_kwh: residual,
    })
}

fn recompute_service_totals_v1(
    rows: &[PriorityServiceIntervalV1],
) -> Result<PriorityServiceTotalsV1, PriorityMicrogridError> {
    let sum = |field: fn(&PriorityServiceIntervalV1) -> f64| rows.iter().map(field).sum::<f64>();
    let values = [
        sum(|row| row.critical_load_kwh),
        sum(|row| row.critical_served_kwh),
        sum(|row| row.critical_unserved_kwh),
        sum(|row| row.noncritical_load_kwh),
        sum(|row| row.noncritical_served_kwh),
        sum(|row| row.noncritical_unserved_kwh),
    ];
    if values.iter().any(|value| !value.is_finite()) {
        return Err(PriorityMicrogridError::ArithmeticOverflow);
    }
    let max_residual = rows
        .iter()
        .map(|row| row.service_balance_residual_kwh.abs())
        .fold(0.0_f64, f64::max);
    if !max_residual.is_finite() {
        return Err(PriorityMicrogridError::ArithmeticOverflow);
    }
    Ok(PriorityServiceTotalsV1 {
        critical_load_kwh: values[0],
        critical_served_kwh: values[1],
        critical_unserved_kwh: values[2],
        noncritical_load_kwh: values[3],
        noncritical_served_kwh: values[4],
        noncritical_unserved_kwh: values[5],
        max_service_balance_residual_kwh: max_residual,
    })
}

fn service_rows_digest_v1(
    rows: &[PriorityServiceIntervalV1],
) -> Result<String, PriorityMicrogridError> {
    let count =
        u32::try_from(rows.len()).map_err(|_| PriorityMicrogridError::ArithmeticOverflow)?;
    let mut bytes = Vec::new();
    bytes.extend_from_slice(SERVICE_ROWS_DOMAIN);
    bytes.extend_from_slice(&count.to_be_bytes());
    for row in rows {
        let encoded =
            serde_json::to_vec(row).map_err(|_| PriorityMicrogridError::SerializationFailure)?;
        let length =
            u64::try_from(encoded.len()).map_err(|_| PriorityMicrogridError::ArithmeticOverflow)?;
        bytes.extend_from_slice(&length.to_be_bytes());
        bytes.extend_from_slice(&encoded);
    }
    Ok(sha256_hex(&bytes))
}

/// Execute aggregate dispatch and add the policy-defined class service rows.
/// This returns a candidate; qualification requires the independent verifier.
pub fn simulate_priority_microgrid_study_v1(
    manifest: &EnergyStudyManifestV1,
    artifacts: EnergyStudyArtifactBytesV1<'_>,
) -> Result<PriorityMicrogridSimulationV1, PriorityMicrogridError> {
    let inputs = load_verified_inputs_v1(manifest, artifacts)?;
    let aggregate = simulate_microgrid(
        &inputs.load_kwh,
        &inputs.renewable_generation_kwh,
        &inputs.grid_available,
        inputs.timestep_hours,
        inputs.battery,
    )?;
    let mut service_intervals = Vec::with_capacity(aggregate.intervals.len());
    for (index, (interval, critical_load)) in aggregate
        .intervals
        .iter()
        .zip(inputs.critical_load_kwh.iter())
        .enumerate()
    {
        if !near(interval.load_kwh, inputs.load_kwh[index]) {
            return Err(PriorityMicrogridError::IntervalInputMismatch { interval: index });
        }
        service_intervals.push(build_service_row_v1(
            index,
            interval.load_kwh,
            *critical_load,
            interval.unserved_load_kwh,
        )?);
    }
    let service_totals = recompute_service_totals_v1(&service_intervals)?;
    let service_rows_sha256 = service_rows_digest_v1(&service_intervals)?;
    let run_receipt = output_receipt_v1(manifest, &aggregate)?;
    Ok(PriorityMicrogridSimulationV1 {
        schema_version: PRIORITY_MICROGRID_SCHEMA_VERSION_V1,
        manifest_sha256: inputs.artifact_receipt.manifest_sha256.clone(),
        case_sha256: inputs.artifact_receipt.case_sha256.clone(),
        artifact_set_sha256: inputs.artifact_receipt.artifact_set_sha256.clone(),
        policy_configuration_sha256: sha256_hex(artifacts.policy_configuration),
        load_profile_sha256: inputs.load_profile_sha256,
        aggregate,
        run_receipt,
        service_intervals,
        service_totals,
        service_rows_sha256,
    })
}

/// Recheck source-flow physics and independently recompute class service from
/// the hashed load profile. A valid accounting report may still fail a hard
/// critical-load limit; such a failure is returned as a non-passing report.
pub fn verify_priority_microgrid_study_v1(
    manifest: &EnergyStudyManifestV1,
    artifacts: EnergyStudyArtifactBytesV1<'_>,
    candidate: &PriorityMicrogridSimulationV1,
) -> Result<PriorityMicrogridQualificationReportV1, PriorityMicrogridError> {
    let inputs = load_verified_inputs_v1(manifest, artifacts)?;
    if candidate.schema_version != PRIORITY_MICROGRID_SCHEMA_VERSION_V1 {
        return Err(PriorityMicrogridError::UnsupportedReceiptSchemaVersion);
    }
    if candidate.manifest_sha256 != inputs.artifact_receipt.manifest_sha256
        || candidate.case_sha256 != inputs.artifact_receipt.case_sha256
        || candidate.artifact_set_sha256 != inputs.artifact_receipt.artifact_set_sha256
        || candidate.policy_configuration_sha256 != sha256_hex(artifacts.policy_configuration)
        || candidate.load_profile_sha256 != inputs.load_profile_sha256
    {
        return Err(PriorityMicrogridError::StudyIdentityMismatch);
    }
    if candidate.aggregate.intervals.len() != manifest.interval_count as usize
        || candidate.service_intervals.len() != manifest.interval_count as usize
    {
        return Err(PriorityMicrogridError::IntervalCountMismatch);
    }
    let interval_bytes: Vec<Vec<u8>> = candidate
        .aggregate
        .intervals
        .iter()
        .map(|interval| {
            serde_json::to_vec(interval).map_err(|_| PriorityMicrogridError::SerializationFailure)
        })
        .collect::<Result<_, _>>()?;
    let interval_slices: Vec<&[u8]> = interval_bytes.iter().map(Vec::as_slice).collect();
    let run_verification =
        verify_energy_study_run_receipt_v1(manifest, &candidate.run_receipt, &interval_slices)?;
    let physical = verify_microgrid_simulation(
        &inputs.load_kwh,
        &inputs.renewable_generation_kwh,
        &inputs.grid_available,
        inputs.timestep_hours,
        inputs.battery,
        &candidate.aggregate,
    )?;

    let mut expected_rows = Vec::with_capacity(candidate.aggregate.intervals.len());
    for (index, interval) in candidate.aggregate.intervals.iter().enumerate() {
        let load = inputs.load_kwh[index];
        let critical = inputs.critical_load_kwh[index];
        if !near(interval.load_kwh, load) {
            return Err(PriorityMicrogridError::IntervalInputMismatch { interval: index });
        }
        if !interval.unserved_load_kwh.is_finite()
            || interval.unserved_load_kwh < 0.0
            || interval.unserved_load_kwh > load
        {
            return Err(PriorityMicrogridError::ServiceAllocationMismatch { interval: index });
        }

        // Independent formula: critical service gets min(critical demand,
        // total served); any remainder can serve only noncritical demand.
        let served = load - interval.unserved_load_kwh;
        let expected_critical_served = if served < critical {
            served.max(0.0)
        } else {
            critical
        };
        let expected_critical_unserved = critical - expected_critical_served;
        let noncritical_load = load - critical;
        let noncritical_served = (served - expected_critical_served)
            .max(0.0)
            .min(noncritical_load);
        let noncritical_unserved = noncritical_load - noncritical_served;
        let residual = load
            - expected_critical_served
            - expected_critical_unserved
            - noncritical_served
            - noncritical_unserved;
        let expected = PriorityServiceIntervalV1 {
            interval_index: u32::try_from(index)
                .map_err(|_| PriorityMicrogridError::ArithmeticOverflow)?,
            critical_load_kwh: critical,
            critical_served_kwh: expected_critical_served,
            critical_unserved_kwh: expected_critical_unserved,
            noncritical_load_kwh: noncritical_load,
            noncritical_served_kwh: noncritical_served,
            noncritical_unserved_kwh: noncritical_unserved,
            service_balance_residual_kwh: residual,
        };
        let actual = &candidate.service_intervals[index];
        if actual.interval_index != expected.interval_index
            || !near(actual.critical_load_kwh, expected.critical_load_kwh)
            || !near(actual.critical_served_kwh, expected.critical_served_kwh)
            || !near(actual.critical_unserved_kwh, expected.critical_unserved_kwh)
            || !near(actual.noncritical_load_kwh, expected.noncritical_load_kwh)
            || !near(
                actual.noncritical_served_kwh,
                expected.noncritical_served_kwh,
            )
            || !near(
                actual.noncritical_unserved_kwh,
                expected.noncritical_unserved_kwh,
            )
            || !near(
                actual.service_balance_residual_kwh,
                expected.service_balance_residual_kwh,
            )
        {
            return Err(PriorityMicrogridError::ServiceAllocationMismatch { interval: index });
        }
        expected_rows.push(expected);
    }
    let expected_totals = recompute_service_totals_v1(&expected_rows)?;
    if !near(
        candidate.service_totals.critical_load_kwh,
        expected_totals.critical_load_kwh,
    ) || !near(
        candidate.service_totals.critical_served_kwh,
        expected_totals.critical_served_kwh,
    ) || !near(
        candidate.service_totals.critical_unserved_kwh,
        expected_totals.critical_unserved_kwh,
    ) || !near(
        candidate.service_totals.noncritical_load_kwh,
        expected_totals.noncritical_load_kwh,
    ) || !near(
        candidate.service_totals.noncritical_served_kwh,
        expected_totals.noncritical_served_kwh,
    ) || !near(
        candidate.service_totals.noncritical_unserved_kwh,
        expected_totals.noncritical_unserved_kwh,
    ) || !near(
        candidate.service_totals.max_service_balance_residual_kwh,
        expected_totals.max_service_balance_residual_kwh,
    ) {
        return Err(PriorityMicrogridError::ServiceTotalsMismatch);
    }
    let service_rows_sha256 = service_rows_digest_v1(&candidate.service_intervals)?;
    if service_rows_sha256 != candidate.service_rows_sha256 {
        return Err(PriorityMicrogridError::ServiceRowsDigestMismatch);
    }

    let physical_residual = physical
        .max_load_balance_residual_kwh
        .max(physical.max_generation_balance_residual_kwh);
    let power_balance_within_limit = physical_residual <= inputs.max_power_balance_residual_kwh;
    let critical_load_within_limit =
        expected_totals.critical_unserved_kwh <= inputs.critical_unserved_energy_max_kwh;
    Ok(PriorityMicrogridQualificationReportV1 {
        manifest_sha256: inputs.artifact_receipt.manifest_sha256,
        case_sha256: inputs.artifact_receipt.case_sha256,
        artifact_set_sha256: inputs.artifact_receipt.artifact_set_sha256,
        interval_outputs_sha256: run_verification.interval_outputs_sha256,
        service_rows_sha256,
        interval_count: manifest.interval_count,
        physical_accounting: physical,
        critical_unserved_energy_kwh: expected_totals.critical_unserved_kwh,
        critical_unserved_energy_max_kwh: inputs.critical_unserved_energy_max_kwh,
        noncritical_unserved_energy_kwh: expected_totals.noncritical_unserved_kwh,
        max_service_balance_residual_kwh: expected_totals.max_service_balance_residual_kwh,
        power_balance_within_limit,
        critical_load_within_limit,
        qualification_passed: power_balance_within_limit
            && critical_load_within_limit
            && expected_totals.max_service_balance_residual_kwh <= 1e-9,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::energy_study::EnergyStudyManifestV1;

    const LOAD: &[u8] = include_bytes!("../../fixtures/energy-study/v1/load-profile.csv");
    const GENERATION: &[u8] =
        include_bytes!("../../fixtures/energy-study/v1/renewable-generation.csv");
    const GRID: &[u8] = include_bytes!("../../fixtures/energy-study/v1/grid-availability.csv");
    const ASSET: &[u8] = include_bytes!("../../fixtures/energy-study/v1/asset-registry.json");
    const CONSTRAINTS: &[u8] = include_bytes!("../../fixtures/energy-study/v1/constraints.json");
    const MODEL: &[u8] = include_bytes!("../../fixtures/energy-study/v1/model-config.json");
    const POLICY: &[u8] =
        include_bytes!("../../fixtures/energy-study/v1/critical-first-policy.json");
    const DATASETS: [(&str, &[u8]); 3] = [
        ("load-profile", LOAD),
        ("renewable-generation", GENERATION),
        ("grid-availability", GRID),
    ];

    fn manifest() -> EnergyStudyManifestV1 {
        serde_json::from_str(include_str!(
            "../../fixtures/energy-study/v1/critical-first-study.json"
        ))
        .unwrap()
    }
    fn artifacts() -> EnergyStudyArtifactBytesV1<'static> {
        EnergyStudyArtifactBytesV1 {
            datasets: &DATASETS,
            asset_registry: ASSET,
            constraints: CONSTRAINTS,
            model_configuration: MODEL,
            policy_configuration: POLICY,
        }
    }

    #[test]
    fn outage_fixture_reports_critical_shortfall_and_does_not_pass() {
        let study = manifest();
        let candidate = simulate_priority_microgrid_study_v1(&study, artifacts()).unwrap();
        let report = verify_priority_microgrid_study_v1(&study, artifacts(), &candidate).unwrap();
        // Independent Python mirror over the frozen fixture produced these
        // scenario-specific expected totals. Critical service is a modeled
        // critical-first allocation, not measurement from physical circuits.
        assert_eq!(
            candidate.manifest_sha256,
            "497e9e832e7aea859196cc668ff34df4761b222db247340f7facd27552121fc8"
        );
        assert_eq!(
            candidate.case_sha256,
            "6b29792b59d9e0add9f85a8468416013448700f5a8f87d8d5db2408fd47f1768"
        );
        assert_eq!(
            report.artifact_set_sha256,
            "3028003c72470c54f7f5a84fe5e7fe9fbf2f74bd8a028c98c91b74f541c29fbd"
        );
        assert_eq!(candidate.aggregate.totals.unserved_load_kwh, 9.5);
        assert_eq!(report.critical_unserved_energy_kwh, 2.5);
        assert_eq!(report.noncritical_unserved_energy_kwh, 7.0);
        assert_eq!(candidate.service_totals.critical_unserved_kwh, 2.5);
        assert_eq!(candidate.service_totals.noncritical_unserved_kwh, 7.0);
        assert_eq!(report.critical_unserved_energy_max_kwh, 0.0);
        assert!(!report.critical_load_within_limit);
        assert!(!report.qualification_passed);
        assert_eq!(candidate.service_intervals.len(), 8);
        assert!(candidate.aggregate.intervals[4].grid_import_kwh.abs() <= 1e-9);
        assert!(candidate.aggregate.intervals[5].unserved_load_kwh > 0.0);
    }

    #[test]
    fn aggregate_preserving_priority_mutation_is_rejected() {
        let study = manifest();
        let mut candidate = simulate_priority_microgrid_study_v1(&study, artifacts()).unwrap();
        let row = &mut candidate.service_intervals[4];
        assert!(row.noncritical_unserved_kwh >= 0.25);
        row.critical_unserved_kwh += 0.25;
        row.noncritical_unserved_kwh -= 0.25;
        candidate.service_rows_sha256 =
            service_rows_digest_v1(&candidate.service_intervals).unwrap();
        assert_eq!(
            verify_priority_microgrid_study_v1(&study, artifacts(), &candidate),
            Err(PriorityMicrogridError::ServiceAllocationMismatch { interval: 4 })
        );
    }

    #[test]
    fn forged_totals_and_missing_service_rows_are_rejected() {
        let study = manifest();
        let mut forged = simulate_priority_microgrid_study_v1(&study, artifacts()).unwrap();
        forged.service_totals.critical_unserved_kwh += 0.25;
        assert_eq!(
            verify_priority_microgrid_study_v1(&study, artifacts(), &forged),
            Err(PriorityMicrogridError::ServiceTotalsMismatch)
        );

        let mut missing = simulate_priority_microgrid_study_v1(&study, artifacts()).unwrap();
        missing.service_intervals.pop();
        missing.service_rows_sha256 = service_rows_digest_v1(&missing.service_intervals).unwrap();
        assert_eq!(
            verify_priority_microgrid_study_v1(&study, artifacts(), &missing),
            Err(PriorityMicrogridError::IntervalCountMismatch)
        );
    }

    #[test]
    fn priority_order_cannot_change_without_changing_bound_artifact_digest() {
        let study = manifest();
        let changed_policy = br#"{"schema_version":1,"policy_id":"critical-first-service-v1","allocation":"critical_first","service_order":["noncritical","critical"]}"#;
        let original = artifacts();
        let changed = EnergyStudyArtifactBytesV1 {
            policy_configuration: changed_policy,
            ..original
        };
        assert!(matches!(
            simulate_priority_microgrid_study_v1(&study, changed),
            Err(PriorityMicrogridError::ArtifactVerification(
                EnergyStudyQualificationError::ArtifactHashMismatch("policy_configuration")
            ))
        ));
    }

    #[test]
    fn parser_accepts_zero_critical_demand_but_rejects_critical_above_total() {
        let (load, critical) =
            parse_load_profile_v1(b"step,load_kwh,critical_load_kwh\n0,1.0,0.0\n").unwrap();
        assert_eq!(load, vec![1.0]);
        assert_eq!(critical, vec![0.0]);
        assert_eq!(
            parse_load_profile_v1(b"step,load_kwh,critical_load_kwh\n0,1.0,1.1\n"),
            Err(PriorityMicrogridError::InvalidCriticalLoadProfile)
        );
    }

    #[test]
    fn allocation_covers_complete_service_and_zero_critical_load() {
        let row = build_service_row_v1(0, 2.0, 1.0, 0.0).unwrap();
        assert_eq!(row.critical_served_kwh, 1.0);
        assert_eq!(row.critical_unserved_kwh, 0.0);
        assert_eq!(row.noncritical_served_kwh, 1.0);
        assert_eq!(row.noncritical_unserved_kwh, 0.0);

        let zero_critical = build_service_row_v1(0, 2.0, 0.0, 0.5).unwrap();
        assert_eq!(zero_critical.critical_unserved_kwh, 0.0);
        assert_eq!(zero_critical.noncritical_unserved_kwh, 0.5);
    }

    #[test]
    fn service_rows_conserve_served_and_unserved_energy_over_small_domain() {
        // Exhaustively cover tenths of a kWh for load, critical demand, and
        // total unserved energy. This is deterministic and dependency-free.
        for load_tenths in 0..=20 {
            let load = f64::from(load_tenths) / 10.0;
            for critical_tenths in 0..=load_tenths {
                let critical = f64::from(critical_tenths) / 10.0;
                for unserved_tenths in 0..=load_tenths {
                    let unserved = f64::from(unserved_tenths) / 10.0;
                    let row = build_service_row_v1(0, load, critical, unserved).unwrap();

                    let class_served = row.critical_served_kwh + row.noncritical_served_kwh;
                    let class_unserved = row.critical_unserved_kwh + row.noncritical_unserved_kwh;
                    assert!((class_served - (load - unserved)).abs() <= 1e-9);
                    assert!((class_unserved - unserved).abs() <= 1e-9);
                    assert!(
                        (row.critical_load_kwh + row.noncritical_load_kwh - load).abs() <= 1e-9
                    );
                    assert!(row.critical_served_kwh >= 0.0);
                    assert!(row.noncritical_served_kwh >= 0.0);
                    assert!(row.critical_unserved_kwh >= 0.0);
                    assert!(row.noncritical_unserved_kwh >= 0.0);
                }
            }
        }
    }

    #[test]
    fn rejects_unserved_energy_above_total_load_even_inside_other_tolerances() {
        // A tolerance-based acceptance here would force service rows to count
        // total unserved energy as no greater than load, silently losing the
        // excess while preserving a zero service residual.
        assert_eq!(
            build_service_row_v1(0, 1.0, 0.5, 1.0 + 5e-10),
            Err(PriorityMicrogridError::InvalidCriticalLoadProfile)
        );
    }
}
