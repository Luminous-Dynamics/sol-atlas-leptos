// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Typed, content-addressed evidence references.
//!
//! This module separates evidence identity from evidence verification.
//! A typed reference says exactly which artifact type, digest context, and
//! digest value are being named. It does not prove that the artifact exists,
//! that the digest recomputes, that the artifact is current, or that its
//! contents are true.
//!
//! Semantic boundaries:
//! - mutable locator != content identity
//! - digest identity != digest verification
//! - digest verification != payload truth
//! - payload truth != qualification

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Representation of the digest value carried by a typed reference.
///
/// These forms are deliberately distinct. A verifier must not silently coerce
/// between them because the wire representation is part of the digest context.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DigestRepresentationV1 {
    /// Reserved for a future byte-backed digest field; invalid for the current
    /// string-backed representation, so V1 constructors fail closed.
    RawBytes,
    LowerHex,
    PrefixedLowerHex,
}

/// Explicit context for computing and comparing a content digest.
///
/// The context binds the pre-image construction, canonicalization, hash
/// algorithm, domain separation, encoding, and carried representation. Two
/// digest values are comparable only when their contexts are compatible.
///
/// canonicalization may name a project-local algorithm. This type does not
/// claim RFC 8785/JCS interoperability merely by naming a canonicalization
/// string.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct DigestContextV1 {
    pub id: String,
    pub preimage_construction: String,
    pub canonicalization: String,
    pub hash_algorithm: String,
    pub domain_separator: String,
    pub preimage_encoding: String,
    pub representation: DigestRepresentationV1,
}

impl DigestContextV1 {
    pub const SCHEMA: &'static str = "sol-atlas:digest-context:v1";

    pub fn is_well_formed(&self) -> bool {
        !self.id.is_empty()
            && !self.preimage_construction.is_empty()
            && !self.canonicalization.is_empty()
            && !self.hash_algorithm.is_empty()
            && !self.domain_separator.is_empty()
            && !self.preimage_encoding.is_empty()
    }
}

/// An explicit acceptance profile for one artifact type and one digest context.
///
/// Multiple authorized contexts should be represented by multiple profiles rather
/// than silently choosing among contexts for the same reference.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceReferenceProfileV1 {
    /// Stable profile identifier.
    pub id: String,
    /// Immutable profile revision. Zero is invalid.
    pub version: u32,
    pub artifact_type: String,
    pub context: DigestContextV1,
    /// Explicit context-selection purpose. Required when a consuming profile
    /// distinguishes multiple contexts for the same artifact type.
    pub purpose: Option<String>,
    pub claim_ceiling: String,
}

impl EvidenceReferenceProfileV1 {
    pub const SCHEMA: &'static str = "sol-atlas:evidence-reference-profile:v1";

    pub fn is_well_formed(&self) -> bool {
        !self.id.is_empty()
            && self.version > 0
            && !self.artifact_type.is_empty()
            && !self.claim_ceiling.is_empty()
            && self.context.is_well_formed()
            && self.context.representation != DigestRepresentationV1::RawBytes
            && self
                .purpose
                .as_ref()
                .is_none_or(|purpose| !purpose.is_empty())
    }

    pub fn accepts(&self, reference: &EvidenceReferenceV1) -> bool {
        self.is_well_formed()
            && reference.is_well_formed()
            && reference.artifact_type == self.artifact_type
            && reference.context == self.context
            && reference.purpose == self.purpose
            && reference.claim_ceiling == self.claim_ceiling
    }

    /// Canonical semantic bytes for this exact published profile identity.
    ///
    /// This is a project-local fixed-field canonical form and makes the
    /// profile's version, purpose, context, and claim ceiling independently
    /// addressable. It does not claim RFC 8785/JCS interoperability.
    pub fn canonical_bytes(&self) -> Vec<u8> {
        #[derive(Serialize)]
        struct CanonicalProfile {
            schema: String,
            id: String,
            version: u32,
            artifact_type: String,
            context: DigestContextV1,
            purpose: Option<String>,
            claim_ceiling: String,
        }

        let canonical = CanonicalProfile {
            schema: Self::SCHEMA.into(),
            id: self.id.clone(),
            version: self.version,
            artifact_type: self.artifact_type.clone(),
            context: self.context.clone(),
            purpose: self.purpose.clone(),
            claim_ceiling: self.claim_ceiling.clone(),
        };

        serde_json::to_vec(&canonical)
            .expect("evidence reference profile contains only serializable primitives")
    }

    /// SHA-256 identity of this exact profile declaration.
    pub fn digest(&self) -> String {
        let digest = Sha256::digest(self.canonical_bytes());
        format!("sha256:{digest:x}")
    }
}

/// Immutable, content-addressed set of evidence-reference profiles.
///
/// The registry owns context selection. It refuses duplicate (artifact_type,
/// purpose) declarations and refuses ambiguous selection when one artifact
/// type has multiple accepted contexts but the reference omits purpose.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceReferenceProfileRegistryV1 {
    pub id: String,
    pub version: u32,
    pub profiles: Vec<EvidenceReferenceProfileV1>,
}

impl EvidenceReferenceProfileRegistryV1 {
    pub const SCHEMA: &'static str = "sol-atlas:evidence-reference-profile-registry:v1";

    pub fn is_well_formed(&self) -> bool {
        if self.id.is_empty() || self.version == 0 || self.profiles.is_empty() {
            return false;
        }

        if self
            .profiles
            .iter()
            .any(|profile| !profile.is_well_formed())
        {
            return false;
        }

        for (index, left) in self.profiles.iter().enumerate() {
            for right in self.profiles.iter().skip(index + 1) {
                if left.artifact_type == right.artifact_type && left.purpose == right.purpose {
                    return false;
                }
                if left.id == right.id && left.version == right.version {
                    return false;
                }
            }
        }

        true
    }

    /// Resolve exactly one profile for a reference.
    ///
    /// A single accepted context permits an omitted purpose. Multiple accepted
    /// contexts require an explicit purpose; there is never entry-order or
    /// context-preference fallback.
    pub fn resolve(&self, reference: &EvidenceReferenceV1) -> Option<&EvidenceReferenceProfileV1> {
        if !self.is_well_formed() || !reference.is_well_formed() {
            return None;
        }

        let candidates = self
            .profiles
            .iter()
            .filter(|profile| profile.artifact_type == reference.artifact_type)
            .collect::<Vec<_>>();

        if candidates.is_empty() {
            return None;
        }

        let selected = match reference.purpose.as_deref() {
            Some(purpose) => {
                let mut matches = candidates
                    .into_iter()
                    .filter(|profile| profile.purpose.as_deref() == Some(purpose));
                let selected = matches.next()?;
                if matches.next().is_some() {
                    return None;
                }
                selected
            }
            None if candidates.len() == 1 => candidates[0],
            None => return None,
        };

        selected.accepts(reference).then_some(selected)
    }

    pub fn canonical_bytes(&self) -> Vec<u8> {
        #[derive(Serialize)]
        struct CanonicalRegistry {
            schema: String,
            id: String,
            version: u32,
            profiles: Vec<EvidenceReferenceProfileV1>,
        }

        let mut profiles = self.profiles.clone();
        profiles.sort_by(|left, right| {
            (
                &left.artifact_type,
                &left.purpose,
                &left.id,
                left.version,
                &left.context,
                &left.claim_ceiling,
            )
                .cmp(&(
                    &right.artifact_type,
                    &right.purpose,
                    &right.id,
                    right.version,
                    &right.context,
                    &right.claim_ceiling,
                ))
        });

        let canonical = CanonicalRegistry {
            schema: Self::SCHEMA.into(),
            id: self.id.clone(),
            version: self.version,
            profiles,
        };

        serde_json::to_vec(&canonical)
            .expect("evidence profile registry contains only serializable primitives")
    }

    /// SHA-256 identity of this exact profile registry snapshot.
    pub fn digest(&self) -> String {
        let digest = Sha256::digest(self.canonical_bytes());
        format!("sha256:{digest:x}")
    }
}

/// A content-addressed, typed reference to one evidence artifact.
///
/// digest is an identity claim under context; it is not itself proof that the
/// artifact was retrieved or that recomputation succeeded.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceReferenceV1 {
    pub artifact_type: String,
    pub context: DigestContextV1,
    pub digest: String,
    /// Selects the profile-declared digest context when a type has multiple
    /// authorized contexts.
    pub purpose: Option<String>,
    pub display_label: Option<String>,
    pub claim_ceiling: String,
}

/// Canonical representation of an evidence reference for snapshot hashing.
///
/// Display labels are intentionally excluded: changing a human-facing label
/// must not change the referenced content identity. The claim ceiling remains
/// included because it changes what the evidence reference is allowed to claim.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceReferenceCanonicalV1 {
    pub artifact_type: String,
    pub context: DigestContextV1,
    pub digest: String,
    pub purpose: Option<String>,
    pub claim_ceiling: String,
}

impl From<&EvidenceReferenceV1> for EvidenceReferenceCanonicalV1 {
    fn from(reference: &EvidenceReferenceV1) -> Self {
        Self {
            artifact_type: reference.artifact_type.clone(),
            context: reference.context.clone(),
            digest: reference.digest.clone(),
            purpose: reference.purpose.clone(),
            claim_ceiling: reference.claim_ceiling.clone(),
        }
    }
}

impl EvidenceReferenceV1 {
    pub const SCHEMA: &'static str = "sol-atlas:evidence-reference:v1";

    fn digest_is_well_formed(&self) -> bool {
        let is_lower_hex = |value: &str| {
            value.len() == 64
                && value
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
        };

        match self.context.representation {
            DigestRepresentationV1::RawBytes => false,
            DigestRepresentationV1::LowerHex => is_lower_hex(&self.digest),
            DigestRepresentationV1::PrefixedLowerHex => {
                let Some((prefix, value)) = self.digest.split_once(':') else {
                    return false;
                };

                let expected_prefix = match self.context.hash_algorithm.as_str() {
                    "SHA-256" => "sha256",
                    _ => return false,
                };

                prefix == expected_prefix && is_lower_hex(value)
            }
        }
    }

    /// A reference is syntactically admissible only when it has an explicit
    /// artifact type, complete digest context, non-empty digest, and claim
    /// ceiling. This does not perform artifact retrieval or digest recomputation.
    pub fn is_well_formed(&self) -> bool {
        !self.artifact_type.is_empty()
            && self.context.is_well_formed()
            && self.digest_is_well_formed()
            && !self.claim_ceiling.is_empty()
            && self
                .purpose
                .as_ref()
                .is_none_or(|purpose| !purpose.is_empty())
    }

    /// Mutable URLs, branch names, labels, and bare IDs are intentionally not
    /// accepted as content identity by this constructor. The digest representation
    /// is validated structurally; actual retrieval and recomputation remain a
    /// separate verification step.
    pub fn content_addressed(
        artifact_type: impl Into<String>,
        context: DigestContextV1,
        digest: impl Into<String>,
        claim_ceiling: impl Into<String>,
    ) -> Result<Self, &'static str> {
        let reference = Self {
            artifact_type: artifact_type.into(),
            context,
            digest: digest.into(),
            purpose: None,
            display_label: None,
            claim_ceiling: claim_ceiling.into(),
        };

        if !reference.is_well_formed() {
            return Err("typed evidence reference is incomplete");
        }

        Ok(reference)
    }

    /// Recompute the digest over an explicitly supplied canonical preimage.
    ///
    /// This primitive intentionally does not guess or perform the declared
    /// canonicalization/preimage-construction transform. The caller must supply
    /// the exact bytes produced by that declared construction. V1 currently
    /// supports SHA-256 with textual lower-hex or prefixed-lower-hex output.
    pub fn verify_preimage(
        &self,
        preimage: &[u8],
        verifier: impl Into<String>,
        verified_at: impl Into<String>,
        claim_ceiling: impl Into<String>,
    ) -> Result<EvidenceReferenceVerificationV1, &'static str> {
        if !self.is_well_formed() {
            return Err("evidence reference is not well formed");
        }
        if self.context.hash_algorithm != "SHA-256" {
            return Err("unsupported digest algorithm for preimage verification");
        }

        let digest = Sha256::digest(preimage);
        let observed_digest = match self.context.representation {
            DigestRepresentationV1::RawBytes => {
                return Err("raw-byte digest representation is unsupported by V1");
            }
            DigestRepresentationV1::LowerHex => format!("{digest:x}"),
            DigestRepresentationV1::PrefixedLowerHex => format!("sha256:{digest:x}"),
        };

        if observed_digest != self.digest {
            return Err("recomputed digest does not match evidence reference");
        }

        let verifier = verifier.into();
        let verified_at = verified_at.into();
        let claim_ceiling = claim_ceiling.into();
        if verifier.is_empty() || verified_at.is_empty() || claim_ceiling.is_empty() {
            return Err("verification metadata is incomplete");
        }

        Ok(EvidenceReferenceVerificationV1 {
            reference: self.clone(),
            resolution: EvidenceReferenceResolutionV1::Verified,
            verifier: Some(verifier),
            verified_at: Some(verified_at),
            observed_digest: Some(observed_digest),
            profile_id: None,
            profile_version: None,
            profile_digest: None,
            registry_id: None,
            registry_version: None,
            registry_digest: None,
            claim_ceiling,
        })
    }

    /// Verify this reference only when an explicit profile authorizes
    /// its artifact type and exact digest context.
    ///
    /// Profile admission and digest recomputation remain distinct checks.
    pub fn verify_preimage_against_profile(
        &self,
        profile: &EvidenceReferenceProfileV1,
        preimage: &[u8],
        verifier: impl Into<String>,
        verified_at: impl Into<String>,
        claim_ceiling: impl Into<String>,
    ) -> Result<EvidenceReferenceVerificationV1, &'static str> {
        if !profile.accepts(self) {
            return Err("evidence reference is not authorized by profile");
        }
        let claim_ceiling = claim_ceiling.into();
        if claim_ceiling != profile.claim_ceiling {
            return Err("verification claim ceiling does not match profile");
        }
        let mut verification =
            self.verify_preimage(preimage, verifier, verified_at, claim_ceiling)?;
        verification.profile_id = Some(profile.id.clone());
        verification.profile_version = Some(profile.version);
        verification.profile_digest = Some(profile.digest());
        Ok(verification)
    }

    /// Verify this reference only through an explicit profile registry.
    ///
    /// Registry resolution is deterministic and fail-closed: an unknown type,
    /// ambiguous context, duplicate declaration, or purpose mismatch cannot be
    /// converted into a verification success.
    pub fn verify_preimage_against_registry(
        &self,
        registry: &EvidenceReferenceProfileRegistryV1,
        preimage: &[u8],
        verifier: impl Into<String>,
        verified_at: impl Into<String>,
        claim_ceiling: impl Into<String>,
    ) -> Result<EvidenceReferenceVerificationV1, &'static str> {
        let profile = registry
            .resolve(self)
            .ok_or("evidence reference cannot be resolved by profile registry")?;
        let mut verification = self.verify_preimage_against_profile(
            profile,
            preimage,
            verifier,
            verified_at,
            claim_ceiling,
        )?;
        verification.registry_id = Some(registry.id.clone());
        verification.registry_version = Some(registry.version);
        verification.registry_digest = Some(registry.digest());
        Ok(verification)
    }

    /// Preserve a legacy/bare evidence locator explicitly as unresolved.
    ///
    /// The locator is carried as display metadata only; the empty digest keeps
    /// the reference structurally inadmissible until a real content identity is
    /// supplied and verified.
    pub fn unresolved_legacy(label: impl Into<String>, claim_ceiling: impl Into<String>) -> Self {
        Self {
            artifact_type: "legacy/unresolved".into(),
            context: DigestContextV1 {
                id: "sol-atlas:unresolved-legacy:v1".into(),
                preimage_construction: "unresolved; no content identity asserted".into(),
                canonicalization: "none".into(),
                hash_algorithm: "none".into(),
                domain_separator: "sol-atlas:unresolved:v1".into(),
                preimage_encoding: "none".into(),
                representation: DigestRepresentationV1::RawBytes,
            },
            digest: String::new(),
            purpose: None,
            display_label: Some(label.into()),
            claim_ceiling: claim_ceiling.into(),
        }
    }

    /// Compare the immutable content identity only; human labels are irrelevant.
    pub fn same_content_identity(&self, other: &Self) -> bool {
        self.artifact_type == other.artifact_type
            && self.context == other.context
            && self.digest == other.digest
    }

    /// Bind an explicit context-selection purpose to this reference.
    pub fn with_purpose(mut self, purpose: impl Into<String>) -> Result<Self, &'static str> {
        let purpose = purpose.into();
        if purpose.is_empty() {
            return Err("reference purpose must not be empty");
        }
        self.purpose = Some(purpose);
        Ok(self)
    }

    pub fn with_display_label(mut self, label: impl Into<String>) -> Self {
        self.display_label = Some(label.into());
        self
    }
}

/// Processing state for a typed evidence reference.
///
/// Unresolved is explicitly not evidence of content binding. Verified means a
/// verifier obtained the referenced artifact, applied the declared digest
/// context, and recomputed the supplied digest successfully.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EvidenceReferenceResolutionV1 {
    Unresolved,
    Failed,
    Verified,
}

impl EvidenceReferenceResolutionV1 {
    pub fn permits_evidence_use(self) -> bool {
        matches!(self, Self::Verified)
    }
}

/// A typed evidence reference plus the result of processing that reference.
///
/// Keeping processing state outside EvidenceReferenceV1 preserves the
/// distinction between immutable identity and a verifier's observation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceReferenceVerificationV1 {
    pub reference: EvidenceReferenceV1,
    pub resolution: EvidenceReferenceResolutionV1,
    pub verifier: Option<String>,
    pub verified_at: Option<String>,
    pub observed_digest: Option<String>,
    /// Exact profile identity used to authorize this verification, when applicable.
    pub profile_id: Option<String>,
    pub profile_version: Option<u32>,
    pub profile_digest: Option<String>,
    /// Exact profile-registry identity used for authorization, when applicable.
    pub registry_id: Option<String>,
    pub registry_version: Option<u32>,
    pub registry_digest: Option<String>,
    pub claim_ceiling: String,
}

impl EvidenceReferenceVerificationV1 {
    pub fn is_verified(&self) -> bool {
        self.resolution == EvidenceReferenceResolutionV1::Verified
            && self.reference.is_well_formed()
            && self.verifier.as_ref().is_some_and(|v| !v.is_empty())
            && self.verified_at.as_ref().is_some_and(|t| !t.is_empty())
            && self.observed_digest.as_ref() == Some(&self.reference.digest)
            && !self.claim_ceiling.is_empty()
            && self.claim_ceiling == self.reference.claim_ceiling
            && match (&self.profile_id, self.profile_version, &self.profile_digest) {
                (None, None, None) => true,
                (Some(id), Some(version), Some(digest)) => {
                    !id.is_empty() && version > 0 && !digest.is_empty()
                }
                _ => false,
            }
            && match (
                &self.registry_id,
                self.registry_version,
                &self.registry_digest,
            ) {
                (None, None, None) => true,
                (Some(id), Some(version), Some(digest)) => {
                    !id.is_empty() && version > 0 && !digest.is_empty()
                }
                _ => false,
            }
    }

    /// Strictly verify that this processing result was authorized by the exact
    /// supplied profile declaration, including its content-addressed identity.
    ///
    /// Unlike is_verified(), this method binds processing to a concrete
    /// authorization object rather than merely checking embedded metadata.
    pub fn is_verified_against_profile(&self, profile: &EvidenceReferenceProfileV1) -> bool {
        self.is_verified()
            && profile.is_well_formed()
            && profile.accepts(&self.reference)
            && self.profile_id.as_deref() == Some(profile.id.as_str())
            && self.profile_version == Some(profile.version)
            && self.profile_digest.as_deref() == Some(profile.digest().as_str())
    }

    /// Strictly verify that this result remains authorized by the supplied
    /// profile registry snapshot and resolves to the exact embedded profile.
    pub fn is_verified_against_registry(
        &self,
        registry: &EvidenceReferenceProfileRegistryV1,
    ) -> bool {
        let Some(profile) = registry.resolve(&self.reference) else {
            return false;
        };

        self.is_verified_against_profile(profile)
            && self.registry_id.as_deref() == Some(registry.id.as_str())
            && self.registry_version == Some(registry.version)
            && self.registry_digest.as_deref() == Some(registry.digest().as_str())
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    fn context() -> DigestContextV1 {
        DigestContextV1 {
            id: "sol-atlas:sha256-jcs:v1".into(),
            preimage_construction: "artifact bytes after declared canonicalization".into(),
            canonicalization: "RFC8785-JCS".into(),
            hash_algorithm: "SHA-256".into(),
            domain_separator: "sol-atlas:evidence:v1".into(),
            preimage_encoding: "UTF-8".into(),
            representation: DigestRepresentationV1::PrefixedLowerHex,
        }
    }

    fn reference() -> EvidenceReferenceV1 {
        EvidenceReferenceV1::content_addressed(
            "recovery-execution-record",
            context(),
            "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "Exact artifact identity only.",
        )
        .unwrap()
    }

    #[test]
    fn profile_accepts_only_exact_artifact_type_and_context() {
        let mut reference = reference();
        reference.claim_ceiling = "Profile-scoped evidence only.".into();
        let reference = reference.with_purpose("recovery-verification").unwrap();
        let profile = EvidenceReferenceProfileV1 {
            id: "profile-001".into(),
            version: 1,
            artifact_type: reference.artifact_type.clone(),
            context: reference.context.clone(),
            purpose: Some("recovery-verification".into()),
            claim_ceiling: "Profile-scoped evidence only.".into(),
        };

        assert!(profile.accepts(&reference));

        let mut broader_claim = reference.clone();
        broader_claim.claim_ceiling = "Broader claim than authorized profile.".into();
        assert!(!profile.accepts(&broader_claim));

        let mut wrong_type = reference.clone();
        wrong_type.artifact_type = "other-artifact".into();
        assert!(!profile.accepts(&wrong_type));

        let mut wrong_context = reference.clone();
        wrong_context.context.domain_separator = "other-domain:v1".into();
        assert!(!profile.accepts(&wrong_context));

        let wrong_purpose = reference.clone().with_purpose("other-purpose").unwrap();
        assert!(!profile.accepts(&wrong_purpose));
    }

    #[test]
    fn profile_binds_preimage_verification_to_authorized_context() {
        let preimage = b"profile-bound-evidence";
        let digest = Sha256::digest(preimage);
        let reference = EvidenceReferenceV1::content_addressed(
            "recovery-execution-record",
            context(),
            format!("sha256:{digest:x}"),
            "Profile-scoped evidence only.",
        )
        .unwrap()
        .with_purpose("recovery-verification")
        .unwrap();
        let profile = EvidenceReferenceProfileV1 {
            id: "profile-002".into(),
            version: 1,
            artifact_type: reference.artifact_type.clone(),
            context: reference.context.clone(),
            purpose: Some("recovery-verification".into()),
            claim_ceiling: "Profile-scoped evidence only.".into(),
        };

        assert!(
            reference
                .verify_preimage_against_profile(
                    &profile,
                    preimage,
                    "deterministic-verifier",
                    "2026-10-02T00:00:00Z",
                    "Profile-scoped evidence only.",
                )
                .unwrap()
                .is_verified()
        );

        let mut mutated_profile = profile.clone();
        mutated_profile.context.domain_separator = "other-domain:v1".into();
        assert!(
            reference
                .verify_preimage_against_profile(
                    &mutated_profile,
                    preimage,
                    "deterministic-verifier",
                    "2026-10-02T00:00:00Z",
                    "Profile-scoped evidence only.",
                )
                .is_err()
        );

        let profile = EvidenceReferenceProfileV1 {
            id: "profile-003".into(),
            version: 1,
            artifact_type: reference.artifact_type.clone(),
            context: reference.context.clone(),
            purpose: reference.purpose.clone(),
            claim_ceiling: reference.claim_ceiling.clone(),
        };
        assert!(
            reference
                .verify_preimage_against_profile(
                    &profile,
                    preimage,
                    "deterministic-verifier",
                    "2026-10-02T00:00:00Z",
                    "Broader than profile.",
                )
                .is_err()
        );
    }

    #[test]
    fn profile_bound_verification_records_authorizing_profile_identity() {
        let preimage = b"profile-audit";
        let digest = Sha256::digest(preimage);
        let reference = EvidenceReferenceV1::content_addressed(
            "recovery-execution-record",
            context(),
            format!("sha256:{digest:x}"),
            "Profile-scoped evidence only.",
        )
        .unwrap()
        .with_purpose("recovery-verification")
        .unwrap();
        let profile = EvidenceReferenceProfileV1 {
            id: "profile-audit-001".into(),
            version: 1,
            artifact_type: reference.artifact_type.clone(),
            context: reference.context.clone(),
            purpose: Some("recovery-verification".into()),
            claim_ceiling: reference.claim_ceiling.clone(),
        };

        let verification = reference
            .verify_preimage_against_profile(
                &profile,
                preimage,
                "deterministic-verifier",
                "2026-10-02T00:00:00Z",
                "Profile-scoped evidence only.",
            )
            .unwrap();

        assert!(verification.is_verified());
        assert_eq!(
            verification.profile_id.as_deref(),
            Some("profile-audit-001")
        );
        assert_eq!(verification.profile_version, Some(1));
        assert_eq!(
            verification.profile_digest.as_deref(),
            Some(profile.digest().as_str())
        );
        assert!(verification.is_verified_against_profile(&profile));

        let mut changed = profile.clone();
        changed.version = 2;
        assert!(!verification.is_verified_against_profile(&changed));
    }

    #[test]
    fn registry_bound_verification_records_and_checks_registry_identity() {
        let preimage = b"registry-audit";
        let digest = Sha256::digest(preimage);
        let reference = EvidenceReferenceV1::content_addressed(
            "recovery-execution-record",
            context(),
            format!("sha256:{digest:x}"),
            "Profile-scoped evidence only.",
        )
        .unwrap()
        .with_purpose("recovery-verification")
        .unwrap();
        let profile = EvidenceReferenceProfileV1 {
            id: "registry-check-profile".into(),
            version: 1,
            artifact_type: reference.artifact_type.clone(),
            context: reference.context.clone(),
            purpose: Some("recovery-verification".into()),
            claim_ceiling: reference.claim_ceiling.clone(),
        };
        let registry = EvidenceReferenceProfileRegistryV1 {
            id: "registry-check".into(),
            version: 1,
            profiles: vec![profile],
        };

        let verification = reference
            .verify_preimage_against_registry(
                &registry,
                preimage,
                "deterministic-verifier",
                "2026-10-02T00:00:00Z",
                "Profile-scoped evidence only.",
            )
            .unwrap();

        assert!(verification.is_verified_against_registry(&registry));
        let mut changed = registry.clone();
        changed.version = 2;
        assert!(!verification.is_verified_against_registry(&changed));
    }

    #[test]
    fn malformed_profile_binding_fails_closed() {
        let reference = reference();
        let mut verification = EvidenceReferenceVerificationV1 {
            reference: reference.clone(),
            resolution: EvidenceReferenceResolutionV1::Verified,
            verifier: Some("deterministic-verifier".into()),
            verified_at: Some("2026-10-02T00:00:00Z".into()),
            observed_digest: Some(reference.digest.clone()),
            profile_id: Some("profile-audit-002".into()),
            profile_version: None,
            profile_digest: Some(
                "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
            ),
            registry_id: None,
            registry_version: None,
            registry_digest: None,
            claim_ceiling: reference.claim_ceiling.clone(),
        };

        assert!(!verification.is_verified());
        verification.profile_version = Some(1);
        assert!(verification.is_verified());
    }

    #[test]
    fn profile_registry_resolves_single_context_without_purpose() {
        let mut reference = reference();
        reference.claim_ceiling = "Profile-scoped evidence only.".into();
        let profile = EvidenceReferenceProfileV1 {
            id: "registry-profile-001".into(),
            version: 1,
            artifact_type: reference.artifact_type.clone(),
            context: reference.context.clone(),
            purpose: None,
            claim_ceiling: "Profile-scoped evidence only.".into(),
        };
        let registry = EvidenceReferenceProfileRegistryV1 {
            id: "registry-001".into(),
            version: 1,
            profiles: vec![profile],
        };

        assert!(registry.is_well_formed());
        assert!(registry.resolve(&reference).is_some());
    }

    #[test]
    fn profile_registry_requires_purpose_when_contexts_are_multiple() {
        let mut reference = reference();
        reference.claim_ceiling = "Profile-scoped evidence only.".into();

        let mut second_context = reference.context.clone();
        second_context.domain_separator = "sol-atlas:alternate:v1".into();

        let first = EvidenceReferenceProfileV1 {
            id: "registry-profile-002".into(),
            version: 1,
            artifact_type: reference.artifact_type.clone(),
            context: reference.context.clone(),
            purpose: Some("recovery-verification".into()),
            claim_ceiling: reference.claim_ceiling.clone(),
        };
        let second = EvidenceReferenceProfileV1 {
            id: "registry-profile-003".into(),
            version: 1,
            artifact_type: reference.artifact_type.clone(),
            context: second_context,
            purpose: Some("recovery-replay".into()),
            claim_ceiling: reference.claim_ceiling.clone(),
        };
        let registry = EvidenceReferenceProfileRegistryV1 {
            id: "registry-002".into(),
            version: 1,
            profiles: vec![first.clone(), second],
        };

        assert!(registry.is_well_formed());
        assert!(registry.resolve(&reference).is_none());

        let explicit = reference
            .clone()
            .with_purpose("recovery-verification")
            .unwrap();
        assert!(registry.resolve(&explicit).is_some());
        assert_eq!(
            registry
                .resolve(&explicit)
                .map(|profile| profile.id.as_str()),
            Some("registry-profile-002")
        );
    }

    #[test]
    fn profile_registry_rejects_duplicate_type_and_purpose() {
        let reference = reference();
        let profile = EvidenceReferenceProfileV1 {
            id: "registry-profile-004".into(),
            version: 1,
            artifact_type: reference.artifact_type.clone(),
            context: reference.context.clone(),
            purpose: Some("recovery-verification".into()),
            claim_ceiling: reference.claim_ceiling.clone(),
        };
        let mut duplicate = profile.clone();
        duplicate.id = "registry-profile-005".into();

        let registry = EvidenceReferenceProfileRegistryV1 {
            id: "registry-003".into(),
            version: 1,
            profiles: vec![profile, duplicate],
        };

        assert!(!registry.is_well_formed());
        let explicit = reference.with_purpose("recovery-verification").unwrap();
        assert!(registry.resolve(&explicit).is_none());
    }

    #[test]
    fn profile_registry_digest_changes_with_version() {
        let reference = reference();
        let profile = EvidenceReferenceProfileV1 {
            id: "registry-profile-006".into(),
            version: 1,
            artifact_type: reference.artifact_type.clone(),
            context: reference.context.clone(),
            purpose: Some("recovery-verification".into()),
            claim_ceiling: reference.claim_ceiling.clone(),
        };
        let registry = EvidenceReferenceProfileRegistryV1 {
            id: "registry-004".into(),
            version: 1,
            profiles: vec![profile],
        };
        let mut changed = registry.clone();
        changed.version = 2;

        assert_ne!(registry.digest(), changed.digest());
    }

    #[test]
    fn profile_identity_changes_when_version_or_purpose_changes() {
        let base = EvidenceReferenceProfileV1 {
            id: "profile-identity".into(),
            version: 1,
            artifact_type: "recovery-execution-record".into(),
            context: context(),
            purpose: Some("recovery-verification".into()),
            claim_ceiling: "Profile-scoped evidence only.".into(),
        };

        let mut versioned = base.clone();
        versioned.version = 2;
        assert_ne!(base.digest(), versioned.digest());

        let mut repurposed = base.clone();
        repurposed.purpose = Some("recovery-planning".into());
        assert_ne!(base.digest(), repurposed.digest());

        let mut relabeled = base.clone();
        relabeled.id = "different-profile-name".into();
        assert_ne!(base.digest(), relabeled.digest());
    }

    #[test]
    fn typed_reference_requires_explicit_digest_context() {
        assert!(reference().is_well_formed());

        let mut context = context();
        context.canonicalization.clear();

        assert!(
            EvidenceReferenceV1::content_addressed(
                "recovery-execution-record",
                context,
                "sha256:aaaaaaaa",
                "Exact artifact identity only.",
            )
            .is_err()
        );
    }

    #[test]
    fn bare_or_mutable_digest_values_are_rejected() {
        let context = context();
        for digest in [
            "branch/main",
            "https://example.invalid/evidence",
            "artifact-123",
        ] {
            assert!(
                EvidenceReferenceV1::content_addressed(
                    "recovery-execution-record",
                    context.clone(),
                    digest,
                    "Exact artifact identity only.",
                )
                .is_err(),
                "mutable/non-digest value unexpectedly accepted: {digest}"
            );
        }
    }

    #[test]
    fn digest_representation_rejects_invalid_hex() {
        let mut context = context();
        context.representation = DigestRepresentationV1::LowerHex;
        assert!(
            EvidenceReferenceV1::content_addressed(
                "recovery-execution-record",
                context,
                "not-a-hex-digest",
                "Exact artifact identity only.",
            )
            .is_err()
        );
    }

    #[test]
    fn lower_hex_rejects_uppercase_digits() {
        let mut context = context();
        context.representation = DigestRepresentationV1::LowerHex;

        assert!(
            EvidenceReferenceV1::content_addressed(
                "recovery-execution-record",
                context,
                "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
                "Exact artifact identity only.",
            )
            .is_err()
        );
    }

    #[test]
    fn prefixed_digest_requires_algorithm_matching_prefix() {
        let context = context();

        assert!(
            EvidenceReferenceV1::content_addressed(
                "recovery-execution-record",
                context,
                "sha512:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
                "Exact artifact identity only.",
            )
            .is_err()
        );
    }

    #[test]
    fn prefixed_lower_hex_rejects_uppercase_digits() {
        let context = context();

        assert!(
            EvidenceReferenceV1::content_addressed(
                "recovery-execution-record",
                context,
                "sha256:AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
                "Exact artifact identity only.",
            )
            .is_err()
        );
    }

    #[test]
    fn prefixed_digest_requires_nonempty_hex_suffix() {
        let context = context();
        for digest in ["sha256:", "sha256:xyz", "branch/main"] {
            assert!(
                EvidenceReferenceV1::content_addressed(
                    "recovery-execution-record",
                    context.clone(),
                    digest,
                    "Exact artifact identity only.",
                )
                .is_err()
            );
        }
    }

    #[test]
    fn labels_are_not_content_identity() {
        let reference = reference().with_display_label("branch/main");
        assert!(reference.is_well_formed());
        assert_eq!(reference.display_label.as_deref(), Some("branch/main"));
    }

    #[test]
    fn digest_context_mutation_changes_identity_context() {
        let first = reference();

        let mut second = first.clone();
        second.context.domain_separator = "other-domain:v1".into();

        assert_ne!(first.context, second.context);
        assert_eq!(first.digest, second.digest);
    }

    #[test]
    fn incompatible_digest_contexts_do_not_share_content_identity() {
        let first = reference();
        let mut second = first.clone();
        second.context.canonicalization = "other-canonicalization:v1".into();

        assert!(!first.same_content_identity(&second));
    }

    #[test]
    fn raw_bytes_representation_fails_closed_for_string_digests() {
        let mut context = context();
        context.representation = DigestRepresentationV1::RawBytes;

        assert!(
            EvidenceReferenceV1::content_addressed(
                "recovery-execution-record",
                context,
                "opaque-bytes",
                "Exact artifact identity only.",
            )
            .is_err()
        );
    }

    #[test]
    fn digest_representation_is_part_of_context() {
        let mut lower_hex = context();
        lower_hex.representation = DigestRepresentationV1::LowerHex;

        let mut prefixed = context();
        prefixed.representation = DigestRepresentationV1::PrefixedLowerHex;

        assert_ne!(lower_hex, prefixed);
    }

    #[test]
    fn display_label_does_not_change_content_identity() {
        let first = reference();
        let second = first.clone().with_display_label("human readable record");

        assert_eq!(first.digest, second.digest);
        assert_eq!(first.context, second.context);
        assert_ne!(first, second);
    }

    #[test]
    fn unresolved_reference_does_not_permit_evidence_use() {
        assert!(!EvidenceReferenceResolutionV1::Unresolved.permits_evidence_use());
        assert!(!EvidenceReferenceResolutionV1::Failed.permits_evidence_use());
        assert!(EvidenceReferenceResolutionV1::Verified.permits_evidence_use());
    }

    #[test]
    fn verify_preimage_recomputes_the_declared_digest() {
        let preimage = b"deterministic evidence payload";
        let digest = Sha256::digest(preimage);

        let reference = EvidenceReferenceV1::content_addressed(
            "recovery-execution-record",
            DigestContextV1 {
                id: "sol-atlas:test-sha256:v1".into(),
                preimage_construction: "exact supplied canonical bytes".into(),
                canonicalization: "sol-atlas-test-canonical-v1".into(),
                hash_algorithm: "SHA-256".into(),
                domain_separator: "sol-atlas:test-evidence:v1".into(),
                preimage_encoding: "UTF-8".into(),
                representation: DigestRepresentationV1::PrefixedLowerHex,
            },
            format!("sha256:{digest:x}"),
            "Exact artifact identity only.",
        )
        .unwrap();

        let verification = reference
            .verify_preimage(
                preimage,
                "deterministic-verifier",
                "2026-10-02T00:00:00Z",
                "Exact artifact identity only.",
            )
            .unwrap();

        assert!(verification.is_verified());
        assert_eq!(
            verification.observed_digest.as_deref(),
            Some(reference.digest.as_str())
        );

        assert!(
            reference
                .verify_preimage(
                    b"tampered evidence payload",
                    "deterministic-verifier",
                    "2026-10-02T00:00:00Z",
                    "Exact artifact identity verified.",
                )
                .is_err()
        );
    }

    #[test]
    fn verified_reference_requires_recomputed_digest_and_verifier_metadata() {
        let reference = reference();

        let unresolved = EvidenceReferenceVerificationV1 {
            reference: reference.clone(),
            resolution: EvidenceReferenceResolutionV1::Unresolved,
            verifier: None,
            verified_at: None,
            observed_digest: None,
            claim_ceiling: "Exact artifact only.".into(),
        };
        assert!(!unresolved.is_verified());

        let verified = EvidenceReferenceVerificationV1 {
            reference: reference.clone(),
            resolution: EvidenceReferenceResolutionV1::Verified,
            verifier: Some("deterministic-verifier".into()),
            verified_at: Some("2026-10-02T00:00:00Z".into()),
            observed_digest: Some(reference.digest.clone()),
            profile_id: None,
            profile_version: None,
            profile_digest: None,
            registry_id: None,
            registry_version: None,
            registry_digest: None,
            claim_ceiling: reference.claim_ceiling.clone(),
        };
        assert!(verified.is_verified());

        let mut mutated = verified;
        mutated.observed_digest = Some("sha256:bbbb".into());
        assert!(!mutated.is_verified());
    }

    #[test]
    fn claim_ceiling_remains_explicit() {
        let mut reference = reference();
        reference.claim_ceiling =
            "Exact record identity only; payload truth not established.".into();
        assert!(reference.claim_ceiling.contains("truth not established"));
    }
}
