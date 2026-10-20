use sa_economic_portfolio_oracle::{Input, Portfolio, RejectionCounts, RobustRegretEvaluation, check_expected, evaluate, evaluate_frontier_search, evaluate_robust_regret, evaluate_robust_regret_from_search};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::fs;

const DEFAULT_INPUT: &str =
    include_str!("../../../docs/research/fixtures/sa-economic-portfolio-v0.1.json");
const DEFAULT_EXPECTED: &str =
    include_str!("../../../docs/research/fixtures/sa-economic-portfolio-v0.1.expected.json");

#[derive(Serialize)]
struct RegretReceipt {
    schema_version: String,
    evidence_class: &'static str,
    synthetic_only: bool,
    claim_ceiling: &'static str,
    input_sha256: String,
    reference_manifest_sha256: String,
    candidate_subset_count: u64,
    feasible_count: u64,
    outcome_frontier_count: usize,
    regret_frontier_count: usize,
    robust_regret: RobustRegretEvaluation,
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
    if args.first().map(String::as_str) == Some("--regret") {
        return run_regret(&args[1..]);
    }
    if args.first().map(String::as_str) == Some("--search-regret") {
        return run_search_regret(&args[1..]);
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

fn build_regret_receipt(
    input_raw: &str,
    reference_raw: &str,
    schema_version: &str,
    candidate_subset_count: u64,
    feasible_count: u64,
    robust_regret: RobustRegretEvaluation,
    validation: &'static str,
) -> RegretReceipt {
    RegretReceipt {
        schema_version: match schema_version {
            "sa-economic-portfolio-v0.1" => "sa-economic-portfolio-regret-receipt-v0.1".to_owned(),
            "sa-economic-portfolio-v0.2" => "sa-economic-portfolio-regret-receipt-v0.2".to_owned(),
            _ => "sa-economic-portfolio-regret-receipt-unsupported".to_owned(),
        },
        evidence_class: "synthetic_robust_regret_analysis",
        synthetic_only: true,
        claim_ceiling: "Exact regret analysis over a complete synthetic frontier only; no real-world impact, ROI, jobs or investment authority.",
        input_sha256: input_raw_sha256(input_raw),
        reference_manifest_sha256: input_raw_sha256(reference_raw),
        candidate_subset_count,
        feasible_count,
        outcome_frontier_count: robust_regret.portfolio_regrets.len(),
        regret_frontier_count: robust_regret.minimax_regret_frontier.len(),
        robust_regret,
        validation,
    }
}

fn input_raw_sha256(raw: &str) -> String {
    Sha256::digest(raw.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn run_regret(args: &[String]) -> Result<(), String> {
    if args.len() != 2 {
        return Err(
            "usage: sa-economic-portfolio-oracle --regret INPUT.json EXPECTED.json".to_owned(),
        );
    }
    let input_raw = fs::read_to_string(&args[0])
        .map_err(|error| format!("cannot read input {}: {error}", args[0]))?;
    let reference_raw = fs::read_to_string(&args[1])
        .map_err(|error| format!("cannot read expected manifest {}: {error}", args[1]))?;
    let input: Input = serde_json::from_str(&input_raw)
        .map_err(|error| format!("invalid input JSON/contract: {error}"))?;
    let exact = evaluate(&input)?;
    check_expected(&exact, &reference_raw)?;
    let regret = evaluate_robust_regret(&exact, &input.scenarios)?;
    let receipt = build_regret_receipt(
        &input_raw,
        &reference_raw,
        &input.schema_version,
        exact.candidate_subset_count,
        exact.feasible_portfolios.len() as u64,
        regret,
        "PASS: exact feasible set/frontier matched the known-answer manifest",
    );
    println!(
        "{}",
        serde_json::to_string_pretty(&receipt).map_err(|error| error.to_string())?
    );
    Ok(())
}

fn run_search_regret(args: &[String]) -> Result<(), String> {
    if args.len() != 2 {
        return Err(
            "usage: sa-economic-portfolio-oracle --search-regret INPUT.json REFERENCE_FRONTIER.json"
                .to_owned(),
        );
    }
    let input_raw = fs::read_to_string(&args[0])
        .map_err(|error| format!("cannot read input {}: {error}", args[0]))?;
    let reference_raw = fs::read_to_string(&args[1])
        .map_err(|error| format!("cannot read reference frontier {}: {error}", args[1]))?;
    let input: Input = serde_json::from_str(&input_raw)
        .map_err(|error| format!("invalid input JSON/contract: {error}"))?;
    let reference: serde_json::Value = serde_json::from_str(&reference_raw)
        .map_err(|error| format!("invalid reference frontier JSON: {error}"))?;
    let mut expected_frontier: Vec<Portfolio> =
        serde_json::from_value(reference["pareto_frontier"].clone())
            .map_err(|error| format!("reference frontier missing/invalid: {error}"))?;
    expected_frontier.sort_by_key(|portfolio| portfolio.project_ids.join("+"));

    let searched = evaluate_frontier_search(&input)?;
    if searched.pareto_frontier != expected_frontier {
        return Err("search frontier does not match supplied reference manifest".to_owned());
    }
    let regret = evaluate_robust_regret_from_search(&searched, &input.scenarios)?;
    let receipt = build_regret_receipt(
        &input_raw,
        &reference_raw,
        &input.schema_version,
        searched.stats.candidate_subset_count,
        searched.stats.feasible_portfolio_count,
        regret,
        "PASS: complete search frontier matched the supplied reference manifest",
    );
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
