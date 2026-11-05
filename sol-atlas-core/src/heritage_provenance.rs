// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Content-addressed heritage provenance, purpose-specific rights and fixity.
//!
//! A valid manifest proves only that its declared references and relationships
//! are structurally coherent. SHA-256 equality proves byte equality with the
//! declared digest; it does not prove authorship, custody, legal authority or
//! historical truth. External signatures and trust roots are not implemented.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, VecDeque};
use std::fmt;

pub const HERITAGE_PROVENANCE_SCHEMA_VERSION: &str = "luminous.heritage-provenance.v0.1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactRole {
    PrimaryCapture,
    ImportedDataset,
    DerivedImage,
    PointCloud,
    Geometry,
    Annotation,
    Adjudication,
    ModelOutput,
    SimulationOutput,
    ClaimRecord,
    Reconstruction,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactOrigin {
    Primary,
    Derived,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AccessTier {
    Public,
    Controlled,
    Restricted,
    Sealed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentKind {
    Person,
    Organization,
    Software,
    Instrument,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ActivityKind {
    Acquisition,
    Ingest,
    Transformation,
    Annotation,
    Adjudication,
    ModelInference,
    Simulation,
    Preservation,
    Publication,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RightsPurpose {
    Preservation,
    ResearchAnalysis,
    Annotation,
    ModelTraining,
    PublicDisplay,
    Redistribution,
    CommercialUse,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RightsStatus {
    Allowed,
    Restricted,
    Prohibited,
    Undetermined,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FixityOutcome {
    Matched,
    Mismatch,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProvenanceArtifact {
    pub artifact_id: String,
    pub sha256: String,
    pub byte_length: u64,
    pub media_type: String,
    pub role: ArtifactRole,
    pub origin: ArtifactOrigin,
    pub access_tier: AccessTier,
    pub rights_decision_id: String,
    /// Required for derived artifacts; absent for primary captures.
    pub generated_by_activity_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProvenanceAgent {
    pub agent_id: String,
    pub kind: AgentKind,
    /// Opaque URI or identifier; identity is not authenticated by this module.
    pub external_reference: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProvenanceActivity {
    pub activity_id: String,
    pub kind: ActivityKind,
    pub purpose: RightsPurpose,
    pub input_artifact_ids: Vec<String>,
    pub generated_artifact_ids: Vec<String>,
    pub responsible_agent_ids: Vec<String>,
    /// Required for Transformation, ModelInference and Simulation.
    pub software_agent_id: Option<String>,
    pub software_build_sha256: Option<String>,
    pub environment_sha256: Option<String>,
    pub parameters_sha256: Option<String>,
    pub protocol_sha256: Option<String>,
    /// Canonical UTC RFC 3339 timestamps (Z suffix).
    pub started_at_utc: String,
    pub ended_at_utc: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactRightsDecision {
    pub decision_id: String,
    pub artifact_id: String,
    pub status: RightsStatus,
    pub allowed_purposes: Vec<RightsPurpose>,
    pub prohibited_purposes: Vec<RightsPurpose>,
    pub decision_receipt_ref: Option<String>,
    pub decision_receipt_sha256: Option<String>,
    pub reviewed_by_agent_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FixityEvent {
    pub event_id: String,
    pub artifact_id: String,
    pub expected_sha256: String,
    pub observed_sha256: String,
    pub observed_byte_length: u64,
    pub outcome: FixityOutcome,
    pub checked_at_utc: String,
    pub checked_by_agent_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeritageProvenanceManifest {
    pub schema_version: String,
    pub artifacts: Vec<ProvenanceArtifact>,
    pub agents: Vec<ProvenanceAgent>,
    pub activities: Vec<ProvenanceActivity>,
    pub rights_decisions: Vec<ArtifactRightsDecision>,
    pub fixity_events: Vec<FixityEvent>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ByteFixityCheck {
    pub artifact_id: String,
    pub expected_sha256: String,
    pub observed_sha256: String,
    pub expected_byte_length: u64,
    pub observed_byte_length: u64,
    pub length_matched: bool,
    pub matched: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HeritageProvenanceAuditReport {
    pub schema_version: String,
    pub artifact_count: usize,
    pub primary_artifact_count: usize,
    pub derived_artifact_count: usize,
    pub agent_count: usize,
    pub activity_count: usize,
    pub rights_decision_count: usize,
    pub undetermined_rights_count: usize,
    pub fixity_event_count: usize,
    pub matched_fixity_event_count: usize,
    pub mismatched_fixity_event_count: usize,
    pub artifacts_without_fixity_event: usize,
    pub canonical_manifest_sha256: String,
    pub graph_is_acyclic: bool,
    pub claim_ceiling: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HeritageProvenanceError {
    SchemaVersionMismatch,
    InvalidOrEmptyIdentity(String),
    DuplicateIdentity(String),
    InvalidArtifactDigest(String),
    InvalidReceiptDigest(String),
    ZeroByteArtifact(String),
    InvalidMediaType(String),
    MissingRightsDecision(String),
    UnknownArtifact(String),
    UnknownAgent(String),
    UnknownActivity(String),
    RightsDecisionArtifactMismatch(String),
    DuplicateRightsDecision(String),
    MissingRightsReviewer(String),
    MissingRightsReceipt(String),
    InvalidRightsPurposeSet(String),
    UndeterminedRightsCannotGrant(String),
    ProhibitedRightsCannotGrant(String),
    ActivityHasNoOutput(String),
    ActivityHasNoResponsibleAgent(String),
    DuplicateActivityReference(String),
    ActivityInputOutputOverlap(String),
    MissingActivityInput(String),
    MissingActivityOutput(String),
    MissingGeneratingActivity(String),
    UnexpectedGeneratingActivity(String),
    OutputGeneratorMismatch(String),
    MultipleGenerators(String),
    MissingAutomatedProvenanceDigest(String),
    InvalidActivityTimestamp(String),
    ActivityTimeOrderInvalid(String),
    PurposeNotAllowed {
        activity_id: String,
        artifact_id: String,
    },
    CycleDetected,
    FixityReferenceMismatch(String),
    InvalidFixityObservedDigest(String),
    InvalidFixityOutcome(String),
    MissingFixityAgent(String),
    SerializationFailure,
}

impl fmt::Display for HeritageProvenanceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}
impl std::error::Error for HeritageProvenanceError {}

pub fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write as _;
        write!(&mut output, "{byte:02x}").expect("writing to String cannot fail");
    }
    output
}

fn non_empty(value: &str) -> bool {
    !value.trim().is_empty()
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn valid_utc_timestamp(value: &str) -> bool {
    value.ends_with('Z')
        && value.contains('T')
        && value.len() >= 20
        && !value.chars().any(char::is_whitespace)
}

fn is_automated(kind: ActivityKind) -> bool {
    matches!(
        kind,
        ActivityKind::Transformation | ActivityKind::ModelInference | ActivityKind::Simulation
    )
}

/// Recomputes the digest from exact payload bytes. The caller must persist the
/// resulting event separately; this function does not authenticate the caller.
pub fn verify_artifact_bytes(artifact: &ProvenanceArtifact, bytes: &[u8]) -> ByteFixityCheck {
    let observed = sha256_hex(bytes);
    ByteFixityCheck {
        artifact_id: artifact.artifact_id.clone(),
        expected_sha256: artifact.sha256.clone(),
        expected_byte_length: artifact.byte_length,
        observed_byte_length: bytes.len() as u64,
        length_matched: bytes.len() as u64 == artifact.byte_length,
        matched: observed == artifact.sha256 && bytes.len() as u64 == artifact.byte_length,
        observed_sha256: observed,
    }
}

fn canonical_manifest_bytes(
    manifest: &HeritageProvenanceManifest,
) -> Result<Vec<u8>, HeritageProvenanceError> {
    let mut canonical = manifest.clone();
    canonical
        .artifacts
        .sort_by(|a, b| a.artifact_id.cmp(&b.artifact_id));
    canonical.agents.sort_by(|a, b| a.agent_id.cmp(&b.agent_id));
    canonical
        .activities
        .sort_by(|a, b| a.activity_id.cmp(&b.activity_id));
    canonical
        .rights_decisions
        .sort_by(|a, b| a.decision_id.cmp(&b.decision_id));
    canonical
        .fixity_events
        .sort_by(|a, b| a.event_id.cmp(&b.event_id));

    for activity in &mut canonical.activities {
        activity.input_artifact_ids.sort();
        activity.generated_artifact_ids.sort();
        activity.responsible_agent_ids.sort();
    }
    for decision in &mut canonical.rights_decisions {
        decision.allowed_purposes.sort();
        decision.prohibited_purposes.sort();
    }

    serde_json::to_vec(&canonical).map_err(|_| HeritageProvenanceError::SerializationFailure)
}

pub fn canonical_manifest_sha256(
    manifest: &HeritageProvenanceManifest,
) -> Result<String, HeritageProvenanceError> {
    Ok(sha256_hex(&canonical_manifest_bytes(manifest)?))
}

fn graph_is_acyclic(artifact_ids: &BTreeSet<String>, activities: &[ProvenanceActivity]) -> bool {
    let mut adjacency = BTreeMap::<String, BTreeSet<String>>::new();
    let mut indegree = artifact_ids
        .iter()
        .cloned()
        .map(|id| (id, 0usize))
        .collect::<BTreeMap<_, _>>();

    for activity in activities {
        for input_id in &activity.input_artifact_ids {
            for output_id in &activity.generated_artifact_ids {
                if adjacency
                    .entry(input_id.clone())
                    .or_default()
                    .insert(output_id.clone())
                {
                    *indegree.entry(output_id.clone()).or_default() += 1;
                }
            }
        }
    }

    let mut queue = VecDeque::new();
    for (id, degree) in &indegree {
        if *degree == 0 {
            queue.push_back(id.clone());
        }
    }

    let mut visited = 0usize;
    while let Some(id) = queue.pop_front() {
        visited += 1;
        if let Some(outputs) = adjacency.get(&id) {
            for output in outputs {
                let degree = indegree
                    .get_mut(output)
                    .expect("all graph nodes are artifacts");
                *degree -= 1;
                if *degree == 0 {
                    queue.push_back(output.clone());
                }
            }
        }
    }
    visited == artifact_ids.len()
}

pub fn audit_heritage_provenance(
    manifest: &HeritageProvenanceManifest,
) -> Result<HeritageProvenanceAuditReport, HeritageProvenanceError> {
    if manifest.schema_version != HERITAGE_PROVENANCE_SCHEMA_VERSION {
        return Err(HeritageProvenanceError::SchemaVersionMismatch);
    }

    let mut artifacts = BTreeMap::<&str, &ProvenanceArtifact>::new();
    let mut artifact_ids = BTreeSet::new();
    for artifact in &manifest.artifacts {
        if !non_empty(&artifact.artifact_id) {
            return Err(HeritageProvenanceError::InvalidOrEmptyIdentity(
                "artifact_id".to_owned(),
            ));
        }
        if !artifact_ids.insert(artifact.artifact_id.clone()) {
            return Err(HeritageProvenanceError::DuplicateIdentity(
                artifact.artifact_id.clone(),
            ));
        }
        if !valid_sha256(&artifact.sha256) {
            return Err(HeritageProvenanceError::InvalidArtifactDigest(
                artifact.artifact_id.clone(),
            ));
        }
        if artifact.byte_length == 0 {
            return Err(HeritageProvenanceError::ZeroByteArtifact(
                artifact.artifact_id.clone(),
            ));
        }
        if !non_empty(&artifact.media_type) {
            return Err(HeritageProvenanceError::InvalidMediaType(
                artifact.artifact_id.clone(),
            ));
        }
        if !non_empty(&artifact.rights_decision_id) {
            return Err(HeritageProvenanceError::MissingRightsDecision(
                artifact.artifact_id.clone(),
            ));
        }
        match (
            artifact.origin,
            artifact.generated_by_activity_id.as_deref(),
        ) {
            (ArtifactOrigin::Primary, None) => {}
            (ArtifactOrigin::Primary, Some(_)) => {
                return Err(HeritageProvenanceError::UnexpectedGeneratingActivity(
                    artifact.artifact_id.clone(),
                ));
            }
            (ArtifactOrigin::Derived, None) => {
                return Err(HeritageProvenanceError::MissingGeneratingActivity(
                    artifact.artifact_id.clone(),
                ));
            }
            (ArtifactOrigin::Derived, Some(id)) if !non_empty(id) => {
                return Err(HeritageProvenanceError::MissingGeneratingActivity(
                    artifact.artifact_id.clone(),
                ));
            }
            (ArtifactOrigin::Derived, Some(_)) => {}
        }
        artifacts.insert(&artifact.artifact_id, artifact);
    }

    let mut agents = BTreeMap::<&str, &ProvenanceAgent>::new();
    for agent in &manifest.agents {
        if !non_empty(&agent.agent_id) || !non_empty(&agent.external_reference) {
            return Err(HeritageProvenanceError::InvalidOrEmptyIdentity(
                "agent".to_owned(),
            ));
        }
        if agents.insert(&agent.agent_id, agent).is_some() {
            return Err(HeritageProvenanceError::DuplicateIdentity(
                agent.agent_id.clone(),
            ));
        }
    }

    let mut rights_by_artifact = BTreeMap::<&str, &ArtifactRightsDecision>::new();
    let mut decision_ids = BTreeSet::new();
    for decision in &manifest.rights_decisions {
        if !non_empty(&decision.decision_id) || !non_empty(&decision.artifact_id) {
            return Err(HeritageProvenanceError::InvalidOrEmptyIdentity(
                "rights_decision".to_owned(),
            ));
        }
        if !decision_ids.insert(decision.decision_id.as_str()) {
            return Err(HeritageProvenanceError::DuplicateRightsDecision(
                decision.decision_id.clone(),
            ));
        }
        if !artifacts.contains_key(decision.artifact_id.as_str()) {
            return Err(HeritageProvenanceError::UnknownArtifact(
                decision.artifact_id.clone(),
            ));
        }
        if decision
            .allowed_purposes
            .iter()
            .collect::<BTreeSet<_>>()
            .len()
            != decision.allowed_purposes.len()
            || decision
                .prohibited_purposes
                .iter()
                .collect::<BTreeSet<_>>()
                .len()
                != decision.prohibited_purposes.len()
        {
            return Err(HeritageProvenanceError::InvalidRightsPurposeSet(
                decision.decision_id.clone(),
            ));
        }
        let allowed = decision
            .allowed_purposes
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        let prohibited = decision
            .prohibited_purposes
            .iter()
            .copied()
            .collect::<BTreeSet<_>>();
        if !allowed.is_disjoint(&prohibited) {
            return Err(HeritageProvenanceError::InvalidRightsPurposeSet(
                decision.decision_id.clone(),
            ));
        }
        match decision.status {
            RightsStatus::Allowed | RightsStatus::Restricted if allowed.is_empty() => {
                return Err(HeritageProvenanceError::InvalidRightsPurposeSet(
                    decision.decision_id.clone(),
                ));
            }
            RightsStatus::Prohibited | RightsStatus::Undetermined if !allowed.is_empty() => {
                return Err(match decision.status {
                    RightsStatus::Prohibited => {
                        HeritageProvenanceError::ProhibitedRightsCannotGrant(
                            decision.decision_id.clone(),
                        )
                    }
                    _ => HeritageProvenanceError::UndeterminedRightsCannotGrant(
                        decision.decision_id.clone(),
                    ),
                });
            }
            _ => {}
        }
        match (
            decision.decision_receipt_ref.as_deref(),
            decision.decision_receipt_sha256.as_deref(),
        ) {
            (Some(reference), Some(digest)) if non_empty(reference) && valid_sha256(digest) => {}
            (None, None) if decision.status == RightsStatus::Undetermined => {}
            (Some(_), Some(_)) => {
                return Err(HeritageProvenanceError::InvalidReceiptDigest(
                    decision.decision_id.clone(),
                ));
            }
            _ if decision.status != RightsStatus::Undetermined => {
                return Err(HeritageProvenanceError::MissingRightsReceipt(
                    decision.decision_id.clone(),
                ));
            }
            _ => {
                return Err(HeritageProvenanceError::InvalidRightsPurposeSet(
                    decision.decision_id.clone(),
                ));
            }
        }
        if !agents.contains_key(decision.reviewed_by_agent_id.as_str()) {
            return Err(HeritageProvenanceError::MissingRightsReviewer(
                decision.reviewed_by_agent_id.clone(),
            ));
        }
        if rights_by_artifact
            .insert(decision.artifact_id.as_str(), decision)
            .is_some()
        {
            return Err(HeritageProvenanceError::DuplicateRightsDecision(
                decision.artifact_id.clone(),
            ));
        }
    }

    for artifact in &manifest.artifacts {
        let decision = rights_by_artifact
            .get(artifact.artifact_id.as_str())
            .ok_or_else(|| {
                HeritageProvenanceError::MissingRightsDecision(artifact.artifact_id.clone())
            })?;
        if artifact.rights_decision_id != decision.decision_id {
            return Err(HeritageProvenanceError::RightsDecisionArtifactMismatch(
                artifact.artifact_id.clone(),
            ));
        }
    }

    let mut activities = BTreeMap::<&str, &ProvenanceActivity>::new();
    let mut generated_by = BTreeMap::<&str, &str>::new();
    for activity in &manifest.activities {
        if !non_empty(&activity.activity_id) {
            return Err(HeritageProvenanceError::InvalidOrEmptyIdentity(
                "activity_id".to_owned(),
            ));
        }
        if activities.insert(&activity.activity_id, activity).is_some() {
            return Err(HeritageProvenanceError::DuplicateIdentity(
                activity.activity_id.clone(),
            ));
        }
        if activity.generated_artifact_ids.is_empty() {
            return Err(HeritageProvenanceError::ActivityHasNoOutput(
                activity.activity_id.clone(),
            ));
        }
        if activity.responsible_agent_ids.is_empty() {
            return Err(HeritageProvenanceError::ActivityHasNoResponsibleAgent(
                activity.activity_id.clone(),
            ));
        }
        if !valid_utc_timestamp(&activity.started_at_utc)
            || !valid_utc_timestamp(&activity.ended_at_utc)
        {
            return Err(HeritageProvenanceError::InvalidActivityTimestamp(
                activity.activity_id.clone(),
            ));
        }
        if activity.started_at_utc > activity.ended_at_utc {
            return Err(HeritageProvenanceError::ActivityTimeOrderInvalid(
                activity.activity_id.clone(),
            ));
        }
        for id in &activity.responsible_agent_ids {
            if !agents.contains_key(id.as_str()) {
                return Err(HeritageProvenanceError::UnknownAgent(id.clone()));
            }
        }
        if activity
            .responsible_agent_ids
            .iter()
            .collect::<BTreeSet<_>>()
            .len()
            != activity.responsible_agent_ids.len()
            || activity
                .input_artifact_ids
                .iter()
                .collect::<BTreeSet<_>>()
                .len()
                != activity.input_artifact_ids.len()
            || activity
                .generated_artifact_ids
                .iter()
                .collect::<BTreeSet<_>>()
                .len()
                != activity.generated_artifact_ids.len()
        {
            return Err(HeritageProvenanceError::DuplicateActivityReference(
                activity.activity_id.clone(),
            ));
        }
        let input_set = activity.input_artifact_ids.iter().collect::<BTreeSet<_>>();
        for input_id in &activity.input_artifact_ids {
            if !artifacts.contains_key(input_id.as_str()) {
                return Err(HeritageProvenanceError::MissingActivityInput(
                    input_id.clone(),
                ));
            }
            if !input_set.contains(input_id) {
                return Err(HeritageProvenanceError::MissingActivityInput(
                    input_id.clone(),
                ));
            }
            let rights = rights_by_artifact
                .get(input_id.as_str())
                .expect("every artifact has one rights decision");
            if !matches!(
                rights.status,
                RightsStatus::Allowed | RightsStatus::Restricted
            ) || !rights.allowed_purposes.contains(&activity.purpose)
                || rights.prohibited_purposes.contains(&activity.purpose)
            {
                return Err(HeritageProvenanceError::PurposeNotAllowed {
                    activity_id: activity.activity_id.clone(),
                    artifact_id: input_id.clone(),
                });
            }
        }
        for output_id in &activity.generated_artifact_ids {
            if input_set.contains(output_id) {
                return Err(HeritageProvenanceError::ActivityInputOutputOverlap(
                    activity.activity_id.clone(),
                ));
            }
            let output = artifacts
                .get(output_id.as_str())
                .ok_or_else(|| HeritageProvenanceError::MissingActivityOutput(output_id.clone()))?;
            if output.origin != ArtifactOrigin::Derived
                || output.generated_by_activity_id.as_deref() != Some(activity.activity_id.as_str())
            {
                return Err(HeritageProvenanceError::OutputGeneratorMismatch(
                    output_id.clone(),
                ));
            }
            if generated_by
                .insert(output_id.as_str(), activity.activity_id.as_str())
                .is_some()
            {
                return Err(HeritageProvenanceError::MultipleGenerators(
                    output_id.clone(),
                ));
            }
        }
        if is_automated(activity.kind) {
            let software_id = activity.software_agent_id.as_deref().ok_or_else(|| {
                HeritageProvenanceError::MissingAutomatedProvenanceDigest(
                    activity.activity_id.clone(),
                )
            })?;
            let software = agents
                .get(software_id)
                .ok_or_else(|| HeritageProvenanceError::UnknownAgent(software_id.to_owned()))?;
            if software.kind != AgentKind::Software {
                return Err(HeritageProvenanceError::UnknownAgent(
                    software_id.to_owned(),
                ));
            }
            for digest in [
                activity.software_build_sha256.as_deref(),
                activity.environment_sha256.as_deref(),
                activity.parameters_sha256.as_deref(),
            ] {
                if digest.is_none_or(|value| !valid_sha256(value)) {
                    return Err(HeritageProvenanceError::MissingAutomatedProvenanceDigest(
                        activity.activity_id.clone(),
                    ));
                }
            }
            if activity
                .protocol_sha256
                .as_deref()
                .is_some_and(|digest| !valid_sha256(digest))
            {
                return Err(HeritageProvenanceError::MissingAutomatedProvenanceDigest(
                    activity.activity_id.clone(),
                ));
            }
        }
    }

    for artifact in &manifest.artifacts {
        match artifact.generated_by_activity_id.as_deref() {
            None if artifact.origin == ArtifactOrigin::Primary => {}
            Some(activity_id) => {
                let activity = activities.get(activity_id).ok_or_else(|| {
                    HeritageProvenanceError::UnknownActivity(activity_id.to_owned())
                })?;
                if !activity
                    .generated_artifact_ids
                    .contains(&artifact.artifact_id)
                {
                    return Err(HeritageProvenanceError::OutputGeneratorMismatch(
                        artifact.artifact_id.clone(),
                    ));
                }
                if !generated_by.contains_key(artifact.artifact_id.as_str()) {
                    return Err(HeritageProvenanceError::MissingGeneratingActivity(
                        artifact.artifact_id.clone(),
                    ));
                }
            }
            None => {
                return Err(HeritageProvenanceError::MissingGeneratingActivity(
                    artifact.artifact_id.clone(),
                ));
            }
        }
    }

    if !graph_is_acyclic(&artifact_ids, &manifest.activities) {
        return Err(HeritageProvenanceError::CycleDetected);
    }

    let mut fixity_event_ids = BTreeSet::new();
    let mut matched_fixity_event_count = 0usize;
    let mut mismatched_fixity_event_count = 0usize;
    let mut verified_artifacts = BTreeSet::new();
    for event in &manifest.fixity_events {
        if !non_empty(&event.event_id) || !fixity_event_ids.insert(event.event_id.as_str()) {
            return Err(HeritageProvenanceError::DuplicateIdentity(
                event.event_id.clone(),
            ));
        }
        let artifact = artifacts
            .get(event.artifact_id.as_str())
            .ok_or_else(|| HeritageProvenanceError::UnknownArtifact(event.artifact_id.clone()))?;
        if event.expected_sha256 != artifact.sha256 {
            return Err(HeritageProvenanceError::FixityReferenceMismatch(
                event.event_id.clone(),
            ));
        }
        if !valid_sha256(&event.observed_sha256) {
            return Err(HeritageProvenanceError::InvalidFixityObservedDigest(
                event.event_id.clone(),
            ));
        }
        if !valid_utc_timestamp(&event.checked_at_utc) {
            return Err(HeritageProvenanceError::InvalidActivityTimestamp(
                event.event_id.clone(),
            ));
        }
        if !agents.contains_key(event.checked_by_agent_id.as_str()) {
            return Err(HeritageProvenanceError::MissingFixityAgent(
                event.checked_by_agent_id.clone(),
            ));
        }
        match (
            event.outcome,
            event.expected_sha256 == event.observed_sha256
                && event.observed_byte_length == artifact.byte_length,
        ) {
            (FixityOutcome::Matched, true) => {
                matched_fixity_event_count += 1;
                verified_artifacts.insert(event.artifact_id.as_str());
            }
            (FixityOutcome::Mismatch, false) => mismatched_fixity_event_count += 1,
            _ => {
                return Err(HeritageProvenanceError::InvalidFixityOutcome(
                    event.event_id.clone(),
                ));
            }
        }
    }

    let canonical_manifest_sha256 = canonical_manifest_sha256(manifest)?;
    let primary_artifact_count = manifest
        .artifacts
        .iter()
        .filter(|artifact| artifact.origin == ArtifactOrigin::Primary)
        .count();
    let derived_artifact_count = manifest.artifacts.len() - primary_artifact_count;
    let undetermined_rights_count = manifest
        .rights_decisions
        .iter()
        .filter(|decision| decision.status == RightsStatus::Undetermined)
        .count();

    Ok(HeritageProvenanceAuditReport {
        schema_version: HERITAGE_PROVENANCE_SCHEMA_VERSION.to_owned(),
        artifact_count: manifest.artifacts.len(),
        primary_artifact_count,
        derived_artifact_count,
        agent_count: manifest.agents.len(),
        activity_count: manifest.activities.len(),
        rights_decision_count: manifest.rights_decisions.len(),
        undetermined_rights_count,
        fixity_event_count: manifest.fixity_events.len(),
        matched_fixity_event_count,
        mismatched_fixity_event_count,
        artifacts_without_fixity_event: manifest.artifacts.len() - verified_artifacts.len(),
        canonical_manifest_sha256,
        graph_is_acyclic: true,
        claim_ceiling: "declared_provenance_and_fixity_records_structurally_valid_only".to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const RAW_BYTES: &[u8] = b"synthetic-raw-capture-bytes";
    const POINT_CLOUD_BYTES: &[u8] = b"synthetic-processed-point-cloud";

    fn rights(
        decision_id: &str,
        artifact_id: &str,
        status: RightsStatus,
        allowed: Vec<RightsPurpose>,
        prohibited: Vec<RightsPurpose>,
    ) -> ArtifactRightsDecision {
        ArtifactRightsDecision {
            decision_id: decision_id.to_owned(),
            artifact_id: artifact_id.to_owned(),
            status,
            allowed_purposes: allowed,
            prohibited_purposes: prohibited,
            decision_receipt_ref: if status == RightsStatus::Undetermined {
                None
            } else {
                Some(format!("rights://receipts/{decision_id}"))
            },
            decision_receipt_sha256: if status == RightsStatus::Undetermined {
                None
            } else {
                Some(sha256_hex(b"synthetic-rights-receipt"))
            },
            reviewed_by_agent_id: "archivist-1".to_owned(),
        }
    }

    fn manifest() -> HeritageProvenanceManifest {
        let raw_digest = sha256_hex(RAW_BYTES);
        let point_cloud_digest = sha256_hex(POINT_CLOUD_BYTES);
        HeritageProvenanceManifest {
            schema_version: HERITAGE_PROVENANCE_SCHEMA_VERSION.to_owned(),
            artifacts: vec![
                ProvenanceArtifact {
                    artifact_id: "raw-capture-1".to_owned(),
                    sha256: raw_digest,
                    byte_length: RAW_BYTES.len() as u64,
                    media_type: "image/tiff".to_owned(),
                    role: ArtifactRole::PrimaryCapture,
                    origin: ArtifactOrigin::Primary,
                    access_tier: AccessTier::Restricted,
                    rights_decision_id: "rights-raw-1".to_owned(),
                    generated_by_activity_id: None,
                },
                ProvenanceArtifact {
                    artifact_id: "point-cloud-1".to_owned(),
                    sha256: point_cloud_digest,
                    byte_length: POINT_CLOUD_BYTES.len() as u64,
                    media_type: "application/vnd.las".to_owned(),
                    role: ArtifactRole::PointCloud,
                    origin: ArtifactOrigin::Derived,
                    access_tier: AccessTier::Restricted,
                    rights_decision_id: "rights-cloud-1".to_owned(),
                    generated_by_activity_id: Some("photogrammetry-1".to_owned()),
                },
            ],
            agents: vec![
                ProvenanceAgent {
                    agent_id: "archivist-1".to_owned(),
                    kind: AgentKind::Person,
                    external_reference: "agent://pseudonymous/archivist-1".to_owned(),
                },
                ProvenanceAgent {
                    agent_id: "pipeline-1".to_owned(),
                    kind: AgentKind::Software,
                    external_reference: "software://sol-atlas/photogrammetry-pipeline".to_owned(),
                },
            ],
            activities: vec![ProvenanceActivity {
                activity_id: "photogrammetry-1".to_owned(),
                kind: ActivityKind::Transformation,
                purpose: RightsPurpose::ResearchAnalysis,
                input_artifact_ids: vec!["raw-capture-1".to_owned()],
                generated_artifact_ids: vec!["point-cloud-1".to_owned()],
                responsible_agent_ids: vec!["archivist-1".to_owned(), "pipeline-1".to_owned()],
                software_agent_id: Some("pipeline-1".to_owned()),
                software_build_sha256: Some(sha256_hex(b"synthetic-software-build")),
                environment_sha256: Some(sha256_hex(b"synthetic-environment-lockfile")),
                parameters_sha256: Some(sha256_hex(b"synthetic-provenance-parameters")),
                protocol_sha256: None,
                started_at_utc: "2026-11-05T08:00:00Z".to_owned(),
                ended_at_utc: "2026-11-05T08:05:00Z".to_owned(),
            }],
            rights_decisions: vec![
                rights(
                    "rights-raw-1",
                    "raw-capture-1",
                    RightsStatus::Restricted,
                    vec![RightsPurpose::ResearchAnalysis, RightsPurpose::Preservation],
                    vec![RightsPurpose::CommercialUse, RightsPurpose::Redistribution],
                ),
                rights(
                    "rights-cloud-1",
                    "point-cloud-1",
                    RightsStatus::Allowed,
                    vec![RightsPurpose::ResearchAnalysis, RightsPurpose::Preservation],
                    vec![RightsPurpose::CommercialUse, RightsPurpose::Redistribution],
                ),
            ],
            fixity_events: vec![FixityEvent {
                event_id: "fixity-raw-1".to_owned(),
                artifact_id: "raw-capture-1".to_owned(),
                expected_sha256: sha256_hex(RAW_BYTES),
                observed_sha256: sha256_hex(RAW_BYTES),
                outcome: FixityOutcome::Matched,
                checked_at_utc: "2026-11-05T08:10:00Z".to_owned(),
                checked_by_agent_id: "archivist-1".to_owned(),
            }],
        }
    }

    #[test]
    fn valid_provenance_graph_audits_and_commits_canonical_manifest() {
        let report = audit_heritage_provenance(&manifest()).expect("valid provenance");
        assert_eq!(report.artifact_count, 2);
        assert_eq!(report.primary_artifact_count, 1);
        assert_eq!(report.derived_artifact_count, 1);
        assert_eq!(report.activity_count, 1);
        assert_eq!(report.matched_fixity_event_count, 1);
        assert_eq!(report.mismatched_fixity_event_count, 0);
        assert_eq!(report.artifacts_without_fixity_event, 1);
        assert_eq!(report.canonical_manifest_sha256.len(), 64);
        assert!(report.graph_is_acyclic);
    }

    #[test]
    fn byte_verifier_detects_exact_payload_mutation() {
        let input = manifest();
        let artifact = &input.artifacts[0];
        assert!(verify_artifact_bytes(artifact, RAW_BYTES).matched);
        let mut mutated = RAW_BYTES.to_vec();
        mutated[0] ^= 0x01;
        let result = verify_artifact_bytes(artifact, &mutated);
        assert!(!result.matched);
        assert_ne!(result.expected_sha256, result.observed_sha256);
    }

    #[test]
    fn byte_verifier_rejects_length_mismatch_even_when_payload_prefix_matches() {
        let input = manifest();
        let artifact = &input.artifacts[0];
        let mut extended = RAW_BYTES.to_vec();
        extended.push(0);
        let result = verify_artifact_bytes(artifact, &extended);
        assert!(!result.length_matched);
        assert!(!result.matched);
    }

    #[test]
    fn mismatch_fixity_event_is_retained_and_reported_not_promoted() {
        let mut input = manifest();
        input.fixity_events.push(FixityEvent {
            event_id: "fixity-raw-mismatch".to_owned(),
            artifact_id: "raw-capture-1".to_owned(),
            expected_sha256: input.artifacts[0].sha256.clone(),
            observed_sha256: sha256_hex(b"mutated-payload"),
            observed_byte_length: b"mutated-payload".len() as u64,
            outcome: FixityOutcome::Mismatch,
            checked_at_utc: "2026-11-05T08:11:00Z".to_owned(),
            checked_by_agent_id: "archivist-1".to_owned(),
        });
        let report = audit_heritage_provenance(&input).expect("mismatch is a retained event");
        assert_eq!(report.mismatched_fixity_event_count, 1);
        assert_eq!(report.matched_fixity_event_count, 1);
    }

    #[test]
    fn research_rights_do_not_imply_model_training_or_redistribution() {
        let mut input = manifest();
        input.activities[0].purpose = RightsPurpose::ModelTraining;
        assert_eq!(
            audit_heritage_provenance(&input),
            Err(HeritageProvenanceError::PurposeNotAllowed {
                activity_id: "photogrammetry-1".to_owned(),
                artifact_id: "raw-capture-1".to_owned(),
            })
        );
    }

    #[test]
    fn undetermined_rights_fail_closed_for_every_activity_purpose() {
        let mut input = manifest();
        input.rights_decisions[0] = rights(
            "rights-raw-1",
            "raw-capture-1",
            RightsStatus::Undetermined,
            Vec::new(),
            Vec::new(),
        );
        assert!(matches!(
            audit_heritage_provenance(&input),
            Err(HeritageProvenanceError::PurposeNotAllowed { .. })
        ));
    }

    #[test]
    fn undetermined_rights_cannot_carry_an_allowed_purpose() {
        let mut input = manifest();
        input.rights_decisions[0].status = RightsStatus::Undetermined;
        input.rights_decisions[0].allowed_purposes = vec![RightsPurpose::ResearchAnalysis];
        assert!(matches!(
            audit_heritage_provenance(&input),
            Err(HeritageProvenanceError::UndeterminedRightsCannotGrant(_))
        ));
    }

    #[test]
    fn automated_transform_requires_build_environment_and_parameters() {
        let mut input = manifest();
        input.activities[0].environment_sha256 = None;
        assert!(matches!(
            audit_heritage_provenance(&input),
            Err(HeritageProvenanceError::MissingAutomatedProvenanceDigest(_))
        ));
    }

    #[test]
    fn derived_artifact_without_generator_is_rejected() {
        let mut input = manifest();
        input.artifacts[1].generated_by_activity_id = None;
        assert!(matches!(
            audit_heritage_provenance(&input),
            Err(HeritageProvenanceError::MissingGeneratingActivity(_))
        ));
    }

    #[test]
    fn cycle_in_activity_lineage_is_rejected() {
        let mut input = manifest();
        input.artifacts[0].origin = ArtifactOrigin::Derived;
        input.artifacts[0].generated_by_activity_id = Some("reverse-transform".to_owned());
        input.activities.push(ProvenanceActivity {
            activity_id: "reverse-transform".to_owned(),
            kind: ActivityKind::Transformation,
            purpose: RightsPurpose::ResearchAnalysis,
            input_artifact_ids: vec!["point-cloud-1".to_owned()],
            generated_artifact_ids: vec!["raw-capture-1".to_owned()],
            responsible_agent_ids: vec!["archivist-1".to_owned(), "pipeline-1".to_owned()],
            software_agent_id: Some("pipeline-1".to_owned()),
            software_build_sha256: Some(sha256_hex(b"synthetic-software-build")),
            environment_sha256: Some(sha256_hex(b"synthetic-environment-lockfile")),
            parameters_sha256: Some(sha256_hex(b"synthetic-provenance-parameters")),
            protocol_sha256: None,
            started_at_utc: "2026-11-05T08:06:00Z".to_owned(),
            ended_at_utc: "2026-11-05T08:07:00Z".to_owned(),
        });
        assert_eq!(
            audit_heritage_provenance(&input),
            Err(HeritageProvenanceError::CycleDetected)
        );
    }

    #[test]
    fn missing_agent_and_orphan_activity_reference_are_rejected() {
        let mut input = manifest();
        input.activities[0]
            .responsible_agent_ids
            .push("unknown-agent".to_owned());
        assert_eq!(
            audit_heritage_provenance(&input),
            Err(HeritageProvenanceError::UnknownAgent(
                "unknown-agent".to_owned()
            ))
        );

        let mut input = manifest();
        input.activities[0]
            .input_artifact_ids
            .push("missing-artifact".to_owned());
        assert_eq!(
            audit_heritage_provenance(&input),
            Err(HeritageProvenanceError::MissingActivityInput(
                "missing-artifact".to_owned()
            ))
        );
    }

    #[test]
    fn fixity_outcome_must_match_expected_and_observed_digests() {
        let mut input = manifest();
        input.fixity_events[0].outcome = FixityOutcome::Mismatch;
        input.fixity_events[0].observed_byte_length = RAW_BYTES.len() as u64;
        assert!(matches!(
            audit_heritage_provenance(&input),
            Err(HeritageProvenanceError::InvalidFixityOutcome(_))
        ));
    }

    #[test]
    fn canonical_manifest_digest_ignores_record_order_but_tracks_semantic_changes() {
        let mut input = manifest();
        let first = canonical_manifest_sha256(&input).expect("canonical digest");
        input.artifacts.reverse();
        input.agents.reverse();
        input.activities[0].responsible_agent_ids.reverse();
        input.rights_decisions.reverse();
        let reordered = canonical_manifest_sha256(&input).expect("reordered canonical digest");
        assert_eq!(first, reordered);

        input.activities[0].parameters_sha256 = Some(sha256_hex(b"changed-parameters"));
        let changed = canonical_manifest_sha256(&input).expect("changed canonical digest");
        assert_ne!(first, changed);
    }

    #[test]
    fn provenance_schema_and_synthetic_fixture_parse_and_audit() {
        let schema: serde_json::Value = serde_json::from_str(include_str!(
            "../../schemas/heritage-provenance-v0.1.schema.json"
        ))
        .expect("provenance schema JSON");
        let fixture: HeritageProvenanceManifest = serde_json::from_str(include_str!(
            "../../schemas/examples/heritage-provenance-synthetic-v0.1.json"
        ))
        .expect("provenance fixture");
        assert_eq!(
            schema["$schema"],
            "https://json-schema.org/draft/2020-12/schema"
        );
        assert!(audit_heritage_provenance(&fixture).is_ok());
    }

    #[test]
    fn every_artifact_requires_an_explicit_rights_decision() {
        let mut input = manifest();
        input.rights_decisions.pop();
        assert!(matches!(
            audit_heritage_provenance(&input),
            Err(HeritageProvenanceError::MissingRightsDecision(_))
        ));
    }
}
