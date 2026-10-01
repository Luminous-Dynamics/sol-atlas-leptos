// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Authority-resolution transport boundary for replay dependencies.
//!
//! This module deliberately does not resolve evidence itself and does not
//! duplicate Mycelix's epistemic/frontier ontology. It provides a typed
//! adapter boundary so an Atlas request, an authoritative response, and a
//! replay execution receipt cannot be represented by one complete-looking
//! local object.

use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub enum DependencyResolutionStatus {
    Unrequested,
    Requested,
    Resolved,
    PartiallyResolved,
    Mismatched,
    Protected,
    Unavailable,
}

/// A typed request hand-off from Atlas to the semantic authority.
///
/// The manifest identity is opaque here: its semantics remain owned by the
/// canonical authority rather than being reconstructed by Atlas.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct DependencyResolutionRequestV1 {
    pub manifest_digest: String,
}

impl DependencyResolutionRequestV1 {
    pub fn new(manifest_digest: impl Into<String>) -> Option<Self> {
        let manifest_digest = manifest_digest.into();
        if manifest_digest.trim().is_empty() {
            return None;
        }
        Some(Self { manifest_digest })
    }
}

/// A reference to an explicit authoritative resolution response.
///
/// This is evidence that an authority answered a manifest request, not proof
/// that the requested replay subsequently executed.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct AuthorityResolutionRefV1 {
    /// Opaque canonical frontier identity owned by Mycelix.
    pub frontier_ref: String,
    pub manifest_digest: String,
    pub resolution_ref: String,
    pub status: DependencyResolutionStatus,
}

impl AuthorityResolutionRefV1 {
    pub fn from_authority(
        frontier_ref: impl Into<String>,
        manifest_digest: impl Into<String>,
        resolution_ref: impl Into<String>,
        status: DependencyResolutionStatus,
    ) -> Option<Self> {
        let frontier_ref = frontier_ref.into();
        let manifest_digest = manifest_digest.into();
        let resolution_ref = resolution_ref.into();
        if frontier_ref.trim().is_empty()
            || manifest_digest.trim().is_empty()
            || resolution_ref.trim().is_empty()
        {
            return None;
        }
        Some(Self {
            frontier_ref,
            manifest_digest,
            resolution_ref,
            status,
        })
    }

    pub fn is_terminally_resolved(&self) -> bool {
        self.status == DependencyResolutionStatus::Resolved
    }
}

/// A replay dependency hand-off state.
///
/// The enum variant is authoritative for the local state classification.
/// This prevents a malformed combination such as PartiallyResolved carrying
/// a Resolved response from accidentally becoming replay-eligible.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DependencyResolutionState {
    NotRequested,
    Requested(DependencyResolutionRequestV1),
    Resolved(AuthorityResolutionRefV1),
    PartiallyResolved(AuthorityResolutionRefV1),
    Rejected(AuthorityResolutionRefV1),
}

impl DependencyResolutionState {
    /// Converts an explicit authority response into the corresponding local
    /// resolution state. Request-only statuses are not valid authority
    /// responses and therefore cannot cross this boundary.
    pub fn try_from_authority(reference: AuthorityResolutionRefV1) -> Option<Self> {
        match reference.status {
            DependencyResolutionStatus::Resolved => Some(Self::Resolved(reference)),
            DependencyResolutionStatus::PartiallyResolved => Some(Self::PartiallyResolved(reference)),
            DependencyResolutionStatus::Mismatched
            | DependencyResolutionStatus::Protected
            | DependencyResolutionStatus::Unavailable => Some(Self::Rejected(reference)),
            DependencyResolutionStatus::Unrequested | DependencyResolutionStatus::Requested => None,
        }
    }

    pub fn status(&self) -> DependencyResolutionStatus {
        match self {
            Self::NotRequested => DependencyResolutionStatus::Unrequested,
            Self::Requested(_) => DependencyResolutionStatus::Requested,
            Self::Resolved(_) => DependencyResolutionStatus::Resolved,
            Self::PartiallyResolved(_) => DependencyResolutionStatus::PartiallyResolved,
            Self::Rejected(reference) => reference.status,
        }
    }

    pub fn resolution_ref(&self) -> Option<&AuthorityResolutionRefV1> {
        match self {
            Self::Resolved(reference)
            | Self::PartiallyResolved(reference)
            | Self::Rejected(reference) => Some(reference),
            Self::NotRequested | Self::Requested(_) => None,
        }
    }

    /// A dependency set is replay-eligible only when:
    /// 1. it is in the explicit Resolved state;
    /// 2. the authority response says Resolved; and
    /// 3. the response is bound to the exact requested manifest identity.
    pub fn is_bound_to(&self, frontier_ref: &str, manifest_digest: &str) -> bool {
        matches!(self, Self::Resolved(reference)
            if reference.frontier_ref == frontier_ref
                && reference.manifest_digest == manifest_digest
                && reference.is_terminally_resolved())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_is_not_a_resolution() {
        let request = DependencyResolutionRequestV1::new("manifest:abc").unwrap();
        let state = DependencyResolutionState::Requested(request);
        assert_eq!(state.status(), DependencyResolutionStatus::Requested);
        assert!(!state.is_bound_to("frontier:001", "manifest:abc"));
        assert!(state.resolution_ref().is_none());
    }

    #[test]
    fn request_rejects_blank_manifest_identity() {
        assert!(DependencyResolutionRequestV1::new("   ").is_none());
    }

    #[test]
    fn authority_reference_requires_explicit_nonblank_identity() {
        assert!(AuthorityResolutionRefV1::from_authority(
            "frontier:001",
            "manifest:abc",
            "resolution:001",
            DependencyResolutionStatus::Resolved,
        ).is_some());

        assert!(AuthorityResolutionRefV1::from_authority(
            "   ",
            "manifest:abc",
            "resolution:001",
            DependencyResolutionStatus::Resolved,
        ).is_none());

        assert!(AuthorityResolutionRefV1::from_authority(
            "frontier:001",
            "   ",
            "resolution:001",
            DependencyResolutionStatus::Resolved,
        ).is_none());

        assert!(AuthorityResolutionRefV1::from_authority(
            "frontier:001",
            "manifest:abc",
            " ",
            DependencyResolutionStatus::Resolved,
        ).is_none());
    }

    #[test]
    fn resolved_state_requires_manifest_binding() {
        let reference = AuthorityResolutionRefV1::from_authority(
            "frontier:001",
            "manifest:abc",
            "resolution:001",
            DependencyResolutionStatus::Resolved,
        ).unwrap();
        let state = DependencyResolutionState::Resolved(reference);

        assert!(state.is_bound_to("frontier:001", "manifest:abc"));
        assert!(!state.is_bound_to("frontier:001", "manifest:other"));
    }

    #[test]
    fn authority_response_constructor_rejects_request_only_statuses() {
        let reference = AuthorityResolutionRefV1::from_authority(
            "frontier:001",
            "manifest:abc",
            "resolution:001",
            DependencyResolutionStatus::Requested,
        ).unwrap();
        assert!(DependencyResolutionState::try_from_authority(reference).is_none());

        let reference = AuthorityResolutionRefV1::from_authority(
            "frontier:001",
            "manifest:abc",
            "resolution:001",
            DependencyResolutionStatus::Resolved,
        ).unwrap();
        assert!(matches!(
            DependencyResolutionState::try_from_authority(reference),
            Some(DependencyResolutionState::Resolved(_))
        ));
    }

    #[test]
    fn authority_response_constructor_maps_nonterminal_outcomes_fail_closed() {
        for status in [
            DependencyResolutionStatus::PartiallyResolved,
            DependencyResolutionStatus::Mismatched,
            DependencyResolutionStatus::Protected,
            DependencyResolutionStatus::Unavailable,
        ] {
            let reference = AuthorityResolutionRefV1::from_authority(
                "manifest:abc",
                "resolution:001",
                status,
            ).unwrap();
            let state = DependencyResolutionState::try_from_authority(reference).unwrap();
            assert!(!state.is_bound_to("frontier:001", "manifest:abc"));
        }
    }

    #[test]
    fn variant_controls_terminal_resolution() {
        let reference = AuthorityResolutionRefV1::from_authority(
            "frontier:001",
            "manifest:abc",
            "resolution:001",
            DependencyResolutionStatus::Resolved,
        ).unwrap();

        assert!(!DependencyResolutionState::PartiallyResolved(reference.clone())
            .is_bound_to("frontier:001", "manifest:abc"));
        assert!(!DependencyResolutionState::Rejected(reference)
            .is_bound_to("frontier:001", "manifest:abc"));
    }

    #[test]
    fn partial_and_protected_states_never_become_terminal_resolution() {
        let partial = AuthorityResolutionRefV1::from_authority(
            "frontier:001",
            "manifest:abc",
            "resolution:partial",
            DependencyResolutionStatus::PartiallyResolved,
        ).unwrap();
        let protected = AuthorityResolutionRefV1::from_authority(
            "frontier:001",
            "manifest:abc",
            "resolution:protected",
            DependencyResolutionStatus::Protected,
        ).unwrap();

        assert_eq!(
            DependencyResolutionState::PartiallyResolved(partial).status(),
            DependencyResolutionStatus::PartiallyResolved
        );
        assert!(!DependencyResolutionState::Rejected(protected)
            .is_bound_to("frontier:001", "manifest:abc"));
    }
}
