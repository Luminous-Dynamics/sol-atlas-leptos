// Copyright (C) 2024-2026 Tristan Stoltz / Luminous Dynamics
// SPDX-License-Identifier: AGPL-3.0-or-later
// Commercial licensing: see COMMERCIAL_LICENSE.md at repository root

use serde::Serialize;
use sha2::{Digest, Sha256};
use sol_atlas_core::energy_dispatch::{
    EnergyDispatchAssessment, EnergyDispatchRequest, assess_energy_dispatch,
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
    case_id: String,
    synthetic: bool,
    claim_ceiling: String,
    verification_boundary: &'static str,
}

#[derive(Debug, Serialize)]
struct RunReport {
    receipt: RunReceipt,
    assessment: EnergyDispatchAssessment,
}

fn sha256_hex(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn report_from_bytes(raw_input: &[u8]) -> Result<RunReport, Box<dyn Error>> {
    let request: EnergyDispatchRequest = serde_json::from_slice(raw_input)?;
    let assessment = assess_energy_dispatch(&request)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, format!("{error:?}")))?;
    let assessment_bytes = serde_json::to_vec(&assessment)?;
    let receipt = RunReceipt {
        receipt_schema_version: "1",
        evaluator_version: "energy-dispatch-v0.1",
        digest_algorithm: "SHA-256",
        digest_scope: "exact_input_bytes_and_compact_assessment_json",
        input_sha256: sha256_hex(raw_input),
        assessment_sha256: sha256_hex(&assessment_bytes),
        input_bytes: raw_input.len(),
        case_id: request.case_id,
        synthetic: request.synthetic,
        claim_ceiling: assessment.claim_ceiling.clone(),
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
            "usage: energy_dispatch_assess <dispatch-request.json>",
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
        eprintln!("energy dispatch assessment failed: {error}");
        std::process::exit(2);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str =
        include_str!("../../../schemas/examples/energy-dispatch-pv-outage.synthetic.json");

    #[test]
    fn fixture_produces_complete_frontier_and_receipt() {
        let report = report_from_bytes(FIXTURE.as_bytes()).expect("frozen case is feasible");
        let best = &report.assessment.pareto_alternatives[0];
        assert_eq!(best.objective.essential_unserved_quanta, 0);
        assert_eq!(best.objective.flexible_unserved_quanta, 0);
        assert_eq!(best.objective.operating_cost_units, 0);
        assert_eq!(best.objective.grid_carbon_grams, 0);
        assert_eq!(best.objective.curtailed_pv_quanta, 0);
        assert_eq!(best.objective.terminal_soc_quanta, 0);
        assert_eq!(report.receipt.input_sha256, sha256_hex(FIXTURE.as_bytes()));
        assert_eq!(report.receipt.input_bytes, FIXTURE.len());
        assert_eq!(report.receipt.case_id, "synthetic-pv-charge-outage");
        assert!(report.assessment.search_is_exact);
        assert!(report.assessment.pareto_frontier_complete);
    }

    #[test]
    fn candidate_cap_failure_is_nonzero_error_not_a_partial_report() {
        let mut request: EnergyDispatchRequest =
            serde_json::from_str(FIXTURE).expect("valid frozen request");
        request.max_candidate_expansions = 1;
        let bytes = serde_json::to_vec(&request).expect("serialize request");
        assert!(report_from_bytes(&bytes).is_err());
    }
}
