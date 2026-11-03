// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later

//! Bounded, proof-carrying source inventory manifests.
//!
//! This module validates the internal structure of a declared source query
//! enumeration. It does not authenticate source signatures, prove that the
//! source claim is true, or prove that the declared owner universe is complete.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet};

const INVENTORY_SCHEMA: &str = "luminous.federated-source-inventory-manifest.v0.1";
const PAGE_RECEIPT_DOMAIN: &[u8] = b"LUMINOUS-SOURCE-INVENTORY-PAGE-RECEIPT-V0.1\0";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SnapshotSemantics {
    ImmutableSnapshot,
    WatermarkBound,
    BestEffort,
    Unsupported,
}

impl SnapshotSemantics {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ImmutableSnapshot => "immutable_snapshot",
            Self::WatermarkBound => "watermark_bound",
            Self::BestEffort => "best_effort",
            Self::Unsupported => "unsupported",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ReportedTotalSemantics {
    UniqueNativeRecords,
    RawPageItems,
}

impl ReportedTotalSemantics {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::UniqueNativeRecords => "unique_native_records",
            Self::RawPageItems => "raw_page_items",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InventoryDisposition {
    CompleteWithinDeclaredQuery,
    CompleteWithDeclaredExclusions,
    IncompleteOrIndeterminate,
    InvalidInput,
    NotComparable,
}

impl InventoryDisposition {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::CompleteWithinDeclaredQuery => "complete_within_declared_query",
            Self::CompleteWithDeclaredExclusions => "complete_with_declared_exclusions",
            Self::IncompleteOrIndeterminate => "incomplete_or_indeterminate",
            Self::InvalidInput => "invalid_input",
            Self::NotComparable => "not_comparable",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InventoryFindingSeverity {
    Incomplete,
    Invalid,
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InventoryFinding {
    pub severity: InventoryFindingSeverity,
    pub code: String,
    pub subject: String,
    pub detail: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InventoryExclusion {
    pub exclusion_code: String,
    pub item_count: u64,
    pub policy_ref: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct InventoryPageReceipt {
    pub inventory_id: String,
    pub partition_id: String,
    pub ordinal: u32,
    /// SHA-256 of the exact source response page bytes. The verifier does not
    /// fetch those bytes; this claim remains separately unauthenticated.
    pub response_payload_sha256: String,
    pub item_count: u64,
    /// Owner-native identity digests for the items in this page. This list
    /// permits cross-page duplicate detection without moving source payloads.
    pub native_record_identity_sha256s: Vec<String>,
    /// Tombstones must be a subset of this page's native identity digest list.
    pub tombstone_identity_sha256s: Vec<String>,
    pub cursor_before_sha256: Option<String>,
    pub cursor_after_sha256: Option<String>,
    /// Hash of the previous page receipt in this partition's chain.
    pub predecessor_page_receipt_sha256: Option<String>,
    pub snapshot_sha256: Option<String>,
    pub source_watermark: Option<String>,
    pub response_at: String,
    pub terminal: bool,
}

impl InventoryPageReceipt {
    /// Hashes the ordered, typed page receipt representation using a domain
    /// separator. This binds cursor state, counts, snapshot and identity list.
    pub fn receipt_sha256(&self) -> Result<String, String> {
        let bytes = serde_json::to_vec(self)
            .map_err(|error| format!("page receipt serialization failed: {error}"))?;
        let mut encoded = PAGE_RECEIPT_DOMAIN.to_vec();
        append_field(&mut encoded, &bytes);
        Ok(sha256_hex(&encoded))
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourceInventoryManifest {
    pub schema_version: String,
    pub inventory_id: String,
    pub source_owner_namespace: String,
    pub source_system_id: String,
    pub connector_id: String,
    pub connector_build_sha256: String,
    pub query_scope_sha256: String,
    pub query_scope_summary: String,
    pub snapshot_semantics: SnapshotSemantics,
    pub snapshot_token: Option<String>,
    pub snapshot_sha256: Option<String>,
    pub source_watermark: Option<String>,
    pub required_partitions: Vec<String>,
    pub enumeration_started_at: String,
    pub enumeration_finished_at: String,
    pub page_receipts: Vec<InventoryPageReceipt>,
    pub reported_total_count: Option<u64>,
    pub reported_total_count_semantics: Option<ReportedTotalSemantics>,
    pub enumerated_item_count: u64,
    pub enumerated_unique_count: u64,
    pub duplicate_count: u64,
    pub excluded_count: u64,
    pub exclusions: Vec<InventoryExclusion>,
    pub tombstone_count: u64,
    pub tombstones_included: bool,
    pub pagination_exhausted: bool,
    pub snapshot_consistent: bool,
    pub truncation_detected: bool,
    pub attestation_ref: Option<String>,
    pub issuer_key_id: Option<String>,
    pub limitations: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InventoryValidation {
    pub disposition: InventoryDisposition,
    pub inventory_id: String,
    pub manifest_sha256: String,
    pub page_count: usize,
    pub required_partition_count: usize,
    pub enumerated_item_count: u64,
    pub enumerated_unique_count: u64,
    pub duplicate_count: u64,
    pub excluded_count: u64,
    pub tombstone_count: u64,
    /// Source response bytes are not retrieved and independently hashed here.
    pub page_payload_byte_verification: String,
    /// Source statement signatures and trust paths are not authenticated here.
    pub source_attestation_authentication: String,
    /// A structurally valid manifest does not establish source truth.
    pub source_truth: String,
    /// The result is bounded to the manifest's exact declared query and snapshot.
    pub claim_scope: String,
    pub findings: Vec<InventoryFinding>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExpectedSourceOwner {
    pub owner_namespace: String,
    pub source_system_id: String,
    pub required: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceUniverseDisposition {
    DeclaredOwnerSetCovered,
    IncompleteOrIndeterminate,
    InvalidInput,
}

impl SourceUniverseDisposition {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::DeclaredOwnerSetCovered => "declared_owner_set_covered",
            Self::IncompleteOrIndeterminate => "incomplete_or_indeterminate",
            Self::InvalidInput => "invalid_input",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SourceUniverseValidation {
    pub disposition: SourceUniverseDisposition,
    pub expected_owner_count: usize,
    pub required_owner_count: usize,
    pub observed_manifest_count: usize,
    pub missing_required_owner_count: usize,
    /// Coverage is only relative to the caller-supplied expected owner set.
    pub claim_scope: String,
    pub findings: Vec<InventoryFinding>,
}

#[derive(Default)]
struct Findings {
    rows: BTreeSet<InventoryFinding>,
}

impl Findings {
    fn invalid(&mut self, code: &str, subject: impl Into<String>, detail: impl Into<String>) {
        self.rows.insert(InventoryFinding {
            severity: InventoryFindingSeverity::Invalid,
            code: code.into(),
            subject: subject.into(),
            detail: detail.into(),
        });
    }

    fn incomplete(&mut self, code: &str, subject: impl Into<String>, detail: impl Into<String>) {
        self.rows.insert(InventoryFinding {
            severity: InventoryFindingSeverity::Incomplete,
            code: code.into(),
            subject: subject.into(),
            detail: detail.into(),
        });
    }

    fn disposition(&self) -> InventoryDisposition {
        if self
            .rows
            .iter()
            .any(|finding| finding.severity == InventoryFindingSeverity::Invalid)
        {
            InventoryDisposition::InvalidInput
        } else if !self.rows.is_empty() {
            InventoryDisposition::IncompleteOrIndeterminate
        } else {
            InventoryDisposition::CompleteWithinDeclaredQuery
        }
    }

    fn rows(&self) -> Vec<InventoryFinding> {
        self.rows.iter().cloned().collect()
    }
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn valid_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn append_field(encoded: &mut Vec<u8>, value: &[u8]) {
    encoded.extend_from_slice(&(value.len() as u64).to_be_bytes());
    encoded.extend_from_slice(value);
}

fn valid_utc_timestamp(value: &str) -> bool {
    let bytes = value.as_bytes();
    if bytes.len() != 20
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || bytes[10] != b'T'
        || bytes[13] != b':'
        || bytes[16] != b':'
        || bytes[19] != b'Z'
    {
        return false;
    }
    for (index, byte) in bytes.iter().enumerate() {
        if !matches!(index, 4 | 7 | 10 | 13 | 16 | 19) && !byte.is_ascii_digit() {
            return false;
        }
    }
    let at = |start: usize, end: usize| {
        std::str::from_utf8(&bytes[start..end])
            .ok()
            .and_then(|part| part.parse::<u32>().ok())
    };
    let year = at(0, 4).unwrap_or(0);
    let month = at(5, 7).unwrap_or(0);
    let day = at(8, 10).unwrap_or(0);
    let hour = at(11, 13).unwrap_or(99);
    let minute = at(14, 16).unwrap_or(99);
    let second = at(17, 19).unwrap_or(99);
    if year == 0 || !(1..=12).contains(&month) || hour > 23 || minute > 59 || second > 59 {
        return false;
    }
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let max_day = match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if leap => 29,
        2 => 28,
        _ => 0,
    };
    day >= 1 && day <= max_day
}

fn validate_page_partition(
    partition_id: &str,
    pages: &[&InventoryPageReceipt],
    manifest: &SourceInventoryManifest,
    findings: &mut Findings,
    all_tombstone_identities: &mut BTreeSet<String>,
) -> (u64, u64, bool) {
    let mut item_count = 0u64;
    let mut identities = BTreeSet::new();
    let mut seen_cursors = BTreeSet::new();
    let mut previous_after: Option<String> = None;
    let mut previous_receipt_sha256: Option<String> = None;
    let mut previous_response_at: Option<&str> = None;
    let mut terminal_seen = false;
    let mut chain_well_formed = true;

    if pages.is_empty() {
        findings.incomplete(
            "INVENTORY_PARTITION_NOT_ENUMERATED",
            partition_id,
            "required partition has no page receipts",
        );
        return (0, 0, false);
    }

    for (expected_ordinal, page) in pages.iter().enumerate() {
        if page.ordinal as usize != expected_ordinal {
            chain_well_formed = false;
            findings.invalid(
                "INVENTORY_PAGE_ORDINAL_GAP",
                format!("{partition_id}:{}", page.ordinal),
                "page ordinals must be contiguous from zero within each partition",
            );
        }
        if terminal_seen {
            chain_well_formed = false;
            findings.invalid(
                "INVENTORY_PAGE_AFTER_TERMINAL",
                format!("{partition_id}:{}", page.ordinal),
                "no page may follow a terminal page in the same partition",
            );
        }
        if page.inventory_id != manifest.inventory_id {
            chain_well_formed = false;
            findings.invalid(
                "INVENTORY_PAGE_ID_MISMATCH",
                format!("{partition_id}:{}", page.ordinal),
                "page receipt inventory ID must match its containing manifest",
            );
        }
        if page.partition_id != partition_id {
            chain_well_formed = false;
            findings.invalid(
                "INVENTORY_PAGE_PARTITION_MISMATCH",
                format!("{partition_id}:{}", page.ordinal),
                "page was grouped under a different partition ID",
            );
        }
        if !valid_sha256(&page.response_payload_sha256) {
            chain_well_formed = false;
            findings.invalid(
                "INVALID_INVENTORY_PAGE_PAYLOAD_DIGEST",
                format!("{partition_id}:{}", page.ordinal),
                "response payload digest must be lowercase SHA-256",
            );
        }
        if !valid_utc_timestamp(&page.response_at) {
            chain_well_formed = false;
            findings.invalid(
                "INVALID_INVENTORY_PAGE_RESPONSE_TIME",
                format!("{partition_id}:{}", page.ordinal),
                "page response time must be canonical UTC whole-second time",
            );
        } else if page.response_at < manifest.enumeration_started_at
            || page.response_at > manifest.enumeration_finished_at
        {
            chain_well_formed = false;
            findings.invalid(
                "INVENTORY_PAGE_RESPONSE_OUTSIDE_ENUMERATION_INTERVAL",
                format!("{partition_id}:{}", page.ordinal),
                "page response time must be within the manifest enumeration interval",
            );
        } else if previous_response_at.is_some_and(|previous| page.response_at.as_str() < previous)
        {
            chain_well_formed = false;
            findings.invalid(
                "INVENTORY_PAGE_RESPONSE_TIME_REVERSED",
                format!("{partition_id}:{}", page.ordinal),
                "page response times must be nondecreasing within a sequential partition chain",
            );
        }
        previous_response_at = Some(page.response_at.as_str());
        if let Some(manifest_snapshot) = manifest.snapshot_sha256.as_deref() {
            if page.snapshot_sha256.as_deref() != Some(manifest_snapshot) {
                chain_well_formed = false;
                findings.invalid(
                    "INVENTORY_PAGE_SNAPSHOT_MISMATCH",
                    format!("{partition_id}:{}", page.ordinal),
                    "page snapshot digest must match the manifest snapshot digest",
                );
            }
        } else if page.snapshot_sha256.is_some() {
            chain_well_formed = false;
            findings.invalid(
                "UNDECLARED_PAGE_SNAPSHOT",
                format!("{partition_id}:{}", page.ordinal),
                "page provides a snapshot digest that is not bound by the manifest",
            );
        }
        if let Some(watermark) = manifest.source_watermark.as_deref() {
            if page.source_watermark.as_deref() != Some(watermark) {
                chain_well_formed = false;
                findings.invalid(
                    "INVENTORY_PAGE_WATERMARK_MISMATCH",
                    format!("{partition_id}:{}", page.ordinal),
                    "page watermark must match the manifest source watermark",
                );
            }
        } else if page.source_watermark.is_some() {
            chain_well_formed = false;
            findings.invalid(
                "UNDECLARED_PAGE_WATERMARK",
                format!("{partition_id}:{}", page.ordinal),
                "page provides a watermark that is not bound by the manifest",
            );
        }
        if page.native_record_identity_sha256s.len() as u64 != page.item_count {
            chain_well_formed = false;
            findings.invalid(
                "INVENTORY_PAGE_ITEM_IDENTITY_COUNT_MISMATCH",
                format!("{partition_id}:{}", page.ordinal),
                "identity digest list length must equal the page item count",
            );
        }
        let page_identity_set = page
            .native_record_identity_sha256s
            .iter()
            .filter(|identity| valid_sha256(identity))
            .cloned()
            .collect::<BTreeSet<_>>();
        for identity in &page.native_record_identity_sha256s {
            if !valid_sha256(identity) {
                chain_well_formed = false;
                findings.invalid(
                    "INVALID_NATIVE_RECORD_IDENTITY_DIGEST",
                    format!("{partition_id}:{}", page.ordinal),
                    "native record identity digest must be lowercase SHA-256",
                );
            } else {
                identities.insert(identity.clone());
            }
        }
        for tombstone in &page.tombstone_identity_sha256s {
            if !valid_sha256(tombstone) || !page_identity_set.contains(tombstone) {
                chain_well_formed = false;
                findings.invalid(
                    "INVENTORY_TOMBSTONE_IDENTITY_NOT_IN_PAGE",
                    format!("{partition_id}:{}", page.ordinal),
                    "each tombstone identity must be valid and present in the same page identity list",
                );
            } else {
                all_tombstone_identities.insert(tombstone.clone());
            }
        }

        if page.cursor_before_sha256 != previous_after {
            chain_well_formed = false;
            findings.invalid(
                "INVENTORY_CURSOR_CHAIN_DISCONTINUITY",
                format!("{partition_id}:{}", page.ordinal),
                "page cursor_before must equal the preceding page cursor_after",
            );
        }
        if page
            .cursor_before_sha256
            .as_deref()
            .is_some_and(|cursor| !valid_sha256(cursor))
        {
            chain_well_formed = false;
            findings.invalid(
                "INVALID_INVENTORY_CURSOR_DIGEST",
                format!("{partition_id}:{}", page.ordinal),
                "cursor hashes must be lowercase SHA-256",
            );
        }

        if page.terminal {
            terminal_seen = true;
            if page.cursor_after_sha256.is_some() {
                chain_well_formed = false;
                findings.invalid(
                    "TERMINAL_PAGE_HAS_NEXT_CURSOR",
                    format!("{partition_id}:{}", page.ordinal),
                    "terminal page must not declare a next cursor",
                );
            }
        } else {
            match page.cursor_after_sha256.as_ref() {
                Some(cursor) if valid_sha256(cursor) && seen_cursors.insert(cursor.clone()) => {
                    previous_after = Some(cursor.clone());
                }
                Some(_) => {
                    chain_well_formed = false;
                    findings.invalid(
                        "INVENTORY_CURSOR_REPLAY",
                        format!("{partition_id}:{}", page.ordinal),
                        "next cursor hash must be valid and not previously observed",
                    );
                }
                None => {
                    chain_well_formed = false;
                    findings.invalid(
                        "NONTERMINAL_PAGE_MISSING_NEXT_CURSOR",
                        format!("{partition_id}:{}", page.ordinal),
                        "a nonterminal page must declare the next cursor hash",
                    );
                }
            }
        }

        if page.predecessor_page_receipt_sha256 != previous_receipt_sha256 {
            chain_well_formed = false;
            findings.invalid(
                "INVENTORY_PAGE_PREDECESSOR_MISMATCH",
                format!("{partition_id}:{}", page.ordinal),
                "page predecessor digest must bind the preceding page receipt",
            );
        }
        match page.receipt_sha256() {
            Ok(receipt_sha256) => previous_receipt_sha256 = Some(receipt_sha256),
            Err(error) => {
                chain_well_formed = false;
                findings.invalid(
                    "INVENTORY_PAGE_RECEIPT_DIGEST_FAILED",
                    format!("{partition_id}:{}", page.ordinal),
                    error,
                );
            }
        }
        match item_count.checked_add(page.item_count) {
            Some(total) => item_count = total,
            None => {
                chain_well_formed = false;
                findings.invalid(
                    "INVENTORY_PAGE_ITEM_COUNT_OVERFLOW",
                    partition_id,
                    "summed page item count exceeds u64",
                );
            }
        }
    }

    if !terminal_seen {
        findings.incomplete(
            "INVENTORY_PARTITION_TERMINAL_NOT_OBSERVED",
            partition_id,
            "authoritative terminal pagination state was not observed for this partition",
        );
    }
    (
        item_count,
        identities.len() as u64,
        chain_well_formed && terminal_seen,
    )
}

/// Validate the exact structure of a source enumeration receipt.
///
/// A complete disposition is bounded to the declared source owner, system,
/// query, partitions, snapshot/watermark and enumeration interval. It is not
/// a claim that the owner set is complete or that the source statement is true.
pub fn validate_source_inventory_manifest(
    manifest: &SourceInventoryManifest,
) -> InventoryValidation {
    let mut findings = Findings::default();

    if manifest.schema_version != INVENTORY_SCHEMA {
        findings.invalid(
            "UNSUPPORTED_INVENTORY_SCHEMA",
            "manifest",
            "unsupported source inventory manifest schema version",
        );
    }
    if manifest.inventory_id.trim().is_empty()
        || manifest.inventory_id != manifest.inventory_id.trim()
        || manifest.source_owner_namespace.trim().is_empty()
        || manifest.source_owner_namespace != manifest.source_owner_namespace.trim()
        || manifest.source_system_id.trim().is_empty()
        || manifest.source_system_id != manifest.source_system_id.trim()
        || manifest.connector_id.trim().is_empty()
        || manifest.connector_id != manifest.connector_id.trim()
        || manifest.query_scope_summary.trim().is_empty()
    {
        findings.invalid(
            "INVENTORY_REQUIRED_IDENTITY_MISSING",
            "manifest",
            "inventory, owner, source system, connector and query scope identifiers are required",
        );
    }
    for (field, value) in [
        (
            "connector_build_sha256",
            manifest.connector_build_sha256.as_str(),
        ),
        ("query_scope_sha256", manifest.query_scope_sha256.as_str()),
    ] {
        if !valid_sha256(value) {
            findings.invalid(
                "INVALID_INVENTORY_DIGEST",
                field,
                "manifest digest field must be lowercase SHA-256",
            );
        }
    }
    if !valid_utc_timestamp(&manifest.enumeration_started_at)
        || !valid_utc_timestamp(&manifest.enumeration_finished_at)
        || manifest.enumeration_started_at > manifest.enumeration_finished_at
    {
        findings.invalid(
            "INVALID_INVENTORY_ENUMERATION_INTERVAL",
            "manifest",
            "enumeration start and finish must be canonical UTC timestamps in ascending order",
        );
    }
    if manifest.limitations.is_empty() {
        findings.invalid(
            "INVENTORY_LIMITATIONS_MISSING",
            "manifest",
            "inventory manifest must state its bounded claim and limitations",
        );
    }
    if manifest.required_partitions.is_empty() {
        findings.invalid(
            "INVENTORY_REQUIRED_PARTITIONS_EMPTY",
            "manifest",
            "at least one required source partition must be declared",
        );
    }
    let required_partitions = manifest
        .required_partitions
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    if required_partitions.len() != manifest.required_partitions.len()
        || manifest
            .required_partitions
            .iter()
            .any(|partition| partition.trim().is_empty())
    {
        findings.invalid(
            "INVENTORY_PARTITION_SET_INVALID",
            "manifest",
            "required partition IDs must be non-empty and unique",
        );
    }

    match manifest.snapshot_semantics {
        SnapshotSemantics::ImmutableSnapshot => {
            if manifest.snapshot_token.as_deref().is_none_or(str::is_empty)
                || manifest
                    .snapshot_sha256
                    .as_deref()
                    .is_none_or(|value| !valid_sha256(value))
            {
                findings.incomplete(
                    "IMMUTABLE_SNAPSHOT_BINDING_MISSING",
                    "manifest",
                    "immutable snapshot semantics require a source snapshot token and snapshot digest",
                );
            }
        }
        SnapshotSemantics::WatermarkBound => {
            if manifest
                .source_watermark
                .as_deref()
                .is_none_or(str::is_empty)
            {
                findings.incomplete(
                    "SOURCE_WATERMARK_MISSING",
                    "manifest",
                    "watermark-bound enumeration requires a source watermark",
                );
            }
        }
        SnapshotSemantics::BestEffort => {
            findings.incomplete(
                "BEST_EFFORT_ENUMERATION_NOT_SNAPSHOT_COMPLETE",
                "manifest",
                "best-effort enumeration cannot establish a stable snapshot boundary",
            );
        }
        SnapshotSemantics::Unsupported => {
            findings.rows.insert(InventoryFinding {
                severity: InventoryFindingSeverity::Incomplete,
                code: "SOURCE_ENUMERATION_SEMANTICS_UNSUPPORTED".into(),
                subject: "manifest".into(),
                detail: "source does not expose comparable snapshot or watermark semantics".into(),
            });
        }
    }

    if !manifest.snapshot_consistent {
        findings.incomplete(
            "SOURCE_SNAPSHOT_CONSISTENCY_NOT_ESTABLISHED",
            "manifest",
            "source did not establish that all pages belong to one consistent snapshot",
        );
    }
    if manifest.truncation_detected {
        findings.incomplete(
            "SOURCE_ENUMERATION_TRUNCATED",
            "manifest",
            "source or connector reported an enumeration truncation",
        );
        if manifest.pagination_exhausted {
            findings.invalid(
                "TRUNCATED_INVENTORY_CLAIMS_PAGINATION_EXHAUSTED",
                "manifest",
                "a truncated enumeration cannot simultaneously claim authoritative pagination exhaustion",
            );
        }
    }
    if !manifest.tombstones_included {
        findings.incomplete(
            "SOURCE_TOMBSTONES_NOT_INCLUDED",
            "manifest",
            "source enumeration does not establish visibility of tombstones/retractions",
        );
    }
    if manifest
        .attestation_ref
        .as_deref()
        .is_none_or(str::is_empty)
        || manifest.issuer_key_id.as_deref().is_none_or(str::is_empty)
    {
        findings.incomplete(
            "SOURCE_INVENTORY_ATTESTATION_REFERENCE_MISSING",
            "manifest",
            "source attestation and issuer key references are not both present",
        );
    }

    let mut grouped_pages = BTreeMap::<String, Vec<&InventoryPageReceipt>>::new();
    for page in &manifest.page_receipts {
        if !required_partitions.contains(&page.partition_id) {
            findings.invalid(
                "UNDECLARED_INVENTORY_PARTITION",
                &page.partition_id,
                "page references a partition not declared by the manifest",
            );
        }
        grouped_pages
            .entry(page.partition_id.clone())
            .or_default()
            .push(page);
    }
    for pages in grouped_pages.values_mut() {
        pages.sort_by_key(|page| page.ordinal);
    }

    let mut total_items = 0u64;
    let mut all_partition_chains_complete = true;
    let mut all_identity_digests = BTreeSet::new();
    let mut all_identity_digest_count = 0u64;
    let mut all_tombstone_identities = BTreeSet::new();
    for partition_id in &manifest.required_partitions {
        let pages = grouped_pages
            .get(partition_id)
            .map_or(&[][..], Vec::as_slice);
        let (partition_items, _partition_unique, chain_complete) = validate_page_partition(
            partition_id,
            pages,
            manifest,
            &mut findings,
            &mut all_tombstone_identities,
        );
        total_items = total_items.saturating_add(partition_items);
        all_partition_chains_complete &= chain_complete;
        for page in pages {
            let page_identity_count =
                u64::try_from(page.native_record_identity_sha256s.len()).unwrap_or(u64::MAX);
            match all_identity_digest_count.checked_add(page_identity_count) {
                Some(total) => all_identity_digest_count = total,
                None => findings.invalid(
                    "INVENTORY_IDENTITY_COUNT_OVERFLOW",
                    partition_id,
                    "summed page identity count exceeds u64",
                ),
            }
            all_identity_digests.extend(page.native_record_identity_sha256s.iter().cloned());
        }
    }
    if total_items != manifest.enumerated_item_count {
        findings.invalid(
            "INVENTORY_ENUMERATED_ITEM_COUNT_MISMATCH",
            "manifest",
            "sum of page item counts disagrees with manifest enumerated_item_count",
        );
    }
    let actual_unique_count = u64::try_from(all_identity_digests.len()).unwrap_or(u64::MAX);
    let actual_duplicate_count = all_identity_digest_count.saturating_sub(actual_unique_count);
    let declared_item_total = manifest
        .enumerated_unique_count
        .checked_add(manifest.duplicate_count);
    if declared_item_total.is_none()
        || actual_unique_count != manifest.enumerated_unique_count
        || actual_duplicate_count != manifest.duplicate_count
        || declared_item_total != Some(total_items)
    {
        findings.invalid(
            "INVENTORY_IDENTITY_COUNT_RECONCILIATION_FAILED",
            "manifest",
            "page identity digests do not reconcile to declared unique and duplicate record counts",
        );
    }
    if manifest.tombstone_count > manifest.enumerated_unique_count {
        findings.invalid(
            "INVENTORY_TOMBSTONE_COUNT_EXCEEDS_UNIQUE_RECORDS",
            "manifest",
            "tombstone count cannot exceed enumerated unique record count",
        );
    }
    if u64::try_from(all_tombstone_identities.len()).unwrap_or(u64::MAX) != manifest.tombstone_count
    {
        findings.invalid(
            "INVENTORY_TOMBSTONE_IDENTITY_COUNT_MISMATCH",
            "manifest",
            "unique page tombstone identity digests do not reconcile to tombstone_count",
        );
    }
    match (manifest.reported_total_count, manifest.reported_total_count_semantics) {
        (Some(source_total), Some(ReportedTotalSemantics::UniqueNativeRecords))
            if source_total != manifest.enumerated_unique_count =>
        {
            findings.invalid(
                "SOURCE_REPORTED_TOTAL_COUNT_MISMATCH",
                "manifest",
                "source-reported unique-native-record count disagrees with enumerated_unique_count",
            );
        }
        (Some(source_total), Some(ReportedTotalSemantics::RawPageItems))
            if source_total != manifest.enumerated_item_count =>
        {
            findings.invalid(
                "SOURCE_REPORTED_TOTAL_COUNT_MISMATCH",
                "manifest",
                "source-reported raw page-item count disagrees with enumerated_item_count",
            );
        }
        (Some(_), Some(_)) => {}
        (None, _) | (_, None) => findings.incomplete(
            "SOURCE_REPORTED_TOTAL_COUNT_SEMANTICS_MISSING",
            "manifest",
            "source total count and its unique-record/raw-page-item semantics must both be declared",
        ),
    }

    let exclusion_total = manifest.exclusions.iter().fold(0u64, |sum, exclusion| {
        sum.saturating_add(exclusion.item_count)
    });
    if exclusion_total != manifest.excluded_count {
        findings.invalid(
            "INVENTORY_EXCLUSION_COUNT_MISMATCH",
            "manifest",
            "exclusion reason counts do not reconcile to excluded_count",
        );
    }
    if manifest.exclusions.iter().any(|exclusion| {
        exclusion.exclusion_code.trim().is_empty()
            || exclusion.policy_ref.trim().is_empty()
            || exclusion.item_count == 0
    }) {
        findings.invalid(
            "INVENTORY_EXCLUSION_REASON_INVALID",
            "manifest",
            "each exclusion requires a non-empty reason, policy reference and positive item count",
        );
    }
    if manifest.excluded_count > 0 && manifest.exclusions.is_empty() {
        findings.invalid(
            "INVENTORY_EXCLUSIONS_UNEXPLAINED",
            "manifest",
            "excluded records require explicit policy-bound exclusion reasons",
        );
    }

    if !manifest.pagination_exhausted {
        findings.incomplete(
            "AUTHORITATIVE_PAGINATION_EXHAUSTION_NOT_CLAIMED",
            "manifest",
            "manifest does not claim the source's authoritative terminal pagination state",
        );
    } else if !all_partition_chains_complete {
        findings.invalid(
            "PAGINATION_EXHAUSTED_WITH_INCOMPLETE_PARTITION_CHAIN",
            "manifest",
            "manifest claims pagination exhaustion while at least one required partition lacks a complete terminal chain",
        );
    }

    let mut disposition = findings.disposition();
    if disposition == InventoryDisposition::CompleteWithinDeclaredQuery
        && manifest.excluded_count > 0
    {
        disposition = InventoryDisposition::CompleteWithDeclaredExclusions;
    }
    if manifest.snapshot_semantics == SnapshotSemantics::Unsupported
        && disposition != InventoryDisposition::InvalidInput
    {
        disposition = InventoryDisposition::NotComparable;
    }

    let manifest_sha256 = serde_json::to_vec(manifest)
        .map(|bytes| sha256_hex(&bytes))
        .unwrap_or_default();
    InventoryValidation {
        disposition,
        inventory_id: manifest.inventory_id.clone(),
        manifest_sha256,
        page_count: manifest.page_receipts.len(),
        required_partition_count: manifest.required_partitions.len(),
        enumerated_item_count: manifest.enumerated_item_count,
        enumerated_unique_count: manifest.enumerated_unique_count,
        duplicate_count: manifest.duplicate_count,
        excluded_count: manifest.excluded_count,
        tombstone_count: manifest.tombstone_count,
        page_payload_byte_verification: "not_performed".into(),
        source_attestation_authentication: "not_performed".into(),
        source_truth: "not_established".into(),
        claim_scope: "declared_source_query_and_snapshot_only".into(),
        findings: findings.rows(),
    }
}

/// Reconcile observed manifests against a caller-supplied expected owner set.
///
/// This does not prove that the expected owner set itself is complete. That
/// discovery obligation must come from an independently maintained source
/// register and appropriate owner attestations. This function only reconciles
/// owner/system pairs; callers must separately validate each manifest's
/// record-set completeness disposition.
pub fn validate_source_universe_coverage(
    expected_owners: &[ExpectedSourceOwner],
    manifests: &[SourceInventoryManifest],
) -> SourceUniverseValidation {
    let mut findings = Findings::default();
    if expected_owners.is_empty() {
        findings.invalid(
            "EXPECTED_SOURCE_OWNER_SET_EMPTY",
            "source_universe",
            "coverage validation requires a non-empty expected owner set",
        );
    }
    let mut expected_keys = BTreeSet::new();
    for owner in expected_owners {
        if owner.owner_namespace.trim().is_empty() || owner.source_system_id.trim().is_empty() {
            findings.invalid(
                "EXPECTED_SOURCE_OWNER_IDENTITY_INVALID",
                &owner.owner_namespace,
                "expected owner namespace and source system ID must be non-empty",
            );
        }
        if !expected_keys.insert((
            owner.owner_namespace.clone(),
            owner.source_system_id.clone(),
        )) {
            findings.invalid(
                "DUPLICATE_EXPECTED_SOURCE_OWNER",
                &owner.owner_namespace,
                "expected source owner/system pairs must be unique",
            );
        }
    }

    let mut observed_keys = BTreeSet::new();
    for manifest in manifests {
        let key = (
            manifest.source_owner_namespace.clone(),
            manifest.source_system_id.clone(),
        );
        if !observed_keys.insert(key.clone()) {
            findings.invalid(
                "DUPLICATE_SOURCE_INVENTORY_MANIFEST",
                format!("{}:{}", key.0, key.1),
                "only one manifest may represent each owner/system pair in one inventory closure",
            );
        }
        if !expected_keys.contains(&key) {
            findings.invalid(
                "UNEXPECTED_SOURCE_OWNER_IN_INVENTORY_SET",
                format!("{}:{}", key.0, key.1),
                "observed inventory manifest is not present in the caller-supplied expected owner set",
            );
        }
    }

    let mut missing_required_owner_count = 0usize;
    for owner in expected_owners.iter().filter(|owner| owner.required) {
        let key = (
            owner.owner_namespace.clone(),
            owner.source_system_id.clone(),
        );
        if !observed_keys.contains(&key) {
            missing_required_owner_count += 1;
            findings.incomplete(
                "REQUIRED_SOURCE_OWNER_NOT_ENUMERATED",
                format!("{}:{}", key.0, key.1),
                "required source owner has no inventory manifest in the declared closure",
            );
        }
    }
    let disposition = if findings
        .rows
        .iter()
        .any(|finding| finding.severity == InventoryFindingSeverity::Invalid)
    {
        SourceUniverseDisposition::InvalidInput
    } else if !findings.rows.is_empty() {
        SourceUniverseDisposition::IncompleteOrIndeterminate
    } else {
        SourceUniverseDisposition::DeclaredOwnerSetCovered
    };
    SourceUniverseValidation {
        disposition,
        expected_owner_count: expected_owners.len(),
        required_owner_count: expected_owners
            .iter()
            .filter(|owner| owner.required)
            .count(),
        observed_manifest_count: manifests.len(),
        missing_required_owner_count,
        claim_scope: "caller_supplied_expected_owner_set_only".into(),
        findings: findings.rows(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn digest(byte: char) -> String {
        std::iter::repeat_n(byte, 64).collect()
    }

    fn page(
        ordinal: u32,
        payload: char,
        identities: Vec<String>,
        cursor_before: Option<String>,
        cursor_after: Option<String>,
        predecessor: Option<String>,
        terminal: bool,
    ) -> InventoryPageReceipt {
        InventoryPageReceipt {
            inventory_id: "inventory-synthetic-001".into(),
            partition_id: "region-a".into(),
            ordinal,
            response_payload_sha256: digest(payload),
            item_count: identities.len() as u64,
            native_record_identity_sha256s: identities,
            tombstone_identity_sha256s: vec![],
            cursor_before_sha256: cursor_before,
            cursor_after_sha256: cursor_after,
            predecessor_page_receipt_sha256: predecessor,
            snapshot_sha256: Some(digest('s')),
            source_watermark: Some("watermark-100".into()),
            response_at: "2026-11-03T10:01:00Z".into(),
            terminal,
        }
    }

    fn digest_seed(seed: u64) -> String {
        format!("{seed:064x}")
    }

    fn valid_manifest() -> SourceInventoryManifest {
        let first = page(
            0,
            'a',
            vec![digest('1'), digest('2')],
            None,
            Some(digest('c')),
            None,
            false,
        );
        let first_sha = first.receipt_sha256().unwrap();
        let second = page(
            1,
            'b',
            vec![digest('3')],
            Some(digest('c')),
            None,
            Some(first_sha),
            true,
        );
        SourceInventoryManifest {
            schema_version: INVENTORY_SCHEMA.into(),
            inventory_id: "inventory-synthetic-001".into(),
            source_owner_namespace: "urn:synthetic:source-owner".into(),
            source_system_id: "urn:synthetic:registry".into(),
            connector_id: "synthetic-connector".into(),
            connector_build_sha256: digest('d'),
            query_scope_sha256: digest('e'),
            query_scope_summary: "all synthetic records in region-a".into(),
            snapshot_semantics: SnapshotSemantics::ImmutableSnapshot,
            snapshot_token: Some("snapshot-100".into()),
            snapshot_sha256: Some(digest('s')),
            source_watermark: Some("watermark-100".into()),
            required_partitions: vec!["region-a".into()],
            enumeration_started_at: "2026-11-03T10:00:00Z".into(),
            enumeration_finished_at: "2026-11-03T10:02:00Z".into(),
            page_receipts: vec![first, second],
            reported_total_count: Some(3),
            reported_total_count_semantics: Some(ReportedTotalSemantics::UniqueNativeRecords),
            enumerated_item_count: 3,
            enumerated_unique_count: 3,
            duplicate_count: 0,
            excluded_count: 0,
            exclusions: vec![],
            tombstone_count: 0,
            tombstones_included: true,
            pagination_exhausted: true,
            snapshot_consistent: true,
            truncation_detected: false,
            attestation_ref: Some("synthetic:attestation:001".into()),
            issuer_key_id: Some("urn:synthetic:key:001".into()),
            limitations: vec!["synthetic-only; authentication is not performed".into()],
        }
    }

    #[test]
    fn one_thousand_page_gap_mutations_never_claim_complete() {
        for seed in 0..1_000u64 {
            let mut manifest = valid_manifest();
            manifest.page_receipts[1].ordinal = u32::try_from(seed + 2).unwrap();
            let result = validate_source_inventory_manifest(&manifest);
            assert_eq!(
                result.disposition,
                InventoryDisposition::InvalidInput,
                "page-gap mutation seed {seed} must be invalid"
            );
        }
    }

    #[test]
    fn one_thousand_cursor_replay_mutations_never_claim_complete() {
        for seed in 0..1_000u64 {
            let mut manifest = valid_manifest();
            let repeated_cursor = digest_seed(seed + 1);
            manifest.page_receipts[0].cursor_after_sha256 = Some(repeated_cursor.clone());
            let first_receipt_sha256 = manifest.page_receipts[0].receipt_sha256().unwrap();
            manifest.page_receipts[1].cursor_before_sha256 = Some(repeated_cursor.clone());
            manifest.page_receipts[1].cursor_after_sha256 = Some(repeated_cursor);
            manifest.page_receipts[1].predecessor_page_receipt_sha256 = Some(first_receipt_sha256);
            manifest.page_receipts[1].terminal = false;
            let result = validate_source_inventory_manifest(&manifest);
            assert_eq!(
                result.disposition,
                InventoryDisposition::InvalidInput,
                "cursor-replay mutation seed {seed} must be invalid"
            );
            assert!(
                result
                    .findings
                    .iter()
                    .any(|finding| { finding.code == "INVENTORY_CURSOR_REPLAY" })
            );
        }
    }

    #[test]
    fn one_thousand_cross_snapshot_mutations_never_claim_complete() {
        for seed in 0..1_000u64 {
            let mut manifest = valid_manifest();
            manifest.page_receipts[1].snapshot_sha256 = Some(digest_seed(seed + 1));
            let result = validate_source_inventory_manifest(&manifest);
            assert_eq!(
                result.disposition,
                InventoryDisposition::InvalidInput,
                "cross-snapshot mutation seed {seed} must be invalid"
            );
        }
    }

    #[test]
    fn one_thousand_source_count_mismatch_mutations_never_claim_complete() {
        for seed in 0..1_000u64 {
            let mut manifest = valid_manifest();
            manifest.reported_total_count = Some(seed + 4);
            let result = validate_source_inventory_manifest(&manifest);
            assert_eq!(
                result.disposition,
                InventoryDisposition::InvalidInput,
                "source-count mutation seed {seed} must be invalid"
            );
            assert!(
                result
                    .findings
                    .iter()
                    .any(|finding| { finding.code == "SOURCE_REPORTED_TOTAL_COUNT_MISMATCH" })
            );
        }
    }

    #[test]
    fn frozen_synthetic_manifest_fixture_deserializes_and_validates() {
        let manifest: SourceInventoryManifest = serde_json::from_str(include_str!(
            "../../schemas/examples/federated-source-inventory-manifest-synthetic-v0.1.json"
        ))
        .unwrap();
        let result = validate_source_inventory_manifest(&manifest);
        assert_eq!(
            result.disposition,
            InventoryDisposition::CompleteWithinDeclaredQuery
        );
        assert_eq!(result.page_count, 2);
        assert_eq!(result.enumerated_unique_count, 3);
        assert_eq!(result.source_attestation_authentication, "not_performed");
    }

    #[test]
    fn complete_frozen_manifest_is_bounded_to_declared_query() {
        let result = validate_source_inventory_manifest(&valid_manifest());
        assert_eq!(
            result.disposition,
            InventoryDisposition::CompleteWithinDeclaredQuery
        );
        assert_eq!(result.enumerated_unique_count, 3);
        assert_eq!(result.page_payload_byte_verification, "not_performed");
        assert_eq!(result.source_attestation_authentication, "not_performed");
        assert_eq!(result.source_truth, "not_established");
        assert_eq!(
            result.claim_scope,
            "declared_source_query_and_snapshot_only"
        );
    }

    #[test]
    fn skipped_page_ordinal_is_invalid_not_incomplete_pass() {
        let mut manifest = valid_manifest();
        manifest.page_receipts[1].ordinal = 2;
        let result = validate_source_inventory_manifest(&manifest);
        assert_eq!(result.disposition, InventoryDisposition::InvalidInput);
        assert!(
            result
                .findings
                .iter()
                .any(|finding| { finding.code == "INVENTORY_PAGE_ORDINAL_GAP" })
        );
    }

    #[test]
    fn cursor_replay_is_invalid() {
        let mut manifest = valid_manifest();
        manifest.page_receipts[1].cursor_before_sha256 = Some(digest('c'));
        manifest.page_receipts[1].cursor_after_sha256 = Some(digest('c'));
        manifest.page_receipts[1].terminal = false;
        let result = validate_source_inventory_manifest(&manifest);
        assert_eq!(result.disposition, InventoryDisposition::InvalidInput);
        assert!(
            result
                .findings
                .iter()
                .any(|finding| { finding.code == "INVENTORY_CURSOR_REPLAY" })
        );
    }

    #[test]
    fn cross_snapshot_page_is_invalid() {
        let mut manifest = valid_manifest();
        manifest.page_receipts[1].snapshot_sha256 = Some(digest('f'));
        let result = validate_source_inventory_manifest(&manifest);
        assert_eq!(result.disposition, InventoryDisposition::InvalidInput);
        assert!(
            result
                .findings
                .iter()
                .any(|finding| { finding.code == "INVENTORY_PAGE_SNAPSHOT_MISMATCH" })
        );
    }

    #[test]
    fn duplicate_native_identity_is_counted_and_rejected_when_unreported() {
        let mut manifest = valid_manifest();
        manifest.page_receipts[1].native_record_identity_sha256s[0] = digest('2');
        let result = validate_source_inventory_manifest(&manifest);
        assert_eq!(result.disposition, InventoryDisposition::InvalidInput);
        assert!(
            result.findings.iter().any(|finding| {
                finding.code == "INVENTORY_IDENTITY_COUNT_RECONCILIATION_FAILED"
            })
        );
    }

    #[test]
    fn missing_terminal_page_is_incomplete_and_cannot_claim_exhaustion() {
        let mut manifest = valid_manifest();
        manifest.page_receipts.pop();
        let result = validate_source_inventory_manifest(&manifest);
        assert_eq!(result.disposition, InventoryDisposition::InvalidInput);
        assert!(result.findings.iter().any(|finding| {
            finding.code == "PAGINATION_EXHAUSTED_WITH_INCOMPLETE_PARTITION_CHAIN"
        }));
    }

    #[test]
    fn unexhausted_pagination_is_incomplete_even_if_pages_are_well_formed() {
        let mut manifest = valid_manifest();
        manifest.pagination_exhausted = false;
        let result = validate_source_inventory_manifest(&manifest);
        assert_eq!(
            result.disposition,
            InventoryDisposition::IncompleteOrIndeterminate
        );
        assert!(
            result.findings.iter().any(|finding| {
                finding.code == "AUTHORITATIVE_PAGINATION_EXHAUSTION_NOT_CLAIMED"
            })
        );
    }

    #[test]
    fn source_count_mismatch_is_invalid() {
        let mut manifest = valid_manifest();
        manifest.reported_total_count = Some(4);
        let result = validate_source_inventory_manifest(&manifest);
        assert_eq!(result.disposition, InventoryDisposition::InvalidInput);
        assert!(
            result
                .findings
                .iter()
                .any(|finding| { finding.code == "SOURCE_REPORTED_TOTAL_COUNT_MISMATCH" })
        );
    }

    #[test]
    fn explicit_policy_exclusions_have_separate_bounded_disposition() {
        let mut manifest = valid_manifest();
        manifest.excluded_count = 2;
        manifest.exclusions = vec![InventoryExclusion {
            exclusion_code: "restricted_record".into(),
            item_count: 2,
            policy_ref: "policy:restricted-records:v1".into(),
        }];
        let result = validate_source_inventory_manifest(&manifest);
        assert_eq!(
            result.disposition,
            InventoryDisposition::CompleteWithDeclaredExclusions
        );
    }

    #[test]
    fn best_effort_source_semantics_cannot_claim_complete_inventory() {
        let mut manifest = valid_manifest();
        manifest.snapshot_semantics = SnapshotSemantics::BestEffort;
        manifest.snapshot_token = None;
        manifest.snapshot_sha256 = None;
        for page in &mut manifest.page_receipts {
            page.snapshot_sha256 = None;
        }
        let first_sha = manifest.page_receipts[0].receipt_sha256().unwrap();
        manifest.page_receipts[1].predecessor_page_receipt_sha256 = Some(first_sha);
        let result = validate_source_inventory_manifest(&manifest);
        assert_eq!(
            result.disposition,
            InventoryDisposition::IncompleteOrIndeterminate
        );
        assert!(
            result
                .findings
                .iter()
                .any(|finding| { finding.code == "BEST_EFFORT_ENUMERATION_NOT_SNAPSHOT_COMPLETE" })
        );
    }

    #[test]
    fn unavailable_enumeration_semantics_are_not_comparable() {
        let mut manifest = valid_manifest();
        manifest.snapshot_semantics = SnapshotSemantics::Unsupported;
        let result = validate_source_inventory_manifest(&manifest);
        assert_eq!(result.disposition, InventoryDisposition::NotComparable);
    }

    #[test]
    fn missing_required_partition_is_incomplete() {
        let mut manifest = valid_manifest();
        manifest.required_partitions.push("region-b".into());
        let result = validate_source_inventory_manifest(&manifest);
        assert_eq!(
            result.disposition,
            InventoryDisposition::InvalidInput,
            "pagination_exhausted cannot coexist with a missing required partition"
        );
        assert!(
            result
                .findings
                .iter()
                .any(|finding| { finding.code == "INVENTORY_PARTITION_TERMINAL_NOT_OBSERVED" })
        );
    }

    #[test]
    fn expected_owner_set_coverage_is_not_source_universe_completeness() {
        let manifest = valid_manifest();
        let expected = vec![
            ExpectedSourceOwner {
                owner_namespace: manifest.source_owner_namespace.clone(),
                source_system_id: manifest.source_system_id.clone(),
                required: true,
            },
            ExpectedSourceOwner {
                owner_namespace: "urn:synthetic:other-owner".into(),
                source_system_id: "urn:synthetic:other-system".into(),
                required: true,
            },
        ];
        let result = validate_source_universe_coverage(&expected, &[manifest]);
        assert_eq!(
            result.disposition,
            SourceUniverseDisposition::IncompleteOrIndeterminate
        );
        assert_eq!(result.missing_required_owner_count, 1);
        assert_eq!(
            result.claim_scope,
            "caller_supplied_expected_owner_set_only"
        );
    }

    #[test]
    fn duplicate_expected_owner_is_invalid() {
        let owner = ExpectedSourceOwner {
            owner_namespace: "urn:synthetic:owner".into(),
            source_system_id: "urn:synthetic:system".into(),
            required: true,
        };
        let result = validate_source_universe_coverage(&[owner.clone(), owner], &[]);
        assert_eq!(result.disposition, SourceUniverseDisposition::InvalidInput);
        assert!(
            result
                .findings
                .iter()
                .any(|finding| { finding.code == "DUPLICATE_EXPECTED_SOURCE_OWNER" })
        );
    }
}
