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
}
