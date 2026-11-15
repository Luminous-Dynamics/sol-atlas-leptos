//! Synthetic, non-authorizing assurance-to-campaign bridge verifier.
//!
//! This module does not call Mycelix or Symtropy, plan observations, schedule
//! resources, calculate a Pareto frontier, authenticate source bytes, verify
//! signatures, or qualify physical evidence. It checks only a caller-supplied
//! benchmark envelope. Production owner profiles remain UNBOUND until owners
//! publish and approve exact versioned source contracts.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

use crate::sha256_hex;

pub const BRIDGE_PROFILE_ID: &str = "luminous.highveld.evidence-campaign-bridge.v0.1";
pub const BRIDGE_VERSION: &str = "0.1.0";
pub const BRIDGE_STATUS: &str = "SYNTHETIC_BRIDGE_CONTRACT_ONLY";
pub const BRIDGE_CLAIM_CEILING: &str = "NON_AUTHORIZING_SYNTHETIC_EVALUATION";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BridgeInput {
    pub profile_id: String,
    pub version: String,
    pub status: String,
    pub claim_ceiling: String,
    pub episode: DecisionEpisode,
    pub acquisitions: Vec<Acquisition>,
    pub campaigns: Vec<Campaign>,
    pub physical_bindings: Vec<PhysicalBinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OwnerBinding {
    pub owner: String,
    pub repository: String,
    pub commit_sha: Option<String>,
    pub artifact_path: Option<String>,
    pub artifact_sha256: Option<String>,
    pub profile_id: Option<String>,
    pub profile_version: Option<String>,
    pub adapter_id: Option<String>,
    pub adapter_sha256: Option<String>,
    pub mapping_status: BindingState,
    pub mapping_sha256: Option<String>,
    pub owner_approval_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BindingState {
    Bound,
    Unbound,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum GateState {
    Allowed,
    Blocked,
    Unknown,
    NotApplicable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DecisionEpisode {
    pub binding: OwnerBinding,
    pub episode_id: String,
    pub as_of_unix: u64,
    pub context: DecisionContext,
    pub unresolved_obligation_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DecisionContext {
    pub project_context_sha256: String,
    pub functional_contract_sha256: String,
    pub option_design_sha256: String,
    pub site_epoch: String,
    pub coordinate_frame_id: String,
    pub clock_domain: String,
    pub currency: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Acquisition {
    pub binding: OwnerBinding,
    pub acquisition_id: String,
    pub episode_id: String,
    pub context: DecisionContext,
    pub kind: AcquisitionKind,
    pub covers_obligation_ids: Vec<String>,
    pub gates: AdmissibilityGates,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AcquisitionKind {
    Remote,
    Physical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AdmissibilityGates {
    pub authority: GateState,
    pub rights_source_use: GateState,
    pub safety: GateState,
    pub schedule: GateState,
    pub physical_realizability: GateState,
    pub data_access: GateState,
    pub source_competence: GateState,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Campaign {
    pub binding: OwnerBinding,
    pub package_id: String,
    pub context: DecisionContext,
    pub search_status: CampaignSearchStatus,
    pub actions: Vec<CampaignAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CampaignSearchStatus {
    CompleteOptimal,
    CompleteFeasibleNotOptimal,
    Incomplete,
    ProvenInfeasible,
    SearchIncomplete,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CampaignAction {
    pub action_id: String,
    pub context: DecisionContext,
    pub covers_obligation_ids: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PhysicalBinding {
    pub mapping_id: String,
    pub acquisition_id: String,
    pub episode_id: String,
    pub campaign_package_id: String,
    pub action_id: String,
    pub context: DecisionContext,
    pub review: MappingReview,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MappingReview {
    pub status: BindingState,
    pub mapping_sha256: Option<String>,
    pub owner_approval_ref: Option<String>,
    pub reviewed_by: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BridgeReport {
    pub report_profile_id: &'static str,
    pub input_sha256: String,
    pub evaluation_status: &'static str,
    pub episode_id: String,
    pub obligation_results: Vec<ObligationResult>,
    pub planned_obligations: usize,
    pub unresolved_obligations: usize,
    pub source_bytes_verified: bool,
    pub signatures_verified: bool,
    pub physical_observations: usize,
    pub qualified_obligations: usize,
    pub authority_effects: &'static str,
    pub claim_ceiling: &'static str,
}

#[derive(Debug, Clone, Serialize)]
pub struct ObligationResult {
    pub obligation_id: String,
    pub disposition: BridgeDisposition,
    pub evidence_state: &'static str,
    pub covered_by_acquisition_ids: Vec<String>,
    pub campaign_package_ids: Vec<String>,
    pub optimality_proven: bool,
    pub reason_codes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum BridgeDisposition {
    PlannedRemoteAcquisition,
    PlannedPhysicalObservationOptimal,
    PlannedPhysicalObservationNotOptimalityProven,
    BlockedAdmissibility,
    UnknownAdmissibility,
    Unbound,
    ContextMismatch,
    CampaignIncomplete,
    CampaignInfeasible,
    CampaignSearchIncomplete,
    NotPlanned,
}

/// Evaluate one caller-supplied synthetic bridge envelope.
///
/// A planned disposition means only that a declared acquisition task is
/// admissible in the supplied synthetic envelope. Physical acquisition
/// requires an owner-reviewed relation to an exact campaign action. Every
/// positive result remains PLANNED_NOT_OBSERVED.
pub fn evaluate_bridge_bytes(input_bytes: &[u8]) -> Result<BridgeReport, String> {
    let input: BridgeInput = serde_json::from_slice(input_bytes)
        .map_err(|error| format!("invalid bridge input JSON: {error}"))?;
    validate_bridge_input(&input)?;

    let mut obligation_results = Vec::with_capacity(input.episode.unresolved_obligation_ids.len());
    let mut planned_obligations = 0usize;

    for obligation_id in &input.episode.unresolved_obligation_ids {
        let candidates: Vec<&Acquisition> = input
            .acquisitions
            .iter()
            .filter(|acquisition| acquisition.covers_obligation_ids.contains(obligation_id))
            .collect();

        let result = evaluate_obligation(&input, obligation_id, &candidates)?;
        if is_planned(&result.disposition) {
            planned_obligations += 1;
        }
        obligation_results.push(result);
    }

    obligation_results.sort_by(|left, right| left.obligation_id.cmp(&right.obligation_id));
    let unresolved_obligations = obligation_results.len().saturating_sub(planned_obligations);

    Ok(BridgeReport {
        report_profile_id: "luminous.highveld.evidence-campaign-bridge-report.v0.1",
        input_sha256: sha256_hex(input_bytes),
        evaluation_status: "SYNTHETIC_BRIDGE_EVALUATION_ONLY",
        episode_id: input.episode.episode_id,
        obligation_results,
        planned_obligations,
        unresolved_obligations,
        source_bytes_verified: false,
        signatures_verified: false,
        physical_observations: 0,
        qualified_obligations: 0,
        authority_effects: "NONE",
        claim_ceiling: BRIDGE_CLAIM_CEILING,
    })
}

fn validate_bridge_input(input: &BridgeInput) -> Result<(), String> {
    if input.profile_id != BRIDGE_PROFILE_ID {
        return Err(format!("unsupported bridge profile_id: {}", input.profile_id));
    }
    if input.version != BRIDGE_VERSION {
        return Err(format!("unsupported bridge version: {}", input.version));
    }
    if input.status != BRIDGE_STATUS {
        return Err("bridge input is not marked SYNTHETIC_BRIDGE_CONTRACT_ONLY".to_owned());
    }
    if input.claim_ceiling != BRIDGE_CLAIM_CEILING {
        return Err("bridge input changed the non-authorizing claim ceiling".to_owned());
    }
    if input.episode.episode_id.trim().is_empty() {
        return Err("episode_id must not be empty".to_owned());
    }
    if input.episode.unresolved_obligation_ids.is_empty() {
        return Err("episode must declare at least one unresolved obligation".to_owned());
    }
    validate_unique_nonempty(
        &input.episode.unresolved_obligation_ids,
        "episode.unresolved_obligation_ids",
    )?;
    validate_context(&input.episode.context, "episode.context")?;
    validate_owner_binding(
        &input.episode.binding,
        "mycelix",
        "Luminous-Dynamics/mycelix",
        "episode.binding",
    )?;

    let known_obligations: BTreeSet<&str> = input
        .episode
        .unresolved_obligation_ids
        .iter()
        .map(String::as_str)
        .collect();

    let mut acquisition_ids = BTreeSet::new();
    for acquisition in &input.acquisitions {
        if acquisition.acquisition_id.trim().is_empty() {
            return Err("acquisition_id must not be empty".to_owned());
        }
        if !acquisition_ids.insert(acquisition.acquisition_id.as_str()) {
            return Err(format!(
                "duplicate acquisition_id: {}",
                acquisition.acquisition_id
            ));
        }
        if acquisition.episode_id.trim().is_empty() {
            return Err(format!(
                "{} has an empty episode_id",
                acquisition.acquisition_id
            ));
        }
        if acquisition.covers_obligation_ids.is_empty() {
            return Err(format!(
                "{} declares no proof obligations",
                acquisition.acquisition_id
            ));
        }
        validate_unique_nonempty(
            &acquisition.covers_obligation_ids,
            &format!("{}.covers_obligation_ids", acquisition.acquisition_id),
        )?;
        for obligation_id in &acquisition.covers_obligation_ids {
            if !known_obligations.contains(obligation_id.as_str()) {
                return Err(format!(
                    "{} references unknown obligation {}",
                    acquisition.acquisition_id, obligation_id
                ));
            }
        }
        validate_context(
            &acquisition.context,
            &format!("{}.context", acquisition.acquisition_id),
        )?;
        validate_owner_binding(
            &acquisition.binding,
            "mycelix",
            "Luminous-Dynamics/mycelix",
            &format!("{}.binding", acquisition.acquisition_id),
        )?;
    }

    let mut campaign_ids = BTreeSet::new();
    let mut action_ids = BTreeSet::new();
    for campaign in &input.campaigns {
        if campaign.package_id.trim().is_empty() {
            return Err("campaign package_id must not be empty".to_owned());
        }
        if !campaign_ids.insert(campaign.package_id.as_str()) {
            return Err(format!(
                "duplicate campaign package_id: {}",
                campaign.package_id
            ));
        }
        validate_context(&campaign.context, &format!("{}.context", campaign.package_id))?;
        validate_owner_binding(
            &campaign.binding,
            "symtropy",
            "Luminous-Dynamics/symtropy",
            &format!("{}.binding", campaign.package_id),
        )?;
        for action in &campaign.actions {
            if action.action_id.trim().is_empty() {
                return Err(format!(
                    "{} contains an empty action_id",
                    campaign.package_id
                ));
            }
            if !action_ids.insert(action.action_id.as_str()) {
                return Err(format!("duplicate campaign action_id: {}", action.action_id));
            }
            validate_unique_nonempty(
                &action.covers_obligation_ids,
                &format!("{}.{}.covers_obligation_ids", campaign.package_id, action.action_id),
            )?;
            for obligation_id in &action.covers_obligation_ids {
                if !known_obligations.contains(obligation_id.as_str()) {
                    return Err(format!(
                        "{} references unknown obligation {}",
                        action.action_id, obligation_id
                    ));
                }
            }
            validate_context(
                &action.context,
                &format!("{}.{}.context", campaign.package_id, action.action_id),
            )?;
        }
    }

    let mut mapping_ids = BTreeSet::new();
    let mut mapping_keys = BTreeSet::new();
    for physical_binding in &input.physical_bindings {
        if physical_binding.mapping_id.trim().is_empty() {
            return Err("physical mapping_id must not be empty".to_owned());
        }
        if !mapping_ids.insert(physical_binding.mapping_id.as_str()) {
            return Err(format!(
                "duplicate physical mapping_id: {}",
                physical_binding.mapping_id
            ));
        }
        if !acquisition_ids.contains(physical_binding.acquisition_id.as_str()) {
            return Err(format!(
                "{} references unknown acquisition {}",
                physical_binding.mapping_id, physical_binding.acquisition_id
            ));
        }
        if !campaign_ids.contains(physical_binding.campaign_package_id.as_str()) {
            return Err(format!(
                "{} references unknown campaign {}",
                physical_binding.mapping_id, physical_binding.campaign_package_id
            ));
        }
        let campaign = input
            .campaigns
            .iter()
            .find(|candidate| candidate.package_id == physical_binding.campaign_package_id)
            .expect("campaign id was checked above");
        if !campaign
            .actions
            .iter()
            .any(|action| action.action_id == physical_binding.action_id)
        {
            return Err(format!(
                "{} references unknown action {}",
                physical_binding.mapping_id, physical_binding.action_id
            ));
        }
        if !mapping_keys.insert((
            physical_binding.acquisition_id.as_str(),
            physical_binding.campaign_package_id.as_str(),
            physical_binding.action_id.as_str(),
        )) {
            return Err(format!(
                "duplicate acquisition-to-action relation: {}",
                physical_binding.mapping_id
            ));
        }
        validate_context(
            &physical_binding.context,
            &format!("{}.context", physical_binding.mapping_id),
        )?;
        validate_mapping_review(&physical_binding.review, &physical_binding.mapping_id)?;
    }

    for acquisition in &input.acquisitions {
        if acquisition.kind == AcquisitionKind::Remote
            && input
                .physical_bindings
                .iter()
                .any(|binding| binding.acquisition_id == acquisition.acquisition_id)
        {
            return Err(format!(
                "remote acquisition {} cannot be converted into a physical observation action",
                acquisition.acquisition_id
            ));
        }
    }

    Ok(())
}

fn validate_owner_binding(
    binding: &OwnerBinding,
    expected_owner: &str,
    expected_repository: &str,
    path: &str,
) -> Result<(), String> {
    if binding.owner != expected_owner {
        return Err(format!(
            "{path} owner {} does not match expected owner {expected_owner}",
            binding.owner
        ));
    }
    if binding.repository != expected_repository {
        return Err(format!(
            "{path} repository {} does not match expected repository {expected_repository}",
            binding.repository
        ));
    }
    for (field, value, length) in [
        ("commit_sha", binding.commit_sha.as_deref(), 40usize),
        ("artifact_sha256", binding.artifact_sha256.as_deref(), 64usize),
        ("adapter_sha256", binding.adapter_sha256.as_deref(), 64usize),
        ("mapping_sha256", binding.mapping_sha256.as_deref(), 64usize),
    ] {
        if let Some(value) = value {
            if !is_lower_hex(value, length) {
                return Err(format!("{path}.{field} is not {length}-character lowercase hex"));
            }
        }
    }
    if binding.mapping_status == BindingState::Bound {
        require_text(binding.commit_sha.as_deref(), &format!("{path}.commit_sha"))?;
        require_text(binding.artifact_path.as_deref(), &format!("{path}.artifact_path"))?;
        require_text(binding.artifact_sha256.as_deref(), &format!("{path}.artifact_sha256"))?;
        require_text(binding.profile_id.as_deref(), &format!("{path}.profile_id"))?;
        require_text(
            binding.profile_version.as_deref(),
            &format!("{path}.profile_version"),
        )?;
        require_text(binding.adapter_id.as_deref(), &format!("{path}.adapter_id"))?;
        require_text(binding.adapter_sha256.as_deref(), &format!("{path}.adapter_sha256"))?;
        require_text(binding.mapping_sha256.as_deref(), &format!("{path}.mapping_sha256"))?;
        require_text(
            binding.owner_approval_ref.as_deref(),
            &format!("{path}.owner_approval_ref"),
        )?;
    }
    Ok(())
}

fn validate_mapping_review(review: &MappingReview, mapping_id: &str) -> Result<(), String> {
    if let Some(digest) = review.mapping_sha256.as_deref() {
        if !is_lower_hex(digest, 64) {
            return Err(format!("{mapping_id}.review.mapping_sha256 is invalid"));
        }
    }
    if review.status == BindingState::Bound {
        require_text(
            review.mapping_sha256.as_deref(),
            &format!("{mapping_id}.review.mapping_sha256"),
        )?;
        require_text(
            review.owner_approval_ref.as_deref(),
            &format!("{mapping_id}.review.owner_approval_ref"),
        )?;
        require_text(
            review.reviewed_by.as_deref(),
            &format!("{mapping_id}.review.reviewed_by"),
        )?;
    }
    Ok(())
}

fn validate_context(context: &DecisionContext, path: &str) -> Result<(), String> {
    for (field, digest) in [
        ("project_context_sha256", context.project_context_sha256.as_str()),
        (
            "functional_contract_sha256",
            context.functional_contract_sha256.as_str(),
        ),
        ("option_design_sha256", context.option_design_sha256.as_str()),
    ] {
        if !is_lower_hex(digest, 64) {
            return Err(format!("{path}.{field} is not a SHA-256 digest"));
        }
    }
    for (field, value) in [
        ("site_epoch", context.site_epoch.as_str()),
        ("coordinate_frame_id", context.coordinate_frame_id.as_str()),
        ("clock_domain", context.clock_domain.as_str()),
        ("currency", context.currency.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(format!("{path}.{field} must not be empty"));
        }
    }
    Ok(())
}

fn validate_unique_nonempty(values: &[String], path: &str) -> Result<(), String> {
    let mut unique = BTreeSet::new();
    for value in values {
        if value.trim().is_empty() {
            return Err(format!("{path} contains an empty value"));
        }
        if !unique.insert(value.as_str()) {
            return Err(format!("{path} contains duplicate value {value}"));
        }
    }
    Ok(())
}

fn require_text(value: Option<&str>, path: &str) -> Result<(), String> {
    match value {
        Some(value) if !value.trim().is_empty() => Ok(()),
        _ => Err(format!("{path} is required for a BOUND source mapping")),
    }
}

fn is_lower_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn evaluate_obligation(
    input: &BridgeInput,
    obligation_id: &str,
    acquisitions: &[&Acquisition],
) -> Result<ObligationResult, String> {
    if acquisitions.is_empty() {
        return Ok(unplanned_result(obligation_id));
    }

    let mut attempts = Vec::new();
    for acquisition in acquisitions {
        attempts.push(evaluate_acquisition(input, obligation_id, acquisition)?);
    }

    if let Some(success) = attempts.iter().find(|attempt| is_planned(&attempt.disposition)) {
        return Ok(success.clone());
    }

    attempts.sort_by_key(|attempt| disposition_priority(&attempt.disposition));
    let mut selected = attempts
        .into_iter()
        .next()
        .unwrap_or_else(|| unplanned_result(obligation_id));
    selected.obligation_id = obligation_id.to_owned();
    Ok(selected)
}

fn evaluate_acquisition(
    input: &BridgeInput,
    obligation_id: &str,
    acquisition: &Acquisition,
) -> Result<ObligationResult, String> {
    let mut result = ObligationResult {
        obligation_id: obligation_id.to_owned(),
        disposition: BridgeDisposition::NotPlanned,
        evidence_state: "PLANNED_NOT_OBSERVED",
        covered_by_acquisition_ids: vec![acquisition.acquisition_id.clone()],
        campaign_package_ids: Vec::new(),
        optimality_proven: false,
        reason_codes: Vec::new(),
    };

    if input.episode.binding.mapping_status == BindingState::Unbound
        || acquisition.binding.mapping_status == BindingState::Unbound
    {
        result.disposition = BridgeDisposition::Unbound;
        result.reason_codes.push("OWNER_PROFILE_OR_ADAPTER_UNBOUND".to_owned());
        return Ok(result);
    }

    if acquisition.episode_id != input.episode.episode_id
        || acquisition.context != input.episode.context
    {
        result.disposition = BridgeDisposition::ContextMismatch;
        result.reason_codes.push("DECISION_EPISODE_OR_CONTEXT_MISMATCH".to_owned());
        return Ok(result);
    }

    match evaluate_gates(&acquisition.kind, &acquisition.gates) {
        GateEvaluation::Blocked(gates) => {
            result.disposition = BridgeDisposition::BlockedAdmissibility;
            result.reason_codes = gates;
            return Ok(result);
        }
        GateEvaluation::Unknown(gates) => {
            result.disposition = BridgeDisposition::UnknownAdmissibility;
            result.reason_codes = gates;
            return Ok(result);
        }
        GateEvaluation::Allowed => {}
    }

    match acquisition.kind {
        AcquisitionKind::Remote => {
            result.disposition = BridgeDisposition::PlannedRemoteAcquisition;
            result.reason_codes.push("ADMISSIBLE_NONPHYSICAL_ACQUISITION_PLANNED".to_owned());
            Ok(result)
        }
        AcquisitionKind::Physical => {
            let relations: Vec<&PhysicalBinding> = input
                .physical_bindings
                .iter()
                .filter(|binding| binding.acquisition_id == acquisition.acquisition_id)
                .collect();
            if relations.is_empty() {
                result.disposition = BridgeDisposition::Unbound;
                result.reason_codes.push("PHYSICAL_ACQUISITION_ACTION_RELATION_UNBOUND".to_owned());
                return Ok(result);
            }

            let mut physical_attempts = Vec::new();
            for relation in relations {
                physical_attempts.push(evaluate_physical_relation(
                    input,
                    obligation_id,
                    acquisition,
                    relation,
                )?);
            }
            if let Some(success) = physical_attempts
                .iter()
                .find(|attempt| is_planned(&attempt.disposition))
            {
                return Ok(success.clone());
            }
            physical_attempts.sort_by_key(|attempt| disposition_priority(&attempt.disposition));
            Ok(physical_attempts
                .into_iter()
                .next()
                .unwrap_or(result))
        }
    }
}

enum GateEvaluation {
    Allowed,
    Blocked(Vec<String>),
    Unknown(Vec<String>),
}

fn evaluate_gates(kind: &AcquisitionKind, gates: &AdmissibilityGates) -> GateEvaluation {
    let required = [
        ("AUTHORITY", &gates.authority),
        ("RIGHTS_SOURCE_USE", &gates.rights_source_use),
        ("SCHEDULE", &gates.schedule),
        ("DATA_ACCESS", &gates.data_access),
        ("SOURCE_COMPETENCE", &gates.source_competence),
    ];
    let mut blocked = Vec::new();
    let mut unknown = Vec::new();

    for (name, gate) in required {
        match gate {
            GateState::Allowed => {}
            GateState::Blocked => blocked.push(format!("GATE_{name}_BLOCKED")),
            GateState::Unknown | GateState::NotApplicable => {
                unknown.push(format!("GATE_{name}_UNKNOWN_OR_NOT_APPLICABLE"))
            }
        }
    }

    let physical_gates = [
        ("SAFETY", &gates.safety),
        ("PHYSICAL_REALIZABILITY", &gates.physical_realizability),
    ];
    for (name, gate) in physical_gates {
        if *kind == AcquisitionKind::Remote && *gate == GateState::NotApplicable {
            continue;
        }
        match gate {
            GateState::Allowed => {}
            GateState::Blocked => blocked.push(format!("GATE_{name}_BLOCKED")),
            GateState::Unknown | GateState::NotApplicable => {
                unknown.push(format!("GATE_{name}_UNKNOWN_OR_NOT_APPLICABLE"))
            }
        }
    }

    if !blocked.is_empty() {
        GateEvaluation::Blocked(blocked)
    } else if !unknown.is_empty() {
        GateEvaluation::Unknown(unknown)
    } else {
        GateEvaluation::Allowed
    }
}

fn evaluate_physical_relation(
    input: &BridgeInput,
    obligation_id: &str,
    acquisition: &Acquisition,
    relation: &PhysicalBinding,
) -> Result<ObligationResult, String> {
    let mut result = ObligationResult {
        obligation_id: obligation_id.to_owned(),
        disposition: BridgeDisposition::NotPlanned,
        evidence_state: "PLANNED_NOT_OBSERVED",
        covered_by_acquisition_ids: vec![acquisition.acquisition_id.clone()],
        campaign_package_ids: vec![relation.campaign_package_id.clone()],
        optimality_proven: false,
        reason_codes: Vec::new(),
    };

    if relation.episode_id != input.episode.episode_id
        || relation.context != input.episode.context
        || relation.context != acquisition.context
    {
        result.disposition = BridgeDisposition::ContextMismatch;
        result.reason_codes.push("PHYSICAL_BINDING_CONTEXT_MISMATCH".to_owned());
        return Ok(result);
    }
    if relation.review.status == BindingState::Unbound {
        result.disposition = BridgeDisposition::Unbound;
        result.reason_codes.push("OWNER_REVIEWED_ACQUISITION_ACTION_MAPPING_UNBOUND".to_owned());
        return Ok(result);
    }

    let campaign = input
        .campaigns
        .iter()
        .find(|candidate| candidate.package_id == relation.campaign_package_id)
        .expect("campaign reference validated");
    if campaign.binding.mapping_status == BindingState::Unbound {
        result.disposition = BridgeDisposition::Unbound;
        result.reason_codes.push("SYMTROPY_CAMPAIGN_PROFILE_UNBOUND".to_owned());
        return Ok(result);
    }
    if campaign.context != input.episode.context {
        result.disposition = BridgeDisposition::ContextMismatch;
        result.reason_codes.push("SYMTROPY_OPTION_OR_SITE_CONTEXT_MISMATCH".to_owned());
        return Ok(result);
    }

    let action = campaign
        .actions
        .iter()
        .find(|candidate| candidate.action_id == relation.action_id)
        .expect("action reference validated");
    if action.context != input.episode.context {
        result.disposition = BridgeDisposition::ContextMismatch;
        result.reason_codes.push("SYMTROPY_ACTION_CONTEXT_MISMATCH".to_owned());
        return Ok(result);
    }
    if !action.covers_obligation_ids.iter().any(|id| id == obligation_id) {
        result.disposition = BridgeDisposition::NotPlanned;
        result.reason_codes.push("CAMPAIGN_ACTION_DOES_NOT_COVER_OBLIGATION".to_owned());
        return Ok(result);
    }

    match campaign.search_status {
        CampaignSearchStatus::CompleteOptimal => {
            result.disposition = BridgeDisposition::PlannedPhysicalObservationOptimal;
            result.optimality_proven = true;
            result.reason_codes.push("FEASIBLE_COMPLETE_CAMPAIGN_OPTIMALITY_PROVEN".to_owned());
        }
        CampaignSearchStatus::CompleteFeasibleNotOptimal => {
            result.disposition =
                BridgeDisposition::PlannedPhysicalObservationNotOptimalityProven;
            result.reason_codes.push("FEASIBLE_COMPLETE_CAMPAIGN_OPTIMALITY_NOT_PROVEN".to_owned());
        }
        CampaignSearchStatus::Incomplete => {
            result.disposition = BridgeDisposition::CampaignIncomplete;
            result.reason_codes.push("CAMPAIGN_DOES_NOT_COVER_ALL_REQUIRED_WORK".to_owned());
        }
        CampaignSearchStatus::ProvenInfeasible => {
            result.disposition = BridgeDisposition::CampaignInfeasible;
            result.reason_codes.push("CAMPAIGN_INFEASIBILITY_PROVEN".to_owned());
        }
        CampaignSearchStatus::SearchIncomplete => {
            result.disposition = BridgeDisposition::CampaignSearchIncomplete;
            result.reason_codes.push("CAMPAIGN_SEARCH_INCOMPLETE_NO_OPTIMALITY_CLAIM".to_owned());
        }
    }
    Ok(result)
}

fn is_planned(disposition: &BridgeDisposition) -> bool {
    matches!(
        disposition,
        BridgeDisposition::PlannedRemoteAcquisition
            | BridgeDisposition::PlannedPhysicalObservationOptimal
            | BridgeDisposition::PlannedPhysicalObservationNotOptimalityProven
    )
}

fn disposition_priority(disposition: &BridgeDisposition) -> u8 {
    match disposition {
        BridgeDisposition::BlockedAdmissibility => 0,
        BridgeDisposition::UnknownAdmissibility => 1,
        BridgeDisposition::Unbound => 2,
        BridgeDisposition::ContextMismatch => 3,
        BridgeDisposition::CampaignInfeasible => 4,
        BridgeDisposition::CampaignIncomplete => 5,
        BridgeDisposition::CampaignSearchIncomplete => 6,
        BridgeDisposition::NotPlanned => 7,
        BridgeDisposition::PlannedRemoteAcquisition
        | BridgeDisposition::PlannedPhysicalObservationOptimal
        | BridgeDisposition::PlannedPhysicalObservationNotOptimalityProven => 8,
    }
}

fn unplanned_result(obligation_id: &str) -> ObligationResult {
    ObligationResult {
        obligation_id: obligation_id.to_owned(),
        disposition: BridgeDisposition::NotPlanned,
        evidence_state: "PLANNED_NOT_OBSERVED",
        covered_by_acquisition_ids: Vec::new(),
        campaign_package_ids: Vec::new(),
        optimality_proven: false,
        reason_codes: vec!["NO_DECLARED_ACQUISITION_FOR_OBLIGATION".to_owned()],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digest(character: char) -> String {
        std::iter::repeat_n(character, 64).collect()
    }

    fn source_binding(owner: &str, repository: &str, bound: bool) -> OwnerBinding {
        OwnerBinding {
            owner: owner.to_owned(),
            repository: repository.to_owned(),
            commit_sha: bound.then(|| "a".repeat(40)),
            artifact_path: bound.then(|| "synthetic/fixture.json".to_owned()),
            artifact_sha256: bound.then(|| digest('b')),
            profile_id: bound.then(|| format!("test.{owner}.profile")),
            profile_version: bound.then(|| "0.1.0".to_owned()),
            adapter_id: bound.then(|| "test.highveld.adapter".to_owned()),
            adapter_sha256: bound.then(|| digest('c')),
            mapping_status: if bound {
                BindingState::Bound
            } else {
                BindingState::Unbound
            },
            mapping_sha256: bound.then(|| digest('d')),
            owner_approval_ref: bound.then(|| "TEST-OWNER-REVIEW-001".to_owned()),
        }
    }

    fn context() -> DecisionContext {
        DecisionContext {
            project_context_sha256: digest('1'),
            functional_contract_sha256: digest('2'),
            option_design_sha256: digest('3'),
            site_epoch: "SITE-EPOCH-7".to_owned(),
            coordinate_frame_id: "FRAME-LOCAL-ENU".to_owned(),
            clock_domain: "UTC-PANEL-LAB".to_owned(),
            currency: "USD".to_owned(),
        }
    }

    fn allowed_gates(kind: AcquisitionKind) -> AdmissibilityGates {
        AdmissibilityGates {
            authority: GateState::Allowed,
            rights_source_use: GateState::Allowed,
            safety: if kind == AcquisitionKind::Remote {
                GateState::NotApplicable
            } else {
                GateState::Allowed
            },
            schedule: GateState::Allowed,
            physical_realizability: if kind == AcquisitionKind::Remote {
                GateState::NotApplicable
            } else {
                GateState::Allowed
            },
            data_access: GateState::Allowed,
            source_competence: GateState::Allowed,
        }
    }

    fn base_input(kind: AcquisitionKind) -> BridgeInput {
        let ctx = context();
        BridgeInput {
            profile_id: BRIDGE_PROFILE_ID.to_owned(),
            version: BRIDGE_VERSION.to_owned(),
            status: BRIDGE_STATUS.to_owned(),
            claim_ceiling: BRIDGE_CLAIM_CEILING.to_owned(),
            episode: DecisionEpisode {
                binding: source_binding("mycelix", "Luminous-Dynamics/mycelix", true),
                episode_id: "EPISODE-HPT-001".to_owned(),
                as_of_unix: 1_794_744_000,
                context: ctx.clone(),
                unresolved_obligation_ids: vec!["OBLIGATION-METROLOGY-001".to_owned()],
            },
            acquisitions: vec![Acquisition {
                binding: source_binding("mycelix", "Luminous-Dynamics/mycelix", true),
                acquisition_id: "ACQUISITION-001".to_owned(),
                episode_id: "EPISODE-HPT-001".to_owned(),
                context: ctx.clone(),
                kind: kind.clone(),
                covers_obligation_ids: vec!["OBLIGATION-METROLOGY-001".to_owned()],
                gates: allowed_gates(kind),
            }],
            campaigns: Vec::new(),
            physical_bindings: Vec::new(),
        }
    }

    fn add_complete_campaign(input: &mut BridgeInput, status: CampaignSearchStatus) {
        let ctx = context();
        input.campaigns.push(Campaign {
            binding: source_binding("symtropy", "Luminous-Dynamics/symtropy", true),
            package_id: "OPTION-PACKAGE-001".to_owned(),
            context: ctx.clone(),
            search_status: status,
            actions: vec![CampaignAction {
                action_id: "OBSERVATION-ACTION-001".to_owned(),
                context: ctx.clone(),
                covers_obligation_ids: vec!["OBLIGATION-METROLOGY-001".to_owned()],
            }],
        });
        input.physical_bindings.push(PhysicalBinding {
            mapping_id: "OWNER-REVIEWED-MAPPING-001".to_owned(),
            acquisition_id: "ACQUISITION-001".to_owned(),
            episode_id: "EPISODE-HPT-001".to_owned(),
            campaign_package_id: "OPTION-PACKAGE-001".to_owned(),
            action_id: "OBSERVATION-ACTION-001".to_owned(),
            context: ctx,
            review: MappingReview {
                status: BindingState::Bound,
                mapping_sha256: Some(digest('e')),
                owner_approval_ref: Some("OWNER-APPROVAL-RELATION-001".to_owned()),
                reviewed_by: Some("test-owner-reviewer".to_owned()),
            },
        });
    }

    fn evaluate(input: &BridgeInput) -> BridgeReport {
        let bytes = serde_json::to_vec(input).unwrap();
        evaluate_bridge_bytes(&bytes).unwrap()
    }

    fn one_result(report: &BridgeReport) -> &ObligationResult {
        assert_eq!(report.obligation_results.len(), 1);
        &report.obligation_results[0]
    }

    #[test]
    fn admissible_remote_acquisition_does_not_need_a_physical_campaign() {
        let input = base_input(AcquisitionKind::Remote);
        let report = evaluate(&input);
        assert_eq!(
            one_result(&report).disposition,
            BridgeDisposition::PlannedRemoteAcquisition
        );
        assert_eq!(one_result(&report).evidence_state, "PLANNED_NOT_OBSERVED");
        assert_eq!(report.physical_observations, 0);
        assert_eq!(report.qualified_obligations, 0);
    }

    #[test]
    fn bound_physical_campaign_plans_the_exact_obligation() {
        let mut input = base_input(AcquisitionKind::Physical);
        add_complete_campaign(&mut input, CampaignSearchStatus::CompleteOptimal);
        let report = evaluate(&input);
        assert_eq!(
            one_result(&report).disposition,
            BridgeDisposition::PlannedPhysicalObservationOptimal
        );
        assert!(one_result(&report).optimality_proven);
        assert_eq!(report.planned_obligations, 1);
    }

    #[test]
    fn feasible_complete_but_nonoptimal_campaign_keeps_nonoptimal_status() {
        let mut input = base_input(AcquisitionKind::Physical);
        add_complete_campaign(&mut input, CampaignSearchStatus::CompleteFeasibleNotOptimal);
        let report = evaluate(&input);
        assert_eq!(
            one_result(&report).disposition,
            BridgeDisposition::PlannedPhysicalObservationNotOptimalityProven
        );
        assert!(!one_result(&report).optimality_proven);
    }

    #[test]
    fn authority_block_is_non_compensatory() {
        let mut input = base_input(AcquisitionKind::Remote);
        input.acquisitions[0].gates.authority = GateState::Blocked;
        assert_eq!(
            one_result(&evaluate(&input)).disposition,
            BridgeDisposition::BlockedAdmissibility
        );
    }

    #[test]
    fn unknown_rights_are_not_admissible() {
        let mut input = base_input(AcquisitionKind::Remote);
        input.acquisitions[0].gates.rights_source_use = GateState::Unknown;
        assert_eq!(
            one_result(&evaluate(&input)).disposition,
            BridgeDisposition::UnknownAdmissibility
        );
    }

    #[test]
    fn physical_safety_block_cannot_be_offset_by_campaign_coverage() {
        let mut input = base_input(AcquisitionKind::Physical);
        add_complete_campaign(&mut input, CampaignSearchStatus::CompleteOptimal);
        input.acquisitions[0].gates.safety = GateState::Blocked;
        assert_eq!(
            one_result(&evaluate(&input)).disposition,
            BridgeDisposition::BlockedAdmissibility
        );
    }

    #[test]
    fn unbound_episode_never_closes_obligations() {
        let mut input = base_input(AcquisitionKind::Remote);
        input.episode.binding = source_binding("mycelix", "Luminous-Dynamics/mycelix", false);
        let report = evaluate(&input);
        assert_eq!(one_result(&report).disposition, BridgeDisposition::Unbound);
        assert_eq!(report.planned_obligations, 0);
    }

    #[test]
    fn unbound_acquisition_never_closes_obligations() {
        let mut input = base_input(AcquisitionKind::Remote);
        input.acquisitions[0].binding =
            source_binding("mycelix", "Luminous-Dynamics/mycelix", false);
        assert_eq!(
            one_result(&evaluate(&input)).disposition,
            BridgeDisposition::Unbound
        );
    }

    #[test]
    fn physical_acquisition_without_reviewed_action_binding_is_unbound() {
        let input = base_input(AcquisitionKind::Physical);
        assert_eq!(
            one_result(&evaluate(&input)).disposition,
            BridgeDisposition::Unbound
        );
    }

    #[test]
    fn site_epoch_or_frame_mismatch_prevents_physical_coverage() {
        let mut input = base_input(AcquisitionKind::Physical);
        add_complete_campaign(&mut input, CampaignSearchStatus::CompleteOptimal);
        input.campaigns[0].context.site_epoch = "SITE-EPOCH-8".to_owned();
        assert_eq!(
            one_result(&evaluate(&input)).disposition,
            BridgeDisposition::ContextMismatch
        );
    }

    #[test]
    fn incomplete_campaign_does_not_close_obligation() {
        let mut input = base_input(AcquisitionKind::Physical);
        add_complete_campaign(&mut input, CampaignSearchStatus::Incomplete);
        assert_eq!(
            one_result(&evaluate(&input)).disposition,
            BridgeDisposition::CampaignIncomplete
        );
        assert_eq!(one_result(&evaluate(&input)).evidence_state, "PLANNED_NOT_OBSERVED");
    }

    #[test]
    fn proven_infeasible_campaign_does_not_close_obligation() {
        let mut input = base_input(AcquisitionKind::Physical);
        add_complete_campaign(&mut input, CampaignSearchStatus::ProvenInfeasible);
        assert_eq!(
            one_result(&evaluate(&input)).disposition,
            BridgeDisposition::CampaignInfeasible
        );
    }

    #[test]
    fn exhausted_campaign_search_is_not_reported_as_infeasible_or_optimal() {
        let mut input = base_input(AcquisitionKind::Physical);
        add_complete_campaign(&mut input, CampaignSearchStatus::SearchIncomplete);
        assert_eq!(
            one_result(&evaluate(&input)).disposition,
            BridgeDisposition::CampaignSearchIncomplete
        );
    }

    #[test]
    fn unbound_campaign_profile_does_not_close_obligation() {
        let mut input = base_input(AcquisitionKind::Physical);
        add_complete_campaign(&mut input, CampaignSearchStatus::CompleteOptimal);
        input.campaigns[0].binding =
            source_binding("symtropy", "Luminous-Dynamics/symtropy", false);
        assert_eq!(
            one_result(&evaluate(&input)).disposition,
            BridgeDisposition::Unbound
        );
    }

    #[test]
    fn unbound_physical_mapping_does_not_close_obligation() {
        let mut input = base_input(AcquisitionKind::Physical);
        add_complete_campaign(&mut input, CampaignSearchStatus::CompleteOptimal);
        input.physical_bindings[0].review.status = BindingState::Unbound;
        input.physical_bindings[0].review.mapping_sha256 = None;
        input.physical_bindings[0].review.owner_approval_ref = None;
        input.physical_bindings[0].review.reviewed_by = None;
        assert_eq!(
            one_result(&evaluate(&input)).disposition,
            BridgeDisposition::Unbound
        );
    }

    #[test]
    fn remote_acquisition_cannot_be_promoted_into_a_physical_scan() {
        let mut input = base_input(AcquisitionKind::Remote);
        add_complete_campaign(&mut input, CampaignSearchStatus::CompleteOptimal);
        assert!(evaluate_bridge_bytes(&serde_json::to_vec(&input).unwrap())
            .unwrap_err()
            .contains("remote acquisition"));
    }

    #[test]
    fn duplicate_acquisition_ids_are_rejected() {
        let mut input = base_input(AcquisitionKind::Remote);
        input.acquisitions.push(input.acquisitions[0].clone());
        assert!(evaluate_bridge_bytes(&serde_json::to_vec(&input).unwrap())
            .unwrap_err()
            .contains("duplicate acquisition_id"));
    }

    #[test]
    fn unknown_obligation_references_are_rejected() {
        let mut input = base_input(AcquisitionKind::Remote);
        input.acquisitions[0].covers_obligation_ids.push("NOT-IN-EPISODE".to_owned());
        assert!(evaluate_bridge_bytes(&serde_json::to_vec(&input).unwrap())
            .unwrap_err()
            .contains("unknown obligation"));
    }

    #[test]
    fn cross_owner_repository_misbinding_is_rejected() {
        let mut input = base_input(AcquisitionKind::Physical);
        add_complete_campaign(&mut input, CampaignSearchStatus::CompleteOptimal);
        input.campaigns[0].binding.repository = "Luminous-Dynamics/mycelix".to_owned();
        assert!(evaluate_bridge_bytes(&serde_json::to_vec(&input).unwrap())
            .unwrap_err()
            .contains("repository"));
    }

    #[test]
    fn input_order_does_not_change_semantic_result_but_changes_digest() {
        let mut input = base_input(AcquisitionKind::Physical);
        add_complete_campaign(&mut input, CampaignSearchStatus::CompleteOptimal);
        let first = evaluate(&input);

        input.acquisitions.reverse();
        input.campaigns.reverse();
        input.physical_bindings.reverse();
        let second = evaluate(&input);
        assert_eq!(first.planned_obligations, second.planned_obligations);
        assert_eq!(
            first.obligation_results[0].disposition,
            second.obligation_results[0].disposition
        );
        assert_ne!(first.input_sha256, second.input_sha256);
    }

    #[test]
    fn all_positive_results_remain_non_authorizing() {
        let mut input = base_input(AcquisitionKind::Physical);
        add_complete_campaign(&mut input, CampaignSearchStatus::CompleteOptimal);
        let report = evaluate(&input);
        assert_eq!(report.source_bytes_verified, false);
        assert_eq!(report.signatures_verified, false);
        assert_eq!(report.physical_observations, 0);
        assert_eq!(report.qualified_obligations, 0);
        assert_eq!(report.authority_effects, "NONE");
        assert_eq!(report.claim_ceiling, BRIDGE_CLAIM_CEILING);
    }
}
