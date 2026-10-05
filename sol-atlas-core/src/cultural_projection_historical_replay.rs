// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Historical V5 replay against an explicitly selected evidence frontier.
//!
//! This module validates the append-only prefix ending at the requested
//! frontier. Later frontier contents are intentionally outside the validation
//! scope, but their identifiers are checked for global uniqueness because
//! selection itself is identifier-based.

use crate::civilizational::{EvidenceFrontierChainV1, EvidenceFrontierId, ProjectionError};
use crate::cultural_projection_audit_v5::CulturalProjectionAuditV5;
use crate::cultural_systems::CanonicalClaimAdmissionV1;

/// Validate a V5 audit at a selected historical frontier.
///
/// The selected frontier must be present in the supplied ordered chain. Only
/// the chain prefix through that frontier is validated. The audit and canonical
/// claim must both name the selected frontier and be safe against its exact
/// admission manifest.
pub fn validate_v5_at(
    audit: &CulturalProjectionAuditV5,
    chain: &EvidenceFrontierChainV1,
    frontier_id: &EvidenceFrontierId,
    claim: &CanonicalClaimAdmissionV1,
) -> Result<(), ProjectionError> {
    // Selection validates only identifier uniqueness; strict content
    // validation remains scoped to the selected historical prefix.
    let prefix = chain.strict_prefix_through(frontier_id)?;

    let selected = prefix
        .current()
        .ok_or(ProjectionError::InvalidEvidenceFrontierManifest)?;

    if audit.evidence_frontier() != &selected.frontier_id
        || claim.evidence_frontier != selected.frontier_id
    {
        return Err(ProjectionError::InvalidEvidenceFrontierManifest);
    }

    claim.validate_frontier_safe(selected)?;
    audit.validate_frontier_safe(selected, claim)?;

    Ok(())
}

/// Boolean convenience wrapper for historical replay/admission.
pub fn is_v5_safe_at(
    audit: &CulturalProjectionAuditV5,
    chain: &EvidenceFrontierChainV1,
    frontier_id: &EvidenceFrontierId,
    claim: &CanonicalClaimAdmissionV1,
) -> bool {
    validate_v5_at(audit, chain, frontier_id, claim).is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::civilizational::{
        ArgumentationTemporalMetadataV1, EvidenceFrontierV1, EvidenceTemporalMetadataV1,
        QualificationStatus, SourceSnapshotTemporalMetadataV1, YearInterval,
    };
    use crate::cultural_argumentation::{CulturalArgumentationKindV1, CulturalArgumentationRefV3};
    use crate::cultural_projection_audit_v5::CulturalProjectionAuditV5;
    use crate::cultural_systems::{
        AccessPolicyV1, CanonicalClaimAdmissionV1, CulturalArgumentationEvidenceClosureV1,
        CulturalProjectionAuditV2, CulturalProjectionIdV1,
    };
    use crate::ontology_context::OntologyMappingRelationV1;
    use crate::ontology_mapping::{
        OntologyMappingKindV1, OntologyMappingStandardV1, OntologyMappingV2,
        OntologyReleaseStatusV1,
    };

    fn fixture() -> (
        CulturalProjectionAuditV5,
        CanonicalClaimAdmissionV1,
        EvidenceFrontierChainV1,
    ) {
        let mut root = EvidenceFrontierV1 {
            frontier_id: "frontier:1950".into(),
            known_by_year: 1950,
            parent_frontier: None,
            policy_version: "v1".into(),
            manifest_hash: String::new(),
            admitted_evidence: ["e:1"].into_iter().map(Into::into).collect(),
            admitted_sources: ["source:1"].into_iter().map(Into::into).collect(),
            evidence_metadata: vec![EvidenceTemporalMetadataV1 {
                evidence_id: "e:1".into(),
                source_snapshot: "source:1".into(),
                artifact_time: Some(YearInterval {
                    from: Some(1940),
                    to: Some(1940),
                }),
                publication_time: Some(1941),
                capture_time: None,
                available_by: 1942,
                validity_time: Some(YearInterval {
                    from: Some(1940),
                    to: Some(1950),
                }),
            }],
            source_metadata: vec![SourceSnapshotTemporalMetadataV1 {
                source_snapshot: "source:1".into(),
                publication_time: Some(1941),
                capture_time: None,
                available_by: 1942,
            }],
            argumentation_metadata: vec![ArgumentationTemporalMetadataV1 {
                assessment: "assessment:1".into(),
                interpretation: "interpretation:1".into(),
                assessment_time: Some(YearInterval {
                    from: Some(1948),
                    to: Some(1948),
                }),
                interpretation_time: Some(YearInterval {
                    from: Some(1949),
                    to: Some(1949),
                }),
                available_by: 1950,
            }],
        };
        root.recompute_manifest_hash().expect("root hash");

        let mut child = root.clone();
        child.frontier_id = "frontier:1951".into();
        child.known_by_year = 1951;
        child.parent_frontier = Some(root.frontier_id.clone());
        child.admitted_evidence.insert("e:2".into());
        child.evidence_metadata.push(EvidenceTemporalMetadataV1 {
            evidence_id: "e:2".into(),
            source_snapshot: "source:1".into(),
            artifact_time: Some(YearInterval {
                from: Some(1945),
                to: Some(1945),
            }),
            publication_time: Some(1950),
            capture_time: None,
            available_by: 1951,
            validity_time: Some(YearInterval {
                from: Some(1945),
                to: Some(1951),
            }),
        });
        child.recompute_manifest_hash().expect("child hash");

        let claim = CanonicalClaimAdmissionV1 {
            claim_ref: "claim:1".into(),
            evidence_refs: vec!["e:1".into()],
            source_snapshots: vec!["source:1".into()],
            qualification: QualificationStatus::Supported,
            evidence_frontier: root.frontier_id.clone(),
        };

        let base = CulturalProjectionAuditV2 {
            projection_id: CulturalProjectionIdV1::Transmission("transmission:1".into()),
            claim_ref: claim.claim_ref.clone(),
            evidence_refs: claim.evidence_refs.clone(),
            source_snapshots: claim.source_snapshots.clone(),
            community_recognition_evidence: vec![],
            assessment: Some("assessment:1".into()),
            argumentation: None,
            event_time: YearInterval {
                from: Some(1900),
                to: Some(1950),
            },
            qualification: claim.qualification,
            access_policy: AccessPolicyV1::Public,
            evidence_frontier: root.frontier_id.clone(),
            frontier_manifest_hash: String::new(),
            projection_semantic_hash: String::new(),
        };

        let mapping = OntologyMappingV2::from_claim(
            "mapping:1",
            OntologyMappingStandardV1::CidocCrm,
            "7.4",
            OntologyReleaseStatusV1::Draft,
            "E7_Activity",
            OntologyMappingKindV1::Class,
            &claim,
        );
        let resolution = crate::ontology_resolution::OntologyMappingResolutionV1::from_mapping(
            &mapping,
            OntologyMappingRelationV1::Exact,
            &claim,
            &root,
        )
        .expect("resolution");
        let v4 = crate::cultural_projection_audit_v4::CulturalProjectionAuditV4::from_v2(
            base,
            vec![resolution],
        )
        .expect("v4 audit");

        let argumentation = CulturalArgumentationRefV3::from_v2(
            CulturalArgumentationKindV1::InferenceMaking,
            crate::cultural_systems::CulturalArgumentationRefV2 {
                assessment: "assessment:1".into(),
                interpretation: "interpretation:1".into(),
                claim_ref: claim.claim_ref.clone(),
                closure: CulturalArgumentationEvidenceClosureV1 {
                    claim_ref: claim.claim_ref.clone(),
                    evidence_refs: vec!["e:1".into()],
                    source_snapshots: vec!["source:1".into()],
                    evidence_frontier: root.frontier_id.clone(),
                },
                assessment_time: Some(YearInterval {
                    from: Some(1948),
                    to: Some(1948),
                }),
                interpretation_time: Some(YearInterval {
                    from: Some(1949),
                    to: Some(1949),
                }),
                available_by: 1950,
            },
            &root,
        )
        .expect("argumentation");

        let audit = CulturalProjectionAuditV5::from_v4(v4, vec![argumentation]).expect("v5 audit");

        (
            audit,
            claim,
            EvidenceFrontierChainV1 {
                frontiers: vec![root, child],
            },
        )
    }

    #[test]
    fn duplicate_later_frontier_id_invalidates_historical_selection() {
        let (audit, claim, mut chain) = fixture();
        let mut duplicate = chain.frontiers[1].clone();
        duplicate.frontier_id = chain.frontiers[0].frontier_id.clone();
        duplicate.manifest_hash = "later-duplicate-unvalidated".into();
        chain.frontiers[1] = duplicate;

        assert_eq!(
            validate_v5_at(&audit, &chain, &chain.frontiers[0].frontier_id, &claim,),
            Err(ProjectionError::InvalidEvidenceFrontierManifest)
        );
        assert!(!is_v5_safe_at(
            &audit,
            &chain,
            &chain.frontiers[0].frontier_id,
            &claim,
        ));
    }

    #[test]
    fn historical_replay_preserves_claim_frontier_diagnostic() {
        let (audit, mut claim, chain) = fixture();
        let mut selected = chain.frontiers[0].clone();
        selected.manifest_hash = "not-a-valid-sha256".into();
        let chain = EvidenceFrontierChainV1 {
            frontiers: vec![selected, chain.frontiers[1].clone()],
        };

        assert_eq!(
            validate_v5_at(&audit, &chain, &"frontier:1950".into(), &claim),
            Err(ProjectionError::InvalidEvidenceFrontierManifest)
        );
        claim.evidence_frontier = "frontier:other".into();
        assert_eq!(
            validate_v5_at(
                &audit,
                &EvidenceFrontierChainV1 {
                    frontiers: chain.frontiers.clone()
                },
                &"frontier:1950".into(),
                &claim
            ),
            Err(ProjectionError::InvalidEvidenceFrontierManifest)
        );
    }
    #[test]
    fn historical_replay_preserves_audit_frontier_diagnostic() {
        let (audit, claim, mut chain) = fixture();
        chain.frontiers[0].known_by_year = 1941;
        chain.frontiers[0]
            .recompute_manifest_hash()
            .expect("late root frontier hash");

        assert_eq!(
            validate_v5_at(&audit, &chain, &"frontier:1950".into(), &claim),
            Err(ProjectionError::LaterEvidenceInFrontier)
        );
    }

    #[test]
    fn historical_replay_accepts_verified_prefix() {
        let (audit, claim, chain) = fixture();
        assert_eq!(
            validate_v5_at(&audit, &chain, &"frontier:1950".into(), &claim),
            Ok(())
        );
        assert!(is_v5_safe_at(
            &audit,
            &chain,
            &"frontier:1950".into(),
            &claim
        ));
    }

    #[test]
    fn later_frontier_does_not_contaminate_earlier_replay() {
        let (audit, claim, mut chain) = fixture();
        let later = chain.frontiers[1].clone();

        // The later frontier is intentionally malformed. Historical replay at
        // the root must still validate because only the root prefix is read.
        chain.frontiers[1].manifest_hash = "malformed-later-frontier".into();

        assert!(validate_v5_at(&audit, &chain, &"frontier:1950".into(), &claim).is_ok());
        assert_ne!(later.manifest_hash, chain.frontiers[1].manifest_hash);
    }

    #[test]
    fn child_audit_cannot_masquerade_as_parent_replay() {
        let (audit, mut claim, chain) = fixture();
        claim.evidence_frontier = "frontier:1951".into();
        let mut child_claim = claim.clone();

        let child_frontier = &chain.frontiers[1];
        child_claim.evidence_refs.push("e:2".into());

        // The audit itself remains bound to the root, so a parent replay must
        // reject the mismatched claim and a child replay must reject the root audit.
        assert!(validate_v5_at(&audit, &chain, &"frontier:1950".into(), &child_claim).is_err());
        assert!(validate_v5_at(&audit, &chain, &child_frontier.frontier_id, &child_claim).is_err());
    }

    #[test]
    fn unknown_frontier_is_rejected() {
        let (audit, claim, chain) = fixture();
        assert!(validate_v5_at(&audit, &chain, &"frontier:unknown".into(), &claim).is_err());
    }
}
