use sa_economic_portfolio_oracle::{Input, Portfolio, RejectionCounts, check_expected, evaluate};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fs;

const DEFAULT_INPUT: &str =
    include_str!("../../../docs/research/fixtures/sa-economic-portfolio-v0.1.json");
const DEFAULT_EXPECTED: &str =
    include_str!("../../../docs/research/fixtures/sa-economic-portfolio-v0.1.expected.json");

#[derive(Serialize)]
struct Receipt<'a> {
    schema_version: &'static str,
    evidence_class: &'static str,
    synthetic_only: bool,
    claim_ceiling: &'static str,
    input_sha256: String,
    candidate_subset_count: u64,
    feasible_count: usize,
    rejected_subset_count: u64,
    rejection_reason_counts: &'a RejectionCounts,
    frontier_count: usize,
    feasible_portfolio_ids: Vec<Vec<String>>,
    pareto_frontier: &'a [Portfolio],
    validation: &'static str,
}

fn run() -> Result<(), String> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.len() > 2 {
        return Err("usage: sa-economic-portfolio-oracle [INPUT.json [EXPECTED.json]]".to_owned());
    }

    let input_raw = match args.first() {
        Some(path) => fs::read_to_string(path)
            .map_err(|error| format!("cannot read input {path}: {error}"))?,
        None => DEFAULT_INPUT.to_owned(),
    };
    let expected_raw = match args.get(1) {
        Some(path) => fs::read_to_string(path)
            .map_err(|error| format!("cannot read expected manifest {path}: {error}"))?,
        None => DEFAULT_EXPECTED.to_owned(),
    };

    let input: Input = serde_json::from_str(&input_raw)
        .map_err(|error| format!("invalid input JSON/contract: {error}"))?;
    let evaluation = evaluate(&input)?;
    check_expected(&evaluation, &expected_raw)?;

    let digest = Sha256::digest(input_raw.as_bytes());
    let input_sha256 = digest.iter().map(|byte| format!("{byte:02x}")).collect();
    let feasible_portfolio_ids = evaluation
        .feasible_portfolios
        .iter()
        .map(|portfolio| portfolio.project_ids.clone())
        .collect();
    let receipt = Receipt {
        schema_version: "sa-economic-portfolio-receipt-v0.1",
        evidence_class: "synthetic_exact_oracle",
        synthetic_only: true,
        claim_ceiling: "Finite synthetic contract validation only; no real-world ROI, service impact, jobs, legal approval, or investment decision.",
        input_sha256,
        candidate_subset_count: evaluation.candidate_subset_count,
        feasible_count: evaluation.feasible_portfolios.len(),
        rejected_subset_count: evaluation.rejection_reason_counts.total(),
        rejection_reason_counts: &evaluation.rejection_reason_counts,
        frontier_count: evaluation.pareto_frontier.len(),
        feasible_portfolio_ids,
        pareto_frontier: &evaluation.pareto_frontier,
        validation: "PASS: exact comparison with the supplied known-answer manifest",
    };

    println!(
        "{}",
        serde_json::to_string_pretty(&receipt).map_err(|error| error.to_string())?
    );
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("sa-economic-portfolio-oracle: {error}");
        std::process::exit(1);
    }
}
