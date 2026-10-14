// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Privacy-aware, append-only civic service cases shared across city-service domains.
//!
//! This is a read-only case/evidence contract, not a municipal ticketing service.
//! It deliberately stores no resident name, phone number, free-form narrative, or
//! precise home location. Keep identity mapping and restricted evidence outside
//! public exports; area_ref should be appropriately coarse for the chosen visibility.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CivicServiceDomain {
    Water,
    Electricity,
    Roads,
    Stormwater,
    Waste,
    TransportAccess,
    Housing,
    LocalEconomy,
    FoodSecurity,
    PublicSafety,
    ClimateResilience,
    HealthcareAccess,
    MunicipalAdministration,
    MultiService,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CaseSeverity {
    Routine,
    Moderate,
    High,
    ImmediateSafetyRisk,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CaseVisibility {
    PublicCase,
    PublicAggregateOnly,
    Restricted,
    Sensitive,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PublicAreaGranularity {
    CityRegion,
    Ward,
    Precinct,
}

/// Explicitly reviewed area code for individual public-case publication.
/// It should be a civic-area identifier, never a coordinate or street address.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicAreaReference {
    pub code: String,
    pub granularity: PublicAreaGranularity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CaseActorKind {
    Reporter,
    ServiceOperator,
    CommunityPartner,
    IndependentReviewer,
    SystemAgent,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceSourceKind {
    ResidentSubmission,
    MunicipalRecord,
    FieldInspection,
    RemoteSensing,
    ServiceProviderNotice,
    PartnerRecord,
    SyntheticTest,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceVisibility {
    Public,
    RedactedPublic,
    Restricted,
    Private,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceReview {
    Unreviewed,
    Corroborated,
    Contested,
    IndependentlyVerified,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CaseStatus {
    Reported,
    Triaged,
    Assigned,
    WorkStarted,
    ResolutionClaimed,
    VerifiedResolved,
    Reopened,
    Dismissed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CaseEventKind {
    Reported,
    Triaged,
    Assigned,
    WorkStarted,
    ResolutionClaimed,
    ResolutionVerified,
    Reopened,
    Dismissed,
    /// An append-only correction event. It never erases or rewrites prior events.
    Correction,
}

impl CaseEventKind {
    fn resulting_status(self) -> Option<CaseStatus> {
        match self {
            Self::Reported => Some(CaseStatus::Reported),
            Self::Triaged => Some(CaseStatus::Triaged),
            Self::Assigned => Some(CaseStatus::Assigned),
            Self::WorkStarted => Some(CaseStatus::WorkStarted),
            Self::ResolutionClaimed => Some(CaseStatus::ResolutionClaimed),
            Self::ResolutionVerified => Some(CaseStatus::VerifiedResolved),
            Self::Reopened => Some(CaseStatus::Reopened),
            Self::Dismissed => Some(CaseStatus::Dismissed),
            Self::Correction => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OfficialReference {
    /// Name of the source system or responsible entity; not a free-form claim.
    pub system_code: String,
    /// Original ticket/reference identifier, retained exactly as received.
    pub reference: String,
    /// Optional public URL. Private URLs and tokens must not be exported.
    pub public_url: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceReference {
    pub evidence_id: String,
    pub source_kind: EvidenceSourceKind,
    pub captured_at_unix_ms: u64,
    /// SHA-256 of captured bytes, not of a URL or an unverified description.
    pub sha256: Option<String>,
    pub review: EvidenceReview,
    pub visibility: EvidenceVisibility,
    /// Opaque artifact pointer kept inside the permitted access boundary.
    pub artifact_ref: Option<String>,
    /// Required only when marked IndependentlyVerified.
    pub reviewer_ref: Option<String>,
    /// Required only when marked IndependentlyVerified.
    pub reviewed_at_unix_ms: Option<u64>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CaseEvent {
    pub event_id: String,
    pub kind: CaseEventKind,
    /// Time the event entered this append-only record.
    pub recorded_at_unix_ms: u64,
    /// Source event time, which may differ from record time for imported evidence.
    pub occurred_at_unix_ms: Option<u64>,
    pub actor_kind: CaseActorKind,
    /// Pseudonymous or organisational reference; identity mapping stays private.
    pub actor_ref: String,
    pub evidence_ids: Vec<String>,
    /// A stable code, never an unredacted free-form resident narrative.
    pub reason_code: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CivicServiceCase {
    pub schema_version: u16,
    pub case_id: String,
    pub domain: CivicServiceDomain,
    pub category_code: String,
    /// Internal area/asset reference. Never copied directly into a public projection.
    pub internal_area_ref: String,
    /// Explicitly reviewed coarse area reference required for public case publication.
    pub public_area: Option<PublicAreaReference>,
    pub severity: CaseSeverity,
    pub visibility: CaseVisibility,
    pub opened_at_unix_ms: u64,
    pub updated_at_unix_ms: u64,
    pub official_references: Vec<OfficialReference>,
    pub evidence: Vec<EvidenceReference>,
    /// Append-only event history. Current status is derived from this history.
    pub events: Vec<CaseEvent>,
}

/// Reduced public types deliberately omit raw ticket identifiers, actor references,
/// and evidence artifact pointers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicEvidenceSummary {
    pub evidence_id: String,
    pub source_kind: EvidenceSourceKind,
    pub captured_at_unix_ms: u64,
    pub sha256: Option<String>,
    pub review: EvidenceReview,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicCaseEvent {
    pub kind: CaseEventKind,
    pub recorded_at_unix_ms: u64,
    pub occurred_at_unix_ms: Option<u64>,
    pub actor_kind: CaseActorKind,
    /// Public/redacted evidence IDs only; private evidence links are omitted.
    pub evidence_ids: Vec<String>,
    pub reason_code: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicCivicCase {
    pub schema_version: u16,
    pub case_id: String,
    pub domain: CivicServiceDomain,
    pub category_code: String,
    pub area: PublicAreaReference,
    pub severity: CaseSeverity,
    pub current_status: Option<CaseStatus>,
    pub opened_at_unix_ms: u64,
    pub updated_at_unix_ms: u64,
    pub evidence: Vec<PublicEvidenceSummary>,
    pub events: Vec<PublicCaseEvent>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CivicCaseValidationError {
    UnsupportedSchemaVersion,
    EmptyCaseId,
    EmptyCategoryCode,
    EmptyAreaReference,
    MissingPublicAreaReference,
    InvalidPublicAreaReference,
    InvalidCaseTimeRange,
    DuplicateOfficialReference,
    EmptyEvidenceId,
    DuplicateEvidenceId,
    InvalidEvidenceDigest,
    InvalidPublicUrl,
    MissingIndependentReviewerMetadata,
    DuplicateEventId,
    MissingInitialReport,
    NonMonotonicRecordTime,
    UpdatedBeforeLastEvent,
    EmptyActorReference,
    UnknownEvidenceReference,
    DuplicateEventEvidenceReference,
    InvalidStatusTransition,
    ResolutionClaimWithoutEvidence,
    ResolutionVerificationNotIndependent,
    CorrectionWithoutReasonOrEvidence,
}

impl CivicServiceCase {
    /// Build a sanitised public projection only when an explicit public-case
    /// visibility classification permits individual publication. Aggregate-only,
    /// restricted, and sensitive cases must go through a separate aggregation layer.
    /// Official ticket identifiers, actor references, and artifact pointers are omitted.
    pub fn public_projection(&self) -> Option<PublicCivicCase> {
        if self.visibility != CaseVisibility::PublicCase || self.validate().is_err() {
            return None;
        }
        let public_area = self.public_area.as_ref()?.clone();

        let public_evidence_ids: BTreeSet<&str> = self
            .evidence
            .iter()
            .filter(|item| {
                matches!(
                    item.visibility,
                    EvidenceVisibility::Public | EvidenceVisibility::RedactedPublic
                )
            })
            .map(|item| item.evidence_id.as_str())
            .collect();

        let evidence = self
            .evidence
            .iter()
            .filter(|item| public_evidence_ids.contains(item.evidence_id.as_str()))
            .map(|item| PublicEvidenceSummary {
                evidence_id: item.evidence_id.clone(),
                source_kind: item.source_kind,
                captured_at_unix_ms: item.captured_at_unix_ms,
                sha256: item.sha256.clone(),
                review: item.review,
            })
            .collect();

        let events = self
            .events
            .iter()
            .map(|event| PublicCaseEvent {
                kind: event.kind,
                recorded_at_unix_ms: event.recorded_at_unix_ms,
                occurred_at_unix_ms: event.occurred_at_unix_ms,
                actor_kind: event.actor_kind,
                evidence_ids: event
                    .evidence_ids
                    .iter()
                    .filter(|evidence_id| public_evidence_ids.contains(evidence_id.as_str()))
                    .cloned()
                    .collect(),
                reason_code: event.reason_code.clone(),
            })
            .collect();

        Some(PublicCivicCase {
            schema_version: self.schema_version,
            case_id: self.case_id.clone(),
            domain: self.domain,
            category_code: self.category_code.clone(),
            area: public_area,
            severity: self.severity,
            current_status: self.current_status(),
            opened_at_unix_ms: self.opened_at_unix_ms,
            updated_at_unix_ms: self.updated_at_unix_ms,
            evidence,
            events,
        })
    }

    /// Current logical status derived from the append-only history.
    pub fn current_status(&self) -> Option<CaseStatus> {
        self.events
            .iter()
            .rev()
            .find_map(|event| event.kind.resulting_status())
    }

    /// Structural/evidence checks. This does not prove a fault exists, authenticate
    /// the source system, or grant an operator/AI authority to dispatch a repair.
    pub fn validate(&self) -> Result<(), Vec<CivicCaseValidationError>> {
        use CivicCaseValidationError as E;

        let mut errors = Vec::new();
        if self.schema_version != 1 {
            errors.push(E::UnsupportedSchemaVersion);
        }
        if self.case_id.trim().is_empty() {
            errors.push(E::EmptyCaseId);
        }
        if self.category_code.trim().is_empty() {
            errors.push(E::EmptyCategoryCode);
        }
        if self.internal_area_ref.trim().is_empty() {
            errors.push(E::EmptyAreaReference);
        }
        if self.visibility == CaseVisibility::PublicCase {
            match self.public_area.as_ref() {
                None => errors.push(E::MissingPublicAreaReference),
                Some(area) => {
                    let valid_code = !area.code.trim().is_empty()
                        && area.code.len() <= 80
                        && area.code.bytes().all(|byte| {
                            byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.')
                        });
                    if !valid_code {
                        errors.push(E::InvalidPublicAreaReference);
                    }
                }
            }
        }
        if self.updated_at_unix_ms < self.opened_at_unix_ms {
            errors.push(E::InvalidCaseTimeRange);
        }

        let mut official_refs = BTreeSet::new();
        for reference in &self.official_references {
            let system = reference.system_code.trim();
            let value = reference.reference.trim();
            if system.is_empty() || value.is_empty()
                || !official_refs.insert((system.to_owned(), value.to_owned()))
            {
                errors.push(E::DuplicateOfficialReference);
            }
            if let Some(url) = reference.public_url.as_deref() {
                // Public links must not contain query/fragment tokens. Use a
                // separately reviewed public link rather than exposing a signed URL.
                if !url.starts_with("https://") || url.contains('?') || url.contains('#') {
                    errors.push(E::InvalidPublicUrl);
                }
            }
        }

        let mut evidence_by_id = BTreeMap::new();
        for evidence in &self.evidence {
            if evidence.evidence_id.trim().is_empty() {
                errors.push(E::EmptyEvidenceId);
                continue;
            }
            if evidence_by_id.insert(evidence.evidence_id.as_str(), evidence).is_some() {
                errors.push(E::DuplicateEvidenceId);
            }
            if let Some(digest) = evidence.sha256.as_deref() {
                if digest.len() != 64 || !digest.bytes().all(|byte| byte.is_ascii_hexdigit()) {
                    errors.push(E::InvalidEvidenceDigest);
                }
            }
            if evidence.review == EvidenceReview::IndependentlyVerified
                && (evidence
                    .reviewer_ref
                    .as_deref()
                    .is_none_or(|value| value.trim().is_empty())
                    || evidence.reviewed_at_unix_ms.is_none())
            {
                errors.push(E::MissingIndependentReviewerMetadata);
            }
        }

        if self.events.is_empty()
            || self.events.first().map(|event| event.kind) != Some(CaseEventKind::Reported)
        {
            errors.push(E::MissingInitialReport);
        }

        let mut event_ids = BTreeSet::new();
        let mut previous_recorded_at = None;
        let mut previous_status = None;
        for event in &self.events {
            if event.event_id.trim().is_empty() || !event_ids.insert(event.event_id.as_str()) {
                errors.push(E::DuplicateEventId);
            }
            if event.actor_ref.trim().is_empty() {
                errors.push(E::EmptyActorReference);
            }
            if previous_recorded_at
                .is_some_and(|previous| event.recorded_at_unix_ms < previous)
            {
                errors.push(E::NonMonotonicRecordTime);
            }
            previous_recorded_at = Some(event.recorded_at_unix_ms);

            let mut event_evidence_ids = BTreeSet::new();
            for evidence_id in &event.evidence_ids {
                if !event_evidence_ids.insert(evidence_id.as_str()) {
                    errors.push(E::DuplicateEventEvidenceReference);
                }
                if !evidence_by_id.contains_key(evidence_id.as_str()) {
                    errors.push(E::UnknownEvidenceReference);
                }
            }

            if event.kind == CaseEventKind::Correction
                && (event
                    .reason_code
                    .as_deref()
                    .is_none_or(|reason| reason.trim().is_empty())
                    || event.evidence_ids.is_empty())
            {
                errors.push(E::CorrectionWithoutReasonOrEvidence);
            }

            let Some(next_status) = event.kind.resulting_status() else {
                continue;
            };
            if let Some(current_status) = previous_status {
                if !transition_allowed(current_status, next_status) {
                    errors.push(E::InvalidStatusTransition);
                }
            } else if next_status != CaseStatus::Reported {
                errors.push(E::InvalidStatusTransition);
            }

            if next_status == CaseStatus::ResolutionClaimed && event.evidence_ids.is_empty() {
                errors.push(E::ResolutionClaimWithoutEvidence);
            }

            if next_status == CaseStatus::VerifiedResolved {
                let independent_evidence = event.evidence_ids.iter().any(|evidence_id| {
                    evidence_by_id.get(evidence_id.as_str()).is_some_and(|evidence| {
                        evidence.review == EvidenceReview::IndependentlyVerified
                    })
                });
                if event.actor_kind != CaseActorKind::IndependentReviewer
                    || previous_status != Some(CaseStatus::ResolutionClaimed)
                    || !independent_evidence
                {
                    errors.push(E::ResolutionVerificationNotIndependent);
                }
            }
            previous_status = Some(next_status);
        }

        if previous_recorded_at.is_some_and(|last| self.updated_at_unix_ms < last) {
            errors.push(E::UpdatedBeforeLastEvent);
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

fn transition_allowed(from: CaseStatus, to: CaseStatus) -> bool {
    use CaseStatus as S;
    matches!(
        (from, to),
        (S::Reported, S::Triaged | S::Dismissed)
            | (S::Triaged, S::Assigned | S::Dismissed)
            | (S::Assigned, S::WorkStarted | S::Triaged | S::Dismissed)
            | (S::WorkStarted, S::ResolutionClaimed | S::Assigned | S::Reopened)
            | (S::ResolutionClaimed, S::VerifiedResolved | S::WorkStarted | S::Reopened)
            | (S::VerifiedResolved, S::Reopened)
            | (S::Reopened, S::Triaged | S::Assigned | S::WorkStarted | S::Dismissed)
            | (S::Dismissed, S::Reopened)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn report_event() -> CaseEvent {
        CaseEvent {
            event_id: "event-1".into(),
            kind: CaseEventKind::Reported,
            recorded_at_unix_ms: 1_000,
            occurred_at_unix_ms: Some(950),
            actor_kind: CaseActorKind::Reporter,
            actor_ref: "reporter-pseudonym-1".into(),
            evidence_ids: Vec::new(),
            reason_code: Some("pothole.reported".into()),
        }
    }

    fn valid_case() -> CivicServiceCase {
        CivicServiceCase {
            schema_version: 1,
            case_id: "case-001".into(),
            domain: CivicServiceDomain::Roads,
            category_code: "pothole".into(),
            internal_area_ref: "restricted-area-reference-001".into(),
            public_area: None,
            severity: CaseSeverity::High,
            visibility: CaseVisibility::PublicAggregateOnly,
            opened_at_unix_ms: 950,
            updated_at_unix_ms: 1_000,
            official_references: vec![OfficialReference {
                system_code: "joburg-connect".into(),
                reference: "REF-EXAMPLE-001".into(),
                public_url: Some("https://example.invalid/case/REF-EXAMPLE-001".into()),
            }],
            evidence: Vec::new(),
            events: vec![report_event()],
        }
    }

    fn independently_verified_evidence() -> EvidenceReference {
        EvidenceReference {
            evidence_id: "after-photo-1".into(),
            source_kind: EvidenceSourceKind::FieldInspection,
            captured_at_unix_ms: 1_350,
            sha256: Some("a".repeat(64)),
            review: EvidenceReview::IndependentlyVerified,
            visibility: EvidenceVisibility::RedactedPublic,
            artifact_ref: Some("restricted-artifact-ref-1".into()),
            reviewer_ref: Some("independent-reviewer-1".into()),
            reviewed_at_unix_ms: Some(1_500),
        }
    }

    fn event(id: &str, kind: CaseEventKind, time: u64) -> CaseEvent {
        CaseEvent {
            event_id: id.into(),
            kind,
            recorded_at_unix_ms: time,
            occurred_at_unix_ms: Some(time),
            actor_kind: CaseActorKind::ServiceOperator,
            actor_ref: "service-operator-team".into(),
            evidence_ids: Vec::new(),
            reason_code: None,
        }
    }

    #[test]
    fn accepts_initial_report_and_preserves_official_reference() {
        let case = valid_case();
        assert_eq!(case.current_status(), Some(CaseStatus::Reported));
        assert!(case.validate().is_ok());
        assert_eq!(case.official_references[0].reference, "REF-EXAMPLE-001");
    }

    #[test]
    fn public_projection_requires_explicit_visibility_and_redacts_private_links() {
        let mut case = valid_case();
        assert!(case.public_projection().is_none());

        case.visibility = CaseVisibility::PublicCase;
        case.public_area = Some(PublicAreaReference {
            code: "ward-59".into(),
            granularity: PublicAreaGranularity::Ward,
        });
        case.evidence = vec![
            EvidenceReference {
                evidence_id: "public-evidence-1".into(),
                source_kind: EvidenceSourceKind::FieldInspection,
                captured_at_unix_ms: 1_050,
                sha256: Some("b".repeat(64)),
                review: EvidenceReview::Corroborated,
                visibility: EvidenceVisibility::RedactedPublic,
                artifact_ref: Some("private-artifact-url?token=never-export".into()),
                reviewer_ref: None,
                reviewed_at_unix_ms: None,
            },
            EvidenceReference {
                evidence_id: "private-evidence-1".into(),
                source_kind: EvidenceSourceKind::ResidentSubmission,
                captured_at_unix_ms: 1_060,
                sha256: None,
                review: EvidenceReview::Unreviewed,
                visibility: EvidenceVisibility::Private,
                artifact_ref: Some("restricted-private-pointer".into()),
                reviewer_ref: None,
                reviewed_at_unix_ms: None,
            },
        ];
        let mut correction = event("event-2", CaseEventKind::Correction, 1_100);
        correction.actor_ref = "internal-steward-reference".into();
        correction.reason_code = Some("metadata.corrected".into());
        correction.evidence_ids = vec!["private-evidence-1".into()];
        case.events.push(correction);
        case.updated_at_unix_ms = 1_100;

        assert!(case.validate().is_ok());
        let public_case = case.public_projection().expect("explicit public visibility");
        assert_eq!(public_case.evidence.len(), 1);
        assert_eq!(public_case.evidence[0].evidence_id, "public-evidence-1");
        assert!(public_case.events[1].evidence_ids.is_empty());

        let serialized = serde_json::to_string(&public_case).unwrap();
        assert!(!serialized.contains("internal-steward-reference"));
        assert!(!serialized.contains("restricted-private-pointer"));
        assert!(!serialized.contains("private-artifact-url"));
        assert!(!serialized.contains("REF-EXAMPLE-001"));
        assert!(!serialized.contains("official_references"));
        assert!(!serialized.contains("restricted-area-reference-001"));
        assert!(serialized.contains("ward-59"));
    }

    #[test]
    fn public_case_requires_a_coarse_public_area_reference() {
        let mut case = valid_case();
        case.visibility = CaseVisibility::PublicCase;
        assert!(case
            .validate()
            .unwrap_err()
            .contains(&CivicCaseValidationError::MissingPublicAreaReference));
        assert!(case.public_projection().is_none());

        case.public_area = Some(PublicAreaReference {
            code: "-26.2_28.1".into(),
            granularity: PublicAreaGranularity::Ward,
        });
        assert!(case
            .validate()
            .unwrap_err()
            .contains(&CivicCaseValidationError::InvalidPublicAreaReference));
    }

    #[test]
    fn public_urls_reject_query_or_fragment_tokens() {
        let mut case = valid_case();
        case.official_references[0].public_url =
            Some("https://example.invalid/case?id=secret-token".into());
        assert!(case
            .validate()
            .unwrap_err()
            .contains(&CivicCaseValidationError::InvalidPublicUrl));
    }

    #[test]
    fn duplicate_event_ids_and_out_of_order_record_times_fail_closed() {
        let mut case = valid_case();
        let mut second = event("event-1", CaseEventKind::Triaged, 900);
        second.actor_ref = "triage-team".into();
        case.events.push(second);
        case.updated_at_unix_ms = 1_000;
        let errors = case.validate().unwrap_err();
        assert!(errors.contains(&CivicCaseValidationError::DuplicateEventId));
        assert!(errors.contains(&CivicCaseValidationError::NonMonotonicRecordTime));
    }

    #[test]
    fn resolution_cannot_be_claimed_without_linked_evidence() {
        let mut case = valid_case();
        let mut triaged = event("event-2", CaseEventKind::Triaged, 1_100);
        triaged.actor_ref = "triage-team".into();
        let mut assigned = event("event-3", CaseEventKind::Assigned, 1_200);
        assigned.actor_ref = "dispatch-team".into();
        let mut started = event("event-4", CaseEventKind::WorkStarted, 1_300);
        started.actor_ref = "repair-team".into();
        let mut claimed = event("event-5", CaseEventKind::ResolutionClaimed, 1_400);
        claimed.actor_ref = "repair-team".into();
        case.events.extend([triaged, assigned, started, claimed]);
        case.updated_at_unix_ms = 1_400;
        assert!(case
            .validate()
            .unwrap_err()
            .contains(&CivicCaseValidationError::ResolutionClaimWithoutEvidence));
    }

    #[test]
    fn resolution_requires_an_independent_review_and_reopens_cleanly() {
        let mut case = valid_case();
        case.evidence = vec![
            independently_verified_evidence(),
            EvidenceReference {
                evidence_id: "repeat-photo-1".into(),
                source_kind: EvidenceSourceKind::ResidentSubmission,
                captured_at_unix_ms: 1_600,
                sha256: None,
                review: EvidenceReview::Unreviewed,
                visibility: EvidenceVisibility::Restricted,
                artifact_ref: Some("restricted-repeat-photo".into()),
                reviewer_ref: None,
                reviewed_at_unix_ms: None,
            },
        ];
        let mut triaged = event("event-2", CaseEventKind::Triaged, 1_100);
        triaged.actor_ref = "triage-team".into();
        let mut assigned = event("event-3", CaseEventKind::Assigned, 1_200);
        assigned.actor_ref = "dispatch-team".into();
        let mut started = event("event-4", CaseEventKind::WorkStarted, 1_300);
        started.actor_ref = "repair-team".into();
        let mut claimed = event("event-5", CaseEventKind::ResolutionClaimed, 1_400);
        claimed.actor_ref = "repair-team".into();
        claimed.evidence_ids = vec!["after-photo-1".into()];
        let mut verified = event("event-6", CaseEventKind::ResolutionVerified, 1_500);
        verified.actor_kind = CaseActorKind::IndependentReviewer;
        verified.actor_ref = "independent-reviewer-1".into();
        verified.evidence_ids = vec!["after-photo-1".into()];
        case.events.extend([triaged, assigned, started, claimed, verified]);
        case.updated_at_unix_ms = 1_500;
        assert!(case.validate().is_ok());
        assert_eq!(case.current_status(), Some(CaseStatus::VerifiedResolved));

        let mut reopened = event("event-7", CaseEventKind::Reopened, 1_700);
        reopened.actor_kind = CaseActorKind::CommunityPartner;
        reopened.actor_ref = "community-monitor-1".into();
        reopened.evidence_ids = vec!["repeat-photo-1".into()];
        case.events.push(reopened);
        case.updated_at_unix_ms = 1_700;
        assert!(case.validate().is_ok());
        assert_eq!(case.current_status(), Some(CaseStatus::Reopened));
    }

    #[test]
    fn verification_without_independent_evidence_fails() {
        let mut case = valid_case();
        let mut triaged = event("event-2", CaseEventKind::Triaged, 1_100);
        triaged.actor_ref = "triage-team".into();
        let mut assigned = event("event-3", CaseEventKind::Assigned, 1_200);
        assigned.actor_ref = "dispatch-team".into();
        let mut started = event("event-4", CaseEventKind::WorkStarted, 1_300);
        started.actor_ref = "repair-team".into();
        let mut claimed = event("event-5", CaseEventKind::ResolutionClaimed, 1_400);
        claimed.actor_ref = "repair-team".into();
        claimed.evidence_ids = vec!["evidence-1".into()];
        case.evidence = vec![EvidenceReference {
            evidence_id: "evidence-1".into(),
            source_kind: EvidenceSourceKind::ResidentSubmission,
            captured_at_unix_ms: 1_350,
            sha256: None,
            review: EvidenceReview::Unreviewed,
            visibility: EvidenceVisibility::Restricted,
            artifact_ref: None,
            reviewer_ref: None,
            reviewed_at_unix_ms: None,
        }];
        let mut verified = event("event-6", CaseEventKind::ResolutionVerified, 1_500);
        verified.actor_kind = CaseActorKind::ServiceOperator;
        verified.evidence_ids = vec!["evidence-1".into()];
        case.events.extend([triaged, assigned, started, claimed, verified]);
        case.updated_at_unix_ms = 1_500;
        assert!(case
            .validate()
            .unwrap_err()
            .contains(&CivicCaseValidationError::ResolutionVerificationNotIndependent));
    }

    #[test]
    fn correction_events_require_reason_and_evidence_and_do_not_change_status() {
        let mut case = valid_case();
        let mut correction = event("event-2", CaseEventKind::Correction, 1_100);
        correction.actor_ref = "data-steward-1".into();
        assert!(case
            .validate()
            .err()
            .is_some_and(|errors| {
                errors.contains(&CivicCaseValidationError::CorrectionWithoutReasonOrEvidence)
            }));
        correction.reason_code = Some("metadata.corrected".into());
        correction.evidence_ids = vec!["proof-1".into()];
        case.evidence.push(EvidenceReference {
            evidence_id: "proof-1".into(),
            source_kind: EvidenceSourceKind::MunicipalRecord,
            captured_at_unix_ms: 1_050,
            sha256: None,
            review: EvidenceReview::Corroborated,
            visibility: EvidenceVisibility::Restricted,
            artifact_ref: None,
            reviewer_ref: None,
            reviewed_at_unix_ms: None,
        });
        case.events.push(correction);
        case.updated_at_unix_ms = 1_100;
        assert!(case.validate().is_ok());
        assert_eq!(case.current_status(), Some(CaseStatus::Reported));
    }
}
