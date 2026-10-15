// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Machine-checkable source acquisition, rights, and qualification state.
//!
//! A public webpage is a source-discovery lead, not proof that a dataset was
//! acquired, that bulk reuse is licensed, or that data are fit for a scenario.
//! This manifest deliberately separates acquisition, rights review, schema and
//! coverage metadata, and scenario qualification. Validation is structural;
//! it does not fetch URLs, authenticate reviewers, verify legal conclusions, or
//! recompute a digest of source bytes.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const TRANSPORT_SOURCE_MANIFEST_SCHEMA_V1: &str =
    "luminous.sol-atlas.transport-source-manifest.v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SourceProductKindV1 {
    GeospatialNetwork,
    FreightFlowDataset,
    AssetInventory,
    TrafficObservations,
    OperatorCapacityFeed,
    OperationalRailPlanningDataset,
    PolicyOrProposalDocument,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SourceAcquisitionStateV1 {
    NotAcquired,
    AccessReviewRequired,
    AcquiredUnreviewed,
    QualifiedForScenario,
    Rejected,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum SourceRightsStatusV1 {
    Unknown,
    ReviewRequired,
    PermittedInternalUse,
    PermittedRedistribution,
    Restricted,
    Denied,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SourceAcquisitionReceiptV1 {
    /// Retrieval instant, not event time or source observation time.
    pub retrieved_at_epoch_seconds: u64,
    /// Exact number of bytes hashed; compressed and expanded sizes are distinct.
    pub byte_count: u64,
    /// SHA-256 of exact acquired bytes, lowercase or uppercase hexadecimal.
    pub content_sha256: String,
    pub media_type: String,
    pub file_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransportSourceRecordV1 {
    pub id: String,
    pub publisher: String,
    pub title: String,
    /// Stable source landing page or direct dataset URI; not proof of API access.
    pub canonical_source_uri: String,
    pub product_kind: SourceProductKindV1,
    pub acquisition_state: SourceAcquisitionStateV1,
    pub rights_status: SourceRightsStatusV1,
    /// Exact URL, agreement ID, or private review record supporting the rights decision.
    pub rights_decision_ref: Option<String>,
    /// Schema/release identifier as supplied by publisher or adapter; never guessed.
    pub source_schema_profile_id: Option<String>,
    /// Required for spatial data. It may identify a CRS or a documented zone/place-ID
    /// vocabulary; coordinate order and transformations live in the adapter profile.
    pub geographic_reference_profile_id: Option<String>,
    /// Required for time-sensitive datasets; identifies the source's clock/time basis.
    pub time_profile_id: Option<String>,
    pub coverage_start_epoch_seconds: Option<u64>,
    pub coverage_end_epoch_seconds: Option<u64>,
    pub acquisition: Option<SourceAcquisitionReceiptV1>,
    /// Exact parser/transformation profile used to produce a frozen projection.
    pub parser_profile_id: Option<String>,
    /// Source mapping, quality, completeness, and qualification review reference.
    pub qualification_review_ref: Option<String>,
    pub missingness_and_limitations: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TransportSourceManifestV1 {
    pub schema_version: String,
    pub manifest_id: String,
    pub generated_at_epoch_seconds: Option<u64>,
    pub records: Vec<TransportSourceRecordV1>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SourceManifestError {
    UnsupportedSchemaVersion,
    MissingIdentityOrSourceUri,
    DuplicateOrBlankSourceId,
    InvalidDigestShape,
    InvalidAcquisitionReceipt,
    AcquisitionStateMismatch,
    RightsNotQualified,
    MissingRightsDecision,
    MissingSchemaProfile,
    MissingGeographicReferenceProfile,
    MissingTimeProfile,
    InvalidCoverageInterval,
    MissingCoverageInterval,
    MissingParserProfile,
    MissingQualificationReview,
    MissingLimitations,
}

fn nonempty(value: &str) -> bool {
    !value.trim().is_empty()
}

fn sha256_shape(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

impl TransportSourceRecordV1 {
    pub fn validate(&self) -> Result<(), SourceManifestError> {
        if !nonempty(&self.id)
            || !nonempty(&self.publisher)
            || !nonempty(&self.title)
            || !nonempty(&self.canonical_source_uri)
        {
            return Err(SourceManifestError::MissingIdentityOrSourceUri);
        }

        if let Some(receipt) = &self.acquisition {
            if receipt.byte_count == 0
                || !nonempty(&receipt.content_sha256)
                || !sha256_shape(&receipt.content_sha256)
                || !nonempty(&receipt.media_type)
                || !nonempty(&receipt.file_name)
            {
                return Err(SourceManifestError::InvalidAcquisitionReceipt);
            }
        }

        if self.coverage_start_epoch_seconds.is_some() != self.coverage_end_epoch_seconds.is_some()
            || self
                .coverage_start_epoch_seconds
                .zip(self.coverage_end_epoch_seconds)
                .is_some_and(|(start, end)| start >= end)
        {
            return Err(SourceManifestError::InvalidCoverageInterval);
        }

        match self.acquisition_state {
            SourceAcquisitionStateV1::NotAcquired => {
                if self.acquisition.is_some() {
                    return Err(SourceManifestError::AcquisitionStateMismatch);
                }
            }
            SourceAcquisitionStateV1::AccessReviewRequired => {
                if self.acquisition.is_some() && self.rights_status == SourceRightsStatusV1::Denied
                {
                    return Err(SourceManifestError::AcquisitionStateMismatch);
                }
            }
            SourceAcquisitionStateV1::AcquiredUnreviewed => {
                if self.acquisition.is_none() {
                    return Err(SourceManifestError::AcquisitionStateMismatch);
                }
            }
            SourceAcquisitionStateV1::QualifiedForScenario => {
                if self.acquisition.is_none() {
                    return Err(SourceManifestError::AcquisitionStateMismatch);
                }
                if !matches!(
                    self.rights_status,
                    SourceRightsStatusV1::PermittedInternalUse
                        | SourceRightsStatusV1::PermittedRedistribution
                ) {
                    return Err(SourceManifestError::RightsNotQualified);
                }
                if !self.rights_decision_ref.as_deref().is_some_and(nonempty) {
                    return Err(SourceManifestError::MissingRightsDecision);
                }
                if !self
                    .source_schema_profile_id
                    .as_deref()
                    .is_some_and(nonempty)
                {
                    return Err(SourceManifestError::MissingSchemaProfile);
                }
                if self.product_kind != SourceProductKindV1::PolicyOrProposalDocument
                    && !self
                        .geographic_reference_profile_id
                        .as_deref()
                        .is_some_and(nonempty)
                {
                    return Err(SourceManifestError::MissingGeographicReferenceProfile);
                }
                if self.product_kind != SourceProductKindV1::PolicyOrProposalDocument
                    && !self.time_profile_id.as_deref().is_some_and(nonempty)
                {
                    return Err(SourceManifestError::MissingTimeProfile);
                }
                if self.product_kind != SourceProductKindV1::PolicyOrProposalDocument
                    && self.coverage_start_epoch_seconds.is_none()
                {
                    return Err(SourceManifestError::MissingCoverageInterval);
                }
                if !self.parser_profile_id.as_deref().is_some_and(nonempty) {
                    return Err(SourceManifestError::MissingParserProfile);
                }
                if !self
                    .qualification_review_ref
                    .as_deref()
                    .is_some_and(nonempty)
                {
                    return Err(SourceManifestError::MissingQualificationReview);
                }
                if self.missingness_and_limitations.is_empty()
                    || self
                        .missingness_and_limitations
                        .iter()
                        .any(|s| !nonempty(s))
                {
                    return Err(SourceManifestError::MissingLimitations);
                }
            }
            SourceAcquisitionStateV1::Rejected => {
                // Rejected records may include the exact bytes that were inspected,
                // or remain pre-acquisition when rights/access could not be resolved.
            }
        }
        Ok(())
    }
}

impl TransportSourceManifestV1 {
    pub fn validate(&self) -> Result<(), SourceManifestError> {
        if self.schema_version != TRANSPORT_SOURCE_MANIFEST_SCHEMA_V1 {
            return Err(SourceManifestError::UnsupportedSchemaVersion);
        }
        if !nonempty(&self.manifest_id) {
            return Err(SourceManifestError::MissingIdentityOrSourceUri);
        }
        let mut ids = BTreeSet::new();
        for record in &self.records {
            if !ids.insert(record.id.as_str()) {
                return Err(SourceManifestError::DuplicateOrBlankSourceId);
            }
            record.validate()?;
        }
        Ok(())
    }

    /// Sort records by ID for repeatable serialization; this is not a
    /// cross-implementation canonical-JSON or cryptographic hashing contract.
    pub fn normalized(mut self) -> Self {
        self.records.sort_by(|a, b| a.id.cmp(&b.id));
        for record in &mut self.records {
            record.missingness_and_limitations.sort();
        }
        self
    }
}

/// Current source-discovery register. These three entries explicitly have no
/// acquired data payload: the National Freight Databank's portal, the draft
/// National Rail Master Plan consultation, and the Western Cape proposal are
/// leads for access review and scenario definition, not route-capacity snapshots.
pub fn south_african_freight_source_readiness_v1() -> TransportSourceManifestV1 {
    use SourceAcquisitionStateV1::*;
    use SourceProductKindV1::*;
    use SourceRightsStatusV1::*;

    let manifest = TransportSourceManifestV1 {
        schema_version: TRANSPORT_SOURCE_MANIFEST_SCHEMA_V1.to_owned(),
        manifest_id: "za-freight-source-readiness-v1".to_owned(),
        generated_at_epoch_seconds: None,
        records: vec![
            TransportSourceRecordV1 {
                id: "za-national-freight-databank".to_owned(),
                publisher: "South African Department of Transport".to_owned(),
                title: "National Freight Databank landing page".to_owned(),
                canonical_source_uri: "https://freight.transport.gov.za/".to_owned(),
                product_kind: FreightFlowDataset,
                acquisition_state: AccessReviewRequired,
                rights_status: ReviewRequired,
                rights_decision_ref: None,
                source_schema_profile_id: None,
                geographic_reference_profile_id: None,
                time_profile_id: None,
                coverage_start_epoch_seconds: None,
                coverage_end_epoch_seconds: None,
                acquisition: None,
                parser_profile_id: None,
                qualification_review_ref: None,
                missingness_and_limitations: vec![
                    "No dataset bytes acquired by this manifest.".to_owned(),
                    "Portal discovery does not prove a supported API or bulk-download permission."
                        .to_owned(),
                    "Freight-flow and national-throughput aggregates are not usable route capacity."
                        .to_owned(),
                ],
            },
            TransportSourceRecordV1 {
                id: "za-draft-national-rail-master-plan".to_owned(),
                publisher: "South African Department of Transport".to_owned(),
                title: "Draft National Rail Master Plan consultation".to_owned(),
                canonical_source_uri: concat!(
                    "https://www.gov.za/documents/notices/department-transport-",
                    "public-consultation-draft-national-rail-master-plan-comments"
                )
                .to_owned(),
                product_kind: PolicyOrProposalDocument,
                acquisition_state: NotAcquired,
                rights_status: ReviewRequired,
                rights_decision_ref: None,
                source_schema_profile_id: None,
                geographic_reference_profile_id: None,
                time_profile_id: None,
                coverage_start_epoch_seconds: None,
                coverage_end_epoch_seconds: None,
                acquisition: None,
                parser_profile_id: None,
                qualification_review_ref: None,
                missingness_and_limitations: vec![
                    "Public consultation page is a research lead, not a data feed.".to_owned(),
                    "Draft consultation only; verify adoption before treating it as policy."
                        .to_owned(),
                ],
            },
            TransportSourceRecordV1 {
                id: "za-western-cape-intermodal-proposal".to_owned(),
                publisher: "Western Cape Government".to_owned(),
                title: "Proposed intermodal freight terminal network".to_owned(),
                canonical_source_uri: concat!(
                    "https://www.westerncape.gov.za/mobility/",
                    "intermodalism-building-more-connected-freight-network"
                )
                .to_owned(),
                product_kind: PolicyOrProposalDocument,
                acquisition_state: NotAcquired,
                rights_status: ReviewRequired,
                rights_decision_ref: None,
                source_schema_profile_id: None,
                geographic_reference_profile_id: None,
                time_profile_id: None,
                coverage_start_epoch_seconds: None,
                coverage_end_epoch_seconds: None,
                acquisition: None,
                parser_profile_id: None,
                qualification_review_ref: None,
                missingness_and_limitations: vec![
                    "Sites are proposed; confirmation and land assessments remain unverified."
                        .to_owned(),
                    "The modelled 10.4 Mt is not guaranteed modal shift or verified capacity."
                        .to_owned(),
                ],
            },
            TransportSourceRecordV1 {
                id: "za-transnet-network-register-2026-06".to_owned(),
                publisher: "Transnet Rail Infrastructure Manager".to_owned(),
                title: "Annexure 33 — Network Register (June 2026_v0.3)".to_owned(),
                canonical_source_uri: "https://www.transnet.net/getFile.ashx?id=21073".to_owned(),
                product_kind: OperationalRailPlanningDataset,
                acquisition_state: AccessReviewRequired,
                rights_status: ReviewRequired,
                rights_decision_ref: None,
                source_schema_profile_id: None,
                geographic_reference_profile_id: None,
                time_profile_id: None,
                coverage_start_epoch_seconds: None,
                coverage_end_epoch_seconds: None,
                acquisition: None,
                parser_profile_id: None,
                qualification_review_ref: None,
                missingness_and_limitations: vec![
                    "Linked from the current Network Statement v4 page; the register is dated 26 June 2026 and states that it is a guide for 2026/27 capacity applications."
                        .to_owned(),
                    "Exact bytes and SHA-256 have not been frozen by this project; reuse and redistribution rights remain unreviewed."
                        .to_owned(),
                    "The displayed route list repeats the Musina–Pyramid South/Hoedspruit label for items 9 and 10; verify source workbook details before asserting ten distinct routes."
                        .to_owned(),
                    "The register distinguishes theoretical, practical, lost, operationally available, allocated and spare capacity; these values must not be collapsed or treated as live availability."
                        .to_owned(),
                ],
            },
            TransportSourceRecordV1 {
                id: "za-transnet-point-to-point-running-times".to_owned(),
                publisher: "Transnet Rail Infrastructure Manager".to_owned(),
                title: "Annexure 33a — Point to Point Running Times".to_owned(),
                canonical_source_uri: "https://www.transnet.net/getFile.ashx?id=22633".to_owned(),
                product_kind: OperationalRailPlanningDataset,
                acquisition_state: AccessReviewRequired,
                rights_status: ReviewRequired,
                rights_decision_ref: None,
                source_schema_profile_id: None,
                geographic_reference_profile_id: None,
                time_profile_id: None,
                coverage_start_epoch_seconds: None,
                coverage_end_epoch_seconds: None,
                acquisition: None,
                parser_profile_id: None,
                qualification_review_ref: None,
                missingness_and_limitations: vec![
                    "Linked as an XLSX workbook from the current Network Statement v4 page; exact workbook bytes and schema have not been inspected or frozen."
                        .to_owned(),
                    "The publisher describes station-to-station running times, distances and scheduled running frequencies; running times are conditional, not service guarantees."
                        .to_owned(),
                    "Access and redistribution terms have not been reviewed."
                        .to_owned(),
                ],
            },
            TransportSourceRecordV1 {
                id: "za-transnet-capacity-statement-2026-27".to_owned(),
                publisher: "Transnet Rail Infrastructure Manager".to_owned(),
                title: "Annexure 33b — Capacity Statement".to_owned(),
                canonical_source_uri: "https://www.transnet.net/getFile.ashx?id=21146".to_owned(),
                product_kind: OperationalRailPlanningDataset,
                acquisition_state: AccessReviewRequired,
                rights_status: ReviewRequired,
                rights_decision_ref: None,
                source_schema_profile_id: None,
                geographic_reference_profile_id: None,
                time_profile_id: None,
                coverage_start_epoch_seconds: None,
                coverage_end_epoch_seconds: None,
                acquisition: None,
                parser_profile_id: None,
                qualification_review_ref: None,
                missingness_and_limitations: vec![
                    "Linked as an XLSM workbook from the current Network Statement v4 page; exact workbook bytes and schema have not been inspected or frozen."
                        .to_owned(),
                    "The linked Network Register describes section-level capacity on a daily and weekly basis across named routes; field meanings and units must be verified from the workbook."
                        .to_owned(),
                    "This published statement is not a live train-path reservation or current spare-capacity guarantee."
                        .to_owned(),
                    "Access and redistribution terms have not been reviewed."
                        .to_owned(),
                ],
            },
        ],
    };
    // Static fixture was hand-authored, so sort during construction as an
    // additional guarantee for byte-comparable source-register snapshots.
    manifest.normalized()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn national_freight_databank_record() -> TransportSourceRecordV1 {
        south_african_freight_source_readiness_v1()
            .records
            .into_iter()
            .find(|record| record.id == "za-national-freight-databank")
            .expect("National Freight Databank discovery record")
    }

    fn transnet_capacity_statement_record() -> TransportSourceRecordV1 {
        south_african_freight_source_readiness_v1()
            .records
            .into_iter()
            .find(|record| record.id == "za-transnet-capacity-statement-2026-27")
            .expect("Transnet capacity-statement discovery record")
    }

    #[test]
    fn south_african_source_register_is_valid_and_does_not_claim_acquisition() {
        let manifest = south_african_freight_source_readiness_v1();
        assert_eq!(manifest.validate(), Ok(()));
        assert_eq!(manifest.records.len(), 6);
        assert!(manifest.records.iter().all(|record| {
            record.acquisition.is_none()
                && record.acquisition_state != SourceAcquisitionStateV1::QualifiedForScenario
        }));
    }

    #[test]
    fn transnet_sources_are_discovered_but_not_misrepresented_as_acquired() {
        let manifest = south_african_freight_source_readiness_v1();
        let transnet = manifest
            .records
            .iter()
            .filter(|record| record.id.starts_with("za-transnet-"))
            .collect::<Vec<_>>();
        assert_eq!(transnet.len(), 3);
        assert!(transnet.iter().all(|record| {
            record.acquisition_state == SourceAcquisitionStateV1::AccessReviewRequired
                && record.rights_status == SourceRightsStatusV1::ReviewRequired
                && record.acquisition.is_none()
                && record.rights_decision_ref.is_none()
                && record.source_schema_profile_id.is_none()
        }));
        assert!(transnet
            .iter()
            .any(|record| record.id == "za-transnet-network-register-2026-06"));
        assert!(transnet
            .iter()
            .any(|record| record.id == "za-transnet-point-to-point-running-times"));
        assert!(transnet
            .iter()
            .any(|record| record.id == "za-transnet-capacity-statement-2026-27"));
    }

    #[test]
    fn source_register_is_deterministically_sorted() {
        let mut manifest = south_african_freight_source_readiness_v1();
        manifest.records.reverse();
        assert_eq!(
            manifest.normalized(),
            south_african_freight_source_readiness_v1()
        );
    }

    #[test]
    fn qualified_dataset_requires_acquisition_and_rights_evidence() {
        let mut record = national_freight_databank_record();
        record.acquisition_state = SourceAcquisitionStateV1::QualifiedForScenario;
        record.rights_status = SourceRightsStatusV1::PermittedInternalUse;
        assert_eq!(
            record.validate(),
            Err(SourceManifestError::AcquisitionStateMismatch)
        );
    }

    #[test]
    fn malformed_acquisition_hash_is_rejected() {
        let mut record = national_freight_databank_record();
        record.acquisition_state = SourceAcquisitionStateV1::AcquiredUnreviewed;
        record.acquisition = Some(SourceAcquisitionReceiptV1 {
            retrieved_at_epoch_seconds: 100,
            byte_count: 5,
            content_sha256: "not-a-digest".to_owned(),
            media_type: "application/json".to_owned(),
            file_name: "source.json".to_owned(),
        });
        assert_eq!(
            record.validate(),
            Err(SourceManifestError::InvalidAcquisitionReceipt)
        );
    }

    #[test]
    fn qualified_dataset_needs_geography_time_coverage_and_review() {
        let mut record = transnet_capacity_statement_record();
        assert_eq!(
            record.product_kind,
            SourceProductKindV1::OperationalRailPlanningDataset
        );
        record.acquisition_state = SourceAcquisitionStateV1::QualifiedForScenario;
        record.rights_status = SourceRightsStatusV1::PermittedInternalUse;
        record.rights_decision_ref = Some("review://rights/approved".to_owned());
        record.source_schema_profile_id = Some("freight-v1".to_owned());
        record.acquisition = Some(SourceAcquisitionReceiptV1 {
            retrieved_at_epoch_seconds: 100,
            byte_count: 64,
            content_sha256: "a".repeat(64),
            media_type: "application/json".to_owned(),
            file_name: "freight.json".to_owned(),
        });
        record.parser_profile_id = Some("parser-v1".to_owned());
        record.qualification_review_ref = Some("review://quality/approved".to_owned());
        record.missingness_and_limitations = vec!["Synthetic test limitations".to_owned()];
        assert_eq!(
            record.validate(),
            Err(SourceManifestError::MissingGeographicReferenceProfile)
        );

        record.geographic_reference_profile_id = Some("SA-zones-v1".to_owned());
        assert_eq!(
            record.validate(),
            Err(SourceManifestError::MissingTimeProfile)
        );
        record.time_profile_id = Some("UTC_UNIX_SECONDS_V1".to_owned());
        assert_eq!(
            record.validate(),
            Err(SourceManifestError::MissingCoverageInterval)
        );
        record.coverage_start_epoch_seconds = Some(10);
        record.coverage_end_epoch_seconds = Some(10);
        assert_eq!(
            record.validate(),
            Err(SourceManifestError::InvalidCoverageInterval)
        );
        record.coverage_end_epoch_seconds = Some(20);
        assert_eq!(record.validate(), Ok(()));
    }

    #[test]
    fn duplicate_source_ids_are_rejected() {
        let mut manifest = south_african_freight_source_readiness_v1();
        manifest.records.push(manifest.records[0].clone());
        assert_eq!(
            manifest.validate(),
            Err(SourceManifestError::DuplicateOrBlankSourceId)
        );
    }
}
