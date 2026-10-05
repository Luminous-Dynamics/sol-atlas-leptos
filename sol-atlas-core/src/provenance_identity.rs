// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Self-describing portable provenance identities.
//!
//! This layer is additive to the existing replay receipt contracts. It makes
//! the digest algorithm, canonicalization contract, and payload domain explicit
//! without changing legacy receipt hashes.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::cultural_projection_replay::V5ReplayReceiptV1;
use crate::civilizational::ProjectionError;

pub const PORTABLE_PROVENANCE_IDENTITY_SCHEMA_V1: &str =
    "sol-atlas:portable-provenance-identity:v1";
pub const V5_REPLAY_RECEIPT_DIGEST_DOMAIN_V1: &str =
    "sol-atlas:v5-replay-receipt:portable-identity:v1";
pub const SERDE_JSON_TUPLE_CANONICALIZATION_V1: &str = "serde-json-tuple-v1";
pub const SHA256_ALGORITHM_V1: &str = "sha256";

fn is_sha256_hex(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

/// Explicit digest metadata for external verifiers.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProvenanceDigestV1 {
    pub algorithm: String,
    pub canonicalization: String,
    pub domain: String,
    pub digest: String,
}

impl ProvenanceDigestV1 {
    pub fn validate(&self) -> Result<(), ProjectionError> {
        if self.algorithm != SHA256_ALGORITHM_V1
            || self.canonicalization != SERDE_JSON_TUPLE_CANONICALIZATION_V1
            || self.domain != V5_REPLAY_RECEIPT_DIGEST_DOMAIN_V1
            || !is_sha256_hex(&self.digest)
        {
            return Err(ProjectionError::InvalidEvidenceFrontierManifest);
        }
        Ok(())
    }
}

/// Portable identity for a validated V5 replay receipt.
///
/// The identity hashes only the already-validated receipt. A construction
/// failure is returned before the digest object exists, so an error variant,
/// display string, or diagnostic code is never part of this identity.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct V5PortableReplayIdentityV1 {
    pub schema: String,
    pub payload_kind: String,
    pub source_receipt_hash: String,
    pub content_digest: ProvenanceDigestV1,
}

impl V5PortableReplayIdentityV1 {
    pub fn from_receipt(
        receipt: &V5ReplayReceiptV1,
    ) -> Result<Self, ProjectionError> {
        receipt.validate()?;
        Self::from_validated_receipt(receipt)
    }

    fn from_validated_receipt(
        receipt: &V5ReplayReceiptV1,
    ) -> Result<Self, ProjectionError> {
        let payload = (
            V5_REPLAY_RECEIPT_DIGEST_DOMAIN_V1,
            SERDE_JSON_TUPLE_CANONICALIZATION_V1,
            SHA256_ALGORITHM_V1,
            receipt,
        );
        let bytes = serde_json::to_vec(&payload)
            .map_err(|_| ProjectionError::InvalidEvidenceFrontierManifest)?;
        let digest = Sha256::digest(bytes);
        let digest = digest.iter().map(|byte| format!("{byte:02x}")).collect();

        let identity = Self {
            schema: PORTABLE_PROVENANCE_IDENTITY_SCHEMA_V1.into(),
            payload_kind: "v5_replay_receipt".into(),
            source_receipt_hash: receipt.receipt_hash.clone(),
            content_digest: ProvenanceDigestV1 {
                algorithm: SHA256_ALGORITHM_V1.into(),
                canonicalization: SERDE_JSON_TUPLE_CANONICALIZATION_V1.into(),
                domain: V5_REPLAY_RECEIPT_DIGEST_DOMAIN_V1.into(),
                digest,
            },
        };
        identity.validate()?;
        Ok(identity)
    }

    pub fn validate(&self) -> Result<(), ProjectionError> {
        if self.schema != PORTABLE_PROVENANCE_IDENTITY_SCHEMA_V1
            || self.payload_kind != "v5_replay_receipt"
            || !is_sha256_hex(&self.source_receipt_hash)
        {
            return Err(ProjectionError::InvalidEvidenceFrontierManifest);
        }
        self.content_digest.validate()
    }

    pub fn computed_digest(
        &self,
        receipt: &V5ReplayReceiptV1,
    ) -> Result<String, ProjectionError> {
        receipt.validate()?;
        let payload = (
            V5_REPLAY_RECEIPT_DIGEST_DOMAIN_V1,
            SERDE_JSON_TUPLE_CANONICALIZATION_V1,
            SHA256_ALGORITHM_V1,
            receipt,
        );
        let bytes = serde_json::to_vec(&payload)
            .map_err(|_| ProjectionError::InvalidEvidenceFrontierManifest)?;
        let digest = Sha256::digest(bytes);
        Ok(digest.iter().map(|byte| format!("{byte:02x}")).collect())
    }

    pub fn validate_against_receipt(
        &self,
        receipt: &V5ReplayReceiptV1,
    ) -> Result<(), ProjectionError> {
        self.validate()?;
        receipt.validate()?;
        if self.source_receipt_hash != receipt.receipt_hash
            || self.content_digest.digest != self.computed_digest(receipt)?
        {
            return Err(ProjectionError::InvalidEvidenceFrontierManifest);
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> V5ReplayReceiptV1 {
        V5ReplayReceiptV1 {
            audit_semantic_hash: "a".repeat(64),
            projection_id: crate::cultural_systems::CulturalProjectionIdV1::Transmission(
                "transmission:1".into(),
            ),
            claim_ref: "claim:1".into(),
            evidence_refs: vec!["e:1".into()],
            source_snapshots: vec!["source:1".into()],
            argumentation: vec![],
            ontology_resolutions: vec![],
            frontier_lineage: vec![("frontier:1950".into(), "b".repeat(64))],
            leaf_frontier: "frontier:1950".into(),
            qualification: crate::civilizational::QualificationStatus::Supported,
            receipt_hash: "c".repeat(64),
        }
    }

    #[test]
    fn portable_identity_is_self_describing() {
        let mut receipt = fixture();
        receipt.recompute_hash().expect("receipt hash");
        let identity = V5PortableReplayIdentityV1::from_receipt(&receipt)
            .expect("portable identity");
        assert_eq!(identity.schema, PORTABLE_PROVENANCE_IDENTITY_SCHEMA_V1);
        assert_eq!(identity.payload_kind, "v5_replay_receipt");
        assert_eq!(
            identity.content_digest.algorithm,
            SHA256_ALGORITHM_V1
        );
        assert_eq!(
            identity.content_digest.canonicalization,
            SERDE_JSON_TUPLE_CANONICALIZATION_V1
        );
        assert_eq!(
            identity.content_digest.domain,
            V5_REPLAY_RECEIPT_DIGEST_DOMAIN_V1
        );
        assert_eq!(
            identity.source_receipt_hash,
            receipt.receipt_hash
        );
        assert!(identity.validate_against_receipt(&receipt).is_ok());
    }

    #[test]
    fn portable_identity_rejects_receipt_rebinding() {
        let mut receipt = fixture();
        receipt.recompute_hash().expect("receipt hash");
        let identity = V5PortableReplayIdentityV1::from_receipt(&receipt)
            .expect("portable identity");

        let mut rebound = receipt.clone();
        rebound.leaf_frontier = "frontier:other".into();
        rebound.recompute_hash().expect("rebound receipt hash");

        assert_eq!(
            identity.validate_against_receipt(&rebound),
            Err(ProjectionError::InvalidEvidenceFrontierManifest)
        );
    }

    #[test]
    fn portable_identity_construction_failures_are_not_hashed() {
        let mut receipt = fixture();
        receipt.receipt_hash = String::new();

        assert_eq!(
            V5PortableReplayIdentityV1::from_receipt(&receipt),
            Err(ProjectionError::InvalidEvidenceFrontierManifest)
        );
    }

    #[test]
    fn portable_digest_domain_tamper_is_rejected() {
        let mut receipt = fixture();
        receipt.recompute_hash().expect("receipt hash");
        let mut identity = V5PortableReplayIdentityV1::from_receipt(&receipt)
            .expect("portable identity");
        identity.content_digest.domain = "other-domain".into();

        assert_eq!(
            identity.validate(),
            Err(ProjectionError::InvalidEvidenceFrontierManifest)
        );
    }

    #[test]
    fn portable_digest_metadata_tamper_is_rejected() {
        let mut receipt = fixture();
        receipt.recompute_hash().expect("receipt hash");
        let mut identity = V5PortableReplayIdentityV1::from_receipt(&receipt)
            .expect("portable identity");
        identity.content_digest.algorithm = "sha512".into();

        assert_eq!(
            identity.validate_against_receipt(&receipt),
            Err(ProjectionError::InvalidEvidenceFrontierManifest)
        );
    }
}
