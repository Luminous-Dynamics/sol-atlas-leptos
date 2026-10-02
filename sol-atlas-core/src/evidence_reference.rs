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


/// A content-addressed, typed reference to one evidence artifact.
///
/// digest is an identity claim under context; it is not itself proof that the
/// artifact was retrieved or that recomputation succeeded.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EvidenceReferenceV1 {
    pub artifact_type: String,
    pub context: DigestContextV1,
    pub digest: String,
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
    pub claim_ceiling: String,
}

impl From<&EvidenceReferenceV1> for EvidenceReferenceCanonicalV1 {
    fn from(reference: &EvidenceReferenceV1) -> Self {
        Self {
            artifact_type: reference.artifact_type.clone(),
            context: reference.context.clone(),
            digest: reference.digest.clone(),
            claim_ceiling: reference.claim_ceiling.clone(),
        }
    }
}

impl EvidenceReferenceV1 {
    pub const SCHEMA: &'static str = "sol-atlas:evidence-reference:v1";

    fn digest_is_well_formed(&self) -> bool {
        match self.context.representation {
            DigestRepresentationV1::RawBytes => false,
            DigestRepresentationV1::LowerHex => {
                self.digest.len() == 64 && self.digest.bytes().all(|b| b.is_ascii_hexdigit())
            }
            DigestRepresentationV1::PrefixedLowerHex => {
                let Some((prefix, value)) = self.digest.split_once(':') else {
                    return false;
                };
                !prefix.is_empty()
                    && value.len() == 64
                    && value.bytes().all(|b| b.is_ascii_hexdigit())
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
            display_label: None,
            claim_ceiling: claim_ceiling.into(),
        };

        if !reference.is_well_formed() {
            return Err("typed evidence reference is incomplete");
        }

        Ok(reference)
    }

    /// Preserve a legacy/bare evidence locator explicitly as unresolved.
    ///
    /// The locator is carried as display metadata only; the empty digest keeps
    /// the reference structurally inadmissible until a real content identity is
    /// supplied and verified.
    pub fn unresolved_legacy(
        label: impl Into<String>,
        claim_ceiling: impl Into<String>,
    ) -> Self {
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
    fn typed_reference_requires_explicit_digest_context() {
        assert!(reference().is_well_formed());

        let mut context = context();
        context.canonicalization.clear();

        assert!(EvidenceReferenceV1::content_addressed(
            "recovery-execution-record",
            context,
            "sha256:aaaaaaaa",
            "Exact artifact identity only.",
        )
        .is_err());
    }

    #[test]
    fn bare_or_mutable_digest_values_are_rejected() {
        let context = context();
        for digest in ["branch/main", "https://example.invalid/evidence", "artifact-123"] {
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
            claim_ceiling: "Exact artifact identity verified.".into(),
        };
        assert!(verified.is_verified());

        let mut mutated = verified;
        mutated.observed_digest = Some("sha256:bbbb".into());
        assert!(!mutated.is_verified());
    }

    #[test]
    fn claim_ceiling_remains_explicit() {
        let mut reference = reference();
        reference.claim_ceiling = "Exact record identity only; payload truth not established.".into();
        assert!(reference.claim_ceiling.contains("truth not established"));
    }
}
