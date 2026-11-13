// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! AGS4 structural preflight for source-preserving geotechnical ingestion.
//!
//! This is deliberately not a replacement for the official AGS4 validator or
//! BGS/NGDC data-submission checks. It verifies the basic file envelope and
//! retains group/heading/unit/type/value rows without interpreting geotechnical
//! quantities. Callers must pin a version/dictionary and obtain an authoritative
//! validator receipt before treating an AGS file as qualified engineering data.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const AGS4_PREFLIGHT_MAX_BYTES: usize = 50 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgsGroup {
    pub name: String,
    pub headings: Vec<String>,
    pub units: Vec<String>,
    pub types: Vec<String>,
    pub rows: Vec<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgsDocument {
    pub groups: Vec<AgsGroup>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgsPreflightError {
    EmptyFile,
    FileTooLarge,
    NonAsciiByte,
    InvalidLineEnding,
    EmptyLine,
    UnquotedField,
    MalformedQuotedField,
    MissingGroupRow,
    EmptyGroupName,
    HeaderOrder,
    EmptyHeading,
    DuplicateHeading,
    HeaderWidthMismatch,
    DataWidthMismatch,
    GroupHasNoDataRows,
    UnsupportedDescriptor(String),
    MissingDeclaredVersion,
    UnsupportedDeclaredVersion(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BgsAgsSubmissionIssue {
    MissingRequiredGroup(&'static str),
    MissingLocationOrHoleGroup,
    MissingGeologyGroup,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct PendingGroup {
    name: String,
    headings: Vec<String>,
    units: Vec<String>,
    types: Vec<String>,
    rows: Vec<Vec<String>>,
    next_header: u8,
}


/// Current BGS AGS File Utilities endpoint and API version. A future API
/// version is not silently accepted as equivalent; callers must update/pin
/// the expected version and rerun the contract suite.
pub const BGS_AGS_API_VERSION: &str = "7.0";
pub const BGS_AGS_VALIDATE_ENDPOINT: &str = "https://agsapi.bgs.ac.uk/v7/validate/";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BgsAgsValidationReceipt {
    pub api_version: String,
    pub api_endpoint: String,
    pub filename: String,
    /// SHA-256 digest of the exact submitted AGS bytes.
    pub source_file_sha256: String,
    pub source_file_bytes: usize,
    /// Normalized dictionary version (e.g. v4_1_1), not a human label.
    pub dictionary_id: String,
    /// Required values for the paired AGS-schema and BGS-submission checks.
    pub checkers: Vec<String>,
    pub validator_library: String,
    pub request_time: String,
    /// SHA-256 digest of the exact JSON response bytes retained by the caller.
    pub response_sha256: String,
    pub valid: bool,
    pub error_count: usize,
    pub warnings_count: usize,
    pub fyi_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgsVersionError {
    Missing,
    Unsupported(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BgsAgsReceiptError {
    ApiVersionMismatch,
    EndpointMismatch,
    EmptyFilename,
    InvalidSourceDigest,
    SourceDigestMismatch,
    SourceSizeMismatch,
    InvalidResponseDigest,
    DictionaryMismatch,
    MissingAgsSchemaChecker,
    MissingBgsSubmissionChecker,
    EmptyValidatorLibrary,
    EmptyRequestTime,
    ValidatorRejectedFile,
    ErrorCountNonZero,
}

fn is_sha256_hex(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

/// Translate a declared TRAN_AGS value into the exact dictionary selector
/// accepted by the current BGS validator API. Values such as "4" and "4.0"
/// are not guessed or upgraded.
pub fn ags_dictionary_id(version: &str) -> Result<&'static str, AgsVersionError> {
    match version.trim() {
        "4.0.3" => Ok("v4_0_3"),
        "4.0.4" => Ok("v4_0_4"),
        "4.1" => Ok("v4_1"),
        "4.1.1" => Ok("v4_1_1"),
        "" => Err(AgsVersionError::Missing),
        other => Err(AgsVersionError::Unsupported(other.to_string())),
    }
}

/// Bind a BGS validation response to the exact submitted file and pinned
/// validator/dictionary. This checks the *receipt contract*, not the network
/// origin or signature. A trusted acquisition layer must preserve response
/// bytes, endpoint, TLS context, request parameters and timestamp; Mycelix or
/// another custody authority must authenticate the receipt before admission.
pub fn validate_bgs_ags_receipt(
    receipt: &BgsAgsValidationReceipt,
    expected_source_sha256: &str,
    expected_source_bytes: usize,
    expected_dictionary_id: &str,
) -> Result<(), Vec<BgsAgsReceiptError>> {
    let mut errors = Vec::new();

    if receipt.api_version != BGS_AGS_API_VERSION {
        errors.push(BgsAgsReceiptError::ApiVersionMismatch);
    }
    if receipt.api_endpoint != BGS_AGS_VALIDATE_ENDPOINT {
        errors.push(BgsAgsReceiptError::EndpointMismatch);
    }
    if receipt.filename.trim().is_empty() {
        errors.push(BgsAgsReceiptError::EmptyFilename);
    }
    if !is_sha256_hex(&receipt.source_file_sha256) {
        errors.push(BgsAgsReceiptError::InvalidSourceDigest);
    } else if receipt.source_file_sha256 != expected_source_sha256 {
        errors.push(BgsAgsReceiptError::SourceDigestMismatch);
    }
    if receipt.source_file_bytes != expected_source_bytes {
        errors.push(BgsAgsReceiptError::SourceSizeMismatch);
    }
    if !is_sha256_hex(&receipt.response_sha256) {
        errors.push(BgsAgsReceiptError::InvalidResponseDigest);
    }
    if receipt.dictionary_id != expected_dictionary_id {
        errors.push(BgsAgsReceiptError::DictionaryMismatch);
    }
    if !receipt.checkers.iter().any(|checker| checker == "ags") {
        errors.push(BgsAgsReceiptError::MissingAgsSchemaChecker);
    }
    if !receipt.checkers.iter().any(|checker| checker == "bgs") {
        errors.push(BgsAgsReceiptError::MissingBgsSubmissionChecker);
    }
    if receipt.validator_library.trim().is_empty() {
        errors.push(BgsAgsReceiptError::EmptyValidatorLibrary);
    }
    if receipt.request_time.trim().is_empty() {
        errors.push(BgsAgsReceiptError::EmptyRequestTime);
    }
    if !receipt.valid {
        errors.push(BgsAgsReceiptError::ValidatorRejectedFile);
    }
    if receipt.error_count != 0 {
        errors.push(BgsAgsReceiptError::ErrorCountNonZero);
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

impl PendingGroup {
    fn new(name: String) -> Self {
        Self {
            name,
            headings: Vec::new(),
            units: Vec::new(),
            types: Vec::new(),
            rows: Vec::new(),
            next_header: 0,
        }
    }

    fn finish(self) -> Result<AgsGroup, AgsPreflightError> {
        if self.next_header != 3 {
            return Err(AgsPreflightError::HeaderOrder);
        }
        if self.rows.is_empty() {
            return Err(AgsPreflightError::GroupHasNoDataRows);
        }
        Ok(AgsGroup {
            name: self.name,
            headings: self.headings,
            units: self.units,
            types: self.types,
            rows: self.rows,
        })
    }
}

fn parse_quoted_row(line: &[u8]) -> Result<Vec<String>, AgsPreflightError> {
    if line.is_empty() {
        return Err(AgsPreflightError::EmptyLine);
    }

    let mut fields = Vec::new();
    let mut index = 0usize;

    loop {
        if index >= line.len() || line[index] != b'"' {
            return Err(AgsPreflightError::UnquotedField);
        }
        index += 1;

        let mut field = Vec::new();
        let mut closed = false;
        while index < line.len() {
            match line[index] {
                b'"' if index + 1 < line.len() && line[index + 1] == b'"' => {
                    field.push(b'"');
                    index += 2;
                }
                b'"' => {
                    index += 1;
                    closed = true;
                    break;
                }
                byte => {
                    field.push(byte);
                    index += 1;
                }
            }
        }

        if !closed {
            return Err(AgsPreflightError::MalformedQuotedField);
        }
        fields.push(
            String::from_utf8(field).map_err(|_| AgsPreflightError::NonAsciiByte)?,
        );

        if index == line.len() {
            break;
        }
        if line[index] != b',' {
            return Err(AgsPreflightError::MalformedQuotedField);
        }
        index += 1;
        if index == line.len() {
            return Err(AgsPreflightError::UnquotedField);
        }
    }

    Ok(fields)
}

/// Parse AGS4's quoted group-row envelope.
///
/// The parser enforces ASCII, CRLF-delimited rows, quoted cells, GROUP/HEADING/
/// UNIT/TYPE order and field-width consistency. It does not validate the AGS
/// standard dictionary, units, geotechnical data, parent references, BGS
/// submission rules, coordinate frames, or the correctness of any measurement.
pub fn parse_ags4_preflight(bytes: &[u8]) -> Result<AgsDocument, AgsPreflightError> {
    if bytes.is_empty() {
        return Err(AgsPreflightError::EmptyFile);
    }
    if bytes.len() > AGS4_PREFLIGHT_MAX_BYTES {
        return Err(AgsPreflightError::FileTooLarge);
    }
    if bytes.iter().any(|byte| *byte > 0x7f) {
        return Err(AgsPreflightError::NonAsciiByte);
    }

    let mut lines: Vec<&[u8]> = Vec::new();
    let mut row_start = 0usize;
    let mut index = 0usize;
    while index < bytes.len() {
        match bytes[index] {
            b'\r' => {
                if index + 1 >= bytes.len() || bytes[index + 1] != b'\n' {
                    return Err(AgsPreflightError::InvalidLineEnding);
                }
                lines.push(&bytes[row_start..index]);
                index += 2;
                row_start = index;
            }
            b'\n' => return Err(AgsPreflightError::InvalidLineEnding),
            _ => index += 1,
        }
    }
    if row_start < bytes.len() {
        lines.push(&bytes[row_start..]);
    }

    let mut groups = Vec::new();
    let mut current: Option<PendingGroup> = None;

    for line in lines {
        let fields = parse_quoted_row(line)?;
        let Some(descriptor) = fields.first().map(String::as_str) else {
            return Err(AgsPreflightError::EmptyLine);
        };

        match descriptor {
            "GROUP" => {
                if fields.len() != 2 {
                    return Err(AgsPreflightError::HeaderWidthMismatch);
                }
                let name = fields[1].trim();
                if name.is_empty() {
                    return Err(AgsPreflightError::EmptyGroupName);
                }
                if let Some(previous) = current.take() {
                    groups.push(previous.finish()?);
                }
                current = Some(PendingGroup::new(name.to_string()));
            }
            "HEADING" => {
                let group = current.as_mut().ok_or(AgsPreflightError::MissingGroupRow)?;
                if group.next_header != 0 {
                    return Err(AgsPreflightError::HeaderOrder);
                }
                let headings = fields.into_iter().skip(1).collect::<Vec<_>>();
                if headings.is_empty() || headings.iter().any(|heading| heading.trim().is_empty()) {
                    return Err(AgsPreflightError::EmptyHeading);
                }
                let unique: BTreeSet<&str> = headings.iter().map(String::as_str).collect();
                if unique.len() != headings.len() {
                    return Err(AgsPreflightError::DuplicateHeading);
                }
                group.headings = headings;
                group.next_header = 1;
            }
            "UNIT" => {
                let group = current.as_mut().ok_or(AgsPreflightError::MissingGroupRow)?;
                if group.next_header != 1 {
                    return Err(AgsPreflightError::HeaderOrder);
                }
                if fields.len() != group.headings.len() + 1 {
                    return Err(AgsPreflightError::HeaderWidthMismatch);
                }
                group.units = fields.into_iter().skip(1).collect();
                group.next_header = 2;
            }
            "TYPE" => {
                let group = current.as_mut().ok_or(AgsPreflightError::MissingGroupRow)?;
                if group.next_header != 2 {
                    return Err(AgsPreflightError::HeaderOrder);
                }
                if fields.len() != group.headings.len() + 1 {
                    return Err(AgsPreflightError::HeaderWidthMismatch);
                }
                group.types = fields.into_iter().skip(1).collect();
                group.next_header = 3;
            }
            "DATA" => {
                let group = current.as_mut().ok_or(AgsPreflightError::MissingGroupRow)?;
                if group.next_header != 3 {
                    return Err(AgsPreflightError::HeaderOrder);
                }
                if fields.len() != group.headings.len() + 1 {
                    return Err(AgsPreflightError::DataWidthMismatch);
                }
                group.rows.push(fields.into_iter().skip(1).collect());
            }
            other => return Err(AgsPreflightError::UnsupportedDescriptor(other.to_string())),
        }
    }

    if let Some(last) = current {
        groups.push(last.finish()?);
    }
    if groups.is_empty() {
        return Err(AgsPreflightError::MissingGroupRow);
    }

    Ok(AgsDocument { groups })
}

impl AgsGroup {
    pub fn column_index(&self, heading: &str) -> Option<usize> {
        self.headings.iter().position(|value| value == heading)
    }

    pub fn values(&self, heading: &str) -> Option<Vec<&str>> {
        let index = self.column_index(heading)?;
        Some(
            self.rows
                .iter()
                .filter_map(|row| row.get(index).map(String::as_str))
                .collect(),
        )
    }
}

impl AgsDocument {
    pub fn group(&self, name: &str) -> Option<&AgsGroup> {
        self.groups.iter().find(|group| group.name == name)
    }

    /// Return a declared TRAN_AGS value if present. A missing/ambiguous
    /// version is not silently rewritten to the BGS validator's default.
    pub fn declared_ags_version(&self) -> Option<&str> {
        let group = self.group("TRAN")?;
        let values = group.values("TRAN_AGS")?;
        if values.len() != 1 {
            return None;
        }
        let first = values[0].trim();
        if first.is_empty() {
            None
        } else {
            Some(first)
        }
    }

    /// Check only the documented BGS required-group set. This is not the full
    /// BGS validator and does not check coordinates, depth, units or references.
    pub fn bgs_required_group_issues(&self) -> Vec<BgsAgsSubmissionIssue> {
        let names: BTreeSet<&str> = self.groups.iter().map(|group| group.name.as_str()).collect();
        let mut issues = Vec::new();

        for required in ["PROJ", "ABBR", "TYPE", "UNIT"] {
            if !names.contains(required) {
                issues.push(BgsAgsSubmissionIssue::MissingRequiredGroup(required));
            }
        }
        if !names.contains("LOCA") && !names.contains("HOLE") {
            issues.push(BgsAgsSubmissionIssue::MissingLocationOrHoleGroup);
        }
        if !names.contains("GEOL") {
            issues.push(BgsAgsSubmissionIssue::MissingGeologyGroup);
        }

        issues
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SYNTHETIC_AGS: &str = include_str!(
        "../../research/earth-observability/fixtures/ags4-synthetic-v0.1.ags"
    );

    #[test]
    fn synthetic_ags4_file_parses_and_declares_version() {
        let doc = parse_ags4_preflight(SYNTHETIC_AGS.as_bytes()).expect("synthetic AGS parses");
        assert_eq!(doc.groups.len(), 7);
        assert_eq!(doc.declared_ags_version(), Some("4.1.1"));
        assert!(doc.bgs_required_group_issues().is_empty());
    }

    #[test]
    fn parser_preserves_null_cells_as_empty_strings() {
        let doc = parse_ags4_preflight(SYNTHETIC_AGS.as_bytes()).expect("synthetic AGS parses");
        let proj = doc.group("PROJ").unwrap();
        assert_eq!(proj.values("PROJ_DESC").unwrap(), vec![""]);
    }

    #[test]
    fn parser_preserves_escaped_quotes_and_commas_inside_values() {
        let bytes = b"GROUP,\"PROJ\"\r\nHEADING,\"PROJ_ID\",\"PROJ_NAME\"\r\nUNIT,\"\",\"\"\r\nTYPE,\"X\",\"X\"\r\nDATA,\"P-1\",\"Borehole, \"\"North\"\"\"\r\n";
        let doc = parse_ags4_preflight(bytes).expect("quoted AGS value parses");
        assert_eq!(
            doc.group("PROJ").unwrap().values("PROJ_NAME").unwrap(),
            vec!["Borehole, \"North\""]
        );
    }

    #[test]
    fn line_endings_must_be_crlf() {
        let bytes = b"GROUP,\"PROJ\"\nHEADING,\"PROJ_ID\"\nUNIT,\"\"\nTYPE,\"X\"\nDATA,\"P-1\"\n";
        assert_eq!(
            parse_ags4_preflight(bytes),
            Err(AgsPreflightError::InvalidLineEnding)
        );
    }

    #[test]
    fn non_ascii_bytes_are_rejected() {
        let bytes = b"GROUP,\"PROJ\"\r\nHEADING,\"PROJ_ID\"\r\nUNIT,\"\"\r\nTYPE,\"X\"\r\nDATA,\"P-\xC3\x89\"\r\n";
        assert_eq!(
            parse_ags4_preflight(bytes),
            Err(AgsPreflightError::NonAsciiByte)
        );
    }

    #[test]
    fn header_order_is_enforced() {
        let bytes = b"GROUP,\"PROJ\"\r\nHEADING,\"PROJ_ID\"\r\nTYPE,\"X\"\r\nUNIT,\"\"\r\nDATA,\"P-1\"\r\n";
        assert_eq!(
            parse_ags4_preflight(bytes),
            Err(AgsPreflightError::HeaderOrder)
        );
    }

    #[test]
    fn data_rows_must_match_heading_width() {
        let bytes = b"GROUP,\"PROJ\"\r\nHEADING,\"PROJ_ID\",\"PROJ_NAME\"\r\nUNIT,\"\",\"\"\r\nTYPE,\"X\",\"X\"\r\nDATA,\"P-1\"\r\n";
        assert_eq!(
            parse_ags4_preflight(bytes),
            Err(AgsPreflightError::DataWidthMismatch)
        );
    }

    #[test]
    fn duplicate_headings_are_rejected() {
        let bytes = b"GROUP,\"PROJ\"\r\nHEADING,\"PROJ_ID\",\"PROJ_ID\"\r\nUNIT,\"\",\"\"\r\nTYPE,\"X\",\"X\"\r\nDATA,\"P-1\",\"P-1\"\r\n";
        assert_eq!(
            parse_ags4_preflight(bytes),
            Err(AgsPreflightError::DuplicateHeading)
        );
    }

    #[test]
    fn empty_groups_are_rejected() {
        let bytes = b"GROUP,\"PROJ\"\r\nHEADING,\"PROJ_ID\"\r\nUNIT,\"\"\r\nTYPE,\"X\"\r\nGROUP,\"LOCA\"\r\nHEADING,\"LOCA_ID\"\r\nUNIT,\"\"\r\nTYPE,\"X\"\r\nDATA,\"BH-1\"\r\n";
        assert_eq!(
            parse_ags4_preflight(bytes),
            Err(AgsPreflightError::GroupHasNoDataRows)
        );
    }

    #[test]
    fn missing_bgs_required_groups_are_reported_separately() {
        let bytes = b"GROUP,\"PROJ\"\r\nHEADING,\"PROJ_ID\"\r\nUNIT,\"\"\r\nTYPE,\"X\"\r\nDATA,\"P-1\"\r\n";
        let doc = parse_ags4_preflight(bytes).expect("structural preflight passes");
        let issues = doc.bgs_required_group_issues();
        assert!(issues.contains(&BgsAgsSubmissionIssue::MissingRequiredGroup("ABBR")));
        assert!(issues.contains(&BgsAgsSubmissionIssue::MissingRequiredGroup("TYPE")));
        assert!(issues.contains(&BgsAgsSubmissionIssue::MissingLocationOrHoleGroup));
        assert!(issues.contains(&BgsAgsSubmissionIssue::MissingGeologyGroup));
    }

    #[test]
    fn malformed_quotes_are_rejected() {
        let bytes = b"GROUP,\"PROJ\"\r\nHEADING,\"PROJ_ID\r\nUNIT,\"\"\r\nTYPE,\"X\"\r\nDATA,\"P-1\"\r\n";
        assert_eq!(
            parse_ags4_preflight(bytes),
            Err(AgsPreflightError::MalformedQuotedField)
        );
    }

    #[test]
    fn missing_group_row_is_rejected() {
        let bytes = b"HEADING,\"PROJ_ID\"\r\nUNIT,\"\"\r\nTYPE,\"X\"\r\nDATA,\"P-1\"\r\n";
        assert_eq!(
            parse_ags4_preflight(bytes),
            Err(AgsPreflightError::MissingGroupRow)
        );
    }

    #[test]
    fn unsupported_descriptors_are_rejected() {
        let bytes = b"GROUP,\"PROJ\"\r\nHEADING,\"PROJ_ID\"\r\nUNIT,\"\"\r\nTYPE,\"X\"\r\nNOTE,\"not a valid AGS descriptor\"\r\n";
        assert!(matches!(
            parse_ags4_preflight(bytes),
            Err(AgsPreflightError::UnsupportedDescriptor(_))
        ));
    }


    fn valid_bgs_receipt() -> BgsAgsValidationReceipt {
        BgsAgsValidationReceipt {
            api_version: BGS_AGS_API_VERSION.to_string(),
            api_endpoint: BGS_AGS_VALIDATE_ENDPOINT.to_string(),
            filename: "synthetic.ags".to_string(),
            source_file_sha256: "a".repeat(64),
            source_file_bytes: SYNTHETIC_AGS.len(),
            dictionary_id: "v4_1_1".to_string(),
            checkers: vec!["ags".to_string(), "bgs".to_string()],
            validator_library: "python_ags4 v1.1.0".to_string(),
            request_time: "2026-11-13T00:00:00Z".to_string(),
            response_sha256: "b".repeat(64),
            valid: true,
            error_count: 0,
            warnings_count: 1,
            fyi_count: 0,
        }
    }

    #[test]
    fn declared_ags_version_maps_to_exact_dictionary_without_fallback() {
        assert_eq!(ags_dictionary_id("4.1.1"), Ok("v4_1_1"));
        assert_eq!(ags_dictionary_id("4.0.3"), Ok("v4_0_3"));
        assert_eq!(ags_dictionary_id(""), Err(AgsVersionError::Missing));
        assert_eq!(
            ags_dictionary_id("4"),
            Err(AgsVersionError::Unsupported("4".to_string()))
        );
        let doc = parse_ags4_preflight(SYNTHETIC_AGS.as_bytes()).expect("synthetic AGS parses");
        assert_eq!(
            ags_dictionary_id(doc.declared_ags_version().unwrap()),
            Ok("v4_1_1")
        );
    }

    #[test]
    fn ambiguous_or_empty_transfer_version_is_not_defaulted() {
        let mut doc = parse_ags4_preflight(SYNTHETIC_AGS.as_bytes())
            .expect("synthetic AGS parses");
        let tran = doc.groups.iter_mut().find(|group| group.name == "TRAN").unwrap();
        tran.rows[0][1].clear();
        assert_eq!(doc.declared_ags_version(), None);

        tran.rows[0][1] = "4.1.1".to_string();
        tran.rows.push(vec![
            "SYN-TRAN-002".to_string(),
            "4.1".to_string(),
            "conflicting version row".to_string(),
        ]);
        assert_eq!(doc.declared_ags_version(), None);
    }

    #[test]
    fn validator_receipt_binds_file_digest_size_dictionary_and_both_checkers() {
        let receipt = valid_bgs_receipt();
        assert_eq!(
            validate_bgs_ags_receipt(
                &receipt,
                &"a".repeat(64),
                SYNTHETIC_AGS.len(),
                "v4_1_1"
            ),
            Ok(())
        );
    }

    #[test]
    fn validator_receipt_rejects_reuse_for_different_file_or_dictionary() {
        let receipt = valid_bgs_receipt();
        let errors = validate_bgs_ags_receipt(
            &receipt,
            &"c".repeat(64),
            SYNTHETIC_AGS.len() + 1,
            "v4_1",
        )
        .expect_err("receipt must not be reused");
        assert!(errors.contains(&BgsAgsReceiptError::SourceDigestMismatch));
        assert!(errors.contains(&BgsAgsReceiptError::SourceSizeMismatch));
        assert!(errors.contains(&BgsAgsReceiptError::DictionaryMismatch));
    }

    #[test]
    fn validator_receipt_requires_pinned_endpoint_checkers_and_response_digest() {
        let mut receipt = valid_bgs_receipt();
        receipt.api_version = "6.9".to_string();
        receipt.api_endpoint = "https://other.invalid/validate".to_string();
        receipt.checkers = vec!["ags".to_string()];
        receipt.response_sha256 = "not-a-hash".to_string();
        let errors = validate_bgs_ags_receipt(
            &receipt,
            &"a".repeat(64),
            SYNTHETIC_AGS.len(),
            "v4_1_1",
        )
        .expect_err("receipt contract must fail");
        assert!(errors.contains(&BgsAgsReceiptError::ApiVersionMismatch));
        assert!(errors.contains(&BgsAgsReceiptError::EndpointMismatch));
        assert!(errors.contains(&BgsAgsReceiptError::MissingBgsSubmissionChecker));
        assert!(errors.contains(&BgsAgsReceiptError::InvalidResponseDigest));
    }

    #[test]
    fn validator_error_status_cannot_be_promoted_by_zeroed_local_assertions() {
        let mut receipt = valid_bgs_receipt();
        receipt.valid = false;
        receipt.error_count = 3;
        let errors = validate_bgs_ags_receipt(
            &receipt,
            &"a".repeat(64),
            SYNTHETIC_AGS.len(),
            "v4_1_1",
        )
        .expect_err("rejected source must stay rejected");
        assert!(errors.contains(&BgsAgsReceiptError::ValidatorRejectedFile));
        assert!(errors.contains(&BgsAgsReceiptError::ErrorCountNonZero));
    }

    #[test]
    fn warning_count_is_preserved_but_is_not_automatically_an_error() {
        let receipt = valid_bgs_receipt();
        assert!(receipt.warnings_count > 0);
        assert!(validate_bgs_ags_receipt(
            &receipt,
            &"a".repeat(64),
            SYNTHETIC_AGS.len(),
            "v4_1_1"
        )
        .is_ok());
    }

}