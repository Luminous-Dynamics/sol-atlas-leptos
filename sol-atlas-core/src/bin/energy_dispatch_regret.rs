// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

use serde::Serialize;
use sha2::{Digest, Sha256};
use sol_atlas_core::energy_dispatch::EnergyDispatchRequest;
use sol_atlas_core::energy_dispatch_baseline::{DispatchRegretAssessment, assess_dispatch_regret};
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
    case_id: String,
    synthetic: bool,
    policy_version: String,
    verification_boundary: &'static str,
}

#[derive(Debug, Serialize)]
struct RunReport {
    receipt: RunReceipt,
    assessment: DispatchRegretAssessment,
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn report_from_bytes(raw_input: &[u8]) -> Result<RunReport, Box<dyn Error>> {
    let request: EnergyDispatchRequest = serde_json::from_slice(raw_input)?;
    let assessment = assess_dispatch_regret(&request)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, format!("{error:?}")))?;
    let assessment_bytes = serde_json::to_vec(&assessment)?;
    let receipt = RunReceipt {
        receipt_schema_version: "1",
        evaluator_version: "energy-dispatch-regret-v0.1",
        digest_algorithm: "SHA-256",
        digest_scope: "exact_input_bytes_and_compact_assessment_json",
        input_sha256: sha256_hex(raw_input),
        assessment_sha256: sha256_hex(&assessment_bytes),
        input_bytes: raw_input.len(),
        case_id: request.case_id,
        synthetic: request.synthetic,
        policy_version: assessment.policy_version.clone(),
        verification_boundary: concat!(
            "digests bind input and assessment identity; they do not authenticate ",
            "evidence truth, meter calibration, physical safety, or model adequacy",
        ),
    };
    Ok(RunReport { receipt, assessment })
}

fn run() -> Result<(), Box<dyn Error>> {
    let args = env::args().collect::<Vec<_>>();
    if args.len() != 2 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "usage: energy_dispatch_regret <dispatch-request.json>",
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
        eprintln!("energy dispatch regret assessment failed: {error}");
        std::process::exit(2);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str =
        include_str!("../../../schemas/examples/energy-dispatch-pv-outage.synthetic.json");

    #[test]
    fn report_receipt_binds_input_and_baseline_comparison() {
        let report = report_from_bytes(FIXTURE.as_bytes()).expect("fixture compare is feasible");
        assert_eq!(report.receipt.input_sha256, sha256_hex(FIXTURE.as_bytes()));
        assert_eq!(report.receipt.input_bytes, FIXTURE.len());
        assert_eq!(report.receipt.case_id, "synthetic-pv-charge-outage");
        assert_eq!(report.receipt.policy_version, "pv-grid-battery-priority-v0.1");
        assert!(report.assessment.baseline_feasible);
        assert_eq!(
            report.assessment.objective_deltas_vs_lexicographic_oracle.as_ref().unwrap().essential_unserved_quanta,
            0
        );
        let first = serde_json::to_vec(&report.assessment).expect("serialize");
        let second_report =
            report_from_bytes(FIXTURE.as_bytes()).expect("same input is reproducible");
        let second = serde_json::to_vec(&second_report.assessment).expect("serialize rerun");
        assert_eq!(first, second);
    }
}
