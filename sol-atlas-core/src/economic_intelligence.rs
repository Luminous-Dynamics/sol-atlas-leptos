// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

//! Evidence-bound, descriptive trade-mirror comparisons.
//!
//! This module deliberately computes only a raw comparison of two reported
//! trade values. It does not estimate illicit proceeds, classify a transaction
//! as criminal, or reproduce any institution's full trade-misinvoicing method.
//! A non-zero gap is an investigative/statistical signal, not proof of intent.
//!
//! Inputs are candidate records selected by a caller under an explicit source
//! and matching policy. Exactly one exporter report and one reciprocal importer
//! report must be supplied; duplicates are surfaced as ambiguity, never
//! silently de-duplicated. The caller remains responsible for source licensing,
//! snapshot retention, and a method-specific pairing policy.

use serde::{Deserialize, Serialize};

/// Direction of the trade flow from the reporting country's perspective.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TradeFlow {
    /// Reporter records goods as exports to its partner.
    Export,
    /// Reporter records goods as imports from its partner.
    Import,
}

/// A normalized shell around one source-reported trade observation.
///
/// Preserve source-native identifiers and values alongside normalized fields.
/// `period` is an opaque, source-defined period label (for example, `2025`);
/// do not infer calendar semantics from its spelling alone. The raw comparison
/// requires equality of period, commodity code, classification revision and
/// currency. Quantities are carried for future, explicitly qualified methods,
/// but are not used to adjust the value gap calculated here.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TradeObservation {
    /// Stable identifier assigned by the ingestion layer.
    pub record_id: String,
    /// Stable source-registry identifier, not just a display name.
    pub source_id: String,
    /// Immutable identity of the source snapshot from which this record came.
    pub source_snapshot_id: String,
    /// Source-native row/observation identifier, if available.
    pub source_record_id: Option<String>,
    /// Canonical source record or dataset URL when redistribution policy permits.
    pub source_url: Option<String>,
    /// Reporter of this observation.
    pub reporter: String,
    /// Partner named by the reporter.
    pub partner: String,
    /// Source-defined reference period.
    pub period: String,
    /// Commodity code as supplied/normalized under the source contract.
    pub commodity_code: String,
    /// Classification system and revision (for example, HS revision).
    pub classification_revision: String,
    /// Direction from the reporter's perspective.
    pub flow: TradeFlow,
    /// Source-native numeric token for the reported trade value.
    ///
    /// This is the calculation input; never reconstruct it from a binary float.
    /// The ingestion adapter must retain the source token or identify any
    /// upstream canonicalization. Plain base-10 decimal notation only:
    /// no exponent notation, separators, or surrounding whitespace.
    pub trade_value_source_text: Option<String>,
    /// Currency identifier for the source-reported value; no implicit conversion.
    pub currency: String,
    /// Optional source-reported quantity; not used in the raw value comparison.
    pub quantity: Option<f64>,
    /// Optional source-native quantity unit; preserve rather than guess.
    pub quantity_unit: Option<String>,
    /// Source publication/revision time when supplied.
    pub source_published_at: Option<String>,
    /// Retrieval/known-at time recorded by the ingestion layer.
    pub retrieved_at: String,
}

/// Why a candidate comparison did or did not produce a raw value difference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MirrorPairStatus {
    /// Exactly one reciprocal candidate exists on each side and values qualify.
    Compared,
    /// Neither side supplied a candidate record.
    NoCandidateReports,
    /// No exporter-side record was supplied.
    MissingExporterReport,
    /// No importer-side record was supplied.
    MissingImporterReport,
    /// More than one exporter-side candidate exists; selection is ambiguous.
    AmbiguousExporterReports,
    /// More than one importer-side candidate exists; selection is ambiguous.
    AmbiguousImporterReports,
    /// More than one candidate exists on both sides.
    AmbiguousReportsBothSides,
    /// Required identity/provenance or matching metadata is empty.
    InvalidRecordIdentity,
    /// Candidate records do not have export/import flow directions as expected.
    InvalidFlowDirection,
    /// Reporter and partner identifiers are not reciprocal.
    NonReciprocalReporters,
    /// The source-defined periods differ.
    PeriodMismatch,
    /// Commodity codes differ.
    CommodityMismatch,
    /// Commodity classification systems/revisions differ.
    ClassificationMismatch,
    /// Currency identifiers differ; no implicit conversion is applied.
    CurrencyMismatch,
    /// Exporter-side value is missing.
    MissingExporterValue,
    /// Importer-side value is missing.
    MissingImporterValue,
    /// Exporter-side source value is not a supported plain decimal.
    InvalidExporterValueFormat,
    /// Importer-side source value is not a supported plain decimal.
    InvalidImporterValueFormat,
    /// Decimal rescaling or subtraction exceeded the exact arithmetic range.
    ComparisonArithmeticOverflow,
    /// Exporter-side value is zero or negative and is excluded from this method.
    NonPositiveExporterValue,
    /// Importer-side value is zero or negative and is excluded from this method.
    NonPositiveImporterValue,
}

/// Maximum permitted interpretation of a result from this module.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MirrorClaimCeiling {
    /// A descriptive difference between two reported values only.
    ///
    /// This is never a finding that a transaction is illegal, intentional,
    /// fraudulent, or equal to illicit proceeds.
    DescriptiveValueDifferenceOnly,
}

/// Result for one candidate exporter/importer pair.
///
/// Metrics are present only when `status == Compared`. The signed difference
/// is importer-reported value minus exporter-reported value. The percentage
/// uses the arithmetic mean of the two positive reported values as denominator;
/// that convention is descriptive and must not be called an illicit-flow rate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct MirrorPairResult {
    pub status: MirrorPairStatus,
    pub exporter_record_id: Option<String>,
    pub importer_record_id: Option<String>,
    pub method_id: String,
    /// Exact normalized decimal difference as text; preferred for display/audit.
    pub signed_difference_source_text: Option<String>,
    /// Exact normalized absolute difference as text; preferred for display/audit.
    pub absolute_difference_source_text: Option<String>,
    /// Approximate binary-float projection for charts/analytics, not canonical.
    pub signed_difference_importer_minus_exporter: Option<f64>,
    /// Approximate binary-float projection for charts/analytics, not canonical.
    pub absolute_difference: Option<f64>,
    /// Approximate percentage using the arithmetic mean as denominator.
    pub difference_percent_of_mean: Option<f64>,
    pub claim_ceiling: MirrorClaimCeiling,
}

impl MirrorPairResult {
    fn excluded(
        status: MirrorPairStatus,
        exporter_record_id: Option<String>,
        importer_record_id: Option<String>,
    ) -> Self {
        Self {
            status,
            exporter_record_id,
            importer_record_id,
            method_id: RAW_MIRROR_METHOD_ID.to_string(),
            signed_difference_source_text: None,
            absolute_difference_source_text: None,
            signed_difference_importer_minus_exporter: None,
            absolute_difference: None,
            difference_percent_of_mean: None,
            claim_ceiling: MirrorClaimCeiling::DescriptiveValueDifferenceOnly,
        }
    }
}

/// Versioned name for the intentionally narrow calculation in this module.
pub const RAW_MIRROR_METHOD_ID: &str = "raw_mirror_value_difference/v1";

/// Compare candidate bilateral reports without silently resolving ambiguity.
///
/// The slices must already represent candidates for the intended bilateral
/// comparison under a documented upstream selection policy. This function
/// still rechecks reciprocal reporters and all value-comparison dimensions.
/// It does not infer equivalence across commodity revisions, convert currencies,
/// adjust for freight/insurance, model re-exports, or perform imputation.
pub fn compare_mirror_pair(
    exporter_candidates: &[TradeObservation],
    importer_candidates: &[TradeObservation],
) -> MirrorPairResult {
    let exporter_count = exporter_candidates.len();
    let importer_count = importer_candidates.len();

    if exporter_count == 0 && importer_count == 0 {
        return MirrorPairResult::excluded(MirrorPairStatus::NoCandidateReports, None, None);
    }
    // Report duplicate candidates even when the reciprocal side is absent.
    // Otherwise a missing counterpart could hide ambiguity on the side that
    // does have records.
    if exporter_count > 1 && importer_count > 1 {
        return MirrorPairResult::excluded(
            MirrorPairStatus::AmbiguousReportsBothSides,
            None,
            None,
        );
    }
    if exporter_count > 1 {
        return MirrorPairResult::excluded(
            MirrorPairStatus::AmbiguousExporterReports,
            None,
            None,
        );
    }
    if importer_count > 1 {
        return MirrorPairResult::excluded(
            MirrorPairStatus::AmbiguousImporterReports,
            None,
            None,
        );
    }
    if exporter_count == 0 {
        return MirrorPairResult::excluded(
            MirrorPairStatus::MissingExporterReport,
            None,
            Some(importer_candidates[0].record_id.clone()),
        );
    }
    if importer_count == 0 {
        return MirrorPairResult::excluded(
            MirrorPairStatus::MissingImporterReport,
            Some(exporter_candidates[0].record_id.clone()),
            None,
        );
    }

    let exporter = &exporter_candidates[0];
    let importer = &importer_candidates[0];
    let exporter_id = Some(exporter.record_id.clone());
    let importer_id = Some(importer.record_id.clone());

    if !has_identity(exporter) || !has_identity(importer) {
        return MirrorPairResult::excluded(
            MirrorPairStatus::InvalidRecordIdentity,
            exporter_id,
            importer_id,
        );
    }
    if exporter.flow != TradeFlow::Export || importer.flow != TradeFlow::Import {
        return MirrorPairResult::excluded(
            MirrorPairStatus::InvalidFlowDirection,
            exporter_id,
            importer_id,
        );
    }
    if exporter.reporter != importer.partner || exporter.partner != importer.reporter {
        return MirrorPairResult::excluded(
            MirrorPairStatus::NonReciprocalReporters,
            exporter_id,
            importer_id,
        );
    }
    if exporter.period != importer.period {
        return MirrorPairResult::excluded(
            MirrorPairStatus::PeriodMismatch,
            exporter_id,
            importer_id,
        );
    }
    if exporter.commodity_code != importer.commodity_code {
        return MirrorPairResult::excluded(
            MirrorPairStatus::CommodityMismatch,
            exporter_id,
            importer_id,
        );
    }
    if exporter.classification_revision != importer.classification_revision {
        return MirrorPairResult::excluded(
            MirrorPairStatus::ClassificationMismatch,
            exporter_id,
            importer_id,
        );
    }
    if exporter.currency != importer.currency {
        return MirrorPairResult::excluded(
            MirrorPairStatus::CurrencyMismatch,
            exporter_id,
            importer_id,
        );
    }

    let Some(export_text) = exporter.trade_value_source_text.as_deref() else {
        return MirrorPairResult::excluded(
            MirrorPairStatus::MissingExporterValue,
            exporter_id,
            importer_id,
        );
    };
    let Some(import_text) = importer.trade_value_source_text.as_deref() else {
        return MirrorPairResult::excluded(
            MirrorPairStatus::MissingImporterValue,
            exporter_id,
            importer_id,
        );
    };
    let Some(export_value) = parse_decimal_source_value(export_text) else {
        return MirrorPairResult::excluded(
            MirrorPairStatus::InvalidExporterValueFormat,
            exporter_id,
            importer_id,
        );
    };
    let Some(import_value) = parse_decimal_source_value(import_text) else {
        return MirrorPairResult::excluded(
            MirrorPairStatus::InvalidImporterValueFormat,
            exporter_id,
            importer_id,
        );
    };
    if export_value.coefficient <= 0 {
        return MirrorPairResult::excluded(
            MirrorPairStatus::NonPositiveExporterValue,
            exporter_id,
            importer_id,
        );
    }
    if import_value.coefficient <= 0 {
        return MirrorPairResult::excluded(
            MirrorPairStatus::NonPositiveImporterValue,
            exporter_id,
            importer_id,
        );
    }

    let scale = export_value.scale.max(import_value.scale);
    let Some(export_scaled) = rescale_decimal(export_value, scale) else {
        return MirrorPairResult::excluded(
            MirrorPairStatus::ComparisonArithmeticOverflow,
            exporter_id,
            importer_id,
        );
    };
    let Some(import_scaled) = rescale_decimal(import_value, scale) else {
        return MirrorPairResult::excluded(
            MirrorPairStatus::ComparisonArithmeticOverflow,
            exporter_id,
            importer_id,
        );
    };
    let Some(signed_scaled_difference) = import_scaled.checked_sub(export_scaled) else {
        return MirrorPairResult::excluded(
            MirrorPairStatus::ComparisonArithmeticOverflow,
            exporter_id,
            importer_id,
        );
    };

    let absolute_scaled_difference = signed_scaled_difference.abs();
    let signed_difference_source_text =
        format_scaled_decimal(signed_scaled_difference, scale);
    let absolute_difference_source_text =
        format_scaled_decimal(absolute_scaled_difference, scale);

    // Float projections exist for plots only. Exact decimal text above remains
    // authoritative for audit and display of the measured difference.
    let divisor = 10_f64.powi(scale as i32);
    let signed_difference = signed_scaled_difference as f64 / divisor;
    let absolute_difference = absolute_scaled_difference as f64 / divisor;
    let mean_scaled = (export_scaled as f64 / 2.0) + (import_scaled as f64 / 2.0);
    let difference_percent_of_mean = if mean_scaled > 0.0 {
        (absolute_scaled_difference as f64 / mean_scaled) * 100.0
    } else {
        0.0
    };

    MirrorPairResult {
        status: MirrorPairStatus::Compared,
        exporter_record_id: exporter_id,
        importer_record_id: importer_id,
        method_id: RAW_MIRROR_METHOD_ID.to_string(),
        signed_difference_source_text: Some(signed_difference_source_text),
        absolute_difference_source_text: Some(absolute_difference_source_text),
        signed_difference_importer_minus_exporter: Some(signed_difference),
        absolute_difference: Some(absolute_difference),
        difference_percent_of_mean: Some(difference_percent_of_mean),
        claim_ceiling: MirrorClaimCeiling::DescriptiveValueDifferenceOnly,
    }
}

/// Exact base-10 decimal parsed into an integer coefficient and scale.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ParsedDecimal {
    coefficient: i128,
    scale: u32,
}

/// Parse a strict decimal token without first rounding through f64.
/// Exponent notation, separators, whitespace, and over-precision are rejected
/// until an adapter has an explicit conversion contract.
fn parse_decimal_source_value(text: &str) -> Option<ParsedDecimal> {
    if text.is_empty() || text.trim() != text {
        return None;
    }

    let (negative, unsigned) = match text.strip_prefix('-') {
        Some(rest) => (true, rest),
        None => (false, text),
    };
    if unsigned.is_empty() {
        return None;
    }

    let mut parts = unsigned.split('.');
    let integer = parts.next()?;
    let fraction = parts.next();
    if parts.next().is_some() || integer.is_empty() {
        return None;
    }
    if !integer.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }

    let fraction = match fraction {
        Some(value) if !value.is_empty() && value.bytes().all(|b| b.is_ascii_digit()) => value,
        Some(_) => return None,
        None => "",
    };
    if fraction.len() > 18 {
        return None;
    }

    let mut digits = String::with_capacity(integer.len() + fraction.len());
    digits.push_str(integer);
    digits.push_str(fraction);
    let mut coefficient = digits.parse::<i128>().ok()?;
    if negative {
        coefficient = coefficient.checked_neg()?;
    }

    Some(ParsedDecimal {
        coefficient,
        scale: fraction.len() as u32,
    })
}

/// Scale a decimal to a common precision without rounding.
fn rescale_decimal(value: ParsedDecimal, target_scale: u32) -> Option<i128> {
    if target_scale < value.scale {
        return None;
    }
    let mut coefficient = value.coefficient;
    for _ in value.scale..target_scale {
        coefficient = coefficient.checked_mul(10)?;
    }
    Some(coefficient)
}

/// Format an integer coefficient as a normalized decimal without exponent notation.
fn format_scaled_decimal(coefficient: i128, scale: u32) -> String {
    let negative = coefficient < 0;
    let digits = coefficient.unsigned_abs().to_string();
    let mut value = if scale == 0 {
        digits
    } else if digits.len() <= scale as usize {
        format!("0.{}{}", "0".repeat(scale as usize - digits.len()), digits)
    } else {
        let split = digits.len() - scale as usize;
        format!("{}.{}", &digits[..split], &digits[split..])
    };

    if value.contains('.') {
        while value.ends_with('0') {
            value.pop();
        }
        if value.ends_with('.') {
            value.pop();
        }
    }
    if negative && value != "0" {
        value.insert(0, '-');
    }
    value
}

fn has_identity(observation: &TradeObservation) -> bool {
    !observation.record_id.trim().is_empty()
        && !observation.source_id.trim().is_empty()
        && !observation.source_snapshot_id.trim().is_empty()
        && !observation.reporter.trim().is_empty()
        && !observation.partner.trim().is_empty()
        && !observation.period.trim().is_empty()
        && !observation.commodity_code.trim().is_empty()
        && !observation.classification_revision.trim().is_empty()
        && !observation.currency.trim().is_empty()
        && !observation.retrieved_at.trim().is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn observation(
        record_id: &str,
        reporter: &str,
        partner: &str,
        flow: TradeFlow,
        value: Option<&str>,
    ) -> TradeObservation {
        TradeObservation {
            record_id: record_id.to_string(),
            source_id: "un-comtrade-fixture".to_string(),
            source_snapshot_id: "sha256:fixture-v1".to_string(),
            source_record_id: Some(format!("source-{record_id}")),
            source_url: Some("https://example.invalid/frozen-fixture".to_string()),
            reporter: reporter.to_string(),
            partner: partner.to_string(),
            period: "2025".to_string(),
            commodity_code: "010121".to_string(),
            classification_revision: "HS-2022".to_string(),
            flow,
            trade_value_source_text: value.map(str::to_owned),
            currency: "USD".to_string(),
            quantity: Some(10.0),
            quantity_unit: Some("kg".to_string()),
            source_published_at: Some("2026-01-15".to_string()),
            retrieved_at: "2026-02-01T00:00:00Z".to_string(),
        }
    }

    fn pair(export_value: Option<&str>, import_value: Option<&str>) -> MirrorPairResult {
        let exporter = observation("exp-1", "AAA", "BBB", TradeFlow::Export, export_value);
        let importer = observation("imp-1", "BBB", "AAA", TradeFlow::Import, import_value);
        compare_mirror_pair(&[exporter], &[importer])
    }

    #[test]
    fn computes_raw_difference_and_never_exceeds_claim_ceiling() {
        let exporter = observation("exp-1", "AAA", "BBB", TradeFlow::Export, Some("100"));
        // The source token is retained as the calculation input.
        assert_eq!(exporter.trade_value_source_text.as_deref(), Some("100"));
        let result = pair(Some("100"), Some("120"));
        assert_eq!(result.status, MirrorPairStatus::Compared);
        assert_eq!(
            result.signed_difference_importer_minus_exporter,
            Some(20.0)
        );
        assert_eq!(result.absolute_difference, Some(20.0));
        assert_eq!(result.signed_difference_source_text.as_deref(), Some("20"));
        assert_eq!(result.absolute_difference_source_text.as_deref(), Some("20"));
        let pct = result.difference_percent_of_mean.unwrap();
        assert!((pct - (20.0 / 110.0 * 100.0)).abs() < 1e-12);
        assert_eq!(
            result.claim_ceiling,
            MirrorClaimCeiling::DescriptiveValueDifferenceOnly
        );
        assert_eq!(result.method_id, RAW_MIRROR_METHOD_ID);
    }

    #[test]
    fn equal_reported_values_are_not_proof_of_no_illicit_activity() {
        let result = pair(Some("100"), Some("100"));
        assert_eq!(result.status, MirrorPairStatus::Compared);
        assert_eq!(result.absolute_difference, Some("0"));
        assert_eq!(
            result.claim_ceiling,
            MirrorClaimCeiling::DescriptiveValueDifferenceOnly
        );
    }

    #[test]
    fn rejects_duplicate_candidates_instead_of_silent_deduplication() {
        let exporter = observation("exp-1", "AAA", "BBB", TradeFlow::Export, Some("100"));
        let duplicate = observation("exp-2", "AAA", "BBB", TradeFlow::Export, Some("100"));
        let importer = observation("imp-1", "BBB", "AAA", TradeFlow::Import, Some("120"));
        let result = compare_mirror_pair(&[exporter, duplicate], &[importer]);
        assert_eq!(result.status, MirrorPairStatus::AmbiguousExporterReports);
        assert_eq!(result.absolute_difference, None);
    }

    #[test]
    fn keeps_missing_sides_distinct() {
        assert_eq!(
            compare_mirror_pair(&[], &[]).status,
            MirrorPairStatus::NoCandidateReports
        );
        assert_eq!(
            compare_mirror_pair(
                &[],
                &[observation("imp-1", "BBB", "AAA", TradeFlow::Import, Some("120"))]
            )
            .status,
            MirrorPairStatus::MissingExporterReport
        );
        assert_eq!(
            compare_mirror_pair(
                &[observation("exp-1", "AAA", "BBB", TradeFlow::Export, Some("100"))],
                &[]
            )
            .status,
            MirrorPairStatus::MissingImporterReport
        );

        let duplicate_a =
            observation("imp-1", "BBB", "AAA", TradeFlow::Import, Some("120"));
        let duplicate_b =
            observation("imp-2", "BBB", "AAA", TradeFlow::Import, Some("121"));
        assert_eq!(
            compare_mirror_pair(&[], &[duplicate_a, duplicate_b]).status,
            MirrorPairStatus::AmbiguousImporterReports
        );
    }

    #[test]
    fn rejects_mismatched_period_and_currency() {
        let exporter = observation("exp-1", "AAA", "BBB", TradeFlow::Export, Some("100"));
        let mut importer = observation("imp-1", "BBB", "AAA", TradeFlow::Import, Some("120"));
        importer.period = "2024".to_string();
        assert_eq!(
            compare_mirror_pair(&[exporter.clone()], &[importer]).status,
            MirrorPairStatus::PeriodMismatch
        );

        let mut importer = observation("imp-2", "BBB", "AAA", TradeFlow::Import, Some("120"));
        importer.currency = "EUR".to_string();
        assert_eq!(
            compare_mirror_pair(&[exporter], &[importer]).status,
            MirrorPairStatus::CurrencyMismatch
        );
    }

    #[test]
    fn zero_missing_and_invalid_values_are_not_gaps() {
        assert_eq!(
            pair(Some("0"), Some("120")).status,
            MirrorPairStatus::NonPositiveExporterValue
        );
        assert_eq!(
            pair(Some("100"), None).status,
            MirrorPairStatus::MissingImporterValue
        );
        assert_eq!(
            pair(Some("NaN"), Some("120")).status,
            MirrorPairStatus::InvalidExporterValueFormat
        );
    }

    #[test]
    fn exact_decimal_difference_survives_f64_precision_boundaries() {
        let exporter = observation(
            "exp-1",
            "AAA",
            "BBB",
            TradeFlow::Export,
            Some("9007199254740992"),
        );
        let mut importer = observation(
            "imp-1",
            "BBB",
            "AAA",
            TradeFlow::Import,
            Some("9007199254740992"),
        );
        // This source value differs by one, despite rounding to the same f64.
        importer.trade_value_source_text = Some("9007199254740993".to_string());
        let result = compare_mirror_pair(&[exporter], &[importer]);
        assert_eq!(result.status, MirrorPairStatus::Compared);
        assert_eq!(result.signed_difference_source_text.as_deref(), Some("1"));
        assert_eq!(result.absolute_difference_source_text.as_deref(), Some("1"));
        assert_eq!(result.absolute_difference, Some(1.0));
    }

    #[test]
    fn decimal_text_normalizes_scale_without_losing_exact_difference() {
        let mut exporter = observation(
            "exp-1",
            "AAA",
            "BBB",
            TradeFlow::Export,
            Some("100"),
        );
        exporter.trade_value_source_text = Some("100.00".to_string());
        let mut importer = observation(
            "imp-1",
            "BBB",
            "AAA",
            TradeFlow::Import,
            Some("120"),
        );
        importer.trade_value_source_text = Some("120.000".to_string());
        let result = compare_mirror_pair(&[exporter], &[importer]);
        assert_eq!(result.status, MirrorPairStatus::Compared);
        assert_eq!(result.signed_difference_source_text.as_deref(), Some("20"));
    }

    #[test]
    fn rejects_exponent_notation_precision_and_integer_overflow() {
        let mut exporter = observation(
            "exp-1",
            "AAA",
            "BBB",
            TradeFlow::Export,
            Some("100"),
        );
        exporter.trade_value_source_text = Some("1e2".to_string());
        let importer = observation(
            "imp-1",
            "BBB",
            "AAA",
            TradeFlow::Import,
            Some("120"),
        );
        assert_eq!(
            compare_mirror_pair(&[exporter], &[importer]).status,
            MirrorPairStatus::InvalidExporterValueFormat
        );

        let mut exporter = observation(
            "exp-2",
            "AAA",
            "BBB",
            TradeFlow::Export,
            Some("100"),
        );
        exporter.trade_value_source_text = Some("1.0000000000000000001".to_string());
        let importer = observation(
            "imp-2",
            "BBB",
            "AAA",
            TradeFlow::Import,
            Some("120"),
        );
        assert_eq!(
            compare_mirror_pair(&[exporter], &[importer]).status,
            MirrorPairStatus::InvalidExporterValueFormat
        );

        let mut exporter = observation(
            "exp-3",
            "AAA",
            "BBB",
            TradeFlow::Export,
            Some("100"),
        );
        exporter.trade_value_source_text =
            Some("999999999999999999999999999999999999999999".to_string());
        let importer = observation(
            "imp-3",
            "BBB",
            "AAA",
            TradeFlow::Import,
            Some("120"),
        );
        assert_eq!(
            compare_mirror_pair(&[exporter], &[importer]).status,
            MirrorPairStatus::InvalidExporterValueFormat
        );
    }

    #[test]
    fn preserves_negative_signed_difference_and_small_fractional_values() {
        let mut exporter =
            observation("exp-1", "AAA", "BBB", TradeFlow::Export, Some("100"));
        exporter.trade_value_source_text = Some("100.000".to_string());
        let mut importer =
            observation("imp-1", "BBB", "AAA", TradeFlow::Import, Some("99.999"));
        importer.trade_value_source_text = Some("99.999".to_string());

        let result = compare_mirror_pair(&[exporter], &[importer]);
        assert_eq!(result.status, MirrorPairStatus::Compared);
        assert_eq!(
            result.signed_difference_source_text.as_deref(),
            Some("-0.001")
        );
        assert_eq!(
            result.absolute_difference_source_text.as_deref(),
            Some("0.001")
        );
    }

    #[test]
    fn returns_explicit_overflow_instead_of_rounding_large_mixed_scale_values() {
        let mut exporter =
            observation("exp-1", "AAA", "BBB", TradeFlow::Export, Some("100"));
        exporter.trade_value_source_text =
            Some("170141183460469231731687303715884105727".to_string());
        let mut importer =
            observation("imp-1", "BBB", "AAA", TradeFlow::Import, Some("0.000000000000000001"));
        importer.trade_value_source_text = Some("0.000000000000000001".to_string());

        let result = compare_mirror_pair(&[exporter], &[importer]);
        assert_eq!(result.status, MirrorPairStatus::ComparisonArithmeticOverflow);
        assert_eq!(result.absolute_difference_source_text, None);
    }

    #[test]
    fn rejects_incompatible_commodity_revision_and_nonreciprocal_reports() {
        let exporter = observation("exp-1", "AAA", "BBB", TradeFlow::Export, Some("100"));
        let mut importer = observation("imp-1", "BBB", "AAA", TradeFlow::Import, Some("120"));
        importer.classification_revision = "HS-2017".to_string();
        assert_eq!(
            compare_mirror_pair(&[exporter.clone()], &[importer]).status,
            MirrorPairStatus::ClassificationMismatch
        );

        let importer = observation("imp-2", "BBB", "CCC", TradeFlow::Import, Some("120"));
        assert_eq!(
            compare_mirror_pair(&[exporter], &[importer]).status,
            MirrorPairStatus::NonReciprocalReporters
        );
    }

    #[test]
    fn rejects_wrong_flow_and_invalid_identity() {
        let exporter = observation("exp-1", "AAA", "BBB", TradeFlow::Import, Some("100"));
        let importer = observation("imp-1", "BBB", "AAA", TradeFlow::Import, Some("120"));
        assert_eq!(
            compare_mirror_pair(&[exporter], &[importer]).status,
            MirrorPairStatus::InvalidFlowDirection
        );

        let mut exporter = observation("exp-2", "AAA", "BBB", TradeFlow::Export, Some("100"));
        exporter.source_snapshot_id.clear();
        let importer = observation("imp-2", "BBB", "AAA", TradeFlow::Import, Some("120"));
        assert_eq!(
            compare_mirror_pair(&[exporter], &[importer]).status,
            MirrorPairStatus::InvalidRecordIdentity
        );
    }
}
