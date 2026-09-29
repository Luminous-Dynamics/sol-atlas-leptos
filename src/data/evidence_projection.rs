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
            Some("evidence".into()),
        );
        assert!(query.is_replayable());
        assert_eq!(query.view, TerminalView::Evidence);
    }

    #[test]
    fn terminal_query_defaults_unknown_views_to_evidence() {
        let query = TerminalQueryV1::from_url_parts(
            Some("entity:fin:ns-energy-01".into()),
            None,
            Some("ef:demo:9d7b".into()),
            Some("future-ui".into()),
        );
        assert_eq!(query.view, TerminalView::Evidence);
    }

    #[test]
    fn terminal_query_is_not_replayable_without_frontier() {
        let query = TerminalQueryV1::from_url_parts(
            Some("entity:fin:ns-energy-01".into()),
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
    pub view: TerminalView,
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
        view: Option<String>,
    ) -> Self {
        Self {
            entity_ref,
            claim_ref,
            frontier_ref,
            view: TerminalView::parse(view.as_deref()),
        }
    }

    pub fn is_replayable(&self) -> bool {
        self.entity_ref.is_some() && self.frontier_ref.is_some()
    }
}
