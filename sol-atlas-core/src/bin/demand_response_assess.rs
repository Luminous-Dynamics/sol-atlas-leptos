//! Compute reference demand-response metrics from an aligned JSON trace.
//!
//! Usage:
//!   cargo run -p sol-atlas-core --bin demand_response_assess -- <trace.json>
//!
//! Both RSR and EDR sections are optional, but at least one must be present.
//! Input intervals are in the order given; interval identifiers must be unique.
//! Power is in kW and interval duration is in hours. The tool computes metrics;
//! it does not authenticate a meter, establish a causal baseline, or qualify a
//! facility for grid-service participation.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sol_atlas_core::demand_response_metrics::{
    CurtailmentAssessment, NormalizedErrorTrackingAssessment, RSR_MAX_ERROR_EXCEEDANCE_FRACTION,
    RSR_NORMALIZED_ERROR_THRESHOLD, RegulationTrackingAssessment, assess_edr_curtailment,
    assess_normalized_rsr_tracking_errors, assess_reference_rsr_tracking,
};
use std::collections::HashSet;
use std::env;
use std::fs;
use std::process;

const INPUT_SCHEMA_VERSION: &str = "1.0";
const OUTPUT_SCHEMA_VERSION: &str = "1.0";

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TraceInput {
    schema_version: String,
    fixture_id: String,
    synthetic: bool,
    /// This tool only reports arithmetic metrics; it does not assert that
    /// the input represents verified metering or proves physical grid benefit.
    claim_ceiling: String,
    #[serde(default)]
    rsr: Option<RsrInput>,
    #[serde(default)]
    rsr_reference: Option<RsrReferenceInput>,
    #[serde(default)]
    edr: Option<EdrInput>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RsrInput {
    /// Chronologically ordered unique interval/sample identifiers.
    interval_ids: Vec<String>,
    target_power_kw: Vec<Option<f64>>,
    measured_power_kw: Vec<Option<f64>>,
    reserve_capacity_kw: f64,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum ErrorSeriesTimeSemantics {
    SourceReportedSeries,
    PriorCycleState,
    ExplicitlyAligned,
    Unknown,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
enum ErrorDenominatorConvention {
    ExactReserve,
    ReservePlusEpsilon,
    SourceNativeUnknown,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct RsrReferenceMetadata {
    source_name: String,
    source_revision: String,
    source_field: String,
    sample_time_semantics: ErrorSeriesTimeSemantics,
    denominator_convention: ErrorDenominatorConvention,
    normalization_note: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RsrReferenceInput {
    /// Metadata needed to interpret the source-normalized values.
    metadata: RsrReferenceMetadata,
    /// Ordered unique sample identifiers from the reference tool's own error series.
    interval_ids: Vec<String>,
    /// Signed, already-normalized tracking error as emitted by the reference tool.
    signed_normalized_tracking_error: Vec<Option<f64>>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct EdrInput {
    /// Chronologically ordered unique interval/sample identifiers.
    interval_ids: Vec<String>,
    baseline_power_kw: Vec<Option<f64>>,
    event_power_kw: Vec<Option<f64>>,
    interval_duration_hours: Vec<f64>,
}

#[derive(Clone, Debug, Serialize)]
struct RunReceipt {
    receipt_schema_version: &'static str,
    digest_algorithm: &'static str,
    digest_scope: &'static str,
    input_sha256: String,
    metrics_sha256: String,
    fixture_id: String,
    synthetic: bool,
    claim_ceiling: String,
    verification_boundary: &'static str,
}

#[derive(Clone, Debug, Serialize)]
struct MetricsResult {
    schema_version: &'static str,
    fixture_id: String,
    synthetic: bool,
    claim_ceiling: String,
    rsr: Option<RegulationTrackingAssessment>,
    rsr_reference: Option<NormalizedErrorTrackingAssessment>,
    rsr_reference_metadata: Option<RsrReferenceMetadata>,
    edr: Option<CurtailmentAssessment>,
}

#[derive(Clone, Debug, Serialize)]
struct RunReport {
    receipt: RunReceipt,
    metrics: MetricsResult,
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    let mut output = String::with_capacity(64);
    for byte in digest {
        use std::fmt::Write as _;
        write!(&mut output, "{byte:02x}").expect("writing to String cannot fail");
    }
    output
}

fn validate_interval_ids(
    section: &'static str,
    interval_ids: &[String],
    expected_len: usize,
) -> Result<(), String> {
    if interval_ids.is_empty() {
        return Err(format!("{section}.interval_ids must not be empty"));
    }
    if interval_ids.len() != expected_len {
        return Err(format!(
            "{section}.interval_ids length mismatch: expected {expected_len}, got {}",
            interval_ids.len()
        ));
    }
    let mut seen = HashSet::with_capacity(interval_ids.len());
    for (index, id) in interval_ids.iter().enumerate() {
        if id.trim().is_empty() {
            return Err(format!("{section}.interval_ids[{index}] is empty"));
        }
        if !seen.insert(id.as_str()) {
            return Err(format!(
                "{section}.interval_ids contains duplicate identifier {id:?}"
            ));
        }
    }
    Ok(())
}

fn validate_reference_metadata(metadata: &RsrReferenceMetadata) -> Result<(), String> {
    for (field, value) in [
        ("source_name", metadata.source_name.as_str()),
        ("source_revision", metadata.source_revision.as_str()),
        ("source_field", metadata.source_field.as_str()),
        ("normalization_note", metadata.normalization_note.as_str()),
    ] {
        if value.trim().is_empty() {
            return Err(format!("rsr_reference.metadata.{field} must not be blank"));
        }
    }
    Ok(())
}

fn build_report(raw_input: &[u8], input: TraceInput) -> Result<RunReport, String> {
    if input.schema_version != INPUT_SCHEMA_VERSION {
        return Err(format!(
            "schema_version must be {INPUT_SCHEMA_VERSION}, got {:?}",
            input.schema_version
        ));
    }
    if input.fixture_id.trim().is_empty() {
        return Err("fixture_id must not be empty".to_owned());
    }
    if input.claim_ceiling != "metric_accounting_only" {
        return Err(
            "claim_ceiling must be metric_accounting_only; this tool does not establish evidence provenance"
                .to_owned(),
        );
    }
    if input.rsr.is_none() && input.rsr_reference.is_none() && input.edr.is_none() {
        return Err("at least one of rsr, rsr_reference, or edr must be present".to_owned());
    }

    let rsr = match input.rsr {
        Some(section) => {
            validate_interval_ids("rsr", &section.interval_ids, section.target_power_kw.len())?;
            if section.interval_ids.len() != section.measured_power_kw.len() {
                return Err(format!(
                    "rsr.interval_ids length mismatch: expected {}, got {}",
                    section.interval_ids.len(),
                    section.measured_power_kw.len()
                ));
            }
            Some(
                assess_reference_rsr_tracking(
                    &section.target_power_kw,
                    &section.measured_power_kw,
                    section.reserve_capacity_kw,
                )
                .map_err(|error| format!("RSR assessment failed: {error}"))?,
            )
        }
        None => None,
    };

    let (rsr_reference, rsr_reference_metadata) = match input.rsr_reference {
        Some(section) => {
            validate_interval_ids(
                "rsr_reference",
                &section.interval_ids,
                section.signed_normalized_tracking_error.len(),
            )?;
            validate_reference_metadata(&section.metadata)?;
            let assessment = assess_normalized_rsr_tracking_errors(
                &section.signed_normalized_tracking_error,
                RSR_NORMALIZED_ERROR_THRESHOLD,
                RSR_MAX_ERROR_EXCEEDANCE_FRACTION,
            )
            .map_err(|error| format!("normalized RSR assessment failed: {error}"))?;
            (Some(assessment), Some(section.metadata))
        }
        None => (None, None),
    };

    let edr = match input.edr {
        Some(section) => {
            validate_interval_ids(
                "edr",
                &section.interval_ids,
                section.baseline_power_kw.len(),
            )?;
            if section.interval_ids.len() != section.event_power_kw.len()
                || section.interval_ids.len() != section.interval_duration_hours.len()
            {
                return Err(format!(
                    "edr input length mismatch: ids={}, event={}, duration={}",
                    section.interval_ids.len(),
                    section.event_power_kw.len(),
                    section.interval_duration_hours.len()
                ));
            }
            Some(
                assess_edr_curtailment(
                    &section.baseline_power_kw,
                    &section.event_power_kw,
                    &section.interval_duration_hours,
                )
                .map_err(|error| format!("EDR assessment failed: {error}"))?,
            )
        }
        None => None,
    };

    let metrics = MetricsResult {
        schema_version: OUTPUT_SCHEMA_VERSION,
        fixture_id: input.fixture_id.clone(),
        synthetic: input.synthetic,
        claim_ceiling: input.claim_ceiling.clone(),
        rsr,
        rsr_reference,
        rsr_reference_metadata,
        edr,
    };
    let metrics_bytes = serde_json::to_vec(&metrics)
        .map_err(|error| format!("could not serialize metrics: {error}"))?;
    let receipt = RunReceipt {
        receipt_schema_version: OUTPUT_SCHEMA_VERSION,
        digest_algorithm: "SHA-256",
        digest_scope: "exact_input_bytes_and_compact_metrics_json",
        input_sha256: sha256_hex(raw_input),
        metrics_sha256: sha256_hex(&metrics_bytes),
        fixture_id: input.fixture_id,
        synthetic: input.synthetic,
        claim_ceiling: input.claim_ceiling,
        verification_boundary: "digests bind byte identity, not sensor calibration, evidence truth, or causal grid benefit",
    };
    Ok(RunReport { receipt, metrics })
}

fn main() {
    let mut args = env::args_os().skip(1);
    let Some(path) = args.next() else {
        eprintln!("usage: demand_response_assess <trace.json>");
        process::exit(64);
    };
    if args.next().is_some() {
        eprintln!("usage: demand_response_assess <trace.json>");
        process::exit(64);
    }

    let raw = match fs::read(&path) {
        Ok(bytes) => bytes,
        Err(error) => {
            eprintln!("could not read trace file: {error}");
            process::exit(66);
        }
    };
    let input: TraceInput = match serde_json::from_slice(&raw) {
        Ok(input) => input,
        Err(error) => {
            eprintln!("could not parse strict demand-response trace JSON: {error}");
            process::exit(65);
        }
    };
    let report = match build_report(&raw, input) {
        Ok(report) => report,
        Err(error) => {
            eprintln!("{error}");
            process::exit(5);
        }
    };
    match serde_json::to_string_pretty(&report) {
        Ok(output) => println!("{output}"),
        Err(error) => {
            eprintln!("could not serialize run report: {error}");
            process::exit(70);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{TraceInput, build_report};
    use serde_json::Value;

    fn fixture() -> (&'static str, TraceInput) {
        let raw = include_str!("../../../schemas/examples/demand-response-trace-v1.synthetic.json");
        let input: TraceInput =
            serde_json::from_str(raw).expect("committed trace fixture should parse strictly");
        (raw, input)
    }

    #[test]
    fn known_answer_fixture_meets_expected_rsr_and_edr_metrics() {
        let (raw, input) = fixture();
        let report =
            build_report(raw.as_bytes(), input).expect("known-answer fixture should assess");
        let rsr = report.metrics.rsr.expect("fixture includes RSR input");
        let edr = report.metrics.edr.expect("fixture includes EDR input");

        let rsr_reference = report
            .metrics
            .rsr_reference
            .expect("fixture includes reference-normalized RSR input");
        assert_eq!(rsr.sample_count, 10);
        assert_eq!(rsr.threshold_exceedance_count, 1);
        assert_eq!(rsr.threshold_exceedance_fraction, 0.1);
        assert!(!rsr.exceedance_rate_requirement_satisfied);
        assert!(rsr.p90_threshold_satisfied);
        assert!((rsr.p90_normalized_absolute_error - 0.08).abs() < 1.0e-12);
        assert_eq!(rsr_reference.sample_count, 10);
        assert!((rsr_reference.p90_normalized_absolute_error - 0.08).abs() < 1.0e-12);
        assert!(!rsr_reference.exceedance_rate_requirement_satisfied);
        assert!(rsr_reference.p90_threshold_satisfied);

        assert_eq!(edr.baseline_energy_kwh, 300.0);
        assert_eq!(edr.event_energy_kwh, 260.0);
        assert_eq!(edr.net_energy_reduction_kwh, 40.0);
        assert_eq!(edr.gross_curtailment_kwh, 50.0);
        assert_eq!(edr.rebound_energy_kwh, 10.0);
        assert_eq!(edr.rebound_interval_count, 1);
        assert_eq!(edr.peak_event_power_excess_over_baseline_kw, 10.0);
    }

    #[test]
    fn receipt_binds_raw_bytes_and_metrics_and_preserves_claim_ceiling() {
        let (raw, input) = fixture();
        let report = build_report(raw.as_bytes(), input.clone()).expect("fixture should assess");
        assert_eq!(
            report.receipt.input_sha256,
            super::sha256_hex(raw.as_bytes())
        );
        assert_eq!(report.receipt.claim_ceiling, "metric_accounting_only");
        assert!(report.receipt.verification_boundary.contains("not sensor"));

        let reflowed = format!(" \n{raw}");
        let reflowed_input: TraceInput =
            serde_json::from_str(&reflowed).expect("whitespace change should remain parseable");
        let second = build_report(reflowed.as_bytes(), reflowed_input)
            .expect("reformatted fixture should assess");
        assert_ne!(report.receipt.input_sha256, second.receipt.input_sha256);
        assert_eq!(report.receipt.metrics_sha256, second.receipt.metrics_sha256);
    }

    #[test]
    fn sha256_hex_matches_the_standard_abc_known_answer() {
        assert_eq!(
            super::sha256_hex(b"abc"),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn changing_observed_power_changes_input_and_metrics_digests() {
        let (raw, _) = fixture();
        let changed = raw.replacen(
            "\"measured_power_kw\": [\n      100,",
            "\"measured_power_kw\": [\n      101,",
            1,
        );
        assert_ne!(raw, changed);
        let input: TraceInput =
            serde_json::from_str(&changed).expect("changed known-answer trace should parse");
        let report = build_report(raw.as_bytes(), fixture().1).expect("original fixture works");
        let changed_report =
            build_report(changed.as_bytes(), input).expect("changed fixture still assesses");

        assert_ne!(
            report.receipt.input_sha256,
            changed_report.receipt.input_sha256
        );
        assert_ne!(
            report.receipt.metrics_sha256,
            changed_report.receipt.metrics_sha256
        );
    }

    #[test]
    fn interval_id_length_mismatch_is_rejected() {
        let (_, mut input) = fixture();
        let rsr = input.rsr.as_mut().expect("fixture has RSR");
        rsr.interval_ids.pop();
        assert!(
            build_report(b"misaligned-ids", input)
                .expect_err("misaligned identifiers must be rejected")
                .contains("interval_ids length mismatch")
        );
    }

    #[test]
    fn changed_claim_ceiling_is_rejected_before_assessment() {
        let (_, mut input) = fixture();
        input.claim_ceiling = "quality_accepted_work_per_kwh".to_owned();
        assert!(
            build_report(b"changed-input", input)
                .expect_err("unsupported claim ceiling must be rejected")
                .contains("metric_accounting_only")
        );
    }

    #[test]
    fn duplicate_interval_ids_are_rejected() {
        let (_, mut input) = fixture();
        let rsr = input.rsr.as_mut().expect("fixture has RSR");
        rsr.interval_ids[1] = rsr.interval_ids[0].clone();
        assert!(
            build_report(b"duplicate-ids", input)
                .expect_err("duplicate identifiers must be rejected")
                .contains("duplicate identifier")
        );
    }

    #[test]
    fn absent_metric_sections_are_rejected() {
        let (_, mut input) = fixture();
        input.rsr = None;
        input.rsr_reference = None;
        input.edr = None;
        assert!(
            build_report(b"no-sections", input)
                .expect_err("empty trace must be rejected")
                .contains("at least one")
        );
    }

    #[test]
    fn strict_trace_envelope_rejects_unknown_fields() {
        let (raw, _) = fixture();
        let altered = raw.replace(
            "\"fixture_id\":",
            "\"unmodeled_resource_claim\": true, \"fixture_id\":",
        );
        assert!(serde_json::from_str::<TraceInput>(&altered).is_err());
    }

    #[test]
    fn report_serializes_without_losing_the_distinct_metrics() {
        let (raw, input) = fixture();
        let report = build_report(raw.as_bytes(), input).expect("fixture should assess");
        let value: Value =
            serde_json::to_value(report).expect("run report should serialize as JSON");
        assert_eq!(
            value["metrics"]["rsr"]["exceedance_rate_requirement_satisfied"],
            false
        );
        assert_eq!(value["metrics"]["rsr"]["p90_threshold_satisfied"], true);
        assert_eq!(
            value["metrics"]["rsr_reference"]["p90_threshold_satisfied"],
            true
        );
        assert_eq!(
            value["metrics"]["rsr_reference"]["normalization_basis"],
            "source_normalized_error_series"
        );
        assert_eq!(
            value["metrics"]["rsr_reference"]["threshold_exceedance_fraction"],
            0.1
        );
        assert_eq!(
            value["metrics"]["rsr_reference_metadata"]["source_name"],
            "synthetic-known-answer"
        );
        assert_eq!(
            value["metrics"]["rsr_reference_metadata"]["sample_time_semantics"],
            "source_reported_series"
        );
        assert_eq!(
            value["metrics"]["rsr_reference_metadata"]["denominator_convention"],
            "source_native_unknown"
        );
        assert_eq!(value["metrics"]["edr"]["gross_curtailment_kwh"], 50.0);
        assert_eq!(value["metrics"]["edr"]["rebound_energy_kwh"], 10.0);
        assert_eq!(value["receipt"]["digest_algorithm"], "SHA-256");
        assert_eq!(value["receipt"]["synthetic"], true);
    }
}
