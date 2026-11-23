// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Typed evidence primitives for spatial-economic coordination.
//!
//! These types encode distinctions between claims; they do not establish that a
//! claim is true. Signatures, source digests, and transparency receipts establish
//! provenance/integrity properties, not physical truth or causal impact.

use serde::{Deserialize, Serialize};

pub const EVIDENCE_SCHEMA_VERSION: &str = "0.1.0";

/// The kind of statement being made. Kinds are deliberately not interchangeable:
/// a commitment is not a delivery, and a settlement is not an impact measurement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceKind {
    Observation,
    Forecast,
    Scenario,
    Commitment,
    Delivery,
    Acceptance,
    Invoice,
    Settlement,
    ImpactMeasurement,
}

/// Epistemic/lifecycle state of a statement, not the business state of an order.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceState {
    Proposed,
    Asserted,
    Corroborated,
    Disputed,
    Superseded,
    Withdrawn,
}

impl EvidenceState {
    /// Return whether a statement may make this evidence-state transition.
    ///
    /// Corrections create a new statement that references the old one; terminal
    /// statements are not rewritten. This is a local state-machine rule, not a
    /// cryptographic audit log.
    pub const fn permits_transition_to(self, next: Self) -> bool {
        use EvidenceState::{Asserted, Corroborated, Disputed, Proposed, Superseded, Withdrawn};

        matches!(
            (self, next),
            (Proposed, Asserted)
                | (Proposed, Withdrawn)
                | (Asserted, Corroborated)
                | (Asserted, Disputed)
                | (Asserted, Superseded)
                | (Asserted, Withdrawn)
                | (Corroborated, Disputed)
                | (Corroborated, Superseded)
                | (Corroborated, Withdrawn)
                | (Disputed, Superseded)
                | (Disputed, Withdrawn)
        )
    }
}

/// Stable place/asset identity plus optional geometry location.
///
/// Geometry is referenced, rather than embedded, so this contract can point at
/// canonical spatial objects in a GeoPackage, OGC API, or another permitted store.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SpatialContext {
    pub entity_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub geometry_uri: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub crs: Option<String>,
}

/// Event/observation time and record time are separate on purpose.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TimeContext {
    /// When the event or observation occurred, RFC 3339.
    pub occurred_at: String,
    /// When this statement was recorded, RFC 3339.
    pub recorded_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_from: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_until: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Quantity {
    pub value: f64,
    /// UCUM-compatible unit where practical; domain-specific codes must be documented.
    pub unit: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uncertainty_abs: Option<f64>,
}

/// A decimal amount and an explicit currency code. Never infer currency from
/// locale, and do not collapse this into a generic score.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Money {
    /// Base-10 decimal string to avoid binary floating-point monetary arithmetic.
    pub amount_decimal: String,
    /// Three uppercase ASCII letters; implementations should validate against
    /// the currency registry used by their deployment.
    pub currency: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceArtifact {
    pub id: String,
    pub uri: String,
    pub role: String,
    /// Digest of the exact captured bytes, when bytes are available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sha256: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Provenance {
    pub method: String,
    #[serde(default)]
    pub source_artifacts: Vec<SourceArtifact>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub derivation_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EvidenceRecord {
    pub id: String,
    pub schema_version: String,
    pub kind: EvidenceKind,
    pub state: EvidenceState,
    pub issuer_id: String,
    pub subject_id: String,
    pub spatial: SpatialContext,
    pub time: TimeContext,
    pub statement: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub quantity: Option<Quantity>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub money: Option<Money>,
    pub provenance: Provenance,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub supersedes: Option<String>,
    /// Reference to an access-control policy; never put secrets in public statements.
    pub access_policy: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContractError {
    UnsupportedSchemaVersion,
    EmptyId,
    EmptyIssuer,
    EmptySubject,
    EmptySpatialEntity,
    EmptyStatement,
    EmptyAccessPolicy,
    EmptyOccurredAt,
    EmptyRecordedAt,
    EmptyProvenanceMethod,
    GeometryMissingCrs,
    EmptyQuantityUnit,
    NonFiniteQuantity,
    InvalidUncertainty,
    InvalidCurrencyCode,
    InvalidMoneyDecimal,
    EmptySourceArtifactId,
    EmptySourceArtifactUri,
    EmptySourceArtifactRole,
    InvalidSha256,
}

impl EvidenceRecord {
    /// Check local structural invariants without pretending to verify the claim.
    pub fn validate(&self) -> Result<(), Vec<ContractError>> {
        let mut errors = Vec::new();

        if self.schema_version != EVIDENCE_SCHEMA_VERSION {
            errors.push(ContractError::UnsupportedSchemaVersion);
        }
        if self.id.trim().is_empty() {
            errors.push(ContractError::EmptyId);
        }
        if self.issuer_id.trim().is_empty() {
            errors.push(ContractError::EmptyIssuer);
        }
        if self.subject_id.trim().is_empty() {
            errors.push(ContractError::EmptySubject);
        }
        if self.spatial.entity_id.trim().is_empty() {
            errors.push(ContractError::EmptySpatialEntity);
        }
        if self.statement.trim().is_empty() {
            errors.push(ContractError::EmptyStatement);
        }
        if self.access_policy.trim().is_empty() {
            errors.push(ContractError::EmptyAccessPolicy);
        }
        if self.time.occurred_at.trim().is_empty() {
            errors.push(ContractError::EmptyOccurredAt);
        }
        if self.time.recorded_at.trim().is_empty() {
            errors.push(ContractError::EmptyRecordedAt);
        }
        if self.provenance.method.trim().is_empty() {
            errors.push(ContractError::EmptyProvenanceMethod);
        }
        if self.spatial.geometry_uri.is_some()
            && self.spatial.crs.as_deref().unwrap_or("").trim().is_empty()
        {
            errors.push(ContractError::GeometryMissingCrs);
        }

        if let Some(quantity) = &self.quantity {
            if quantity.unit.trim().is_empty() {
                errors.push(ContractError::EmptyQuantityUnit);
            }
            if !quantity.value.is_finite() {
                errors.push(ContractError::NonFiniteQuantity);
            }
            if quantity
                .uncertainty_abs
                .is_some_and(|v| !v.is_finite() || v < 0.0)
            {
                errors.push(ContractError::InvalidUncertainty);
            }
        }

        if let Some(money) = &self.money {
            let code = money.currency.as_bytes();
            if code.len() != 3 || !code.iter().all(u8::is_ascii_uppercase) {
                errors.push(ContractError::InvalidCurrencyCode);
            }
            if !is_decimal(&money.amount_decimal) {
                errors.push(ContractError::InvalidMoneyDecimal);
            }
        }

        for source in &self.provenance.source_artifacts {
            if source.id.trim().is_empty() {
                errors.push(ContractError::EmptySourceArtifactId);
            }
            if source.uri.trim().is_empty() {
                errors.push(ContractError::EmptySourceArtifactUri);
            }
            if source.role.trim().is_empty() {
                errors.push(ContractError::EmptySourceArtifactRole);
            }
            if source.sha256.as_deref().is_some_and(|digest| {
                digest.len() != 64 || !digest.bytes().all(|b| b.is_ascii_hexdigit())
            }) {
                errors.push(ContractError::InvalidSha256);
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

/// Small decimal grammar for string-based money values; no exponent notation.
fn is_decimal(value: &str) -> bool {
    let value = value.strip_prefix('-').unwrap_or(value);
    let mut parts = value.split('.');
    let whole = parts.next().unwrap_or_default();
    if whole.len() > 1 && whole.starts_with('0') {
        return false;
    }
    let fraction = parts.next();
    if parts.next().is_some() || whole.is_empty() || !whole.bytes().all(|b| b.is_ascii_digit()) {
        return false;
    }
    fraction.is_none_or(|digits| !digits.is_empty() && digits.bytes().all(|b| b.is_ascii_digit()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_record() -> EvidenceRecord {
        EvidenceRecord {
            id: "urn:example:evidence:1".into(),
            schema_version: EVIDENCE_SCHEMA_VERSION.into(),
            kind: EvidenceKind::Observation,
            state: EvidenceState::Asserted,
            issuer_id: "did:example:observer".into(),
            subject_id: "urn:example:water-site:7".into(),
            spatial: SpatialContext {
                entity_id: "urn:example:water-site:7".into(),
                geometry_uri: Some("https://example.test/geometries/site-7".into()),
                crs: Some("EPSG:4326".into()),
            },
            time: TimeContext {
                occurred_at: "2026-10-15T10:00:00Z".into(),
                recorded_at: "2026-10-15T10:01:00Z".into(),
                valid_from: None,
                valid_until: None,
            },
            statement: "Water level observed at the monitoring site".into(),
            quantity: Some(Quantity {
                value: 12.5,
                unit: "m".into(),
                uncertainty_abs: Some(0.2),
            }),
            money: None,
            provenance: Provenance {
                method: "manual_gauge_observation".into(),
                source_artifacts: vec![SourceArtifact {
                    id: "urn:example:artifact:1".into(),
                    uri: "https://example.test/capture/1".into(),
                    role: "raw_measurement".into(),
                    sha256: Some("a".repeat(64)),
                }],
                derivation_id: None,
            },
            supersedes: None,
            access_policy: "policy:public-observation".into(),
        }
    }

    #[test]
    fn accepts_structurally_valid_observation() {
        assert_eq!(valid_record().validate(), Ok(()));
    }

    #[test]
    fn rejects_geometry_without_crs() {
        let mut record = valid_record();
        record.spatial.crs = None;
        assert!(
            record
                .validate()
                .unwrap_err()
                .contains(&ContractError::GeometryMissingCrs)
        );
    }

    #[test]
    fn rejects_non_finite_measurement_and_negative_uncertainty() {
        let mut record = valid_record();
        record.quantity.as_mut().unwrap().value = f64::NAN;
        record.quantity.as_mut().unwrap().uncertainty_abs = Some(-1.0);
        let errors = record.validate().unwrap_err();
        assert!(errors.contains(&ContractError::NonFiniteQuantity));
        assert!(errors.contains(&ContractError::InvalidUncertainty));
    }

    #[test]
    fn money_uses_decimal_strings_and_explicit_currency() {
        let mut record = valid_record();
        record.money = Some(Money {
            amount_decimal: "101.25".into(),
            currency: "MZN".into(),
        });
        assert_eq!(record.validate(), Ok(()));
        record.money.as_mut().unwrap().amount_decimal = "01.25".into();
        let leading_zero_errors = record.validate().unwrap_err();
        assert!(leading_zero_errors.contains(&ContractError::InvalidMoneyDecimal));

        record.money.as_mut().unwrap().amount_decimal = "1e2".into();
        record.money.as_mut().unwrap().currency = "usd".into();
        let errors = record.validate().unwrap_err();
        assert!(errors.contains(&ContractError::InvalidMoneyDecimal));
        assert!(errors.contains(&ContractError::InvalidCurrencyCode));
    }

    #[test]
    fn rejects_bad_digest_and_unknown_schema_version() {
        let mut record = valid_record();
        record.schema_version = "9.9.9".into();
        record.provenance.source_artifacts[0].sha256 = Some("not-a-digest".into());
        let errors = record.validate().unwrap_err();
        assert!(errors.contains(&ContractError::UnsupportedSchemaVersion));
        assert!(errors.contains(&ContractError::InvalidSha256));
    }

    #[test]
    fn evidence_state_machine_has_terminal_corrections() {
        assert!(EvidenceState::Proposed.permits_transition_to(EvidenceState::Asserted));
        assert!(EvidenceState::Asserted.permits_transition_to(EvidenceState::Disputed));
        assert!(EvidenceState::Disputed.permits_transition_to(EvidenceState::Superseded));
        assert!(!EvidenceState::Corroborated.permits_transition_to(EvidenceState::Proposed));
        assert!(!EvidenceState::Superseded.permits_transition_to(EvidenceState::Asserted));
        assert!(!EvidenceState::Withdrawn.permits_transition_to(EvidenceState::Corroborated));
    }

    #[test]
    fn evidence_kinds_are_not_business_statuses() {
        let mut record = valid_record();
        record.kind = EvidenceKind::Commitment;
        assert_eq!(record.kind, EvidenceKind::Commitment);
        // Fulfilling a commitment must be represented by a separate Delivery
        // record, not by converting this statement's kind in place.
        let delivery = EvidenceKind::Delivery;
        assert_ne!(record.kind, delivery);
        assert_eq!(record.validate(), Ok(()));
    }

    #[test]
    fn serde_round_trip_preserves_explicit_evidence_kind() {
        let record = valid_record();
        let encoded = serde_json::to_string(&record).unwrap();
        let decoded: EvidenceRecord = serde_json::from_str(&encoded).unwrap();
        assert_eq!(decoded, record);
    }
    #[test]
    fn schema_enums_match_rust_contract() {
        let schema: serde_json::Value = serde_json::from_str(include_str!(
            "../../schemas/spatial-economic-evidence-v0.1.schema.json"
        ))
        .unwrap();

        let kinds = schema["properties"]["kind"]["enum"].as_array().unwrap();
        let expected_kinds = [
            ("observation", EvidenceKind::Observation),
            ("forecast", EvidenceKind::Forecast),
            ("scenario", EvidenceKind::Scenario),
            ("commitment", EvidenceKind::Commitment),
            ("delivery", EvidenceKind::Delivery),
            ("acceptance", EvidenceKind::Acceptance),
            ("invoice", EvidenceKind::Invoice),
            ("settlement", EvidenceKind::Settlement),
            ("impact_measurement", EvidenceKind::ImpactMeasurement),
        ];
        for (name, kind) in expected_kinds {
            assert!(kinds.contains(&serde_json::json!(name)));
            assert_eq!(serde_json::to_value(kind).unwrap(), serde_json::json!(name));
        }

        let states = schema["properties"]["state"]["enum"].as_array().unwrap();
        let expected_states = [
            ("proposed", EvidenceState::Proposed),
            ("asserted", EvidenceState::Asserted),
            ("corroborated", EvidenceState::Corroborated),
            ("disputed", EvidenceState::Disputed),
            ("superseded", EvidenceState::Superseded),
            ("withdrawn", EvidenceState::Withdrawn),
        ];
        for (name, state) in expected_states {
            assert!(states.contains(&serde_json::json!(name)));
            assert_eq!(
                serde_json::to_value(state).unwrap(),
                serde_json::json!(name)
            );
        }

        assert_eq!(
            schema["properties"]["schema_version"],
            serde_json::json!({"const": EVIDENCE_SCHEMA_VERSION})
        );
        let money_pattern =
            schema["properties"]["money"]["properties"]["amount_decimal"]["pattern"]
                .as_str()
                .unwrap();
        assert!(money_pattern.contains(r"(?:\.[0-9]+)?"));
    }

    #[test]
    fn synthetic_example_fixture_deserializes_and_validates() {
        let record: EvidenceRecord = serde_json::from_str(include_str!(
            "../../schemas/examples/water-observation.json"
        ))
        .unwrap();
        assert_eq!(record.kind, EvidenceKind::Observation);
        assert_eq!(record.provenance.method, "synthetic_fixture");
        assert!(record.statement.starts_with("SYNTHETIC EXAMPLE ONLY"));
        assert_eq!(record.validate(), Ok(()));
    }
}
