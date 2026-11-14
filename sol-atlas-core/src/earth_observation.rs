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

use crate::geospatial_assets::{AssetId, ObservationTime, ObservationTimeKind};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::convert::TryFrom;

pub const EARTH_OBSERVATION_SCHEMA_VERSION: u32 = 1;

/// The EO sensor family. Provider-specific sensor and product identifiers
/// remain attached to the referenced asset/processing receipts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObservationModality {
    Optical,
    SyntheticApertureRadar,
    ThermalInfrared,
    Hyperspectral,
    Multispectral,
    Other,
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
    source_asset_receipt_sha256: String,
    processing_receipt_sha256: String,
    rights: RightsReceiptRef,
    support_id: String,
    acquisition: ObservationTime,
    modality: ObservationModality,
    quality: ObservationQuality,
}

impl ObservationRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        observation_id: impl Into<String>,
        asset_id: AssetId,
        source_id: impl Into<String>,
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
        let support_id = support_id.into();
        validate_identifier(&observation_id, "observation ID")?;
        validate_identifier(&source_id, "source ID")?;
        validate_identifier(&support_id, "support ID")?;
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
        }

        if self.disposition == ChangeDisposition::NoChangeWithinEnvelope {
            if referenced
                .iter()
                .any(|observation| !observation.is_usable_observation())
            {
                return Err("no-change claim includes an EO-unusable observation".into());
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
            CoverageState::Partial, None, Some(digest(b'e')),
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
            Some(DetectionLimit::new("min-change", 0.05, "fraction").unwrap())
        ).is_err());
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
    fn mismatched_support_and_missing_receipt_digests_fail() {
        assert!(ObservationRecord::new(
            "obs-b", AssetId::new("fixture:obs-b").unwrap(), "fixture-source", "not-a-digest", digest(b'c'),
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
            CoverageState::Unknown, None, None, None
        ).unwrap();
        assert!(EvidenceBundle::new(vec![], vec![missing]).is_err());

        let claim = ChangeClaim::new(
            "candidate", "built-up-change", "ontology-v1", "support-a",
            TimeWindow::new("2024-01-01", "2025-01-01").unwrap(),
            ChangeDisposition::CandidateChange, None,
            vec!["obs-b".into()], vec!["obs-b".into()], vec![],
            CoverageState::Unknown, None, None, None
        ).unwrap();
        let other_support = ObservationRecord::new(
            "obs-b", AssetId::new("fixture:obs-b").unwrap(), "fixture-source", digest(b'b'), digest(b'c'),
            rights("fixture:obs-b"), "support-b",
            ObservationTime::new(ObservationTimeKind::Instant, Some("2024-01-01".into()), None).unwrap(),
            ObservationModality::Optical, ObservationQuality::QualifiedUsable
        ).unwrap();
        assert!(EvidenceBundle::new(vec![other_support], vec![claim]).is_err());
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
