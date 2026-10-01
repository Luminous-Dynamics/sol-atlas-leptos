// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Projection-only contract for the Sol Atlas / Mycelix boundary.
//!
//! This module intentionally contains no evidence qualification logic. Mycelix
//! remains the semantic authority; Atlas receives an already-qualified
//! projection and renders its claim ceiling without upgrading it.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct AtlasEvidenceProjectionV1 {
    pub projection_id: String,
    pub entity_ref: String,
    pub claim_ref: String,
    pub claim_kind: ClaimKind,
    pub statement: String,
    pub evidence_refs: Vec<String>,
    pub derivation_ref: Option<String>,
    pub frontier_ref: String,
    pub qualification_ref: String,
    pub qualification: EpistemicState,
    pub contradictions: Vec<ContradictionRef>,
    pub source_snapshot_refs: Vec<String>,
    pub visibility: Visibility,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ContradictionRef {
    pub claim_ref: String,
    pub evidence_refs: Vec<String>,
    pub summary: String,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum ClaimKind {
    Observed,
    Derived,
    Hypothesis,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum EpistemicState {
    Qualified,
    ObservedUnqualified,
    Conflicting,
    Stale,
    Protected,
    Unknown,
    FutureInaccessible,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum Visibility {
    Public,
    Redacted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LineageCompleteness {
    Complete,
    Incomplete,
}

/// Typed inspection target for a declared projection dependency.
///
/// These references are addresses, not proofs of existence. A projection is
/// only authoritative when the referenced object is resolved by the upstream
/// semantic authority (Mycelix); Atlas must never invent a missing node.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LineageNodeRef {
    Projection(String),
    Claim(String),
    Evidence(String),
    SourceSnapshot(String),
    Qualification(String),
    Frontier(String),
    Derivation(String),
    ReasoningReceipt(String),
}

impl LineageNodeRef {
    pub fn key(&self) -> &'static str {
        match self {
            Self::Projection(_) => "projection",
            Self::Claim(_) => "claim",
            Self::Evidence(_) => "evidence",
            Self::SourceSnapshot(_) => "source",
            Self::Qualification(_) => "qualification",
            Self::Frontier(_) => "frontier",
            Self::Derivation(_) => "derivation",
            Self::ReasoningReceipt(_) => "reasoning",
        }
    }

    pub fn id(&self) -> &str {
        match self {
            Self::Projection(id)
            | Self::Claim(id)
            | Self::Evidence(id)
            | Self::SourceSnapshot(id)
            | Self::Qualification(id)
            | Self::Frontier(id)
            | Self::Derivation(id)
            | Self::ReasoningReceipt(id) => id,
        }
    }

    pub fn parse(value: Option<&str>) -> Option<Self> {
        let (kind, id) = value?.split_once(':')?;
        if id.trim().is_empty() {
            return None;
        }
        // IDs themselves may contain colons; split_once intentionally preserves
        // the complete suffix as the canonical identifier.
        match kind {
            "projection" => Some(Self::Projection(id.to_string())),
            "claim" => Some(Self::Claim(id.to_string())),
            "evidence" => Some(Self::Evidence(id.to_string())),
            "source" => Some(Self::SourceSnapshot(id.to_string())),
            "qualification" => Some(Self::Qualification(id.to_string())),
            "frontier" => Some(Self::Frontier(id.to_string())),
            "derivation" => Some(Self::Derivation(id.to_string())),
            "reasoning" => Some(Self::ReasoningReceipt(id.to_string())),
            _ => None,
        }
    }

    pub fn query_value(&self) -> String {
        format!("{}:{}", self.key(), self.id())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LineageResolution {
    /// The identifier is declared by the local projection; this is not an authoritative fetch.
    Declared,
    Unavailable,
    Protected,
    Incomplete,
}

impl AtlasEvidenceProjectionV1 {
    /// Resolves only references declared by this projection. No current-state
    /// fallback or inferred relationship is permitted.
    pub fn resolve_lineage_node(&self, node: &LineageNodeRef) -> LineageResolution {
        match node {
            LineageNodeRef::Projection(id) => {
                if id == &self.projection_id { LineageResolution::Declared } else { LineageResolution::Unavailable }
            }
            LineageNodeRef::Claim(id) => {
                if id == &self.claim_ref { LineageResolution::Declared } else { LineageResolution::Unavailable }
            }
            LineageNodeRef::Evidence(id) => {
                if self.evidence_refs.iter().any(|ref_id| ref_id == id) {
                    if matches!(self.visibility, Visibility::Redacted) {
                        LineageResolution::Protected
                    } else {
                        LineageResolution::Declared
                    }
                } else {
                    LineageResolution::Unavailable
                }
            }
            LineageNodeRef::SourceSnapshot(id) => {
                if self.source_snapshot_refs.iter().any(|ref_id| ref_id == id) {
                    LineageResolution::Declared
                } else {
                    LineageResolution::Unavailable
                }
            }
            LineageNodeRef::Qualification(id) => {
                if id == &self.qualification_ref {
                    match self.qualification {
                        EpistemicState::Protected => LineageResolution::Protected,
                        _ => LineageResolution::Declared,
                    }
                } else {
                    LineageResolution::Unavailable
                }
            }
            LineageNodeRef::Frontier(id) => {
                if id == &self.frontier_ref { LineageResolution::Declared } else { LineageResolution::Unavailable }
            }
            LineageNodeRef::Derivation(id) => {
                if matches!(self.claim_kind, ClaimKind::Derived) && self.derivation_ref.as_deref() == Some(id) {
                    LineageResolution::Declared
                } else {
                    LineageResolution::Unavailable
                }
            }
            LineageNodeRef::ReasoningReceipt(id) => {
                if matches!(self.claim_kind, ClaimKind::Hypothesis) && self.derivation_ref.as_deref() == Some(id) {
                    LineageResolution::Declared
                } else {
                    LineageResolution::Unavailable
                }
            }
        }
    }

    pub fn lineage_nodes(&self) -> Vec<LineageNodeRef> {
        let mut nodes = vec![
            LineageNodeRef::Projection(self.projection_id.clone()),
            LineageNodeRef::Claim(self.claim_ref.clone()),
        ];
        nodes.extend(self.evidence_refs.iter().cloned().map(LineageNodeRef::Evidence));
        nodes.extend(self.source_snapshot_refs.iter().cloned().map(LineageNodeRef::SourceSnapshot));
        nodes.push(LineageNodeRef::Qualification(self.qualification_ref.clone()));
        nodes.push(LineageNodeRef::Frontier(self.frontier_ref.clone()));
        if let Some(id) = &self.derivation_ref {
            nodes.push(match self.claim_kind {
                ClaimKind::Hypothesis => LineageNodeRef::ReasoningReceipt(id.clone()),
                _ => LineageNodeRef::Derivation(id.clone()),
            });
        }
        nodes
    }
}

impl AtlasEvidenceProjectionV1 {
    /// The UI may display this claim, but it may never render it as stronger
    /// than its source qualification or claim kind.
    pub fn can_render_statement(&self) -> bool {
        !matches!(self.visibility, Visibility::Redacted)
            && !self.evidence_refs.is_empty()
            && self.evidence_refs.iter().all(|id| !id.trim().is_empty())
            && !self.frontier_ref.trim().is_empty()
            && !self.claim_ref.trim().is_empty()
    }

    /// Stable identifiers required for a reversible "why is this shown?"
    /// interaction.
    pub fn lineage_ids(&self) -> impl Iterator<Item = &str> {
        self.evidence_refs
            .iter()
            .map(String::as_str)
            .chain(self.source_snapshot_refs.iter().map(String::as_str))
            .chain(self.derivation_ref.iter().map(String::as_str))
    }

    /// Returns whether the projection exposes enough identifiers to make its
    /// semantic lineage inspectable without inventing missing dependencies.
    pub fn lineage_completeness(&self) -> LineageCompleteness {
        let has_core = !self.claim_ref.trim().is_empty()
            && !self.frontier_ref.trim().is_empty()
            && !self.qualification_ref.trim().is_empty()
            && !self.evidence_refs.is_empty()
            && self.evidence_refs.iter().all(|id| !id.trim().is_empty())
            && !self.source_snapshot_refs.is_empty()
            && self.source_snapshot_refs.iter().all(|id| !id.trim().is_empty());

        let derivation_required = matches!(self.claim_kind, ClaimKind::Derived | ClaimKind::Hypothesis);
        let has_derivation = self
            .derivation_ref
            .as_ref()
            .map(|id| !id.trim().is_empty())
            .unwrap_or(false);

        if has_core && (!derivation_required || has_derivation) {
            LineageCompleteness::Complete
        } else {
            LineageCompleteness::Incomplete
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> AtlasEvidenceProjectionV1 {
        AtlasEvidenceProjectionV1 {
            projection_id: "view:entity:01".into(),
            entity_ref: "entity:fin:ns-energy-01".into(),
            claim_ref: "claim:obs:7f31".into(),
            claim_kind: ClaimKind::Observed,
            statement: "Observed statement".into(),
            evidence_refs: vec!["evidence:4a90".into()],
            derivation_ref: None,
            frontier_ref: "ef:demo:9d7b".into(),
            qualification_ref: "profile:fin-001c0".into(),
            qualification: EpistemicState::Qualified,
            contradictions: vec![],
            source_snapshot_refs: vec!["source:filing:v3".into()],
            visibility: Visibility::Public,
        }
    }

    #[test]
    fn projection_requires_lineage_to_render() {
        assert!(fixture().can_render_statement());

        let mut missing_evidence = fixture();
        missing_evidence.evidence_refs.clear();
        assert!(!missing_evidence.can_render_statement());

        let mut missing_frontier = fixture();
        missing_frontier.frontier_ref.clear();
        assert!(!missing_frontier.can_render_statement());
    }

    #[test]
    fn redacted_projection_never_renders_as_public_claim() {
        let mut redacted = fixture();
        redacted.visibility = Visibility::Redacted;
        assert!(!redacted.can_render_statement());
    }

    #[test]
    fn lineage_is_stable_and_contains_material_dependencies() {
        let projection = fixture();
        let ids: Vec<_> = projection.lineage_ids().collect();
        assert_eq!(
            ids,
            vec!["evidence:4a90", "source:filing:v3"]
        );
    }

    #[test]
    fn lineage_completeness_requires_derivation_for_derived_claims() {
        let mut projection = fixture();
        assert_eq!(projection.lineage_completeness(), LineageCompleteness::Complete);

        projection.claim_kind = ClaimKind::Derived;
        assert_eq!(projection.lineage_completeness(), LineageCompleteness::Incomplete);

        projection.derivation_ref = Some("derivation:fin:42ac".into());
        assert_eq!(projection.lineage_completeness(), LineageCompleteness::Complete);
    }

    #[test]
    fn lineage_completeness_rejects_missing_source_snapshot() {
        let mut projection = fixture();
        projection.source_snapshot_refs.clear();
        assert_eq!(projection.lineage_completeness(), LineageCompleteness::Incomplete);
    }

    #[test]
    fn lineage_nodes_are_typed_and_resolve_without_inference() {
        let projection = fixture();
        let claim = LineageNodeRef::Claim("claim:obs:7f31".into());
        assert_eq!(projection.resolve_lineage_node(&claim), LineageResolution::Declared);

        let missing = LineageNodeRef::Evidence("evidence:not-declared".into());
        assert_eq!(projection.resolve_lineage_node(&missing), LineageResolution::Unavailable);

        let nodes = projection.lineage_nodes();
        assert!(nodes.iter().any(|node| matches!(node, LineageNodeRef::Qualification(id) if id == "profile:fin-001c0")));
    }

    #[test]
    fn lineage_node_parser_preserves_colons_in_ids() {
        let node = LineageNodeRef::parse(Some("evidence:provider:observation:17")).unwrap();
        assert_eq!(node.id(), "provider:observation:17");
        assert_eq!(node.query_value(), "evidence:provider:observation:17");
    }

    #[test]
    fn protected_evidence_never_becomes_resolved_publicly() {
        let mut projection = fixture();
        projection.visibility = Visibility::Redacted;
        let evidence = LineageNodeRef::Evidence("evidence:4a90".into());
        assert_eq!(projection.resolve_lineage_node(&evidence), LineageResolution::Protected);
    }

    #[test]
    fn hypothesis_derivation_is_exposed_as_reasoning_receipt() {
        let mut projection = fixture();
        projection.claim_kind = ClaimKind::Hypothesis;
        projection.derivation_ref = Some("reasoning:symthaea:91e2".into());
        let nodes = projection.lineage_nodes();
        assert!(nodes.iter().any(|node| matches!(node, LineageNodeRef::ReasoningReceipt(id) if id == "reasoning:symthaea:91e2")));
    }

    #[test]
    fn hypothesis_remains_a_distinct_claim_kind() {
        let mut projection = fixture();
        projection.claim_kind = ClaimKind::Hypothesis;
        assert_eq!(projection.claim_kind, ClaimKind::Hypothesis);
    }

    #[test]
    fn malformed_explicit_query_identity_never_becomes_absence() {
        let query = TerminalQueryV1::from_url_parts(
            Some("entity:fin:ns-energy-01".into()),
            Some("   ".into()),
            Some("ef:demo:9d7b".into()),
            None,
            None,
            None,
            Some("evidence".into()),
        );
        assert_eq!(query.validity(), TerminalQueryValidity::Malformed);
        assert!(!query.is_replay_addressable());
        assert_eq!(query.replay_resolution_state(None), ReplayResolutionState::Unresolved);
    }

    #[test]
    fn valid_partial_query_identity_remains_distinct_from_malformed() {
        let query = TerminalQueryV1::from_url_parts(
            Some("entity:fin:ns-energy-01".into()),
            None,
            Some("ef:demo:9d7b".into()),
            None,
            None,
            None,
            Some("evidence".into()),
        );
        assert_eq!(query.validity(), TerminalQueryValidity::Valid);
        assert_eq!(query.replay_resolution_state(None), ReplayResolutionState::Addressable);
        assert!(!query.is_replay_addressable());
    }

    #[test]
    fn replay_target_construction_rejects_malformed_query() {
        let query = TerminalQueryV1::from_url_parts(
            Some("entity:fin:ns-energy-01".into()),
            Some("   ".into()),
            Some("ef:demo:9d7b".into()),
            Some("profile:terminal:v1".into()),
            Some("reasoning:baseline:v1".into()),
            Some("model:symthaea:v1".into()),
            Some("evidence".into()),
        );

        assert_eq!(
            ReplayTargetV1::try_from_terminal_query(&query),
            Err(ReplayTargetConstructionError::MalformedQuery)
        );
        assert!(query.replay_target().is_none());
    }

    #[test]
    fn replay_target_construction_rejects_incomplete_query() {
        let query = TerminalQueryV1::from_url_parts(
            Some("entity:fin:ns-energy-01".into()),
            Some("claim:obs:7f31".into()),
            Some("ef:demo:9d7b".into()),
            None,
            None,
            None,
            Some("evidence".into()),
        );

        assert_eq!(
            ReplayTargetV1::try_from_terminal_query(&query),
            Err(ReplayTargetConstructionError::IncompleteQuery)
        );
        assert!(query.replay_target().is_none());
    }

    #[test]
    fn replay_target_construction_preserves_complete_terminal_identity() {
        let query = TerminalQueryV1::from_url_parts(
            Some("entity:fin:ns-energy-01".into()),
            Some("claim:obs:7f31".into()),
            Some("ef:demo:9d7b".into()),
            Some("profile:terminal:v1".into()),
            Some("reasoning:baseline:v1".into()),
            Some("model:symthaea:v1".into()),
            Some("evidence".into()),
        );

        let target = ReplayTargetV1::try_from_terminal_query(&query)
            .expect("complete query should construct a replay target");

        assert_eq!(target.entity_ref, "entity:fin:ns-energy-01");
        assert_eq!(target.claim_ref, "claim:obs:7f31");
        assert_eq!(target.frontier_ref, "ef:demo:9d7b");
        assert_eq!(target.projection_profile, "profile:terminal:v1");
        assert_eq!(target.reasoning_program, "reasoning:baseline:v1");
        assert_eq!(target.model_version, "model:symthaea:v1");
    }

    #[test]
    fn replay_target_exposes_distinct_navigation_and_execution_contexts() {
        let query = TerminalQueryV1::from_url_parts(
            Some("entity:fin:ns-energy-01".into()),
            Some("claim:obs:7f31".into()),
            Some("ef:demo:9d7b".into()),
            Some("profile:terminal:v1".into()),
            Some("reasoning:baseline:v1".into()),
            Some("model:symthaea:v1".into()),
            Some("evidence".into()),
        );
        let target = query.replay_target().expect("complete replay target");

        let navigation = target.navigation_identity().expect("navigation identity");
        assert_eq!(navigation.entity_ref(), "entity:fin:ns-energy-01");
        assert_eq!(navigation.claim_ref(), "claim:obs:7f31");

        let context = target.execution_context().expect("execution context");
        assert_eq!(context.frontier_ref(), "ef:demo:9d7b");
        assert_eq!(context.projection_profile(), "profile:terminal:v1");
        assert_eq!(context.reasoning_program(), "reasoning:baseline:v1");
        assert_eq!(context.model_version(), "model:symthaea:v1");

        let manifest = DependencyManifestV1::from_execution_context(context);
        assert_eq!(manifest.frontier_ref, "ef:demo:9d7b");
        assert_eq!(manifest.model_versions, vec!["model:symthaea:v1"]);
        assert!(manifest.evidence_roots.is_empty());
        assert!(manifest.qualification_profile.is_none());
    }

    #[test]
    fn terminal_query_preserves_replay_identity() {
        let query = TerminalQueryV1::from_url_parts(
            Some("entity:fin:ns-energy-01".into()),
            Some("claim:obs:7f31".into()),
            Some("ef:demo:9d7b".into()),
            Some("profile:terminal:v1".into()),
            Some("reasoning:baseline:v1".into()),
            Some("model:symthaea:v1".into()),
            Some("evidence".into()),
        );
        assert!(query.is_replayable());
        assert!(!query.is_replay_addressable());
        let replay = query.with_replay_context(
            Some("profile:terminal:v1".into()),
            Some("reasoning:baseline:v1".into()),
            Some("model:symthaea:v1".into()),
        );
        assert!(replay.is_replay_addressable());
        assert_eq!(replay.replay_readiness(), ReplayReadiness::Addressable);
        assert_eq!(
            replay.replay_resolution_state(None),
            ReplayResolutionState::Addressable
        );
        let target = replay.replay_target().expect("complete replay target");
        assert_eq!(target.frontier_ref, "ef:demo:9d7b");
        assert!(!target.dependency_manifest_request().is_complete());
        assert_eq!(query.view, TerminalView::Evidence);
    }

    #[test]
    fn terminal_query_defaults_unknown_views_to_evidence() {
        let query = TerminalQueryV1::from_url_parts(
            Some("entity:fin:ns-energy-01".into()),
            None,
            Some("ef:demo:9d7b".into()),
            None,
            None,
            None,
            Some("future-ui".into()),
        );
        assert_eq!(query.view, TerminalView::Evidence);
    }

    #[test]
    fn terminal_query_is_not_replay_addressable_without_full_context() {
        let query = TerminalQueryV1::from_url_parts(
            Some("entity:fin:ns-energy-01".into()),
            Some("claim:obs:7f31".into()),
            Some("ef:demo:9d7b".into()),
            None,
            None,
            None,
            Some("evidence".into()),
        );
        assert!(!query.is_replay_addressable());
        assert_eq!(query.replay_readiness(), ReplayReadiness::Incomplete);
        assert_eq!(
            query.replay_resolution_state(None),
            ReplayResolutionState::Incomplete
        );
    }

    #[test]
    fn complete_manifest_is_not_a_replay_receipt() {
        let query = TerminalQueryV1::from_url_parts(
            Some("entity:fin:ns-energy-01".into()),
            Some("claim:obs:7f31".into()),
            Some("ef:demo:9d7b".into()),
            Some("profile:terminal:v1".into()),
            Some("reasoning:baseline:v1".into()),
            Some("model:symthaea:v1".into()),
            Some("evidence".into()),
        );
        let manifest = DependencyManifestV1 {
            frontier_ref: "ef:demo:9d7b".into(),
            evidence_roots: vec!["evidence-root:1".into()],
            source_versions: vec!["source:filing:v3".into()],
            canonical_state_root: Some("state-root:1".into()),
            model_versions: vec!["model:symthaea:v1".into()],
            ontology_version: Some("ontology:fin:v1".into()),
            projection_profile: "profile:terminal:v1".into(),
            reasoning_program: "reasoning:baseline:v1".into(),
            qualification_profile: Some("qualification:fin-001c0".into()),
        };
        assert_eq!(
            query.replay_resolution_state(Some(&manifest)),
            ReplayResolutionState::DependencyContextMatched
        );
        assert!(query.is_replay_addressable());
        let mut mismatched = manifest.clone();
        mismatched.frontier_ref = "ef:later".into();
        assert_eq!(
            query.replay_resolution_state(Some(&mismatched)),
            ReplayResolutionState::Unresolved
        );
        let mut wrong_model = manifest.clone();
        wrong_model.model_versions = vec!["model:other:v9".into()];
        assert_eq!(
            query.replay_resolution_state(Some(&wrong_model)),
            ReplayResolutionState::Unresolved
        );

        let mut wrong_profile = manifest.clone();
        wrong_profile.projection_profile = "profile:other:v2".into();
        assert_eq!(
            query.replay_resolution_state(Some(&wrong_profile)),
            ReplayResolutionState::Unresolved
        );

        let mut wrong_program = manifest;
        wrong_program.reasoning_program = "reasoning:other:v9".into();
        assert_eq!(
            query.replay_resolution_state(Some(&wrong_program)),
            ReplayResolutionState::Unresolved
        );
    }

    #[test]
    fn incomplete_manifest_rejects_whitespace_only_dependencies() {
        let query = TerminalQueryV1::from_url_parts(
            Some("entity:fin:ns-energy-01".into()),
            Some("claim:obs:7f31".into()),
            Some("ef:demo:9d7b".into()),
            Some("profile:terminal:v1".into()),
            Some("reasoning:baseline:v1".into()),
            Some("model:symthaea:v1".into()),
            Some("evidence".into()),
        );
        let target = query.replay_target().expect("complete replay target");
        let mut manifest = target.dependency_manifest_request();
        manifest.evidence_roots = vec!["   ".into()];
        manifest.source_versions = vec!["source:filing:v3".into()];
        manifest.canonical_state_root = Some("state-root:1".into());
        manifest.ontology_version = Some("ontology:fin:v1".into());
        manifest.qualification_profile = Some("qualification:fin-001c0".into());
        assert!(!manifest.is_complete());

        manifest.evidence_roots = vec!["evidence-root:1".into()];
        manifest.source_versions = vec!["  ".into()];
        assert!(!manifest.is_complete());
    }

    #[test]
    fn manifest_match_requires_projection_and_reasoning_identity() {
        let target = ReplayTargetV1 {
            entity_ref: "entity:fin:ns-energy-01".into(),
            claim_ref: "claim:obs:7f31".into(),
            frontier_ref: "ef:demo:9d7b".into(),
            projection_profile: "profile:terminal:v1".into(),
            reasoning_program: "reasoning:baseline:v1".into(),
            model_version: "model:symthaea:v1".into(),
        };
        let mut manifest = DependencyManifestV1 {
            frontier_ref: target.frontier_ref.clone(),
            evidence_roots: vec!["evidence-root:1".into()],
            source_versions: vec!["source:filing:v3".into()],
            canonical_state_root: Some("state-root:1".into()),
            model_versions: vec![target.model_version.clone()],
            ontology_version: Some("ontology:fin:v1".into()),
            projection_profile: target.projection_profile.clone(),
            reasoning_program: target.reasoning_program.clone(),
            qualification_profile: Some("qualification:fin-001c0".into()),
        };
        assert_eq!(
            manifest.dependency_context_match(&target),
            DependencyContextMatch::Matched
        );
        assert!(manifest.matches_dependency_context(&target));

        // The dependency manifest intentionally does not bind entity/claim.
        // That semantic query identity must come from the canonical authority.
        let mut different_query = target.clone();
        different_query.entity_ref = "entity:other:99".into();
        different_query.claim_ref = "claim:other:99".into();
        assert_eq!(
            manifest.dependency_context_match(&different_query),
            DependencyContextMatch::Matched
        );
        assert!(manifest.matches_dependency_context(&different_query));

        manifest.projection_profile = "profile:terminal:v2".into();
        assert_eq!(
            manifest.dependency_context_match(&target),
            DependencyContextMatch::Mismatch
        );
        assert!(!manifest.matches_dependency_context(&target));

        manifest.projection_profile = target.projection_profile.clone();
        manifest.reasoning_program = "reasoning:baseline:v2".into();
        assert_eq!(
            manifest.dependency_context_match(&target),
            DependencyContextMatch::Mismatch
        );
        assert!(!manifest.matches_dependency_context(&target));
    }

    #[test]
    fn incomplete_manifest_is_distinct_from_context_mismatch() {
        let target = ReplayTargetV1 {
            entity_ref: "entity:fin:ns-energy-01".into(),
            claim_ref: "claim:obs:7f31".into(),
            frontier_ref: "ef:demo:9d7b".into(),
            projection_profile: "profile:terminal:v1".into(),
            reasoning_program: "reasoning:baseline:v1".into(),
            model_version: "model:symthaea:v1".into(),
        };
        let manifest = target.dependency_manifest_request();

        assert_eq!(
            manifest.dependency_context_match(&target),
            DependencyContextMatch::Incomplete
        );
        assert!(!manifest.matches_dependency_context(&target));
    }

    #[test]
    fn replay_resolution_state_distinguishes_addressable_unresolved_and_matched() {
        let query = TerminalQueryV1::from_url_parts(
            Some("entity:fin:ns-energy-01".into()),
            Some("claim:obs:7f31".into()),
            Some("ef:demo:9d7b".into()),
            Some("profile:terminal:v1".into()),
            Some("reasoning:baseline:v1".into()),
            Some("model:symthaea:v1".into()),
            Some("evidence".into()),
        );

        assert_eq!(
            query.replay_resolution_state(None),
            ReplayResolutionState::Addressable
        );

        let incomplete = query
            .replay_target()
            .expect("complete replay target")
            .dependency_manifest_request();
        assert_eq!(
            query.replay_resolution_state(Some(&incomplete)),
            ReplayResolutionState::Unresolved
        );

        let target = query.replay_target().expect("complete replay target");
        let mut mismatched = DependencyManifestV1 {
            frontier_ref: target.frontier_ref.clone(),
            evidence_roots: vec!["evidence-root:1".into()],
            source_versions: vec!["source:filing:v3".into()],
            canonical_state_root: Some("state-root:1".into()),
            model_versions: vec![target.model_version.clone()],
            ontology_version: Some("ontology:fin:v1".into()),
            projection_profile: target.projection_profile.clone(),
            reasoning_program: "reasoning:other:v1".into(),
            qualification_profile: Some("qualification:fin-001c0".into()),
        };
        assert_eq!(
            query.replay_resolution_state(Some(&mismatched)),
            ReplayResolutionState::Unresolved
        );

        mismatched.reasoning_program = target.reasoning_program.clone();
        assert_eq!(
            query.replay_resolution_state(Some(&mismatched)),
            ReplayResolutionState::DependencyContextMatched
        );
    }

    #[test]
    fn empty_replay_identifiers_are_incomplete() {
        let query = TerminalQueryV1::from_url_parts(
            Some("entity:fin:ns-energy-01".into()),
            Some("claim:obs:7f31".into()),
            Some("ef:demo:9d7b".into()),
            Some(" ".into()),
            Some("reasoning:baseline:v1".into()),
            Some("model:symthaea:v1".into()),
            None,
        );
        assert_eq!(query.replay_readiness(), ReplayReadiness::Incomplete);
    }

    #[test]
    fn terminal_query_is_not_replayable_without_frontier() {
        let query = TerminalQueryV1::from_url_parts(
            Some("entity:fin:ns-energy-01".into()),
            None,
            None,
            None,
            None,
            None,
            None,
        );
        assert!(!query.is_replayable());
    }
}


/// A complete address for a requested historical computation.
///
/// This is deliberately narrower than a replay receipt: it identifies the
/// requested computation, but does not prove that its dependencies exist or
/// that the computation has been reconstructed.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct ReplayTargetV1 {
    pub entity_ref: String,
    pub claim_ref: String,
    pub frontier_ref: String,
    pub projection_profile: String,
    pub reasoning_program: String,
    pub model_version: String,
}

/// Local navigation identity for an addressable replay request.
///
/// This is intentionally a terminal/navigation identity, not a canonical
/// semantic query identifier. Canonical query semantics remain upstream.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReplayNavigationIdentityV1 {
    entity_ref: String,
    claim_ref: String,
}

/// Local execution context for an addressable replay request.
///
/// These identifiers describe the requested projection/model/reasoning
/// context. They do not establish that the referenced versions exist or are
/// authoritative.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReplayExecutionContextV1 {
    frontier_ref: String,
    projection_profile: String,
    reasoning_program: String,
    model_version: String,
}

impl ReplayNavigationIdentityV1 {
    pub fn entity_ref(&self) -> &str {
        &self.entity_ref
    }

    pub fn claim_ref(&self) -> &str {
        &self.claim_ref
    }
}

impl ReplayExecutionContextV1 {
    pub fn frontier_ref(&self) -> &str {
        &self.frontier_ref
    }

    pub fn projection_profile(&self) -> &str {
        &self.projection_profile
    }

    pub fn reasoning_program(&self) -> &str {
        &self.reasoning_program
    }

    pub fn model_version(&self) -> &str {
        &self.model_version
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReplayReadiness {
    Incomplete,
    Addressable,
}

/// State of the replay hand-off. A complete manifest does not imply execution.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReplayResolutionState {
    Incomplete,
    Addressable,
    Unresolved,
    /// The dependency manifest is complete and its execution context matches the
    /// requested replay target. This still does not prove authoritative Mycelix
    /// resolution or replay execution.
    DependencyContextMatched,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReplayTargetConstructionError {
    MalformedQuery,
    IncompleteQuery,
}

impl ReplayTargetV1 {
    /// Constructs a replay target only from a structurally valid terminal query.
    ///
    /// This is a composition boundary, not a semantic validation step:
    /// Atlas can prove that the URL-derived fields are present and non-blank,
    /// but only the upstream semantic authority can establish their canonical
    /// meaning and dependency relationships.
    pub fn try_from_terminal_query(
        query: &TerminalQueryV1,
    ) -> Result<Self, ReplayTargetConstructionError> {
        if query.validity() == TerminalQueryValidity::Malformed {
            return Err(ReplayTargetConstructionError::MalformedQuery);
        }

        let target = Self {
            entity_ref: query
                .entity_ref
                .clone()
                .ok_or(ReplayTargetConstructionError::IncompleteQuery)?,
            claim_ref: query
                .claim_ref
                .clone()
                .ok_or(ReplayTargetConstructionError::IncompleteQuery)?,
            frontier_ref: query
                .frontier_ref
                .clone()
                .ok_or(ReplayTargetConstructionError::IncompleteQuery)?,
            projection_profile: query
                .projection_profile
                .clone()
                .ok_or(ReplayTargetConstructionError::IncompleteQuery)?,
            reasoning_program: query
                .reasoning_program
                .clone()
                .ok_or(ReplayTargetConstructionError::IncompleteQuery)?,
            model_version: query
                .model_version
                .clone()
                .ok_or(ReplayTargetConstructionError::IncompleteQuery)?,
        };

        target
            .is_well_formed()
            .then_some(target)
            .ok_or(ReplayTargetConstructionError::MalformedQuery)
    }

    pub fn is_well_formed(&self) -> bool {
        [
            self.entity_ref.as_str(),
            self.claim_ref.as_str(),
            self.frontier_ref.as_str(),
            self.projection_profile.as_str(),
            self.reasoning_program.as_str(),
            self.model_version.as_str(),
        ]
        .iter()
        .all(|value| !value.trim().is_empty())
    }

    pub fn readiness(&self) -> ReplayReadiness {
        if self.is_well_formed() {
            ReplayReadiness::Addressable
        } else {
            ReplayReadiness::Incomplete
        }
    }

    /// Returns the navigation identity only after the target's structural
    /// completeness invariant has been established.
    pub fn navigation_identity(&self) -> Option<ReplayNavigationIdentityV1> {
        self.is_well_formed().then(|| ReplayNavigationIdentityV1 {
            entity_ref: self.entity_ref.clone(),
            claim_ref: self.claim_ref.clone(),
        })
    }

    /// Returns the execution context only after the target's structural
    /// completeness invariant has been established.
    pub fn execution_context(&self) -> Option<ReplayExecutionContextV1> {
        self.is_well_formed().then(|| ReplayExecutionContextV1 {
            frontier_ref: self.frontier_ref.clone(),
            projection_profile: self.projection_profile.clone(),
            reasoning_program: self.reasoning_program.clone(),
            model_version: self.model_version.clone(),
        })
    }

    /// Dependency completeness belongs to the semantic authority; Atlas only
    /// carries the identifiers needed to request that resolution.
    pub fn dependency_manifest_request(&self) -> DependencyManifestV1 {
        let context = self
            .execution_context()
            .expect("dependency manifest requires an addressable replay target");

        DependencyManifestV1::from_execution_context(context)
    }
}

/// Manifest shape requested from the upstream semantic authority before a
/// replay can be considered resolved. Empty/absent fields are intentionally
/// not filled by Atlas.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct DependencyManifestV1 {
    pub frontier_ref: String,
    pub evidence_roots: Vec<String>,
    pub source_versions: Vec<String>,
    pub canonical_state_root: Option<String>,
    pub model_versions: Vec<String>,
    pub ontology_version: Option<String>,
    pub projection_profile: String,
    pub reasoning_program: String,
    pub qualification_profile: Option<String>,
}

/// Result of comparing a dependency manifest with the execution context
/// required by a replay target.
///
/// This distinguishes an absent/incomplete dependency set from a genuine
/// context mismatch. Neither matched nor mismatched local context is an
/// authoritative Mycelix resolution result.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DependencyContextMatch {
    Incomplete,
    Mismatch,
    Matched,
}

impl DependencyManifestV1 {
    /// Builds the local dependency hand-off from a structurally complete
    /// execution context. Authority-owned dependency roots and qualification
    /// data remain intentionally absent.
    pub fn from_execution_context(context: ReplayExecutionContextV1) -> Self {
        Self {
            frontier_ref: context.frontier_ref,
            evidence_roots: Vec::new(),
            source_versions: Vec::new(),
            canonical_state_root: None,
            model_versions: vec![context.model_version],
            ontology_version: None,
            projection_profile: context.projection_profile,
            reasoning_program: context.reasoning_program,
            qualification_profile: None,
        }
    }

    /// Classifies the dependency execution context required by the requested replay.
    ///
    /// This deliberately does **not** bind the entity/claim/query identity.
    /// Atlas must not invent a semantic query identifier; that binding belongs
    /// to the canonical Mycelix replay contract when exposed upstream.
    pub fn dependency_context_match(&self, target: &ReplayTargetV1) -> DependencyContextMatch {
        if !self.is_complete() {
            return DependencyContextMatch::Incomplete;
        }

        if self.frontier_ref == target.frontier_ref
            && self.projection_profile == target.projection_profile
            && self.reasoning_program == target.reasoning_program
            && self.model_versions.iter().any(|version| version == &target.model_version)
        {
            DependencyContextMatch::Matched
        } else {
            DependencyContextMatch::Mismatch
        }
    }

    /// Compatibility predicate for callers that only need a boolean.
    pub fn matches_dependency_context(&self, target: &ReplayTargetV1) -> bool {
        self.dependency_context_match(target) == DependencyContextMatch::Matched
    }

    pub fn is_complete(&self) -> bool {
        !self.frontier_ref.trim().is_empty()
            && !self.evidence_roots.is_empty()
            && self.evidence_roots.iter().all(|v| !v.trim().is_empty())
            && !self.source_versions.is_empty()
            && self.source_versions.iter().all(|v| !v.trim().is_empty())
            && self.canonical_state_root.as_deref().is_some_and(|v| !v.trim().is_empty())
            && !self.model_versions.is_empty()
            && self.model_versions.iter().all(|v| !v.trim().is_empty())
            && self.ontology_version.as_deref().is_some_and(|v| !v.trim().is_empty())
            && !self.projection_profile.trim().is_empty()
            && !self.reasoning_program.trim().is_empty()
            && self.qualification_profile.as_deref().is_some_and(|v| !v.trim().is_empty())
    }
}

/// URL-addressable research state for the terminal projection.
///
/// This is intentionally a view/query contract, not a semantic authority.
/// Mycelix remains responsible for resolving the referenced frontier, claim,
/// evidence and derivation before Atlas renders them.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
/// Structural validation state for URL-derived terminal identity.
///
/// A malformed explicit value is deliberately distinct from an absent value:
/// callers may fall back only for absence, never for an invalid identity.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TerminalQueryValidity {
    Valid,
    Malformed,
}

pub struct TerminalQueryV1 {
    pub entity_ref: Option<String>,
    pub claim_ref: Option<String>,
    pub frontier_ref: Option<String>,
    pub projection_profile: Option<String>,
    pub reasoning_program: Option<String>,
    pub model_version: Option<String>,
    pub view: TerminalView,
}

impl TerminalQueryV1 {
    /// Classifies URL-derived identity without normalizing malformed values into
    /// absence. This preserves the distinction required for fail-closed lookup.
    pub fn validity(&self) -> TerminalQueryValidity {
        let values = [
            self.entity_ref.as_deref(),
            self.claim_ref.as_deref(),
            self.frontier_ref.as_deref(),
            self.projection_profile.as_deref(),
            self.reasoning_program.as_deref(),
            self.model_version.as_deref(),
        ];
        if values.into_iter().flatten().any(|value| value.trim().is_empty()) {
            TerminalQueryValidity::Malformed
        } else {
            TerminalQueryValidity::Valid
        }
    }

    pub fn replay_target(&self) -> Option<ReplayTargetV1> {
        ReplayTargetV1::try_from_terminal_query(self).ok()
    }

    /// A frontier URL is not by itself a full replay contract.
    ///
    /// Full replay requires the same query target plus the projection,
    /// reasoning-program and model identities. This deliberately prevents
    /// Atlas from presenting a navigation URL as a verified historical replay.
    pub fn is_replay_addressable(&self) -> bool {
        self.replay_target().is_some()
    }

    pub fn replay_readiness(&self) -> ReplayReadiness {
        if self.replay_target().is_some() {
            ReplayReadiness::Addressable
        } else {
            ReplayReadiness::Incomplete
        }
    }

    /// Classifies only the local hand-off state. This is not a replay receipt.
    pub fn replay_resolution_state(
        &self,
        manifest: Option<&DependencyManifestV1>,
    ) -> ReplayResolutionState {
        if self.validity() == TerminalQueryValidity::Malformed {
            return ReplayResolutionState::Unresolved;
        }
        if self.replay_target().is_none() {
            ReplayResolutionState::Incomplete
        } else {
            match manifest {
                None => ReplayResolutionState::Addressable,
                Some(manifest) => {
                    let target = self.replay_target().expect("checked above");
                    match manifest.dependency_context_match(&target) {
                        DependencyContextMatch::Matched => ReplayResolutionState::DependencyContextMatched,
                        DependencyContextMatch::Incomplete | DependencyContextMatch::Mismatch => {
                            ReplayResolutionState::Unresolved
                        }
                    }
                }
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum TerminalView {
    #[default]
    Evidence,
    Lineage,
    Research,
}

impl TerminalView {
    pub fn parse(value: Option<&str>) -> Self {
        match value {
            Some("lineage") => Self::Lineage,
            Some("research") => Self::Research,
            _ => Self::Evidence,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Evidence => "evidence",
            Self::Lineage => "lineage",
            Self::Research => "research",
        }
    }
}

impl TerminalQueryV1 {
    pub fn from_url_parts(
        entity_ref: Option<String>,
        claim_ref: Option<String>,
        frontier_ref: Option<String>,
        projection_profile: Option<String>,
        reasoning_program: Option<String>,
        model_version: Option<String>,
        view: Option<String>,
    ) -> Self {
        Self {
            entity_ref,
            claim_ref,
            frontier_ref,
            projection_profile,
            reasoning_program,
            model_version,
            view: TerminalView::parse(view.as_deref()),
        }
    }

    pub fn with_replay_context(
        mut self,
        projection_profile: Option<String>,
        reasoning_program: Option<String>,
        model_version: Option<String>,
    ) -> Self {
        self.projection_profile = projection_profile;
        self.reasoning_program = reasoning_program;
        self.model_version = model_version;
        self
    }

    /// Backwards-compatible name for the old URL-addressability check.
    /// This means "has a frontier target", not "verified replay".
    pub fn is_replayable(&self) -> bool {
        self.entity_ref.is_some() && self.frontier_ref.is_some()
    }
}
