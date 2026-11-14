// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Satellite Earth-observation evidence claims.
//!
//! This module deliberately sits above the canonical geospatial asset model.
//! It references AssetId and source/processing receipts; it does not introduce
//! another asset catalog, fetch imagery, establish data rights, or infer
//! scientific truth. RightsReceiptRef binds an external decision receipt but
//! does not interpret or authenticate it; Mycelix / the custodian owns that
//! authority check. Cross-record validation is performed when constructing or
//! deserializing an EvidenceBundle.
//!
//! A missing, obscured, invalid, below-limit, out-of-domain, conflicted, or
//! rights-unresolved observation is never equivalent to "no change".

use crate::earth_observability::{
    classify_non_detection, DetectionQualification, EvidenceState, NonDetectionDisposition,
};
use crate::geospatial_assets::{AssetId, ObservationTime, ObservationTimeKind};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::convert::TryFrom;

pub const EARTH_OBSERVATION_SCHEMA_VERSION: u32 = 1;

/// The EO sensor family. Provider-specific sensor and product identifiers
/// remain attached to the referenced asset/processing receipts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObservationModality {
    Optical,
    SyntheticApertureRadar,
    ThermalInfrared,
    Hyperspectral,
    Multispectral,
    Other,
}

/// Declares how the source-qualified sensor identity was obtained.
/// This is provenance about identity resolution, not an independent sensor registry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SensorIdentityOrigin {
    ProviderDeclared,
    DerivedFromPlatformAndInstrument,
    Unknown,
}

/// Quality/observability state for the declared spatial and temporal support.
/// This is not a source-rights or authorization state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObservationQuality {
    QualifiedUsable,
    CloudContaminated,
    ShadowContaminated,
    HazeOrSmokeContaminated,
    Saturated,
    RadiometricInvalid,
    GeometricInvalid,
    IncompleteCoverage,
    BelowDetectionLimit,
    OutOfDomain,
    Conflicted,
}

impl ObservationQuality {
    /// Project the EO-specific sensor-quality result into the shared evidence
    /// state vocabulary. This is a one-way classification, not a second
    /// independent status field that can contradict the quality flags.
    pub fn evidence_state(self) -> EvidenceState {
        match self {
            Self::QualifiedUsable => EvidenceState::Present,
            Self::CloudContaminated
            | Self::ShadowContaminated
            | Self::HazeOrSmokeContaminated
            | Self::IncompleteCoverage
            | Self::BelowDetectionLimit => EvidenceState::Partial,
            Self::Saturated | Self::RadiometricInvalid | Self::GeometricInvalid => {
                EvidenceState::Invalid
            }
            Self::OutOfDomain => EvidenceState::OutOfDomain,
            Self::Conflicted => EvidenceState::Conflicted,
        }
    }
}

/// Reference to an external purpose-specific rights decision.
/// This is a locator/binding only: Sol-Atlas does not interpret the decision
/// or authenticate its issuer. Mycelix / the authoritative custodian owns that
/// decision and must verify it before any use, export or action.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "RightsReceiptRefWire")]
pub struct RightsReceiptRef {
    subject_id: String,
    purpose_id: String,
    source_id: String,
    asset_id: String,
    decision_receipt_sha256: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RightsReceiptRefWire {
    subject_id: String,
    purpose_id: String,
    source_id: String,
    asset_id: String,
    decision_receipt_sha256: String,
}

impl RightsReceiptRef {
    pub fn new(
        subject_id: impl Into<String>,
        purpose_id: impl Into<String>,
        source_id: impl Into<String>,
        asset_id: impl Into<String>,
        decision_receipt_sha256: impl Into<String>,
    ) -> Result<Self, String> {
        let subject_id = subject_id.into();
        let purpose_id = purpose_id.into();
        let source_id = source_id.into();
        let asset_id = asset_id.into();
        validate_identifier(&subject_id, "rights subject ID")?;
        validate_identifier(&purpose_id, "rights purpose ID")?;
        validate_identifier(&source_id, "rights source ID")?;
        validate_identifier(&asset_id, "rights asset ID")?;
        let decision_receipt_sha256 = normalize_sha256(
            decision_receipt_sha256.into(),
            "rights decision receipt",
        )?;
        Ok(Self {
            subject_id,
            purpose_id,
            source_id,
            asset_id,
            decision_receipt_sha256,
        })
    }

    pub fn subject_id(&self) -> &str {
        &self.subject_id
    }

    pub fn purpose_id(&self) -> &str {
        &self.purpose_id
    }

    pub fn source_id(&self) -> &str {
        &self.source_id
    }

    pub fn asset_id(&self) -> &str {
        &self.asset_id
    }

    pub fn decision_receipt_sha256(&self) -> &str {
        &self.decision_receipt_sha256
    }
}

impl TryFrom<RightsReceiptRefWire> for RightsReceiptRef {
    type Error = String;

    fn try_from(wire: RightsReceiptRefWire) -> Result<Self, Self::Error> {
        Self::new(
            wire.subject_id,
            wire.purpose_id,
            wire.source_id,
            wire.asset_id,
            wire.decision_receipt_sha256,
        )
    }
}

/// A single observation of a declared support. The source asset and receipts
/// are references to canonical Sol-Atlas / external records, not copied claims
/// about source authenticity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "ObservationRecordWire")]
pub struct ObservationRecord {
    observation_id: String,
    asset_id: AssetId,
    /// Stable source identity reference in the canonical Sol-Atlas / custodian source registry.
    source_id: String,
    /// Provider-native platform/satellite identifier.
    platform_id: String,
    /// Source-registry-qualified sensor identity, preserving its provider namespace.
    sensor_id: String,
    /// Whether sensor_id was provider-declared, derived from source fields, or unresolved.
    sensor_identity_origin: SensorIdentityOrigin,
    /// Provider-native instrument identifier where distinct from sensor ID.
    instrument_id: Option<String>,
    /// Provider-native STAC collection or product-family identifier.
    collection_id: Option<String>,
    /// Provider-native processing baseline/version; absent remains unknown.
    processing_baseline: Option<String>,
    /// Digest of the exact asset-level band/polarization/measurement profile.
    /// Missing means the source measurement configuration is not fully identified.
    measurement_profile_sha256: Option<String>,
    source_asset_receipt_sha256: String,
    processing_receipt_sha256: String,
    rights: RightsReceiptRef,
    support_id: String,
    acquisition: ObservationTime,
    modality: ObservationModality,
    quality: ObservationQuality,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ObservationRecordWire {
    observation_id: String,
    asset_id: AssetId,
    /// Stable source identity reference in the canonical Sol-Atlas / custodian source registry.
    source_id: String,
    platform_id: String,
    sensor_id: String,
    sensor_identity_origin: SensorIdentityOrigin,
    instrument_id: Option<String>,
    collection_id: Option<String>,
    processing_baseline: Option<String>,
    #[serde(default)]
    measurement_profile_sha256: Option<String>,
    source_asset_receipt_sha256: String,
    processing_receipt_sha256: String,
    rights: RightsReceiptRef,
    support_id: String,
    acquisition: ObservationTime,
    modality: ObservationModality,
    quality: ObservationQuality,
}

/// Structural reference to an independently qualified cross-system comparison.
/// The receipt digest identifies the external qualification payload; the copied
/// scope fields are checked against the local claim before the reference is accepted.
/// Hash presence alone does not authenticate or qualify the external receipt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "CrossSystemComparabilityReceiptRefWire")]
pub struct CrossSystemComparabilityReceiptRef {
    receipt_sha256: String,
    observation_ids: Vec<String>,
    system_signatures: Vec<ObservationSystemSignature>,
    support_id: String,
    target_class_id: String,
    ontology_version: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CrossSystemComparabilityReceiptRefWire {
    receipt_sha256: String,
    observation_ids: Vec<String>,
    system_signatures: Vec<ObservationSystemSignature>,
    support_id: String,
    target_class_id: String,
    ontology_version: String,
}

impl CrossSystemComparabilityReceiptRef {
    pub fn new(
        receipt_sha256: impl Into<String>,
        observation_ids: Vec<String>,
        system_signatures: Vec<ObservationSystemSignature>,
        support_id: impl Into<String>,
        target_class_id: impl Into<String>,
        ontology_version: impl Into<String>,
    ) -> Result<Self, String> {
        let receipt_sha256 = normalize_sha256(receipt_sha256.into(), "cross-system comparability receipt")?;
        let mut observation_ids = observation_ids;
        validate_unique_ids(&observation_ids, "comparability observation reference")?;
        if observation_ids.is_empty() {
            return Err("cross-system comparability receipt requires at least one observation ID".into());
        }
        for id in &observation_ids {
            validate_identifier(id, "comparability observation reference")?;
        }
        observation_ids.sort();
        let mut system_signatures = system_signatures;
        system_signatures.sort();
        let unique_system_signatures = system_signatures.iter().cloned().collect::<BTreeSet<_>>();
        if system_signatures.is_empty() || unique_system_signatures.len() != system_signatures.len() {
            return Err("comparability receipt requires a non-empty, duplicate-free system-signature set".into());
        }
        let support_id = support_id.into();
        let target_class_id = target_class_id.into();
        let ontology_version = ontology_version.into();
        validate_identifier(&support_id, "comparability support ID")?;
        validate_identifier(&target_class_id, "comparability target class ID")?;
        validate_identifier(&ontology_version, "comparability ontology version")?;
        Ok(Self {
            receipt_sha256,
            observation_ids,
            system_signatures,
            support_id,
            target_class_id,
            ontology_version,
        })
    }

    pub fn receipt_sha256(&self) -> &str { &self.receipt_sha256 }
    pub fn observation_ids(&self) -> &[String] { &self.observation_ids }
    pub fn system_signatures(&self) -> &[ObservationSystemSignature] { &self.system_signatures }
    pub fn support_id(&self) -> &str { &self.support_id }
    pub fn target_class_id(&self) -> &str { &self.target_class_id }
    pub fn ontology_version(&self) -> &str { &self.ontology_version }
}

impl TryFrom<CrossSystemComparabilityReceiptRefWire> for CrossSystemComparabilityReceiptRef {
    type Error = String;

    fn try_from(wire: CrossSystemComparabilityReceiptRefWire) -> Result<Self, Self::Error> {
        Self::new(
            wire.receipt_sha256,
            wire.observation_ids,
            wire.system_signatures,
            wire.support_id,
            wire.target_class_id,
            wire.ontology_version,
        )
    }
}

/// Deterministic signature of the measurement system that produced an observation.
/// This is a comparison key, not a sensor registry or proof of cross-system equivalence.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ObservationSystemSignature {
    source_id: String,
    platform_id: String,
    sensor_id: String,
    sensor_identity_origin: SensorIdentityOrigin,
    instrument_id: Option<String>,
    modality: ObservationModality,
    collection_id: Option<String>,
    processing_baseline: Option<String>,
    measurement_profile_sha256: Option<String>,
}

impl ObservationSystemSignature {
    pub fn source_id(&self) -> &str { &self.source_id }
    pub fn platform_id(&self) -> &str { &self.platform_id }
    pub fn sensor_id(&self) -> &str { &self.sensor_id }
    pub fn sensor_identity_origin(&self) -> SensorIdentityOrigin { self.sensor_identity_origin }
    pub fn instrument_id(&self) -> Option<&str> { self.instrument_id.as_deref() }
    pub fn modality(&self) -> ObservationModality { self.modality }
    pub fn collection_id(&self) -> Option<&str> { self.collection_id.as_deref() }
    pub fn processing_baseline(&self) -> Option<&str> { self.processing_baseline.as_deref() }
    pub fn measurement_profile_sha256(&self) -> Option<&str> { self.measurement_profile_sha256.as_deref() }
}

impl ObservationRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        observation_id: impl Into<String>,
        asset_id: AssetId,
        source_id: impl Into<String>,
        platform_id: impl Into<String>,
        sensor_id: impl Into<String>,
        sensor_identity_origin: SensorIdentityOrigin,
        instrument_id: Option<String>,
        collection_id: Option<String>,
        processing_baseline: Option<String>,
        source_asset_receipt_sha256: impl Into<String>,
        processing_receipt_sha256: impl Into<String>,
        rights: RightsReceiptRef,
        support_id: impl Into<String>,
        acquisition: ObservationTime,
        modality: ObservationModality,
        quality: ObservationQuality,
    ) -> Result<Self, String> {
        let observation_id = observation_id.into();
        let source_id = source_id.into();
        let platform_id = platform_id.into();
        let sensor_id = sensor_id.into();
        let support_id = support_id.into();
        validate_identifier(&observation_id, "observation ID")?;
        validate_identifier(&source_id, "source ID")?;
        validate_identifier(&platform_id, "platform ID")?;
        validate_identifier(&sensor_id, "sensor ID")?;
        validate_identifier(&support_id, "support ID")?;
        validate_optional_identifier(instrument_id.as_deref(), "instrument ID")?;
        match sensor_identity_origin {
            SensorIdentityOrigin::ProviderDeclared => {
                if sensor_id == "unknown" || sensor_id.starts_with("derived:") {
                    return Err("provider-declared sensor ID cannot use unknown/derived sentinel".into());
                }
            }
            SensorIdentityOrigin::DerivedFromPlatformAndInstrument => {
                if instrument_id.is_none() || !sensor_id.starts_with("derived:") {
                    return Err("derived sensor identity requires an instrument and derived: ID".into());
                }
            }
            SensorIdentityOrigin::Unknown => {
                if sensor_id != "unknown" {
                    return Err("unknown sensor identity must use the explicit unknown sentinel".into());
                }
            }
        }
        validate_optional_identifier(collection_id.as_deref(), "collection ID")?;
        validate_optional_identifier(processing_baseline.as_deref(), "processing baseline")?;
        if acquisition.kind() == ObservationTimeKind::Unknown
            && quality == ObservationQuality::QualifiedUsable
        {
            return Err("qualified usable observation requires declared acquisition time".into());
        }
        if rights.asset_id() != asset_id.as_str() {
            return Err("rights decision receipt must bind the exact observation asset ID".into());
        }
        if rights.source_id() != source_id {
            return Err("rights decision receipt must bind the exact observation source ID".into());
        }
        Ok(Self {
            observation_id,
            asset_id,
            source_id,
            platform_id,
            sensor_id,
            sensor_identity_origin,
            instrument_id,
            collection_id,
            processing_baseline,
            measurement_profile_sha256: None,
            source_asset_receipt_sha256: normalize_sha256(
                source_asset_receipt_sha256.into(),
                "source asset receipt",
            )?,
            processing_receipt_sha256: normalize_sha256(
                processing_receipt_sha256.into(),
                "processing receipt",
            )?,
            rights,
            support_id,
            acquisition,
            modality,
            quality,
        })
    }

    pub fn observation_id(&self) -> &str {
        &self.observation_id
    }

    pub fn asset_id(&self) -> &AssetId {
        &self.asset_id
    }

    /// Canonical source registry identity referenced by this observation.
    pub fn source_id(&self) -> &str {
        &self.source_id
    }

    pub fn platform_id(&self) -> &str {
        &self.platform_id
    }

    pub fn sensor_id(&self) -> &str {
        &self.sensor_id
    }

    pub fn sensor_identity_origin(&self) -> SensorIdentityOrigin {
        self.sensor_identity_origin
    }

    pub fn instrument_id(&self) -> Option<&str> {
        self.instrument_id.as_deref()
    }

    pub fn collection_id(&self) -> Option<&str> {
        self.collection_id.as_deref()
    }

    pub fn processing_baseline(&self) -> Option<&str> {
        self.processing_baseline.as_deref()
    }

    pub fn measurement_profile_sha256(&self) -> Option<&str> {
        self.measurement_profile_sha256.as_deref()
    }

    /// Bind an externally computed, canonical fingerprint of this exact asset's
    /// band/polarization/measurement configuration. The caller must compute it
    /// from the frozen source profile; Sol-Atlas does not guess missing bands.
    pub fn with_measurement_profile_sha256(
        mut self,
        value: Option<String>,
    ) -> Result<Self, String> {
        self.measurement_profile_sha256 = value
            .map(|digest| normalize_sha256(digest, "measurement profile"))
            .transpose()?;
        Ok(self)
    }

    pub fn system_signature(&self) -> ObservationSystemSignature {
        ObservationSystemSignature {
            source_id: self.source_id.clone(),
            platform_id: self.platform_id.clone(),
            sensor_id: self.sensor_id.clone(),
            sensor_identity_origin: self.sensor_identity_origin,
            instrument_id: self.instrument_id.clone(),
            modality: self.modality,
            collection_id: self.collection_id.clone(),
            processing_baseline: self.processing_baseline.clone(),
            measurement_profile_sha256: self.measurement_profile_sha256.clone(),
        }
    }

    pub fn source_asset_receipt_sha256(&self) -> &str {
        &self.source_asset_receipt_sha256
    }

    pub fn processing_receipt_sha256(&self) -> &str {
        &self.processing_receipt_sha256
    }

    pub fn rights(&self) -> &RightsReceiptRef {
        &self.rights
    }

    pub fn support_id(&self) -> &str {
        &self.support_id
    }

    pub fn acquisition(&self) -> &ObservationTime {
        &self.acquisition
    }

    pub fn modality(&self) -> ObservationModality {
        self.modality
    }

    pub fn quality(&self) -> ObservationQuality {
        self.quality
    }

    /// EO-quality check only. Rights decisions are external references and
    /// must be authenticated by the Mycelix / custodian verification boundary.
    pub fn is_usable_observation(&self) -> bool {
        self.quality == ObservationQuality::QualifiedUsable
    }

    /// True only when an actual source-qualified sensor identity is available.
    /// Unknown sensor identity can be retained in the evidence bundle, but
    /// cannot support a promoted detection/change claim.
    pub fn has_qualified_source_identity(&self) -> bool {
        self.sensor_identity_origin != SensorIdentityOrigin::Unknown
    }

    fn acquisition_signature(&self) -> Option<String> {
        if self.acquisition.kind() == ObservationTimeKind::Unknown {
            return None;
        }
        Some(format!(
            "{:?}|{}|{}",
            self.acquisition.kind(),
            self.acquisition.datetime().unwrap_or(""),
            self.acquisition.start().unwrap_or("")
        ) + &format!("|{}", self.acquisition.end().unwrap_or("")))
    }
}

impl TryFrom<ObservationRecordWire> for ObservationRecord {
    type Error = String;

    fn try_from(wire: ObservationRecordWire) -> Result<Self, Self::Error> {
        Self::new(
            wire.observation_id,
            wire.asset_id,
            wire.source_id,
            wire.platform_id,
            wire.sensor_id,
            wire.sensor_identity_origin,
            wire.instrument_id,
            wire.collection_id,
            wire.processing_baseline,
            wire.source_asset_receipt_sha256,
            wire.processing_receipt_sha256,
            wire.rights,
            wire.support_id,
            wire.acquisition,
            wire.modality,
            wire.quality,
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoverageState {
    CompleteQualified,
    Partial,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "DetectionLimitWire")]
pub struct DetectionLimit {
    metric: String,
    value: f64,
    unit: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct DetectionLimitWire {
    metric: String,
    value: f64,
    unit: String,
}

impl DetectionLimit {
    pub fn new(
        metric: impl Into<String>,
        value: f64,
        unit: impl Into<String>,
    ) -> Result<Self, String> {
        let metric = metric.into();
        let unit = unit.into();
        validate_identifier(&metric, "detection-limit metric")?;
        validate_identifier(&unit, "detection-limit unit")?;
        if !value.is_finite() || value <= 0.0 {
            return Err("detection limit must be finite and greater than zero".into());
        }
        Ok(Self { metric, value, unit })
    }

    pub fn metric(&self) -> &str {
        &self.metric
    }

    pub fn value(&self) -> f64 {
        self.value
    }

    pub fn unit(&self) -> &str {
        &self.unit
    }
}

impl TryFrom<DetectionLimitWire> for DetectionLimit {
    type Error = String;

    fn try_from(wire: DetectionLimitWire) -> Result<Self, Self::Error> {
        Self::new(wire.metric, wire.value, wire.unit)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ChangeDisposition {
    CandidateChange,
    SupportedChange,
    NoChangeWithinEnvelope,
    Unresolved,
}

/// A declared time window for a change claim. Values are retained as supplied;
/// this type does not authenticate timestamps or infer their ordering.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "TimeWindowWire")]
pub struct TimeWindow {
    start: String,
    end: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TimeWindowWire {
    start: String,
    end: String,
}

impl TimeWindow {
    pub fn new(start: impl Into<String>, end: impl Into<String>) -> Result<Self, String> {
        let start = start.into();
        let end = end.into();
        if start.trim().is_empty()
            || end.trim().is_empty()
            || start.trim() != start
            || end.trim() != end
        {
            return Err("time-window bounds must be non-empty and trimmed".into());
        }
        Ok(Self { start, end })
    }

    pub fn start(&self) -> &str {
        &self.start
    }

    pub fn end(&self) -> &str {
        &self.end
    }
}

impl TryFrom<TimeWindowWire> for TimeWindow {
    type Error = String;

    fn try_from(wire: TimeWindowWire) -> Result<Self, Self::Error> {
        Self::new(wire.start, wire.end)
    }
}

/// Evidence claim over one declared spatial support and time window.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "ChangeClaimWire")]
pub struct ChangeClaim {
    claim_id: String,
    target_class_id: String,
    ontology_version: String,
    support_id: String,
    observation_window: TimeWindow,
    disposition: ChangeDisposition,
    event_time_bounds: Option<TimeWindow>,
    observation_ids: Vec<String>,
    supporting_observation_ids: Vec<String>,
    contradicting_observation_ids: Vec<String>,
    coverage: CoverageState,
    coverage_receipt_sha256: Option<String>,
    detection_qualification_receipt_sha256: Option<String>,
    /// Required when one claim combines distinct source/platform/sensor/product/processing systems.
    cross_system_comparability_receipt: Option<CrossSystemComparabilityReceiptRef>,
    detection_qualification: Option<DetectionQualification>,
    detection_limit: Option<DetectionLimit>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ChangeClaimWire {
    claim_id: String,
    target_class_id: String,
    ontology_version: String,
    support_id: String,
    observation_window: TimeWindow,
    disposition: ChangeDisposition,
    event_time_bounds: Option<TimeWindow>,
    observation_ids: Vec<String>,
    supporting_observation_ids: Vec<String>,
    contradicting_observation_ids: Vec<String>,
    coverage: CoverageState,
    coverage_receipt_sha256: Option<String>,
    detection_qualification_receipt_sha256: Option<String>,
    cross_system_comparability_receipt: Option<CrossSystemComparabilityReceiptRef>,
    detection_qualification: Option<DetectionQualification>,
    detection_limit: Option<DetectionLimit>,
}

impl ChangeClaim {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        claim_id: impl Into<String>,
        target_class_id: impl Into<String>,
        ontology_version: impl Into<String>,
        support_id: impl Into<String>,
        observation_window: TimeWindow,
        disposition: ChangeDisposition,
        event_time_bounds: Option<TimeWindow>,
        observation_ids: Vec<String>,
        supporting_observation_ids: Vec<String>,
        contradicting_observation_ids: Vec<String>,
        coverage: CoverageState,
        coverage_receipt_sha256: Option<String>,
        detection_qualification_receipt_sha256: Option<String>,
        cross_system_comparability_receipt: Option<CrossSystemComparabilityReceiptRef>,
        detection_qualification: Option<DetectionQualification>,
        detection_limit: Option<DetectionLimit>,
    ) -> Result<Self, String> {
        let claim_id = claim_id.into();
        let target_class_id = target_class_id.into();
        let ontology_version = ontology_version.into();
        let support_id = support_id.into();
        validate_identifier(&claim_id, "claim ID")?;
        validate_identifier(&target_class_id, "target class ID")?;
        validate_identifier(&ontology_version, "ontology version")?;
        validate_identifier(&support_id, "claim support ID")?;
        validate_unique_ids(&observation_ids, "observation reference")?;
        validate_unique_ids(&supporting_observation_ids, "supporting observation reference")?;
        validate_unique_ids(&contradicting_observation_ids, "contradicting observation reference")?;

        for id in supporting_observation_ids
            .iter()
            .chain(contradicting_observation_ids.iter())
        {
            if !observation_ids.contains(id) {
                return Err("supporting/contradicting observation must be in observation_ids".into());
            }
        }
        for id in &observation_ids {
            validate_identifier(id, "observation reference")?;
        }

        let coverage_receipt_sha256 = coverage_receipt_sha256
            .map(|value| normalize_sha256(value, "coverage receipt"))
            .transpose()?;
        let detection_qualification_receipt_sha256 = detection_qualification_receipt_sha256
            .map(|value| normalize_sha256(value, "detection qualification receipt"))
            .transpose()?;


        if coverage == CoverageState::CompleteQualified && coverage_receipt_sha256.is_none() {
            return Err("complete qualified coverage requires a coverage receipt digest".into());
        }
        if coverage != CoverageState::CompleteQualified && coverage_receipt_sha256.is_some() {
            return Err("coverage receipt is only admissible for complete qualified coverage".into());
        }

        match disposition {
            ChangeDisposition::NoChangeWithinEnvelope => {
                if coverage != CoverageState::CompleteQualified {
                    return Err("no-change claim requires complete qualified observation coverage".into());
                }
                if detection_limit.is_none() {
                    return Err("no-change claim requires an explicit detection limit".into());
                }
                if detection_qualification_receipt_sha256.is_none() {
                    return Err("no-change claim requires an exact target/sensor detection-qualification receipt".into());
                }
                let qualification = detection_qualification.ok_or_else(|| {
                    "no-change claim requires the structured detection-qualification result".to_string()
                })?;
                if classify_non_detection(qualification)
                    != NonDetectionDisposition::AbsenceEvidenceEligible
                {
                    return Err("no-change claim requires target coverage, valid calibration, in-domain applicability and sufficient qualified probability of detection".into());
                }
                if supporting_observation_ids.len() < 2 {
                    return Err("no-change claim requires at least two supporting observations".into());
                }
                if !contradicting_observation_ids.is_empty() {
                    return Err("no-change claim cannot contain contradicting observations".into());
                }
                if event_time_bounds.is_some() {
                    return Err("no-change claim uses an observation window, not an event-time interval".into());
                }
            }
            ChangeDisposition::SupportedChange => {
                if supporting_observation_ids.is_empty() {
                    return Err("supported-change claim requires supporting observations".into());
                }
                if event_time_bounds.is_none() {
                    return Err("supported-change claim requires bounded event-time evidence".into());
                }
            }
            ChangeDisposition::CandidateChange => {
                if supporting_observation_ids.is_empty() {
                    return Err("candidate-change claim requires at least one supporting observation".into());
                }
            }
            ChangeDisposition::Unresolved => {}
        }

        Ok(Self {
            claim_id,
            target_class_id,
            ontology_version,
            support_id,
            observation_window,
            disposition,
            event_time_bounds,
            observation_ids,
            supporting_observation_ids,
            contradicting_observation_ids,
            coverage,
            coverage_receipt_sha256,
            detection_qualification_receipt_sha256,
            cross_system_comparability_receipt,
            detection_qualification,
            detection_limit,
        })
    }

    pub fn claim_id(&self) -> &str {
        &self.claim_id
    }

    pub fn disposition(&self) -> ChangeDisposition {
        self.disposition
    }

    pub fn observation_window(&self) -> &TimeWindow {
        &self.observation_window
    }

    pub fn observation_ids(&self) -> &[String] {
        &self.observation_ids
    }

    pub fn cross_system_comparability_receipt(&self) -> Option<&CrossSystemComparabilityReceiptRef> {
        self.cross_system_comparability_receipt.as_ref()
    }

    /// Validate references and evidence admissibility against the exact
    /// observation records in the bundle.
    pub fn validate_against(&self, observations: &[ObservationRecord]) -> Result<(), String> {
        let mut by_id = BTreeMap::new();
        for observation in observations {
            if by_id.insert(observation.observation_id(), observation).is_some() {
                return Err("duplicate observation ID in evidence bundle".into());
            }
        }

        let mut referenced = Vec::with_capacity(self.observation_ids.len());
        for id in &self.observation_ids {
            let observation = by_id
                .get(id.as_str())
                .ok_or_else(|| format!("missing referenced observation: {id}"))?;
            if observation.support_id() != self.support_id {
                return Err(format!("observation {id} has a different spatial support"));
            }
            referenced.push(*observation);
        }

        let system_signatures = referenced
            .iter()
            .map(|observation| observation.system_signature())
            .collect::<BTreeSet<_>>();
        let incomplete_measurement_identity = referenced.iter().any(|observation| {
            observation.collection_id.is_none()
                || observation.processing_baseline.is_none()
                || observation.measurement_profile_sha256.is_none()
        });
        match (&self.cross_system_comparability_receipt, system_signatures.len(), incomplete_measurement_identity) {
            (None, count, false) if count <= 1 => {}
            (None, _, _) => {
                return Err("claim combines distinct or incompletely identified measurement systems without a cross-system comparability receipt".into());
            }
            (Some(_), count, false) if count <= 1 => {
                return Err("cross-system comparability receipt is present but the claim references one fully identified observation-system signature".into());
            }
            (Some(receipt), _, _) => {
                let claim_observation_ids = self.observation_ids.iter().cloned().collect::<BTreeSet<_>>();
                let receipt_observation_ids = receipt.observation_ids.iter().cloned().collect::<BTreeSet<_>>();
                if receipt_observation_ids != claim_observation_ids {
                    return Err("cross-system comparability receipt observation set does not exactly match the claim".into());
                }
                let receipt_signatures = receipt.system_signatures.iter().cloned().collect::<BTreeSet<_>>();
                if receipt_signatures != system_signatures || receipt_signatures.len() != receipt.system_signatures.len() {
                    return Err("cross-system comparability receipt measurement-system signatures do not exactly match current observations".into());
                }
                if receipt.support_id != self.support_id
                    || receipt.target_class_id != self.target_class_id
                    || receipt.ontology_version != self.ontology_version
                {
                    return Err("cross-system comparability receipt scope does not match claim support/class/ontology".into());
                }
            }
            _ => {}
        }

        for id in self
            .supporting_observation_ids
            .iter()
            .chain(self.contradicting_observation_ids.iter())
        {
            let observation = by_id
                .get(id.as_str())
                .ok_or_else(|| format!("missing referenced observation: {id}"))?;
            if !observation.is_usable_observation() {
                return Err(format!("observation {id} is not EO-qualified usable"));
            }
            if !observation.has_qualified_source_identity() {
                return Err(format!("observation {id} has unknown source sensor identity"));
            }
        }

        if self.disposition == ChangeDisposition::NoChangeWithinEnvelope {
            if referenced
                .iter()
                .any(|observation| {
                    !observation.is_usable_observation()
                        || !observation.has_qualified_source_identity()
                })
            {
                return Err("no-change claim includes an EO-unusable or source-identity-unresolved observation".into());
            }
            let unique_acquisitions = self
                .supporting_observation_ids
                .iter()
                .filter_map(|id| by_id.get(id.as_str()).and_then(|o| o.acquisition_signature()))
                .collect::<BTreeSet<_>>();
            if unique_acquisitions.len() < 2 {
                return Err("no-change claim requires two distinct declared acquisition times".into());
            }
        }
        Ok(())
    }
}

impl TryFrom<ChangeClaimWire> for ChangeClaim {
    type Error = String;

    fn try_from(wire: ChangeClaimWire) -> Result<Self, Self::Error> {
        Self::new(
            wire.claim_id,
            wire.target_class_id,
            wire.ontology_version,
            wire.support_id,
            wire.observation_window,
            wire.disposition,
            wire.event_time_bounds,
            wire.observation_ids,
            wire.supporting_observation_ids,
            wire.contradicting_observation_ids,
            wire.coverage,
            wire.coverage_receipt_sha256,
            wire.detection_qualification_receipt_sha256,
            wire.cross_system_comparability_receipt,
            wire.detection_qualification,
            wire.detection_limit,
        )
    }
}

/// Cross-record validated evidence package. Its deserialization path validates
/// claim references and the no-change negative-control invariants.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "EvidenceBundleWire")]
pub struct EvidenceBundle {
    schema_version: u32,
    observations: Vec<ObservationRecord>,
    claims: Vec<ChangeClaim>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct EvidenceBundleWire {
    schema_version: u32,
    observations: Vec<ObservationRecord>,
    claims: Vec<ChangeClaim>,
}

impl EvidenceBundle {
    pub fn new(
        observations: Vec<ObservationRecord>,
        claims: Vec<ChangeClaim>,
    ) -> Result<Self, String> {
        let mut observation_ids = BTreeSet::new();
        let rights_scope = observations.first().map(|observation| {
            (
                observation.rights().subject_id().to_string(),
                observation.rights().purpose_id().to_string(),
            )
        });
        for observation in &observations {
            if !observation_ids.insert(observation.observation_id().to_string()) {
                return Err("duplicate observation ID in evidence bundle".into());
            }
            if observation.rights().asset_id() != observation.asset_id().as_str() {
                return Err("rights decision receipt asset identity mismatch".into());
            }
            if let Some((subject_id, purpose_id)) = &rights_scope {
                if observation.rights().subject_id() != subject_id.as_str()
                    || observation.rights().purpose_id() != purpose_id.as_str()
                {
                    return Err("evidence bundle mixes rights subjects or purposes".into());
                }
            }
        }

        let mut claim_ids = BTreeSet::new();
        for claim in &claims {
            if !claim_ids.insert(claim.claim_id().to_string()) {
                return Err("duplicate claim ID in evidence bundle".into());
            }
            claim.validate_against(&observations)?;
        }

        Ok(Self {
            schema_version: EARTH_OBSERVATION_SCHEMA_VERSION,
            observations,
            claims,
        })
    }

    pub fn schema_version(&self) -> u32 {
        self.schema_version
    }

    pub fn observations(&self) -> &[ObservationRecord] {
        &self.observations
    }

    pub fn claims(&self) -> &[ChangeClaim] {
        &self.claims
    }
}

impl TryFrom<EvidenceBundleWire> for EvidenceBundle {
    type Error = String;

    fn try_from(wire: EvidenceBundleWire) -> Result<Self, Self::Error> {
        if wire.schema_version != EARTH_OBSERVATION_SCHEMA_VERSION {
            return Err(format!(
                "unsupported Earth-observation schema version {}; expected {}",
                wire.schema_version, EARTH_OBSERVATION_SCHEMA_VERSION
            ));
        }
        Self::new(wire.observations, wire.claims)
    }
}

fn validate_identifier(value: &str, label: &str) -> Result<(), String> {
    if value.is_empty()
        || value.trim() != value
        || value.len() > 128
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b':'))
    {
        return Err(format!("{label} must be 1-128 trimmed ASCII token characters"));
    }
    Ok(())
}

fn validate_unique_ids(values: &[String], label: &str) -> Result<(), String> {
    let mut seen = BTreeSet::new();
    for value in values {
        validate_identifier(value, label)?;
        if !seen.insert(value) {
            return Err(format!("duplicate {label}: {value}"));
        }
    }
    Ok(())
}

fn normalize_sha256(value: String, label: &str) -> Result<String, String> {
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(format!("{label} must be a 64-character SHA-256 hex digest"));
    }
    Ok(value.to_ascii_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digest(byte: u8) -> String {
        std::iter::repeat_n(byte as char, 64).collect()
    }

    fn rights(asset_id: &str) -> RightsReceiptRef {
        RightsReceiptRef::new(
            "fixture-subject",
            "internal-evaluation",
            "fixture-source",
            asset_id,
            digest(b'a'),
        )
        .unwrap()
    }

    fn observation(id: &str, date: &str, quality: ObservationQuality) -> ObservationRecord {
        ObservationRecord::new(
            id,
            AssetId::new(format!("fixture:{id}")).unwrap(),
            "fixture-source",
            "fixture-platform",
            "fixture-sensor",
            SensorIdentityOrigin::ProviderDeclared,
            Some("fixture-instrument".into()),
            Some("fixture-collection".into()),
            Some("fixture-baseline".into()),
            digest(b'b'),
            digest(b'c'),
            rights(&format!("fixture:{id}")),
            "support-a",
            ObservationTime::new(
                ObservationTimeKind::Instant,
                Some(date.to_string()),
                None,
            )
            .unwrap(),
            ObservationModality::Optical,
            quality,
        )
        .unwrap()
        .with_measurement_profile_sha256(Some(digest(b'm')))
        .unwrap()
    }


    fn no_change_claim() -> ChangeClaim {
        ChangeClaim::new(
            "claim-a",
            "vegetation-loss",
            "fixture-ontology-v1",
            "support-a",
            TimeWindow::new("2024-01-01", "2025-01-01").unwrap(),
            ChangeDisposition::NoChangeWithinEnvelope,
            None,
            vec!["obs-a".into(), "obs-b".into()],
            vec!["obs-a".into(), "obs-b".into()],
            vec![],
            CoverageState::CompleteQualified,
            Some(digest(b'd')),
            Some(digest(b'e')),
            None,
            Some(DetectionQualification {
                footprint_covers_target: true,
                calibration_valid: true,
                target_in_validated_domain: true,
                probability_of_detection: Some(0.99),
                minimum_required_probability: 0.95,
            }),
            Some(DetectionLimit::new("minimum-change", 0.05, "fraction").unwrap()),
        )
        .unwrap()
    }

    #[test]
    fn no_change_requires_complete_coverage_receipt_and_detection_limit() {
        assert!(ChangeClaim::new(
            "claim", "class", "ontology-v1", "support",
            TimeWindow::new("2024-01-01", "2025-01-01").unwrap(),
            ChangeDisposition::NoChangeWithinEnvelope, None,
            vec!["a".into(), "b".into()], vec!["a".into(), "b".into()], vec![],
            CoverageState::Partial, None, Some(digest(b'e')), None, None,
            Some(DetectionLimit::new("min-change", 1.0, "unit").unwrap())
        ).is_err());
    }

    #[test]
    fn no_change_requires_target_sensor_detection_qualification_receipt() {
        assert!(ChangeClaim::new(
            "claim", "class", "ontology-v1", "support-a",
            TimeWindow::new("2024-01-01", "2025-01-01").unwrap(),
            ChangeDisposition::NoChangeWithinEnvelope, None,
            vec!["obs-a".into(), "obs-b".into()],
            vec!["obs-a".into(), "obs-b".into()], vec![],
            CoverageState::CompleteQualified,
            Some(digest(b'd')),
            None,
            None,
            None,
            Some(DetectionLimit::new("min-change", 0.05, "fraction").unwrap())
        ).is_err());
    }

    #[test]
    fn no_change_requires_absence_eligible_shared_detection_qualification() {
        for qualification in [
            DetectionQualification {
                footprint_covers_target: false,
                calibration_valid: true,
                target_in_validated_domain: true,
                probability_of_detection: Some(0.99),
                minimum_required_probability: 0.95,
            },
            DetectionQualification {
                footprint_covers_target: true,
                calibration_valid: false,
                target_in_validated_domain: true,
                probability_of_detection: Some(0.99),
                minimum_required_probability: 0.95,
            },
            DetectionQualification {
                footprint_covers_target: true,
                calibration_valid: true,
                target_in_validated_domain: false,
                probability_of_detection: Some(0.99),
                minimum_required_probability: 0.95,
            },
            DetectionQualification {
                footprint_covers_target: true,
                calibration_valid: true,
                target_in_validated_domain: true,
                probability_of_detection: Some(0.70),
                minimum_required_probability: 0.95,
            },
        ] {
            let result = ChangeClaim::new(
                "claim", "class", "ontology-v1", "support-a",
                TimeWindow::new("2024-01-01", "2025-01-01").unwrap(),
                ChangeDisposition::NoChangeWithinEnvelope, None,
                vec!["obs-a".into(), "obs-b".into()],
                vec!["obs-a".into(), "obs-b".into()], vec![],
                CoverageState::CompleteQualified,
                Some(digest(b'd')),
                Some(digest(b'e')),
                None,
                Some(qualification),
                Some(DetectionLimit::new("minimum-change", 0.05, "fraction").unwrap()),
            );
            assert!(result.is_err(), "unqualified POD result was accepted as no-change");
        }
    }

    #[test]
    fn eo_quality_maps_into_shared_evidence_state_without_parallel_state_field() {
        assert_eq!(
            ObservationQuality::QualifiedUsable.evidence_state(),
            EvidenceState::Present
        );
        assert_eq!(
            ObservationQuality::CloudContaminated.evidence_state(),
            EvidenceState::Partial
        );
        assert_eq!(
            ObservationQuality::RadiometricInvalid.evidence_state(),
            EvidenceState::Invalid
        );
        assert_eq!(
            ObservationQuality::OutOfDomain.evidence_state(),
            EvidenceState::OutOfDomain
        );
        assert_eq!(
            ObservationQuality::Conflicted.evidence_state(),
            EvidenceState::Conflicted
        );
    }

    #[test]
    fn non_usable_eo_quality_cannot_be_no_change() {
        for quality in [
            ObservationQuality::CloudContaminated,
            ObservationQuality::ShadowContaminated,
            ObservationQuality::HazeOrSmokeContaminated,
            ObservationQuality::Saturated,
            ObservationQuality::RadiometricInvalid,
            ObservationQuality::GeometricInvalid,
            ObservationQuality::IncompleteCoverage,
            ObservationQuality::BelowDetectionLimit,
            ObservationQuality::OutOfDomain,
            ObservationQuality::Conflicted,
        ] {
            let bundle = EvidenceBundle::new(
                vec![
                    observation("obs-a", "2024-01-01", quality),
                    observation("obs-b", "2025-01-01", ObservationQuality::QualifiedUsable),
                ],
                vec![no_change_claim()],
            );
            assert!(bundle.is_err(), "quality {quality:?} was promoted to no-change");
        }
    }

    #[test]
    fn comparability_receipt_rejects_bad_digest_duplicate_ids_and_sorts_ids() {
        assert!(CrossSystemComparabilityReceiptRef::new(
            "not-a-sha256",
            vec!["obs-a".into()],
            "support-a",
            "vegetation-loss",
            "ontology-v1",
        ).is_err());
        assert!(CrossSystemComparabilityReceiptRef::new(
            digest(b'r'),
            vec!["obs-a".into(), "obs-a".into()],
            "support-a",
            "vegetation-loss",
            "ontology-v1",
        ).is_err());
        let receipt = CrossSystemComparabilityReceiptRef::new(
            digest(b'r'),
            vec!["obs-b".into(), "obs-a".into()],
            "support-a",
            "vegetation-loss",
            "ontology-v1",
        ).unwrap();
        assert_eq!(
            receipt.observation_ids().iter().map(String::as_str).collect::<Vec<_>>(),
            vec!["obs-a", "obs-b"]
        );
    }

    #[test]
    fn cross_processing_generation_claim_requires_comparability_receipt() {
        let obs_a = observation("obs-a", "2024-01-01", ObservationQuality::QualifiedUsable);
        let mut obs_b = observation("obs-b", "2025-01-01", ObservationQuality::QualifiedUsable);
        obs_b.processing_baseline = Some("fixture-baseline-v2".into());
        let bridge_signatures = vec![obs_a.system_signature(), obs_b.system_signature()];

        // Same target/support and high POD are not enough when the processing generation changed.
        let claim_without_bridge = no_change_claim();
        let blocked = EvidenceBundle::new(vec![obs_a.clone(), obs_b.clone()], vec![claim_without_bridge]);
        assert!(blocked.unwrap_err().contains("cross-system comparability receipt"));

        let mut claim_with_bridge = no_change_claim();
        claim_with_bridge.cross_system_comparability_receipt = Some(
            CrossSystemComparabilityReceiptRef::new(
                digest(b'f'),
                vec!["obs-a".into(), "obs-b".into()],
                bridge_signatures.clone(),
                "support-a",
                "vegetation-loss",
                "fixture-ontology-v1",
            ).unwrap()
        );
        let admitted_reference = EvidenceBundle::new(vec![obs_a.clone(), obs_b.clone()], vec![claim_with_bridge.clone()]);
        assert!(admitted_reference.is_ok());

        let mut wrong_observation_set = claim_with_bridge.clone();
        wrong_observation_set.cross_system_comparability_receipt = Some(
            CrossSystemComparabilityReceiptRef::new(
                digest(b'f'),
                vec!["obs-a".into()],
                bridge_signatures.clone(),
                "support-a",
                "vegetation-loss",
                "fixture-ontology-v1",
            ).unwrap()
        );
        assert!(EvidenceBundle::new(vec![obs_a.clone(), obs_b.clone()], vec![wrong_observation_set])
            .unwrap_err().contains("observation set"));

        let mut wrong_target = claim_with_bridge.clone();
        wrong_target.cross_system_comparability_receipt = Some(
            CrossSystemComparabilityReceiptRef::new(
                digest(b'f'),
                vec!["obs-a".into(), "obs-b".into()],
                bridge_signatures.clone(),
                "support-a",
                "built-up-expansion",
                "fixture-ontology-v1",
            ).unwrap()
        );
        assert!(EvidenceBundle::new(vec![obs_a.clone(), obs_b.clone()], vec![wrong_target])
            .unwrap_err().contains("scope"));

        let mut wrong_support = claim_with_bridge.clone();
        wrong_support.cross_system_comparability_receipt = Some(
            CrossSystemComparabilityReceiptRef::new(
                digest(b'f'),
                vec!["obs-a".into(), "obs-b".into()],
                bridge_signatures.clone(),
                "other-support",
                "vegetation-loss",
                "fixture-ontology-v1",
            ).unwrap()
        );
        assert!(EvidenceBundle::new(vec![obs_a.clone(), obs_b.clone()], vec![wrong_support])
            .unwrap_err().contains("scope"));

        let mut wrong_ontology = claim_with_bridge;
        wrong_ontology.cross_system_comparability_receipt = Some(
            CrossSystemComparabilityReceiptRef::new(
                digest(b'f'),
                vec!["obs-a".into(), "obs-b".into()],
                bridge_signatures.clone(),
                "support-a",
                "vegetation-loss",
                "fixture-ontology-v2",
            ).unwrap()
        );
        assert!(EvidenceBundle::new(vec![obs_a, obs_b], vec![wrong_ontology])
            .unwrap_err().contains("scope"));
    }

    #[test]
    fn asset_band_or_polarization_profile_changes_the_measurement_system_signature() {
        let a = observation("obs-a", "2024-01-01", ObservationQuality::QualifiedUsable);
        let mut b = observation("obs-b", "2025-01-01", ObservationQuality::QualifiedUsable);
        b.measurement_profile_sha256 = Some(digest(b'x'));
        assert_ne!(a.system_signature(), b.system_signature());
    }

    #[test]
    fn missing_asset_measurement_profile_requires_comparability_receipt() {
        let mut a = observation("obs-a", "2024-01-01", ObservationQuality::QualifiedUsable);
        let b = observation("obs-b", "2025-01-01", ObservationQuality::QualifiedUsable);
        a.measurement_profile_sha256 = None;
        let result = EvidenceBundle::new(vec![a.clone(), b.clone()], vec![no_change_claim()]);
        assert!(result.unwrap_err().contains("incompletely identified measurement systems"));

        let mut claim = no_change_claim();
        claim.cross_system_comparability_receipt = Some(
            CrossSystemComparabilityReceiptRef::new(
                digest(b'g'),
                vec!["obs-a".into(), "obs-b".into()],
                bridge_signatures.clone(),
                "support-a",
                "vegetation-loss",
                "fixture-ontology-v1",
            ).unwrap()
        );
        assert!(EvidenceBundle::new(vec![a,b], vec![claim]).is_ok());
    }

    #[test]
    fn two_unknown_processing_baselines_are_not_assumed_to_match() {
        let mut a = observation("obs-a", "2024-01-01", ObservationQuality::QualifiedUsable);
        let mut b = observation("obs-b", "2025-01-01", ObservationQuality::QualifiedUsable);
        a.processing_baseline = None;
        b.processing_baseline = None;
        assert_eq!(a.system_signature(), b.system_signature());
        let blocked = EvidenceBundle::new(vec![a.clone(), b.clone()], vec![no_change_claim()]);
        assert!(blocked.unwrap_err().contains("incompletely identified measurement systems"));

        let mut claim = no_change_claim();
        claim.cross_system_comparability_receipt = Some(
            CrossSystemComparabilityReceiptRef::new(
                digest(b"u"),
                vec!["obs-a".into(), "obs-b".into()],
                bridge_signatures.clone(),
                "support-a",
                "vegetation-loss",
                "fixture-ontology-v1",
            ).unwrap()
        );
        assert!(EvidenceBundle::new(vec![a,b], vec![claim]).is_ok());
    }

    #[test]
    fn cross_platform_and_collection_changes_are_distinct_system_signatures() {
        let mut obs_b = observation("obs-b", "2025-01-01", ObservationQuality::QualifiedUsable);
        obs_b.platform_id = "fixture-platform-b".into();
        obs_b.collection_id = Some("fixture-collection-v2".into());
        let signatures = [
            observation("obs-a", "2024-01-01", ObservationQuality::QualifiedUsable).system_signature(),
            obs_b.system_signature()
        ].into_iter().collect::<BTreeSet<_>>();
        assert_eq!(signatures.len(), 2);
        assert_ne!(
            signatures.iter().next().unwrap(),
            signatures.iter().next_back().unwrap()
        );
    }

    #[test]
    fn same_system_claim_rejects_unnecessary_cross_system_receipt() {
        let mut claim = no_change_claim();
        claim.cross_system_comparability_receipt = Some(
            CrossSystemComparabilityReceiptRef::new(
                digest(b'f'),
                vec!["obs-a".into(), "obs-b".into()],
                bridge_signatures.clone(),
                "support-a",
                "vegetation-loss",
                "fixture-ontology-v1",
            ).unwrap()
        );
        let observations = vec![
            observation("obs-a", "2024-01-01", ObservationQuality::QualifiedUsable),
            observation("obs-b", "2025-01-01", ObservationQuality::QualifiedUsable),
        ];
        let result = EvidenceBundle::new(observations, vec![claim]);
        assert!(result.unwrap_err().contains("only one observation-system signature"));
    }

    #[test]
    fn no_change_requires_two_distinct_acquisition_times() {
        let bundle = EvidenceBundle::new(
            vec![
                observation("obs-a", "2024-01-01", ObservationQuality::QualifiedUsable),
                observation("obs-b", "2024-01-01", ObservationQuality::QualifiedUsable),
            ],
            vec![no_change_claim()],
        );
        assert!(bundle.unwrap_err().contains("two distinct declared acquisition times"));
    }

    #[test]
    fn rights_receipt_must_bind_the_exact_asset_and_bundle_scope() {
        let wrong_asset_rights = rights("fixture:some-other-asset");
        assert!(ObservationRecord::new(
            "obs-a",
            AssetId::new("fixture:obs-a").unwrap(),
            "fixture-source",
            "fixture-platform",
            "fixture-sensor",
            SensorIdentityOrigin::ProviderDeclared,
            Some("fixture-instrument".into()),
            Some("fixture-collection".into()),
            Some("fixture-baseline".into()),
            digest(b'b'),
            digest(b'c'),
            wrong_asset_rights,
            "support-a",
            ObservationTime::new(
                ObservationTimeKind::Instant,
                Some("2024-01-01".into()),
                None
            ).unwrap(),
            ObservationModality::Optical,
            ObservationQuality::QualifiedUsable,
        ).is_err());

        let wrong_source_rights = RightsReceiptRef::new(
            "fixture-subject",
            "internal-evaluation",
            "different-source",
            "fixture:obs-a",
            digest(b'a'),
        ).unwrap();
        assert!(ObservationRecord::new(
            "obs-a",
            AssetId::new("fixture:obs-a").unwrap(),
            "fixture-source",
            "fixture-platform",
            "fixture-sensor",
            SensorIdentityOrigin::ProviderDeclared,
            Some("fixture-instrument".into()),
            Some("fixture-collection".into()),
            Some("fixture-baseline".into()),
            digest(b'b'),
            digest(b'c'),
            wrong_source_rights,
            "support-a",
            ObservationTime::new(
                ObservationTimeKind::Instant,
                Some("2024-01-01".into()),
                None
            ).unwrap(),
            ObservationModality::Optical,
            ObservationQuality::QualifiedUsable,
        ).is_err());

        let mismatched_scope = ObservationRecord::new(
            "obs-b",
            AssetId::new("fixture:obs-b").unwrap(),
            "fixture-source",
            "fixture-platform",
            "fixture-sensor",
            SensorIdentityOrigin::ProviderDeclared,
            Some("fixture-instrument".into()),
            Some("fixture-collection".into()),
            Some("fixture-baseline".into()),
            digest(b'b'),
            digest(b'c'),
            RightsReceiptRef::new(
                "different-subject",
                "internal-evaluation",
                "fixture-source",
                "fixture:obs-b",
                digest(b'a'),
            ).unwrap(),
            "support-a",
            ObservationTime::new(
                ObservationTimeKind::Instant,
                Some("2025-01-01".into()),
                None
            ).unwrap(),
            ObservationModality::Optical,
            ObservationQuality::QualifiedUsable,
        ).unwrap();
        assert!(EvidenceBundle::new(
            vec![
                observation("obs-a", "2024-01-01", ObservationQuality::QualifiedUsable),
                mismatched_scope
            ],
            vec![]
        ).is_err());
    }

    #[test]
    fn sensor_identity_origin_is_preserved_and_unknown_cannot_support_a_claim() {
        let unknown = ObservationRecord::new(
            "obs-unknown-sensor",
            AssetId::new("fixture:obs-unknown-sensor").unwrap(),
            "fixture-source",
            "fixture-platform",
            "unknown",
            SensorIdentityOrigin::Unknown,
            None,
            Some("fixture-collection".into()),
            None,
            digest(b'b'),
            digest(b'c'),
            rights("fixture:obs-unknown-sensor"),
            "support-a",
            ObservationTime::new(ObservationTimeKind::Instant, Some("2024-01-01".into()), None).unwrap(),
            ObservationModality::Optical,
            ObservationQuality::QualifiedUsable,
        ).unwrap();
        assert!(!unknown.has_qualified_source_identity());
        let unresolved_claim = ChangeClaim::new(
            "claim-unknown-sensor",
            "vegetation-loss",
            "fixture-ontology-v1",
            "support-a",
            TimeWindow::new("2024-01-01", "2025-01-01").unwrap(),
            ChangeDisposition::CandidateChange,
            None,
            vec!["obs-unknown-sensor".into()],
            vec!["obs-unknown-sensor".into()],
            vec![],
            CoverageState::Unknown,
            None,
            None,
            None,
            None,
            None,
        ).unwrap();
        assert!(EvidenceBundle::new(vec![unknown], vec![unresolved_claim]).is_err());

        let derived = ObservationRecord::new(
            "obs-derived-sensor",
            AssetId::new("fixture:obs-derived-sensor").unwrap(),
            "fixture-source",
            "sentinel-2a",
            "derived:c2VudGluZWwtMmE:TVNJ",
            SensorIdentityOrigin::DerivedFromPlatformAndInstrument,
            Some("MSI".into()),
            Some("sentinel-2-l2a".into()),
            Some("04.00".into()),
            digest(b'b'),
            digest(b'c'),
            rights("fixture:obs-derived-sensor"),
            "support-a",
            ObservationTime::new(ObservationTimeKind::Instant, Some("2024-01-01".into()), None).unwrap(),
            ObservationModality::Optical,
            ObservationQuality::QualifiedUsable,
        ).unwrap();
        assert!(derived.has_qualified_source_identity());
        assert_eq!(
            derived.sensor_identity_origin(),
            SensorIdentityOrigin::DerivedFromPlatformAndInstrument
        );
        assert!(ObservationRecord::new(
            "obs-invalid-derived",
            AssetId::new("fixture:obs-invalid-derived").unwrap(),
            "fixture-source",
            "sentinel-2a",
            "sentinel-2a:MSI",
            SensorIdentityOrigin::DerivedFromPlatformAndInstrument,
            None,
            Some("sentinel-2-l2a".into()),
            Some("04.00".into()),
            digest(b'b'),
            digest(b'c'),
            rights("fixture:obs-invalid-derived"),
            "support-a",
            ObservationTime::new(ObservationTimeKind::Instant, Some("2024-01-01".into()), None).unwrap(),
            ObservationModality::Optical,
            ObservationQuality::QualifiedUsable,
        ).is_err());
    }

    #[test]
    fn provider_native_platform_and_sensor_identity_are_required() {
        assert!(ObservationRecord::new(
            "obs-invalid-sensor",
            AssetId::new("fixture:obs-invalid-sensor").unwrap(),
            "fixture-source",
            " ",
            "fixture-sensor",
            SensorIdentityOrigin::ProviderDeclared,
            Some("fixture-instrument".into()),
            Some("fixture-collection".into()),
            Some("fixture-baseline".into()),
            digest(b'b'),
            digest(b'c'),
            rights("fixture:obs-invalid-sensor"),
            "support-a",
            ObservationTime::new(
                ObservationTimeKind::Instant,
                Some("2024-01-01".into()),
                None
            ).unwrap(),
            ObservationModality::Optical,
            ObservationQuality::QualifiedUsable,
        ).is_err());
    }

    #[test]
    fn mismatched_support_and_missing_receipt_digests_fail() {
        assert!(ObservationRecord::new(
            "obs-b", AssetId::new("fixture:obs-b").unwrap(), "fixture-source", "fixture-platform", "fixture-sensor", SensorIdentityOrigin::ProviderDeclared, Some("fixture-instrument".into()), Some("fixture-collection".into()), Some("fixture-baseline".into()), "not-a-digest", digest(b'c'),
            rights("fixture:obs-b"), "support-a",
            ObservationTime::new(ObservationTimeKind::Instant, Some("2024-01-01".into()), None).unwrap(),
            ObservationModality::Optical, ObservationQuality::QualifiedUsable
        ).is_err());
    }

    #[test]
    fn serialized_bundle_revalidates_cross_record_invariants() {
        let valid = EvidenceBundle::new(
            vec![
                observation("obs-a", "2024-01-01", ObservationQuality::QualifiedUsable),
                observation("obs-b", "2025-01-01", ObservationQuality::QualifiedUsable),
            ],
            vec![no_change_claim()],
        ).unwrap();
        let json = serde_json::to_string(&valid).unwrap();
        let decoded: EvidenceBundle = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded, valid);
        assert_eq!(decoded.observations()[0].platform_id(), "fixture-platform");
        assert_eq!(decoded.observations()[0].sensor_id(), "fixture-sensor");
        assert_eq!(
            decoded.observations()[0].sensor_identity_origin(),
            SensorIdentityOrigin::ProviderDeclared
        );
        assert_eq!(decoded.observations()[0].instrument_id(), Some("fixture-instrument"));
        assert_eq!(decoded.observations()[0].collection_id(), Some("fixture-collection"));
        assert_eq!(decoded.observations()[0].processing_baseline(), Some("fixture-baseline"));
        assert_eq!(
            decoded.observations()[0].measurement_profile_sha256(),
            Some("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa")
        );

        let mut value: serde_json::Value = serde_json::from_str(&json).unwrap();
        value["observations"][0]["quality"] = serde_json::json!("cloud_contaminated");
        let invalid = serde_json::from_value::<EvidenceBundle>(value);
        assert!(invalid.is_err());
    }

    #[test]
    fn claim_reference_must_exist_and_be_in_the_same_support() {
        let missing = ChangeClaim::new(
            "candidate", "built-up-change", "ontology-v1", "support-a",
            TimeWindow::new("2024-01-01", "2025-01-01").unwrap(),
            ChangeDisposition::CandidateChange, None,
            vec!["absent".into()], vec!["absent".into()], vec![],
            CoverageState::Unknown, None, None, None, None, None
        ).unwrap();
        assert!(EvidenceBundle::new(vec![], vec![missing]).is_err());

        let claim = ChangeClaim::new(
            "candidate", "built-up-change", "ontology-v1", "support-a",
            TimeWindow::new("2024-01-01", "2025-01-01").unwrap(),
            ChangeDisposition::CandidateChange, None,
            vec!["obs-b".into()], vec!["obs-b".into()], vec![],
            CoverageState::Unknown, None, None, None, None, None
        ).unwrap();
        let other_support = ObservationRecord::new(
            "obs-b", AssetId::new("fixture:obs-b").unwrap(), "fixture-source", "fixture-platform", "fixture-sensor", SensorIdentityOrigin::ProviderDeclared, Some("fixture-instrument".into()), Some("fixture-collection".into()), Some("fixture-baseline".into()), digest(b'b'), digest(b'c'),
            rights("fixture:obs-b"), "support-b",
            ObservationTime::new(ObservationTimeKind::Instant, Some("2024-01-01".into()), None).unwrap(),
            ObservationModality::Optical, ObservationQuality::QualifiedUsable
        ).unwrap();
        assert!(EvidenceBundle::new(vec![other_support], vec![claim]).is_err());
    }

    #[test]
    fn checked_in_cross_sensor_bridge_fixture_deserializes_and_revalidates() {
        let fixture = include_str!("../tests/fixtures/earth_observation/cross_sensor_comparability_bundle.json");
        let bundle: EvidenceBundle = serde_json::from_str(fixture).unwrap();
        assert_eq!(bundle.observations().len(), 2);
        assert_eq!(bundle.claims().len(), 1);
        let claim = &bundle.claims()[0];
        let receipt = claim.cross_system_comparability_receipt().unwrap();
        assert_eq!(
            receipt.observation_ids().iter().map(String::as_str).collect::<Vec<_>>(),
            vec!["obs-2024", "obs-2025"]
        );
        assert_eq!(receipt.support_id(), "support-jukskei-synthetic");
        assert_eq!(receipt.target_class_id(), "vegetation-loss");
        assert_eq!(receipt.ontology_version(), "fixture-ontology-v1");
        assert_eq!(
            bundle.observations()[1].system_signature().platform_id(),
            "sentinel-2b"
        );
    }

    #[test]
    fn checked_in_no_change_fixture_deserializes_and_revalidates() {
        let fixture = include_str!("../tests/fixtures/earth_observation/valid_no_change_bundle.json");
        let bundle: EvidenceBundle = serde_json::from_str(fixture).unwrap();
        assert_eq!(bundle.schema_version(), EARTH_OBSERVATION_SCHEMA_VERSION);
        assert_eq!(bundle.observations().len(), 2);
        assert_eq!(bundle.claims().len(), 1);
        assert_eq!(
            bundle.claims()[0].disposition(),
            ChangeDisposition::NoChangeWithinEnvelope
        );
    }

    #[test]
    fn strict_wire_contract_rejects_unknown_fields_and_wrong_schema_version() {
        let valid = EvidenceBundle::new(
            vec![
                observation("obs-a", "2024-01-01", ObservationQuality::QualifiedUsable),
                observation("obs-b", "2025-01-01", ObservationQuality::QualifiedUsable),
            ],
            vec![no_change_claim()],
        ).unwrap();
        let mut value: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&valid).unwrap()).unwrap();
        value["schema_version"] = serde_json::json!(99);
        assert!(serde_json::from_value::<EvidenceBundle>(value).is_err());

        let mut value: serde_json::Value =
            serde_json::from_str(&serde_json::to_string(&valid).unwrap()).unwrap();
        value["unexpected"] = serde_json::json!(true);
        assert!(serde_json::from_value::<EvidenceBundle>(value).is_err());
    }
}
