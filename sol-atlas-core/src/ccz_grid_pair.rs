// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Strict metadata and cell-semantics checks for paired bathymetry/TID grids.
//!
//! This is not a NetCDF parser, a navigation product, or an ecological classifier.
//! It makes pairing assumptions explicit before any parser or downstream model
//! can treat the grid as contextual research evidence.

use serde::{Deserialize, Serialize};

use crate::ccz_commons::valid_sha256;

/// Horizontal axis represented by a grid dimension.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GridAxis {
    Latitude,
    Longitude,
    X,
    Y,
    Unknown,
}

/// Grid registration convention stated by the source artifact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GridRegistration {
    PixelCenter,
    PixelArea,
    Unknown,
}

/// Explicit representation of missing cells in a source grid.
///
/// `ExplicitNoSentinel` is valid only when the artifact specification explicitly
/// states that no numeric sentinel is used. It must not be used just because the
/// parser has not discovered a NoData value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum NoDataEncoding {
    I16Sentinel(i16),
    U8Sentinel(u8),
    ExplicitNoSentinel,
    Unknown,
}

impl NoDataEncoding {
    fn is_resolved(self) -> bool {
        !matches!(self, Self::Unknown)
    }
}

/// Geographic grid bounds as outer cell edges, encoded exactly in microarcseconds.
///
/// Integer angular units avoid fuzzy floating-point equality when verifying that
/// two raster products share the same grid. This profile is geographic, not a
/// general-purpose projected-raster geometry type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GridBounds {
    pub west_edge_microarcseconds: i64,
    pub east_edge_microarcseconds: i64,
    pub south_edge_microarcseconds: i64,
    pub north_edge_microarcseconds: i64,
}

/// Metadata for one exact grid artifact. The snapshot ID points into the existing
/// CCZ source-snapshot/provenance path; this module does not replace that graph.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GridMetadata {
    pub source_snapshot_id: String,
    /// SHA-256 of the exact artifact bytes; byte verification is a separate step.
    pub artifact_sha256: String,
    pub product_id: String,
    pub product_version: String,
    /// Shared logical grid identity, not the individual file identity.
    pub grid_id: String,
    pub rows: u64,
    pub columns: u64,
    pub bounds: GridBounds,
    pub cell_width_microarcseconds: u64,
    pub cell_height_microarcseconds: u64,
    pub coordinate_reference_system: Option<String>,
    pub horizontal_unit: Option<String>,
    pub row_axis: GridAxis,
    pub column_axis: GridAxis,
    pub registration: GridRegistration,
    pub no_data: NoDataEncoding,
    /// Required before downstream bathymetry-derived scientific claims.
    pub vertical_datum: Option<String>,
    pub vertical_unit: Option<String>,
}

/// Pair of bathymetry values and their matching GEBCO-style Type Identifier values.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BathymetryTidPair {
    pub pair_id: String,
    pub bathymetry: GridMetadata,
    pub type_identifier: GridMetadata,
}

/// Stable validation codes suitable for machine-readable receipts and regression tests.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GridPairIssueCode {
    MissingPairId,
    MissingSnapshotId,
    InvalidArtifactSha256,
    MissingProductId,
    MissingProductVersion,
    MissingGridId,
    InvalidDimensions,
    InvalidBounds,
    InvalidCellSpacing,
    DimensionsDoNotMatchBounds,
    MissingCoordinateReferenceSystem,
    MissingHorizontalUnit,
    UnknownAxisOrder,
    DuplicateAxisOrder,
    UnknownRegistration,
    UnknownNoDataEncoding,
    MissingVerticalDatum,
    MissingVerticalUnit,
    ProductMismatch,
    ProductVersionMismatch,
    GridIdentityMismatch,
    DimensionMismatch,
    BoundsMismatch,
    CellSpacingMismatch,
    CoordinateReferenceMismatch,
    HorizontalUnitMismatch,
    AxisOrderMismatch,
    RegistrationMismatch,
}

/// One deterministic validation finding. Messages explain the failed contract,
/// not whether a location is environmentally safe.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GridPairIssue {
    pub code: GridPairIssueCode,
    pub message: String,
}

/// Deterministic metadata-only pairing report. A valid report is not proof of
/// parser correctness, source accuracy, legal rights, ecological safety, or
/// navigational suitability.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GridPairValidationReport {
    pub pair_id: String,
    pub issues: Vec<GridPairIssue>,
}

impl GridPairValidationReport {
    pub fn is_valid(&self) -> bool {
        self.issues.is_empty()
    }
}

fn add_issue(issues: &mut Vec<GridPairIssue>, code: GridPairIssueCode, message: &'static str) {
    issues.push(GridPairIssue {
        code,
        message: message.to_owned(),
    });
}

fn validate_one_grid(grid: &GridMetadata, is_bathymetry: bool, issues: &mut Vec<GridPairIssue>) {
    if grid.source_snapshot_id.trim().is_empty() {
        add_issue(
            issues,
            GridPairIssueCode::MissingSnapshotId,
            "grid must reference an exact source snapshot",
        );
    }
    if !valid_sha256(&grid.artifact_sha256) {
        add_issue(
            issues,
            GridPairIssueCode::InvalidArtifactSha256,
            "artifact_sha256 must be 64 hexadecimal characters; exact bytes still require verification",
        );
    }
    if grid.product_id.trim().is_empty() {
        add_issue(
            issues,
            GridPairIssueCode::MissingProductId,
            "product_id must be declared",
        );
    }
    if grid.product_version.trim().is_empty() {
        add_issue(
            issues,
            GridPairIssueCode::MissingProductVersion,
            "product_version must be declared",
        );
    }
    if grid.grid_id.trim().is_empty() {
        add_issue(
            issues,
            GridPairIssueCode::MissingGridId,
            "grid_id must be declared independently from the file name",
        );
    }
    if grid.rows == 0 || grid.columns == 0 {
        add_issue(
            issues,
            GridPairIssueCode::InvalidDimensions,
            "row and column counts must both be positive",
        );
    }
    if grid.bounds.west_edge_microarcseconds >= grid.bounds.east_edge_microarcseconds
        || grid.bounds.south_edge_microarcseconds >= grid.bounds.north_edge_microarcseconds
    {
        add_issue(
            issues,
            GridPairIssueCode::InvalidBounds,
            "grid bounds must have positive width and height",
        );
    }
    if grid.cell_width_microarcseconds == 0 || grid.cell_height_microarcseconds == 0 {
        add_issue(
            issues,
            GridPairIssueCode::InvalidCellSpacing,
            "cell width and height must both be positive",
        );
    } else if grid.rows > 0 && grid.columns > 0 {
        let width = i64::try_from(grid.cell_width_microarcseconds)
            .ok()
            .and_then(|spacing| spacing.checked_mul(i64::try_from(grid.columns).ok()?));
        let height = i64::try_from(grid.cell_height_microarcseconds)
            .ok()
            .and_then(|spacing| spacing.checked_mul(i64::try_from(grid.rows).ok()?));
        let expected_width = grid
            .bounds
            .east_edge_microarcseconds
            .checked_sub(grid.bounds.west_edge_microarcseconds);
        let expected_height = grid
            .bounds
            .north_edge_microarcseconds
            .checked_sub(grid.bounds.south_edge_microarcseconds);
        if width != expected_width || height != expected_height {
            add_issue(
                issues,
                GridPairIssueCode::DimensionsDoNotMatchBounds,
                "dimensions multiplied by cell spacing must equal the declared outer-edge extent",
            );
        }
    }
    if grid
        .coordinate_reference_system
        .as_deref()
        .is_none_or(|value| value.trim().is_empty())
    {
        add_issue(
            issues,
            GridPairIssueCode::MissingCoordinateReferenceSystem,
            "coordinate reference system is required",
        );
    }
    if grid
        .horizontal_unit
        .as_deref()
        .is_none_or(|value| value.trim().is_empty())
    {
        add_issue(
            issues,
            GridPairIssueCode::MissingHorizontalUnit,
            "horizontal coordinate units must be explicit",
        );
    }
    if grid.row_axis == GridAxis::Unknown || grid.column_axis == GridAxis::Unknown {
        add_issue(
            issues,
            GridPairIssueCode::UnknownAxisOrder,
            "row and column axes must be explicitly identified",
        );
    }
    if grid.row_axis == grid.column_axis && grid.row_axis != GridAxis::Unknown {
        add_issue(
            issues,
            GridPairIssueCode::DuplicateAxisOrder,
            "row and column axes cannot name the same axis",
        );
    }
    if grid.registration == GridRegistration::Unknown {
        add_issue(
            issues,
            GridPairIssueCode::UnknownRegistration,
            "pixel-center versus pixel-area registration must be explicit",
        );
    }
    if !grid.no_data.is_resolved() {
        add_issue(
            issues,
            GridPairIssueCode::UnknownNoDataEncoding,
            "NoData encoding is unknown; cells must not be interpreted as measured values",
        );
    }
    if is_bathymetry {
        if grid
            .vertical_datum
            .as_deref()
            .is_none_or(|value| value.trim().is_empty())
        {
            add_issue(
                issues,
                GridPairIssueCode::MissingVerticalDatum,
                "bathymetry vertical datum must be explicit before scientific interpretation",
            );
        }
        if grid
            .vertical_unit
            .as_deref()
            .is_none_or(|value| value.trim().is_empty())
        {
            add_issue(
                issues,
                GridPairIssueCode::MissingVerticalUnit,
                "bathymetry vertical units must be explicit before scientific interpretation",
            );
        }
    }
}

/// Validate that two raster metadata records describe the same grid, while
/// preserving separate artifact identity, hashes, source snapshots and NoData rules.
pub fn validate_grid_pair(pair: &BathymetryTidPair) -> GridPairValidationReport {
    let mut issues = Vec::new();
    if pair.pair_id.trim().is_empty() {
        add_issue(
            &mut issues,
            GridPairIssueCode::MissingPairId,
            "pair_id must be stable and non-empty",
        );
    }

    validate_one_grid(&pair.bathymetry, true, &mut issues);
    validate_one_grid(&pair.type_identifier, false, &mut issues);

    let bathy = &pair.bathymetry;
    let tid = &pair.type_identifier;
    if bathy.product_id != tid.product_id {
        add_issue(
            &mut issues,
            GridPairIssueCode::ProductMismatch,
            "bathymetry and TID must belong to the same declared product family",
        );
    }
    if bathy.product_version != tid.product_version {
        add_issue(
            &mut issues,
            GridPairIssueCode::ProductVersionMismatch,
            "bathymetry and TID product versions must match exactly",
        );
    }
    if bathy.grid_id != tid.grid_id {
        add_issue(
            &mut issues,
            GridPairIssueCode::GridIdentityMismatch,
            "bathymetry and TID grid identities must match exactly",
        );
    }
    if bathy.rows != tid.rows || bathy.columns != tid.columns {
        add_issue(
            &mut issues,
            GridPairIssueCode::DimensionMismatch,
            "bathymetry and TID row/column counts must match exactly",
        );
    }
    if bathy.bounds != tid.bounds {
        add_issue(
            &mut issues,
            GridPairIssueCode::BoundsMismatch,
            "bathymetry and TID outer-edge bounds must match exactly",
        );
    }
    if bathy.cell_width_microarcseconds != tid.cell_width_microarcseconds
        || bathy.cell_height_microarcseconds != tid.cell_height_microarcseconds
    {
        add_issue(
            &mut issues,
            GridPairIssueCode::CellSpacingMismatch,
            "bathymetry and TID cell spacing must match exactly",
        );
    }
    if bathy.coordinate_reference_system != tid.coordinate_reference_system {
        add_issue(
            &mut issues,
            GridPairIssueCode::CoordinateReferenceMismatch,
            "bathymetry and TID coordinate reference systems must match exactly",
        );
    }
    if bathy.horizontal_unit != tid.horizontal_unit {
        add_issue(
            &mut issues,
            GridPairIssueCode::HorizontalUnitMismatch,
            "bathymetry and TID horizontal coordinate units must match exactly",
        );
    }
    if bathy.row_axis != tid.row_axis || bathy.column_axis != tid.column_axis {
        add_issue(
            &mut issues,
            GridPairIssueCode::AxisOrderMismatch,
            "bathymetry and TID row/column axis order must match exactly",
        );
    }
    if bathy.registration != tid.registration {
        add_issue(
            &mut issues,
            GridPairIssueCode::RegistrationMismatch,
            "bathymetry and TID registration conventions must match exactly",
        );
    }

    GridPairValidationReport {
        pair_id: pair.pair_id.clone(),
        issues,
    }
}

/// Decoded bathymetry cell. NoData remains a separate variant, never numeric zero.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BathymetryCell {
    Value(i16),
    NoData { raw_value: i16 },
}

/// Decoded TID cell. Unrecognized codes retain their numeric value for audit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TidCell {
    NoData { raw_value: u8 },
    SourceClass { raw_code: u8, class: TidSourceClass },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TidSourceClass {
    Land,
    DirectSingleBeam,
    DirectMultibeam,
    DirectSeismic,
    IsolatedSounding,
    ElectronicNavigationChartSounding,
    DirectLidar,
    DirectOptical,
    CombinedDirectMeasurements,
    SatelliteGravityPredicted,
    AlgorithmInterpolated,
    ChartContour,
    ElectronicNavigationChartContour,
    SoundingConstrainedInterpolation,
    FlightGravityPredicted,
    IcebergDraftEstimated,
    PreGeneratedMixedSources,
    UnknownSource,
    SteeringPoint,
    UnrecognizedCode(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellDecodeError {
    UnknownNoDataSemantics,
    NoDataEncodingTypeMismatch,
}

/// Interpret a raw signed-16-bit depth value using explicitly declared NoData semantics.
pub fn decode_bathymetry_cell(
    raw_value: i16,
    encoding: NoDataEncoding,
) -> Result<BathymetryCell, CellDecodeError> {
    match encoding {
        NoDataEncoding::I16Sentinel(sentinel) if raw_value == sentinel => {
            Ok(BathymetryCell::NoData { raw_value })
        }
        NoDataEncoding::I16Sentinel(_) | NoDataEncoding::ExplicitNoSentinel => {
            Ok(BathymetryCell::Value(raw_value))
        }
        NoDataEncoding::Unknown => Err(CellDecodeError::UnknownNoDataSemantics),
        NoDataEncoding::U8Sentinel(_) => Err(CellDecodeError::NoDataEncodingTypeMismatch),
    }
}

/// Interpret a raw TID byte while preserving NoData, known source classes, and unknown codes.
pub fn decode_tid_cell(raw_code: u8, encoding: NoDataEncoding) -> Result<TidCell, CellDecodeError> {
    let class = match encoding {
        NoDataEncoding::U8Sentinel(sentinel) if raw_code == sentinel => {
            return Ok(TidCell::NoData {
                raw_value: raw_code,
            });
        }
        NoDataEncoding::U8Sentinel(_) | NoDataEncoding::ExplicitNoSentinel => match raw_code {
            0 => TidSourceClass::Land,
            10 => TidSourceClass::DirectSingleBeam,
            11 => TidSourceClass::DirectMultibeam,
            12 => TidSourceClass::DirectSeismic,
            13 => TidSourceClass::IsolatedSounding,
            14 => TidSourceClass::ElectronicNavigationChartSounding,
            15 => TidSourceClass::DirectLidar,
            16 => TidSourceClass::DirectOptical,
            17 => TidSourceClass::CombinedDirectMeasurements,
            40 => TidSourceClass::SatelliteGravityPredicted,
            41 => TidSourceClass::AlgorithmInterpolated,
            42 => TidSourceClass::ChartContour,
            43 => TidSourceClass::ElectronicNavigationChartContour,
            44 => TidSourceClass::SoundingConstrainedInterpolation,
            45 => TidSourceClass::FlightGravityPredicted,
            46 => TidSourceClass::IcebergDraftEstimated,
            70 => TidSourceClass::PreGeneratedMixedSources,
            71 => TidSourceClass::UnknownSource,
            72 => TidSourceClass::SteeringPoint,
            value => TidSourceClass::UnrecognizedCode(value),
        },
        NoDataEncoding::Unknown => return Err(CellDecodeError::UnknownNoDataSemantics),
        NoDataEncoding::I16Sentinel(_) => {
            return Err(CellDecodeError::NoDataEncodingTypeMismatch);
        }
    };

    Ok(TidCell::SourceClass { raw_code, class })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn metadata() -> GridMetadata {
        let bounds = GridBounds {
            west_edge_microarcseconds: 0,
            east_edge_microarcseconds: 30_000_000,
            south_edge_microarcseconds: 0,
            north_edge_microarcseconds: 30_000_000,
        };
        GridMetadata {
            source_snapshot_id: "synthetic-snapshot-v1".into(),
            artifact_sha256: "a".repeat(64),
            product_id: "synthetic-gebco-family".into(),
            product_version: "synthetic-v1".into(),
            grid_id: "synthetic-grid-01".into(),
            rows: 2,
            columns: 2,
            bounds,
            cell_width_microarcseconds: 15_000_000,
            cell_height_microarcseconds: 15_000_000,
            coordinate_reference_system: Some("EPSG:4326".into()),
            horizontal_unit: Some("degrees".into()),
            row_axis: GridAxis::Latitude,
            column_axis: GridAxis::Longitude,
            registration: GridRegistration::PixelCenter,
            no_data: NoDataEncoding::I16Sentinel(i16::MIN),
            vertical_datum: Some("synthetic vertical datum".into()),
            vertical_unit: Some("metres".into()),
        }
    }

    fn valid_pair() -> BathymetryTidPair {
        let bathymetry = metadata();
        let mut type_identifier = metadata();
        type_identifier.source_snapshot_id = "synthetic-tid-snapshot-v1".into();
        type_identifier.artifact_sha256 = "b".repeat(64);
        type_identifier.no_data = NoDataEncoding::U8Sentinel(u8::MAX);
        type_identifier.vertical_datum = None;
        type_identifier.vertical_unit = None;
        BathymetryTidPair {
            pair_id: "synthetic-pair-v1".into(),
            bathymetry,
            type_identifier,
        }
    }

    fn has_issue(pair: &BathymetryTidPair, code: GridPairIssueCode) -> bool {
        validate_grid_pair(pair)
            .issues
            .iter()
            .any(|issue| issue.code == code)
    }

    #[test]
    fn aligned_synthetic_grid_pair_is_valid() {
        assert!(validate_grid_pair(&valid_pair()).is_valid());
    }

    #[test]
    fn mismatched_product_version_fails_closed() {
        let mut pair = valid_pair();
        pair.type_identifier.product_version = "different-edition".into();
        let report = validate_grid_pair(&pair);
        assert!(
            report
                .issues
                .iter()
                .any(|i| i.code == GridPairIssueCode::ProductVersionMismatch)
        );
    }

    #[test]
    fn mismatched_dimensions_fail_closed() {
        let mut pair = valid_pair();
        pair.type_identifier.columns = 3;
        let report = validate_grid_pair(&pair);
        assert!(
            report
                .issues
                .iter()
                .any(|i| i.code == GridPairIssueCode::DimensionMismatch)
        );
    }

    #[test]
    fn mismatched_bounds_fail_closed() {
        let mut pair = valid_pair();
        pair.type_identifier.bounds.east_edge_microarcseconds += 15_000_000;
        let report = validate_grid_pair(&pair);
        assert!(
            report
                .issues
                .iter()
                .any(|i| i.code == GridPairIssueCode::BoundsMismatch)
        );
    }

    #[test]
    fn mismatched_crs_fails_closed() {
        let mut pair = valid_pair();
        pair.type_identifier.coordinate_reference_system = Some("EPSG:3857".into());
        let report = validate_grid_pair(&pair);
        assert!(
            report
                .issues
                .iter()
                .any(|i| i.code == GridPairIssueCode::CoordinateReferenceMismatch)
        );
    }

    #[test]
    fn mismatched_axis_order_fails_closed() {
        let mut pair = valid_pair();
        pair.type_identifier.row_axis = GridAxis::Longitude;
        pair.type_identifier.column_axis = GridAxis::Latitude;
        let report = validate_grid_pair(&pair);
        assert!(
            report
                .issues
                .iter()
                .any(|i| i.code == GridPairIssueCode::AxisOrderMismatch)
        );
    }

    #[test]
    fn dimension_and_spacing_must_reproduce_declared_bounds() {
        let mut pair = valid_pair();
        pair.bathymetry.cell_width_microarcseconds = 10_000_000;
        let report = validate_grid_pair(&pair);
        assert!(
            report
                .issues
                .iter()
                .any(|i| i.code == GridPairIssueCode::DimensionsDoNotMatchBounds)
        );
    }

    #[test]
    fn missing_vertical_datum_or_units_blocks_scientific_pair_validation() {
        let mut pair = valid_pair();
        pair.bathymetry.vertical_datum = None;
        pair.bathymetry.vertical_unit = None;
        let report = validate_grid_pair(&pair);
        assert!(
            report
                .issues
                .iter()
                .any(|i| i.code == GridPairIssueCode::MissingVerticalDatum)
        );
        assert!(
            report
                .issues
                .iter()
                .any(|i| i.code == GridPairIssueCode::MissingVerticalUnit)
        );
    }

    #[test]
    fn unknown_nodata_encoding_fails_closed() {
        let mut pair = valid_pair();
        pair.bathymetry.no_data = NoDataEncoding::Unknown;
        let report = validate_grid_pair(&pair);
        assert!(
            report
                .issues
                .iter()
                .any(|i| i.code == GridPairIssueCode::UnknownNoDataEncoding)
        );
        assert!(!report.is_valid());
    }

    #[test]
    fn nodata_does_not_become_zero_depth() {
        assert_eq!(
            decode_bathymetry_cell(-32768, NoDataEncoding::I16Sentinel(-32768)),
            Ok(BathymetryCell::NoData { raw_value: -32768 })
        );
        assert_eq!(
            decode_bathymetry_cell(0, NoDataEncoding::I16Sentinel(-32768)),
            Ok(BathymetryCell::Value(0))
        );
    }

    #[test]
    fn unknown_nodata_semantics_cannot_decode_a_cell() {
        assert_eq!(
            decode_bathymetry_cell(0, NoDataEncoding::Unknown),
            Err(CellDecodeError::UnknownNoDataSemantics)
        );
    }

    #[test]
    fn tid_unknown_and_mixed_codes_remain_distinct() {
        assert_eq!(
            decode_tid_cell(70, NoDataEncoding::U8Sentinel(u8::MAX)),
            Ok(TidCell::SourceClass {
                raw_code: 70,
                class: TidSourceClass::PreGeneratedMixedSources
            })
        );
        assert_eq!(
            decode_tid_cell(71, NoDataEncoding::U8Sentinel(u8::MAX)),
            Ok(TidCell::SourceClass {
                raw_code: 71,
                class: TidSourceClass::UnknownSource
            })
        );
    }

    #[test]
    fn unrecognized_tid_codes_are_preserved_not_downgraded_to_nodata() {
        assert_eq!(
            decode_tid_cell(255, NoDataEncoding::ExplicitNoSentinel),
            Ok(TidCell::SourceClass {
                raw_code: 255,
                class: TidSourceClass::UnrecognizedCode(255)
            })
        );
    }

    #[test]
    fn tid_nodata_is_not_misclassified_as_unknown_source() {
        assert_eq!(
            decode_tid_cell(255, NoDataEncoding::U8Sentinel(255)),
            Ok(TidCell::NoData { raw_value: 255 })
        );
    }

    #[test]
    fn unreviewed_no_data_semantics_do_not_degrade_into_a_valid_pair() {
        let mut pair = valid_pair();
        pair.type_identifier.no_data = NoDataEncoding::Unknown;
        assert!(!validate_grid_pair(&pair).is_valid());
    }

    #[test]
    fn malformed_digest_reference_is_rejected() {
        let mut pair = valid_pair();
        pair.type_identifier.artifact_sha256 = "not-a-digest".into();
        let report = validate_grid_pair(&pair);
        assert!(
            report
                .issues
                .iter()
                .any(|i| i.code == GridPairIssueCode::InvalidArtifactSha256)
        );
    }
    #[test]
    fn required_provenance_identity_fields_fail_closed() {
        let mut pair = valid_pair();
        pair.bathymetry.source_snapshot_id.clear();
        assert!(has_issue(&pair, GridPairIssueCode::MissingSnapshotId));

        let mut pair = valid_pair();
        pair.bathymetry.product_id.clear();
        assert!(has_issue(&pair, GridPairIssueCode::MissingProductId));

        let mut pair = valid_pair();
        pair.bathymetry.product_version.clear();
        assert!(has_issue(&pair, GridPairIssueCode::MissingProductVersion));

        let mut pair = valid_pair();
        pair.bathymetry.grid_id.clear();
        assert!(has_issue(&pair, GridPairIssueCode::MissingGridId));
    }

    #[test]
    fn invalid_grid_geometry_metadata_fails_closed() {
        let mut pair = valid_pair();
        pair.bathymetry.rows = 0;
        assert!(has_issue(&pair, GridPairIssueCode::InvalidDimensions));

        let mut pair = valid_pair();
        pair.bathymetry.bounds.east_edge_microarcseconds =
            pair.bathymetry.bounds.west_edge_microarcseconds;
        assert!(has_issue(&pair, GridPairIssueCode::InvalidBounds));

        let mut pair = valid_pair();
        pair.bathymetry.cell_width_microarcseconds = 0;
        assert!(has_issue(&pair, GridPairIssueCode::InvalidCellSpacing));
    }

    #[test]
    fn incomplete_coordinate_metadata_fails_closed() {
        let mut pair = valid_pair();
        pair.bathymetry.coordinate_reference_system = None;
        assert!(has_issue(
            &pair,
            GridPairIssueCode::MissingCoordinateReferenceSystem
        ));

        let mut pair = valid_pair();
        pair.bathymetry.horizontal_unit = None;
        assert!(has_issue(&pair, GridPairIssueCode::MissingHorizontalUnit));

        let mut pair = valid_pair();
        pair.bathymetry.row_axis = GridAxis::Unknown;
        assert!(has_issue(&pair, GridPairIssueCode::UnknownAxisOrder));

        let mut pair = valid_pair();
        pair.bathymetry.column_axis = GridAxis::Latitude;
        assert!(has_issue(&pair, GridPairIssueCode::DuplicateAxisOrder));

        let mut pair = valid_pair();
        pair.bathymetry.registration = GridRegistration::Unknown;
        assert!(has_issue(&pair, GridPairIssueCode::UnknownRegistration));
    }

    #[test]
    fn pair_identity_spacing_units_and_registration_mismatches_fail_closed() {
        let mut pair = valid_pair();
        pair.type_identifier.product_id = "different-product".into();
        assert!(has_issue(&pair, GridPairIssueCode::ProductMismatch));

        let mut pair = valid_pair();
        pair.type_identifier.grid_id = "different-grid".into();
        assert!(has_issue(&pair, GridPairIssueCode::GridIdentityMismatch));

        let mut pair = valid_pair();
        pair.type_identifier.cell_width_microarcseconds += 1;
        assert!(has_issue(&pair, GridPairIssueCode::CellSpacingMismatch));

        let mut pair = valid_pair();
        pair.type_identifier.horizontal_unit = Some("radians".into());
        assert!(has_issue(&pair, GridPairIssueCode::HorizontalUnitMismatch));

        let mut pair = valid_pair();
        pair.type_identifier.registration = GridRegistration::PixelArea;
        assert!(has_issue(&pair, GridPairIssueCode::RegistrationMismatch));
    }

    #[test]
    fn cell_decoders_reject_wrong_nodata_encoding_width() {
        assert_eq!(
            decode_bathymetry_cell(7, NoDataEncoding::U8Sentinel(255)),
            Err(CellDecodeError::NoDataEncodingTypeMismatch)
        );
        assert_eq!(
            decode_tid_cell(7, NoDataEncoding::I16Sentinel(-32768)),
            Err(CellDecodeError::NoDataEncodingTypeMismatch)
        );
    }

    #[test]
    fn same_pair_emits_same_ordered_validation_report() {
        let mut pair = valid_pair();
        pair.type_identifier.product_version = "mismatched".into();
        pair.type_identifier.bounds.east_edge_microarcseconds += 1;
        let first = validate_grid_pair(&pair);
        let second = validate_grid_pair(&pair);
        assert_eq!(first, second);
    }

    #[test]
    fn serde_rejects_unknown_grid_metadata_fields() {
        let encoded = serde_json::to_value(metadata()).unwrap();
        let mut object = encoded.as_object().unwrap().clone();
        object.insert(
            "owner".into(),
            serde_json::Value::String("unverified".into()),
        );
        assert!(serde_json::from_value::<GridMetadata>(serde_json::Value::Object(object)).is_err());
    }
}
