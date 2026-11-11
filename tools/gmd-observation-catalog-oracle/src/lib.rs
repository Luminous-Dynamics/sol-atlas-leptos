//! Rights-aware GMD observation-catalogue metadata qualifier.
//!
//! This crate validates source/catalogue records and explicit use-rights
//! declarations. It does not fetch data, verify a sensor calibration,
//! certify scientific representativeness, or infer station/dataset presence.

use std::collections::BTreeSet;
use std::error::Error;
use std::fmt;

/// Scope of the record. Repository-level catalogues are not station records.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecordScope {
    RepositoryCatalog,
    StationInstrument,
    DatasetProduct,
}

/// Evidence class attached to the record itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvidenceClass {
    CatalogueMetadata,
    Measured,
    PublishedDerived,
    Simulated,
    Unknown,
}

/// Intended use being requested by a downstream consumer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UsePurpose {
    MetadataDiscovery,
    DatasetDownload,
    AcademicAnalysis,
    CommercialIngestion,
    ModelTraining,
    DerivativeRedistribution,
}

/// Source-owner rights state for one precisely named purpose.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RightsStatus {
    Permitted,
    RegistrationRequired,
    AgreementRequired,
    Prohibited,
    Unknown,
}

/// Explicit right declaration. A permission needs a source evidence reference
/// and optional validity bounds. No right is inferred from another right.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RightGrant {
    pub status: RightsStatus,
    pub evidence_ref: Option<String>,
    pub valid_from_epoch_seconds: Option<i64>,
    pub valid_until_epoch_seconds: Option<i64>,
}

/// Rights are independent by purpose; download does not imply training.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AccessRights {
    pub metadata_discovery: RightGrant,
    pub dataset_download: RightGrant,
    pub academic_analysis: RightGrant,
    pub commercial_ingestion: RightGrant,
    pub model_training: RightGrant,
    pub derivative_redistribution: RightGrant,
}

/// Optional station-specific metadata. Missing fields remain missing and
/// reduce metadata completeness; they are never borrowed from nearby stations.
#[derive(Debug, Clone, PartialEq)]
pub struct StationMetadata {
    pub station_id: String,
    pub latitude_deg: Option<f64>,
    pub longitude_deg: Option<f64>,
    pub coordinate_reference: Option<String>,
    pub measured_quantity: Option<String>,
    pub native_unit: Option<String>,
    pub sensor_axis_orientation: Option<String>,
    pub sampling_interval_seconds: Option<f64>,
    pub clock_basis: Option<String>,
    pub calibration_reference: Option<String>,
}

/// One specific versioned dataset product. A missing digest is permitted as a
/// discovery record but prevents reproducibility qualification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DatasetMetadata {
    pub product_id: String,
    pub product_uri: String,
    pub processing_level: String,
    pub coverage_start_epoch_seconds: i64,
    pub coverage_end_epoch_seconds: i64,
    pub artifact_sha256: Option<String>,
}

/// Catalogue/source record. RepositoryCatalog records must not smuggle
/// station or dataset details into a network-level catalogue entry.
#[derive(Debug, Clone, PartialEq)]
pub struct CatalogueRecord {
    pub record_id: String,
    pub provider_name: String,
    pub owner_ref: String,
    pub record_scope: RecordScope,
    pub repository_uri: String,
    pub source_evidence_uri: String,
    pub verified_on_utc_date: String,
    pub source_snapshot_sha256: Option<String>,
    pub evidence_class: EvidenceClass,
    pub instrument_classes: Vec<String>,
    pub searchable_metadata_fields: Vec<String>,
    pub station: Option<StationMetadata>,
    pub dataset: Option<DatasetMetadata>,
    pub rights: AccessRights,
}

/// Machine-readable use decision. declared_right preserves the owner's
/// stated policy even when a repository/station record is not a dataset.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UseDecision {
    pub purpose: UsePurpose,
    pub disposition: UseDisposition,
    pub declared_right: RightsStatus,
}

/// Operational disposition for a requested purpose.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UseDisposition {
    Permitted,
    RegistrationRequired,
    AgreementRequired,
    Prohibited,
    Unknown,
    RequiresDatasetProduct,
    NotYetValid,
    Expired,
}

/// Metadata completeness is not a scientific-validity verdict.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MetadataCompleteness {
    NotApplicable,
    Incomplete(Vec<&'static str>),
    CompleteForExpertReview,
}

/// Dataset content reproducibility state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Reproducibility {
    ReproducibleDigestPresent,
    MissingImmutableDigest,
}

/// Fail-closed structural validation errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CatalogError {
    EmptyRequiredField,
    InvalidEvidenceDate,
    InvalidUri,
    InvalidSha256,
    DuplicateInstrumentClass,
    DuplicateMetadataField,
    ScopePayloadMismatch,
    InvalidStationIdentifier,
    InvalidCoordinate,
    InvalidCoordinateReference,
    InvalidSampleInterval,
    InvalidDatasetCoverage,
    InvalidRightsEvidence,
    InvalidRightsValidityWindow,
}

impl fmt::Display for CatalogError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let message = match self {
            Self::EmptyRequiredField => "required catalogue field is empty or untrimmed",
            Self::InvalidEvidenceDate => "verification date must use YYYY-MM-DD",
            Self::InvalidUri => "source URI must be an absolute http(s) URI or URN",
            Self::InvalidSha256 => "SHA-256 digest must be 64 lowercase hexadecimal characters",
            Self::DuplicateInstrumentClass => "instrument class entries must be unique",
            Self::DuplicateMetadataField => "searchable metadata fields must be unique",
            Self::ScopePayloadMismatch => "station/dataset payload does not match record scope",
            Self::InvalidStationIdentifier => "station identifier must be non-empty and trimmed",
            Self::InvalidCoordinate => "coordinates must be finite and within geographic bounds",
            Self::InvalidCoordinateReference => "coordinate reference must be non-empty and trimmed",
            Self::InvalidSampleInterval => "sampling interval must be finite and positive",
            Self::InvalidDatasetCoverage => "dataset coverage start must precede coverage end",
            Self::InvalidRightsEvidence => "a permitted right requires a non-empty evidence reference",
            Self::InvalidRightsValidityWindow => "right validity start must precede validity end",
        };
        f.write_str(message)
    }
}

impl Error for CatalogError {}

/// Validate structural/provenance fields without claiming scientific validity.
pub fn validate_record(record: &CatalogueRecord) -> Result<(), CatalogError> {
    for field in [
        record.record_id.as_str(),
        record.provider_name.as_str(),
        record.owner_ref.as_str(),
        record.repository_uri.as_str(),
        record.source_evidence_uri.as_str(),
    ] {
        if field.trim().is_empty() || field.trim() != field {
            return Err(CatalogError::EmptyRequiredField);
        }
    }
    if !is_supported_uri(&record.repository_uri)
        || !is_supported_uri(&record.source_evidence_uri)
    {
        return Err(CatalogError::InvalidUri);
    }
    if !valid_date(&record.verified_on_utc_date) {
        return Err(CatalogError::InvalidEvidenceDate);
    }
    if let Some(digest) = record.source_snapshot_sha256.as_deref() {
        if !valid_sha256(digest) {
            return Err(CatalogError::InvalidSha256);
        }
    }

    validate_unique_strings(&record.instrument_classes)
        .map_err(|()| CatalogError::DuplicateInstrumentClass)?;
    validate_unique_strings(&record.searchable_metadata_fields)
        .map_err(|()| CatalogError::DuplicateMetadataField)?;

    match record.record_scope {
        RecordScope::RepositoryCatalog => {
            if record.station.is_some() || record.dataset.is_some() {
                return Err(CatalogError::ScopePayloadMismatch);
            }
        }
        RecordScope::StationInstrument => {
            if record.dataset.is_some() {
                return Err(CatalogError::ScopePayloadMismatch);
            }
            let Some(station) = record.station.as_ref() else {
                return Err(CatalogError::ScopePayloadMismatch);
            };
            validate_station(station)?;
        }
        RecordScope::DatasetProduct => {
            let Some(dataset) = record.dataset.as_ref() else {
                return Err(CatalogError::ScopePayloadMismatch);
            };
            if let Some(station) = record.station.as_ref() {
                validate_station(station)?;
            }
            validate_dataset(dataset)?;
        }
    }

    for grant in [
        &record.rights.metadata_discovery,
        &record.rights.dataset_download,
        &record.rights.academic_analysis,
        &record.rights.commercial_ingestion,
        &record.rights.model_training,
        &record.rights.derivative_redistribution,
    ] {
        if grant.status == RightsStatus::Permitted
            && grant
                .evidence_ref
                .as_deref()
                .is_none_or(|value| value.trim().is_empty() || value.trim() != value)
        {
            return Err(CatalogError::InvalidRightsEvidence);
        }
        if let (Some(start), Some(end)) =
            (grant.valid_from_epoch_seconds, grant.valid_until_epoch_seconds)
        {
            if start >= end {
                return Err(CatalogError::InvalidRightsValidityWindow);
            }
        }
    }
    Ok(())
}

/// Assess one requested purpose at a caller-supplied, deterministic time.
///
/// Non-dataset records can support metadata discovery but cannot grant use of
/// an unspecified dataset. A downstream user must bind a DatasetProduct record.
pub fn assess_use(
    record: &CatalogueRecord,
    purpose: UsePurpose,
    as_of_epoch_seconds: i64,
) -> UseDecision {
    let grant = grant_for_purpose(&record.rights, purpose);
    let declared_right = grant.status;

    let disposition = if purpose != UsePurpose::MetadataDiscovery
        && record.record_scope != RecordScope::DatasetProduct
    {
        UseDisposition::RequiresDatasetProduct
    } else if grant
        .valid_from_epoch_seconds
        .is_some_and(|start| as_of_epoch_seconds < start)
    {
        UseDisposition::NotYetValid
    } else if grant
        .valid_until_epoch_seconds
        .is_some_and(|end| as_of_epoch_seconds >= end)
    {
        UseDisposition::Expired
    } else {
        match grant.status {
            RightsStatus::Permitted => UseDisposition::Permitted,
            RightsStatus::RegistrationRequired => UseDisposition::RegistrationRequired,
            RightsStatus::AgreementRequired => UseDisposition::AgreementRequired,
            RightsStatus::Prohibited => UseDisposition::Prohibited,
            RightsStatus::Unknown => UseDisposition::Unknown,
        }
    };

    UseDecision {
        purpose,
        disposition,
        declared_right,
    }
}

/// Report the metadata needed before an expert can review station suitability.
/// CompleteForExpertReview is not a calibration or physical-validity PASS.
pub fn station_metadata_completeness(record: &CatalogueRecord) -> MetadataCompleteness {
    let Some(station) = record.station.as_ref() else {
        return MetadataCompleteness::NotApplicable;
    };

    let mut missing = Vec::new();
    if station.latitude_deg.is_none() || station.longitude_deg.is_none() {
        missing.push("site_coordinates");
    }
    if station.coordinate_reference.is_none() {
        missing.push("coordinate_reference");
    }
    if station.measured_quantity.is_none() || station.native_unit.is_none() {
        missing.push("quantity_and_unit");
    }
    if station.sensor_axis_orientation.is_none() {
        missing.push("sensor_axis_orientation");
    }
    if station.sampling_interval_seconds.is_none() {
        missing.push("sampling_interval");
    }
    if station.clock_basis.is_none() {
        missing.push("clock_basis");
    }
    if station.calibration_reference.is_none() {
        missing.push("calibration_reference");
    }

    if missing.is_empty() {
        MetadataCompleteness::CompleteForExpertReview
    } else {
        MetadataCompleteness::Incomplete(missing)
    }
}

/// A DatasetProduct without an immutable artefact digest is not reproducible.
pub fn dataset_reproducibility(record: &CatalogueRecord) -> Reproducibility {
    match record.dataset.as_ref() {
        Some(dataset) if dataset.artifact_sha256.as_deref().is_some_and(valid_sha256) => {
            Reproducibility::ReproducibleDigestPresent
        }
        _ => Reproducibility::MissingImmutableDigest,
    }
}

fn validate_station(station: &StationMetadata) -> Result<(), CatalogError> {
    if station.station_id.trim().is_empty() || station.station_id.trim() != station.station_id {
        return Err(CatalogError::InvalidStationIdentifier);
    }
    match (station.latitude_deg, station.longitude_deg) {
        (Some(latitude), Some(longitude))
            if latitude.is_finite()
                && longitude.is_finite()
                && (-90.0..=90.0).contains(&latitude)
                && (-180.0..=180.0).contains(&longitude) => {}
        (None, None) | (Some(_), None) | (None, Some(_)) => {}
        _ => return Err(CatalogError::InvalidCoordinate),
    }
    if station.coordinate_reference.as_deref().is_some_and(|value| {
        value.trim().is_empty() || value.trim() != value
    }) {
        return Err(CatalogError::InvalidCoordinateReference);
    }
    if station
        .sampling_interval_seconds
        .is_some_and(|value| !value.is_finite() || value <= 0.0)
    {
        return Err(CatalogError::InvalidSampleInterval);
    }
    Ok(())
}

fn validate_dataset(dataset: &DatasetMetadata) -> Result<(), CatalogError> {
    if dataset.product_id.trim().is_empty()
        || dataset.product_id.trim() != dataset.product_id
        || dataset.processing_level.trim().is_empty()
        || dataset.processing_level.trim() != dataset.processing_level
    {
        return Err(CatalogError::EmptyRequiredField);
    }
    if !is_supported_uri(&dataset.product_uri) {
        return Err(CatalogError::InvalidUri);
    }
    if dataset.coverage_start_epoch_seconds >= dataset.coverage_end_epoch_seconds {
        return Err(CatalogError::InvalidDatasetCoverage);
    }
    if let Some(digest) = dataset.artifact_sha256.as_deref() {
        if !valid_sha256(digest) {
            return Err(CatalogError::InvalidSha256);
        }
    }
    Ok(())
}

fn grant_for_purpose(rights: &AccessRights, purpose: UsePurpose) -> &RightGrant {
    match purpose {
        UsePurpose::MetadataDiscovery => &rights.metadata_discovery,
        UsePurpose::DatasetDownload => &rights.dataset_download,
        UsePurpose::AcademicAnalysis => &rights.academic_analysis,
        UsePurpose::CommercialIngestion => &rights.commercial_ingestion,
        UsePurpose::ModelTraining => &rights.model_training,
        UsePurpose::DerivativeRedistribution => &rights.derivative_redistribution,
    }
}

fn is_supported_uri(value: &str) -> bool {
    value.starts_with("https://")
        || value.starts_with("http://")
        || value.starts_with("urn:")
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn valid_date(value: &str) -> bool {
    let bytes = value.as_bytes();
    bytes.len() == 10
        && bytes[4] == b'-'
        && bytes[7] == b'-'
        && bytes
            .iter()
            .enumerate()
            .all(|(index, byte)| index == 4 || index == 7 || byte.is_ascii_digit())
}

fn validate_unique_strings(values: &[String]) -> Result<(), ()> {
    let mut seen = BTreeSet::new();
    for value in values {
        if value.trim().is_empty() || value.trim() != value || !seen.insert(value.as_str()) {
            return Err(());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    const DIGEST: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";

    fn grant(status: RightsStatus, evidence: Option<&str>) -> RightGrant {
        RightGrant {
            status,
            evidence_ref: evidence.map(str::to_owned),
            valid_from_epoch_seconds: None,
            valid_until_epoch_seconds: None,
        }
    }

    fn rights() -> AccessRights {
        AccessRights {
            metadata_discovery: grant(RightsStatus::Permitted, Some("https://sansa.org.za/terms")),
            dataset_download: grant(RightsStatus::RegistrationRequired, None),
            academic_analysis: grant(RightsStatus::RegistrationRequired, None),
            commercial_ingestion: grant(RightsStatus::AgreementRequired, None),
            model_training: grant(RightsStatus::Unknown, None),
            derivative_redistribution: grant(RightsStatus::Unknown, None),
        }
    }

    fn sansa_repository_record() -> CatalogueRecord {
        CatalogueRecord {
            record_id: "sansa-sandims-repository-catalogue".to_owned(),
            provider_name: "South African National Space Agency".to_owned(),
            owner_ref: "SANSA Space Science".to_owned(),
            record_scope: RecordScope::RepositoryCatalog,
            repository_uri: "https://sandims.sansa.org.za/".to_owned(),
            source_evidence_uri: "https://www.sansa.org.za/scientific-research-and-global-collaboration/"
                .to_owned(),
            verified_on_utc_date: "2026-11-11".to_owned(),
            source_snapshot_sha256: None,
            evidence_class: EvidenceClass::CatalogueMetadata,
            instrument_classes: vec![
                "Geomagnetic".to_owned(),
                "Ionospheric".to_owned(),
                "Magnetospheric".to_owned(),
            ],
            searchable_metadata_fields: vec![
                "instrument_site".to_owned(),
                "instrument_type".to_owned(),
                "sampling_interval".to_owned(),
                "GPS_location".to_owned(),
                "manufacturer".to_owned(),
                "principal_investigator".to_owned(),
                "date_range".to_owned(),
                "file_type".to_owned(),
                "processing_level".to_owned(),
            ],
            station: None,
            dataset: None,
            rights: rights(),
        }
    }

    fn dataset_record() -> CatalogueRecord {
        let mut record = sansa_repository_record();
        record.record_id = "sansa-dataset-example-01".to_owned();
        record.record_scope = RecordScope::DatasetProduct;
        record.dataset = Some(DatasetMetadata {
            product_id: "synthetic-mag-vector-2024-05".to_owned(),
            product_uri: "urn:synthetic:sansa:mag-vector:2024-05".to_owned(),
            processing_level: "synthetic-level-1".to_owned(),
            coverage_start_epoch_seconds: 1_700_000_000,
            coverage_end_epoch_seconds: 1_700_086_400,
            artifact_sha256: Some(DIGEST.to_owned()),
        });
        record
    }

    fn station_record() -> CatalogueRecord {
        let mut record = sansa_repository_record();
        record.record_id = "synthetic-station-01".to_owned();
        record.record_scope = RecordScope::StationInstrument;
        record.station = Some(StationMetadata {
            station_id: "SYNTH-01".to_owned(),
            latitude_deg: Some(-34.4),
            longitude_deg: Some(19.2),
            coordinate_reference: Some("EPSG:4326".to_owned()),
            measured_quantity: Some("magnetic_field_vector".to_owned()),
            native_unit: Some("nT".to_owned()),
            sensor_axis_orientation: Some("XYZ documented".to_owned()),
            sampling_interval_seconds: Some(5.0),
            clock_basis: Some("UTC".to_owned()),
            calibration_reference: Some("urn:synthetic:calibration:01".to_owned()),
        });
        record
    }

    #[test]
    fn repository_catalogue_is_valid_without_inventing_station_records() {
        let record = sansa_repository_record();
        assert!(validate_record(&record).is_ok());
        assert!(record.station.is_none());
        assert!(record.dataset.is_none());
        assert_eq!(
            station_metadata_completeness(&record),
            MetadataCompleteness::NotApplicable
        );
    }

    #[test]
    fn repository_catalogue_only_permits_metadata_discovery() {
        let record = sansa_repository_record();
        assert_eq!(
            assess_use(&record, UsePurpose::MetadataDiscovery, 1_800_000_000).disposition,
            UseDisposition::Permitted
        );
        assert_eq!(
            assess_use(&record, UsePurpose::AcademicAnalysis, 1_800_000_000).disposition,
            UseDisposition::RequiresDatasetProduct
        );
        assert_eq!(
            assess_use(&record, UsePurpose::CommercialIngestion, 1_800_000_000).declared_right,
            RightsStatus::AgreementRequired
        );
    }

    #[test]
    fn academic_access_does_not_grant_commercial_ingestion() {
        let mut record = dataset_record();
        record.rights.academic_analysis =
            grant(RightsStatus::Permitted, Some("urn:agreement:academic-01"));
        record.rights.commercial_ingestion = grant(RightsStatus::AgreementRequired, None);

        assert_eq!(
            assess_use(&record, UsePurpose::AcademicAnalysis, 1_800_000_000).disposition,
            UseDisposition::Permitted
        );
        assert_eq!(
            assess_use(&record, UsePurpose::CommercialIngestion, 1_800_000_000).disposition,
            UseDisposition::AgreementRequired
        );
    }

    #[test]
    fn download_permission_does_not_grant_training_or_redistribution() {
        let mut record = dataset_record();
        record.rights.dataset_download =
            grant(RightsStatus::Permitted, Some("urn:permission:download"));
        record.rights.model_training = grant(RightsStatus::Unknown, None);
        record.rights.derivative_redistribution = grant(RightsStatus::Unknown, None);

        assert_eq!(
            assess_use(&record, UsePurpose::DatasetDownload, 1_800_000_000).disposition,
            UseDisposition::Permitted
        );
        assert_eq!(
            assess_use(&record, UsePurpose::ModelTraining, 1_800_000_000).disposition,
            UseDisposition::Unknown
        );
        assert_eq!(
            assess_use(&record, UsePurpose::DerivativeRedistribution, 1_800_000_000).disposition,
            UseDisposition::Unknown
        );
    }

    #[test]
    fn explicit_prohibition_is_never_overridden_by_another_permission() {
        let mut record = dataset_record();
        record.rights.dataset_download =
            grant(RightsStatus::Permitted, Some("urn:permission:download"));
        record.rights.commercial_ingestion =
            grant(RightsStatus::Prohibited, Some("urn:terms:prohibition"));

        assert_eq!(
            assess_use(&record, UsePurpose::CommercialIngestion, 1_800_000_000).disposition,
            UseDisposition::Prohibited
        );
    }

    #[test]
    fn a_permitted_right_requires_an_evidence_reference() {
        let mut record = dataset_record();
        record.rights.model_training = grant(RightsStatus::Permitted, None);
        assert_eq!(validate_record(&record), Err(CatalogError::InvalidRightsEvidence));
    }

    #[test]
    fn permission_validity_window_is_enforced_deterministically() {
        let mut record = dataset_record();
        record.rights.commercial_ingestion = RightGrant {
            status: RightsStatus::Permitted,
            evidence_ref: Some("urn:agreement:commercial-01".to_owned()),
            valid_from_epoch_seconds: Some(100),
            valid_until_epoch_seconds: Some(200),
        };
        assert_eq!(
            assess_use(&record, UsePurpose::CommercialIngestion, 99).disposition,
            UseDisposition::NotYetValid
        );
        assert_eq!(
            assess_use(&record, UsePurpose::CommercialIngestion, 200).disposition,
            UseDisposition::Expired
        );
    }

    #[test]
    fn repository_catalogue_cannot_smuggle_station_or_dataset_payloads() {
        let mut record = sansa_repository_record();
        record.station = station_record().station;
        assert_eq!(validate_record(&record), Err(CatalogError::ScopePayloadMismatch));
    }

    #[test]
    fn station_record_with_missing_scientific_metadata_is_incomplete_not_fabricated() {
        let mut record = station_record();
        let station = record.station.as_mut().unwrap();
        station.latitude_deg = None;
        station.longitude_deg = None;
        station.measured_quantity = None;
        station.native_unit = None;
        station.calibration_reference = None;

        assert!(validate_record(&record).is_ok());
        assert_eq!(
            station_metadata_completeness(&record),
            MetadataCompleteness::Incomplete(vec![
                "site_coordinates",
                "quantity_and_unit",
                "calibration_reference",
            ])
        );
    }

    #[test]
    fn complete_station_metadata_is_only_ready_for_expert_review() {
        let record = station_record();
        assert!(validate_record(&record).is_ok());
        assert_eq!(
            station_metadata_completeness(&record),
            MetadataCompleteness::CompleteForExpertReview
        );
    }

    #[test]
    fn dataset_without_digest_is_cataloguable_but_not_reproducible() {
        let mut record = dataset_record();
        record.dataset.as_mut().unwrap().artifact_sha256 = None;
        assert!(validate_record(&record).is_ok());
        assert_eq!(
            dataset_reproducibility(&record),
            Reproducibility::MissingImmutableDigest
        );
    }

    #[test]
    fn malformed_dataset_digest_is_rejected() {
        let mut record = dataset_record();
        record.dataset.as_mut().unwrap().artifact_sha256 = Some("bad-digest".to_owned());
        assert_eq!(validate_record(&record), Err(CatalogError::InvalidSha256));
    }

    #[test]
    fn station_coordinates_are_range_checked() {
        let mut record = station_record();
        record.station.as_mut().unwrap().latitude_deg = Some(-95.0);
        assert_eq!(validate_record(&record), Err(CatalogError::InvalidCoordinate));
    }

    #[test]
    fn searchable_fields_must_be_unique() {
        let mut record = sansa_repository_record();
        record.searchable_metadata_fields.push("sampling_interval".to_owned());
        assert_eq!(validate_record(&record), Err(CatalogError::DuplicateMetadataField));
    }

    #[test]
    fn source_snapshot_digest_is_validated_when_present() {
        let mut record = sansa_repository_record();
        record.source_snapshot_sha256 = Some("xyz".to_owned());
        assert_eq!(validate_record(&record), Err(CatalogError::InvalidSha256));
    }

    #[test]
    fn station_and_dataset_scope_payloads_are_distinct() {
        let mut record = station_record();
        record.dataset = Some(dataset_record().dataset.unwrap());
        assert_eq!(validate_record(&record), Err(CatalogError::ScopePayloadMismatch));
    }

    #[test]
    fn dataset_coverage_must_be_ordered() {
        let mut record = dataset_record();
        let product = record.dataset.as_mut().unwrap();
        product.coverage_start_epoch_seconds = 5;
        product.coverage_end_epoch_seconds = 5;
        assert_eq!(validate_record(&record), Err(CatalogError::InvalidDatasetCoverage));
    }
}
