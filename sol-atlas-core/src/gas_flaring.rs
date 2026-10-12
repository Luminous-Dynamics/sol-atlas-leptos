// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Evidence-bounded admission for satellite-derived gas-flaring estimates.
//!
//! This module deliberately stops before economic or engineering qualification.
//! A flare-volume estimate is not recoverable gas, measured methane leakage,
//! verified emissions reduction, production capacity, or project profitability.
//! Data may be admitted only with a pinned source snapshot, attribution, and an
//! explicit reuse review. Consumers must preserve the claim ceiling.
//!
//! The current World Bank source is an XLSX workbook. This module accepts a
//! *reviewed, normalized JSON representation*; it does not guess the workbook's
//! live column mapping. An importer should be added only after the downloaded
//! workbook's sheet names, columns, units, terms, and snapshot digest are reviewed.

use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::fmt;

/// Current World Bank Global Gas Flaring Tracker individual-site dataset page.
pub const WORLD_BANK_FLARING_DATA_URL: &str =
    "https://www.worldbank.org/en/programs/gasflaringreduction/global-flaring-data";

/// Explicitly bounded statement about what a site record represents.
pub const CLAIM_CEILING: &str = "Satellite-derived flaring-volume estimate for screening only; not recoverability, methane leakage, verified emissions reduction, engineering viability, or financial return.";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum ReuseReview {
    /// The source's terms have not been reviewed for the intended use.
    Unreviewed,
    /// A human has reviewed the source terms for the stated intended use.
    Approved {
        reviewer: String,
        reviewed_at: String,
        terms_uri: String,
        approved_purpose: String,
    },
    /// Review found that the intended use is not approved.
    Denied {
        reviewer: String,
        reviewed_at: String,
        terms_uri: String,
        reason: String,
    },
}

/// How the adapter derives stable site identity. The reviewed method is part of
/// provenance so consumers do not mistake approximate spatial matching for a
/// source-provided identifier.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub enum SiteIdentityMethod {
    /// Use a stable identifier that is explicitly present in the source workbook.
    SourceField { field_name: String },
    /// SHA-256 of a canonical encoding of the listed exact source fields.
    ///
    /// Field names must be sorted and unique. The adapter must not round
    /// coordinates or use workbook row numbers in this fingerprint. A changed
    /// source field value can change identity; this is not a fuzzy entity-resolution
    /// claim.
    DerivedFingerprint {
        algorithm_version: String,
        source_fields: Vec<String>,
    },
}

/// Provenance bound to the exact source snapshot used to create these records.
///
/// `snapshot_sha256` must be computed over the exact downloaded source bytes
/// by the acquisition step. This crate validates its shape; it does not pretend
/// that a syntactically valid digest proves the downloaded bytes match it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FlareSourceProvenance {
    pub source_name: String,
    pub source_uri: String,
    pub source_version: String,
    pub site_identity_method: SiteIdentityMethod,
    pub snapshot_sha256: String,
    pub published_at: Option<String>,
    pub retrieved_at: String,
    pub reference_year_start: u16,
    pub reference_year_end: u16,
    pub attribution: String,
    pub reuse_review: ReuseReview,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GeoPoint {
    pub latitude_deg: f64,
    pub longitude_deg: f64,
}

/// Why a value is absent. Absence must not be converted to a numeric zero.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MissingValueReason {
    SourceBlank,
    NotPublished,
    NotApplicable,
    Unknown,
}

/// A source-reported annual volume estimate or explicit missingness.
///
/// Units are fixed to million cubic metres per year (million m³/year), matching
/// the World Bank page's individual flare-site visualization. This value remains
/// a satellite-derived estimate; unit clarity does not elevate its evidence class.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case", deny_unknown_fields)]
pub enum AnnualFlareValue {
    Estimated {
        volume_million_m3_per_year: f64,
    },
    Missing {
        reason: MissingValueReason,
    },
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnnualFlareEstimate {
    pub year: u16,
    pub value: AnnualFlareValue,
}

/// One normalized flare-site candidate. Its stable key must follow the
/// provenance-declared source-field or versioned-fingerprint method.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FlareSite {
    pub site_key: String,
    pub name: Option<String>,
    pub country: Option<String>,
    /// May be absent or stale in the source. No map marker is allowed without it.
    pub location: Option<GeoPoint>,
    pub annual_estimates: Vec<AnnualFlareEstimate>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FlareSnapshot {
    pub schema_version: String,
    pub provenance: FlareSourceProvenance,
    pub sites: Vec<FlareSite>,
}

/// Only the admission function can create this wrapper; it cannot be deserialized
/// from untrusted JSON or mutated by downstream callers.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ValidatedFlareSnapshot {
    snapshot: FlareSnapshot,
    claim_ceiling: &'static str,
}

impl ValidatedFlareSnapshot {
    /// Access the immutable snapshot that passed admission checks.
    pub fn snapshot(&self) -> &FlareSnapshot {
        &self.snapshot
    }

    /// Explicitly state the strongest claim this admitted data supports.
    pub fn claim_ceiling(&self) -> &'static str {
        self.claim_ceiling
    }
}

/// A structured, stable validation finding suitable for test assertions and receipts.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct ValidationIssue {
    pub code: String,
    pub path: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AdmissionError {
    InvalidJson(String),
    Rejected(Vec<ValidationIssue>),
}

impl fmt::Display for AdmissionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidJson(message) => write!(f, "invalid flare snapshot JSON: {message}"),
            Self::Rejected(issues) => {
                write!(f, "flare snapshot rejected with {} issue(s)", issues.len())
            }
        }
    }
}

impl std::error::Error for AdmissionError {}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FlareCoverageSummary {
    pub site_count: usize,
    pub geolocated_site_count: usize,
    pub expected_site_year_pairs: usize,
    pub represented_site_year_pairs: usize,
    pub omitted_site_year_pairs: usize,
    pub estimated_value_count: usize,
    pub explicit_missing_value_count: usize,
    pub reported_zero_count: usize,
    pub represented_years: Vec<u16>,
}

/// Parse and admit a normalized JSON source snapshot.
///
/// The caller must compute `snapshot_sha256` over the exact downloaded XLSX
/// bytes, map the reviewed worksheet columns explicitly, and attach the source
/// attribution and reuse decision. Any admission issue rejects the whole snapshot.
pub fn parse_json_snapshot(json: &str) -> Result<ValidatedFlareSnapshot, AdmissionError> {
    let snapshot: FlareSnapshot =
        serde_json::from_str(json).map_err(|error| AdmissionError::InvalidJson(error.to_string()))?;
    admit_snapshot(snapshot).map_err(AdmissionError::Rejected)
}

/// Fail-closed validation followed by deterministic ordering.
///
/// Duplicate site IDs and duplicate year observations are rejected rather than
/// silently deduplicated or chosen by input order. Sites and annual rows are
/// sorted only after the input passes validation.
pub fn admit_snapshot(
    mut snapshot: FlareSnapshot,
) -> Result<ValidatedFlareSnapshot, Vec<ValidationIssue>> {
    let mut issues = Vec::new();
    validate_provenance(&snapshot.provenance, &mut issues);

    if snapshot.schema_version.trim().is_empty() {
        issues.push(issue(
            "missing_schema_version",
            "schema_version",
            "schema version must be explicit",
        ));
    }

    if snapshot.sites.is_empty() {
        issues.push(issue(
            "empty_snapshot",
            "sites",
            "an empty site list is not evidence of zero flaring",
        ));
    }

    let mut seen_site_keys = BTreeSet::new();
    for (site_index, site) in snapshot.sites.iter().enumerate() {
        let base = format!("sites[{site_index}]");
        if site.site_key.trim().is_empty() {
            issues.push(issue(
                "missing_site_identity",
                format!("{base}.site_key"),
                "a stable site key emitted by the reviewed source adapter is required",
            ));
        } else if site.site_key.trim() != site.site_key {
            issues.push(issue(
                "noncanonical_site_identity",
                format!("{base}.site_key"),
                "site key must not contain leading or trailing whitespace",
            ));
        } else if matches!(
            &snapshot.provenance.site_identity_method,
            SiteIdentityMethod::DerivedFingerprint { .. }
        ) && !is_sha256_hex(&site.site_key) {
            issues.push(issue(
                "invalid_derived_site_fingerprint",
                format!("{base}.site_key"),
                "derived site keys must be SHA-256 hex digests of canonical source fields",
            ));
        } else if !seen_site_keys.insert(site.site_key.clone()) {
            issues.push(issue(
                "duplicate_site_identity",
                format!("{base}.site_key"),
                "duplicate site keys are ambiguous and must be reconciled upstream",
            ));
        }

        if matches!(site.name.as_deref(), Some(name) if name.trim().is_empty()) {
            issues.push(issue(
                "blank_site_name",
                format!("{base}.name"),
                "use None for absent names rather than an empty string",
            ));
        }
        if matches!(site.country.as_deref(), Some(country) if country.trim().is_empty()) {
            issues.push(issue(
                "blank_country",
                format!("{base}.country"),
                "use None for absent country metadata rather than an empty string",
            ));
        }

        if let Some(location) = &site.location {
            if !location.latitude_deg.is_finite()
                || !(-90.0..=90.0).contains(&location.latitude_deg)
            {
                issues.push(issue(
                    "invalid_latitude",
                    format!("{base}.location.latitude_deg"),
                    "latitude must be finite and within [-90, 90]",
                ));
            }
            if !location.longitude_deg.is_finite()
                || !(-180.0..=180.0).contains(&location.longitude_deg)
            {
                issues.push(issue(
                    "invalid_longitude",
                    format!("{base}.location.longitude_deg"),
                    "longitude must be finite and within [-180, 180]",
                ));
            }
        }

        if site.annual_estimates.is_empty() {
            issues.push(issue(
                "missing_annual_rows",
                format!("{base}.annual_estimates"),
                "missing years must be explicit rows or explicitly declared absent by the adapter",
            ));
        }

        let mut seen_years = BTreeSet::new();
        for (row_index, estimate) in site.annual_estimates.iter().enumerate() {
            let row_path = format!("{base}.annual_estimates[{row_index}]");
            if estimate.year < snapshot.provenance.reference_year_start
                || estimate.year > snapshot.provenance.reference_year_end
            {
                issues.push(issue(
                    "year_outside_snapshot_period",
                    format!("{row_path}.year"),
                    "observation year must be inside the source snapshot's declared period",
                ));
            }
            if !seen_years.insert(estimate.year) {
                issues.push(issue(
                    "duplicate_site_year",
                    format!("{row_path}.year"),
                    "a site may have at most one value per year; resolve conflicts upstream",
                ));
            }
            if let AnnualFlareValue::Estimated {
                volume_million_m3_per_year,
            } = &estimate.value
            {
                if !volume_million_m3_per_year.is_finite()
                    || *volume_million_m3_per_year < 0.0
                {
                    issues.push(issue(
                        "invalid_flare_volume",
                        format!("{row_path}.value.volume_million_m3_per_year"),
                        "estimated flare volume must be finite and non-negative",
                    ));
                }
            }
        }
    }

    issues.sort();
    if !issues.is_empty() {
        return Err(issues);
    }

    snapshot.sites.sort_by(|left, right| left.site_key.cmp(&right.site_key));
    for site in &mut snapshot.sites {
        site.annual_estimates.sort_by_key(|estimate| estimate.year);
    }
    Ok(ValidatedFlareSnapshot {
        snapshot,
        claim_ceiling: CLAIM_CEILING,
    })
}

/// Summarize data coverage without calculating gas totals or economic potential.
///
/// Expected cells are the complete Cartesian product of admitted sites and the
/// declared period. An absent row is counted as omitted, never turned into zero.
pub fn coverage_summary(admitted: &ValidatedFlareSnapshot) -> FlareCoverageSummary {
    let snapshot = &admitted.snapshot;
    let years_in_period = usize::from(
        snapshot.provenance.reference_year_end - snapshot.provenance.reference_year_start,
    ) + 1;
    let expected_site_year_pairs = snapshot.sites.len().saturating_mul(years_in_period);
    let represented_site_year_pairs: usize = snapshot
        .sites
        .iter()
        .map(|site| site.annual_estimates.len())
        .sum();
    let estimated_value_count = snapshot
        .sites
        .iter()
        .flat_map(|site| &site.annual_estimates)
        .filter(|row| matches!(&row.value, AnnualFlareValue::Estimated { .. }))
        .count();
    let explicit_missing_value_count = snapshot
        .sites
        .iter()
        .flat_map(|site| &site.annual_estimates)
        .filter(|row| matches!(&row.value, AnnualFlareValue::Missing { .. }))
        .count();
    let reported_zero_count = snapshot
        .sites
        .iter()
        .flat_map(|site| &site.annual_estimates)
        .filter(|row| {
            matches!(
                &row.value,
                AnnualFlareValue::Estimated {
                    volume_million_m3_per_year: value
                } if *value == 0.0
            )
        })
        .count();
    let years: BTreeSet<u16> = snapshot
        .sites
        .iter()
        .flat_map(|site| site.annual_estimates.iter().map(|row| row.year))
        .collect();

    FlareCoverageSummary {
        site_count: snapshot.sites.len(),
        geolocated_site_count: snapshot
            .sites
            .iter()
            .filter(|site| site.location.is_some())
            .count(),
        expected_site_year_pairs,
        represented_site_year_pairs,
        omitted_site_year_pairs: expected_site_year_pairs.saturating_sub(represented_site_year_pairs),
        estimated_value_count,
        explicit_missing_value_count,
        reported_zero_count,
        represented_years: years.into_iter().collect(),
    }
}

fn validate_provenance(provenance: &FlareSourceProvenance, issues: &mut Vec<ValidationIssue>) {
    for (field, value) in [
        ("source_name", provenance.source_name.as_str()),
        ("source_version", provenance.source_version.as_str()),
        ("retrieved_at", provenance.retrieved_at.as_str()),
        ("attribution", provenance.attribution.as_str()),
    ] {
        if value.trim().is_empty() {
            issues.push(issue(
                "missing_source_metadata",
                format!("provenance.{field}"),
                "source metadata must be explicit",
            ));
        }
    }

    if !provenance.source_uri.starts_with("https://") {
        issues.push(issue(
            "invalid_source_uri",
            "provenance.source_uri",
            "source URI must be HTTPS",
        ));
    }
    if provenance.reference_year_start > provenance.reference_year_end {
        issues.push(issue(
            "invalid_reference_period",
            "provenance.reference_year_start",
            "reference period start must not exceed its end",
        ));
    }
    if !is_sha256_hex(&provenance.snapshot_sha256) {
        issues.push(issue(
            "invalid_snapshot_digest",
            "provenance.snapshot_sha256",
            "expected 64 hexadecimal characters for SHA-256 of the exact source bytes",
        ));
    }

    match &provenance.site_identity_method {
        SiteIdentityMethod::SourceField { field_name } => {
            if field_name.trim().is_empty() {
                issues.push(issue(
                    "missing_identity_source_field",
                    "provenance.site_identity_method.field_name",
                    "source-field identity requires the exact reviewed workbook column name",
                ));
            }
        }
        SiteIdentityMethod::DerivedFingerprint {
            algorithm_version,
            source_fields,
        } => {
            if algorithm_version.trim().is_empty() {
                issues.push(issue(
                    "missing_identity_algorithm_version",
                    "provenance.site_identity_method.algorithm_version",
                    "derived identity requires a versioned algorithm",
                ));
            }
            if source_fields.is_empty()
                || source_fields.iter().any(|field| field.trim().is_empty())
            {
                issues.push(issue(
                    "missing_identity_source_fields",
                    "provenance.site_identity_method.source_fields",
                    "derived identity requires non-empty canonical source-field names",
                ));
            }
            if source_fields
                .windows(2)
                .any(|pair| pair[0].as_str() >= pair[1].as_str())
            {
                issues.push(issue(
                    "noncanonical_identity_source_fields",
                    "provenance.site_identity_method.source_fields",
                    "source field names must be sorted and unique for deterministic fingerprinting",
                ));
            }
        }
    }

    match &provenance.reuse_review {
        ReuseReview::Unreviewed => issues.push(issue(
            "reuse_terms_unreviewed",
            "provenance.reuse_review",
            "do not admit or redistribute source data before intended-use terms are reviewed",
        )),
        ReuseReview::Denied {
            reviewer,
            reviewed_at,
            terms_uri,
            reason,
        } => {
            for (field, value) in [
                ("reviewer", reviewer.as_str()),
                ("reviewed_at", reviewed_at.as_str()),
                ("terms_uri", terms_uri.as_str()),
                ("reason", reason.as_str()),
            ] {
                if value.trim().is_empty() {
                    issues.push(issue(
                        "incomplete_reuse_denial",
                        format!("provenance.reuse_review.{field}"),
                        "a denied reuse decision must include its audit metadata",
                    ));
                }
            }
            issues.push(issue(
                "reuse_not_approved",
                "provenance.reuse_review",
                "the source terms do not approve the intended use",
            ));
        }
        ReuseReview::Approved {
            reviewer,
            reviewed_at,
            terms_uri,
            approved_purpose,
        } => {
            for (field, value) in [
                ("reviewer", reviewer.as_str()),
                ("reviewed_at", reviewed_at.as_str()),
                ("terms_uri", terms_uri.as_str()),
                ("approved_purpose", approved_purpose.as_str()),
            ] {
                if value.trim().is_empty() {
                    issues.push(issue(
                        "incomplete_reuse_approval",
                        format!("provenance.reuse_review.{field}"),
                        "reuse approval must include its reviewer, date, terms, and scoped purpose",
                    ));
                }
            }
            if !terms_uri.starts_with("https://") {
                issues.push(issue(
                    "invalid_terms_uri",
                    "provenance.reuse_review.terms_uri",
                    "terms URI must be HTTPS",
                ));
            }
        }
    }
}

fn is_sha256_hex(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit())
}

fn issue(
    code: impl Into<String>,
    path: impl Into<String>,
    message: impl Into<String>,
) -> ValidationIssue {
    ValidationIssue {
        code: code.into(),
        path: path.into(),
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn provenance() -> FlareSourceProvenance {
        FlareSourceProvenance {
            source_name: "Synthetic flare fixture (not real World Bank data)".into(),
            source_uri: "https://example.invalid/synthetic-flare-fixture".into(),
            source_version: "synthetic-fixture-v1".into(),
            site_identity_method: SiteIdentityMethod::SourceField {
                field_name: "Synthetic Site ID".into(),
            },
            snapshot_sha256: "a".repeat(64),
            published_at: None,
            retrieved_at: "not-applicable-synthetic-fixture".into(),
            reference_year_start: 2024,
            reference_year_end: 2025,
            attribution: "Synthetic unit-test data only; no real World Bank source values included".into(),
            reuse_review: ReuseReview::Approved {
                reviewer: "test-fixture-review".into(),
                reviewed_at: "2026-10-12".into(),
                terms_uri: "https://example.invalid/synthetic-test-terms".into(),
                approved_purpose: "Synthetic unit-test fixture only; source workbook redistribution not implied".into(),
            },
        }
    }

    fn site(id: &str, estimates: Vec<AnnualFlareEstimate>) -> FlareSite {
        FlareSite {
            site_key: id.into(),
            name: Some(format!("Fixture {id}")),
            country: Some("Example".into()),
            location: Some(GeoPoint {
                latitude_deg: 0.0,
                longitude_deg: 0.0,
            }),
            annual_estimates: estimates,
        }
    }

    fn estimate(year: u16, value: AnnualFlareValue) -> AnnualFlareEstimate {
        AnnualFlareEstimate { year, value }
    }

    fn sample_snapshot() -> FlareSnapshot {
        FlareSnapshot {
            schema_version: "gas-flaring-screening-v1".into(),
            provenance: provenance(),
            sites: vec![
                site(
                    "site-b",
                    vec![
                        estimate(
                            2025,
                            AnnualFlareValue::Estimated {
                                volume_million_m3_per_year: 0.0,
                            },
                        ),
                        estimate(
                            2024,
                            AnnualFlareValue::Missing {
                                reason: MissingValueReason::SourceBlank,
                            },
                        ),
                    ],
                ),
                site(
                    "site-a",
                    vec![estimate(
                        2024,
                        AnnualFlareValue::Estimated {
                            volume_million_m3_per_year: 2.5,
                        },
                    )],
                ),
            ],
        }
    }

    #[test]
    fn admission_orders_sites_and_years_deterministically() {
        let validated = admit_snapshot(sample_snapshot()).expect("sample should be admitted");
        assert!(validated.claim_ceiling().contains("screening only"));
        assert_eq!(validated.snapshot.sites[0].site_key, "site-a");
        assert_eq!(validated.snapshot.sites[1].site_key, "site-b");
        assert_eq!(validated.snapshot.sites[1].annual_estimates[0].year, 2024);
        assert_eq!(validated.snapshot.sites[1].annual_estimates[1].year, 2025);

        let mut shuffled = sample_snapshot();
        shuffled.sites.reverse();
        for site in &mut shuffled.sites {
            site.annual_estimates.reverse();
        }
        assert_eq!(validated, admit_snapshot(shuffled).expect("shuffled sample"));
    }

    #[test]
    fn missing_and_zero_remain_distinct() {
        let admitted = admit_snapshot(sample_snapshot()).expect("sample should be admitted");
        let summary = coverage_summary(&admitted);
        assert_eq!(summary.site_count, 2);
        assert_eq!(summary.expected_site_year_pairs, 4);
        assert_eq!(summary.represented_site_year_pairs, 3);
        assert_eq!(summary.omitted_site_year_pairs, 1);
        assert_eq!(summary.estimated_value_count, 2);
        assert_eq!(summary.explicit_missing_value_count, 1);
        assert_eq!(summary.reported_zero_count, 1);
        assert_eq!(summary.represented_years, vec![2024, 2025]);
    }

    #[test]
    fn duplicate_site_ids_reject_whole_snapshot() {
        let mut snapshot = sample_snapshot();
        let duplicate = snapshot.sites[0].clone();
        snapshot.sites.push(duplicate);
        let issues = admit_snapshot(snapshot).expect_err("duplicate must fail closed");
        assert!(issues.iter().any(|item| item.code == "duplicate_site_identity"));
    }

    #[test]
    fn duplicate_site_year_rejects_whole_snapshot() {
        let mut snapshot = sample_snapshot();
        let duplicate_year = snapshot.sites[0].annual_estimates[0].clone();
        snapshot.sites[0].annual_estimates.push(duplicate_year);
        let issues = admit_snapshot(snapshot).expect_err("duplicate year must fail closed");
        assert!(issues.iter().any(|item| item.code == "duplicate_site_year"));
    }

    #[test]
    fn invalid_coordinates_and_negative_or_nonfinite_values_reject_snapshot() {
        let mut snapshot = sample_snapshot();
        snapshot.sites[0].location.as_mut().unwrap().latitude_deg = 91.0;
        snapshot.sites[1].annual_estimates[0].value = AnnualFlareValue::Estimated {
            volume_million_m3_per_year: f64::NAN,
        };
        let issues = admit_snapshot(snapshot).expect_err("invalid values must fail closed");
        assert!(issues.iter().any(|item| item.code == "invalid_latitude"));
        assert!(issues.iter().any(|item| item.code == "invalid_flare_volume"));
    }

    #[test]
    fn unreviewed_terms_and_malformed_digest_reject_snapshot() {
        let mut snapshot = sample_snapshot();
        snapshot.provenance.snapshot_sha256 = "not-a-digest".into();
        snapshot.provenance.reuse_review = ReuseReview::Unreviewed;
        let issues = admit_snapshot(snapshot).expect_err("unreviewed source must fail closed");
        assert!(issues.iter().any(|item| item.code == "invalid_snapshot_digest"));
        assert!(issues.iter().any(|item| item.code == "reuse_terms_unreviewed"));
    }

    #[test]
    fn no_geolocation_is_retained_but_not_counted_as_mappable() {
        let mut snapshot = sample_snapshot();
        snapshot.sites[0].location = None;
        let admitted = admit_snapshot(snapshot).expect("location may be unknown");
        let summary = coverage_summary(&admitted);
        assert_eq!(summary.site_count, 2);
        assert_eq!(summary.geolocated_site_count, 1);
    }

    #[test]
    fn derived_identity_requires_a_versioned_canonical_fingerprint() {
        let mut snapshot = sample_snapshot();
        snapshot.provenance.site_identity_method = SiteIdentityMethod::DerivedFingerprint {
            algorithm_version: "sha256-canonical-json-v1".into(),
            source_fields: vec![
                "Country".into(),
                "Field Name".into(),
                "Latitude".into(),
                "Longitude".into(),
            ],
        };
        snapshot.sites[0].site_key = "b".repeat(64);
        snapshot.sites[1].site_key = "a".repeat(64);
        let admitted = admit_snapshot(snapshot.clone()).expect("canonical fingerprints pass");
        assert_eq!(admitted.snapshot.sites[0].site_key, "a".repeat(64));

        snapshot.sites[0].site_key = "not-a-fingerprint".into();
        let issues = admit_snapshot(snapshot).expect_err("non-digest derived key rejects");
        assert!(issues.iter().any(|item| item.code == "invalid_derived_site_fingerprint"));
    }

    #[test]
    fn json_parser_requires_the_same_admission_checks() {
        let json = serde_json::to_string(&sample_snapshot()).unwrap();
        let admitted = parse_json_snapshot(&json).expect("valid normalized JSON");
        assert_eq!(admitted.snapshot.schema_version, "gas-flaring-screening-v1");

        let mut invalid = sample_snapshot();
        invalid.provenance.source_uri = "http://example.invalid".into();
        let invalid_json = serde_json::to_string(&invalid).unwrap();
        let error = parse_json_snapshot(&invalid_json).expect_err("HTTP sources must reject");
        match error {
            AdmissionError::Rejected(issues) => {
                assert!(issues.iter().any(|item| item.code == "invalid_source_uri"));
            }
            other => panic!("unexpected error: {other}"),
        }

        let mut drifted = serde_json::to_value(sample_snapshot()).unwrap();
        drifted["surprise_field"] = serde_json::json!("must not be ignored");
        let drifted_json = serde_json::to_string(&drifted).unwrap();
        assert!(matches!(
            parse_json_snapshot(&drifted_json),
            Err(AdmissionError::InvalidJson(_))
        ));

        let mut enum_drift = serde_json::to_value(sample_snapshot()).unwrap();
        enum_drift["sites"][0]["annual_estimates"][0]["value"]["extra_field"] =
            serde_json::json!("must not be ignored");
        let enum_drift_json = serde_json::to_string(&enum_drift).unwrap();
        assert!(matches!(
            parse_json_snapshot(&enum_drift_json),
            Err(AdmissionError::InvalidJson(_))
        ));
    }
}
