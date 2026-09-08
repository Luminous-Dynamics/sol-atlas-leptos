// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Provenance-bearing claims about Atlas entities.
//!
//! Claims are deliberately separate from entity identity. Multiple sources may
//! describe the same entity, disagree, supersede earlier observations, or carry
//! different authority contexts without mutating the entity's stable identity.

use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use std::fmt;

use crate::entity::AtlasEntityId;

/// Stable identity for one assertion.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ClaimId(pub String);

impl ClaimId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }
}

impl From<&str> for ClaimId {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for ClaimId {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

/// How an assertion relates to evidence and reality.
///
/// `Verified` means the supplied evidence or verification procedure passed its
/// stated checks. It does not by itself grant governance, operational, or write
/// authority over the subject entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceClass {
    Observed,
    Verified,
    Curated,
    Estimated,
    Predicted,
    Contested,
    Scenario,
}

impl From<crate::types::DataKind> for EvidenceClass {
    fn from(value: crate::types::DataKind) -> Self {
        match value {
            crate::types::DataKind::Observed => Self::Observed,
            crate::types::DataKind::Curated => Self::Curated,
            crate::types::DataKind::Scenario => Self::Scenario,
        }
    }
}

/// Confidence constrained to the inclusive interval `[0, 1]`.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct Confidence(f32);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConfidenceError {
    NonFinite,
    OutOfRange,
}

impl fmt::Display for ConfidenceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NonFinite => write!(f, "confidence must be finite"),
            Self::OutOfRange => write!(f, "confidence must be within [0, 1]"),
        }
    }
}

impl Confidence {
    pub fn new(value: f32) -> Result<Self, ConfidenceError> {
        if !value.is_finite() {
            return Err(ConfidenceError::NonFinite);
        }
        if !(0.0..=1.0).contains(&value) {
            return Err(ConfidenceError::OutOfRange);
        }
        Ok(Self(value))
    }

    pub const fn certain() -> Self {
        Self(1.0)
    }

    pub const fn value(self) -> f32 {
        self.0
    }
}

impl<'de> Deserialize<'de> for Confidence {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = f32::deserialize(deserializer)?;
        Self::new(value).map_err(serde::de::Error::custom)
    }
}

/// Optional authority context associated with a claim.
///
/// Absence is meaningful: evidence can be useful without claiming any right to
/// command, operate, govern, or mutate the represented entity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AuthorityRef {
    pub authority_id: String,
    pub capability: Option<String>,
    pub jurisdiction: Option<String>,
}

/// Where a claim came from and how it was produced.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Provenance {
    pub source: String,
    pub source_record_id: Option<String>,
    pub source_uri: Option<String>,
    pub evidence_class: EvidenceClass,
    pub observed_at_unix_ms: Option<i64>,
    pub ingested_at_unix_ms: Option<i64>,
    pub method: Option<String>,
}

impl Provenance {
    pub fn new(source: impl Into<String>, evidence_class: EvidenceClass) -> Self {
        Self {
            source: source.into(),
            source_record_id: None,
            source_uri: None,
            evidence_class,
            observed_at_unix_ms: None,
            ingested_at_unix_ms: None,
            method: None,
        }
    }
}

/// Lifecycle state for a claim. Historical claims remain addressable rather
/// than being silently deleted from the evidence lineage.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum ClaimStatus {
    Active,
    Retracted { reason: String },
    Superseded { by: ClaimId },
}

/// One assertion about one Atlas entity.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AtlasClaim {
    pub id: ClaimId,
    pub subject: AtlasEntityId,
    /// Open predicate vocabulary, e.g. `icu_beds_available`, `operator`,
    /// `reservoir_level_m`, or `accessibility_status`.
    pub predicate: String,
    pub value: Value,
    pub confidence: Confidence,
    pub provenance: Provenance,
    pub authority: Option<AuthorityRef>,
    pub status: ClaimStatus,
}

impl AtlasClaim {
    pub fn new(
        id: impl Into<ClaimId>,
        subject: impl Into<AtlasEntityId>,
        predicate: impl Into<String>,
        value: Value,
        confidence: Confidence,
        provenance: Provenance,
    ) -> Self {
        Self {
            id: id.into(),
            subject: subject.into(),
            predicate: predicate.into(),
            value,
            confidence,
            provenance,
            authority: None,
            status: ClaimStatus::Active,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn confidence_rejects_invalid_values_and_invalid_wire_data() {
        assert_eq!(Confidence::new(-0.01), Err(ConfidenceError::OutOfRange));
        assert_eq!(Confidence::new(1.01), Err(ConfidenceError::OutOfRange));
        assert_eq!(Confidence::new(f32::NAN), Err(ConfidenceError::NonFinite));
        assert!(serde_json::from_str::<Confidence>("1.2").is_err());
    }

    #[test]
    fn competing_claims_can_coexist_for_one_subject() {
        let subject = AtlasEntityId::new("atlas:hospital-1");
        let a = AtlasClaim::new(
            "claim:a",
            subject.clone(),
            "icu_beds_available",
            serde_json::json!(7),
            Confidence::new(0.8).unwrap(),
            Provenance::new("hospital-feed", EvidenceClass::Observed),
        );
        let b = AtlasClaim::new(
            "claim:b",
            subject.clone(),
            "icu_beds_available",
            serde_json::json!(5),
            Confidence::new(0.6).unwrap(),
            Provenance::new("community-report", EvidenceClass::Estimated),
        );

        assert_eq!(a.subject, b.subject);
        assert_eq!(a.predicate, b.predicate);
        assert_ne!(a.value, b.value);
    }

    #[test]
    fn verification_does_not_imply_authority() {
        let claim = AtlasClaim::new(
            "claim:verified",
            "atlas:bridge-1",
            "structural_condition",
            serde_json::json!("serviceable"),
            Confidence::certain(),
            Provenance::new("inspection-evidence", EvidenceClass::Verified),
        );
        assert!(claim.authority.is_none());
    }

    #[test]
    fn existing_layer_provenance_has_a_lossless_class_bridge() {
        assert_eq!(
            EvidenceClass::from(crate::types::DataKind::Observed),
            EvidenceClass::Observed
        );
        assert_eq!(
            EvidenceClass::from(crate::types::DataKind::Curated),
            EvidenceClass::Curated
        );
        assert_eq!(
            EvidenceClass::from(crate::types::DataKind::Scenario),
            EvidenceClass::Scenario
        );
    }

    #[test]
    fn claim_roundtrips_with_provenance() {
        let claim = AtlasClaim::new(
            "claim:1",
            "atlas:river-1",
            "flow_m3_s",
            serde_json::json!(112.4),
            Confidence::new(0.92).unwrap(),
            Provenance::new("sensor-network", EvidenceClass::Observed),
        );
        let encoded = serde_json::to_string(&claim).unwrap();
        let decoded: AtlasClaim = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded, claim);
    }
}
