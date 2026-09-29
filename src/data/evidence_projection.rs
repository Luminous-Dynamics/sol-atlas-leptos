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
        if id.is_empty() {
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
    Resolved,
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
                if id == &self.projection_id { LineageResolution::Resolved } else { LineageResolution::Unavailable }
            }
            LineageNodeRef::Claim(id) => {
                if id == &self.claim_ref { LineageResolution::Resolved } else { LineageResolution::Unavailable }
            }
            LineageNodeRef::Evidence(id) => {
                if self.evidence_refs.iter().any(|ref_id| ref_id == id) {
                    if matches!(self.visibility, Visibility::Redacted) {
                        LineageResolution::Protected
                    } else {
                        LineageResolution::Resolved
                    }
                } else {
                    LineageResolution::Unavailable
                }
            }
            LineageNodeRef::SourceSnapshot(id) => {
                if self.source_snapshot_refs.iter().any(|ref_id| ref_id == id) {
                    LineageResolution::Resolved
                } else {
                    LineageResolution::Unavailable
                }
            }
            LineageNodeRef::Qualification(id) => {
                if id == &self.qualification_ref {
                    match self.qualification {
                        EpistemicState::Protected => LineageResolution::Protected,
                        _ => LineageResolution::Resolved,
                    }
                } else {
                    LineageResolution::Unavailable
                }
            }
            LineageNodeRef::Frontier(id) => {
                if id == &self.frontier_ref { LineageResolution::Resolved } else { LineageResolution::Unavailable }
            }
            LineageNodeRef::Derivation(id) => {
                if matches!(self.claim_kind, ClaimKind::Derived) && self.derivation_ref.as_deref() == Some(id) {
                    LineageResolution::Resolved
                } else {
                    LineageResolution::Unavailable
                }
            }
            LineageNodeRef::ReasoningReceipt(id) => {
                if matches!(self.claim_kind, ClaimKind::Hypothesis) && self.derivation_ref.as_deref() == Some(id) {
                    LineageResolution::Resolved
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


impl AtlasEvidenceProjectionV1 {
    /// The UI may display this claim, but it may never render it as stronger
    /// than its source qualification or claim kind.
    pub fn can_render_statement(&self) -> bool {
        !matches!(self.visibility, Visibility::Redacted)
            && !self.evidence_refs.is_empty()
            && !self.frontier_ref.is_empty()
            && !self.claim_ref.is_empty()
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
        let has_core = !self.claim_ref.is_empty()
            && !self.frontier_ref.is_empty()
            && !self.qualification_ref.is_empty()
            && !self.evidence_refs.is_empty()
            && !self.source_snapshot_refs.is_empty();

        let derivation_required = matches!(self.claim_kind, ClaimKind::Derived | ClaimKind::Hypothesis);
        let has_derivation = self
            .derivation_ref
            .as_ref()
            .map(|id| !id.is_empty())
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
        assert_eq!(projection.resolve_lineage_node(&claim), LineageResolution::Resolved);

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
        );
        assert!(!query.is_replayable());
    }
}


/// URL-addressable research state for the terminal projection.
///
/// This is intentionally a view/query contract, not a semantic authority.
/// Mycelix remains responsible for resolving the referenced frontier, claim,
/// evidence and derivation before Atlas renders them.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
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
    /// A frontier URL is not by itself a full replay contract.
    ///
    /// Full replay requires the same query target plus the projection,
    /// reasoning-program and model identities. This deliberately prevents
    /// Atlas from presenting a navigation URL as a verified historical replay.
    pub fn is_replay_addressable(&self) -> bool {
        self.entity_ref.is_some()
            && self.claim_ref.is_some()
            && self.frontier_ref.is_some()
            && self.projection_profile.is_some()
            && self.reasoning_program.is_some()
            && self.model_version.is_some()
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
