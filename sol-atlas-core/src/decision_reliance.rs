// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Reference semantics for time-qualified engineering decision reliance.
//!
//! This module is a research reference oracle, not a production approval or
//! authorization API. It folds proof-provider outcomes and bitemporal change
//! events into typed refusal states. It does not authenticate source bytes,
//! signatures, rights, actor credentials, or engineering-domain results.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

pub const DECISION_RELIANCE_SCHEMA_V01: &str =
    "luminous.engineering-decision-reliance-receipt.v0.1";
pub const REFERENCE_ORACLE_VERSION: &str = "decision-reliance-reference-oracle-v0.1";

#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize,
)]
#[serde(rename_all = "snake_case")]
pub enum DecisionRelianceProofClass {
    DecisionEventIntegrity,
    NativeSourceIdentity,
    SourcePayloadBytes,
    SourceUseRights,
    SourceAttestationTrust,
    SourceCoverage,
    ProjectionIntegrity,
    EngineeringDomainValidity,
    LifecycleCurrentness,
    ImpactFrontierCorrectness,
    ActorRelianceAuthority,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionRelianceProofStatus {
    Qualified,
    Stale,
    Missing,
    Unsupported,
    NotComparable,
    Invalid,
    Revoked,
    NotAuthorized,
    NotApplicable,
    RevalidationRequired,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionRelianceOutcome {
    ReliableForDeclaredScope,
    RevalidationRequired,
    InsufficientEvidence,
    NotAuthorized,
    InvalidEvidence,
}

impl DecisionRelianceOutcome {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ReliableForDeclaredScope => "reliable_for_declared_scope",
            Self::RevalidationRequired => "revalidation_required",
            Self::InsufficientEvidence => "insufficient_evidence",
            Self::NotAuthorized => "not_authorized",
            Self::InvalidEvidence => "invalid_evidence",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelianceValidTime {
    pub valid_from: Option<String>,
    pub valid_until: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelianceObservationTime {
    pub observed_at: Option<String>,
    pub recorded_at: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionRelianceProof {
    pub proof_id: String,
    pub proof_class: DecisionRelianceProofClass,
    pub status: DecisionRelianceProofStatus,
    pub subject_ref: String,
    pub subject_sha256: Option<String>,
    pub receipt_sha256: String,
    pub policy_sha256: Option<String>,
    pub verifier_build_sha256: Option<String>,
    pub checked_at: String,
    pub valid_time: RelianceValidTime,
    pub observation_time: RelianceObservationTime,
    pub trusted_at: Option<String>,
    pub fresh_until: Option<String>,
    pub revoked_at: Option<String>,
    pub finding_codes: Vec<String>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionRelianceChangeEventType {
    SourceCorrected,
    SourceSuperseded,
    RightsRevoked,
    KeyRevoked,
    CalibrationExpired,
    SchemaChanged,
    DependencyAdded,
    DecisionSuperseded,
    AuthorityExpired,
    SolverBuildRevoked,
    InspectionHoldAdded,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionRelianceEventTrustStatus {
    Qualified,
    Unverified,
    Invalid,
    Revoked,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionRelianceChangeEvent {
    pub event_id: String,
    pub event_type: DecisionRelianceChangeEventType,
    pub affected_proof_ids: Vec<String>,
    pub valid_at: String,
    pub observed_at: Option<String>,
    pub recorded_at: String,
    pub trusted_at: Option<String>,
    pub trust_status: DecisionRelianceEventTrustStatus,
    pub event_sha256: String,
    pub receipt_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionRelianceSubject {
    pub case_id: String,
    pub project_id: String,
    pub decision_event_id: String,
    pub decision_identity_sha256: String,
    pub decision_event_payload_sha256: String,
    pub ieac_graph_snapshot_sha256: String,
    pub projection_receipt_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionRelianceRequest {
    pub evaluating_actor_id: String,
    pub purpose: String,
    pub jurisdiction: String,
    pub scope_subjects: Vec<String>,
    pub decision_time: String,
    pub evaluation_time: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionRelianceFinding {
    pub severity: String,
    pub code: String,
    pub subject_ref: String,
    pub detail: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionRelianceEvaluator {
    pub profile_sha256: String,
    pub build_sha256: String,
    pub reference_oracle_version: String,
    pub evaluated_at: String,
}

/// Receipt wire model matching the Draft 2020-12 schema. The canonical receipt
/// digest is supplied by the #347 canonicalization/signing layer; this module
/// deliberately does not choose JCS versus deterministic CBOR.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionRelianceReceipt {
    pub schema_version: String,
    pub reliance_id: String,
    pub decision_subject: DecisionRelianceSubject,
    pub reliance_request: DecisionRelianceRequest,
    pub required_proof_classes: Vec<DecisionRelianceProofClass>,
    pub proofs: Vec<DecisionRelianceProof>,
    pub change_events: Vec<DecisionRelianceChangeEvent>,
    pub outcome: DecisionRelianceOutcome,
    pub findings: Vec<DecisionRelianceFinding>,
    pub evaluator: DecisionRelianceEvaluator,
    pub claim_ceiling: String,
    pub action_authority: String,
    pub canonicalization_profile: String,
    pub receipt_sha256: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct DecisionRelianceReferenceResult {
    pub outcome: DecisionRelianceOutcome,
    pub findings: Vec<DecisionRelianceFinding>,
    pub qualified_proof_count: usize,
    pub required_proof_class_count: usize,
    pub missing_proof_class_count: usize,
    pub known_change_event_count: usize,
    pub late_arriving_change_event_count: usize,
    pub future_or_untrusted_change_event_count: usize,
    pub unresolved_proof_count: usize,
    pub action_authority: String,
}

#[derive(Default)]
struct OutcomeFlags {
    invalid: bool,
    not_authorized: bool,
    revalidation: bool,
    insufficient: bool,
}

impl OutcomeFlags {
    fn outcome(&self) -> DecisionRelianceOutcome {
        if self.invalid {
            DecisionRelianceOutcome::InvalidEvidence
        } else if self.not_authorized {
            DecisionRelianceOutcome::NotAuthorized
        } else if self.revalidation {
            DecisionRelianceOutcome::RevalidationRequired
        } else if self.insufficient {
            DecisionRelianceOutcome::InsufficientEvidence
        } else {
            DecisionRelianceOutcome::ReliableForDeclaredScope
        }
    }
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn valid_utc_timestamp(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 20
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || bytes[10] != b'T'
        || bytes[13] != b':'
        || bytes[16] != b':'
        || bytes[19] != b'Z'
    {
        return false;
    }
    for (index, byte) in bytes.iter().enumerate() {
        if !matches!(index, 4 | 7 | 10 | 13 | 16 | 19) && !byte.is_ascii_digit() {
            return false;
        }
    }
    let part = |start: usize, end: usize| {
        std::str::from_utf8(&bytes[start..end])
            .ok()
            .and_then(|value| value.parse::<u32>().ok())
    };
    let year = part(0, 4).unwrap_or(0);
    let month = part(5, 7).unwrap_or(0);
    let day = part(8, 10).unwrap_or(0);
    let hour = part(11, 13).unwrap_or(99);
    let minute = part(14, 16).unwrap_or(99);
    let second = part(17, 19).unwrap_or(99);
    if year == 0 || !(1..=12).contains(&month) || hour > 23 || minute > 59 || second > 59 {
        return false;
    }
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let last_day = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => 0,
    };
    (1..=last_day).contains(&day)
}

fn push_finding(
    findings: &mut BTreeMap<(String, String, String), DecisionRelianceFinding>,
    severity: &str,
    code: &str,
    subject_ref: &str,
    detail: &str,
) {
    findings.insert(
        (
            code.to_owned(),
            subject_ref.to_owned(),
            detail.to_owned(),
        ),
        DecisionRelianceFinding {
            severity: severity.to_owned(),
            code: code.to_owned(),
            subject_ref: subject_ref.to_owned(),
            detail: detail.to_owned(),
        },
    );
}

fn mark_proof_status(
    proof: &DecisionRelianceProof,
    flags: &mut OutcomeFlags,
    findings: &mut BTreeMap<(String, String, String), DecisionRelianceFinding>,
) {
    let subject = &proof.subject_ref;
    match proof.status {
        DecisionRelianceProofStatus::Qualified => {}
        DecisionRelianceProofStatus::Invalid => {
            flags.invalid = true;
            push_finding(
                findings,
                "invalid",
                "PROOF_INVALID",
                subject,
                "a required proof provider marked this evidence invalid",
            );
        }
        DecisionRelianceProofStatus::Revoked => {
            if matches!(
                proof.proof_class,
                DecisionRelianceProofClass::SourceUseRights
                    | DecisionRelianceProofClass::ActorRelianceAuthority
            ) {
                flags.not_authorized = true;
                push_finding(
                    findings,
                    "not_authorized",
                    "RIGHT_OR_AUTHORITY_PROOF_REVOKED",
                    subject,
                    "a required rights or actor-authority proof was revoked",
                );
            } else {
                flags.invalid = true;
                push_finding(
                    findings,
                    "invalid",
                    "PROOF_REVOKED",
                    subject,
                    "a required proof was revoked",
                );
            }
        }
        DecisionRelianceProofStatus::NotAuthorized => {
            flags.not_authorized = true;
            push_finding(
                findings,
                "not_authorized",
                "PROOF_NOT_AUTHORIZED",
                subject,
                "a required authority or rights proof does not authorize this reliance request",
            );
        }
        DecisionRelianceProofStatus::Stale
        | DecisionRelianceProofStatus::RevalidationRequired => {
            flags.revalidation = true;
            push_finding(
                findings,
                "revalidation",
                "PROOF_STALE_OR_REQUIRES_REVALIDATION",
                subject,
                "a required proof is no longer current for the requested reliance time",
            );
        }
        DecisionRelianceProofStatus::Missing
        | DecisionRelianceProofStatus::Unsupported
        | DecisionRelianceProofStatus::NotComparable
        | DecisionRelianceProofStatus::NotApplicable => {
            flags.insufficient = true;
            push_finding(
                findings,
                "incomplete",
                "PROOF_MISSING_UNSUPPORTED_OR_NOT_COMPARABLE",
                subject,
                "a required proof is missing, unsupported, not comparable, or not applicable to this declared scope",
            );
        }
    }
}

fn validate_proof_temporal_state(
    proof: &DecisionRelianceProof,
    evaluation_time: &str,
    flags: &mut OutcomeFlags,
    findings: &mut BTreeMap<(String, String, String), DecisionRelianceFinding>,
) {
    let subject = &proof.subject_ref;
    let timestamps = [
        ("checked_at", proof.checked_at.as_str()),
        ("recorded_at", proof.observation_time.recorded_at.as_str()),
    ];
    for (field, timestamp) in timestamps {
        if !valid_utc_timestamp(timestamp) {
            flags.invalid = true;
            push_finding(
                findings,
                "invalid",
                "INVALID_PROOF_TIMESTAMP",
                subject,
                &format!("{field} is not canonical UTC whole-second time"),
            );
        }
    }
    for (field, timestamp) in [
        ("valid_from", proof.valid_time.valid_from.as_deref()),
        ("valid_until", proof.valid_time.valid_until.as_deref()),
        ("observed_at", proof.observation_time.observed_at.as_deref()),
        ("trusted_at", proof.trusted_at.as_deref()),
        ("fresh_until", proof.fresh_until.as_deref()),
        ("revoked_at", proof.revoked_at.as_deref()),
    ] {
        if timestamp.is_some_and(|value| !valid_utc_timestamp(value)) {
            flags.invalid = true;
            push_finding(
                findings,
                "invalid",
                "INVALID_PROOF_TIMESTAMP",
                subject,
                &format!("{field} is not canonical UTC whole-second time"),
            );
        }
    }

    if proof.valid_time.valid_from.as_deref().zip(proof.valid_time.valid_until.as_deref())
        .is_some_and(|(from, until)| from >= until)
    {
        flags.invalid = true;
        push_finding(
            findings,
            "invalid",
            "INVALID_PROOF_VALID_TIME_INTERVAL",
            subject,
            "proof valid_until must be later than valid_from",
        );
    }
    if proof
        .observation_time
        .observed_at
        .as_deref()
        .is_some_and(|observed| observed > proof.observation_time.recorded_at.as_str())
    {
        flags.insufficient = true;
        push_finding(
            findings,
            "incomplete",
            "OBSERVATION_AFTER_RECORDING",
            subject,
            "source observation time is later than the system-recorded time",
        );
    }
    if proof.observation_time.recorded_at.as_str() > evaluation_time
        || proof.checked_at.as_str() > evaluation_time
        || proof.trusted_at.as_deref().is_some_and(|time| time > evaluation_time)
    {
        flags.insufficient = true;
        push_finding(
            findings,
            "incomplete",
            "PROOF_NOT_KNOWN_AT_EVALUATION_TIME",
            subject,
            "proof recording, checking, or trusted registration is later than the requested evaluation time",
        );
    }
    if proof
        .valid_time
        .valid_from
        .as_deref()
        .is_some_and(|valid_from| valid_from > evaluation_time)
    {
        flags.insufficient = true;
        push_finding(
            findings,
            "incomplete",
            "PROOF_NOT_YET_VALID",
            subject,
            "proof valid time begins after the reliance evaluation time",
        );
    }
    if proof
        .valid_time
        .valid_until
        .as_deref()
        .is_some_and(|valid_until| valid_until <= evaluation_time)
        || proof
            .fresh_until
            .as_deref()
            .is_some_and(|fresh_until| fresh_until <= evaluation_time)
    {
        if matches!(
            proof.proof_class,
            DecisionRelianceProofClass::SourceUseRights
                | DecisionRelianceProofClass::ActorRelianceAuthority
        ) {
            flags.not_authorized = true;
            push_finding(
                findings,
                "not_authorized",
                "RIGHT_OR_AUTHORITY_EXPIRED",
                subject,
                "rights or actor authority are no longer valid at the requested evaluation time",
            );
        } else {
            flags.revalidation = true;
            push_finding(
                findings,
                "revalidation",
                "PROOF_VALIDITY_OR_FRESHNESS_EXPIRED",
                subject,
                "a required proof validity or freshness interval has expired",
            );
        }
    }
    if proof.revoked_at.as_deref().is_some_and(|revoked_at| revoked_at <= evaluation_time) {
        if matches!(
            proof.proof_class,
            DecisionRelianceProofClass::SourceUseRights
                | DecisionRelianceProofClass::ActorRelianceAuthority
        ) {
            flags.not_authorized = true;
            push_finding(
                findings,
                "not_authorized",
                "RIGHT_OR_AUTHORITY_REVOKED",
                subject,
                "rights or actor authority were revoked before the requested evaluation time",
            );
        } else {
            flags.invalid = true;
            push_finding(
                findings,
                "invalid",
                "PROOF_REVOKED",
                subject,
                "a required proof was revoked before the requested evaluation time",
            );
        }
    }
}

fn event_effect(
    event: &DecisionRelianceChangeEvent,
    decision_time: &str,
    flags: &mut OutcomeFlags,
    findings: &mut BTreeMap<(String, String, String), DecisionRelianceFinding>,
) {
    let subject = &event.event_id;
    let late_arriving = event.valid_at.as_str() <= decision_time
        && event.recorded_at.as_str() > decision_time;
    if late_arriving {
        push_finding(
            findings,
            "revalidation",
            "LATE_ARRIVING_RETROACTIVE_CHANGE",
            subject,
            "the change was recorded after the decision but has a valid time at or before the decision time",
        );
    }

    match event.event_type {
        DecisionRelianceChangeEventType::RightsRevoked
        | DecisionRelianceChangeEventType::AuthorityExpired => {
            flags.not_authorized = true;
            push_finding(
                findings,
                "not_authorized",
                "RIGHT_OR_AUTHORITY_CHANGE",
                subject,
                "a trusted change event removed rights or actor authority for this reliance request",
            );
        }
        DecisionRelianceChangeEventType::KeyRevoked => {
            if event.valid_at.as_str() <= decision_time {
                flags.invalid = true;
                push_finding(
                    findings,
                    "invalid",
                    "KEY_REVOKED_BEFORE_DECISION",
                    subject,
                    "the signing key was revoked with effect at or before the historical decision time",
                );
            } else {
                flags.revalidation = true;
                push_finding(
                    findings,
                    "revalidation",
                    "KEY_REVOKED_AFTER_DECISION",
                    subject,
                    "the signing key was revoked after the historical decision; current reliance requires trust re-evaluation",
                );
            }
        }
        DecisionRelianceChangeEventType::SourceCorrected
        | DecisionRelianceChangeEventType::SourceSuperseded
        | DecisionRelianceChangeEventType::CalibrationExpired
        | DecisionRelianceChangeEventType::SchemaChanged
        | DecisionRelianceChangeEventType::DependencyAdded
        | DecisionRelianceChangeEventType::DecisionSuperseded
        | DecisionRelianceChangeEventType::SolverBuildRevoked
        | DecisionRelianceChangeEventType::InspectionHoldAdded => {
            flags.revalidation = true;
            push_finding(
                findings,
                "revalidation",
                "KNOWN_CHANGE_INVALIDATES_RELIANCE",
                subject,
                "a trusted source, lifecycle, schema, dependency, solver, or inspection event requires downstream revalidation",
            );
        }
    }
}

pub fn evaluate_decision_reliance_reference(
    decision_time: &str,
    evaluation_time: &str,
    required_proof_classes: &[DecisionRelianceProofClass],
    proofs: &[DecisionRelianceProof],
    change_events: &[DecisionRelianceChangeEvent],
) -> DecisionRelianceReferenceResult {
    let mut flags = OutcomeFlags::default();
    let mut findings = BTreeMap::new();
    let mut proof_ids = BTreeSet::new();
    let mut event_ids = BTreeSet::new();
    let mut required = BTreeSet::new();
    let mut present_classes = BTreeSet::new();

    if !valid_utc_timestamp(decision_time)
        || !valid_utc_timestamp(evaluation_time)
        || decision_time > evaluation_time
    {
        flags.invalid = true;
        push_finding(
            &mut findings,
            "invalid",
            "INVALID_RELIANCE_TIME_ORDER",
            "reliance_request",
            "decision and evaluation times must be canonical UTC whole-second timestamps with evaluation at or after decision time",
        );
    }

    if required_proof_classes.is_empty() {
        flags.invalid = true;
        push_finding(
            &mut findings,
            "invalid",
            "EMPTY_REQUIRED_PROOF_SET",
            "reliance_request",
            "at least one mandatory proof class must be declared for a reliance assessment",
        );
    }
    for proof_class in required_proof_classes {
        if !required.insert(*proof_class) {
            flags.invalid = true;
            push_finding(
                &mut findings,
                "invalid",
                "DUPLICATE_REQUIRED_PROOF_CLASS",
                "reliance_request",
                "required proof classes must be unique",
            );
        }
    }

    let mut qualified_proof_count = 0usize;
    let mut unresolved_proof_count = 0usize;
    for proof in proofs {
        if proof.proof_id.trim().is_empty() || proof.subject_ref.trim().is_empty() {
            flags.invalid = true;
            push_finding(
                &mut findings,
                "invalid",
                "EMPTY_PROOF_IDENTITY",
                &proof.subject_ref,
                "proof_id and subject_ref must be non-empty",
            );
        }
        for (field, digest) in [
            ("receipt_sha256", Some(proof.receipt_sha256.as_str())),
            ("subject_sha256", proof.subject_sha256.as_deref()),
            ("policy_sha256", proof.policy_sha256.as_deref()),
            ("verifier_build_sha256", proof.verifier_build_sha256.as_deref()),
        ] {
            if digest.is_some_and(|value| !valid_sha256(value)) {
                flags.invalid = true;
                push_finding(
                    &mut findings,
                    "invalid",
                    "INVALID_PROOF_DIGEST",
                    &proof.subject_ref,
                    &format!("{field} is not lowercase SHA-256"),
                );
            }
        }
        if !proof_ids.insert(proof.proof_id.as_str()) {
            flags.invalid = true;
            push_finding(
                &mut findings,
                "invalid",
                "DUPLICATE_PROOF_ID",
                &proof.subject_ref,
                "proof IDs must be unique within one decision-reliance receipt",
            );
        }
        present_classes.insert(proof.proof_class);
        mark_proof_status(proof, &mut flags, &mut findings);
        if proof.status == DecisionRelianceProofStatus::Qualified {
            qualified_proof_count += 1;
            validate_proof_temporal_state(
                proof,
                evaluation_time,
                &mut flags,
                &mut findings,
            );
        } else if required.contains(&proof.proof_class) {
            unresolved_proof_count += 1;
        }
    }

    let missing_proof_classes = required
        .difference(&present_classes)
        .copied()
        .collect::<Vec<_>>();
    for missing in &missing_proof_classes {
        flags.insufficient = true;
        push_finding(
            &mut findings,
            "incomplete",
            "REQUIRED_PROOF_CLASS_MISSING",
            "reliance_request",
            &format!("required proof class {missing:?} has no proof result"),
        );
    }

    let mut known_change_event_count = 0usize;
    let mut late_arriving_change_event_count = 0usize;
    let mut future_or_untrusted_change_event_count = 0usize;
    for event in change_events {
        let unique_affected = event
            .affected_proof_ids
            .iter()
            .collect::<BTreeSet<_>>();
        if event.event_id.trim().is_empty()
            || unique_affected.len() != event.affected_proof_ids.len()
        {
            flags.invalid = true;
            push_finding(
                &mut findings,
                "invalid",
                "INVALID_CHANGE_EVENT_IDENTITY",
                &event.event_id,
                "event ID must be non-empty and affected proof IDs must be unique",
            );
        }
        for (field, digest) in [
            ("event_sha256", event.event_sha256.as_str()),
            ("receipt_sha256", event.receipt_sha256.as_str()),
        ] {
            if !valid_sha256(digest) {
                flags.invalid = true;
                push_finding(
                    &mut findings,
                    "invalid",
                    "INVALID_CHANGE_EVENT_DIGEST",
                    &event.event_id,
                    &format!("{field} is not lowercase SHA-256"),
                );
            }
        }
        if event.affected_proof_ids.is_empty() {
            flags.invalid = true;
            push_finding(
                &mut findings,
                "invalid",
                "CHANGE_EVENT_WITHOUT_AFFECTED_PROOF",
                &event.event_id,
                "change event must identify at least one affected proof",
            );
        }
        if !event_ids.insert(event.event_id.as_str()) {
            flags.invalid = true;
            push_finding(
                &mut findings,
                "invalid",
                &event.event_id,
                "DUPLICATE_CHANGE_EVENT_ID",
                "change-event IDs must be unique within one decision-reliance receipt",
            );
        }

        for (field, value) in [
            ("valid_at", event.valid_at.as_str()),
            ("recorded_at", event.recorded_at.as_str()),
        ] {
            if !valid_utc_timestamp(value) {
                flags.invalid = true;
                push_finding(
                    &mut findings,
                    "invalid",
                    &event.event_id,
                    "INVALID_CHANGE_EVENT_TIME",
                    &format!("{field} is not canonical UTC whole-second time"),
                );
            }
        }
        if event
            .observed_at
            .as_deref()
            .is_some_and(|time| !valid_utc_timestamp(time))
            || event
                .trusted_at
                .as_deref()
                .is_some_and(|time| !valid_utc_timestamp(time))
        {
            flags.invalid = true;
            push_finding(
                &mut findings,
                "invalid",
                &event.event_id,
                "INVALID_CHANGE_EVENT_TIME",
                "observed_at or trusted_at is not canonical UTC whole-second time",
            );
        }

        if event.recorded_at.as_str() > evaluation_time
            || event
                .trusted_at
                .as_deref()
                .is_some_and(|trusted_at| trusted_at > evaluation_time)
            || event.valid_at.as_str() > evaluation_time
        {
            future_or_untrusted_change_event_count += 1;
            push_finding(
                &mut findings,
                "info",
                "CHANGE_EVENT_NOT_EFFECTIVE_OR_KNOWN_AT_EVALUATION",
                &event.event_id,
                "a future or not-yet-registered change event is not applied to this historical query",
            );
            continue;
        }

        if event.observed_at.as_deref().is_some_and(|observed| observed > event.recorded_at.as_str()) {
            flags.insufficient = true;
            push_finding(
                &mut findings,
                "incomplete",
                "CHANGE_EVENT_OBSERVED_AFTER_RECORDING",
                &event.event_id,
                "change-event observation time is later than its recorded time",
            );
        }

        match event.trust_status {
            DecisionRelianceEventTrustStatus::Invalid
            | DecisionRelianceEventTrustStatus::Revoked => {
                flags.invalid = true;
                push_finding(
                    &mut findings,
                    "invalid",
                    &event.event_id,
                    "CHANGE_EVENT_TRUST_INVALID",
                    "change event provenance or registration is invalid or revoked",
                );
                continue;
            }
            DecisionRelianceEventTrustStatus::Unverified => {
                flags.insufficient = true;
                future_or_untrusted_change_event_count += 1;
                push_finding(
                    &mut findings,
                    "incomplete",
                    &event.event_id,
                    "CHANGE_EVENT_NOT_AUTHENTICATED",
                    "an unverified change event cannot be used as a trusted invalidation event",
                );
                continue;
            }
            DecisionRelianceEventTrustStatus::Qualified => {
                if event.trusted_at.is_none() {
                    flags.insufficient = true;
                    future_or_untrusted_change_event_count += 1;
                    push_finding(
                        &mut findings,
                        "incomplete",
                        &event.event_id,
                        "CHANGE_EVENT_TRUST_TIME_MISSING",
                        "trusted change event has no trusted registration time",
                    );
                    continue;
                }
            }
        }

        let unlisted = event
            .affected_proof_ids
            .iter()
            .filter(|proof_id| !proof_ids.contains(proof_id.as_str()))
            .collect::<Vec<_>>();
        if !unlisted.is_empty() {
            flags.insufficient = true;
            push_finding(
                &mut findings,
                "incomplete",
                &event.event_id,
                "CHANGE_EVENT_REFERENCES_UNLISTED_PROOF",
                "change event affects proof IDs not represented in this reliance receipt",
            );
        }

        known_change_event_count += 1;
        if event.valid_at.as_str() <= decision_time && event.recorded_at.as_str() > decision_time {
            late_arriving_change_event_count += 1;
        }
        event_effect(event, decision_time, &mut flags, &mut findings);
    }

    let outcome = flags.outcome();
    DecisionRelianceReferenceResult {
        outcome,
        findings: findings.into_values().collect(),
        qualified_proof_count,
        required_proof_class_count: required.len(),
        missing_proof_class_count: missing_proof_classes.len(),
        known_change_event_count,
        late_arriving_change_event_count,
        future_or_untrusted_change_event_count,
        unresolved_proof_count,
        action_authority: "none".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DECISION_TIME: &str = "2026-10-01T10:00:00Z";
    const EVALUATION_TIME: &str = "2026-11-04T10:00:00Z";

    fn proof(proof_id: &str, proof_class: DecisionRelianceProofClass) -> DecisionRelianceProof {
        DecisionRelianceProof {
            proof_id: proof_id.into(),
            proof_class,
            status: DecisionRelianceProofStatus::Qualified,
            subject_ref: format!("urn:synthetic:{proof_id}"),
            subject_sha256: Some("a".repeat(64)),
            receipt_sha256: "b".repeat(64),
            policy_sha256: Some("c".repeat(64)),
            verifier_build_sha256: Some("d".repeat(64)),
            checked_at: "2026-11-04T09:00:00Z".into(),
            valid_time: RelianceValidTime {
                valid_from: Some("2026-09-01T00:00:00Z".into()),
                valid_until: Some("2026-12-01T00:00:00Z".into()),
            },
            observation_time: RelianceObservationTime {
                observed_at: Some("2026-10-01T09:00:00Z".into()),
                recorded_at: "2026-10-01T10:00:00Z".into(),
            },
            trusted_at: Some("2026-10-01T10:05:00Z".into()),
            fresh_until: Some("2026-12-01T00:00:00Z".into()),
            revoked_at: None,
            finding_codes: Vec::new(),
        }
    }

    fn event(
        event_id: &str,
        event_type: DecisionRelianceChangeEventType,
        proof_id: &str,
        valid_at: &str,
        recorded_at: &str,
    ) -> DecisionRelianceChangeEvent {
        DecisionRelianceChangeEvent {
            event_id: event_id.into(),
            event_type,
            affected_proof_ids: vec![proof_id.into()],
            valid_at: valid_at.into(),
            observed_at: Some(recorded_at.into()),
            recorded_at: recorded_at.into(),
            trusted_at: Some(recorded_at.into()),
            trust_status: DecisionRelianceEventTrustStatus::Qualified,
            event_sha256: "e".repeat(64),
            receipt_sha256: "f".repeat(64),
        }
    }

    fn base_proofs() -> Vec<DecisionRelianceProof> {
        vec![
            proof("decision-event", DecisionRelianceProofClass::DecisionEventIntegrity),
            proof("source-bytes", DecisionRelianceProofClass::SourcePayloadBytes),
            proof("coverage", DecisionRelianceProofClass::SourceCoverage),
            proof("actor-authority", DecisionRelianceProofClass::ActorRelianceAuthority),
        ]
    }

    fn required() -> Vec<DecisionRelianceProofClass> {
        vec![
            DecisionRelianceProofClass::DecisionEventIntegrity,
            DecisionRelianceProofClass::SourcePayloadBytes,
            DecisionRelianceProofClass::SourceCoverage,
            DecisionRelianceProofClass::ActorRelianceAuthority,
        ]
    }

    #[derive(Deserialize)]
    struct SyntheticTemporalCorpus {
        schema_version: String,
        scenarios: Vec<SyntheticRelianceScenario>,
    }

    #[derive(Deserialize)]
    struct SyntheticRelianceScenario {
        scenario_id: String,
        decision_time: String,
        evaluation_time: String,
        required_proof_classes: Vec<DecisionRelianceProofClass>,
        proofs: Vec<DecisionRelianceProof>,
        change_events: Vec<DecisionRelianceChangeEvent>,
        expected_outcome: DecisionRelianceOutcome,
        expected_late_arriving_events: Option<usize>,
    }

    #[test]
    fn frozen_synthetic_temporal_corpus_matches_independent_reference_oracle() {
        let corpus: SyntheticTemporalCorpus = serde_json::from_str(include_str!(
            "../../schemas/examples/decision-reliance-temporal-corpus-synthetic-v0.1.json"
        ))
        .expect("frozen temporal corpus must deserialize");
        assert_eq!(
            corpus.schema_version,
            "luminous.decision-reliance-temporal-corpus.v0.1"
        );
        assert_eq!(corpus.scenarios.len(), 8);

        for scenario in corpus.scenarios {
            let result = evaluate_decision_reliance_reference(
                &scenario.decision_time,
                &scenario.evaluation_time,
                &scenario.required_proof_classes,
                &scenario.proofs,
                &scenario.change_events,
            );
            assert_eq!(
                result.outcome,
                scenario.expected_outcome,
                "scenario {}",
                scenario.scenario_id
            );
            if let Some(expected_late_arriving_events) = scenario.expected_late_arriving_events {
                assert_eq!(
                    result.late_arriving_change_event_count,
                    expected_late_arriving_events,
                    "scenario {}",
                    scenario.scenario_id
                );
            }
        }
    }

    #[test]
    fn all_required_current_proofs_allow_reliance_only_for_declared_scope() {
        let result = evaluate_decision_reliance_reference(
            DECISION_TIME,
            EVALUATION_TIME,
            &required(),
            &base_proofs(),
            &[],
        );
        assert_eq!(
            result.outcome,
            DecisionRelianceOutcome::ReliableForDeclaredScope
        );
        assert_eq!(result.missing_proof_class_count, 0);
        assert_eq!(result.unresolved_proof_count, 0);
        assert_eq!(result.action_authority, "none");
    }

    #[test]
    fn missing_required_proof_is_insufficient_not_false_or_reliable() {
        let mut proofs = base_proofs();
        proofs.retain(|proof| proof.proof_class != DecisionRelianceProofClass::SourceCoverage);
        let result = evaluate_decision_reliance_reference(
            DECISION_TIME,
            EVALUATION_TIME,
            &required(),
            &proofs,
            &[],
        );
        assert_eq!(
            result.outcome,
            DecisionRelianceOutcome::InsufficientEvidence
        );
        assert_eq!(result.missing_proof_class_count, 1);
    }

    #[test]
    fn expired_proof_requires_revalidation() {
        let mut proofs = base_proofs();
        proofs[1].fresh_until = Some("2026-11-04T09:59:59Z".into());
        let result = evaluate_decision_reliance_reference(
            DECISION_TIME,
            EVALUATION_TIME,
            &required(),
            &proofs,
            &[],
        );
        assert_eq!(result.outcome, DecisionRelianceOutcome::RevalidationRequired);
        assert!(result.findings.iter().any(|finding| {
            finding.code == "PROOF_VALIDITY_OR_FRESHNESS_EXPIRED"
        }));
    }

    #[test]
    fn late_arriving_retroactive_source_correction_requires_revalidation() {
        let event = event(
            "late-source-correction",
            DecisionRelianceChangeEventType::SourceCorrected,
            "source-bytes",
            "2026-09-20T00:00:00Z",
            "2026-11-03T12:00:00Z",
        );
        let result = evaluate_decision_reliance_reference(
            DECISION_TIME,
            EVALUATION_TIME,
            &required(),
            &base_proofs(),
            &[event],
        );
        assert_eq!(result.outcome, DecisionRelianceOutcome::RevalidationRequired);
        assert_eq!(result.late_arriving_change_event_count, 1);
        assert!(result.findings.iter().any(|finding| {
            finding.code == "LATE_ARRIVING_RETROACTIVE_CHANGE"
        }));
    }

    #[test]
    fn key_revoked_before_historical_decision_is_invalid_evidence() {
        let event = event(
            "compromised-key-before-decision",
            DecisionRelianceChangeEventType::KeyRevoked,
            "source-bytes",
            "2026-09-25T00:00:00Z",
            "2026-09-26T00:00:00Z",
        );
        let result = evaluate_decision_reliance_reference(
            DECISION_TIME,
            EVALUATION_TIME,
            &required(),
            &base_proofs(),
            &[event],
        );
        assert_eq!(result.outcome, DecisionRelianceOutcome::InvalidEvidence);
    }

    #[test]
    fn key_revoked_after_historical_decision_requires_current_revalidation() {
        let event = event(
            "compromised-key-after-decision",
            DecisionRelianceChangeEventType::KeyRevoked,
            "source-bytes",
            "2026-11-01T00:00:00Z",
            "2026-11-02T00:00:00Z",
        );
        let result = evaluate_decision_reliance_reference(
            DECISION_TIME,
            EVALUATION_TIME,
            &required(),
            &base_proofs(),
            &[event],
        );
        assert_eq!(result.outcome, DecisionRelianceOutcome::RevalidationRequired);
    }

    #[test]
    fn rights_revocation_is_not_authorized_not_just_stale() {
        let event = event(
            "rights-revoked",
            DecisionRelianceChangeEventType::RightsRevoked,
            "source-bytes",
            "2026-11-01T00:00:00Z",
            "2026-11-02T00:00:00Z",
        );
        let result = evaluate_decision_reliance_reference(
            DECISION_TIME,
            EVALUATION_TIME,
            &required(),
            &base_proofs(),
            &[event],
        );
        assert_eq!(result.outcome, DecisionRelianceOutcome::NotAuthorized);
    }

    #[test]
    fn unverified_change_event_cannot_invalidate_or_restore_reliance() {
        let mut event = event(
            "untrusted-correction",
            DecisionRelianceChangeEventType::SourceCorrected,
            "source-bytes",
            "2026-11-01T00:00:00Z",
            "2026-11-02T00:00:00Z",
        );
        event.trust_status = DecisionRelianceEventTrustStatus::Unverified;
        let result = evaluate_decision_reliance_reference(
            DECISION_TIME,
            EVALUATION_TIME,
            &required(),
            &base_proofs(),
            &[event],
        );
        assert_eq!(
            result.outcome,
            DecisionRelianceOutcome::InsufficientEvidence
        );
        assert_eq!(result.known_change_event_count, 0);
        assert_eq!(result.future_or_untrusted_change_event_count, 1);
    }

    #[test]
    fn future_registered_change_event_is_not_leaked_into_historical_query() {
        let event = event(
            "future-correction",
            DecisionRelianceChangeEventType::SourceCorrected,
            "source-bytes",
            "2026-11-05T00:00:00Z",
            "2026-11-05T00:00:01Z",
        );
        let result = evaluate_decision_reliance_reference(
            DECISION_TIME,
            EVALUATION_TIME,
            &required(),
            &base_proofs(),
            &[event],
        );
        assert_eq!(
            result.outcome,
            DecisionRelianceOutcome::ReliableForDeclaredScope
        );
        assert_eq!(result.known_change_event_count, 0);
        assert_eq!(result.future_or_untrusted_change_event_count, 1);
    }

    #[test]
    fn invalid_evidence_outweighs_missing_evidence_in_outcome_precedence() {
        let mut proofs = base_proofs();
        proofs[0].status = DecisionRelianceProofStatus::Invalid;
        proofs.remove(1);
        let result = evaluate_decision_reliance_reference(
            DECISION_TIME,
            EVALUATION_TIME,
            &required(),
            &proofs,
            &[],
        );
        assert_eq!(result.outcome, DecisionRelianceOutcome::InvalidEvidence);
        assert_eq!(result.missing_proof_class_count, 1);
    }

    #[test]
    fn expired_actor_authority_is_not_authorized() {
        let mut proofs = base_proofs();
        let authority = proofs
            .iter_mut()
            .find(|proof| proof.proof_class == DecisionRelianceProofClass::ActorRelianceAuthority)
            .unwrap();
        authority.valid_time.valid_until = Some("2026-11-04T09:00:00Z".into());
        let result = evaluate_decision_reliance_reference(
            DECISION_TIME,
            EVALUATION_TIME,
            &required(),
            &proofs,
            &[],
        );
        assert_eq!(result.outcome, DecisionRelianceOutcome::NotAuthorized);
    }
}
