// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

use serde::Serialize;
use sha2::{Digest, Sha256};
use sol_atlas_core::thermal_exergy::{
    ThermalExergyAssessment, ThermalExergyRequest, assess_thermal_exergy,
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
    scenario_id: String,
    synthetic: bool,
    claim_ceiling: String,
    verification_boundary: &'static str,
}

#[derive(Debug, Serialize)]
struct RunReport {
    receipt: RunReceipt,
    assessment: ThermalExergyAssessment,
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn report_from_bytes(raw_input: &[u8]) -> Result<RunReport, Box<dyn Error>> {
    let request: ThermalExergyRequest = serde_json::from_slice(raw_input)?;
    let assessment = assess_thermal_exergy(&request)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, format!("{error:?}")))?;
    let assessment_bytes = serde_json::to_vec(&assessment)?;
    let receipt = RunReceipt {
        receipt_schema_version: "1",
        evaluator_version: "thermal-exergy-v0.1",
        digest_algorithm: "SHA-256",
        digest_scope: "exact_input_bytes_and_compact_assessment_json",
        input_sha256: sha256_hex(raw_input),
        assessment_sha256: sha256_hex(&assessment_bytes),
        input_bytes: raw_input.len(),
        scenario_id: request.scenario_id,
        synthetic: request.synthetic,
        claim_ceiling: assessment.claim_ceiling.clone(),
        verification_boundary: concat!(
            "digests bind input and assessment identity; they do not authenticate evidence ",
            "or prove physical performance",
        ),
    };
    Ok(RunReport { receipt, assessment })
}

fn run() -> Result<(), Box<dyn Error>> {
    let args = env::args().collect::<Vec<_>>();
    if args.len() != 2 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "usage: thermal_exergy_assess <scenario.json>",
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
        eprintln!("thermal exergy assessment failed: {error}");
        std::process::exit(2);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str =
        include_str!("../../../schemas/examples/thermal-exergy-80c.synthetic.json");

    #[test]
    fn fixture_matches_hand_derived_exergy_known_answer_and_receipt() {
        let report = report_from_bytes(FIXTURE.as_bytes()).unwrap();
        let assessment = &report.assessment;
        assert!((assessment.thermal_exergy_kwh - 6.795979045731277).abs() < 1.0e-12);
        assert!((assessment.exergy_fraction - (60.0 / 353.15)).abs() < 1.0e-14);
        assert_eq!(assessment.heat_quantity_kwh, 40.0);
        assert!(assessment.thermal_exergy_kwh <= assessment.heat_quantity_kwh);
        assert_eq!(report.receipt.input_sha256, sha256_hex(FIXTURE.as_bytes()));
        assert_eq!(report.receipt.input_bytes, FIXTURE.len());
        assert!(report.receipt.synthetic);
    }

    #[test]
    fn receipt_binds_exact_input_bytes() {
        let trimmed = FIXTURE.trim().as_bytes();
        let original = report_from_bytes(FIXTURE.as_bytes()).unwrap();
        let whitespace_variant = report_from_bytes(trimmed).unwrap();
        assert_ne!(original.receipt.input_sha256, whitespace_variant.receipt.input_sha256);
        assert_eq!(
            original.assessment_sha256_for_test(),
            whitespace_variant.assessment_sha256_for_test()
        );
    }

    #[test]
    fn unknown_json_fields_fail_closed() {
        let input = FIXTURE.replace(
            "\"synthetic\": true",
            "\"synthetic\": true, \"pretend_measured\": true",
        );
        assert!(report_from_bytes(input.as_bytes()).is_err());
    }

    #[test]
    fn wrong_schema_version_fails_closed() {
        let input = FIXTURE.replace("thermal-exergy-v0.1", "thermal-exergy-v0.2");
        assert!(report_from_bytes(input.as_bytes()).is_err());
    }
}

#[cfg(test)]
impl RunReport {
    fn assessment_sha256_for_test(&self) -> String {
        sha256_hex(&serde_json::to_vec(&self.assessment).unwrap())
    }
}
