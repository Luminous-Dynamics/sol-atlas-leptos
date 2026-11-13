// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! AGS4 structural preflight for source-preserving geotechnical ingestion.
//!
//! This is deliberately not a replacement for the official AGS4 validator or
//! BGS/NGDC data-submission checks. It verifies the basic file envelope and
//! retains group/heading/unit/type/value rows without interpreting geotechnical
//! quantities. Callers must pin a version/dictionary and obtain an authoritative
//! validator receipt before treating an AGS file as qualified engineering data.

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
        let first = values.first()?.trim();
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
        assert!(issues.contains(&BgsAgsSubmissionIssue::MissingRequiredGroup("TYPE")) == false);
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
}