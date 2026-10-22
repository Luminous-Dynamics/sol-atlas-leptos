// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

use serde::Serialize;
use sha2::{Digest, Sha256};
use sol_atlas_core::energy_reliability::{
    EnergyReliabilityAssessment, EnergyReliabilityRequest, assess_energy_reliability,
};
use std::{env, error::Error, fs, io};

#[derive(Debug, Serialize)]
struct RunReceipt {
    receipt_schema_version: &'static str,
    evaluator_version: &'static str,
    digest_algorithm: &'static str,
    digest_scope: &'static str,
    input_sha256: String,
    assessment_sha256: String,
    input_bytes: usize,
    trace_id: String,
    synthetic: bool,
    claim_ceiling: String,
    verification_boundary: &'static str,
}

#[derive(Debug, Serialize)]
struct RunReport {
    receipt: RunReceipt,
    assessment: EnergyReliabilityAssessment,
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn report_from_bytes(raw_input: &[u8]) -> Result<RunReport, Box<dyn Error>> {
    let request: EnergyReliabilityRequest = serde_json::from_slice(raw_input)?;
    let assessment = assess_energy_reliability(&request)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, format!("{error:?}")))?;
    let assessment_bytes = serde_json::to_vec(&assessment)?;
    let receipt = RunReceipt {
        receipt_schema_version: "1",
        evaluator_version: "energy-reliability-v0.1",
        digest_algorithm: "SHA-256",
        digest_scope: "exact_input_bytes_and_compact_assessment_json",
        input_sha256: sha256_hex(raw_input),
        assessment_sha256: sha256_hex(&assessment_bytes),
        input_bytes: raw_input.len(),
        trace_id: request.trace_id,
        synthetic: request.synthetic,
        claim_ceiling: assessment.claim_ceiling.clone(),
        verification_boundary: concat!(
            "digests bind input and assessment identity; they do not authenticate meter ",
            "calibration, evidence truth, causal supply, or physical service delivery",
        ),
    };
    Ok(RunReport { receipt, assessment })
}

fn run() -> Result<(), Box<dyn Error>> {
    let args = env::args().collect::<Vec<_>>();
    if args.len() != 2 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "usage: energy_reliability_assess <trace.json>",
        )
        .into());
    }
    let input = fs::read(&args[1])?;
    let report = report_from_bytes(&input)?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("energy reliability assessment failed: {error}");
        std::process::exit(2);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str =
        include_str!("../../../schemas/examples/energy-reliability-outage.synthetic.json");

    #[test]
    fn fixture_matches_outage_duration_deficit_and_receipt() {
        let report = report_from_bytes(FIXTURE.as_bytes()).unwrap();
        let assessment = &report.assessment;
        assert_eq!(assessment.total_requested_service_kwh, 40.0);
        assert_eq!(assessment.total_delivered_service_kwh, 28.0);
        assert_eq!(assessment.total_unserved_service_kwh, 12.0);
        assert_eq!(assessment.grid_outage_duration_seconds, 7_200);
        assert_eq!(assessment.outage_essential_unserved_service_kwh, 12.0);
        assert_eq!(assessment.longest_consecutive_essential_deficit_seconds, 7_200);
        assert_eq!(assessment.longest_consecutive_outage_essential_deficit_seconds, 7_200);
        assert_eq!(report.receipt.input_sha256, sha256_hex(FIXTURE.as_bytes()));
        assert_eq!(report.receipt.input_bytes, FIXTURE.len());
        assert!(report.receipt.synthetic);
    }

    #[test]
    fn receipt_binds_exact_input_bytes_and_assessment() {
        let trimmed = FIXTURE.trim().as_bytes();
        let original = report_from_bytes(FIXTURE.as_bytes()).unwrap();
        let whitespace_variant = report_from_bytes(trimmed).unwrap();
        assert_ne!(original.receipt.input_sha256, whitespace_variant.receipt.input_sha256);
        assert_eq!(
            original.receipt.assessment_sha256,
            original.assessment_sha256_for_test()
        );
        assert_eq!(
            whitespace_variant.receipt.assessment_sha256,
            whitespace_variant.assessment_sha256_for_test()
        );
        assert_eq!(
            original.assessment_sha256_for_test(),
            whitespace_variant.assessment_sha256_for_test()
        );
    }

    #[test]
    fn unknown_json_fields_fail_closed() {
        let input = FIXTURE.replace(
            "\"synthetic\": true",
            "\"synthetic\": true, \"pretend_site_survived\": true",
        );
        assert!(report_from_bytes(input.as_bytes()).is_err());
    }
}

#[cfg(test)]
impl RunReport {
    fn assessment_sha256_for_test(&self) -> String {
        sha256_hex(&serde_json::to_vec(&self.assessment).unwrap())
    }
}
