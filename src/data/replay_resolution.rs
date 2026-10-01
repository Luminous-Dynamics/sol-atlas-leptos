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

/// The outcome reported by the semantic authority for a dependency request.
///
/// This is transport state, not an Atlas-derived epistemic judgment. In
/// particular, Resolved means the authority supplied a complete response
/// bound to the requested manifest; it does not mean that replay executed.
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
/// The manifest payload remains opaque to this adapter. Atlas therefore cannot
/// accidentally turn locally-known execution context into authoritative
/// evidence roots, source versions, state roots, ontology, or qualification.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct DependencyResolutionRequestV1 {
    pub manifest_digest: String,
}

/// A reference to an authoritative resolution response.
///
/// The response is identified separately from the request so a request cannot
/// masquerade as proof that the authority answered it.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
pub struct AuthorityResolutionRefV1 {
    pub manifest_digest: String,
    pub resolution_ref: String,
    pub status: DependencyResolutionStatus,
}

impl AuthorityResolutionRefV1 {
    /// Construct only an explicit authority-response reference.
    ///
    /// The adapter does not infer this from a manifest or from local fixture
    /// data. Callers must supply both the manifest binding and authority
    /// response identifier.
    pub fn from_authority(
        manifest_digest: impl Into<String>,
        resolution_ref: impl Into<String>,
        status: DependencyResolutionStatus,
    ) -> Option<Self> {
        let manifest_digest = manifest_digest.into();
        let resolution_ref = resolution_ref.into();
        if manifest_digest.trim().is_empty() || resolution_ref.trim().is_empty() {
            return None;
        }
        Some(Self {
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
/// AuthorityResolutionRefV1 is intentionally not a replay receipt. Actual
/// execution must remain represented by the canonical replay/derivation
/// authority rather than being synthesized in Atlas.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DependencyResolutionState {
    NotRequested,
    Requested(DependencyResolutionRequestV1),
    Resolved(AuthorityResolutionRefV1),
    PartiallyResolved(AuthorityResolutionRefV1),
    Rejected(AuthorityResolutionRefV1),
}

impl DependencyResolutionState {
    pub fn status(&self) -> DependencyResolutionStatus {
        match self {
            Self::NotRequested => DependencyResolutionStatus::Unrequested,
            Self::Requested(_) => DependencyResolutionStatus::Requested,
            Self::Resolved(_) => DependencyResolutionStatus::Resolved,
            Self::PartiallyResolved(reference) => reference.status,
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

    /// A resolved dependency set is usable only when its authority response is
    /// explicitly bound to the requested manifest digest.
    pub fn is_bound_to(&self, manifest_digest: &str) -> bool {
        self.resolution_ref()
            .map(|reference| {
                reference.manifest_digest == manifest_digest
                    && reference.is_terminally_resolved()
            })
            .unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn request_is_not_a_resolution() {
        let request = DependencyResolutionRequestV1 {
            manifest_digest: "manifest:abc".into(),
        };
        let state = DependencyResolutionState::Requested(request);
        assert_eq!(state.status(), DependencyResolutionStatus::Requested);
        assert!(!state.is_bound_to("manifest:abc"));
        assert!(state.resolution_ref().is_none());
    }

    #[test]
    fn authority_reference_requires_explicit_nonblank_identity() {
        assert!(AuthorityResolutionRefV1::from_authority(
            "manifest:abc",
            "resolution:001",
            DependencyResolutionStatus::Resolved,
        )
        .is_some());

        assert!(AuthorityResolutionRefV1::from_authority(
            "   ",
            "resolution:001",
            DependencyResolutionStatus::Resolved,
        )
        .is_none());

        assert!(AuthorityResolutionRefV1::from_authority(
            "manifest:abc",
            " ",
            DependencyResolutionStatus::Resolved,
        )
        .is_none());
    }

    #[test]
    fn resolved_state_requires_manifest_binding() {
        let reference = AuthorityResolutionRefV1::from_authority(
            "manifest:abc",
            "resolution:001",
            DependencyResolutionStatus::Resolved,
        )
        .unwrap();
        let state = DependencyResolutionState::Resolved(reference);

        assert!(state.is_bound_to("manifest:abc"));
        assert!(!state.is_bound_to("manifest:other"));
    }

    #[test]
    fn partial_and_protected_states_never_become_terminal_resolution() {
        let partial = AuthorityResolutionRefV1::from_authority(
            "manifest:abc",
            "resolution:partial",
            DependencyResolutionStatus::PartiallyResolved,
        )
        .unwrap();
        let protected = AuthorityResolutionRefV1::from_authority(
            "manifest:abc",
            "resolution:protected",
            DependencyResolutionStatus::Protected,
        )
        .unwrap();

        assert!(!DependencyResolutionState::PartiallyResolved(partial)
            .is_bound_to("manifest:abc"));
        assert!(!DependencyResolutionState::Rejected(protected)
            .is_bound_to("manifest:abc"));
    }
}
