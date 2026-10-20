use sa_economic_portfolio_oracle::{
    evaluate_frontier_search, FrontierSearchStats, Input, Portfolio, RejectionCounts,
    check_expected, evaluate,
};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fs;

const DEFAULT_INPUT: &str =
    include_str!("../../../docs/research/fixtures/sa-economic-portfolio-v0.1.json");
const DEFAULT_EXPECTED: &str =
    include_str!("../../../docs/research/fixtures/sa-economic-portfolio-v0.1.expected.json");

#[derive(Serialize)]
struct SearchReceipt<'a> {
    schema_version: String,
    evidence_class: &'static str,
    synthetic_only: bool,
    claim_ceiling: &'static str,
    input_sha256: String,
    reference_manifest_sha256: Option<String>,
    stats: &'a FrontierSearchStats,
    frontier_count: usize,
    pareto_frontier: &'a [Portfolio],
    validation: &'static str,
}

#[derive(Serialize)]
struct Receipt<'a> {
    schema_version: String,
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
    if args.first().map(String::as_str) == Some("--search") {
        return run_search(&args[1..]);
    }
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
        schema_version: match input.schema_version.as_str() {
            "sa-economic-portfolio-v0.1" => "sa-economic-portfolio-receipt-v0.1".to_owned(),
            "sa-economic-portfolio-v0.2" => "sa-economic-portfolio-receipt-v0.2".to_owned(),
            _ => return Err("unsupported schema version after evaluation".to_owned()),
        },
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

fn run_search(args: &[String]) -> Result<(), String> {
    if args.is_empty() || args.len() > 2 {
        return Err(
            "usage: sa-economic-portfolio-oracle --search INPUT.json [REFERENCE_FRONTIER.json]"
                .to_owned(),
        );
    }

    let input_path = &args[0];
    let input_raw = fs::read_to_string(input_path)
        .map_err(|error| format!("cannot read input {input_path}: {error}"))?;
    let input: Input = serde_json::from_str(&input_raw)
        .map_err(|error| format!("invalid input JSON/contract: {error}"))?;
    let evaluation = evaluate_frontier_search(&input)?;

    let (reference_manifest_sha256, validation) = if let Some(reference_path) = args.get(1) {
        let reference_raw = fs::read_to_string(reference_path)
            .map_err(|error| format!("cannot read reference frontier {reference_path}: {error}"))?;
        let reference: serde_json::Value = serde_json::from_str(&reference_raw)
            .map_err(|error| format!("invalid reference frontier JSON: {error}"))?;
        let mut expected_frontier: Vec<Portfolio> =
            serde_json::from_value(reference["pareto_frontier"].clone())
                .map_err(|error| format!("reference frontier missing/invalid: {error}"))?;
        expected_frontier.sort_by_key(|portfolio| portfolio.project_ids.join("+"));
        if evaluation.pareto_frontier != expected_frontier {
            return Err("frontier-search output does not match supplied reference frontier".to_owned());
        }
        let digest = Sha256::digest(reference_raw.as_bytes());
        let manifest_sha256 = digest.iter().map(|byte| format!("{byte:02x}")).collect();
        (
            Some(manifest_sha256),
            "PASS: frontier matches supplied reference manifest",
        )
    } else {
        (None, "NOT_VERIFIED: no reference frontier manifest supplied")
    };

    let input_digest = Sha256::digest(input_raw.as_bytes());
    let input_sha256 = input_digest.iter().map(|byte| format!("{byte:02x}")).collect();
    let schema_version = match input.schema_version.as_str() {
        "sa-economic-portfolio-v0.1" => {
            "sa-economic-portfolio-search-receipt-v0.1".to_owned()
        }
        "sa-economic-portfolio-v0.2" => {
            "sa-economic-portfolio-search-receipt-v0.2".to_owned()
        }
        _ => return Err("unsupported schema version after frontier search".to_owned()),
    };
    let receipt = SearchReceipt {
        schema_version,
        evidence_class: "synthetic_frontier_search",
        synthetic_only: true,
        claim_ceiling: "Exact synthetic frontier search only; no real-world impact, ROI, jobs or investment authority.",
        input_sha256,
        reference_manifest_sha256,
        stats: &evaluation.stats,
        frontier_count: evaluation.pareto_frontier.len(),
        pareto_frontier: &evaluation.pareto_frontier,
        validation,
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
